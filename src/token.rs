use logos::Logos;
use std::fmt;

/// Aw the different kinds o' tokens in mdhavers
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r]+")] // Skip whitespace but nae newlines
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
    #[token("nil")]
    #[token("nowt")]
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
    #[token("haud_yer_wheesht")]
    #[token("gang_on")]
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

    // === Logging Keywords ===
    /// log_whisper - TRACE level (most verbose)
    #[token("log_whisper")]
    LogWhisper,

    /// log_mutter - DEBUG level
    #[token("log_mutter")]
    LogMutter,

    /// log_blether - INFO level
    #[token("log_blether")]
    LogBlether,

    /// log_holler - WARN level
    #[token("log_holler")]
    LogHoller,

    /// log_roar - ERROR level
    #[token("log_roar")]
    LogRoar,

    /// hurl - throw/raise an exception
    #[token("hurl")]
    Hurl,

    // === Literals ===
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Integer(i64),

    #[regex(r"[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?", |lex| lex.slice().parse::<f64>().ok())]
    #[regex(r"[0-9]+[eE][+-]?[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
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
    DotDotDot, // Spread operator (skail = scatter in Scots)

    #[token("..")]
    DotDot,

    #[token(".")]
    Dot,

    #[token("_", priority = 3)]
    Underscore, // Wildcard/ignore pattern

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
    PipeForward, // Pipe operator fer chaining: x |> f means f(x)

    #[token("|")]
    Pipe,

    // Newlines are significant in mdhavers (like Python)
    #[token("\n")]
    Newline,

    // Comments - skip them
    #[regex(r"#[^\n]*", logos::skip)]
    #[regex(r"//[^\n]*", logos::skip)]
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
            TokenKind::LogWhisper => write!(f, "log_whisper"),
            TokenKind::LogMutter => write!(f, "log_mutter"),
            TokenKind::LogBlether => write!(f, "log_blether"),
            TokenKind::LogHoller => write!(f, "log_holler"),
            TokenKind::LogRoar => write!(f, "log_roar"),
            TokenKind::Hurl => write!(f, "hurl"),
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

#[cfg(test)]
#[allow(clippy::approx_constant)]
mod tests {
    use super::*;

    #[test]
    fn test_token_kind_display_keywords() {
        assert_eq!(format!("{}", TokenKind::Ken), "ken");
        assert_eq!(format!("{}", TokenKind::Gin), "gin");
        assert_eq!(format!("{}", TokenKind::Ither), "ither");
        assert_eq!(format!("{}", TokenKind::Than), "than");
        assert_eq!(format!("{}", TokenKind::Whiles), "whiles");
        assert_eq!(format!("{}", TokenKind::Fer), "fer");
        assert_eq!(format!("{}", TokenKind::Gie), "gie");
        assert_eq!(format!("{}", TokenKind::Blether), "blether");
        assert_eq!(format!("{}", TokenKind::Speir), "speir");
        assert_eq!(format!("{}", TokenKind::Fae), "fae");
        assert_eq!(format!("{}", TokenKind::Tae), "tae");
        assert_eq!(format!("{}", TokenKind::An), "an");
        assert_eq!(format!("{}", TokenKind::Or), "or");
        assert_eq!(format!("{}", TokenKind::Nae), "nae");
        assert_eq!(format!("{}", TokenKind::Aye), "aye");
        assert_eq!(format!("{}", TokenKind::Naething), "naething");
        assert_eq!(format!("{}", TokenKind::Dae), "dae");
        assert_eq!(format!("{}", TokenKind::Thing), "thing");
        assert_eq!(format!("{}", TokenKind::Fetch), "fetch");
        assert_eq!(format!("{}", TokenKind::Kin), "kin");
        assert_eq!(format!("{}", TokenKind::Brak), "brak");
        assert_eq!(format!("{}", TokenKind::Haud), "haud");
        assert_eq!(format!("{}", TokenKind::In), "in");
        assert_eq!(format!("{}", TokenKind::Is), "is");
        assert_eq!(format!("{}", TokenKind::Masel), "masel");
        assert_eq!(format!("{}", TokenKind::HaeABash), "hae_a_bash");
        assert_eq!(
            format!("{}", TokenKind::GinItGangsWrang),
            "gin_it_gangs_wrang"
        );
        assert_eq!(format!("{}", TokenKind::Keek), "keek");
        assert_eq!(format!("{}", TokenKind::Whan), "whan");
        assert_eq!(format!("{}", TokenKind::MakSiccar), "mak_siccar");
    }

