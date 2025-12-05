use thiserror::Error;

/// Scots error messages - gie the user a guid tellin' aff!
#[derive(Error, Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum HaversError {
    #[error("Och! Ah dinnae ken whit '{lexeme}' is at line {line}, column {column}")]
    UnkentToken {
        lexeme: String,
        line: usize,
        column: usize,
    },

    #[error("Haud yer wheesht! Unexpected '{found}' at line {line} - ah wis expectin' {expected}")]
    UnexpectedToken {
        expected: String,
        found: String,
        line: usize,
    },

    #[error("Awa' an bile yer heid! '{name}' hasnae been defined yet at line {line}")]
    UndefinedVariable { name: String, line: usize },

    #[error("Ye numpty! Tryin' tae divide by zero at line {line}")]
    DivisionByZero { line: usize },

    #[error("That's pure mince! Type error at line {line}: {message}")]
    TypeError { message: String, line: usize },

    #[error("Whit's aw this aboot? '{name}' isnae a function at line {line}")]
    NotCallable { name: String, line: usize },

    #[error("Yer bum's oot the windae! Function '{name}' expects {expected} arguments but ye gave it {got} at line {line}")]
    WrongArity {
        name: String,
        expected: usize,
        got: usize,
        line: usize,
    },

    #[error("Hoachin'! Index {index} is oot o' bounds (size is {size}) at line {line}")]
    IndexOutOfBounds {
        index: i64,
        size: usize,
        line: usize,
    },

    #[error("Dinnae be daft! Cannae read the file '{path}': {reason}")]
    FileError { path: String, reason: String },

    #[error("Yer code's a richt guddle! Parser gave up at line {line}: {message}")]
    ParseError { message: String, line: usize },

    #[error("Jings! Something went awfy wrang: {0}")]
    InternalError(String),

    #[error("Wheesht! Break statement ootside a loop at line {line} - ye can only brak fae inside a whiles or fer loop!")]
    BreakOutsideLoop { line: usize },

    #[error("Haud on there! Continue statement ootside a loop at line {line} - ye can only haud inside a whiles or fer loop!")]
    ContinueOutsideLoop { line: usize },

    #[error("Stack's fair puggled! Too many nested calls at line {line} - yer recursion's gone radge!")]
    StackOverflow { line: usize },

    #[error("Cannae find module '{name}' - hae ye checked the path is richt?")]
    ModuleNotFound { name: String },

    #[error("That string's no' finished! Missin' closing quote at line {line}")]
    UnterminatedString { line: usize },

    #[error("Yer number's aw wrang at line {line}: {value}")]
    InvalidNumber { value: String, line: usize },

    #[error("Haud yer horses! '{name}' is awready defined at line {line}")]
    AlreadyDefined { name: String, line: usize },

    #[error("Whit are ye playin' at? '{name}' isnae an object at line {line}")]
    NotAnObject { name: String, line: usize },

    #[error("Och away! '{property}' doesnae exist on this object at line {line}")]
    UndefinedProperty { property: String, line: usize },

    #[error("Yer loop's gone doolally! Infinite loop detected at line {line}")]
    InfiniteLoop { line: usize },

    #[error("That's no' a list, ya bampot! Expected a list at line {line}")]
    NotAList { line: usize },

    #[error("That's no' a dictionary! Expected a dict at line {line}")]
    NotADict { line: usize },

    #[error("Key '{key}' doesnae exist in the dictionary at line {line}")]
    KeyNotFound { key: String, line: usize },

    #[error("Ye cannae dae that! {operation} is no' allowed at line {line}")]
    InvalidOperation { operation: String, line: usize },

    #[error("The import's gone in a fankle! Circular import detected: {path}")]
    CircularImport { path: String },

    #[error("Mak siccar failed at line {line}! {message}")]
    AssertionFailed { message: String, line: usize },

    #[error("Ye've fair scunnered it! Return statement ootside a function at line {line}")]
    ReturnOutsideFunction { line: usize },

    #[error("Haud on! Cannae iterate over a {type_name} at line {line} - need a list or range")]
    NotIterable { type_name: String, line: usize },

    #[error("Yer pattern's aw wrang at line {line}: {message}")]
    PatternError { message: String, line: usize },
}

impl HaversError {
    pub fn line(&self) -> Option<usize> {
        match self {
            HaversError::UnkentToken { line, .. } => Some(*line),
            HaversError::UnexpectedToken { line, .. } => Some(*line),
            HaversError::UndefinedVariable { line, .. } => Some(*line),
            HaversError::DivisionByZero { line } => Some(*line),
            HaversError::TypeError { line, .. } => Some(*line),
            HaversError::NotCallable { line, .. } => Some(*line),
            HaversError::WrongArity { line, .. } => Some(*line),
            HaversError::IndexOutOfBounds { line, .. } => Some(*line),
            HaversError::ParseError { line, .. } => Some(*line),
            HaversError::BreakOutsideLoop { line } => Some(*line),
            HaversError::ContinueOutsideLoop { line } => Some(*line),
            HaversError::StackOverflow { line } => Some(*line),
            HaversError::UnterminatedString { line } => Some(*line),
            HaversError::InvalidNumber { line, .. } => Some(*line),
            HaversError::AlreadyDefined { line, .. } => Some(*line),
            HaversError::NotAnObject { line, .. } => Some(*line),
            HaversError::UndefinedProperty { line, .. } => Some(*line),
            HaversError::InfiniteLoop { line } => Some(*line),
            HaversError::NotAList { line } => Some(*line),
            HaversError::NotADict { line } => Some(*line),
            HaversError::KeyNotFound { line, .. } => Some(*line),
            HaversError::InvalidOperation { line, .. } => Some(*line),
            HaversError::AssertionFailed { line, .. } => Some(*line),
            HaversError::ReturnOutsideFunction { line } => Some(*line),
            HaversError::NotIterable { line, .. } => Some(*line),
            HaversError::PatternError { line, .. } => Some(*line),
            _ => None,
        }
    }
}

/// Scots phrases fer random error decoration
pub fn random_scots_exclamation() -> &'static str {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as usize;

    const PHRASES: &[&str] = &[
        "Och naw!",
        "Jings crivvens!",
        "Haud yer wheesht!",
        "Michty me!",
        "Hoots mon!",
        "Blimey!",
        "Ach, fer cryin' oot loud!",
        "By the wee man!",
        "Guid grief!",
        "Haud the bus!",
    ];

    PHRASES[seed % PHRASES.len()]
}

pub type HaversResult<T> = Result<T, HaversError>;

/// A wee helper tae format errors bonnie-like
pub fn format_error_context(source: &str, line: usize) -> String {
    let lines: Vec<&str> = source.lines().collect();
    if line == 0 || line > lines.len() {
        return String::new();
    }

    let mut result = String::new();
    let line_idx = line - 1;

    // Show a wee bit o' context
    if line_idx > 0 {
        result.push_str(&format!("  {} | {}\n", line - 1, lines[line_idx - 1]));
    }
    result.push_str(&format!("> {} | {}\n", line, lines[line_idx]));
    if line_idx + 1 < lines.len() {
        result.push_str(&format!("  {} | {}\n", line + 1, lines[line_idx + 1]));
    }

    result
}
