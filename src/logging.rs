use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io::Write;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock};

use chrono::Local;
use serde_json::{json, Map, Value as JsonValue};

use crate::ast::LogLevel;
use crate::error::HaversResult;
use crate::value::{DictValue, NativeObject, Value};

/// Global log level (default: Blether/INFO)
static GLOBAL_LOG_LEVEL: AtomicU8 = AtomicU8::new(LogLevel::Blether as u8);

#[derive(Debug, Clone, Copy)]
pub enum LogFormat {
    Text,
    Json,
    Compact,
}

#[derive(Debug, Clone)]
pub struct LogFilter {
    pub default: LogLevel,
    pub rules: Vec<(String, LogLevel)>,
}

impl LogFilter {
    fn level_for_target(&self, target: &str) -> LogLevel {
        let mut best: Option<(usize, LogLevel)> = None;
        for (rule_target, level) in &self.rules {
            if rule_target.is_empty() {
                continue;
            }
            if target.starts_with(rule_target) {
                let len = rule_target.len();
                if best.map(|(best_len, _)| len > best_len).unwrap_or(true) {
                    best = Some((len, *level));
                }
            }
        }
        best.map(|(_, level)| level).unwrap_or(self.default)
    }
}

static LOG_FILTER: OnceLock<Mutex<LogFilter>> = OnceLock::new();
static LOG_FILTER_SPEC: OnceLock<Mutex<String>> = OnceLock::new();

fn filter_state() -> &'static Mutex<LogFilter> {
    LOG_FILTER.get_or_init(|| {
        Mutex::new(LogFilter {
            default: LogLevel::Blether,
            rules: Vec::new(),
        })
    })
}

fn filter_spec() -> &'static Mutex<String> {
    LOG_FILTER_SPEC.get_or_init(|| Mutex::new(String::new()))
}

pub fn parse_filter(spec: &str) -> Result<LogFilter, String> {
    let mut default = None;
    let mut rules = Vec::new();
    for part in spec.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        if let Some((target, level_str)) = part.split_once('=') {
            let level = LogLevel::parse_level(level_str.trim())
                .ok_or_else(|| format!("Invalid log level '{}'", level_str.trim()))?;
            rules.push((target.trim().to_string(), level));
        } else {
            let level = LogLevel::parse_level(part)
                .ok_or_else(|| format!("Invalid log level '{}'", part))?;
            default = Some(level);
        }
    }

    Ok(LogFilter {
        default: default.unwrap_or(LogLevel::Blether),
        rules,
    })
}

pub fn set_filter(spec: &str) -> Result<(), String> {
    let filter = parse_filter(spec)?;
    let mut guard = filter_state()
        .lock()
        .map_err(|_| "log filter lock poisoned".to_string())?;
    *guard = filter;
    GLOBAL_LOG_LEVEL.store(guard.default as u8, Ordering::Relaxed);
    let mut spec_guard = filter_spec()
        .lock()
        .map_err(|_| "log filter lock poisoned".to_string())?;
    *spec_guard = spec.to_string();
    Ok(())
}

pub fn get_filter() -> String {
    filter_spec()
        .lock()
        .map(|s| s.clone())
        .unwrap_or_else(|_| String::new())
}

pub fn log_enabled(level: LogLevel, target: &str) -> bool {
    let filter = filter_state().lock().unwrap_or_else(|e| e.into_inner());
    let effective = filter.level_for_target(target);
    (level as u8) <= (effective as u8)
}

pub fn get_global_log_level() -> LogLevel {
    match GLOBAL_LOG_LEVEL.load(Ordering::Relaxed) {
        0 => LogLevel::Wheesht,
        1 => LogLevel::Roar,
        2 => LogLevel::Holler,
        3 => LogLevel::Blether,
        4 => LogLevel::Mutter,
        5 => LogLevel::Whisper,
        _ => LogLevel::Blether,
    }
}

pub fn set_global_log_level(level: LogLevel) {
    GLOBAL_LOG_LEVEL.store(level as u8, Ordering::Relaxed);
    if let Ok(mut guard) = filter_state().lock() {
        guard.default = level;
    }
}

#[cfg(coverage)]
#[allow(dead_code)]
pub fn set_global_log_level_raw(level: u8) {
    GLOBAL_LOG_LEVEL.store(level, Ordering::Relaxed);
}

#[derive(Debug, Clone)]
pub struct LogRecord {
    pub level: LogLevel,
    pub message: String,
    pub target: String,
    pub file: String,
    pub line: usize,
    pub fields: Vec<(String, Value)>,
    pub span_path: Vec<String>,
}

#[derive(Debug)]
pub enum LogSink {
    Stderr,
    Stdout,
    File {
        path: String,
        append: bool,
        file: Option<std::fs::File>,
    },
    Memory {
        entries: Vec<String>,
        max: usize,
    },
}

#[derive(Debug)]
pub struct LoggerCore {
    pub format: LogFormat,
    pub color: bool,
    pub timestamps: bool,
    pub sinks: Vec<LogSink>,
}

