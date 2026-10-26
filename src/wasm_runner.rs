use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::rc::Rc;

use wasmtime::{Caller, Engine, FuncType, Linker, Memory, MemoryType, Module, Store, Val, ValType};

use crate::error::{HaversError, HaversResult};
use crate::interpreter::Interpreter;
use crate::value::{
    DictValue, HaversClass, HaversInstance, NativeFunction, NativeObject, SetValue, Value,
};
use crate::wasm_compiler::{
    AUDIO_IMPORTS, HOF_BUILTINS, MAX_BUILTIN_ARGS, RAISE_ASSERT, RAISE_HURL, UNSET_GLOBAL,
};

type Handle = i64;

#[derive(Debug, Clone)]
enum HostValue {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Handle>),
    Dict(DictEntries),
    /// A creel: each member's identity key and its handle (like dict entries, minus values).
    Set(DictEntries),
    /// A byte buffer (`bytes_new` an' friends), copied tae an' fae the interpreter's.
    Bytes(Vec<u8>),
    NativeObject(TriObject),
    NativeCtor(String),
    /// A guest function value: its `__mdh_fns` table slot, arity, how many arguments
    /// are required (fewer than `arity` when it has defaults: it then takes nil fer each
    /// missing one an' the argument count last), captured cells (a list, or nil) an' name.
    Function {
        index: u32,
        arity: usize,
        required: usize,
        env: Handle,
        name: String,
    },
    /// An interpreter builtin used as a value.
    Builtin(String),
    /// A variable shared between a function an' its closures.
    Cell(Handle),
    /// A `kin`: its parent class (nil fer nane) an' its ain methods.
    Class {
        name: String,
        parent: Handle,
        methods: Vec<Method>,
    },
    /// An instance: its class an' its fields in the order they were first set.
    Instance {
        class: Handle,
        fields: Vec<(String, Handle)>,
    },
    /// `obj.method` as a value: the method wi' `masel` bound tae `recv`.
    Bound {
        recv: Handle,
        method: Method,
    },
}

/// A class method: a guest function in `__mdh_fns` taking `(env, masel, params.., argc)`.
/// `env` is the list o' cells it captures (a class made inside a function), or nil.
#[derive(Debug, Clone)]
struct Method {
    name: String,
    index: u32,
    params: usize,
    required: usize,
    env: Handle,
}

/// Dict entries in insertion order (like the interpreter's dicts).
type DictEntries = Vec<(ValueKey, Handle)>;

fn dict_get(entries: &DictEntries, key: &ValueKey) -> Option<Handle> {
    entries.iter().find(|(k, _)| k == key).map(|(_, v)| *v)
}

