use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::rc::Rc;

use wasmtime::{Caller, Engine, FuncType, Linker, Memory, MemoryType, Module, Store, Val, ValType};

use crate::error::HaversError;
use crate::interpreter::Interpreter;
use crate::value::{DictValue, NativeFunction, Value};
use crate::wasm_compiler::{AUDIO_IMPORTS, MAX_BUILTIN_ARGS};

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
    NativeObject(TriObject),
    NativeCtor(String),
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
    NativeObject(Handle),
    NativeCtor(Handle),
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
            HostValue::NativeObject(_) => Some(ValueKey::NativeObject(handle)),
            HostValue::NativeCtor(_) => Some(ValueKey::NativeCtor(handle)),
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
            Some(HostValue::NativeObject(_)) => true,
            Some(HostValue::NativeCtor(_)) => true,
            None => false,
        }
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
                        ValueKey::String(s) => s.clone(),
                        _ => format!("{:?}", key),
                    };
                    parts.push(format!("\"{}\": {}", key_str, self.to_string(*value)));
                }
                format!("{{{}}}", parts.join(", "))
            }
            Some(HostValue::NativeObject(obj)) => format!("<native {}>", obj.kind),
            Some(HostValue::NativeCtor(kind)) => format!("<native dae tri.{}>", kind),
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
                    | ValueKey::NativeObject(h)
                    | ValueKey::NativeCtor(h) => out.push(*h),
                    _ => {}
                }
                out.push(*value);
            }
        }
        HostValue::NativeObject(obj) => out.extend(obj.fields.values().copied()),
        _ => {}
    }
}

/// The Rc address o' a list or dict, its identity across a builtin call.
fn container_ptr(value: &Value) -> Option<usize> {
    match value {
        Value::List(rc) => Some(Rc::as_ptr(rc) as *const () as usize),
        Value::Dict(rc) => Some(Rc::as_ptr(rc) as *const () as usize),
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
            other => Err(format!(
                "That's pure mince! Type error: a {} cannae be passed tae a builtin in WASM yet",
                host_type_name(other)
            )),
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
            | ValueKey::NativeObject(h)
            | ValueKey::NativeCtor(h) => self.to_value(store, *h)?,
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
            Value::List(_) | Value::Dict(_) => {
                let handle = store.alloc(HostValue::List(Vec::new()));
                if let Some(ptr) = container_ptr(value) {
                    self.by_ptr.insert(ptr, handle);
                }
                self.fill(store, handle, value)?;
                Ok(handle)
            }
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
            (ValueKey::List(h) | ValueKey::Dict(h), _) => self.same_slot(store, *h, value),
            (ValueKey::Nil, Value::Nil) => true,
            (ValueKey::Bool(a), Value::Bool(b)) => a == b,
            (ValueKey::Int(a), Value::Integer(b)) => a == b,
            (ValueKey::Float(a), Value::Float(b)) => *a == b.to_bits(),
            (ValueKey::String(a), Value::String(b)) => a == b,
            _ => false,
        }
    }
}

/// Run one o' the interpreter's natives on borrowed argument handles; the result is a
/// fresh handle in the current scope (or a passed-in container's ain handle).
fn call_builtin(state: &mut HostState, name: &str, args: &[Handle]) -> Result<Handle, String> {
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
        return Err(format!(
            "Yer bum's oot the windae! Function '{}' expects {} arguments but ye gave it {}",
            name,
            native.arity,
            args.len()
        ));
    }
    let store = &mut state.store;
    let mut bridge = ValueBridge::default();
    let mut values = Vec::with_capacity(args.len());
    for arg in args {
        values.push(bridge.to_value(store, *arg)?);
    }
    let result = (native.func)(values).map_err(|m| HaversError::InternalError(m).to_string())?;
    bridge.write_back(store)?;
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
                    let result = call_builtin(caller.data_mut(), &name, &args);
                    results[0] = Val::I64(finish(&mut caller, result)?);
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
    }
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
}

impl HostState {
    fn new(capture: bool) -> Self {
        HostState {
            store: HostStore::new(),
            tri_modules: HashSet::new(),
            natives: None,
            captured: if capture { Some(String::new()) } else { None },
            runtime_error: None,
        }
    }
}

/// Record a runtime error (so the caller can report it cleanly) and turn it into a trap.
fn trap(state: &mut HostState, message: String) -> wasmtime::Error {
    state.runtime_error = Some(message.clone());
    wasmtime::Error::msg(message)
}

