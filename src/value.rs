use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::ast::Stmt;

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
    Dict(Rc<RefCell<HashMap<String, Value>>>),
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
    /// Range iterator
    Range(RangeValue),
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
            Value::Function(_) => "function",
            Value::NativeFunction(_) => "native function",
            Value::Class(_) => "class",
            Value::Instance(_) => "instance",
            Value::Struct(_) => "struct",
            Value::Range(_) => "range",
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

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Integer(n) => Some(*n as f64),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
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
            Value::Function(func) => write!(f, "<dae {}>", func.name),
            Value::NativeFunction(func) => write!(f, "<native dae {}>", func.name),
            Value::Class(class) => write!(f, "<kin {}>", class.name),
            Value::Instance(inst) => write!(f, "<{} instance>", inst.borrow().class.name),
            Value::Struct(s) => write!(f, "<thing {}>", s.name),
            Value::Range(r) => write!(f, "{}..{}", r.start, r.end),
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
            _ => false,
        }
    }
}

/// A user-defined function
#[derive(Debug)]
pub struct HaversFunction {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
    pub closure: Option<Rc<RefCell<Environment>>>,
}

impl HaversFunction {
    pub fn new(
        name: String,
        params: Vec<String>,
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
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