/// Insert or replace; an existing key keeps its original position.
fn dict_put(entries: &mut DictEntries, key: ValueKey, value: Handle) {
    match entries.iter_mut().find(|(k, _)| *k == key) {
        Some(slot) => slot.1 = value,
        None => entries.push((key, value)),
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum ValueKey {
    Nil,
    Bool(bool),
    Int(i64),
    Float(u64),
    String(String),
    List(Handle),
    Dict(Handle),
    Set(Handle),
    Bytes(Handle),
    NativeObject(Handle),
    NativeCtor(Handle),
    /// A function or builtin value, by identity.
    Function(Handle),
    /// An instance or a class, by identity.
    Object(Handle),
}

#[derive(Debug, Clone)]
struct TriObject {
    kind: String,
    fields: HashMap<String, Handle>,
}

impl TriObject {
    fn new(kind: &str) -> Self {
        TriObject {
            kind: kind.to_string(),
            fields: HashMap::new(),
        }
    }
}

#[derive(Debug)]
struct HostStore {
    values: Vec<HostValue>,
    /// Allocation sequence number of each slot (`FREED` once reclaimed). The fixed
    /// nil/false/true slots are born at 0, so they are never young and never freed.
    births: Vec<u64>,
    /// Reclaimed slots, reused by `alloc` before the store grows.
    free: Vec<usize>,
    next_birth: u64,
    /// Open host scopes (one per running guest statement), innermost last.
    scopes: Vec<Scope>,
    /// Slots allocated while a scope was open, in birth order: each scope's young
    /// objects are the suffix from its `log_start`.
    young: Vec<usize>,
    /// Out-of-range ints and floats, interned by value so repeats share one slot.
    wide_ints: HashMap<i64, Handle>,
    wide_floats: HashMap<u64, Handle>,
}

// Reclamation (MDH-BUG-FLU-00092). The compiler wraps every guest statement in a host
// scope. When a scope closes, the guest hands over all the current function's locals as
// roots; everything born since the scope opened that is not reachable from them, or from
// an older object mutated during the scope (the `remembered` write barrier in `get_mut`),
// is freed. Older frames cannot hold younger handles (a callee only passes back its
// return value, which is a root), and the guest's operand stack is empty between
// statements, so the roots are complete. Survivors stay young for the enclosing scope.
// A loop's scope stays open for the whole loop, so survivors that later iterations drop
// would pile up in it: at each iteration boundary (a safe point in the loop's own frame)
// the guest asks `scope_collect_due` and, when the region has doubled, sweeps it in place.
#[derive(Debug)]
struct Scope {
    mark: u64,
    log_start: usize,
    remembered: HashSet<usize>,
    /// Region length (`young.len() - log_start`) after the last in-place sweep.
    swept_len: usize,
}

const FREED: u64 = u64::MAX;
/// A loop's region must exceed this many young slots before an in-place sweep.
const COLLECT_FLOOR: usize = 256;

// Handle layout. Plain heap handles are store indices (>= 0, far below FLOAT_TAG).
// Ints in [-2^61, 2^61) live in the handle itself as a negative number (`n - INT_BIAS`).
// Floats whose top exponent bits are 011 or 100 (about 1e-77..1e77) live in the handle
// as FLOAT_TAG | payload. Arithmetic on those never touches the store, so loops of
// ordinary numeric work cannot grow it. Anything else is interned in the store.
const INT_BIAS: i64 = 1 << 62;
const INT_MIN_INLINE: i64 = -(1 << 61);
const INT_MAX_INLINE: i64 = (1 << 61) - 1;
const FLOAT_TAG: i64 = 1 << 62;
const LOW60: u64 = (1 << 60) - 1;

impl HostStore {
    fn new() -> Self {
        // Nil, false and true are interned at fixed handles so they never grow the store.
        HostStore {
            values: vec![
                HostValue::Nil,
                HostValue::Bool(false),
                HostValue::Bool(true),
            ],
            births: vec![0; 3],
            free: Vec::new(),
            next_birth: 1,
            scopes: Vec::new(),
            young: Vec::new(),
            wide_ints: HashMap::new(),
            wide_floats: HashMap::new(),
        }
    }

    fn int_handle(&mut self, value: i64) -> Handle {
        if (INT_MIN_INLINE..=INT_MAX_INLINE).contains(&value) {
            return value - INT_BIAS;
        }
        if let Some(handle) = self.wide_ints.get(&value) {
            return *handle;
        }
        let handle = self.alloc(HostValue::Int(value));
        self.wide_ints.insert(value, handle);
        handle
    }

    fn float_handle(&mut self, value: f64) -> Handle {
        let bits = value.to_bits();
        let top = (bits >> 60) & 7;
        if top == 3 || top == 4 {
            let sign = bits >> 63;
            let sel = u64::from(top == 4);
            return FLOAT_TAG | ((sel << 61) | (sign << 60) | (bits & LOW60)) as i64;
        }
        if let Some(handle) = self.wide_floats.get(&bits) {
            return *handle;
        }
        let handle = self.alloc(HostValue::Float(value));
        self.wide_floats.insert(bits, handle);
        handle
    }

    /// The value behind a handle; inline numbers are decoded on the fly.
    fn view(&self, handle: Handle) -> Option<Cow<'_, HostValue>> {
        if handle < 0 {
            return Some(Cow::Owned(HostValue::Int(handle + INT_BIAS)));
        }
        if handle >= FLOAT_TAG {
            let payload = (handle & (FLOAT_TAG - 1)) as u64;
            let top: u64 = if (payload >> 61) & 1 == 1 { 4 } else { 3 };
            let bits = (((payload >> 60) & 1) << 63) | (top << 60) | (payload & LOW60);
            return Some(Cow::Owned(HostValue::Float(f64::from_bits(bits))));
        }
        self.get(handle).map(Cow::Borrowed)
    }

    fn alloc(&mut self, value: HostValue) -> Handle {
        let birth = self.next_birth;
        self.next_birth += 1;
        let idx = match self.free.pop() {
            Some(idx) => {
                self.values[idx] = value;
                self.births[idx] = birth;
                idx
            }
            None => {
                self.values.push(value);
                self.births.push(birth);
                self.values.len() - 1
            }
        };
        if !self.scopes.is_empty() {
            self.young.push(idx);
        }
        idx as Handle
    }

    fn get(&self, handle: Handle) -> Option<&HostValue> {
        let idx = handle as usize;
        self.values.get(idx)
    }

    /// Mutable access is the write barrier: an object older than the innermost scope
    /// may be about tae hold a young handle, so that scope remembers it as a root.
    fn get_mut(&mut self, handle: Handle) -> Option<&mut HostValue> {
        let idx = handle as usize;
        let birth = *self.births.get(idx)?;
        if let Some(top) = self.scopes.last_mut() {
            if birth < top.mark {
                top.remembered.insert(idx);
            }
        }
        self.values.get_mut(idx)
    }

    /// Slots currently holding a value (the leak regression guard).
    fn live_count(&self) -> usize {
        self.values.len() - self.free.len()
    }

    fn is_live(&self, handle: Handle) -> bool {
        self.births
            .get(handle as usize)
            .is_some_and(|birth| *birth != FREED)
    }

    fn scope_enter(&mut self) {
        self.scopes.push(Scope {
            mark: self.next_birth,
            log_start: self.young.len(),
            remembered: HashSet::new(),
            swept_len: 0,
        });
    }

    /// Whether the innermost scope's region has grown enough (doubled since its last
    /// sweep, and past a floor) tae be worth sweeping in place. Amortised linear.
    fn scope_collect_due(&self) -> bool {
        self.scopes.last().is_some_and(|top| {
            let region = self.young.len() - top.log_start;
            region > (2 * top.swept_len).max(COLLECT_FLOOR)
        })
    }

    /// Sweep the innermost scope's region in place, keeping the scope open.
    fn scope_collect(&mut self, roots: &[Handle]) {
        let Some(top) = self.scopes.last_mut() else {
            return;
        };
        let (mark, log_start) = (top.mark, top.log_start);
        let remembered = std::mem::take(&mut top.remembered);
        self.sweep(mark, log_start, roots, &remembered);
        let region = self.young.len() - log_start;
        if let Some(top) = self.scopes.last_mut() {
            top.remembered = remembered;
            top.swept_len = region;
        }
    }

    /// Whether closing the innermost `pop` scopes has anything tae sweep (and so needs
    /// the roots); most statements o' numeric work allocate nothing.
    fn scope_exit_needs_roots(&self, pop: usize) -> bool {
        match self.scopes.len().checked_sub(pop) {
            Some(keep) if pop > 0 => self.young.len() > self.scopes[keep].log_start,
            _ => false,
        }
    }

    /// Close the innermost `pop` scopes and free their unreachable young objects.
    fn scope_exit(&mut self, roots: &[Handle], pop: usize) -> Result<(), String> {
        if pop == 0 || pop > self.scopes.len() {
            return Err(format!(
                "Host scope stack is in a fankle: cannae close {} o' {} scopes",
                pop,
                self.scopes.len()
            ));
        }
        let keep = self.scopes.len() - pop;
        let mut mark = 0;
        let mut log_start = 0;
        let mut remembered = HashSet::new();
        for (i, scope) in self.scopes.drain(keep..).enumerate() {
            if i == 0 {
                mark = scope.mark;
                log_start = scope.log_start;
            }
            remembered.extend(scope.remembered);
        }

        self.sweep(mark, log_start, roots, &remembered);

        match self.scopes.last_mut() {
            Some(parent) => {
                for idx in remembered {
                    if self.births[idx] < parent.mark {
                        parent.remembered.insert(idx);
                    }
                }
            }
            // Nae scope left tae count anything as young.
            None => self.young.clear(),
        }
        Ok(())
    }

    /// Free every object born at or after `mark` (the log suffix fae `log_start`) that
    /// is unreachable fae the roots or fae the remembered older objects.
    fn sweep(
        &mut self,
        mark: u64,
        log_start: usize,
        roots: &[Handle],
        remembered: &HashSet<usize>,
    ) {
        if self.young.len() <= log_start {
            return;
        }
        let mut live: HashSet<usize> = HashSet::new();
        let mut work: Vec<Handle> = roots.to_vec();
        for &idx in remembered {
            push_children(&self.values[idx], &mut work);
        }
        while let Some(handle) = work.pop() {
            if !(0..FLOAT_TAG).contains(&handle) {
                continue;
            }
            let idx = handle as usize;
            match self.births.get(idx) {
                Some(&birth) if birth >= mark && birth != FREED => {}
                _ => continue,
            }
            if live.insert(idx) {
                push_children(&self.values[idx], &mut work);
            }
        }
        let mut kept = log_start;
        for i in log_start..self.young.len() {
            let idx = self.young[i];
            if live.contains(&idx) {
                self.young[kept] = idx;
                kept += 1;
            } else {
                self.release(idx);
            }
        }
        self.young.truncate(kept);
    }

    fn release(&mut self, idx: usize) {
        match std::mem::replace(&mut self.values[idx], HostValue::Nil) {
            HostValue::Int(n) if self.wide_ints.get(&n) == Some(&(idx as Handle)) => {
                self.wide_ints.remove(&n);
            }
            HostValue::Float(f) if self.wide_floats.get(&f.to_bits()) == Some(&(idx as Handle)) => {
                self.wide_floats.remove(&f.to_bits());
            }
            _ => {}
        }
        self.births[idx] = FREED;
        self.free.push(idx);
    }

    fn key_for(&self, handle: Handle) -> Option<ValueKey> {
        match &*self.view(handle)? {
            HostValue::Nil => Some(ValueKey::Nil),
            HostValue::Bool(b) => Some(ValueKey::Bool(*b)),
            HostValue::Int(n) => Some(ValueKey::Int(*n)),
            HostValue::Float(f) => Some(ValueKey::Float(f.to_bits())),
            HostValue::String(s) => Some(ValueKey::String(s.clone())),
            HostValue::List(_) => Some(ValueKey::List(handle)),
            HostValue::Dict(_) => Some(ValueKey::Dict(handle)),
            HostValue::Set(_) => Some(ValueKey::Set(handle)),
            HostValue::Bytes(_) => Some(ValueKey::Bytes(handle)),
            HostValue::NativeObject(_) => Some(ValueKey::NativeObject(handle)),
            HostValue::NativeCtor(_) => Some(ValueKey::NativeCtor(handle)),
            HostValue::Function { .. } | HostValue::Builtin(_) | HostValue::Bound { .. } => {
                Some(ValueKey::Function(handle))
            }
            HostValue::Class { .. } | HostValue::Instance { .. } => Some(ValueKey::Object(handle)),
            HostValue::Cell(_) => None,
        }
    }

    fn is_truthy(&self, handle: Handle) -> bool {
        match self.view(handle).as_deref() {
            Some(HostValue::Nil) => false,
            Some(HostValue::Bool(b)) => *b,
            Some(HostValue::Int(n)) => *n != 0,
            Some(HostValue::Float(f)) => *f != 0.0,
            Some(HostValue::String(s)) => !s.is_empty(),
            Some(HostValue::List(items)) => !items.is_empty(),
            Some(HostValue::Dict(map)) => !map.is_empty(),
            Some(HostValue::Set(members)) => !members.is_empty(),
            Some(HostValue::Bytes(bytes)) => !bytes.is_empty(),
            Some(HostValue::NativeObject(_)) => true,
            Some(HostValue::NativeCtor(_))
            | Some(HostValue::Function { .. })
            | Some(HostValue::Builtin(_))
            | Some(HostValue::Cell(_))
            | Some(HostValue::Class { .. })
            | Some(HostValue::Instance { .. })
            | Some(HostValue::Bound { .. }) => true,
            None => false,
        }
    }

    fn class_name(&self, class: Handle) -> String {
        match self.get(class) {
            Some(HostValue::Class { name, .. }) => name.clone(),
            _ => String::new(),
        }
    }

    /// A method by name on a class or its ancestors (the interpreter's `find_method`).
    fn find_method(&self, class: Handle, name: &str) -> Option<Method> {
        let mut current = class;
        while let Some(HostValue::Class {
            parent, methods, ..
        }) = self.get(current)
        {
            if let Some(method) = methods.iter().find(|m| m.name == name) {
                return Some(method.clone());
            }
            if *parent == current {
                return None;
            }
            current = *parent;
        }
        None
    }

    fn to_string(&self, handle: Handle) -> String {
        match self.view(handle).as_deref() {
            Some(HostValue::Nil) => "naething".to_string(),
            Some(HostValue::Bool(true)) => "aye".to_string(),
            Some(HostValue::Bool(false)) => "nae".to_string(),
            Some(HostValue::Int(n)) => n.to_string(),
            Some(HostValue::Float(f)) => f.to_string(),
            Some(HostValue::String(s)) => s.clone(),
            Some(HostValue::List(items)) => {
                let mut parts = Vec::new();
                for item in items {
                    parts.push(self.to_string(*item));
                }
                format!("[{}]", parts.join(", "))
            }
            Some(HostValue::Dict(map)) => {
                let mut parts = Vec::new();
                for (key, value) in map {
                    let key_str = match key {
                        ValueKey::Nil => "naething".to_string(),
                        ValueKey::Bool(true) => "aye".to_string(),
                        ValueKey::Bool(false) => "nae".to_string(),
                        ValueKey::Int(n) => n.to_string(),
                        ValueKey::Float(bits) => f64::from_bits(*bits).to_string(),
                        ValueKey::String(s) => s.clone(),
                        ValueKey::List(h)
                        | ValueKey::Dict(h)
                        | ValueKey::Set(h)
                        | ValueKey::Bytes(h)
                        | ValueKey::NativeObject(h)
                        | ValueKey::NativeCtor(h)
                        | ValueKey::Function(h)
                        | ValueKey::Object(h) => self.to_string(*h),
                    };
                    parts.push(format!("\"{}\": {}", key_str, self.to_string(*value)));
                }
                format!("{{{}}}", parts.join(", "))
            }
            Some(HostValue::Set(members)) => {
                let mut parts: Vec<String> =
                    members.iter().map(|(_, h)| self.to_string(*h)).collect();
                parts.sort();
                // The interpreter quotes every member (MDH-BUG-FLU-00124).
                let quoted: Vec<String> = parts.iter().map(|p| format!("\"{}\"", p)).collect();
                format!("creel{{{}}}", quoted.join(", "))
            }
            Some(HostValue::Bytes(bytes)) => format!("bytes[{}]", bytes.len()),
            Some(HostValue::NativeObject(obj)) => format!("<native {}>", obj.kind),
            Some(HostValue::NativeCtor(kind)) => format!("<native dae tri.{}>", kind),
            Some(HostValue::Function { name, .. }) => format!("<dae {}>", name),
            Some(HostValue::Builtin(name)) => format!("<native dae {}>", name),
            Some(HostValue::Cell(value)) => self.to_string(*value),
            Some(HostValue::Class { name, .. }) => format!("<kin {}>", name),
            Some(HostValue::Instance { class, .. }) => {
                format!("<{} instance>", self.class_name(*class))
            }
            Some(HostValue::Bound { method, .. }) => format!("<dae {}>", method.name),
            None => "<invalid>".to_string(),
        }
    }
}

/// Every handle a value holds (list items, dict keys and values, object fields).
fn push_children(value: &HostValue, out: &mut Vec<Handle>) {
    match value {
        HostValue::List(items) => out.extend_from_slice(items),
        HostValue::Dict(entries) => {
            for (key, value) in entries {
                match key {
                    ValueKey::List(h)
                    | ValueKey::Dict(h)
                    | ValueKey::Set(h)
                    | ValueKey::Bytes(h)
                    | ValueKey::NativeObject(h)
                    | ValueKey::NativeCtor(h)
                    | ValueKey::Function(h)
                    | ValueKey::Object(h) => out.push(*h),
                    _ => {}
                }
                out.push(*value);
            }
        }
        HostValue::Set(members) => out.extend(members.iter().map(|(_, h)| *h)),
        HostValue::NativeObject(obj) => out.extend(obj.fields.values().copied()),
        HostValue::Function { env, .. } => out.push(*env),
        HostValue::Cell(value) => out.push(*value),
        HostValue::Class {
            parent, methods, ..
        } => {
            out.push(*parent);
            out.extend(methods.iter().map(|m| m.env));
        }
        HostValue::Instance { class, fields } => {
            out.push(*class);
            out.extend(fields.iter().map(|(_, h)| *h));
        }
        HostValue::Bound { recv, method } => {
            out.push(*recv);
            out.push(method.env);
        }
        _ => {}
    }
}

/// The Rc address o' a list or dict, its identity across a builtin call.
fn container_ptr(value: &Value) -> Option<usize> {
    match value {
        Value::List(rc) => Some(Rc::as_ptr(rc) as *const () as usize),
        Value::Dict(rc) => Some(Rc::as_ptr(rc) as *const () as usize),
        Value::Set(rc) => Some(Rc::as_ptr(rc) as *const () as usize),
        Value::Bytes(rc) => Some(Rc::as_ptr(rc) as *const () as usize),
        Value::Instance(rc) => Some(Rc::as_ptr(rc) as *const () as usize),
        Value::NativeObject(rc) => Some(Rc::as_ptr(rc) as *const () as usize),
        _ => None,
    }
}

const ERR_INVALID: &str = "That's pure mince! Type error: invalid value";

/// Converts handles tae interpreter `Value`s an' back fer one builtin call. Each host
/// list or dict becomes one shared `Value` (so aliasing an' cycles survive), an' comes
/// back as its ain handle. Arguments are borrowed: after the call, ony container the
/// builtin changed is written back through `get_mut`, the reclamation write barrier.
#[derive(Default)]
struct ValueBridge {
    by_handle: HashMap<Handle, Value>,
    by_ptr: HashMap<usize, Handle>,
}

impl ValueBridge {
    fn remember(&mut self, handle: Handle, value: &Value) {
        if let Some(ptr) = container_ptr(value) {
            self.by_ptr.insert(ptr, handle);
        }
        self.by_handle.insert(handle, value.clone());
    }

    fn to_value(&mut self, store: &HostStore, handle: Handle) -> Result<Value, String> {
        if let Some(value) = self.by_handle.get(&handle) {
            return Ok(value.clone());
        }
        let viewed = store.view(handle).ok_or_else(|| ERR_INVALID.to_string())?;
        match viewed.as_ref() {
            HostValue::Nil => Ok(Value::Nil),
            HostValue::Bool(b) => Ok(Value::Bool(*b)),
            HostValue::Int(n) => Ok(Value::Integer(*n)),
            HostValue::Float(f) => Ok(Value::Float(*f)),
            HostValue::String(s) => Ok(Value::String(s.clone())),
            HostValue::List(items) => {
                let rc = Rc::new(RefCell::new(Vec::with_capacity(items.len())));
                let value = Value::List(rc.clone());
                self.remember(handle, &value);
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(self.to_value(store, *item)?);
                }
                *rc.borrow_mut() = out;
                Ok(value)
            }
            HostValue::Dict(entries) => {
                let rc = Rc::new(RefCell::new(DictValue::new()));
                let value = Value::Dict(rc.clone());
                self.remember(handle, &value);
                for (key, item) in entries {
                    let key = self.key_to_value(store, key)?;
                    let item = self.to_value(store, *item)?;
                    rc.borrow_mut().set(key, item);
                }
                Ok(value)
            }
            HostValue::Set(members) => {
                let rc = Rc::new(RefCell::new(SetValue::new()));
                let value = Value::Set(rc.clone());
                self.remember(handle, &value);
                for (_, item) in members {
                    let item = self.to_value(store, *item)?;
                    rc.borrow_mut().insert(item);
                }
                Ok(value)
            }
            HostValue::Bytes(bytes) => {
                let value = Value::Bytes(Rc::new(RefCell::new(bytes.clone())));
                self.remember(handle, &value);
                Ok(value)
            }
            // A builtin sees a name-only instance (`tae_string`, `whit_kind`, equality).
            HostValue::Instance { class, .. } => {
                let class = Rc::new(HaversClass::new(store.class_name(*class), None));
                let value = Value::Instance(Rc::new(RefCell::new(HaversInstance::new(class))));
                self.remember(handle, &value);
                Ok(value)
            }
            HostValue::Cell(_) => Err(ERR_INVALID.to_string()),
            // A function, builtin, class, bound method or tri value rides through as an
            // opaque `HostOpaque`, an' `to_handle` gies back its ain handle.
            other => {
                let value = Value::NativeObject(Rc::new(HostOpaque {
                    handle,
                    kind: host_type_name(other),
                    shown: store.to_string(handle),
                }));
                self.remember(handle, &value);
                Ok(value)
            }
        }
    }

    fn key_to_value(&mut self, store: &HostStore, key: &ValueKey) -> Result<Value, String> {
        Ok(match key {
            ValueKey::Nil => Value::Nil,
            ValueKey::Bool(b) => Value::Bool(*b),
            ValueKey::Int(n) => Value::Integer(*n),
            ValueKey::Float(bits) => Value::Float(f64::from_bits(*bits)),
            ValueKey::String(s) => Value::String(s.clone()),
            ValueKey::List(h)
            | ValueKey::Dict(h)
            | ValueKey::Set(h)
            | ValueKey::Bytes(h)
            | ValueKey::NativeObject(h)
            | ValueKey::NativeCtor(h)
            | ValueKey::Function(h)
            | ValueKey::Object(h) => self.to_value(store, *h)?,
        })
    }

    /// A value fae the builtin as a handle: a container that wis passed in keeps its
    /// handle, anything else is a fresh allocation in the current scope.
    fn to_handle(&mut self, store: &mut HostStore, value: &Value) -> Result<Handle, String> {
        if let Some(handle) = container_ptr(value).and_then(|ptr| self.by_ptr.get(&ptr)) {
            return Ok(*handle);
        }
        match value {
            Value::Nil => Ok(0),
            Value::Bool(b) => Ok(alloc_bool(store, *b)),
            Value::Integer(n) => Ok(alloc_int(store, *n)),
            Value::Float(f) => Ok(alloc_float(store, *f)),
            Value::String(s) => Ok(alloc_string(store, s.clone())),
            Value::List(_) | Value::Dict(_) | Value::Set(_) | Value::Bytes(_) => {
                let handle = store.alloc(HostValue::List(Vec::new()));
                if let Some(ptr) = container_ptr(value) {
                    self.by_ptr.insert(ptr, handle);
                }
                self.fill(store, handle, value)?;
                Ok(handle)
            }
            Value::NativeObject(obj) => match obj.as_any().downcast_ref::<HostOpaque>() {
                Some(opaque) => Ok(opaque.handle),
                None => Err(format!(
                    "That's pure mince! Type error: WASM cannae haud a {} yet",
                    obj.type_name()
                )),
            },
            other => Err(format!(
                "That's pure mince! Type error: WASM cannae haud a {} yet",
                other.type_name()
            )),
        }
    }

    /// Set a host container's contents fae a list or dict `Value`.
    fn fill(&mut self, store: &mut HostStore, handle: Handle, value: &Value) -> Result<(), String> {
        let contents = match value {
            Value::List(rc) => {
                let mut items = Vec::new();
                for item in rc.borrow().iter() {
                    items.push(self.to_handle(store, item)?);
                }
                HostValue::List(items)
            }
            Value::Dict(rc) => {
                let mut entries = Vec::new();
                for (key, item) in rc.borrow().iter() {
                    let key = self.to_handle(store, key)?;
                    let key = store.key_for(key).ok_or_else(|| ERR_INVALID.to_string())?;
                    entries.push((key, self.to_handle(store, item)?));
                }
                HostValue::Dict(entries)
            }
            Value::Set(rc) => {
                let mut members = Vec::new();
                for item in rc.borrow().iter() {
                    let item = self.to_handle(store, item)?;
                    let key = store.key_for(item).ok_or_else(|| ERR_INVALID.to_string())?;
                    dict_put(&mut members, key, item);
                }
                HostValue::Set(members)
            }
            Value::Bytes(rc) => HostValue::Bytes(rc.borrow().clone()),
            _ => return Err(ERR_INVALID.to_string()),
        };
        if let Some(slot) = store.get_mut(handle) {
            *slot = contents;
        }
        Ok(())
    }

    /// Write back every argument container the builtin changed (`shove`, `yank`, ...).
    fn write_back(&mut self, store: &mut HostStore) -> Result<(), String> {
        let passed: Vec<(Handle, Value)> = self
            .by_handle
            .iter()
            .map(|(h, v)| (*h, v.clone()))
            .collect();
        for (handle, value) in passed {
            if !self.unchanged(store, handle, &value) {
                self.fill(store, handle, &value)?;
            }
        }
        Ok(())
    }

    fn unchanged(&self, store: &HostStore, handle: Handle, value: &Value) -> bool {
        match (store.get(handle), value) {
            (Some(HostValue::List(items)), Value::List(rc)) => {
                let now = rc.borrow();
                items.len() == now.len()
                    && items
                        .iter()
                        .zip(now.iter())
                        .all(|(h, v)| self.same_slot(store, *h, v))
            }
            (Some(HostValue::Dict(entries)), Value::Dict(rc)) => {
                let now = rc.borrow();
                entries.len() == now.len()
                    && entries.iter().zip(now.iter()).all(|((key, h), (k, v))| {
                        self.same_key(store, key, k) && self.same_slot(store, *h, v)
                    })
            }
            (Some(HostValue::Bytes(held)), Value::Bytes(rc)) => *held == *rc.borrow(),
            // A builtin cannae change an instance's fields, nor a carried host value.
            (Some(HostValue::Instance { .. }), Value::Instance(_)) => true,
            (Some(_), Value::NativeObject(_)) => true,
            _ => false,
        }
    }

    /// A container slot must still hold the very container passed in; a scalar, an equal one.
    fn same_slot(&self, store: &HostStore, handle: Handle, value: &Value) -> bool {
        if let Some(ptr) = container_ptr(value) {
            return self.by_ptr.get(&ptr) == Some(&handle);
        }
        match (store.view(handle).as_deref(), value) {
            (Some(HostValue::Nil), Value::Nil) => true,
            (Some(HostValue::Bool(a)), Value::Bool(b)) => a == b,
            (Some(HostValue::Int(a)), Value::Integer(b)) => a == b,
            (Some(HostValue::Float(a)), Value::Float(b)) => a.to_bits() == b.to_bits(),
            (Some(HostValue::String(a)), Value::String(b)) => a == b,
            _ => false,
        }
    }

    fn same_key(&self, store: &HostStore, key: &ValueKey, value: &Value) -> bool {
        match (key, value) {
            (
                ValueKey::List(h)
                | ValueKey::Dict(h)
                | ValueKey::Set(h)
                | ValueKey::Bytes(h)
                | ValueKey::NativeObject(h)
                | ValueKey::NativeCtor(h)
                | ValueKey::Function(h)
                | ValueKey::Object(h),
                _,
            ) => self.same_slot(store, *h, value),
            (ValueKey::Nil, Value::Nil) => true,
            (ValueKey::Bool(a), Value::Bool(b)) => a == b,
            (ValueKey::Int(a), Value::Integer(b)) => a == b,
            (ValueKey::Float(a), Value::Float(b)) => *a == b.to_bits(),
            (ValueKey::String(a), Value::String(b)) => a == b,
            _ => false,
        }
    }
}