impl LoggerCore {
    pub fn new() -> Self {
        LoggerCore {
            format: LogFormat::Text,
            color: false,
            timestamps: true,
            sinks: vec![LogSink::Stderr],
        }
    }

    pub fn log(&mut self, record: &LogRecord) {
        let formatted = self.format_record(record);
        for sink in &mut self.sinks {
            match sink {
                LogSink::Stderr => {
                    eprintln!("{}", formatted);
                }
                LogSink::Stdout => {
                    println!("{}", formatted);
                }
                LogSink::File { path, append, file } => {
                    if file.is_none() {
                        let mut opts = OpenOptions::new();
                        opts.create(true).write(true);
                        if *append {
                            opts.append(true);
                        } else {
                            opts.truncate(true);
                        }
                        match opts.open(path.as_str()) {
                            Ok(handle) => {
                                *file = Some(handle);
                            }
                            Err(err) => {
                                eprintln!("Warning: Couldnae open log file '{}': {}", path, err);
                            }
                        }
                    }
                    if let Some(handle) = file {
                        let _ = writeln!(handle, "{}", formatted);
                    }
                }
                LogSink::Memory { entries, max } => {
                    entries.push(formatted.clone());
                    if entries.len() > *max {
                        let drain = entries.len() - *max;
                        entries.drain(0..drain);
                    }
                }
            }
        }
    }

    fn format_record(&self, record: &LogRecord) -> String {
        match self.format {
            LogFormat::Json => self.format_json(record),
            LogFormat::Compact => self.format_compact(record),
            LogFormat::Text => self.format_text(record),
        }
    }

    fn format_text(&self, record: &LogRecord) -> String {
        let timestamp = if self.timestamps {
            format!("{}", Local::now().format("%Y-%m-%d %H:%M:%S%.3f"))
        } else {
            String::new()
        };
        let thread_id = std::thread::current().id();
        let thread_num: u64 = format!("{:?}", thread_id)
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse()
            .unwrap_or(0)
            % 10000;

        let mut parts = Vec::new();
        parts.push(format!("[{:7}]", record.level.name()));
        if !timestamp.is_empty() {
            parts.push(timestamp);
        }
        parts.push(format!("[thread:{:04}]", thread_num));
        if !record.target.is_empty() {
            parts.push(record.target.clone());
        }
        parts.push(format!("{}:{}", record.file, record.line));

        let mut msg = record.message.clone();
        if !record.fields.is_empty() {
            let fields = format_fields(&record.fields);
            msg = format!("{} {}", msg, fields);
        }
        if !record.span_path.is_empty() {
            msg = format!("{} span={}", msg, record.span_path.join(">"));
        }

        format!("{} | {}", parts.join(" "), msg)
    }

    fn format_compact(&self, record: &LogRecord) -> String {
        let mut msg = record.message.clone();
        if !record.fields.is_empty() {
            msg = format!("{} {}", msg, format_fields(&record.fields));
        }
        if !record.span_path.is_empty() {
            msg = format!("{} span={}", msg, record.span_path.join(">"));
        }
        format!("[{}] {}", record.level.name(), msg)
    }

    fn format_json(&self, record: &LogRecord) -> String {
        let mut obj = Map::new();
        obj.insert(
            "ts".to_string(),
            JsonValue::String(format!("{}", Local::now().format("%Y-%m-%d %H:%M:%S%.3f"))),
        );
        obj.insert(
            "level".to_string(),
            JsonValue::String(record.level.name().to_string()),
        );
        obj.insert(
            "target".to_string(),
            JsonValue::String(record.target.clone()),
        );
        obj.insert("file".to_string(), JsonValue::String(record.file.clone()));
        obj.insert("line".to_string(), json!(record.line));
        obj.insert("msg".to_string(), JsonValue::String(record.message.clone()));

        let mut fields = Map::new();
        for (k, v) in &record.fields {
            fields.insert(k.clone(), value_to_json(v));
        }
        obj.insert("fields".to_string(), JsonValue::Object(fields));
        obj.insert(
            "span".to_string(),
            JsonValue::Array(
                record
                    .span_path
                    .iter()
                    .cloned()
                    .map(JsonValue::String)
                    .collect(),
            ),
        );

        JsonValue::Object(obj).to_string()
    }
}

impl Default for LoggerCore {
    fn default() -> Self {
        Self::new()
    }
}

fn format_fields(fields: &[(String, Value)]) -> String {
    fields
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(" ")
}

fn value_to_json(value: &Value) -> JsonValue {
    match value {
        Value::Nil => JsonValue::Null,
        Value::Bool(b) => JsonValue::Bool(*b),
        Value::Integer(n) => json!(n),
        Value::Float(f) => json!(f),
        Value::String(s) => JsonValue::String(s.clone()),
        Value::List(list) => JsonValue::Array(list.borrow().iter().map(value_to_json).collect()),
        Value::Dict(dict) => {
            let mut map = Map::new();
            for (k, v) in dict.borrow().iter() {
                let key = match k {
                    Value::String(s) => s.clone(),
                    _ => format!("{}", k),
                };
                map.insert(key, value_to_json(v));
            }
            JsonValue::Object(map)
        }
        Value::Set(set) => JsonValue::Array(set.borrow().iter().map(value_to_json).collect()),
        Value::Bytes(bytes) => JsonValue::String(format!("bytes[{}]", bytes.borrow().len())),
        other => JsonValue::String(format!("{}", other)),
    }
}

