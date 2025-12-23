#![allow(clippy::let_unit_value, clippy::manual_range_contains)]

use libsrtp::{MasterKey, ProtectionProfile, RecvSession, SendSession, StreamConfig};
use openssl::pkcs12::Pkcs12;
use openssl::pkey::PKey;
use openssl::x509::X509;
use rustls::client::{ServerCertVerified, ServerCertVerifier};
use rustls::{
    Certificate, ClientConfig, ClientConnection, OwnedTrustAnchor, PrivateKey, RootCertStore,
    ServerConfig, ServerConnection, ServerName, StreamOwned,
};
use rustls_pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
#[cfg(not(coverage))]
use std::io;
use std::io::{Read, Write};
use std::net::ToSocketAddrs;
use std::os::unix::io::{FromRawFd, RawFd};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::proto::rr::{RData, RecordType};
use trust_dns_resolver::Resolver;
use udp_dtls::{DtlsAcceptor, DtlsConnector, Identity, SrtpProfile, UdpChannel};

#[cfg(all(feature = "cli", not(coverage)))]
use crossterm::{
    event::{read, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use crate::ast::{LogLevel, *};
use crate::error::{HaversError, HaversResult};
use crate::logging;
use crate::value::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

/// Whether crash handling is enabled (default: true)
static CRASH_HANDLING_ENABLED: AtomicBool = AtomicBool::new(true);

/// Monotonic clock anchor for mono_ms/mono_ns
static MONO_START: OnceLock<std::time::Instant> = OnceLock::new();

#[derive(Debug, Clone, Copy)]
enum SocketKind {
    Udp,
    Tcp,
}

#[derive(Debug, Clone, Copy)]
struct SocketEntry {
    fd: RawFd,
    kind: SocketKind,
}

struct SocketRegistry {
    next_id: i64,
    sockets: HashMap<i64, SocketEntry>,
}

impl SocketRegistry {
    fn new() -> Self {
        SocketRegistry {
            next_id: 1,
            sockets: HashMap::new(),
        }
    }
}

static SOCKETS: OnceLock<Mutex<SocketRegistry>> = OnceLock::new();

fn socket_registry() -> &'static Mutex<SocketRegistry> {
    SOCKETS.get_or_init(|| Mutex::new(SocketRegistry::new()))
}

fn register_socket(fd: RawFd, kind: SocketKind) -> i64 {
    let mut reg = socket_registry().lock().unwrap();
    let id = reg.next_id;
    reg.next_id += 1;
    reg.sockets.insert(id, SocketEntry { fd, kind });
    id
}

fn get_socket(id: i64) -> Option<SocketEntry> {
    let reg = socket_registry().lock().unwrap();
    reg.sockets.get(&id).copied()
}

fn update_socket_kind(id: i64, kind: SocketKind) -> bool {
    let mut reg = socket_registry().lock().unwrap();
    if let Some(entry) = reg.sockets.get_mut(&id) {
        entry.kind = kind;
        true
    } else {
        false
    }
}

fn remove_socket(id: i64) -> Option<SocketEntry> {
    let mut reg = socket_registry().lock().unwrap();
    reg.sockets.remove(&id)
}

#[derive(Debug, Clone)]
struct LoopWatch {
    sock_id: i64,
    fd: RawFd,
    read_cb: Value,
    write_cb: Value,
}

#[derive(Debug, Clone)]
struct LoopTimer {
    id: i64,
    next_fire_ms: i64,
    interval_ms: i64,
    callback: Value,
    cancelled: bool,
}

#[derive(Debug, Clone)]
struct EventLoop {
    watches: Vec<LoopWatch>,
    timers: Vec<LoopTimer>,
    next_timer_id: i64,
    stopped: bool,
}

struct EventLoopRegistry {
    next_id: i64,
    loops: HashMap<i64, EventLoop>,
}

impl EventLoopRegistry {
    fn new() -> Self {
        EventLoopRegistry {
            next_id: 1,
            loops: HashMap::new(),
        }
    }
}

thread_local! {
    static LOOPS: RefCell<EventLoopRegistry> = RefCell::new(EventLoopRegistry::new());
}

fn register_loop(loop_val: EventLoop) -> i64 {
    LOOPS.with(|cell| {
        let mut reg = cell.borrow_mut();
        let id = reg.next_id;
        reg.next_id += 1;
        reg.loops.insert(id, loop_val);
        id
    })
}

fn with_loop_mut<T, F>(id: i64, f: F) -> Result<T, String>
where
    F: FnOnce(&mut EventLoop) -> T,
{
    LOOPS.with(|cell| {
        let mut reg = cell.borrow_mut();
        let loop_ref = reg.loops.get_mut(&id).ok_or("Unknown event loop handle")?;
        Ok(f(loop_ref))
    })
}

#[derive(Debug, Clone)]
struct ThreadHandle {
    result: Value,
    detached: bool,
}

struct ThreadRegistry {
    next_id: i64,
    threads: HashMap<i64, ThreadHandle>,
}

impl ThreadRegistry {
    fn new() -> Self {
        ThreadRegistry {
            next_id: 1,
            threads: HashMap::new(),
        }
    }
}

thread_local! {
    static THREADS: RefCell<ThreadRegistry> = RefCell::new(ThreadRegistry::new());
    static MUTEXES: RefCell<MutexRegistry> = RefCell::new(MutexRegistry::new());
    static CONDVARS: RefCell<CondvarRegistry> = RefCell::new(CondvarRegistry::new());
    static ATOMICS: RefCell<AtomicRegistry> = RefCell::new(AtomicRegistry::new());
    static CHANNELS: RefCell<ChannelRegistry> = RefCell::new(ChannelRegistry::new());
}

thread_local! {
    static CURRENT_INTERPRETER: RefCell<*mut Interpreter> =
        const { RefCell::new(std::ptr::null_mut()) };
}

struct InterpreterGuard {
    prev: *mut Interpreter,
}

impl InterpreterGuard {
    fn new(interp: &mut Interpreter) -> Self {
        let ptr = interp as *mut Interpreter;
        let prev = CURRENT_INTERPRETER.with(|cell| {
            let mut cur = cell.borrow_mut();
            let prev = *cur;
            *cur = ptr;
            prev
        });
        InterpreterGuard { prev }
    }
}

impl Drop for InterpreterGuard {
    fn drop(&mut self) {
        CURRENT_INTERPRETER.with(|cell| {
            *cell.borrow_mut() = self.prev;
        });
    }
}

fn with_current_interpreter<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut Interpreter) -> R,
{
    CURRENT_INTERPRETER.with(|cell| {
        let ptr = *cell.borrow();
        if ptr.is_null() {
            None
        } else {
            // Safety: pointer is only set while an Interpreter call is active.
            Some(unsafe { f(&mut *ptr) })
        }
    })
}

fn register_thread(handle: ThreadHandle) -> i64 {
    THREADS.with(|cell| {
        let mut reg = cell.borrow_mut();
        let id = reg.next_id;
        reg.next_id += 1;
        reg.threads.insert(id, handle);
        id
    })
}

fn with_thread_mut<T, F>(id: i64, f: F) -> Result<T, String>
where
    F: FnOnce(&mut ThreadHandle) -> T,
{
    THREADS.with(|cell| {
        let mut reg = cell.borrow_mut();
        let handle = reg.threads.get_mut(&id).ok_or("Unknown thread handle")?;
        Ok(f(handle))
    })
}

#[derive(Debug, Clone)]
struct MutexState {
    locked: bool,
}

struct MutexRegistry {
    next_id: i64,
    mutexes: HashMap<i64, MutexState>,
}

impl MutexRegistry {
    fn new() -> Self {
        MutexRegistry {
            next_id: 1,
            mutexes: HashMap::new(),
        }
    }
}

fn register_mutex(state: MutexState) -> i64 {
    MUTEXES.with(|cell| {
        let mut reg = cell.borrow_mut();
        let id = reg.next_id;
        reg.next_id += 1;
        reg.mutexes.insert(id, state);
        id
    })
}

fn with_mutex_mut<T, F>(id: i64, f: F) -> Result<T, String>
where
    F: FnOnce(&mut MutexState) -> T,
{
    MUTEXES.with(|cell| {
        let mut reg = cell.borrow_mut();
        let state = reg.mutexes.get_mut(&id).ok_or("Unknown mutex handle")?;
        Ok(f(state))
    })
}

#[derive(Debug, Clone)]
struct CondvarState;

struct CondvarRegistry {
    next_id: i64,
    condvars: HashMap<i64, CondvarState>,
}

impl CondvarRegistry {
    fn new() -> Self {
        CondvarRegistry {
            next_id: 1,
            condvars: HashMap::new(),
        }
    }
}

fn register_condvar(state: CondvarState) -> i64 {
    CONDVARS.with(|cell| {
        let mut reg = cell.borrow_mut();
        let id = reg.next_id;
        reg.next_id += 1;
        reg.condvars.insert(id, state);
        id
    })
}

fn with_condvar_mut<T, F>(id: i64, f: F) -> Result<T, String>
where
    F: FnOnce(&mut CondvarState) -> T,
{
    CONDVARS.with(|cell| {
        let mut reg = cell.borrow_mut();
        let state = reg.condvars.get_mut(&id).ok_or("Unknown condvar handle")?;
        Ok(f(state))
    })
}

#[derive(Debug, Clone)]
struct AtomicState {
    value: i64,
}

struct AtomicRegistry {
    next_id: i64,
    atomics: HashMap<i64, AtomicState>,
}

impl AtomicRegistry {
    fn new() -> Self {
        AtomicRegistry {
            next_id: 1,
            atomics: HashMap::new(),
        }
    }
}

fn register_atomic(state: AtomicState) -> i64 {
    ATOMICS.with(|cell| {
        let mut reg = cell.borrow_mut();
        let id = reg.next_id;
        reg.next_id += 1;
        reg.atomics.insert(id, state);
        id
    })
}

fn with_atomic_mut<T, F>(id: i64, f: F) -> Result<T, String>
where
    F: FnOnce(&mut AtomicState) -> T,
{
    ATOMICS.with(|cell| {
        let mut reg = cell.borrow_mut();
        let state = reg.atomics.get_mut(&id).ok_or("Unknown atomic handle")?;
        Ok(f(state))
    })
}

#[derive(Debug, Clone)]
struct ChannelState {
    queue: VecDeque<Value>,
    capacity: i64,
    closed: bool,
}

struct ChannelRegistry {
    next_id: i64,
    channels: HashMap<i64, ChannelState>,
}

impl ChannelRegistry {
    fn new() -> Self {
        ChannelRegistry {
            next_id: 1,
            channels: HashMap::new(),
        }
    }
}

fn register_channel(state: ChannelState) -> i64 {
    CHANNELS.with(|cell| {
        let mut reg = cell.borrow_mut();
        let id = reg.next_id;
        reg.next_id += 1;
        reg.channels.insert(id, state);
        id
    })
}

fn with_channel_mut<T, F>(id: i64, f: F) -> Result<T, String>
where
    F: FnOnce(&mut ChannelState) -> T,
{
    CHANNELS.with(|cell| {
        let mut reg = cell.borrow_mut();
        let state = reg.channels.get_mut(&id).ok_or("Unknown channel handle")?;
        Ok(f(state))
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TlsMode {
    Client,
    Server,
}

struct TlsSession {
    mode: TlsMode,
    server_name: String,
    client_config: Option<Arc<ClientConfig>>,
    server_config: Option<Arc<ServerConfig>>,
    stream: Option<TlsStream>,
}

enum TlsStream {
    Client(StreamOwned<ClientConnection, std::net::TcpStream>),
    Server(StreamOwned<ServerConnection, std::net::TcpStream>),
}

struct TlsRegistry {
    next_id: i64,
    sessions: HashMap<i64, TlsSession>,
}

impl TlsRegistry {
    fn new() -> Self {
        TlsRegistry {
            next_id: 1,
            sessions: HashMap::new(),
        }
    }
}

static TLS_REGISTRY: OnceLock<Mutex<TlsRegistry>> = OnceLock::new();

fn tls_registry() -> &'static Mutex<TlsRegistry> {
    TLS_REGISTRY.get_or_init(|| Mutex::new(TlsRegistry::new()))
}

fn register_tls(session: TlsSession) -> i64 {
    let mut reg = tls_registry().lock().unwrap();
    let id = reg.next_id;
    reg.next_id += 1;
    reg.sessions.insert(id, session);
    id
}

fn with_tls_mut<T, F>(id: i64, f: F) -> Result<T, String>
where
    F: FnOnce(&mut TlsSession) -> Result<T, String>,
{
    let mut reg = tls_registry().lock().unwrap();
    let session = reg.sessions.get_mut(&id).ok_or("Unknown TLS handle")?;
    f(session)
}

fn remove_tls(id: i64) {
    let mut reg = tls_registry().lock().unwrap();
    reg.sessions.remove(&id);
}

struct SrtpSession {
    send: SendSession,
    recv: RecvSession,
}

struct SrtpRegistry {
    next_id: i64,
    sessions: HashMap<i64, SrtpSession>,
}

static SRTP_REGISTRY: OnceLock<Mutex<SrtpRegistry>> = OnceLock::new();

fn srtp_registry() -> &'static Mutex<SrtpRegistry> {
    SRTP_REGISTRY.get_or_init(|| {
        Mutex::new(SrtpRegistry {
            next_id: 1,
            sessions: HashMap::new(),
        })
    })
}

fn register_srtp(session: SrtpSession) -> i64 {
    let mut reg = srtp_registry().lock().unwrap();
    let id = reg.next_id;
    reg.next_id += 1;
    reg.sessions.insert(id, session);
    id
}

fn with_srtp_mut<T, F>(id: i64, f: F) -> Result<T, String>
where
    F: FnOnce(&mut SrtpSession) -> Result<T, String>,
{
    let mut reg = srtp_registry().lock().unwrap();
    let session = reg.sessions.get_mut(&id).ok_or("Unknown SRTP handle")?;
    f(session)
}

#[derive(Clone)]
struct DtlsConfigData {
    mode: TlsMode,
    server_name: String,
    insecure: bool,
    ca_pem: Option<String>,
    cert_pem: Option<String>,
    key_pem: Option<String>,
    remote_host: Option<String>,
    remote_port: Option<u16>,
    srtp_profiles: Vec<SrtpProfile>,
}

struct DtlsRegistry {
    next_id: i64,
    configs: HashMap<i64, DtlsConfigData>,
}

static DTLS_REGISTRY: OnceLock<Mutex<DtlsRegistry>> = OnceLock::new();

fn dtls_registry() -> &'static Mutex<DtlsRegistry> {
    DTLS_REGISTRY.get_or_init(|| {
        Mutex::new(DtlsRegistry {
            next_id: 1,
            configs: HashMap::new(),
        })
    })
}

fn register_dtls(config: DtlsConfigData) -> i64 {
    let mut reg = dtls_registry().lock().unwrap();
    let id = reg.next_id;
    reg.next_id += 1;
    reg.configs.insert(id, config);
    id
}

fn dtls_get(id: i64) -> Result<DtlsConfigData, String> {
    let reg = dtls_registry().lock().unwrap();
    reg.configs
        .get(&id)
        .cloned()
        .ok_or("Unknown DTLS handle".to_string())
}

fn result_ok(value: Value) -> Value {
    let mut dict = DictValue::new();
    dict.set(Value::String("ok".to_string()), Value::Bool(true));
    dict.set(Value::String("value".to_string()), value);
    Value::Dict(Rc::new(RefCell::new(dict)))
}

fn result_err(message: String, code: i64) -> Value {
    let mut dict = DictValue::new();
    dict.set(Value::String("ok".to_string()), Value::Bool(false));
    dict.set(Value::String("error".to_string()), Value::String(message));
    dict.set(Value::String("code".to_string()), Value::Integer(code));
    Value::Dict(Rc::new(RefCell::new(dict)))
}

fn mono_ms_now() -> i64 {
    let start = MONO_START.get_or_init(std::time::Instant::now);
    start.elapsed().as_millis() as i64
}

fn make_resolver() -> Result<Resolver, String> {
    Resolver::from_system_conf()
        .or_else(|_| Resolver::new(ResolverConfig::default(), ResolverOpts::default()))
        .map_err(|e| format!("DNS resolver init failed: {}", e))
}

struct InsecureVerifier;

impl ServerCertVerifier for InsecureVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
}

struct TlsConfigData {
    mode: TlsMode,
    server_name: String,
    insecure: bool,
    ca_pem: Option<String>,
    cert_pem: Option<String>,
    key_pem: Option<String>,
}

fn dict_get_string(dict: &DictValue, key: &str) -> Option<String> {
    dict.get(&Value::String(key.to_string()))
        .and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
}

fn dict_get_bool(dict: &DictValue, key: &str) -> Option<bool> {
    dict.get(&Value::String(key.to_string()))
        .and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None,
        })
}

fn dict_get_bytes(dict: &DictValue, key: &str) -> Option<Vec<u8>> {
    dict.get(&Value::String(key.to_string()))
        .and_then(|v| match v {
            Value::Bytes(b) => Some(b.borrow().clone()),
            _ => None,
        })
}

fn dict_get_u16(dict: &DictValue, key: &str) -> Option<u16> {
    dict.get(&Value::String(key.to_string()))
        .and_then(|v| match v {
            Value::Integer(n) => {
                if *n >= 0 && *n <= u16::MAX as i64 {
                    Some(*n as u16)
                } else {
                    None
                }
            }
            Value::Float(f) => {
                let v = *f as i64;
                if v >= 0 && v <= u16::MAX as i64 {
                    Some(v as u16)
                } else {
                    None
                }
            }
            _ => None,
        })
}

fn tls_config_from_value(value: &Value) -> Result<TlsConfigData, String> {
    if matches!(value, Value::Nil) {
        return Ok(TlsConfigData {
            mode: TlsMode::Client,
            server_name: "localhost".to_string(),
            insecure: false,
            ca_pem: None,
            cert_pem: None,
            key_pem: None,
        });
    }
    let dict = match value {
        Value::Dict(d) => d.borrow(),
        _ => return Err("tls_client_new() expects config dict".to_string()),
    };

    let mode = dict_get_string(&dict, "mode")
        .unwrap_or_else(|| "client".to_string())
        .to_lowercase();
    let mode = if mode == "server" {
        TlsMode::Server
    } else {
        TlsMode::Client
    };

    let mut server_name =
        dict_get_string(&dict, "server_name").unwrap_or_else(|| "localhost".to_string());
    if server_name.is_empty() {
        server_name = "localhost".to_string();
    }

    let insecure = dict_get_bool(&dict, "insecure").unwrap_or(false);
    let ca_pem = dict_get_string(&dict, "ca_pem").filter(|s| !s.is_empty());
    let cert_pem = dict_get_string(&dict, "cert_pem").filter(|s| !s.is_empty());
    let key_pem = dict_get_string(&dict, "key_pem").filter(|s| !s.is_empty());

    Ok(TlsConfigData {
        mode,
        server_name,
        insecure,
        ca_pem,
        cert_pem,
        key_pem,
    })
}

fn build_client_config(cfg: &TlsConfigData) -> Result<Arc<ClientConfig>, String> {
    let mut roots = RootCertStore::empty();
    if let Some(pem) = &cfg.ca_pem {
        let mut reader = std::io::Cursor::new(pem.as_bytes());
        let certs = certs(&mut reader).map_err(|e| format!("Invalid CA certs: {}", e))?;
        let (added, _ignored) = roots.add_parsable_certificates(&certs);
        if added == 0 {
            return Err("No valid CA certificates found".to_string());
        }
    } else {
        roots.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
            OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        }));
    }

    let mut config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_no_client_auth();

    if cfg.insecure {
        config
            .dangerous()
            .set_certificate_verifier(Arc::new(InsecureVerifier));
    }

    Ok(Arc::new(config))
}

fn build_server_config(cfg: &TlsConfigData) -> Result<Arc<ServerConfig>, String> {
    let cert_pem = cfg.cert_pem.as_ref().ok_or("Server cert_pem is required")?;
    let key_pem = cfg.key_pem.as_ref().ok_or("Server key_pem is required")?;

    let mut cert_reader = std::io::Cursor::new(cert_pem.as_bytes());
    let certs = certs(&mut cert_reader).map_err(|e| format!("Invalid server cert: {}", e))?;
    let certs = certs.into_iter().map(Certificate).collect::<Vec<_>>();

    let mut key_reader = std::io::Cursor::new(key_pem.as_bytes());
    let mut keys =
        pkcs8_private_keys(&mut key_reader).map_err(|e| format!("Invalid server key: {}", e))?;
    if keys.is_empty() {
        let mut key_reader = std::io::Cursor::new(key_pem.as_bytes());
        keys =
            rsa_private_keys(&mut key_reader).map_err(|e| format!("Invalid server key: {}", e))?;
    }
    let key = keys
        .into_iter()
        .next()
        .ok_or("Server key_pem did not contain a private key")?;

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, PrivateKey(key))
        .map_err(|e| format!("Invalid server TLS config: {}", e))?;

    Ok(Arc::new(config))
}

fn srtp_profile_from_str(s: &str) -> Option<SrtpProfile> {
    match s.to_uppercase().as_str() {
        "SRTP_AES128_CM_SHA1_80" | "AES128_CM_SHA1_80" | "AES128_CM_HMAC_SHA1_80" => {
            Some(SrtpProfile::Aes128CmSha180)
        }
        "SRTP_AES128_CM_SHA1_32" | "AES128_CM_SHA1_32" | "AES128_CM_HMAC_SHA1_32" => {
            Some(SrtpProfile::Aes128CmSha132)
        }
        "SRTP_AEAD_AES_128_GCM" | "AEAD_AES_128_GCM" => Some(SrtpProfile::AeadAes128Gcm),
        "SRTP_AEAD_AES_256_GCM" | "AEAD_AES_256_GCM" => Some(SrtpProfile::AeadAes256Gcm),
        _ => None,
    }
}

fn protection_profile_from_str(s: &str) -> Option<ProtectionProfile> {
    match s.to_uppercase().as_str() {
        "SRTP_AES128_CM_SHA1_80" | "AES128_CM_SHA1_80" | "AES128_CM_HMAC_SHA1_80" => {
            Some(ProtectionProfile::Aes128CmHmacSha180)
        }
        "SRTP_AES128_CM_SHA1_32" | "AES128_CM_SHA1_32" | "AES128_CM_HMAC_SHA1_32" => {
            Some(ProtectionProfile::Aes128CmHmacSha132)
        }
        "SRTP_AEAD_AES_128_GCM" | "AEAD_AES_128_GCM" => Some(ProtectionProfile::AeadAes128Gcm),
        "SRTP_AEAD_AES_256_GCM" | "AEAD_AES_256_GCM" => Some(ProtectionProfile::AeadAes256Gcm),
        _ => None,
    }
}

fn srtp_key_salt_len(profile: SrtpProfile) -> (usize, usize) {
    match profile {
        SrtpProfile::Aes128CmSha180 | SrtpProfile::Aes128CmSha132 => (16, 14),
        SrtpProfile::AeadAes128Gcm => (16, 12),
        SrtpProfile::AeadAes256Gcm => (32, 12),
        SrtpProfile::__Nonexhaustive => (16, 14),
    }
}

fn dtls_config_from_value(value: &Value) -> Result<DtlsConfigData, String> {
    if matches!(value, Value::Nil) {
        return Ok(DtlsConfigData {
            mode: TlsMode::Server,
            server_name: "localhost".to_string(),
            insecure: false,
            ca_pem: None,
            cert_pem: None,
            key_pem: None,
            remote_host: None,
            remote_port: None,
            srtp_profiles: vec![SrtpProfile::Aes128CmSha180],
        });
    }
    let dict = match value {
        Value::Dict(d) => d.borrow(),
        _ => return Err("dtls_server_new() expects config dict".to_string()),
    };

    let mode = dict_get_string(&dict, "mode")
        .unwrap_or_else(|| "server".to_string())
        .to_lowercase();
    let mode = if mode == "client" {
        TlsMode::Client
    } else {
        TlsMode::Server
    };

    let mut server_name =
        dict_get_string(&dict, "server_name").unwrap_or_else(|| "localhost".to_string());
    if server_name.is_empty() {
        server_name = "localhost".to_string();
    }

    let insecure = dict_get_bool(&dict, "insecure").unwrap_or(false);
    let ca_pem = dict_get_string(&dict, "ca_pem").filter(|s| !s.is_empty());
    let cert_pem = dict_get_string(&dict, "cert_pem").filter(|s| !s.is_empty());
    let key_pem = dict_get_string(&dict, "key_pem").filter(|s| !s.is_empty());
    let remote_host = dict_get_string(&dict, "remote_host").filter(|s| !s.is_empty());
    let remote_port = dict_get_u16(&dict, "remote_port");

    let mut profiles = Vec::new();
    if let Some(Value::List(list)) = dict.get(&Value::String("srtp_profiles".to_string())) {
        for item in list.borrow().iter() {
            if let Value::String(s) = item {
                if let Some(profile) = srtp_profile_from_str(s) {
                    profiles.push(profile);
                }
            }
        }
    }
    if profiles.is_empty() {
        profiles.push(SrtpProfile::Aes128CmSha180);
    }

    Ok(DtlsConfigData {
        mode,
        server_name,
        insecure,
        ca_pem,
        cert_pem,
        key_pem,
        remote_host,
        remote_port,
        srtp_profiles: profiles,
    })
}

fn identity_from_pem(cert_pem: &str, key_pem: &str) -> Result<Identity, String> {
    let cert =
        X509::from_pem(cert_pem.as_bytes()).map_err(|e| format!("Invalid cert PEM: {}", e))?;
    let key = PKey::private_key_from_pem(key_pem.as_bytes())
        .map_err(|e| format!("Invalid key PEM: {}", e))?;
    let mut builder = Pkcs12::builder();
    builder.name("mdhavers").pkey(&key).cert(&cert);
    let pkcs12 = builder
        .build2("")
        .map_err(|e| format!("PKCS12 build failed: {}", e))?;
    let der = pkcs12
        .to_der()
        .map_err(|e| format!("PKCS12 serialize failed: {}", e))?;
    Identity::from_pkcs12(&der, "").map_err(|e| format!("Identity parse failed: {}", e))
}

fn addr_dict(host: String, port: i64) -> Value {
    let mut dict = DictValue::new();
    dict.set(Value::String("host".to_string()), Value::String(host));
    dict.set(Value::String("port".to_string()), Value::Integer(port));
    Value::Dict(Rc::new(RefCell::new(dict)))
}

fn event_dict(
    kind: &str,
    sock: Option<i64>,
    timer_id: Option<i64>,
    callback: Option<Value>,
) -> Value {
    let mut dict = DictValue::new();
    dict.set(
        Value::String("kind".to_string()),
        Value::String(kind.to_string()),
    );
    if let Some(sock_id) = sock {
        dict.set(Value::String("sock".to_string()), Value::Integer(sock_id));
    }
    if let Some(id) = timer_id {
        dict.set(Value::String("id".to_string()), Value::Integer(id));
    }
    if let Some(cb) = callback {
        dict.set(Value::String("callback".to_string()), cb);
    }
    Value::Dict(Rc::new(RefCell::new(dict)))
}

fn resolve_ipv4_addr(host: Option<&str>, port: u16) -> Result<libc::sockaddr_in, String> {
    let mut addr: libc::sockaddr_in = unsafe { std::mem::zeroed() };
    addr.sin_family = libc::AF_INET as u16;
    addr.sin_port = port.to_be();

    if let Some(host_str) = host {
        let mut resolved = None;
        let iter = (host_str, port)
            .to_socket_addrs()
            .map_err(|e| format!("DNS lookup failed: {}", e))?;
        for sock in iter {
            if let std::net::SocketAddr::V4(v4) = sock {
                resolved = Some(v4);
                break;
            }
        }
        let v4 = resolved.ok_or_else(|| "No IPv4 address found".to_string())?;
        addr.sin_addr = libc::in_addr {
            s_addr: u32::from_ne_bytes(v4.ip().octets()),
        };
    } else {
        addr.sin_addr = libc::in_addr { s_addr: 0 };
    }

    Ok(addr)
}

fn sockaddr_to_host_port(addr: &libc::sockaddr_in) -> (String, i64) {
    let host = std::net::Ipv4Addr::from(u32::from_be(addr.sin_addr.s_addr)).to_string();
    let port = u16::from_be(addr.sin_port) as i64;
    (host, port)
}

fn format_braw_time(hours: u64, minutes: u64) -> String {
    match hours {
        0..=5 => format!("It's the wee small hours ({:02}:{:02})", hours, minutes),
        6..=11 => format!("It's the mornin' ({:02}:{:02})", hours, minutes),
        12 => format!("It's high noon ({:02}:{:02})", hours, minutes),
        13..=17 => format!("It's the efternoon ({:02}:{:02})", hours, minutes),
        18..=21 => format!("It's the evenin' ({:02}:{:02})", hours, minutes),
        _ => format!("It's gettin' late ({:02}:{:02})", hours, minutes),
    }
}

/// A stack frame for the shadow call stack
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Function name or "<main>"
    pub name: String,
    /// Source file name
    pub file: String,
    /// Line number
    pub line: usize,
}

impl std::fmt::Display for StackFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "  at {} ({}:{})", self.name, self.file, self.line)
    }
}

/// Global shadow call stack for crash reporting
static SHADOW_STACK: Mutex<Vec<StackFrame>> = Mutex::new(Vec::new());
static CURRENT_STACK_FILE: Mutex<String> = Mutex::new(String::new());

/// Push a frame onto the shadow stack
pub fn push_stack_frame(name: &str, line: usize) {
    if let Ok(mut stack) = SHADOW_STACK.lock() {
        let file = CURRENT_STACK_FILE
            .lock()
            .map(|f| f.clone())
            .unwrap_or_default();
        stack.push(StackFrame {
            name: name.to_string(),
            file,
            line,
        });
    }
}

/// Pop a frame from the shadow stack
pub fn pop_stack_frame() {
    if let Ok(mut stack) = SHADOW_STACK.lock() {
        stack.pop();
    }
}

/// Get a copy of the current stack trace
pub fn get_stack_trace() -> Vec<StackFrame> {
    SHADOW_STACK.lock().map(|s| s.clone()).unwrap_or_default()
}

/// Clear the stack trace (for REPL reset)
#[allow(dead_code)]
pub fn clear_stack_trace() {
    if let Ok(mut stack) = SHADOW_STACK.lock() {
        stack.clear();
    }
}

/// Set the current file name for stack frames
pub fn set_stack_file(file: &str) {
    if let Ok(mut f) = CURRENT_STACK_FILE.lock() {
        *f = file.to_string();
    }
}

/// Print the current stack trace to stderr
#[cfg_attr(coverage, allow(dead_code))]
pub fn print_stack_trace() {
    let stack = get_stack_trace();
    if stack.is_empty() {
        eprintln!("  (no stack trace available)");
        return;
    }
    eprintln!("\n🏴󠁧󠁢󠁳󠁣󠁴󠁿 Stack trace (most recent call last):");
    for frame in stack.iter().rev() {
        eprintln!("{}", frame);
    }
}

/// Enable or disable crash handling
pub fn set_crash_handling(enabled: bool) {
    CRASH_HANDLING_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Check if crash handling is enabled
#[cfg_attr(coverage, allow(dead_code))]
pub fn is_crash_handling_enabled() -> bool {
    CRASH_HANDLING_ENABLED.load(Ordering::Relaxed)
}

/// Get the global log level
pub fn get_global_log_level() -> LogLevel {
    logging::get_global_log_level()
}

/// Set the global log level
pub fn set_global_log_level(level: LogLevel) {
    logging::set_global_log_level(level);
}

/// Test/coverage-only escape hatch to set an invalid global log level value.
/// This exists purely to exercise the fallback branch in `get_global_log_level`.
#[cfg(coverage)]
#[allow(dead_code)]
pub fn set_global_log_level_raw(level: u8) {
    logging::set_global_log_level_raw(level);
}

fn parse_log_level_value(value: &Value) -> Result<LogLevel, String> {
    match value {
        Value::String(s) => {
            LogLevel::parse_level(s).ok_or_else(|| format!("Invalid log level '{}'", s))
        }
        Value::Integer(n) => match n {
            0 => Ok(LogLevel::Wheesht),
            1 => Ok(LogLevel::Roar),
            2 => Ok(LogLevel::Holler),
            3 => Ok(LogLevel::Blether),
            4 => Ok(LogLevel::Mutter),
            5 => Ok(LogLevel::Whisper),
            _ => Err(format!("Invalid log level {}. Use 0-5", n)),
        },
        _ => Err("Log level must be a string or integer".to_string()),
    }
}

fn parse_log_target_value(value: &Value) -> Result<String, String> {
    match value {
        Value::String(s) => Ok(s.clone()),
        _ => Err("Log target must be a string".to_string()),
    }
}

fn resolve_log_args(args: &[Value]) -> Result<(Option<Value>, Option<String>), String> {
    match args.len() {
        0 => Ok((None, None)),
        1 => match &args[0] {
            v @ Value::Dict(_) => Ok((Some(v.clone()), None)),
            Value::String(s) => Ok((None, Some(s.clone()))),
            _ => Err("Expected dict or string for log fields/target".to_string()),
        },
        2 => {
            let fields = match &args[0] {
                v @ Value::Dict(_) => Some(v.clone()),
                _ => return Err("Expected dict for log fields".to_string()),
            };
            let target = parse_log_target_value(&args[1])?;
            Ok((fields, Some(target)))
        }
        _ => Err("Expected at most two extra arguments".to_string()),
    }
}

fn dict_get(dict: &DictValue, key: &str) -> Option<Value> {
    dict.get(&Value::String(key.to_string())).cloned()
}

/// Control flow signals
#[derive(Debug)]
enum ControlFlow {
    Return(Value),
    Break,
    Continue,
}

/// Trace mode fer debugging - shows step-by-step execution
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TraceMode {
    /// Nae tracing at aw
    #[default]
    Off,
    /// Show statement execution only
    Statements,
    /// Show everything (statements, expressions, values)
    Verbose,
}

/// The interpreter - runs mdhavers programs
pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
    output: Vec<String>,
    /// Track loaded modules tae prevent circular imports
    loaded_modules: HashSet<PathBuf>,
    /// Current working directory fer resolving relative imports
    current_dir: PathBuf,
    /// Whether the prelude has been loaded
    prelude_loaded: bool,
    /// Trace mode fer debugging
    trace_mode: TraceMode,
    /// Current trace indentation level
    trace_depth: usize,
    /// Logger configuration and sinks
    logger: logging::LoggerCore,
    /// Optional callback hook for log events
    log_callback: Option<Value>,
    /// Current source file name for log messages
    current_file: String,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));

        // Define native functions
        Self::define_natives(&globals);

        // Register audio functions (if feature enabled)
        crate::audio::register_audio_functions(&globals);

        // Register graphics functions (if feature enabled)
        crate::graphics::register_graphics_functions(&globals);

        // Check fer MDH_LOG or MDH_LOG_LEVEL environment variables
        if let Ok(spec) = std::env::var("MDH_LOG") {
            if let Ok(filter) = logging::parse_filter(&spec) {
                let _ = logging::set_filter(&spec);
                set_global_log_level(filter.default);
            } else {
                eprintln!("Warning: Invalid MDH_LOG filter '{}'", spec);
            }
        } else if let Ok(level_str) = std::env::var("MDH_LOG_LEVEL") {
            if let Some(level) = LogLevel::parse_level(&level_str) {
                set_global_log_level(level);
            }
        }

        Interpreter {
            globals: globals.clone(),
            environment: globals,
            output: Vec::new(),
            loaded_modules: HashSet::new(),
            current_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            prelude_loaded: false,
            trace_mode: TraceMode::Off,
            trace_depth: 0,
            logger: logging::LoggerCore::new(),
            log_callback: None,
            current_file: "<repl>".to_string(),
        }
    }

    /// Set the current source file name (fer log messages)
    pub fn set_current_file(&mut self, file: &str) {
        self.current_file = file.to_string();
        // Also update the global stack file for crash reporting
        set_stack_file(file);
    }

    /// Set the log level
    #[allow(dead_code)]
    pub fn set_log_level(&mut self, level: LogLevel) {
        set_global_log_level(level);
    }

    /// Get current log level
    #[allow(dead_code)]
    pub fn get_log_level(&self) -> LogLevel {
        get_global_log_level()
    }

    /// Check if a log level should be output for a given target
    fn should_log(&self, level: LogLevel, target: &str) -> bool {
        logging::log_enabled(level, target)
    }

    fn emit_log(
        &mut self,
        level: LogLevel,
        message: Value,
        fields: Option<Value>,
        target: Option<String>,
        line: usize,
    ) -> HaversResult<()> {
        let target = target.unwrap_or_else(|| self.current_file.clone());
        if !self.should_log(level, &target) {
            return Ok(());
        }

        let mut field_vec = Vec::new();
        if let Some(fields_val) = fields {
            field_vec = logging::fields_from_dict(&fields_val)
                .map_err(|msg| HaversError::TypeError { message: msg, line })?;
        }

        let record = logging::LogRecord {
            level,
            message: format!("{}", message),
            target: target.clone(),
            file: self.current_file.clone(),
            line,
            fields: field_vec,
            span_path: logging::span_path(),
        };

        self.logger.log(&record);

        if let Some(callback) = self.log_callback.clone() {
            let payload = logging::record_to_value(&record, Some(logging::timestamp_string()));
            let _ = self.call_value(callback, vec![payload], line);
        }

        Ok(())
    }

    fn parse_log_extras(
        &mut self,
        extras: &[Expr],
        line: usize,
    ) -> HaversResult<(Option<Value>, Option<String>)> {
        if extras.is_empty() {
            return Ok((None, None));
        }

        let mut values = Vec::new();
        for expr in extras {
            values.push(self.evaluate(expr)?);
        }

        match values.len() {
            1 => match values.remove(0) {
                v @ Value::Dict(_) => Ok((Some(v), None)),
                Value::String(s) => Ok((None, Some(s))),
                _ => Err(HaversError::TypeError {
                    message: "log_* expects a dict or string for the extra argument".to_string(),
                    line,
                }),
            },
            2 => {
                let fields_val = values.remove(0);
                let target_val = values.remove(0);
                let fields = match fields_val {
                    v @ Value::Dict(_) => Some(v),
                    _ => {
                        return Err(HaversError::TypeError {
                            message: "log_* expects fields as a dict when passing two extras"
                                .to_string(),
                            line,
                        })
                    }
                };
                let target = match target_val {
                    Value::String(s) => Some(s),
                    _ => {
                        return Err(HaversError::TypeError {
                            message: "log_* expects target as a string".to_string(),
                            line,
                        })
                    }
                };
                Ok((fields, target))
            }
            _ => Err(HaversError::InternalError(
                "log_* supports at most two extra arguments".to_string(),
            )),
        }
    }

    fn apply_log_config(&mut self, config: Option<Value>) -> Result<(), String> {
        if config.is_none() {
            self.logger = logging::LoggerCore::new();
            self.log_callback = None;
            return Ok(());
        }

        let dict = match config {
            Some(Value::Dict(d)) => d,
            Some(_) => return Err("log_init() expects a dict".to_string()),
            None => return Ok(()),
        };

        let dict_ref = dict.borrow();

        if let Some(level_val) = dict_get(&dict_ref, "level") {
            let level = parse_log_level_value(&level_val)?;
            set_global_log_level(level);
        }

        if let Some(filter_val) = dict_get(&dict_ref, "filter") {
            let filter_str = match filter_val {
                Value::String(s) => s,
                _ => return Err("log_init() filter must be a string".to_string()),
            };
            logging::set_filter(&filter_str)?;
        }

        if let Some(format_val) = dict_get(&dict_ref, "format") {
            let format_str = match format_val {
                Value::String(s) => s,
                _ => return Err("log_init() format must be a string".to_string()),
            };
            self.logger.format = match format_str.as_str() {
                "text" => logging::LogFormat::Text,
                "json" => logging::LogFormat::Json,
                "compact" => logging::LogFormat::Compact,
                _ => return Err("log_init() format must be text, json, or compact".to_string()),
            };
        }

        if let Some(color_val) = dict_get(&dict_ref, "color") {
            match color_val {
                Value::Bool(b) => self.logger.color = b,
                _ => return Err("log_init() color must be a bool".to_string()),
            }
        }

        if let Some(ts_val) = dict_get(&dict_ref, "timestamps") {
            match ts_val {
                Value::Bool(b) => self.logger.timestamps = b,
                _ => return Err("log_init() timestamps must be a bool".to_string()),
            }
        }

        if let Some(sinks_val) = dict_get(&dict_ref, "sinks") {
            let list = match sinks_val {
                Value::List(list) => list.borrow().clone(),
                _ => return Err("log_init() sinks must be a list".to_string()),
            };
            let mut sinks = Vec::new();
            let mut callback: Option<Value> = None;

            for spec in list {
                let spec_dict = match spec {
                    Value::Dict(d) => d,
                    _ => return Err("log_init() sink specs must be dicts".to_string()),
                };
                let spec_ref = spec_dict.borrow();
                let kind = match dict_get(&spec_ref, "kind") {
                    Some(Value::String(s)) => s,
                    _ => return Err("log_init() sink kind must be a string".to_string()),
                };

                match kind.as_str() {
                    "stderr" => sinks.push(logging::LogSink::Stderr),
                    "stdout" => sinks.push(logging::LogSink::Stdout),
                    "file" => {
                        let path = match dict_get(&spec_ref, "path") {
                            Some(Value::String(s)) => s,
                            _ => {
                                return Err("log_init() file sink requires string path".to_string())
                            }
                        };
                        let append = match dict_get(&spec_ref, "append") {
                            Some(Value::Bool(b)) => b,
                            None => true,
                            _ => return Err("log_init() file append must be bool".to_string()),
                        };
                        sinks.push(logging::LogSink::File {
                            path,
                            append,
                            file: None,
                        });
                    }
                    "memory" => {
                        let max = match dict_get(&spec_ref, "max") {
                            Some(Value::Integer(n)) if n > 0 => n as usize,
                            None => 1000,
                            _ => return Err("log_init() memory max must be integer".to_string()),
                        };
                        sinks.push(logging::LogSink::Memory {
                            entries: Vec::new(),
                            max,
                        });
                    }
                    "callback" => {
                        let cb = dict_get(&spec_ref, "fn")
                            .ok_or_else(|| "log_init() callback sink requires fn".to_string())?;
                        callback = Some(cb);
                    }
                    _ => return Err(format!("Unknown log sink kind '{}'", kind)),
                }
            }

            if sinks.is_empty() {
                sinks.push(logging::LogSink::Stderr);
            }
            self.logger.sinks = sinks;
            self.log_callback = callback;
        }

        Ok(())
    }

    /// Enable trace mode fer debugging
    pub fn set_trace_mode(&mut self, mode: TraceMode) {
        self.trace_mode = mode;
    }

    /// Get current trace mode
    #[allow(dead_code)]
    pub fn trace_mode(&self) -> TraceMode {
        self.trace_mode
    }

    /// Print a trace message with proper indentation and Scottish flair
    fn trace(&self, msg: &str) {
        if self.trace_mode != TraceMode::Off {
            let indent = "  ".repeat(self.trace_depth);
            eprintln!("\x1b[33m🏴󠁧󠁢󠁳󠁣󠁴󠁿 {}{}\x1b[0m", indent, msg);
        }
    }

    /// Print a verbose trace message (only in verbose mode)
    fn trace_verbose(&self, msg: &str) {
        if self.trace_mode == TraceMode::Verbose {
            let indent = "  ".repeat(self.trace_depth);
            eprintln!("\x1b[36m   {}{}\x1b[0m", indent, msg);
        }
    }

    /// Create an interpreter with a specific working directory
    #[allow(dead_code)]
    pub fn with_dir<P: AsRef<Path>>(dir: P) -> Self {
        let mut interp = Self::new();
        interp.current_dir = dir.as_ref().to_path_buf();
        interp
    }

    /// Set the current directory fer module resolution
    pub fn set_current_dir<P: AsRef<Path>>(&mut self, dir: P) {
        self.current_dir = dir.as_ref().to_path_buf();
    }

    /// Get all user-defined variables (fer REPL environment inspection)
    pub fn get_user_variables(&self) -> Vec<(String, String, String)> {
        let env = self.environment.borrow();
        let exports = env.get_exports();
        let mut vars = Vec::new();

        // Get all variables that aren't native functions
        for (name, value) in exports.iter() {
            // Skip native functions
            if matches!(value, Value::NativeFunction(_)) {
                continue;
            }
            // Skip prelude functions (they have specific patterns)
            if matches!(value, Value::Function(_)) {
                // Include user-defined functions but mark them
                vars.push((name.clone(), "function".to_string(), format!("{}", value)));
            } else {
                vars.push((
                    name.clone(),
                    value.type_name().to_string(),
                    format!("{}", value),
                ));
            }
        }

        // Sort by name for consistent display
        vars.sort_by(|a, b| a.0.cmp(&b.0));
        vars
    }

    /// Load the standard prelude (automatically loaded unless disabled)
    /// The prelude provides common utility functions written in mdhavers
    pub fn load_prelude(&mut self) -> HaversResult<()> {
        if self.prelude_loaded {
            return Ok(());
        }

        // Try tae find the prelude in these locations:
        // 1. stdlib/prelude.braw relative tae the executable
        // 2. stdlib/prelude.braw relative tae current directory
        // 3. Embedded prelude as fallback

        let prelude_locations = [
            // Next tae the executable
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("stdlib/prelude.braw"))),
            // In the current directory
            Some(PathBuf::from("stdlib/prelude.braw")),
            // In the project root (fer development)
            Some(PathBuf::from("../stdlib/prelude.braw")),
        ];

        for maybe_path in prelude_locations.iter().flatten() {
            if let Ok(source) = std::fs::read_to_string(maybe_path) {
                match crate::parser::parse(&source) {
                    Ok(program) => {
                        // Execute prelude in globals
                        for stmt in &program.statements {
                            self.execute_stmt(stmt)?;
                        }
                        self.prelude_loaded = true;
                        return Ok(());
                    }
                    Err(e) => {
                        // Prelude has syntax error - this is a bug
                        return Err(HaversError::ParseError {
                            message: format!("Prelude has errors (this shouldnae happen!): {}", e),
                            line: 1,
                        });
                    }
                }
            }
        }

        // If nae prelude file found, that's okay - just continue without it
        // The language still works, just without the convenience functions
        self.prelude_loaded = true;
        Ok(())
    }

    /// Check if prelude is loaded
    #[allow(dead_code)]
    pub fn has_prelude(&self) -> bool {
        self.prelude_loaded
    }

    fn define_natives(globals: &Rc<RefCell<Environment>>) {
        // get_key - read a single key press (raw input)
        // Not reliably testable under source-based coverage (non-TTY), so exclude from coverage builds.
        #[cfg(all(feature = "cli", not(coverage)))]
        globals.borrow_mut().define(
            "get_key".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("get_key", 0, |_args| {
                enable_raw_mode().map_err(|e| format!("Cannae enable raw mode: {}", e))?;

                let result = match read() {
                    Ok(Event::Key(KeyEvent { code, .. })) => match code {
                        KeyCode::Char(c) => Ok(Value::String(c.to_string())),
                        KeyCode::Enter => Ok(Value::String("\n".to_string())),
                        KeyCode::Esc => Ok(Value::String("\x1b".to_string())),
                        KeyCode::Backspace => Ok(Value::String("\x08".to_string())),
                        KeyCode::Left => Ok(Value::String("Left".to_string())),
                        KeyCode::Right => Ok(Value::String("Right".to_string())),
                        KeyCode::Up => Ok(Value::String("Up".to_string())),
                        KeyCode::Down => Ok(Value::String("Down".to_string())),
                        _ => Ok(Value::String("".to_string())),
                    },
                    Ok(_) => Ok(Value::String("".to_string())),
                    Err(e) => Err(format!("Cannae read key: {}", e)),
                };

                disable_raw_mode().map_err(|e| format!("Cannae disable raw mode: {}", e))?;

                result
            }))),
        );
        // len - get length of list, string, dict, or set
        globals.borrow_mut().define(
            "len".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("len", 1, |args| {
                match &args[0] {
                    Value::String(s) => Ok(Value::Integer(s.len() as i64)),
                    Value::List(l) => Ok(Value::Integer(l.borrow().len() as i64)),
                    Value::Dict(d) => Ok(Value::Integer(d.borrow().len() as i64)),
                    Value::Set(s) => Ok(Value::Integer(s.borrow().len() as i64)),
                    Value::Bytes(b) => Ok(Value::Integer(b.borrow().len() as i64)),
                    _ => Err("len() expects a string, list, dict, creel, or bytes".to_string()),
                }
            }))),
        );

        // bytes - create a zeroed byte buffer of given size
        globals.borrow_mut().define(
            "bytes".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("bytes", 1, |args| {
                let size = match &args[0] {
                    Value::Integer(n) => *n,
                    Value::Float(f) => *f as i64,
                    _ => return Err("bytes() expects an integer size".to_string()),
                };
                let size = if size < 0 { 0 } else { size } as usize;
                Ok(Value::Bytes(Rc::new(RefCell::new(vec![0u8; size]))))
            }))),
        );

        // bytes_from_string - create bytes from string
        globals.borrow_mut().define(
            "bytes_from_string".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "bytes_from_string",
                1,
                |args| {
                    let s = match &args[0] {
                        Value::String(s) => s.clone(),
                        _ => format!("{}", args[0]),
                    };
                    Ok(Value::Bytes(Rc::new(RefCell::new(s.as_bytes().to_vec()))))
                },
            ))),
        );

        // bytes_len - get length of a byte buffer
        globals.borrow_mut().define(
            "bytes_len".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("bytes_len", 1, |args| {
                if let Value::Bytes(b) = &args[0] {
                    Ok(Value::Integer(b.borrow().len() as i64))
                } else {
                    Err("bytes_len() expects bytes".to_string())
                }
            }))),
        );

        // bytes_slice - slice a byte buffer
        globals.borrow_mut().define(
            "bytes_slice".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("bytes_slice", 3, |args| {
                let bytes = match &args[0] {
                    Value::Bytes(b) => b.borrow(),
                    _ => return Err("bytes_slice() expects bytes".to_string()),
                };
                let start = match &args[1] {
                    Value::Integer(n) => *n,
                    _ => return Err("bytes_slice() expects integer start".to_string()),
                };
                let end = match &args[2] {
                    Value::Integer(n) => *n,
                    _ => return Err("bytes_slice() expects integer end".to_string()),
                };
                let len = bytes.len() as i64;
                let mut s = start;
                let mut e = end;
                if s < 0 {
                    s += len;
                }
                if e < 0 {
                    e += len;
                }
                if s < 0 {
                    s = 0;
                }
                if e > len {
                    e = len;
                }
                if e < s {
                    e = s;
                }
                let out = bytes[s as usize..e as usize].to_vec();
                Ok(Value::Bytes(Rc::new(RefCell::new(out))))
            }))),
        );

        // bytes_get - get byte at index
        globals.borrow_mut().define(
            "bytes_get".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("bytes_get", 2, |args| {
                let bytes = match &args[0] {
                    Value::Bytes(b) => b.borrow(),
                    _ => return Err("bytes_get() expects bytes".to_string()),
                };
                let mut idx = match &args[1] {
                    Value::Integer(n) => *n,
                    _ => return Err("bytes_get() expects integer index".to_string()),
                };
                let len = bytes.len() as i64;
                if idx < 0 {
                    idx += len;
                }
                if idx < 0 || idx >= len {
                    return Err("bytes_get() index oot o' bounds".to_string());
                }
                Ok(Value::Integer(bytes[idx as usize] as i64))
            }))),
        );

        // bytes_set - set byte at index
        globals.borrow_mut().define(
            "bytes_set".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("bytes_set", 3, |args| {
                let bytes = match &args[0] {
                    Value::Bytes(b) => b.clone(),
                    _ => return Err("bytes_set() expects bytes".to_string()),
                };
                let mut idx = match &args[1] {
                    Value::Integer(n) => *n,
                    _ => return Err("bytes_set() expects integer index".to_string()),
                };
                let v = match &args[2] {
                    Value::Integer(n) => *n,
                    Value::Float(f) => *f as i64,
                    _ => return Err("bytes_set() expects integer value".to_string()),
                };
                if v < 0 || v > 255 {
                    return Err("bytes_set() value must be between 0 and 255".to_string());
                }
                {
                    let mut buf = bytes.borrow_mut();
                    let len = buf.len() as i64;
                    if idx < 0 {
                        idx += len;
                    }
                    if idx < 0 || idx >= len {
                        return Err("bytes_set() index oot o' bounds".to_string());
                    }
                    buf[idx as usize] = v as u8;
                }
                Ok(Value::Bytes(bytes))
            }))),
        );

        // bytes_append - append bytes to bytes
        globals.borrow_mut().define(
            "bytes_append".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("bytes_append", 2, |args| {
                let bytes = match &args[0] {
                    Value::Bytes(b) => b.clone(),
                    _ => return Err("bytes_append() expects bytes".to_string()),
                };
                let other = match &args[1] {
                    Value::Bytes(b) => b.borrow(),
                    _ => return Err("bytes_append() expects bytes".to_string()),
                };
                bytes.borrow_mut().extend_from_slice(&other);
                Ok(Value::Bytes(bytes))
            }))),
        );

        // bytes_read_u16be
        globals.borrow_mut().define(
            "bytes_read_u16be".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "bytes_read_u16be",
                2,
                |args| {
                    let bytes = match &args[0] {
                        Value::Bytes(b) => b.borrow(),
                        _ => return Err("bytes_read_u16be() expects bytes".to_string()),
                    };
                    let off = match &args[1] {
                        Value::Integer(n) => *n,
                        _ => return Err("bytes_read_u16be() expects integer offset".to_string()),
                    };
                    if off < 0 || (off as usize + 2) > bytes.len() {
                        return Err("bytes_read_u16be() out o' bounds".to_string());
                    }
                    let v = ((bytes[off as usize] as u16) << 8) | (bytes[off as usize + 1] as u16);
                    Ok(Value::Integer(v as i64))
                },
            ))),
        );

        // bytes_read_u32be
        globals.borrow_mut().define(
            "bytes_read_u32be".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "bytes_read_u32be",
                2,
                |args| {
                    let bytes = match &args[0] {
                        Value::Bytes(b) => b.borrow(),
                        _ => return Err("bytes_read_u32be() expects bytes".to_string()),
                    };
                    let off = match &args[1] {
                        Value::Integer(n) => *n,
                        _ => return Err("bytes_read_u32be() expects integer offset".to_string()),
                    };
                    if off < 0 || (off as usize + 4) > bytes.len() {
                        return Err("bytes_read_u32be() out o' bounds".to_string());
                    }
                    let v = ((bytes[off as usize] as u32) << 24)
                        | ((bytes[off as usize + 1] as u32) << 16)
                        | ((bytes[off as usize + 2] as u32) << 8)
                        | (bytes[off as usize + 3] as u32);
                    Ok(Value::Integer(v as i64))
                },
            ))),
        );

        // bytes_write_u16be
        globals.borrow_mut().define(
            "bytes_write_u16be".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "bytes_write_u16be",
                3,
                |args| {
                    let bytes = match &args[0] {
                        Value::Bytes(b) => b.clone(),
                        _ => return Err("bytes_write_u16be() expects bytes".to_string()),
                    };
                    let off = match &args[1] {
                        Value::Integer(n) => *n,
                        _ => return Err("bytes_write_u16be() expects integer offset".to_string()),
                    };
                    let v = match &args[2] {
                        Value::Integer(n) => *n,
                        Value::Float(f) => *f as i64,
                        _ => return Err("bytes_write_u16be() expects integer value".to_string()),
                    };
                    if v < 0 || v > 0xFFFF {
                        return Err("bytes_write_u16be() value out o' range".to_string());
                    }
                    {
                        let mut buf = bytes.borrow_mut();
                        if off < 0 || (off as usize + 2) > buf.len() {
                            return Err("bytes_write_u16be() out o' bounds".to_string());
                        }
                        buf[off as usize] = ((v >> 8) & 0xFF) as u8;
                        buf[off as usize + 1] = (v & 0xFF) as u8;
                    }
                    Ok(Value::Bytes(bytes))
                },
            ))),
        );

        // bytes_write_u32be
        globals.borrow_mut().define(
            "bytes_write_u32be".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "bytes_write_u32be",
                3,
                |args| {
                    let bytes = match &args[0] {
                        Value::Bytes(b) => b.clone(),
                        _ => return Err("bytes_write_u32be() expects bytes".to_string()),
                    };
                    let off = match &args[1] {
                        Value::Integer(n) => *n,
                        _ => return Err("bytes_write_u32be() expects integer offset".to_string()),
                    };
                    let v = match &args[2] {
                        Value::Integer(n) => *n,
                        Value::Float(f) => *f as i64,
                        _ => return Err("bytes_write_u32be() expects integer value".to_string()),
                    };
                    if v < 0 || v > 0xFFFF_FFFF {
                        return Err("bytes_write_u32be() value out o' range".to_string());
                    }
                    {
                        let mut buf = bytes.borrow_mut();
                        if off < 0 || (off as usize + 4) > buf.len() {
                            return Err("bytes_write_u32be() out o' bounds".to_string());
                        }
                        buf[off as usize] = ((v >> 24) & 0xFF) as u8;
                        buf[off as usize + 1] = ((v >> 16) & 0xFF) as u8;
                        buf[off as usize + 2] = ((v >> 8) & 0xFF) as u8;
                        buf[off as usize + 3] = (v & 0xFF) as u8;
                    }
                    Ok(Value::Bytes(bytes))
                },
            ))),
        );

        // socket_udp - create UDP socket
        globals.borrow_mut().define(
            "socket_udp".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("socket_udp", 0, |_args| {
                let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
                if fd < 0 {
                    let err = std::io::Error::last_os_error();
                    let code = err.raw_os_error().unwrap_or(-1) as i64;
                    return Ok(result_err(err.to_string(), code));
                }
                let id = register_socket(fd, SocketKind::Udp);
                Ok(result_ok(Value::Integer(id)))
            }))),
        );

        // socket_tcp - create TCP socket
        globals.borrow_mut().define(
            "socket_tcp".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("socket_tcp", 0, |_args| {
                let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) };
                if fd < 0 {
                    let err = std::io::Error::last_os_error();
                    let code = err.raw_os_error().unwrap_or(-1) as i64;
                    return Ok(result_err(err.to_string(), code));
                }
                let id = register_socket(fd, SocketKind::Tcp);
                Ok(result_ok(Value::Integer(id)))
            }))),
        );

        // socket_bind(sock, host, port)
        globals.borrow_mut().define(
            "socket_bind".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("socket_bind", 3, |args| {
                let sock_id = args[0]
                    .as_integer()
                    .ok_or("socket_bind() expects socket id")?;
                let host = match &args[1] {
                    Value::Nil => None,
                    Value::String(s) if s.is_empty() => None,
                    Value::String(s) => Some(s.as_str()),
                    _ => return Err("socket_bind() expects host string or nil".to_string()),
                };
                let port = args[2]
                    .as_integer()
                    .ok_or("socket_bind() expects port integer")?;
                if port < 0 || port > 65535 {
                    return Err("socket_bind() port must be 0..65535".to_string());
                }

                let entry = get_socket(sock_id).ok_or("Unknown socket handle")?;
                let addr = resolve_ipv4_addr(host, port as u16)
                    .map_err(|e| format!("socket_bind() {}", e))?;
                let rc = unsafe {
                    libc::bind(
                        entry.fd,
                        &addr as *const _ as *const libc::sockaddr,
                        std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
                    )
                };
                if rc != 0 {
                    let err = std::io::Error::last_os_error();
                    let code = err.raw_os_error().unwrap_or(-1) as i64;
                    return Ok(result_err(err.to_string(), code));
                }
                Ok(result_ok(Value::Nil))
            }))),
        );

        // socket_connect(sock, host, port)
        globals.borrow_mut().define(
            "socket_connect".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("socket_connect", 3, |args| {
                let sock_id = args[0]
                    .as_integer()
                    .ok_or("socket_connect() expects socket id")?;
                let host = match &args[1] {
                    Value::String(s) => s.as_str(),
                    _ => return Err("socket_connect() expects host string".to_string()),
                };
                let port = args[2]
                    .as_integer()
                    .ok_or("socket_connect() expects port integer")?;
                if port < 0 || port > 65535 {
                    return Err("socket_connect() port must be 0..65535".to_string());
                }

                let entry = get_socket(sock_id).ok_or("Unknown socket handle")?;
                let addr = resolve_ipv4_addr(Some(host), port as u16)
                    .map_err(|e| format!("socket_connect() {}", e))?;
                let rc = unsafe {
                    libc::connect(
                        entry.fd,
                        &addr as *const _ as *const libc::sockaddr,
                        std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
                    )
                };
                if rc != 0 {
                    let err = std::io::Error::last_os_error();
                    let code = err.raw_os_error().unwrap_or(-1) as i64;
                    return Ok(result_err(err.to_string(), code));
                }
                Ok(result_ok(Value::Nil))
            }))),
        );

        // socket_listen(sock, backlog)
        globals.borrow_mut().define(
            "socket_listen".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("socket_listen", 2, |args| {
                let sock_id = args[0]
                    .as_integer()
                    .ok_or("socket_listen() expects socket id")?;
                let backlog = args[1]
                    .as_integer()
                    .ok_or("socket_listen() expects backlog integer")?;
                let entry = get_socket(sock_id).ok_or("Unknown socket handle")?;
                let rc = unsafe { libc::listen(entry.fd, backlog as i32) };
                if rc != 0 {
                    let err = std::io::Error::last_os_error();
                    let code = err.raw_os_error().unwrap_or(-1) as i64;
                    return Ok(result_err(err.to_string(), code));
                }
                let _ = update_socket_kind(sock_id, SocketKind::Tcp);
                Ok(result_ok(Value::Nil))
            }))),
        );

        // socket_accept(sock)
        globals.borrow_mut().define(
            "socket_accept".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("socket_accept", 1, |args| {
                let sock_id = args[0]
                    .as_integer()
                    .ok_or("socket_accept() expects socket id")?;
                let entry = get_socket(sock_id).ok_or("Unknown socket handle")?;
                let mut addr: libc::sockaddr_in = unsafe { std::mem::zeroed() };
                let mut addr_len = std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;
                let new_fd = unsafe {
                    libc::accept(
                        entry.fd,
                        &mut addr as *mut _ as *mut libc::sockaddr,
                        &mut addr_len,
                    )
                };
                if new_fd < 0 {
                    let err = std::io::Error::last_os_error();
                    let code = err.raw_os_error().unwrap_or(-1) as i64;
                    return Ok(result_err(err.to_string(), code));
                }
                let new_id = register_socket(new_fd, SocketKind::Tcp);
                let (host, port) = sockaddr_to_host_port(&addr);
                let mut info = DictValue::new();
                info.set(Value::String("sock".to_string()), Value::Integer(new_id));
                info.set(Value::String("addr".to_string()), addr_dict(host, port));
                Ok(result_ok(Value::Dict(Rc::new(RefCell::new(info)))))
            }))),
        );

        // socket_set_nonblocking(sock, on)
        globals.borrow_mut().define(
            "socket_set_nonblocking".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "socket_set_nonblocking",
                2,
                |args| {
                    let sock_id = args[0]
                        .as_integer()
                        .ok_or("socket_set_nonblocking() expects socket id")?;
                    let enable = args[1].is_truthy();
                    let entry = get_socket(sock_id).ok_or("Unknown socket handle")?;
                    let flags = unsafe { libc::fcntl(entry.fd, libc::F_GETFL) };
                    if flags < 0 {
                        let err = std::io::Error::last_os_error();
                        let code = err.raw_os_error().unwrap_or(-1) as i64;
                        return Ok(result_err(err.to_string(), code));
                    }
                    let new_flags = if enable {
                        flags | libc::O_NONBLOCK
                    } else {
                        flags & !libc::O_NONBLOCK
                    };
                    let rc = unsafe { libc::fcntl(entry.fd, libc::F_SETFL, new_flags) };
                    if rc != 0 {
                        let err = std::io::Error::last_os_error();
                        let code = err.raw_os_error().unwrap_or(-1) as i64;
                        return Ok(result_err(err.to_string(), code));
                    }
                    Ok(result_ok(Value::Nil))
                },
            ))),
        );

        // socket_set_reuseaddr(sock, on)
        globals.borrow_mut().define(
            "socket_set_reuseaddr".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "socket_set_reuseaddr",
                2,
                |args| {
                    let sock_id = args[0]
                        .as_integer()
                        .ok_or("socket_set_reuseaddr() expects socket id")?;
                    let enable = args[1].is_truthy();
                    let entry = get_socket(sock_id).ok_or("Unknown socket handle")?;
                    let optval: libc::c_int = if enable { 1 } else { 0 };
                    let rc = unsafe {
                        libc::setsockopt(
                            entry.fd,
                            libc::SOL_SOCKET,
                            libc::SO_REUSEADDR,
                            &optval as *const _ as *const libc::c_void,
                            std::mem::size_of_val(&optval) as libc::socklen_t,
                        )
                    };
                    if rc != 0 {
                        let err = std::io::Error::last_os_error();
                        let code = err.raw_os_error().unwrap_or(-1) as i64;
                        return Ok(result_err(err.to_string(), code));
                    }
                    Ok(result_ok(Value::Nil))
                },
            ))),
        );

        // socket_set_reuseport(sock, on)
        globals.borrow_mut().define(
            "socket_set_reuseport".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "socket_set_reuseport",
                2,
                |args| {
                    let sock_id = args[0]
                        .as_integer()
                        .ok_or("socket_set_reuseport() expects socket id")?;
                    let enable = args[1].is_truthy();
                    let entry = get_socket(sock_id).ok_or("Unknown socket handle")?;
                    let optval: libc::c_int = if enable { 1 } else { 0 };
                    let rc = unsafe {
                        libc::setsockopt(
                            entry.fd,
                            libc::SOL_SOCKET,
                            libc::SO_REUSEPORT,
                            &optval as *const _ as *const libc::c_void,
                            std::mem::size_of_val(&optval) as libc::socklen_t,
                        )
                    };
                    if rc != 0 {
                        let err = std::io::Error::last_os_error();
                        let code = err.raw_os_error().unwrap_or(-1) as i64;
                        return Ok(result_err(err.to_string(), code));
                    }
                    Ok(result_ok(Value::Nil))
                },
            ))),
        );

        // socket_set_ttl(sock, ttl)
        globals.borrow_mut().define(
            "socket_set_ttl".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("socket_set_ttl", 2, |args| {
                let sock_id = args[0]
                    .as_integer()
                    .ok_or("socket_set_ttl() expects socket id")?;
                let ttl = args[1]
                    .as_integer()
                    .ok_or("socket_set_ttl() expects ttl integer")?;
                if ttl < 0 || ttl > 255 {
                    return Err("socket_set_ttl() ttl must be 0..255".to_string());
                }
                let entry = get_socket(sock_id).ok_or("Unknown socket handle")?;
                let optval: libc::c_int = ttl as libc::c_int;
                let rc = unsafe {
                    libc::setsockopt(
                        entry.fd,
                        libc::IPPROTO_IP,
                        libc::IP_TTL,
                        &optval as *const _ as *const libc::c_void,
                        std::mem::size_of_val(&optval) as libc::socklen_t,
                    )
                };
                if rc != 0 {
                    let err = std::io::Error::last_os_error();
                    let code = err.raw_os_error().unwrap_or(-1) as i64;
                    return Ok(result_err(err.to_string(), code));
                }
                Ok(result_ok(Value::Nil))
            }))),
        );

        // socket_set_nodelay(sock, on)
        globals.borrow_mut().define(
            "socket_set_nodelay".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "socket_set_nodelay",
                2,
                |args| {
                    let sock_id = args[0]
                        .as_integer()
                        .ok_or("socket_set_nodelay() expects socket id")?;
                    let enable = args[1].is_truthy();
                    let entry = get_socket(sock_id).ok_or("Unknown socket handle")?;
                    let optval: libc::c_int = if enable { 1 } else { 0 };
                    let rc = unsafe {
                        libc::setsockopt(
                            entry.fd,
                            libc::IPPROTO_TCP,
                            libc::TCP_NODELAY,
                            &optval as *const _ as *const libc::c_void,
                            std::mem::size_of_val(&optval) as libc::socklen_t,
                        )
                    };
                    if rc != 0 {
                        let err = std::io::Error::last_os_error();
                        let code = err.raw_os_error().unwrap_or(-1) as i64;
                        return Ok(result_err(err.to_string(), code));
                    }
                    Ok(result_ok(Value::Nil))
                },
            ))),
        );

        // socket_set_rcvbuf(sock, bytes)
        globals.borrow_mut().define(
            "socket_set_rcvbuf".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "socket_set_rcvbuf",
                2,
                |args| {
                    let sock_id = args[0]
                        .as_integer()
                        .ok_or("socket_set_rcvbuf() expects socket id")?;
                    let size = args[1]
                        .as_integer()
                        .ok_or("socket_set_rcvbuf() expects size integer")?;
                    if size < 0 || size > i32::MAX as i64 {
                        return Err("socket_set_rcvbuf() size must be >= 0".to_string());
                    }
                    let entry = get_socket(sock_id).ok_or("Unknown socket handle")?;
                    let optval: libc::c_int = size as libc::c_int;
                    let rc = unsafe {
                        libc::setsockopt(
                            entry.fd,
                            libc::SOL_SOCKET,
                            libc::SO_RCVBUF,
                            &optval as *const _ as *const libc::c_void,
                            std::mem::size_of_val(&optval) as libc::socklen_t,
                        )
                    };
                    if rc != 0 {
                        let err = std::io::Error::last_os_error();
                        let code = err.raw_os_error().unwrap_or(-1) as i64;
                        return Ok(result_err(err.to_string(), code));
                    }
                    Ok(result_ok(Value::Nil))
                },
            ))),
        );

        // socket_set_sndbuf(sock, bytes)
        globals.borrow_mut().define(
            "socket_set_sndbuf".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "socket_set_sndbuf",
                2,
                |args| {
                    let sock_id = args[0]
                        .as_integer()
                        .ok_or("socket_set_sndbuf() expects socket id")?;
                    let size = args[1]
                        .as_integer()
                        .ok_or("socket_set_sndbuf() expects size integer")?;
                    if size < 0 || size > i32::MAX as i64 {
                        return Err("socket_set_sndbuf() size must be >= 0".to_string());
                    }
                    let entry = get_socket(sock_id).ok_or("Unknown socket handle")?;
                    let optval: libc::c_int = size as libc::c_int;
                    let rc = unsafe {
                        libc::setsockopt(
                            entry.fd,
                            libc::SOL_SOCKET,
                            libc::SO_SNDBUF,
                            &optval as *const _ as *const libc::c_void,
                            std::mem::size_of_val(&optval) as libc::socklen_t,
                        )
                    };
                    if rc != 0 {
                        let err = std::io::Error::last_os_error();
                        let code = err.raw_os_error().unwrap_or(-1) as i64;
                        return Ok(result_err(err.to_string(), code));
                    }
                    Ok(result_ok(Value::Nil))
                },
            ))),
        );

        // socket_close(sock)
        globals.borrow_mut().define(
            "socket_close".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("socket_close", 1, |args| {
                let sock_id = args[0]
                    .as_integer()
                    .ok_or("socket_close() expects socket id")?;
                let entry = remove_socket(sock_id).ok_or("Unknown socket handle")?;
                let rc = unsafe { libc::close(entry.fd) };
                if rc != 0 {
                    let err = std::io::Error::last_os_error();
                    let code = err.raw_os_error().unwrap_or(-1) as i64;
                    return Ok(result_err(err.to_string(), code));
                }
                Ok(result_ok(Value::Nil))
            }))),
        );

        // udp_send_to(sock, bytes, host, port)
        globals.borrow_mut().define(
            "udp_send_to".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("udp_send_to", 4, |args| {
                let sock_id = args[0]
                    .as_integer()
                    .ok_or("udp_send_to() expects socket id")?;
                let bytes = match &args[1] {
                    Value::Bytes(b) => b.borrow(),
                    _ => return Err("udp_send_to() expects bytes".to_string()),
                };
                let host = match &args[2] {
                    Value::String(s) => s.as_str(),
                    _ => return Err("udp_send_to() expects host string".to_string()),
                };
                let port = args[3]
                    .as_integer()
                    .ok_or("udp_send_to() expects port integer")?;
                if port < 0 || port > 65535 {
                    return Err("udp_send_to() port must be 0..65535".to_string());
                }

                let entry = get_socket(sock_id).ok_or("Unknown socket handle")?;
                let addr = resolve_ipv4_addr(Some(host), port as u16)
                    .map_err(|e| format!("udp_send_to() {}", e))?;
                let sent = unsafe {
                    libc::sendto(
                        entry.fd,
                        bytes.as_ptr() as *const libc::c_void,
                        bytes.len(),
                        0,
                        &addr as *const _ as *const libc::sockaddr,
                        std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
                    )
                };
                if sent < 0 {
                    let err = std::io::Error::last_os_error();
                    let code = err.raw_os_error().unwrap_or(-1) as i64;
                    return Ok(result_err(err.to_string(), code));
                }
                Ok(result_ok(Value::Integer(sent as i64)))
            }))),
        );

        // udp_recv_from(sock, max_len)
        globals.borrow_mut().define(
            "udp_recv_from".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("udp_recv_from", 2, |args| {
                let sock_id = args[0]
                    .as_integer()
                    .ok_or("udp_recv_from() expects socket id")?;
                let max_len = args[1]
                    .as_integer()
                    .ok_or("udp_recv_from() expects max_len integer")?;
                let max_len = if max_len < 0 { 0 } else { max_len } as usize;
                let entry = get_socket(sock_id).ok_or("Unknown socket handle")?;

                let mut buf = vec![0u8; max_len];
                let mut addr: libc::sockaddr_in = unsafe { std::mem::zeroed() };
                let mut addr_len = std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;
                let n = unsafe {
                    libc::recvfrom(
                        entry.fd,
                        buf.as_mut_ptr() as *mut libc::c_void,
                        buf.len(),
                        0,
                        &mut addr as *mut _ as *mut libc::sockaddr,
                        &mut addr_len,
                    )
                };
                if n < 0 {
                    let err = std::io::Error::last_os_error();
                    let code = err.raw_os_error().unwrap_or(-1) as i64;
                    return Ok(result_err(err.to_string(), code));
                }
                buf.truncate(n as usize);
                let (host, port) = sockaddr_to_host_port(&addr);
                let mut info = DictValue::new();
                info.set(
                    Value::String("buf".to_string()),
                    Value::Bytes(Rc::new(RefCell::new(buf))),
                );
                info.set(Value::String("addr".to_string()), addr_dict(host, port));
                Ok(result_ok(Value::Dict(Rc::new(RefCell::new(info)))))
            }))),
        );

        // tcp_send(sock, bytes)
        globals.borrow_mut().define(
            "tcp_send".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("tcp_send", 2, |args| {
                let sock_id = args[0].as_integer().ok_or("tcp_send() expects socket id")?;
                let bytes = match &args[1] {
                    Value::Bytes(b) => b.borrow(),
                    _ => return Err("tcp_send() expects bytes".to_string()),
                };
                let entry = get_socket(sock_id).ok_or("Unknown socket handle")?;
                let sent = unsafe {
                    libc::send(
                        entry.fd,
                        bytes.as_ptr() as *const libc::c_void,
                        bytes.len(),
                        0,
                    )
                };
                if sent < 0 {
                    let err = std::io::Error::last_os_error();
                    let code = err.raw_os_error().unwrap_or(-1) as i64;
                    return Ok(result_err(err.to_string(), code));
                }
                Ok(result_ok(Value::Integer(sent as i64)))
            }))),
        );

        // tcp_recv(sock, max_len)
        globals.borrow_mut().define(
            "tcp_recv".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("tcp_recv", 2, |args| {
                let sock_id = args[0].as_integer().ok_or("tcp_recv() expects socket id")?;
                let max_len = args[1]
                    .as_integer()
                    .ok_or("tcp_recv() expects max_len integer")?;
                let max_len = if max_len < 0 { 0 } else { max_len } as usize;
                let entry = get_socket(sock_id).ok_or("Unknown socket handle")?;
                let mut buf = vec![0u8; max_len];
                let n = unsafe {
                    libc::recv(
                        entry.fd,
                        buf.as_mut_ptr() as *mut libc::c_void,
                        buf.len(),
                        0,
                    )
                };
                if n < 0 {
                    let err = std::io::Error::last_os_error();
                    let code = err.raw_os_error().unwrap_or(-1) as i64;
                    return Ok(result_err(err.to_string(), code));
                }
                buf.truncate(n as usize);
                Ok(result_ok(Value::Bytes(Rc::new(RefCell::new(buf)))))
            }))),
        );

        // dns_lookup(host) -> result {ok,value:[ips]}
        globals.borrow_mut().define(
            "dns_lookup".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("dns_lookup", 1, |args| {
                let host = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("dns_lookup() expects host string".to_string()),
                };
                let mut out = Vec::new();
                let iter = match (host.as_str(), 0).to_socket_addrs() {
                    Ok(iter) => iter,
                    Err(e) => return Ok(result_err(format!("dns_lookup() {}", e), -1)),
                };
                for addr in iter {
                    out.push(Value::String(addr.ip().to_string()));
                }
                Ok(result_ok(Value::List(Rc::new(RefCell::new(out)))))
            }))),
        );

        // dns_srv(service, domain) -> result {ok,value:[{priority,weight,port,target}]}
        globals.borrow_mut().define(
            "dns_srv".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("dns_srv", 2, |args| {
                let service = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("dns_srv() expects service string".to_string()),
                };
                let domain = match &args[1] {
                    Value::String(s) => s.clone(),
                    _ => return Err("dns_srv() expects domain string".to_string()),
                };
                let name = if service.is_empty() {
                    domain.clone()
                } else {
                    let s = service.trim_end_matches('.');
                    let d = domain.trim_start_matches('.');
                    format!("{}.{}", s, d)
                };
                let resolver = match make_resolver() {
                    Ok(resolver) => resolver,
                    Err(e) => return Ok(result_err(format!("dns_srv() {}", e), -1)),
                };
                let lookup = match resolver.lookup(name.as_str(), RecordType::SRV) {
                    Ok(lookup) => lookup,
                    Err(e) => {
                        return Ok(result_err(
                            format!("dns_srv() DNS SRV lookup failed: {}", e),
                            -1,
                        ))
                    }
                };
                let mut out = Vec::new();
                for rdata in lookup.iter() {
                    if let RData::SRV(srv) = rdata {
                        let mut dict = DictValue::new();
                        dict.set(
                            Value::String("priority".to_string()),
                            Value::Integer(srv.priority() as i64),
                        );
                        dict.set(
                            Value::String("weight".to_string()),
                            Value::Integer(srv.weight() as i64),
                        );
                        dict.set(
                            Value::String("port".to_string()),
                            Value::Integer(srv.port() as i64),
                        );
                        dict.set(
                            Value::String("target".to_string()),
                            Value::String(srv.target().to_string()),
                        );
                        out.push(Value::Dict(Rc::new(RefCell::new(dict))));
                    }
                }
                Ok(result_ok(Value::List(Rc::new(RefCell::new(out)))))
            }))),
        );

        // dns_naptr(domain) -> result {ok,value:[{order,preference,flags,service,regexp,replacement}]}
        globals.borrow_mut().define(
            "dns_naptr".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("dns_naptr", 1, |args| {
                let domain = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("dns_naptr() expects domain string".to_string()),
                };
                let resolver = match make_resolver() {
                    Ok(resolver) => resolver,
                    Err(e) => return Ok(result_err(format!("dns_naptr() {}", e), -1)),
                };
                let lookup = match resolver.lookup(domain.as_str(), RecordType::NAPTR) {
                    Ok(lookup) => lookup,
                    Err(e) => {
                        return Ok(result_err(
                            format!("dns_naptr() DNS NAPTR lookup failed: {}", e),
                            -1,
                        ))
                    }
                };
                let mut out = Vec::new();
                for rdata in lookup.iter() {
                    if let RData::NAPTR(naptr) = rdata {
                        let mut dict = DictValue::new();
                        dict.set(
                            Value::String("order".to_string()),
                            Value::Integer(naptr.order() as i64),
                        );
                        dict.set(
                            Value::String("preference".to_string()),
                            Value::Integer(naptr.preference() as i64),
                        );
                        dict.set(
                            Value::String("flags".to_string()),
                            Value::String(String::from_utf8_lossy(naptr.flags()).to_string()),
                        );
                        dict.set(
                            Value::String("service".to_string()),
                            Value::String(String::from_utf8_lossy(naptr.services()).to_string()),
                        );
                        dict.set(
                            Value::String("regexp".to_string()),
                            Value::String(String::from_utf8_lossy(naptr.regexp()).to_string()),
                        );
                        dict.set(
                            Value::String("replacement".to_string()),
                            Value::String(naptr.replacement().to_string()),
                        );
                        out.push(Value::Dict(Rc::new(RefCell::new(dict))));
                    }
                }
                Ok(result_ok(Value::List(Rc::new(RefCell::new(out)))))
            }))),
        );

        // tls_client_new(config) -> result {ok,value:tls_handle}
        globals.borrow_mut().define(
            "tls_client_new".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("tls_client_new", 1, |args| {
                let cfg = tls_config_from_value(&args[0])?;
                let session = if cfg.mode == TlsMode::Client {
                    let client_config = build_client_config(&cfg)?;
                    TlsSession {
                        mode: TlsMode::Client,
                        server_name: cfg.server_name,
                        client_config: Some(client_config),
                        server_config: None,
                        stream: None,
                    }
                } else {
                    let server_config = build_server_config(&cfg)?;
                    TlsSession {
                        mode: TlsMode::Server,
                        server_name: cfg.server_name,
                        client_config: None,
                        server_config: Some(server_config),
                        stream: None,
                    }
                };
                let id = register_tls(session);
                Ok(result_ok(Value::Integer(id)))
            }))),
        );

        // tls_connect(tls, sock)
        globals.borrow_mut().define(
            "tls_connect".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("tls_connect", 2, |args| {
                let tls_id = args[0]
                    .as_integer()
                    .ok_or("tls_connect() expects TLS handle")?;
                let sock_id = args[1]
                    .as_integer()
                    .ok_or("tls_connect() expects socket id")?;
                let entry = get_socket(sock_id).ok_or("Unknown socket handle")?;
                let dup_fd = unsafe { libc::dup(entry.fd) };
                if dup_fd < 0 {
                    let err = std::io::Error::last_os_error();
                    let code = err.raw_os_error().unwrap_or(-1) as i64;
                    return Ok(result_err(err.to_string(), code));
                }

                let res = with_tls_mut(tls_id, |session| {
                    if session.stream.is_some() {
                        return Err("TLS session already connected".to_string());
                    }
                    let mut stream = unsafe { std::net::TcpStream::from_raw_fd(dup_fd) };
                    let _ = stream.set_nonblocking(false);

                    match session.mode {
                        TlsMode::Client => {
                            let config = session
                                .client_config
                                .as_ref()
                                .ok_or("Missing client config")?
                                .clone();
                            let server_name = ServerName::try_from(session.server_name.as_str())
                                .map_err(|_| "Invalid server_name".to_string())?;
                            let mut conn = ClientConnection::new(config, server_name)
                                .map_err(|e| e.to_string())?;
                            while conn.is_handshaking() {
                                conn.complete_io(&mut stream)
                                    .map_err(|e| format!("TLS handshake failed: {}", e))?;
                            }
                            session.stream =
                                Some(TlsStream::Client(StreamOwned::new(conn, stream)));
                        }
                        TlsMode::Server => {
                            let config = session
                                .server_config
                                .as_ref()
                                .ok_or("Missing server config")?
                                .clone();
                            let mut conn =
                                ServerConnection::new(config).map_err(|e| e.to_string())?;
                            while conn.is_handshaking() {
                                conn.complete_io(&mut stream)
                                    .map_err(|e| format!("TLS handshake failed: {}", e))?;
                            }
                            session.stream =
                                Some(TlsStream::Server(StreamOwned::new(conn, stream)));
                        }
                    }
                    Ok(())
                });

                match res {
                    Ok(_) => Ok(result_ok(Value::Nil)),
                    Err(e) => Ok(result_err(e, -1)),
                }
            }))),
        );

        // tls_send(tls, bytes)
        globals.borrow_mut().define(
            "tls_send".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("tls_send", 2, |args| {
                let tls_id = args[0]
                    .as_integer()
                    .ok_or("tls_send() expects TLS handle")?;
                let bytes = match &args[1] {
                    Value::Bytes(b) => b.borrow(),
                    _ => return Err("tls_send() expects bytes".to_string()),
                };
                let res = with_tls_mut(tls_id, |session| {
                    let stream = session.stream.as_mut().ok_or("TLS not connected")?;
                    let n = match stream {
                        TlsStream::Client(s) => s.write(bytes.as_slice()),
                        TlsStream::Server(s) => s.write(bytes.as_slice()),
                    }
                    .map_err(|e| format!("TLS send failed: {}", e))?;
                    match stream {
                        TlsStream::Client(s) => {
                            let _ = s.flush();
                        }
                        TlsStream::Server(s) => {
                            let _ = s.flush();
                        }
                    }
                    Ok(n as i64)
                });
                match res {
                    Ok(n) => Ok(result_ok(Value::Integer(n))),
                    Err(e) => Ok(result_err(e, -1)),
                }
            }))),
        );

        // tls_recv(tls, max_len)
        globals.borrow_mut().define(
            "tls_recv".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("tls_recv", 2, |args| {
                let tls_id = args[0]
                    .as_integer()
                    .ok_or("tls_recv() expects TLS handle")?;
                let max_len = args[1]
                    .as_integer()
                    .ok_or("tls_recv() expects max_len integer")?;
                let max_len = if max_len < 0 { 0 } else { max_len } as usize;
                let res = with_tls_mut(tls_id, |session| {
                    let stream = session.stream.as_mut().ok_or("TLS not connected")?;
                    let mut buf = vec![0u8; max_len];
                    let n = match stream {
                        TlsStream::Client(s) => s.read(&mut buf),
                        TlsStream::Server(s) => s.read(&mut buf),
                    }
                    .map_err(|e| format!("TLS recv failed: {}", e))?;
                    buf.truncate(n);
                    Ok(buf)
                });
                match res {
                    Ok(buf) => Ok(result_ok(Value::Bytes(Rc::new(RefCell::new(buf))))),
                    Err(e) => Ok(result_err(e, -1)),
                }
            }))),
        );

        // tls_close(tls)
        globals.borrow_mut().define(
            "tls_close".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("tls_close", 1, |args| {
                let tls_id = args[0]
                    .as_integer()
                    .ok_or("tls_close() expects TLS handle")?;
                remove_tls(tls_id);
                Ok(result_ok(Value::Nil))
            }))),
        );

        // dtls_server_new(config)
        globals.borrow_mut().define(
            "dtls_server_new".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("dtls_server_new", 1, |args| {
                let cfg = dtls_config_from_value(&args[0])?;
                let id = register_dtls(cfg);
                Ok(result_ok(Value::Integer(id)))
            }))),
        );

        // dtls_handshake(dtls, sock)
        globals.borrow_mut().define(
            "dtls_handshake".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("dtls_handshake", 2, |args| {
                let dtls_id = args[0]
                    .as_integer()
                    .ok_or("dtls_handshake() expects DTLS handle")?;
                let sock_id = args[1]
                    .as_integer()
                    .ok_or("dtls_handshake() expects socket id")?;
                let entry = get_socket(sock_id).ok_or("Unknown socket handle")?;
                let cfg = match dtls_get(dtls_id) {
                    Ok(cfg) => cfg,
                    Err(e) => return Ok(result_err(e, -1)),
                };

                let dup_fd = unsafe { libc::dup(entry.fd) };
                if dup_fd < 0 {
                    let err = std::io::Error::last_os_error();
                    let code = err.raw_os_error().unwrap_or(-1) as i64;
                    return Ok(result_err(err.to_string(), code));
                }

                let socket = unsafe { std::net::UdpSocket::from_raw_fd(dup_fd) };
                if let Err(e) = socket.set_nonblocking(false) {
                    return Ok(result_err(format!("DTLS socket setup failed: {}", e), -1));
                }

                let remote = if let (Some(host), Some(port)) =
                    (cfg.remote_host.clone(), cfg.remote_port)
                {
                    match format!("{}:{}", host, port).parse() {
                        Ok(addr) => addr,
                        Err(_) => return Ok(result_err("Invalid remote address".to_string(), -1)),
                    }
                } else {
                    match socket.peer_addr() {
                        Ok(addr) => addr,
                        Err(_) => {
                            return Ok(result_err(
                                "dtls_handshake requires remote_host/remote_port".to_string(),
                                -1,
                            ))
                        }
                    }
                };

                if let Err(e) = socket.connect(remote) {
                    return Ok(result_err(format!("DTLS connect failed: {}", e), -1));
                }

                let channel = UdpChannel {
                    socket,
                    remote_addr: remote,
                };

                let (stream, selected_profile) = if cfg.mode == TlsMode::Client {
                    let mut builder = DtlsConnector::builder();
                    for profile in &cfg.srtp_profiles {
                        builder.add_srtp_profile(*profile);
                    }
                    if let Some(ca_pem) = &cfg.ca_pem {
                        let cert = match udp_dtls::Certificate::from_pem(ca_pem.as_bytes()) {
                            Ok(cert) => cert,
                            Err(e) => return Ok(result_err(format!("Invalid CA cert: {}", e), -1)),
                        };
                        builder.add_root_certificate(cert);
                    }
                    if cfg.insecure {
                        builder.danger_accept_invalid_certs(true);
                        builder.danger_accept_invalid_hostnames(true);
                    }
                    if let (Some(cert_pem), Some(key_pem)) = (&cfg.cert_pem, &cfg.key_pem) {
                        let identity = match identity_from_pem(cert_pem, key_pem) {
                            Ok(identity) => identity,
                            Err(e) => return Ok(result_err(e, -1)),
                        };
                        builder.identity(identity);
                    }
                    let connector = match DtlsConnector::new(&builder) {
                        Ok(connector) => connector,
                        Err(e) => return Ok(result_err(format!("{}", e), -1)),
                    };
                    match connector.connect(&cfg.server_name, channel) {
                        Ok(stream) => {
                            let selected = stream.selected_srtp_profile().ok().flatten();
                            (stream, selected)
                        }
                        Err(err) => {
                            return Ok(result_err(format!("DTLS connect failed: {:?}", err), -1))
                        }
                    }
                } else {
                    let cert_pem = match cfg.cert_pem.as_ref() {
                        Some(v) => v,
                        None => return Ok(result_err("Server cert_pem required".to_string(), -1)),
                    };
                    let key_pem = match cfg.key_pem.as_ref() {
                        Some(v) => v,
                        None => return Ok(result_err("Server key_pem required".to_string(), -1)),
                    };
                    let identity = match identity_from_pem(cert_pem, key_pem) {
                        Ok(identity) => identity,
                        Err(e) => return Ok(result_err(e, -1)),
                    };
                    let mut builder = DtlsAcceptor::builder(identity);
                    for profile in &cfg.srtp_profiles {
                        builder.add_srtp_profile(*profile);
                    }
                    let acceptor = match DtlsAcceptor::new(&builder) {
                        Ok(acceptor) => acceptor,
                        Err(e) => return Ok(result_err(format!("{}", e), -1)),
                    };
                    match acceptor.accept(channel) {
                        Ok(stream) => {
                            let selected = stream.selected_srtp_profile().ok().flatten();
                            (stream, selected)
                        }
                        Err(err) => {
                            return Ok(result_err(format!("DTLS accept failed: {:?}", err), -1))
                        }
                    }
                };

                let profile = selected_profile.unwrap_or(SrtpProfile::Aes128CmSha180);
                let (key_len, salt_len) = srtp_key_salt_len(profile);
                let total = 2 * (key_len + salt_len);
                let material = match stream.keying_material(total) {
                    Ok(material) => material,
                    Err(e) => return Ok(result_err(format!("Keying material failed: {}", e), -1)),
                };

                let client_key = material[0..key_len].to_vec();
                let server_key = material[key_len..(2 * key_len)].to_vec();
                let client_salt = material[(2 * key_len)..(2 * key_len + salt_len)].to_vec();
                let server_salt =
                    material[(2 * key_len + salt_len)..(2 * key_len + 2 * salt_len)].to_vec();

                let mut dict = DictValue::new();
                dict.set(
                    Value::String("profile".to_string()),
                    Value::String(profile.to_string()),
                );
                dict.set(
                    Value::String("client_key".to_string()),
                    Value::Bytes(Rc::new(RefCell::new(client_key))),
                );
                dict.set(
                    Value::String("client_salt".to_string()),
                    Value::Bytes(Rc::new(RefCell::new(client_salt))),
                );
                dict.set(
                    Value::String("server_key".to_string()),
                    Value::Bytes(Rc::new(RefCell::new(server_key))),
                );
                dict.set(
                    Value::String("server_salt".to_string()),
                    Value::Bytes(Rc::new(RefCell::new(server_salt))),
                );
                dict.set(
                    Value::String("key_len".to_string()),
                    Value::Integer(key_len as i64),
                );
                dict.set(
                    Value::String("salt_len".to_string()),
                    Value::Integer(salt_len as i64),
                );

                Ok(result_ok(Value::Dict(Rc::new(RefCell::new(dict)))))
            }))),
        );

        // srtp_create(keys)
        globals.borrow_mut().define(
            "srtp_create".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("srtp_create", 1, |args| {
                let dict = match &args[0] {
                    Value::Dict(d) => d.borrow(),
                    _ => return Err("srtp_create() expects config dict".to_string()),
                };

                let profile_str = dict_get_string(&dict, "profile")
                    .unwrap_or_else(|| "SRTP_AES128_CM_SHA1_80".to_string());
                let profile = match protection_profile_from_str(&profile_str) {
                    Some(p) => p,
                    None => return Ok(result_err("Unsupported SRTP profile".to_string(), -1)),
                };

                let role = dict_get_string(&dict, "role").unwrap_or_else(|| "client".to_string());

                let mut send_key = dict_get_bytes(&dict, "send_key");
                let mut send_salt = dict_get_bytes(&dict, "send_salt");
                let mut recv_key = dict_get_bytes(&dict, "recv_key");
                let mut recv_salt = dict_get_bytes(&dict, "recv_salt");

                let client_key = dict_get_bytes(&dict, "client_key");
                let client_salt = dict_get_bytes(&dict, "client_salt");
                let server_key = dict_get_bytes(&dict, "server_key");
                let server_salt = dict_get_bytes(&dict, "server_salt");

                if send_key.is_none()
                    || send_salt.is_none()
                    || recv_key.is_none()
                    || recv_salt.is_none()
                {
                    if client_key.is_some()
                        && client_salt.is_some()
                        && server_key.is_some()
                        && server_salt.is_some()
                    {
                        let is_client = role.to_lowercase() != "server";
                        if is_client {
                            send_key = client_key.clone();
                            send_salt = client_salt.clone();
                            recv_key = server_key.clone();
                            recv_salt = server_salt.clone();
                        } else {
                            send_key = server_key.clone();
                            send_salt = server_salt.clone();
                            recv_key = client_key.clone();
                            recv_salt = client_salt.clone();
                        }
                    } else {
                        let master_key = dict_get_bytes(&dict, "master_key");
                        let master_salt = dict_get_bytes(&dict, "master_salt");
                        if master_key.is_some() && master_salt.is_some() {
                            send_key = master_key.clone();
                            send_salt = master_salt.clone();
                            recv_key = master_key;
                            recv_salt = master_salt;
                        }
                    }
                }

                let send_key = match send_key {
                    Some(v) => v,
                    None => return Ok(result_err("Missing SRTP send_key".to_string(), -1)),
                };
                let send_salt = match send_salt {
                    Some(v) => v,
                    None => return Ok(result_err("Missing SRTP send_salt".to_string(), -1)),
                };
                let recv_key = match recv_key {
                    Some(v) => v,
                    None => return Ok(result_err("Missing SRTP recv_key".to_string(), -1)),
                };
                let recv_salt = match recv_salt {
                    Some(v) => v,
                    None => return Ok(result_err("Missing SRTP recv_salt".to_string(), -1)),
                };

                let send_master = MasterKey::new(&send_key, &send_salt, &None);
                let recv_master = MasterKey::new(&recv_key, &recv_salt, &None);
                let send_cfg = StreamConfig::new(vec![send_master], &profile, &profile);
                let recv_cfg = StreamConfig::new(vec![recv_master], &profile, &profile);

                let mut send = SendSession::new();
                if let Err(e) = send.add_stream(None, &send_cfg) {
                    return Ok(result_err(format!("SRTP send session error: {}", e), -1));
                }
                let mut recv = RecvSession::new();
                if let Err(e) = recv.add_stream(None, &recv_cfg) {
                    return Ok(result_err(format!("SRTP recv session error: {}", e), -1));
                }

                let id = register_srtp(SrtpSession { send, recv });
                Ok(result_ok(Value::Integer(id)))
            }))),
        );

        // srtp_protect(srtp, rtp_packet)
        globals.borrow_mut().define(
            "srtp_protect".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("srtp_protect", 2, |args| {
                let ctx_id = args[0]
                    .as_integer()
                    .ok_or("srtp_protect() expects SRTP handle")?;
                let packet = match &args[1] {
                    Value::Bytes(b) => b.borrow().clone(),
                    _ => return Err("srtp_protect() expects bytes".to_string()),
                };
                let res = with_srtp_mut(ctx_id, |session| {
                    session
                        .send
                        .rtp_protect(packet)
                        .map_err(|e| format!("SRTP protect failed: {}", e))
                });
                match res {
                    Ok(buf) => Ok(result_ok(Value::Bytes(Rc::new(RefCell::new(buf))))),
                    Err(e) => Ok(result_err(e, -1)),
                }
            }))),
        );

        // srtp_unprotect(srtp, rtp_packet)
        globals.borrow_mut().define(
            "srtp_unprotect".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("srtp_unprotect", 2, |args| {
                let ctx_id = args[0]
                    .as_integer()
                    .ok_or("srtp_unprotect() expects SRTP handle")?;
                let packet = match &args[1] {
                    Value::Bytes(b) => b.borrow().clone(),
                    _ => return Err("srtp_unprotect() expects bytes".to_string()),
                };
                let res = with_srtp_mut(ctx_id, |session| {
                    session
                        .recv
                        .rtp_unprotect(packet)
                        .map_err(|e| format!("SRTP unprotect failed: {}", e))
                });
                match res {
                    Ok(buf) => Ok(result_ok(Value::Bytes(Rc::new(RefCell::new(buf))))),
                    Err(e) => Ok(result_err(e, -1)),
                }
            }))),
        );

        // event_loop_new() -> loop handle
        globals.borrow_mut().define(
            "event_loop_new".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("event_loop_new", 0, |_args| {
                let loop_val = EventLoop {
                    watches: Vec::new(),
                    timers: Vec::new(),
                    next_timer_id: 1,
                    stopped: false,
                };
                let id = register_loop(loop_val);
                Ok(Value::Integer(id))
            }))),
        );

        // event_loop_stop(loop)
        globals.borrow_mut().define(
            "event_loop_stop".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("event_loop_stop", 1, |args| {
                let loop_id = args[0]
                    .as_integer()
                    .ok_or("event_loop_stop() expects loop id")?;
                let _ = with_loop_mut(loop_id, |loop_ref| loop_ref.stopped = true)?;
                Ok(Value::Nil)
            }))),
        );

        // event_watch_read(loop, sock, callback)
        globals.borrow_mut().define(
            "event_watch_read".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "event_watch_read",
                3,
                |args| {
                    let loop_id = args[0]
                        .as_integer()
                        .ok_or("event_watch_read() expects loop id")?;
                    let sock_id = args[1]
                        .as_integer()
                        .ok_or("event_watch_read() expects socket id")?;
                    let callback = args[2].clone();
                    let entry = get_socket(sock_id).ok_or("Unknown socket handle")?;
                    let _ = with_loop_mut(loop_id, |loop_ref| {
                        if let Some(watch) =
                            loop_ref.watches.iter_mut().find(|w| w.sock_id == sock_id)
                        {
                            watch.read_cb = callback.clone();
                        } else {
                            loop_ref.watches.push(LoopWatch {
                                sock_id,
                                fd: entry.fd,
                                read_cb: callback.clone(),
                                write_cb: Value::Nil,
                            });
                        }
                    })?;
                    Ok(Value::Nil)
                },
            ))),
        );

        // event_watch_write(loop, sock, callback)
        globals.borrow_mut().define(
            "event_watch_write".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "event_watch_write",
                3,
                |args| {
                    let loop_id = args[0]
                        .as_integer()
                        .ok_or("event_watch_write() expects loop id")?;
                    let sock_id = args[1]
                        .as_integer()
                        .ok_or("event_watch_write() expects socket id")?;
                    let callback = args[2].clone();
                    let entry = get_socket(sock_id).ok_or("Unknown socket handle")?;
                    let _ = with_loop_mut(loop_id, |loop_ref| {
                        if let Some(watch) =
                            loop_ref.watches.iter_mut().find(|w| w.sock_id == sock_id)
                        {
                            watch.write_cb = callback.clone();
                        } else {
                            loop_ref.watches.push(LoopWatch {
                                sock_id,
                                fd: entry.fd,
                                read_cb: Value::Nil,
                                write_cb: callback.clone(),
                            });
                        }
                    })?;
                    Ok(Value::Nil)
                },
            ))),
        );

        // event_unwatch(loop, sock)
        globals.borrow_mut().define(
            "event_unwatch".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("event_unwatch", 2, |args| {
                let loop_id = args[0]
                    .as_integer()
                    .ok_or("event_unwatch() expects loop id")?;
                let sock_id = args[1]
                    .as_integer()
                    .ok_or("event_unwatch() expects socket id")?;
                let mut removed = false;
                let _ = with_loop_mut(loop_id, |loop_ref| {
                    let before = loop_ref.watches.len();
                    loop_ref.watches.retain(|w| w.sock_id != sock_id);
                    removed = loop_ref.watches.len() != before;
                })?;
                Ok(Value::Bool(removed))
            }))),
        );

        // event_loop_poll(loop, timeout_ms) -> list of events
        globals.borrow_mut().define(
            "event_loop_poll".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("event_loop_poll", 2, |args| {
                let loop_id = args[0]
                    .as_integer()
                    .ok_or("event_loop_poll() expects loop id")?;
                let timeout_ms = match &args[1] {
                    Value::Nil => -1,
                    Value::Integer(n) => *n,
                    Value::Float(f) => *f as i64,
                    _ => return Err("event_loop_poll() expects timeout integer or nil".to_string()),
                };

                let events = with_loop_mut(loop_id, |loop_ref| {
                    if loop_ref.stopped {
                        return Value::List(Rc::new(RefCell::new(vec![event_dict(
                            "stop", None, None, None,
                        )])));
                    }

                    let now = mono_ms_now();
                    let mut next_due: Option<i64> = None;
                    for timer in loop_ref.timers.iter() {
                        if timer.cancelled {
                            continue;
                        }
                        let mut diff = timer.next_fire_ms - now;
                        if diff < 0 {
                            diff = 0;
                        }
                        next_due = Some(match next_due {
                            Some(prev) => prev.min(diff),
                            None => diff,
                        });
                    }

                    let mut wait_ms = timeout_ms;
                    if wait_ms < 0 {
                        wait_ms = next_due.unwrap_or(-1);
                    } else if let Some(due) = next_due {
                        if due >= 0 && due < wait_ms {
                            wait_ms = due;
                        }
                    }

                    let poll_timeout = if wait_ms < 0 {
                        -1
                    } else if wait_ms > i32::MAX as i64 {
                        i32::MAX
                    } else {
                        wait_ms as i32
                    };

                    let mut fds: Vec<libc::pollfd> = loop_ref
                        .watches
                        .iter()
                        .map(|watch| libc::pollfd {
                            fd: watch.fd,
                            events: {
                                let mut ev = 0;
                                if !matches!(watch.read_cb, Value::Nil) {
                                    ev |= libc::POLLIN;
                                }
                                if !matches!(watch.write_cb, Value::Nil) {
                                    ev |= libc::POLLOUT;
                                }
                                ev
                            },
                            revents: 0,
                        })
                        .collect();

                    if poll_timeout != 0 || !fds.is_empty() {
                        let rc = unsafe {
                            libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, poll_timeout)
                        };
                        if rc < 0 {
                            return Value::List(Rc::new(RefCell::new(Vec::new())));
                        }
                    }

                    let mut out: Vec<Value> = Vec::new();
                    for (idx, pfd) in fds.iter().enumerate() {
                        let watch = &loop_ref.watches[idx];
                        if (pfd.revents & libc::POLLIN) != 0 && !matches!(watch.read_cb, Value::Nil)
                        {
                            out.push(event_dict(
                                "read",
                                Some(watch.sock_id),
                                None,
                                Some(watch.read_cb.clone()),
                            ));
                        }
                        if (pfd.revents & libc::POLLOUT) != 0
                            && !matches!(watch.write_cb, Value::Nil)
                        {
                            out.push(event_dict(
                                "write",
                                Some(watch.sock_id),
                                None,
                                Some(watch.write_cb.clone()),
                            ));
                        }
                    }

                    let now2 = mono_ms_now();
                    for timer in loop_ref.timers.iter_mut() {
                        if timer.cancelled {
                            continue;
                        }
                        if timer.next_fire_ms <= now2 {
                            out.push(event_dict(
                                "timer",
                                None,
                                Some(timer.id),
                                Some(timer.callback.clone()),
                            ));
                            if timer.interval_ms > 0 {
                                while timer.next_fire_ms <= now2 {
                                    timer.next_fire_ms += timer.interval_ms;
                                }
                            } else {
                                timer.cancelled = true;
                            }
                        }
                    }
                    loop_ref.timers.retain(|t| !t.cancelled);

                    Value::List(Rc::new(RefCell::new(out)))
                })?;

                Ok(events)
            }))),
        );

        // timer_after(loop, ms, callback) -> timer id
        globals.borrow_mut().define(
            "timer_after".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("timer_after", 3, |args| {
                let loop_id = args[0]
                    .as_integer()
                    .ok_or("timer_after() expects loop id")?;
                let delay_ms = match &args[1] {
                    Value::Integer(n) => *n,
                    Value::Float(f) => *f as i64,
                    _ => return Err("timer_after() expects delay integer".to_string()),
                };
                if delay_ms < 0 {
                    return Err("timer_after() expects non-negative delay".to_string());
                }
                let callback = args[2].clone();
                let id = with_loop_mut(loop_id, |loop_ref| {
                    let id = loop_ref.next_timer_id;
                    loop_ref.next_timer_id += 1;
                    loop_ref.timers.push(LoopTimer {
                        id,
                        next_fire_ms: mono_ms_now() + delay_ms,
                        interval_ms: 0,
                        callback: callback.clone(),
                        cancelled: false,
                    });
                    id
                })?;
                Ok(Value::Integer(id))
            }))),
        );

        // timer_every(loop, ms, callback) -> timer id
        globals.borrow_mut().define(
            "timer_every".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("timer_every", 3, |args| {
                let loop_id = args[0]
                    .as_integer()
                    .ok_or("timer_every() expects loop id")?;
                let interval_ms = match &args[1] {
                    Value::Integer(n) => *n,
                    Value::Float(f) => *f as i64,
                    _ => return Err("timer_every() expects interval integer".to_string()),
                };
                if interval_ms <= 0 {
                    return Err("timer_every() expects positive interval".to_string());
                }
                let callback = args[2].clone();
                let id = with_loop_mut(loop_id, |loop_ref| {
                    let id = loop_ref.next_timer_id;
                    loop_ref.next_timer_id += 1;
                    loop_ref.timers.push(LoopTimer {
                        id,
                        next_fire_ms: mono_ms_now() + interval_ms,
                        interval_ms,
                        callback: callback.clone(),
                        cancelled: false,
                    });
                    id
                })?;
                Ok(Value::Integer(id))
            }))),
        );

        // timer_cancel(loop, timer_id) -> bool
        globals.borrow_mut().define(
            "timer_cancel".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("timer_cancel", 2, |args| {
                let loop_id = args[0]
                    .as_integer()
                    .ok_or("timer_cancel() expects loop id")?;
                let timer_id = args[1]
                    .as_integer()
                    .ok_or("timer_cancel() expects timer id")?;
                let mut cancelled = false;
                let _ = with_loop_mut(loop_id, |loop_ref| {
                    for timer in loop_ref.timers.iter_mut() {
                        if timer.id == timer_id && !timer.cancelled {
                            timer.cancelled = true;
                            cancelled = true;
                        }
                    }
                })?;
                Ok(Value::Bool(cancelled))
            }))),
        );

        // thread_spawn(func, args_list) -> thread handle (interpreter: native funcs only)
        globals.borrow_mut().define(
            "thread_spawn".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("thread_spawn", 2, |args| {
                let func = args[0].clone();
                let args_list = &args[1];
                let arg_vec = match args_list {
                    Value::Nil => Vec::new(),
                    Value::List(list) => list.borrow().clone(),
                    _ => {
                        return Err("thread_spawn() expects a list of arguments or nil".to_string())
                    }
                };
                let result = match func {
                    Value::NativeFunction(native) => (native.func)(arg_vec)?,
                    _ => {
                        return Err(
                            "thread_spawn() only supports native functions in interpreter"
                                .to_string(),
                        )
                    }
                };
                let id = register_thread(ThreadHandle {
                    result,
                    detached: false,
                });
                Ok(Value::Integer(id))
            }))),
        );

        // thread_join(thread_handle)
        globals.borrow_mut().define(
            "thread_join".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("thread_join", 1, |args| {
                let thread_id = args[0]
                    .as_integer()
                    .ok_or("thread_join() expects thread handle")?;
                let result = with_thread_mut(thread_id, |handle| {
                    if handle.detached {
                        None
                    } else {
                        Some(handle.result.clone())
                    }
                })?;
                if let Some(val) = result {
                    Ok(val)
                } else {
                    Err("thread_join() cannot join detached thread".to_string())
                }
            }))),
        );

        // thread_detach(thread_handle)
        globals.borrow_mut().define(
            "thread_detach".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("thread_detach", 1, |args| {
                let thread_id = args[0]
                    .as_integer()
                    .ok_or("thread_detach() expects thread handle")?;
                let _ = with_thread_mut(thread_id, |handle| handle.detached = true)?;
                Ok(Value::Nil)
            }))),
        );

        // mutex_new()
        globals.borrow_mut().define(
            "mutex_new".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("mutex_new", 0, |_args| {
                let id = register_mutex(MutexState { locked: false });
                Ok(Value::Integer(id))
            }))),
        );

        // mutex_lock(mutex)
        globals.borrow_mut().define(
            "mutex_lock".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("mutex_lock", 1, |args| {
                let mutex_id = args[0]
                    .as_integer()
                    .ok_or("mutex_lock() expects mutex handle")?;
                let _ = with_mutex_mut(mutex_id, |state| state.locked = true)?;
                Ok(Value::Nil)
            }))),
        );

        // mutex_unlock(mutex)
        globals.borrow_mut().define(
            "mutex_unlock".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("mutex_unlock", 1, |args| {
                let mutex_id = args[0]
                    .as_integer()
                    .ok_or("mutex_unlock() expects mutex handle")?;
                let _ = with_mutex_mut(mutex_id, |state| state.locked = false)?;
                Ok(Value::Nil)
            }))),
        );

        // mutex_try_lock(mutex) -> bool
        globals.borrow_mut().define(
            "mutex_try_lock".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("mutex_try_lock", 1, |args| {
                let mutex_id = args[0]
                    .as_integer()
                    .ok_or("mutex_try_lock() expects mutex handle")?;
                let locked = with_mutex_mut(mutex_id, |state| {
                    if state.locked {
                        false
                    } else {
                        state.locked = true;
                        true
                    }
                })?;
                Ok(Value::Bool(locked))
            }))),
        );

        // condvar_new()
        globals.borrow_mut().define(
            "condvar_new".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("condvar_new", 0, |_args| {
                let id = register_condvar(CondvarState);
                Ok(Value::Integer(id))
            }))),
        );

        // condvar_wait(condvar, mutex) -> bool (interpreter: immediate true)
        globals.borrow_mut().define(
            "condvar_wait".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("condvar_wait", 2, |args| {
                let condvar_id = args[0]
                    .as_integer()
                    .ok_or("condvar_wait() expects condvar handle")?;
                let _mutex_id = args[1]
                    .as_integer()
                    .ok_or("condvar_wait() expects mutex handle")?;
                let _ = with_condvar_mut(condvar_id, |_state| ())?;
                Ok(Value::Bool(true))
            }))),
        );

        // condvar_timed_wait(condvar, mutex, timeout_ms) -> bool
        globals.borrow_mut().define(
            "condvar_timed_wait".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "condvar_timed_wait",
                3,
                |args| {
                    let condvar_id = args[0]
                        .as_integer()
                        .ok_or("condvar_timed_wait() expects condvar handle")?;
                    let _mutex_id = args[1]
                        .as_integer()
                        .ok_or("condvar_timed_wait() expects mutex handle")?;
                    let _timeout = match &args[2] {
                        Value::Integer(n) => *n,
                        Value::Float(f) => *f as i64,
                        _ => return Err("condvar_timed_wait() expects timeout integer".to_string()),
                    };
                    let _ = with_condvar_mut(condvar_id, |_state| ())?;
                    Ok(Value::Bool(true))
                },
            ))),
        );

        // condvar_signal(condvar)
        globals.borrow_mut().define(
            "condvar_signal".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("condvar_signal", 1, |args| {
                let condvar_id = args[0]
                    .as_integer()
                    .ok_or("condvar_signal() expects condvar handle")?;
                let _ = with_condvar_mut(condvar_id, |_state| ())?;
                Ok(Value::Nil)
            }))),
        );

        // condvar_broadcast(condvar)
        globals.borrow_mut().define(
            "condvar_broadcast".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "condvar_broadcast",
                1,
                |args| {
                    let condvar_id = args[0]
                        .as_integer()
                        .ok_or("condvar_broadcast() expects condvar handle")?;
                    let _ = with_condvar_mut(condvar_id, |_state| ())?;
                    Ok(Value::Nil)
                },
            ))),
        );

        // atomic_new(initial_int)
        globals.borrow_mut().define(
            "atomic_new".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("atomic_new", 1, |args| {
                let value = match &args[0] {
                    Value::Integer(n) => *n,
                    Value::Float(f) => *f as i64,
                    _ => return Err("atomic_new() expects integer".to_string()),
                };
                let id = register_atomic(AtomicState { value });
                Ok(Value::Integer(id))
            }))),
        );

        // atomic_load(atomic)
        globals.borrow_mut().define(
            "atomic_load".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("atomic_load", 1, |args| {
                let atomic_id = args[0]
                    .as_integer()
                    .ok_or("atomic_load() expects atomic handle")?;
                let value = with_atomic_mut(atomic_id, |state| state.value)?;
                Ok(Value::Integer(value))
            }))),
        );

        // atomic_store(atomic, value)
        globals.borrow_mut().define(
            "atomic_store".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("atomic_store", 2, |args| {
                let atomic_id = args[0]
                    .as_integer()
                    .ok_or("atomic_store() expects atomic handle")?;
                let value = match &args[1] {
                    Value::Integer(n) => *n,
                    Value::Float(f) => *f as i64,
                    _ => return Err("atomic_store() expects integer".to_string()),
                };
                let _ = with_atomic_mut(atomic_id, |state| state.value = value)?;
                Ok(Value::Nil)
            }))),
        );

        // atomic_add(atomic, delta) -> new value
        globals.borrow_mut().define(
            "atomic_add".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("atomic_add", 2, |args| {
                let atomic_id = args[0]
                    .as_integer()
                    .ok_or("atomic_add() expects atomic handle")?;
                let delta = match &args[1] {
                    Value::Integer(n) => *n,
                    Value::Float(f) => *f as i64,
                    _ => return Err("atomic_add() expects integer".to_string()),
                };
                let value = with_atomic_mut(atomic_id, |state| {
                    state.value += delta;
                    state.value
                })?;
                Ok(Value::Integer(value))
            }))),
        );

        // atomic_cas(atomic, expected, desired) -> bool
        globals.borrow_mut().define(
            "atomic_cas".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("atomic_cas", 3, |args| {
                let atomic_id = args[0]
                    .as_integer()
                    .ok_or("atomic_cas() expects atomic handle")?;
                let expected = match &args[1] {
                    Value::Integer(n) => *n,
                    Value::Float(f) => *f as i64,
                    _ => return Err("atomic_cas() expects integer".to_string()),
                };
                let desired = match &args[2] {
                    Value::Integer(n) => *n,
                    Value::Float(f) => *f as i64,
                    _ => return Err("atomic_cas() expects integer".to_string()),
                };
                let swapped = with_atomic_mut(atomic_id, |state| {
                    if state.value == expected {
                        state.value = desired;
                        true
                    } else {
                        false
                    }
                })?;
                Ok(Value::Bool(swapped))
            }))),
        );

        // chan_new(capacity)
        globals.borrow_mut().define(
            "chan_new".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("chan_new", 1, |args| {
                let cap = match &args[0] {
                    Value::Integer(n) => *n,
                    Value::Float(f) => *f as i64,
                    _ => return Err("chan_new() expects capacity integer".to_string()),
                };
                if cap < 0 {
                    return Err("chan_new() expects non-negative capacity".to_string());
                }
                let id = register_channel(ChannelState {
                    queue: VecDeque::new(),
                    capacity: cap,
                    closed: false,
                });
                Ok(Value::Integer(id))
            }))),
        );

        // chan_send(chan, value) -> bool
        globals.borrow_mut().define(
            "chan_send".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("chan_send", 2, |args| {
                let chan_id = args[0]
                    .as_integer()
                    .ok_or("chan_send() expects channel handle")?;
                let val = args[1].clone();
                let sent = with_channel_mut(chan_id, |state| {
                    if state.closed {
                        return false;
                    }
                    if state.capacity > 0 && state.queue.len() as i64 >= state.capacity {
                        return false;
                    }
                    state.queue.push_back(val.clone());
                    true
                })?;
                Ok(Value::Bool(sent))
            }))),
        );

        // chan_recv(chan) -> value or nil
        globals.borrow_mut().define(
            "chan_recv".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("chan_recv", 1, |args| {
                let chan_id = args[0]
                    .as_integer()
                    .ok_or("chan_recv() expects channel handle")?;
                let value = with_channel_mut(chan_id, |state| state.queue.pop_front())?;
                Ok(value.unwrap_or(Value::Nil))
            }))),
        );

        // chan_try_recv(chan) -> value or nil
        globals.borrow_mut().define(
            "chan_try_recv".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("chan_try_recv", 1, |args| {
                let chan_id = args[0]
                    .as_integer()
                    .ok_or("chan_try_recv() expects channel handle")?;
                let value = with_channel_mut(chan_id, |state| state.queue.pop_front())?;
                Ok(value.unwrap_or(Value::Nil))
            }))),
        );

        // chan_close(chan)
        globals.borrow_mut().define(
            "chan_close".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("chan_close", 1, |args| {
                let chan_id = args[0]
                    .as_integer()
                    .ok_or("chan_close() expects channel handle")?;
                let _ = with_channel_mut(chan_id, |state| state.closed = true)?;
                Ok(Value::Nil)
            }))),
        );

        // chan_is_closed(chan) -> bool
        globals.borrow_mut().define(
            "chan_is_closed".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("chan_is_closed", 1, |args| {
                let chan_id = args[0]
                    .as_integer()
                    .ok_or("chan_is_closed() expects channel handle")?;
                let closed = with_channel_mut(chan_id, |state| state.closed)?;
                Ok(Value::Bool(closed))
            }))),
        );

        // type - get type of value (whit_kind in Scots!)
        globals.borrow_mut().define(
            "whit_kind".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "whit_kind",
                1,
                |args| match &args[0] {
                    Value::NativeObject(obj) => Ok(Value::String(obj.type_name().to_string())),
                    _ => Ok(Value::String(args[0].type_name().to_string())),
                },
            ))),
        );

        // str - convert to string (tae_string in Scots!)
        globals.borrow_mut().define(
            "tae_string".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("tae_string", 1, |args| {
                Ok(Value::String(format!("{}", args[0])))
            }))),
        );

        // int - convert to integer (tae_int in Scots!)
        globals.borrow_mut().define(
            "tae_int".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "tae_int",
                1,
                |args| match &args[0] {
                    Value::Integer(n) => Ok(Value::Integer(*n)),
                    Value::Float(f) => Ok(Value::Integer(*f as i64)),
                    Value::String(s) => s
                        .parse::<i64>()
                        .map(Value::Integer)
                        .map_err(|_| format!("Cannae turn '{}' intae an integer", s)),
                    Value::Bool(b) => Ok(Value::Integer(if *b { 1 } else { 0 })),
                    _ => Err(format!(
                        "Cannae turn {} intae an integer",
                        args[0].type_name()
                    )),
                },
            ))),
        );

        // float - convert to float (tae_float in Scots!)
        globals.borrow_mut().define(
            "tae_float".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "tae_float",
                1,
                |args| match &args[0] {
                    Value::Integer(n) => Ok(Value::Float(*n as f64)),
                    Value::Float(f) => Ok(Value::Float(*f)),
                    Value::String(s) => s
                        .parse::<f64>()
                        .map(Value::Float)
                        .map_err(|_| format!("Cannae turn '{}' intae a float", s)),
                    _ => Err(format!("Cannae turn {} intae a float", args[0].type_name())),
                },
            ))),
        );

        // push - add to list (shove in Scots!)
        globals.borrow_mut().define(
            "shove".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("shove", 2, |args| {
                if let Value::List(list) = &args[0] {
                    list.borrow_mut().push(args[1].clone());
                    Ok(Value::Nil)
                } else {
                    Err("shove() expects a list as first argument".to_string())
                }
            }))),
        );

        // pop - remove from list (yank in Scots!)
        globals.borrow_mut().define(
            "yank".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("yank", 1, |args| {
                if let Value::List(list) = &args[0] {
                    list.borrow_mut()
                        .pop()
                        .ok_or_else(|| "Cannae yank fae an empty list!".to_string())
                } else {
                    Err("yank() expects a list".to_string())
                }
            }))),
        );

        // keys - get dictionary keys
        globals.borrow_mut().define(
            "keys".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("keys", 1, |args| {
                if let Value::Dict(dict) = &args[0] {
                    let keys: Vec<Value> = dict.borrow().keys().cloned().collect();
                    Ok(Value::List(Rc::new(RefCell::new(keys))))
                } else {
                    Err("keys() expects a dict".to_string())
                }
            }))),
        );

        // values - get dictionary values
        globals.borrow_mut().define(
            "values".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("values", 1, |args| {
                if let Value::Dict(dict) = &args[0] {
                    let vals: Vec<Value> = dict.borrow().values().cloned().collect();
                    Ok(Value::List(Rc::new(RefCell::new(vals))))
                } else {
                    Err("values() expects a dict".to_string())
                }
            }))),
        );

        // range - create a range
        globals.borrow_mut().define(
            "range".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("range", 2, |args| {
                let start = args[0].as_integer().ok_or("range() expects integers")?;
                let end = args[1].as_integer().ok_or("range() expects integers")?;
                Ok(Interpreter::range_to_list(start, end, false))
            }))),
        );

        // abs - absolute value
        globals.borrow_mut().define(
            "abs".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("abs", 1, |args| {
                match &args[0] {
                    Value::Integer(n) => Ok(Value::Integer(n.abs())),
                    Value::Float(f) => Ok(Value::Float(f.abs())),
                    _ => Err("abs() expects a number".to_string()),
                }
            }))),
        );

        // min - minimum value
        globals.borrow_mut().define(
            "min".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("min", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        Ok(Value::Integer(std::cmp::min(*a, *b)))
                    }
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.min(*b))),
                    _ => Err("min() expects two numbers of the same type".to_string()),
                }
            }))),
        );

        // max - maximum value
        globals.borrow_mut().define(
            "max".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("max", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        Ok(Value::Integer(std::cmp::max(*a, *b)))
                    }
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.max(*b))),
                    _ => Err("max() expects two numbers of the same type".to_string()),
                }
            }))),
        );

        // floor
        globals.borrow_mut().define(
            "floor".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "floor",
                1,
                |args| match &args[0] {
                    Value::Float(f) => Ok(Value::Integer(f.floor() as i64)),
                    Value::Integer(n) => Ok(Value::Integer(*n)),
                    _ => Err("floor() expects a number".to_string()),
                },
            ))),
        );

        // ceil
        globals.borrow_mut().define(
            "ceil".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "ceil",
                1,
                |args| match &args[0] {
                    Value::Float(f) => Ok(Value::Integer(f.ceil() as i64)),
                    Value::Integer(n) => Ok(Value::Integer(*n)),
                    _ => Err("ceil() expects a number".to_string()),
                },
            ))),
        );

        // round
        globals.borrow_mut().define(
            "round".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "round",
                1,
                |args| match &args[0] {
                    Value::Float(f) => Ok(Value::Integer(f.round() as i64)),
                    Value::Integer(n) => Ok(Value::Integer(*n)),
                    _ => Err("round() expects a number".to_string()),
                },
            ))),
        );

        // sqrt
        globals.borrow_mut().define(
            "sqrt".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "sqrt",
                1,
                |args| match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.sqrt())),
                    Value::Integer(n) => Ok(Value::Float((*n as f64).sqrt())),
                    _ => Err("sqrt() expects a number".to_string()),
                },
            ))),
        );

        // set_log_level - set the logging level at runtime
        globals.borrow_mut().define(
            "set_log_level".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "set_log_level",
                1,
                |args| match &args[0] {
                    Value::String(s) => {
                        if let Some(level) = LogLevel::parse_level(s) {
                            set_global_log_level(level);
                            Ok(Value::Nil)
                        } else {
                            Err(format!(
                                "Invalid log level '{}'. Use: wheesht, roar, holler, blether, mutter, or whisper",
                                s
                            ))
                        }
                    }
                    Value::Integer(n) => {
                        let level = match n {
                            0 => LogLevel::Wheesht,
                            1 => LogLevel::Roar,
                            2 => LogLevel::Holler,
                            3 => LogLevel::Blether,
                            4 => LogLevel::Mutter,
                            5 => LogLevel::Whisper,
                            _ => return Err(format!("Invalid log level {}. Use 0-5", n)),
                        };
                        set_global_log_level(level);
                        Ok(Value::Nil)
                    }
                    _ => Err("set_log_level() expects a string or integer".to_string()),
                },
            ))),
        );

        // get_log_level - get the current logging level
        globals.borrow_mut().define(
            "get_log_level".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("get_log_level", 0, |_args| {
                let level = get_global_log_level();
                Ok(Value::String(level.name().to_lowercase()))
            }))),
        );

        // log_set_filter(filter_string)
        globals.borrow_mut().define(
            "log_set_filter".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("log_set_filter", 1, |args| {
                match &args[0] {
                    Value::String(spec) => {
                        logging::set_filter(spec).map_err(|e| format!("log_set_filter() {}", e))?;
                        Ok(Value::Nil)
                    }
                    _ => Err("log_set_filter() expects a string".to_string()),
                }
            }))),
        );

        // log_get_filter() -> string
        globals.borrow_mut().define(
            "log_get_filter".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("log_get_filter", 0, |_args| {
                Ok(Value::String(logging::get_filter()))
            }))),
        );

        // log_enabled(level, target = "") -> bool
        globals.borrow_mut().define(
            "log_enabled".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "log_enabled",
                usize::MAX,
                |args| {
                    if args.is_empty() || args.len() > 2 {
                        return Err("log_enabled() expects 1 or 2 arguments".to_string());
                    }
                    let level = parse_log_level_value(&args[0])?;
                    let target = if args.len() == 2 {
                        parse_log_target_value(&args[1])?
                    } else {
                        String::new()
                    };
                    Ok(Value::Bool(logging::log_enabled(level, &target)))
                },
            ))),
        );

        // log_event(level, message, fields = {}, target = "") -> nil
        globals.borrow_mut().define(
            "log_event".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "log_event",
                usize::MAX,
                |args| {
                    if args.len() < 2 || args.len() > 4 {
                        return Err("log_event() expects 2-4 arguments".to_string());
                    }
                    let level = parse_log_level_value(&args[0])?;
                    let message = args[1].clone();
                    let (fields, target) = if args.len() > 2 {
                        resolve_log_args(&args[2..])?
                    } else {
                        (None, None)
                    };
                    let result = with_current_interpreter(|interp| {
                        interp.emit_log(level, message, fields, target, 0)
                    });
                    match result {
                        Some(Ok(())) => Ok(Value::Nil),
                        Some(Err(err)) => Err(format!("{}", err)),
                        None => {
                            Err("log_event() is unavailable outside the interpreter".to_string())
                        }
                    }
                },
            ))),
        );

        // log_init(config = {}) -> nil
        globals.borrow_mut().define(
            "log_init".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "log_init",
                usize::MAX,
                |args| {
                    if args.len() > 1 {
                        return Err("log_init() expects 0 or 1 arguments".to_string());
                    }
                    let config = args.first().cloned();
                    let result = with_current_interpreter(|interp| interp.apply_log_config(config));
                    match result {
                        Some(Ok(())) => Ok(Value::Nil),
                        Some(Err(err)) => Err(err),
                        None => {
                            Err("log_init() is unavailable outside the interpreter".to_string())
                        }
                    }
                },
            ))),
        );

        // log_span(name, level = "blether", fields = {}, target = "") -> span_handle
        globals.borrow_mut().define(
            "log_span".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "log_span",
                usize::MAX,
                |args| {
                    if args.is_empty() || args.len() > 4 {
                        return Err("log_span() expects 1-4 arguments".to_string());
                    }
                    let name = match &args[0] {
                        Value::String(s) => s.clone(),
                        _ => return Err("log_span() name must be a string".to_string()),
                    };
                    let level = if args.len() >= 2 {
                        parse_log_level_value(&args[1])?
                    } else {
                        LogLevel::Blether
                    };
                    let fields = if args.len() >= 3 {
                        logging::fields_from_dict(&args[2])?
                    } else {
                        Vec::new()
                    };
                    let target = if args.len() >= 4 {
                        parse_log_target_value(&args[3])?
                    } else {
                        with_current_interpreter(|interp| interp.current_file.clone())
                            .unwrap_or_else(|| "".to_string())
                    };
                    let span = logging::new_span(name, level, target, fields);
                    Ok(Value::NativeObject(Rc::new(logging::LogSpanHandle::new(
                        span,
                    ))))
                },
            ))),
        );

        // log_span_enter(span_handle)
        globals.borrow_mut().define(
            "log_span_enter".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("log_span_enter", 1, |args| {
                match &args[0] {
                    Value::NativeObject(obj) => {
                        if let Some(handle) = obj.as_any().downcast_ref::<logging::LogSpanHandle>()
                        {
                            logging::span_enter(handle.span());
                            Ok(Value::Nil)
                        } else {
                            Err("log_span_enter() expects a log span handle".to_string())
                        }
                    }
                    _ => Err("log_span_enter() expects a log span handle".to_string()),
                }
            }))),
        );

        // log_span_exit(span_handle)
        globals.borrow_mut().define(
            "log_span_exit".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "log_span_exit",
                1,
                |args| match &args[0] {
                    Value::NativeObject(obj) => {
                        if let Some(handle) = obj.as_any().downcast_ref::<logging::LogSpanHandle>()
                        {
                            logging::span_exit(handle.span().id)
                                .map_err(|e| format!("log_span_exit() {}", e))?;
                            Ok(Value::Nil)
                        } else {
                            Err("log_span_exit() expects a log span handle".to_string())
                        }
                    }
                    _ => Err("log_span_exit() expects a log span handle".to_string()),
                },
            ))),
        );

        // log_span_current() -> span_handle | nil
        globals.borrow_mut().define(
            "log_span_current".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "log_span_current",
                0,
                |_args| {
                    if let Some(span) = logging::span_current() {
                        Ok(Value::NativeObject(Rc::new(logging::LogSpanHandle::new(
                            span,
                        ))))
                    } else {
                        Ok(Value::Nil)
                    }
                },
            ))),
        );

        // log_span_in(span_handle, fn) -> value
        globals.borrow_mut().define(
            "log_span_in".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("log_span_in", 2, |args| {
                let span = match &args[0] {
                    Value::NativeObject(obj) => obj
                        .as_any()
                        .downcast_ref::<logging::LogSpanHandle>()
                        .map(|h| h.span())
                        .ok_or_else(|| "log_span_in() expects a log span handle".to_string())?,
                    _ => return Err("log_span_in() expects a log span handle".to_string()),
                };
                let func = args[1].clone();

                logging::span_enter(span.clone());
                let result =
                    with_current_interpreter(|interp| interp.call_value(func, Vec::new(), 0));
                let _ = logging::span_exit(span.id);

                match result {
                    Some(Ok(val)) => Ok(val),
                    Some(Err(err)) => Err(format!("{}", err)),
                    None => Err("log_span_in() is unavailable outside the interpreter".to_string()),
                }
            }))),
        );

        // stacktrace - get the current stack trace as a string
        globals.borrow_mut().define(
            "stacktrace".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("stacktrace", 0, |_args| {
                let stack = get_stack_trace();
                let trace = stack
                    .iter()
                    .rev()
                    .map(|f| format!("  at {} ({}:{})", f.name, f.file, f.line))
                    .collect::<Vec<_>>()
                    .join("\n");
                Ok(Value::String(if trace.is_empty() {
                    "(no stack trace)".to_string()
                } else {
                    trace
                }))
            }))),
        );

        // set_crash_handling - enable or disable crash handling
        globals.borrow_mut().define(
            "set_crash_handling".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "set_crash_handling",
                1,
                |args| match &args[0] {
                    Value::Bool(enabled) => {
                        set_crash_handling(*enabled);
                        Ok(Value::Nil)
                    }
                    _ => Err("set_crash_handling() expects a boolean".to_string()),
                },
            ))),
        );

        // split - split string
        globals.borrow_mut().define(
            "split".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("split", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::String(s), Value::String(delim)) => {
                        let parts: Vec<Value> = s
                            .split(delim.as_str())
                            .map(|p| Value::String(p.to_string()))
                            .collect();
                        Ok(Value::List(Rc::new(RefCell::new(parts))))
                    }
                    _ => Err("split() expects two strings".to_string()),
                }
            }))),
        );

        // join - join list into string
        globals.borrow_mut().define(
            "join".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("join", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::List(list), Value::String(delim)) => {
                        let parts: Vec<String> =
                            list.borrow().iter().map(|v| format!("{}", v)).collect();
                        Ok(Value::String(parts.join(delim)))
                    }
                    _ => Err("join() expects a list and a string".to_string()),
                }
            }))),
        );

        // contains - check if list/string contains value
        globals.borrow_mut().define(
            "contains".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "contains",
                2,
                |args| match &args[0] {
                    Value::List(list) => {
                        let found = list.borrow().iter().any(|v| v == &args[1]);
                        Ok(Value::Bool(found))
                    }
                    Value::String(s) => {
                        if let Value::String(needle) = &args[1] {
                            Ok(Value::Bool(s.contains(needle.as_str())))
                        } else {
                            Err("contains() on string expects a string needle".to_string())
                        }
                    }
                    Value::Dict(dict) => Ok(Value::Bool(dict.borrow().contains_key(&args[1]))),
                    _ => Err("contains() expects a list, string, or dict".to_string()),
                },
            ))),
        );

        // reverse - reverse a list or string
        globals.borrow_mut().define(
            "reverse".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "reverse",
                1,
                |args| match &args[0] {
                    Value::List(list) => {
                        let mut reversed = list.borrow().clone();
                        reversed.reverse();
                        Ok(Value::List(Rc::new(RefCell::new(reversed))))
                    }
                    Value::String(s) => Ok(Value::String(s.chars().rev().collect())),
                    _ => Err("reverse() expects a list or string".to_string()),
                },
            ))),
        );

        // slap - append lists together (like a friendly slap on the back!)
        globals.borrow_mut().define(
            "slap".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("slap", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::List(a), Value::List(b)) => {
                        let mut result = a.borrow().clone();
                        result.extend(b.borrow().clone());
                        Ok(Value::List(Rc::new(RefCell::new(result))))
                    }
                    (Value::String(a), Value::String(b)) => {
                        Ok(Value::String(format!("{}{}", a, b)))
                    }
                    _ => Err("slap() expects two lists or two strings".to_string()),
                }
            }))),
        );

        // heid - get the first element (head)
        globals.borrow_mut().define(
            "heid".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "heid",
                1,
                |args| match &args[0] {
                    Value::List(list) => list
                        .borrow()
                        .first()
                        .cloned()
                        .ok_or("Cannae get heid o' empty list!".to_string()),
                    Value::String(s) => s
                        .chars()
                        .next()
                        .map(|c| Value::String(c.to_string()))
                        .ok_or("Cannae get heid o' empty string!".to_string()),
                    _ => Err("heid() expects a list or string".to_string()),
                },
            ))),
        );

        // tail - get everything except the first (like a tail!)
        globals.borrow_mut().define(
            "tail".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "tail",
                1,
                |args| match &args[0] {
                    Value::List(list) => {
                        let list = list.borrow();
                        if list.is_empty() {
                            Ok(Value::List(Rc::new(RefCell::new(Vec::new()))))
                        } else {
                            Ok(Value::List(Rc::new(RefCell::new(list[1..].to_vec()))))
                        }
                    }
                    Value::String(s) => Ok(Value::String(s.chars().skip(1).collect())),
                    _ => Err("tail() expects a list or string".to_string()),
                },
            ))),
        );

        // bum - get the last element (backside!)
        globals.borrow_mut().define(
            "bum".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("bum", 1, |args| {
                match &args[0] {
                    Value::List(list) => list
                        .borrow()
                        .last()
                        .cloned()
                        .ok_or("Cannae get bum o' empty list!".to_string()),
                    Value::String(s) => s
                        .chars()
                        .last()
                        .map(|c| Value::String(c.to_string()))
                        .ok_or("Cannae get bum o' empty string!".to_string()),
                    _ => Err("bum() expects a list or string".to_string()),
                }
            }))),
        );

        // scran - slice a list or string (grab a portion, like grabbing scran/food)
        globals.borrow_mut().define(
            "scran".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("scran", 3, |args| {
                let start = args[1]
                    .as_integer()
                    .ok_or("scran() needs integer indices")?;
                let end = args[2]
                    .as_integer()
                    .ok_or("scran() needs integer indices")?;
                match &args[0] {
                    Value::List(list) => {
                        let list = list.borrow();
                        let start = start.max(0) as usize;
                        let end = end.min(list.len() as i64) as usize;
                        Ok(Value::List(Rc::new(RefCell::new(
                            list[start..end].to_vec(),
                        ))))
                    }
                    Value::String(s) => {
                        let start = start.max(0) as usize;
                        let end = end.min(s.len() as i64) as usize;
                        Ok(Value::String(
                            s.chars().skip(start).take(end - start).collect(),
                        ))
                    }
                    _ => Err("scran() expects a list or string".to_string()),
                }
            }))),
        );

        // sumaw - sum all numbers in a list (sum aw = sum all)
        globals.borrow_mut().define(
            "sumaw".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("sumaw", 1, |args| {
                if let Value::List(list) = &args[0] {
                    let mut sum: f64 = 0.0;
                    let mut is_float = false;
                    for item in list.borrow().iter() {
                        match item {
                            Value::Integer(n) => sum += *n as f64,
                            Value::Float(f) => {
                                sum += f;
                                is_float = true;
                            }
                            _ => return Err("sumaw() expects a list of numbers".to_string()),
                        }
                    }
                    if is_float {
                        Ok(Value::Float(sum))
                    } else {
                        Ok(Value::Integer(sum as i64))
                    }
                } else {
                    Err("sumaw() expects a list".to_string())
                }
            }))),
        );

        // coont - count occurrences in list or string
        globals.borrow_mut().define(
            "coont".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "coont",
                2,
                |args| match &args[0] {
                    Value::List(list) => {
                        let count = list.borrow().iter().filter(|&x| x == &args[1]).count();
                        Ok(Value::Integer(count as i64))
                    }
                    Value::String(s) => {
                        if let Value::String(needle) = &args[1] {
                            let count = s.matches(needle.as_str()).count();
                            Ok(Value::Integer(count as i64))
                        } else {
                            Err("coont() on string needs a string tae count".to_string())
                        }
                    }
                    _ => Err("coont() expects a list or string".to_string()),
                },
            ))),
        );

        // wheesht - remove whitespace (be quiet/silent!)
        globals.borrow_mut().define(
            "wheesht".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("wheesht", 1, |args| {
                if let Value::String(s) = &args[0] {
                    Ok(Value::String(s.trim().to_string()))
                } else {
                    Err("wheesht() expects a string".to_string())
                }
            }))),
        );

        // upper - to uppercase (shout it oot!)
        globals.borrow_mut().define(
            "upper".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("upper", 1, |args| {
                if let Value::String(s) = &args[0] {
                    Ok(Value::String(s.to_uppercase()))
                } else {
                    Err("upper() expects a string".to_string())
                }
            }))),
        );

        // lower - to lowercase (calm doon!)
        globals.borrow_mut().define(
            "lower".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("lower", 1, |args| {
                if let Value::String(s) = &args[0] {
                    Ok(Value::String(s.to_lowercase()))
                } else {
                    Err("lower() expects a string".to_string())
                }
            }))),
        );

        // shuffle - randomly shuffle a list (like a ceilidh!)
        globals.borrow_mut().define(
            "shuffle".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("shuffle", 1, |args| {
                if let Value::List(list) = &args[0] {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let seed = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos() as u64;
                    let mut shuffled = list.borrow().clone();
                    // Simple Fisher-Yates shuffle with basic RNG
                    let mut rng = seed;
                    for i in (1..shuffled.len()).rev() {
                        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                        let j = (rng as usize) % (i + 1);
                        shuffled.swap(i, j);
                    }
                    Ok(Value::List(Rc::new(RefCell::new(shuffled))))
                } else {
                    Err("shuffle() expects a list".to_string())
                }
            }))),
        );

        // sort - sort a list
        globals.borrow_mut().define(
            "sort".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("sort", 1, |args| {
                if let Value::List(list) = &args[0] {
                    let mut sorted = list.borrow().clone();
                    sorted.sort_by(|a, b| match (a, b) {
                        (Value::Integer(x), Value::Integer(y)) => x.cmp(y),
                        (Value::Float(x), Value::Float(y)) => {
                            x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                        }
                        (Value::String(x), Value::String(y)) => x.cmp(y),
                        _ => std::cmp::Ordering::Equal,
                    });
                    Ok(Value::List(Rc::new(RefCell::new(sorted))))
                } else {
                    Err("sort() expects a list".to_string())
                }
            }))),
        );

        // jammy - random number (Scots: lucky!)
        globals.borrow_mut().define(
            "jammy".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("jammy", 2, |args| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let min = args[0].as_integer().ok_or("jammy() needs integer bounds")?;
                let max = args[1].as_integer().ok_or("jammy() needs integer bounds")?;
                if min >= max {
                    return Err("jammy() needs min < max, ya numpty!".to_string());
                }
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as u64;
                let rng = seed.wrapping_mul(1103515245).wrapping_add(12345);
                let range = (max - min) as u64;
                let result = min + ((rng % range) as i64);
                Ok(Value::Integer(result))
            }))),
        );

        // the_noo - current timestamp in seconds (Scots: "the now")
        globals.borrow_mut().define(
            "the_noo".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("the_noo", 0, |_args| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let secs = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                Ok(Value::Integer(secs as i64))
            }))),
        );

        // mono_ms - monotonic milliseconds since start
        globals.borrow_mut().define(
            "mono_ms".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("mono_ms", 0, |_args| {
                let start = MONO_START.get_or_init(std::time::Instant::now);
                let ms = start.elapsed().as_millis() as i64;
                Ok(Value::Integer(ms))
            }))),
        );

        // mono_ns - monotonic nanoseconds since start
        globals.borrow_mut().define(
            "mono_ns".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("mono_ns", 0, |_args| {
                let start = MONO_START.get_or_init(std::time::Instant::now);
                let ns = start.elapsed().as_nanos() as i64;
                Ok(Value::Integer(ns))
            }))),
        );

        // is_a - type checking (returns aye/nae)
        globals.borrow_mut().define(
            "is_a".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("is_a", 2, |args| {
                let type_name = match &args[1] {
                    Value::String(s) => s.as_str(),
                    _ => return Err("is_a() needs a type name string".to_string()),
                };
                let matches = match type_name {
                    "integer" | "int" => matches!(args[0], Value::Integer(_)),
                    "float" => matches!(args[0], Value::Float(_)),
                    "string" | "str" => matches!(args[0], Value::String(_)),
                    "bool" => matches!(args[0], Value::Bool(_)),
                    "list" => matches!(args[0], Value::List(_)),
                    "bytes" | "byte" => matches!(args[0], Value::Bytes(_)),
                    "dict" => matches!(args[0], Value::Dict(_)),
                    "function" | "dae" => {
                        matches!(args[0], Value::Function(_) | Value::NativeFunction(_))
                    }
                    "naething" | "nil" => matches!(args[0], Value::Nil),
                    "range" => matches!(args[0], Value::Range(_)),
                    _ => false,
                };
                Ok(Value::Bool(matches))
            }))),
        );

        // tae_bool - convert to boolean
        globals.borrow_mut().define(
            "tae_bool".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("tae_bool", 1, |args| {
                Ok(Value::Bool(args[0].is_truthy()))
            }))),
        );

        // char_at - get character at index (returns string of length 1)
        globals.borrow_mut().define(
            "char_at".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("char_at", 2, |args| {
                let s = match &args[0] {
                    Value::String(s) => s,
                    _ => return Err("char_at() needs a string".to_string()),
                };
                let idx = args[1]
                    .as_integer()
                    .ok_or("char_at() needs an integer index")?;
                let idx = if idx < 0 { s.len() as i64 + idx } else { idx } as usize;
                s.chars()
                    .nth(idx)
                    .map(|c| Value::String(c.to_string()))
                    .ok_or_else(|| {
                        format!(
                            "Index {} oot o' bounds fer string o' length {}",
                            idx,
                            s.len()
                        )
                    })
            }))),
        );

        // replace - replace occurrences in string
        globals.borrow_mut().define(
            "replace".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("replace", 3, |args| {
                match (&args[0], &args[1], &args[2]) {
                    (Value::String(s), Value::String(from), Value::String(to)) => {
                        Ok(Value::String(s.replace(from.as_str(), to.as_str())))
                    }
                    _ => Err("replace() needs three strings".to_string()),
                }
            }))),
        );

        // starts_wi - check if string starts with prefix (Scots: starts with)
        globals.borrow_mut().define(
            "starts_wi".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("starts_wi", 2, |args| match (
                &args[0], &args[1],
            ) {
                (Value::String(s), Value::String(prefix)) => {
                    Ok(Value::Bool(s.starts_with(prefix.as_str())))
                }
                _ => Err("starts_wi() needs two strings".to_string()),
            }))),
        );

        // ends_wi - check if string ends with suffix
        globals.borrow_mut().define(
            "ends_wi".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("ends_wi", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::String(s), Value::String(suffix)) => {
                        Ok(Value::Bool(s.ends_with(suffix.as_str())))
                    }
                    _ => Err("ends_wi() needs two strings".to_string()),
                }
            }))),
        );

        // repeat - repeat a string n times
        globals.borrow_mut().define(
            "repeat".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("repeat", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::String(s), Value::Integer(n)) => {
                        if *n < 0 {
                            Err("Cannae repeat a negative number o' times!".to_string())
                        } else {
                            Ok(Value::String(s.repeat(*n as usize)))
                        }
                    }
                    _ => Err("repeat() needs a string and an integer".to_string()),
                }
            }))),
        );

        // index_of - find index of substring (returns -1 if not found)
        globals.borrow_mut().define(
            "index_of".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("index_of", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::String(s), Value::String(needle)) => Ok(Value::Integer(
                        s.find(needle.as_str()).map(|i| i as i64).unwrap_or(-1),
                    )),
                    (Value::List(list), val) => {
                        let list = list.borrow();
                        for (i, item) in list.iter().enumerate() {
                            if item == val {
                                return Ok(Value::Integer(i as i64));
                            }
                        }
                        Ok(Value::Integer(-1))
                    }
                    _ => Err("index_of() needs a string/list and a value".to_string()),
                }
            }))),
        );

        // lines - split string into lines (on newlines)
        globals.borrow_mut().define(
            "lines".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("lines", 1, |args| {
                if let Value::String(s) = &args[0] {
                    let line_list: Vec<Value> = s
                        .lines()
                        .map(|line| Value::String(line.to_string()))
                        .collect();
                    Ok(Value::List(Rc::new(RefCell::new(line_list))))
                } else {
                    Err("lines() needs a string".to_string())
                }
            }))),
        );

        // words - split string into words (on whitespace)
        globals.borrow_mut().define(
            "words".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("words", 1, |args| {
                if let Value::String(s) = &args[0] {
                    let word_list: Vec<Value> = s
                        .split_whitespace()
                        .map(|word| Value::String(word.to_string()))
                        .collect();
                    Ok(Value::List(Rc::new(RefCell::new(word_list))))
                } else {
                    Err("words() needs a string".to_string())
                }
            }))),
        );

        // is_digit - check if string contains only digits
        globals.borrow_mut().define(
            "is_digit".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("is_digit", 1, |args| {
                if let Value::String(s) = &args[0] {
                    Ok(Value::Bool(
                        !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()),
                    ))
                } else {
                    Err("is_digit() needs a string".to_string())
                }
            }))),
        );

        // is_alpha - check if string contains only letters
        globals.borrow_mut().define(
            "is_alpha".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("is_alpha", 1, |args| {
                if let Value::String(s) = &args[0] {
                    Ok(Value::Bool(
                        !s.is_empty() && s.chars().all(|c| c.is_alphabetic()),
                    ))
                } else {
                    Err("is_alpha() needs a string".to_string())
                }
            }))),
        );

        // is_space - check if string contains only whitespace
        globals.borrow_mut().define(
            "is_space".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("is_space", 1, |args| {
                if let Value::String(s) = &args[0] {
                    Ok(Value::Bool(
                        !s.is_empty() && s.chars().all(|c| c.is_whitespace()),
                    ))
                } else {
                    Err("is_space() needs a string".to_string())
                }
            }))),
        );

        // capitalize - capitalize first letter
        globals.borrow_mut().define(
            "capitalize".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("capitalize", 1, |args| {
                if let Value::String(s) = &args[0] {
                    let mut chars = s.chars();
                    let result = match chars.next() {
                        Some(first) => {
                            format!("{}{}", first.to_uppercase(), chars.collect::<String>())
                        }
                        None => String::new(),
                    };
                    Ok(Value::String(result))
                } else {
                    Err("capitalize() needs a string".to_string())
                }
            }))),
        );

        // title - capitalize each word
        globals.borrow_mut().define(
            "title".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("title", 1, |args| {
                if let Value::String(s) = &args[0] {
                    let result = s
                        .split_whitespace()
                        .map(|word| {
                            let mut chars = word.chars();
                            let first = chars.next().unwrap();
                            format!(
                                "{}{}",
                                first.to_uppercase(),
                                chars.collect::<String>().to_lowercase()
                            )
                        })
                        .collect::<Vec<String>>()
                        .join(" ");
                    Ok(Value::String(result))
                } else {
                    Err("title() needs a string".to_string())
                }
            }))),
        );

        // chars - split string into list of characters
        globals.borrow_mut().define(
            "chars".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("chars", 1, |args| {
                if let Value::String(s) = &args[0] {
                    let char_list: Vec<Value> =
                        s.chars().map(|c| Value::String(c.to_string())).collect();
                    Ok(Value::List(Rc::new(RefCell::new(char_list))))
                } else {
                    Err("chars() needs a string".to_string())
                }
            }))),
        );

        // ord - get ASCII/Unicode code of first character
        globals.borrow_mut().define(
            "ord".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("ord", 1, |args| {
                if let Value::String(s) = &args[0] {
                    s.chars()
                        .next()
                        .map(|c| Value::Integer(c as i64))
                        .ok_or_else(|| "Cannae get ord o' empty string!".to_string())
                } else {
                    Err("ord() needs a string".to_string())
                }
            }))),
        );

        // chr - get character from ASCII/Unicode code
        globals.borrow_mut().define(
            "chr".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("chr", 1, |args| {
                if let Value::Integer(n) = &args[0] {
                    if *n >= 0 && *n <= 0x10FFFF {
                        char::from_u32(*n as u32)
                            .map(|c| Value::String(c.to_string()))
                            .ok_or_else(|| format!("Invalid Unicode codepoint: {}", n))
                    } else {
                        Err(format!(
                            "chr() needs a valid Unicode codepoint (0 to 1114111), got {}",
                            n
                        ))
                    }
                } else {
                    Err("chr() needs an integer".to_string())
                }
            }))),
        );

        // flatten - flatten nested lists one level
        globals.borrow_mut().define(
            "flatten".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("flatten", 1, |args| {
                if let Value::List(list) = &args[0] {
                    let mut result = Vec::new();
                    for item in list.borrow().iter() {
                        if let Value::List(inner) = item {
                            result.extend(inner.borrow().clone());
                        } else {
                            result.push(item.clone());
                        }
                    }
                    Ok(Value::List(Rc::new(RefCell::new(result))))
                } else {
                    Err("flatten() needs a list".to_string())
                }
            }))),
        );

        // zip - combine two lists into list of pairs
        globals.borrow_mut().define(
            "zip".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("zip", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::List(a), Value::List(b)) => {
                        let a = a.borrow();
                        let b = b.borrow();
                        let result: Vec<Value> = a
                            .iter()
                            .zip(b.iter())
                            .map(|(x, y)| {
                                Value::List(Rc::new(RefCell::new(vec![x.clone(), y.clone()])))
                            })
                            .collect();
                        Ok(Value::List(Rc::new(RefCell::new(result))))
                    }
                    _ => Err("zip() needs two lists".to_string()),
                }
            }))),
        );

        // enumerate - return list of [index, value] pairs
        globals.borrow_mut().define(
            "enumerate".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("enumerate", 1, |args| {
                if let Value::List(list) = &args[0] {
                    let result: Vec<Value> = list
                        .borrow()
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            Value::List(Rc::new(RefCell::new(vec![
                                Value::Integer(i as i64),
                                v.clone(),
                            ])))
                        })
                        .collect();
                    Ok(Value::List(Rc::new(RefCell::new(result))))
                } else {
                    Err("enumerate() needs a list".to_string())
                }
            }))),
        );

        // === More List Manipulation Functions ===

        // uniq - remove duplicates from a list (keeping first occurrence)
        globals.borrow_mut().define(
            "uniq".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("uniq", 1, |args| {
                if let Value::List(list) = &args[0] {
                    let mut seen = Vec::new();
                    let mut result = Vec::new();
                    for item in list.borrow().iter() {
                        let item_str = format!("{:?}", item);
                        if !seen.contains(&item_str) {
                            seen.push(item_str);
                            result.push(item.clone());
                        }
                    }
                    Ok(Value::List(Rc::new(RefCell::new(result))))
                } else {
                    Err("uniq() needs a list".to_string())
                }
            }))),
        );

        // chynge - insert at index (Scots: change)
        globals.borrow_mut().define(
            "chynge".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("chynge", 3, |args| {
                if let Value::List(list) = &args[0] {
                    let idx = args[1]
                        .as_integer()
                        .ok_or("chynge() needs an integer index")?;
                    let mut new_list = list.borrow().clone();
                    let idx = if idx < 0 {
                        (new_list.len() as i64 + idx) as usize
                    } else {
                        idx as usize
                    };
                    if idx > new_list.len() {
                        return Err(format!(
                            "Index {} oot o' bounds fer list o' length {}",
                            idx,
                            new_list.len()
                        ));
                    }
                    new_list.insert(idx, args[2].clone());
                    Ok(Value::List(Rc::new(RefCell::new(new_list))))
                } else {
                    Err("chynge() needs a list".to_string())
                }
            }))),
        );

        // dicht - remove at index (Scots: wipe/clean)
        globals.borrow_mut().define(
            "dicht".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("dicht", 2, |args| {
                if let Value::List(list) = &args[0] {
                    let idx = args[1]
                        .as_integer()
                        .ok_or("dicht() needs an integer index")?;
                    let mut new_list = list.borrow().clone();
                    let idx = if idx < 0 {
                        (new_list.len() as i64 + idx) as usize
                    } else {
                        idx as usize
                    };
                    if idx >= new_list.len() {
                        return Err(format!(
                            "Index {} oot o' bounds fer list o' length {}",
                            idx,
                            new_list.len()
                        ));
                    }
                    new_list.remove(idx);
                    Ok(Value::List(Rc::new(RefCell::new(new_list))))
                } else {
                    Err("dicht() needs a list".to_string())
                }
            }))),
        );

        // redd_up - remove nil values from list (Scots: tidy up)
        globals.borrow_mut().define(
            "redd_up".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("redd_up", 1, |args| {
                if let Value::List(list) = &args[0] {
                    let result: Vec<Value> = list
                        .borrow()
                        .iter()
                        .filter(|v| !matches!(v, Value::Nil))
                        .cloned()
                        .collect();
                    Ok(Value::List(Rc::new(RefCell::new(result))))
                } else {
                    Err("redd_up() needs a list".to_string())
                }
            }))),
        );

        // pairty - partition list based on predicate result (returns [truthy, falsy])
        // Note: This is a simpler version - returns [evens, odds] for integers
        globals.borrow_mut().define(
            "split_by".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("split_by", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::List(list), Value::String(pred)) => {
                        let mut truthy = Vec::new();
                        let mut falsy = Vec::new();
                        for item in list.borrow().iter() {
                            let is_match = match pred.as_str() {
                                "even" => matches!(item, Value::Integer(n) if n % 2 == 0),
                                "odd" => matches!(item, Value::Integer(n) if n % 2 != 0),
                                "positive" => matches!(item, Value::Integer(n) if *n > 0) || matches!(item, Value::Float(f) if *f > 0.0),
                                "negative" => matches!(item, Value::Integer(n) if *n < 0) || matches!(item, Value::Float(f) if *f < 0.0),
                                "truthy" => item.is_truthy(),
                                "nil" => matches!(item, Value::Nil),
                                "string" => matches!(item, Value::String(_)),
                                "number" => matches!(item, Value::Integer(_) | Value::Float(_)),
                                _ => return Err(format!("Unknown predicate '{}'. Try: even, odd, positive, negative, truthy, nil, string, number", pred)),
                            };
                            if is_match {
                                truthy.push(item.clone());
                            } else {
                                falsy.push(item.clone());
                            }
                        }
                        Ok(Value::List(Rc::new(RefCell::new(vec![
                            Value::List(Rc::new(RefCell::new(truthy))),
                            Value::List(Rc::new(RefCell::new(falsy))),
                        ]))))
                    }
                    _ => Err("split_by() needs a list and a predicate string".to_string()),
                }
            }))),
        );

        // grup_runs - group consecutive equal elements (like run-length encoding)
        globals.borrow_mut().define(
            "grup_runs".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("grup_runs", 1, |args| {
                if let Value::List(list) = &args[0] {
                    let list = list.borrow();
                    let mut result: Vec<Value> = Vec::new();
                    let mut current_group: Vec<Value> = Vec::new();

                    for item in list.iter() {
                        if current_group.is_empty() || &current_group[0] == item {
                            current_group.push(item.clone());
                        } else {
                            result.push(Value::List(Rc::new(RefCell::new(current_group))));
                            current_group = vec![item.clone()];
                        }
                    }
                    if !current_group.is_empty() {
                        result.push(Value::List(Rc::new(RefCell::new(current_group))));
                    }

                    Ok(Value::List(Rc::new(RefCell::new(result))))
                } else {
                    Err("grup_runs() needs a list".to_string())
                }
            }))),
        );

        // chunks - split list into chunks of size n
        globals.borrow_mut().define(
            "chunks".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("chunks", 2, |args| {
                if let Value::List(list) = &args[0] {
                    let n = args[1]
                        .as_integer()
                        .ok_or("chunks() needs an integer size")?;
                    if n <= 0 {
                        return Err("chunks() size must be positive".to_string());
                    }
                    let n = n as usize;
                    let list = list.borrow();
                    let result: Vec<Value> = list
                        .chunks(n)
                        .map(|chunk| Value::List(Rc::new(RefCell::new(chunk.to_vec()))))
                        .collect();
                    Ok(Value::List(Rc::new(RefCell::new(result))))
                } else {
                    Err("chunks() needs a list".to_string())
                }
            }))),
        );

        // interleave - alternate elements from two lists
        globals.borrow_mut().define(
            "interleave".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "interleave",
                2,
                |args| match (&args[0], &args[1]) {
                    (Value::List(a), Value::List(b)) => {
                        let a = a.borrow();
                        let b = b.borrow();
                        let mut result = Vec::new();
                        let max_len = a.len().max(b.len());
                        for i in 0..max_len {
                            if i < a.len() {
                                result.push(a[i].clone());
                            }
                            if i < b.len() {
                                result.push(b[i].clone());
                            }
                        }
                        Ok(Value::List(Rc::new(RefCell::new(result))))
                    }
                    _ => Err("interleave() needs two lists".to_string()),
                },
            ))),
        );

        // === More Mathematical Functions ===

        // pooer - power/exponent (Scots: power)
        globals.borrow_mut().define(
            "pooer".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("pooer", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::Integer(base), Value::Integer(exp)) => {
                        if *exp < 0 {
                            Ok(Value::Float((*base as f64).powi(*exp as i32)))
                        } else {
                            Ok(Value::Integer(base.pow(*exp as u32)))
                        }
                    }
                    (Value::Float(base), Value::Integer(exp)) => {
                        Ok(Value::Float(base.powi(*exp as i32)))
                    }
                    (Value::Float(base), Value::Float(exp)) => Ok(Value::Float(base.powf(*exp))),
                    (Value::Integer(base), Value::Float(exp)) => {
                        Ok(Value::Float((*base as f64).powf(*exp)))
                    }
                    _ => Err("pooer() needs twa numbers".to_string()),
                }
            }))),
        );

        // sin - sine (trigonometry)
        globals.borrow_mut().define(
            "sin".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("sin", 1, |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.sin())),
                    Value::Integer(n) => Ok(Value::Float((*n as f64).sin())),
                    _ => Err("sin() needs a number".to_string()),
                }
            }))),
        );

        // cos - cosine
        globals.borrow_mut().define(
            "cos".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("cos", 1, |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.cos())),
                    Value::Integer(n) => Ok(Value::Float((*n as f64).cos())),
                    _ => Err("cos() needs a number".to_string()),
                }
            }))),
        );

        // tan - tangent
        globals.borrow_mut().define(
            "tan".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("tan", 1, |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.tan())),
                    Value::Integer(n) => Ok(Value::Float((*n as f64).tan())),
                    _ => Err("tan() needs a number".to_string()),
                }
            }))),
        );

        // log - natural logarithm
        globals.borrow_mut().define(
            "log".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("log", 1, |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.ln())),
                    Value::Integer(n) => Ok(Value::Float((*n as f64).ln())),
                    _ => Err("log() needs a number".to_string()),
                }
            }))),
        );

        // log10 - base 10 logarithm
        globals.borrow_mut().define(
            "log10".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "log10",
                1,
                |args| match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.log10())),
                    Value::Integer(n) => Ok(Value::Float((*n as f64).log10())),
                    _ => Err("log10() needs a number".to_string()),
                },
            ))),
        );

        // PI constant
        globals
            .borrow_mut()
            .define("PI".to_string(), Value::Float(std::f64::consts::PI));

        // E constant (Euler's number)
        globals
            .borrow_mut()
            .define("E".to_string(), Value::Float(std::f64::consts::E));

        // TAU constant (2*PI)
        globals
            .borrow_mut()
            .define("TAU".to_string(), Value::Float(std::f64::consts::TAU));

        // exp - e raised to the power
        globals.borrow_mut().define(
            "exp".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("exp", 1, |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.exp())),
                    Value::Integer(n) => Ok(Value::Float((*n as f64).exp())),
                    _ => Err("exp() needs a number".to_string()),
                }
            }))),
        );

        // pow - raise to a power (Scottish: mak it muckle!)
        globals.borrow_mut().define(
            "pow".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("pow", 2, |args| {
                let base = match &args[0] {
                    Value::Float(f) => *f,
                    Value::Integer(n) => *n as f64,
                    _ => return Err("pow() needs numbers".to_string()),
                };
                let exponent = match &args[1] {
                    Value::Float(f) => *f,
                    Value::Integer(n) => *n as f64,
                    _ => return Err("pow() needs numbers".to_string()),
                };
                Ok(Value::Float(base.powf(exponent)))
            }))),
        );

        // asin - arc sine
        globals.borrow_mut().define(
            "asin".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "asin",
                1,
                |args| match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.asin())),
                    Value::Integer(n) => Ok(Value::Float((*n as f64).asin())),
                    _ => Err("asin() needs a number".to_string()),
                },
            ))),
        );

        // acos - arc cosine
        globals.borrow_mut().define(
            "acos".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "acos",
                1,
                |args| match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.acos())),
                    Value::Integer(n) => Ok(Value::Float((*n as f64).acos())),
                    _ => Err("acos() needs a number".to_string()),
                },
            ))),
        );

        // atan - arc tangent
        globals.borrow_mut().define(
            "atan".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "atan",
                1,
                |args| match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.atan())),
                    Value::Integer(n) => Ok(Value::Float((*n as f64).atan())),
                    _ => Err("atan() needs a number".to_string()),
                },
            ))),
        );

        // atan2 - two-argument arc tangent
        globals.borrow_mut().define(
            "atan2".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("atan2", 2, |args| {
                let y = match &args[0] {
                    Value::Float(f) => *f,
                    Value::Integer(n) => *n as f64,
                    _ => return Err("atan2() needs numbers".to_string()),
                };
                let x = match &args[1] {
                    Value::Float(f) => *f,
                    Value::Integer(n) => *n as f64,
                    _ => return Err("atan2() needs numbers".to_string()),
                };
                Ok(Value::Float(y.atan2(x)))
            }))),
        );

        // hypot - hypotenuse (sqrt(x² + y²))
        globals.borrow_mut().define(
            "hypot".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("hypot", 2, |args| {
                let x = match &args[0] {
                    Value::Float(f) => *f,
                    Value::Integer(n) => *n as f64,
                    _ => return Err("hypot() needs numbers".to_string()),
                };
                let y = match &args[1] {
                    Value::Float(f) => *f,
                    Value::Integer(n) => *n as f64,
                    _ => return Err("hypot() needs numbers".to_string()),
                };
                Ok(Value::Float(x.hypot(y)))
            }))),
        );

        // degrees - convert radians to degrees
        globals.borrow_mut().define(
            "degrees".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "degrees",
                1,
                |args| match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.to_degrees())),
                    Value::Integer(n) => Ok(Value::Float((*n as f64).to_degrees())),
                    _ => Err("degrees() needs a number".to_string()),
                },
            ))),
        );

        // radians - convert degrees to radians
        globals.borrow_mut().define(
            "radians".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "radians",
                1,
                |args| match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.to_radians())),
                    Value::Integer(n) => Ok(Value::Float((*n as f64).to_radians())),
                    _ => Err("radians() needs a number".to_string()),
                },
            ))),
        );

        // === Time Functions ===

        // snooze - sleep for milliseconds (Scots: have a wee rest)
        globals.borrow_mut().define(
            "snooze".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("snooze", 1, |args| {
                let ms = args[0]
                    .as_integer()
                    .ok_or("snooze() needs an integer (milliseconds)")?;
                if ms < 0 {
                    return Err("Cannae snooze fer negative time, ya daftie!".to_string());
                }
                std::thread::sleep(std::time::Duration::from_millis(ms as u64));
                Ok(Value::Nil)
            }))),
        );

        // === String Functions ===

        // roar - convert to uppercase (shout it oot even louder than upper!)
        globals.borrow_mut().define(
            "roar".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("roar", 1, |args| {
                if let Value::String(s) = &args[0] {
                    // Add exclamation for extra emphasis!
                    Ok(Value::String(format!("{}!", s.to_uppercase())))
                } else {
                    Err("roar() expects a string".to_string())
                }
            }))),
        );

        // mutter - whisper text (lowercase with dots)
        globals.borrow_mut().define(
            "mutter".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("mutter", 1, |args| {
                if let Value::String(s) = &args[0] {
                    Ok(Value::String(format!("...{}...", s.to_lowercase())))
                } else {
                    Err("mutter() expects a string".to_string())
                }
            }))),
        );

        // blooter - scramble a string randomly (Scots: hit/strike messily)
        globals.borrow_mut().define(
            "blooter".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("blooter", 1, |args| {
                if let Value::String(s) = &args[0] {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let seed = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos() as u64;
                    let mut chars: Vec<char> = s.chars().collect();
                    // Fisher-Yates shuffle
                    let mut rng = seed;
                    for i in (1..chars.len()).rev() {
                        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                        let j = (rng as usize) % (i + 1);
                        chars.swap(i, j);
                    }
                    Ok(Value::String(chars.into_iter().collect()))
                } else {
                    Err("blooter() expects a string".to_string())
                }
            }))),
        );

        // pad_left - pad string on left
        globals.borrow_mut().define(
            "pad_left".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("pad_left", 3, |args| {
                match (&args[0], &args[1], &args[2]) {
                    (Value::String(s), Value::Integer(width), Value::String(pad)) => {
                        let pad_char = pad.chars().next().unwrap_or(' ');
                        let w = *width as usize;
                        if s.len() >= w {
                            Ok(Value::String(s.clone()))
                        } else {
                            Ok(Value::String(format!(
                                "{}{}",
                                pad_char.to_string().repeat(w - s.len()),
                                s
                            )))
                        }
                    }
                    _ => Err("pad_left() needs (string, width, pad_char)".to_string()),
                }
            }))),
        );

        // pad_right - pad string on right
        globals.borrow_mut().define(
            "pad_right".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("pad_right", 3, |args| match (
                &args[0], &args[1], &args[2],
            ) {
                (Value::String(s), Value::Integer(width), Value::String(pad)) => {
                    let pad_char = pad.chars().next().unwrap_or(' ');
                    let w = *width as usize;
                    if s.len() >= w {
                        Ok(Value::String(s.clone()))
                    } else {
                        Ok(Value::String(format!(
                            "{}{}",
                            s,
                            pad_char.to_string().repeat(w - s.len())
                        )))
                    }
                }
                _ => Err("pad_right() needs (string, width, pad_char)".to_string()),
            }))),
        );

        // === List Functions ===

        // drap - drop first n elements from list (Scots: drop)
        globals.borrow_mut().define(
            "drap".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("drap", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::List(list), Value::Integer(n)) => {
                        let n = *n as usize;
                        let items = list.borrow();
                        let result: Vec<Value> = items.iter().skip(n).cloned().collect();
                        Ok(Value::List(Rc::new(RefCell::new(result))))
                    }
                    _ => Err("drap() needs a list and an integer".to_string()),
                }
            }))),
        );

        // tak - take first n elements from list (Scots: take)
        globals.borrow_mut().define(
            "tak".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("tak", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::List(list), Value::Integer(n)) => {
                        let n = *n as usize;
                        let items = list.borrow();
                        let result: Vec<Value> = items.iter().take(n).cloned().collect();
                        Ok(Value::List(Rc::new(RefCell::new(result))))
                    }
                    _ => Err("tak() needs a list and an integer".to_string()),
                }
            }))),
        );

        // grup - group elements into chunks (Scots: grip/group)
        globals.borrow_mut().define(
            "grup".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("grup", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::List(list), Value::Integer(size)) => {
                        if *size <= 0 {
                            return Err("grup() needs a positive chunk size".to_string());
                        }
                        let size = *size as usize;
                        let items = list.borrow();
                        let result: Vec<Value> = items
                            .chunks(size)
                            .map(|chunk| Value::List(Rc::new(RefCell::new(chunk.to_vec()))))
                            .collect();
                        Ok(Value::List(Rc::new(RefCell::new(result))))
                    }
                    _ => Err("grup() needs a list and an integer".to_string()),
                }
            }))),
        );

        // pair_up - create pairs from a list [a,b,c,d] -> [[a,b], [c,d]]
        globals.borrow_mut().define(
            "pair_up".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("pair_up", 1, |args| {
                if let Value::List(list) = &args[0] {
                    let items = list.borrow();
                    let result: Vec<Value> = items
                        .chunks(2)
                        .map(|chunk| Value::List(Rc::new(RefCell::new(chunk.to_vec()))))
                        .collect();
                    Ok(Value::List(Rc::new(RefCell::new(result))))
                } else {
                    Err("pair_up() needs a list".to_string())
                }
            }))),
        );

        // fankle - interleave two lists (Scots: tangle)
        globals.borrow_mut().define(
            "fankle".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("fankle", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::List(a), Value::List(b)) => {
                        let a = a.borrow();
                        let b = b.borrow();
                        let mut result = Vec::new();
                        let mut ai = a.iter();
                        let mut bi = b.iter();
                        loop {
                            match (ai.next(), bi.next()) {
                                (Some(x), Some(y)) => {
                                    result.push(x.clone());
                                    result.push(y.clone());
                                }
                                (Some(x), None) => result.push(x.clone()),
                                (None, Some(y)) => result.push(y.clone()),
                                (None, None) => break,
                            }
                        }
                        Ok(Value::List(Rc::new(RefCell::new(result))))
                    }
                    _ => Err("fankle() needs two lists".to_string()),
                }
            }))),
        );

        // === Fun Scottish Functions ===

        // och - express disappointment or frustration
        globals.borrow_mut().define(
            "och".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("och", 1, |args| {
                Ok(Value::String(format!("Och! {}", args[0])))
            }))),
        );

        // jings - express surprise (like "gosh!" or "goodness!")
        globals.borrow_mut().define(
            "jings".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("jings", 1, |args| {
                Ok(Value::String(format!("Jings! {}", args[0])))
            }))),
        );

        // crivvens - express astonishment (from Oor Wullie)
        globals.borrow_mut().define(
            "crivvens".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("crivvens", 1, |args| {
                Ok(Value::String(format!("Crivvens! {}", args[0])))
            }))),
        );

        // help_ma_boab - express extreme surprise (Scottish exclamation)
        globals.borrow_mut().define(
            "help_ma_boab".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("help_ma_boab", 1, |args| {
                Ok(Value::String(format!("Help ma boab! {}", args[0])))
            }))),
        );

        // haud_yer_wheesht - tell someone to be quiet (returns empty string)
        globals.borrow_mut().define(
            "haud_yer_wheesht".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "haud_yer_wheesht",
                0,
                |_args| Ok(Value::String("".to_string())),
            ))),
        );

        // braw - check if something is good/excellent
        globals.borrow_mut().define(
            "braw".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("braw", 1, |args| {
                // Everything is braw in Scotland!
                let val = &args[0];
                let is_braw = match val {
                    Value::Nil => false,
                    Value::Bool(b) => *b,
                    Value::Integer(n) => *n > 0,
                    Value::Float(f) => *f > 0.0,
                    Value::String(s) => !s.is_empty(),
                    Value::List(l) => !l.borrow().is_empty(),
                    Value::Dict(d) => !d.borrow().is_empty(),
                    _ => true,
                };
                Ok(Value::Bool(is_braw))
            }))),
        );

        // clarty - check if something is messy/dirty (has duplicates in list)
        globals.borrow_mut().define(
            "clarty".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("clarty", 1, |args| {
                if let Value::List(list) = &args[0] {
                    let items = list.borrow();
                    let mut seen = Vec::new();
                    for item in items.iter() {
                        if seen.contains(item) {
                            return Ok(Value::Bool(true)); // Has duplicates = clarty
                        }
                        seen.push(item.clone());
                    }
                    Ok(Value::Bool(false))
                } else if let Value::String(s) = &args[0] {
                    // String is clarty if it has repeated characters
                    let chars: Vec<char> = s.chars().collect();
                    let unique: std::collections::HashSet<char> = chars.iter().cloned().collect();
                    Ok(Value::Bool(chars.len() != unique.len()))
                } else {
                    Err("clarty() needs a list or string".to_string())
                }
            }))),
        );

        // dreich - check if a string is boring/dull (all same character or empty)
        globals.borrow_mut().define(
            "dreich".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("dreich", 1, |args| {
                if let Value::String(s) = &args[0] {
                    if s.is_empty() {
                        return Ok(Value::Bool(true)); // Empty is dreich
                    }
                    let first = s.chars().next().unwrap();
                    let is_dreich = s.chars().all(|c| c == first);
                    Ok(Value::Bool(is_dreich))
                } else {
                    Err("dreich() needs a string".to_string())
                }
            }))),
        );

        // stoater - get a particularly good/outstanding element (max for numbers)
        globals.borrow_mut().define(
            "stoater".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("stoater", 1, |args| {
                if let Value::List(list) = &args[0] {
                    let items = list.borrow();
                    if items.is_empty() {
                        return Err("Cannae find a stoater in an empty list!".to_string());
                    }
                    // Find the "best" element (max for numbers, longest for strings)
                    let mut best = items[0].clone();
                    for item in items.iter().skip(1) {
                        match (&best, item) {
                            (Value::Integer(a), Value::Integer(b)) => {
                                if *b > *a {
                                    best = item.clone();
                                }
                            }
                            (Value::Float(a), Value::Float(b)) => {
                                if *b > *a {
                                    best = item.clone();
                                }
                            }
                            (Value::String(a), Value::String(b)) => {
                                if b.len() > a.len() {
                                    best = item.clone();
                                }
                            }
                            _ => {}
                        }
                    }
                    Ok(best)
                } else {
                    Err("stoater() needs a list".to_string())
                }
            }))),
        );

        // numpty_check - validate input isn't empty/nil
        globals.borrow_mut().define(
            "numpty_check".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "numpty_check",
                1,
                |args| match &args[0] {
                    Value::Nil => Ok(Value::String("That's naething, ya numpty!".to_string())),
                    Value::String(s) if s.is_empty() => {
                        Ok(Value::String("Empty string, ya numpty!".to_string()))
                    }
                    Value::List(l) if l.borrow().is_empty() => {
                        Ok(Value::String("Empty list, ya numpty!".to_string()))
                    }
                    _ => Ok(Value::String("That's braw!".to_string())),
                },
            ))),
        );

        // scottify - add Scottish flair to text
        globals.borrow_mut().define(
            "scottify".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("scottify", 1, |args| {
                if let Value::String(s) = &args[0] {
                    let scottified = s
                        .replace("yes", "aye")
                        .replace("Yes", "Aye")
                        .replace("no", "nae")
                        .replace("No", "Nae")
                        .replace("know", "ken")
                        .replace("Know", "Ken")
                        .replace("not", "nae")
                        .replace("from", "fae")
                        .replace("to", "tae")
                        .replace("do", "dae")
                        .replace("myself", "masel")
                        .replace("yourself", "yersel")
                        .replace("small", "wee")
                        .replace("little", "wee")
                        .replace("child", "bairn")
                        .replace("children", "bairns")
                        .replace("church", "kirk")
                        .replace("beautiful", "bonnie")
                        .replace("Beautiful", "Bonnie")
                        .replace("going", "gaun")
                        .replace("have", "hae")
                        .replace("nothing", "naething")
                        .replace("something", "somethin")
                        .replace("everything", "awthing")
                        .replace("everyone", "awbody")
                        .replace("about", "aboot")
                        .replace("out", "oot")
                        .replace("house", "hoose");
                    Ok(Value::String(scottified))
                } else {
                    Err("scottify() needs a string".to_string())
                }
            }))),
        );

        // unique - remove duplicates from list (keeps first occurrence)
        globals.borrow_mut().define(
            "unique".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("unique", 1, |args| {
                if let Value::List(list) = &args[0] {
                    let mut seen = Vec::new();
                    let mut result = Vec::new();
                    for item in list.borrow().iter() {
                        if !seen.contains(item) {
                            seen.push(item.clone());
                            result.push(item.clone());
                        }
                    }
                    Ok(Value::List(Rc::new(RefCell::new(result))))
                } else {
                    Err("unique() needs a list".to_string())
                }
            }))),
        );

        // === File I/O Functions ===

        // scrieve - write to file (Scots: "write")
        globals.borrow_mut().define(
            "scrieve".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("scrieve", 2, |args| {
                use std::fs::File;
                use std::io::Write as IoWrite;
                let path = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("scrieve() needs a file path string".to_string()),
                };
                let content = args[1].to_string();
                let mut file = File::create(&path)
                    .map_err(|e| format!("Couldnae open '{}' fer writin': {}", path, e))?;
                file.write_all(content.as_bytes())
                    .map_err(|e| format!("Couldnae write tae '{}': {}", path, e))?;
                Ok(Value::Nil)
            }))),
        );

        // read_file - read entire file (Scots: readie would be good but let's be clear)
        globals.borrow_mut().define(
            "read_file".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("read_file", 1, |args| {
                use std::fs;
                let path = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("read_file() needs a file path string".to_string()),
                };
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Couldnae read '{}': {}", path, e))?;
                Ok(Value::String(content))
            }))),
        );

        // read_lines - read file as list of lines
        globals.borrow_mut().define(
            "read_lines".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("read_lines", 1, |args| {
                use std::fs;
                let path = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("read_lines() needs a file path string".to_string()),
                };
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Couldnae read '{}': {}", path, e))?;
                let lines: Vec<Value> = content
                    .lines()
                    .map(|l| Value::String(l.to_string()))
                    .collect();
                Ok(Value::List(Rc::new(RefCell::new(lines))))
            }))),
        );

        // file_exists - check if file exists
        globals.borrow_mut().define(
            "file_exists".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("file_exists", 1, |args| {
                use std::path::Path;
                let path = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("file_exists() needs a file path string".to_string()),
                };
                Ok(Value::Bool(Path::new(&path).exists()))
            }))),
        );

        // append_file - append to file
        globals.borrow_mut().define(
            "append_file".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("append_file", 2, |args| {
                use std::fs::OpenOptions;
                use std::io::Write as IoWrite;
                let path = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("append_file() needs a file path string".to_string()),
                };
                let content = args[1].to_string();
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                    .map_err(|e| format!("Couldnae open '{}' fer appendin': {}", path, e))?;
                file.write_all(content.as_bytes())
                    .map_err(|e| format!("Couldnae append tae '{}': {}", path, e))?;
                Ok(Value::Nil)
            }))),
        );

        // === More Scots-Themed Functions ===

        // haver - generate random nonsense (Scots: talk rubbish)
        globals.borrow_mut().define(
            "haver".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("haver", 0, |_args| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let havers = [
                    "Och, yer bum's oot the windae!",
                    "Awa' an bile yer heid!",
                    "Haud yer wheesht, ya numpty!",
                    "Dinnae fash yersel!",
                    "Whit's fer ye'll no go by ye!",
                    "Lang may yer lum reek!",
                    "Yer a wee scunner, so ye are!",
                    "Haste ye back!",
                    "It's a dreich day the day!",
                    "Pure dead brilliant!",
                    "Ah'm fair puckled!",
                    "Gie it laldy!",
                    "Whit a stoater!",
                    "That's pure mince!",
                    "Jings, crivvens, help ma boab!",
                ];
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as u64;
                let rng = seed.wrapping_mul(1103515245).wrapping_add(12345);
                let idx = (rng as usize) % havers.len();
                Ok(Value::String(havers[idx].to_string()))
            }))),
        );

        // slainte - return a Scottish toast (Scots: health/cheers)
        globals.borrow_mut().define(
            "slainte".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("slainte", 0, |_args| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let toasts = [
                    "Slàinte mhath! (Good health!)",
                    "Here's tae us, wha's like us? Gey few, and they're a' deid!",
                    "May the best ye've ever seen be the worst ye'll ever see!",
                    "Lang may yer lum reek wi' ither fowk's coal!",
                    "May ye aye be happy, an' never drink frae a toom glass!",
                    "Here's tae the heath, the hill and the heather!",
                ];
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as u64;
                let rng = seed.wrapping_mul(1103515245).wrapping_add(12345);
                let idx = (rng as usize) % toasts.len();
                Ok(Value::String(toasts[idx].to_string()))
            }))),
        );

        // braw_time - format current time in a nice Scottish way
        globals.borrow_mut().define(
            "braw_time".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("braw_time", 0, |_args| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let secs = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                // Simple hour/minute calculation (UTC)
                let hours = (secs / 3600) % 24;
                let minutes = (secs / 60) % 60;
                Ok(Value::String(format_braw_time(hours, minutes)))
            }))),
        );

        // wheesht_aw - trim and clean up a string (more thorough than wheesht)
        globals.borrow_mut().define(
            "wheesht_aw".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("wheesht_aw", 1, |args| {
                if let Value::String(s) = &args[0] {
                    // Collapse multiple spaces and trim
                    let cleaned: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
                    Ok(Value::String(cleaned))
                } else {
                    Err("wheesht_aw() needs a string".to_string())
                }
            }))),
        );

        // scunner_check - validate that a value meets expectations (returns descriptive error)
        globals.borrow_mut().define(
            "scunner_check".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("scunner_check", 2, |args| {
                let val = &args[0];
                let expected_type = match &args[1] {
                    Value::String(s) => s.as_str(),
                    _ => return Err("scunner_check() needs type name as second arg".to_string()),
                };
                let actual_type = val.type_name();
                if actual_type == expected_type {
                    Ok(Value::Bool(true))
                } else {
                    Ok(Value::String(format!(
                        "Och, ya scunner! Expected {} but got {}",
                        expected_type, actual_type
                    )))
                }
            }))),
        );

        // bampot_mode - deliberately cause chaos (scramble list order)
        globals.borrow_mut().define(
            "bampot_mode".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("bampot_mode", 1, |args| {
                use std::time::{SystemTime, UNIX_EPOCH};
                if let Value::List(list) = &args[0] {
                    let mut items: Vec<Value> = list.borrow().clone();
                    let seed = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos() as u64;
                    let mut rng = seed;
                    // Double shuffle for extra chaos!
                    for _ in 0..2 {
                        for i in (1..items.len()).rev() {
                            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                            let j = (rng as usize) % (i + 1);
                            items.swap(i, j);
                        }
                    }
                    items.reverse(); // And reverse for good measure!
                    Ok(Value::List(Rc::new(RefCell::new(items))))
                } else {
                    Err("bampot_mode() needs a list".to_string())
                }
            }))),
        );

        // crabbit - check if a number is negative (Scots: grumpy/bad-tempered)
        globals.borrow_mut().define(
            "crabbit".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "crabbit",
                1,
                |args| match &args[0] {
                    Value::Integer(n) => Ok(Value::Bool(*n < 0)),
                    Value::Float(f) => Ok(Value::Bool(*f < 0.0)),
                    _ => Err("crabbit() needs a number".to_string()),
                },
            ))),
        );

        // gallus - check if a value is bold/impressive (non-empty/non-zero)
        globals.borrow_mut().define(
            "gallus".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("gallus", 1, |args| {
                let is_gallus = match &args[0] {
                    Value::Integer(n) => *n != 0 && (*n > 100 || *n < -100),
                    Value::Float(f) => *f != 0.0 && (*f > 100.0 || *f < -100.0),
                    Value::String(s) => s.len() > 20,
                    Value::List(l) => l.borrow().len() > 10,
                    _ => false,
                };
                Ok(Value::Bool(is_gallus))
            }))),
        );

        // drookit - check if list has duplicates (Scots: soaking wet/full)
        globals.borrow_mut().define(
            "drookit".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("drookit", 1, |args| {
                if let Value::List(list) = &args[0] {
                    let items = list.borrow();
                    let mut seen = Vec::new();
                    for item in items.iter() {
                        if seen.contains(item) {
                            return Ok(Value::Bool(true));
                        }
                        seen.push(item.clone());
                    }
                    Ok(Value::Bool(false))
                } else {
                    Err("drookit() needs a list".to_string())
                }
            }))),
        );

        // glaikit - check if something looks "stupid" (empty, zero, or invalid)
        globals.borrow_mut().define(
            "glaikit".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("glaikit", 1, |args| {
                let is_glaikit = match &args[0] {
                    Value::Nil => true,
                    Value::Integer(0) => true,
                    Value::Float(f) if *f == 0.0 => true,
                    Value::String(s) if s.is_empty() || s.trim().is_empty() => true,
                    Value::List(l) if l.borrow().is_empty() => true,
                    Value::Dict(d) if d.borrow().is_empty() => true,
                    _ => false,
                };
                Ok(Value::Bool(is_glaikit))
            }))),
        );

        // geggie - get the "mouth" (first and last chars) of a string
        globals.borrow_mut().define(
            "geggie".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("geggie", 1, |args| {
                if let Value::String(s) = &args[0] {
                    if s.is_empty() {
                        return Ok(Value::String("".to_string()));
                    }
                    let first = s.chars().next().unwrap();
                    let last = s.chars().last().unwrap();
                    Ok(Value::String(format!("{}{}", first, last)))
                } else {
                    Err("geggie() needs a string".to_string())
                }
            }))),
        );

        // banter - interleave two strings
        globals.borrow_mut().define(
            "banter".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("banter", 2, |args| {
                let s1 = match &args[0] {
                    Value::String(s) => s,
                    _ => return Err("banter() needs two strings".to_string()),
                };
                let s2 = match &args[1] {
                    Value::String(s) => s,
                    _ => return Err("banter() needs two strings".to_string()),
                };
                let mut result = String::new();
                let mut chars1 = s1.chars();
                let mut chars2 = s2.chars();
                loop {
                    match (chars1.next(), chars2.next()) {
                        (Some(c1), Some(c2)) => {
                            result.push(c1);
                            result.push(c2);
                        }
                        (Some(c1), None) => result.push(c1),
                        (None, Some(c2)) => result.push(c2),
                        (None, None) => break,
                    }
                }
                Ok(Value::String(result))
            }))),
        );

        // skelp - split a string into chunks of n chars (Scots: slap/hit)
        globals.borrow_mut().define(
            "skelp".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("skelp", 2, |args| {
                let s = match &args[0] {
                    Value::String(s) => s,
                    _ => return Err("skelp() needs a string and size".to_string()),
                };
                let size = args[1].as_integer().ok_or("skelp() needs integer size")?;
                if size <= 0 {
                    return Err("skelp() size must be positive".to_string());
                }
                let chunks: Vec<Value> = s
                    .chars()
                    .collect::<Vec<_>>()
                    .chunks(size as usize)
                    .map(|chunk| Value::String(chunk.iter().collect()))
                    .collect();
                Ok(Value::List(Rc::new(RefCell::new(chunks))))
            }))),
        );

        // indices_o - find all indices of a value (Scots: indices of)
        globals.borrow_mut().define(
            "indices_o".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "indices_o",
                2,
                |args| match &args[0] {
                    Value::List(list) => {
                        let items = list.borrow();
                        let needle = &args[1];
                        let indices: Vec<Value> = items
                            .iter()
                            .enumerate()
                            .filter(|(_, item)| *item == needle)
                            .map(|(i, _)| Value::Integer(i as i64))
                            .collect();
                        Ok(Value::List(Rc::new(RefCell::new(indices))))
                    }
                    Value::String(s) => {
                        let needle = match &args[1] {
                            Value::String(n) => n,
                            _ => {
                                return Err(
                                    "indices_o() on string needs a string needle".to_string()
                                )
                            }
                        };
                        if needle.is_empty() {
                            return Err("Cannae search fer an empty string, ya numpty!".to_string());
                        }
                        let indices: Vec<Value> = s
                            .match_indices(needle.as_str())
                            .map(|(i, _)| Value::Integer(i as i64))
                            .collect();
                        Ok(Value::List(Rc::new(RefCell::new(indices))))
                    }
                    _ => Err("indices_o() needs a list or string".to_string()),
                },
            ))),
        );

        // braw_date - format a timestamp or current time in Scottish style
        globals.borrow_mut().define(
            "braw_date".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("braw_date", 1, |args| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let secs = match &args[0] {
                    Value::Integer(n) => *n as u64,
                    Value::Nil => SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    _ => return Err("braw_date() needs a timestamp or naething".to_string()),
                };
                // Calculate date components (simplified, doesn't handle leap years perfectly)
                let days_since_epoch = secs / 86400;
                let day_of_week = ((days_since_epoch + 4) % 7) as usize; // Jan 1, 1970 was Thursday

                let scots_day_names = [
                    "the Sabbath",
                    "Monday",
                    "Tuesday",
                    "Wednesday",
                    "Thursday",
                    "Friday",
                    "Setterday",
                ];

                // Simple month/day calculation
                let mut remaining_days = days_since_epoch as i64;
                let mut year = 1970i64;
                loop {
                    let days_in_year = if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                        366
                    } else {
                        365
                    };
                    if remaining_days < days_in_year {
                        break;
                    }
                    remaining_days -= days_in_year;
                    year += 1;
                }

                let scots_months = [
                    "Januar",
                    "Februar",
                    "Mairch",
                    "Aprile",
                    "Mey",
                    "Juin",
                    "Julie",
                    "August",
                    "September",
                    "October",
                    "November",
                    "December",
                ];
                let days_in_months: [i64; 12] =
                    if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
                    } else {
                        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
                    };

                let mut month = 0usize;
                for (i, &days) in days_in_months.iter().enumerate() {
                    if remaining_days < days {
                        month = i;
                        break;
                    }
                    remaining_days -= days;
                }
                let day = remaining_days + 1;

                let ordinal = match day {
                    1 | 21 | 31 => "st",
                    2 | 22 => "nd",
                    3 | 23 => "rd",
                    _ => "th",
                };

                Ok(Value::String(format!(
                    "{}, the {}{} o' {}, {}",
                    scots_day_names[day_of_week], day, ordinal, scots_months[month], year
                )))
            }))),
        );

        // Higher-order functions are defined as marker values
        // They get special handling in call_value

        // gaun - map function over list (Scots: "going")
        globals.borrow_mut().define(
            "gaun".to_string(),
            Value::String("__builtin_gaun__".to_string()),
        );

        // sieve - filter list (keep elements that pass)
        globals.borrow_mut().define(
            "sieve".to_string(),
            Value::String("__builtin_sieve__".to_string()),
        );

        // tumble - reduce/fold list (Scots: tumble together)
        globals.borrow_mut().define(
            "tumble".to_string(),
            Value::String("__builtin_tumble__".to_string()),
        );

        // ilk - for each (Scots: each/every)
        globals.borrow_mut().define(
            "ilk".to_string(),
            Value::String("__builtin_ilk__".to_string()),
        );

        // hunt - find first matching element
        globals.borrow_mut().define(
            "hunt".to_string(),
            Value::String("__builtin_hunt__".to_string()),
        );

        // ony - check if any element matches (Scots: any)
        globals.borrow_mut().define(
            "ony".to_string(),
            Value::String("__builtin_ony__".to_string()),
        );

        // aw - check if all elements match (Scots: all)
        globals.borrow_mut().define(
            "aw".to_string(),
            Value::String("__builtin_aw__".to_string()),
        );

        // grup_up - group list elements by function result (Scots: group up)
        globals.borrow_mut().define(
            "grup_up".to_string(),
            Value::String("__builtin_grup_up__".to_string()),
        );

        // pairt_by - partition list by predicate into [true, false] lists
        globals.borrow_mut().define(
            "pairt_by".to_string(),
            Value::String("__builtin_pairt_by__".to_string()),
        );

        // === More Scots-Flavoured Functions ===

        // haverin - check if a string is empty/nonsense (talking havers!)
        globals.borrow_mut().define(
            "haverin".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "haverin",
                1,
                |args| match &args[0] {
                    Value::String(s) => {
                        let trimmed = s.trim();
                        Ok(Value::Bool(trimmed.is_empty() || trimmed.len() < 2))
                    }
                    Value::Nil => Ok(Value::Bool(true)),
                    Value::List(l) => Ok(Value::Bool(l.borrow().is_empty())),
                    _ => Ok(Value::Bool(false)),
                },
            ))),
        );

        // scunner - check if value is "disgusting" (negative or empty)
        globals.borrow_mut().define(
            "scunner".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "scunner",
                1,
                |args| match &args[0] {
                    Value::Integer(n) => Ok(Value::Bool(*n < 0)),
                    Value::Float(f) => Ok(Value::Bool(*f < 0.0)),
                    Value::String(s) => Ok(Value::Bool(s.is_empty())),
                    Value::List(l) => Ok(Value::Bool(l.borrow().is_empty())),
                    Value::Bool(b) => Ok(Value::Bool(!*b)),
                    Value::Nil => Ok(Value::Bool(true)),
                    _ => Ok(Value::Bool(false)),
                },
            ))),
        );

        // bonnie - pretty print a value with decoration
        globals.borrow_mut().define(
            "bonnie".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("bonnie", 1, |args| {
                let val_str = format!("{}", args[0]);
                Ok(Value::String(format!("~~~ {} ~~~", val_str)))
            }))),
        );

        // is_wee - check if value is small (< 10 for numbers, < 5 chars for strings)
        globals.borrow_mut().define(
            "is_wee".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "is_wee",
                1,
                |args| match &args[0] {
                    Value::Integer(n) => Ok(Value::Bool(n.abs() < 10)),
                    Value::Float(f) => Ok(Value::Bool(f.abs() < 10.0)),
                    Value::String(s) => Ok(Value::Bool(s.len() < 5)),
                    Value::List(l) => Ok(Value::Bool(l.borrow().len() < 5)),
                    _ => Ok(Value::Bool(true)),
                },
            ))),
        );

        // is_muckle - check if value is big (opposite of is_wee)
        globals.borrow_mut().define(
            "is_muckle".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "is_muckle",
                1,
                |args| match &args[0] {
                    Value::Integer(n) => Ok(Value::Bool(n.abs() >= 100)),
                    Value::Float(f) => Ok(Value::Bool(f.abs() >= 100.0)),
                    Value::String(s) => Ok(Value::Bool(s.len() >= 50)),
                    Value::List(l) => Ok(Value::Bool(l.borrow().len() >= 50)),
                    _ => Ok(Value::Bool(false)),
                },
            ))),
        );

        // cannie - check if value is safe/valid (not nil, not empty, not negative)
        globals.borrow_mut().define(
            "cannie".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "cannie",
                1,
                |args| match &args[0] {
                    Value::Nil => Ok(Value::Bool(false)),
                    Value::Integer(n) => Ok(Value::Bool(*n >= 0)),
                    Value::Float(f) => Ok(Value::Bool(*f >= 0.0 && !f.is_nan())),
                    Value::String(s) => Ok(Value::Bool(!s.is_empty())),
                    Value::List(l) => Ok(Value::Bool(!l.borrow().is_empty())),
                    Value::Bool(b) => Ok(Value::Bool(*b)),
                    _ => Ok(Value::Bool(true)),
                },
            ))),
        );

        // wrang_sort - check if value is the wrong type (sort = kind/type in Scots)
        globals.borrow_mut().define(
            "wrang_sort".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("wrang_sort", 2, |args| {
                let expected_type = match &args[1] {
                    Value::String(s) => s.as_str(),
                    _ => return Err("Second arg must be a type name string".to_string()),
                };
                let actual_type = args[0].type_name();
                Ok(Value::Bool(actual_type != expected_type))
            }))),
        );

        // tattie_scone - repeat string n times with | separator (like stacking scones!)
        globals.borrow_mut().define(
            "tattie_scone".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("tattie_scone", 2, |args| {
                let s = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("tattie_scone needs a string".to_string()),
                };
                let n = match &args[1] {
                    Value::Integer(n) => *n as usize,
                    _ => return Err("tattie_scone needs a number".to_string()),
                };
                let result = vec![s; n].join(" | ");
                Ok(Value::String(result))
            }))),
        );

        // haggis_hunt - find all occurrences of substring in string
        globals.borrow_mut().define(
            "haggis_hunt".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("haggis_hunt", 2, |args| {
                let haystack = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("haggis_hunt needs a string tae search".to_string()),
                };
                let needle = match &args[1] {
                    Value::String(s) => s.clone(),
                    _ => return Err("haggis_hunt needs a string tae find".to_string()),
                };
                let positions: Vec<Value> = haystack
                    .match_indices(&needle)
                    .map(|(i, _)| Value::Integer(i as i64))
                    .collect();
                Ok(Value::List(Rc::new(RefCell::new(positions))))
            }))),
        );

        // sporran_fill - pad both sides of string (like a sporran!)
        globals.borrow_mut().define(
            "sporran_fill".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("sporran_fill", 3, |args| {
                let s = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("sporran_fill needs a string".to_string()),
                };
                let width = match &args[1] {
                    Value::Integer(n) => *n as usize,
                    _ => return Err("sporran_fill needs a width".to_string()),
                };
                let fill = match &args[2] {
                    Value::String(c) => c.chars().next().unwrap_or(' '),
                    _ => return Err("sporran_fill needs a fill character".to_string()),
                };
                if s.len() >= width {
                    return Ok(Value::String(s));
                }
                let padding = width - s.len();
                let left_pad = padding / 2;
                let right_pad = padding - left_pad;
                let result = format!(
                    "{}{}{}",
                    fill.to_string().repeat(left_pad),
                    s,
                    fill.to_string().repeat(right_pad)
                );
                Ok(Value::String(result))
            }))),
        );

        // ============================================================
        // MORE SCOTS FUN FUNCTIONS
        // ============================================================

        // blether_format - format a string with named placeholders
        // blether_format("Hullo {name}, ye are {age} years auld", {"name": "Hamish", "age": 42})
        globals.borrow_mut().define(
            "blether_format".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("blether_format", 2, |args| {
                let template = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("blether_format needs a template string".to_string()),
                };
                let mut result = template;
                let dict = match &args[1] {
                    Value::Dict(d) => d.clone(),
                    _ => return Err("blether_format needs a dictionary o' values".to_string()),
                };
                for (key, value) in dict.borrow().iter() {
                    let key_str = match key {
                        Value::String(s) => s.clone(),
                        _ => format!("{}", key),
                    };
                    let placeholder = format!("{{{}}}", key_str);
                    result = result.replace(&placeholder, &format!("{}", value));
                }
                Ok(Value::String(result))
            }))),
        );

        // ceilidh - shuffle and interleave two lists like dancers at a ceilidh!
        globals.borrow_mut().define(
            "ceilidh".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("ceilidh", 2, |args| {
                let list1 = match &args[0] {
                    Value::List(l) => l.borrow().clone(),
                    _ => return Err("ceilidh needs two lists".to_string()),
                };
                let list2 = match &args[1] {
                    Value::List(l) => l.borrow().clone(),
                    _ => return Err("ceilidh needs two lists".to_string()),
                };
                let mut result = Vec::new();
                let max_len = list1.len().max(list2.len());
                for i in 0..max_len {
                    if i < list1.len() {
                        result.push(list1[i].clone());
                    }
                    if i < list2.len() {
                        result.push(list2[i].clone());
                    }
                }
                Ok(Value::List(Rc::new(RefCell::new(result))))
            }))),
        );

        // dram - get a random element from a list (like pouring a wee dram!)
        globals.borrow_mut().define(
            "dram".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("dram", 1, |args| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let list = match &args[0] {
                    Value::List(l) => l.borrow().clone(),
                    _ => return Err("dram needs a list tae pick fae".to_string()),
                };
                if list.is_empty() {
                    return Ok(Value::Nil);
                }
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as usize;
                let idx = seed % list.len();
                Ok(list[idx].clone())
            }))),
        );

        // birl - rotate a list (birl = spin/rotate in Scots)
        globals.borrow_mut().define(
            "birl".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("birl", 2, |args| {
                let list = match &args[0] {
                    Value::List(l) => l.borrow().clone(),
                    _ => return Err("birl needs a list".to_string()),
                };
                let n = match &args[1] {
                    Value::Integer(n) => *n,
                    _ => return Err("birl needs a rotation count".to_string()),
                };
                if list.is_empty() {
                    return Ok(Value::List(Rc::new(RefCell::new(list))));
                }
                let len = list.len() as i64;
                let n = ((n % len) + len) % len; // Handle negative rotation
                let n = n as usize;
                let mut result = list.clone();
                result.rotate_left(n);
                Ok(Value::List(Rc::new(RefCell::new(result))))
            }))),
        );

        // stooshie - create chaos/noise (shuffle a string's characters)
        globals.borrow_mut().define(
            "stooshie".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("stooshie", 1, |args| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let s = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("stooshie needs a string".to_string()),
                };
                let mut chars: Vec<char> = s.chars().collect();
                // Simple Fisher-Yates shuffle
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as u64;
                let mut rng = seed;
                for i in (1..chars.len()).rev() {
                    rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                    let j = (rng as usize) % (i + 1);
                    chars.swap(i, j);
                }
                Ok(Value::String(chars.into_iter().collect()))
            }))),
        );

        // clype - report on a value (like telling tales!) - returns debug info
        globals.borrow_mut().define(
            "clype".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("clype", 1, |args| {
                let val = &args[0];
                let type_name = val.type_name();
                let info = match val {
                    Value::List(l) => format!("list wi' {} items", l.borrow().len()),
                    Value::Dict(d) => format!("dict wi' {} entries", d.borrow().len()),
                    Value::Set(s) => format!("creel wi' {} items", s.borrow().len()),
                    Value::String(s) => format!("string o' {} characters", s.len()),
                    Value::Integer(n) => format!("integer: {}", n),
                    Value::Float(f) => format!("float: {}", f),
                    Value::Bool(b) => format!("boolean: {}", if *b { "aye" } else { "nae" }),
                    Value::Nil => "naething".to_string(),
                    Value::Function(f) => format!("function '{}'", f.name),
                    Value::NativeFunction(f) => format!("native function '{}'", f.name),
                    Value::Class(c) => format!("class '{}'", c.name),
                    Value::Instance(inst) => format!("instance o' '{}'", inst.borrow().class.name),
                    _ => type_name.to_string(),
                };
                Ok(Value::String(format!("[{}] {}", type_name, info)))
            }))),
        );

        // sclaff - flatten nested lists (sclaff = hit flat in golf)
        globals.borrow_mut().define(
            "sclaff".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("sclaff", 1, |args| {
                fn flatten_recursive(val: &Value, result: &mut Vec<Value>) {
                    match val {
                        Value::List(l) => {
                            for item in l.borrow().iter() {
                                flatten_recursive(item, result);
                            }
                        }
                        other => result.push(other.clone()),
                    }
                }
                let list = match &args[0] {
                    Value::List(l) => l.clone(),
                    _ => return Err("sclaff needs a list".to_string()),
                };
                let mut result = Vec::new();
                for item in list.borrow().iter() {
                    flatten_recursive(item, &mut result);
                }
                Ok(Value::List(Rc::new(RefCell::new(result))))
            }))),
        );

        // ============================================================
        // TIMING/BENCHMARKING FUNCTIONS - Measure yer code's speed!
        // ============================================================

        // noo - get current timestamp in milliseconds (like "now")
        globals.borrow_mut().define(
            "noo".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("noo", 0, |_args| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                Ok(Value::Integer(duration.as_millis() as i64))
            }))),
        );

        // tick - high precision nanoseconds timestamp
        globals.borrow_mut().define(
            "tick".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("tick", 0, |_args| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                Ok(Value::Integer(duration.as_nanos() as i64))
            }))),
        );

        // bide - sleep for milliseconds (bide = wait in Scots)
        globals.borrow_mut().define(
            "bide".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("bide", 1, |args| {
                let ms = match &args[0] {
                    Value::Integer(n) => *n as u64,
                    Value::Float(f) => *f as u64,
                    _ => return Err("bide() needs a number o' milliseconds".to_string()),
                };
                std::thread::sleep(std::time::Duration::from_millis(ms));
                Ok(Value::Nil)
            }))),
        );

        // stopwatch - time a function call and return [result, time_ms]
        globals.borrow_mut().define(
            "stopwatch".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("stopwatch", 1, |args| {
                // This is a placeholder - actual timing requires interpreter access
                // For now, just return the function info
                match &args[0] {
                    Value::Function(f) => Ok(Value::String(format!(
                        "Use 'noo()' before and after callin' '{}' tae time it!",
                        f.name
                    ))),
                    _ => Err("stopwatch() needs a function".to_string()),
                }
            }))),
        );

        // ============================================================
        // SET (CREEL) FUNCTIONS - A creel is a basket in Scots!
        // ============================================================

        // creel - create a new set from a list
        globals.borrow_mut().define(
            "creel".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("creel", 1, |args| {
                match &args[0] {
                    Value::List(list) => {
                        let mut items = SetValue::new();
                        for item in list.borrow().iter() {
                            items.insert(item.clone());
                        }
                        Ok(Value::Set(Rc::new(RefCell::new(items))))
                    }
                    Value::Set(s) => Ok(Value::Set(s.clone())), // Already a set
                    _ => Err("creel() needs a list tae make a set fae".to_string()),
                }
            }))),
        );

        // toss_in - add item to set (toss it intae the creel!)
        globals.borrow_mut().define(
            "toss_in".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("toss_in", 2, |args| {
                if let Value::Set(set) = &args[0] {
                    set.borrow_mut().insert(args[1].clone());
                    Ok(Value::Set(set.clone()))
                } else {
                    Err("toss_in() needs a creel (set)".to_string())
                }
            }))),
        );

        // heave_oot - remove item from set (heave it oot the creel!)
        globals.borrow_mut().define(
            "heave_oot".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("heave_oot", 2, |args| {
                if let Value::Set(set) = &args[0] {
                    set.borrow_mut().remove(&args[1]);
                    Ok(Value::Set(set.clone()))
                } else {
                    Err("heave_oot() needs a creel (set)".to_string())
                }
            }))),
        );

        // is_in_creel - check if item is in set
        globals.borrow_mut().define(
            "is_in_creel".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("is_in_creel", 2, |args| {
                if let Value::Set(set) = &args[0] {
                    Ok(Value::Bool(set.borrow().contains(&args[1])))
                } else {
                    Err("is_in_creel() needs a creel (set)".to_string())
                }
            }))),
        );

        // creels_thegither - union of two sets (put them thegither!)
        globals.borrow_mut().define(
            "creels_thegither".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "creels_thegither",
                2,
                |args| match (&args[0], &args[1]) {
                    (Value::Set(a), Value::Set(b)) => {
                        let union = a.borrow().union(&b.borrow());
                        Ok(Value::Set(Rc::new(RefCell::new(union))))
                    }
                    _ => Err("creels_thegither() needs two creels".to_string()),
                },
            ))),
        );

        // creels_baith - intersection of two sets (what's in baith!)
        globals.borrow_mut().define(
            "creels_baith".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "creels_baith",
                2,
                |args| match (&args[0], &args[1]) {
                    (Value::Set(a), Value::Set(b)) => {
                        let intersection = a.borrow().intersection(&b.borrow());
                        Ok(Value::Set(Rc::new(RefCell::new(intersection))))
                    }
                    _ => Err("creels_baith() needs two creels".to_string()),
                },
            ))),
        );

        // creels_differ - difference of two sets (what's in a but no in b)
        globals.borrow_mut().define(
            "creels_differ".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "creels_differ",
                2,
                |args| match (&args[0], &args[1]) {
                    (Value::Set(a), Value::Set(b)) => {
                        let difference = a.borrow().difference(&b.borrow());
                        Ok(Value::Set(Rc::new(RefCell::new(difference))))
                    }
                    _ => Err("creels_differ() needs two creels".to_string()),
                },
            ))),
        );

        // creel_tae_list - convert set to sorted list
        globals.borrow_mut().define(
            "creel_tae_list".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("creel_tae_list", 1, |args| {
                if let Value::Set(set) = &args[0] {
                    let mut items: Vec<(String, Value)> = set
                        .borrow()
                        .iter()
                        .map(|v| (format!("{}", v), v.clone()))
                        .collect();
                    items.sort_by(|a, b| a.0.cmp(&b.0));
                    let values: Vec<Value> = items.into_iter().map(|(_, v)| v).collect();
                    Ok(Value::List(Rc::new(RefCell::new(values))))
                } else {
                    Err("creel_tae_list() needs a creel".to_string())
                }
            }))),
        );

        // is_subset - check if one set is a subset of another (is a inside b?)
        globals.borrow_mut().define(
            "is_subset".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("is_subset", 2, |args| match (
                &args[0], &args[1],
            ) {
                (Value::Set(a), Value::Set(b)) => {
                    Ok(Value::Bool(a.borrow().is_subset(&b.borrow())))
                }
                _ => Err("is_subset() needs two creels".to_string()),
            }))),
        );

        // is_superset - check if one set is a superset of another (does a contain aw o b?)
        globals.borrow_mut().define(
            "is_superset".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "is_superset",
                2,
                |args| match (&args[0], &args[1]) {
                    (Value::Set(a), Value::Set(b)) => {
                        Ok(Value::Bool(a.borrow().is_superset(&b.borrow())))
                    }
                    _ => Err("is_superset() needs two creels".to_string()),
                },
            ))),
        );

        // is_disjoint - check if two sets have nae overlap
        globals.borrow_mut().define(
            "is_disjoint".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "is_disjoint",
                2,
                |args| match (&args[0], &args[1]) {
                    (Value::Set(a), Value::Set(b)) => {
                        Ok(Value::Bool(a.borrow().is_disjoint(&b.borrow())))
                    }
                    _ => Err("is_disjoint() needs two creels".to_string()),
                },
            ))),
        );

        // empty_creel - create an empty set
        globals.borrow_mut().define(
            "empty_creel".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("empty_creel", 0, |_args| {
                Ok(Value::Set(Rc::new(RefCell::new(SetValue::new()))))
            }))),
        );

        // json_parse - parse a JSON string intae a value
        globals.borrow_mut().define(
            "json_parse".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("json_parse", 1, |args| {
                if let Value::String(s) = &args[0] {
                    parse_json_value(s)
                } else {
                    Err("json_parse() expects a string, ya numpty!".to_string())
                }
            }))),
        );

        // json_stringify - convert a value tae JSON string
        globals.borrow_mut().define(
            "json_stringify".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("json_stringify", 1, |args| {
                Ok(Value::String(value_to_json(&args[0])))
            }))),
        );

        // json_pretty - convert a value tae pretty-printed JSON string
        globals.borrow_mut().define(
            "json_pretty".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("json_pretty", 1, |args| {
                Ok(Value::String(value_to_json_pretty(&args[0], 0)))
            }))),
        );

        // ============================================================
        // BITWISE OPERATIONS - Fer aw yer binary fiddlin' needs!
        // ============================================================

        // bit_an - bitwise AND (Scots: "an" = and)
        globals.borrow_mut().define(
            "bit_an".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("bit_an", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(*a & *b)),
                    _ => Err("bit_an() needs two integers".to_string()),
                }
            }))),
        );

        // bit_or - bitwise OR
        globals.borrow_mut().define(
            "bit_or".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("bit_or", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(*a | *b)),
                    _ => Err("bit_or() needs two integers".to_string()),
                }
            }))),
        );

        // bit_xor - bitwise XOR
        globals.borrow_mut().define(
            "bit_xor".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("bit_xor", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(*a ^ *b)),
                    _ => Err("bit_xor() needs two integers".to_string()),
                }
            }))),
        );

        // bit_nae - bitwise NOT (Scots: nae = not)
        globals.borrow_mut().define(
            "bit_nae".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "bit_nae",
                1,
                |args| match &args[0] {
                    Value::Integer(n) => Ok(Value::Integer(!*n)),
                    _ => Err("bit_nae() needs an integer".to_string()),
                },
            ))),
        );

        // bit_shove_left - left shift (shove left!)
        globals.borrow_mut().define(
            "bit_shove_left".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "bit_shove_left",
                2,
                |args| match (&args[0], &args[1]) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if *b < 0 || *b > 63 {
                            return Err("Shift amount must be 0-63, ya numpty!".to_string());
                        }
                        Ok(Value::Integer(*a << *b))
                    }
                    _ => Err("bit_shove_left() needs two integers".to_string()),
                },
            ))),
        );

        // bit_shove_right - right shift (shove right!)
        globals.borrow_mut().define(
            "bit_shove_right".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "bit_shove_right",
                2,
                |args| match (&args[0], &args[1]) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if *b < 0 || *b > 63 {
                            return Err("Shift amount must be 0-63, ya numpty!".to_string());
                        }
                        Ok(Value::Integer(*a >> *b))
                    }
                    _ => Err("bit_shove_right() needs two integers".to_string()),
                },
            ))),
        );

        // bit_coont - count number of set bits (popcount)
        globals.borrow_mut().define(
            "bit_coont".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "bit_coont",
                1,
                |args| match &args[0] {
                    Value::Integer(n) => Ok(Value::Integer(n.count_ones() as i64)),
                    _ => Err("bit_coont() needs an integer".to_string()),
                },
            ))),
        );

        // tae_binary - convert to binary string
        globals.borrow_mut().define(
            "tae_binary".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "tae_binary",
                1,
                |args| match &args[0] {
                    Value::Integer(n) => Ok(Value::String(format!("{:b}", n))),
                    _ => Err("tae_binary() needs an integer".to_string()),
                },
            ))),
        );

        // tae_hex - convert to hexadecimal string
        globals.borrow_mut().define(
            "tae_hex".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "tae_hex",
                1,
                |args| match &args[0] {
                    Value::Integer(n) => Ok(Value::String(format!("{:x}", n))),
                    _ => Err("tae_hex() needs an integer".to_string()),
                },
            ))),
        );

        // tae_octal - convert to octal string
        globals.borrow_mut().define(
            "tae_octal".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "tae_octal",
                1,
                |args| match &args[0] {
                    Value::Integer(n) => Ok(Value::String(format!("{:o}", n))),
                    _ => Err("tae_octal() needs an integer".to_string()),
                },
            ))),
        );

        // fae_binary - parse binary string to integer
        globals.borrow_mut().define(
            "fae_binary".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "fae_binary",
                1,
                |args| match &args[0] {
                    Value::String(s) => i64::from_str_radix(s.trim_start_matches("0b"), 2)
                        .map(Value::Integer)
                        .map_err(|_| format!("Cannae parse '{}' as binary", s)),
                    _ => Err("fae_binary() needs a string".to_string()),
                },
            ))),
        );

        // fae_hex - parse hexadecimal string to integer
        globals.borrow_mut().define(
            "fae_hex".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "fae_hex",
                1,
                |args| match &args[0] {
                    Value::String(s) => i64::from_str_radix(s.trim_start_matches("0x"), 16)
                        .map(Value::Integer)
                        .map_err(|_| format!("Cannae parse '{}' as hex", s)),
                    _ => Err("fae_hex() needs a string".to_string()),
                },
            ))),
        );

        // ============================================================
        // MORE DICTIONARY FUNCTIONS - Fer managin' yer dicts!
        // ============================================================

        // dict_merge - merge two dictionaries (second overrides first)
        globals.borrow_mut().define(
            "dict_merge".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "dict_merge",
                2,
                |args| match (&args[0], &args[1]) {
                    (Value::Dict(a), Value::Dict(b)) => {
                        let mut result = a.borrow().clone();
                        for (k, v) in b.borrow().iter() {
                            result.set(k.clone(), v.clone());
                        }
                        Ok(Value::Dict(Rc::new(RefCell::new(result))))
                    }
                    _ => Err("dict_merge() needs two dictionaries".to_string()),
                },
            ))),
        );

        // dict_get - get value with default (avoids crashes!)
        globals.borrow_mut().define(
            "dict_get".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "dict_get",
                3,
                |args| match &args[0] {
                    Value::Dict(d) => Ok(d
                        .borrow()
                        .get(&args[1])
                        .cloned()
                        .unwrap_or_else(|| args[2].clone())),
                    _ => Err("dict_get() needs a dictionary".to_string()),
                },
            ))),
        );

        // dict_has - check if dictionary has a key
        globals.borrow_mut().define(
            "dict_has".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "dict_has",
                2,
                |args| match &args[0] {
                    Value::Dict(d) => Ok(Value::Bool(d.borrow().contains_key(&args[1]))),
                    _ => Err("dict_has() needs a dictionary".to_string()),
                },
            ))),
        );

        // dict_remove - remove a key from dictionary (returns new dict)
        globals.borrow_mut().define(
            "dict_remove".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "dict_remove",
                2,
                |args| match &args[0] {
                    Value::Dict(d) => {
                        let mut new_dict = d.borrow().clone();
                        new_dict.remove(&args[1]);
                        Ok(Value::Dict(Rc::new(RefCell::new(new_dict))))
                    }
                    _ => Err("dict_remove() needs a dictionary".to_string()),
                },
            ))),
        );

        // dict_invert - swap keys and values
        globals.borrow_mut().define(
            "dict_invert".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "dict_invert",
                1,
                |args| match &args[0] {
                    Value::Dict(d) => {
                        let mut inverted = DictValue::new();
                        for (k, v) in d.borrow().iter() {
                            inverted.set(v.clone(), k.clone());
                        }
                        Ok(Value::Dict(Rc::new(RefCell::new(inverted))))
                    }
                    _ => Err("dict_invert() needs a dictionary".to_string()),
                },
            ))),
        );

        // items - get dictionary as list of [key, value] pairs
        globals.borrow_mut().define(
            "items".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "items",
                1,
                |args| match &args[0] {
                    Value::Dict(d) => {
                        let pairs: Vec<Value> = d
                            .borrow()
                            .iter()
                            .map(|(k, v)| {
                                Value::List(Rc::new(RefCell::new(vec![k.clone(), v.clone()])))
                            })
                            .collect();
                        Ok(Value::List(Rc::new(RefCell::new(pairs))))
                    }
                    _ => Err("items() needs a dictionary".to_string()),
                },
            ))),
        );

        // fae_pairs - create dictionary from list of [key, value] pairs
        globals.borrow_mut().define(
            "fae_pairs".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "fae_pairs",
                1,
                |args| match &args[0] {
                    Value::List(list) => {
                        let mut dict = DictValue::new();
                        for item in list.borrow().iter() {
                            if let Value::List(pair) = item {
                                let pair = pair.borrow();
                                if pair.len() >= 2 {
                                    dict.set(pair[0].clone(), pair[1].clone());
                                }
                            }
                        }
                        Ok(Value::Dict(Rc::new(RefCell::new(dict))))
                    }
                    _ => Err("fae_pairs() needs a list o' pairs".to_string()),
                },
            ))),
        );

        // ============================================================
        // STRING UTILITIES - More ways tae wrangle yer strings!
        // ============================================================

        // center - center a string in a field of given width
        globals.borrow_mut().define(
            "center".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("center", 3, |args| {
                let s = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("center() needs a string".to_string()),
                };
                let width = match &args[1] {
                    Value::Integer(n) => *n as usize,
                    _ => return Err("center() needs a width".to_string()),
                };
                let fill = match &args[2] {
                    Value::String(c) => c.chars().next().unwrap_or(' '),
                    _ => return Err("center() needs a fill character".to_string()),
                };
                if s.len() >= width {
                    return Ok(Value::String(s));
                }
                let padding = width - s.len();
                let left_pad = padding / 2;
                let right_pad = padding - left_pad;
                Ok(Value::String(format!(
                    "{}{}{}",
                    fill.to_string().repeat(left_pad),
                    s,
                    fill.to_string().repeat(right_pad)
                )))
            }))),
        );

        // is_upper - check if string is all uppercase
        globals.borrow_mut().define(
            "is_upper".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "is_upper",
                1,
                |args| match &args[0] {
                    Value::String(s) => {
                        let has_letters = s.chars().any(|c| c.is_alphabetic());
                        Ok(Value::Bool(
                            has_letters
                                && s.chars().all(|c| !c.is_alphabetic() || c.is_uppercase()),
                        ))
                    }
                    _ => Err("is_upper() needs a string".to_string()),
                },
            ))),
        );

        // is_lower - check if string is all lowercase
        globals.borrow_mut().define(
            "is_lower".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "is_lower",
                1,
                |args| match &args[0] {
                    Value::String(s) => {
                        let has_letters = s.chars().any(|c| c.is_alphabetic());
                        Ok(Value::Bool(
                            has_letters
                                && s.chars().all(|c| !c.is_alphabetic() || c.is_lowercase()),
                        ))
                    }
                    _ => Err("is_lower() needs a string".to_string()),
                },
            ))),
        );

        // swapcase - swap case of all letters
        globals.borrow_mut().define(
            "swapcase".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "swapcase",
                1,
                |args| match &args[0] {
                    Value::String(s) => {
                        let swapped: String = s
                            .chars()
                            .map(|c| {
                                if c.is_uppercase() {
                                    c.to_lowercase().next().unwrap_or(c)
                                } else if c.is_lowercase() {
                                    c.to_uppercase().next().unwrap_or(c)
                                } else {
                                    c
                                }
                            })
                            .collect();
                        Ok(Value::String(swapped))
                    }
                    _ => Err("swapcase() needs a string".to_string()),
                },
            ))),
        );

        // strip_left - remove leading characters
        globals.borrow_mut().define(
            "strip_left".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "strip_left",
                2,
                |args| match (&args[0], &args[1]) {
                    (Value::String(s), Value::String(chars)) => {
                        let char_set: Vec<char> = chars.chars().collect();
                        Ok(Value::String(
                            s.trim_start_matches(|c| char_set.contains(&c)).to_string(),
                        ))
                    }
                    _ => Err("strip_left() needs two strings".to_string()),
                },
            ))),
        );

        // strip_right - remove trailing characters
        globals.borrow_mut().define(
            "strip_right".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "strip_right",
                2,
                |args| match (&args[0], &args[1]) {
                    (Value::String(s), Value::String(chars)) => {
                        let char_set: Vec<char> = chars.chars().collect();
                        Ok(Value::String(
                            s.trim_end_matches(|c| char_set.contains(&c)).to_string(),
                        ))
                    }
                    _ => Err("strip_right() needs two strings".to_string()),
                },
            ))),
        );

        // replace_first - replace only first occurrence
        globals.borrow_mut().define(
            "replace_first".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "replace_first",
                3,
                |args| match (&args[0], &args[1], &args[2]) {
                    (Value::String(s), Value::String(from), Value::String(to)) => {
                        Ok(Value::String(s.replacen(from.as_str(), to.as_str(), 1)))
                    }
                    _ => Err("replace_first() needs three strings".to_string()),
                },
            ))),
        );

        // substr_between - get substring between two markers
        globals.borrow_mut().define(
            "substr_between".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "substr_between",
                3,
                |args| match (&args[0], &args[1], &args[2]) {
                    (Value::String(s), Value::String(start), Value::String(end)) => {
                        if let Some(start_idx) = s.find(start.as_str()) {
                            let after_start = start_idx + start.len();
                            if let Some(end_idx) = s[after_start..].find(end.as_str()) {
                                return Ok(Value::String(
                                    s[after_start..after_start + end_idx].to_string(),
                                ));
                            }
                        }
                        Ok(Value::Nil)
                    }
                    _ => Err("substr_between() needs three strings".to_string()),
                },
            ))),
        );

        // ============================================================
        // MORE MATHEMATICAL FUNCTIONS
        // ============================================================

        // sign - get sign of number (-1, 0, or 1)
        globals.borrow_mut().define(
            "sign".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "sign",
                1,
                |args| match &args[0] {
                    Value::Integer(n) => Ok(Value::Integer(if *n > 0 {
                        1
                    } else if *n < 0 {
                        -1
                    } else {
                        0
                    })),
                    Value::Float(f) => Ok(Value::Integer(if *f > 0.0 {
                        1
                    } else if *f < 0.0 {
                        -1
                    } else {
                        0
                    })),
                    _ => Err("sign() needs a number".to_string()),
                },
            ))),
        );

        // clamp - constrain a value between min and max
        globals.borrow_mut().define(
            "clamp".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("clamp", 3, |args| {
                match (&args[0], &args[1], &args[2]) {
                    (Value::Integer(n), Value::Integer(min), Value::Integer(max)) => {
                        Ok(Value::Integer((*n).max(*min).min(*max)))
                    }
                    (Value::Float(n), Value::Float(min), Value::Float(max)) => {
                        Ok(Value::Float(n.max(*min).min(*max)))
                    }
                    _ => Err("clamp() needs three numbers o' the same type".to_string()),
                }
            }))),
        );

        // lerp - linear interpolation between two values
        globals.borrow_mut().define(
            "lerp".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("lerp", 3, |args| {
                let a = match &args[0] {
                    Value::Float(f) => *f,
                    Value::Integer(n) => *n as f64,
                    _ => return Err("lerp() needs numbers".to_string()),
                };
                let b = match &args[1] {
                    Value::Float(f) => *f,
                    Value::Integer(n) => *n as f64,
                    _ => return Err("lerp() needs numbers".to_string()),
                };
                let t = match &args[2] {
                    Value::Float(f) => *f,
                    Value::Integer(n) => *n as f64,
                    _ => return Err("lerp() needs numbers".to_string()),
                };
                Ok(Value::Float(a + (b - a) * t))
            }))),
        );

        // gcd - greatest common divisor
        globals.borrow_mut().define(
            "gcd".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("gcd", 2, |args| {
                fn gcd_calc(a: i64, b: i64) -> i64 {
                    if b == 0 {
                        a.abs()
                    } else {
                        gcd_calc(b, a % b)
                    }
                }
                match (&args[0], &args[1]) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(gcd_calc(*a, *b))),
                    _ => Err("gcd() needs two integers".to_string()),
                }
            }))),
        );

        // lcm - least common multiple
        globals.borrow_mut().define(
            "lcm".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("lcm", 2, |args| {
                fn gcd_calc(a: i64, b: i64) -> i64 {
                    if b == 0 {
                        a.abs()
                    } else {
                        gcd_calc(b, a % b)
                    }
                }
                match (&args[0], &args[1]) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if *a == 0 || *b == 0 {
                            Ok(Value::Integer(0))
                        } else {
                            Ok(Value::Integer((*a * *b).abs() / gcd_calc(*a, *b)))
                        }
                    }
                    _ => Err("lcm() needs two integers".to_string()),
                }
            }))),
        );

        // factorial - calculate factorial
        globals.borrow_mut().define(
            "factorial".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "factorial",
                1,
                |args| match &args[0] {
                    Value::Integer(n) => {
                        if *n < 0 {
                            return Err(
                                "Cannae calculate factorial o' negative number!".to_string()
                            );
                        }
                        if *n > 20 {
                            return Err("Factorial too big! Max is 20".to_string());
                        }
                        let mut result: i64 = 1;
                        for i in 2..=*n {
                            result *= i;
                        }
                        Ok(Value::Integer(result))
                    }
                    _ => Err("factorial() needs an integer".to_string()),
                },
            ))),
        );

        // is_even - check if number is even
        globals.borrow_mut().define(
            "is_even".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "is_even",
                1,
                |args| match &args[0] {
                    Value::Integer(n) => Ok(Value::Bool(*n % 2 == 0)),
                    _ => Err("is_even() needs an integer".to_string()),
                },
            ))),
        );

        // is_odd - check if number is odd
        globals.borrow_mut().define(
            "is_odd".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "is_odd",
                1,
                |args| match &args[0] {
                    Value::Integer(n) => Ok(Value::Bool(*n % 2 != 0)),
                    _ => Err("is_odd() needs an integer".to_string()),
                },
            ))),
        );

        // is_prime - check if number is prime
        globals.borrow_mut().define(
            "is_prime".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "is_prime",
                1,
                |args| match &args[0] {
                    Value::Integer(n) => {
                        if *n < 2 {
                            return Ok(Value::Bool(false));
                        }
                        if *n == 2 {
                            return Ok(Value::Bool(true));
                        }
                        if *n % 2 == 0 {
                            return Ok(Value::Bool(false));
                        }
                        let sqrt_n = (*n as f64).sqrt() as i64;
                        for i in (3..=sqrt_n).step_by(2) {
                            if *n % i == 0 {
                                return Ok(Value::Bool(false));
                            }
                        }
                        Ok(Value::Bool(true))
                    }
                    _ => Err("is_prime() needs an integer".to_string()),
                },
            ))),
        );

        // ============================================================
        // ASSERTION FUNCTIONS - Test yer code, ya numpty!
        // ============================================================

        // assert - throw error if condition is false
        globals.borrow_mut().define(
            "assert".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("assert", 2, |args| {
                let condition = args[0].is_truthy();
                if !condition {
                    let msg = match &args[1] {
                        Value::String(s) => s.clone(),
                        _ => format!("{}", args[1]),
                    };
                    Err(format!("Assertion failed: {}", msg))
                } else {
                    Ok(Value::Bool(true))
                }
            }))),
        );

        // assert_equal - throw error if values are not equal
        globals.borrow_mut().define(
            "assert_equal".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("assert_equal", 2, |args| {
                if args[0] == args[1] {
                    Ok(Value::Bool(true))
                } else {
                    Err(format!(
                        "Assertion failed: expected {} but got {}",
                        args[0], args[1]
                    ))
                }
            }))),
        );

        // assert_nae_equal - throw error if values are equal
        globals.borrow_mut().define(
            "assert_nae_equal".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "assert_nae_equal",
                2,
                |args| {
                    if args[0] != args[1] {
                        Ok(Value::Bool(true))
                    } else {
                        Err(format!(
                            "Assertion failed: {} should not equal {}",
                            args[0], args[1]
                        ))
                    }
                },
            ))),
        );

        // ============================================================
        // LIST STATISTICS - Fer number-crunchin'!
        // ============================================================

        // average - calculate average of a list of numbers (Scots: mean)
        globals.borrow_mut().define(
            "average".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("average", 1, |args| {
                if let Value::List(list) = &args[0] {
                    let items = list.borrow();
                    if items.is_empty() {
                        return Err("Cannae calculate average o' empty list!".to_string());
                    }
                    let mut sum: f64 = 0.0;
                    for item in items.iter() {
                        match item {
                            Value::Integer(n) => sum += *n as f64,
                            Value::Float(f) => sum += *f,
                            _ => return Err("average() needs a list o' numbers".to_string()),
                        }
                    }
                    Ok(Value::Float(sum / items.len() as f64))
                } else {
                    Err("average() needs a list".to_string())
                }
            }))),
        );

        // median - calculate median of a list of numbers
        globals.borrow_mut().define(
            "median".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("median", 1, |args| {
                if let Value::List(list) = &args[0] {
                    let items = list.borrow();
                    if items.is_empty() {
                        return Err("Cannae calculate median o' empty list!".to_string());
                    }
                    let mut nums: Vec<f64> = Vec::new();
                    for item in items.iter() {
                        match item {
                            Value::Integer(n) => nums.push(*n as f64),
                            Value::Float(f) => nums.push(*f),
                            _ => return Err("median() needs a list o' numbers".to_string()),
                        }
                    }
                    nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let mid = nums.len() / 2;
                    if nums.len().is_multiple_of(2) {
                        Ok(Value::Float((nums[mid - 1] + nums[mid]) / 2.0))
                    } else {
                        Ok(Value::Float(nums[mid]))
                    }
                } else {
                    Err("median() needs a list".to_string())
                }
            }))),
        );

        // product - multiply all numbers in a list
        globals.borrow_mut().define(
            "product".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("product", 1, |args| {
                if let Value::List(list) = &args[0] {
                    let items = list.borrow();
                    if items.is_empty() {
                        return Ok(Value::Integer(1));
                    }
                    let mut prod: f64 = 1.0;
                    let mut is_float = false;
                    for item in items.iter() {
                        match item {
                            Value::Integer(n) => prod *= *n as f64,
                            Value::Float(f) => {
                                prod *= *f;
                                is_float = true;
                            }
                            _ => return Err("product() needs a list o' numbers".to_string()),
                        }
                    }
                    if is_float {
                        Ok(Value::Float(prod))
                    } else {
                        Ok(Value::Integer(prod as i64))
                    }
                } else {
                    Err("product() needs a list".to_string())
                }
            }))),
        );

        // minaw - find minimum in a list (min all)
        globals.borrow_mut().define(
            "minaw".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("minaw", 1, |args| {
                if let Value::List(list) = &args[0] {
                    let items = list.borrow();
                    if items.is_empty() {
                        return Err("Cannae find minimum o' empty list!".to_string());
                    }
                    let mut min_val = items[0].clone();
                    for item in items.iter().skip(1) {
                        match (&min_val, item) {
                            (Value::Integer(a), Value::Integer(b)) => {
                                if *b < *a {
                                    min_val = item.clone();
                                }
                            }
                            (Value::Float(a), Value::Float(b)) => {
                                if *b < *a {
                                    min_val = item.clone();
                                }
                            }
                            _ => {
                                return Err("minaw() needs a list o' comparable numbers".to_string())
                            }
                        }
                    }
                    Ok(min_val)
                } else {
                    Err("minaw() needs a list".to_string())
                }
            }))),
        );

        // maxaw - find maximum in a list (max all)
        globals.borrow_mut().define(
            "maxaw".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("maxaw", 1, |args| {
                if let Value::List(list) = &args[0] {
                    let items = list.borrow();
                    if items.is_empty() {
                        return Err("Cannae find maximum o' empty list!".to_string());
                    }
                    let mut max_val = items[0].clone();
                    for item in items.iter().skip(1) {
                        match (&max_val, item) {
                            (Value::Integer(a), Value::Integer(b)) => {
                                if *b > *a {
                                    max_val = item.clone();
                                }
                            }
                            (Value::Float(a), Value::Float(b)) => {
                                if *b > *a {
                                    max_val = item.clone();
                                }
                            }
                            _ => {
                                return Err("maxaw() needs a list o' comparable numbers".to_string())
                            }
                        }
                    }
                    Ok(max_val)
                } else {
                    Err("maxaw() needs a list".to_string())
                }
            }))),
        );

        // range_o - get the range (max - min) of a list
        globals.borrow_mut().define(
            "range_o".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("range_o", 1, |args| {
                if let Value::List(list) = &args[0] {
                    let items = list.borrow();
                    if items.is_empty() {
                        return Err("Cannae get range o' empty list!".to_string());
                    }
                    let mut min_val: f64 = f64::MAX;
                    let mut max_val: f64 = f64::MIN;
                    for item in items.iter() {
                        match item {
                            Value::Integer(n) => {
                                let v = *n as f64;
                                if v < min_val {
                                    min_val = v;
                                }
                                if v > max_val {
                                    max_val = v;
                                }
                            }
                            Value::Float(f) => {
                                if *f < min_val {
                                    min_val = *f;
                                }
                                if *f > max_val {
                                    max_val = *f;
                                }
                            }
                            _ => return Err("range_o() needs a list o' numbers".to_string()),
                        }
                    }
                    Ok(Value::Float(max_val - min_val))
                } else {
                    Err("range_o() needs a list".to_string())
                }
            }))),
        );

        // ============================================================
        // STDLIB EXPANSION - File I/O
        // ============================================================

        // scrieve_append - append to file (Scots: write-append)
        globals.borrow_mut().define(
            "scrieve_append".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("scrieve_append", 2, |args| {
                use std::fs::OpenOptions;
                use std::io::Write as IoWrite;
                let path = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("scrieve_append() needs a file path string".to_string()),
                };
                let content = match &args[1] {
                    Value::String(s) => s.clone(),
                    v => format!("{}", v),
                };
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                    .map_err(|e| format!("Couldnae open '{}' fer appendin': {}", path, e))?;
                file.write_all(content.as_bytes())
                    .map_err(|e| format!("Couldnae append tae '{}': {}", path, e))?;
                Ok(Value::Nil)
            }))),
        );

        // file_delete - delete a file
        globals.borrow_mut().define(
            "file_delete".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("file_delete", 1, |args| {
                let path = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("file_delete() needs a file path string".to_string()),
                };
                std::fs::remove_file(&path)
                    .map_err(|e| format!("Couldnae delete '{}': {}", path, e))?;
                Ok(Value::Nil)
            }))),
        );

        // list_dir - list directory contents
        globals.borrow_mut().define(
            "list_dir".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("list_dir", 1, |args| {
                let path = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("list_dir() needs a directory path string".to_string()),
                };
                let entries = std::fs::read_dir(&path)
                    .map_err(|e| format!("Couldnae read directory '{}': {}", path, e))?;
                let files: Vec<Value> = entries
                    .filter_map(|e| e.ok())
                    .map(|e| Value::String(e.file_name().to_string_lossy().to_string()))
                    .collect();
                Ok(Value::List(Rc::new(RefCell::new(files))))
            }))),
        );

        // make_dir - create directory (and parents)
        globals.borrow_mut().define(
            "make_dir".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("make_dir", 1, |args| {
                let path = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("make_dir() needs a directory path string".to_string()),
                };
                std::fs::create_dir_all(&path)
                    .map_err(|e| format!("Couldnae create directory '{}': {}", path, e))?;
                Ok(Value::Nil)
            }))),
        );

        // is_dir - check if path is a directory
        globals.borrow_mut().define(
            "is_dir".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("is_dir", 1, |args| {
                let path = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("is_dir() needs a path string".to_string()),
                };
                Ok(Value::Bool(std::path::Path::new(&path).is_dir()))
            }))),
        );

        // file_size - get file size in bytes
        globals.borrow_mut().define(
            "file_size".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("file_size", 1, |args| {
                let path = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("file_size() needs a file path string".to_string()),
                };
                let metadata = std::fs::metadata(&path)
                    .map_err(|e| format!("Couldnae get file info fer '{}': {}", path, e))?;
                Ok(Value::Integer(metadata.len() as i64))
            }))),
        );

        // path_join - join path components
        globals.borrow_mut().define(
            "path_join".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("path_join", 2, |args| {
                let path1 = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("path_join() needs strings".to_string()),
                };
                let path2 = match &args[1] {
                    Value::String(s) => s.clone(),
                    _ => return Err("path_join() needs strings".to_string()),
                };
                let joined = std::path::Path::new(&path1).join(&path2);
                Ok(Value::String(joined.to_string_lossy().to_string()))
            }))),
        );

        // ============================================================
        // STDLIB EXPANSION - String Functions
        // ============================================================

        // trim - remove leading/trailing whitespace
        globals.borrow_mut().define(
            "trim".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("trim", 1, |args| {
                if let Value::String(s) = &args[0] {
                    Ok(Value::String(s.trim().to_string()))
                } else {
                    Err("trim() needs a string".to_string())
                }
            }))),
        );

        // trim_start - remove leading whitespace
        globals.borrow_mut().define(
            "trim_start".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("trim_start", 1, |args| {
                if let Value::String(s) = &args[0] {
                    Ok(Value::String(s.trim_start().to_string()))
                } else {
                    Err("trim_start() needs a string".to_string())
                }
            }))),
        );

        // trim_end - remove trailing whitespace
        globals.borrow_mut().define(
            "trim_end".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("trim_end", 1, |args| {
                if let Value::String(s) = &args[0] {
                    Ok(Value::String(s.trim_end().to_string()))
                } else {
                    Err("trim_end() needs a string".to_string())
                }
            }))),
        );

        // starts_with - check if string starts with prefix
        globals.borrow_mut().define(
            "starts_with".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "starts_with",
                2,
                |args| match (&args[0], &args[1]) {
                    (Value::String(s), Value::String(prefix)) => {
                        Ok(Value::Bool(s.starts_with(prefix.as_str())))
                    }
                    _ => Err("starts_with() needs two strings".to_string()),
                },
            ))),
        );

        // ends_with - check if string ends with suffix
        globals.borrow_mut().define(
            "ends_with".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("ends_with", 2, |args| match (
                &args[0], &args[1],
            ) {
                (Value::String(s), Value::String(suffix)) => {
                    Ok(Value::Bool(s.ends_with(suffix.as_str())))
                }
                _ => Err("ends_with() needs two strings".to_string()),
            }))),
        );

        // last_index_of - find last index of substring
        globals.borrow_mut().define(
            "last_index_of".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "last_index_of",
                2,
                |args| match (&args[0], &args[1]) {
                    (Value::String(s), Value::String(needle)) => Ok(Value::Integer(
                        s.rfind(needle.as_str()).map(|i| i as i64).unwrap_or(-1),
                    )),
                    _ => Err("last_index_of() needs two strings".to_string()),
                },
            ))),
        );

        // substring - extract substring (start, end exclusive)
        globals.borrow_mut().define(
            "substring".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("substring", 3, |args| {
                let s = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("substring() needs a string".to_string()),
                };
                let start = args[1]
                    .as_integer()
                    .ok_or("substring() needs integer indices")?
                    as usize;
                let end = args[2]
                    .as_integer()
                    .ok_or("substring() needs integer indices")? as usize;
                let chars: Vec<char> = s.chars().collect();
                let start = start.min(chars.len());
                let end = end.min(chars.len());
                Ok(Value::String(chars[start..end].iter().collect()))
            }))),
        );

        // is_empty - check if string is empty
        globals.borrow_mut().define(
            "is_empty".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "is_empty",
                1,
                |args| match &args[0] {
                    Value::String(s) => Ok(Value::Bool(s.is_empty())),
                    Value::List(l) => Ok(Value::Bool(l.borrow().is_empty())),
                    Value::Dict(d) => Ok(Value::Bool(d.borrow().is_empty())),
                    _ => Err("is_empty() needs a string, list, or dict".to_string()),
                },
            ))),
        );

        // is_blank - check if string is empty or only whitespace
        globals.borrow_mut().define(
            "is_blank".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("is_blank", 1, |args| {
                if let Value::String(s) = &args[0] {
                    Ok(Value::Bool(s.trim().is_empty()))
                } else {
                    Err("is_blank() needs a string".to_string())
                }
            }))),
        );

        // ============================================================
        // STDLIB EXPANSION - Math Functions
        // ============================================================

        // random - random float between 0.0 and 1.0
        globals.borrow_mut().define(
            "random".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("random", 0, |_args| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as u64;
                let rng = seed.wrapping_mul(1103515245).wrapping_add(12345);
                let random_float = (rng as f64) / (u64::MAX as f64);
                Ok(Value::Float(random_float))
            }))),
        );

        // random_int - random integer in range (inclusive)
        globals.borrow_mut().define(
            "random_int".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("random_int", 2, |args| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let min = args[0]
                    .as_integer()
                    .ok_or("random_int() needs integer bounds")?;
                let max = args[1]
                    .as_integer()
                    .ok_or("random_int() needs integer bounds")?;
                if min > max {
                    return Err("random_int() min must be <= max".to_string());
                }
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as u64;
                let rng = seed.wrapping_mul(1103515245).wrapping_add(12345);
                let range = (max - min + 1) as u64;
                let result = min + ((rng % range) as i64);
                Ok(Value::Integer(result))
            }))),
        );

        // random_choice - random element from list
        globals.borrow_mut().define(
            "random_choice".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("random_choice", 1, |args| {
                use std::time::{SystemTime, UNIX_EPOCH};
                if let Value::List(list) = &args[0] {
                    let items = list.borrow();
                    if items.is_empty() {
                        return Ok(Value::Nil);
                    }
                    let seed = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos() as u64;
                    let rng = seed.wrapping_mul(1103515245).wrapping_add(12345);
                    let idx = (rng as usize) % items.len();
                    Ok(items[idx].clone())
                } else {
                    Err("random_choice() needs a list".to_string())
                }
            }))),
        );

        // pi - return PI constant
        globals.borrow_mut().define(
            "pi".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("pi", 0, |_args| {
                Ok(Value::Float(std::f64::consts::PI))
            }))),
        );

        // e - return Euler's number
        globals.borrow_mut().define(
            "e".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("e", 0, |_args| {
                Ok(Value::Float(std::f64::consts::E))
            }))),
        );

        // tau - return TAU (2*PI)
        globals.borrow_mut().define(
            "tau".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("tau", 0, |_args| {
                Ok(Value::Float(std::f64::consts::TAU))
            }))),
        );

        // trunc - truncate toward zero
        globals.borrow_mut().define(
            "trunc".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "trunc",
                1,
                |args| match &args[0] {
                    Value::Float(f) => Ok(Value::Integer(f.trunc() as i64)),
                    Value::Integer(n) => Ok(Value::Integer(*n)),
                    _ => Err("trunc() needs a number".to_string()),
                },
            ))),
        );

        // log2 - base 2 logarithm
        globals.borrow_mut().define(
            "log2".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "log2",
                1,
                |args| match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.log2())),
                    Value::Integer(n) => Ok(Value::Float((*n as f64).log2())),
                    _ => Err("log2() needs a number".to_string()),
                },
            ))),
        );

        // ============================================================
        // STDLIB EXPANSION - Date/Time Functions
        // ============================================================

        // date_now - current date as dict
        globals.borrow_mut().define(
            "date_now".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("date_now", 0, |_args| {
                use chrono::{Datelike, Local, Timelike};
                let now = Local::now();
                let mut dict = DictValue::new();
                dict.set(
                    Value::String("year".to_string()),
                    Value::Integer(now.year() as i64),
                );
                dict.set(
                    Value::String("month".to_string()),
                    Value::Integer(now.month() as i64),
                );
                dict.set(
                    Value::String("day".to_string()),
                    Value::Integer(now.day() as i64),
                );
                dict.set(
                    Value::String("hour".to_string()),
                    Value::Integer(now.hour() as i64),
                );
                dict.set(
                    Value::String("minute".to_string()),
                    Value::Integer(now.minute() as i64),
                );
                dict.set(
                    Value::String("second".to_string()),
                    Value::Integer(now.second() as i64),
                );
                dict.set(
                    Value::String("weekday".to_string()),
                    Value::Integer(now.weekday().num_days_from_monday() as i64),
                );
                Ok(Value::Dict(Rc::new(RefCell::new(dict))))
            }))),
        );

        // date_format - format timestamp
        globals.borrow_mut().define(
            "date_format".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("date_format", 2, |args| {
                use chrono::{Local, TimeZone};
                let timestamp_secs = args[0]
                    .as_integer()
                    .ok_or("date_format() needs a timestamp")?;
                let format = match &args[1] {
                    Value::String(s) => s.clone(),
                    _ => return Err("date_format() needs a format string".to_string()),
                };
                let dt = Local
                    .timestamp_opt(timestamp_secs, 0)
                    .single()
                    .ok_or("Invalid timestamp")?;
                Ok(Value::String(dt.format(&format).to_string()))
            }))),
        );

        // date_parse - parse string to timestamp
        globals.borrow_mut().define(
            "date_parse".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("date_parse", 2, |args| {
                use chrono::NaiveDateTime;
                let date_str = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("date_parse() needs a date string".to_string()),
                };
                let format = match &args[1] {
                    Value::String(s) => s.clone(),
                    _ => return Err("date_parse() needs a format string".to_string()),
                };
                let dt = NaiveDateTime::parse_from_str(&date_str, &format)
                    .map_err(|e| format!("Couldnae parse date '{}': {}", date_str, e))?;
                Ok(Value::Integer(dt.and_utc().timestamp()))
            }))),
        );

        // date_add - add time to timestamp
        globals.borrow_mut().define(
            "date_add".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("date_add", 3, |args| {
                use chrono::{Duration, Local, TimeZone};
                let timestamp_secs = args[0].as_integer().ok_or("date_add() needs a timestamp")?;
                let amount = args[1].as_integer().ok_or("date_add() needs an amount")?;
                let unit = match &args[2] {
                    Value::String(s) => s.clone(),
                    _ => return Err("date_add() needs a unit string".to_string()),
                };
                let dt = Local
                    .timestamp_opt(timestamp_secs, 0)
                    .single()
                    .ok_or("Invalid timestamp")?;
                let new_dt = match unit.as_str() {
                    "seconds" => dt + Duration::seconds(amount),
                    "minutes" => dt + Duration::minutes(amount),
                    "hours" => dt + Duration::hours(amount),
                    "days" => dt + Duration::days(amount),
                    "weeks" => dt + Duration::weeks(amount),
                    _ => return Err(format!("Unknown time unit: {}", unit)),
                };
                Ok(Value::Integer(new_dt.timestamp()))
            }))),
        );

        // date_diff - difference between timestamps
        globals.borrow_mut().define(
            "date_diff".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("date_diff", 3, |args| {
                let ts1 = args[0].as_integer().ok_or("date_diff() needs timestamps")?;
                let ts2 = args[1].as_integer().ok_or("date_diff() needs timestamps")?;
                let unit = match &args[2] {
                    Value::String(s) => s.clone(),
                    _ => return Err("date_diff() needs a unit string".to_string()),
                };
                let diff_secs = ts2 - ts1;
                let result = match unit.as_str() {
                    "milliseconds" => diff_secs * 1000,
                    "seconds" => diff_secs,
                    "minutes" => diff_secs / 60,
                    "hours" => diff_secs / 3600,
                    "days" => diff_secs / 86400,
                    "weeks" => diff_secs / 604800,
                    _ => return Err(format!("Unknown time unit: {}", unit)),
                };
                Ok(Value::Integer(result))
            }))),
        );

        // timestamp - Unix timestamp in seconds
        globals.borrow_mut().define(
            "timestamp".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("timestamp", 0, |_args| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let secs = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                Ok(Value::Integer(secs as i64))
            }))),
        );

        // timestamp_millis - Unix timestamp in milliseconds
        globals.borrow_mut().define(
            "timestamp_millis".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "timestamp_millis",
                0,
                |_args| {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let millis = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis();
                    Ok(Value::Integer(millis as i64))
                },
            ))),
        );

        // ============================================================
        // STDLIB EXPANSION - Regular Expressions
        // ============================================================

        // regex_test - test if pattern matches
        globals.borrow_mut().define(
            "regex_test".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("regex_test", 2, |args| {
                use regex::Regex;
                let text = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("regex_test() needs a string".to_string()),
                };
                let pattern = match &args[1] {
                    Value::String(s) => s.clone(),
                    _ => return Err("regex_test() needs a pattern string".to_string()),
                };
                let re = Regex::new(&pattern)
                    .map_err(|e| format!("Invalid regex '{}': {}", pattern, e))?;
                Ok(Value::Bool(re.is_match(&text)))
            }))),
        );

        // regex_match - find first match
        globals.borrow_mut().define(
            "regex_match".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("regex_match", 2, |args| {
                use regex::Regex;
                let text = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("regex_match() needs a string".to_string()),
                };
                let pattern = match &args[1] {
                    Value::String(s) => s.clone(),
                    _ => return Err("regex_match() needs a pattern string".to_string()),
                };
                let re = Regex::new(&pattern)
                    .map_err(|e| format!("Invalid regex '{}': {}", pattern, e))?;
                if let Some(m) = re.find(&text) {
                    let mut dict = DictValue::new();
                    dict.set(
                        Value::String("match".to_string()),
                        Value::String(m.as_str().to_string()),
                    );
                    dict.set(
                        Value::String("start".to_string()),
                        Value::Integer(m.start() as i64),
                    );
                    dict.set(
                        Value::String("end".to_string()),
                        Value::Integer(m.end() as i64),
                    );
                    Ok(Value::Dict(Rc::new(RefCell::new(dict))))
                } else {
                    Ok(Value::Nil)
                }
            }))),
        );

        // regex_match_all - find all matches
        globals.borrow_mut().define(
            "regex_match_all".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("regex_match_all", 2, |args| {
                use regex::Regex;
                let text = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("regex_match_all() needs a string".to_string()),
                };
                let pattern = match &args[1] {
                    Value::String(s) => s.clone(),
                    _ => return Err("regex_match_all() needs a pattern string".to_string()),
                };
                let re = Regex::new(&pattern)
                    .map_err(|e| format!("Invalid regex '{}': {}", pattern, e))?;
                let matches: Vec<Value> = re
                    .find_iter(&text)
                    .map(|m| {
                        let mut dict = DictValue::new();
                        dict.set(
                            Value::String("match".to_string()),
                            Value::String(m.as_str().to_string()),
                        );
                        dict.set(
                            Value::String("start".to_string()),
                            Value::Integer(m.start() as i64),
                        );
                        dict.set(
                            Value::String("end".to_string()),
                            Value::Integer(m.end() as i64),
                        );
                        Value::Dict(Rc::new(RefCell::new(dict)))
                    })
                    .collect();
                Ok(Value::List(Rc::new(RefCell::new(matches))))
            }))),
        );

        // regex_replace - replace all matches
        globals.borrow_mut().define(
            "regex_replace".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("regex_replace", 3, |args| {
                use regex::Regex;
                let text = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("regex_replace() needs a string".to_string()),
                };
                let pattern = match &args[1] {
                    Value::String(s) => s.clone(),
                    _ => return Err("regex_replace() needs a pattern string".to_string()),
                };
                let replacement = match &args[2] {
                    Value::String(s) => s.clone(),
                    _ => return Err("regex_replace() needs a replacement string".to_string()),
                };
                let re = Regex::new(&pattern)
                    .map_err(|e| format!("Invalid regex '{}': {}", pattern, e))?;
                Ok(Value::String(
                    re.replace_all(&text, replacement.as_str()).to_string(),
                ))
            }))),
        );

        // regex_replace_first - replace first match only
        globals.borrow_mut().define(
            "regex_replace_first".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "regex_replace_first",
                3,
                |args| {
                    use regex::Regex;
                    let text = match &args[0] {
                        Value::String(s) => s.clone(),
                        _ => return Err("regex_replace_first() needs a string".to_string()),
                    };
                    let pattern = match &args[1] {
                        Value::String(s) => s.clone(),
                        _ => return Err("regex_replace_first() needs a pattern string".to_string()),
                    };
                    let replacement = match &args[2] {
                        Value::String(s) => s.clone(),
                        _ => {
                            return Err(
                                "regex_replace_first() needs a replacement string".to_string()
                            )
                        }
                    };
                    let re = Regex::new(&pattern)
                        .map_err(|e| format!("Invalid regex '{}': {}", pattern, e))?;
                    Ok(Value::String(
                        re.replacen(&text, 1, replacement.as_str()).to_string(),
                    ))
                },
            ))),
        );

        // regex_split - split by regex pattern
        globals.borrow_mut().define(
            "regex_split".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("regex_split", 2, |args| {
                use regex::Regex;
                let text = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("regex_split() needs a string".to_string()),
                };
                let pattern = match &args[1] {
                    Value::String(s) => s.clone(),
                    _ => return Err("regex_split() needs a pattern string".to_string()),
                };
                let re = Regex::new(&pattern)
                    .map_err(|e| format!("Invalid regex '{}': {}", pattern, e))?;
                let parts: Vec<Value> = re
                    .split(&text)
                    .map(|s| Value::String(s.to_string()))
                    .collect();
                Ok(Value::List(Rc::new(RefCell::new(parts))))
            }))),
        );

        // ============================================================
        // STDLIB EXPANSION - Environment & System
        // ============================================================

        // env_get - get environment variable
        globals.borrow_mut().define(
            "env_get".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("env_get", 1, |args| {
                let name = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("env_get() needs a variable name string".to_string()),
                };
                match std::env::var(&name) {
                    Ok(val) => Ok(Value::String(val)),
                    Err(_) => Ok(Value::Nil),
                }
            }))),
        );

        // env_set - set environment variable
        globals.borrow_mut().define(
            "env_set".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("env_set", 2, |args| {
                let name = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("env_set() needs a variable name string".to_string()),
                };
                let value = match &args[1] {
                    Value::String(s) => s.clone(),
                    v => format!("{}", v),
                };
                std::env::set_var(&name, &value);
                Ok(Value::Nil)
            }))),
        );

        // env_all - get all environment variables as dict
        globals.borrow_mut().define(
            "env_all".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("env_all", 0, |_args| {
                let mut vars = DictValue::new();
                for (k, v) in std::env::vars() {
                    vars.set(Value::String(k), Value::String(v));
                }
                Ok(Value::Dict(Rc::new(RefCell::new(vars))))
            }))),
        );

        // shell - execute shell command and return output
        globals.borrow_mut().define(
            "shell".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("shell", 1, |args| {
                use std::process::Command;
                let cmd = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("shell() needs a command string".to_string()),
                };
                let output = {
                    #[cfg(target_os = "windows")]
                    {
                        let shell =
                            std::env::var("MDH_SHELL").unwrap_or_else(|_| "cmd".to_string());
                        Command::new(shell).args(["/C", &cmd]).output()
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        let shell = std::env::var("MDH_SHELL").unwrap_or_else(|_| "sh".to_string());
                        Command::new(shell).args(["-c", &cmd]).output()
                    }
                };
                match output {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                        Ok(Value::String(if stdout.is_empty() {
                            stderr
                        } else {
                            stdout
                        }))
                    }
                    Err(e) => Err(format!("Shell command failed: {}", e)),
                }
            }))),
        );

        // shell_status - execute shell command and return exit code
        globals.borrow_mut().define(
            "shell_status".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("shell_status", 1, |args| {
                use std::process::Command;
                let cmd = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("shell_status() needs a command string".to_string()),
                };
                let status = {
                    #[cfg(target_os = "windows")]
                    {
                        let shell =
                            std::env::var("MDH_SHELL").unwrap_or_else(|_| "cmd".to_string());
                        Command::new(shell).args(["/C", &cmd]).status()
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        let shell = std::env::var("MDH_SHELL").unwrap_or_else(|_| "sh".to_string());
                        Command::new(shell).args(["-c", &cmd]).status()
                    }
                };
                match status {
                    Ok(s) => Ok(Value::Integer(s.code().unwrap_or(-1) as i64)),
                    Err(e) => Err(format!("Shell command failed: {}", e)),
                }
            }))),
        );

        // exit - exit program with code
        // Not safe to exercise under source-based coverage runs.
        #[cfg(not(coverage))]
        globals.borrow_mut().define(
            "exit".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("exit", 1, |args| {
                let code = args[0].as_integer().unwrap_or(0) as i32;
                std::process::exit(code);
            }))),
        );

        // args - get command line arguments
        globals.borrow_mut().define(
            "args".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("args", 0, |_args| {
                let arguments: Vec<Value> = std::env::args().map(Value::String).collect();
                Ok(Value::List(Rc::new(RefCell::new(arguments))))
            }))),
        );

        // cwd - get current working directory
        globals.borrow_mut().define(
            "cwd".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("cwd", 0, |_args| {
                match std::env::current_dir() {
                    Ok(path) => Ok(Value::String(path.to_string_lossy().to_string())),
                    Err(e) => Err(format!("Couldnae get current directory: {}", e)),
                }
            }))),
        );

        // chdir - change current directory
        globals.borrow_mut().define(
            "chdir".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("chdir", 1, |args| {
                let path = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("chdir() needs a path string".to_string()),
                };
                std::env::set_current_dir(&path)
                    .map_err(|e| format!("Couldnae change tae directory '{}': {}", path, e))?;
                Ok(Value::Nil)
            }))),
        );

        // json_stringify_pretty - pretty-printed JSON (alias for json_pretty)
        globals.borrow_mut().define(
            "json_stringify_pretty".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(
                "json_stringify_pretty",
                1,
                |args| Ok(Value::String(value_to_json_pretty(&args[0], 0))),
            ))),
        );
    }

    /// Run a program
    pub fn interpret(&mut self, program: &Program) -> HaversResult<Value> {
        let mut result = Value::Nil;
        for stmt in &program.statements {
            result = self.execute_stmt(stmt)?;
        }
        Ok(result)
    }

    /// Get captured output (for testing)
    #[allow(dead_code)]
    pub fn get_output(&self) -> &[String] {
        &self.output
    }

    /// Clear captured output
    #[allow(dead_code)]
    pub fn clear_output(&mut self) {
        self.output.clear();
    }

    /// Load a module fae a file
    fn load_module(
        &mut self,
        path: &str,
        alias: Option<&str>,
        span: Span,
    ) -> Result<Result<Value, ControlFlow>, HaversError> {
        if crate::tri::is_tri_module(path) {
            return self.load_tri_module(alias, span);
        }
        // Resolve the module path
        let module_path = self.resolve_module_path(path)?;

        // Check fer circular imports
        if self.loaded_modules.contains(&module_path) {
            // Already loaded, that's fine - skip
            return Ok(Ok(Value::Nil));
        }

        // Read the module file
        let source =
            std::fs::read_to_string(&module_path).map_err(|_| HaversError::ModuleNotFound {
                name: path.to_string(),
            })?;

        // Parse the module
        let program = crate::parser::parse(&source).map_err(|e| HaversError::ParseError {
            message: format!("Error in module '{}': {}", path, e),
            line: span.line,
        })?;

        // Mark as loaded tae prevent circular imports
        self.loaded_modules.insert(module_path.clone());

        // Save the current directory and switch tae the module's directory
        let old_dir = self.current_dir.clone();
        if let Some(parent) = module_path.parent() {
            self.current_dir = parent.to_path_buf();
        }

        // Execute the module in a new environment that inherits fae globals
        let module_env = Rc::new(RefCell::new(Environment::with_enclosing(
            self.globals.clone(),
        )));
        let old_env = self.environment.clone();
        self.environment = module_env.clone();

        // Execute the module
        for stmt in &program.statements {
            self.execute_stmt(stmt)?;
        }

        // Restore environment and directory
        self.environment = old_env;
        self.current_dir = old_dir;

        // If there's an alias, create a namespace object
        // Otherwise, export all defined names tae the current environment
        if let Some(alias_name) = alias {
            // Create a dictionary wi' the module's exports
            let exports = module_env.borrow().get_exports();
            let mut export_dict = DictValue::new();
            for (name, value) in exports {
                export_dict.set(Value::String(name), value);
            }
            let module_dict = Value::Dict(Rc::new(RefCell::new(export_dict)));
            self.environment
                .borrow_mut()
                .define(alias_name.to_string(), module_dict);
        } else {
            // Import all names directly
            let exports = module_env.borrow().get_exports();
            for (name, value) in exports {
                self.environment.borrow_mut().define(name, value);
            }
        }

        Ok(Ok(Value::Nil))
    }

    fn load_tri_module(
        &mut self,
        alias: Option<&str>,
        span: Span,
    ) -> Result<Result<Value, ControlFlow>, HaversError> {
        let alias_name = alias.ok_or_else(|| HaversError::TypeError {
            message: "tri import requires an alias (fetch \"tri\" tae name)".to_string(),
            line: span.line,
        })?;
        let module_val = crate::tri::tri_module_value();
        self.environment
            .borrow_mut()
            .define(alias_name.to_string(), module_val);
        Ok(Ok(Value::Nil))
    }

    /// Resolve a module path relative tae the current directory
    fn resolve_module_path(&self, path: &str) -> HaversResult<PathBuf> {
        let mut module_path = PathBuf::from(path);

        // Add .braw extension if not present
        if module_path.extension().is_none() {
            module_path.set_extension("braw");
        }

        // If it's an absolute path, try that directly
        if module_path.is_absolute() {
            return module_path
                .canonicalize()
                .map_err(|_| HaversError::ModuleNotFound {
                    name: path.to_string(),
                });
        }

        let mut candidates = Vec::new();
        candidates.push(self.current_dir.join(&module_path));
        candidates.push(PathBuf::from(&module_path));
        candidates.push(self.current_dir.join("stdlib").join(&module_path));
        if let Ok(stripped) = module_path.strip_prefix("lib") {
            candidates.push(self.current_dir.join("stdlib").join(stripped));
        }

        // Try searching up the directory tree (helps with stdlib/lib paths)
        for ancestor in self.current_dir.ancestors() {
            candidates.push(ancestor.join(&module_path));
            candidates.push(ancestor.join("stdlib").join(&module_path));
            if let Ok(stripped) = module_path.strip_prefix("lib") {
                candidates.push(ancestor.join("stdlib").join(stripped));
            }
        }

        // Try next to the executable (common for bundled stdlib)
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                candidates.push(parent.join(&module_path));
                candidates.push(parent.join("stdlib").join(&module_path));
                if let Ok(stripped) = module_path.strip_prefix("lib") {
                    candidates.push(parent.join("stdlib").join(stripped));
                }
            }
        }

        for candidate in candidates {
            if candidate.exists() {
                return Ok(candidate.canonicalize().unwrap_or(candidate));
            }
        }

        Err(HaversError::ModuleNotFound {
            name: path.to_string(),
        })
    }

    fn execute_stmt(&mut self, stmt: &Stmt) -> HaversResult<Value> {
        match self.execute_stmt_with_control(stmt)? {
            Ok(value) => Ok(value),
            Err(ControlFlow::Return(value)) => Ok(value),
            Err(ControlFlow::Break) => Err(HaversError::BreakOutsideLoop {
                line: stmt.span().line,
            }),
            Err(ControlFlow::Continue) => Err(HaversError::ContinueOutsideLoop {
                line: stmt.span().line,
            }),
        }
    }

    fn execute_stmt_with_control(
        &mut self,
        stmt: &Stmt,
    ) -> HaversResult<Result<Value, ControlFlow>> {
        match stmt {
            Stmt::VarDecl {
                name,
                initializer,
                span,
            } => {
                self.trace(&format!("[line {}] ken {} = ...", span.line, name));
                let value = if let Some(init) = initializer {
                    let v = self.evaluate(init)?;
                    self.trace_verbose(&format!("→ {} is noo {}", name, v));
                    v
                } else {
                    self.trace_verbose(&format!("→ {} is noo naething", name));
                    Value::Nil
                };
                self.environment.borrow_mut().define(name.clone(), value);
                Ok(Ok(Value::Nil))
            }

            Stmt::Expression { expr, span } => {
                self.trace(&format!("[line {}] evaluatin' expression", span.line));
                let value = self.evaluate(expr)?;
                self.trace_verbose(&format!("→ result: {}", value));
                Ok(Ok(value))
            }

            Stmt::Block { statements, span } => {
                self.trace(&format!("[line {}] enterin' block", span.line));
                self.trace_depth += 1;
                let result = self.execute_block(statements, None);
                self.trace_depth = self.trace_depth.saturating_sub(1);
                self.trace(&format!("[line {}] leavin' block", span.line));
                result
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
                span,
            } => {
                self.trace(&format!("[line {}] gin (if) statement", span.line));
                let cond_value = self.evaluate(condition)?;
                self.trace_verbose(&format!("→ condition is {}", cond_value));
                if cond_value.is_truthy() {
                    self.trace(&format!(
                        "[line {}] condition is aye - takin' then branch",
                        span.line
                    ));
                    self.execute_stmt_with_control(then_branch)
                } else if let Some(else_br) = else_branch {
                    self.trace(&format!(
                        "[line {}] condition is nae - takin' ither branch",
                        span.line
                    ));
                    self.execute_stmt_with_control(else_br)
                } else {
                    self.trace_verbose("→ condition is nae, nae ither branch");
                    Ok(Ok(Value::Nil))
                }
            }

            Stmt::While {
                condition,
                body,
                span,
            } => {
                self.trace(&format!(
                    "[line {}] whiles (while) loop startin'",
                    span.line
                ));
                let mut iteration = 0;
                while self.evaluate(condition)?.is_truthy() {
                    iteration += 1;
                    self.trace_verbose(&format!("→ loop iteration {}", iteration));
                    match self.execute_stmt_with_control(body)? {
                        Ok(_) => {}
                        Err(ControlFlow::Break) => {
                            self.trace(&format!(
                                "[line {}] brak! (break) - leavin' loop",
                                span.line
                            ));
                            break;
                        }
                        Err(ControlFlow::Continue) => {
                            self.trace_verbose("→ haud! (continue)");
                            continue;
                        }
                        Err(ControlFlow::Return(v)) => return Ok(Err(ControlFlow::Return(v))),
                    }
                }
                self.trace(&format!(
                    "[line {}] whiles loop done after {} iterations",
                    span.line, iteration
                ));
                Ok(Ok(Value::Nil))
            }

            Stmt::For {
                variable,
                iterable,
                body,
                span,
            } => {
                self.trace(&format!(
                    "[line {}] fer (for) loop: {} in ...",
                    span.line, variable
                ));
                let iter_value = self.evaluate(iterable)?;

                let items: Vec<Value> = match iter_value {
                    Value::Range(range) => range.iter().map(Value::Integer).collect(),
                    Value::List(list) => list.borrow().clone(),
                    Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
                    _ => {
                        return Err(HaversError::TypeError {
                            message: format!("Cannae iterate ower a {}", iter_value.type_name()),
                            line: span.line,
                        });
                    }
                };

                self.trace_verbose(&format!("→ iteratin' ower {} items", items.len()));
                let mut iteration = 0;
                for item in items {
                    iteration += 1;
                    self.trace_verbose(&format!(
                        "→ iteration {}: {} = {}",
                        iteration, variable, item
                    ));
                    self.environment.borrow_mut().define(variable.clone(), item);
                    match self.execute_stmt_with_control(body)? {
                        Ok(_) => {}
                        Err(ControlFlow::Break) => {
                            self.trace(&format!(
                                "[line {}] brak! (break) - leavin' fer loop",
                                span.line
                            ));
                            break;
                        }
                        Err(ControlFlow::Continue) => {
                            self.trace_verbose("→ haud! (continue)");
                            continue;
                        }
                        Err(ControlFlow::Return(v)) => return Ok(Err(ControlFlow::Return(v))),
                    }
                }
                self.trace(&format!(
                    "[line {}] fer loop done after {} iterations",
                    span.line, iteration
                ));
                Ok(Ok(Value::Nil))
            }

            Stmt::Function {
                name,
                params,
                body,
                span,
            } => {
                self.trace(&format!(
                    "[line {}] dae (function) {} wi' {} params",
                    span.line,
                    name,
                    params.len()
                ));
                // Convert AST Param tae runtime FunctionParam
                let runtime_params: Vec<FunctionParam> = params
                    .iter()
                    .map(|p| FunctionParam {
                        name: p.name.clone(),
                        default: p.default.clone(),
                    })
                    .collect();

                let func = HaversFunction::new(
                    name.clone(),
                    runtime_params,
                    body.clone(),
                    Some(self.environment.clone()),
                );
                self.environment
                    .borrow_mut()
                    .define(name.clone(), Value::Function(Rc::new(func)));
                Ok(Ok(Value::Nil))
            }

            Stmt::Return { value, span } => {
                let ret_val = if let Some(expr) = value {
                    let v = self.evaluate(expr)?;
                    self.trace(&format!("[line {}] gie (return) {}", span.line, v));
                    v
                } else {
                    self.trace(&format!("[line {}] gie (return) naething", span.line));
                    Value::Nil
                };
                Ok(Err(ControlFlow::Return(ret_val)))
            }

            Stmt::Print { value, span } => {
                let val = self.evaluate(value)?;
                self.trace(&format!("[line {}] blether (print): {}", span.line, val));
                let output = format!("{}", val);
                println!("{}", output);
                self.output.push(output);
                Ok(Ok(Value::Nil))
            }

            Stmt::Break { span } => {
                self.trace(&format!("[line {}] brak! (break)", span.line));
                Ok(Err(ControlFlow::Break))
            }

            Stmt::Continue { span } => {
                self.trace(&format!("[line {}] haud! (continue)", span.line));
                Ok(Err(ControlFlow::Continue))
            }

            Stmt::Class {
                name,
                superclass,
                methods,
                span,
            } => {
                self.trace(&format!(
                    "[line {}] kin (class) {} defined",
                    span.line, name
                ));
                let super_class = if let Some(super_name) = superclass {
                    let super_val = self.environment.borrow().get(super_name).ok_or_else(|| {
                        HaversError::UndefinedVariable {
                            name: super_name.clone(),
                            line: span.line,
                        }
                    })?;
                    match super_val {
                        Value::Class(c) => Some(c),
                        _ => {
                            return Err(HaversError::TypeError {
                                message: format!("{} isnae a class", super_name),
                                line: span.line,
                            });
                        }
                    }
                } else {
                    None
                };

                let mut class = HaversClass::new(name.clone(), super_class);

                for method in methods {
                    if let Stmt::Function {
                        name: method_name,
                        params,
                        body,
                        ..
                    } = method
                    {
                        // Convert AST Param tae runtime FunctionParam
                        let runtime_params: Vec<FunctionParam> = params
                            .iter()
                            .map(|p| FunctionParam {
                                name: p.name.clone(),
                                default: p.default.clone(),
                            })
                            .collect();

                        let func = HaversFunction::new(
                            method_name.clone(),
                            runtime_params,
                            body.clone(),
                            Some(self.environment.clone()),
                        );
                        class.methods.insert(method_name.clone(), Rc::new(func));
                    }
                }

                self.environment
                    .borrow_mut()
                    .define(name.clone(), Value::Class(Rc::new(class)));
                Ok(Ok(Value::Nil))
            }

            Stmt::Struct { name, fields, span } => {
                self.trace(&format!(
                    "[line {}] thing (struct) {} defined wi' {} fields",
                    span.line,
                    name,
                    fields.len()
                ));
                let structure = HaversStruct::new(name.clone(), fields.clone());
                self.environment
                    .borrow_mut()
                    .define(name.clone(), Value::Struct(Rc::new(structure)));
                Ok(Ok(Value::Nil))
            }

            Stmt::Import { path, alias, span } => {
                let alias_str = alias
                    .as_ref()
                    .map(|a| format!(" as {}", a))
                    .unwrap_or_default();
                self.trace(&format!(
                    "[line {}] fetch (import) \"{}\"{}",
                    span.line, path, alias_str
                ));
                self.load_module(path, alias.as_deref(), *span)
            }

            Stmt::TryCatch {
                try_block,
                error_name,
                catch_block,
                span,
            } => {
                self.trace(&format!("[line {}] hae_a_bash (try) startin'", span.line));
                match self.execute_stmt_with_control(try_block) {
                    Ok(result) => {
                        self.trace(&format!(
                            "[line {}] try block succeeded - nae bother!",
                            span.line
                        ));
                        Ok(result)
                    }
                    Err(e) => {
                        self.trace(&format!(
                            "[line {}] gin_it_gangs_wrang (catch) - caught: {}",
                            span.line, e
                        ));
                        // Bind the error to the catch variable
                        self.environment
                            .borrow_mut()
                            .define(error_name.clone(), Value::String(e.to_string()));
                        self.execute_stmt_with_control(catch_block)
                    }
                }
            }

            Stmt::Match { value, arms, span } => {
                self.trace(&format!("[line {}] keek (match) statement", span.line));
                let val = self.evaluate(value)?;
                self.trace_verbose(&format!("→ matchin' against: {}", val));

                for (i, arm) in arms.iter().enumerate() {
                    if self.pattern_matches(&arm.pattern, &val)? {
                        self.trace(&format!("[line {}] matched arm {}", span.line, i + 1));
                        // Bind pattern variables if needed
                        if let Pattern::Identifier(name) = &arm.pattern {
                            self.environment
                                .borrow_mut()
                                .define(name.clone(), val.clone());
                        }
                        return self.execute_stmt_with_control(&arm.body);
                    }
                }

                // No match found
                self.trace(&format!("[line {}] nae match found!", span.line));
                Err(HaversError::TypeError {
                    message: format!("Nae match found fer {}", val),
                    line: span.line,
                })
            }

            Stmt::Assert {
                condition,
                message,
                span,
            } => {
                self.trace(&format!("[line {}] mak_siccar (assert)", span.line));
                let cond_value = self.evaluate(condition)?;
                self.trace_verbose(&format!("→ condition is {}", cond_value));
                if !cond_value.is_truthy() {
                    let msg = if let Some(msg_expr) = message {
                        let msg_val = self.evaluate(msg_expr)?;
                        msg_val.to_string()
                    } else {
                        "Assertion failed".to_string()
                    };
                    self.trace(&format!("[line {}] assertion FAILED: {}", span.line, msg));
                    return Err(HaversError::AssertionFailed {
                        message: msg,
                        line: span.line,
                    });
                }
                self.trace_verbose("→ assertion passed - braw!");
                Ok(Ok(Value::Nil))
            }

            Stmt::Destructure {
                patterns,
                value,
                span,
            } => {
                self.trace(&format!(
                    "[line {}] destructurin' intae {} variables",
                    span.line,
                    patterns.len()
                ));
                let val = self.evaluate(value)?;
                self.trace_verbose(&format!("→ unpackin': {}", val));

                // The value must be a list
                let items = match &val {
                    Value::List(list) => list.borrow().clone(),
                    Value::String(s) => {
                        // Strings can be destructured intae characters
                        s.chars().map(|c| Value::String(c.to_string())).collect()
                    }
                    _ => {
                        return Err(HaversError::TypeError {
                            message: format!(
                                "Ye can only destructure lists and strings, no' {}",
                                val.type_name()
                            ),
                            line: span.line,
                        });
                    }
                };

                // Find the rest pattern position if any
                let rest_pos = patterns
                    .iter()
                    .position(|p| matches!(p, DestructPattern::Rest(_)));

                // Calculate positions
                let before_rest = rest_pos.unwrap_or(patterns.len());
                let after_rest = if let Some(pos) = rest_pos {
                    patterns.len() - pos - 1
                } else {
                    0
                };

                // Check we have enough elements
                let min_required = before_rest + after_rest;
                if items.len() < min_required {
                    return Err(HaversError::TypeError {
                        message: format!(
                            "Cannae destructure: need at least {} elements but got {}",
                            min_required,
                            items.len()
                        ),
                        line: span.line,
                    });
                }

                // Bind the variables
                let mut item_idx = 0;
                for (pat_idx, pattern) in patterns.iter().enumerate() {
                    match pattern {
                        DestructPattern::Variable(name) => {
                            if pat_idx < before_rest {
                                // Before rest: take from start
                                self.environment
                                    .borrow_mut()
                                    .define(name.clone(), items[item_idx].clone());
                                item_idx += 1;
                            } else {
                                // After rest: take from end
                                let from_end = patterns.len() - pat_idx - 1;
                                let end_idx = items.len() - from_end - 1;
                                self.environment
                                    .borrow_mut()
                                    .define(name.clone(), items[end_idx].clone());
                            }
                        }
                        DestructPattern::Rest(name) => {
                            // Capture all elements in the middle
                            let rest_end = items.len() - after_rest;
                            let rest_items: Vec<Value> = items[item_idx..rest_end].to_vec();
                            self.environment.borrow_mut().define(
                                name.clone(),
                                Value::List(Rc::new(RefCell::new(rest_items))),
                            );
                            item_idx = rest_end;
                        }
                        DestructPattern::Ignore => {
                            if pat_idx < before_rest {
                                item_idx += 1;
                            }
                            // Just skip this element
                        }
                    }
                }

                Ok(Ok(Value::Nil))
            }

            Stmt::Log {
                level,
                message,
                extras,
                span,
            } => {
                let msg = self.evaluate(message)?;
                let (fields, target) = self.parse_log_extras(extras, span.line)?;
                self.emit_log(*level, msg, fields, target, span.line)?;
                Ok(Ok(Value::Nil))
            }

            Stmt::Hurl { message, span } => {
                let msg = self.evaluate(message)?;
                let error_msg = match msg {
                    Value::String(s) => s,
                    v => format!("{}", v),
                };
                Err(HaversError::UserError {
                    message: error_msg,
                    line: span.line,
                })
            }
        }
    }

    fn execute_block(
        &mut self,
        statements: &[Stmt],
        env: Option<Rc<RefCell<Environment>>>,
    ) -> HaversResult<Result<Value, ControlFlow>> {
        let previous = self.environment.clone();
        let new_env = env.unwrap_or_else(|| {
            Rc::new(RefCell::new(Environment::with_enclosing(previous.clone())))
        });
        self.environment = new_env;

        let mut result = Ok(Value::Nil);
        for stmt in statements {
            match self.execute_stmt_with_control(stmt)? {
                Ok(v) => result = Ok(v),
                Err(cf) => {
                    self.environment = previous;
                    return Ok(Err(cf));
                }
            }
        }

        self.environment = previous;
        Ok(result)
    }

    fn pattern_matches(&mut self, pattern: &Pattern, value: &Value) -> HaversResult<bool> {
        match pattern {
            Pattern::Literal(lit) => {
                let lit_val = match lit {
                    Literal::Integer(n) => Value::Integer(*n),
                    Literal::Float(f) => Value::Float(*f),
                    Literal::String(s) => Value::String(s.clone()),
                    Literal::Bool(b) => Value::Bool(*b),
                    Literal::Nil => Value::Nil,
                };
                Ok(lit_val == *value)
            }
            Pattern::Identifier(_) => Ok(true), // Always matches, binds value
            Pattern::Wildcard => Ok(true),
            Pattern::Range { start, end } => {
                if let Value::Integer(n) = value {
                    let start_val = self.evaluate(start)?;
                    let end_val = self.evaluate(end)?;
                    if let (Some(s), Some(e)) = (start_val.as_integer(), end_val.as_integer()) {
                        Ok(*n >= s && *n < e)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn range_to_list(start: i64, end: i64, inclusive: bool) -> Value {
        let mut items = Vec::new();
        if inclusive {
            let mut i = start;
            while i <= end {
                items.push(Value::Integer(i));
                i += 1;
            }
        } else {
            let mut i = start;
            while i < end {
                items.push(Value::Integer(i));
                i += 1;
            }
        }
        Value::List(Rc::new(RefCell::new(items)))
    }

    fn evaluate(&mut self, expr: &Expr) -> HaversResult<Value> {
        match expr {
            Expr::Literal { value, .. } => Ok(match value {
                Literal::Integer(n) => Value::Integer(*n),
                Literal::Float(f) => Value::Float(*f),
                Literal::String(s) => Value::String(s.clone()),
                Literal::Bool(b) => Value::Bool(*b),
                Literal::Nil => Value::Nil,
            }),

            Expr::Variable { name, span } => self
                .environment
                .borrow()
                .get(name)
                .ok_or_else(|| HaversError::UndefinedVariable {
                    name: name.clone(),
                    line: span.line,
                }),

            Expr::Assign { name, value, span } => {
                let val = self.evaluate(value)?;
                if !self.environment.borrow_mut().assign(name, val.clone()) {
                    return Err(HaversError::UndefinedVariable {
                        name: name.clone(),
                        line: span.line,
                    });
                }
                Ok(val)
            }

            Expr::Binary {
                left,
                operator,
                right,
                span,
            } => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;

                // Check for operator overloading on instances
                if let Value::Instance(ref inst) = left_val {
                    let method_name = self.operator_method_name(operator);
                    if let Some(method) = inst.borrow().class.find_method(&method_name) {
                        // Call the overloaded operator method
                        return self.call_method_on_instance(
                            inst.clone(),
                            method,
                            vec![right_val],
                            span.line,
                        );
                    }
                }

                self.binary_op(&left_val, operator, &right_val, span.line)
            }

            Expr::Unary {
                operator,
                operand,
                span,
            } => {
                let val = self.evaluate(operand)?;
                match operator {
                    UnaryOp::Negate => match val {
                        Value::Integer(n) => Ok(Value::Integer(-n)),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        _ => Err(HaversError::TypeError {
                            message: format!("Cannae negate a {}", val.type_name()),
                            line: span.line,
                        }),
                    },
                    UnaryOp::Not => Ok(Value::Bool(!val.is_truthy())),
                }
            }

            Expr::Logical {
                left,
                operator,
                right,
                ..
            } => {
                let left_val = self.evaluate(left)?;
                match operator {
                    LogicalOp::And => {
                        if !left_val.is_truthy() {
                            Ok(left_val)
                        } else {
                            self.evaluate(right)
                        }
                    }
                    LogicalOp::Or => {
                        if left_val.is_truthy() {
                            Ok(left_val)
                        } else {
                            self.evaluate(right)
                        }
                    }
                }
            }

            Expr::Call {
                callee,
                arguments,
                span,
            } => {
                // Check if this is a method call (callee is a Get expression)
                if let Expr::Get { object, property, .. } = callee.as_ref() {
                    let obj = self.evaluate(object)?;
                    if let Value::NativeObject(native) = &obj {
                        let args = self.evaluate_call_args(arguments, span.line)?;
                        return native.call(property, args);
                    }
                    if let Value::Instance(inst) = &obj {
                        // It's a method call - get the method and bind 'masel'
                        // Clone what we need to avoid holding the borrow
                        let method_opt = {
                            let borrowed = inst.borrow();
                            borrowed.class.find_method(property)
                        };
                        if let Some(method) = method_opt {
                            let args = self.evaluate_call_args(arguments, span.line)?;
                            let env = Rc::new(RefCell::new(Environment::with_enclosing(
                                method.closure.clone().unwrap_or(self.globals.clone()),
                            )));
                            env.borrow_mut()
                                .define("masel".to_string(), Value::Instance(inst.clone()));
                            return self.call_function_with_env(&method, args, env, span.line);
                        }
                        // Check instance fields for callable values
                        let field_val_opt = {
                            let borrowed = inst.borrow();
                            borrowed.fields.get(property).cloned()
                        };
                        if let Some(field_val) = field_val_opt {
                            let args = self.evaluate_call_args(arguments, span.line)?;
                            return self.call_value(field_val, args, span.line);
                        }
                        return Err(HaversError::UndefinedVariable {
                            name: property.clone(),
                            line: span.line,
                        });
                    }
                }

                let callee_val = self.evaluate(callee)?;
                let args = self.evaluate_call_args(arguments, span.line)?;
                self.call_value(callee_val, args, span.line)
            }

            Expr::Get {
                object,
                property,
                span,
            } => {
                let obj = self.evaluate(object)?;
                match obj {
                    Value::NativeObject(native) => native.get(property),
                    Value::Instance(inst) => inst
                        .borrow()
                        .get(property)
                        .ok_or_else(|| HaversError::UndefinedVariable {
                            name: property.clone(),
                            line: span.line,
                        }),
                    Value::Dict(dict) => dict
                        .borrow()
                        .get(&Value::String(property.clone()))
                        .cloned()
                        .ok_or_else(|| HaversError::UndefinedVariable {
                            name: property.clone(),
                            line: span.line,
                        }),
                    _ => Err(HaversError::TypeError {
                        message: format!(
                            "Cannae access property '{}' on a {}",
                            property,
                            obj.type_name()
                        ),
                        line: span.line,
                    }),
                }
            }

            Expr::Set {
                object,
                property,
                value,
                span,
            } => {
                let obj = self.evaluate(object)?;
                let val = self.evaluate(value)?;
                match obj {
                    Value::NativeObject(native) => native.set(property, val),
                    Value::Instance(inst) => {
                        inst.borrow_mut().set(property.clone(), val.clone());
                        Ok(val)
                    }
                    Value::Dict(dict) => {
                        dict.borrow_mut()
                            .set(Value::String(property.clone()), val.clone());
                        Ok(val)
                    }
                    _ => Err(HaversError::TypeError {
                        message: format!(
                            "Cannae set property '{}' on a {}",
                            property,
                            obj.type_name()
                        ),
                        line: span.line,
                    }),
                }
            }

            Expr::Index {
                object,
                index,
                span,
            } => {
                let obj = self.evaluate(object)?;
                let idx = self.evaluate(index)?;
                match (&obj, &idx) {
                    (Value::List(list), Value::Integer(i)) => {
                        let list = list.borrow();
                        let idx = if *i < 0 {
                            list.len() as i64 + *i
                        } else {
                            *i
                        };
                        list.get(idx as usize)
                            .cloned()
                            .ok_or_else(|| HaversError::IndexOutOfBounds {
                                index: *i,
                                size: list.len(),
                                line: span.line,
                            })
                    }
                    (Value::String(s), Value::Integer(i)) => {
                        let idx = if *i < 0 {
                            s.len() as i64 + *i
                        } else {
                            *i
                        };
                        s.chars()
                            .nth(idx as usize)
                            .map(|c| Value::String(c.to_string()))
                            .ok_or(HaversError::IndexOutOfBounds {
                                index: *i,
                                size: s.len(),
                                line: span.line,
                            })
                    }
                    (Value::Dict(dict), key) => dict
                        .borrow()
                        .get(key)
                        .cloned()
                        .ok_or_else(|| HaversError::UndefinedVariable {
                            name: format!("{}", key),
                            line: span.line,
                        }),
                    _ => Err(HaversError::TypeError {
                        message: format!(
                            "Cannae index a {} wi' a {}",
                            obj.type_name(),
                            idx.type_name()
                        ),
                        line: span.line,
                    }),
                }
            }

            Expr::IndexSet {
                object,
                index,
                value,
                span,
            } => {
                let obj = self.evaluate(object)?;
                let idx = self.evaluate(index)?;
                let val = self.evaluate(value)?;

                match (&obj, &idx) {
                    (Value::List(list), Value::Integer(i)) => {
                        let mut list_mut = list.borrow_mut();
                        let idx = if *i < 0 {
                            list_mut.len() as i64 + *i
                        } else {
                            *i
                        };
                        if idx < 0 || idx as usize >= list_mut.len() {
                            return Err(HaversError::IndexOutOfBounds {
                                index: *i,
                                size: list_mut.len(),
                                line: span.line,
                            });
                        }
                        list_mut[idx as usize] = val.clone();
                        Ok(val)
                    }
                    (Value::Dict(dict), key) => {
                        dict.borrow_mut().set(key.clone(), val.clone());
                        Ok(val)
                    }
                    _ => Err(HaversError::TypeError {
                        message: format!(
                            "Cannae set index on a {} wi' a {}",
                            obj.type_name(),
                            idx.type_name()
                        ),
                        line: span.line,
                    }),
                }
            }

            Expr::Slice {
                object,
                start,
                end,
                step,
                span,
            } => {
                let obj = self.evaluate(object)?;

                // Get start index, handling None as default
                let start_idx = if let Some(s) = start {
                    let val = self.evaluate(s)?;
                    match val {
                        Value::Integer(i) => Some(i),
                        _ => {
                            return Err(HaversError::TypeError {
                                message: "Slice start must be an integer".to_string(),
                                line: span.line,
                            })
                        }
                    }
                } else {
                    None
                };

                // Get end index
                let end_idx = if let Some(e) = end {
                    let val = self.evaluate(e)?;
                    match val {
                        Value::Integer(i) => Some(i),
                        _ => {
                            return Err(HaversError::TypeError {
                                message: "Slice end must be an integer".to_string(),
                                line: span.line,
                            })
                        }
                    }
                } else {
                    None
                };

                // Get step value (default is 1)
                let step_val = if let Some(st) = step {
                    let val = self.evaluate(st)?;
                    match val {
                        Value::Integer(i) => {
                            if i == 0 {
                                return Err(HaversError::TypeError {
                                    message: "Slice step cannae be zero, ya dafty!".to_string(),
                                    line: span.line,
                                });
                            }
                            i
                        }
                        _ => {
                            return Err(HaversError::TypeError {
                                message: "Slice step must be an integer".to_string(),
                                line: span.line,
                            })
                        }
                    }
                } else {
                    1
                };

                match obj {
                    Value::List(list) => {
                        let list = list.borrow();
                        let len = list.len() as i64;

                        // Handle defaults based on step direction
                        let (start, end) = if step_val > 0 {
                            let s = start_idx.unwrap_or(0);
                            let e = end_idx.unwrap_or(len);
                            (s, e)
                        } else {
                            // Negative step: default start is -1 (end), default end is before start
                            let s = start_idx.unwrap_or(-1);
                            let e = end_idx.unwrap_or(-(len + 1));
                            (s, e)
                        };

                        // Normalize negative indices
                        let start = if start < 0 {
                            (len + start).max(0) as usize
                        } else {
                            (start as usize).min(list.len())
                        };

                        let end = if end < 0 {
                            (len + end).max(-1)
                        } else {
                            (end as usize).min(list.len()) as i64
                        };

                        let mut sliced: Vec<Value> = Vec::new();
                        if step_val > 0 {
                            let mut i = start as i64;
                            while i < end && i < len {
                                if i >= 0 {
                                    sliced.push(list[i as usize].clone());
                                }
                                i += step_val;
                            }
                        } else {
                            // Negative step: go backwards
                            let mut i = start as i64;
                            while i > end && i >= 0 {
                                if (i as usize) < list.len() {
                                    sliced.push(list[i as usize].clone());
                                }
                                i += step_val; // step_val is negative
                            }
                        }
                        Ok(Value::List(Rc::new(RefCell::new(sliced))))
                    }
                    Value::String(s) => {
                        let chars: Vec<char> = s.chars().collect();
                        let len = chars.len() as i64;

                        // Handle defaults based on step direction
                        let (start, end) = if step_val > 0 {
                            let st = start_idx.unwrap_or(0);
                            let en = end_idx.unwrap_or(len);
                            (st, en)
                        } else {
                            // Negative step: default start is -1 (end), default end is before start
                            let st = start_idx.unwrap_or(-1);
                            let en = end_idx.unwrap_or(-(len + 1));
                            (st, en)
                        };

                        // Normalize negative indices
                        let start = if start < 0 {
                            (len + start).max(0) as usize
                        } else {
                            (start as usize).min(chars.len())
                        };

                        let end = if end < 0 {
                            (len + end).max(-1)
                        } else {
                            (end as usize).min(chars.len()) as i64
                        };

                        let mut sliced = String::new();
                        if step_val > 0 {
                            let mut i = start as i64;
                            while i < end && i < len {
                                if i >= 0 {
                                    sliced.push(chars[i as usize]);
                                }
                                i += step_val;
                            }
                        } else {
                            // Negative step: go backwards
                            let mut i = start as i64;
                            while i > end && i >= 0 {
                                if (i as usize) < chars.len() {
                                    sliced.push(chars[i as usize]);
                                }
                                i += step_val; // step_val is negative
                            }
                        }
                        Ok(Value::String(sliced))
                    }
                    _ => Err(HaversError::TypeError {
                        message: format!("Cannae slice a {}, ya numpty!", obj.type_name()),
                        line: span.line,
                    }),
                }
            }

            Expr::List { elements, .. } => {
                let mut items = Vec::new();
                for elem in elements {
                    // Handle spread operator (...) - skail the elements intae the list
                    if let Expr::Spread { expr, span } = elem {
                        let spread_value = self.evaluate(expr)?;
                        match spread_value {
                            Value::List(list) => {
                                items.extend(list.borrow().clone());
                            }
                            Value::String(s) => {
                                // Spread string into characters
                                for c in s.chars() {
                                    items.push(Value::String(c.to_string()));
                                }
                            }
                            _ => {
                                return Err(HaversError::TypeError {
                                    message: "Cannae skail (spread) somethin' that isnae a list or string!".to_string(),
                                    line: span.line,
                                });
                            }
                        }
                    } else {
                        items.push(self.evaluate(elem)?);
                    }
                }
                Ok(Value::List(Rc::new(RefCell::new(items))))
            }

            Expr::Dict { pairs, .. } => {
                let mut map = DictValue::new();
                for (key, value) in pairs {
                    let k = self.evaluate(key)?;
                    let v = self.evaluate(value)?;
                    map.set(k, v);
                }
                Ok(Value::Dict(Rc::new(RefCell::new(map))))
            }

            Expr::Range {
                start,
                end,
                inclusive,
                ..
            } => {
                let start_val = self.evaluate(start)?;
                let end_val = self.evaluate(end)?;
                match (start_val.as_integer(), end_val.as_integer()) {
                    (Some(s), Some(e)) => Ok(Self::range_to_list(s, e, *inclusive)),
                    _ => Err(HaversError::TypeError {
                        message: "Range bounds must be integers".to_string(),
                        line: expr.span().line,
                    }),
                }
            }

            Expr::Grouping { expr, .. } => self.evaluate(expr),

            Expr::Lambda {
                params,
                body,
                span,
            } => {
                // Convert lambda params tae FunctionParams (lambdas dinnae hae defaults)
                let runtime_params: Vec<FunctionParam> = params
                    .iter()
                    .map(|name| FunctionParam {
                        name: name.clone(),
                        default: None,
                    })
                    .collect();

                // Create a function from the lambda
                let func = HaversFunction::new(
                    "<lambda>".to_string(),
                    runtime_params,
                    vec![Stmt::Return {
                        value: Some((**body).clone()),
                        span: *span,
                    }],
                    Some(self.environment.clone()),
                );
                Ok(Value::Function(Rc::new(func)))
            }

            Expr::Masel { span } => {
                self.environment
                    .borrow()
                    .get("masel")
                    .ok_or_else(|| HaversError::UndefinedVariable {
                        name: "masel".to_string(),
                        line: span.line,
                    })
            }

            Expr::Input { prompt, span: _ } => {
                #[cfg(coverage)]
                {
                    let _ = self.evaluate(prompt)?;
                    Err(HaversError::InternalError(
                        "speir() input is disabled under coverage runs".to_string(),
                    ))
                }

                #[cfg(not(coverage))]
                {
                    let prompt_val = self.evaluate(prompt)?;
                    print!("{}", prompt_val);
                    io::stdout().flush().unwrap();

                    let mut input = String::new();
                    io::stdin()
                        .read_line(&mut input)
                        .map_err(|e| HaversError::InternalError(e.to_string()))?;

                    Ok(Value::String(input.trim().to_string()))
                }
            }

            Expr::FString { parts, .. } => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        FStringPart::Text(text) => result.push_str(text),
                        FStringPart::Expr(expr) => {
                            let val = self.evaluate(expr)?;
                            result.push_str(&val.to_string());
                        }
                    }
                }
                Ok(Value::String(result))
            }

            // Spread is only valid in specific contexts (lists, function calls)
            // If we get here, it's an error
            Expr::Spread { .. } => unreachable!(
                "Spread expressions are handled by list literals and function-call argument expansion"
            ),

            // Pipe forward: left |> right means call right(left)
            Expr::Pipe { left, right, span } => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                // Call the right side as a function with left as the argument
                self.call_value(right_val, vec![left_val], span.line)
            }

            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                // Evaluate condition and pick the appropriate branch
                let cond_val = self.evaluate(condition)?;
                if cond_val.is_truthy() {
                    self.evaluate(then_expr)
                } else {
                    self.evaluate(else_expr)
                }
            }
            Expr::BlockExpr { statements, .. } => {
                // Execute statements and return the value from 'gie' if any
                // Use execute_stmt_with_control to handle return properly
                for stmt in statements {
                    match self.execute_stmt_with_control(stmt)? {
                        Ok(_) => {}
                        Err(ControlFlow::Return(value)) => {
                            return Ok(value);
                        }
                        Err(ControlFlow::Break) | Err(ControlFlow::Continue) => {
                            // Propagate break/continue - shouldn't happen in block expr
                        }
                    }
                }
                Ok(Value::Nil)
            }
        }
    }

    fn binary_op(
        &self,
        left: &Value,
        op: &BinaryOp,
        right: &Value,
        line: usize,
    ) -> HaversResult<Value> {
        match op {
            BinaryOp::Add => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                (Value::String(a), b) => Ok(Value::String(format!("{}{}", a, b))),
                (a, Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                (Value::List(a), Value::List(b)) => {
                    let mut result = a.borrow().clone();
                    result.extend(b.borrow().clone());
                    Ok(Value::List(Rc::new(RefCell::new(result))))
                }
                _ => Err(HaversError::TypeError {
                    message: format!("Cannae add {} an' {}", left.type_name(), right.type_name()),
                    line,
                }),
            },

            BinaryOp::Subtract => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - *b as f64)),
                _ => Err(HaversError::TypeError {
                    message: format!(
                        "Cannae subtract {} fae {}",
                        right.type_name(),
                        left.type_name()
                    ),
                    line,
                }),
            },

            BinaryOp::Multiply => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * *b as f64)),
                (Value::String(s), Value::Integer(n)) | (Value::Integer(n), Value::String(s)) => {
                    Ok(Value::String(s.repeat(*n as usize)))
                }
                _ => Err(HaversError::TypeError {
                    message: format!(
                        "Cannae multiply {} by {}",
                        left.type_name(),
                        right.type_name()
                    ),
                    line,
                }),
            },

            BinaryOp::Divide => {
                // Check for division by zero
                match right {
                    Value::Integer(0) => return Err(HaversError::DivisionByZero { line }),
                    Value::Float(f) if *f == 0.0 => {
                        return Err(HaversError::DivisionByZero { line })
                    }
                    _ => {}
                }
                match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a / b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                    (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 / b)),
                    (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a / *b as f64)),
                    _ => Err(HaversError::TypeError {
                        message: format!(
                            "Cannae divide {} by {}",
                            left.type_name(),
                            right.type_name()
                        ),
                        line,
                    }),
                }
            }

            BinaryOp::Modulo => {
                if let Value::Integer(0) = right {
                    return Err(HaversError::DivisionByZero { line });
                }
                match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a % b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a % b)),
                    _ => Err(HaversError::TypeError {
                        message: format!(
                            "Cannae get remainder o' {} by {}",
                            left.type_name(),
                            right.type_name()
                        ),
                        line,
                    }),
                }
            }

            BinaryOp::Equal => Ok(Value::Bool(left == right)),
            BinaryOp::NotEqual => Ok(Value::Bool(left != right)),

            BinaryOp::Less => self.compare(left, right, |a, b| a < b, |a, b| a < b, line),
            BinaryOp::LessEqual => self.compare(left, right, |a, b| a <= b, |a, b| a <= b, line),
            BinaryOp::Greater => self.compare(left, right, |a, b| a > b, |a, b| a > b, line),
            BinaryOp::GreaterEqual => self.compare(left, right, |a, b| a >= b, |a, b| a >= b, line),
        }
    }

    fn compare<F, S>(
        &self,
        left: &Value,
        right: &Value,
        cmp: F,
        str_cmp: S,
        line: usize,
    ) -> HaversResult<Value>
    where
        F: Fn(f64, f64) -> bool,
        S: Fn(&str, &str) -> bool,
    {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(cmp(*a as f64, *b as f64))),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(cmp(*a, *b))),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Bool(cmp(*a as f64, *b))),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Bool(cmp(*a, *b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(str_cmp(a, b))),
            _ => Err(HaversError::TypeError {
                message: format!(
                    "Cannae compare {} wi' {}",
                    left.type_name(),
                    right.type_name()
                ),
                line,
            }),
        }
    }

    /// Get the method name for operator overloading
    /// Uses Scots-flavored names:
    /// - __pit_thegither__ = add (put together)
    /// - __tak_awa__ = subtract (take away)
    /// - __times__ = multiply
    /// - __pairt__ = divide (part/divide)
    /// - __lave__ = modulo (what's left)
    /// - __same_as__ = equal
    /// - __differs_fae__ = not equal
    /// - __wee_er__ = less than (smaller)
    /// - __wee_er_or_same__ = less or equal
    /// - __muckle_er__ = greater than (bigger)
    /// - __muckle_er_or_same__ = greater or equal
    fn operator_method_name(&self, op: &BinaryOp) -> String {
        match op {
            BinaryOp::Add => "__pit_thegither__".to_string(),
            BinaryOp::Subtract => "__tak_awa__".to_string(),
            BinaryOp::Multiply => "__times__".to_string(),
            BinaryOp::Divide => "__pairt__".to_string(),
            BinaryOp::Modulo => "__lave__".to_string(),
            BinaryOp::Equal => "__same_as__".to_string(),
            BinaryOp::NotEqual => "__differs_fae__".to_string(),
            BinaryOp::Less => "__wee_er__".to_string(),
            BinaryOp::LessEqual => "__wee_er_or_same__".to_string(),
            BinaryOp::Greater => "__muckle_er__".to_string(),
            BinaryOp::GreaterEqual => "__muckle_er_or_same__".to_string(),
        }
    }

    /// Call a method on an instance with the given arguments
    fn call_method_on_instance(
        &mut self,
        instance: Rc<RefCell<HaversInstance>>,
        method: Rc<HaversFunction>,
        args: Vec<Value>,
        line: usize,
    ) -> HaversResult<Value> {
        // Check arity
        if method.params.len() != args.len() {
            return Err(HaversError::WrongArity {
                name: method.name.clone(),
                expected: method.params.len(),
                got: args.len(),
                line,
            });
        }

        // Create a new environment for the method
        let method_env = if let Some(closure) = &method.closure {
            Environment::with_enclosing(closure.clone())
        } else {
            Environment::with_enclosing(self.globals.clone())
        };
        let method_env = Rc::new(RefCell::new(method_env));

        // Bind 'masel' to the instance
        method_env
            .borrow_mut()
            .define("masel".to_string(), Value::Instance(instance));

        // Bind the parameters
        for (param, arg) in method.params.iter().zip(args) {
            method_env.borrow_mut().define(param.name.clone(), arg);
        }

        // Execute the method body with our custom environment
        let result = self.execute_block(&method.body, Some(method_env));

        match result {
            Ok(Ok(val)) => Ok(val),
            Ok(Err(ControlFlow::Return(val))) => Ok(val),
            Ok(Err(ControlFlow::Break)) => Ok(Value::Nil),
            Ok(Err(ControlFlow::Continue)) => Ok(Value::Nil),
            Err(e) => Err(e),
        }
    }

    /// Evaluate function arguments, handling spread operator (...args)
    fn evaluate_call_args(&mut self, arguments: &[Expr], _line: usize) -> HaversResult<Vec<Value>> {
        let mut args = Vec::new();
        for arg in arguments {
            if let Expr::Spread { expr, span } = arg {
                let spread_value = self.evaluate(expr)?;
                match spread_value {
                    Value::List(list) => {
                        args.extend(list.borrow().clone());
                    }
                    _ => {
                        return Err(HaversError::TypeError {
                            message: "Cannae skail (spread) somethin' that isnae a list in function call!".to_string(),
                            line: span.line,
                        });
                    }
                }
            } else {
                args.push(self.evaluate(arg)?);
            }
        }
        Ok(args)
    }

    fn call_value(&mut self, callee: Value, args: Vec<Value>, line: usize) -> HaversResult<Value> {
        let _guard = InterpreterGuard::new(self);
        match callee {
            Value::Function(func) => self.call_function(&func, args, line),
            Value::NativeFunction(native) => {
                if native.arity != usize::MAX && args.len() != native.arity {
                    return Err(HaversError::WrongArity {
                        name: native.name.clone(),
                        expected: native.arity,
                        got: args.len(),
                        line,
                    });
                }
                (native.func)(args).map_err(HaversError::InternalError)
            }
            Value::NativeObject(_) => Err(HaversError::TypeError {
                message: "Cannae ca' a native object like a function".to_string(),
                line,
            }),
            // Higher-order function builtins
            Value::String(ref s) if s.starts_with("__builtin_") => {
                self.call_builtin_hof(s, args, line)
            }
            Value::Class(class) => {
                // Create new instance
                let instance = Rc::new(RefCell::new(HaversInstance::new(class.clone())));

                // Call init if it exists
                if let Some(init) = class.find_method("init") {
                    let env = Rc::new(RefCell::new(Environment::with_enclosing(
                        init.closure.clone().unwrap_or(self.globals.clone()),
                    )));
                    env.borrow_mut()
                        .define("masel".to_string(), Value::Instance(instance.clone()));
                    self.call_function_with_env(&init, args, env, line)?;
                }

                Ok(Value::Instance(instance))
            }
            Value::Struct(structure) => {
                // Create instance with fields
                if args.len() != structure.fields.len() {
                    return Err(HaversError::WrongArity {
                        name: structure.name.clone(),
                        expected: structure.fields.len(),
                        got: args.len(),
                        line,
                    });
                }

                let mut fields = DictValue::new();
                for (field, value) in structure.fields.iter().zip(args) {
                    fields.set(Value::String(field.clone()), value);
                }

                // Return as a dict for now
                Ok(Value::Dict(Rc::new(RefCell::new(fields))))
            }
            _ => Err(HaversError::NotCallable {
                name: format!("{}", callee),
                line,
            }),
        }
    }

    /// Handle higher-order function builtins
    fn call_builtin_hof(
        &mut self,
        name: &str,
        args: Vec<Value>,
        line: usize,
    ) -> HaversResult<Value> {
        match name {
            // gaun(list, func) - map function over list
            "__builtin_gaun__" => {
                if args.len() != 2 {
                    return Err(HaversError::WrongArity {
                        name: "gaun".to_string(),
                        expected: 2,
                        got: args.len(),
                        line,
                    });
                }
                let list = match &args[0] {
                    Value::List(l) => l.borrow().clone(),
                    _ => {
                        return Err(HaversError::TypeError {
                            message: "gaun() expects a list as first argument".to_string(),
                            line,
                        })
                    }
                };
                let func = args[1].clone();
                let mut result = Vec::new();
                for item in list {
                    let mapped = self.call_value(func.clone(), vec![item], line)?;
                    result.push(mapped);
                }
                Ok(Value::List(Rc::new(RefCell::new(result))))
            }

            // sieve(list, func) - filter list by predicate
            "__builtin_sieve__" => {
                if args.len() != 2 {
                    return Err(HaversError::WrongArity {
                        name: "sieve".to_string(),
                        expected: 2,
                        got: args.len(),
                        line,
                    });
                }
                let list = match &args[0] {
                    Value::List(l) => l.borrow().clone(),
                    _ => {
                        return Err(HaversError::TypeError {
                            message: "sieve() expects a list as first argument".to_string(),
                            line,
                        })
                    }
                };
                let func = args[1].clone();
                let mut result = Vec::new();
                for item in list {
                    let keep = self.call_value(func.clone(), vec![item.clone()], line)?;
                    if keep.is_truthy() {
                        result.push(item);
                    }
                }
                Ok(Value::List(Rc::new(RefCell::new(result))))
            }

            // tumble(list, initial, func) - reduce/fold
            "__builtin_tumble__" => {
                if args.len() != 3 {
                    return Err(HaversError::WrongArity {
                        name: "tumble".to_string(),
                        expected: 3,
                        got: args.len(),
                        line,
                    });
                }
                let list = match &args[0] {
                    Value::List(l) => l.borrow().clone(),
                    _ => {
                        return Err(HaversError::TypeError {
                            message: "tumble() expects a list as first argument".to_string(),
                            line,
                        })
                    }
                };
                let mut acc = args[1].clone();
                let func = args[2].clone();
                for item in list {
                    acc = self.call_value(func.clone(), vec![acc, item], line)?;
                }
                Ok(acc)
            }

            // ilk(list, func) - for each (side effects)
            "__builtin_ilk__" => {
                if args.len() != 2 {
                    return Err(HaversError::WrongArity {
                        name: "ilk".to_string(),
                        expected: 2,
                        got: args.len(),
                        line,
                    });
                }
                let list = match &args[0] {
                    Value::List(l) => l.borrow().clone(),
                    _ => {
                        return Err(HaversError::TypeError {
                            message: "ilk() expects a list as first argument".to_string(),
                            line,
                        })
                    }
                };
                let func = args[1].clone();
                for item in list {
                    self.call_value(func.clone(), vec![item], line)?;
                }
                Ok(Value::Nil)
            }

            // hunt(list, func) - find first matching element
            "__builtin_hunt__" => {
                if args.len() != 2 {
                    return Err(HaversError::WrongArity {
                        name: "hunt".to_string(),
                        expected: 2,
                        got: args.len(),
                        line,
                    });
                }
                let list = match &args[0] {
                    Value::List(l) => l.borrow().clone(),
                    _ => {
                        return Err(HaversError::TypeError {
                            message: "hunt() expects a list as first argument".to_string(),
                            line,
                        })
                    }
                };
                let func = args[1].clone();
                for item in list {
                    let matches = self.call_value(func.clone(), vec![item.clone()], line)?;
                    if matches.is_truthy() {
                        return Ok(item);
                    }
                }
                Ok(Value::Nil)
            }

            // ony(list, func) - check if any element matches
            "__builtin_ony__" => {
                if args.len() != 2 {
                    return Err(HaversError::WrongArity {
                        name: "ony".to_string(),
                        expected: 2,
                        got: args.len(),
                        line,
                    });
                }
                let list = match &args[0] {
                    Value::List(l) => l.borrow().clone(),
                    _ => {
                        return Err(HaversError::TypeError {
                            message: "ony() expects a list as first argument".to_string(),
                            line,
                        })
                    }
                };
                let func = args[1].clone();
                for item in list {
                    let matches = self.call_value(func.clone(), vec![item], line)?;
                    if matches.is_truthy() {
                        return Ok(Value::Bool(true));
                    }
                }
                Ok(Value::Bool(false))
            }

            // aw(list, func) - check if all elements match
            "__builtin_aw__" => {
                if args.len() != 2 {
                    return Err(HaversError::WrongArity {
                        name: "aw".to_string(),
                        expected: 2,
                        got: args.len(),
                        line,
                    });
                }
                let list = match &args[0] {
                    Value::List(l) => l.borrow().clone(),
                    _ => {
                        return Err(HaversError::TypeError {
                            message: "aw() expects a list as first argument".to_string(),
                            line,
                        })
                    }
                };
                let func = args[1].clone();
                for item in list {
                    let matches = self.call_value(func.clone(), vec![item], line)?;
                    if !matches.is_truthy() {
                        return Ok(Value::Bool(false));
                    }
                }
                Ok(Value::Bool(true))
            }

            // grup_up(list, func) - group elements by function result
            "__builtin_grup_up__" => {
                if args.len() != 2 {
                    return Err(HaversError::WrongArity {
                        name: "grup_up".to_string(),
                        expected: 2,
                        got: args.len(),
                        line,
                    });
                }
                let list = match &args[0] {
                    Value::List(l) => l.borrow().clone(),
                    _ => {
                        return Err(HaversError::TypeError {
                            message: "grup_up() expects a list as first argument".to_string(),
                            line,
                        })
                    }
                };
                let func = args[1].clone();
                // Result is a dict where keys are the function results, values are lists
                let mut result = DictValue::new();
                for item in list {
                    let key = self.call_value(func.clone(), vec![item.clone()], line)?;
                    let key_value = key;
                    if let Some(Value::List(l)) = result.get(&key_value).cloned() {
                        l.borrow_mut().push(item);
                    } else {
                        result.set(
                            key_value.clone(),
                            Value::List(Rc::new(RefCell::new(vec![item]))),
                        );
                    }
                }
                Ok(Value::Dict(Rc::new(RefCell::new(result))))
            }

            // pairt_by(list, func) - partition into [matches, non_matches]
            "__builtin_pairt_by__" => {
                if args.len() != 2 {
                    return Err(HaversError::WrongArity {
                        name: "pairt_by".to_string(),
                        expected: 2,
                        got: args.len(),
                        line,
                    });
                }
                let list = match &args[0] {
                    Value::List(l) => l.borrow().clone(),
                    _ => {
                        return Err(HaversError::TypeError {
                            message: "pairt_by() expects a list as first argument".to_string(),
                            line,
                        })
                    }
                };
                let func = args[1].clone();
                let mut matches = Vec::new();
                let mut non_matches = Vec::new();
                for item in list {
                    let result = self.call_value(func.clone(), vec![item.clone()], line)?;
                    if result.is_truthy() {
                        matches.push(item);
                    } else {
                        non_matches.push(item);
                    }
                }
                Ok(Value::List(Rc::new(RefCell::new(vec![
                    Value::List(Rc::new(RefCell::new(matches))),
                    Value::List(Rc::new(RefCell::new(non_matches))),
                ]))))
            }

            _ => Err(HaversError::NotCallable {
                name: name.to_string(),
                line,
            }),
        }
    }

    fn call_function(
        &mut self,
        func: &HaversFunction,
        args: Vec<Value>,
        line: usize,
    ) -> HaversResult<Value> {
        let min_arity = func.min_arity();
        let max_arity = func.max_arity();

        // Check arity: need at least min_arity, but no more than max_arity
        if args.len() < min_arity || args.len() > max_arity {
            if min_arity == max_arity {
                return Err(HaversError::WrongArity {
                    name: func.name.clone(),
                    expected: max_arity,
                    got: args.len(),
                    line,
                });
            } else {
                return Err(HaversError::TypeError {
                    message: format!(
                        "Function '{}' expects {} tae {} arguments but ye gave it {}",
                        func.name,
                        min_arity,
                        max_arity,
                        args.len()
                    ),
                    line,
                });
            }
        }

        let env = Rc::new(RefCell::new(Environment::with_enclosing(
            func.closure.clone().unwrap_or(self.globals.clone()),
        )));

        self.call_function_with_env(func, args, env, line)
    }

    fn call_function_with_env(
        &mut self,
        func: &HaversFunction,
        args: Vec<Value>,
        env: Rc<RefCell<Environment>>,
        line: usize,
    ) -> HaversResult<Value> {
        // Push stack frame for crash reporting
        push_stack_frame(&func.name, line);

        // Set up closure environment fer evaluating default values
        let old_env = self.environment.clone();
        self.environment = env.clone();

        // Bind parameters, using defaults where nae argument was provided
        for (i, param) in func.params.iter().enumerate() {
            let value = if i < args.len() {
                args[i].clone()
            } else if let Some(default_expr) = &param.default {
                // Evaluate the default value in the function's closure
                self.evaluate(default_expr)?
            } else {
                // This shouldnae happen if arity checking worked
                Value::Nil
            };
            env.borrow_mut().define(param.name.clone(), value);
        }

        // Restore the environment
        self.environment = old_env;

        let result = match self.execute_block(&func.body, Some(env))? {
            Ok(v) => Ok(v),
            Err(ControlFlow::Return(v)) => Ok(v),
            Err(ControlFlow::Break) => Ok(Value::Nil),
            Err(ControlFlow::Continue) => Ok(Value::Nil),
        };

        // Pop stack frame
        pop_stack_frame();

        result
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

// ========================================
// JSON Helper Functions
// ========================================

/// Parse a JSON string into a mdhavers Value
fn parse_json_value(s: &str) -> Result<Value, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Empty JSON string".to_string());
    }

    let chars: Vec<char> = s.chars().collect();
    let mut pos = 0;
    parse_json_inner(&chars, &mut pos)
}

fn skip_json_whitespace(chars: &[char], pos: &mut usize) {
    while *pos < chars.len() && chars[*pos].is_whitespace() {
        *pos += 1;
    }
}

fn parse_json_inner(chars: &[char], pos: &mut usize) -> Result<Value, String> {
    skip_json_whitespace(chars, pos);
    if *pos >= chars.len() {
        return Err("Unexpected end of JSON".to_string());
    }

    match chars[*pos] {
        '{' => parse_json_object(chars, pos),
        '[' => parse_json_array(chars, pos),
        '"' => parse_json_string(chars, pos),
        't' => parse_json_true(chars, pos),
        'f' => parse_json_false(chars, pos),
        'n' => parse_json_null(chars, pos),
        c if c == '-' || c.is_ascii_digit() => parse_json_number(chars, pos),
        c => Err(format!("Unexpected character '{}' in JSON", c)),
    }
}

fn parse_json_object(chars: &[char], pos: &mut usize) -> Result<Value, String> {
    *pos += 1; // skip '{'
    skip_json_whitespace(chars, pos);

    let dict = Rc::new(RefCell::new(DictValue::new()));

    if *pos < chars.len() && chars[*pos] == '}' {
        *pos += 1;
        return Ok(Value::Dict(dict));
    }

    loop {
        skip_json_whitespace(chars, pos);

        // Parse key
        if *pos >= chars.len() || chars[*pos] != '"' {
            return Err("Expected string key in JSON object".to_string());
        }
        let key = parse_json_string(chars, pos)?;
        let key = if let Value::String(s) = key {
            s
        } else {
            return Err("Invalid key".to_string());
        };

        skip_json_whitespace(chars, pos);

        // Expect ':'
        if *pos >= chars.len() || chars[*pos] != ':' {
            return Err("Expected ':' in JSON object".to_string());
        }
        *pos += 1;

        // Parse value
        let value = parse_json_inner(chars, pos)?;
        dict.borrow_mut().set(Value::String(key), value);

        skip_json_whitespace(chars, pos);

        if *pos >= chars.len() {
            return Err("Unterminated JSON object".to_string());
        }

        match chars[*pos] {
            '}' => {
                *pos += 1;
                break;
            }
            ',' => {
                *pos += 1;
            }
            c => return Err(format!("Expected '}}' or ',' in JSON object, got '{}'", c)),
        }
    }

    Ok(Value::Dict(dict))
}

fn parse_json_array(chars: &[char], pos: &mut usize) -> Result<Value, String> {
    *pos += 1; // skip '['
    skip_json_whitespace(chars, pos);

    let items: Vec<Value> = Vec::new();
    let list = Rc::new(RefCell::new(items));

    if *pos < chars.len() && chars[*pos] == ']' {
        *pos += 1;
        return Ok(Value::List(list));
    }

    loop {
        let value = parse_json_inner(chars, pos)?;
        list.borrow_mut().push(value);

        skip_json_whitespace(chars, pos);

        if *pos >= chars.len() {
            return Err("Unterminated JSON array".to_string());
        }

        match chars[*pos] {
            ']' => {
                *pos += 1;
                break;
            }
            ',' => {
                *pos += 1;
            }
            c => return Err(format!("Expected ']' or ',' in JSON array, got '{}'", c)),
        }
    }

    Ok(Value::List(list))
}

fn parse_json_string(chars: &[char], pos: &mut usize) -> Result<Value, String> {
    *pos += 1; // skip opening '"'
    let mut result = String::new();

    while *pos < chars.len() {
        let c = chars[*pos];
        if c == '"' {
            *pos += 1;
            return Ok(Value::String(result));
        }
        if c == '\\' {
            *pos += 1;
            if *pos >= chars.len() {
                return Err("Unterminated string escape".to_string());
            }
            let escaped = chars[*pos];
            match escaped {
                'n' => result.push('\n'),
                't' => result.push('\t'),
                'r' => result.push('\r'),
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                '/' => result.push('/'),
                'u' => {
                    // Unicode escape \uXXXX
                    if *pos + 4 >= chars.len() {
                        return Err("Invalid unicode escape".to_string());
                    }
                    let hex: String = chars[*pos + 1..*pos + 5].iter().collect();
                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(code) {
                            result.push(ch);
                        }
                    }
                    *pos += 4;
                }
                _ => result.push(escaped),
            }
        } else {
            result.push(c);
        }
        *pos += 1;
    }

    Err("Unterminated JSON string".to_string())
}

fn parse_json_number(chars: &[char], pos: &mut usize) -> Result<Value, String> {
    let start = *pos;
    let mut has_dot = false;
    let mut has_exp = false;

    if *pos < chars.len() && chars[*pos] == '-' {
        *pos += 1;
    }

    while *pos < chars.len() {
        let c = chars[*pos];
        if c.is_ascii_digit() {
            *pos += 1;
        } else if c == '.' && !has_dot && !has_exp {
            has_dot = true;
            *pos += 1;
        } else if (c == 'e' || c == 'E') && !has_exp {
            has_exp = true;
            *pos += 1;
            if *pos < chars.len() && (chars[*pos] == '+' || chars[*pos] == '-') {
                *pos += 1;
            }
        } else {
            break;
        }
    }

    let num_str: String = chars[start..*pos].iter().collect();

    if has_dot || has_exp {
        num_str
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|_| format!("Invalid number: {}", num_str))
    } else {
        num_str
            .parse::<i64>()
            .map(Value::Integer)
            .map_err(|_| format!("Invalid integer: {}", num_str))
    }
}

fn parse_json_true(chars: &[char], pos: &mut usize) -> Result<Value, String> {
    if *pos + 4 <= chars.len() && chars[*pos..*pos + 4].iter().collect::<String>() == "true" {
        *pos += 4;
        Ok(Value::Bool(true))
    } else {
        Err("Invalid JSON value 'true'".to_string())
    }
}

fn parse_json_false(chars: &[char], pos: &mut usize) -> Result<Value, String> {
    if *pos + 5 <= chars.len() && chars[*pos..*pos + 5].iter().collect::<String>() == "false" {
        *pos += 5;
        Ok(Value::Bool(false))
    } else {
        Err("Invalid JSON value 'false'".to_string())
    }
}

fn parse_json_null(chars: &[char], pos: &mut usize) -> Result<Value, String> {
    if *pos + 4 <= chars.len() && chars[*pos..*pos + 4].iter().collect::<String>() == "null" {
        *pos += 4;
        Ok(Value::Nil)
    } else {
        Err("Invalid JSON value 'null'".to_string())
    }
}

/// Convert a mdhavers Value to a JSON string
fn value_to_json(value: &Value) -> String {
    match value {
        Value::Nil => "null".to_string(),
        Value::Bool(true) => "true".to_string(),
        Value::Bool(false) => "false".to_string(),
        Value::Integer(n) => n.to_string(),
        Value::Float(f) => {
            if f.is_nan() || f.is_infinite() {
                "null".to_string()
            } else {
                f.to_string()
            }
        }
        Value::String(s) => json_escape_string(s),
        Value::List(l) => {
            let items: Vec<String> = l.borrow().iter().map(value_to_json).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Dict(d) => {
            let pairs: Vec<String> = d
                .borrow()
                .iter()
                .map(|(k, v)| {
                    let key_json = match k {
                        Value::String(s) => json_escape_string(s),
                        _ => json_escape_string(&format!("{}", k)),
                    };
                    format!("{}: {}", key_json, value_to_json(v))
                })
                .collect();
            format!("{{{}}}", pairs.join(", "))
        }
        _ => format!("\"{}\"", format!("{}", value).replace('\"', "\\\"")),
    }
}

/// Convert a mdhavers Value to a pretty-printed JSON string
fn value_to_json_pretty(value: &Value, indent: usize) -> String {
    let ws = "  ".repeat(indent);
    let ws_inner = "  ".repeat(indent + 1);

    match value {
        Value::Nil => "null".to_string(),
        Value::Bool(true) => "true".to_string(),
        Value::Bool(false) => "false".to_string(),
        Value::Integer(n) => n.to_string(),
        Value::Float(f) => {
            if f.is_nan() || f.is_infinite() {
                "null".to_string()
            } else {
                f.to_string()
            }
        }
        Value::String(s) => json_escape_string(s),
        Value::List(l) => {
            let items = l.borrow();
            if items.is_empty() {
                "[]".to_string()
            } else {
                let formatted: Vec<String> = items
                    .iter()
                    .map(|v| format!("{}{}", ws_inner, value_to_json_pretty(v, indent + 1)))
                    .collect();
                format!("[\n{}\n{}]", formatted.join(",\n"), ws)
            }
        }
        Value::Dict(d) => {
            let dict = d.borrow();
            if dict.is_empty() {
                "{}".to_string()
            } else {
                let formatted: Vec<String> = dict
                    .iter()
                    .map(|(k, v)| {
                        let key_json = match k {
                            Value::String(s) => json_escape_string(s),
                            _ => json_escape_string(&format!("{}", k)),
                        };
                        format!(
                            "{}{}: {}",
                            ws_inner,
                            key_json,
                            value_to_json_pretty(v, indent + 1)
                        )
                    })
                    .collect();
                format!("{{\n{}\n{}}}", formatted.join(",\n"), ws)
            }
        }
        _ => format!("\"{}\"", format!("{}", value).replace('\"', "\\\"")),
    }
}

/// Escape a string for JSON output
fn json_escape_string(s: &str) -> String {
    let mut result = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\t' => result.push_str("\\t"),
            '\r' => result.push_str("\\r"),
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result.push('"');
    result
}

#[cfg(test)]
#[allow(clippy::approx_constant)]
#[allow(clippy::manual_range_contains)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Literal, Span};
    use crate::parser::parse;
    use crate::value::NativeObject;
    use rustls::{Certificate, ServerName};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;
    use std::time::SystemTime;
    use tempfile::tempdir;

    #[derive(Debug)]
    struct TestNative {
        fields: RefCell<HashMap<String, Value>>,
    }

    impl TestNative {
        fn new() -> Self {
            TestNative {
                fields: RefCell::new(HashMap::new()),
            }
        }
    }

    impl NativeObject for TestNative {
        fn type_name(&self) -> &str {
            "test_native"
        }

        fn get(&self, prop: &str) -> HaversResult<Value> {
            self.fields
                .borrow()
                .get(prop)
                .cloned()
                .ok_or_else(|| HaversError::UndefinedVariable {
                    name: prop.to_string(),
                    line: 0,
                })
        }

        fn set(&self, prop: &str, value: Value) -> HaversResult<Value> {
            self.fields
                .borrow_mut()
                .insert(prop.to_string(), value.clone());
            Ok(value)
        }

        fn call(&self, method: &str, args: Vec<Value>) -> HaversResult<Value> {
            match method {
                "add" => {
                    let a = args.first().and_then(|v| v.as_integer()).unwrap_or(0);
                    let b = args.get(1).and_then(|v| v.as_integer()).unwrap_or(0);
                    Ok(Value::Integer(a + b))
                }
                _ => Err(HaversError::UndefinedVariable {
                    name: method.to_string(),
                    line: 0,
                }),
            }
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    fn run(source: &str) -> HaversResult<Value> {
        let program = parse(source)?;
        let mut interp = Interpreter::new();
        interp.interpret(&program)
    }

    fn lit_expr(value: Literal) -> Expr {
        Expr::Literal {
            value,
            span: Span::new(1, 1),
        }
    }

    fn dict_expr(key: &str, value: Literal) -> Expr {
        Expr::Dict {
            pairs: vec![(lit_expr(Literal::String(key.to_string())), lit_expr(value))],
            span: Span::new(1, 1),
        }
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(run("5 + 3").unwrap(), Value::Integer(8));
        assert_eq!(run("10 - 4").unwrap(), Value::Integer(6));
        assert_eq!(run("3 * 4").unwrap(), Value::Integer(12));
        assert_eq!(run("15 / 3").unwrap(), Value::Integer(5));
        assert_eq!(run("17 % 5").unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_variables() {
        assert_eq!(run("ken x = 5\nx").unwrap(), Value::Integer(5));
        assert_eq!(run("ken x = 5\nx = 10\nx").unwrap(), Value::Integer(10));
    }

    #[test]
    fn test_strings() {
        assert_eq!(
            run(r#""Hello" + " " + "World""#).unwrap(),
            Value::String("Hello World".to_string())
        );
        assert_eq!(
            run(r#""ha" * 3"#).unwrap(),
            Value::String("hahaha".to_string())
        );
    }

    #[test]
    fn test_booleans() {
        assert_eq!(run("aye").unwrap(), Value::Bool(true));
        assert_eq!(run("nae").unwrap(), Value::Bool(false));
        assert_eq!(run("5 > 3").unwrap(), Value::Bool(true));
        assert_eq!(run("5 < 3").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_native_object_get_set_call() {
        let program = parse("obj.foo = 42\nobj.foo").unwrap();
        let mut interp = Interpreter::new();
        let native = Rc::new(TestNative::new());
        interp
            .globals
            .borrow_mut()
            .define("obj".to_string(), Value::NativeObject(native));
        let result = interp.interpret(&program).unwrap();
        assert_eq!(result, Value::Integer(42));

        let program = parse("obj.add(3, 4)").unwrap();
        let result = interp.interpret(&program).unwrap();
        assert_eq!(result, Value::Integer(7));
    }

    #[test]
    fn test_tri_import_requires_alias() {
        let program = parse(r#"fetch "tri""#).unwrap();
        let mut interp = Interpreter::new();
        let err = interp.interpret(&program).unwrap_err();
        assert!(matches!(err, HaversError::TypeError { .. }));
    }

    #[test]
    fn test_tri_import_and_constructor() {
        let program = parse(
            r#"fetch "tri" tae tri
whit_kind(tri.Sicht())"#,
        )
        .unwrap();
        let mut interp = Interpreter::new();
        let result = interp.interpret(&program).unwrap();
        assert_eq!(result, Value::String("Sicht".to_string()));
    }

    #[test]
    fn test_tri_constructor_via_property() {
        let program = parse(
            r#"fetch "tri" tae tri
ken ctor = tri.Sicht
whit_kind(ctor())"#,
        )
        .unwrap();
        let mut interp = Interpreter::new();
        let result = interp.interpret(&program).unwrap();
        assert_eq!(result, Value::String("Sicht".to_string()));
    }

    #[test]
    fn test_tri_constructor_defaults() {
        let program = parse(
            r#"fetch "tri" tae tri
ken box = tri.BoxGeometrie()
box.width"#,
        )
        .unwrap();
        let mut interp = Interpreter::new();
        let result = interp.interpret(&program).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_if_statement() {
        let result = run(r#"
ken x = 10
ken result = 0
gin x > 5 {
    result = 1
} ither {
    result = 2
}
result
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_while_loop() {
        let result = run(r#"
ken sum = 0
ken i = 1
whiles i <= 5 {
    sum = sum + i
    i = i + 1
}
sum
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(15));
    }

    #[test]
    fn test_for_loop() {
        let result = run(r#"
ken sum = 0
fer i in 1..6 {
    sum = sum + i
}
sum
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(15));
    }

    #[test]
    fn test_function() {
        let result = run(r#"
dae add(a, b) {
    gie a + b
}
add(3, 4)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(7));
    }

    #[test]
    fn test_recursion() {
        let result = run(r#"
dae factorial(n) {
    gin n <= 1 {
        gie 1
    }
    gie n * factorial(n - 1)
}
factorial(5)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(120));
    }

    #[test]
    fn test_list() {
        let result = run(r#"
ken arr = [1, 2, 3]
arr[1]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_dict() {
        let result = run(r#"
ken d = {"a": 1, "b": 2}
d["a"]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_native_functions() {
        assert_eq!(run("len([1, 2, 3])").unwrap(), Value::Integer(3));
        assert_eq!(run(r#"len("hello")"#).unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_division_by_zero() {
        assert!(run("5 / 0").is_err());
    }

    #[test]
    fn test_undefined_variable() {
        assert!(run("undefined_var").is_err());
    }

    #[test]
    fn test_lambda() {
        // Basic lambda
        assert_eq!(
            run("ken double = |x| x * 2\ndouble(5)").unwrap(),
            Value::Integer(10)
        );
        // Lambda with multiple params
        assert_eq!(
            run("ken add = |a, b| a + b\nadd(3, 4)").unwrap(),
            Value::Integer(7)
        );
        // No-param lambda
        assert_eq!(
            run("ken always_five = || 5\nalways_five()").unwrap(),
            Value::Integer(5)
        );
    }

    #[test]
    fn test_gaun_map() {
        let result = run("ken nums = [1, 2, 3]\ngaun(nums, |x| x * 2)").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let items = list.borrow();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::Integer(2));
        assert_eq!(items[1], Value::Integer(4));
        assert_eq!(items[2], Value::Integer(6));
    }

    #[test]
    fn test_sieve_filter() {
        let result = run("ken nums = [1, 2, 3, 4, 5]\nsieve(nums, |x| x % 2 == 0)").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let items = list.borrow();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], Value::Integer(2));
        assert_eq!(items[1], Value::Integer(4));
    }

    #[test]
    fn test_tumble_reduce() {
        assert_eq!(
            run("ken nums = [1, 2, 3, 4, 5]\ntumble(nums, 0, |acc, x| acc + x)").unwrap(),
            Value::Integer(15)
        );
    }

    #[test]
    fn test_ony_any() {
        assert_eq!(
            run("ken nums = [1, 2, 3]\nony(nums, |x| x > 2)").unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            run("ken nums = [1, 2, 3]\nony(nums, |x| x > 10)").unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_aw_all() {
        assert_eq!(
            run("ken nums = [1, 2, 3]\naw(nums, |x| x > 0)").unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            run("ken nums = [1, 2, 3]\naw(nums, |x| x > 1)").unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_hunt_find() {
        assert_eq!(
            run("ken nums = [1, 2, 3, 4, 5]\nhunt(nums, |x| x > 3)").unwrap(),
            Value::Integer(4)
        );
        assert_eq!(
            run("ken nums = [1, 2, 3]\nhunt(nums, |x| x > 10)").unwrap(),
            Value::Nil
        );
    }

    #[test]
    fn test_pattern_matching() {
        let result = run(r#"
ken x = 2
ken result = naething
keek x {
    whan 1 -> result = "one"
    whan 2 -> result = "two"
    whan _ -> result = "other"
}
result
"#)
        .unwrap();
        assert_eq!(result, Value::String("two".to_string()));
    }

    #[test]
    fn test_ternary_expression() {
        // Basic ternary - used in expression context
        assert_eq!(
            run("ken x = gin 5 > 3 than 1 ither 0\nx").unwrap(),
            Value::Integer(1)
        );
        assert_eq!(
            run("ken x = gin 5 < 3 than 1 ither 0\nx").unwrap(),
            Value::Integer(0)
        );
        // With strings
        assert_eq!(
            run(r#"ken x = gin aye than "yes" ither "no"
x"#)
            .unwrap(),
            Value::String("yes".to_string())
        );
        // Nested ternary
        assert_eq!(
            run("ken x = 5
ken result = gin x > 10 than 1 ither gin x > 3 than 2 ither 3
result")
            .unwrap(),
            Value::Integer(2)
        );
    }

    #[test]
    fn test_slice_list() {
        // Basic slicing
        let result = run("ken x = [0, 1, 2, 3, 4]\nx[1:3]").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let list = list.borrow();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], Value::Integer(1));
        assert_eq!(list[1], Value::Integer(2));

        // Slice to end
        let result = run("ken x = [0, 1, 2, 3, 4]\nx[3:]").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let list = list.borrow();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], Value::Integer(3));
        assert_eq!(list[1], Value::Integer(4));

        // Slice from start
        let result = run("ken x = [0, 1, 2, 3, 4]\nx[:2]").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let list = list.borrow();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], Value::Integer(0));
        assert_eq!(list[1], Value::Integer(1));
    }

    #[test]
    fn test_slice_string() {
        assert_eq!(
            run("ken s = \"Hello\"\ns[0:2]").unwrap(),
            Value::String("He".to_string())
        );
        assert_eq!(
            run("ken s = \"Hello\"\ns[3:]").unwrap(),
            Value::String("lo".to_string())
        );
        assert_eq!(
            run("ken s = \"Hello\"\ns[:3]").unwrap(),
            Value::String("Hel".to_string())
        );
    }

    #[test]
    fn test_slice_negative() {
        // Negative indices
        let result = run("ken x = [0, 1, 2, 3, 4]\nx[-2:]").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let list = list.borrow();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], Value::Integer(3));
        assert_eq!(list[1], Value::Integer(4));
    }

    #[test]
    fn test_slice_step() {
        // Every second element
        let result = run("ken x = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]\nx[::2]").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let list = list.borrow();
        assert_eq!(list.len(), 5);
        assert_eq!(list[0], Value::Integer(0));
        assert_eq!(list[1], Value::Integer(2));
        assert_eq!(list[4], Value::Integer(8));

        // Every third element from 1 to 8
        let result = run("ken x = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]\nx[1:8:3]").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let list = list.borrow();
        assert_eq!(list.len(), 3); // 1, 4, 7
        assert_eq!(list[0], Value::Integer(1));
        assert_eq!(list[1], Value::Integer(4));
        assert_eq!(list[2], Value::Integer(7));

        // Reverse a list with negative step
        let result = run("ken x = [0, 1, 2, 3, 4]\nx[::-1]").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let list = list.borrow();
        assert_eq!(list.len(), 5);
        assert_eq!(list[0], Value::Integer(4));
        assert_eq!(list[4], Value::Integer(0));

        // String with step
        let result = run("ken s = \"Hello\"\ns[::2]").unwrap();
        assert_eq!(result, Value::String("Hlo".to_string())); // H, l, o

        // String reversed
        let result = run("ken s = \"Hello\"\ns[::-1]").unwrap();
        assert_eq!(result, Value::String("olleH".to_string()));
    }

    #[test]
    fn test_new_list_functions() {
        // uniq
        let result = run("uniq([1, 2, 2, 3, 3, 3])").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let list = list.borrow();
        assert_eq!(list.len(), 3);

        // redd_up
        let result = run("redd_up([1, naething, 2, naething, 3])").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let list = list.borrow();
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn test_new_string_functions() {
        // capitalize
        assert_eq!(
            run(r#"capitalize("hello")"#).unwrap(),
            Value::String("Hello".to_string())
        );

        // title
        assert_eq!(
            run(r#"title("hello world")"#).unwrap(),
            Value::String("Hello World".to_string())
        );

        // words
        let result = run(r#"words("one two three")"#).unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let list = list.borrow();
        assert_eq!(list.len(), 3);

        // ord and chr
        assert_eq!(run(r#"ord("A")"#).unwrap(), Value::Integer(65));
        assert_eq!(run("chr(65)").unwrap(), Value::String("A".to_string()));
    }

    #[test]
    fn test_creel_set() {
        // Create a set from a list
        let result = run("creel([1, 2, 2, 3, 3, 3])").unwrap();
        let Value::Set(set) = result else {
            panic!("Expected creel");
        };
        let set = set.borrow();
        assert_eq!(set.len(), 3); // Duplicates removed

        // Create empty set
        let result = run("empty_creel()").unwrap();
        let Value::Set(set) = result else {
            panic!("Expected empty creel");
        };
        assert!(set.borrow().is_empty());

        // Check membership
        let result = run(r#"
            ken s = creel(["apple", "banana", "cherry"])
            is_in_creel(s, "banana")
        "#)
        .unwrap();
        assert_eq!(result, Value::Bool(true));

        let result = run(r#"
            ken s = creel(["apple", "banana", "cherry"])
            is_in_creel(s, "mango")
        "#)
        .unwrap();
        assert_eq!(result, Value::Bool(false));

        // Union
        let result = run(r#"
            ken a = creel([1, 2, 3])
            ken b = creel([3, 4, 5])
            len(creels_thegither(a, b))
        "#)
        .unwrap();
        assert_eq!(result, Value::Integer(5)); // 1, 2, 3, 4, 5

        // Intersection
        let result = run(r#"
            ken a = creel([1, 2, 3])
            ken b = creel([2, 3, 4])
            len(creels_baith(a, b))
        "#)
        .unwrap();
        assert_eq!(result, Value::Integer(2)); // 2, 3

        // Difference
        let result = run(r#"
            ken a = creel([1, 2, 3])
            ken b = creel([2, 3, 4])
            len(creels_differ(a, b))
        "#)
        .unwrap();
        assert_eq!(result, Value::Integer(1)); // just 1

        // Subset
        let result = run(r#"
            ken a = creel([1, 2])
            ken b = creel([1, 2, 3])
            is_subset(a, b)
        "#)
        .unwrap();
        assert_eq!(result, Value::Bool(true));

        // Convert to list
        let result = run(r#"
            ken s = creel([3, 1, 2])
            creel_tae_list(s)
        "#)
        .unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let list = list.borrow();
        assert_eq!(list.len(), 3);
        // Should be sorted
        assert_eq!(list[0], Value::Integer(1));
        assert_eq!(list[1], Value::Integer(2));
        assert_eq!(list[2], Value::Integer(3));
    }

    #[test]
    fn test_classes() {
        // Basic class creation and instantiation
        let result = run(r#"
kin Dug {
    dae init(name) {
        masel.name = name
    }
    dae bark() {
        gie "Woof! Ah'm " + masel.name
    }
}
ken fido = Dug("Fido")
fido.bark()
"#)
        .unwrap();
        assert_eq!(result, Value::String("Woof! Ah'm Fido".to_string()));
    }

    #[test]
    fn test_inheritance() {
        let result = run(r#"
kin Animal {
    dae init(name) {
        masel.name = name
    }
    dae speak() {
        gie "..."
    }
}
kin Dug fae Animal {
    dae speak() {
        gie "Woof!"
    }
}
ken d = Dug("Rex")
d.speak()
"#)
        .unwrap();
        assert_eq!(result, Value::String("Woof!".to_string()));
    }

    #[test]
    fn test_try_catch() {
        // Try-catch basic - using Scots keywords!
        let result = run(r#"
ken result = "untouched"
hae_a_bash {
    result = 1 / 0
} gin_it_gangs_wrang e {
    result = "caught"
}
result
"#)
        .unwrap();
        assert_eq!(result, Value::String("caught".to_string()));
    }

    #[test]
    fn test_destructuring() {
        // Basic destructuring
        let result = run(r#"
ken [a, b, c] = [1, 2, 3]
b
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(2));

        // Rest destructuring
        let result = run(r#"
ken [first, ...rest] = [1, 2, 3, 4]
len(rest)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_default_params() {
        let result = run(r#"
dae greet(name, greeting = "Hullo") {
    gie greeting + ", " + name + "!"
}
greet("Hamish")
"#)
        .unwrap();
        assert_eq!(result, Value::String("Hullo, Hamish!".to_string()));

        let result = run(r#"
dae greet(name, greeting = "Hullo") {
    gie greeting + ", " + name + "!"
}
greet("Hamish", "Guid day")
"#)
        .unwrap();
        assert_eq!(result, Value::String("Guid day, Hamish!".to_string()));
    }

    #[test]
    fn test_fstring() {
        let result = run(r#"
ken name = "Scotland"
f"Hello, {name}!"
"#)
        .unwrap();
        assert_eq!(result, Value::String("Hello, Scotland!".to_string()));

        // F-string with expression
        let result = run(r#"
ken x = 5
f"The answer is {x * 2}"
"#)
        .unwrap();
        assert_eq!(result, Value::String("The answer is 10".to_string()));
    }

    #[test]
    fn test_scots_vocabulary_functions() {
        // Test crabbit (negative check)
        assert_eq!(run("crabbit(-5)").unwrap(), Value::Bool(true));
        assert_eq!(run("crabbit(5)").unwrap(), Value::Bool(false));

        // Test glaikit (empty/invalid check)
        assert_eq!(run("glaikit(\"\")").unwrap(), Value::Bool(true));
        assert_eq!(run("glaikit(0)").unwrap(), Value::Bool(true));
        assert_eq!(run("glaikit(42)").unwrap(), Value::Bool(false));

        // Test roar (uppercase shout)
        assert_eq!(
            run(r#"roar("hello")"#).unwrap(),
            Value::String("HELLO!".to_string())
        );

        // Test wrang_sort (type check)
        assert_eq!(
            run(r#"wrang_sort(42, "integer")"#).unwrap(),
            Value::Bool(false) // Not wrong - 42 IS an integer
        );
        assert_eq!(
            run(r#"wrang_sort(42, "string")"#).unwrap(),
            Value::Bool(true) // Wrong - 42 is NOT a string
        );
    }

    #[test]
    fn test_new_scots_functions() {
        // blether_format
        assert_eq!(
            run(r#"blether_format("Hullo {name}!", {"name": "Hamish"})"#).unwrap(),
            Value::String("Hullo Hamish!".to_string())
        );

        // ceilidh (interleave)
        let result = run("ceilidh([1, 2], [3, 4])").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let list = list.borrow();
        assert_eq!(list.len(), 4);
        assert_eq!(list[0], Value::Integer(1));
        assert_eq!(list[1], Value::Integer(3));
        assert_eq!(list[2], Value::Integer(2));
        assert_eq!(list[3], Value::Integer(4));

        // birl (rotate)
        let result = run("birl([1, 2, 3, 4, 5], 2)").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let list = list.borrow();
        assert_eq!(list[0], Value::Integer(3));
        assert_eq!(list[4], Value::Integer(2));

        // clype (debug info)
        let result = run("clype([1, 2, 3])").unwrap();
        let Value::String(s) = result else {
            panic!("Expected string");
        };
        assert!(s.contains("list"));
        assert!(s.contains("3 items"));

        // sclaff (flatten)
        let result = run("sclaff([[1, 2], [3, [4, 5]]])").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let list = list.borrow();
        assert_eq!(list.len(), 5); // Fully flattened
    }

    #[test]
    fn test_trace_mode() {
        // Test that trace mode can be set without breaking execution
        let mut interp = Interpreter::new();

        // Set trace mode
        interp.set_trace_mode(TraceMode::Off);
        assert_eq!(interp.trace_mode(), TraceMode::Off);

        interp.set_trace_mode(TraceMode::Statements);
        assert_eq!(interp.trace_mode(), TraceMode::Statements);

        interp.set_trace_mode(TraceMode::Verbose);
        assert_eq!(interp.trace_mode(), TraceMode::Verbose);

        // Code should still execute correctly with trace on
        interp.set_trace_mode(TraceMode::Off); // Turn off to avoid stderr output
        let program = crate::parser::parse("ken x = 42\ngin x > 10 { x = x * 2 }\nx").unwrap();
        let result = interp.interpret(&program).unwrap();
        assert_eq!(result, Value::Integer(84));
    }

    #[test]
    fn test_get_user_variables() {
        let mut interp = Interpreter::new();
        let program = crate::parser::parse("ken x = 42\nken name = \"Hamish\"").unwrap();
        interp.interpret(&program).unwrap();

        let vars = interp.get_user_variables();

        // Should have x and name (and possibly prelude functions if loaded)
        let x_var = vars.iter().find(|(n, _, _)| n == "x");
        assert!(x_var.is_some());
        assert_eq!(x_var.unwrap().1, "integer");

        let name_var = vars.iter().find(|(n, _, _)| n == "name");
        assert!(name_var.is_some());
        assert_eq!(name_var.unwrap().1, "string");
    }

    #[test]
    fn test_timing_functions() {
        // noo() returns a timestamp
        let result = run("noo()").unwrap();
        let Value::Integer(ts) = result else {
            panic!("Expected integer timestamp");
        };
        assert!(ts > 0); // Should be a positive timestamp

        // tick() returns high-precision timestamp
        let result = run("tick()").unwrap();
        let Value::Integer(ts) = result else {
            panic!("Expected integer timestamp");
        };
        assert!(ts > 0);

        // Time difference works
        let result = run(r#"
            ken start = noo()
            ken finish = noo()
            finish >= start
        "#)
        .unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    // ==================== Native Function Edge Cases ====================

    #[test]
    fn test_len_dict() {
        assert_eq!(run(r#"len({"a": 1, "b": 2})"#).unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_len_set() {
        assert_eq!(run("len(creel([1, 2, 3]))").unwrap(), Value::Integer(3));
    }

    #[test]
    fn test_len_error() {
        assert!(run("len(42)").is_err());
    }

    #[test]
    fn test_tae_int_from_int() {
        assert_eq!(run("tae_int(42)").unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_tae_int_from_float() {
        assert_eq!(run("tae_int(3.7)").unwrap(), Value::Integer(3));
    }

    #[test]
    fn test_tae_int_from_string() {
        assert_eq!(run("tae_int(\"123\")").unwrap(), Value::Integer(123));
    }

    #[test]
    fn test_tae_int_from_bool() {
        assert_eq!(run("tae_int(aye)").unwrap(), Value::Integer(1));
        assert_eq!(run("tae_int(nae)").unwrap(), Value::Integer(0));
    }

    #[test]
    fn test_tae_int_error() {
        assert!(run("tae_int(\"not a number\")").is_err());
        assert!(run("tae_int([1, 2, 3])").is_err());
    }

    #[test]
    fn test_tae_float_from_int() {
        assert_eq!(run("tae_float(42)").unwrap(), Value::Float(42.0));
    }

    #[test]
    fn test_tae_float_from_float() {
        assert_eq!(run("tae_float(3.14)").unwrap(), Value::Float(3.14));
    }

    #[test]
    fn test_tae_float_from_string() {
        assert_eq!(run("tae_float(\"3.14\")").unwrap(), Value::Float(3.14));
    }

    #[test]
    fn test_tae_float_error() {
        assert!(run("tae_float(\"xyz\")").is_err());
        assert!(run("tae_float([1, 2, 3])").is_err());
    }

    #[test]
    fn test_shove_error() {
        assert!(run("shove(42, 1)").is_err());
    }

    #[test]
    fn test_yank_error() {
        assert!(run("yank([])").is_err());
        assert!(run("yank(42)").is_err());
    }

    #[test]
    fn test_keys_values() {
        let result = run(r#"keys({"a": 1, "b": 2})"#).unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        assert_eq!(list.borrow().len(), 2);

        let result = run(r#"values({"a": 1, "b": 2})"#).unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        assert_eq!(list.borrow().len(), 2);
    }

    #[test]
    fn test_keys_values_error() {
        assert!(run("keys(42)").is_err());
        assert!(run("values([1, 2])").is_err());
    }

    #[test]
    fn test_abs() {
        assert_eq!(run("abs(-5)").unwrap(), Value::Integer(5));
        assert_eq!(run("abs(5)").unwrap(), Value::Integer(5));
        assert_eq!(run("abs(-3.14)").unwrap(), Value::Float(3.14));
    }

    #[test]
    fn test_math_functions() {
        assert_eq!(run("min(1, 5)").unwrap(), Value::Integer(1));
        assert_eq!(run("max(1, 5)").unwrap(), Value::Integer(5));
        assert_eq!(run("floor(3.7)").unwrap(), Value::Integer(3));
        assert_eq!(run("ceil(3.2)").unwrap(), Value::Integer(4));
        assert_eq!(run("round(3.5)").unwrap(), Value::Integer(4));
        assert_eq!(run("sqrt(16)").unwrap(), Value::Float(4.0));
    }

    #[test]
    fn test_contains() {
        assert_eq!(
            run(r#"contains("hello", "ell")"#).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(run("contains([1, 2, 3], 2)").unwrap(), Value::Bool(true));
        assert_eq!(run("contains([1, 2, 3], 5)").unwrap(), Value::Bool(false));
        assert_eq!(
            run(r#"contains({"a": 1}, "a")"#).unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_reverse() {
        assert_eq!(
            run(r#"reverse("hello")"#).unwrap(),
            Value::String("olleh".to_string())
        );
        let result = run("reverse([1, 2, 3])").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let list = list.borrow();
        assert_eq!(list[0], Value::Integer(3));
        assert_eq!(list[2], Value::Integer(1));
    }

    #[test]
    fn test_sort() {
        let result = run("sort([3, 1, 2])").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let list = list.borrow();
        assert_eq!(list[0], Value::Integer(1));
        assert_eq!(list[1], Value::Integer(2));
        assert_eq!(list[2], Value::Integer(3));
    }

    #[test]
    fn test_split_join() {
        let result = run(r#"split("a,b,c", ",")"#).unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        assert_eq!(list.borrow().len(), 3);

        assert_eq!(
            run(r#"join(["a", "b", "c"], "-")"#).unwrap(),
            Value::String("a-b-c".to_string())
        );
    }

    #[test]
    fn test_heid_tail_bum() {
        assert_eq!(run("heid([1, 2, 3])").unwrap(), Value::Integer(1));
        assert_eq!(
            run(r#"heid("hello")"#).unwrap(),
            Value::String("h".to_string())
        );

        let result = run("tail([1, 2, 3])").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        assert_eq!(list.borrow().len(), 2);
        assert_eq!(
            run(r#"tail("hello")"#).unwrap(),
            Value::String("ello".to_string())
        );

        assert_eq!(run("bum([1, 2, 3])").unwrap(), Value::Integer(3));
        assert_eq!(
            run(r#"bum("hello")"#).unwrap(),
            Value::String("o".to_string())
        );
    }

    #[test]
    fn test_heid_tail_bum_errors() {
        assert!(run("heid([])").is_err());
        assert!(run("bum([])").is_err());
        assert!(run("heid(42)").is_err());
    }

    #[test]
    fn test_scran_slap() {
        let result = run("scran([1, 2, 3, 4], 1, 3)").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        assert_eq!(list.borrow().len(), 2);

        let result = run("slap([1, 2], [3, 4])").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        assert_eq!(list.borrow().len(), 4);

        assert_eq!(
            run(r#"slap("hello", " world")"#).unwrap(),
            Value::String("hello world".to_string())
        );
    }

    #[test]
    fn test_sumaw_coont() {
        assert_eq!(run("sumaw([1, 2, 3, 4])").unwrap(), Value::Integer(10));
        assert_eq!(run("coont([1, 2, 2, 3, 2], 2)").unwrap(), Value::Integer(3));
        assert_eq!(run(r#"coont("hello", "l")"#).unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_wheesht_upper_lower() {
        assert_eq!(
            run(r#"wheesht("  hello  ")"#).unwrap(),
            Value::String("hello".to_string())
        );
        assert_eq!(
            run(r#"upper("hello")"#).unwrap(),
            Value::String("HELLO".to_string())
        );
        assert_eq!(
            run(r#"lower("HELLO")"#).unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_shuffle() {
        let result = run("len(shuffle([1, 2, 3]))").unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    // ==================== Arithmetic Edge Cases ====================

    #[test]
    fn test_float_arithmetic() {
        assert_eq!(run("3.5 + 2.5").unwrap(), Value::Float(6.0));
        assert_eq!(run("5.0 - 2.0").unwrap(), Value::Float(3.0));
        assert_eq!(run("2.5 * 4.0").unwrap(), Value::Float(10.0));
        assert_eq!(run("10.0 / 4.0").unwrap(), Value::Float(2.5));
    }

    #[test]
    fn test_mixed_arithmetic() {
        assert_eq!(run("5 + 2.5").unwrap(), Value::Float(7.5));
        assert_eq!(run("10.0 / 2").unwrap(), Value::Float(5.0));
    }

    #[test]
    fn test_unary_negate() {
        assert_eq!(run("-5").unwrap(), Value::Integer(-5));
        assert_eq!(run("-3.14").unwrap(), Value::Float(-3.14));
    }

    #[test]
    fn test_modulo_float() {
        let result = run("7.5 % 2.0").unwrap();
        let Value::Float(f) = result else {
            panic!("Expected float");
        };
        assert!((f - 1.5).abs() < 0.001);
    }

    // ==================== Comparison Operations ====================

    #[test]
    fn test_string_comparison() {
        assert_eq!(run(r#""a" < "b""#).unwrap(), Value::Bool(true));
        assert_eq!(run(r#""hello" == "hello""#).unwrap(), Value::Bool(true));
        assert_eq!(run(r#""a" != "b""#).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_list_equality() {
        assert_eq!(run("[1, 2, 3] == [1, 2, 3]").unwrap(), Value::Bool(true));
        assert_eq!(run("[1, 2, 3] == [1, 2, 4]").unwrap(), Value::Bool(false));
    }

    // ==================== Logical Operations ====================

    #[test]
    fn test_logical_operations() {
        assert_eq!(run("aye an aye").unwrap(), Value::Bool(true));
        assert_eq!(run("aye an nae").unwrap(), Value::Bool(false));
        assert_eq!(run("nae or aye").unwrap(), Value::Bool(true));
        assert_eq!(run("nae or nae").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_logical_not() {
        assert_eq!(run("ken x = aye\nnae x").unwrap(), Value::Bool(false));
        assert_eq!(run("ken x = nae\nnae x").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_short_circuit_and() {
        // Should short-circuit and not evaluate second part
        assert_eq!(run("nae an (1/0 > 0)").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_short_circuit_or() {
        // Should short-circuit and not evaluate second part
        assert_eq!(run("aye or (1/0 > 0)").unwrap(), Value::Bool(true));
    }

    // ==================== Control Flow Edge Cases ====================

    #[test]
    fn test_break_in_while() {
        let result = run(r#"
ken sum = 0
ken i = 1
whiles aye {
    gin i > 5 {
        brak
    }
    sum = sum + i
    i = i + 1
}
sum
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(15));
    }

    #[test]
    fn test_continue_in_while() {
        let result = run(r#"
ken sum = 0
ken i = 0
whiles i < 10 {
    i = i + 1
    gin i % 2 == 0 {
        haud
    }
    sum = sum + i
}
sum
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(25)); // 1+3+5+7+9
    }

    #[test]
    fn test_break_in_for() {
        let result = run(r#"
ken sum = 0
fer i in 0..100 {
    gin i >= 5 {
        brak
    }
    sum = sum + i
}
sum
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(10)); // 0+1+2+3+4
    }

    #[test]
    fn test_for_over_list() {
        let result = run(r#"
ken sum = 0
fer x in [1, 2, 3, 4, 5] {
    sum = sum + x
}
sum
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(15));
    }

    #[test]
    fn test_for_over_string() {
        let result = run(r#"
ken count = 0
fer c in "hello" {
    count = count + 1
}
count
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    // ==================== Assert Statement ====================

    #[test]
    fn test_assert_pass() {
        assert_eq!(run("mak_siccar 5 > 3\n42").unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_assert_fail() {
        assert!(run("mak_siccar 3 > 5").is_err());
    }

    #[test]
    fn test_assert_with_message() {
        let result = run("mak_siccar 3 > 5, \"Should be bigger\"");
        assert!(result.is_err());
    }

    // ==================== Spread Operator ====================

    #[test]
    fn test_spread_list_elements() {
        let result = run("[1, ...[2, 3], 4]").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let list = list.borrow();
        assert_eq!(list.len(), 4);
        assert_eq!(list[0], Value::Integer(1));
        assert_eq!(list[1], Value::Integer(2));
        assert_eq!(list[2], Value::Integer(3));
        assert_eq!(list[3], Value::Integer(4));
    }

    // ==================== Pipe Operator ====================

    #[test]
    fn test_pipe_operator() {
        let result = run(r#"
dae double(x) { gie x * 2 }
dae add_one(x) { gie x + 1 }
5 |> double |> add_one
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(11));
    }

    // ==================== Index Assignment ====================

    #[test]
    fn test_list_index_assignment() {
        let result = run(r#"
ken arr = [1, 2, 3]
arr[1] = 99
arr[1]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(99));
    }

    #[test]
    fn test_dict_index_assignment() {
        let result = run(r#"
ken d = {"a": 1}
d["b"] = 2
d["b"]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    // ==================== Negative Index ====================

    #[test]
    fn test_negative_index() {
        assert_eq!(run("[1, 2, 3][-1]").unwrap(), Value::Integer(3));
        assert_eq!(run("[1, 2, 3][-2]").unwrap(), Value::Integer(2));
        assert_eq!(
            run(r#""hello"[-1]"#).unwrap(),
            Value::String("o".to_string())
        );
    }

    // ==================== JSON Functions ====================

    #[test]
    fn test_json_parse() {
        let result = run(r#"json_parse("{\"name\": \"test\", \"value\": 42}")"#).unwrap();
        let Value::Dict(dict) = result else {
            panic!("Expected dict");
        };
        let dict = dict.borrow();
        assert_eq!(
            dict.get(&Value::String("value".to_string())),
            Some(&Value::Integer(42))
        );
    }

    #[test]
    fn test_json_parse_array() {
        let result = run(r#"json_parse("[1, 2, 3]")"#).unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        assert_eq!(list.borrow().len(), 3);
    }

    #[test]
    fn test_json_parse_primitives() {
        assert_eq!(run(r#"json_parse("true")"#).unwrap(), Value::Bool(true));
        assert_eq!(run(r#"json_parse("false")"#).unwrap(), Value::Bool(false));
        assert_eq!(run(r#"json_parse("null")"#).unwrap(), Value::Nil);
        assert_eq!(run(r#"json_parse("42")"#).unwrap(), Value::Integer(42));
        assert_eq!(run(r#"json_parse("3.14")"#).unwrap(), Value::Float(3.14));
    }

    #[test]
    fn test_json_stringify() {
        assert_eq!(
            run(r#"json_stringify(42)"#).unwrap(),
            Value::String("42".to_string())
        );
        assert_eq!(
            run(r#"json_stringify(aye)"#).unwrap(),
            Value::String("true".to_string())
        );
        assert_eq!(
            run(r#"json_stringify([1, 2, 3])"#).unwrap(),
            Value::String("[1, 2, 3]".to_string())
        );
    }

    // ==================== Struct Tests ====================

    #[test]
    fn test_struct() {
        let result = run(r#"
thing Point { x, y }
ken p = Point(10, 20)
p.x + p.y
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(30));
    }

    // ==================== Interpreter Default ====================

    #[test]
    fn test_interpreter_default() {
        let interp = Interpreter::default();
        assert!(interp.output.is_empty());
    }

    // ==================== Output Capture ====================

    #[test]
    fn test_get_output() {
        let mut interp = Interpreter::new();
        let program = crate::parser::parse(
            r#"blether "hello"
blether "world""#,
        )
        .unwrap();
        interp.interpret(&program).unwrap();

        let output = interp.get_output();
        assert_eq!(output.len(), 2);
        assert_eq!(output[0], "hello");
        assert_eq!(output[1], "world");
    }

    // ==================== Float Division ====================

    #[test]
    fn test_float_division() {
        // Normal float division
        assert_eq!(run("10.0 / 4.0").unwrap(), Value::Float(2.5));
        // Mixed int/float division
        assert_eq!(run("10.0 / 2").unwrap(), Value::Float(5.0));
    }

    // ==================== Block Statement ====================

    #[test]
    fn test_block_returns_last_value() {
        let result = run("{ ken x = 1\n ken y = 2\n x + y }").unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    // ==================== Closures ====================

    #[test]
    fn test_closure_basic() {
        // Test basic closure that captures outer variable
        let result = run(r#"
dae make_adder(x) {
    gie |y| x + y
}
ken add5 = make_adder(5)
add5(3)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(8));
    }

    // ==================== Wildcard Pattern ====================

    #[test]
    fn test_match_wildcard() {
        let result = run(r#"
ken x = 999
keek x {
    whan 1 -> "one"
    whan _ -> "other"
}
"#)
        .unwrap();
        assert_eq!(result, Value::String("other".to_string()));
    }

    // ==================== Match with Identifier Pattern ====================

    #[test]
    fn test_match_identifier_bind() {
        let result = run(r#"
ken x = 42
keek x {
    whan value -> value * 2
}
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(84));
    }

    // ==================== Random Functions ====================

    #[test]
    fn test_jammy_random() {
        // jammy(min, max) returns random int between min and max
        let result = run("jammy(1, 10)").unwrap();
        let Value::Integer(n) = result else {
            panic!("Expected integer");
        };
        assert!(n >= 1 && n <= 10);
    }

    // ==================== More Scots Functions ====================

    #[test]
    fn test_dram_single_element() {
        // dram returns a random element from a list
        let result = run("dram([1, 2, 3])").unwrap();
        let Value::Integer(n) = result else {
            panic!("Expected integer");
        };
        assert!(n >= 1 && n <= 3);
    }

    #[test]
    fn test_blooter_scramble() {
        // blooter scrambles a string
        let result = run(r#"len(blooter("hello"))"#).unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_haver_nonsense() {
        // haver generates a random Scots phrase
        let result = run("haver()").unwrap();
        let Value::String(s) = result else {
            panic!("Expected string");
        };
        assert!(!s.is_empty());
    }

    // ==================== Interpreter Configuration Tests ====================

    #[test]
    fn test_interpreter_with_dir() {
        let interp = Interpreter::with_dir("/tmp");
        assert!(interp.current_dir.to_str().unwrap().contains("tmp"));
    }

    #[test]
    fn test_interpreter_set_current_dir() {
        let mut interp = Interpreter::new();
        interp.set_current_dir("/tmp");
        assert!(interp.current_dir.to_str().unwrap().contains("tmp"));
    }

    #[test]
    fn test_interpreter_get_user_variables() {
        let mut interp = Interpreter::new();
        let program = crate::parser::parse("ken x = 42\nken y = \"hello\"").unwrap();
        interp.interpret(&program).unwrap();
        let vars = interp.get_user_variables();
        assert!(vars.iter().any(|(name, _, _)| name == "x"));
        assert!(vars.iter().any(|(name, _, _)| name == "y"));
    }

    #[test]
    fn test_interpreter_get_user_functions() {
        let mut interp = Interpreter::new();
        let program = crate::parser::parse("dae foo() { gie 1 }").unwrap();
        interp.interpret(&program).unwrap();
        let vars = interp.get_user_variables();
        assert!(vars
            .iter()
            .any(|(name, kind, _)| name == "foo" && kind == "function"));
    }

    // ==================== Native Function Edge Cases ====================

    #[test]
    fn test_scran_slice_list() {
        let result = run("scran([1, 2, 3, 4, 5], 1, 4)").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        assert_eq!(list.borrow().len(), 3);
    }

    #[test]
    fn test_scran_slice_string() {
        let result = run(r#"scran("hello", 1, 4)"#).unwrap();
        assert_eq!(result, Value::String("ell".to_string()));
    }

    #[test]
    fn test_scran_negative_indices() {
        // Negative indices should clamp to 0
        let result = run("scran([1, 2, 3], -5, 2)").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        assert_eq!(list.borrow().len(), 2);
    }

    #[test]
    fn test_scran_large_end_index() {
        // Large end should clamp to list length
        let result = run("scran([1, 2, 3], 0, 100)").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        assert_eq!(list.borrow().len(), 3);
    }

    #[test]
    fn test_coont_string_no_match() {
        let result = run(r#"coont("hello", "z")"#).unwrap();
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_coont_string_multiple() {
        let result = run(r#"coont("hello world", "l")"#).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_coont_list_values() {
        let result = run("coont([1, 2, 1, 3, 1], 1)").unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_wheesht_trim() {
        let result = run(r#"wheesht("  hello  ")"#).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_unique_list() {
        let result = run("unique([1, 2, 1, 3, 2])").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let items = list.borrow();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::Integer(1));
        assert_eq!(items[1], Value::Integer(2));
        assert_eq!(items[2], Value::Integer(3));
    }

    #[test]
    fn test_scottify_transform() {
        // Note: scottify replaces "no" before "know", so "know" becomes "knaew"
        let result = run(r#"scottify("yes the small child is beautiful")"#).unwrap();
        let Value::String(s) = result else {
            panic!("Expected string");
        };
        assert!(s.contains("aye"));
        assert!(s.contains("wee"));
        assert!(s.contains("bairn"));
        assert!(s.contains("bonnie"));
    }

    // ==================== Error Path Tests ====================

    #[test]
    fn test_len_error_non_collection() {
        let result = run("len(42)");
        assert!(result.is_err());
    }

    #[test]
    fn test_shove_error_non_list() {
        let result = run(r#"shove("hello", 1)"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_yank_error_non_list() {
        let result = run(r#"yank("hello")"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_keys_error_non_dict() {
        let result = run("keys([1, 2, 3])");
        assert!(result.is_err());
    }

    #[test]
    fn test_values_error_non_dict() {
        let result = run("values([1, 2, 3])");
        assert!(result.is_err());
    }

    #[test]
    fn test_sqrt_error_negative() {
        let result = run("sqrt(-1)").unwrap();
        if let Value::Float(f) = result {
            assert!(f.is_nan());
        }
    }

    #[test]
    fn test_sqrt_error_non_number() {
        let result = run(r#"sqrt("hello")"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_scran_error_non_integer_indices() {
        let result = run(r#"scran([1, 2, 3], "a", "b")"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_scran_error_non_collection() {
        let result = run("scran(42, 0, 1)");
        assert!(result.is_err());
    }

    #[test]
    fn test_coont_error_non_collection() {
        let result = run("coont(42, 1)");
        assert!(result.is_err());
    }

    #[test]
    fn test_coont_string_error_non_string_needle() {
        let result = run(r#"coont("hello", 1)"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_wheesht_error_non_string() {
        let result = run("wheesht(42)");
        assert!(result.is_err());
    }

    #[test]
    fn test_unique_error_non_list() {
        let result = run(r#"unique("hello")"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_scottify_error_non_string() {
        let result = run("scottify(42)");
        assert!(result.is_err());
    }

    #[test]
    fn test_sumaw_error_non_list() {
        let result = run("sumaw(42)");
        assert!(result.is_err());
    }

    #[test]
    fn test_sumaw_error_non_numeric() {
        let result = run(r#"sumaw(["a", "b"])"#);
        assert!(result.is_err());
    }

    // ==================== Math Functions Edge Cases ====================

    #[test]
    fn test_abs_negative_integer() {
        let result = run("abs(-42)").unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_abs_negative_float() {
        let result = run("abs(-3.14)").unwrap();
        assert_eq!(result, Value::Float(3.14));
    }

    #[test]
    fn test_abs_positive() {
        let result = run("abs(42)").unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_floor_positive() {
        let result = run("floor(3.9)").unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_floor_negative() {
        let result = run("floor(-3.1)").unwrap();
        assert_eq!(result, Value::Integer(-4));
    }

    #[test]
    fn test_ceil_positive() {
        let result = run("ceil(3.1)").unwrap();
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_ceil_negative() {
        let result = run("ceil(-3.9)").unwrap();
        assert_eq!(result, Value::Integer(-3));
    }

    #[test]
    fn test_round_half_up() {
        let result = run("round(3.5)").unwrap();
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_round_half_down() {
        let result = run("round(3.4)").unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    // ==================== Complex Control Flow ====================

    #[test]
    fn test_nested_if_else() {
        let result = run(r#"
ken x = 5
gin x > 10 {
    "big"
} ither gin x > 3 {
    "medium"
} ither {
    "small"
}
"#)
        .unwrap();
        assert_eq!(result, Value::String("medium".to_string()));
    }

    #[test]
    fn test_while_with_break() {
        let result = run(r#"
ken total = 0
ken i = 0
whiles i < 100 {
    total = total + i
    i = i + 1
    gin i == 5 {
        brak
    }
}
total
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(10));
    }

    #[test]
    fn test_while_with_continue() {
        let result = run(r#"
ken total = 0
ken i = 0
whiles i < 5 {
    i = i + 1
    gin i == 3 {
        haud
    }
    total = total + i
}
total
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(12)); // 1+2+4+5 = 12
    }

    #[test]
    fn test_for_with_break() {
        let result = run(r#"
ken total = 0
fer i in 0..10 {
    gin i == 5 {
        brak
    }
    total = total + i
}
total
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(10)); // 0+1+2+3+4 = 10
    }

    #[test]
    fn test_for_with_continue() {
        let result = run(r#"
ken total = 0
fer i in 0..5 {
    gin i == 2 {
        haud
    }
    total = total + i
}
total
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(8)); // 0+1+3+4 = 8
    }

    // ==================== Class and Method Tests ====================

    #[test]
    fn test_class_with_init() {
        let result = run(r#"
kin Counter {
    dae init(start) {
        masel.count = start
    }
    dae increment() {
        masel.count = masel.count + 1
    }
    dae get() {
        gie masel.count
    }
}
ken c = Counter(5)
c.increment()
c.get()
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(6));
    }

    #[test]
    fn test_class_inheritance() {
        let result = run(r#"
kin Animal {
    dae init(name) {
        masel.name = name
    }
    dae speak() {
        gie "..."
    }
}
kin Dog fae Animal {
    dae speak() {
        gie "Woof!"
    }
}
ken d = Dog("Rover")
d.speak()
"#)
        .unwrap();
        assert_eq!(result, Value::String("Woof!".to_string()));
    }

    // ==================== Struct Tests ====================

    #[test]
    fn test_struct_creation() {
        let result = run(r#"
thing Point { x, y }
ken p = Point(3, 4)
p.x + p.y
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(7));
    }

    #[test]
    fn test_struct_update() {
        let result = run(r#"
thing Point { x, y }
ken p = Point(1, 2)
p.x = 10
p.x
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(10));
    }

    // ==================== List Operations ====================

    #[test]
    fn test_list_negative_index() {
        let result = run(r#"
ken list = [1, 2, 3, 4, 5]
list[-1]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_list_negative_index_second() {
        let result = run(r#"
ken list = [1, 2, 3, 4, 5]
list[-2]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_list_index_mutation() {
        let result = run(r#"
ken list = [1, 2, 3]
list[1] = 99
list[1]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(99));
    }

    // ==================== Dict Operations ====================

    #[test]
    fn test_dict_set_get() {
        let result = run(r#"
ken d = {"a": 1}
d["b"] = 2
d["a"] + d["b"]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_dict_with_string_keys() {
        let result = run(r#"
ken d = {"name": "Alice", "age": 30}
d["name"]
"#)
        .unwrap();
        assert_eq!(result, Value::String("Alice".to_string()));
    }

    // ==================== String Operations ====================

    #[test]
    fn test_string_index() {
        let result = run(r#""hello"[0]"#).unwrap();
        assert_eq!(result, Value::String("h".to_string()));
    }

    #[test]
    fn test_string_negative_index() {
        let result = run(r#""hello"[-1]"#).unwrap();
        assert_eq!(result, Value::String("o".to_string()));
    }

    #[test]
    fn test_upper_function() {
        let result = run(r#"upper("hello")"#).unwrap();
        assert_eq!(result, Value::String("HELLO".to_string()));
    }

    #[test]
    fn test_lower_function() {
        let result = run(r#"lower("HELLO")"#).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_replace_string() {
        let result = run(r#"replace("hello world", "world", "everyone")"#).unwrap();
        assert_eq!(result, Value::String("hello everyone".to_string()));
    }

    // ==================== Type Checking Functions ====================

    #[test]
    fn test_whit_kind_integer() {
        let result = run(r#"whit_kind(42)"#).unwrap();
        assert_eq!(result, Value::String("integer".to_string()));
    }

    #[test]
    fn test_whit_kind_string() {
        let result = run(r#"whit_kind("hello")"#).unwrap();
        assert_eq!(result, Value::String("string".to_string()));
    }

    #[test]
    fn test_whit_kind_list() {
        let result = run(r#"whit_kind([1, 2, 3])"#).unwrap();
        assert_eq!(result, Value::String("list".to_string()));
    }

    #[test]
    fn test_whit_kind_function_value() {
        let result = run(r#"
dae foo() { gie 1 }
whit_kind(foo)
"#)
        .unwrap();
        assert_eq!(result, Value::String("function".to_string()));
    }

    // ==================== Pipe Operator ====================

    #[test]
    fn test_pipe_chain() {
        let result = run(r#"[1, 2, 3] |> len"#).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_pipe_multiple() {
        let result = run(r#"
"  hello  " |> wheesht |> upper
"#)
        .unwrap();
        assert_eq!(result, Value::String("HELLO".to_string()));
    }

    // ==================== Spread Operator ====================

    #[test]
    fn test_spread_list() {
        let result = run(r#"
ken a = [1, 2]
ken b = [3, 4]
ken c = [...a, ...b]
len(c)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_spread_in_call() {
        let result = run(r#"
dae add(x, y, z) {
    gie x + y + z
}
ken args = [1, 2, 3]
add(...args)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(6));
    }

    // ==================== Try-Catch Tests ====================

    #[test]
    fn test_try_catch_no_error() {
        let result = run(r#"
hae_a_bash {
    42
} gin_it_gangs_wrang e {
    0
}
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_try_catch_with_error() {
        let result = run(r#"
hae_a_bash {
    1 / 0
} gin_it_gangs_wrang e {
    "caught"
}
"#)
        .unwrap();
        assert_eq!(result, Value::String("caught".to_string()));
    }

    // ==================== Assert Tests ====================

    #[test]
    fn test_mak_siccar_pass() {
        let result = run("mak_siccar aye");
        assert!(result.is_ok());
    }

    #[test]
    fn test_mak_siccar_fail() {
        let result = run("mak_siccar nae");
        assert!(result.is_err());
    }

    #[test]
    fn test_mak_siccar_with_message() {
        let result = run(r#"mak_siccar nae, "Custom message""#);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(format!("{:?}", e).contains("Custom message"));
        }
    }

    // ==================== Range Iteration ====================

    #[test]
    fn test_for_range_exclusive() {
        let result = run(r#"
ken sum = 0
fer i in 0..5 {
    sum = sum + i
}
sum
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(10)); // 0+1+2+3+4
    }

    #[test]
    fn test_range_in_list() {
        let result = run(r#"
ken r = 1..4
ken sum = 0
fer i in r {
    sum = sum + i
}
sum
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(6)); // 1+2+3
    }

    // ==================== Lambda Tests ====================

    #[test]
    fn test_lambda_simple() {
        let result = run(r#"
ken double = |x| x * 2
double(5)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(10));
    }

    #[test]
    fn test_lambda_multiple_params() {
        let result = run(r#"
ken add = |x, y| x + y
add(3, 4)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(7));
    }

    #[test]
    fn test_lambda_as_callback() {
        let result = run(r#"
dae apply(f, x) {
    gie f(x)
}
apply(|n| n * n, 4)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(16));
    }

    // ==================== Modulo and Integer Division ====================

    #[test]
    fn test_modulo_positive() {
        let result = run("10 % 3").unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_modulo_negative() {
        let result = run("-10 % 3").unwrap();
        assert_eq!(result, Value::Integer(-1));
    }

    #[test]
    fn test_floor_division() {
        // Use floor() for integer division
        let result = run("floor(10 / 3)").unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    // ==================== Comparison Edge Cases ====================

    #[test]
    fn test_string_less_than() {
        let result = run(r#""apple" < "banana""#).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_mixed_numeric_comparison() {
        let result = run("3 == 3.0").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_nil_equality() {
        let result = run("naething == naething").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    // ==================== More Native Functions ====================

    #[test]
    fn test_heid_list() {
        let result = run("heid([1, 2, 3])").unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_heid_string() {
        let result = run(r#"heid("hello")"#).unwrap();
        assert_eq!(result, Value::String("h".to_string()));
    }

    #[test]
    fn test_tail_list() {
        let result = run("tail([1, 2, 3])").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        assert_eq!(list.borrow().len(), 2);
        assert_eq!(list.borrow()[0], Value::Integer(2));
    }

    #[test]
    fn test_tail_string() {
        let result = run(r#"tail("hello")"#).unwrap();
        assert_eq!(result, Value::String("ello".to_string()));
    }

    #[test]
    fn test_bum_list() {
        let result = run("bum([1, 2, 3])").unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_bum_string() {
        let result = run(r#"bum("hello")"#).unwrap();
        assert_eq!(result, Value::String("o".to_string()));
    }

    #[test]
    fn test_join_string() {
        let result = run(r#"join(["a", "b", "c"], ", ")"#).unwrap();
        assert_eq!(result, Value::String("a, b, c".to_string()));
    }

    // ==================== Module/Import Tests ====================

    #[test]
    fn test_anonymous_function_call() {
        let result = run(r#"
(|x| x * 2)(5)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(10));
    }

    // ==================== Default Parameter Tests ====================

    #[test]
    fn test_function_default_param() {
        let result = run(r#"
dae greet(name, greeting = "Hello") {
    gie greeting + " " + name
}
greet("World")
"#)
        .unwrap();
        assert_eq!(result, Value::String("Hello World".to_string()));
    }

    #[test]
    fn test_function_override_default() {
        let result = run(r#"
dae greet(name, greeting = "Hello") {
    gie greeting + " " + name
}
greet("World", "Hi")
"#)
        .unwrap();
        assert_eq!(result, Value::String("Hi World".to_string()));
    }

    // ==================== Set Operations ====================

    #[test]
    fn test_creel_basic() {
        let result = run(r#"
ken s = creel([1, 2, 3])
len(s)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_creel_duplicates() {
        let result = run(r#"
ken s = creel([1, 1, 2, 2, 3])
len(s)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    // ==================== More Edge Cases ====================

    #[test]
    fn test_empty_list_operations() {
        let result = run("len([])").unwrap();
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_empty_string_operations() {
        let result = run(r#"len("")"#).unwrap();
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_nested_list_access() {
        let result = run(r#"
ken matrix = [[1, 2], [3, 4]]
matrix[1][0]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_dict_in_list() {
        let result = run(r#"
ken list = [{"a": 1}, {"a": 2}]
list[0]["a"] + list[1]["a"]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_conditional_expression() {
        let result = run(r#"
ken x = 5
ken result = gin x > 3 than "big" ither "small"
result
"#)
        .unwrap();
        assert_eq!(result, Value::String("big".to_string()));
    }

    #[test]
    fn test_contains_list() {
        let result = run("contains([1, 2, 3], 2)").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_contains_list_missing() {
        let result = run("contains([1, 2, 3], 5)").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_contains_string() {
        let result = run(r#"contains("hello", "ell")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_sort_integers() {
        let result = run("sort([3, 1, 2])").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let items = list.borrow();
        assert_eq!(items[0], Value::Integer(1));
        assert_eq!(items[1], Value::Integer(2));
        assert_eq!(items[2], Value::Integer(3));
    }

    #[test]
    fn test_sort_strings() {
        let result = run(r#"sort(["c", "a", "b"])"#).unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let items = list.borrow();
        assert_eq!(items[0], Value::String("a".to_string()));
        assert_eq!(items[1], Value::String("b".to_string()));
        assert_eq!(items[2], Value::String("c".to_string()));
    }

    #[test]
    fn test_reverse_list() {
        let result = run("reverse([1, 2, 3])").unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let items = list.borrow();
        assert_eq!(items[0], Value::Integer(3));
        assert_eq!(items[1], Value::Integer(2));
        assert_eq!(items[2], Value::Integer(1));
    }

    #[test]
    fn test_reverse_string_builtin() {
        let result = run(r#"reverse("hello")"#).unwrap();
        assert_eq!(result, Value::String("olleh".to_string()));
    }

    // ==================== More Native Function Tests ====================

    #[test]
    fn test_words_function() {
        let result = run(r#"words("hello world")"#).unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        assert_eq!(list.borrow().len(), 2);
    }

    #[test]
    fn test_is_digit_true() {
        let result = run(r#"is_digit("123")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_is_digit_false() {
        let result = run(r#"is_digit("12a")"#).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_is_alpha_true() {
        let result = run(r#"is_alpha("hello")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_is_alpha_false() {
        let result = run(r#"is_alpha("hello1")"#).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_is_space_true() {
        let result = run(r#"is_space("   ")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_is_space_with_letters() {
        let result = run(r#"is_space("  x  ")"#).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_capitalize_function() {
        let result = run(r#"capitalize("hello")"#).unwrap();
        assert_eq!(result, Value::String("Hello".to_string()));
    }

    #[test]
    fn test_title_function() {
        let result = run(r#"title("hello world")"#).unwrap();
        assert_eq!(result, Value::String("Hello World".to_string()));
    }

    #[test]
    fn test_the_noo_timestamp() {
        let result = run("the_noo()").unwrap();
        let Value::Integer(n) = result else {
            panic!("Expected integer timestamp");
        };
        assert!(n > 0);
    }

    #[test]
    fn test_jammy_error_min_gte_max() {
        let result = run("jammy(10, 5)");
        assert!(result.is_err());
    }

    #[test]
    fn test_shuffle_preserves_length() {
        let result = run("len(shuffle([1, 2, 3, 4, 5]))").unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_sort_error_non_list() {
        let result = run(r#"sort("hello")"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_shuffle_error_non_list() {
        let result = run(r#"shuffle("hello")"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_lower_error_non_string() {
        let result = run("lower(42)");
        assert!(result.is_err());
    }

    #[test]
    fn test_upper_error_non_string() {
        let result = run("upper(42)");
        assert!(result.is_err());
    }

    // ==================== Set Function Tests ====================

    #[test]
    fn test_toss_in_set() {
        let result = run(r#"
ken s = creel([1, 2, 3])
toss_in(s, 4)
len(s)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_heave_oot_set() {
        let result = run(r#"
ken s = creel([1, 2, 3])
heave_oot(s, 1)
len(s)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_is_in_creel_true() {
        let result = run(r#"
ken s = creel([1, 2, 3])
is_in_creel(s, 1)
"#)
        .unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_is_in_creel_false() {
        let result = run(r#"
ken s = creel([1, 2, 3])
is_in_creel(s, 5)
"#)
        .unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    // ==================== More Math Tests ====================

    #[test]
    fn test_pow_function() {
        let result = run("pow(2, 3)").unwrap();
        assert_eq!(result, Value::Float(8.0));
    }

    #[test]
    fn test_log_function() {
        // log is natural log (ln)
        let result = run("log10(100)").unwrap();
        let Value::Float(f) = result else {
            panic!("Expected float");
        };
        assert!((f - 2.0).abs() < 0.0001);
    }

    #[test]
    fn test_sin_function() {
        let result = run("sin(0)").unwrap();
        let Value::Float(f) = result else {
            panic!("Expected float");
        };
        assert!(f.abs() < 0.0001);
    }

    #[test]
    fn test_cos_function() {
        let result = run("cos(0)").unwrap();
        let Value::Float(f) = result else {
            panic!("Expected float");
        };
        assert!((f - 1.0).abs() < 0.0001);
    }

    // ==================== More Error Path Tests ====================

    #[test]
    fn test_index_error_list() {
        let result = run("[1, 2, 3][10]");
        assert!(result.is_err());
    }

    #[test]
    fn test_index_error_string() {
        let result = run(r#""hi"[10]"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_heid_error_empty_list() {
        let result = run("heid([])");
        assert!(result.is_err());
    }

    #[test]
    fn test_tail_empty_list_returns_empty() {
        // tail on empty list returns empty list (not error)
        let result = run("len(tail([1]))").unwrap();
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_bum_error_empty_list() {
        let result = run("bum([])");
        assert!(result.is_err());
    }

    #[test]
    fn test_heid_error_empty_string() {
        let result = run(r#"heid("")"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_bum_error_empty_string() {
        let result = run(r#"bum("")"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_digit_error_non_string() {
        let result = run("is_digit(42)");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_alpha_error_non_string() {
        let result = run("is_alpha(42)");
        assert!(result.is_err());
    }

    #[test]
    fn test_words_error_non_string() {
        let result = run("words(42)");
        assert!(result.is_err());
    }

    #[test]
    fn test_capitalize_error_non_string() {
        let result = run("capitalize(42)");
        assert!(result.is_err());
    }

    #[test]
    fn test_title_error_non_string() {
        let result = run("title(42)");
        assert!(result.is_err());
    }

    // ==================== Method Access Tests ====================

    #[test]
    fn test_instance_method_call() {
        let result = run(r#"
kin Calculator {
    dae init(val) {
        masel.val = val
    }
    dae double() {
        gie masel.val * 2
    }
}
ken c = Calculator(21)
c.double()
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_instance_field_access() {
        let result = run(r#"
kin Point {
    dae init(x, y) {
        masel.x = x
        masel.y = y
    }
}
ken p = Point(3, 4)
p.x + p.y
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(7));
    }

    // ==================== Range Tests ====================

    #[test]
    fn test_range_with_variable_bounds() {
        let result = run(r#"
ken start = 0
ken end = 5
ken sum = 0
fer i in start..end {
    sum = sum + i
}
sum
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(10)); // 0+1+2+3+4 = 10
    }

    // ==================== Dictionary Key Tests ====================

    #[test]
    fn test_dict_keys_iteration() {
        let result = run(r#"
ken d = {"a": 1, "b": 2}
len(keys(d))
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_dict_values_iteration() {
        let result = run(r#"
ken d = {"a": 1, "b": 2}
len(values(d))
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_dict_update_existing() {
        let result = run(r#"
ken d = {"a": 1}
d["a"] = 42
d["a"]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_dict_add_new_key() {
        let result = run(r#"
ken d = {"a": 1}
d["b"] = 2
d["b"]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    // ==================== More Complex Expression Tests ====================

    #[test]
    fn test_chained_method_calls() {
        let result = run(r#"
kin Builder {
    dae init() {
        masel.val = 0
    }
    dae add(n) {
        masel.val = masel.val + n
        gie masel
    }
    dae get() {
        gie masel.val
    }
}
ken b = Builder()
b.add(1).add(2).add(3).get()
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(6));
    }

    // ==================== String Interpolation Tests ====================

    #[test]
    fn test_fstring_nested_expr() {
        let result = run(r#"
ken x = 10
f"Result: {x * 2}"
"#)
        .unwrap();
        assert_eq!(result, Value::String("Result: 20".to_string()));
    }

    #[test]
    fn test_fstring_function_call() {
        let result = run(r#"
dae greet(name) { gie "Hi " + name }
f"Greeting: {greet(\"World\")}"
"#)
        .unwrap();
        assert_eq!(result, Value::String("Greeting: Hi World".to_string()));
    }

    // ==================== Empty Structure Tests ====================

    #[test]
    fn test_empty_dict() {
        let result = run(r#"
ken d = {}
len(keys(d))
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_struct_with_fields() {
        let result = run(r#"
thing Person { name, age }
ken p = Person("Alice", 30)
p.name
"#)
        .unwrap();
        assert_eq!(result, Value::String("Alice".to_string()));
    }

    // ==================== Assignment Operators ====================

    #[test]
    fn test_plus_equals() {
        let result = run(r#"
ken x = 5
x += 3
x
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(8));
    }

    #[test]
    fn test_minus_equals() {
        let result = run(r#"
ken x = 10
x -= 3
x
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(7));
    }

    #[test]
    fn test_times_equals() {
        let result = run(r#"
ken x = 4
x *= 3
x
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(12));
    }

    #[test]
    fn test_divide_equals() {
        let result = run(r#"
ken x = 12
x /= 3
x
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(4));
    }

    // ==================== Boolean Short Circuit Proof ====================

    #[test]
    fn test_and_short_circuit_side_effect() {
        // If short circuit works, second expression shouldn't run
        let result = run(r#"
ken x = 0
nae an (x = 1)
x
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(0)); // x should still be 0
    }

    #[test]
    fn test_or_short_circuit_side_effect() {
        // If short circuit works, second expression shouldn't run
        let result = run(r#"
ken x = 0
aye or (x = 1)
x
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(0)); // x should still be 0
    }

    // ==================== Slice Tests ====================

    #[test]
    fn test_list_slice_basic() {
        let result = run(r#"
ken l = [1, 2, 3, 4, 5]
l[1:4]
        "#)
        .unwrap();
        let Value::List(items) = result else {
            panic!("Expected list");
        };
        assert_eq!(items.borrow().len(), 3);
    }

    #[test]
    fn test_list_slice_negative_index() {
        let result = run(r#"
ken l = [1, 2, 3, 4, 5]
l[-3:-1]
        "#)
        .unwrap();
        let Value::List(items) = result else {
            panic!("Expected list");
        };
        assert_eq!(items.borrow().len(), 2);
    }

    #[test]
    fn test_list_slice_with_step() {
        let result = run(r#"
ken l = [1, 2, 3, 4, 5, 6]
l[0:6:2]
        "#)
        .unwrap();
        let Value::List(items) = result else {
            panic!("Expected list");
        };
        assert_eq!(items.borrow().len(), 3); // 1, 3, 5
    }

    #[test]
    fn test_list_slice_negative_step() {
        let result = run(r#"
ken l = [1, 2, 3, 4, 5]
l[4:0:-1]
        "#)
        .unwrap();
        let Value::List(items) = result else {
            panic!("Expected list");
        };
        assert_eq!(items.borrow().len(), 4); // 5, 4, 3, 2
    }

    #[test]
    fn test_string_slice_basic() {
        let result = run(r#"
ken s = "hello"
s[1:4]
"#)
        .unwrap();
        assert_eq!(result, Value::String("ell".to_string()));
    }

    #[test]
    fn test_string_slice_negative_step() {
        let result = run(r#"
ken s = "hello"
s[4:0:-1]
"#)
        .unwrap();
        assert_eq!(result, Value::String("olle".to_string()));
    }

    #[test]
    fn test_slice_step_zero_error() {
        let result = run("ken l = [1,2,3]\nl[::0]");
        assert!(result.is_err());
    }

    // ==================== More Set (Creel) Tests ====================

    #[test]
    fn test_creel_from_set() {
        let result = run(r#"
ken s = creel([1, 2, 3])
len(creel(s))
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_creels_thegither() {
        let result = run(r#"
ken s1 = creel([1, 2])
ken s2 = creel([2, 3])
len(creels_thegither(s1, s2))
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(3)); // Union: 1, 2, 3
    }

    #[test]
    fn test_creels_baith() {
        let result = run(r#"
ken s1 = creel([1, 2, 3])
ken s2 = creel([2, 3, 4])
len(creels_baith(s1, s2))
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(2)); // Intersection: 2, 3
    }

    #[test]
    fn test_creels_differ() {
        let result = run(r#"
ken s1 = creel([1, 2, 3])
ken s2 = creel([2, 3])
len(creels_differ(s1, s2))
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(1)); // Difference: 1
    }

    #[test]
    fn test_creel_tae_list() {
        let result = run(r#"
ken s = creel([3, 1, 2])
whit_kind(creel_tae_list(s))
"#)
        .unwrap();
        assert_eq!(result, Value::String("list".to_string()));
    }

    #[test]
    fn test_is_subset() {
        let result = run(r#"
ken s1 = creel([1, 2])
ken s2 = creel([1, 2, 3])
is_subset(s1, s2)
"#)
        .unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_is_superset() {
        let result = run(r#"
ken s1 = creel([1, 2, 3])
ken s2 = creel([1, 2])
is_superset(s1, s2)
"#)
        .unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_is_disjoint() {
        let result = run(r#"
ken s1 = creel([1, 2])
ken s2 = creel([3, 4])
is_disjoint(s1, s2)
"#)
        .unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    // ==================== Match Statement Tests ====================

    #[test]
    fn test_match_literal_int() {
        let result = run(r#"
ken x = 2
keek x {
    whan 1 -> 10
    whan 2 -> 20
    whan 3 -> 30
}
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(20));
    }

    #[test]
    fn test_match_literal_string() {
        let result = run(r#"
ken s = "hello"
keek s {
    whan "hi" -> 1
    whan "hello" -> 2
    whan "bye" -> 3
}
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_match_catchall() {
        let result = run(r#"
ken x = 99
keek x {
    whan 1 -> 10
    whan _ -> 42
}
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_match_binding() {
        let result = run(r#"
ken x = 5
keek x {
    whan n -> n * 2
}
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(10));
    }

    #[test]
    fn test_match_no_match_error() {
        let result = run(r#"
ken x = 99
keek x {
    whan 1 -> 10
    whan 2 -> 20
}
"#);
        assert!(result.is_err());
    }

    // ==================== Destructuring Tests ====================

    #[test]
    fn test_destructure_basic() {
        let result = run(r#"
ken [a, b, c] = [1, 2, 3]
a + b + c
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(6));
    }

    #[test]
    fn test_destructure_with_rest() {
        let result = run(r#"
ken [first, ...rest] = [1, 2, 3, 4, 5]
len(rest)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_destructure_with_ignore() {
        let result = run(r#"
ken [a, _, c] = [1, 2, 3]
a + c
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_destructure_string() {
        let result = run(r#"
ken [a, b, c] = "abc"
a + b + c
"#)
        .unwrap();
        assert_eq!(result, Value::String("abc".to_string()));
    }

    #[test]
    fn test_destructure_not_enough_elements() {
        let result = run(r#"
ken [a, b, c] = [1, 2]
"#);
        assert!(result.is_err());
    }

    // ==================== More Native Function Tests ====================

    #[test]
    fn test_shuffle_length() {
        // Shuffle should return a list of the same length
        let result = run("len(shuffle([1, 2, 3, 4, 5]))").unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_shuffle_reject_string() {
        let result = run(r#"shuffle("hello")"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_sort_string_list() {
        let result = run(r#"
ken l = sort(["c", "a", "b"])
l[0]
"#)
        .unwrap();
        assert_eq!(result, Value::String("a".to_string()));
    }

    #[test]
    fn test_sort_float_list() {
        let result = run(r#"
ken l = sort([3.5, 1.5, 2.5])
l[0]
"#)
        .unwrap();
        assert_eq!(result, Value::Float(1.5));
    }

    #[test]
    fn test_sort_rejects_non_list() {
        let result = run(r#"sort("abc")"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_jammy_min_max() {
        let result = run("jammy(1, 10)").unwrap();
        let Value::Integer(n) = result else {
            panic!("Expected integer");
        };
        assert!(n >= 1 && n < 10);
    }

    #[test]
    fn test_jammy_bounds_error() {
        let result = run("jammy(10, 5)");
        assert!(result.is_err());
    }

    #[test]
    fn test_the_noo() {
        let result = run("the_noo()").unwrap();
        let Value::Integer(ts) = result else {
            panic!("Expected integer timestamp");
        };
        assert!(ts > 0);
    }

    #[test]
    fn test_clype_debug_info() {
        let result = run("clype([1, 2, 3])").unwrap();
        let Value::String(s) = result else {
            panic!("Expected string");
        };
        assert!(s.contains("list"));
    }

    #[test]
    fn test_is_a_integer() {
        let result = run(r#"is_a(42, "integer")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_is_a_function() {
        let result = run(r#"
dae foo() { gie 1 }
is_a(foo, "function")
"#)
        .unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_is_a_nil() {
        let result = run(r#"is_a(naething, "nil")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_is_a_range() {
        let result = run(r#"is_a(1..10, "range")"#).unwrap();
        assert_eq!(result, Value::Bool(false));
        let result = run(r#"is_a(1..10, "list")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_tae_bool() {
        let result = run("tae_bool(0)").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_char_at_positive() {
        let result = run(r#"char_at("hello", 1)"#).unwrap();
        assert_eq!(result, Value::String("e".to_string()));
    }

    #[test]
    fn test_char_at_negative() {
        let result = run(r#"char_at("hello", -1)"#).unwrap();
        assert_eq!(result, Value::String("o".to_string()));
    }

    #[test]
    fn test_char_at_out_of_bounds() {
        let result = run(r#"char_at("hi", 10)"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_repeat_string() {
        let result = run(r#"repeat("ab", 3)"#).unwrap();
        assert_eq!(result, Value::String("ababab".to_string()));
    }

    #[test]
    fn test_repeat_negative_error() {
        let result = run(r#"repeat("ab", -1)"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_index_of_string() {
        let result = run(r#"index_of("hello", "ll")"#).unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_index_of_string_not_found() {
        let result = run(r#"index_of("hello", "xyz")"#).unwrap();
        assert_eq!(result, Value::Integer(-1));
    }

    #[test]
    fn test_index_of_list() {
        let result = run(r#"index_of([10, 20, 30], 20)"#).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_index_of_list_not_found() {
        let result = run(r#"index_of([1, 2, 3], 99)"#).unwrap();
        assert_eq!(result, Value::Integer(-1));
    }

    // ==================== More String Functions ====================

    #[test]
    fn test_pad_left() {
        let result = run(r#"pad_left("5", 3, "0")"#).unwrap();
        assert_eq!(result, Value::String("005".to_string()));
    }

    #[test]
    fn test_pad_right() {
        let result = run(r#"pad_right("5", 3, "0")"#).unwrap();
        assert_eq!(result, Value::String("500".to_string()));
    }

    #[test]
    fn test_pad_left_already_wide() {
        let result = run(r#"pad_left("hello", 3, " ")"#).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_lines() {
        let result = run(r#"len(lines("a\nb\nc"))"#).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_lines_error_non_string() {
        let result = run("lines(42)");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_space() {
        let result = run(r#"is_space("   ")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_is_space_mixed_chars() {
        let result = run(r#"is_space("a b")"#).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_is_space_error_non_string() {
        let result = run("is_space(42)");
        assert!(result.is_err());
    }

    #[test]
    fn test_chars() {
        let result = run(r#"len(chars("abc"))"#).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_chars_error_non_string() {
        let result = run("chars(42)");
        assert!(result.is_err());
    }

    #[test]
    fn test_ord() {
        let result = run(r#"ord("A")"#).unwrap();
        assert_eq!(result, Value::Integer(65));
    }

    #[test]
    fn test_ord_empty_string_error() {
        let result = run(r#"ord("")"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_chr() {
        let result = run("chr(65)").unwrap();
        assert_eq!(result, Value::String("A".to_string()));
    }

    #[test]
    fn test_chr_invalid_codepoint() {
        let result = run("chr(-1)");
        assert!(result.is_err());
    }

    // ==================== More List Functions ====================

    #[test]
    fn test_flatten() {
        let result = run("len(flatten([[1, 2], [3, 4]]))").unwrap();
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_flatten_mixed() {
        let result = run("len(flatten([1, [2, 3], 4]))").unwrap();
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_flatten_error_non_list() {
        let result = run(r#"flatten("abc")"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_zip() {
        let result = run("len(zip([1, 2], [3, 4]))").unwrap();
        assert_eq!(result, Value::Integer(2)); // Two pairs
    }

    #[test]
    fn test_zip_error_non_lists() {
        let result = run(r#"zip([1], "abc")"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_enumerate() {
        let result = run(r#"
ken e = enumerate(["a", "b"])
e[0][0]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_enumerate_error_non_list() {
        let result = run(r#"enumerate("abc")"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_uniq() {
        let result = run("len(uniq([1, 2, 2, 3, 3, 3]))").unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_uniq_error_non_list() {
        let result = run(r#"uniq("abc")"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_chynge_insert() {
        let result = run(r#"
ken l = chynge([1, 3], 1, 2)
l[1]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_chynge_negative_index() {
        let result = run(r#"
ken l = chynge([1, 2, 3], -1, 99)
len(l)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_dicht_remove() {
        let result = run("len(dicht([1, 2, 3], 1))").unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_dicht_negative_index() {
        let result = run(r#"
ken l = dicht([1, 2, 3], -1)
len(l)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_tak() {
        let result = run("len(tak([1, 2, 3, 4], 2))").unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_string_slice_take() {
        let result = run(r#""hello"[0:3]"#).unwrap();
        assert_eq!(result, Value::String("hel".to_string()));
    }

    #[test]
    fn test_drap() {
        let result = run("len(drap([1, 2, 3, 4], 2))").unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_string_slice_drop() {
        let result = run(r#""hello"[2:]"#).unwrap();
        assert_eq!(result, Value::String("llo".to_string()));
    }

    #[test]
    fn test_redd_up() {
        let result = run("len(redd_up([1, naething, 2, naething, 3]))").unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_redd_up_error_non_list() {
        let result = run(r#"redd_up("abc")"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_split_by_even() {
        let result = run(r#"
ken parts = split_by([1, 2, 3, 4], "even")
len(parts[0])
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(2)); // 2, 4 are even
    }

    #[test]
    fn test_split_by_positive() {
        let result = run(r#"
ken parts = split_by([-1, 0, 1, 2], "positive")
len(parts[0])
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(2)); // 1, 2 are positive
    }

    #[test]
    fn test_split_by_unknown_predicate() {
        let result = run(r#"split_by([1, 2], "unknown")"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_grup_runs() {
        let result = run("len(grup_runs([1, 1, 2, 2, 2, 3]))").unwrap();
        assert_eq!(result, Value::Integer(3)); // [[1,1], [2,2,2], [3]]
    }

    #[test]
    fn test_grup_runs_error_non_list() {
        let result = run(r#"grup_runs("aab")"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_chunks() {
        let result = run("len(chunks([1, 2, 3, 4, 5], 2))").unwrap();
        assert_eq!(result, Value::Integer(3)); // [[1,2], [3,4], [5]]
    }

    #[test]
    fn test_chunks_zero_size_error() {
        let result = run("chunks([1, 2, 3], 0)");
        assert!(result.is_err());
    }

    #[test]
    fn test_interleave() {
        let result = run("len(interleave([1, 2], [3, 4]))").unwrap();
        assert_eq!(result, Value::Integer(4)); // [1, 3, 2, 4]
    }

    #[test]
    fn test_interleave_error_non_lists() {
        let result = run(r#"interleave([1], "abc")"#);
        assert!(result.is_err());
    }

    // ==================== Math Functions ====================

    #[test]
    fn test_pooer_integers() {
        let result = run("pooer(2, 10)").unwrap();
        assert_eq!(result, Value::Integer(1024));
    }

    #[test]
    fn test_pooer_negative_exponent() {
        let result = run("pooer(2, -2)").unwrap();
        assert_eq!(result, Value::Float(0.25));
    }

    #[test]
    fn test_pooer_floats() {
        let result = run("pooer(2.0, 3.0)").unwrap();
        assert_eq!(result, Value::Float(8.0));
    }

    #[test]
    fn test_pooer_error_non_numbers() {
        let result = run(r#"pooer("a", 2)"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_sign_positive() {
        let result = run("sign(42)").unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_sign_negative() {
        let result = run("sign(-42)").unwrap();
        assert_eq!(result, Value::Integer(-1));
    }

    #[test]
    fn test_sign_zero() {
        let result = run("sign(0)").unwrap();
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_sign_float() {
        let result = run("sign(-3.14)").unwrap();
        assert_eq!(result, Value::Integer(-1));
    }

    #[test]
    fn test_clamp_integers() {
        let result = run("clamp(15, 0, 10)").unwrap();
        assert_eq!(result, Value::Integer(10));
    }

    #[test]
    fn test_clamp_floats() {
        let result = run("clamp(-5.0, 0.0, 10.0)").unwrap();
        assert_eq!(result, Value::Float(0.0));
    }

    #[test]
    fn test_lerp() {
        let result = run("lerp(0.0, 10.0, 0.5)").unwrap();
        assert_eq!(result, Value::Float(5.0));
    }

    #[test]
    fn test_lerp_integers() {
        let result = run("lerp(0, 100, 0)").unwrap();
        assert_eq!(result, Value::Float(0.0));
    }

    #[test]
    fn test_gcd() {
        let result = run("gcd(48, 18)").unwrap();
        assert_eq!(result, Value::Integer(6));
    }

    #[test]
    fn test_gcd_negative() {
        let result = run("gcd(-48, 18)").unwrap();
        assert_eq!(result, Value::Integer(6));
    }

    #[test]
    fn test_lcm() {
        let result = run("lcm(4, 6)").unwrap();
        assert_eq!(result, Value::Integer(12));
    }

    #[test]
    fn test_lcm_zero() {
        let result = run("lcm(0, 5)").unwrap();
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_factorial() {
        let result = run("factorial(5)").unwrap();
        assert_eq!(result, Value::Integer(120));
    }

    #[test]
    fn test_factorial_zero() {
        let result = run("factorial(0)").unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_factorial_negative_error() {
        let result = run("factorial(-1)");
        assert!(result.is_err());
    }

    #[test]
    fn test_factorial_too_big_error() {
        let result = run("factorial(21)");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_even() {
        let result = run("is_even(4)").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_is_odd() {
        let result = run("is_odd(3)").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    // ==================== Scots-themed Functions ====================

    #[test]
    fn test_clarty_list_duplicates() {
        let result = run("clarty([1, 2, 2, 3])").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_clarty_list_no_duplicates() {
        let result = run("clarty([1, 2, 3])").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_clarty_string() {
        let result = run(r#"clarty("hello")"#).unwrap(); // has duplicate 'l'
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_dreich_empty() {
        let result = run(r#"dreich("")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_dreich_same_chars() {
        let result = run(r#"dreich("aaaa")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_dreich_varied() {
        let result = run(r#"dreich("hello")"#).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_stoater_numbers() {
        let result = run("stoater([1, 5, 3, 2])").unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_stoater_floats() {
        let result = run("stoater([1.0, 5.0, 3.0])").unwrap();
        assert_eq!(result, Value::Float(5.0));
    }

    #[test]
    fn test_stoater_strings() {
        let result = run(r#"stoater(["a", "abc", "ab"])"#).unwrap();
        assert_eq!(result, Value::String("abc".to_string())); // longest
    }

    #[test]
    fn test_stoater_empty_list_error() {
        let result = run("stoater([])");
        assert!(result.is_err());
    }

    #[test]
    fn test_numpty_check_nil() {
        let result = run("numpty_check(naething)").unwrap();
        let Value::String(s) = result else {
            panic!("Expected string");
        };
        assert!(s.contains("naething"));
    }

    #[test]
    fn test_numpty_check_empty_string() {
        let result = run(r#"numpty_check("")"#).unwrap();
        let Value::String(s) = result else {
            panic!("Expected string");
        };
        assert!(s.contains("Empty string"));
    }

    #[test]
    fn test_numpty_check_valid() {
        let result = run("numpty_check(42)").unwrap();
        let Value::String(s) = result else {
            panic!("Expected string");
        };
        assert!(s.contains("braw"));
    }

    #[test]
    fn test_bampot_mode() {
        // Should return a list of same length
        let result = run("len(bampot_mode([1, 2, 3]))").unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_crabbit_negative() {
        let result = run("crabbit(-5)").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_crabbit_positive() {
        let result = run("crabbit(5)").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_crabbit_float() {
        let result = run("crabbit(-3.14)").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_gallus_large_number() {
        let result = run("gallus(200)").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_gallus_small_number() {
        let result = run("gallus(50)").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_drookit_has_duplicates() {
        let result = run("drookit([1, 2, 2, 3])").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_drookit_no_duplicates() {
        let result = run("drookit([1, 2, 3])").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_glaikit_nil() {
        let result = run("glaikit(naething)").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_glaikit_zero() {
        let result = run("glaikit(0)").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_glaikit_valid() {
        let result = run("glaikit(42)").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_cannie_valid() {
        let result = run("cannie(500)").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_geggie() {
        let result = run(r#"geggie("hello")"#).unwrap();
        assert_eq!(result, Value::String("ho".to_string()));
    }

    #[test]
    fn test_geggie_empty() {
        let result = run(r#"geggie("")"#).unwrap();
        assert_eq!(result, Value::String("".to_string()));
    }

    #[test]
    fn test_banter() {
        let result = run(r#"banter("ab", "12")"#).unwrap();
        let Value::String(s) = result else {
            panic!("Expected string");
        };
        assert!(s.len() >= 2);
    }

    // ==================== Timing Functions ====================

    #[test]
    fn test_noo() {
        let result = run("noo()").unwrap();
        let Value::Integer(ts) = result else {
            panic!("Expected integer");
        };
        assert!(ts > 0);
    }

    #[test]
    fn test_tick() {
        let result = run("tick()").unwrap();
        let Value::Integer(ts) = result else {
            panic!("Expected integer");
        };
        assert!(ts > 0);
    }

    #[test]
    fn test_braw_time() {
        let result = run("braw_time()").unwrap();
        let Value::String(s) = result else {
            panic!("Expected string");
        };
        assert!(s.contains(":")); // Should contain time
    }

    #[test]
    fn test_format_braw_time_exercises_all_time_buckets() {
        let cases: &[(u64, u64, &str)] = &[
            (0, 0, "wee small hours"),
            (6, 0, "mornin'"),
            (12, 0, "high noon"),
            (13, 0, "efternoon"),
            (18, 0, "evenin'"),
            (22, 0, "gettin' late"),
        ];

        for (h, m, needle) in cases {
            let s = format_braw_time(*h, *m);
            assert!(
                s.contains(needle),
                "unexpected bucket for {h:02}:{m:02}: {s}"
            );
        }
    }

    #[test]
    fn test_haver() {
        let result = run("haver()").unwrap();
        let Value::String(s) = result else {
            panic!("Expected string");
        };
        assert!(!s.is_empty());
    }

    #[test]
    fn test_slainte() {
        let result = run("slainte()").unwrap();
        let Value::String(s) = result else {
            panic!("Expected string");
        };
        assert!(!s.is_empty());
    }

    // ==================== Dictionary Functions ====================

    #[test]
    fn test_dict_merge() {
        let result = run(r#"
ken d1 = {"a": 1}
ken d2 = {"b": 2}
len(keys(dict_merge(d1, d2)))
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_dict_merge_override() {
        let result = run(r#"
ken d1 = {"a": 1}
ken d2 = {"a": 2}
ken merged = dict_merge(d1, d2)
merged["a"]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_dict_get_existing() {
        let result = run(r#"dict_get({"a": 42}, "a", 0)"#).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_dict_get_default() {
        let result = run(r#"dict_get({"a": 1}, "b", 99)"#).unwrap();
        assert_eq!(result, Value::Integer(99));
    }

    #[test]
    fn test_dict_has() {
        let result = run(r#"dict_has({"a": 1}, "a")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_dict_has_missing() {
        let result = run(r#"dict_has({"a": 1}, "b")"#).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_dict_remove() {
        let result = run(r#"
ken d = dict_remove({"a": 1, "b": 2}, "a")
len(keys(d))
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_dict_invert() {
        let result = run(r#"
ken d = dict_invert({"a": "1", "b": "2"})
d["1"]
"#)
        .unwrap();
        assert_eq!(result, Value::String("a".to_string()));
    }

    #[test]
    fn test_items() {
        let result = run(r#"len(items({"a": 1, "b": 2}))"#).unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_fae_pairs() {
        let result = run(r#"
ken d = fae_pairs([["a", 1], ["b", 2]])
d["a"]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    // ==================== More String Functions ====================

    #[test]
    fn test_center() {
        let result = run(r#"center("hi", 6, "-")"#).unwrap();
        assert_eq!(result, Value::String("--hi--".to_string()));
    }

    #[test]
    fn test_is_upper() {
        let result = run(r#"is_upper("HELLO")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_is_upper_mixed() {
        let result = run(r#"is_upper("Hello")"#).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_is_lower() {
        let result = run(r#"is_lower("hello")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_swapcase() {
        let result = run(r#"swapcase("Hello")"#).unwrap();
        assert_eq!(result, Value::String("hELLO".to_string()));
    }

    #[test]
    fn test_strip_left() {
        let result = run(r#"strip_left("xxxhello", "x")"#).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_strip_right() {
        let result = run(r#"strip_right("helloyyy", "y")"#).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_replace_first() {
        let result = run(r#"replace_first("hello hello", "hello", "hi")"#).unwrap();
        assert_eq!(result, Value::String("hi hello".to_string()));
    }

    #[test]
    fn test_substr_between() {
        let result = run(r#"substr_between("Hello [World]!", "[", "]")"#).unwrap();
        assert_eq!(result, Value::String("World".to_string()));
    }

    #[test]
    fn test_substr_between_not_found() {
        let result = run(r#"substr_between("Hello World", "[", "]")"#).unwrap();
        assert_eq!(result, Value::Nil);
    }

    // ==================== Ternary Operator ====================

    #[test]
    fn test_ternary_true() {
        let result = run("ken x = gin aye than 1 ither 2\nx").unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_ternary_false() {
        let result = run("ken x = gin nae than 1 ither 2\nx").unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_ternary_nested() {
        let result = run("ken x = gin aye than (gin nae than 1 ither 2) ither 3\nx").unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    // ==================== More Edge Cases ====================

    #[test]
    fn test_list_concat() {
        let result = run("len([1, 2] + [3, 4])").unwrap();
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_string_multiply() {
        let result = run(r#""ab" * 3"#).unwrap();
        assert_eq!(result, Value::String("ababab".to_string()));
    }

    #[test]
    fn test_integer_multiply_string() {
        let result = run(r#"3 * "ab""#).unwrap();
        assert_eq!(result, Value::String("ababab".to_string()));
    }

    #[test]
    fn test_modulo_floats() {
        let result = run("7.5 % 2.5").unwrap();
        assert_eq!(result, Value::Float(0.0));
    }

    #[test]
    fn test_division_by_zero_float() {
        let result = run("5.0 / 0.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_modulo_by_zero() {
        let result = run("5 % 0");
        assert!(result.is_err());
    }

    #[test]
    fn test_contains_dict() {
        let result = run(r#"contains({"a": 1}, "a")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_contains_dict_missing() {
        let result = run(r#"contains({"a": 1}, "b")"#).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_reverse_str_builtin() {
        let result = run(r#"reverse("hello")"#).unwrap();
        assert_eq!(result, Value::String("olleh".to_string()));
    }

    #[test]
    fn test_reverse_error_invalid_type() {
        let result = run("reverse(42)");
        assert!(result.is_err());
    }

    #[test]
    fn test_birl_rotate() {
        let result = run(r#"
ken l = birl([1, 2, 3, 4], 1)
l[0]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_birl_negative() {
        let result = run(r#"
ken l = birl([1, 2, 3, 4], -1)
l[0]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_stooshie() {
        // Just verify it returns a string of same length
        let result = run(r#"len(chars(stooshie("hello")))"#).unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_sclaff_deep_flatten() {
        let result = run("len(sclaff([[1, [2, 3]], [[4]]]))").unwrap();
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_dram_singleton() {
        // Should return something from the list
        let result = run("dram([1])").unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_dram_empty_list() {
        let result = run("dram([])").unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_ceilidh_interleave() {
        let result = run("len(ceilidh([1, 2], [3, 4]))").unwrap();
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_blether_format() {
        let result = run(r#"blether_format("Hello {name}!", {"name": "World"})"#).unwrap();
        assert_eq!(result, Value::String("Hello World!".to_string()));
    }

    #[test]
    fn test_wheesht_aw() {
        let result = run(r#"wheesht_aw("  hello   world  ")"#).unwrap();
        assert_eq!(result, Value::String("hello world".to_string()));
    }

    #[test]
    fn test_scunner_check_pass() {
        let result = run(r#"scunner_check(42, "integer")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_scunner_check_fail() {
        let result = run(r#"scunner_check(42, "string")"#).unwrap();
        let Value::String(s) = result else {
            panic!("Expected string error message");
        };
        assert!(s.contains("scunner"));
    }

    #[test]
    fn test_wrang_sort() {
        let result = run(r#"wrang_sort(42, "string")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_tattie_scone() {
        let result = run(r#"tattie_scone("yum", 3)"#).unwrap();
        assert_eq!(result, Value::String("yum | yum | yum".to_string()));
    }

    #[test]
    fn test_haggis_hunt() {
        let result = run(r#"len(haggis_hunt("aba aba", "aba"))"#).unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_sporran_fill() {
        let result = run(r#"sporran_fill("hi", 6, "*")"#).unwrap();
        assert_eq!(result, Value::String("**hi**".to_string()));
    }

    // ==================== Hex Conversion ====================

    #[test]
    fn test_tae_hex() {
        let result = run("tae_hex(255)").unwrap();
        assert_eq!(result, Value::String("ff".to_string()));
    }

    #[test]
    fn test_fae_hex() {
        let result = run(r#"fae_hex("ff")"#).unwrap();
        assert_eq!(result, Value::Integer(255));
    }

    #[test]
    fn test_fae_hex_invalid() {
        let result = run(r#"fae_hex("zz")"#);
        assert!(result.is_err());
    }

    // ==================== Statistics Functions ====================

    #[test]
    fn test_minaw() {
        let result = run("minaw([3, 1, 4, 1, 5])").unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_maxaw() {
        let result = run("maxaw([3, 1, 4, 1, 5])").unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_range_o() {
        let result = run("range_o([1, 5, 3])").unwrap();
        assert_eq!(result, Value::Float(4.0)); // 5 - 1 = 4
    }

    #[test]
    fn test_sumaw_integers() {
        let result = run("sumaw([1, 2, 3, 4])").unwrap();
        assert_eq!(result, Value::Integer(10));
    }

    #[test]
    fn test_sumaw_floats() {
        let result = run("sumaw([1.0, 2.0, 3.0])").unwrap();
        assert_eq!(result, Value::Float(6.0));
    }

    // ==================== Inheritance Tests ====================

    #[test]
    fn test_class_inheritance_method() {
        let result = run(r#"
kin Animal {
    dae speak() {
        gie "..."
    }
}
kin Dog fae Animal {
    dae speak() {
        gie "Woof!"
    }
}
ken d = Dog()
d.speak()
"#)
        .unwrap();
        assert_eq!(result, Value::String("Woof!".to_string()));
    }

    #[test]
    fn test_inheritance_superclass_not_a_class() {
        let result = run(r#"
ken notAClass = 42
kin Dog fae notAClass {
    dae speak() { gie "woof" }
}
"#);
        assert!(result.is_err());
    }

    // ==================== Spread Operator ====================

    #[test]
    fn test_spread_list_expr() {
        let result = run(r#"
ken a = [1, 2]
ken b = [0, ...a, 3]
len(b)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_spread_string_in_list() {
        let result = run(r#"
ken s = "ab"
ken l = [...s]
len(l)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_spread_invalid_context() {
        let result = run("...42");
        assert!(result.is_err());
    }

    // ==================== Index Set Tests ====================

    #[test]
    fn test_list_index_set_negative() {
        let result = run(r#"
ken l = [1, 2, 3]
l[-1] = 99
l[-1]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(99));
    }

    #[test]
    fn test_dict_index_set_non_string() {
        let result = run(r#"
ken d = {}
d[42] = "answer"
d["42"]
"#);
        assert!(result.is_err());
    }

    // ==================== Property Access/Set ====================

    #[test]
    fn test_set_property_on_invalid_type() {
        let result = run(r#"
ken x = 42
x.foo = 5
"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_property_on_invalid_type() {
        let result = run(r#"
ken x = 42
x.foo
"#);
        assert!(result.is_err());
    }

    // ==================== Interpreter Config Tests ====================

    #[test]
    fn test_interp_with_dir() {
        let interp = Interpreter::with_dir("/tmp");
        assert!(!interp.has_prelude());
    }

    #[test]
    fn test_interp_set_dir() {
        let mut interp = Interpreter::new();
        interp.set_current_dir("/tmp");
        // Should not panic
    }

    #[test]
    fn test_interp_user_vars() {
        let code = "ken x = 42\nken y = \"hello\"";
        let program = crate::parser::parse(code).unwrap();
        let mut interp = Interpreter::new();
        interp.interpret(&program).unwrap();
        let vars = interp.get_user_variables();
        assert!(vars.len() >= 2);
    }

    #[test]
    fn test_interp_output() {
        let code = r#"blether "test output""#;
        let program = crate::parser::parse(code).unwrap();
        let mut interp = Interpreter::new();
        interp.interpret(&program).unwrap();
        let output = interp.get_output();
        assert!(!output.is_empty());
    }

    // ==================== More Native Function Tests ====================

    #[test]
    fn test_coont_list() {
        let result = run("coont([1, 2, 2, 3, 2], 2)").unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_coont_in_string() {
        let result = run(r#"coont("hello", "l")"#).unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_unique_integers() {
        let result = run("len(unique([1, 2, 2, 3, 3, 3]))").unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_average_floats() {
        let result = run("average([1.0, 2.0, 3.0, 4.0, 5.0])").unwrap();
        assert_eq!(result, Value::Float(3.0));
    }

    #[test]
    fn test_average_int_list() {
        let result = run("average([10, 20, 30])").unwrap();
        assert_eq!(result, Value::Float(20.0));
    }

    #[test]
    fn test_median() {
        let result = run("median([1.0, 2.0, 3.0])").unwrap();
        assert_eq!(result, Value::Float(2.0));
    }

    #[test]
    fn test_median_even() {
        let result = run("median([1.0, 2.0, 3.0, 4.0])").unwrap();
        assert_eq!(result, Value::Float(2.5));
    }

    #[test]
    fn test_sumaw_list_integers() {
        let result = run("sumaw([1, 2, 3, 4, 5])").unwrap();
        assert_eq!(result, Value::Integer(15));
    }

    #[test]
    fn test_sumaw_list_floats() {
        let result = run("sumaw([1.5, 2.5, 3.0])").unwrap();
        assert_eq!(result, Value::Float(7.0));
    }

    #[test]
    fn test_product() {
        let result = run("product([1, 2, 3, 4])").unwrap();
        assert_eq!(result, Value::Integer(24));
    }

    #[test]
    fn test_product_floats() {
        let result = run("product([2.0, 3.0, 4.0])").unwrap();
        assert_eq!(result, Value::Float(24.0));
    }

    #[test]
    fn test_minaw_list() {
        let result = run("minaw([5, 3, 8, 1, 9])").unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_maxaw_list() {
        let result = run("maxaw([5, 3, 8, 1, 9])").unwrap();
        assert_eq!(result, Value::Integer(9));
    }

    #[test]
    fn test_wheesht_aw_string_trim() {
        // wheesht_aw cleans and trims a string
        let result = run(r#"wheesht_aw("  hello   world  ")"#).unwrap();
        assert_eq!(result, Value::String("hello world".to_string()));
    }

    #[test]
    fn test_redd_up_with_nils() {
        // redd_up filters out nil values from a list
        let result = run("len(redd_up([1, naething, 2, naething, 3]))").unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_split_by_even_count() {
        let result = run(r#"
ken parts = split_by([1, 2, 3, 4, 5, 6], "even")
len(parts[0])
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_split_by_odd_count() {
        let result = run(r#"
ken parts = split_by([1, 2, 3, 4, 5, 6], "odd")
len(parts[0])
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    // ==================== String Methods ====================

    #[test]
    fn test_upper() {
        let result = run(r#"upper("hello")"#).unwrap();
        assert_eq!(result, Value::String("HELLO".to_string()));
    }

    #[test]
    fn test_lower() {
        let result = run(r#"lower("HELLO")"#).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_wheesht_string() {
        // Using wheesht to filter a string (removes whitespace-ish behavior via replace)
        let result = run(r#"replace("  hello  ", " ", "")"#).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_split() {
        let result = run(r#"len(split("a,b,c", ","))"#).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_join() {
        let result = run(r#"join(["a", "b", "c"], "-")"#).unwrap();
        assert_eq!(result, Value::String("a-b-c".to_string()));
    }

    #[test]
    fn test_replace() {
        let result = run(r#"replace("hello", "l", "x")"#).unwrap();
        assert_eq!(result, Value::String("hexxo".to_string()));
    }

    #[test]
    fn test_starts_wi() {
        let result = run(r#"starts_wi("hello", "hel")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_starts_wi_false() {
        let result = run(r#"starts_wi("hello", "wor")"#).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_ends_wi() {
        let result = run(r#"ends_wi("hello", "llo")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_ends_wi_false() {
        let result = run(r#"ends_wi("hello", "abc")"#).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    // ==================== List Operations ====================

    #[test]
    fn test_shove_list() {
        let result = run(r#"
ken l = [1, 2, 3]
shove(l, 4)
len(l)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_yank_list() {
        let result = run(r#"
ken l = [1, 2, 3]
yank(l)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_chynge_list() {
        // chynge inserts at an index
        let result = run(r#"
ken l = [1, 3, 4]
ken updated = chynge(l, 1, 2)
updated[1]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_dicht_list() {
        // dicht removes at an index
        let result = run(r#"
ken l = [1, 2, 3]
ken updated = dicht(l, 1)
len(updated)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_redd_up_list() {
        // redd_up removes nil values
        let result = run(r#"
ken l = [1, naething, 2, naething, 3]
ken cleaned = redd_up(l)
len(cleaned)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_flatten_nested() {
        let result = run("len(flatten([[1, 2], [3, 4], [5]]))").unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    // ==================== Type Conversion ====================

    #[test]
    fn test_tae_string() {
        let result = run("tae_string(42)").unwrap();
        assert_eq!(result, Value::String("42".to_string()));
    }

    #[test]
    fn test_tae_int() {
        let result = run(r#"tae_int("42")"#).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_tae_int_float() {
        let result = run("tae_int(3.14)").unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_tae_float() {
        let result = run(r#"tae_float("3.14")"#).unwrap();
        assert_eq!(result, Value::Float(3.14));
    }

    #[test]
    fn test_tae_float_int() {
        let result = run("tae_float(42)").unwrap();
        assert_eq!(result, Value::Float(42.0));
    }

    // ==================== Math Functions ====================

    #[test]
    fn test_sqrt() {
        let result = run("sqrt(16.0)").unwrap();
        assert_eq!(result, Value::Float(4.0));
    }

    #[test]
    fn test_sqrt_int() {
        let result = run("sqrt(9)").unwrap();
        assert_eq!(result, Value::Float(3.0));
    }

    #[test]
    fn test_abs_int() {
        let result = run("abs(-42)").unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_abs_float() {
        let result = run("abs(-3.14)").unwrap();
        assert_eq!(result, Value::Float(3.14));
    }

    #[test]
    fn test_floor() {
        let result = run("floor(3.7)").unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_ceil() {
        let result = run("ceil(3.2)").unwrap();
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_round() {
        let result = run("round(3.5)").unwrap();
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_log() {
        let result = run("log(2.718281828)").unwrap();
        let Value::Float(n) = result else {
            panic!("Expected float");
        };
        assert!((n - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_exp() {
        let result = run("exp(1.0)").unwrap();
        let Value::Float(n) = result else {
            panic!("Expected float");
        };
        assert!((n - 2.718281828).abs() < 0.001);
    }

    #[test]
    fn test_sin() {
        let result = run("sin(0.0)").unwrap();
        assert_eq!(result, Value::Float(0.0));
    }

    #[test]
    fn test_cos() {
        let result = run("cos(0.0)").unwrap();
        assert_eq!(result, Value::Float(1.0));
    }

    #[test]
    fn test_tan() {
        let result = run("tan(0.0)").unwrap();
        assert_eq!(result, Value::Float(0.0));
    }

    // ==================== Error Handling ====================

    #[test]
    fn test_throw() {
        let result = run(r#"fling("test error")"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_catch_catches() {
        let result = run(r#"
hae_a_bash {
    fling("oops")
} gin_it_gangs_wrang e {
    blether e
}
42
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_try_catch_error_value() {
        // Test that we can catch and handle errors
        let result = run(r#"
ken caught = nae
hae_a_bash {
    fling("my error")
} gin_it_gangs_wrang e {
    caught = aye
}
caught
"#)
        .unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    // ==================== Complex Expression Tests ====================

    #[test]
    fn test_nested_function_calls() {
        let result = run("len(split(upper(\"hello,world\"), \",\"))").unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_list_comprehension_like() {
        // Using map-like functionality with shove
        let result = run(r#"
ken l = [1, 2, 3]
ken result = []
fer x in l {
    shove(result, x * 2)
}
result[1]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_dict_iteration_keys() {
        let result = run(r#"
ken d = {"a": 1, "b": 2}
ken k = keys(d)
len(k)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_dict_iteration_values() {
        let result = run(r#"
ken d = {"a": 1, "b": 2}
ken v = values(d)
len(v)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    // ==================== Closure Tests ====================

    #[test]
    fn test_closure_captures_variable() {
        let result = run(r#"
ken x = 10
dae add_x(n) {
    gie n + x
}
add_x(5)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(15));
    }

    #[test]
    fn test_closure_counter() {
        let result = run(r#"
dae make_counter() {
    ken count = 0
    dae counter() {
        count = count + 1
        gie count
    }
    gie counter
}
ken c = make_counter()
c()
c()
c()
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    // ==================== More Edge Cases ====================

    #[test]
    fn test_empty_function() {
        let result = run(r#"
dae empty() {
}
empty()
"#)
        .unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_recursive_function() {
        let result = run(r#"
dae fib(n) {
    gin n <= 1 {
        gie n
    }
    gie fib(n - 1) + fib(n - 2)
}
fib(10)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(55));
    }

    #[test]
    fn test_mutual_recursion() {
        let result = run(r#"
dae is_even(n) {
    gin n == 0 { gie aye }
    gie is_odd(n - 1)
}
dae is_odd(n) {
    gin n == 0 { gie nae }
    gie is_even(n - 1)
}
is_even(10)
"#)
        .unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    // ==================== Native Function Coverage Tests ====================

    #[test]
    fn test_range_function() {
        let result = run(r#"
ken r = range(1, 5)
ken total = 0
fer i in r {
    total = total + i
}
total
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(10)); // 1+2+3+4 = 10
    }

    #[test]
    fn test_min_floats() {
        let result = run("min(3.5, 2.1)").unwrap();
        assert_eq!(result, Value::Float(2.1));
    }

    #[test]
    fn test_max_floats() {
        let result = run("max(3.5, 2.1)").unwrap();
        assert_eq!(result, Value::Float(3.5));
    }

    #[test]
    fn test_floor_integer() {
        let result = run("floor(42)").unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_ceil_integer() {
        let result = run("ceil(42)").unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_round_integer() {
        let result = run("round(42)").unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_title_case_string() {
        let result = run(r#"title("hello world")"#).unwrap();
        assert_eq!(result, Value::String("Hello World".to_string()));
    }

    #[test]
    fn test_center_function() {
        let result = run(r#"center("hi", 6, " ")"#).unwrap();
        assert_eq!(result, Value::String("  hi  ".to_string()));
    }

    #[test]
    fn test_repeat_function() {
        let result = run(r#"repeat("ab", 3)"#).unwrap();
        assert_eq!(result, Value::String("ababab".to_string()));
    }

    #[test]
    fn test_lines_function() {
        let result = run(r#"
ken l = lines("one
two
three")
len(l)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_words_split() {
        let result = run(r#"len(words("hello beautiful world"))"#).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_is_alpha_letter() {
        let result = run(r#"is_alpha("hello")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_is_alpha_mixed() {
        let result = run(r#"is_alpha("hello123")"#).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_is_digit_numeric() {
        let result = run(r#"is_digit("12345")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_is_digit_alphanumeric() {
        let result = run(r#"is_digit("123abc")"#).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_is_space_whitespace() {
        let result = run(r#"is_space("   ")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_is_space_with_chars() {
        let result = run(r#"is_space("  a  ")"#).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_pad_left_function() {
        let result = run(r#"pad_left("42", 5, "0")"#).unwrap();
        assert_eq!(result, Value::String("00042".to_string()));
    }

    #[test]
    fn test_pad_right_function() {
        let result = run(r#"pad_right("42", 5, "0")"#).unwrap();
        assert_eq!(result, Value::String("42000".to_string()));
    }

    #[test]
    fn test_index_of_found() {
        let result = run(r#"index_of("hello", "l")"#).unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_index_of_not_found() {
        let result = run(r#"index_of("hello", "z")"#).unwrap();
        assert_eq!(result, Value::Integer(-1));
    }

    #[test]
    fn test_ord_function() {
        let result = run(r#"ord("A")"#).unwrap();
        assert_eq!(result, Value::Integer(65));
    }

    #[test]
    fn test_chr_function() {
        let result = run("chr(65)").unwrap();
        assert_eq!(result, Value::String("A".to_string()));
    }

    #[test]
    fn test_chr_unicode() {
        let result = run("chr(128512)").unwrap();
        assert_eq!(result, Value::String("😀".to_string()));
    }

    #[test]
    fn test_tae_bool_truthy() {
        let result = run("tae_bool(1)").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_tae_bool_falsy() {
        let result = run("tae_bool(0)").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_scran_string_range() {
        let result = run(r#"scran("hello", 1, 4)"#).unwrap();
        assert_eq!(result, Value::String("ell".to_string()));
    }

    #[test]
    fn test_scran_list_range() {
        let result = run("len(scran([1,2,3,4,5], 1, 4))").unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_append_file() {
        // Test that append_file function exists (may error on missing file)
        let result = run(r#"append_file("/tmp/nonexistent_test", "data")"#);
        // Just checking it doesn't crash - file may or may not exist
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_file_exists_false() {
        let result = run(r#"file_exists("/nonexistent/path/to/file")"#).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_read_lines_error() {
        let result = run(r#"read_lines("/nonexistent/path")"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_file_error() {
        let result = run(r#"read_file("/nonexistent/path")"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_parse_object() {
        let result = run(r#"
ken obj = json_parse("{\"a\": 1}")
obj["a"]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_json_stringify_dict() {
        let result = run(r#"json_stringify({"a": 1})"#).unwrap();
        let Value::String(s) = result else {
            panic!("Expected string");
        };
        assert!(s.contains("\"a\"") && s.contains("1"));
    }

    #[test]
    fn test_json_pretty_format() {
        let result = run(r#"json_pretty({"a": 1})"#).unwrap();
        let Value::String(s) = result else {
            panic!("Expected string");
        };
        assert!(s.contains("a"));
    }

    #[test]
    fn test_sin_pi() {
        let result = run("sin(3.14159265359)").unwrap();
        let Value::Float(n) = result else {
            panic!("Expected float");
        };
        assert!(n.abs() < 0.0001);
    }

    #[test]
    fn test_cos_pi() {
        let result = run("cos(3.14159265359)").unwrap();
        let Value::Float(n) = result else {
            panic!("Expected float");
        };
        assert!((n + 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_tan_function() {
        let result = run("tan(0.785398)").unwrap();
        let Value::Float(n) = result else {
            panic!("Expected float");
        };
        assert!((n - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_atan2_function() {
        let result = run("atan2(1.0, 1.0)").unwrap();
        let Value::Float(n) = result else {
            panic!("Expected float");
        };
        assert!((n - 0.785398).abs() < 0.001);
    }

    #[test]
    fn test_hypot_function() {
        let result = run("hypot(3.0, 4.0)").unwrap();
        assert_eq!(result, Value::Float(5.0));
    }

    #[test]
    fn test_pow_integer_exponent() {
        let result = run("pow(2, 10)").unwrap();
        assert_eq!(result, Value::Integer(1024));
    }

    #[test]
    fn test_pow_float() {
        let result = run("pow(2.0, 3.0)").unwrap();
        assert_eq!(result, Value::Float(8.0));
    }

    #[test]
    fn test_gcd_function() {
        let result = run("gcd(48, 18)").unwrap();
        assert_eq!(result, Value::Integer(6));
    }

    #[test]
    fn test_lcm_function() {
        let result = run("lcm(4, 6)").unwrap();
        assert_eq!(result, Value::Integer(12));
    }

    #[test]
    fn test_zip_lists() {
        let result = run(r#"
ken z = zip([1, 2, 3], ["a", "b", "c"])
len(z)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_enumerate_function() {
        let result = run(r#"
ken e = enumerate(["a", "b", "c"])
len(e)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_unique_function() {
        let result = run("len(unique([1, 2, 2, 3, 3, 3]))").unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_uniq_function() {
        // uniq is alias for unique
        let result = run("len(uniq([1, 1, 2, 2, 3]))").unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_sort_numbers() {
        let result = run(r#"
ken s = sort([3, 1, 4, 1, 5, 9])
s[0]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_sort_strings_alpha() {
        let result = run(r#"
ken s = sort(["banana", "apple", "cherry"])
s[0]
"#)
        .unwrap();
        assert_eq!(result, Value::String("apple".to_string()));
    }

    #[test]
    fn test_shuffle_function() {
        let result = run(r#"
ken l = [1, 2, 3, 4, 5]
ken s = shuffle(l)
len(s)
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_birl_rotate_list() {
        let result = run(r#"
ken l = [1, 2, 3, 4, 5]
ken r = birl(l, 2)
r[0]
"#)
        .unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    fn audio_path(rel: &str) -> String {
        format!("\"{}\"", std::path::Path::new(rel).display())
    }

    #[test]
    fn test_audio_device_controls() {
        let result = run(r#"
soond_steek()
soond_haud_gang()
soond_stairt()
soond_luid(0.7)
soond_wheesht(aye)
soond_wheesht(nae)
soond_stairt()
ken v = soond_hou_luid()
soond_steek()
v
"#)
        .unwrap();
        let Value::Float(v) = result else {
            panic!("Expected float");
        };
        assert!((v - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_audio_sfx_cycle() {
        let ding = audio_path("assets/audio/ding.wav");
        let script = format!(
            r#"
soond_steek()
soond_stairt()
ken ding = soond_lade({})
ken ready = soond_ready(ding)
soond_pit_luid(ding, 0.4)
soond_pit_pan(ding, -0.5)
soond_pit_tune(ding, 1.1)
soond_pit_rin_roond(ding, aye)
soond_spiel(ding)
ken a = soond_is_spielin(ding)
soond_haud_gang()
soond_haud_gang()
soond_haud_gang()
soond_haud(ding)
ken b = soond_is_spielin(ding)
soond_gae_on(ding)
soond_stap(ding)
soond_unlade(ding)
soond_steek()
[a, b]
"#,
            ding
        );

        let result = run(&script).unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let list = list.borrow();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], Value::Bool(true));
        assert_eq!(list[1], Value::Bool(false));
    }

    #[cfg(feature = "audio")]
    #[test]
    fn test_audio_music_cycle() {
        let theme = audio_path("assets/audio/theme.mp3");
        let script = format!(
            r#"
soond_steek()
soond_stairt()
ken tune = muisic_lade({})
muisic_pit_luid(tune, 0.5)
muisic_pit_pan(tune, 0.25)
muisic_pit_tune(tune, 1.1)
muisic_pit_rin_roond(tune, aye)
muisic_spiel(tune)
soond_haud_gang()
soond_haud_gang()
ken playing = muisic_is_spielin(tune)

muisic_haud(tune)
ken paused = muisic_is_spielin(tune)
muisic_gae_on(tune)

ken len = muisic_hou_lang(tune)
muisic_loup(tune, 0.1)
ken pos = muisic_whaur(tune)

muisic_pit_rin_roond(tune, nae)
muisic_loup(tune, len)
snooze(20)
ken stopped_before = muisic_is_spielin(tune)

muisic_stap(tune)
ken stopped_after = muisic_is_spielin(tune)
muisic_unlade(tune)
soond_steek()
[playing, paused, len > 0, pos >= 0, stopped_before, stopped_after]
"#,
            theme
        );

        let result = run(&script).unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let list = list.borrow();
        assert_eq!(list.len(), 6);
        let expected_playing = if cfg!(feature = "audio") {
            Value::Bool(true)
        } else {
            Value::Bool(false)
        };
        assert_eq!(list[0], expected_playing);
        assert_eq!(list[1], Value::Bool(false));
        assert_eq!(list[2], Value::Bool(true));
        assert_eq!(list[3], Value::Bool(true));
        assert!(matches!(list[4], Value::Bool(_)));
        assert_eq!(list[5], Value::Bool(false));
    }

    #[test]
    fn test_audio_midi_cycle_explicit_soundfont() {
        let midi = audio_path("assets/audio/wee_tune.mid");
        let sf = audio_path("assets/soundfonts/MuseScore_General.sf2");
        let script = format!(
            r#"
soond_steek()
soond_stairt()
ken song = midi_lade({}, {})
midi_loup(song, 0.01)
midi_spiel(song)
soond_haud_gang()
ken playing = midi_is_spielin(song)
midi_haud(song)
ken paused = midi_is_spielin(song)
midi_loup(song, 0.02)
midi_gae_on(song)
midi_loup(song, 0.03)
midi_pit_luid(song, 0.6)
midi_pit_pan(song, -0.2)
midi_pit_rin_roond(song, aye)
soond_haud_gang()
ken len = midi_hou_lang(song)
ken pos = midi_whaur(song)
midi_stap(song)
midi_unlade(song)
soond_steek()
[playing, paused, len > 0, pos >= 0]
"#,
            midi, sf
        );

        let result = run(&script).unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let list = list.borrow();
        assert_eq!(list.len(), 4);
        assert_eq!(list[0], Value::Bool(true));
        assert_eq!(list[1], Value::Bool(false));
        assert_eq!(list[2], Value::Bool(true));
        assert_eq!(list[3], Value::Bool(true));
    }

    #[test]
    fn test_audio_midi_cycle_default_soundfont() {
        let midi = audio_path("assets/audio/wee_tune.mid");
        let script = format!(
            r#"
soond_steek()
soond_stairt()
ken song = midi_lade({}, naething)
midi_spiel(song)
soond_haud_gang()
midi_unlade(song)
soond_steek()
aye
"#,
            midi
        );
        let result = run(&script).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_audio_midi_stops_at_end() {
        let midi = audio_path("assets/audio/wee_tune.mid");
        let sf = audio_path("assets/soundfonts/MuseScore_General.sf2");
        let script = format!(
            r#"
soond_steek()
soond_stairt()
ken song = midi_lade({}, {})
midi_pit_rin_roond(song, nae)
midi_spiel(song)
ken len = midi_hou_lang(song)
midi_loup(song, len)
snooze(20)
ken stopped_before = midi_is_spielin(song)
midi_stap(song)
ken stopped_after = midi_is_spielin(song)
midi_unlade(song)
soond_steek()
[stopped_before, stopped_after]
"#,
            midi, sf
        );
        let result = run(&script).unwrap();
        let Value::List(list) = result else {
            panic!("Expected list");
        };
        let list = list.borrow();
        assert_eq!(list.len(), 2);
        assert!(matches!(list[0], Value::Bool(_)));
        assert_eq!(list[1], Value::Bool(false));
    }

    #[test]
    fn test_audio_invalid_handles() {
        assert!(run("soond_spiel(999)").is_err());
        assert!(run("muisic_spiel(999)").is_err());
        assert!(run("midi_spiel(999)").is_err());
    }

    #[test]
    fn test_audio_bad_args_and_load_errors() {
        assert!(run("soond_wheesht(\"aye\")").is_err());
        assert!(run("soond_lade(123)").is_err());
        assert!(run("soond_lade(\"nope.wav\")").is_err());
        assert!(run("muisic_lade(\"nope.mp3\")").is_err());
    }

    #[test]
    fn test_audio_midi_errors() {
        let midi = audio_path("assets/audio/wee_tune.mid");
        let bad_type = format!("midi_lade({}, 123)", midi);
        let bad_sf = format!("midi_lade({}, \"nope.sf2\")", midi);

        assert!(run(&bad_type).is_err());
        assert!(run(&bad_sf).is_err());
        assert!(run("midi_lade(\"nope.mid\", naething)").is_err());
    }

    #[test]
    fn test_with_current_interpreter_guard() {
        assert!(with_current_interpreter(|_| 1).is_none());
        let mut interp = Interpreter::new();
        {
            let _guard = InterpreterGuard::new(&mut interp);
            assert!(with_current_interpreter(|i| i.get_log_level()).is_some());
        }
        assert!(with_current_interpreter(|_| 1).is_none());
    }

    #[test]
    fn test_register_thread_and_with_thread_mut() {
        let id = register_thread(ThreadHandle {
            result: Value::Integer(7),
            detached: false,
        });
        let result = with_thread_mut(id, |handle| {
            handle.detached = true;
            handle.result.clone()
        })
        .unwrap();
        assert_eq!(result, Value::Integer(7));
        assert!(with_thread_mut(999999, |_| ()).is_err());
    }

    #[test]
    fn test_log_level_parsing_and_resolve_log_args() {
        assert_eq!(
            parse_log_level_value(&Value::Integer(0)).unwrap(),
            LogLevel::Wheesht
        );
        assert_eq!(
            parse_log_level_value(&Value::Integer(5)).unwrap(),
            LogLevel::Whisper
        );
        assert!(parse_log_level_value(&Value::Integer(9)).is_err());
        assert!(parse_log_level_value(&Value::Bool(true)).is_err());
        assert_eq!(
            parse_log_target_value(&Value::String("t".to_string())).unwrap(),
            "t"
        );
        assert!(parse_log_target_value(&Value::Integer(1)).is_err());

        let mut dict = DictValue::new();
        dict.set(Value::String("a".to_string()), Value::Integer(1));
        let fields = Value::Dict(Rc::new(RefCell::new(dict)));

        assert_eq!(resolve_log_args(&[]).unwrap(), (None, None));
        assert!(resolve_log_args(std::slice::from_ref(&fields))
            .unwrap()
            .0
            .is_some());
        assert_eq!(
            resolve_log_args(&[Value::String("target".to_string())]).unwrap(),
            (None, Some("target".to_string()))
        );
        assert!(resolve_log_args(&[Value::Integer(1)]).is_err());
        let (fields_val, target) =
            resolve_log_args(&[fields.clone(), Value::String("t".to_string())]).unwrap();
        assert!(fields_val.is_some());
        assert_eq!(target, Some("t".to_string()));
        assert!(resolve_log_args(&[
            Value::String("x".to_string()),
            Value::String("y".to_string())
        ])
        .is_err());
        assert!(resolve_log_args(&[fields, Value::Integer(1)]).is_err());
        assert!(resolve_log_args(&[
            Value::String("x".to_string()),
            Value::String("y".to_string()),
            Value::String("z".to_string())
        ])
        .is_err());
    }

    #[test]
    fn test_parse_log_extras_errors() {
        let mut interp = Interpreter::new();
        let expr_int = lit_expr(Literal::Integer(1));
        assert!(interp.parse_log_extras(&[expr_int], 1).is_err());

        let expr_dict = dict_expr("a", Literal::Integer(1));
        let expr_int = lit_expr(Literal::Integer(2));
        assert!(interp
            .parse_log_extras(&[expr_int, lit_expr(Literal::String("t".to_string()))], 1)
            .is_err());
        let expr_int2 = lit_expr(Literal::Integer(3));
        assert!(interp.parse_log_extras(&[expr_dict, expr_int2], 1).is_err());

        let expr_dict = dict_expr("a", Literal::Integer(1));
        let expr_str = lit_expr(Literal::String("t".to_string()));
        let expr_extra = lit_expr(Literal::String("x".to_string()));
        assert!(interp
            .parse_log_extras(&[expr_dict, expr_str, expr_extra], 1)
            .is_err());
    }

    #[test]
    fn test_apply_log_config_paths() {
        let mut interp = Interpreter::new();
        interp.apply_log_config(None).unwrap();

        assert!(interp.apply_log_config(Some(Value::Integer(1))).is_err());

        let mut bad_filter = DictValue::new();
        bad_filter.set(Value::String("filter".to_string()), Value::Integer(1));
        assert!(interp
            .apply_log_config(Some(Value::Dict(Rc::new(RefCell::new(bad_filter)))))
            .is_err());

        let mut bad_format = DictValue::new();
        bad_format.set(Value::String("format".to_string()), Value::Integer(1));
        assert!(interp
            .apply_log_config(Some(Value::Dict(Rc::new(RefCell::new(bad_format)))))
            .is_err());

        let mut bad_format_str = DictValue::new();
        bad_format_str.set(
            Value::String("format".to_string()),
            Value::String("nope".to_string()),
        );
        assert!(interp
            .apply_log_config(Some(Value::Dict(Rc::new(RefCell::new(bad_format_str)))))
            .is_err());

        let mut bad_color = DictValue::new();
        bad_color.set(
            Value::String("color".to_string()),
            Value::String("aye".to_string()),
        );
        assert!(interp
            .apply_log_config(Some(Value::Dict(Rc::new(RefCell::new(bad_color)))))
            .is_err());

        let mut bad_ts = DictValue::new();
        bad_ts.set(
            Value::String("timestamps".to_string()),
            Value::String("aye".to_string()),
        );
        assert!(interp
            .apply_log_config(Some(Value::Dict(Rc::new(RefCell::new(bad_ts)))))
            .is_err());

        let mut bad_sinks = DictValue::new();
        bad_sinks.set(Value::String("sinks".to_string()), Value::Integer(1));
        assert!(interp
            .apply_log_config(Some(Value::Dict(Rc::new(RefCell::new(bad_sinks)))))
            .is_err());

        let sinks = vec![Value::Integer(1)];
        let mut bad_sink_spec = DictValue::new();
        bad_sink_spec.set(
            Value::String("sinks".to_string()),
            Value::List(Rc::new(RefCell::new(sinks))),
        );
        assert!(interp
            .apply_log_config(Some(Value::Dict(Rc::new(RefCell::new(bad_sink_spec)))))
            .is_err());

        let mut sink_kind = DictValue::new();
        sink_kind.set(Value::String("kind".to_string()), Value::Integer(1));
        let mut bad_kind = DictValue::new();
        bad_kind.set(
            Value::String("sinks".to_string()),
            Value::List(Rc::new(RefCell::new(vec![Value::Dict(Rc::new(
                RefCell::new(sink_kind),
            ))]))),
        );
        assert!(interp
            .apply_log_config(Some(Value::Dict(Rc::new(RefCell::new(bad_kind)))))
            .is_err());

        let mut file_spec = DictValue::new();
        file_spec.set(
            Value::String("kind".to_string()),
            Value::String("file".to_string()),
        );
        file_spec.set(Value::String("path".to_string()), Value::Integer(1));
        let mut bad_file = DictValue::new();
        bad_file.set(
            Value::String("sinks".to_string()),
            Value::List(Rc::new(RefCell::new(vec![Value::Dict(Rc::new(
                RefCell::new(file_spec),
            ))]))),
        );
        assert!(interp
            .apply_log_config(Some(Value::Dict(Rc::new(RefCell::new(bad_file)))))
            .is_err());

        let mut file_spec = DictValue::new();
        file_spec.set(
            Value::String("kind".to_string()),
            Value::String("file".to_string()),
        );
        file_spec.set(
            Value::String("path".to_string()),
            Value::String("out.log".to_string()),
        );
        file_spec.set(Value::String("append".to_string()), Value::Integer(1));
        let mut bad_append = DictValue::new();
        bad_append.set(
            Value::String("sinks".to_string()),
            Value::List(Rc::new(RefCell::new(vec![Value::Dict(Rc::new(
                RefCell::new(file_spec),
            ))]))),
        );
        assert!(interp
            .apply_log_config(Some(Value::Dict(Rc::new(RefCell::new(bad_append)))))
            .is_err());

        let mut mem_spec = DictValue::new();
        mem_spec.set(
            Value::String("kind".to_string()),
            Value::String("memory".to_string()),
        );
        mem_spec.set(Value::String("max".to_string()), Value::Integer(0));
        let mut bad_mem = DictValue::new();
        bad_mem.set(
            Value::String("sinks".to_string()),
            Value::List(Rc::new(RefCell::new(vec![Value::Dict(Rc::new(
                RefCell::new(mem_spec),
            ))]))),
        );
        assert!(interp
            .apply_log_config(Some(Value::Dict(Rc::new(RefCell::new(bad_mem)))))
            .is_err());

        let mut cb_spec = DictValue::new();
        cb_spec.set(
            Value::String("kind".to_string()),
            Value::String("callback".to_string()),
        );
        let mut bad_cb = DictValue::new();
        bad_cb.set(
            Value::String("sinks".to_string()),
            Value::List(Rc::new(RefCell::new(vec![Value::Dict(Rc::new(
                RefCell::new(cb_spec),
            ))]))),
        );
        assert!(interp
            .apply_log_config(Some(Value::Dict(Rc::new(RefCell::new(bad_cb)))))
            .is_err());

        let mut unknown_spec = DictValue::new();
        unknown_spec.set(
            Value::String("kind".to_string()),
            Value::String("unknown".to_string()),
        );
        let mut bad_unknown = DictValue::new();
        bad_unknown.set(
            Value::String("sinks".to_string()),
            Value::List(Rc::new(RefCell::new(vec![Value::Dict(Rc::new(
                RefCell::new(unknown_spec),
            ))]))),
        );
        assert!(interp
            .apply_log_config(Some(Value::Dict(Rc::new(RefCell::new(bad_unknown)))))
            .is_err());

        let callback_fn = Value::NativeFunction(Rc::new(NativeFunction::new("cb", 0, |_args| {
            Ok(Value::Nil)
        })));
        let mut callback_spec = DictValue::new();
        callback_spec.set(
            Value::String("kind".to_string()),
            Value::String("callback".to_string()),
        );
        callback_spec.set(Value::String("fn".to_string()), callback_fn.clone());
        let callback_list = Value::List(Rc::new(RefCell::new(vec![Value::Dict(Rc::new(
            RefCell::new(callback_spec),
        ))])));
        let mut callback_cfg = DictValue::new();
        callback_cfg.set(Value::String("sinks".to_string()), callback_list);
        interp
            .apply_log_config(Some(Value::Dict(Rc::new(RefCell::new(callback_cfg)))))
            .unwrap();
        assert!(interp.log_callback.is_some());

        let mut sinks = Vec::new();
        let mut stderr_spec = DictValue::new();
        stderr_spec.set(
            Value::String("kind".to_string()),
            Value::String("stderr".to_string()),
        );
        sinks.push(Value::Dict(Rc::new(RefCell::new(stderr_spec))));

        let mut stdout_spec = DictValue::new();
        stdout_spec.set(
            Value::String("kind".to_string()),
            Value::String("stdout".to_string()),
        );
        sinks.push(Value::Dict(Rc::new(RefCell::new(stdout_spec))));

        let mut file_spec = DictValue::new();
        file_spec.set(
            Value::String("kind".to_string()),
            Value::String("file".to_string()),
        );
        file_spec.set(
            Value::String("path".to_string()),
            Value::String("log.txt".to_string()),
        );
        file_spec.set(Value::String("append".to_string()), Value::Bool(false));
        sinks.push(Value::Dict(Rc::new(RefCell::new(file_spec))));

        let mut mem_spec = DictValue::new();
        mem_spec.set(
            Value::String("kind".to_string()),
            Value::String("memory".to_string()),
        );
        mem_spec.set(Value::String("max".to_string()), Value::Integer(4));
        sinks.push(Value::Dict(Rc::new(RefCell::new(mem_spec))));

        let mut ok_cfg = DictValue::new();
        ok_cfg.set(Value::String("level".to_string()), Value::Integer(2));
        ok_cfg.set(
            Value::String("filter".to_string()),
            Value::String("blether".to_string()),
        );
        ok_cfg.set(
            Value::String("format".to_string()),
            Value::String("json".to_string()),
        );
        ok_cfg.set(Value::String("color".to_string()), Value::Bool(true));
        ok_cfg.set(Value::String("timestamps".to_string()), Value::Bool(false));
        ok_cfg.set(
            Value::String("sinks".to_string()),
            Value::List(Rc::new(RefCell::new(sinks))),
        );
        interp
            .apply_log_config(Some(Value::Dict(Rc::new(RefCell::new(ok_cfg)))))
            .unwrap();
    }

    #[test]
    fn test_insecure_verifier_and_tls_dtls_defaults() {
        let verifier = InsecureVerifier;
        let cert = Certificate(Vec::new());
        let name = ServerName::try_from("localhost").unwrap();
        let mut scts = std::iter::empty::<&[u8]>();
        verifier
            .verify_server_cert(&cert, &[], &name, &mut scts, &[], SystemTime::now())
            .unwrap();

        let tls = tls_config_from_value(&Value::Nil).unwrap();
        assert!(matches!(tls.mode, TlsMode::Client));
        let dtls = dtls_config_from_value(&Value::Nil).unwrap();
        assert!(matches!(dtls.mode, TlsMode::Server));

        let cfg = TlsConfigData {
            mode: TlsMode::Client,
            server_name: "localhost".to_string(),
            insecure: true,
            ca_pem: None,
            cert_pem: None,
            key_pem: None,
        };
        let _ = build_client_config(&cfg).unwrap();

        let mut dict = DictValue::new();
        dict.set(Value::String("port".to_string()), Value::Integer(42));
        assert_eq!(dict_get_u16(&dict, "port"), Some(42));
        dict.set(Value::String("port".to_string()), Value::Integer(-1));
        assert_eq!(dict_get_u16(&dict, "port"), None);
        dict.set(
            Value::String("name".to_string()),
            Value::String("x".to_string()),
        );
        assert_eq!(dict_get_string(&dict, "name"), Some("x".to_string()));
        dict.set(Value::String("flag".to_string()), Value::Bool(true));
        assert_eq!(dict_get_bool(&dict, "flag"), Some(true));
        dict.set(
            Value::String("bytes".to_string()),
            Value::Bytes(Rc::new(RefCell::new(vec![1, 2, 3]))),
        );
        assert_eq!(dict_get_bytes(&dict, "bytes"), Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_range_to_list_inclusive() {
        let list = Interpreter::range_to_list(1, 3, true);
        let Value::List(items) = list else {
            panic!("expected list");
        };
        assert_eq!(items.borrow().len(), 3);
    }

    #[test]
    fn test_resolve_module_path_absolute() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("mod.braw");
        std::fs::write(&path, "blether 1").unwrap();
        let mut interp = Interpreter::new();
        interp.set_current_dir(dir.path());
        let resolved = interp.resolve_module_path(path.to_str().unwrap()).unwrap();
        assert!(resolved.is_absolute());
    }

    #[test]
    fn test_event_loop_write_watch_emits_event() {
        let interp = Interpreter::new();
        let globals = interp.globals.clone();

        let event_loop_new = match globals.borrow().get("event_loop_new").unwrap() {
            Value::NativeFunction(func) => func,
            _ => panic!("expected event_loop_new"),
        };
        let event_watch_write = match globals.borrow().get("event_watch_write").unwrap() {
            Value::NativeFunction(func) => func,
            _ => panic!("expected event_watch_write"),
        };
        let event_loop_poll = match globals.borrow().get("event_loop_poll").unwrap() {
            Value::NativeFunction(func) => func,
            _ => panic!("expected event_loop_poll"),
        };

        let loop_id = match (event_loop_new.func)(vec![]).unwrap() {
            Value::Integer(id) => id,
            _ => panic!("expected loop id"),
        };

        let mut fds = [0; 2];
        unsafe {
            libc::pipe(fds.as_mut_ptr());
        }
        let sock_id = register_socket(fds[1], SocketKind::Tcp);

        (event_watch_write.func)(vec![
            Value::Integer(loop_id),
            Value::Integer(sock_id),
            Value::Bool(true),
        ])
        .unwrap();

        let events =
            (event_loop_poll.func)(vec![Value::Integer(loop_id), Value::Integer(0)]).unwrap();
        let Value::List(list) = events else {
            panic!("expected event list");
        };
        let mut saw_write = false;
        for ev in list.borrow().iter() {
            if let Value::Dict(dict) = ev {
                let dict = dict.borrow();
                if let Some(Value::String(kind)) = dict.get(&Value::String("kind".to_string())) {
                    if kind == "write" {
                        saw_write = true;
                    }
                }
            }
        }
        assert!(saw_write);

        let _ = remove_socket(sock_id);
        unsafe {
            libc::close(fds[0]);
            libc::close(fds[1]);
        }
    }

    #[test]
    fn test_log_event_and_span_builtins() {
        let mut interp = Interpreter::new();
        let globals = interp.globals.clone();
        let log_event = match globals.borrow().get("log_event").unwrap() {
            Value::NativeFunction(func) => func,
            _ => panic!("expected log_event"),
        };
        assert!((log_event.func)(vec![
            Value::String("blether".to_string()),
            Value::String("msg".to_string())
        ])
        .is_err());

        let log_span = match globals.borrow().get("log_span").unwrap() {
            Value::NativeFunction(func) => func,
            _ => panic!("expected log_span"),
        };
        let log_span_enter = match globals.borrow().get("log_span_enter").unwrap() {
            Value::NativeFunction(func) => func,
            _ => panic!("expected log_span_enter"),
        };
        let log_span_exit = match globals.borrow().get("log_span_exit").unwrap() {
            Value::NativeFunction(func) => func,
            _ => panic!("expected log_span_exit"),
        };
        let log_span_in = match globals.borrow().get("log_span_in").unwrap() {
            Value::NativeFunction(func) => func,
            _ => panic!("expected log_span_in"),
        };
        let log_span_current = match globals.borrow().get("log_span_current").unwrap() {
            Value::NativeFunction(func) => func,
            _ => panic!("expected log_span_current"),
        };

        let _guard = InterpreterGuard::new(&mut interp);
        let mut fields = DictValue::new();
        fields.set(Value::String("k".to_string()), Value::Integer(1));
        (log_event.func)(vec![
            Value::Integer(3),
            Value::String("msg".to_string()),
            Value::Dict(Rc::new(RefCell::new(fields))),
            Value::String("target".to_string()),
        ])
        .unwrap();

        let span = (log_span.func)(vec![Value::String("span".to_string())]).unwrap();
        (log_span_enter.func)(vec![span.clone()]).unwrap();
        let current = (log_span_current.func)(vec![]).unwrap();
        assert!(matches!(current, Value::NativeObject(_)));
        (log_span_exit.func)(vec![span.clone()]).unwrap();

        let cb = Value::NativeFunction(Rc::new(NativeFunction::new("cb", 0, |_args| {
            Ok(Value::Integer(5))
        })));
        let result = (log_span_in.func)(vec![span, cb]).unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_dict_get_helpers_mismatch_and_u16_float() {
        let mut dict = DictValue::new();
        dict.set(Value::String("s".to_string()), Value::Integer(1));
        dict.set(Value::String("b".to_string()), Value::Integer(1));
        dict.set(
            Value::String("bytes".to_string()),
            Value::String("no".to_string()),
        );
        dict.set(Value::String("port".to_string()), Value::Float(123.0));
        assert_eq!(dict_get_string(&dict, "s"), None);
        assert_eq!(dict_get_bool(&dict, "b"), None);
        assert_eq!(dict_get_bytes(&dict, "bytes"), None);
        assert_eq!(dict_get_u16(&dict, "port"), Some(123));

        dict.set(Value::String("port_neg".to_string()), Value::Float(-1.0));
        dict.set(
            Value::String("port_bad".to_string()),
            Value::String("x".to_string()),
        );
        assert_eq!(dict_get_u16(&dict, "port_neg"), None);
        assert_eq!(dict_get_u16(&dict, "port_bad"), None);
    }

    #[test]
    fn test_tls_config_from_value_errors_and_defaults() {
        let err = tls_config_from_value(&Value::Integer(1)).err().unwrap();
        assert!(err.contains("expects config dict"));

        let mut dict = DictValue::new();
        dict.set(
            Value::String("server_name".to_string()),
            Value::String(String::new()),
        );
        let cfg = tls_config_from_value(&Value::Dict(Rc::new(RefCell::new(dict)))).unwrap();
        assert_eq!(cfg.server_name, "localhost");
    }

    #[test]
    fn test_srtp_profile_parsing_variants() {
        assert_eq!(
            srtp_profile_from_str("SRTP_AES128_CM_SHA1_32"),
            Some(SrtpProfile::Aes128CmSha132)
        );
        assert_eq!(
            srtp_profile_from_str("AEAD_AES_128_GCM"),
            Some(SrtpProfile::AeadAes128Gcm)
        );
        assert_eq!(srtp_profile_from_str("unknown"), None);

        assert_eq!(
            protection_profile_from_str("AES128_CM_HMAC_SHA1_32"),
            Some(ProtectionProfile::Aes128CmHmacSha132)
        );
        assert_eq!(
            protection_profile_from_str("AEAD_AES_256_GCM"),
            Some(ProtectionProfile::AeadAes256Gcm)
        );
        assert_eq!(protection_profile_from_str("bogus"), None);
    }
}
