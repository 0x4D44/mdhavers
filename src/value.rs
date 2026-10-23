use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::Rc;

use crate::ast::{Expr, Stmt};
use crate::error::HaversResult;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueKey {
    Nil,
    Bool(bool),
    Int(i64),
    Float(u64),
    String(String),
    List(usize),
    Dict(usize),
    Set(usize),
    Bytes(usize),
    Function(usize),
    NativeFunction(usize),
    Class(usize),
    Instance(usize),
    Struct(usize),
    NativeObject(usize),
    Range {
        start: i64,
        end: i64,
        inclusive: bool,
    },
}

pub trait NativeObject: fmt::Debug {
    fn type_name(&self) -> &str;
    fn get(&self, prop: &str) -> HaversResult<Value>;
    fn set(&self, prop: &str, value: Value) -> HaversResult<Value>;
    fn call(&self, method: &str, args: Vec<Value>) -> HaversResult<Value>;
    fn as_any(&self) -> &dyn Any;
    fn to_string(&self) -> String {
        format!("<native {}>", self.type_name())
    }
}

/// Runtime values in mdhavers
///
/// `Debug` is written by hand (see `debug_value`) so it terminates on
/// self-referential containers and never walks a function's closure.
#[derive(Clone)]
pub enum Value {
    /// Integer number
    Integer(i64),
    /// Floating point number
    Float(f64),
    /// String
    String(String),
    /// Boolean (aye/nae)
    Bool(bool),
    /// Null value (naething)
    Nil,
    /// List/Array
    List(Rc<RefCell<Vec<Value>>>),
    /// Dictionary/Map
    Dict(Rc<RefCell<DictValue>>),
    /// Set (creel = basket/set in Scots)
    Set(Rc<RefCell<SetValue>>),
    /// Function
    Function(Rc<HaversFunction>),
    /// Native/built-in function
    NativeFunction(Rc<NativeFunction>),
    /// Class
    Class(Rc<HaversClass>),
    /// Instance of a class
    Instance(Rc<RefCell<HaversInstance>>),
    /// Struct definition
    Struct(Rc<HaversStruct>),
    /// Native object with property/method hooks
    NativeObject(Rc<dyn NativeObject>),
    /// Range iterator
    #[allow(dead_code)]
    Range(RangeValue),
    /// Byte buffer
    #[allow(dead_code)]
    Bytes(Rc<RefCell<Vec<u8>>>),
}

/// Move the children of a uniquely-owned container into `out`, leaving the
/// container empty, so teardown can proceed iteratively (MDH-BUG-FLU-00085).
fn take_children(v: &mut Value, out: &mut Vec<Value>) {
    match v {
        Value::List(rc) if Rc::strong_count(rc) == 1 => {
            if let Ok(mut b) = rc.try_borrow_mut() {
                out.append(&mut b);
            }
        }
        Value::Dict(rc) if Rc::strong_count(rc) == 1 => {
            if let Ok(mut b) = rc.try_borrow_mut() {
                b.index.clear();
                for (k, val) in std::mem::take(&mut b.entries) {
                    out.push(k);
                    out.push(val);
                }
            }
        }
        Value::Set(rc) if Rc::strong_count(rc) == 1 => {
            if let Ok(mut b) = rc.try_borrow_mut() {
                out.extend(std::mem::take(&mut b.items).into_values());
            }
        }
        Value::Instance(rc) if Rc::strong_count(rc) == 1 => {
            if let Ok(mut b) = rc.try_borrow_mut() {
                out.extend(std::mem::take(&mut b.fields).into_values());
            }
        }
        _ => {}
    }
}

impl Drop for Value {
    /// Iterative teardown: a million-deep nested list would otherwise recurse
    /// once per level in the compiler-generated drop glue and overflow the
    /// native stack.
    fn drop(&mut self) {
        let mut stack = Vec::new();
        take_children(self, &mut stack);
        while let Some(mut v) = stack.pop() {
            take_children(&mut v, &mut stack);
            // `v` drops here with its children already removed, so its own
            // Drop finds nothing to do.
        }
    }
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Bool(_) => "bool",
            Value::Nil => "naething",
            Value::List(_) => "list",
            Value::Dict(_) => "dict",
            Value::Set(_) => "creel",
            Value::Bytes(_) => "bytes",
            Value::Function(_) => "function",
            Value::NativeFunction(_) => "native function",
            Value::Class(_) => "class",
            Value::Instance(_) => "instance",
            Value::Struct(_) => "struct",
            Value::Range(_) => "range",
            Value::NativeObject(_) => "native object",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Nil => false,
            Value::Integer(0) => false,
            Value::Float(f) if *f == 0.0 => false,
            Value::String(s) if s.is_empty() => false,
            Value::List(l) if l.borrow().is_empty() => false,
            Value::Set(s) if s.borrow().is_empty() => false,
            Value::Bytes(b) if b.borrow().is_empty() => false,
            _ => true,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(n) => Some(*n),
            Value::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Integer(n) => Some(*n as f64),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn as_list(&self) -> Option<&Rc<RefCell<Vec<Value>>>> {
        match self {
            Value::List(list) => Some(list),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn as_dict(&self) -> Option<&Rc<RefCell<DictValue>>> {
        match self {
            Value::Dict(dict) => Some(dict),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn as_set(&self) -> Option<&Rc<RefCell<SetValue>>> {
        match self {
            Value::Set(set) => Some(set),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn as_bytes(&self) -> Option<&Rc<RefCell<Vec<u8>>>> {
        match self {
            Value::Bytes(bytes) => Some(bytes),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn as_native_function(&self) -> Option<Rc<NativeFunction>> {
        match self {
            Value::NativeFunction(func) => Some(Rc::clone(func)),
            _ => None,
        }
    }

    pub fn as_key(&self) -> ValueKey {
        match self {
            Value::Nil => ValueKey::Nil,
            Value::Bool(b) => ValueKey::Bool(*b),
            Value::Integer(n) => ValueKey::Int(*n),
            Value::Float(f) => ValueKey::Float(f.to_bits()),
            Value::String(s) => ValueKey::String(s.clone()),
            Value::List(l) => ValueKey::List(Rc::as_ptr(l) as usize),
            Value::Dict(d) => ValueKey::Dict(Rc::as_ptr(d) as usize),
            Value::Set(s) => ValueKey::Set(Rc::as_ptr(s) as usize),
            Value::Bytes(b) => ValueKey::Bytes(Rc::as_ptr(b) as usize),
            Value::Function(func) => ValueKey::Function(Rc::as_ptr(func) as usize),
            Value::NativeFunction(func) => ValueKey::NativeFunction(Rc::as_ptr(func) as usize),
            Value::Class(class) => ValueKey::Class(Rc::as_ptr(class) as usize),
            Value::Instance(inst) => ValueKey::Instance(Rc::as_ptr(inst) as usize),
            Value::Struct(s) => ValueKey::Struct(Rc::as_ptr(s) as usize),
            Value::NativeObject(obj) => {
                ValueKey::NativeObject(Rc::as_ptr(obj) as *const () as usize)
            }
            Value::Range(r) => ValueKey::Range {
                start: r.start,
                end: r.end,
                inclusive: r.inclusive,
            },
        }
    }
}

/// Containers nested deeper than this print as `[...]` / `{...}` rather
/// than risk overflowing the native stack.
const MAX_DISPLAY_DEPTH: usize = 1000;

/// Write a value, tracking the containers already being written (`open`)
/// so a self-referential list or dict prints `[...]` / `{...}` instead of
/// recursing forever.
fn write_value(value: &Value, out: &mut dyn fmt::Write, open: &mut Vec<usize>) -> fmt::Result {
    match value {
        Value::Integer(n) => write!(out, "{}", n),
        Value::Float(n) => write!(out, "{}", n),
        Value::String(s) => write!(out, "{}", s),
        Value::Bool(true) => write!(out, "aye"),
        Value::Bool(false) => write!(out, "nae"),
        Value::Nil => write!(out, "naething"),
        Value::List(items) => {
            let id = Rc::as_ptr(items) as *const () as usize;
            if open.contains(&id) || open.len() >= MAX_DISPLAY_DEPTH {
                return write!(out, "[...]");
            }
            open.push(id);
            let items = items.borrow();
            write!(out, "[")?;
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ")?;
                }
                write_value(item, out, open)?;
            }
            open.pop();
            write!(out, "]")
        }
        Value::Dict(map) => {
            let id = Rc::as_ptr(map) as *const () as usize;
            if open.contains(&id) || open.len() >= MAX_DISPLAY_DEPTH {
                return write!(out, "{{...}}");
            }
            open.push(id);
            let map = map.borrow();
            write!(out, "{{")?;
            for (i, (k, v)) in map.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ")?;
                }
                write!(out, "\"")?;
                write_value(k, out, open)?;
                write!(out, "\": ")?;
                write_value(v, out, open)?;
            }
            open.pop();
            write!(out, "}}")
        }
        Value::Set(set) => {
            let id = Rc::as_ptr(set) as *const () as usize;
            if open.contains(&id) || open.len() >= MAX_DISPLAY_DEPTH {
                return write!(out, "creel{{...}}");
            }
            open.push(id);
            let set = set.borrow();
            let mut strs: Vec<String> = Vec::with_capacity(set.len());
            for item in set.iter() {
                let mut s = String::new();
                write_value(item, &mut s, open)?;
                strs.push(s);
            }
            open.pop();
            strs.sort(); // Sort fer consistent display
            write!(out, "creel{{")?;
            for (i, s) in strs.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ")?;
                }
                write!(out, "\"{}\"", s)?;
            }
            write!(out, "}}")
        }
        Value::Bytes(bytes) => {
            let len = bytes.borrow().len();
            write!(out, "bytes[{}]", len)
        }
        Value::Function(func) => write!(out, "<dae {}>", func.name),
        Value::NativeFunction(func) => write!(out, "<native dae {}>", func.name),
        Value::Class(class) => write!(out, "<kin {}>", class.name),
        Value::Instance(inst) => write!(out, "<{} instance>", inst.borrow().class.name),
        Value::Struct(s) => write!(out, "<thing {}>", s.name),
        Value::Range(r) => write!(out, "{}..{}", r.start, r.end),
        Value::NativeObject(obj) => write!(out, "{}", obj.to_string()),
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_value(self, f, &mut Vec::new())
    }
}

// ---------- Debug ----------

/// Containers already being written by `debug_value`, by address.
type DebugOpen = RefCell<Vec<usize>>;

/// Marks a container as open for the life of the guard. `None` if it is
/// already open (a cycle) or the nesting is past `MAX_DISPLAY_DEPTH`.
struct DebugEnter<'a>(&'a DebugOpen);

impl<'a> DebugEnter<'a> {
    fn new(open: &'a DebugOpen, id: usize) -> Option<Self> {
        let mut stack = open.borrow_mut();
        if stack.contains(&id) || stack.len() >= MAX_DISPLAY_DEPTH {
            return None;
        }
        stack.push(id);
        Some(DebugEnter(open))
    }
}

impl Drop for DebugEnter<'_> {
    fn drop(&mut self) {
        self.0.borrow_mut().pop();
    }
}

