use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::io::{Read, Write};
use std::os::raw::c_char;
use std::os::unix::io::FromRawFd;
use std::sync::{Arc, Mutex, OnceLock};

use regex::Regex;
use serde_json::Value as JsonValue;
use rustls::client::{ServerCertVerified, ServerCertVerifier};
use rustls::{Certificate, ClientConfig, ClientConnection, PrivateKey, RootCertStore, ServerConfig, ServerConnection, ServerName, StreamOwned, OwnedTrustAnchor};
use rustls_pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use libsrtp::{MasterKey, ProtectionProfile, RecvSession, SendSession, StreamConfig};
use openssl::pkcs12::Pkcs12;
use openssl::pkey::PKey;
use openssl::x509::X509;
use udp_dtls::{DtlsAcceptor, DtlsAcceptorBuilder, DtlsConnector, DtlsConnectorBuilder, Identity, SrtpProfile, UdpChannel};
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::proto::rr::{RData, RecordType};
use trust_dns_resolver::Resolver;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MdhValue {
    pub tag: u8,
    pub data: i64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MdhRsResult {
    pub ok: u8,
    pub value: MdhValue,
    pub error: MdhValue,
}

#[repr(C)]
struct MdhList {
    items: *mut MdhValue,
    length: i64,
    capacity: i64,
}

#[repr(C)]
struct MdhBytes {
    data: *mut u8,
    length: i64,
    capacity: i64,
}

const MDH_TAG_NIL: u8 = 0;
const MDH_TAG_BOOL: u8 = 1;
const MDH_TAG_INT: u8 = 2;
const MDH_TAG_FLOAT: u8 = 3;
const MDH_TAG_STRING: u8 = 4;
const MDH_TAG_LIST: u8 = 5;
const MDH_TAG_DICT: u8 = 6;
const MDH_TAG_FUNCTION: u8 = 7;
const MDH_TAG_SET: u8 = 11;
const MDH_TAG_CLOSURE: u8 = 12;
const MDH_TAG_BYTES: u8 = 13;

extern "C" {
    fn __mdh_make_nil() -> MdhValue;
    fn __mdh_make_bool(value: bool) -> MdhValue;
    fn __mdh_make_int(value: i64) -> MdhValue;
    fn __mdh_make_float(value: f64) -> MdhValue;
    fn __mdh_make_string(value: *const c_char) -> MdhValue;
    fn __mdh_make_list(capacity: i32) -> MdhValue;
    fn __mdh_list_push(list: MdhValue, value: MdhValue);
    fn __mdh_bytes_new(size: MdhValue) -> MdhValue;
    fn __mdh_empty_dict() -> MdhValue;
    fn __mdh_dict_set(dict: MdhValue, key: MdhValue, value: MdhValue) -> MdhValue;
    fn __mdh_dict_get_default(dict: MdhValue, key: MdhValue, default_val: MdhValue) -> MdhValue;
    fn __mdh_to_string(value: MdhValue) -> MdhValue;
}

fn cstring_lossy(s: &str) -> CString {
    CString::new(s).unwrap_or_else(|_| CString::new("Runtime error").expect("CString literal"))
}

unsafe fn mdh_ok(value: MdhValue) -> MdhRsResult {
    MdhRsResult {
        ok: 1,
        value,
        error: __mdh_make_nil(),
    }
}

unsafe fn mdh_err(msg: &str) -> MdhRsResult {
    MdhRsResult {
        ok: 0,
        value: __mdh_make_nil(),
        error: mdh_make_string_from_rust(msg),
    }
}

unsafe fn mdh_string_to_rust(value: MdhValue) -> String {
    if value.tag != MDH_TAG_STRING || value.data == 0 {
        return String::new();
    }
    let ptr = value.data as *const c_char;
    if ptr.is_null() {
        return String::new();
    }
    CStr::from_ptr(ptr).to_string_lossy().into_owned()
}

unsafe fn mdh_make_string_from_rust(s: &str) -> MdhValue {
    let cstr = cstring_lossy(s);
    __mdh_make_string(cstr.as_ptr())
}

unsafe fn mdh_value_to_string(value: MdhValue) -> String {
    let s_val = __mdh_to_string(value);
    mdh_string_to_rust(s_val)
}

unsafe fn mdh_float_value(value: MdhValue) -> f64 {
    f64::from_bits(value.data as u64)
}

unsafe fn mdh_bytes_to_vec(value: MdhValue) -> Option<Vec<u8>> {
    if value.tag != MDH_TAG_BYTES || value.data == 0 {
        return None;
    }
    let ptr = value.data as *const MdhBytes;
    if ptr.is_null() {
        return None;
    }
    let len = (*ptr).length.max(0) as usize;
    let data = (*ptr).data;
    if data.is_null() || len == 0 {
        return Some(Vec::new());
    }
    Some(std::slice::from_raw_parts(data, len).to_vec())
}

unsafe fn mdh_make_bytes_from_vec(data: &[u8]) -> MdhValue {
    let bytes_val = __mdh_bytes_new(__mdh_make_int(data.len() as i64));
    let ptr = bytes_val.data as *mut MdhBytes;
    if !ptr.is_null() && !data.is_empty() {
        std::ptr::copy_nonoverlapping(data.as_ptr(), (*ptr).data, data.len());
        (*ptr).length = data.len() as i64;
    }
    bytes_val
}

unsafe fn mdh_dict_get_string(dict: MdhValue, key: &str) -> Option<String> {
    let val = __mdh_dict_get_default(dict, mdh_make_string_from_rust(key), __mdh_make_nil());
    if val.tag == MDH_TAG_STRING {
        let s = mdh_string_to_rust(val);
        if s.is_empty() { None } else { Some(s) }
    } else {
        None
    }
}

unsafe fn mdh_dict_get_bool(dict: MdhValue, key: &str) -> Option<bool> {
    let val = __mdh_dict_get_default(dict, mdh_make_string_from_rust(key), __mdh_make_nil());
    if val.tag == MDH_TAG_BOOL {
        Some(val.data != 0)
    } else {
        None
    }
}

unsafe fn mdh_dict_get_bytes(dict: MdhValue, key: &str) -> Option<Vec<u8>> {
    let val = __mdh_dict_get_default(dict, mdh_make_string_from_rust(key), __mdh_make_nil());
    mdh_bytes_to_vec(val)
}

unsafe fn mdh_dict_get_u16(dict: MdhValue, key: &str) -> Option<u16> {
    let val = __mdh_dict_get_default(dict, mdh_make_string_from_rust(key), __mdh_make_nil());
    if val.tag == MDH_TAG_INT {
        if val.data >= 0 && val.data <= u16::MAX as i64 {
            Some(val.data as u16)
        } else {
            None
        }
    } else if val.tag == MDH_TAG_FLOAT {
        let v = mdh_float_value(val) as i64;
        if v >= 0 && v <= u16::MAX as i64 {
            Some(v as u16)
        } else {
            None
        }
    } else {
        None
    }
}

fn make_resolver() -> Result<Resolver, String> {
    Resolver::from_system_conf()
        .or_else(|_| Resolver::new(ResolverConfig::default(), ResolverOpts::default()))
        .map_err(|e| format!("DNS resolver init failed: {}", e))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TlsMode {
    Client,
    Server,
}

struct TlsConfigData {
    mode: TlsMode,
    server_name: String,
    insecure: bool,
    ca_pem: Option<String>,
    cert_pem: Option<String>,
    key_pem: Option<String>,
}

enum TlsStream {
    Client(StreamOwned<ClientConnection, std::net::TcpStream>),
    Server(StreamOwned<ServerConnection, std::net::TcpStream>),
}

struct TlsSession {
    mode: TlsMode,
    server_name: String,
    client_config: Option<Arc<ClientConfig>>,
    server_config: Option<Arc<ServerConfig>>,
    stream: Option<TlsStream>,
}

struct TlsRegistry {
    next_id: i64,
    sessions: HashMap<i64, TlsSession>,
}

static TLS_REGISTRY: OnceLock<Mutex<TlsRegistry>> = OnceLock::new();

fn tls_registry() -> &'static Mutex<TlsRegistry> {
    TLS_REGISTRY.get_or_init(|| Mutex::new(TlsRegistry {
        next_id: 1,
        sessions: HashMap::new(),
    }))
}

fn tls_register(session: TlsSession) -> i64 {
    let mut reg = tls_registry().lock().unwrap();
    let id = reg.next_id;
    reg.next_id += 1;
    reg.sessions.insert(id, session);
    id
}

fn tls_with_mut<T, F>(id: i64, f: F) -> Result<T, String>
where
    F: FnOnce(&mut TlsSession) -> Result<T, String>,
{
    let mut reg = tls_registry().lock().unwrap();
    let session = reg.sessions.get_mut(&id).ok_or("Unknown TLS handle")?;
    f(session)
}

fn tls_remove(id: i64) {
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
    SRTP_REGISTRY.get_or_init(|| Mutex::new(SrtpRegistry {
        next_id: 1,
        sessions: HashMap::new(),
    }))
}

fn srtp_register(session: SrtpSession) -> i64 {
    let mut reg = srtp_registry().lock().unwrap();
    let id = reg.next_id;
    reg.next_id += 1;
    reg.sessions.insert(id, session);
    id
}

fn srtp_with_mut<T, F>(id: i64, f: F) -> Result<T, String>
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
    DTLS_REGISTRY.get_or_init(|| Mutex::new(DtlsRegistry {
        next_id: 1,
        configs: HashMap::new(),
    }))
}