/// A host value wi' nae interpreter twin (a function, closure, builtin, class, bound
/// method or tri value), carried through one builtin call by its handle. Natives may
/// move, compare an' show it; it comes back as the very same handle. It is no' a store
/// root: it lives only while the call does, an' the arguments that reach it are rooted.
#[derive(Debug)]
struct HostOpaque {
    handle: Handle,
    kind: &'static str,
    shown: String,
}

impl HostOpaque {
    fn refuse(&self, what: &str) -> HaversError {
        HaversError::TypeError {
            message: format!(
                "a builtin cannae {} a WASM {} ({})",
                what, self.kind, self.shown
            ),
            line: 0,
        }
    }
}

impl NativeObject for HostOpaque {
    fn type_name(&self) -> &str {
        self.kind
    }

    fn get(&self, _prop: &str) -> HaversResult<Value> {
        Err(self.refuse("look intae"))
    }

    fn set(&self, _prop: &str, _value: Value) -> HaversResult<Value> {
        Err(self.refuse("chynge"))
    }

    fn call(&self, _method: &str, _args: Vec<Value>) -> HaversResult<Value> {
        Err(self.refuse("ca'"))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn to_string(&self) -> String {
        self.shown.clone()
    }
}

/// Natives that never change their arguments, so a call needs nae write-back scan.
/// Keep this list explicit: a native missing fae it is merely slower, never wrong.
const READ_ONLY_NATIVES: &[&str] = &[
    "len",
    "keys",
    "values",
    "heid",
    "tail",
    "bum",
    "contains",
    "is_in_creel",
];

/// An inline scalar handle (nil, a bool, or an int/float held in the handle itself).
fn is_inline_scalar(handle: Handle) -> bool {
    !(3..FLOAT_TAG).contains(&handle)
}

/// `len`, `bum` an' `shove` answered straight fae the store, so a loop over a big list
/// isnae quadratic. `None` means "no shortcut": the native runs (an' reports ony error).
fn builtin_shortcut(store: &mut HostStore, name: &str, args: &[Handle]) -> Option<Handle> {
    if let ("shove", [list, item]) = (name, args) {
        // `get_mut` is the reclamation write barrier, as for ony other mutation.
        if let Some(HostValue::List(items)) = store.get_mut(*list) {
            items.push(*item);
            return Some(0);
        }
        return None;
    }
    let [arg] = args else { return None };
    let count = match (name, store.view(*arg)?.as_ref()) {
        ("len", HostValue::List(items)) => items.len(),
        ("len", HostValue::Dict(entries) | HostValue::Set(entries)) => entries.len(),
        ("len", HostValue::String(text)) => text.chars().count(),
        ("bum", HostValue::List(items)) => return items.last().copied(),
        _ => return None,
    };
    Some(alloc_int(store, count as i64))
}

/// Run one o' the interpreter's natives on borrowed argument handles; the result is a
/// fresh handle in the current scope (or a passed-in container's ain handle). Anything
/// the native writes tae stdout joins the runner's capture, in order.
fn call_builtin(
    state: &mut HostState,
    name: &str,
    args: &[Handle],
    line: i32,
) -> Result<Handle, String> {
    crate::logging::begin_stdout_capture();
    let result = call_builtin_inner(state, name, args, line);
    let printed = crate::logging::end_stdout_capture();
    match state.captured.as_mut() {
        Some(buffer) => buffer.push_str(&printed),
        None => print!("{}", printed),
    }
    result
}

fn call_builtin_inner(
    state: &mut HostState,
    name: &str,
    args: &[Handle],
    line: i32,
) -> Result<Handle, String> {
    let natives = state
        .natives
        .get_or_insert_with(Interpreter::native_builtins);
    let native = natives.get(name).cloned().ok_or_else(|| {
        format!(
            "Whit's aw this aboot? '{}' isnae a builtin the WASM host kens",
            name
        )
    })?;
    if native.arity != usize::MAX && args.len() != native.arity {
        return Err(HaversError::WrongArity {
            name: name.to_string(),
            expected: native.arity,
            got: args.len(),
            line: line.max(0) as usize,
        }
        .to_string());
    }
    let store = &mut state.store;
    if let Some(handle) = builtin_shortcut(store, name, args) {
        return Ok(handle);
    }
    let mut bridge = ValueBridge::default();
    let mut values = Vec::with_capacity(args.len());
    for arg in args {
        values.push(bridge.to_value(store, *arg)?);
    }
    // `tak`/`drap` take a string as a list o' its characters, as the CLI interpreter does
    // through its prelude (MDH-BUG-FLU-00170).
    if matches!(name, "tak" | "drap") {
        if let Some(Value::String(text)) = values.first() {
            let chars = text.chars().map(|c| Value::String(c.to_string())).collect();
            values[0] = Value::List(Rc::new(RefCell::new(chars)));
        }
    }
    let result = (native.func)(values).map_err(|m| HaversError::InternalError(m).to_string())?;
    // Scalar arguments and read-only natives cannae change a host container.
    if !args.iter().all(|a| is_inline_scalar(*a)) && !READ_ONLY_NATIVES.contains(&name) {
        bridge.write_back(store)?;
    }
    bridge.to_handle(store, &result)
}

/// `__mdh_builtin0` ..= `__mdh_builtin{MAX_BUILTIN_ARGS}`: (name_ptr, name_len, args...) -> handle.
fn add_builtin_imports(linker: &mut Linker<HostState>, memory: Memory) -> Result<(), String> {
    for argc in 0..=MAX_BUILTIN_ARGS {
        let params = [ValType::I32, ValType::I32]
            .into_iter()
            .chain(std::iter::repeat(ValType::I64).take(argc));
        let ty = FuncType::new(params, [ValType::I64]);
        linker
            .func_new(
                "env",
                &format!("__mdh_builtin{argc}"),
                ty,
                move |mut caller: Caller<'_, HostState>, params: &[Val], results: &mut [Val]| {
                    let ptr = params[0].i32().unwrap_or(0);
                    let len = params[1].i32().unwrap_or(0);
                    let name = read_memory_string(&caller, &memory, ptr, len);
                    let args: Vec<Handle> =
                        params[2..].iter().map(|v| v.i64().unwrap_or(0)).collect();
                    let line = guest_line(&mut caller);
                    results[0] = Val::I64(call_builtin_value(&mut caller, &name, &args, line)?);
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// The line o' the statement the guest is running: its exported `__mdh_line` global
/// (0 if the module has nane). Read only when the host raises an error.
fn guest_line(caller: &mut Caller<'_, HostState>) -> i32 {
    caller
        .get_export("__mdh_line")
        .and_then(|export| export.into_global())
        .and_then(|global| global.get(&mut *caller).i32())
        .unwrap_or(0)
}

/// Put "at line N" intae a line-less host error, as the interpreter's errors have it.
/// Messages that already name a line, or ony ither text, pass through unchanged.
fn locate_error(message: String, line: i32) -> String {
    if line <= 0 {
        return message;
    }
    let at = line as usize;
    if message == ERR_OVERFLOW {
        return HaversError::IntegerOverflow { line: at }.to_string();
    }
    if message == ERR_DIV_ZERO {
        return HaversError::DivisionByZero { line: at }.to_string();
    }
    match message.strip_prefix(TYPE_ERROR_PREFIX) {
        Some(rest) => type_error(rest.to_string(), line),
        None => message,
    }
}

const TYPE_ERROR_PREFIX: &str = "That's pure mince! Type error: ";

/// A host error message, wi' the line when the guest passed one.
fn host_error(message: String, line: i32) -> String {
    if line > 0 {
        type_error(message, line)
    } else {
        HaversError::InternalError(message).to_string()
    }
}

/// Ca' a function value: a guest closure or dae (re-entrant, through the exported
/// `__mdh_fns` table), a builtin, or a tri constructor. Errors go through `finish`, so
/// inside a `hae_a_bash` they are left pending an' the result is nil.
fn call_value(
    caller: &mut Caller<'_, HostState>,
    callee: Handle,
    args: &[Handle],
    line: i32,
) -> wasmtime::Result<Handle> {
    let value = caller.data().store.view(callee).map(Cow::into_owned);
    let result = match value {
        Some(HostValue::Function {
            index,
            arity,
            required,
            env,
            name,
        }) => {
            if required < arity {
                let shape = (index, env, None);
                return call_with_defaults(caller, shape, &name, arity, required, args, line);
            }
            if args.len() == arity {
                return call_guest(caller, index, env, args);
            }
            Err(HaversError::WrongArity {
                name,
                expected: arity,
                got: args.len(),
                line: line.max(0) as usize,
            }
            .to_string())
        }
        Some(HostValue::Builtin(name)) => return call_builtin_value(caller, &name, args, line),
        Some(HostValue::Class { .. }) => return construct(caller, callee, args, line),
        Some(HostValue::Bound { recv, method }) => {
            return call_method(caller, recv, &method, args, line)
        }
        _ => match tri_handle_call(caller.data_mut(), callee, "call", args) {
            Some(handle) => Ok(handle),
            None => Err(HaversError::NotCallable {
                name: caller.data().store.to_string(callee),
                line: line.max(0) as usize,
            }
            .to_string()),
        },
    };
    finish(caller, result)
}

/// Ca'in a class makes an instance an' runs its `init` (if ony) wi' `masel` bound.
fn construct(
    caller: &mut Caller<'_, HostState>,
    class: Handle,
    args: &[Handle],
    line: i32,
) -> wasmtime::Result<Handle> {
    let store = &mut caller.data_mut().store;
    let instance = store.alloc(HostValue::Instance {
        class,
        fields: Vec::new(),
    });
    if let Some(init) = store.find_method(class, "init") {
        call_method(caller, instance, &init, args, line)?;
    }
    Ok(instance)
}

/// Run a method on `recv` (see `call_with_defaults`).
fn call_method(
    caller: &mut Caller<'_, HostState>,
    recv: Handle,
    method: &Method,
    args: &[Handle],
    line: i32,
) -> wasmtime::Result<Handle> {
    let shape = (method.index, method.env, Some(recv));
    call_with_defaults(
        caller,
        shape,
        &method.name,
        method.params,
        method.required,
        args,
        line,
    )
}

/// Ca' a guest function that fills in its ain defaults, as the interpreter binds
/// parameters: its arity check (defaults allowed), then the function (`shape` is its
/// table slot, env an' a method's `masel`) wi' nil fer each missing argument an' the
/// count last.
fn call_with_defaults(
    caller: &mut Caller<'_, HostState>,
    shape: (u32, Handle, Option<Handle>),
    name: &str,
    params: usize,
    required: usize,
    args: &[Handle],
    line: i32,
) -> wasmtime::Result<Handle> {
    let (index, env, recv) = shape;
    let got = args.len();
    if got < required || got > params {
        let line = if line > 0 { line } else { guest_line(caller) };
        let line = line.max(0) as usize;
        let message = if required == params {
            HaversError::WrongArity {
                name: name.to_string(),
                expected: params,
                got,
                line,
            }
        } else {
            HaversError::TypeError {
                message: format!(
                    "Function '{}' expects {} tae {} arguments but ye gave it {}",
                    name, required, params, got
                ),
                line,
            }
        };
        return finish(caller, Err(message.to_string()));
    }
    let argc = caller.data_mut().store.int_handle(got as i64);
    let mut call_args = Vec::with_capacity(params + 2);
    call_args.extend(recv);
    call_args.extend_from_slice(args);
    call_args.resize(params + usize::from(recv.is_some()), 0);
    call_args.push(argc);
    call_guest(caller, index, env, &call_args)
}

/// `a op b` on an instance whose class overloads `op` (`__pit_thegither__`, ...): the
/// method's result, as the interpreter's `eval_binary` gies it.
fn operator_overload(
    caller: &mut Caller<'_, HostState>,
    import: &str,
    a: Handle,
    b: Handle,
) -> wasmtime::Result<Option<Handle>> {
    let Some(HostValue::Instance { class, .. }) = caller.data().store.get(a) else {
        return Ok(None);
    };
    let method_name = match import {
        "__mdh_add" => "__pit_thegither__",
        "__mdh_sub" => "__tak_awa__",
        "__mdh_mul" => "__times__",
        "__mdh_div" => "__pairt__",
        "__mdh_mod" => "__lave__",
        "__mdh_eq" => "__same_as__",
        "__mdh_ne" => "__differs_fae__",
        "__mdh_lt" => "__wee_er__",
        "__mdh_le" => "__wee_er_or_same__",
        "__mdh_gt" => "__muckle_er__",
        "__mdh_ge" => "__muckle_er_or_same__",
        _ => return Ok(None),
    };
    match caller.data().store.find_method(*class, method_name) {
        Some(method) => call_method(caller, a, &method, &[b], 0).map(Some),
        None => Ok(None),
    }
}

/// Run guest table entry `index` wi' its captured cells an' the arguments. The callee
/// opens an' closes its ain host scopes on top o' the caller's (see the design doc,
/// 2026.10.24 DES WASM function values), so handles the host holds stay safe.
fn call_guest(
    caller: &mut Caller<'_, HostState>,
    index: u32,
    env: Handle,
    args: &[Handle],
) -> wasmtime::Result<Handle> {
    let func = caller
        .get_export("__mdh_fns")
        .and_then(|export| export.into_table())
        .and_then(|table| table.get(&mut *caller, index))
        .and_then(|entry| match entry {
            Val::FuncRef(func) => func,
            _ => None,
        });
    let Some(func) = func else {
        return Err(trap(
            caller.data_mut(),
            "Och! That function value points naewhere in the WASM module".to_string(),
        ));
    };
    let params: Vec<Val> = std::iter::once(env)
        .chain(args.iter().copied())
        .map(Val::I64)
        .collect();
    let mut results = [Val::I64(0)];
    // The callee moves the guest's line; put it back so a later host error is located.
    let line = guest_line(caller);
    let called = func.call(&mut *caller, &params, &mut results);
    if let Some(global) = caller
        .get_export("__mdh_line")
        .and_then(|export| export.into_global())
    {
        let _ = global.set(&mut *caller, Val::I32(line));
    }
    called?;
    Ok(results[0].i64().unwrap_or(0))
}

/// A builtin ca'd by name: a higher-order one runs here, the rest through the bridge.
fn call_builtin_value(
    caller: &mut Caller<'_, HostState>,
    name: &str,
    args: &[Handle],
    line: i32,
) -> wasmtime::Result<Handle> {
    if HOF_BUILTINS.iter().any(|(hof, _)| *hof == name) {
        return call_hof(caller, name, args, line);
    }
    let result = call_builtin(caller.data_mut(), name, args, line);
    finish(caller, result)
}

/// The higher-order builtins, as `Interpreter::call_builtin_hof` does them. Each
/// callback's result is young in the caller's statement scope, which stays open while
/// the host runs; after a callback leaves an error pending the host stops at once.
fn call_hof(
    caller: &mut Caller<'_, HostState>,
    name: &str,
    args: &[Handle],
    line: i32,
) -> wasmtime::Result<Handle> {
    let arity = HOF_BUILTINS
        .iter()
        .find(|(hof, _)| *hof == name)
        .map_or(2, |(_, arity)| *arity);
    if args.len() != arity {
        let message = HaversError::WrongArity {
            name: name.to_string(),
            expected: arity,
            got: args.len(),
            line: line.max(0) as usize,
        }
        .to_string();
        return finish(caller, Err(message));
    }
    let items = match caller.data().store.get(args[0]) {
        Some(HostValue::List(items)) => items.clone(),
        _ => {
            let message = format!("{}() expects a list as first argument", name);
            return finish(caller, Err(host_error(message, line)));
        }
    };
    let func = args[arity - 1];
    let mut acc = args[1];
    let mut kept = Vec::new();
    let mut rest = Vec::new();
    let mut groups: DictEntries = Vec::new();
    for item in items {
        let call_args = if name == "tumble" {
            vec![acc, item]
        } else {
            vec![item]
        };
        let out = call_value(caller, func, &call_args, line)?;
        let state = caller.data_mut();
        if state.pending.is_some() {
            return Ok(0);
        }
        let store = &mut state.store;
        let truthy = store.is_truthy(out);
        match name {
            "gaun" => kept.push(out),
            "sieve" | "pairt_by" if truthy => kept.push(item),
            "pairt_by" => rest.push(item),
            "tumble" => acc = out,
            "hunt" if truthy => return Ok(item),
            "ony" if truthy => return Ok(TRUE_HANDLE),
            "aw" if !truthy => return Ok(FALSE_HANDLE),
            "grup_up" => {
                let key = store.key_for(out).unwrap_or(ValueKey::Nil);
                match dict_get(&groups, &key) {
                    Some(list) => {
                        if let Some(HostValue::List(members)) = store.get_mut(list) {
                            members.push(item);
                        }
                    }
                    None => {
                        let list = store.alloc(HostValue::List(vec![item]));
                        groups.push((key, list));
                    }
                }
            }
            _ => {}
        }
    }
    let store = &mut caller.data_mut().store;
    Ok(match name {
        "gaun" | "sieve" => store.alloc(HostValue::List(kept)),
        "tumble" => acc,
        "ony" => FALSE_HANDLE,
        "aw" => TRUE_HANDLE,
        "grup_up" => store.alloc(HostValue::Dict(groups)),
        "pairt_by" => {
            let matches = store.alloc(HostValue::List(kept));
            let others = store.alloc(HostValue::List(rest));
            store.alloc(HostValue::List(vec![matches, others]))
        }
        _ => 0,
    })
}

/// `obj.method(args)`: tri objects, or a dict field holdin' a function value.
fn method_call(
    caller: &mut Caller<'_, HostState>,
    obj: Handle,
    method: Handle,
    args: &[Handle],
) -> wasmtime::Result<Handle> {
    let state = caller.data_mut();
    let name = match state.store.get(method) {
        Some(HostValue::String(name)) => name.clone(),
        _ => return Ok(0),
    };
    if let Some(result) = tri_handle_call(state, obj, &name, args) {
        return Ok(result);
    }
    if let Some(HostValue::Instance { class, fields }) = state.store.get(obj) {
        // A method (bound tae `masel`) wins ower a callable field, as in the interpreter.
        let field = fields.iter().find(|(f, _)| *f == name).map(|(_, h)| *h);
        if let Some(method) = state.store.find_method(*class, &name) {
            return call_method(caller, obj, &method, args, 0);
        }
        return match field {
            Some(f) => call_value(caller, f, args, 0),
            None => {
                let line = guest_line(caller);
                finish(caller, Err(undefined(name, line)))
            }
        };
    }
    let field = match state.store.get(obj) {
        Some(HostValue::Dict(map)) => dict_get(map, &ValueKey::String(name)),
        _ => None,
    };
    match field {
        Some(f)
            if matches!(
                state.store.get(f),
                Some(HostValue::Function { .. } | HostValue::Builtin(_))
            ) =>
        {
            call_value(caller, f, args, 0)
        }
        _ => Ok(0),
    }
}

/// Function values: `__mdh_make_fn`, `__mdh_make_builtin`, the cell imports,
/// `__mdh_callN` an' `__mdh_method_callN`.
fn add_function_imports(linker: &mut Linker<HostState>, memory: Memory) -> Result<(), String> {
    let err = |e: wasmtime::Error| e.to_string();
    linker
        .func_wrap(
            "env",
            "__mdh_make_fn",
            move |mut caller: Caller<'_, HostState>,
                  index: i32,
                  arity: i32,
                  required: i32,
                  env: i64,
                  ptr: i32,
                  len: i32|
                  -> i64 {
                let name = read_memory_string(&caller, &memory, ptr, len);
                caller.data_mut().store.alloc(HostValue::Function {
                    index: index.max(0) as u32,
                    arity: arity.max(0) as usize,
                    required: required.max(0) as usize,
                    env,
                    name,
                })
            },
        )
        .map_err(err)?;
    linker
        .func_wrap(
            "env",
            "__mdh_make_builtin",
            move |mut caller: Caller<'_, HostState>, ptr: i32, len: i32| -> i64 {
                let name = read_memory_string(&caller, &memory, ptr, len);
                caller.data_mut().store.alloc(HostValue::Builtin(name))
            },
        )
        .map_err(err)?;
    linker
        .func_wrap(
            "env",
            "__mdh_cell_new",
            |mut caller: Caller<'_, HostState>, value: i64| -> i64 {
                caller.data_mut().store.alloc(HostValue::Cell(value))
            },
        )
        .map_err(err)?;
    linker
        .func_wrap(
            "env",
            "__mdh_cell_get",
            |caller: Caller<'_, HostState>, cell: i64| -> i64 {
                match caller.data().store.get(cell) {
                    Some(HostValue::Cell(value)) => *value,
                    _ => 0,
                }
            },
        )
        .map_err(err)?;
    linker
        .func_wrap(
            "env",
            "__mdh_cell_set",
            |mut caller: Caller<'_, HostState>, cell: i64, value: i64| -> i64 {
                // `get_mut` is the write barrier: an older cell may tak' a young value.
                if let Some(HostValue::Cell(slot)) = caller.data_mut().store.get_mut(cell) {
                    *slot = value;
                }
                value
            },
        )
        .map_err(err)?;
    // A global read or assigned before its `ken` has run is the interpreter's
    // "hasnae been defined" error; otherwise these are `cell_get` an' `cell_set`.
    linker
        .func_wrap(
            "env",
            "__mdh_global_get",
            move |mut caller: Caller<'_, HostState>,
                  cell: i64,
                  ptr: i32,
                  len: i32|
                  -> wasmtime::Result<i64> {
                let value = match caller.data().store.get(cell) {
                    Some(HostValue::Cell(value)) => *value,
                    _ => 0,
                };
                let result = if value == UNSET_GLOBAL {
                    let name = read_memory_string(&caller, &memory, ptr, len);
                    Err(undefined(name, guest_line(&mut caller)))
                } else {
                    Ok(value)
                };
                finish(&mut caller, result)
            },
        )
        .map_err(err)?;
    linker
        .func_wrap(
            "env",
            "__mdh_global_set",
            move |mut caller: Caller<'_, HostState>,
                  cell: i64,
                  value: i64,
                  ptr: i32,
                  len: i32|
                  -> wasmtime::Result<i64> {
                let unset = matches!(
                    caller.data().store.get(cell),
                    Some(HostValue::Cell(old)) if *old == UNSET_GLOBAL
                );
                let result = if unset {
                    let name = read_memory_string(&caller, &memory, ptr, len);
                    Err(undefined(name, guest_line(&mut caller)))
                } else {
                    // `get_mut` is the write barrier, as in `__mdh_cell_set`.
                    if let Some(HostValue::Cell(slot)) = caller.data_mut().store.get_mut(cell) {
                        *slot = value;
                    }
                    Ok(value)
                };
                finish(&mut caller, result)
            },
        )
        .map_err(err)?;
    linker
        .func_wrap(
            "env",
            "__mdh_destructure",
            |mut caller: Caller<'_, HostState>,
             value: i64,
             count: i32,
             rest: i32,
             line: i32|
             -> wasmtime::Result<i64> {
                let store = &mut caller.data_mut().store;
                let rest = usize::try_from(rest).ok();
                let result = destructure(store, value, count.max(0) as usize, rest, line);
                finish(&mut caller, result)
            },
        )
        .map_err(err)?;
    linker
        .func_wrap(
            "env",
            "__mdh_make_class",
            move |mut caller: Caller<'_, HostState>,
                  parent: i64,
                  ptr: i32,
                  len: i32,
                  parent_ptr: i32,
                  parent_len: i32,
                  line: i32|
                  -> wasmtime::Result<i64> {
                let name = read_memory_string(&caller, &memory, ptr, len);
                let store = &mut caller.data_mut().store;
                let result = if parent == 0
                    || matches!(store.get(parent), Some(HostValue::Class { .. }))
                {
                    Ok(store.alloc(HostValue::Class {
                        name,
                        parent,
                        methods: Vec::new(),
                    }))
                } else {
                    let parent_name = read_memory_string(&caller, &memory, parent_ptr, parent_len);
                    Err(type_error(format!("{} isnae a class", parent_name), line))
                };
                finish(&mut caller, result)
            },
        )
        .map_err(err)?;
    linker
        .func_wrap(
            "env",
            "__mdh_class_method",
            move |mut caller: Caller<'_, HostState>,
                  class: i64,
                  ptr: i32,
                  len: i32,
                  index: i32,
                  params: i32,
                  required: i32,
                  env: i64|
                  -> i64 {
                let name = read_memory_string(&caller, &memory, ptr, len);
                let method = Method {
                    name,
                    index: index.max(0) as u32,
                    params: params.max(0) as usize,
                    required: required.max(0) as usize,
                    env,
                };
                // The class is brand new (this statement made it), but go through the barrier.
                if let Some(HostValue::Class { methods, .. }) =
                    caller.data_mut().store.get_mut(class)
                {
                    methods.retain(|m| m.name != method.name);
                    methods.push(method);
                }
                class
            },
        )
        .map_err(err)?;
    for argc in 0..=MAX_BUILTIN_ARGS {
        let params = std::iter::repeat_n(ValType::I64, argc + 1).chain([ValType::I32]);
        let ty = FuncType::new(params, [ValType::I64]);
        linker
            .func_new(
                "env",
                &format!("__mdh_call{argc}"),
                ty,
                move |mut caller: Caller<'_, HostState>, params: &[Val], results: &mut [Val]| {
                    let callee = params[0].i64().unwrap_or(0);
                    let args: Vec<Handle> = params[1..=argc]
                        .iter()
                        .map(|v| v.i64().unwrap_or(0))
                        .collect();
                    let line = params[argc + 1].i32().unwrap_or(0);
                    results[0] = Val::I64(call_value(&mut caller, callee, &args, line)?);
                    Ok(())
                },
            )
            .map_err(err)?;
    }
    for argc in 0..=8 {
        let ty = FuncType::new(std::iter::repeat_n(ValType::I64, argc + 2), [ValType::I64]);
        linker
            .func_new(
                "env",
                &format!("__mdh_method_call{argc}"),
                ty,
                move |mut caller: Caller<'_, HostState>, params: &[Val], results: &mut [Val]| {
                    let handles: Vec<Handle> =
                        params.iter().map(|v| v.i64().unwrap_or(0)).collect();
                    let result = method_call(&mut caller, handles[0], handles[1], &handles[2..])?;
                    results[0] = Val::I64(result);
                    Ok(())
                },
            )
            .map_err(err)?;
    }
    Ok(())
}

