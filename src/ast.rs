use std::fmt;

/// Span information for error reporting
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(line: usize, column: usize) -> Self {
        Span { line, column }
    }
}

/// A program is a list of statements
#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

impl Program {
    pub fn new(statements: Vec<Stmt>) -> Self {
        Program { statements }
    }
}

/// Statements in mdhavers
#[derive(Debug, Clone)]
pub enum Stmt {
    /// Variable declaration: ken x = 5
    VarDecl {
        name: String,
        initializer: Option<Expr>,
        span: Span,
    },

    /// Expression statement: blether "hello"
    Expression {
        expr: Expr,
        span: Span,
    },

    /// Block of statements: { ... }
    Block {
        statements: Vec<Stmt>,
        span: Span,
    },

    /// If statement: gin x > 5 { ... } ither { ... }
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
        span: Span,
    },

    /// While loop: whiles x < 10 { ... }
    While {
        condition: Expr,
        body: Box<Stmt>,
        span: Span,
    },

    /// For loop: fer i in 1..10 { ... }
    For {
        variable: String,
        iterable: Expr,
        body: Box<Stmt>,
        span: Span,
    },

    /// Function definition: dae greet(name) { ... }
    /// Supports default parameter values: dae greet(name, greeting = "Hullo") { ... }
    Function {
        name: String,
        params: Vec<Param>,
        body: Vec<Stmt>,
        span: Span,
    },

    /// Return statement: gie value
    Return {
        value: Option<Expr>,
        span: Span,
    },

    /// Print statement: blether "hello"
    Print {
        value: Expr,
        span: Span,
    },

    /// Break statement: brak
    Break { span: Span },

    /// Continue statement: haud
    Continue { span: Span },

    /// Class definition: kin Animal { ... }
    Class {
        name: String,
        superclass: Option<String>,
        methods: Vec<Stmt>,
        span: Span,
    },

    /// Struct definition: thing Point { x, y }
    Struct {
        name: String,
        fields: Vec<String>,
        span: Span,
    },

    /// Import statement: fetch "module"
    Import {
        path: String,
        alias: Option<String>,
        span: Span,
    },

    /// Try-catch: hae_a_bash { ... } gin_it_gangs_wrang e { ... }
    TryCatch {
        try_block: Box<Stmt>,
        error_name: String,
        catch_block: Box<Stmt>,
        span: Span,
    },

    /// Match statement: keek value { whan 1 -> ..., whan 2 -> ... }
    Match {
        value: Expr,
        arms: Vec<MatchArm>,
        span: Span,
    },

    /// Assert statement: mak_siccar condition, "message"
    Assert {
        condition: Expr,
        message: Option<Expr>,
        span: Span,
    },

    /// Destructuring assignment: ken [a, b, ...rest] = list
    Destructure {
        patterns: Vec<DestructPattern>,
        value: Expr,
        span: Span,
    },
}

/// A match arm: whan pattern -> body
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Stmt,
    pub span: Span,
}

/// Patterns for matching
#[derive(Debug, Clone)]
pub enum Pattern {
    /// Literal value
    Literal(Literal),
    /// Identifier (binds the value)
    Identifier(String),
    /// Wildcard (_)
    Wildcard,
    /// Range pattern: 1..10
    Range { start: Box<Expr>, end: Box<Expr> },
}

/// A function parameter with optional default value
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub default: Option<Expr>,
}

/// Destructuring pattern fer unpacking lists
#[derive(Debug, Clone)]
pub enum DestructPattern {
    /// Single variable: x
    Variable(String),
    /// Rest pattern: ...rest (captures remaining elements)
    Rest(String),
    /// Ignore: _ (skip this element)
    Ignore,
}

/// Expressions in mdhavers
#[derive(Debug, Clone)]
pub enum Expr {
    /// Literal values
    Literal {
        value: Literal,
        span: Span,
    },

    /// Variable reference
    Variable {
        name: String,
        span: Span,
    },

    /// Assignment: x = 5
    Assign {
        name: String,
        value: Box<Expr>,
        span: Span,
    },

    /// Binary operation: x + y
    Binary {
        left: Box<Expr>,
        operator: BinaryOp,
        right: Box<Expr>,
        span: Span,
    },

    /// Unary operation: -x, nae x
    Unary {
        operator: UnaryOp,
        operand: Box<Expr>,
        span: Span,
    },

    /// Logical operation: x an y, x or y
    Logical {
        left: Box<Expr>,
        operator: LogicalOp,
        right: Box<Expr>,
        span: Span,
    },