fn dtls_register(config: DtlsConfigData) -> i64 {
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

fn tls_config_from_value(config: MdhValue) -> Result<TlsConfigData, String> {
    unsafe {
        if config.tag == MDH_TAG_NIL {
            return Ok(TlsConfigData {
                mode: TlsMode::Client,
                server_name: "localhost".to_string(),
                insecure: false,
                ca_pem: None,
                cert_pem: None,
                key_pem: None,
            });
        }
        if config.tag != MDH_TAG_DICT {
            return Err("tls_client_new expects a config dict".to_string());
        }

        let mode_val = __mdh_dict_get_default(
            config,
            mdh_make_string_from_rust("mode"),
            __mdh_make_nil(),
        );
        let mode = if mode_val.tag == MDH_TAG_STRING {
            let m = mdh_string_to_rust(mode_val).to_lowercase();
            if m == "server" {
                TlsMode::Server
            } else {
                TlsMode::Client
            }
        } else {
            TlsMode::Client
        };

        let server_name_val = __mdh_dict_get_default(
            config,
            mdh_make_string_from_rust("server_name"),
            __mdh_make_nil(),
        );
        let server_name = if server_name_val.tag == MDH_TAG_STRING {
            let s = mdh_string_to_rust(server_name_val);
            if s.is_empty() { "localhost".to_string() } else { s }
        } else {
            "localhost".to_string()
        };

        let insecure_val = __mdh_dict_get_default(
            config,
            mdh_make_string_from_rust("insecure"),
            __mdh_make_nil(),
        );
        let insecure = insecure_val.tag == MDH_TAG_BOOL && insecure_val.data != 0;

        let ca_pem_val = __mdh_dict_get_default(
            config,
            mdh_make_string_from_rust("ca_pem"),
            __mdh_make_nil(),
        );
        let ca_pem = if ca_pem_val.tag == MDH_TAG_STRING {
            let s = mdh_string_to_rust(ca_pem_val);
            if s.is_empty() { None } else { Some(s) }
        } else {
            None
        };

        let cert_pem_val = __mdh_dict_get_default(
            config,
            mdh_make_string_from_rust("cert_pem"),
            __mdh_make_nil(),
        );
        let cert_pem = if cert_pem_val.tag == MDH_TAG_STRING {
            let s = mdh_string_to_rust(cert_pem_val);
            if s.is_empty() { None } else { Some(s) }
        } else {
            None
        };

        let key_pem_val = __mdh_dict_get_default(
            config,
            mdh_make_string_from_rust("key_pem"),
            __mdh_make_nil(),
        );
        let key_pem = if key_pem_val.tag == MDH_TAG_STRING {
            let s = mdh_string_to_rust(key_pem_val);
            if s.is_empty() { None } else { Some(s) }
        } else {
            None
        };

        Ok(TlsConfigData {
            mode,
            server_name,
            insecure,
            ca_pem,
            cert_pem,
            key_pem,
        })
    }
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
    let cert_pem = cfg
        .cert_pem
        .as_ref()
        .ok_or("Server cert_pem is required")?;
    let key_pem = cfg.key_pem.as_ref().ok_or("Server key_pem is required")?;

    let mut cert_reader = std::io::Cursor::new(cert_pem.as_bytes());
    let certs = certs(&mut cert_reader).map_err(|e| format!("Invalid server cert: {}", e))?;
    let certs = certs.into_iter().map(Certificate).collect::<Vec<_>>();

    let mut key_reader = std::io::Cursor::new(key_pem.as_bytes());
    let mut keys = pkcs8_private_keys(&mut key_reader)
        .map_err(|e| format!("Invalid server key: {}", e))?;
    if keys.is_empty() {
        let mut key_reader = std::io::Cursor::new(key_pem.as_bytes());
        keys = rsa_private_keys(&mut key_reader).map_err(|e| format!("Invalid server key: {}", e))?;
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

fn dtls_config_from_value(config: MdhValue) -> Result<DtlsConfigData, String> {
    unsafe {
        if config.tag == MDH_TAG_NIL {
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
        if config.tag != MDH_TAG_DICT {
            return Err("dtls_server_new expects a config dict".to_string());
        }

        let mode_str = mdh_dict_get_string(config, "mode").unwrap_or_else(|| "server".to_string());
        let mode = if mode_str.to_lowercase() == "client" {
            TlsMode::Client
        } else {
            TlsMode::Server
        };

        let server_name = mdh_dict_get_string(config, "server_name")
            .unwrap_or_else(|| "localhost".to_string());
        let insecure = mdh_dict_get_bool(config, "insecure").unwrap_or(false);

        let ca_pem = mdh_dict_get_string(config, "ca_pem");
        let cert_pem = mdh_dict_get_string(config, "cert_pem");
        let key_pem = mdh_dict_get_string(config, "key_pem");

        let remote_host = mdh_dict_get_string(config, "remote_host");
        let remote_port = mdh_dict_get_u16(config, "remote_port");

        let profiles_val = __mdh_dict_get_default(
            config,
            mdh_make_string_from_rust("srtp_profiles"),
            __mdh_make_nil(),
        );
        let mut profiles = Vec::new();
        if profiles_val.tag == MDH_TAG_LIST {
            let list_ptr = profiles_val.data as *const MdhList;
            if !list_ptr.is_null() {
                let list = &*list_ptr;
                let items = std::slice::from_raw_parts(list.items, list.length as usize);
                for item in items {
                    if item.tag == MDH_TAG_STRING {
                        let s = mdh_string_to_rust(*item);
                        if let Some(profile) = srtp_profile_from_str(&s) {
                            profiles.push(profile);
                        }
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
}

fn lenient_json_for_serde_json(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_string = false;
    let mut escape = false;

    for ch in input.chars() {
        if !in_string {
            if ch == '"' {
                in_string = true;
            }
            out.push(ch);
            continue;
        }

        if escape {
            match ch {
                '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' | 'u' => {
                    out.push('\\');
                    out.push(ch);
                }
                other => {
                    // Interpreter compatibility: unknown escapes become the literal char.
                    out.push(other);
                }
            }
            escape = false;
            continue;
        }

        match ch {
            '\\' => {
                escape = true;
            }
            '"' => {
                in_string = false;
                out.push('"');
            }
            other => out.push(other),
        }
    }

    if escape {
        // Trailing backslash - keep it so serde_json reports an error.
        out.push('\\');
    }

    out
}

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

unsafe fn json_fallback_string(value: MdhValue) -> String {
    let s = mdh_value_to_string(value);
    let escaped = s.replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

unsafe fn mdh_value_to_json(value: MdhValue, pretty: bool, indent: usize) -> String {
    let ws = "  ".repeat(indent);
    let ws_inner = "  ".repeat(indent + 1);

    match value.tag {
        MDH_TAG_NIL => "null".to_string(),
        MDH_TAG_BOOL => {
            if value.data != 0 {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        MDH_TAG_INT => value.data.to_string(),
        MDH_TAG_FLOAT => {
            let f = mdh_float_value(value);
            if f.is_nan() || f.is_infinite() {
                "null".to_string()
            } else {
                f.to_string()
            }
        }
        MDH_TAG_STRING => {
            let s = mdh_string_to_rust(value);
            json_escape_string(&s)
        }
        MDH_TAG_LIST => {
            if value.data == 0 {
                return "[]".to_string();
            }
            let list_ptr = value.data as *const MdhList;
            if list_ptr.is_null() {
                return "[]".to_string();
            }
            let list = &*list_ptr;
            let len = list.length.max(0) as usize;
            if len == 0 {
                return "[]".to_string();
            }
            let items = if list.items.is_null() {
                &[] as &[MdhValue]
            } else {
                std::slice::from_raw_parts(list.items, len)
            };
            if pretty {
                let parts: Vec<String> = items
                    .iter()
                    .map(|v| format!("{}{}", ws_inner, mdh_value_to_json(*v, true, indent + 1)))
                    .collect();
                format!("[\n{}\n{}]", parts.join(",\n"), ws)
            } else {
                let parts: Vec<String> = items
                    .iter()
                    .map(|v| mdh_value_to_json(*v, false, indent))
                    .collect();
                format!("[{}]", parts.join(", "))
            }
        }
        MDH_TAG_SET | MDH_TAG_FUNCTION | MDH_TAG_CLOSURE | MDH_TAG_BYTES => {
            json_fallback_string(value)
        }
        MDH_TAG_DICT => {
            if value.data == 0 {
                return "{}".to_string();
            }
            let dict_ptr = value.data as *const i64;
            if dict_ptr.is_null() {
                return "{}".to_string();
            }
            let count = *dict_ptr;
            if count <= 0 {
                return "{}".to_string();
            }
            let entries_ptr = dict_ptr.add(1) as *const MdhValue;
            let entries = std::slice::from_raw_parts(entries_ptr, (count * 2) as usize);
            let mut parts = Vec::with_capacity(count as usize);
            for i in 0..count {
                let key_val = entries[(i * 2) as usize];
                let val = entries[(i * 2 + 1) as usize];
                let key_str = if key_val.tag == MDH_TAG_STRING {
                    mdh_string_to_rust(key_val)
                } else {
                    mdh_value_to_string(key_val)
                };
                let key_json = json_escape_string(&key_str);
                let val_json = if pretty {
                    mdh_value_to_json(val, true, indent + 1)
                } else {
                    mdh_value_to_json(val, false, indent)
                };
                if pretty {
                    parts.push(format!("{}{}: {}", ws_inner, key_json, val_json));
                } else {
                    parts.push(format!("{}: {}", key_json, val_json));
                }
            }
            if pretty {
                format!("{{\n{}\n{}}}", parts.join(",\n"), ws)
            } else {
                format!("{{{}}}", parts.join(", "))
            }
        }
        _ => json_fallback_string(value),
    }
}

fn json_to_mdh(value: &JsonValue) -> Result<MdhValue, String> {
    unsafe {
        match value {
            JsonValue::Null => Ok(__mdh_make_nil()),
            JsonValue::Bool(b) => Ok(__mdh_make_bool(*b)),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    return Ok(__mdh_make_int(i));
                }
                if let Some(u) = n.as_u64() {
                    if u <= i64::MAX as u64 {
                        return Ok(__mdh_make_int(u as i64));
                    }
                    return Err("Integer out of range".to_string());
                }
                if let Some(f) = n.as_f64() {
                    return Ok(__mdh_make_float(f));
                }
                Err("Invalid number".to_string())
            }
            JsonValue::String(s) => Ok(mdh_make_string_from_rust(s)),
            JsonValue::Array(items) => {
                let list = __mdh_make_list(items.len() as i32);
                for item in items {
                    let v = json_to_mdh(item)?;
                    __mdh_list_push(list, v);
                }
                Ok(list)
            }
            JsonValue::Object(map) => {
                let mut dict = __mdh_empty_dict();
                for (k, v) in map.iter() {
                    let key = mdh_make_string_from_rust(k);
                    let val = json_to_mdh(v)?;
                    dict = __mdh_dict_set(dict, key, val);
                }
                Ok(dict)
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_json_parse(json_str: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        if json_str.tag != MDH_TAG_STRING {
            return mdh_err("json_parse expects a string");
        }
        let text = mdh_string_to_rust(json_str);
        let text = lenient_json_for_serde_json(&text);
        let parsed: JsonValue = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => return mdh_err(&format!("Invalid JSON: {}", e)),
        };
        match json_to_mdh(&parsed) {
            Ok(v) => mdh_ok(v),
            Err(e) => mdh_err(&format!("Invalid JSON: {}", e)),
        }
    }) {
        Ok(result) => result,
        Err(_) => unsafe { mdh_err("Rust panic in json_parse") },
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_json_stringify(value: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        let s = mdh_value_to_json(value, false, 0);
        mdh_ok(mdh_make_string_from_rust(&s))
    }) {
        Ok(result) => result,
        Err(_) => unsafe { mdh_err("Rust panic in json_stringify") },
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_json_pretty(value: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        let s = mdh_value_to_json(value, true, 0);
        mdh_ok(mdh_make_string_from_rust(&s))
    }) {
        Ok(result) => result,
        Err(_) => unsafe { mdh_err("Rust panic in json_pretty") },
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_regex_test(text: MdhValue, pattern: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        if text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING {
            return mdh_err("regex_test expects strings");
        }

        let text_s = mdh_string_to_rust(text);
        let pat_s = mdh_string_to_rust(pattern);
        let re = match Regex::new(&pat_s) {
            Ok(r) => r,
            Err(e) => return mdh_err(&format!("Invalid regex '{}': {}", pat_s, e)),
        };

        mdh_ok(__mdh_make_bool(re.is_match(&text_s)))
    }) {
        Ok(result) => result,
        Err(_) => unsafe { mdh_err("Rust panic in regex_test") },
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_regex_match(text: MdhValue, pattern: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        if text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING {
            return mdh_err("regex_match expects strings");
        }

        let text_s = mdh_string_to_rust(text);
        let pat_s = mdh_string_to_rust(pattern);
        let re = match Regex::new(&pat_s) {
            Ok(r) => r,
            Err(e) => return mdh_err(&format!("Invalid regex '{}': {}", pat_s, e)),
        };

        if let Some(m) = re.find(&text_s) {
            let mut dict = __mdh_empty_dict();
            dict = __mdh_dict_set(
                dict,
                mdh_make_string_from_rust("match"),
                mdh_make_string_from_rust(m.as_str()),
            );
            dict = __mdh_dict_set(
                dict,
                mdh_make_string_from_rust("start"),
                __mdh_make_int(m.start() as i64),
            );
            dict = __mdh_dict_set(
                dict,
                mdh_make_string_from_rust("end"),
                __mdh_make_int(m.end() as i64),
            );
            mdh_ok(dict)
        } else {
            mdh_ok(__mdh_make_nil())
        }
    }) {
        Ok(result) => result,
        Err(_) => unsafe { mdh_err("Rust panic in regex_match") },
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_regex_match_all(text: MdhValue, pattern: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        if text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING {
            return mdh_err("regex_match_all expects strings");
        }

        let text_s = mdh_string_to_rust(text);
        let pat_s = mdh_string_to_rust(pattern);
        let re = match Regex::new(&pat_s) {
            Ok(r) => r,
            Err(e) => return mdh_err(&format!("Invalid regex '{}': {}", pat_s, e)),
        };

        let result = __mdh_make_list(8);
        for m in re.find_iter(&text_s) {
            let mut dict = __mdh_empty_dict();
            dict = __mdh_dict_set(
                dict,
                mdh_make_string_from_rust("match"),
                mdh_make_string_from_rust(m.as_str()),
            );
            dict = __mdh_dict_set(
                dict,
                mdh_make_string_from_rust("start"),
                __mdh_make_int(m.start() as i64),
            );
            dict = __mdh_dict_set(
                dict,
                mdh_make_string_from_rust("end"),
                __mdh_make_int(m.end() as i64),
            );
            __mdh_list_push(result, dict);
        }
        mdh_ok(result)
    }) {
        Ok(result) => result,
        Err(_) => unsafe { mdh_err("Rust panic in regex_match_all") },
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_regex_replace(
    text: MdhValue,
    pattern: MdhValue,
    replacement: MdhValue,
) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        if text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING || replacement.tag != MDH_TAG_STRING {
            return mdh_err("regex_replace expects strings");
        }

        let text_s = mdh_string_to_rust(text);
        let pat_s = mdh_string_to_rust(pattern);
        let repl_s = mdh_string_to_rust(replacement);
        let re = match Regex::new(&pat_s) {
            Ok(r) => r,
            Err(e) => return mdh_err(&format!("Invalid regex '{}': {}", pat_s, e)),
        };

        let replaced = re.replace_all(&text_s, repl_s.as_str()).to_string();
        mdh_ok(mdh_make_string_from_rust(&replaced))
    }) {
        Ok(result) => result,
        Err(_) => unsafe { mdh_err("Rust panic in regex_replace") },
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_regex_replace_first(
    text: MdhValue,
    pattern: MdhValue,
    replacement: MdhValue,
) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        if text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING || replacement.tag != MDH_TAG_STRING {
            return mdh_err("regex_replace_first expects strings");
        }

        let text_s = mdh_string_to_rust(text);
        let pat_s = mdh_string_to_rust(pattern);
        let repl_s = mdh_string_to_rust(replacement);
        let re = match Regex::new(&pat_s) {
            Ok(r) => r,
            Err(e) => return mdh_err(&format!("Invalid regex '{}': {}", pat_s, e)),
        };

        let replaced = re.replacen(&text_s, 1, repl_s.as_str()).to_string();
        mdh_ok(mdh_make_string_from_rust(&replaced))
    }) {
        Ok(result) => result,
        Err(_) => unsafe { mdh_err("Rust panic in regex_replace_first") },
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_regex_split(text: MdhValue, pattern: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        if text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING {
            return mdh_err("regex_split expects strings");
        }

        let text_s = mdh_string_to_rust(text);
        let pat_s = mdh_string_to_rust(pattern);
        let re = match Regex::new(&pat_s) {
            Ok(r) => r,
            Err(e) => return mdh_err(&format!("Invalid regex '{}': {}", pat_s, e)),
        };

        let result = __mdh_make_list(8);
        for part in re.split(&text_s) {
            __mdh_list_push(result, mdh_make_string_from_rust(part));
        }
        mdh_ok(result)
    }) {
        Ok(result) => result,
        Err(_) => unsafe { mdh_err("Rust panic in regex_split") },
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_dns_srv(service: MdhValue, domain: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        if service.tag != MDH_TAG_STRING || domain.tag != MDH_TAG_STRING {
            return mdh_err("dns_srv expects strings");
        }
        let service_s = mdh_string_to_rust(service);
        let domain_s = mdh_string_to_rust(domain);
        let name = if service_s.is_empty() {
            domain_s.clone()
        } else {
            let s = service_s.trim_end_matches('.');
            let d = domain_s.trim_start_matches('.');
            format!("{}.{}", s, d)
        };
        let resolver = match make_resolver() {
            Ok(r) => r,
            Err(e) => return mdh_err(&e),
        };
        let lookup = match resolver.lookup(name.as_str(), RecordType::SRV) {
            Ok(l) => l,
            Err(e) => return mdh_err(&format!("DNS SRV lookup failed: {}", e)),
        };
        let list = __mdh_make_list(8);
        for rdata in lookup.iter() {
            if let RData::SRV(srv) = rdata {
                let mut dict = __mdh_empty_dict();
                dict = __mdh_dict_set(
                    dict,
                    mdh_make_string_from_rust("priority"),
                    __mdh_make_int(srv.priority() as i64),
                );
                dict = __mdh_dict_set(
                    dict,
                    mdh_make_string_from_rust("weight"),
                    __mdh_make_int(srv.weight() as i64),
                );
                dict = __mdh_dict_set(
                    dict,
                    mdh_make_string_from_rust("port"),
                    __mdh_make_int(srv.port() as i64),
                );
                dict = __mdh_dict_set(
                    dict,
                    mdh_make_string_from_rust("target"),
                    mdh_make_string_from_rust(&srv.target().to_string()),
                );
                __mdh_list_push(list, dict);
            }
        }
        mdh_ok(list)
    }) {
        Ok(result) => result,
        Err(_) => unsafe { mdh_err("Rust panic in dns_srv") },
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_dns_naptr(domain: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        if domain.tag != MDH_TAG_STRING {
            return mdh_err("dns_naptr expects string");
        }
        let domain_s = mdh_string_to_rust(domain);
        let resolver = match make_resolver() {
            Ok(r) => r,
            Err(e) => return mdh_err(&e),
        };
        let lookup = match resolver.lookup(domain_s.as_str(), RecordType::NAPTR) {
            Ok(l) => l,
            Err(e) => return mdh_err(&format!("DNS NAPTR lookup failed: {}", e)),
        };
        let list = __mdh_make_list(8);
        for rdata in lookup.iter() {
            if let RData::NAPTR(naptr) = rdata {
                let mut dict = __mdh_empty_dict();
                dict = __mdh_dict_set(
                    dict,
                    mdh_make_string_from_rust("order"),
                    __mdh_make_int(naptr.order() as i64),
                );
                dict = __mdh_dict_set(
                    dict,
                    mdh_make_string_from_rust("preference"),
                    __mdh_make_int(naptr.preference() as i64),
                );
                dict = __mdh_dict_set(
                    dict,
                    mdh_make_string_from_rust("flags"),
                    mdh_make_string_from_rust(String::from_utf8_lossy(naptr.flags()).as_ref()),
                );
                dict = __mdh_dict_set(
                    dict,
                    mdh_make_string_from_rust("service"),
                    mdh_make_string_from_rust(String::from_utf8_lossy(naptr.services()).as_ref()),
                );
                dict = __mdh_dict_set(
                    dict,
                    mdh_make_string_from_rust("regexp"),
                    mdh_make_string_from_rust(String::from_utf8_lossy(naptr.regexp()).as_ref()),
                );
                dict = __mdh_dict_set(
                    dict,
                    mdh_make_string_from_rust("replacement"),
                    mdh_make_string_from_rust(&naptr.replacement().to_string()),
                );
                __mdh_list_push(list, dict);
            }
        }
        mdh_ok(list)
    }) {
        Ok(result) => result,
        Err(_) => unsafe { mdh_err("Rust panic in dns_naptr") },
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_tls_client_new(config: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        let cfg = match tls_config_from_value(config) {
            Ok(cfg) => cfg,
            Err(e) => return mdh_err(&e),
        };

        let session = if cfg.mode == TlsMode::Client {
            let client_config = match build_client_config(&cfg) {
                Ok(c) => c,
                Err(e) => return mdh_err(&e),
            };
            TlsSession {
                mode: TlsMode::Client,
                server_name: cfg.server_name,
                client_config: Some(client_config),
                server_config: None,
                stream: None,
            }
        } else {
            let server_config = match build_server_config(&cfg) {
                Ok(c) => c,
                Err(e) => return mdh_err(&e),
            };
            TlsSession {
                mode: TlsMode::Server,
                server_name: cfg.server_name,
                client_config: None,
                server_config: Some(server_config),
                stream: None,
            }
        };

        let id = tls_register(session);
        mdh_ok(__mdh_make_int(id))
    }) {
        Ok(result) => result,
        Err(_) => unsafe { mdh_err("Rust panic in tls_client_new") },
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_tls_connect(tls: MdhValue, sock: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        let tls_id = tls.data;
        if tls.tag != MDH_TAG_INT || tls_id <= 0 {
            return mdh_err("tls_connect expects a TLS handle");
        }
        if sock.tag != MDH_TAG_INT {
            return mdh_err("tls_connect expects a socket fd");
        }
        let fd = sock.data as i32;

        let res = tls_with_mut(tls_id, |session| {
            if session.stream.is_some() {
                return Err("TLS session already connected".to_string());
            }
            let mut stream = std::net::TcpStream::from_raw_fd(fd);
            let _ = stream.set_nonblocking(false);

            match session.mode {
                TlsMode::Client => {
                    let config = session
                        .client_config
                        .as_ref()
                        .ok_or("Missing client config")?
                        .clone();
                    let server_name = ServerName::try_from(session.server_name.as_str())
                        .map_err(|_| "Invalid server_name")?;
                    let mut conn =
                        ClientConnection::new(config, server_name).map_err(|e| e.to_string())?;
                    while conn.is_handshaking() {
                        conn.complete_io(&mut stream)
                            .map_err(|e| format!("TLS handshake failed: {}", e))?;
                    }
                    session.stream = Some(TlsStream::Client(StreamOwned::new(conn, stream)));
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
                    session.stream = Some(TlsStream::Server(StreamOwned::new(conn, stream)));
                }
            }
            Ok(())
        });

        match res {
            Ok(_) => mdh_ok(__mdh_make_nil()),
            Err(e) => mdh_err(&e),
        }
    }) {
        Ok(result) => result,
        Err(_) => unsafe { mdh_err("Rust panic in tls_connect") },
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_tls_send(tls: MdhValue, buf: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        if tls.tag != MDH_TAG_INT || tls.data <= 0 {
            return mdh_err("tls_send expects a TLS handle");
        }
        if buf.tag != MDH_TAG_BYTES {
            return mdh_err("tls_send expects bytes");
        }
        let bytes = buf.data as *mut MdhBytes;
        if bytes.is_null() {
            return mdh_ok(__mdh_make_int(0));
        }
        let slice = std::slice::from_raw_parts((*bytes).data, (*bytes).length as usize);

        let res = tls_with_mut(tls.data, |session| {
            let stream = session.stream.as_mut().ok_or("TLS not connected")?;
            let n = match stream {
                TlsStream::Client(s) => s.write(slice),
                TlsStream::Server(s) => s.write(slice),
            }
            .map_err(|e| format!("TLS send failed: {}", e))?;
            match stream {
                TlsStream::Client(s) => {
                    s.flush().ok();
                }
                TlsStream::Server(s) => {
                    s.flush().ok();
                }
            }
            Ok(n as i64)
        });

        match res {
            Ok(n) => mdh_ok(__mdh_make_int(n)),
            Err(e) => mdh_err(&e),
        }
    }) {
        Ok(result) => result,
        Err(_) => unsafe { mdh_err("Rust panic in tls_send") },
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_tls_recv(tls: MdhValue, max_len: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        if tls.tag != MDH_TAG_INT || tls.data <= 0 {
            return mdh_err("tls_recv expects a TLS handle");
        }
        if max_len.tag != MDH_TAG_INT && max_len.tag != MDH_TAG_FLOAT {
            return mdh_err("tls_recv expects max_len integer");
        }
        let mut len = if max_len.tag == MDH_TAG_INT {
            max_len.data
        } else {
            mdh_float_value(max_len) as i64
        };
        if len < 0 {
            len = 0;
        }

        let res = tls_with_mut(tls.data, |session| {
            let stream = session.stream.as_mut().ok_or("TLS not connected")?;
            let mut buf = vec![0u8; len as usize];
            let n = match stream {
                TlsStream::Client(s) => s.read(&mut buf),
                TlsStream::Server(s) => s.read(&mut buf),
            }
            .map_err(|e| format!("TLS recv failed: {}", e))?;
            buf.truncate(n);
            Ok(buf)
        });

        match res {
            Ok(buf) => {
                let bytes_val = __mdh_bytes_new(__mdh_make_int(buf.len() as i64));
                let bytes_ptr = bytes_val.data as *mut MdhBytes;
                if !bytes_ptr.is_null() && !buf.is_empty() {
                    std::ptr::copy_nonoverlapping(buf.as_ptr(), (*bytes_ptr).data, buf.len());
                    (*bytes_ptr).length = buf.len() as i64;
                }
                mdh_ok(bytes_val)
            }
            Err(e) => mdh_err(&e),
        }
    }) {
        Ok(result) => result,
        Err(_) => unsafe { mdh_err("Rust panic in tls_recv") },
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_tls_close(tls: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        if tls.tag != MDH_TAG_INT || tls.data <= 0 {
            return mdh_err("tls_close expects a TLS handle");
        }
        tls_remove(tls.data);
        mdh_ok(__mdh_make_nil())
    }) {
        Ok(result) => result,
        Err(_) => unsafe { mdh_err("Rust panic in tls_close") },
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_srtp_create(config: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        if config.tag != MDH_TAG_DICT {
            return mdh_err("srtp_create expects config dict");
        }

        let profile_str = mdh_dict_get_string(config, "profile")
            .unwrap_or_else(|| "SRTP_AES128_CM_SHA1_80".to_string());
        let profile = match protection_profile_from_str(&profile_str) {
            Some(p) => p,
            None => return mdh_err("Unsupported SRTP profile"),
        };

        let role = mdh_dict_get_string(config, "role").unwrap_or_else(|| "client".to_string());

        let mut send_key = mdh_dict_get_bytes(config, "send_key");
        let mut send_salt = mdh_dict_get_bytes(config, "send_salt");
        let mut recv_key = mdh_dict_get_bytes(config, "recv_key");
        let mut recv_salt = mdh_dict_get_bytes(config, "recv_salt");

        let client_key = mdh_dict_get_bytes(config, "client_key");
        let client_salt = mdh_dict_get_bytes(config, "client_salt");
        let server_key = mdh_dict_get_bytes(config, "server_key");
        let server_salt = mdh_dict_get_bytes(config, "server_salt");

        if send_key.is_none() || send_salt.is_none() || recv_key.is_none() || recv_salt.is_none() {
            if client_key.is_some() && client_salt.is_some() && server_key.is_some() && server_salt.is_some() {
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
                let master_key = mdh_dict_get_bytes(config, "master_key");
                let master_salt = mdh_dict_get_bytes(config, "master_salt");
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
            None => return mdh_err("Missing SRTP send_key"),
        };
        let send_salt = match send_salt {
            Some(v) => v,
            None => return mdh_err("Missing SRTP send_salt"),
        };
        let recv_key = match recv_key {
            Some(v) => v,
            None => return mdh_err("Missing SRTP recv_key"),
        };
        let recv_salt = match recv_salt {
            Some(v) => v,
            None => return mdh_err("Missing SRTP recv_salt"),
        };

        let send_master = MasterKey::new(&send_key, &send_salt, &None);
        let recv_master = MasterKey::new(&recv_key, &recv_salt, &None);
        let send_cfg = StreamConfig::new(vec![send_master], &profile, &profile);
        let recv_cfg = StreamConfig::new(vec![recv_master], &profile, &profile);

        let mut send = SendSession::new();
        if let Err(e) = send.add_stream(None, &send_cfg) {
            return mdh_err(&format!("SRTP send session error: {}", e));
        }
        let mut recv = RecvSession::new();
        if let Err(e) = recv.add_stream(None, &recv_cfg) {
            return mdh_err(&format!("SRTP recv session error: {}", e));
        }

        let id = srtp_register(SrtpSession { send, recv });
        mdh_ok(__mdh_make_int(id))
    }) {
        Ok(result) => result,
        Err(_) => unsafe { mdh_err("Rust panic in srtp_create") },
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_srtp_protect(ctx: MdhValue, packet: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        if ctx.tag != MDH_TAG_INT || ctx.data <= 0 {
            return mdh_err("srtp_protect expects SRTP handle");
        }
        let data = match mdh_bytes_to_vec(packet) {
            Some(v) => v,
            None => return mdh_err("srtp_protect expects bytes"),
        };
        let res = srtp_with_mut(ctx.data, |session| {
            session
                .send
                .rtp_protect(data)
                .map_err(|e| format!("SRTP protect failed: {}", e))
        });
        match res {
            Ok(buf) => mdh_ok(mdh_make_bytes_from_vec(&buf)),
            Err(e) => mdh_err(&e),
        }
    }) {
        Ok(result) => result,
        Err(_) => unsafe { mdh_err("Rust panic in srtp_protect") },
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_srtp_unprotect(ctx: MdhValue, packet: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        if ctx.tag != MDH_TAG_INT || ctx.data <= 0 {
            return mdh_err("srtp_unprotect expects SRTP handle");
        }
        let data = match mdh_bytes_to_vec(packet) {
            Some(v) => v,
            None => return mdh_err("srtp_unprotect expects bytes"),
        };
        let res = srtp_with_mut(ctx.data, |session| {
            session
                .recv
                .rtp_unprotect(data)
                .map_err(|e| format!("SRTP unprotect failed: {}", e))
        });
        match res {
            Ok(buf) => mdh_ok(mdh_make_bytes_from_vec(&buf)),
            Err(e) => mdh_err(&e),
        }
    }) {
        Ok(result) => result,
        Err(_) => unsafe { mdh_err("Rust panic in srtp_unprotect") },
    }
}

fn identity_from_pem(cert_pem: &str, key_pem: &str) -> Result<Identity, String> {
    let cert = X509::from_pem(cert_pem.as_bytes()).map_err(|e| format!("Invalid cert PEM: {}", e))?;
    let key = PKey::private_key_from_pem(key_pem.as_bytes())
        .map_err(|e| format!("Invalid key PEM: {}", e))?;
    let builder = Pkcs12::builder();
    let pkcs12 = builder
        .build("", "mdhavers", &key, &cert)
        .map_err(|e| format!("PKCS12 build failed: {}", e))?;
    let der = pkcs12.to_der().map_err(|e| format!("PKCS12 serialize failed: {}", e))?;
    Identity::from_pkcs12(&der, "").map_err(|e| format!("Identity parse failed: {}", e))
}

#[no_mangle]
pub extern "C" fn __mdh_rs_dtls_server_new(config: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        let cfg = match dtls_config_from_value(config) {
            Ok(cfg) => cfg,
            Err(e) => return mdh_err(&e),
        };
        let id = dtls_register(cfg);
        mdh_ok(__mdh_make_int(id))
    }) {
        Ok(result) => result,
        Err(_) => unsafe { mdh_err("Rust panic in dtls_server_new") },
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_dtls_handshake(dtls: MdhValue, sock: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        if dtls.tag != MDH_TAG_INT || dtls.data <= 0 {
            return mdh_err("dtls_handshake expects DTLS handle");
        }
        if sock.tag != MDH_TAG_INT {
            return mdh_err("dtls_handshake expects socket fd");
        }
        let cfg = match dtls_get(dtls.data) {
            Ok(cfg) => cfg,
            Err(e) => return mdh_err(&e),
        };

        let fd = sock.data as i32;
        let socket = std::net::UdpSocket::from_raw_fd(fd);
        if let Err(e) = socket.set_nonblocking(false) {
            return mdh_err(&format!("DTLS socket setup failed: {}", e));
        }

        let remote = if let (Some(host), Some(port)) = (cfg.remote_host.clone(), cfg.remote_port) {
            match format!("{}:{}", host, port).parse() {
                Ok(addr) => addr,
                Err(_) => return mdh_err("Invalid remote address"),
            }
        } else {
            match socket.peer_addr() {
                Ok(addr) => addr,
                Err(_) => return mdh_err("dtls_handshake requires remote_host/remote_port"),
            }
        };

        if let Err(e) = socket.connect(remote) {
            return mdh_err(&format!("DTLS connect failed: {}", e));
        }

        let channel = UdpChannel {
            socket,
            remote_addr: remote,
        };

        let mut selected_profile: Option<SrtpProfile> = None;
        let stream = if cfg.mode == TlsMode::Client {
            let mut builder = DtlsConnector::builder();
            for profile in &cfg.srtp_profiles {
                builder.add_srtp_profile(*profile);
            }
            if let Some(ca_pem) = &cfg.ca_pem {
                let cert = match udp_dtls::Certificate::from_pem(ca_pem.as_bytes()) {
                    Ok(cert) => cert,
                    Err(e) => return mdh_err(&format!("Invalid CA cert: {}", e)),
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
                    Err(e) => return mdh_err(&e),
                };
                builder.identity(identity);
            }
            let connector = match DtlsConnector::new(&builder) {
                Ok(connector) => connector,
                Err(e) => return mdh_err(&format!("{}", e)),
            };
            match connector.connect(&cfg.server_name, channel) {
                Ok(stream) => {
                    selected_profile = stream.selected_srtp_profile().ok().flatten();
                    stream
                }
                Err(err) => {
                    return mdh_err(&format!("DTLS connect failed: {}", err));
                }
            }
        } else {
            let cert_pem = match cfg.cert_pem.as_ref() {
                Some(v) => v,
                None => return mdh_err("Server cert_pem required"),
            };
            let key_pem = match cfg.key_pem.as_ref() {
                Some(v) => v,
                None => return mdh_err("Server key_pem required"),
            };
            let identity = match identity_from_pem(cert_pem, key_pem) {
                Ok(identity) => identity,
                Err(e) => return mdh_err(&e),
            };
            let mut builder = DtlsAcceptor::builder(identity);
            for profile in &cfg.srtp_profiles {
                builder.add_srtp_profile(*profile);
            }
            let acceptor = match DtlsAcceptor::new(&builder) {
                Ok(acceptor) => acceptor,
                Err(e) => return mdh_err(&format!("{}", e)),
            };
            match acceptor.accept(channel) {
                Ok(stream) => {
                    selected_profile = stream.selected_srtp_profile().ok().flatten();
                    stream
                }
                Err(err) => {
                    return mdh_err(&format!("DTLS accept failed: {}", err));
                }
            }
        };

        let profile = selected_profile.unwrap_or(SrtpProfile::Aes128CmSha180);
        let (key_len, salt_len) = srtp_key_salt_len(profile);
        let total = 2 * (key_len + salt_len);
        let material = match stream.keying_material(total) {
            Ok(material) => material,
            Err(e) => return mdh_err(&format!("Keying material failed: {}", e)),
        };

        let client_key = &material[0..key_len];
        let server_key = &material[key_len..(2 * key_len)];
        let client_salt = &material[(2 * key_len)..(2 * key_len + salt_len)];
        let server_salt = &material[(2 * key_len + salt_len)..(2 * key_len + 2 * salt_len)];

        let mut dict = __mdh_empty_dict();
        dict = __mdh_dict_set(dict, mdh_make_string_from_rust("profile"), mdh_make_string_from_rust(&profile.to_string()));
        dict = __mdh_dict_set(dict, mdh_make_string_from_rust("client_key"), mdh_make_bytes_from_vec(client_key));
        dict = __mdh_dict_set(dict, mdh_make_string_from_rust("client_salt"), mdh_make_bytes_from_vec(client_salt));
        dict = __mdh_dict_set(dict, mdh_make_string_from_rust("server_key"), mdh_make_bytes_from_vec(server_key));
        dict = __mdh_dict_set(dict, mdh_make_string_from_rust("server_salt"), mdh_make_bytes_from_vec(server_salt));
        dict = __mdh_dict_set(dict, mdh_make_string_from_rust("key_len"), __mdh_make_int(key_len as i64));
        dict = __mdh_dict_set(dict, mdh_make_string_from_rust("salt_len"), __mdh_make_int(salt_len as i64));

        mdh_ok(dict)
    }) {
        Ok(result) => result,
        Err(_) => unsafe { mdh_err("Rust panic in dtls_handshake") },
    }
}
