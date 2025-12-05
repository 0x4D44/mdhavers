use logos::Logos;
use std::fmt;

/// Aw the different kinds o' tokens in mdhavers
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r]+")]  // Skip whitespace but nae newlines
pub enum TokenKind {
    // === Scots Keywords ===

    /// ken - variable declaration (I know/understand)
    #[token("ken")]
    Ken,

    /// gin - if statement (if/when)
    #[token("gin")]
    Gin,

    /// ither - else
    #[token("ither")]
    Ither,

    /// than - then (for ternary expressions: gin x than y ither z)
    #[token("than")]
    Than,

    /// whiles - while loop
    #[token("whiles")]
    Whiles,

    /// fer - for loop
    #[token("fer")]
    Fer,

    /// gie - return (give back)
    #[token("gie")]
    Gie,

    /// blether - print (chat/talk)
    #[token("blether")]
    Blether,

    /// speir - input (ask)
    #[token("speir")]
    Speir,

    /// fae - from
    #[token("fae")]
    Fae,

    /// tae - to
    #[token("tae")]
    Tae,

    /// an - and (logical)
    #[token("an")]
    An,

    /// or - or (logical)
    #[token("or")]
    Or,

    /// nae - not / false
    #[token("nae")]
    Nae,

    /// aye - true
    #[token("aye")]
    Aye,

    /// naething - null/none/nil
    #[token("naething")]
    Naething,

    /// dae - function definition (do)
    #[token("dae")]
    Dae,

    /// thing - struct definition
    #[token("thing")]
    Thing,

    /// fetch - import
    #[token("fetch")]
    Fetch,

    /// kin - class (family/type)
    #[token("kin")]
    Kin,

    /// brak - break
    #[token("brak")]
    Brak,

    /// haud - continue (hold on)
    #[token("haud")]
    Haud,

    /// in - in (for loops)
    #[token("in")]
    In,

    /// is - is (type checking/comparison)
    #[token("is")]
    Is,

    /// self/this reference
    #[token("masel")]
    Masel,

    /// try block
    #[token("hae_a_bash")]
    HaeABash,

    /// catch block
    #[token("gin_it_gangs_wrang")]
    GinItGangsWrang,

    /// match/switch statement
    #[token("keek")]
    Keek,

    /// case in match
    #[token("whan")]
    Whan,

    /// assert - mak_siccar (make sure - famously said by Robert the Bruce!)
    #[token("mak_siccar")]
    MakSiccar,

