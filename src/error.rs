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

    #[error("Och! Compilation went tits up: {0}")]
    CompileError(String),

    #[error("Wheesht! Break statement ootside a loop at line {line} - ye can only brak fae inside a whiles or fer loop!")]
    BreakOutsideLoop { line: usize },

    #[error("Haud on there! Continue statement ootside a loop at line {line} - ye can only haud inside a whiles or fer loop!")]
    ContinueOutsideLoop { line: usize },

    #[error(
        "Stack's fair puggled! Too many nested calls at line {line} - yer recursion's gone radge!"
    )]
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

    #[error("Wheesht! Yer number's too muckle at line {line} - it's overflowed!")]
    IntegerOverflow { line: usize },

    #[error("By the bonnie banks! Negative index {index} is oot o' range at line {line}")]
    NegativeIndexOutOfBounds { index: i64, line: usize },

    #[error("Haud yer wheesht! Empty list at line {line} - ye cannae {operation} on naething!")]
    EmptyCollection { operation: String, line: usize },

    #[error("Yer regex is mince at line {line}: {message}")]
    InvalidRegex { message: String, line: usize },

    #[error("That format string's a guddle at line {line}: {message}")]
    FormatError { message: String, line: usize },

    #[error("The JSON's aw wrang at line {line}: {message}")]
    JsonError { message: String, line: usize },

    #[error("Ye cannae compare {left_type} wi' {right_type} at line {line} - they're like chalk an' cheese!")]
    IncomparableTypes {
        left_type: String,
        right_type: String,
        line: usize,
    },

    #[error("That number's nae use at line {line}: {message}")]
    InvalidNumberOperation { message: String, line: usize },

    #[error("Yer match hasnae covered aw the cases at line {line}!")]
    NonExhaustiveMatch { line: usize },

    #[error("Ye've got duplicate keys in yer dict at line {line}: '{key}'")]
    DuplicateKey { key: String, line: usize },

    #[error("Timeout! Yer code took too lang at line {line} - maybe an infinite loop?")]
    ExecutionTimeout { line: usize },

    #[error("Memory's fair scunnered! Ran oot o' space at line {line}")]
    OutOfMemory { line: usize },

    #[error(
        "That's a private member! Ye cannae access '{member}' fae ootside the class at line {line}"
    )]
    PrivateMemberAccess { member: String, line: usize },

    #[error("Immutable! Ye cannae change '{name}' at line {line} - it's set in stone!")]
    ImmutableVariable { name: String, line: usize },
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
            HaversError::IntegerOverflow { line } => Some(*line),
            HaversError::NegativeIndexOutOfBounds { line, .. } => Some(*line),
            HaversError::EmptyCollection { line, .. } => Some(*line),
            HaversError::InvalidRegex { line, .. } => Some(*line),
            HaversError::FormatError { line, .. } => Some(*line),
            HaversError::JsonError { line, .. } => Some(*line),
            HaversError::IncomparableTypes { line, .. } => Some(*line),
            HaversError::InvalidNumberOperation { line, .. } => Some(*line),
            HaversError::NonExhaustiveMatch { line } => Some(*line),
            HaversError::DuplicateKey { line, .. } => Some(*line),
            HaversError::ExecutionTimeout { line } => Some(*line),
            HaversError::OutOfMemory { line } => Some(*line),
            HaversError::PrivateMemberAccess { line, .. } => Some(*line),
            HaversError::ImmutableVariable { line, .. } => Some(*line),
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
        "Haud yer horses!",
        "Whit's aw this then?",
        "Och, here we go again!",
        "By the bonnie banks!",
        "Fit like? No' guid!",
        "Awa' an' dinnae come back!",
        "Yer aff yer heid!",
        "Pure radge!",
        "Whit a palaver!",
        "Gonnae no' dae that!",
        "Gie's peace!",
        "Yer havin' a laugh!",
        "Stone the crows!",
        "Sakes o' mercy!",
        "Whit a fankle!",
        "Heavens tae Betsy!",
        "Lang may yer lum reek... but no' the day!",
        "By ma grannie's tartan knickers!",
        "Haud me back!",
        "Whit in tarnation!",
        "Yer code's gone doolally!",
        "Bletherin' bogles!",
        "Sufferin' sporrans!",
        "Nessie's nostrils!",
        "Haggis tae Highlands!",
        "Tatties an' neeps!",
        "By the Loch Ness Monster!",
        "Burns wid be birlin' in his grave!",
        "Haud yer weesht an' fix it!",
        "Whit's aw the stooshie?",
        "Yer code's fair glaikit!",
        "By the ghost o' Robert Bruce!",
        "Awa' wi' the fairies!",
        "That's a real howler!",
        "Och, fit a cairry on!",
        "Nae messin' aboot!",
        "Yer code's in a right fankle!",
        "Guid heavens above!",
        "Whit's the malky?",
        "By the beard o' Rob Roy!",
        "Cannae believe ma een!",
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
        "Rome wisnae built in a day, an' neither is guid code!",
        "Every maister wis once a disaster!",
        "Yer code's getting' better - keep goin'!",
        "Bugs are just features ye didnae plan fer!",
        "Even the best programmers get scunnered sometimes!",
        "A wee setback isnae a defeat!",
        "Whit doesnae kill yer code makes it stronger!",
        "The best debugging is done efter a cup o' tea!",
        "Ye're daein' braw - dinnae gie up!",
        "Mony a mickle maks a muckle - keep at it!",
        "Practice maks perfect, an' bugs mak ye smarter!",
    ];

    PHRASES[(seed / 2) % PHRASES.len()]
}

