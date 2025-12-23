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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{DictValue, RangeValue, SetValue, Value};
    use serde_json::Value as JsonValue;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::Mutex;
    use tempfile::tempdir;

    static LOG_LOCK: Mutex<()> = Mutex::new(());

    fn sample_record(fields: Vec<(String, Value)>) -> LogRecord {
        LogRecord {
            level: LogLevel::Blether,
            message: "hullo".to_string(),
            target: "tests".to_string(),
            file: "file.braw".to_string(),
            line: 42,
            fields,
            span_path: vec!["outer".to_string(), "inner".to_string()],
        }
    }

    #[test]
    fn test_log_filter_empty_rule_ignored() {
        let filter = parse_filter("=holler").unwrap();
        assert_eq!(filter.level_for_target("any"), LogLevel::Blether);
    }

    #[test]
    fn test_filter_parse_set_and_log_enabled() {
        let _lock = LOG_LOCK.lock().unwrap();

        let filter = parse_filter("mutter,net=holler,net.http=whisper").unwrap();
        assert_eq!(filter.default, LogLevel::Mutter);
        assert_eq!(filter.rules.len(), 2);

        set_filter("mutter,net=holler,net.http=whisper").unwrap();
        assert_eq!(get_filter(), "mutter,net=holler,net.http=whisper");

        assert!(log_enabled(LogLevel::Whisper, "net.http.server"));
        assert!(!log_enabled(LogLevel::Mutter, "net.udp"));
        assert!(log_enabled(LogLevel::Roar, "net.udp"));

        assert!(parse_filter("wat").is_err());
    }

    #[test]
    fn test_global_log_level_round_trip() {
        let _lock = LOG_LOCK.lock().unwrap();
        set_global_log_level(LogLevel::Roar);
        assert_eq!(get_global_log_level(), LogLevel::Roar);
        set_global_log_level(LogLevel::Blether);
    }

    #[cfg(coverage)]
    #[test]
    fn test_global_log_level_raw_fallback() {
        let _lock = LOG_LOCK.lock().unwrap();
        set_global_log_level_raw(99);
        assert_eq!(get_global_log_level(), LogLevel::Blether);
    }

    #[test]
    fn test_logger_formats_and_sinks() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("log.txt");

        let record = sample_record(vec![("n".to_string(), Value::Integer(1))]);

        let mut logger = LoggerCore {
            format: LogFormat::Text,
            color: false,
            timestamps: false,
            sinks: vec![LogSink::Memory {
                entries: Vec::new(),
                max: 2,
            }],
        };
        logger.log(&record);
        logger.log(&record);
        logger.log(&record);

        assert!(matches!(
            &logger.sinks[0],
            LogSink::Memory { entries, max } if *max == 2 && entries.len() == 2
        ));

        let mut file_logger = LoggerCore {
            format: LogFormat::Compact,
            color: false,
            timestamps: false,
            sinks: vec![LogSink::File {
                path: file_path.to_string_lossy().to_string(),
                append: false,
                file: None,
            }],
        };
        file_logger.log(&record);
        file_logger.log(&record);
        let contents = std::fs::read_to_string(&file_path).unwrap();
        assert!(contents.contains("hullo"));

        let mut bad_file_logger = LoggerCore {
            format: LogFormat::Text,
            color: false,
            timestamps: false,
            sinks: vec![LogSink::File {
                path: dir.path().to_string_lossy().to_string(),
                append: false,
                file: None,
            }],
        };
        bad_file_logger.log(&record);
        assert!(matches!(
            &bad_file_logger.sinks[0],
            LogSink::File { file, .. } if file.is_none()
        ));
    }

    #[test]
    fn test_logger_default_stdout_and_append_file() {
        let record = sample_record(vec![]);
        let mut logger = LoggerCore {
            timestamps: false,
            sinks: vec![LogSink::Stdout],
            ..LoggerCore::default()
        };
        logger.log(&record);

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("append.txt");
        let mut file_logger = LoggerCore {
            format: LogFormat::Text,
            color: false,
            timestamps: false,
            sinks: vec![LogSink::File {
                path: file_path.to_string_lossy().to_string(),
                append: true,
                file: None,
            }],
        };
        file_logger.log(&record);
        assert!(file_path.exists());
    }

    #[test]
    fn test_format_json_and_value_to_json_branches() {
        let list = Value::List(Rc::new(RefCell::new(vec![Value::Integer(1)])));
        let mut dict = DictValue::new();
        dict.set(
            Value::String("k".to_string()),
            Value::String("v".to_string()),
        );
        let dict = Value::Dict(Rc::new(RefCell::new(dict)));
        let mut set = SetValue::new();
        set.insert(Value::String("a".to_string()));
        let set = Value::Set(Rc::new(RefCell::new(set)));
        let bytes = Value::Bytes(Rc::new(RefCell::new(vec![1, 2, 3])));
        let range = Value::Range(RangeValue::new(1, 3, false));

        let fields = vec![
            ("nil".to_string(), Value::Nil),
            ("bool".to_string(), Value::Bool(true)),
            ("int".to_string(), Value::Integer(7)),
            ("float".to_string(), Value::Float(1.5)),
            ("string".to_string(), Value::String("hi".to_string())),
            ("list".to_string(), list),
            ("dict".to_string(), dict),
            ("set".to_string(), set),
            ("bytes".to_string(), bytes),
            ("other".to_string(), range),
        ];
        let record = sample_record(fields);
        let logger = LoggerCore {
            format: LogFormat::Json,
            color: false,
            timestamps: true,
            sinks: vec![LogSink::Stderr],
        };
        let json = logger.format_record(&record);
        let parsed: JsonValue = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("fields").is_some());
        assert!(parsed.get("span").is_some());
    }

    #[test]
    fn test_value_to_json_non_string_key() {
        let mut dict = DictValue::new();
        dict.set(Value::Integer(5), Value::String("v".to_string()));
        let value = Value::Dict(Rc::new(RefCell::new(dict)));
        let json = value_to_json(&value);
        let obj = json.as_object().unwrap();
        assert!(obj.contains_key("5"));
    }

    #[test]
    fn test_spans_fields_and_record_value() {
        let span = new_span(
            "outer".to_string(),
            LogLevel::Blether,
            "tests".to_string(),
            vec![("x".to_string(), Value::Integer(1))],
        );
        span_enter(span.clone());
        assert!(span_current().is_some());
        assert_eq!(span_path(), vec!["outer".to_string()]);
        assert!(span_exit(span.id).is_ok());
        assert!(span_exit(span.id).is_err());

        let span2 = new_span(
            "inner".to_string(),
            LogLevel::Mutter,
            "".to_string(),
            Vec::new(),
        );
        span_enter(span2.clone());
        assert!(span_exit(span.id).is_err());
        assert!(span_exit(span2.id).is_ok());

        let mut dict = DictValue::new();
        dict.set(Value::String("k".to_string()), Value::Integer(2));
        let fields = fields_from_dict(&Value::Dict(Rc::new(RefCell::new(dict)))).unwrap();
        assert_eq!(fields.len(), 1);

        let mut bad_dict = DictValue::new();
        bad_dict.set(Value::Integer(1), Value::Integer(2));
        assert!(fields_from_dict(&Value::Dict(Rc::new(RefCell::new(bad_dict)))).is_err());

        let record = sample_record(vec![("a".to_string(), Value::Integer(1))]);
        let val = record_to_value(&record, Some("now".to_string()));
        assert!(matches!(
            val,
            Value::Dict(ref map)
                if map.borrow().contains_key(&Value::String("timestamp".to_string()))
        ));
    }

    #[test]
    fn test_fields_from_dict_non_dict_error() {
        assert!(fields_from_dict(&Value::Integer(1)).is_err());
    }

    #[test]
    fn test_span_handle_properties() {
        let span = new_span(
            "test".to_string(),
            LogLevel::Holler,
            "target".to_string(),
            vec![("k".to_string(), Value::Integer(1))],
        );
        let handle = LogSpanHandle::new(span);
        assert_eq!(
            handle.get("name").unwrap(),
            Value::String("test".to_string())
        );
        assert_eq!(
            handle.get("target").unwrap(),
            Value::String("target".to_string())
        );
        assert_eq!(
            handle.get("level").unwrap(),
            Value::String("holler".to_string())
        );
        let fields = handle.get("fields").unwrap();
        assert!(matches!(
            fields,
            Value::Dict(ref dict)
                if dict.borrow().get(&Value::String("k".to_string())) == Some(&Value::Integer(1))
        ));
    }

    #[test]
    fn test_span_handle_misc_accessors() {
        let span = new_span(
            "misc".to_string(),
            LogLevel::Blether,
            "target".to_string(),
            vec![],
        );
        let handle = LogSpanHandle::new(span);
        assert_eq!(handle.type_name(), "log_span");
        assert_eq!(handle.get("nope").unwrap(), Value::Nil);
        handle.set("anything", Value::Nil).unwrap();
        handle.call("noop", vec![]).unwrap();
        assert!(handle.as_any().is::<LogSpanHandle>());
    }

    #[test]
    fn test_timestamp_string_format() {
        let ts = timestamp_string();
        assert!(ts.contains('-'));
        assert!(ts.contains(':'));
    }
}
