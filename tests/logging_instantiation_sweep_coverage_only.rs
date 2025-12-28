#![cfg(coverage)]

use std::cell::RefCell;
use std::rc::Rc;

use mdhavers::ast::LogLevel;
use mdhavers::logging::{self, LogFormat, LogRecord, LogSink, LoggerCore};
use mdhavers::value::{DictValue, NativeObject, SetValue, Value};

#[test]
fn logging_instantiations_are_covered_in_dependency_instance() {
    // Mutates global state; restore at end to avoid leaking between tests.
    let previous_filter = logging::get_filter();

    logging::set_filter("blether,,examples=roar,examples.sub=mutter").expect("set_filter");
    let err = logging::parse_filter("net=nae-a-level").unwrap_err();
    assert!(err.contains("Invalid log level"));

    // Exercise LogFilter::level_for_target's "best match" selection logic (overlapping targets).
    assert!(logging::log_enabled(LogLevel::Roar, "examples.sub.feature"));
    assert!(!logging::log_enabled(LogLevel::Blether, "examples"));

    // Exercise span stack + span_path closures.
    let root = logging::new_span(
        "root".to_string(),
        LogLevel::Blether,
        "tests.logging".to_string(),
        vec![("k".to_string(), Value::Integer(1))],
    );
    logging::span_enter(root.clone());
    let child = logging::new_span(
        "child".to_string(),
        LogLevel::Blether,
        "tests.logging".to_string(),
        Vec::new(),
    );
    logging::span_enter(child.clone());

    let path = logging::span_path();
    assert_eq!(path, vec!["root".to_string(), "child".to_string()]);

    logging::span_exit(child.id).expect("span_exit child");
    logging::span_exit(root.id).expect("span_exit root");

    // Exercise LogSpanHandle NativeObject methods.
    let handle = logging::LogSpanHandle::new(root);
    let _ = handle.get("name").expect("get name");
    let _ = handle.get("fields").expect("get fields");
    let _ = handle.set("ignored", Value::Integer(1)).expect("set");
    let _ = handle.call("ignored", Vec::new()).expect("call");

    // Exercise LoggerCore formatters + value_to_json.
    let mut dict = DictValue::new();
    dict.set(Value::String("a".to_string()), Value::Bool(true));

    let mut set = SetValue::new();
    set.insert(Value::Integer(1));

    let record = LogRecord {
        level: LogLevel::Blether,
        message: "hi".to_string(),
        target: "examples.logging".to_string(),
        file: "file.braw".to_string(),
        line: 1,
        fields: vec![
            (
                "list".to_string(),
                Value::List(Rc::new(RefCell::new(vec![Value::Integer(1)]))),
            ),
            ("dict".to_string(), Value::Dict(Rc::new(RefCell::new(dict)))),
            ("set".to_string(), Value::Set(Rc::new(RefCell::new(set)))),
            (
                "bytes".to_string(),
                Value::Bytes(Rc::new(RefCell::new(vec![0u8, 1, 2]))),
            ),
        ],
        span_path: vec!["root".to_string()],
    };

    let mut core = LoggerCore {
        format: LogFormat::Json,
        color: false,
        timestamps: false,
        sinks: vec![LogSink::Memory {
            entries: Vec::new(),
            max: 8,
        }],
    };

    core.log(&record);
    core.format = LogFormat::Compact;
    core.log(&record);

    let _ = LoggerCore::default();

    let _ = logging::set_filter(&previous_filter);
}