/// The interpreter's error fer a missing field or method.
fn undefined(name: String, line: i32) -> String {
    let line = line.max(0) as usize;
    HaversError::UndefinedVariable { name, line }.to_string()
}

/// `obj.name` on an instance: a field, else a method bound tae it, else `Err(())` (missing).
fn instance_get(store: &mut HostStore, obj: Handle, name: &str) -> Option<Result<Handle, ()>> {
    let Some(HostValue::Instance { class, fields }) = store.get(obj) else {
        return None;
    };
    if let Some((_, value)) = fields.iter().find(|(f, _)| f == name) {
        return Some(Ok(*value));
    }
    Some(match store.find_method(*class, name) {
        Some(method) => Ok(store.alloc(HostValue::Bound { recv: obj, method })),
        None => Err(()),
    })
}

fn type_error(message: String, line: i32) -> String {
    HaversError::TypeError {
        message,
        line: line.max(0) as usize,
    }
    .to_string()
}

/// The value behind a handle, owned (so the store can be changed afterwards).
fn host_view(store: &HostStore, handle: Handle) -> Result<HostValue, String> {
    store
        .view(handle)
        .map(Cow::into_owned)
        .ok_or_else(|| ERR_INVALID.to_string())
}

/// A string's characters as fresh one-character strings.
fn char_handles(store: &mut HostStore, s: &str) -> Vec<Handle> {
    s.chars()
        .map(|c| alloc_string(store, c.to_string()))
        .collect()
}

/// `ken [a, _, ...rest, z] = value` like the interpreter's `exec_destructure`: a list o'
/// `count` items, one per pattern, the one at `rest` (if ony) a list o' the middle.
fn destructure(
    store: &mut HostStore,
    value: Handle,
    count: usize,
    rest: Option<usize>,
    line: i32,
) -> Result<Handle, String> {
    let items = match host_view(store, value)? {
        HostValue::List(items) => items,
        HostValue::String(s) => char_handles(store, &s),
        other => {
            let message = format!(
                "Ye can only destructure lists and strings, no' {}",
                host_type_name(&other)
            );
            return Err(type_error(message, line));
        }
    };
    let before = rest.unwrap_or(count);
    let after = rest.map_or(0, |r| count - r - 1);
    if items.len() < before + after {
        let message = format!(
            "Cannae destructure: need at least {} elements but got {}",
            before + after,
            items.len()
        );
        return Err(type_error(message, line));
    }
    let out: Vec<Handle> = (0..count)
        .map(|i| match rest {
            Some(r) if i == r => {
                let middle = items[before..items.len() - after].to_vec();
                store.alloc(HostValue::List(middle))
            }
            Some(r) if i > r => items[items.len() - (count - i)],
            _ => items[i],
        })
        .collect();
    Ok(store.alloc(HostValue::List(out)))
}