fn finish(
    caller: &mut Caller<'_, HostState>,
    result: Result<Handle, String>,
) -> Result<i64, wasmtime::Error> {
    result.map_err(|message| trap(caller.data_mut(), message))
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
                let result = arith(&mut caller.data_mut().store, op, a, b);
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
                let store = &mut caller.data_mut().store;
                let result = op(store, a, b).map(|v| alloc_bool(store, v));
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
        HostValue::NativeObject(_) | HostValue::NativeCtor(_) => "native",
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
            |mut caller: Caller<'_, HostState>, obj: i64, prop: i64| -> i64 {
                let state = caller.data_mut();
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
                    _ => {}
                }
                value
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_method_call0",
            move |mut caller: Caller<'_, HostState>, obj: i64, method: i64| -> i64 {
                let state = caller.data_mut();
                let name = match state.store.get(method) {
                    Some(HostValue::String(name)) => name.clone(),
                    _ => return 0,
                };
                if let Some(result) = tri_handle_call(state, obj, &name, &[]) {
                    return result;
                }
                0
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_method_call1",
            move |mut caller: Caller<'_, HostState>, obj: i64, method: i64, a0: i64| -> i64 {
                let state = caller.data_mut();
                let name = match state.store.get(method) {
                    Some(HostValue::String(name)) => name.clone(),
                    _ => return 0,
                };
                let args = [a0];
                if let Some(result) = tri_handle_call(state, obj, &name, &args) {
                    return result;
                }
                0
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_method_call2",
            move |mut caller: Caller<'_, HostState>,
                  obj: i64,
                  method: i64,
                  a0: i64,
                  a1: i64|
                  -> i64 {
                let state = caller.data_mut();
                let name = match state.store.get(method) {
                    Some(HostValue::String(name)) => name.clone(),
                    _ => return 0,
                };
                let args = [a0, a1];
                if let Some(result) = tri_handle_call(state, obj, &name, &args) {
                    return result;
                }
                0
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_method_call3",
            move |mut caller: Caller<'_, HostState>,
                  obj: i64,
                  method: i64,
                  a0: i64,
                  a1: i64,
                  a2: i64|
                  -> i64 {
                let state = caller.data_mut();
                let name = match state.store.get(method) {
                    Some(HostValue::String(name)) => name.clone(),
                    _ => return 0,
                };
                let args = [a0, a1, a2];
                if let Some(result) = tri_handle_call(state, obj, &name, &args) {
                    return result;
                }
                0
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_method_call4",
            move |mut caller: Caller<'_, HostState>,
                  obj: i64,
                  method: i64,
                  a0: i64,
                  a1: i64,
                  a2: i64,
                  a3: i64|
                  -> i64 {
                let state = caller.data_mut();
                let name = match state.store.get(method) {
                    Some(HostValue::String(name)) => name.clone(),
                    _ => return 0,
                };
                let args = [a0, a1, a2, a3];
                if let Some(result) = tri_handle_call(state, obj, &name, &args) {
                    return result;
                }
                0
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_method_call5",
            move |mut caller: Caller<'_, HostState>,
                  obj: i64,
                  method: i64,
                  a0: i64,
                  a1: i64,
                  a2: i64,
                  a3: i64,
                  a4: i64|
                  -> i64 {
                let state = caller.data_mut();
                let name = match state.store.get(method) {
                    Some(HostValue::String(name)) => name.clone(),
                    _ => return 0,
                };
                let args = [a0, a1, a2, a3, a4];
                if let Some(result) = tri_handle_call(state, obj, &name, &args) {
                    return result;
                }
                0
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_method_call6",
            move |mut caller: Caller<'_, HostState>,
                  obj: i64,
                  method: i64,
                  a0: i64,
                  a1: i64,
                  a2: i64,
                  a3: i64,
                  a4: i64,
                  a5: i64|
                  -> i64 {
                let state = caller.data_mut();
                let name = match state.store.get(method) {
                    Some(HostValue::String(name)) => name.clone(),
                    _ => return 0,
                };
                let args = [a0, a1, a2, a3, a4, a5];
                if let Some(result) = tri_handle_call(state, obj, &name, &args) {
                    return result;
                }
                0
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_method_call7",
            move |mut caller: Caller<'_, HostState>,
                  obj: i64,
                  method: i64,
                  a0: i64,
                  a1: i64,
                  a2: i64,
                  a3: i64,
                  a4: i64,
                  a5: i64,
                  a6: i64|
                  -> i64 {
                let state = caller.data_mut();
                let name = match state.store.get(method) {
                    Some(HostValue::String(name)) => name.clone(),
                    _ => return 0,
                };
                let args = [a0, a1, a2, a3, a4, a5, a6];
                if let Some(result) = tri_handle_call(state, obj, &name, &args) {
                    return result;
                }
                0
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_method_call8",
            move |mut caller: Caller<'_, HostState>,
                  obj: i64,
                  method: i64,
                  a0: i64,
                  a1: i64,
                  a2: i64,
                  a3: i64,
                  a4: i64,
                  a5: i64,
                  a6: i64,
                  a7: i64|
                  -> i64 {
                let state = caller.data_mut();
                let name = match state.store.get(method) {
                    Some(HostValue::String(name)) => name.clone(),
                    _ => return 0,
                };
                let args = [a0, a1, a2, a3, a4, a5, a6, a7];
                if let Some(result) = tri_handle_call(state, obj, &name, &args) {
                    return result;
                }
                0
            },
        )
        .map_err(|e| e.to_string())?;

    add_builtin_imports(&mut linker, memory)?;

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
}