    // === Literals ===

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Integer(i64),

    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),

    // String with double quotes
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())
    })]
    String(String),

    // String with single quotes (fer use inside f-strings and general convenience)
    #[regex(r#"'([^'\\]|\\.)*'"#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())
    })]
    SingleQuoteString(String),

    // F-string (format string) with interpolation: f"Hello {name}!"
    #[regex(r#"f"([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        Some(s[2..s.len()-1].to_string())  // Skip 'f"' and '"'
    })]
    FString(String),

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // === Operators ===

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("%")]
    Percent,

    #[token("=")]
    Equals,

    #[token("==")]
    EqualsEquals,

    #[token("!=")]
    BangEquals,

    #[token("<")]
    Less,

    #[token("<=")]
    LessEquals,

    #[token(">")]
    Greater,

    #[token(">=")]
    GreaterEquals,

    #[token("!")]
    Bang,

    #[token("+=")]
    PlusEquals,

    #[token("-=")]
    MinusEquals,

    #[token("*=")]
    StarEquals,

    #[token("/=")]
    SlashEquals,

    #[token("...")]
    DotDotDot,  // Spread operator (skail = scatter in Scots)

    #[token("..")]
    DotDot,

    #[token(".")]
    Dot,

    #[token("_", priority = 3)]
    Underscore,  // Wildcard/ignore pattern

    // === Delimiters ===

    #[token("(")]
    LeftParen,

    #[token(")")]
    RightParen,

    #[token("{")]
    LeftBrace,

    #[token("}")]
    RightBrace,

    #[token("[")]
    LeftBracket,

    #[token("]")]
    RightBracket,

    #[token(",")]
    Comma,

    #[token(":")]
    Colon,

    #[token(";")]
    Semicolon,

    #[token("->")]
    Arrow,

    #[token("|>")]
    PipeForward,  // Pipe operator fer chaining: x |> f means f(x)

    #[token("|")]
    Pipe,

    // Newlines are significant in mdhavers (like Python)
    #[token("\n")]
    Newline,

    // Comments - skip them
    #[regex(r"#[^\n]*", logos::skip)]
    Comment,

    // End of file
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Ken => write!(f, "ken"),
            TokenKind::Gin => write!(f, "gin"),
            TokenKind::Ither => write!(f, "ither"),
            TokenKind::Than => write!(f, "than"),
            TokenKind::Whiles => write!(f, "whiles"),
            TokenKind::Fer => write!(f, "fer"),
            TokenKind::Gie => write!(f, "gie"),
            TokenKind::Blether => write!(f, "blether"),
            TokenKind::Speir => write!(f, "speir"),
            TokenKind::Fae => write!(f, "fae"),
            TokenKind::Tae => write!(f, "tae"),
            TokenKind::An => write!(f, "an"),
            TokenKind::Or => write!(f, "or"),
            TokenKind::Nae => write!(f, "nae"),
            TokenKind::Aye => write!(f, "aye"),
            TokenKind::Naething => write!(f, "naething"),
            TokenKind::Dae => write!(f, "dae"),
            TokenKind::Thing => write!(f, "thing"),
            TokenKind::Fetch => write!(f, "fetch"),
            TokenKind::Kin => write!(f, "kin"),
            TokenKind::Brak => write!(f, "brak"),
            TokenKind::Haud => write!(f, "haud"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Is => write!(f, "is"),
            TokenKind::Masel => write!(f, "masel"),
            TokenKind::HaeABash => write!(f, "hae_a_bash"),
            TokenKind::GinItGangsWrang => write!(f, "gin_it_gangs_wrang"),
            TokenKind::Keek => write!(f, "keek"),
            TokenKind::Whan => write!(f, "whan"),
            TokenKind::MakSiccar => write!(f, "mak_siccar"),
            TokenKind::Integer(n) => write!(f, "{}", n),
            TokenKind::Float(n) => write!(f, "{}", n),
            TokenKind::String(s) => write!(f, "\"{}\"", s),
            TokenKind::SingleQuoteString(s) => write!(f, "'{}'", s),
            TokenKind::FString(s) => write!(f, "f\"{}\"", s),
            TokenKind::Identifier(s) => write!(f, "{}", s),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Equals => write!(f, "="),
            TokenKind::EqualsEquals => write!(f, "=="),
            TokenKind::BangEquals => write!(f, "!="),
            TokenKind::Less => write!(f, "<"),
            TokenKind::LessEquals => write!(f, "<="),
            TokenKind::Greater => write!(f, ">"),
            TokenKind::GreaterEquals => write!(f, ">="),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::PlusEquals => write!(f, "+="),
            TokenKind::MinusEquals => write!(f, "-="),
            TokenKind::StarEquals => write!(f, "*="),
            TokenKind::SlashEquals => write!(f, "/="),
            TokenKind::DotDotDot => write!(f, "..."),
            TokenKind::DotDot => write!(f, ".."),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Underscore => write!(f, "_"),
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::LeftBrace => write!(f, "{{"),
            TokenKind::RightBrace => write!(f, "}}"),
            TokenKind::LeftBracket => write!(f, "["),
            TokenKind::RightBracket => write!(f, "]"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::PipeForward => write!(f, "|>"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::Newline => write!(f, "newline"),
            TokenKind::Comment => write!(f, "comment"),
            TokenKind::Eof => write!(f, "end of file"),
        }
    }
}

/// A token wi' its position in the source
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, line: usize, column: usize) -> Self {
        Token {
            kind,
            lexeme,
            line,
            column,
        }
    }

    pub fn eof(line: usize) -> Self {
        Token {
            kind: TokenKind::Eof,
            lexeme: String::new(),
            line,
            column: 0,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at line {}", self.kind, self.line)
    }
}
