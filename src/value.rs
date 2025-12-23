use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
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
#[derive(Debug, Clone)]
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

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(true) => write!(f, "aye"),
            Value::Bool(false) => write!(f, "nae"),
            Value::Nil => write!(f, "naething"),
            Value::List(items) => {
                let items = items.borrow();
                let strs: Vec<String> = items.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", strs.join(", "))
            }
            Value::Dict(map) => {
                let map = map.borrow();
                let strs: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, v))
                    .collect();
                write!(f, "{{{}}}", strs.join(", "))
            }
            Value::Set(set) => {
                let set = set.borrow();
                let mut strs: Vec<String> = set.iter().map(|v| format!("{}", v)).collect();
                strs.sort(); // Sort fer consistent display
                write!(
                    f,
                    "creel{{{}}}",
                    strs.iter()
                        .map(|s| format!("\"{}\"", s))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Value::Bytes(bytes) => {
                let len = bytes.borrow().len();
                write!(f, "bytes[{}]", len)
            }
            Value::Function(func) => write!(f, "<dae {}>", func.name),
            Value::NativeFunction(func) => write!(f, "<native dae {}>", func.name),
            Value::Class(class) => write!(f, "<kin {}>", class.name),
            Value::Instance(inst) => write!(f, "<{} instance>", inst.borrow().class.name),
            Value::Struct(s) => write!(f, "<thing {}>", s.name),
            Value::Range(r) => write!(f, "{}..{}", r.start, r.end),
            Value::NativeObject(obj) => write!(f, "{}", obj.to_string()),
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
            (Value::List(a), Value::List(b)) => *a.borrow() == *b.borrow(),
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
            if let Some((_, v)) = self.entries.get_mut(idx) {
                *v = value;
            }
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
        }
    }
}

pub struct RangeIterator {
    current: i64,
    end: i64,
    inclusive: bool,
}

impl Iterator for RangeIterator {
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        let should_yield = if self.inclusive {
            self.current <= self.end
        } else {
            self.current < self.end
        };

        if should_yield {
            let val = self.current;
            self.current += 1;
            Some(val)
        } else {
            None
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

        let func = HaversFunction::new("test".to_string(), vec![], vec![], None);
        assert_eq!(Value::Function(Rc::new(func)).type_name(), "function");

        let native = NativeFunction::new("native", 0, |_| Ok(Value::Nil));
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
    fn test_value_display_function() {
        let func = HaversFunction::new("add".to_string(), vec![], vec![], None);
        let val = Value::Function(Rc::new(func));
        assert_eq!(format!("{}", val), "<dae add>");
    }

    #[test]
    fn test_value_display_native_function() {
        let native = NativeFunction::new("len", 1, |_| Ok(Value::Nil));
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
            if let Some(Value::Function(f)) = found {
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
        assert!(matches!(before, Some(Value::Function(_))));

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
        assert!(matches!(Value::Nil.as_key(), ValueKey::Nil));
        assert!(matches!(Value::Bool(true).as_key(), ValueKey::Bool(true)));
        assert!(matches!(Value::Integer(3).as_key(), ValueKey::Int(3)));
        assert!(matches!(Value::Float(1.5).as_key(), ValueKey::Float(_)));
        assert!(matches!(
            Value::String("x".to_string()).as_key(),
            ValueKey::String(_)
        ));

        let list = Value::List(Rc::new(RefCell::new(vec![])));
        assert!(matches!(list.as_key(), ValueKey::List(_)));
        let dict = Value::Dict(Rc::new(RefCell::new(DictValue::new())));
        assert!(matches!(dict.as_key(), ValueKey::Dict(_)));
        let set = Value::Set(Rc::new(RefCell::new(SetValue::new())));
        assert!(matches!(set.as_key(), ValueKey::Set(_)));
        let bytes = Value::Bytes(Rc::new(RefCell::new(vec![1, 2])));
        assert!(matches!(bytes.as_key(), ValueKey::Bytes(_)));

        let func = HaversFunction::new("f".to_string(), vec![], vec![], None);
        assert!(matches!(
            Value::Function(Rc::new(func)).as_key(),
            ValueKey::Function(_)
        ));
        let native = NativeFunction::new("nf", 0, |_| Ok(Value::Nil));
        assert!(matches!(
            Value::NativeFunction(Rc::new(native)).as_key(),
            ValueKey::NativeFunction(_)
        ));
        let class = HaversClass::new("C".to_string(), None);
        assert!(matches!(
            Value::Class(Rc::new(class)).as_key(),
            ValueKey::Class(_)
        ));
        let struct_def = HaversStruct::new("S".to_string(), vec![]);
        assert!(matches!(
            Value::Struct(Rc::new(struct_def)).as_key(),
            ValueKey::Struct(_)
        ));
        let class = Rc::new(HaversClass::new("I".to_string(), None));
        let instance = HaversInstance::new(class);
        assert!(matches!(
            Value::Instance(Rc::new(RefCell::new(instance))).as_key(),
            ValueKey::Instance(_)
        ));
        let native_obj = Value::NativeObject(Rc::new(TestNative));
        assert!(matches!(native_obj.as_key(), ValueKey::NativeObject(_)));
        let range = Value::Range(RangeValue::new(1, 3, true));
        assert!(matches!(range.as_key(), ValueKey::Range { .. }));
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
        assert_eq!(
            Value::NativeFunction(native_fn.clone()),
            Value::NativeFunction(native_fn.clone())
        );
        assert_ne!(
            Value::NativeFunction(native_fn.clone()),
            Value::NativeFunction(Rc::new(NativeFunction::new("nf", 0, |_| Ok(Value::Nil))))
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
}