/// Get a Scottish programming proverb
pub fn scots_programming_wisdom() -> &'static str {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as usize;

    const PROVERBS: &[&str] = &[
        "A guid programmer kens when tae ask fer help.",
        "Mony a guid tune is played on an auld fiddle - an' auld code can still work!",
        "Better a wee error noo than a muckle disaster later.",
        "Test early, test often, or ye'll be greetin'.",
        "Comments are like shortbread - ye can never hae too many.",
        "Readable code is worth its weight in haggis.",
        "Dinnae optimize afore yer time.",
        "A bug in the hand is worth two in production.",
        "When in doubt, blether it oot (print debugging)!",
        "Keep yer functions wee, like a dram o' whisky.",
        "Git commit early an' often - save yer work!",
        "The best code is the code ye dinnae hae tae write.",
        "Variable names should tell a story, no' a riddle.",
        "If it works, dinnae touch it. If it doesnae, fix it!",
        "A rubber duck debugging session is worth a thousand breakpoints.",
        "Fools look tae tomorrow - wise coders push tae main today.",
        "He that winna be ruled by the compiler must be ruled by the debugger.",
        "A stitch in time saves nine - an' a unit test saves ninety!",
        "What's fer ye'll no go past ye - but ye still hae tae write the code.",
        "Be happy while ye're livin', fer ye're a lang time debuggin'.",
        "The proof o' the puddin' is in the eatin' - the proof o' the code is in the testin'.",
        "Ye cannae make a silk purse oot o' a soo's lug - or guid code fae bad requirements.",
        "A nod's as guid as a wink tae a blind horse - but explicit code is better than implicit!",
        "Mony mickles mak a muckle - an' mony wee functions mak guid code.",
        "Gie a man a fish an' ye feed him fer a day; teach a man tae code an' he'll hae bugs forever.",
    ];

    PROVERBS[(seed / 3) % PROVERBS.len()]
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
                "trim" => Some("💡 Did ye mean 'wheesht'? Use 'wheesht(str)' tae trim whitespace!"),
                "slice" | "substring" | "substr" => Some("💡 Did ye mean 'scran'? Use 'scran(str, start, end)' tae slice!"),
                "find" | "indexof" => Some("💡 Did ye mean 'index_of'? Use 'index_of(str, substr)' tae find!"),
                "random" | "rand" => Some("💡 Did ye mean 'jammy'? Use 'jammy(min, max)' fer random numbers!"),
                "sleep" | "wait" | "delay" => Some("💡 Did ye mean 'snooze'? Use 'snooze(ms)' tae pause!"),
                "now" | "time" | "timestamp" => Some("💡 Did ye mean 'noo' or 'the_noo'? That's how we get the time!"),
                "exit" | "quit" => Some("💡 Did ye mean 'awa'? Use 'awa(code)' tae exit the program!"),
                "throw" | "raise" => Some("💡 Did ye mean 'fling'? Use 'fling \"error message\"' tae throw errors!"),
                "lambda" | "arrow" => Some("💡 Use '|x| x * 2' fer lambdas - nae need fer a keyword!"),
                "extends" | "inherit" => Some("💡 Use 'kin Child frae Parent { }' fer inheritance!"),
                "in" => Some("💡 Use 'contains(list, item)' tae check if an item is in a list!"),
                "or" | "||" => Some("💡 Use 'or' fer logical OR: 'x or y'"),
                "array" | "list" | "vec" => Some("💡 Use square brackets: '[1, 2, 3]' tae create a list!"),
                "dict" | "hash" | "hashmap" | "object" => Some("💡 Use curly braces: '{\"key\": value}' tae create a dict!"),
                "first" | "head" => Some("💡 Did ye mean 'heid'? Use 'heid(list)' tae get the first element!"),
                "last" => Some("💡 Did ye mean 'bum'? Use 'bum(list)' tae get the last element!"),
                "rest" | "tail" => Some("💡 Did ye mean 'tail'? Use 'tail(list)' tae get all but the first!"),
                "sort" => Some("💡 Use 'sort(list)' tae sort a list!"),
                "reverse" => Some("💡 Use 'reverse(list)' tae reverse a list!"),
                "join" => Some("💡 Use 'join(list, sep)' tae join a list intae a string!"),
                "split" => Some("💡 Use 'split(str, sep)' tae split a string intae a list!"),
                "format" => Some("💡 Use f-strings: f\"Hello {name}!\" fer string formatting!"),
                "debug" | "inspect" => Some("💡 Use 'clype(x)' tae print debug info aboot a value!"),
                "range" => Some("💡 Use 'start..end' fer ranges! E.g., 'fer i in 0..10 { }'"),
                "foreach" => Some("💡 Use 'fer item in list { }' tae iterate over items!"),
                "async" | "await" => Some("💡 mdhavers doesnae support async yet - stick tae synchronous code!"),
                "match" => Some("💡 Did ye mean 'keek'? Use 'keek value { whan x -> ... }'!"),
                "enum" => Some("💡 Use dictionaries or constants fer enum-like patterns!"),
                "interface" | "trait" | "protocol" => Some("💡 Use classes (kin) - mdhavers doesnae have interfaces!"),
                "static" => Some("💡 All functions at module level are like static - nae need fer a keyword!"),
                "public" | "private" | "protected" => Some("💡 mdhavers doesnae have access modifiers - everything's public!"),
                "new" => Some("💡 Just call the class like a function: 'MyClass()' - nae 'new' needed!"),
                "super" => Some("💡 Did ye mean 'auld'? Use 'auld.method()' tae call parent class methods!"),
                "final" | "readonly" => Some("💡 Use 'ken' fer all variables - they're mutable by default!"),
                "void" => Some("💡 Functions wi' nae return value automatically return 'naething'!"),
                "boolean" | "bool" => Some("💡 Booleans are 'aye' (true) an' 'nae' (false) in mdhavers!"),
                "String" => Some("💡 Strings are created wi' quotes: \"hello\" or 'hello'!"),
                "char" | "character" => Some("💡 Use 'char_at(str, index)' tae get a character fae a string!"),
                "float" | "double" | "decimal" => Some("💡 Just use numbers wi' decimal points: '3.14'!"),
                "byte" | "bytes" => Some("💡 Strings handle text - fer binary, use lists o' integers!"),
                "set" => Some("💡 Use the Set class fae the structures module, or a dict wi' dummy values!"),
                "tuple" => Some("💡 Use lists fer tuples: '[1, \"hello\", aye]'!"),
                "global" => Some("💡 Variables at module level are global - nae keyword needed!"),
                "do" => Some("💡 Did ye mean 'dae'? Use 'dae name() { }' fer functions!"),
                "end" | "endif" | "endfor" | "endwhile" => Some("💡 Use curly braces { } tae end blocks - nae 'end' keyword!"),
                "then" => Some("💡 Nae 'then' keyword - use 'gin condition { ... }'!"),
                "begin" => Some("💡 Use { tae start a block - nae 'begin' keyword!"),
                "puts" | "write" | "output" => Some("💡 Did ye mean 'blether'? Use 'blether \"text\"' tae print!"),
                "gets" => Some("💡 Did ye mean 'speir'? Use 'speir(\"prompt\")' tae get input!"),
                "sprintf" | "printf" => Some("💡 Use f-strings: f\"Value is {x}\" fer formatting!"),
                "len" => Some("💡 'len' is built-in! Use 'len(list)' or 'len(string)'!"),
                "abs" | "absolute" => Some("💡 Use 'abs(x)' fer absolute value - it's built-in!"),
                "max" | "maximum" => Some("💡 Use 'max(a, b)' or 'max(list)' - it's built-in!"),
                "min" | "minimum" => Some("💡 Use 'min(a, b)' or 'min(list)' - it's built-in!"),
                "floor" | "ceil" | "round" => Some("💡 Use 'floor(x)', 'ceil(x)', or 'round(x)' - they're built-in!"),
                "sqrt" | "squareroot" => Some("💡 Use the maths module: 'fetch \"lib/maths\"' fer sqrt!"),
                "sin" | "cos" | "tan" => Some("💡 Use the maths module: 'fetch \"lib/maths\"' fer trig functions!"),
                "log" | "exp" => Some("💡 Use the maths module: 'fetch \"lib/maths\"' fer logarithms!"),
                "open" | "fopen" => Some("💡 Use 'read_file(path)' or 'write_file(path, content)' fer files!"),
                "close" | "fclose" => Some("💡 File handles close automatically - nae need tae close manually!"),
                "module" | "package" | "namespace" => Some("💡 Modules are just .braw files! Use 'fetch \"lib/name\"' tae import!"),
                "from" => Some("💡 Did ye mean 'frae'? Use 'kin Child frae Parent { }' fer inheritance!"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_suggestions() {
        // Test common keyword misspellings
        let err = HaversError::UndefinedVariable {
            name: "true".to_string(),
            line: 1,
        };
        let suggestion = get_error_suggestion(&err);
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("aye"));

        let err = HaversError::UndefinedVariable {
            name: "print".to_string(),
            line: 1,
        };
        let suggestion = get_error_suggestion(&err);
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("blether"));

        let err = HaversError::UndefinedVariable {
            name: "null".to_string(),
            line: 1,
        };
        let suggestion = get_error_suggestion(&err);
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("naething"));

        let err = HaversError::UndefinedVariable {
            name: "function".to_string(),
            line: 1,
        };
        let suggestion = get_error_suggestion(&err);
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("dae"));
    }

    #[test]
    fn test_error_suggestions_more_keywords() {
        // Test more keyword misspellings
        let keywords = vec![
            ("false", "nae"),
            ("if", "gin"),
            ("else", "ither"),
            ("while", "whiles"),
            ("for", "fer"),
            ("let", "ken"),
            ("return", "gie"),
            ("class", "kin"),
            ("self", "masel"),
            ("try", "hae_a_bash"),
            ("catch", "gin_it_gangs_wrang"),
            ("import", "fetch"),
            ("break", "brak"),
            ("continue", "haud"),
            ("switch", "keek"),
            ("assert", "mak_siccar"),
            ("and", "an"),
            ("map", "gaun"),
            ("filter", "sieve"),
            ("reduce", "tumble"),
            ("length", "len"),
            ("type", "whit_kind"),
            ("str", "tae_string"),
            ("int", "tae_int"),
            ("push", "shove"),
            ("pop", "yank"),
            ("input", "speir"),
            ("struct", "thing"),
            ("trim", "wheesht"),
            ("random", "jammy"),
            ("sleep", "snooze"),
            ("now", "noo"),
            ("exit", "awa"),
            ("throw", "fling"),
            ("extends", "frae"),
            ("super", "auld"),
            ("first", "heid"),
            ("last", "bum"),
            ("debug", "clype"),
        ];

        for (keyword, expected) in keywords {
            let err = HaversError::UndefinedVariable {
                name: keyword.to_string(),
                line: 1,
            };
            let suggestion = get_error_suggestion(&err);
            assert!(suggestion.is_some(), "Expected suggestion for {}", keyword);
            assert!(
                suggestion.unwrap().to_lowercase().contains(expected),
                "Expected '{}' in suggestion for '{}'",
                expected,
                keyword
            );
        }
    }

    #[test]
    fn test_error_suggestions_other_errors() {
        // Test division by zero suggestion
        let err = HaversError::DivisionByZero { line: 1 };
        let suggestion = get_error_suggestion(&err);
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("zero"));

        // Test stack overflow suggestion
        let err = HaversError::StackOverflow { line: 1 };
        let suggestion = get_error_suggestion(&err);
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("recursion"));

        // Test index out of bounds for empty list
        let err = HaversError::IndexOutOfBounds {
            index: 0,
            size: 0,
            line: 1,
        };
        let suggestion = get_error_suggestion(&err);
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("empty"));

        // Test negative index
        let err = HaversError::IndexOutOfBounds {
            index: -1,
            size: 5,
            line: 1,
        };
        let suggestion = get_error_suggestion(&err);
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("Negative"));

        // Test positive index out of bounds
        let err = HaversError::IndexOutOfBounds {
            index: 10,
            size: 5,
            line: 1,
        };
        let suggestion = get_error_suggestion(&err);
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("0"));

        // Test not callable
        let err = HaversError::NotCallable {
            name: "x".to_string(),
            line: 1,
        };
        let suggestion = get_error_suggestion(&err);
        assert!(suggestion.is_some());

        // Test key not found
        let err = HaversError::KeyNotFound {
            key: "foo".to_string(),
            line: 1,
        };
        let suggestion = get_error_suggestion(&err);
        assert!(suggestion.is_some());

        // Test unterminated string
        let err = HaversError::UnterminatedString { line: 1 };
        let suggestion = get_error_suggestion(&err);
        assert!(suggestion.is_some());

        // Test circular import
        let err = HaversError::CircularImport {
            path: "lib".to_string(),
        };
        let suggestion = get_error_suggestion(&err);
        assert!(suggestion.is_some());

        // Test break outside loop
        let err = HaversError::BreakOutsideLoop { line: 1 };
        let suggestion = get_error_suggestion(&err);
        assert!(suggestion.is_some());

        // Test continue outside loop
        let err = HaversError::ContinueOutsideLoop { line: 1 };
        let suggestion = get_error_suggestion(&err);
        assert!(suggestion.is_some());

        // Test return outside function
        let err = HaversError::ReturnOutsideFunction { line: 1 };
        let suggestion = get_error_suggestion(&err);
        assert!(suggestion.is_some());

        // Test wrong arity - no args expected
        let err = HaversError::WrongArity {
            name: "foo".to_string(),
            expected: 0,
            got: 2,
            line: 1,
        };
        let suggestion = get_error_suggestion(&err);
        assert!(suggestion.is_some());

        // Test wrong arity - args expected
        let err = HaversError::WrongArity {
            name: "foo".to_string(),
            expected: 2,
            got: 0,
            line: 1,
        };
        let suggestion = get_error_suggestion(&err);
        assert!(suggestion.is_some());

        // Test type error with string add
        let err = HaversError::TypeError {
            message: "Cannot add string".to_string(),
            line: 1,
        };
        let suggestion = get_error_suggestion(&err);
        assert!(suggestion.is_some());

        // Test unexpected token - closing brace
        let err = HaversError::UnexpectedToken {
            expected: "expression".to_string(),
            found: "}".to_string(),
            line: 1,
        };
        let suggestion = get_error_suggestion(&err);
        assert!(suggestion.is_some());

        // Test unexpected token - equals
        let err = HaversError::UnexpectedToken {
            expected: "expression".to_string(),
            found: "=".to_string(),
            line: 1,
        };
        let suggestion = get_error_suggestion(&err);
        assert!(suggestion.is_some());

        // Test unexpected token - paren
        let err = HaversError::UnexpectedToken {
            expected: "something".to_string(),
            found: ")".to_string(),
            line: 1,
        };
        let suggestion = get_error_suggestion(&err);
        assert!(suggestion.is_some());
    }

    #[test]
    fn test_scots_phrases() {
        // Test that random phrases return valid strings
        let phrase = random_scots_exclamation();
        assert!(!phrase.is_empty());

        let encouragement = scots_encouragement();
        assert!(!encouragement.is_empty());

        let wisdom = scots_programming_wisdom();
        assert!(!wisdom.is_empty());
    }

    #[test]
    fn test_format_error_context() {
        let source = "ken x = 1\nken y = 2\nken z = 3";
        let context = format_error_context(source, 2);
        assert!(context.contains("ken y = 2"));
        assert!(context.contains("> 2 |"));
    }

    #[test]
    fn test_format_error_context_edge_cases() {
        // Test first line
        let source = "ken x = 1\nken y = 2";
        let context = format_error_context(source, 1);
        assert!(context.contains("ken x = 1"));
        assert!(context.contains("> 1 |"));

        // Test last line
        let context = format_error_context(source, 2);
        assert!(context.contains("ken y = 2"));

        // Test invalid line 0
        let context = format_error_context(source, 0);
        assert!(context.is_empty());

        // Test line beyond source
        let context = format_error_context(source, 10);
        assert!(context.is_empty());
    }

    #[test]
    fn test_error_line_method() {
        // Test all error variants that have line
        assert_eq!(
            HaversError::UnkentToken {
                lexeme: "x".to_string(),
                line: 5,
                column: 3
            }
            .line(),
            Some(5)
        );

        assert_eq!(
            HaversError::UnexpectedToken {
                expected: "a".to_string(),
                found: "b".to_string(),
                line: 10
            }
            .line(),
            Some(10)
        );

        assert_eq!(
            HaversError::UndefinedVariable {
                name: "x".to_string(),
                line: 3
            }
            .line(),
            Some(3)
        );

        assert_eq!(HaversError::DivisionByZero { line: 7 }.line(), Some(7));

        assert_eq!(
            HaversError::TypeError {
                message: "msg".to_string(),
                line: 2
            }
            .line(),
            Some(2)
        );

        assert_eq!(
            HaversError::NotCallable {
                name: "x".to_string(),
                line: 4
            }
            .line(),
            Some(4)
        );

        assert_eq!(
            HaversError::WrongArity {
                name: "f".to_string(),
                expected: 1,
                got: 2,
                line: 6
            }
            .line(),
            Some(6)
        );

        assert_eq!(
            HaversError::IndexOutOfBounds {
                index: 5,
                size: 3,
                line: 8
            }
            .line(),
            Some(8)
        );

        assert_eq!(
            HaversError::ParseError {
                message: "err".to_string(),
                line: 9
            }
            .line(),
            Some(9)
        );

        assert_eq!(HaversError::BreakOutsideLoop { line: 11 }.line(), Some(11));
        assert_eq!(
            HaversError::ContinueOutsideLoop { line: 12 }.line(),
            Some(12)
        );
        assert_eq!(HaversError::StackOverflow { line: 13 }.line(), Some(13));
        assert_eq!(
            HaversError::UnterminatedString { line: 14 }.line(),
            Some(14)
        );
        assert_eq!(
            HaversError::InvalidNumber {
                value: "x".to_string(),
                line: 15
            }
            .line(),
            Some(15)
        );
        assert_eq!(
            HaversError::AlreadyDefined {
                name: "x".to_string(),
                line: 16
            }
            .line(),
            Some(16)
        );
        assert_eq!(
            HaversError::NotAnObject {
                name: "x".to_string(),
                line: 17
            }
            .line(),
            Some(17)
        );
        assert_eq!(
            HaversError::UndefinedProperty {
                property: "x".to_string(),
                line: 18
            }
            .line(),
            Some(18)
        );
        assert_eq!(HaversError::InfiniteLoop { line: 19 }.line(), Some(19));
        assert_eq!(HaversError::NotAList { line: 20 }.line(), Some(20));
        assert_eq!(HaversError::NotADict { line: 21 }.line(), Some(21));
        assert_eq!(
            HaversError::KeyNotFound {
                key: "x".to_string(),
                line: 22
            }
            .line(),
            Some(22)
        );
        assert_eq!(
            HaversError::InvalidOperation {
                operation: "op".to_string(),
                line: 23
            }
            .line(),
            Some(23)
        );
        assert_eq!(
            HaversError::AssertionFailed {
                message: "msg".to_string(),
                line: 24
            }
            .line(),
            Some(24)
        );
        assert_eq!(
            HaversError::ReturnOutsideFunction { line: 25 }.line(),
            Some(25)
        );
        assert_eq!(
            HaversError::NotIterable {
                type_name: "int".to_string(),
                line: 26
            }
            .line(),
            Some(26)
        );
        assert_eq!(
            HaversError::PatternError {
                message: "msg".to_string(),
                line: 27
            }
            .line(),
            Some(27)
        );
        assert_eq!(HaversError::IntegerOverflow { line: 28 }.line(), Some(28));
        assert_eq!(
            HaversError::NegativeIndexOutOfBounds {
                index: -1,
                line: 29
            }
            .line(),
            Some(29)
        );
        assert_eq!(
            HaversError::EmptyCollection {
                operation: "op".to_string(),
                line: 30
            }
            .line(),
            Some(30)
        );
        assert_eq!(
            HaversError::InvalidRegex {
                message: "msg".to_string(),
                line: 31
            }
            .line(),
            Some(31)
        );
        assert_eq!(
            HaversError::FormatError {
                message: "msg".to_string(),
                line: 32
            }
            .line(),
            Some(32)
        );
        assert_eq!(
            HaversError::JsonError {
                message: "msg".to_string(),
                line: 33
            }
            .line(),
            Some(33)
        );
        assert_eq!(
            HaversError::IncomparableTypes {
                left_type: "a".to_string(),
                right_type: "b".to_string(),
                line: 34
            }
            .line(),
            Some(34)
        );
        assert_eq!(
            HaversError::InvalidNumberOperation {
                message: "msg".to_string(),
                line: 35
            }
            .line(),
            Some(35)
        );
        assert_eq!(
            HaversError::NonExhaustiveMatch { line: 36 }.line(),
            Some(36)
        );
        assert_eq!(
            HaversError::DuplicateKey {
                key: "x".to_string(),
                line: 37
            }
            .line(),
            Some(37)
        );
        assert_eq!(HaversError::ExecutionTimeout { line: 38 }.line(), Some(38));
        assert_eq!(HaversError::OutOfMemory { line: 39 }.line(), Some(39));
        assert_eq!(
            HaversError::PrivateMemberAccess {
                member: "x".to_string(),
                line: 40
            }
            .line(),
            Some(40)
        );
        assert_eq!(
            HaversError::ImmutableVariable {
                name: "x".to_string(),
                line: 41
            }
            .line(),
            Some(41)
        );

        // Errors without line
        assert_eq!(
            HaversError::FileError {
                path: "x".to_string(),
                reason: "r".to_string()
            }
            .line(),
            None
        );
        assert_eq!(HaversError::InternalError("msg".to_string()).line(), None);
        assert_eq!(
            HaversError::ModuleNotFound {
                name: "x".to_string()
            }
            .line(),
            None
        );
        assert_eq!(
            HaversError::CircularImport {
                path: "x".to_string()
            }
            .line(),
            None
        );
    }

    #[test]
    fn test_error_display() {
        // Test that error messages format correctly
        let err = HaversError::UndefinedVariable {
            name: "x".to_string(),
            line: 5,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("x"));
        assert!(msg.contains("5"));

        let err = HaversError::DivisionByZero { line: 3 };
        let msg = format!("{}", err);
        assert!(msg.contains("3"));
        assert!(msg.contains("zero"));

        let err = HaversError::WrongArity {
            name: "foo".to_string(),
            expected: 2,
            got: 3,
            line: 7,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("foo"));
        assert!(msg.contains("2"));
        assert!(msg.contains("3"));
    }

    #[test]
    fn test_no_suggestion_for_unknown() {
        let err = HaversError::UndefinedVariable {
            name: "my_custom_variable".to_string(),
            line: 1,
        };
        let suggestion = get_error_suggestion(&err);
        assert!(suggestion.is_none());
    }
}
