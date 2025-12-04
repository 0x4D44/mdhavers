use thiserror::Error;

/// Scots error messages - gie the user a guid tellin' aff!
#[derive(Error, Debug, Clone, PartialEq)]
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

    #[error("Wheesht! Break statement ootside a loop at line {line}")]
    BreakOutsideLoop { line: usize },

    #[error("Haud on! Continue statement ootside a loop at line {line}")]
    ContinueOutsideLoop { line: usize },

    #[error("Stack's fair puggled! Too many nested calls at line {line}")]
    StackOverflow { line: usize },

    #[error("Cannae find module '{name}' - hae ye checked it exists?")]
    ModuleNotFound { name: String },

    #[error("That string's no' finished! Missin' closing quote at line {line}")]
    UnterminatedString { line: usize },

    #[error("Yer number's aw wrang at line {line}: {value}")]
    InvalidNumber { value: String, line: usize },
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
            _ => None,
        }
    }
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