/// `object[index]` like the interpreter's `eval_index`. A list or dict hands back the
/// item's ain handle (aliasing survives); a string's character is fresh.
fn index_get(store: &mut HostStore, obj: Handle, idx: Handle, line: i32) -> Result<Handle, String> {
    let line_no = line.max(0) as usize;
    match (host_view(store, obj)?, host_view(store, idx)?) {
        (HostValue::List(items), HostValue::Int(i)) => {
            let at = if i < 0 { items.len() as i64 + i } else { i };
            usize::try_from(at)
                .ok()
                .and_then(|at| items.get(at).copied())
                .ok_or_else(|| {
                    HaversError::IndexOutOfBounds {
                        index: i,
                        size: items.len(),
                        line: line_no,
                    }
                    .to_string()
                })
        }
        (HostValue::String(s), HostValue::Int(i)) => {
            let char_len = s.chars().count();
            let at = if i < 0 { char_len as i64 + i } else { i };
            if at < 0 || at as usize >= char_len {
                return Err(HaversError::IndexOutOfBounds {
                    index: i,
                    size: char_len,
                    line: line_no,
                }
                .to_string());
            }
            let c = s.chars().nth(at as usize).unwrap_or_default();
            Ok(alloc_string(store, c.to_string()))
        }
        (HostValue::Dict(entries), _) => store
            .key_for(idx)
            .and_then(|key| dict_get(&entries, &key))
            .ok_or_else(|| {
                HaversError::UndefinedVariable {
                    name: store.to_string(idx),
                    line: line_no,
                }
                .to_string()
            }),
        (o, i) => Err(type_error(
            format!(
                "Cannae index a {} wi' a {}",
                host_type_name(&o),
                host_type_name(&i)
            ),
            line,
        )),
    }
}

/// `object[index] = value` like the interpreter's `eval_index_set`. The container is
/// changed through `get_mut`, the write barrier, so an older one keeps `value` alive.
fn index_set(
    store: &mut HostStore,
    obj: Handle,
    idx: Handle,
    value: Handle,
    line: i32,
) -> Result<Handle, String> {
    match (host_view(store, obj)?, host_view(store, idx)?) {
        (HostValue::List(items), HostValue::Int(i)) => {
            let at = if i < 0 { items.len() as i64 + i } else { i };
            if at < 0 || at as usize >= items.len() {
                return Err(HaversError::IndexOutOfBounds {
                    index: i,
                    size: items.len(),
                    line: line.max(0) as usize,
                }
                .to_string());
            }
            if let Some(HostValue::List(items)) = store.get_mut(obj) {
                items[at as usize] = value;
            }
            Ok(value)
        }
        (HostValue::Dict(_), _) => {
            let key = store.key_for(idx).ok_or_else(|| ERR_INVALID.to_string())?;
            if let Some(HostValue::Dict(entries)) = store.get_mut(obj) {
                dict_put(entries, key, value);
            }
            Ok(value)
        }
        (o, i) => Err(type_error(
            format!(
                "Cannae set index on a {} wi' a {}",
                host_type_name(&o),
                host_type_name(&i)
            ),
            line,
        )),
    }
}

/// The positions `[start:end:step]` picks fae a sequence o' `len` items, exactly as
/// the interpreter's `eval_slice` walks them.
fn slice_positions(len: usize, start: Option<i64>, end: Option<i64>, step: i64) -> Vec<usize> {
    let n = len as i64;
    let (start, end) = if step > 0 {
        (start.unwrap_or(0), end.unwrap_or(n))
    } else {
        (start.unwrap_or(-1), end.unwrap_or(-(n + 1)))
    };
    let start = if start < 0 {
        (n + start).max(0)
    } else {
        start.min(n)
    };
    let end = if end < 0 {
        (n + end).max(-1)
    } else {
        end.min(n)
    };
    let mut out = Vec::new();
    let mut i = start;
    while (step > 0 && i < end) || (step < 0 && i > end) {
        if (i as usize) < len {
            out.push(i as usize);
        }
        match i.checked_add(step) {
            Some(next) => i = next,
            None => break,
        }
    }
    out
}

/// `object[start:end:step]`; bit 0/1/2 o' `present` says which parts were written.
fn slice(
    store: &mut HostStore,
    obj: Handle,
    parts: [Handle; 3],
    present: i32,
    line: i32,
) -> Result<Handle, String> {
    let mut bounds = [None; 3];
    for (bit, (handle, name)) in parts.iter().zip(["start", "end", "step"]).enumerate() {
        if present & (1 << bit) == 0 {
            continue;
        }
        match host_view(store, *handle)? {
            HostValue::Int(i) => bounds[bit] = Some(i),
            _ => {
                return Err(type_error(
                    format!("Slice {} must be an integer", name),
                    line,
                ))
            }
        }
    }
    let step = bounds[2].unwrap_or(1);
    if step == 0 {
        return Err(type_error(
            "Slice step cannae be zero, ya dafty!".to_string(),
            line,
        ));
    }
    let value = match host_view(store, obj)? {
        HostValue::List(items) => HostValue::List(
            slice_positions(items.len(), bounds[0], bounds[1], step)
                .into_iter()
                .map(|i| items[i])
                .collect(),
        ),
        HostValue::String(s) => {
            let chars: Vec<char> = s.chars().collect();
            HostValue::String(
                slice_positions(chars.len(), bounds[0], bounds[1], step)
                    .into_iter()
                    .map(|i| chars[i])
                    .collect(),
            )
        }
        other => {
            return Err(type_error(
                format!("Cannae slice a {}, ya numpty!", host_type_name(&other)),
                line,
            ))
        }
    };
    Ok(store.alloc(value))
}

/// A `fer` loop's snapshot o' what it walks: a fresh list o' the items (or characters).
fn iter_items(store: &mut HostStore, value: Handle, line: i32) -> Result<Handle, String> {
    let items = match host_view(store, value)? {
        HostValue::List(items) => items,
        HostValue::String(s) => char_handles(store, &s),
        other => {
            return Err(type_error(
                format!("Cannae iterate ower a {}", host_type_name(&other)),
                line,
            ))
        }
    };
    Ok(store.alloc(HostValue::List(items)))
}

/// `...value` intae the list being built.
fn list_extend(
    store: &mut HostStore,
    list: Handle,
    value: Handle,
    line: i32,
) -> Result<Handle, String> {
    let extra = match host_view(store, value)? {
        HostValue::List(items) => items,
        HostValue::String(s) => char_handles(store, &s),
        _ => {
            return Err(type_error(
                "Cannae skail (spread) somethin' that isnae a list or string!".to_string(),
                line,
            ))
        }
    };
    if let Some(HostValue::List(items)) = store.get_mut(list) {
        items.extend(extra);
    }
    Ok(list)
}

/// A number as an integer the way `Value::as_integer` does it (floats truncate).
fn as_integer(store: &HostStore, handle: Handle) -> Option<i64> {
    match store.view(handle).as_deref() {
        Some(HostValue::Int(n)) => Some(*n),
        Some(HostValue::Float(f)) => Some(*f as i64),
        _ => None,
    }
}

/// An error the guest raises itself (`__mdh_raise`), as the interpreter words it.
fn raise_message(store: &HostStore, code: i32, value: Handle, line: i32) -> String {
    let message = store.to_string(value);
    let line = line.max(0) as usize;
    match code {
        RAISE_HURL => HaversError::UserError { message, line },
        RAISE_ASSERT => HaversError::AssertionFailed { message, line },
        _ => HaversError::TypeError {
            message: format!("Nae match found fer {}", message),
            line,
        },
    }
    .to_string()
}

/// Host imports fer `hae_a_bash`, `hurl`, `mak_siccar`, `keek`, indexing, slicing an' `fer`.
fn add_statement_imports(linker: &mut Linker<HostState>, memory: Memory) -> Result<(), String> {
    let err = |e: wasmtime::Error| e.to_string();
    linker
        .func_wrap(
            "env",
            "__mdh_pending",
            |caller: Caller<'_, HostState>| -> i32 { i32::from(caller.data().pending.is_some()) },
        )
        .map_err(err)?;
    linker
        .func_wrap(
            "env",
            "__mdh_try_enter",
            |mut caller: Caller<'_, HostState>| {
                let state = caller.data_mut();
                let depth = state.store.scopes.len();
                state.try_frames.push(depth);
            },
        )
        .map_err(err)?;
    linker
        .func_wrap(
            "env",
            "__mdh_try_pop",
            |mut caller: Caller<'_, HostState>, n: i32| {
                let frames = &mut caller.data_mut().try_frames;
                frames.truncate(frames.len().saturating_sub(n.max(0) as usize));
            },
        )
        .map_err(err)?;
    linker
        .func_wrap(
            "env",
            "__mdh_try_catch",
            move |mut caller: Caller<'_, HostState>,
                  ptr: i32,
                  count: i32|
                  -> wasmtime::Result<i64> {
                // Close the scopes the unwind skipped (this frame's and any callee's),
                // keepin' what the catching function's locals still reach.
                let depth = caller.data_mut().try_frames.pop().unwrap_or(0);
                let open = caller.data().store.scopes.len();
                if open > depth {
                    guest_scope_exit(&mut caller, memory, ptr, count, (open - depth) as i32)?;
                }
                let state = caller.data_mut();
                let message = state.pending.take().unwrap_or_default();
                Ok(alloc_string(&mut state.store, message))
            },
        )
        .map_err(err)?;
    linker
        .func_wrap(
            "env",
            "__mdh_raise",
            |mut caller: Caller<'_, HostState>,
             code: i32,
             value: i64,
             line: i32|
             -> wasmtime::Result<i64> {
                let message = raise_message(&caller.data().store, code, value, line);
                finish(&mut caller, Err(message))
            },
        )
        .map_err(err)?;
    linker
        .func_wrap(
            "env",
            "__mdh_index",
            |mut caller: Caller<'_, HostState>,
             obj: i64,
             idx: i64,
             line: i32|
             -> wasmtime::Result<i64> {
                let result = index_get(&mut caller.data_mut().store, obj, idx, line);
                finish(&mut caller, result)
            },
        )
        .map_err(err)?;
    linker
        .func_wrap(
            "env",
            "__mdh_index_set",
            |mut caller: Caller<'_, HostState>,
             obj: i64,
             idx: i64,
             value: i64,
             line: i32|
             -> wasmtime::Result<i64> {
                let result = index_set(&mut caller.data_mut().store, obj, idx, value, line);
                finish(&mut caller, result)
            },
        )
        .map_err(err)?;
    linker
        .func_wrap(
            "env",
            "__mdh_slice",
            |mut caller: Caller<'_, HostState>,
             obj: i64,
             start: i64,
             end: i64,
             step: i64,
             present: i32,
             line: i32|
             -> wasmtime::Result<i64> {
                let store = &mut caller.data_mut().store;
                let result = slice(store, obj, [start, end, step], present, line);
                finish(&mut caller, result)
            },
        )
        .map_err(err)?;
    linker
        .func_wrap(
            "env",
            "__mdh_iter_items",
            |mut caller: Caller<'_, HostState>, value: i64, line: i32| -> wasmtime::Result<i64> {
                let result = iter_items(&mut caller.data_mut().store, value, line);
                finish(&mut caller, result)
            },
        )
        .map_err(err)?;
    linker
        .func_wrap(
            "env",
            "__mdh_list_len",
            |caller: Caller<'_, HostState>, list: i64| -> i64 {
                match caller.data().store.get(list) {
                    Some(HostValue::List(items)) => items.len() as i64,
                    _ => 0,
                }
            },
        )
        .map_err(err)?;
    linker
        .func_wrap(
            "env",
            "__mdh_list_item",
            |caller: Caller<'_, HostState>, list: i64, i: i64| -> i64 {
                match caller.data().store.get(list) {
                    Some(HostValue::List(items)) => usize::try_from(i)
                        .ok()
                        .and_then(|i| items.get(i).copied())
                        .unwrap_or(0),
                    _ => 0,
                }
            },
        )
        .map_err(err)?;
    linker
        .func_wrap(
            "env",
            "__mdh_list_extend",
            |mut caller: Caller<'_, HostState>,
             list: i64,
             value: i64,
             line: i32|
             -> wasmtime::Result<i64> {
                let result = list_extend(&mut caller.data_mut().store, list, value, line);
                finish(&mut caller, result)
            },
        )
        .map_err(err)?;
    linker
        .func_wrap(
            "env",
            "__mdh_range_int",
            |mut caller: Caller<'_, HostState>,
             value: i64,
             other: i64,
             line: i32|
             -> wasmtime::Result<i64> {
                let store = &caller.data().store;
                let result = match (as_integer(store, value), as_integer(store, other)) {
                    (Some(n), Some(_)) => Ok(n),
                    _ => Err(type_error(
                        "Range bounds must be integers".to_string(),
                        line,
                    )),
                };
                finish(&mut caller, result)
            },
        )
        .map_err(err)?;
    linker
        .func_wrap(
            "env",
            "__mdh_match_range",
            |caller: Caller<'_, HostState>,
             value: i64,
             start: i64,
             end: i64,
             inclusive: i32|
             -> i32 {
                let store = &caller.data().store;
                let Some(HostValue::Int(n)) = store.view(value).as_deref().cloned() else {
                    return 0;
                };
                match (as_integer(store, start), as_integer(store, end)) {
                    (Some(s), Some(e)) => {
                        i32::from(n >= s && if inclusive != 0 { n <= e } else { n < e })
                    }
                    _ => 0,
                }
            },
        )
        .map_err(err)?;
    Ok(())
}

#[derive(Debug)]
struct HostState {
    store: HostStore,
    tri_modules: HashSet<Handle>,
    /// The interpreter's natives, built on the first builtin call.
    natives: Option<HashMap<String, Rc<NativeFunction>>>,
    /// `Some` when stdout is being captured (tests) instead of printed.
    captured: Option<String>,
    /// The message of the runtime error that trapped the module, if any.
    runtime_error: Option<String>,
    /// Open `hae_a_bash` bodies, innermost last: the host scope depth at entry.
    try_frames: Vec<usize>,
    /// An error raised inside a `hae_a_bash`, waiting fer the guest tae unwind tae it.
    pending: Option<String>,
}

impl HostState {
    fn new(capture: bool) -> Self {
        HostState {
            store: HostStore::new(),
            tri_modules: HashSet::new(),
            natives: None,
            captured: if capture { Some(String::new()) } else { None },
            runtime_error: None,
            try_frames: Vec::new(),
            pending: None,
        }
    }
}

/// Record a runtime error (so the caller can report it cleanly) and turn it into a trap.
fn trap(state: &mut HostState, message: String) -> wasmtime::Error {
    state.runtime_error = Some(message.clone());
    wasmtime::Error::msg(message)
}

/// A host call's result fer the guest. Inside a `hae_a_bash` an error is left pending
/// (the guest checks `__mdh_pending` straight after an' unwinds tae the catch) and the
/// call returns nil; ootside one it traps.
fn finish(
    caller: &mut Caller<'_, HostState>,
    result: Result<Handle, String>,
) -> Result<i64, wasmtime::Error> {
    let state = caller.data_mut();
    match result {
        Ok(handle) => Ok(handle),
        Err(message) if !state.try_frames.is_empty() => {
            state.pending.get_or_insert(message);
            Ok(0)
        }
        Err(message) => Err(trap(state, message)),
    }
}

fn read_memory_string(caller: &Caller<'_, HostState>, mem: &Memory, ptr: i32, len: i32) -> String {
    let data = mem.data(caller);
    let start = ptr.max(0) as usize;
    let end = start.saturating_add(len.max(0) as usize);
    if end <= data.len() {
        String::from_utf8_lossy(&data[start..end]).to_string()
    } else {
        "".to_string()
    }
}

