use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::ast::*;
use crate::error::{HaversError, HaversResult};
use crate::value::*;

/// Control flow signals
#[derive(Debug)]
enum ControlFlow {
    Return(Value),
    Break,
    Continue,
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
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));

        // Define native functions
        Self::define_natives(&globals);

        Interpreter {
            globals: globals.clone(),
            environment: globals,
            output: Vec::new(),
            loaded_modules: HashSet::new(),
            current_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            prelude_loaded: false,
        }
    }

    /// Create an interpreter with a specific working directory
    pub fn with_dir<P: AsRef<Path>>(dir: P) -> Self {
        let mut interp = Self::new();
        interp.current_dir = dir.as_ref().to_path_buf();
        interp
    }

    /// Set the current directory fer module resolution
    pub fn set_current_dir<P: AsRef<Path>>(&mut self, dir: P) {
        self.current_dir = dir.as_ref().to_path_buf();
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
    pub fn has_prelude(&self) -> bool {
        self.prelude_loaded
    }

    fn define_natives(globals: &Rc<RefCell<Environment>>) {
        // len - get length of list, string, dict, or set
        globals.borrow_mut().define(
            "len".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("len", 1, |args| {
                match &args[0] {
                    Value::String(s) => Ok(Value::Integer(s.len() as i64)),
                    Value::List(l) => Ok(Value::Integer(l.borrow().len() as i64)),
                    Value::Dict(d) => Ok(Value::Integer(d.borrow().len() as i64)),
                    Value::Set(s) => Ok(Value::Integer(s.borrow().len() as i64)),
                    _ => Err("len() expects a string, list, dict, or creel".to_string()),
                }
            }))),
        );

        // type - get type of value (whit_kind in Scots!)
        globals.borrow_mut().define(
            "whit_kind".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("whit_kind", 1, |args| {
                Ok(Value::String(args[0].type_name().to_string()))
            }))),
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
            Value::NativeFunction(Rc::new(NativeFunction::new("tae_int", 1, |args| {
                match &args[0] {
                    Value::Integer(n) => Ok(Value::Integer(*n)),
                    Value::Float(f) => Ok(Value::Integer(*f as i64)),
                    Value::String(s) => s
                        .parse::<i64>()
                        .map(Value::Integer)
                        .map_err(|_| format!("Cannae turn '{}' intae an integer", s)),
                    Value::Bool(b) => Ok(Value::Integer(if *b { 1 } else { 0 })),
                    _ => Err(format!("Cannae turn {} intae an integer", args[0].type_name())),
                }
            }))),
        );

        // float - convert to float (tae_float in Scots!)
        globals.borrow_mut().define(
            "tae_float".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("tae_float", 1, |args| {
                match &args[0] {
                    Value::Integer(n) => Ok(Value::Float(*n as f64)),
                    Value::Float(f) => Ok(Value::Float(*f)),
                    Value::String(s) => s
                        .parse::<f64>()
                        .map(Value::Float)
                        .map_err(|_| format!("Cannae turn '{}' intae a float", s)),
                    _ => Err(format!("Cannae turn {} intae a float", args[0].type_name())),
                }
            }))),
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
                    let keys: Vec<Value> = dict
                        .borrow()
                        .keys()
                        .map(|k| Value::String(k.clone()))
                        .collect();
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
                let start = args[0]
                    .as_integer()
                    .ok_or("range() expects integers")?;
                let end = args[1]
                    .as_integer()
                    .ok_or("range() expects integers")?;
                Ok(Value::Range(RangeValue::new(start, end, false)))
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
            Value::NativeFunction(Rc::new(NativeFunction::new("floor", 1, |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Integer(f.floor() as i64)),
                    Value::Integer(n) => Ok(Value::Integer(*n)),
                    _ => Err("floor() expects a number".to_string()),
                }
            }))),
        );

        // ceil
        globals.borrow_mut().define(
            "ceil".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("ceil", 1, |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Integer(f.ceil() as i64)),
                    Value::Integer(n) => Ok(Value::Integer(*n)),
                    _ => Err("ceil() expects a number".to_string()),
                }
            }))),
        );

        // round
        globals.borrow_mut().define(
            "round".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("round", 1, |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Integer(f.round() as i64)),
                    Value::Integer(n) => Ok(Value::Integer(*n)),
                    _ => Err("round() expects a number".to_string()),
                }
            }))),
        );

        // sqrt
        globals.borrow_mut().define(
            "sqrt".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("sqrt", 1, |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.sqrt())),
                    Value::Integer(n) => Ok(Value::Float((*n as f64).sqrt())),
                    _ => Err("sqrt() expects a number".to_string()),
                }
            }))),
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
                        let parts: Vec<String> = list
                            .borrow()
                            .iter()
                            .map(|v| format!("{}", v))
                            .collect();
                        Ok(Value::String(parts.join(delim)))
                    }
                    _ => Err("join() expects a list and a string".to_string()),
                }
            }))),
        );

        // contains - check if list/string contains value
        globals.borrow_mut().define(
            "contains".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("contains", 2, |args| {
                match &args[0] {
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
                    Value::Dict(dict) => {
                        if let Value::String(key) = &args[1] {
                            Ok(Value::Bool(dict.borrow().contains_key(key)))
                        } else {
                            Err("contains() on dict expects a string key".to_string())
                        }
                    }
                    _ => Err("contains() expects a list, string, or dict".to_string()),
                }
            }))),
        );

        // reverse - reverse a list or string
        globals.borrow_mut().define(
            "reverse".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("reverse", 1, |args| {
                match &args[0] {
                    Value::List(list) => {
                        let mut reversed = list.borrow().clone();
                        reversed.reverse();
                        Ok(Value::List(Rc::new(RefCell::new(reversed))))
                    }
                    Value::String(s) => {
                        Ok(Value::String(s.chars().rev().collect()))
                    }
                    _ => Err("reverse() expects a list or string".to_string()),
                }
            }))),
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
            Value::NativeFunction(Rc::new(NativeFunction::new("heid", 1, |args| {
                match &args[0] {
                    Value::List(list) => {
                        list.borrow().first().cloned().ok_or("Cannae get heid o' empty list!".to_string())
                    }
                    Value::String(s) => {
                        s.chars().next().map(|c| Value::String(c.to_string())).ok_or("Cannae get heid o' empty string!".to_string())
                    }
                    _ => Err("heid() expects a list or string".to_string()),
                }
            }))),
        );

        // tail - get everything except the first (like a tail!)
        globals.borrow_mut().define(
            "tail".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("tail", 1, |args| {
                match &args[0] {
                    Value::List(list) => {
                        let list = list.borrow();
                        if list.is_empty() {
                            Ok(Value::List(Rc::new(RefCell::new(Vec::new()))))
                        } else {
                            Ok(Value::List(Rc::new(RefCell::new(list[1..].to_vec()))))
                        }
                    }
                    Value::String(s) => {
                        Ok(Value::String(s.chars().skip(1).collect()))
                    }
                    _ => Err("tail() expects a list or string".to_string()),
                }
            }))),
        );

        // bum - get the last element (backside!)
        globals.borrow_mut().define(
            "bum".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("bum", 1, |args| {
                match &args[0] {
                    Value::List(list) => {
                        list.borrow().last().cloned().ok_or("Cannae get bum o' empty list!".to_string())
                    }
                    Value::String(s) => {
                        s.chars().last().map(|c| Value::String(c.to_string())).ok_or("Cannae get bum o' empty string!".to_string())
                    }
                    _ => Err("bum() expects a list or string".to_string()),
                }
            }))),
        );

        // scran - slice a list or string (grab a portion, like grabbing scran/food)
        globals.borrow_mut().define(
            "scran".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("scran", 3, |args| {
                let start = args[1].as_integer().ok_or("scran() needs integer indices")?;
                let end = args[2].as_integer().ok_or("scran() needs integer indices")?;
                match &args[0] {
                    Value::List(list) => {
                        let list = list.borrow();
                        let start = start.max(0) as usize;
                        let end = end.min(list.len() as i64) as usize;
                        Ok(Value::List(Rc::new(RefCell::new(list[start..end].to_vec()))))
                    }
                    Value::String(s) => {
                        let start = start.max(0) as usize;
                        let end = end.min(s.len() as i64) as usize;
                        Ok(Value::String(s.chars().skip(start).take(end - start).collect()))
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
                            Value::Float(f) => { sum += f; is_float = true; }
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
            Value::NativeFunction(Rc::new(NativeFunction::new("coont", 2, |args| {
                match &args[0] {
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
                }
            }))),
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
                    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
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
                    sorted.sort_by(|a, b| {
                        match (a, b) {
                            (Value::Integer(x), Value::Integer(y)) => x.cmp(y),
                            (Value::Float(x), Value::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                            (Value::String(x), Value::String(y)) => x.cmp(y),
                            _ => std::cmp::Ordering::Equal,
                        }
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
                let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
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
                let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                Ok(Value::Integer(secs as i64))
            }))),
        );

        // clype - debug print with type info (Scots: tell/inform/snitch)
        globals.borrow_mut().define(
            "clype".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("clype", 1, |args| {
                let val = &args[0];
                Ok(Value::String(format!("[{}] {}", val.type_name(), val)))
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
                    "dict" => matches!(args[0], Value::Dict(_)),
                    "function" | "dae" => matches!(args[0], Value::Function(_) | Value::NativeFunction(_)),
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
                let idx = args[1].as_integer().ok_or("char_at() needs an integer index")?;
                let idx = if idx < 0 { s.len() as i64 + idx } else { idx } as usize;
                s.chars().nth(idx)
                    .map(|c| Value::String(c.to_string()))
                    .ok_or_else(|| format!("Index {} oot o' bounds fer string o' length {}", idx, s.len()))
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
            Value::NativeFunction(Rc::new(NativeFunction::new("starts_wi", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::String(s), Value::String(prefix)) => {
                        Ok(Value::Bool(s.starts_with(prefix.as_str())))
                    }
                    _ => Err("starts_wi() needs two strings".to_string()),
                }
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
                    (Value::String(s), Value::String(needle)) => {
                        Ok(Value::Integer(s.find(needle.as_str()).map(|i| i as i64).unwrap_or(-1)))
                    }
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

        // === More String Functions ===

        // pad_left - pad string on the left to reach target length
        globals.borrow_mut().define(
            "pad_left".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("pad_left", 3, |args| {
                match (&args[0], &args[1], &args[2]) {
                    (Value::String(s), Value::Integer(width), Value::String(pad_char)) => {
                        let width = *width as usize;
                        if s.len() >= width {
                            Ok(Value::String(s.clone()))
                        } else {
                            let pad = pad_char.chars().next().unwrap_or(' ');
                            let padding: String = std::iter::repeat(pad).take(width - s.len()).collect();
                            Ok(Value::String(format!("{}{}", padding, s)))
                        }
                    }
                    _ => Err("pad_left() needs a string, integer width, and pad character".to_string()),
                }
            }))),
        );

        // pad_right - pad string on the right to reach target length
        globals.borrow_mut().define(
            "pad_right".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("pad_right", 3, |args| {
                match (&args[0], &args[1], &args[2]) {
                    (Value::String(s), Value::Integer(width), Value::String(pad_char)) => {
                        let width = *width as usize;
                        if s.len() >= width {
                            Ok(Value::String(s.clone()))
                        } else {
                            let pad = pad_char.chars().next().unwrap_or(' ');
                            let padding: String = std::iter::repeat(pad).take(width - s.len()).collect();
                            Ok(Value::String(format!("{}{}", s, padding)))
                        }
                    }
                    _ => Err("pad_right() needs a string, integer width, and pad character".to_string()),
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
                    Ok(Value::Bool(!s.is_empty() && s.chars().all(|c| c.is_ascii_digit())))
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
                    Ok(Value::Bool(!s.is_empty() && s.chars().all(|c| c.is_alphabetic())))
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
                    Ok(Value::Bool(!s.is_empty() && s.chars().all(|c| c.is_whitespace())))
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
                        Some(first) => format!("{}{}", first.to_uppercase(), chars.collect::<String>()),
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
                            match chars.next() {
                                Some(first) => format!("{}{}", first.to_uppercase(), chars.collect::<String>().to_lowercase()),
                                None => String::new(),
                            }
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
                    let char_list: Vec<Value> = s
                        .chars()
                        .map(|c| Value::String(c.to_string()))
                        .collect();
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
                        Err(format!("chr() needs a valid Unicode codepoint (0 to 1114111), got {}", n))
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
                        let result: Vec<Value> = a.iter().zip(b.iter())
                            .map(|(x, y)| Value::List(Rc::new(RefCell::new(vec![x.clone(), y.clone()]))))
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
                    let result: Vec<Value> = list.borrow().iter().enumerate()
                        .map(|(i, v)| Value::List(Rc::new(RefCell::new(vec![Value::Integer(i as i64), v.clone()]))))
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
                    let idx = args[1].as_integer().ok_or("chynge() needs an integer index")?;
                    let mut new_list = list.borrow().clone();
                    let idx = if idx < 0 {
                        (new_list.len() as i64 + idx) as usize
                    } else {
                        idx as usize
                    };
                    if idx > new_list.len() {
                        return Err(format!("Index {} oot o' bounds fer list o' length {}", idx, new_list.len()));
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
                    let idx = args[1].as_integer().ok_or("dicht() needs an integer index")?;
                    let mut new_list = list.borrow().clone();
                    let idx = if idx < 0 {
                        (new_list.len() as i64 + idx) as usize
                    } else {
                        idx as usize
                    };
                    if idx >= new_list.len() {
                        return Err(format!("Index {} oot o' bounds fer list o' length {}", idx, new_list.len()));
                    }
                    new_list.remove(idx);
                    Ok(Value::List(Rc::new(RefCell::new(new_list))))
                } else {
                    Err("dicht() needs a list".to_string())
                }
            }))),
        );

        // tak - take first n elements (Scots: take)
        globals.borrow_mut().define(
            "tak".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("tak", 2, |args| {
                let n = args[1].as_integer().ok_or("tak() needs an integer count")?;
                let n = n.max(0) as usize;
                match &args[0] {
                    Value::List(list) => {
                        let list = list.borrow();
                        let result: Vec<Value> = list.iter().take(n).cloned().collect();
                        Ok(Value::List(Rc::new(RefCell::new(result))))
                    }
                    Value::String(s) => {
                        let taken: String = s.chars().take(n).collect();
                        Ok(Value::String(taken))
                    }
                    _ => Err("tak() needs a list or string".to_string()),
                }
            }))),
        );

        // drap - drop first n elements (Scots: drop)
        globals.borrow_mut().define(
            "drap".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("drap", 2, |args| {
                let n = args[1].as_integer().ok_or("drap() needs an integer count")?;
                let n = n.max(0) as usize;
                match &args[0] {
                    Value::List(list) => {
                        let list = list.borrow();
                        let result: Vec<Value> = list.iter().skip(n).cloned().collect();
                        Ok(Value::List(Rc::new(RefCell::new(result))))
                    }
                    Value::String(s) => {
                        let dropped: String = s.chars().skip(n).collect();
                        Ok(Value::String(dropped))
                    }
                    _ => Err("drap() needs a list or string".to_string()),
                }
            }))),
        );

        // redd_up - remove nil values from list (Scots: tidy up)
        globals.borrow_mut().define(
            "redd_up".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("redd_up", 1, |args| {
                if let Value::List(list) = &args[0] {
                    let result: Vec<Value> = list.borrow()
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
                    let n = args[1].as_integer().ok_or("chunks() needs an integer size")?;
                    if n <= 0 {
                        return Err("chunks() size must be positive".to_string());
                    }
                    let n = n as usize;
                    let list = list.borrow();
                    let result: Vec<Value> = list.chunks(n)
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
            Value::NativeFunction(Rc::new(NativeFunction::new("interleave", 2, |args| {
                match (&args[0], &args[1]) {
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
                }
            }))),
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
                    (Value::Float(base), Value::Float(exp)) => {
                        Ok(Value::Float(base.powf(*exp)))
                    }
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
            Value::NativeFunction(Rc::new(NativeFunction::new("log10", 1, |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.log10())),
                    Value::Integer(n) => Ok(Value::Float((*n as f64).log10())),
                    _ => Err("log10() needs a number".to_string()),
                }
            }))),
        );

        // PI constant
        globals.borrow_mut().define(
            "PI".to_string(),
            Value::Float(std::f64::consts::PI),
        );

        // E constant (Euler's number)
        globals.borrow_mut().define(
            "E".to_string(),
            Value::Float(std::f64::consts::E),
        );

        // === Time Functions ===

        // snooze - sleep for milliseconds (Scots: have a wee rest)
        globals.borrow_mut().define(
            "snooze".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("snooze", 1, |args| {
                let ms = args[0].as_integer().ok_or("snooze() needs an integer (milliseconds)")?;
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
                    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
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
                            Ok(Value::String(format!("{}{}", pad_char.to_string().repeat(w - s.len()), s)))
                        }
                    }
                    _ => Err("pad_left() needs (string, width, pad_char)".to_string()),
                }
            }))),
        );

        // pad_right - pad string on right
        globals.borrow_mut().define(
            "pad_right".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("pad_right", 3, |args| {
                match (&args[0], &args[1], &args[2]) {
                    (Value::String(s), Value::Integer(width), Value::String(pad)) => {
                        let pad_char = pad.chars().next().unwrap_or(' ');
                        let w = *width as usize;
                        if s.len() >= w {
                            Ok(Value::String(s.clone()))
                        } else {
                            Ok(Value::String(format!("{}{}", s, pad_char.to_string().repeat(w - s.len()))))
                        }
                    }
                    _ => Err("pad_right() needs (string, width, pad_char)".to_string()),
                }
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
                        let result: Vec<Value> = items.chunks(size)
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
                    let result: Vec<Value> = items.chunks(2)
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
            Value::NativeFunction(Rc::new(NativeFunction::new("haud_yer_wheesht", 0, |_args| {
                Ok(Value::String("".to_string()))
            }))),
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
                                if *b > *a { best = item.clone(); }
                            }
                            (Value::Float(a), Value::Float(b)) => {
                                if *b > *a { best = item.clone(); }
                            }
                            (Value::String(a), Value::String(b)) => {
                                if b.len() > a.len() { best = item.clone(); }
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
            Value::NativeFunction(Rc::new(NativeFunction::new("numpty_check", 1, |args| {
                match &args[0] {
                    Value::Nil => Ok(Value::String("That's naething, ya numpty!".to_string())),
                    Value::String(s) if s.is_empty() => Ok(Value::String("Empty string, ya numpty!".to_string())),
                    Value::List(l) if l.borrow().is_empty() => Ok(Value::String("Empty list, ya numpty!".to_string())),
                    _ => Ok(Value::String("That's braw!".to_string())),
                }
            }))),
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
                let lines: Vec<Value> = content.lines()
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
                let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
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
                let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
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
                let time_str = match hours {
                    0..=5 => format!("It's the wee small hours ({:02}:{:02})", hours, minutes),
                    6..=11 => format!("It's the mornin' ({:02}:{:02})", hours, minutes),
                    12 => format!("It's high noon ({:02}:{:02})", hours, minutes),
                    13..=17 => format!("It's the efternoon ({:02}:{:02})", hours, minutes),
                    18..=21 => format!("It's the evenin' ({:02}:{:02})", hours, minutes),
                    _ => format!("It's gettin' late ({:02}:{:02})", hours, minutes),
                };
                Ok(Value::String(time_str))
            }))),
        );

        // wheesht_aw - trim and clean up a string (more thorough than wheesht)
        globals.borrow_mut().define(
            "wheesht_aw".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("wheesht_aw", 1, |args| {
                if let Value::String(s) = &args[0] {
                    // Collapse multiple spaces and trim
                    let cleaned: String = s
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ");
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
                    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
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
            Value::NativeFunction(Rc::new(NativeFunction::new("crabbit", 1, |args| {
                match &args[0] {
                    Value::Integer(n) => Ok(Value::Bool(*n < 0)),
                    Value::Float(f) => Ok(Value::Bool(*f < 0.0)),
                    _ => Err("crabbit() needs a number".to_string()),
                }
            }))),
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

        // cannie - check if a value is "careful"/safe (within reasonable bounds)
        globals.borrow_mut().define(
            "cannie".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("cannie", 1, |args| {
                let is_cannie = match &args[0] {
                    Value::Integer(n) => *n >= -1000 && *n <= 1000,
                    Value::Float(f) => *f >= -1000.0 && *f <= 1000.0 && f.is_finite(),
                    Value::String(s) => s.len() <= 1000 && !s.contains(|c: char| c.is_control()),
                    Value::List(l) => l.borrow().len() <= 1000,
                    Value::Dict(d) => d.borrow().len() <= 100,
                    _ => true,
                };
                Ok(Value::Bool(is_cannie))
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
            Value::NativeFunction(Rc::new(NativeFunction::new("indices_o", 2, |args| {
                match &args[0] {
                    Value::List(list) => {
                        let items = list.borrow();
                        let needle = &args[1];
                        let indices: Vec<Value> = items.iter()
                            .enumerate()
                            .filter(|(_, item)| *item == needle)
                            .map(|(i, _)| Value::Integer(i as i64))
                            .collect();
                        Ok(Value::List(Rc::new(RefCell::new(indices))))
                    }
                    Value::String(s) => {
                        let needle = match &args[1] {
                            Value::String(n) => n,
                            _ => return Err("indices_o() on string needs a string needle".to_string()),
                        };
                        if needle.is_empty() {
                            return Err("Cannae search fer an empty string, ya numpty!".to_string());
                        }
                        let indices: Vec<Value> = s.match_indices(needle.as_str())
                            .map(|(i, _)| Value::Integer(i as i64))
                            .collect();
                        Ok(Value::List(Rc::new(RefCell::new(indices))))
                    }
                    _ => Err("indices_o() needs a list or string".to_string()),
                }
            }))),
        );

        // braw_date - format a timestamp or current time in Scottish style
        globals.borrow_mut().define(
            "braw_date".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("braw_date", 1, |args| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let secs = match &args[0] {
                    Value::Integer(n) => *n as u64,
                    Value::Nil => SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    _ => return Err("braw_date() needs a timestamp or naething".to_string()),
                };
                // Calculate date components (simplified, doesn't handle leap years perfectly)
                let days_since_epoch = secs / 86400;
                let day_of_week = ((days_since_epoch + 4) % 7) as usize; // Jan 1, 1970 was Thursday

                let scots_day_names = [
                    "the Sabbath", "Monday", "Tuesday", "Wednesday",
                    "Thursday", "Friday", "Setterday"
                ];

                // Simple month/day calculation
                let mut remaining_days = days_since_epoch as i64;
                let mut year = 1970i64;
                loop {
                    let days_in_year = if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 { 366 } else { 365 };
                    if remaining_days < days_in_year {
                        break;
                    }
                    remaining_days -= days_in_year;
                    year += 1;
                }

                let scots_months = [
                    "Januar", "Februar", "Mairch", "Aprile", "Mey", "Juin",
                    "Julie", "August", "September", "October", "November", "December"
                ];
                let days_in_months: [i64; 12] = if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
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
            Value::NativeFunction(Rc::new(NativeFunction::new("haverin", 1, |args| {
                match &args[0] {
                    Value::String(s) => {
                        let trimmed = s.trim();
                        Ok(Value::Bool(trimmed.is_empty() || trimmed.len() < 2))
                    }
                    Value::Nil => Ok(Value::Bool(true)),
                    Value::List(l) => Ok(Value::Bool(l.borrow().is_empty())),
                    _ => Ok(Value::Bool(false)),
                }
            }))),
        );

        // scunner - check if value is "disgusting" (negative or empty)
        globals.borrow_mut().define(
            "scunner".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("scunner", 1, |args| {
                match &args[0] {
                    Value::Integer(n) => Ok(Value::Bool(*n < 0)),
                    Value::Float(f) => Ok(Value::Bool(*f < 0.0)),
                    Value::String(s) => Ok(Value::Bool(s.is_empty())),
                    Value::List(l) => Ok(Value::Bool(l.borrow().is_empty())),
                    Value::Bool(b) => Ok(Value::Bool(!*b)),
                    Value::Nil => Ok(Value::Bool(true)),
                    _ => Ok(Value::Bool(false)),
                }
            }))),
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
            Value::NativeFunction(Rc::new(NativeFunction::new("is_wee", 1, |args| {
                match &args[0] {
                    Value::Integer(n) => Ok(Value::Bool(n.abs() < 10)),
                    Value::Float(f) => Ok(Value::Bool(f.abs() < 10.0)),
                    Value::String(s) => Ok(Value::Bool(s.len() < 5)),
                    Value::List(l) => Ok(Value::Bool(l.borrow().len() < 5)),
                    _ => Ok(Value::Bool(true)),
                }
            }))),
        );

        // is_muckle - check if value is big (opposite of is_wee)
        globals.borrow_mut().define(
            "is_muckle".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("is_muckle", 1, |args| {
                match &args[0] {
                    Value::Integer(n) => Ok(Value::Bool(n.abs() >= 100)),
                    Value::Float(f) => Ok(Value::Bool(f.abs() >= 100.0)),
                    Value::String(s) => Ok(Value::Bool(s.len() >= 50)),
                    Value::List(l) => Ok(Value::Bool(l.borrow().len() >= 50)),
                    _ => Ok(Value::Bool(false)),
                }
            }))),
        );

        // crabbit - make string all uppercase and grumpy (add !)
        globals.borrow_mut().define(
            "crabbit".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("crabbit", 1, |args| {
                match &args[0] {
                    Value::String(s) => {
                        Ok(Value::String(format!("{}!", s.to_uppercase())))
                    }
                    _ => Err("crabbit() needs a string tae shout!".to_string()),
                }
            }))),
        );

        // cannie - check if value is safe/valid (not nil, not empty, not negative)
        globals.borrow_mut().define(
            "cannie".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("cannie", 1, |args| {
                match &args[0] {
                    Value::Nil => Ok(Value::Bool(false)),
                    Value::Integer(n) => Ok(Value::Bool(*n >= 0)),
                    Value::Float(f) => Ok(Value::Bool(*f >= 0.0 && !f.is_nan())),
                    Value::String(s) => Ok(Value::Bool(!s.is_empty())),
                    Value::List(l) => Ok(Value::Bool(!l.borrow().is_empty())),
                    Value::Bool(b) => Ok(Value::Bool(*b)),
                    _ => Ok(Value::Bool(true)),
                }
            }))),
        );

        // glaikit - check if value is silly/wrong type for context
        globals.borrow_mut().define(
            "glaikit".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("glaikit", 2, |args| {
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
        // SET (CREEL) FUNCTIONS - A creel is a basket in Scots!
        // ============================================================

        // creel - create a new set from a list
        globals.borrow_mut().define(
            "creel".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("creel", 1, |args| {
                match &args[0] {
                    Value::List(list) => {
                        let items: HashSet<String> = list
                            .borrow()
                            .iter()
                            .map(|v| format!("{}", v))
                            .collect();
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
                    let item = format!("{}", args[1]);
                    set.borrow_mut().insert(item);
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
                    let item = format!("{}", args[1]);
                    set.borrow_mut().remove(&item);
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
                    let item = format!("{}", args[1]);
                    Ok(Value::Bool(set.borrow().contains(&item)))
                } else {
                    Err("is_in_creel() needs a creel (set)".to_string())
                }
            }))),
        );

        // creels_thegither - union of two sets (put them thegither!)
        globals.borrow_mut().define(
            "creels_thegither".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("creels_thegither", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::Set(a), Value::Set(b)) => {
                        let union: HashSet<String> = a
                            .borrow()
                            .union(&*b.borrow())
                            .cloned()
                            .collect();
                        Ok(Value::Set(Rc::new(RefCell::new(union))))
                    }
                    _ => Err("creels_thegither() needs two creels".to_string()),
                }
            }))),
        );

        // creels_baith - intersection of two sets (what's in baith!)
        globals.borrow_mut().define(
            "creels_baith".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("creels_baith", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::Set(a), Value::Set(b)) => {
                        let intersection: HashSet<String> = a
                            .borrow()
                            .intersection(&*b.borrow())
                            .cloned()
                            .collect();
                        Ok(Value::Set(Rc::new(RefCell::new(intersection))))
                    }
                    _ => Err("creels_baith() needs two creels".to_string()),
                }
            }))),
        );

        // creels_differ - difference of two sets (what's in a but no in b)
        globals.borrow_mut().define(
            "creels_differ".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("creels_differ", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::Set(a), Value::Set(b)) => {
                        let difference: HashSet<String> = a
                            .borrow()
                            .difference(&*b.borrow())
                            .cloned()
                            .collect();
                        Ok(Value::Set(Rc::new(RefCell::new(difference))))
                    }
                    _ => Err("creels_differ() needs two creels".to_string()),
                }
            }))),
        );

        // creel_tae_list - convert set to sorted list
        globals.borrow_mut().define(
            "creel_tae_list".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("creel_tae_list", 1, |args| {
                if let Value::Set(set) = &args[0] {
                    let mut items: Vec<String> = set.borrow().iter().cloned().collect();
                    items.sort();
                    let values: Vec<Value> = items.into_iter().map(Value::String).collect();
                    Ok(Value::List(Rc::new(RefCell::new(values))))
                } else {
                    Err("creel_tae_list() needs a creel".to_string())
                }
            }))),
        );

        // is_subset - check if one set is a subset of another (is a inside b?)
        globals.borrow_mut().define(
            "is_subset".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("is_subset", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::Set(a), Value::Set(b)) => {
                        Ok(Value::Bool(a.borrow().is_subset(&*b.borrow())))
                    }
                    _ => Err("is_subset() needs two creels".to_string()),
                }
            }))),
        );

        // is_superset - check if one set is a superset of another (does a contain aw o b?)
        globals.borrow_mut().define(
            "is_superset".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("is_superset", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::Set(a), Value::Set(b)) => {
                        Ok(Value::Bool(a.borrow().is_superset(&*b.borrow())))
                    }
                    _ => Err("is_superset() needs two creels".to_string()),
                }
            }))),
        );

        // is_disjoint - check if two sets have nae overlap
        globals.borrow_mut().define(
            "is_disjoint".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("is_disjoint", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::Set(a), Value::Set(b)) => {
                        Ok(Value::Bool(a.borrow().is_disjoint(&*b.borrow())))
                    }
                    _ => Err("is_disjoint() needs two creels".to_string()),
                }
            }))),
        );

        // empty_creel - create an empty set
        globals.borrow_mut().define(
            "empty_creel".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("empty_creel", 0, |_args| {
                Ok(Value::Set(Rc::new(RefCell::new(HashSet::new()))))
            }))),
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
    pub fn get_output(&self) -> &[String] {
        &self.output
    }

    /// Clear captured output
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
        // Resolve the module path
        let module_path = self.resolve_module_path(path)?;

        // Check fer circular imports
        if self.loaded_modules.contains(&module_path) {
            // Already loaded, that's fine - skip
            return Ok(Ok(Value::Nil));
        }

        // Read the module file
        let source = std::fs::read_to_string(&module_path).map_err(|_| {
            HaversError::ModuleNotFound {
                name: path.to_string(),
            }
        })?;

        // Parse the module
        let program = crate::parser::parse(&source).map_err(|e| {
            HaversError::ParseError {
                message: format!("Error in module '{}': {}", path, e),
                line: span.line,
            }
        })?;

        // Mark as loaded tae prevent circular imports
        self.loaded_modules.insert(module_path.clone());

        // Save the current directory and switch tae the module's directory
        let old_dir = self.current_dir.clone();
        if let Some(parent) = module_path.parent() {
            self.current_dir = parent.to_path_buf();
        }

        // Execute the module in a new environment that inherits fae globals
        let module_env = Rc::new(RefCell::new(Environment::with_enclosing(self.globals.clone())));
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
            let module_dict = Value::Dict(Rc::new(RefCell::new(exports)));
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

    /// Resolve a module path relative tae the current directory
    fn resolve_module_path(&self, path: &str) -> HaversResult<PathBuf> {
        let mut module_path = PathBuf::from(path);

        // Add .braw extension if not present
        if module_path.extension().is_none() {
            module_path.set_extension("braw");
        }

        // If it's a relative path, resolve it fae the current directory
        if module_path.is_relative() {
            module_path = self.current_dir.join(module_path);
        }

        // Canonicalize the path
        module_path.canonicalize().map_err(|_| {
            HaversError::ModuleNotFound {
                name: path.to_string(),
            }
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
                name, initializer, ..
            } => {
                let value = if let Some(init) = initializer {
                    self.evaluate(init)?
                } else {
                    Value::Nil
                };
                self.environment.borrow_mut().define(name.clone(), value);
                Ok(Ok(Value::Nil))
            }

            Stmt::Expression { expr, .. } => {
                let value = self.evaluate(expr)?;
                Ok(Ok(value))
            }

            Stmt::Block { statements, .. } => {
                self.execute_block(statements, None)
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let cond_value = self.evaluate(condition)?;
                if cond_value.is_truthy() {
                    self.execute_stmt_with_control(then_branch)
                } else if let Some(else_br) = else_branch {
                    self.execute_stmt_with_control(else_br)
                } else {
                    Ok(Ok(Value::Nil))
                }
            }

            Stmt::While {
                condition, body, ..
            } => {
                while self.evaluate(condition)?.is_truthy() {
                    match self.execute_stmt_with_control(body)? {
                        Ok(_) => {}
                        Err(ControlFlow::Break) => break,
                        Err(ControlFlow::Continue) => continue,
                        Err(ControlFlow::Return(v)) => return Ok(Err(ControlFlow::Return(v))),
                    }
                }
                Ok(Ok(Value::Nil))
            }

            Stmt::For {
                variable,
                iterable,
                body,
                span,
            } => {
                let iter_value = self.evaluate(iterable)?;

                let items: Vec<Value> = match iter_value {
                    Value::Range(range) => range.iter().map(Value::Integer).collect(),
                    Value::List(list) => list.borrow().clone(),
                    Value::String(s) => {
                        s.chars().map(|c| Value::String(c.to_string())).collect()
                    }
                    _ => {
                        return Err(HaversError::TypeError {
                            message: format!(
                                "Cannae iterate ower a {}",
                                iter_value.type_name()
                            ),
                            line: span.line,
                        });
                    }
                };

                for item in items {
                    self.environment
                        .borrow_mut()
                        .define(variable.clone(), item);
                    match self.execute_stmt_with_control(body)? {
                        Ok(_) => {}
                        Err(ControlFlow::Break) => break,
                        Err(ControlFlow::Continue) => continue,
                        Err(ControlFlow::Return(v)) => return Ok(Err(ControlFlow::Return(v))),
                    }
                }
                Ok(Ok(Value::Nil))
            }

            Stmt::Function {
                name,
                params,
                body,
                ..
            } => {
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

            Stmt::Return { value, .. } => {
                let ret_val = if let Some(expr) = value {
                    self.evaluate(expr)?
                } else {
                    Value::Nil
                };
                Ok(Err(ControlFlow::Return(ret_val)))
            }

            Stmt::Print { value, .. } => {
                let val = self.evaluate(value)?;
                let output = format!("{}", val);
                println!("{}", output);
                self.output.push(output);
                Ok(Ok(Value::Nil))
            }

            Stmt::Break { .. } => Ok(Err(ControlFlow::Break)),

            Stmt::Continue { .. } => Ok(Err(ControlFlow::Continue)),

            Stmt::Class {
                name,
                superclass,
                methods,
                span,
            } => {
                let super_class = if let Some(super_name) = superclass {
                    let super_val = self
                        .environment
                        .borrow()
                        .get(super_name)
                        .ok_or_else(|| HaversError::UndefinedVariable {
                            name: super_name.clone(),
                            line: span.line,
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

            Stmt::Struct { name, fields, .. } => {
                let structure = HaversStruct::new(name.clone(), fields.clone());
                self.environment
                    .borrow_mut()
                    .define(name.clone(), Value::Struct(Rc::new(structure)));
                Ok(Ok(Value::Nil))
            }

            Stmt::Import { path, alias, span } => {
                self.load_module(path, alias.as_deref(), *span)
            }

            Stmt::TryCatch {
                try_block,
                error_name,
                catch_block,
                ..
            } => {
                match self.execute_stmt_with_control(try_block) {
                    Ok(result) => Ok(result),
                    Err(e) => {
                        // Bind the error to the catch variable
                        self.environment
                            .borrow_mut()
                            .define(error_name.clone(), Value::String(e.to_string()));
                        self.execute_stmt_with_control(catch_block)
                    }
                }
            }

            Stmt::Match { value, arms, span } => {
                let val = self.evaluate(value)?;

                for arm in arms {
                    if self.pattern_matches(&arm.pattern, &val)? {
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
                let cond_value = self.evaluate(condition)?;
                if !cond_value.is_truthy() {
                    let msg = if let Some(msg_expr) = message {
                        let msg_val = self.evaluate(msg_expr)?;
                        msg_val.to_string()
                    } else {
                        "Assertion failed".to_string()
                    };
                    return Err(HaversError::AssertionFailed {
                        message: msg,
                        line: span.line,
                    });
                }
                Ok(Ok(Value::Nil))
            }

            Stmt::Destructure {
                patterns,
                value,
                span,
            } => {
                let val = self.evaluate(value)?;

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
                let rest_pos = patterns.iter().position(|p| matches!(p, DestructPattern::Rest(_)));

                // Calculate positions
                let before_rest = rest_pos.unwrap_or(patterns.len());
                let after_rest = if rest_pos.is_some() {
                    patterns.len() - rest_pos.unwrap() - 1
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
                    Value::Instance(inst) => inst
                        .borrow()
                        .get(property)
                        .ok_or_else(|| HaversError::UndefinedVariable {
                            name: property.clone(),
                            line: span.line,
                        }),
                    Value::Dict(dict) => dict
                        .borrow()
                        .get(property)
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
                    Value::Instance(inst) => {
                        inst.borrow_mut().set(property.clone(), val.clone());
                        Ok(val)
                    }
                    Value::Dict(dict) => {
                        dict.borrow_mut().insert(property.clone(), val.clone());
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
                            .ok_or_else(|| HaversError::IndexOutOfBounds {
                                index: *i,
                                size: s.len(),
                                line: span.line,
                            })
                    }
                    (Value::Dict(dict), Value::String(key)) => dict
                        .borrow()
                        .get(key)
                        .cloned()
                        .ok_or_else(|| HaversError::UndefinedVariable {
                            name: key.clone(),
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
                    (Value::Dict(dict), Value::String(key)) => {
                        dict.borrow_mut().insert(key.clone(), val.clone());
                        Ok(val)
                    }
                    (Value::Dict(dict), key) => {
                        // Convert non-string key to string
                        let key_str = format!("{}", key);
                        dict.borrow_mut().insert(key_str, val.clone());
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
                            (len + end).max(-1) as i64
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
                            (len + end).max(-1) as i64
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
                let mut map = HashMap::new();
                for (key, value) in pairs {
                    let k = self.evaluate(key)?;
                    let v = self.evaluate(value)?;
                    let key_str = match k {
                        Value::String(s) => s,
                        _ => format!("{}", k),
                    };
                    map.insert(key_str, v);
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
                    (Some(s), Some(e)) => Ok(Value::Range(RangeValue::new(s, e, *inclusive))),
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
                let prompt_val = self.evaluate(prompt)?;
                print!("{}", prompt_val);
                io::stdout().flush().unwrap();

                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .map_err(|e| HaversError::InternalError(e.to_string()))?;

                Ok(Value::String(input.trim().to_string()))
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
            Expr::Spread { span, .. } => Err(HaversError::TypeError {
                message: "The spread operator (...) can only be used in lists or function calls, ya numpty!".to_string(),
                line: span.line,
            }),

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
                (Value::String(a), Value::String(b)) => {
                    Ok(Value::String(format!("{}{}", a, b)))
                }
                (Value::String(a), b) => Ok(Value::String(format!("{}{}", a, b))),
                (a, Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                (Value::List(a), Value::List(b)) => {
                    let mut result = a.borrow().clone();
                    result.extend(b.borrow().clone());
                    Ok(Value::List(Rc::new(RefCell::new(result))))
                }
                _ => Err(HaversError::TypeError {
                    message: format!(
                        "Cannae add {} an' {}",
                        left.type_name(),
                        right.type_name()
                    ),
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
                match right {
                    Value::Integer(0) => return Err(HaversError::DivisionByZero { line }),
                    _ => {}
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

            BinaryOp::Less => self.compare(left, right, |a, b| a < b, line),
            BinaryOp::LessEqual => self.compare(left, right, |a, b| a <= b, line),
            BinaryOp::Greater => self.compare(left, right, |a, b| a > b, line),
            BinaryOp::GreaterEqual => self.compare(left, right, |a, b| a >= b, line),
        }
    }

    fn compare<F>(&self, left: &Value, right: &Value, cmp: F, line: usize) -> HaversResult<Value>
    where
        F: Fn(f64, f64) -> bool,
    {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                Ok(Value::Bool(cmp(*a as f64, *b as f64)))
            }
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(cmp(*a, *b))),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Bool(cmp(*a as f64, *b))),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Bool(cmp(*a, *b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(cmp(
                a.len() as f64,
                b.len() as f64,
            ))),
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
        match callee {
            Value::Function(func) => self.call_function(&func, args, line),
            Value::NativeFunction(native) => {
                if args.len() != native.arity {
                    return Err(HaversError::WrongArity {
                        name: native.name.clone(),
                        expected: native.arity,
                        got: args.len(),
                        line,
                    });
                }
                (native.func)(args).map_err(|e| HaversError::InternalError(e))
            }
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

                let mut fields = HashMap::new();
                for (field, value) in structure.fields.iter().zip(args) {
                    fields.insert(field.clone(), value);
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
    fn call_builtin_hof(&mut self, name: &str, args: Vec<Value>, line: usize) -> HaversResult<Value> {
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
                    _ => return Err(HaversError::TypeError {
                        message: "gaun() expects a list as first argument".to_string(),
                        line,
                    }),
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
                    _ => return Err(HaversError::TypeError {
                        message: "sieve() expects a list as first argument".to_string(),
                        line,
                    }),
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
                    _ => return Err(HaversError::TypeError {
                        message: "tumble() expects a list as first argument".to_string(),
                        line,
                    }),
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
                    _ => return Err(HaversError::TypeError {
                        message: "ilk() expects a list as first argument".to_string(),
                        line,
                    }),
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
                    _ => return Err(HaversError::TypeError {
                        message: "hunt() expects a list as first argument".to_string(),
                        line,
                    }),
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
                    _ => return Err(HaversError::TypeError {
                        message: "ony() expects a list as first argument".to_string(),
                        line,
                    }),
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
                    _ => return Err(HaversError::TypeError {
                        message: "aw() expects a list as first argument".to_string(),
                        line,
                    }),
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
                    _ => return Err(HaversError::TypeError {
                        message: "grup_up() expects a list as first argument".to_string(),
                        line,
                    }),
                };
                let func = args[1].clone();
                // Result is a dict where keys are the function results, values are lists
                let result = Rc::new(RefCell::new(std::collections::HashMap::new()));
                for item in list {
                    let key = self.call_value(func.clone(), vec![item.clone()], line)?;
                    let key_str = format!("{}", key);
                    let mut dict = result.borrow_mut();
                    let group = dict.entry(key_str).or_insert_with(|| {
                        Value::List(Rc::new(RefCell::new(Vec::new())))
                    });
                    if let Value::List(l) = group {
                        l.borrow_mut().push(item);
                    }
                }
                Ok(Value::Dict(result))
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
                    _ => return Err(HaversError::TypeError {
                        message: "pairt_by() expects a list as first argument".to_string(),
                        line,
                    }),
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
                        func.name, min_arity, max_arity, args.len()
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
        _line: usize,
    ) -> HaversResult<Value> {
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

        match self.execute_block(&func.body, Some(env))? {
            Ok(v) => Ok(v),
            Err(ControlFlow::Return(v)) => Ok(v),
            Err(ControlFlow::Break) => Ok(Value::Nil),
            Err(ControlFlow::Continue) => Ok(Value::Nil),
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    fn run(source: &str) -> HaversResult<Value> {
        let program = parse(source)?;
        let mut interp = Interpreter::new();
        interp.interpret(&program)
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
    fn test_if_statement() {
        let result = run(
            r#"
ken x = 10
ken result = 0
gin x > 5 {
    result = 1
} ither {
    result = 2
}
result
"#,
        )
        .unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_while_loop() {
        let result = run(
            r#"
ken sum = 0
ken i = 1
whiles i <= 5 {
    sum = sum + i
    i = i + 1
}
sum
"#,
        )
        .unwrap();
        assert_eq!(result, Value::Integer(15));
    }

    #[test]
    fn test_for_loop() {
        let result = run(
            r#"
ken sum = 0
fer i in 1..6 {
    sum = sum + i
}
sum
"#,
        )
        .unwrap();
        assert_eq!(result, Value::Integer(15));
    }

    #[test]
    fn test_function() {
        let result = run(
            r#"
dae add(a, b) {
    gie a + b
}
add(3, 4)
"#,
        )
        .unwrap();
        assert_eq!(result, Value::Integer(7));
    }

    #[test]
    fn test_recursion() {
        let result = run(
            r#"
dae factorial(n) {
    gin n <= 1 {
        gie 1
    }
    gie n * factorial(n - 1)
}
factorial(5)
"#,
        )
        .unwrap();
        assert_eq!(result, Value::Integer(120));
    }

    #[test]
    fn test_list() {
        let result = run(
            r#"
ken arr = [1, 2, 3]
arr[1]
"#,
        )
        .unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_dict() {
        let result = run(
            r#"
ken d = {"a": 1, "b": 2}
d["a"]
"#,
        )
        .unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_native_functions() {
        assert_eq!(run("len([1, 2, 3])").unwrap(), Value::Integer(3));
        assert_eq!(
            run(r#"len("hello")"#).unwrap(),
            Value::Integer(5)
        );
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
        if let Value::List(list) = result {
            let items = list.borrow();
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Integer(2));
            assert_eq!(items[1], Value::Integer(4));
            assert_eq!(items[2], Value::Integer(6));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_sieve_filter() {
        let result = run("ken nums = [1, 2, 3, 4, 5]\nsieve(nums, |x| x % 2 == 0)").unwrap();
        if let Value::List(list) = result {
            let items = list.borrow();
            assert_eq!(items.len(), 2);
            assert_eq!(items[0], Value::Integer(2));
            assert_eq!(items[1], Value::Integer(4));
        } else {
            panic!("Expected list");
        }
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
"#).unwrap();
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
x"#).unwrap(),
            Value::String("yes".to_string())
        );
        // Nested ternary
        assert_eq!(
            run("ken x = 5
ken result = gin x > 10 than 1 ither gin x > 3 than 2 ither 3
result").unwrap(),
            Value::Integer(2)
        );
    }

    #[test]
    fn test_slice_list() {
        // Basic slicing
        let result = run("ken x = [0, 1, 2, 3, 4]\nx[1:3]").unwrap();
        if let Value::List(list) = result {
            let list = list.borrow();
            assert_eq!(list.len(), 2);
            assert_eq!(list[0], Value::Integer(1));
            assert_eq!(list[1], Value::Integer(2));
        } else {
            panic!("Expected list");
        }

        // Slice to end
        let result = run("ken x = [0, 1, 2, 3, 4]\nx[3:]").unwrap();
        if let Value::List(list) = result {
            let list = list.borrow();
            assert_eq!(list.len(), 2);
            assert_eq!(list[0], Value::Integer(3));
            assert_eq!(list[1], Value::Integer(4));
        } else {
            panic!("Expected list");
        }

        // Slice from start
        let result = run("ken x = [0, 1, 2, 3, 4]\nx[:2]").unwrap();
        if let Value::List(list) = result {
            let list = list.borrow();
            assert_eq!(list.len(), 2);
            assert_eq!(list[0], Value::Integer(0));
            assert_eq!(list[1], Value::Integer(1));
        } else {
            panic!("Expected list");
        }
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
        if let Value::List(list) = result {
            let list = list.borrow();
            assert_eq!(list.len(), 2);
            assert_eq!(list[0], Value::Integer(3));
            assert_eq!(list[1], Value::Integer(4));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_slice_step() {
        // Every second element
        let result = run("ken x = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]\nx[::2]").unwrap();
        if let Value::List(list) = result {
            let list = list.borrow();
            assert_eq!(list.len(), 5);
            assert_eq!(list[0], Value::Integer(0));
            assert_eq!(list[1], Value::Integer(2));
            assert_eq!(list[4], Value::Integer(8));
        } else {
            panic!("Expected list");
        }

        // Every third element from 1 to 8
        let result = run("ken x = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]\nx[1:8:3]").unwrap();
        if let Value::List(list) = result {
            let list = list.borrow();
            assert_eq!(list.len(), 3); // 1, 4, 7
            assert_eq!(list[0], Value::Integer(1));
            assert_eq!(list[1], Value::Integer(4));
            assert_eq!(list[2], Value::Integer(7));
        } else {
            panic!("Expected list");
        }

        // Reverse a list with negative step
        let result = run("ken x = [0, 1, 2, 3, 4]\nx[::-1]").unwrap();
        if let Value::List(list) = result {
            let list = list.borrow();
            assert_eq!(list.len(), 5);
            assert_eq!(list[0], Value::Integer(4));
            assert_eq!(list[4], Value::Integer(0));
        } else {
            panic!("Expected list");
        }

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
        if let Value::List(list) = result {
            let list = list.borrow();
            assert_eq!(list.len(), 3);
        } else {
            panic!("Expected list");
        }

        // redd_up
        let result = run("redd_up([1, naething, 2, naething, 3])").unwrap();
        if let Value::List(list) = result {
            let list = list.borrow();
            assert_eq!(list.len(), 3);
        } else {
            panic!("Expected list");
        }
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
        if let Value::List(list) = result {
            let list = list.borrow();
            assert_eq!(list.len(), 3);
        } else {
            panic!("Expected list");
        }

        // ord and chr
        assert_eq!(run(r#"ord("A")"#).unwrap(), Value::Integer(65));
        assert_eq!(run("chr(65)").unwrap(), Value::String("A".to_string()));
    }

    #[test]
    fn test_creel_set() {
        // Create a set from a list
        let result = run("creel([1, 2, 2, 3, 3, 3])").unwrap();
        if let Value::Set(set) = result {
            let set = set.borrow();
            assert_eq!(set.len(), 3); // Duplicates removed
        } else {
            panic!("Expected creel");
        }

        // Create empty set
        let result = run("empty_creel()").unwrap();
        if let Value::Set(set) = result {
            assert!(set.borrow().is_empty());
        } else {
            panic!("Expected empty creel");
        }

        // Check membership
        let result = run(r#"
            ken s = creel(["apple", "banana", "cherry"])
            is_in_creel(s, "banana")
        "#).unwrap();
        assert_eq!(result, Value::Bool(true));

        let result = run(r#"
            ken s = creel(["apple", "banana", "cherry"])
            is_in_creel(s, "mango")
        "#).unwrap();
        assert_eq!(result, Value::Bool(false));

        // Union
        let result = run(r#"
            ken a = creel([1, 2, 3])
            ken b = creel([3, 4, 5])
            len(creels_thegither(a, b))
        "#).unwrap();
        assert_eq!(result, Value::Integer(5)); // 1, 2, 3, 4, 5

        // Intersection
        let result = run(r#"
            ken a = creel([1, 2, 3])
            ken b = creel([2, 3, 4])
            len(creels_baith(a, b))
        "#).unwrap();
        assert_eq!(result, Value::Integer(2)); // 2, 3

        // Difference
        let result = run(r#"
            ken a = creel([1, 2, 3])
            ken b = creel([2, 3, 4])
            len(creels_differ(a, b))
        "#).unwrap();
        assert_eq!(result, Value::Integer(1)); // just 1

        // Subset
        let result = run(r#"
            ken a = creel([1, 2])
            ken b = creel([1, 2, 3])
            is_subset(a, b)
        "#).unwrap();
        assert_eq!(result, Value::Bool(true));

        // Convert to list
        let result = run(r#"
            ken s = creel([3, 1, 2])
            creel_tae_list(s)
        "#).unwrap();
        if let Value::List(list) = result {
            let list = list.borrow();
            assert_eq!(list.len(), 3);
            // Should be sorted
            assert_eq!(list[0], Value::String("1".to_string()));
            assert_eq!(list[1], Value::String("2".to_string()));
            assert_eq!(list[2], Value::String("3".to_string()));
        } else {
            panic!("Expected list");
        }
    }
}
