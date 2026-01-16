#![cfg(coverage)]

use mdhavers::ast::LogLevel;
use mdhavers::logging::{
    log_enabled, new_span, parse_filter, record_to_value, set_filter, span_enter, span_exit,
    LogFormat, LogRecord, LogSink, LoggerCore,
};
use mdhavers::value::Value;
use tempfile::tempdir;

#[test]
fn logging_dependency_branch_matrix_for_coverage() {
    let _ = parse_filter("=holler").unwrap();
    set_filter("=holler").unwrap();
    let _ = log_enabled(LogLevel::Blether, "any");

    set_filter("net.http=whisper,net=roar").unwrap();
    let _ = log_enabled(LogLevel::Mutter, "net.http.server");
    set_filter("net=roar,net.http=whisper").unwrap();
    let _ = log_enabled(LogLevel::Mutter, "net.http.server");

    let record_empty = LogRecord {
        level: LogLevel::Blether,
        message: "hullo".to_string(),
        target: String::new(),
        file: "file.braw".to_string(),
        line: 42,
        fields: Vec::new(),
        span_path: Vec::new(),
    };
    let record_full = LogRecord {
        level: LogLevel::Blether,
        message: "hullo".to_string(),
        target: "tests".to_string(),
        file: "file.braw".to_string(),
        line: 42,
        fields: vec![("k".to_string(), Value::Integer(1))],
        span_path: vec!["span".to_string()],
    };

    let mut logger = LoggerCore {
        format: LogFormat::Text,
        color: false,
        timestamps: false,
        sinks: vec![LogSink::Memory {
            entries: Vec::new(),
            max: 10,
        }],
    };
    logger.log(&record_empty);
    logger.timestamps = true;
    logger.log(&record_full);
    logger.format = LogFormat::Compact;
    logger.log(&record_empty);
    logger.log(&record_full);

    let _ = record_to_value(&record_full, Some("ts".to_string()));
    let _ = record_to_value(&record_full, None);

    let span = new_span(
        "span".to_string(),
        LogLevel::Blether,
        "target".to_string(),
        vec![],
    );
    span_enter(span.clone());
    let _ = span_exit(span.id + 1);
    span_exit(span.id).unwrap();

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("log.txt");

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
    file_logger.log(&record_full);
    file_logger.log(&record_full);

    let mut bad_logger = LoggerCore {
        format: LogFormat::Text,
        color: false,
        timestamps: false,
        sinks: vec![LogSink::File {
            path: dir.path().to_string_lossy().to_string(),
            append: false,
            file: None,
        }],
    };
    bad_logger.log(&record_full);

    let mut memory_logger = LoggerCore {
        format: LogFormat::Text,
        color: false,
        timestamps: false,
        sinks: vec![LogSink::Memory {
            entries: Vec::new(),
            max: 0,
        }],
    };
    memory_logger.log(&record_full);
}