const FALSE_HANDLE: Handle = 1;
const TRUE_HANDLE: Handle = 2;

fn alloc_bool(_store: &mut HostStore, value: bool) -> Handle {
    if value {
        TRUE_HANDLE
    } else {
        FALSE_HANDLE
    }
}

fn alloc_int(store: &mut HostStore, value: i64) -> Handle {
    store.int_handle(value)
}

fn alloc_float(store: &mut HostStore, value: f64) -> Handle {
    store.float_handle(value)
}

fn alloc_string(store: &mut HostStore, value: String) -> Handle {
    store.alloc(HostValue::String(value))
}

fn alloc_tri_ctor(store: &mut HostStore, kind: &str) -> Handle {
    store.alloc(HostValue::NativeCtor(kind.to_string()))
}

fn alloc_tri_object(store: &mut HostStore, kind: &str, args: &[Handle]) -> Handle {
    let mut obj = TriObject::new(kind);
    let type_handle = alloc_string(store, kind.to_string());
    obj.fields.insert("type".to_string(), type_handle);
    if tri_has_transform(kind) {
        let pos = alloc_tri_vec3(store, "Vec3", 0.0, 0.0, 0.0);
        let rot = alloc_tri_vec3(store, "Euler", 0.0, 0.0, 0.0);
        let scl = alloc_tri_vec3(store, "Vec3", 1.0, 1.0, 1.0);
        obj.fields.insert("position".to_string(), pos);
        obj.fields.insert("rotation".to_string(), rot);
        obj.fields.insert("scale".to_string(), scl);
        obj.fields.insert(
            "children".to_string(),
            store.alloc(HostValue::List(Vec::new())),
        );
        obj.fields.insert("parent".to_string(), 0);
    }
    apply_constructor_args(store, kind, &mut obj.fields, args);
    store.alloc(HostValue::NativeObject(obj))
}

fn alloc_tri_vec3(store: &mut HostStore, kind: &str, x: f64, y: f64, z: f64) -> Handle {
    let mut obj = TriObject::new(kind);
    let type_handle = alloc_string(store, kind.to_string());
    obj.fields.insert("type".to_string(), type_handle);
    obj.fields.insert("x".to_string(), alloc_float(store, x));
    obj.fields.insert("y".to_string(), alloc_float(store, y));
    obj.fields.insert("z".to_string(), alloc_float(store, z));
    store.alloc(HostValue::NativeObject(obj))
}

fn tri_has_transform(kind: &str) -> bool {
    matches!(
        kind,
        "Sicht"
            | "Thing3D"
            | "Clump"
            | "Mesch"
            | "Kamera"
            | "PerspectivKamera"
            | "OrthograffikKamera"
            | "Licht"
            | "AmbiantLicht"
            | "DireksionalLicht"
            | "PyntLicht"
    )
}

fn ensure_children_handle(store: &mut HostStore, obj: Handle) -> Option<Handle> {
    let existing = match store.get(obj) {
        Some(HostValue::NativeObject(native)) => native.fields.get("children").copied(),
        _ => None,
    };
    if let Some(handle) = existing {
        if matches!(store.get(handle), Some(HostValue::List(_))) {
            return Some(handle);
        }
    }
    let new_handle = store.alloc(HostValue::List(Vec::new()));
    if let Some(HostValue::NativeObject(native)) = store.get_mut(obj) {
        native.fields.insert("children".to_string(), new_handle);
    }
    Some(new_handle)
}

fn tri_object_method(store: &mut HostStore, obj: Handle, method: &str, args: &[Handle]) -> Handle {
    match method {
        "cloan" | "clone" => {
            let (kind, fields) = match store.get(obj) {
                Some(HostValue::NativeObject(native)) => {
                    (native.kind.clone(), native.fields.clone())
                }
                _ => return 0,
            };
            let clone = TriObject { kind, fields };
            store.alloc(HostValue::NativeObject(clone))
        }
        "adde" | "add" => {
            if let Some(children) = ensure_children_handle(store, obj) {
                if let Some(HostValue::List(items)) = store.get_mut(children) {
                    items.extend_from_slice(args);
                }
            }
            0
        }
        "remuiv" | "remove" => {
            if let Some(children) = ensure_children_handle(store, obj) {
                if let Some(HostValue::List(items)) = store.get_mut(children) {
                    let to_remove: Vec<Handle> = args.to_vec();
                    items.retain(|item| !to_remove.contains(item));
                }
            }
            0
        }
        "dyspos" | "dispose" => 0,
        "luik_at" | "lookAt" => {
            if let Some(target) = args.first() {
                if let Some(HostValue::NativeObject(native)) = store.get_mut(obj) {
                    native.fields.insert("lookAtTarget".to_string(), *target);
                }
            }
            0
        }
        "set_sise" | "setSize" => {
            if let Some(HostValue::NativeObject(native)) = store.get_mut(obj) {
                if let Some(width) = args.first() {
                    native.fields.insert("width".to_string(), *width);
                }
                if let Some(height) = args.get(1) {
                    native.fields.insert("height".to_string(), *height);
                }
            }
            0
        }
        "set_pixel_ratio" | "setPixelRatio" => {
            if let Some(HostValue::NativeObject(native)) = store.get_mut(obj) {
                if let Some(ratio) = args.first() {
                    native.fields.insert("pixelRatio".to_string(), *ratio);
                }
            }
            0
        }
        "render" => {
            if let Some(HostValue::NativeObject(native)) = store.get_mut(obj) {
                if let Some(scene) = args.first() {
                    native.fields.insert("scene".to_string(), *scene);
                }
                if let Some(camera) = args.get(1) {
                    native.fields.insert("camera".to_string(), *camera);
                }
            }
            0
        }
        "loop" => {
            if let Some(HostValue::NativeObject(native)) = store.get_mut(obj) {
                if let Some(callback) = args.first() {
                    native.fields.insert("loopFn".to_string(), *callback);
                }
            }
            0
        }
        _ => 0,
    }
}

fn apply_constructor_args(
    _store: &mut HostStore,
    kind: &str,
    fields: &mut HashMap<String, Handle>,
    args: &[Handle],
) {
    fn arg_or_default<F>(args: &[Handle], index: usize, default: F) -> Handle
    where
        F: FnOnce() -> Handle,
    {
        args.get(index).copied().unwrap_or_else(default)
    }

    match kind {
        "Mesch" => {
            fields.insert("geometry".to_string(), arg_or_default(args, 0, || 0));
            fields.insert("material".to_string(), arg_or_default(args, 1, || 0));
        }
        "PerspectivKamera" => {
            fields.insert(
                "fov".to_string(),
                arg_or_default(args, 0, || alloc_int(_store, 50)),
            );
            fields.insert(
                "aspect".to_string(),
                arg_or_default(args, 1, || alloc_int(_store, 1)),
            );
            fields.insert(
                "near".to_string(),
                arg_or_default(args, 2, || alloc_float(_store, 0.1)),
            );
            fields.insert(
                "far".to_string(),
                arg_or_default(args, 3, || alloc_int(_store, 2000)),
            );
        }
        "OrthograffikKamera" => {
            fields.insert(
                "left".to_string(),
                arg_or_default(args, 0, || alloc_int(_store, -1)),
            );
            fields.insert(
                "right".to_string(),
                arg_or_default(args, 1, || alloc_int(_store, 1)),
            );
            fields.insert(
                "top".to_string(),
                arg_or_default(args, 2, || alloc_int(_store, 1)),
            );
            fields.insert(
                "bottom".to_string(),
                arg_or_default(args, 3, || alloc_int(_store, -1)),
            );
            fields.insert(
                "near".to_string(),
                arg_or_default(args, 4, || alloc_float(_store, 0.1)),
            );
            fields.insert(
                "far".to_string(),
                arg_or_default(args, 5, || alloc_int(_store, 2000)),
            );
        }
        "BoxGeometrie" => {
            fields.insert(
                "width".to_string(),
                arg_or_default(args, 0, || alloc_int(_store, 1)),
            );
            fields.insert(
                "height".to_string(),
                arg_or_default(args, 1, || alloc_int(_store, 1)),
            );
            fields.insert(
                "depth".to_string(),
                arg_or_default(args, 2, || alloc_int(_store, 1)),
            );
        }
        "SpherGeometrie" => {
            fields.insert(
                "radius".to_string(),
                arg_or_default(args, 0, || alloc_int(_store, 1)),
            );
            fields.insert(
                "widthSegments".to_string(),
                arg_or_default(args, 1, || alloc_int(_store, 8)),
            );
            fields.insert(
                "heightSegments".to_string(),
                arg_or_default(args, 2, || alloc_int(_store, 6)),
            );
        }
        "Maiterial" | "MeshBasicMaiterial" | "MeshStandardMaiterial" | "Renderar" => {
            fields.insert("opts".to_string(), arg_or_default(args, 0, || 0));
        }
        "Licht" | "AmbiantLicht" | "DireksionalLicht" => {
            fields.insert("color".to_string(), arg_or_default(args, 0, || 0));
            fields.insert(
                "intensity".to_string(),
                arg_or_default(args, 1, || alloc_int(_store, 1)),
            );
        }
        "PyntLicht" => {
            fields.insert("color".to_string(), arg_or_default(args, 0, || 0));
            fields.insert(
                "intensity".to_string(),
                arg_or_default(args, 1, || alloc_int(_store, 1)),
            );
            fields.insert(
                "distance".to_string(),
                arg_or_default(args, 2, || alloc_int(_store, 0)),
            );
            fields.insert(
                "decay".to_string(),
                arg_or_default(args, 3, || alloc_int(_store, 2)),
            );
        }
        "Colour" => {
            fields.insert("value".to_string(), arg_or_default(args, 0, || 0));
        }
        _ => {}
    }
}

fn tri_constructor_kind(method: &str) -> Option<&'static str> {
    match method {
        "Sicht" => Some("Sicht"),
        "Thing3D" => Some("Thing3D"),
        "Clump" => Some("Clump"),
        "Mesch" => Some("Mesch"),
        "Kamera" => Some("Kamera"),
        "PerspectivKamera" => Some("PerspectivKamera"),
        "OrthograffikKamera" => Some("OrthograffikKamera"),
        "Geometrie" => Some("Geometrie"),
        "BoxGeometrie" => Some("BoxGeometrie"),
        "SpherGeometrie" => Some("SpherGeometrie"),
        "Maiterial" => Some("Maiterial"),
        "MeshBasicMaiterial" => Some("MeshBasicMaiterial"),
        "MeshStandardMaiterial" => Some("MeshStandardMaiterial"),
        "Licht" => Some("Licht"),
        "AmbiantLicht" => Some("AmbiantLicht"),
        "DireksionalLicht" => Some("DireksionalLicht"),
        "PyntLicht" => Some("PyntLicht"),
        "Textur" => Some("Textur"),
        "Renderar" => Some("Renderar"),
        "Colour" => Some("Colour"),
        _ => None,
    }
}

fn tri_handle_call(
    state: &mut HostState,
    obj: Handle,
    name: &str,
    args: &[Handle],
) -> Option<Handle> {
    if state.tri_modules.contains(&obj) {
        if let Some(kind) = tri_constructor_kind(name) {
            return Some(alloc_tri_object(&mut state.store, kind, args));
        }
        return Some(tri_object_method(&mut state.store, obj, name, args));
    }

    let ctor_kind = match state.store.get(obj) {
        Some(HostValue::NativeCtor(kind)) => Some(kind.clone()),
        _ => None,
    };
    if let Some(kind) = ctor_kind {
        if name == "call" || name == kind {
            return Some(alloc_tri_object(&mut state.store, &kind, args));
        }
        return Some(0);
    }

    if matches!(state.store.get(obj), Some(HostValue::NativeObject(_))) {
        return Some(tri_object_method(&mut state.store, obj, name, args));
    }

    None
}

#[derive(Clone, Copy)]
enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

fn add_arith(
    linker: &mut Linker<HostState>,
    name: &'static str,
    op: ArithOp,
) -> Result<(), String> {
    linker
        .func_wrap(
            "env",
            name,
            move |mut caller: Caller<'_, HostState>,
                  a: i64,
                  b: i64|
                  -> Result<i64, wasmtime::Error> {
                if let Some(result) = operator_overload(&mut caller, name, a, b)? {
                    return Ok(result);
                }
                let result = arith(&mut caller.data_mut().store, op, a, b);
                let result = match result {
                    Err(message) => Err(locate_error(message, guest_line(&mut caller))),
                    ok => ok,
                };
                finish(&mut caller, result)
            },
        )
        .map_err(|e| e.to_string())
        .map(|_| ())
}

fn add_cmp(
    linker: &mut Linker<HostState>,
    name: &'static str,
    op: fn(&HostStore, i64, i64) -> Result<bool, String>,
) -> Result<(), String> {
    linker
        .func_wrap(
            "env",
            name,
            move |mut caller: Caller<'_, HostState>,
                  a: i64,
                  b: i64|
                  -> Result<i64, wasmtime::Error> {
                if let Some(result) = operator_overload(&mut caller, name, a, b)? {
                    return Ok(result);
                }
                let store = &mut caller.data_mut().store;
                let result = op(store, a, b).map(|v| alloc_bool(store, v));
                let result = match result {
                    Err(message) => Err(locate_error(message, guest_line(&mut caller))),
                    ok => ok,
                };
                finish(&mut caller, result)
            },
        )
        .map_err(|e| e.to_string())
        .map(|_| ())
}

fn add_audio_stub0(linker: &mut Linker<HostState>, name: &str) -> Result<(), String> {
    linker
        .func_wrap("env", name, |_caller: Caller<'_, HostState>| -> i64 { 0 })
        .map_err(|e| e.to_string())
        .map(|_| ())
}

fn add_audio_stub1(linker: &mut Linker<HostState>, name: &str) -> Result<(), String> {
    linker
        .func_wrap(
            "env",
            name,
            |_caller: Caller<'_, HostState>, _a0: i64| -> i64 { 0 },
        )
        .map_err(|e| e.to_string())
        .map(|_| ())
}

fn add_audio_stub2(linker: &mut Linker<HostState>, name: &str) -> Result<(), String> {
    linker
        .func_wrap(
            "env",
            name,
            |_caller: Caller<'_, HostState>, _a0: i64, _a1: i64| -> i64 { 0 },
        )
        .map_err(|e| e.to_string())
        .map(|_| ())
}

fn host_type_name(value: &HostValue) -> &'static str {
    match value {
        HostValue::Nil => "naething",
        HostValue::Bool(_) => "bool",
        HostValue::Int(_) => "integer",
        HostValue::Float(_) => "float",
        HostValue::String(_) => "string",
        HostValue::List(_) => "list",
        HostValue::Dict(_) => "dict",
        HostValue::Set(_) => "creel",
        HostValue::Bytes(_) => "bytes",
        HostValue::NativeObject(_) | HostValue::NativeCtor(_) => "native",
        HostValue::Function { .. } => "function",
        HostValue::Builtin(_) => "native function",
        HostValue::Cell(_) => "cell",
        HostValue::Class { .. } => "class",
        HostValue::Instance { .. } => "instance",
        HostValue::Bound { .. } => "function",
    }
}