    /// Function call: greet("world")
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
        span: Span,
    },

    /// Property access: obj.property
    Get {
        object: Box<Expr>,
        property: String,
        span: Span,
    },

    /// Property assignment: obj.property = value
    Set {
        object: Box<Expr>,
        property: String,
        value: Box<Expr>,
        span: Span,
    },

    /// Index access: arr[0]
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },

    /// Index assignment: arr[0] = value
    IndexSet {
        object: Box<Expr>,
        index: Box<Expr>,
        value: Box<Expr>,
        span: Span,
    },

    /// Slice expression: arr[1:3] or arr[:3] or arr[1:] or arr[::2]
    Slice {
        object: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        step: Option<Box<Expr>>,
        span: Span,
    },

    /// List literal: [1, 2, 3]
    List {
        elements: Vec<Expr>,
        span: Span,
    },

    /// Dictionary literal: {key: value}
    Dict {
        pairs: Vec<(Expr, Expr)>,
        span: Span,
    },

    /// Range: 1..10
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
        span: Span,
    },

    /// Grouping: (x + y)
    Grouping {
        expr: Box<Expr>,
        span: Span,
    },

    /// Lambda/anonymous function: |x, y| x + y
    Lambda {
        params: Vec<String>,
        body: Box<Expr>,
        span: Span,
    },

    /// Self reference: masel
    Masel { span: Span },

    /// Input: speir "What's yer name?"
    Input {
        prompt: Box<Expr>,
        span: Span,
    },

    /// Format string: f"Hullo {name}!"
    FString {
        parts: Vec<FStringPart>,
        span: Span,
    },

    /// Spread expression: ...list (skail = scatter in Scots)
    Spread {
        expr: Box<Expr>,
        span: Span,
    },

    /// Pipe forward: x |> f means f(x) - fer fluent chaining
    Pipe {
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },

    /// Ternary/conditional expression: gin condition than truthy ither falsy
    Ternary {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
        span: Span,
    },
}

/// Parts of an f-string
#[derive(Debug, Clone)]
pub enum FStringPart {
    /// Literal text
    Text(String),
    /// Interpolated expression
    Expr(Box<Expr>),
}

/// Literal values
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Nil,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Integer(n) => write!(f, "{}", n),
            Literal::Float(n) => write!(f, "{}", n),
            Literal::String(s) => write!(f, "\"{}\"", s),
            Literal::Bool(true) => write!(f, "aye"),
            Literal::Bool(false) => write!(f, "nae"),
            Literal::Nil => write!(f, "naething"),
        }
    }
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Subtract => write!(f, "-"),
            BinaryOp::Multiply => write!(f, "*"),
            BinaryOp::Divide => write!(f, "/"),
            BinaryOp::Modulo => write!(f, "%"),
            BinaryOp::Equal => write!(f, "=="),
            BinaryOp::NotEqual => write!(f, "!="),
            BinaryOp::Less => write!(f, "<"),
            BinaryOp::LessEqual => write!(f, "<="),
            BinaryOp::Greater => write!(f, ">"),
            BinaryOp::GreaterEqual => write!(f, ">="),
        }
    }
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Negate,
    Not,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Negate => write!(f, "-"),
            UnaryOp::Not => write!(f, "nae"),
        }
    }
}

/// Logical operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogicalOp {
    And,
    Or,
}

impl fmt::Display for LogicalOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogicalOp::And => write!(f, "an"),
            LogicalOp::Or => write!(f, "or"),
        }
    }
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal { span, .. } => *span,
            Expr::Variable { span, .. } => *span,
            Expr::Assign { span, .. } => *span,
            Expr::Binary { span, .. } => *span,
            Expr::Unary { span, .. } => *span,
            Expr::Logical { span, .. } => *span,
            Expr::Call { span, .. } => *span,
            Expr::Get { span, .. } => *span,
            Expr::Set { span, .. } => *span,
            Expr::Index { span, .. } => *span,
            Expr::IndexSet { span, .. } => *span,
            Expr::Slice { span, .. } => *span,
            Expr::List { span, .. } => *span,
            Expr::Dict { span, .. } => *span,
            Expr::Range { span, .. } => *span,
            Expr::Grouping { span, .. } => *span,
            Expr::Lambda { span, .. } => *span,
            Expr::Masel { span } => *span,
            Expr::Input { span, .. } => *span,
            Expr::FString { span, .. } => *span,
            Expr::Spread { span, .. } => *span,
            Expr::Pipe { span, .. } => *span,
            Expr::Ternary { span, .. } => *span,
        }
    }
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::VarDecl { span, .. } => *span,
            Stmt::Expression { span, .. } => *span,
            Stmt::Block { span, .. } => *span,
            Stmt::If { span, .. } => *span,
            Stmt::While { span, .. } => *span,
            Stmt::For { span, .. } => *span,
            Stmt::Function { span, .. } => *span,
            Stmt::Return { span, .. } => *span,
            Stmt::Print { span, .. } => *span,
            Stmt::Break { span } => *span,
            Stmt::Continue { span } => *span,
            Stmt::Class { span, .. } => *span,
            Stmt::Struct { span, .. } => *span,
            Stmt::Import { span, .. } => *span,
            Stmt::TryCatch { span, .. } => *span,
            Stmt::Match { span, .. } => *span,
            Stmt::Assert { span, .. } => *span,
            Stmt::Destructure { span, .. } => *span,
        }
    }
}