#[derive(Debug)]
pub struct LogSpan {
    pub id: u64,
    pub name: String,
    pub level: LogLevel,
    pub target: String,
    pub fields: Vec<(String, Value)>,
}

#[derive(Debug)]
pub struct LogSpanHandle {
    span: Rc<LogSpan>,
}

impl LogSpanHandle {
    pub fn new(span: Rc<LogSpan>) -> Self {
        LogSpanHandle { span }
    }

    pub fn span(&self) -> Rc<LogSpan> {
        self.span.clone()
    }
}

impl NativeObject for LogSpanHandle {
    fn type_name(&self) -> &str {
        "log_span"
    }

    fn get(&self, prop: &str) -> HaversResult<Value> {
        match prop {
            "name" => Ok(Value::String(self.span.name.clone())),
            "target" => Ok(Value::String(self.span.target.clone())),
            "level" => Ok(Value::String(self.span.level.name().to_lowercase())),
            "fields" => {
                let mut dict = DictValue::new();
                for (k, v) in &self.span.fields {
                    dict.set(Value::String(k.clone()), v.clone());
                }
                Ok(Value::Dict(Rc::new(RefCell::new(dict))))
            }
            _ => Ok(Value::Nil),
        }
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

thread_local! {
    static LOG_SPAN_STACK: RefCell<Vec<Rc<LogSpan>>> = const { RefCell::new(Vec::new()) };
}

static LOG_SPAN_ID: AtomicU64 = AtomicU64::new(1);

pub fn new_span(
    name: String,
    level: LogLevel,
    target: String,
    fields: Vec<(String, Value)>,
) -> Rc<LogSpan> {
    let id = LOG_SPAN_ID.fetch_add(1, Ordering::Relaxed);
    Rc::new(LogSpan {
        id,
        name,
        level,
        target,
        fields,
    })
}

pub fn span_enter(span: Rc<LogSpan>) {
    LOG_SPAN_STACK.with(|stack| stack.borrow_mut().push(span));
}

pub fn span_exit(span_id: u64) -> Result<(), String> {
    LOG_SPAN_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        if let Some(top) = stack.pop() {
            if top.id != span_id {
                stack.push(top);
                return Err("log_span_exit() got a mismatched span".to_string());
            }
            Ok(())
        } else {
            Err("log_span_exit() called with nae active spans".to_string())
        }
    })
}

pub fn span_current() -> Option<Rc<LogSpan>> {
    LOG_SPAN_STACK.with(|stack| stack.borrow().last().cloned())
}

pub fn span_path() -> Vec<String> {
    LOG_SPAN_STACK.with(|stack| stack.borrow().iter().map(|s| s.name.clone()).collect())
}

pub fn fields_from_dict(value: &Value) -> Result<Vec<(String, Value)>, String> {
    let dict = match value {
        Value::Dict(d) => d.clone(),
        _ => return Err("Expected dict for log fields".to_string()),
    };
    let mut fields = Vec::new();
    for (k, v) in dict.borrow().iter() {
        let key = match k {
            Value::String(s) => s.clone(),
            _ => return Err("Log field keys must be strings".to_string()),
        };
        fields.push((key, v.clone()));
    }
    Ok(fields)
}

pub fn record_to_value(record: &LogRecord, timestamp: Option<String>) -> Value {
    let mut dict = DictValue::new();
    dict.set(
        Value::String("level".to_string()),
        Value::String(record.level.name().to_lowercase()),
    );
    dict.set(
        Value::String("message".to_string()),
        Value::String(record.message.clone()),
    );
    dict.set(
        Value::String("target".to_string()),
        Value::String(record.target.clone()),
    );
    dict.set(
        Value::String("file".to_string()),
        Value::String(record.file.clone()),
    );
    dict.set(
        Value::String("line".to_string()),
        Value::Integer(record.line as i64),
    );

    if let Some(ts) = timestamp {
        dict.set(Value::String("timestamp".to_string()), Value::String(ts));
    }

    let mut fields_dict = DictValue::new();
    for (k, v) in &record.fields {
        fields_dict.set(Value::String(k.clone()), v.clone());
    }
    dict.set(
        Value::String("fields".to_string()),
        Value::Dict(Rc::new(RefCell::new(fields_dict))),
    );

    let span_list = record
        .span_path
        .iter()
        .cloned()
        .map(Value::String)
        .collect::<Vec<_>>();
    dict.set(
        Value::String("span".to_string()),
        Value::List(Rc::new(RefCell::new(span_list))),
    );

    Value::Dict(Rc::new(RefCell::new(dict)))
}

pub fn timestamp_string() -> String {
    format!("{}", Local::now().format("%Y-%m-%d %H:%M:%S%.3f"))
}