const ERR_OVERFLOW: &str = "Wheesht! Yer number's too muckle - it's overflowed!";
const ERR_DIV_ZERO: &str = "Ye numpty! Tryin' tae divide by zero";

/// Longest string a `string * integer` repeat may build (guards the host against OOM).
const MAX_REPEAT_BYTES: usize = 1 << 28;

/// Arithmetic wi' the interpreter's semantics: integers stay i64 wi' checked overflow,
/// integer division truncates, dividing by zero and mismatched types are errors.
fn arith(store: &mut HostStore, op: ArithOp, a: Handle, b: Handle) -> Result<Handle, String> {
    use HostValue as V;
    let (left_cow, right_cow) = match (store.view(a), store.view(b)) {
        (Some(l), Some(r)) => (l, r),
        _ => return Err("That's pure mince! Type error: invalid value".to_string()),
    };
    let (left, right): (&HostValue, &HostValue) = (&left_cow, &right_cow);
    let type_error = |what: String| format!("That's pure mince! Type error: {}", what);
    let value = match op {
        ArithOp::Add => match (left, right) {
            (V::Int(x), V::Int(y)) => V::Int(x.checked_add(*y).ok_or(ERR_OVERFLOW)?),
            (V::Float(x), V::Float(y)) => V::Float(x + y),
            (V::Int(x), V::Float(y)) => V::Float(*x as f64 + y),
            (V::Float(x), V::Int(y)) => V::Float(x + *y as f64),
            (V::String(x), _) => V::String(format!("{}{}", x, store.to_string(b))),
            (_, V::String(y)) => V::String(format!("{}{}", store.to_string(a), y)),
            (V::List(x), V::List(y)) => V::List(x.iter().chain(y.iter()).copied().collect()),
            _ => {
                return Err(type_error(format!(
                    "Cannae add {} an' {}",
                    host_type_name(left),
                    host_type_name(right)
                )))
            }
        },
        ArithOp::Sub => match (left, right) {
            (V::Int(x), V::Int(y)) => V::Int(x.checked_sub(*y).ok_or(ERR_OVERFLOW)?),
            (V::Float(x), V::Float(y)) => V::Float(x - y),
            (V::Int(x), V::Float(y)) => V::Float(*x as f64 - y),
            (V::Float(x), V::Int(y)) => V::Float(x - *y as f64),
            _ => {
                return Err(type_error(format!(
                    "Cannae subtract {} fae {}",
                    host_type_name(right),
                    host_type_name(left)
                )))
            }
        },
        ArithOp::Mul => match (left, right) {
            (V::Int(x), V::Int(y)) => V::Int(x.checked_mul(*y).ok_or(ERR_OVERFLOW)?),
            (V::Float(x), V::Float(y)) => V::Float(x * y),
            (V::Int(x), V::Float(y)) => V::Float(*x as f64 * y),
            (V::Float(x), V::Int(y)) => V::Float(x * *y as f64),
            (V::String(s), V::Int(n)) | (V::Int(n), V::String(s)) => {
                if *n < 0 {
                    return Err(type_error("Repeat count must be non-negative".to_string()));
                }
                let count = usize::try_from(*n).unwrap_or(usize::MAX);
                if s.len().saturating_mul(count) > MAX_REPEAT_BYTES {
                    return Err(type_error("Repeat count is too large".to_string()));
                }
                V::String(s.repeat(count))
            }
            _ => {
                return Err(type_error(format!(
                    "Cannae multiply {} by {}",
                    host_type_name(left),
                    host_type_name(right)
                )))
            }
        },
        ArithOp::Div => {
            if matches!(right, V::Int(0)) || matches!(right, V::Float(f) if *f == 0.0) {
                return Err(ERR_DIV_ZERO.to_string());
            }
            match (left, right) {
                (V::Int(x), V::Int(y)) => V::Int(x.checked_div(*y).ok_or(ERR_OVERFLOW)?),
                (V::Float(x), V::Float(y)) => V::Float(x / y),
                (V::Int(x), V::Float(y)) => V::Float(*x as f64 / y),
                (V::Float(x), V::Int(y)) => V::Float(x / *y as f64),
                _ => {
                    return Err(type_error(format!(
                        "Cannae divide {} by {}",
                        host_type_name(left),
                        host_type_name(right)
                    )))
                }
            }
        }
        ArithOp::Mod => {
            if matches!(right, V::Int(0)) {
                return Err(ERR_DIV_ZERO.to_string());
            }
            match (left, right) {
                (V::Int(x), V::Int(y)) => V::Int(x.checked_rem(*y).ok_or(ERR_OVERFLOW)?),
                (V::Float(x), V::Float(y)) => V::Float(x % y),
                _ => {
                    return Err(type_error(format!(
                        "Cannae get remainder o' {} by {}",
                        host_type_name(left),
                        host_type_name(right)
                    )))
                }
            }
        }
    };
    Ok(match value {
        V::Int(n) => store.int_handle(n),
        V::Float(f) => store.float_handle(f),
        other => store.alloc(other),
    })
}

/// `<`, `<=`, `>`, `>=` like the interpreter's `compare`: numbers (int/float mixed by
/// value) and strings order; anything else is a type error. `test` maps the ordering
/// to the operator's answer; a NaN operand makes every ordering false.
fn order_values(
    store: &HostStore,
    a: Handle,
    b: Handle,
    test: fn(std::cmp::Ordering) -> bool,
) -> Result<bool, String> {
    use HostValue as V;
    let (left, right) = match (store.view(a), store.view(b)) {
        (Some(l), Some(r)) => (l, r),
        _ => return Err("That's pure mince! Type error: invalid value".to_string()),
    };
    let ord = match (&*left, &*right) {
        (V::Int(x), V::Int(y)) => Some(x.cmp(y)),
        (V::Float(x), V::Float(y)) => x.partial_cmp(y),
        (V::Int(x), V::Float(y)) => (*x as f64).partial_cmp(y),
        (V::Float(x), V::Int(y)) => x.partial_cmp(&(*y as f64)),
        (V::String(x), V::String(y)) => Some(x.cmp(y)),
        (l, r) => {
            return Err(format!(
                "That's pure mince! Type error: Cannae compare {} wi' {}",
                host_type_name(l),
                host_type_name(r)
            ))
        }
    };
    Ok(ord.is_some_and(test))
}

/// Mirrors `impl PartialEq for Value`: lists an' dicts compare structurally.
fn eq_values(store: &HostStore, a: Handle, b: Handle) -> bool {
    eq_values_on_path(store, a, b, &mut Vec::new())
}

/// `path` holds the container pairs being compared right now; meeting one
/// again counts as equal, so self-referential containers finish.
fn eq_values_on_path(
    store: &HostStore,
    a: Handle,
    b: Handle,
    path: &mut Vec<(Handle, Handle)>,
) -> bool {
    let (left, right) = (store.view(a), store.view(b));
    let containers = matches!(
        (left.as_deref(), right.as_deref()),
        (Some(HostValue::List(_)), Some(HostValue::List(_)))
            | (Some(HostValue::Dict(_)), Some(HostValue::Dict(_)))
            | (Some(HostValue::Set(_)), Some(HostValue::Set(_)))
    );
    if containers {
        if a == b || path.contains(&(a, b)) {
            return true;
        }
        path.push((a, b));
        let same = match (left.as_deref(), right.as_deref()) {
            (Some(HostValue::List(x)), Some(HostValue::List(y))) => {
                x.len() == y.len()
                    && x.iter()
                        .zip(y.iter())
                        .all(|(p, q)| eq_values_on_path(store, *p, *q, path))
            }
            (Some(HostValue::Dict(x)), Some(HostValue::Dict(y))) => {
                x.len() == y.len()
                    && x.iter().all(|(key, p)| {
                        dict_get(y, key).is_some_and(|q| eq_values_on_path(store, *p, q, path))
                    })
            }
            // Creels compare by their members (decision F1).
            (Some(HostValue::Set(x)), Some(HostValue::Set(y))) => {
                x.len() == y.len() && x.iter().all(|(key, _)| dict_get(y, key).is_some())
            }
            _ => false,
        };
        path.pop();
        return same;
    }
    match (left.as_deref(), right.as_deref()) {
        (Some(HostValue::Nil), Some(HostValue::Nil)) => true,
        (Some(HostValue::Bool(x)), Some(HostValue::Bool(y))) => x == y,
        (Some(HostValue::Int(x)), Some(HostValue::Int(y))) => x == y,
        (Some(HostValue::Float(x)), Some(HostValue::Float(y))) => x == y,
        (Some(HostValue::Int(x)), Some(HostValue::Float(y))) => (*x as f64) == *y,
        (Some(HostValue::Float(x)), Some(HostValue::Int(y))) => *x == (*y as f64),
        (Some(HostValue::String(x)), Some(HostValue::String(y))) => x == y,
        (Some(HostValue::Bytes(x)), Some(HostValue::Bytes(y))) => x == y,
        _ => a == b,
    }
}

/// What a captured run produced: everything `blether` printed, and the runtime
/// error that stopped the module, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmRunOutput {
    pub stdout: String,
    pub error: Option<String>,
    /// Live slots in the host value store when the run ended (leak regression guard).
    pub store_size: usize,
    /// The most slots ever live at once (freed slots are reused before the store grows).
    pub peak_store_size: usize,
}