    #[test]
    fn test_token_kind_display_literals() {
        assert_eq!(format!("{}", TokenKind::Integer(42)), "42");
        assert_eq!(format!("{}", TokenKind::Integer(-17)), "-17");
        assert_eq!(format!("{}", TokenKind::Float(3.14)), "3.14");
        assert_eq!(
            format!("{}", TokenKind::String("hello".to_string())),
            "\"hello\""
        );
        assert_eq!(
            format!("{}", TokenKind::SingleQuoteString("world".to_string())),
            "'world'"
        );
        assert_eq!(
            format!("{}", TokenKind::FString("Hi {name}".to_string())),
            "f\"Hi {name}\""
        );
        assert_eq!(
            format!("{}", TokenKind::Identifier("my_var".to_string())),
            "my_var"
        );
    }

    #[test]
    fn test_token_kind_display_operators() {
        assert_eq!(format!("{}", TokenKind::Plus), "+");
        assert_eq!(format!("{}", TokenKind::Minus), "-");
        assert_eq!(format!("{}", TokenKind::Star), "*");
        assert_eq!(format!("{}", TokenKind::Slash), "/");
        assert_eq!(format!("{}", TokenKind::Percent), "%");
        assert_eq!(format!("{}", TokenKind::Equals), "=");
        assert_eq!(format!("{}", TokenKind::EqualsEquals), "==");
        assert_eq!(format!("{}", TokenKind::BangEquals), "!=");
        assert_eq!(format!("{}", TokenKind::Less), "<");
        assert_eq!(format!("{}", TokenKind::LessEquals), "<=");
        assert_eq!(format!("{}", TokenKind::Greater), ">");
        assert_eq!(format!("{}", TokenKind::GreaterEquals), ">=");
        assert_eq!(format!("{}", TokenKind::Bang), "!");
        assert_eq!(format!("{}", TokenKind::PlusEquals), "+=");
        assert_eq!(format!("{}", TokenKind::MinusEquals), "-=");
        assert_eq!(format!("{}", TokenKind::StarEquals), "*=");
        assert_eq!(format!("{}", TokenKind::SlashEquals), "/=");
        assert_eq!(format!("{}", TokenKind::DotDotDot), "...");
        assert_eq!(format!("{}", TokenKind::DotDot), "..");
        assert_eq!(format!("{}", TokenKind::Dot), ".");
        assert_eq!(format!("{}", TokenKind::Underscore), "_");
    }

    #[test]
    fn test_token_kind_display_delimiters() {
        assert_eq!(format!("{}", TokenKind::LeftParen), "(");
        assert_eq!(format!("{}", TokenKind::RightParen), ")");
        assert_eq!(format!("{}", TokenKind::LeftBrace), "{");
        assert_eq!(format!("{}", TokenKind::RightBrace), "}");
        assert_eq!(format!("{}", TokenKind::LeftBracket), "[");
        assert_eq!(format!("{}", TokenKind::RightBracket), "]");
        assert_eq!(format!("{}", TokenKind::Comma), ",");
        assert_eq!(format!("{}", TokenKind::Colon), ":");
        assert_eq!(format!("{}", TokenKind::Semicolon), ";");
        assert_eq!(format!("{}", TokenKind::Arrow), "->");
        assert_eq!(format!("{}", TokenKind::PipeForward), "|>");
        assert_eq!(format!("{}", TokenKind::Pipe), "|");
    }

    #[test]
    fn test_token_kind_display_special() {
        assert_eq!(format!("{}", TokenKind::Newline), "newline");
        assert_eq!(format!("{}", TokenKind::Comment), "comment");
        assert_eq!(format!("{}", TokenKind::Eof), "end of file");
    }

    #[test]
    fn test_token_new() {
        let token = Token::new(TokenKind::Ken, "ken".to_string(), 1, 5);
        assert_eq!(token.kind, TokenKind::Ken);
        assert_eq!(token.lexeme, "ken");
        assert_eq!(token.line, 1);
        assert_eq!(token.column, 5);
    }

    #[test]
    fn test_token_eof() {
        let token = Token::eof(10);
        assert_eq!(token.kind, TokenKind::Eof);
        assert_eq!(token.lexeme, "");
        assert_eq!(token.line, 10);
        assert_eq!(token.column, 0);
    }

    #[test]
    fn test_token_display() {
        let token = Token::new(TokenKind::Ken, "ken".to_string(), 5, 1);
        assert_eq!(format!("{}", token), "ken at line 5");

        let token2 = Token::new(TokenKind::Integer(42), "42".to_string(), 3, 10);
        assert_eq!(format!("{}", token2), "42 at line 3");
    }
}