/// A value plus the shared open-container set, so nested `Debug` calls
/// (including pretty `{:#?}` ones) keep tracking cycles.
struct DebugWith<'a> {
    value: &'a Value,
    open: &'a DebugOpen,
}

impl fmt::Debug for DebugWith<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_value(self.value, f, self.open)
    }
}

/// Writes its text as-is (for `...` and pre-rendered parts).
struct DebugRaw<'a>(&'a str);

impl fmt::Debug for DebugRaw<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

/// `RefCell { value: <inner> }`, the shape derived Debug gave containers.
struct DebugRefCell<T>(T);

impl<T: fmt::Debug> fmt::Debug for DebugRefCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RefCell").field("value", &self.0).finish()
    }
}

struct DebugItems<'a> {
    items: &'a [Value],
    open: &'a DebugOpen,
}

impl fmt::Debug for DebugItems<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.items.iter().map(|value| DebugWith {
                value,
                open: self.open,
            }))
            .finish()
    }
}

struct DebugDict<'a> {
    dict: &'a DictValue,
    open: &'a DebugOpen,
}

impl fmt::Debug for DebugDict<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries: Vec<(DebugWith<'_>, DebugWith<'_>)> = self
            .dict
            .iter()
            .map(|(k, v)| {
                (
                    DebugWith {
                        value: k,
                        open: self.open,
                    },
                    DebugWith {
                        value: v,
                        open: self.open,
                    },
                )
            })
            .collect();
        f.debug_struct("DictValue")
            .field("entries", &entries)
            .finish()
    }
}

/// Fields of an instance, sorted by name so the output is stable.
struct DebugFields<'a> {
    fields: &'a HashMap<String, Value>,
    open: &'a DebugOpen,
}

impl fmt::Debug for DebugFields<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut names: Vec<&String> = self.fields.keys().collect();
        names.sort();
        f.debug_map()
            .entries(names.into_iter().map(|name| {
                (
                    name,
                    DebugWith {
                        value: &self.fields[name],
                        open: self.open,
                    },
                )
            }))
            .finish()
    }
}

/// `<dae name @0x...>`: a function or class by name and identity. Their
/// closures hold environments that usually contain the function itself.
struct DebugIdentity<'a> {
    kind: &'a str,
    name: &'a str,
    ptr: *const (),
}

impl fmt::Debug for DebugIdentity<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{} {} @{:p}>", self.kind, self.name, self.ptr)
    }
}

/// Cycle-safe `Debug` for a value. Scalars, structs, ranges, bytes and
/// native values print as derived `Debug` did; lists keep the
/// `List(RefCell { value: [...] })` shape. A list, dict, creel or
/// instance met again inside itself (or nested past `MAX_DISPLAY_DEPTH`)
/// prints as `List(...)` / `Dict(...)` / `Set(...)` / `Instance(...)`.
/// Dicts list their entries in order, creels their items sorted, instances
/// their class name and sorted fields; functions and classes print by name
/// and address rather than walking their closures.
fn debug_value(value: &Value, f: &mut fmt::Formatter<'_>, open: &DebugOpen) -> fmt::Result {
    let elided = DebugRaw("...");
    match value {
        Value::Integer(n) => f.debug_tuple("Integer").field(n).finish(),
        Value::Float(x) => f.debug_tuple("Float").field(x).finish(),
        Value::String(s) => f.debug_tuple("String").field(s).finish(),
        Value::Bool(b) => f.debug_tuple("Bool").field(b).finish(),
        Value::Nil => f.write_str("Nil"),
        Value::Range(r) => f.debug_tuple("Range").field(r).finish(),
        Value::Bytes(b) => f.debug_tuple("Bytes").field(b).finish(),
        Value::NativeFunction(func) => f.debug_tuple("NativeFunction").field(func).finish(),
        Value::NativeObject(obj) => f.debug_tuple("NativeObject").field(obj).finish(),
        Value::Struct(s) => f.debug_tuple("Struct").field(s).finish(),
        Value::Function(func) => f
            .debug_tuple("Function")
            .field(&DebugIdentity {
                kind: "dae",
                name: &func.name,
                ptr: Rc::as_ptr(func) as *const (),
            })
            .finish(),
        Value::Class(class) => f
            .debug_tuple("Class")
            .field(&DebugIdentity {
                kind: "kin",
                name: &class.name,
                ptr: Rc::as_ptr(class) as *const (),
            })
            .finish(),
        Value::List(items) => {
            let Some(_entered) = DebugEnter::new(open, Rc::as_ptr(items) as *const () as usize)
            else {
                return f.debug_tuple("List").field(&elided).finish();
            };
            match items.try_borrow() {
                Ok(items) => f
                    .debug_tuple("List")
                    .field(&DebugRefCell(DebugItems {
                        items: &items,
                        open,
                    }))
                    .finish(),
                Err(_) => f
                    .debug_tuple("List")
                    .field(&DebugRefCell(DebugRaw("<borrowed>")))
                    .finish(),
            }
        }
        Value::Dict(dict) => {
            let Some(_entered) = DebugEnter::new(open, Rc::as_ptr(dict) as *const () as usize)
            else {
                return f.debug_tuple("Dict").field(&elided).finish();
            };
            match dict.try_borrow() {
                Ok(dict) => f
                    .debug_tuple("Dict")
                    .field(&DebugRefCell(DebugDict { dict: &dict, open }))
                    .finish(),
                Err(_) => f
                    .debug_tuple("Dict")
                    .field(&DebugRefCell(DebugRaw("<borrowed>")))
                    .finish(),
            }
        }
        Value::Set(set) => {
            let Some(_entered) = DebugEnter::new(open, Rc::as_ptr(set) as *const () as usize)
            else {
                return f.debug_tuple("Set").field(&elided).finish();
            };
            let Ok(set) = set.try_borrow() else {
                return f
                    .debug_tuple("Set")
                    .field(&DebugRefCell(DebugRaw("<borrowed>")))
                    .finish();
            };
            // A creel's own order is a hash order; sort for stable output.
            let mut items: Vec<String> = set
                .iter()
                .map(|value| format!("{:?}", DebugWith { value, open }))
                .collect();
            items.sort();
            let items: Vec<DebugRaw<'_>> = items.iter().map(|s| DebugRaw(s)).collect();
            f.debug_tuple("Set").field(&DebugRefCell(items)).finish()
        }
        Value::Instance(inst) => {
            let Some(_entered) = DebugEnter::new(open, Rc::as_ptr(inst) as *const () as usize)
            else {
                return f.debug_tuple("Instance").field(&elided).finish();
            };
            let Ok(inst) = inst.try_borrow() else {
                return f
                    .debug_tuple("Instance")
                    .field(&DebugRefCell(DebugRaw("<borrowed>")))
                    .finish();
            };
            f.debug_tuple("Instance")
                .field(&DebugRefCell(DebugInstance {
                    class: &inst.class.name,
                    fields: DebugFields {
                        fields: &inst.fields,
                        open,
                    },
                }))
                .finish()
        }
    }
}