pub fn run_wasm_file(path: &Path) -> Result<(), String> {
    let engine = Engine::default();

    let module = match path.extension().and_then(|s| s.to_str()) {
        Some("wat") => {
            let bytes = wat::parse_file(path).map_err(|e| e.to_string())?;
            Module::new(&engine, bytes).map_err(|e| e.to_string())?
        }
        _ => Module::from_file(&engine, path).map_err(|e| e.to_string())?,
    };

    let output = run_module(&engine, &module, false)?;
    match output.error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

/// Parse (and so validate) WAT text, run it, and capture what it prints.
///
/// `Err` means the module did not load (bad WAT, unresolved import, no `main`);
/// a trap while running is reported in `WasmRunOutput::error`.
pub fn run_wat_captured(wat_text: &str) -> Result<WasmRunOutput, String> {
    let engine = Engine::default();
    let bytes = wat::parse_str(wat_text).map_err(|e| e.to_string())?;
    let module = Module::new(&engine, bytes).map_err(|e| e.to_string())?;
    run_module(&engine, &module, true)
}

/// Read `count` spilled root handles fae guest memory at `ptr`.
fn read_roots(
    caller: &mut Caller<'_, HostState>,
    memory: Memory,
    ptr: i32,
    count: i32,
) -> wasmtime::Result<Vec<Handle>> {
    let mut roots = Vec::with_capacity(count.max(0) as usize);
    let data = memory.data(&*caller);
    for i in 0..count.max(0) as usize {
        let start = ptr.max(0) as usize + 8 * i;
        match data
            .get(start..start + 8)
            .and_then(|b| <[u8; 8]>::try_from(b).ok())
        {
            Some(b) => roots.push(i64::from_le_bytes(b)),
            None => {
                let state = caller.data_mut();
                return Err(trap(
                    state,
                    "Scope roots lie ootside the WASM memory".to_string(),
                ));
            }
        }
    }
    Ok(roots)
}

/// `__mdh_scope_exit`: close `pop` host scopes, reading the guest's spilled roots
/// fae memory only when there is something young tae sweep.
fn guest_scope_exit(
    caller: &mut Caller<'_, HostState>,
    memory: Memory,
    ptr: i32,
    count: i32,
    pop: i32,
) -> wasmtime::Result<()> {
    let pop = pop.max(0) as usize;
    if !caller.data().store.scope_exit_needs_roots(pop) {
        // Nothing young tae sweep: just close the scopes (or report the fankle).
        let state = caller.data_mut();
        return state
            .store
            .scope_exit(&[], pop)
            .map_err(|message| trap(state, message));
    }
    let roots = read_roots(caller, memory, ptr, count)?;
    let state = caller.data_mut();
    if let Err(message) = state.store.scope_exit(&roots, pop) {
        return Err(trap(state, message));
    }
    let store = &state.store;
    state.tri_modules.retain(|h| store.is_live(*h));
    Ok(())
}

/// `obj.prop` on a dict, a tri object or the tri module (nil when absent).
fn prop_get(state: &mut HostState, obj: Handle, prop: Handle) -> Handle {
    if state.tri_modules.contains(&obj) {
        if let Some(ValueKey::String(key)) = state.store.key_for(prop) {
            if let Some(kind) = tri_constructor_kind(&key) {
                return alloc_tri_ctor(&mut state.store, kind);
            }
        }
    }
    match state.store.get(obj) {
        Some(HostValue::Dict(map)) => {
            if let Some(ValueKey::String(key)) = state.store.key_for(prop) {
                return dict_get(map, &ValueKey::String(key)).unwrap_or(0);
            }
            0
        }
        Some(HostValue::NativeObject(native)) => {
            if let Some(ValueKey::String(key)) = state.store.key_for(prop) {
                return native.fields.get(&key).copied().unwrap_or(0);
            }
            0
        }
        _ => 0,
    }
}

fn run_module(engine: &Engine, module: &Module, capture: bool) -> Result<WasmRunOutput, String> {
    let mut store = Store::new(engine, HostState::new(capture));
    let memory = Memory::new(&mut store, MemoryType::new(1, None)).map_err(|e| e.to_string())?;

    let mut linker = Linker::new(engine);
    linker
        .define(&mut store, "env", "memory", memory)
        .map_err(|e| e.to_string())?;

    // Value constructors
    linker
        .func_wrap(
            "env",
            "__mdh_make_nil",
            |mut caller: Caller<'_, HostState>| -> i64 {
                let _ = caller.data_mut();
                0
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_make_bool",
            |mut caller: Caller<'_, HostState>, value: i32| -> i64 {
                let store = &mut caller.data_mut().store;
                alloc_bool(store, value != 0)
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_make_int",
            |mut caller: Caller<'_, HostState>, value: i64| -> i64 {
                let store = &mut caller.data_mut().store;
                alloc_int(store, value)
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_make_float",
            |mut caller: Caller<'_, HostState>, value: f64| -> i64 {
                let store = &mut caller.data_mut().store;
                alloc_float(store, value)
            },
        )
        .map_err(|e| e.to_string())?;

    let mem_for_string = memory;
    linker
        .func_wrap(
            "env",
            "__mdh_make_string",
            move |mut caller: Caller<'_, HostState>, ptr: i32, len: i32| -> i64 {
                let s = read_memory_string(&caller, &mem_for_string, ptr, len);
                let store = &mut caller.data_mut().store;
                alloc_string(store, s)
            },
        )
        .map_err(|e| e.to_string())?;

    // Truthiness
    linker
        .func_wrap(
            "env",
            "__mdh_truthy",
            |mut caller: Caller<'_, HostState>, value: i64| -> i32 {
                let store = &mut caller.data_mut().store;
                if store.is_truthy(value) {
                    1
                } else {
                    0
                }
            },
        )
        .map_err(|e| e.to_string())?;

    // Arithmetic and comparisons
    add_arith(&mut linker, "__mdh_add", ArithOp::Add)?;
    add_arith(&mut linker, "__mdh_sub", ArithOp::Sub)?;
    add_arith(&mut linker, "__mdh_mul", ArithOp::Mul)?;
    add_arith(&mut linker, "__mdh_div", ArithOp::Div)?;
    add_arith(&mut linker, "__mdh_mod", ArithOp::Mod)?;

    add_cmp(&mut linker, "__mdh_eq", |store, a, b| {
        Ok(eq_values(store, a, b))
    })?;
    add_cmp(&mut linker, "__mdh_ne", |store, a, b| {
        Ok(!eq_values(store, a, b))
    })?;
    add_cmp(&mut linker, "__mdh_lt", |store, a, b| {
        order_values(store, a, b, |o| o.is_lt())
    })?;
    add_cmp(&mut linker, "__mdh_le", |store, a, b| {
        order_values(store, a, b, |o| o.is_le())
    })?;
    add_cmp(&mut linker, "__mdh_gt", |store, a, b| {
        order_values(store, a, b, |o| o.is_gt())
    })?;
    add_cmp(&mut linker, "__mdh_ge", |store, a, b| {
        order_values(store, a, b, |o| o.is_ge())
    })?;

    linker
        .func_wrap(
            "env",
            "__mdh_neg",
            |mut caller: Caller<'_, HostState>, value: i64| -> Result<i64, wasmtime::Error> {
                let store = &mut caller.data_mut().store;
                let viewed = store.view(value).map(|v| v.into_owned());
                let result = match viewed {
                    Some(HostValue::Int(n)) => match n.checked_neg() {
                        Some(neg) => Ok(alloc_int(store, neg)),
                        None => Err(ERR_OVERFLOW.to_string()),
                    },
                    Some(HostValue::Float(f)) => {
                        let neg = -f;
                        Ok(alloc_float(store, neg))
                    }
                    Some(other) => Err(format!(
                        "That's pure mince! Type error: Cannae negate a {}",
                        host_type_name(&other)
                    )),
                    None => Err("That's pure mince! Type error: invalid value".to_string()),
                };
                finish(&mut caller, result)
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_not",
            |mut caller: Caller<'_, HostState>, value: i64| -> i64 {
                let store = &mut caller.data_mut().store;
                alloc_bool(store, !store.is_truthy(value))
            },
        )
        .map_err(|e| e.to_string())?;

    // Printing
    linker
        .func_wrap(
            "env",
            "__mdh_blether",
            |mut caller: Caller<'_, HostState>, value: i64| {
                let state = caller.data_mut();
                let line = state.store.to_string(value);
                match state.captured.as_mut() {
                    Some(buffer) => {
                        buffer.push_str(&line);
                        buffer.push('\n');
                    }
                    None => println!("{}", line),
                }
            },
        )
        .map_err(|e| e.to_string())?;

    // List helpers
    linker
        .func_wrap(
            "env",
            "__mdh_make_list",
            |mut caller: Caller<'_, HostState>, _cap: i32| -> i64 {
                let store = &mut caller.data_mut().store;
                store.alloc(HostValue::List(Vec::new()))
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_list_push",
            |mut caller: Caller<'_, HostState>, list: i64, value: i64| -> i64 {
                let store = &mut caller.data_mut().store;
                if let Some(HostValue::List(items)) = store.get_mut(list) {
                    items.push(value);
                }
                list
            },
        )
        .map_err(|e| e.to_string())?;

    // Dict helpers
    linker
        .func_wrap(
            "env",
            "__mdh_make_dict",
            |mut caller: Caller<'_, HostState>| -> i64 {
                let store = &mut caller.data_mut().store;
                store.alloc(HostValue::Dict(Vec::new()))
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_scope_enter",
            |mut caller: Caller<'_, HostState>| {
                caller.data_mut().store.scope_enter();
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_scope_exit",
            move |mut caller: Caller<'_, HostState>,
                  ptr: i32,
                  count: i32,
                  pop: i32|
                  -> wasmtime::Result<()> {
                guest_scope_exit(&mut caller, memory, ptr, count, pop)
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_scope_collect_due",
            |caller: Caller<'_, HostState>| -> i32 {
                i32::from(caller.data().store.scope_collect_due())
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_scope_collect",
            move |mut caller: Caller<'_, HostState>,
                  ptr: i32,
                  count: i32|
                  -> wasmtime::Result<()> {
                let roots = read_roots(&mut caller, memory, ptr, count)?;
                let state = caller.data_mut();
                state.store.scope_collect(&roots);
                let store = &state.store;
                state.tri_modules.retain(|h| store.is_live(*h));
                Ok(())
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_scope_next",
            move |mut caller: Caller<'_, HostState>,
                  ptr: i32,
                  count: i32|
                  -> wasmtime::Result<()> {
                guest_scope_exit(&mut caller, memory, ptr, count, 1)?;
                caller.data_mut().store.scope_enter();
                Ok(())
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_tri_module",
            |mut caller: Caller<'_, HostState>| -> i64 {
                let state = caller.data_mut();
                let module_handle = state.store.alloc(HostValue::Dict(Vec::new()));
                let deg = alloc_float(&mut state.store, std::f64::consts::PI / 180.0);
                let rad = alloc_float(&mut state.store, 180.0 / std::f64::consts::PI);
                if let Some(HostValue::Dict(map)) = state.store.get_mut(module_handle) {
                    dict_put(map, ValueKey::String("DEG_TO_RAD".to_string()), deg);
                    dict_put(map, ValueKey::String("RAD_TO_DEG".to_string()), rad);
                }
                state.tri_modules.insert(module_handle);
                module_handle
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_dict_set",
            |mut caller: Caller<'_, HostState>, dict: i64, key: i64, value: i64| -> i64 {
                let store = &mut caller.data_mut().store;
                let key_value = store.key_for(key);
                if let Some(HostValue::Dict(map)) = store.get_mut(dict) {
                    if let Some(k) = key_value {
                        dict_put(map, k, value);
                    }
                }
                dict
            },
        )
        .map_err(|e| e.to_string())?;

    // Property access
    linker
        .func_wrap(
            "env",
            "__mdh_prop_get",
            |mut caller: Caller<'_, HostState>, obj: i64, prop: i64| -> wasmtime::Result<i64> {
                let state = caller.data_mut();
                if let Some(ValueKey::String(key)) = state.store.key_for(prop) {
                    if let Some(result) = instance_get(&mut state.store, obj, &key) {
                        let result = match result {
                            Ok(handle) => Ok(handle),
                            Err(()) => Err(undefined(key, guest_line(&mut caller))),
                        };
                        return finish(&mut caller, result);
                    }
                }
                Ok(prop_get(state, obj, prop))
            },
        )
        .map_err(|e| e.to_string())?;
    linker
        .func_wrap(
            "env",
            "__mdh_prop_set",
            |mut caller: Caller<'_, HostState>, obj: i64, prop: i64, value: i64| -> i64 {
                let store = &mut caller.data_mut().store;
                let prop_key = store.key_for(prop);
                match store.get_mut(obj) {
                    Some(HostValue::Dict(map)) => {
                        if let Some(ValueKey::String(key)) = prop_key {
                            dict_put(map, ValueKey::String(key), value);
                        }
                    }
                    Some(HostValue::NativeObject(native)) => {
                        if let Some(ValueKey::String(key)) = prop_key {
                            native.fields.insert(key, value);
                        }
                    }
                    // `get_mut` above is the write barrier fer the field's new value.
                    Some(HostValue::Instance { fields, .. }) => {
                        if let Some(ValueKey::String(key)) = prop_key {
                            match fields.iter_mut().find(|(f, _)| *f == key) {
                                Some(slot) => slot.1 = value,
                                None => fields.push((key, value)),
                            }
                        }
                    }
                    _ => {}
                }
                value
            },
        )
        .map_err(|e| e.to_string())?;

    add_function_imports(&mut linker, memory)?;
    add_builtin_imports(&mut linker, memory)?;
    add_statement_imports(&mut linker, memory)?;

    // Audio stubs come from the compiler's own import table so the two cannot drift.
    for (name, arity) in AUDIO_IMPORTS {
        match *arity {
            0 => add_audio_stub0(&mut linker, name)?,
            1 => add_audio_stub1(&mut linker, name)?,
            2 => add_audio_stub2(&mut linker, name)?,
            other => {
                return Err(format!(
                    "No audio stub fer {} wi' {} arguments",
                    name, other
                ))
            }
        }
    }

    let instance = linker
        .instantiate(&mut store, module)
        .map_err(|e| e.to_string())?;

    let outcome = if let Ok(func) = instance.get_typed_func::<(), i64>(&mut store, "main") {
        func.call(&mut store, ()).map(|_| ())
    } else if let Ok(func) = instance.get_typed_func::<(), ()>(&mut store, "_start") {
        func.call(&mut store, ())
    } else {
        return Err("WASM module lacks exported 'main' or '_start'".to_string());
    };

    let state = store.data_mut();
    let stdout = state.captured.take().unwrap_or_default();
    let store_size = state.store.live_count();
    let peak_store_size = state.store.values.len();
    let error = outcome.err().map(|e| {
        state
            .runtime_error
            .take()
            .unwrap_or_else(|| format!("{:#}", e))
    });
    Ok(WasmRunOutput {
        stdout,
        error,
        store_size,
        peak_store_size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nil_and_bools_are_interned_and_never_grow_the_store() {
        let mut store = HostStore::new();
        let before = store.values.len();
        let t1 = alloc_bool(&mut store, true);
        let t2 = alloc_bool(&mut store, true);
        let f1 = alloc_bool(&mut store, false);
        assert_eq!(t1, t2);
        assert_ne!(t1, f1);
        assert_eq!(store.values.len(), before);
        assert!(matches!(store.get(t1), Some(HostValue::Bool(true))));
        assert!(matches!(store.get(f1), Some(HostValue::Bool(false))));
        assert!(matches!(store.get(0), Some(HostValue::Nil)));
    }

    #[test]
    fn dict_put_keeps_insertion_order_and_replaces_in_place() {
        let mut entries: DictEntries = Vec::new();
        dict_put(&mut entries, ValueKey::String("b".to_string()), 1);
        dict_put(&mut entries, ValueKey::String("a".to_string()), 2);
        dict_put(&mut entries, ValueKey::String("b".to_string()), 3);
        assert_eq!(
            entries,
            vec![
                (ValueKey::String("b".to_string()), 3),
                (ValueKey::String("a".to_string()), 2)
            ]
        );
        assert_eq!(
            dict_get(&entries, &ValueKey::String("a".to_string())),
            Some(2)
        );
        assert_eq!(dict_get(&entries, &ValueKey::String("z".to_string())), None);
    }

    #[test]
    fn arith_follows_the_interpreter() {
        let mut store = HostStore::new();
        let seven = alloc_int(&mut store, 7);
        let two = alloc_int(&mut store, 2);
        let zero = alloc_int(&mut store, 0);
        let q = arith(&mut store, ArithOp::Div, seven, two).unwrap();
        assert!(matches!(store.view(q).as_deref(), Some(HostValue::Int(3))));
        assert!(arith(&mut store, ArithOp::Div, seven, zero)
            .unwrap_err()
            .contains("divide by zero"));
        let max = alloc_int(&mut store, i64::MAX);
        assert!(arith(&mut store, ArithOp::Add, max, two)
            .unwrap_err()
            .contains("overflowed"));
        let text = alloc_string(&mut store, "a".to_string());
        assert!(arith(&mut store, ArithOp::Sub, seven, text)
            .unwrap_err()
            .contains("Cannae subtract"));
    }

    // MDH-BUG-FLU-00118: a native's stdout line joins the runner's capture.
    #[test]
    fn a_native_that_writes_stdout_joins_the_captured_output() {
        let mut state = HostState::new(true);
        let chatter = NativeFunction::new("chatter", 0, |_args| {
            crate::logging::stdout_line("hullo fae a native");
            Ok(Value::Nil)
        });
        let mut natives = HashMap::new();
        natives.insert("chatter".to_string(), Rc::new(chatter));
        state.natives = Some(natives);
        state.captured.as_mut().unwrap().push_str("before\n");
        call_builtin(&mut state, "chatter", &[], 0).unwrap();
        assert_eq!(
            state.captured.as_deref(),
            Some("before\nhullo fae a native\n")
        );
    }

    fn guest_fn(store: &mut HostStore, name: &str) -> Handle {
        store.alloc(HostValue::Function {
            index: 0,
            arity: 0,
            required: 0,
            env: 0,
            name: name.to_string(),
        })
    }

    // MDH-BUG-FLU-00127: a dict o' lambdas goes through a builtin, an' its function
    // values come back as the very same handles.
    #[test]
    fn function_values_round_trip_through_the_builtin_bridge() {
        let mut state = HostState::new(true);
        let go = guest_fn(&mut state.store, "go");
        let look = state.store.alloc(HostValue::Builtin("len".to_string()));
        let key = alloc_string(&mut state.store, "go".to_string());
        let dict = state.store.alloc(HostValue::Dict(vec![
            (ValueKey::String("go".to_string()), go),
            (ValueKey::String("look".to_string()), look),
        ]));
        let found = call_builtin(&mut state, "contains", &[dict, key], 0).unwrap();
        assert!(matches!(
            state.store.get(found),
            Some(HostValue::Bool(true))
        ));
        let values = call_builtin(&mut state, "values", &[dict], 0).unwrap();
        assert!(matches!(
            state.store.get(values),
            Some(HostValue::List(items)) if items == &[go, look]
        ));
        let kind = call_builtin(&mut state, "whit_kind", &[go], 0).unwrap();
        assert_eq!(state.store.to_string(kind), "function");
        let shown = call_builtin(&mut state, "tae_string", &[look], 0).unwrap();
        assert_eq!(state.store.to_string(shown), "<native dae len>");
    }

    // The write-back o' a mutated container puts the identical function handles back.
    #[test]
    fn write_back_restores_function_handles() {
        let mut state = HostState::new(true);
        let a = guest_fn(&mut state.store, "a");
        let b = guest_fn(&mut state.store, "b");
        let list = state.store.alloc(HostValue::List(vec![a, b, a]));
        let popped = call_builtin(&mut state, "yank", &[list], 0).unwrap();
        assert_eq!(popped, a);
        assert!(matches!(
            state.store.get(list),
            Some(HostValue::List(items)) if items == &[a, b]
        ));
        let reversed = call_builtin(&mut state, "reverse", &[list], 0).unwrap();
        assert!(matches!(
            state.store.get(reversed),
            Some(HostValue::List(items)) if items == &[b, a]
        ));
        let fn_key = state
            .store
            .alloc(HostValue::Dict(vec![(ValueKey::Function(a), b)]));
        let keys = call_builtin(&mut state, "keys", &[fn_key], 0).unwrap();
        assert!(matches!(
            state.store.get(keys),
            Some(HostValue::List(items)) if items == &[a]
        ));
    }

    // A native that tries tae ca' a carried host value gets a clear error.
    #[test]
    fn a_carried_host_value_cannot_be_called_by_a_native() {
        let opaque = HostOpaque {
            handle: 7,
            kind: "function",
            shown: "<dae go>".to_string(),
        };
        let err = opaque.call("call", Vec::new()).unwrap_err().to_string();
        assert!(err.contains("cannae ca'"), "{err}");
        assert_eq!(opaque.to_string(), "<dae go>");
    }

    #[test]
    fn shove_shortcut_pushes_on_the_host_list() {
        let mut state = HostState::new(true);
        let list = state.store.alloc(HostValue::List(Vec::new()));
        call_builtin(&mut state, "shove", &[list, 5 - INT_BIAS], 0).unwrap();
        assert!(matches!(
            state.store.get(list),
            Some(HostValue::List(items)) if items == &[5 - INT_BIAS]
        ));
    }
}
