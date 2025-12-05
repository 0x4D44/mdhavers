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
        "Help ma boab!",
        "Crivvens!",
        "Whit a scunner!",
        "Aw naw!",
        "Dearie me!",
        "Sakes alive!",
        "Whit in the name o' the wee man!",
        "For ony favour!",
    ];

    PHRASES[seed % PHRASES.len()]
}

/// Get a wee bit o' encouragement after an error
#[allow(dead_code)]
pub fn scots_encouragement() -> &'static str {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as usize;

    const PHRASES: &[&str] = &[
        "Dinnae fash yersel - hae anither go!",
        "Keep the heid an' try again!",
        "Ye'll get it next time, nae bother!",
        "Haud on - ye're nearly there!",
        "Gie it anither bash!",
        "Chin up, it's nae the end o' the warld!",
    ];

    PHRASES[(seed / 2) % PHRASES.len()]
}

pub type HaversResult<T> = Result<T, HaversError>;

/// Get a helpful suggestion fer common errors
pub fn get_error_suggestion(error: &HaversError) -> Option<&'static str> {
    match error {
        HaversError::UndefinedVariable { name, .. } => {
            // Check for common misspellings of keywords
            let name_lower = name.to_lowercase();
            match name_lower.as_str() {
                "true" | "false" => Some("💡 Did ye mean 'aye' or 'nae'? In mdhavers we use Scots words fer booleans!"),
                "if" | "else" => Some("💡 Did ye mean 'gin' (if) or 'ither' (else)? We speak Scots here!"),
                "while" => Some("💡 Did ye mean 'whiles'? That's how we say 'while' in Scots!"),
                "for" => Some("💡 Did ye mean 'fer'? That's the Scots way tae loop!"),
                "let" | "var" | "const" => Some("💡 Did ye mean 'ken'? Use 'ken x = 42' tae declare variables!"),
                "print" | "println" | "console" | "echo" => Some("💡 Did ye mean 'blether'? That's how we print in mdhavers!"),
                "return" => Some("💡 Did ye mean 'gie'? Use 'gie value' tae return fae a function!"),
                "function" | "func" | "fn" | "def" => Some("💡 Did ye mean 'dae'? Use 'dae name() { }' tae define functions!"),
                "null" | "nil" | "none" | "undefined" => Some("💡 Did ye mean 'naething'? That's oor word fer null!"),
                "class" => Some("💡 Did ye mean 'kin'? Use 'kin ClassName { }' tae define classes!"),
                "self" | "this" => Some("💡 Did ye mean 'masel'? Use 'masel.property' inside classes!"),
                "try" => Some("💡 Did ye mean 'hae_a_bash'? That's how we try things in Scots!"),
                "catch" | "except" => Some("💡 Did ye mean 'gin_it_gangs_wrang'? That's oor catch block!"),
                "import" | "require" | "include" => Some("💡 Did ye mean 'fetch'? Use 'fetch \"module\"' tae import!"),
                "break" => Some("💡 Did ye mean 'brak'? That's how we break oot o' loops!"),
                "continue" => Some("💡 Did ye mean 'haud'? That's how we continue tae the next iteration!"),
                "switch" | "case" => Some("💡 Did ye mean 'keek' and 'whan'? Use 'keek value { whan 1 -> ... }'!"),
                "assert" => Some("💡 Did ye mean 'mak_siccar'? Like Robert the Bruce said!"),
                "and" | "&&" => Some("💡 Did ye mean 'an'? Use 'x an y' fer logical AND!"),
                "not" | "!" => Some("💡 Did ye mean 'nae'? Use 'nae x' fer logical NOT!"),
                "map" => Some("💡 Did ye mean 'gaun'? Use 'gaun(list, |x| x * 2)' tae map!"),
                "filter" => Some("💡 Did ye mean 'sieve'? Use 'sieve(list, |x| x > 0)' tae filter!"),
                "reduce" | "fold" => Some("💡 Did ye mean 'tumble'? Use 'tumble(list, init, |acc, x| acc + x)'!"),
                "length" | "size" | "count" => Some("💡 Did ye mean 'len'? Use 'len(list)' tae get the length!"),
                "type" | "typeof" => Some("💡 Did ye mean 'whit_kind'? Use 'whit_kind(x)' tae get the type!"),
                "str" | "string" | "tostring" => Some("💡 Did ye mean 'tae_string'? Use 'tae_string(x)' tae convert!"),
                "int" | "integer" | "toint" => Some("💡 Did ye mean 'tae_int'? Use 'tae_int(x)' tae convert!"),
                "push" | "append" | "add" => Some("💡 Did ye mean 'shove'? Use 'shove(list, item)' tae add tae a list!"),
                "pop" | "remove" => Some("💡 Did ye mean 'yank'? Use 'yank(list)' tae remove fae a list!"),
                "input" | "read" | "readline" => Some("💡 Did ye mean 'speir'? Use 'speir(\"prompt\")' tae get input!"),
                "struct" => Some("💡 Did ye mean 'thing'? Use 'thing Name { fields }' fer structs!"),
                _ => None,
            }
        }
        HaversError::UnexpectedToken { found, expected, .. } => {
            if found == "}" && expected.contains("expression") {
                Some("💡 Ye might be missin' an expression before the closing brace!")
            } else if found == "=" && expected.contains("expression") {
                Some("💡 Did ye mean '==' fer comparison? Single '=' is fer assignment!")
            } else if found == ")" {
                Some("💡 Check yer brackets - ye might hae an extra ')' or be missin' something!")
            } else {
                None
            }
        }
        HaversError::TypeError { message, .. } => {
            if message.contains("add") && message.contains("string") {
                Some("💡 Use 'tae_string(x)' tae convert numbers tae strings before concatenatin'!")
            } else if message.contains("integer") && message.contains("index") {
                Some("💡 List indices must be integers. Use 'tae_int(x)' if needed!")
            } else {
                None
            }
        }
        HaversError::WrongArity { expected, got, .. } => {
            if *expected == 0 && *got > 0 {
                Some("💡 This function takes nae arguments - remove the bits in the brackets!")
            } else if *got == 0 && *expected > 0 {
                Some("💡 This function needs arguments - check the function definition!")
            } else {
                None
            }
        }
        HaversError::IndexOutOfBounds { index, size, .. } => {
            if *index < 0 {
                Some("💡 Negative indices count fae the end. -1 is the last element!")
            } else if *size == 0 {
                Some("💡 The list is empty! Check ye've added items before accessin' them.")
            } else {
                Some("💡 Remember, indices start at 0! The last valid index is len - 1.")
            }
        }
        HaversError::DivisionByZero { .. } => {
            Some("💡 Check yer divisor - ye cannae divide by zero! Maybe add a 'gin x != 0' check?")
        }
        HaversError::StackOverflow { .. } => {
            Some("💡 Yer recursion needs a base case! Make sure ye're returnin' somewhere.")
        }
        HaversError::NotCallable { .. } => {
            Some("💡 Ye can only call functions wi' brackets. Check the variable type wi' 'whit_kind(x)'!")
        }
        HaversError::KeyNotFound { .. } => {
            Some("💡 Use 'keys(dict)' tae see whit keys exist, or check wi' 'has_key(dict, key)'!")
        }
        HaversError::UnterminatedString { .. } => {
            Some("💡 Ye forgot tae close yer string! Add a \" at the end.")
        }
        HaversError::CircularImport { .. } => {
            Some("💡 Module A imports B which imports A - that's a loop! Reorganise yer imports.")
        }
        HaversError::BreakOutsideLoop { .. } => {
            Some("💡 'brak' only works inside 'whiles' or 'fer' loops!")
        }
        HaversError::ContinueOutsideLoop { .. } => {
            Some("💡 'haud' only works inside 'whiles' or 'fer' loops!")
        }
        HaversError::ReturnOutsideFunction { .. } => {
            Some("💡 'gie' only works inside functions! Define a function wi' 'dae name() { }'")
        }
        _ => None,
    }
}

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