struct DebugInstance<'a> {
    class: &'a str,
    fields: DebugFields<'a>,
}

impl fmt::Debug for DebugInstance<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HaversInstance")
            .field("class", &self.class)
            .field("fields", &self.fields)
            .finish()
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_value(self, f, &RefCell::new(Vec::new()))
    }
}

// ---------- Equality ----------

/// Address pair identifying two lists being compared.
fn list_pair(a: &Rc<RefCell<Vec<Value>>>, b: &Rc<RefCell<Vec<Value>>>) -> (usize, usize) {
    (
        Rc::as_ptr(a) as *const () as usize,
        Rc::as_ptr(b) as *const () as usize,
    )
}

/// One pair of lists on the comparison path, and the next index to check.
struct ListEqFrame {
    a: Rc<RefCell<Vec<Value>>>,
    b: Rc<RefCell<Vec<Value>>>,
    pair: (usize, usize),
    next: usize,
}

/// Paths shorter than this are searched linearly for repeats; deeper ones
/// also keep a hash set so very deep lists stay linear time.
const LIST_EQ_LINEAR_DEPTH: usize = 64;

/// Structural list equality that terminates on cyclic lists and works on
/// any nesting depth. It walks the lists with an explicit heap stack rather
/// than native recursion, so a list nested a million deep cannae overflow
/// the native stack (PartialEq has no way to report an error).
///
/// The path holds the (left, right) list pairs being compared right now.
/// Meeting one again means the comparison has come full circle without
/// finding a difference, so that pair counts as equal (bisimulation).
fn lists_equal(a: &Rc<RefCell<Vec<Value>>>, b: &Rc<RefCell<Vec<Value>>>) -> bool {
    if a.borrow().len() != b.borrow().len() {
        return false;
    }
    let mut path = vec![ListEqFrame {
        a: a.clone(),
        b: b.clone(),
        pair: list_pair(a, b),
        next: 0,
    }];
    // Pairs on the path below LIST_EQ_LINEAR_DEPTH (only filled once the
    // path gets that deep).
    let mut deep: HashSet<(usize, usize)> = HashSet::new();

    loop {
        let Some(top) = path.last_mut() else {
            return true;
        };
        // Look at the next element pair of the innermost lists.
        let child = {
            let xs = top.a.borrow();
            let ys = top.b.borrow();
            let i = top.next;
            if i >= xs.len() || i >= ys.len() {
                None
            } else {
                top.next += 1;
                match (&xs[i], &ys[i]) {
                    (Value::List(xl), Value::List(yl)) => Some((xl.clone(), yl.clone())),
                    (x, y) => {
                        // Not both lists, so this does not recurse.
                        if x != y {
                            return false;
                        }
                        continue;
                    }
                }
            }
        };
        match child {
            None => {
                // This pair is equal; leave it.
                let done = path.pop().expect("path is not empty");
                if path.len() >= LIST_EQ_LINEAR_DEPTH {
                    deep.remove(&done.pair);
                }
            }
            Some((xl, yl)) => {
                let pair = list_pair(&xl, &yl);
                let on_path = path
                    .iter()
                    .take(LIST_EQ_LINEAR_DEPTH)
                    .any(|f| f.pair == pair)
                    || deep.contains(&pair);
                if on_path {
                    continue;
                }
                if xl.borrow().len() != yl.borrow().len() {
                    return false;
                }
                if path.len() >= LIST_EQ_LINEAR_DEPTH {
                    deep.insert(pair);
                }
                path.push(ListEqFrame {
                    a: xl,
                    b: yl,
                    pair,
                    next: 0,
                });
            }
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Integer(a), Value::Float(b)) => (*a as f64) == *b,
            (Value::Float(a), Value::Integer(b)) => *a == (*b as f64),
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::List(a), Value::List(b)) => lists_equal(a, b),
            (Value::Dict(a), Value::Dict(b)) => Rc::ptr_eq(a, b),
            (Value::Set(a), Value::Set(b)) => Rc::ptr_eq(a, b),
            (Value::Bytes(a), Value::Bytes(b)) => *a.borrow() == *b.borrow(),
            (Value::Function(a), Value::Function(b)) => Rc::ptr_eq(a, b),
            (Value::NativeFunction(a), Value::NativeFunction(b)) => Rc::ptr_eq(a, b),
            (Value::Class(a), Value::Class(b)) => Rc::ptr_eq(a, b),
            (Value::Instance(a), Value::Instance(b)) => Rc::ptr_eq(a, b),
            (Value::Struct(a), Value::Struct(b)) => Rc::ptr_eq(a, b),
            (Value::NativeObject(a), Value::NativeObject(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DictValue {
    index: HashMap<ValueKey, usize>,
    entries: Vec<(Value, Value)>,
}

impl DictValue {
    pub fn new() -> Self {
        DictValue {
            index: HashMap::new(),
            entries: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn contains_key(&self, key: &Value) -> bool {
        self.index.contains_key(&key.as_key())
    }

    pub fn get(&self, key: &Value) -> Option<&Value> {
        self.index
            .get(&key.as_key())
            .and_then(|&idx| self.entries.get(idx))
            .map(|(_, v)| v)
    }

    pub fn set(&mut self, key: Value, value: Value) {
        let key_id = key.as_key();
        if let Some(&idx) = self.index.get(&key_id) {
            self.entries[idx].1 = value;
            return;
        }

        let idx = self.entries.len();
        self.entries.push((key, value));
        self.index.insert(key_id, idx);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Value, &Value)> {
        self.entries.iter().map(|(k, v)| (k, v))
    }

    pub fn keys(&self) -> impl Iterator<Item = &Value> {
        self.entries.iter().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl Iterator<Item = &Value> {
        self.entries.iter().map(|(_, v)| v)
    }

    pub fn remove(&mut self, key: &Value) -> Option<Value> {
        let key_id = key.as_key();
        let idx = self.index.remove(&key_id)?;
        let (_k, v) = self.entries.remove(idx);

        // Rebuild index for shifted entries.
        self.index.clear();
        for (i, (k, _)) in self.entries.iter().enumerate() {
            self.index.insert(k.as_key(), i);
        }

        Some(v)
    }
}

impl Default for DictValue {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SetValue {
    items: HashMap<ValueKey, Value>,
}

impl SetValue {
    pub fn new() -> Self {
        SetValue {
            items: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn insert(&mut self, value: Value) -> bool {
        let key = value.as_key();
        self.items.insert(key, value).is_none()
    }

    pub fn remove(&mut self, value: &Value) -> bool {
        self.items.remove(&value.as_key()).is_some()
    }

    pub fn contains(&self, value: &Value) -> bool {
        self.items.contains_key(&value.as_key())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Value> {
        self.items.values()
    }

    pub fn is_subset(&self, other: &SetValue) -> bool {
        self.items.keys().all(|k| other.items.contains_key(k))
    }

    pub fn is_superset(&self, other: &SetValue) -> bool {
        other.is_subset(self)
    }

    pub fn is_disjoint(&self, other: &SetValue) -> bool {
        self.items.keys().all(|k| !other.items.contains_key(k))
    }

    pub fn intersection(&self, other: &SetValue) -> SetValue {
        let mut out = SetValue::new();
        for (k, v) in &self.items {
            if other.items.contains_key(k) {
                out.items.insert(k.clone(), v.clone());
            }
        }
        out
    }

    pub fn difference(&self, other: &SetValue) -> SetValue {
        let mut out = SetValue::new();
        for (k, v) in &self.items {
            if !other.items.contains_key(k) {
                out.items.insert(k.clone(), v.clone());
            }
        }
        out
    }

    pub fn union(&self, other: &SetValue) -> SetValue {
        let mut out = SetValue::new();
        for (k, v) in &self.items {
            out.items.insert(k.clone(), v.clone());
        }
        for (k, v) in &other.items {
            out.items.insert(k.clone(), v.clone());
        }
        out
    }
}

impl Default for SetValue {
    fn default() -> Self {
        Self::new()
    }
}

/// A function parameter with optional default value (fer runtime)
#[derive(Debug, Clone)]
pub struct FunctionParam {
    pub name: String,
    pub default: Option<Expr>,
}

/// A user-defined function
#[derive(Debug)]
pub struct HaversFunction {
    pub name: String,
    pub params: Vec<FunctionParam>,
    pub body: Vec<Stmt>,
    pub closure: Option<Rc<RefCell<Environment>>>,
}

impl HaversFunction {
    pub fn new(
        name: String,
        params: Vec<FunctionParam>,
        body: Vec<Stmt>,
        closure: Option<Rc<RefCell<Environment>>>,
    ) -> Self {
        HaversFunction {
            name,
            params,
            body,
            closure,
        }
    }

    /// Count the minimum number of required arguments (those wi'oot defaults)
    pub fn min_arity(&self) -> usize {
        self.params.iter().filter(|p| p.default.is_none()).count()
    }

    /// Maximum number of arguments (all params)
    pub fn max_arity(&self) -> usize {
        self.params.len()
    }
}

/// A native/built-in function
pub struct NativeFunction {
    pub name: String,
    pub arity: usize,
    pub func: Box<dyn Fn(Vec<Value>) -> Result<Value, String>>,
    /// A higher-order builtin (gaun, sieve, ...) that needs the interpreter
    /// to call its function argument; the interpreter dispatches it by name
    /// instead of calling `func`.
    pub higher_order: bool,
}

impl NativeFunction {
    pub fn new<F>(name: &str, arity: usize, func: F) -> Self
    where
        F: Fn(Vec<Value>) -> Result<Value, String> + 'static,
    {
        NativeFunction {
            name: name.to_string(),
            arity,
            func: Box::new(func),
            higher_order: false,
        }
    }

    /// A higher-order builtin. Its `func` only reports that it needs the
    /// interpreter; `Interpreter::call_value` runs the real thing.
    pub fn higher_order(name: &str, arity: usize) -> Self {
        let owned = name.to_string();
        NativeFunction {
            name: owned.clone(),
            arity,
            func: Box::new(move |_| {
                Err(format!(
                    "{}() needs the interpreter tae ca' its function",
                    owned
                ))
            }),
            higher_order: true,
        }
    }
}

impl fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NativeFunction({})", self.name)
    }
}

/// A class definition
#[derive(Debug)]
pub struct HaversClass {
    pub name: String,
    pub superclass: Option<Rc<HaversClass>>,
    pub methods: HashMap<String, Rc<HaversFunction>>,
}

impl HaversClass {
    pub fn new(name: String, superclass: Option<Rc<HaversClass>>) -> Self {
        HaversClass {
            name,
            superclass,
            methods: HashMap::new(),
        }
    }

    pub fn find_method(&self, name: &str) -> Option<Rc<HaversFunction>> {
        if let Some(method) = self.methods.get(name) {
            return Some(method.clone());
        }
        if let Some(superclass) = &self.superclass {
            return superclass.find_method(name);
        }
        None
    }
}

/// An instance of a class
#[derive(Debug)]
pub struct HaversInstance {
    pub class: Rc<HaversClass>,
    pub fields: HashMap<String, Value>,
}

impl HaversInstance {
    pub fn new(class: Rc<HaversClass>) -> Self {
        HaversInstance {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.fields.get(name) {
            return Some(value.clone());
        }
        if let Some(method) = self.class.find_method(name) {
            return Some(Value::Function(method));
        }
        None
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.fields.insert(name, value);
    }
}

/// A struct definition
#[derive(Debug)]
pub struct HaversStruct {
    pub name: String,
    pub fields: Vec<String>,
}

impl HaversStruct {
    pub fn new(name: String, fields: Vec<String>) -> Self {
        HaversStruct { name, fields }
    }
}

/// A range value
#[derive(Debug, Clone)]
pub struct RangeValue {
    pub start: i64,
    pub end: i64,
    pub inclusive: bool,
}

impl RangeValue {
    #[allow(dead_code)]
    pub fn new(start: i64, end: i64, inclusive: bool) -> Self {
        RangeValue {
            start,
            end,
            inclusive,
        }
    }

    pub fn iter(&self) -> RangeIterator {
        RangeIterator {
            current: self.start,
            end: self.end,
            inclusive: self.inclusive,
            done: false,
        }
    }
}

pub struct RangeIterator {
    current: i64,
    end: i64,
    inclusive: bool,
    /// Set once the last value is handed out, so `..=i64::MAX` never overflows
    done: bool,
}

impl RangeIterator {
    /// Items still to come, as an i128 so no range can overflow it
    fn remaining(&self) -> i128 {
        if self.done {
            return 0;
        }
        let span = self.end as i128 - self.current as i128 + i128::from(self.inclusive);
        span.max(0)
    }
}

impl Iterator for RangeIterator {
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining() == 0 {
            return None;
        }
        let val = self.current;
        match self.current.checked_add(1) {
            Some(next) => self.current = next,
            None => self.done = true,
        }
        Some(val)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match usize::try_from(self.remaining()) {
            Ok(n) => (n, Some(n)),
            Err(_) => (usize::MAX, None),
        }
    }
}

/// Environment for variable bindings
#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.values.get(name) {
            return Some(value.clone());
        }
        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get(name);
        }
        None
    }

    pub fn assign(&mut self, name: &str, value: Value) -> bool {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            return true;
        }
        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
        }
        false
    }

    /// Get all values defined in this environment (not including enclosing)
    /// Used fer module exports
    pub fn get_exports(&self) -> HashMap<String, Value> {
        self.values.clone()
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::approx_constant)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestNative;

    impl NativeObject for TestNative {
        fn type_name(&self) -> &str {
            "test_native"
        }

        fn get(&self, _prop: &str) -> HaversResult<Value> {
            Ok(Value::Nil)
        }

        fn set(&self, _prop: &str, _value: Value) -> HaversResult<Value> {
            Ok(Value::Nil)
        }

        fn call(&self, _method: &str, _args: Vec<Value>) -> HaversResult<Value> {
            Ok(Value::Nil)
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[test]
    fn test_native_object_default_to_string_is_covered() {
        let native = TestNative;
        let obj: &dyn NativeObject = &native;
        assert_eq!(obj.to_string(), "<native test_native>");
    }

    // ==================== Value::type_name() Tests ====================

    #[test]
    fn test_value_type_name_all_types() {
        assert_eq!(Value::Integer(42).type_name(), "integer");
        assert_eq!(Value::Float(3.14).type_name(), "float");
        assert_eq!(Value::String("hello".to_string()).type_name(), "string");
        assert_eq!(Value::Bool(true).type_name(), "bool");
        assert_eq!(Value::Bool(false).type_name(), "bool");
        assert_eq!(Value::Nil.type_name(), "naething");

        let list = Value::List(Rc::new(RefCell::new(vec![Value::Integer(1)])));
        assert_eq!(list.type_name(), "list");

        let dict = Value::Dict(Rc::new(RefCell::new(DictValue::new())));
        assert_eq!(dict.type_name(), "dict");

        let set = Value::Set(Rc::new(RefCell::new(SetValue::new())));
        assert_eq!(set.type_name(), "creel");

        let bytes = Value::Bytes(Rc::new(RefCell::new(vec![1u8, 2u8, 3u8])));
        assert_eq!(bytes.type_name(), "bytes");

        let func = HaversFunction::new("test".to_string(), vec![], vec![], None);
        assert_eq!(Value::Function(Rc::new(func)).type_name(), "function");

        let native = NativeFunction::new("native", 0, |_| Ok(Value::Nil));
        assert_eq!((native.func)(vec![]).unwrap(), Value::Nil);
        assert_eq!(
            Value::NativeFunction(Rc::new(native)).type_name(),
            "native function"
        );

        let class = HaversClass::new("TestClass".to_string(), None);
        assert_eq!(Value::Class(Rc::new(class)).type_name(), "class");

        let class2 = Rc::new(HaversClass::new("TestClass".to_string(), None));
        let instance = HaversInstance::new(class2);
        assert_eq!(
            Value::Instance(Rc::new(RefCell::new(instance))).type_name(),
            "instance"
        );

        let strct = HaversStruct::new("TestStruct".to_string(), vec![]);
        assert_eq!(Value::Struct(Rc::new(strct)).type_name(), "struct");

        let range = RangeValue::new(0, 10, false);
        assert_eq!(Value::Range(range).type_name(), "range");

        let native = Value::NativeObject(Rc::new(TestNative));
        assert_eq!(native.type_name(), "native object");
    }

    #[test]
    fn test_value_as_helpers_cover_some_and_none_branches() {
        let list_rc = Rc::new(RefCell::new(vec![Value::Integer(1)]));
        let list = Value::List(Rc::clone(&list_rc));
        assert!(list.as_list().is_some());
        assert!(Rc::ptr_eq(list.as_list().unwrap(), &list_rc));
        assert!(Value::Nil.as_list().is_none());

        let dict_rc = Rc::new(RefCell::new(DictValue::new()));
        let dict = Value::Dict(Rc::clone(&dict_rc));
        assert!(dict.as_dict().is_some());
        assert!(Rc::ptr_eq(dict.as_dict().unwrap(), &dict_rc));
        assert!(Value::Nil.as_dict().is_none());

        let set_rc = Rc::new(RefCell::new(SetValue::new()));
        let set = Value::Set(Rc::clone(&set_rc));
        assert!(set.as_set().is_some());
        assert!(Rc::ptr_eq(set.as_set().unwrap(), &set_rc));
        assert!(Value::Nil.as_set().is_none());

        let bytes_rc = Rc::new(RefCell::new(vec![1u8, 2u8, 3u8]));
        let bytes = Value::Bytes(Rc::clone(&bytes_rc));
        assert!(bytes.as_bytes().is_some());
        assert!(Rc::ptr_eq(bytes.as_bytes().unwrap(), &bytes_rc));
        assert!(Value::Nil.as_bytes().is_none());

        let native_rc = Rc::new(NativeFunction::new("native", 0, |_| Ok(Value::Nil)));
        assert_eq!((&native_rc.func)(vec![]).unwrap(), Value::Nil);
        let native = Value::NativeFunction(Rc::clone(&native_rc));
        assert!(native.as_native_function().is_some());
        assert!(Rc::ptr_eq(
            &native.as_native_function().unwrap(),
            &native_rc
        ));
        assert!(Value::Nil.as_native_function().is_none());
    }

    // ==================== Value::is_truthy() Tests ====================

    #[test]
    fn test_value_is_truthy() {
        // Booleans
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Bool(false).is_truthy());

        // Nil is always falsy
        assert!(!Value::Nil.is_truthy());

        // Integer 0 is falsy, others truthy
        assert!(!Value::Integer(0).is_truthy());
        assert!(Value::Integer(1).is_truthy());
        assert!(Value::Integer(-1).is_truthy());
        assert!(Value::Integer(42).is_truthy());

        // Float 0.0 is falsy, others truthy
        assert!(!Value::Float(0.0).is_truthy());
        assert!(Value::Float(0.1).is_truthy());
        assert!(Value::Float(-0.1).is_truthy());
        assert!(Value::Float(3.14).is_truthy());

        // Empty string is falsy, non-empty truthy
        assert!(!Value::String("".to_string()).is_truthy());
        assert!(Value::String("hello".to_string()).is_truthy());
        assert!(Value::String(" ".to_string()).is_truthy());

        // Empty list is falsy, non-empty truthy
        let empty_list = Value::List(Rc::new(RefCell::new(vec![])));
        let non_empty_list = Value::List(Rc::new(RefCell::new(vec![Value::Integer(1)])));
        assert!(!empty_list.is_truthy());
        assert!(non_empty_list.is_truthy());

        // Empty set is falsy, non-empty truthy
        let empty_set = Value::Set(Rc::new(RefCell::new(SetValue::new())));
        let mut non_empty = SetValue::new();
        non_empty.insert(Value::String("item".to_string()));
        let non_empty_set = Value::Set(Rc::new(RefCell::new(non_empty)));
        assert!(!empty_set.is_truthy());
        assert!(non_empty_set.is_truthy());

        // Dict is always truthy (even if empty - default case)
        let empty_dict = Value::Dict(Rc::new(RefCell::new(DictValue::new())));
        assert!(empty_dict.is_truthy());

        // Bytes are falsy when empty
        let empty_bytes = Value::Bytes(Rc::new(RefCell::new(Vec::new())));
        let non_empty_bytes = Value::Bytes(Rc::new(RefCell::new(vec![1, 2, 3])));
        assert!(!empty_bytes.is_truthy());
        assert!(non_empty_bytes.is_truthy());

        // Functions are truthy
        let func = HaversFunction::new("test".to_string(), vec![], vec![], None);
        assert!(Value::Function(Rc::new(func)).is_truthy());

        // Classes and instances are truthy
        let class = Rc::new(HaversClass::new("Test".to_string(), None));
        assert!(Value::Class(class.clone()).is_truthy());
        let instance = HaversInstance::new(class);
        assert!(Value::Instance(Rc::new(RefCell::new(instance))).is_truthy());
    }

    // ==================== Value::as_* Conversion Tests ====================

    #[test]
    fn test_value_as_integer() {
        assert_eq!(Value::Integer(42).as_integer(), Some(42));
        assert_eq!(Value::Float(3.7).as_integer(), Some(3)); // truncates
        assert_eq!(Value::Float(3.2).as_integer(), Some(3));
        assert_eq!(Value::String("hello".to_string()).as_integer(), None);
        assert_eq!(Value::Bool(true).as_integer(), None);
        assert_eq!(Value::Nil.as_integer(), None);
    }

    #[test]
    fn test_value_as_float() {
        assert_eq!(Value::Float(3.14).as_float(), Some(3.14));
        assert_eq!(Value::Integer(42).as_float(), Some(42.0));
        assert_eq!(Value::String("hello".to_string()).as_float(), None);
        assert_eq!(Value::Bool(true).as_float(), None);
        assert_eq!(Value::Nil.as_float(), None);
    }

    #[test]
    fn test_value_as_string() {
        assert_eq!(
            Value::String("hello".to_string()).as_string(),
            Some("hello")
        );
        assert_eq!(Value::Integer(42).as_string(), None);
        assert_eq!(Value::Float(3.14).as_string(), None);
        assert_eq!(Value::Bool(true).as_string(), None);
        assert_eq!(Value::Nil.as_string(), None);
    }

    // ==================== Value Display Tests ====================

    #[test]
    fn test_value_display_primitives() {
        assert_eq!(format!("{}", Value::Integer(42)), "42");
        assert_eq!(format!("{}", Value::Integer(-123)), "-123");
        assert_eq!(format!("{}", Value::Float(3.14)), "3.14");
        assert_eq!(format!("{}", Value::String("hello".to_string())), "hello");
        assert_eq!(format!("{}", Value::Bool(true)), "aye");
        assert_eq!(format!("{}", Value::Bool(false)), "nae");
        assert_eq!(format!("{}", Value::Nil), "naething");
    }

    #[test]
    fn test_value_display_list() {
        let empty = Value::List(Rc::new(RefCell::new(vec![])));
        assert_eq!(format!("{}", empty), "[]");

        let single = Value::List(Rc::new(RefCell::new(vec![Value::Integer(42)])));
        assert_eq!(format!("{}", single), "[42]");

        let multi = Value::List(Rc::new(RefCell::new(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ])));
        assert_eq!(format!("{}", multi), "[1, 2, 3]");

        // Nested list
        let inner = Value::List(Rc::new(RefCell::new(vec![
            Value::Integer(1),
            Value::Integer(2),
        ])));
        let outer = Value::List(Rc::new(RefCell::new(vec![inner, Value::Integer(3)])));
        assert_eq!(format!("{}", outer), "[[1, 2], 3]");
    }

    #[test]
    fn test_value_display_dict() {
        let empty = Value::Dict(Rc::new(RefCell::new(DictValue::new())));
        assert_eq!(format!("{}", empty), "{}");

        let mut map = DictValue::new();
        map.set(Value::String("a".to_string()), Value::Integer(1));
        let single = Value::Dict(Rc::new(RefCell::new(map)));
        assert_eq!(format!("{}", single), "{\"a\": 1}");
    }

    #[test]
    fn test_value_display_set() {
        let empty = Value::Set(Rc::new(RefCell::new(SetValue::new())));
        assert_eq!(format!("{}", empty), "creel{}");

        let mut set = SetValue::new();
        set.insert(Value::String("a".to_string()));
        let single = Value::Set(Rc::new(RefCell::new(set)));
        assert_eq!(format!("{}", single), "creel{\"a\"}");

        let mut multi_set = SetValue::new();
        multi_set.insert(Value::String("a".to_string()));
        multi_set.insert(Value::String("b".to_string()));
        let multi = Value::Set(Rc::new(RefCell::new(multi_set)));
        // Sorted output
        assert_eq!(format!("{}", multi), "creel{\"a\", \"b\"}");
    }

    #[test]
    fn set_value_intersection_and_difference_cover_both_branches_for_coverage() {
        let mut a = SetValue::new();
        a.insert(Value::String("a".to_string()));
        a.insert(Value::String("b".to_string()));

        let mut b = SetValue::new();
        b.insert(Value::String("b".to_string()));
        b.insert(Value::String("c".to_string()));

        let inter = a.intersection(&b);
        assert_eq!(inter.len(), 1);
        assert!(inter.contains(&Value::String("b".to_string())));

        let diff = a.difference(&b);
        assert_eq!(diff.len(), 1);
        assert!(diff.contains(&Value::String("a".to_string())));
    }

    #[test]
    fn test_value_display_function() {
        let func = HaversFunction::new("add".to_string(), vec![], vec![], None);
        let val = Value::Function(Rc::new(func));
        assert_eq!(format!("{}", val), "<dae add>");
    }

    #[test]
    fn test_value_display_native_function() {
        let native = NativeFunction::new("len", 1, |_| Ok(Value::Nil));
        assert_eq!((native.func)(vec![Value::Nil]).unwrap(), Value::Nil);
        let val = Value::NativeFunction(Rc::new(native));
        assert_eq!(format!("{}", val), "<native dae len>");
    }

    #[test]
    fn test_value_display_class() {
        let class = HaversClass::new("Person".to_string(), None);
        let val = Value::Class(Rc::new(class));
        assert_eq!(format!("{}", val), "<kin Person>");
    }

    #[test]
    fn test_value_display_instance() {
        let class = Rc::new(HaversClass::new("Dog".to_string(), None));
        let instance = HaversInstance::new(class);
        let val = Value::Instance(Rc::new(RefCell::new(instance)));
        assert_eq!(format!("{}", val), "<Dog instance>");
    }

    fn is_function_value(value: Option<Value>) -> bool {
        matches!(value, Some(Value::Function(_)))
    }

    #[test]
    fn test_havers_instance_get_covers_field_and_method_branches() {
        let mut class = HaversClass::new("Test".to_string(), None);
        class.methods.insert(
            "method".to_string(),
            Rc::new(HaversFunction::new(
                "method".to_string(),
                vec![],
                vec![],
                None,
            )),
        );
        let class = Rc::new(class);

        let mut instance = HaversInstance::new(class);
        instance.set("field".to_string(), Value::Integer(123));

        assert_eq!(instance.get("field"), Some(Value::Integer(123)));
        assert!(is_function_value(instance.get("method")));
        assert!(!is_function_value(instance.get("field")));
        assert_eq!(instance.get("missing"), None);
    }

    #[test]
    fn test_value_display_struct() {
        let strct = HaversStruct::new("Point".to_string(), vec!["x".to_string(), "y".to_string()]);
        let val = Value::Struct(Rc::new(strct));
        assert_eq!(format!("{}", val), "<thing Point>");
    }

    #[test]
    fn test_value_display_range() {
        let range = RangeValue::new(0, 10, false);
        let val = Value::Range(range);
        assert_eq!(format!("{}", val), "0..10");

        let inclusive = RangeValue::new(1, 5, true);
        let val2 = Value::Range(inclusive);
        assert_eq!(format!("{}", val2), "1..5");
    }

    // ==================== Value PartialEq Tests ====================

    #[test]
    fn test_value_equality_integers() {
        assert_eq!(Value::Integer(42), Value::Integer(42));
        assert_ne!(Value::Integer(42), Value::Integer(43));
    }

    #[test]
    fn test_value_equality_floats() {
        assert_eq!(Value::Float(3.14), Value::Float(3.14));
        assert_ne!(Value::Float(3.14), Value::Float(3.15));
    }

    #[test]
    fn test_value_equality_mixed_numeric() {
        assert_eq!(Value::Integer(42), Value::Float(42.0));
        assert_eq!(Value::Float(42.0), Value::Integer(42));
        assert_ne!(Value::Integer(42), Value::Float(42.5));
    }

    #[test]
    fn test_value_equality_strings() {
        assert_eq!(
            Value::String("hello".to_string()),
            Value::String("hello".to_string())
        );
        assert_ne!(
            Value::String("hello".to_string()),
            Value::String("world".to_string())
        );
    }

    #[test]
    fn test_value_equality_bools() {
        assert_eq!(Value::Bool(true), Value::Bool(true));
        assert_eq!(Value::Bool(false), Value::Bool(false));
        assert_ne!(Value::Bool(true), Value::Bool(false));
    }

    #[test]
    fn test_value_equality_nil() {
        assert_eq!(Value::Nil, Value::Nil);
    }

    #[test]
    fn test_value_equality_lists() {
        let list1 = Value::List(Rc::new(RefCell::new(vec![
            Value::Integer(1),
            Value::Integer(2),
        ])));
        let list2 = Value::List(Rc::new(RefCell::new(vec![
            Value::Integer(1),
            Value::Integer(2),
        ])));
        let list3 = Value::List(Rc::new(RefCell::new(vec![
            Value::Integer(1),
            Value::Integer(3),
        ])));

        assert_eq!(list1, list2);
        assert_ne!(list1, list3);
    }

    #[test]
    fn test_value_equality_different_types() {
        assert_ne!(Value::Integer(42), Value::String("42".to_string()));
        assert_ne!(Value::Bool(true), Value::Integer(1));
        assert_ne!(Value::Nil, Value::Integer(0));
        assert_ne!(Value::Nil, Value::Bool(false));
    }

    // ==================== FunctionParam Tests ====================

    #[test]
    fn test_function_param() {
        let param_no_default = FunctionParam {
            name: "x".to_string(),
            default: None,
        };
        assert_eq!(param_no_default.name, "x");
        assert!(param_no_default.default.is_none());
    }

    // ==================== HaversFunction Tests ====================

    #[test]
    fn test_havers_function_new() {
        let func = HaversFunction::new("test".to_string(), vec![], vec![], None);
        assert_eq!(func.name, "test");
        assert!(func.params.is_empty());
        assert!(func.body.is_empty());
        assert!(func.closure.is_none());
    }

    #[test]
    fn test_havers_function_arity_no_defaults() {
        let params = vec![
            FunctionParam {
                name: "a".to_string(),
                default: None,
            },
            FunctionParam {
                name: "b".to_string(),
                default: None,
            },
            FunctionParam {
                name: "c".to_string(),
                default: None,
            },
        ];
        let func = HaversFunction::new("add".to_string(), params, vec![], None);
        assert_eq!(func.min_arity(), 3);
        assert_eq!(func.max_arity(), 3);
    }

    #[test]
    fn test_havers_function_arity_with_defaults() {
        use crate::ast::{Expr, Literal, Span};

        let default_expr = Expr::Literal {
            value: Literal::Integer(0),
            span: Span::new(1, 1),
        };
        let params = vec![
            FunctionParam {
                name: "a".to_string(),
                default: None,
            },
            FunctionParam {
                name: "b".to_string(),
                default: Some(default_expr.clone()),
            },
            FunctionParam {
                name: "c".to_string(),
                default: Some(default_expr),
            },
        ];
        let func = HaversFunction::new("test".to_string(), params, vec![], None);
        assert_eq!(func.min_arity(), 1); // Only 'a' required
        assert_eq!(func.max_arity(), 3); // All three can be provided
    }

    #[test]
    fn test_havers_function_with_closure() {
        let env = Rc::new(RefCell::new(Environment::new()));
        env.borrow_mut().define("x".to_string(), Value::Integer(42));

        let func = HaversFunction::new(
            "closure_test".to_string(),
            vec![],
            vec![],
            Some(env.clone()),
        );
        assert!(func.closure.is_some());
    }

    // ==================== NativeFunction Tests ====================

    #[test]
    fn test_native_function_new() {
        let native = NativeFunction::new("len", 1, |args| {
            if let Value::String(s) = &args[0] {
                Ok(Value::Integer(s.len() as i64))
            } else {
                Err("Expected string".to_string())
            }
        });
        assert_eq!(native.name, "len");
        assert_eq!(native.arity, 1);
        let result = (native.func)(vec![Value::String("abcd".to_string())]).unwrap();
        assert_eq!(result, Value::Integer(4));
        let err = (native.func)(vec![Value::Integer(1)]).unwrap_err();
        assert_eq!(err, "Expected string");
    }

    #[test]
    fn test_native_function_call() {
        let native = NativeFunction::new("double", 1, |args| {
            if let Value::Integer(n) = &args[0] {
                Ok(Value::Integer(n * 2))
            } else {
                Err("Expected integer".to_string())
            }
        });

        let result = (native.func)(vec![Value::Integer(21)]);
        assert_eq!(result, Ok(Value::Integer(42)));

        let error = (native.func)(vec![Value::String("x".to_string())]);
        assert!(error.is_err());
    }

    #[test]
    fn test_native_function_debug() {
        let native = NativeFunction::new("test", 0, |_| Ok(Value::Nil));
        assert_eq!((native.func)(vec![]).unwrap(), Value::Nil);
        let debug_str = format!("{:?}", native);
        assert_eq!(debug_str, "NativeFunction(test)");
    }

    // ==================== HaversClass Tests ====================

    #[test]
    fn test_havers_class_new() {
        let class = HaversClass::new("Animal".to_string(), None);
        assert_eq!(class.name, "Animal");
        assert!(class.superclass.is_none());
        assert!(class.methods.is_empty());
    }

    #[test]
    fn test_havers_class_with_methods() {
        let mut class = HaversClass::new("Calculator".to_string(), None);
        let method = Rc::new(HaversFunction::new("add".to_string(), vec![], vec![], None));
        class.methods.insert("add".to_string(), method);

        assert!(class.find_method("add").is_some());
        assert!(class.find_method("subtract").is_none());
    }

    #[test]
    fn test_havers_class_inheritance() {
        // Parent class with a method
        let mut parent = HaversClass::new("Animal".to_string(), None);
        let speak = Rc::new(HaversFunction::new(
            "speak".to_string(),
            vec![],
            vec![],
            None,
        ));
        parent.methods.insert("speak".to_string(), speak);
        let parent_rc = Rc::new(parent);

        // Child class inheriting from parent
        let child = HaversClass::new("Dog".to_string(), Some(parent_rc));

        // Child should find parent's method
        assert!(child.find_method("speak").is_some());
        assert!(child.find_method("nonexistent").is_none());
    }

    #[test]
    fn test_havers_class_method_override() {
        // Parent class
        let mut parent = HaversClass::new("Parent".to_string(), None);
        let parent_method = Rc::new(HaversFunction::new(
            "greet".to_string(),
            vec![],
            vec![],
            None,
        ));
        parent.methods.insert("greet".to_string(), parent_method);
        let parent_rc = Rc::new(parent);

        // Child class with overridden method
        let mut child = HaversClass::new("Child".to_string(), Some(parent_rc));
        let child_method = Rc::new(HaversFunction::new(
            "greet_child".to_string(),
            vec![],
            vec![],
            None,
        ));
        child.methods.insert("greet".to_string(), child_method);

        // Child's method should take precedence
        let found = child.find_method("greet").unwrap();
        assert_eq!(found.name, "greet_child");
    }

    // ==================== HaversInstance Tests ====================

    #[test]
    fn test_havers_instance_new() {
        let class = Rc::new(HaversClass::new("Person".to_string(), None));
        let instance = HaversInstance::new(class.clone());
        assert_eq!(instance.class.name, "Person");
        assert!(instance.fields.is_empty());
    }

    #[test]
    fn test_havers_instance_set_get() {
        let class = Rc::new(HaversClass::new("Person".to_string(), None));
        let mut instance = HaversInstance::new(class);

        instance.set("name".to_string(), Value::String("Alice".to_string()));
        instance.set("age".to_string(), Value::Integer(30));

        assert_eq!(
            instance.get("name"),
            Some(Value::String("Alice".to_string()))
        );
        assert_eq!(instance.get("age"), Some(Value::Integer(30)));
        assert_eq!(instance.get("nonexistent"), None);
    }

    #[test]
    fn test_havers_instance_get_method() {
        let mut class = HaversClass::new("Calculator".to_string(), None);
        let method = Rc::new(HaversFunction::new(
            "calculate".to_string(),
            vec![],
            vec![],
            None,
        ));
        class.methods.insert("calculate".to_string(), method);
        let class_rc = Rc::new(class);

        let instance = HaversInstance::new(class_rc);

        for name in ["calculate", "nonexistent"] {
            let found = instance.get(name);
            if let Some(Value::Function(ref f)) = found {
                assert_eq!(name, "calculate");
                assert_eq!(f.name, "calculate");
            } else {
                assert_eq!(name, "nonexistent");
            }
        }
    }

    #[test]
    fn test_havers_instance_field_shadows_method() {
        let mut class = HaversClass::new("Test".to_string(), None);
        let method = Rc::new(HaversFunction::new(
            "value".to_string(),
            vec![],
            vec![],
            None,
        ));
        class.methods.insert("value".to_string(), method);
        let class_rc = Rc::new(class);

        let mut instance = HaversInstance::new(class_rc);

        // Before setting field, get returns method
        let before = instance.get("value");
        let is_function = |value: &Option<Value>| matches!(value, Some(Value::Function(_)));
        assert!(is_function(&before));
        assert!(!is_function(&Some(Value::Nil)));

        // After setting field, field takes precedence
        instance.set("value".to_string(), Value::Integer(42));
        let after = instance.get("value");
        assert_eq!(after, Some(Value::Integer(42)));
    }

    // ==================== HaversStruct Tests ====================

    #[test]
    fn test_havers_struct_new() {
        let strct = HaversStruct::new("Point".to_string(), vec!["x".to_string(), "y".to_string()]);
        assert_eq!(strct.name, "Point");
        assert_eq!(strct.fields, vec!["x", "y"]);
    }

    #[test]
    fn test_havers_struct_empty_fields() {
        let strct = HaversStruct::new("Empty".to_string(), vec![]);
        assert_eq!(strct.name, "Empty");
        assert!(strct.fields.is_empty());
    }

    // ==================== RangeValue Tests ====================

    #[test]
    fn test_range_value_new() {
        let range = RangeValue::new(0, 10, false);
        assert_eq!(range.start, 0);
        assert_eq!(range.end, 10);
        assert!(!range.inclusive);

        let inclusive = RangeValue::new(1, 5, true);
        assert!(inclusive.inclusive);
    }

    #[test]
    fn test_range_iterator_exclusive() {
        let range = RangeValue::new(0, 5, false);
        let values: Vec<i64> = range.iter().collect();
        assert_eq!(values, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_range_iterator_inclusive() {
        let range = RangeValue::new(0, 5, true);
        let values: Vec<i64> = range.iter().collect();
        assert_eq!(values, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_range_iterator_empty() {
        // Exclusive range where start == end
        let range = RangeValue::new(5, 5, false);
        let values: Vec<i64> = range.iter().collect();
        assert!(values.is_empty());
    }

    #[test]
    fn test_range_iterator_single_inclusive() {
        // Inclusive range where start == end
        let range = RangeValue::new(5, 5, true);
        let values: Vec<i64> = range.iter().collect();
        assert_eq!(values, vec![5]);
    }

    #[test]
    fn test_range_iterator_negative() {
        let range = RangeValue::new(-3, 2, false);
        let values: Vec<i64> = range.iter().collect();
        assert_eq!(values, vec![-3, -2, -1, 0, 1]);
    }

    // ==================== Environment Tests ====================

    #[test]
    fn test_environment_new() {
        let env = Environment::new();
        assert!(env.enclosing.is_none());
    }

    #[test]
    fn test_environment_default() {
        let env = Environment::default();
        assert!(env.enclosing.is_none());
    }

    #[test]
    fn test_environment_define_get() {
        let mut env = Environment::new();
        env.define("x".to_string(), Value::Integer(42));

        assert_eq!(env.get("x"), Some(Value::Integer(42)));
        assert_eq!(env.get("y"), None);
    }

    #[test]
    fn test_environment_with_enclosing() {
        let outer = Rc::new(RefCell::new(Environment::new()));
        outer
            .borrow_mut()
            .define("outer_var".to_string(), Value::Integer(1));

        let inner = Environment::with_enclosing(outer.clone());

        // Inner should find outer's variable
        assert_eq!(inner.get("outer_var"), Some(Value::Integer(1)));
        assert_eq!(inner.get("nonexistent"), None);
    }

    #[test]
    fn test_environment_shadowing() {
        let outer = Rc::new(RefCell::new(Environment::new()));
        outer
            .borrow_mut()
            .define("x".to_string(), Value::Integer(1));

        let mut inner = Environment::with_enclosing(outer.clone());
        inner.define("x".to_string(), Value::Integer(2));

        // Inner's x shadows outer's x
        assert_eq!(inner.get("x"), Some(Value::Integer(2)));
        // Outer's x unchanged
        assert_eq!(outer.borrow().get("x"), Some(Value::Integer(1)));
    }

    #[test]
    fn test_environment_assign_local() {
        let mut env = Environment::new();
        env.define("x".to_string(), Value::Integer(1));

        let result = env.assign("x", Value::Integer(2));
        assert!(result);
        assert_eq!(env.get("x"), Some(Value::Integer(2)));
    }

    #[test]
    fn test_environment_assign_nonexistent() {
        let mut env = Environment::new();

        let result = env.assign("x", Value::Integer(1));
        assert!(!result);
        assert_eq!(env.get("x"), None);
    }

    #[test]
    fn test_environment_assign_enclosing() {
        let outer = Rc::new(RefCell::new(Environment::new()));
        outer
            .borrow_mut()
            .define("x".to_string(), Value::Integer(1));

        let mut inner = Environment::with_enclosing(outer.clone());

        // Assign to outer's variable from inner
        let result = inner.assign("x", Value::Integer(2));
        assert!(result);

        // Both should see the new value
        assert_eq!(inner.get("x"), Some(Value::Integer(2)));
        assert_eq!(outer.borrow().get("x"), Some(Value::Integer(2)));
    }

    #[test]
    fn test_environment_get_exports() {
        let mut env = Environment::new();
        env.define("a".to_string(), Value::Integer(1));
        env.define("b".to_string(), Value::Integer(2));

        let exports = env.get_exports();
        assert_eq!(exports.len(), 2);
        assert_eq!(exports.get("a"), Some(&Value::Integer(1)));
        assert_eq!(exports.get("b"), Some(&Value::Integer(2)));
    }

    #[test]
    fn test_environment_get_exports_excludes_enclosing() {
        let outer = Rc::new(RefCell::new(Environment::new()));
        outer
            .borrow_mut()
            .define("outer".to_string(), Value::Integer(1));

        let mut inner = Environment::with_enclosing(outer);
        inner.define("inner".to_string(), Value::Integer(2));

        let exports = inner.get_exports();
        assert_eq!(exports.len(), 1);
        assert!(exports.contains_key("inner"));
        assert!(!exports.contains_key("outer"));
    }

    #[test]
    fn test_value_as_key_variants() {
        let key_nil = Value::Nil.as_key();
        let key_bool_true = Value::Bool(true).as_key();
        let key_bool_false = Value::Bool(false).as_key();
        let key_int_3 = Value::Integer(3).as_key();
        let key_int_4 = Value::Integer(4).as_key();
        let key_float = Value::Float(1.5).as_key();
        let key_string = Value::String("x".to_string()).as_key();

        let is_nil = |key: &ValueKey| matches!(key, ValueKey::Nil);
        assert!(is_nil(&key_nil));
        assert!(!is_nil(&key_int_3));

        let is_bool_true = |key: &ValueKey| matches!(key, ValueKey::Bool(true));
        assert!(is_bool_true(&key_bool_true));
        assert!(!is_bool_true(&key_bool_false));

        let is_int_3 = |key: &ValueKey| matches!(key, ValueKey::Int(3));
        assert!(is_int_3(&key_int_3));
        assert!(!is_int_3(&key_int_4));

        let is_float = |key: &ValueKey| matches!(key, ValueKey::Float(_));
        assert!(is_float(&key_float));
        assert!(!is_float(&key_nil));

        let is_string = |key: &ValueKey| matches!(key, ValueKey::String(_));
        assert!(is_string(&key_string));
        assert!(!is_string(&key_nil));

        let list = Value::List(Rc::new(RefCell::new(vec![])));
        let list_key = list.as_key();
        let is_list = |key: &ValueKey| matches!(key, ValueKey::List(_));
        assert!(is_list(&list_key));
        assert!(!is_list(&key_nil));

        let dict = Value::Dict(Rc::new(RefCell::new(DictValue::new())));
        let dict_key = dict.as_key();
        let is_dict = |key: &ValueKey| matches!(key, ValueKey::Dict(_));
        assert!(is_dict(&dict_key));
        assert!(!is_dict(&key_nil));

        let set = Value::Set(Rc::new(RefCell::new(SetValue::new())));
        let set_key = set.as_key();
        let is_set = |key: &ValueKey| matches!(key, ValueKey::Set(_));
        assert!(is_set(&set_key));
        assert!(!is_set(&key_nil));

        let bytes = Value::Bytes(Rc::new(RefCell::new(vec![1, 2])));
        let bytes_key = bytes.as_key();
        let is_bytes = |key: &ValueKey| matches!(key, ValueKey::Bytes(_));
        assert!(is_bytes(&bytes_key));
        assert!(!is_bytes(&key_nil));

        let func = HaversFunction::new("f".to_string(), vec![], vec![], None);
        let func_key = Value::Function(Rc::new(func)).as_key();
        let is_function = |key: &ValueKey| matches!(key, ValueKey::Function(_));
        assert!(is_function(&func_key));
        assert!(!is_function(&key_nil));

        let native = NativeFunction::new("nf", 0, |_| Ok(Value::Nil));
        assert_eq!((native.func)(vec![]).unwrap(), Value::Nil);
        let native_key = Value::NativeFunction(Rc::new(native)).as_key();
        let is_native_function = |key: &ValueKey| matches!(key, ValueKey::NativeFunction(_));
        assert!(is_native_function(&native_key));
        assert!(!is_native_function(&key_nil));

        let class = HaversClass::new("C".to_string(), None);
        let class_key = Value::Class(Rc::new(class)).as_key();
        let is_class = |key: &ValueKey| matches!(key, ValueKey::Class(_));
        assert!(is_class(&class_key));
        assert!(!is_class(&key_nil));

        let struct_def = HaversStruct::new("S".to_string(), vec![]);
        let struct_key = Value::Struct(Rc::new(struct_def)).as_key();
        let is_struct = |key: &ValueKey| matches!(key, ValueKey::Struct(_));
        assert!(is_struct(&struct_key));
        assert!(!is_struct(&key_nil));

        let class = Rc::new(HaversClass::new("I".to_string(), None));
        let instance = HaversInstance::new(class);
        let instance_key = Value::Instance(Rc::new(RefCell::new(instance))).as_key();
        let is_instance = |key: &ValueKey| matches!(key, ValueKey::Instance(_));
        assert!(is_instance(&instance_key));
        assert!(!is_instance(&key_nil));

        let native_obj = Value::NativeObject(Rc::new(TestNative));
        let native_obj_key = native_obj.as_key();
        let is_native_obj = |key: &ValueKey| matches!(key, ValueKey::NativeObject(_));
        assert!(is_native_obj(&native_obj_key));
        assert!(!is_native_obj(&key_nil));

        let range = Value::Range(RangeValue::new(1, 3, true));
        let range_key = range.as_key();
        let is_range = |key: &ValueKey| matches!(key, ValueKey::Range { .. });
        assert!(is_range(&range_key));
        assert!(!is_range(&key_nil));
    }

    #[test]
    fn test_value_equality_pointer_types() {
        let dict_rc = Rc::new(RefCell::new(DictValue::new()));
        let dict_val = Value::Dict(dict_rc.clone());
        assert_eq!(dict_val, Value::Dict(dict_rc.clone()));
        assert_ne!(
            dict_val,
            Value::Dict(Rc::new(RefCell::new(DictValue::new())))
        );

        let set_rc = Rc::new(RefCell::new(SetValue::new()));
        let set_val = Value::Set(set_rc.clone());
        assert_eq!(set_val, Value::Set(set_rc.clone()));
        assert_ne!(set_val, Value::Set(Rc::new(RefCell::new(SetValue::new()))));

        let bytes_a = Value::Bytes(Rc::new(RefCell::new(vec![1, 2])));
        let bytes_b = Value::Bytes(Rc::new(RefCell::new(vec![1, 2])));
        let bytes_c = Value::Bytes(Rc::new(RefCell::new(vec![2, 3])));
        assert_eq!(bytes_a, bytes_b);
        assert_ne!(bytes_a, bytes_c);

        let func = Rc::new(HaversFunction::new("f".to_string(), vec![], vec![], None));
        assert_eq!(Value::Function(func.clone()), Value::Function(func.clone()));
        assert_ne!(
            Value::Function(func.clone()),
            Value::Function(Rc::new(HaversFunction::new(
                "g".to_string(),
                vec![],
                vec![],
                None
            )))
        );

        let class = Rc::new(HaversClass::new("C".to_string(), None));
        assert_eq!(Value::Class(class.clone()), Value::Class(class.clone()));

        let instance = Rc::new(RefCell::new(HaversInstance::new(class)));
        assert_eq!(
            Value::Instance(instance.clone()),
            Value::Instance(instance.clone())
        );

        let st = Rc::new(HaversStruct::new("S".to_string(), vec![]));
        assert_eq!(Value::Struct(st.clone()), Value::Struct(st.clone()));

        let native = Rc::new(TestNative);
        assert_eq!(
            Value::NativeObject(native.clone()),
            Value::NativeObject(native.clone())
        );

        let native_fn = Rc::new(NativeFunction::new("nf", 0, |_| Ok(Value::Nil)));
        assert_eq!((native_fn.func)(vec![]).unwrap(), Value::Nil);
        assert_eq!(
            Value::NativeFunction(native_fn.clone()),
            Value::NativeFunction(native_fn.clone())
        );
        let other_native_fn = Rc::new(NativeFunction::new("nf", 0, |_| Ok(Value::Nil)));
        assert_eq!((other_native_fn.func)(vec![]).unwrap(), Value::Nil);
        assert_ne!(
            Value::NativeFunction(native_fn.clone()),
            Value::NativeFunction(other_native_fn)
        );
    }

    #[test]
    fn test_dict_set_default_and_native_methods() {
        let dict: DictValue = Default::default();
        assert!(dict.is_empty());
        let set: SetValue = Default::default();
        assert!(set.is_empty());

        let native = TestNative;
        assert_eq!(native.type_name(), "test_native");
        assert_eq!(native.get("x").unwrap(), Value::Nil);
        assert_eq!(native.set("x", Value::Integer(2)).unwrap(), Value::Nil);
        assert_eq!(native.call("unknown", vec![]).unwrap(), Value::Nil);
        assert!(native.as_any().is::<TestNative>());
    }

    #[test]
    fn test_dict_value_set_overwrites_existing_key() {
        let mut dict = DictValue::new();
        dict.set(Value::String("a".to_string()), Value::Integer(1));
        assert_eq!(
            dict.get(&Value::String("a".to_string()))
                .and_then(Value::as_integer),
            Some(1)
        );

        dict.set(Value::String("a".to_string()), Value::Integer(2));
        assert_eq!(
            dict.get(&Value::String("a".to_string()))
                .and_then(Value::as_integer),
            Some(2)
        );
        assert_eq!(dict.len(), 1);
    }
}
