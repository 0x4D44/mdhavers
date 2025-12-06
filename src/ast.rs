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
    Expression { expr: Expr, span: Span },

    /// Block of statements: { ... }
    Block { statements: Vec<Stmt>, span: Span },

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
    Return { value: Option<Expr>, span: Span },

    /// Print statement: blether "hello"
    Print { value: Expr, span: Span },

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
    Literal { value: Literal, span: Span },

    /// Variable reference
    Variable { name: String, span: Span },

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
    List { elements: Vec<Expr>, span: Span },

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
    Grouping { expr: Box<Expr>, span: Span },

    /// Lambda/anonymous function: |x, y| x + y
    Lambda {
        params: Vec<String>,
        body: Box<Expr>,
        span: Span,
    },

    /// Self reference: masel
    Masel { span: Span },

    /// Input: speir "What's yer name?"
    Input { prompt: Box<Expr>, span: Span },

    /// Format string: f"Hullo {name}!"
    FString { parts: Vec<FStringPart>, span: Span },

    /// Spread expression: ...list (skail = scatter in Scots)
    Spread { expr: Box<Expr>, span: Span },

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_new() {
        let span = Span::new(5, 10);
        assert_eq!(span.line, 5);
        assert_eq!(span.column, 10);
    }

    #[test]
    fn test_program_new() {
        let stmts = vec![
            Stmt::Break { span: Span::new(1, 1) },
        ];
        let program = Program::new(stmts);
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_literal_display_integer() {
        assert_eq!(format!("{}", Literal::Integer(42)), "42");
        assert_eq!(format!("{}", Literal::Integer(-17)), "-17");
        assert_eq!(format!("{}", Literal::Integer(0)), "0");
    }

    #[test]
    fn test_literal_display_float() {
        assert_eq!(format!("{}", Literal::Float(3.14)), "3.14");
        assert_eq!(format!("{}", Literal::Float(-2.5)), "-2.5");
    }

    #[test]
    fn test_literal_display_string() {
        assert_eq!(format!("{}", Literal::String("hello".to_string())), "\"hello\"");
        assert_eq!(format!("{}", Literal::String("".to_string())), "\"\"");
    }

    #[test]
    fn test_literal_display_bool() {
        assert_eq!(format!("{}", Literal::Bool(true)), "aye");
        assert_eq!(format!("{}", Literal::Bool(false)), "nae");
    }

    #[test]
    fn test_literal_display_nil() {
        assert_eq!(format!("{}", Literal::Nil), "naething");
    }

    #[test]
    fn test_binary_op_display() {
        assert_eq!(format!("{}", BinaryOp::Add), "+");
        assert_eq!(format!("{}", BinaryOp::Subtract), "-");
        assert_eq!(format!("{}", BinaryOp::Multiply), "*");
        assert_eq!(format!("{}", BinaryOp::Divide), "/");
        assert_eq!(format!("{}", BinaryOp::Modulo), "%");
        assert_eq!(format!("{}", BinaryOp::Equal), "==");
        assert_eq!(format!("{}", BinaryOp::NotEqual), "!=");
        assert_eq!(format!("{}", BinaryOp::Less), "<");
        assert_eq!(format!("{}", BinaryOp::LessEqual), "<=");
        assert_eq!(format!("{}", BinaryOp::Greater), ">");
        assert_eq!(format!("{}", BinaryOp::GreaterEqual), ">=");
    }

    #[test]
    fn test_unary_op_display() {
        assert_eq!(format!("{}", UnaryOp::Negate), "-");
        assert_eq!(format!("{}", UnaryOp::Not), "nae");
    }

    #[test]
    fn test_logical_op_display() {
        assert_eq!(format!("{}", LogicalOp::And), "an");
        assert_eq!(format!("{}", LogicalOp::Or), "or");
    }

    #[test]
    fn test_param() {
        let param_no_default = Param {
            name: "x".to_string(),
            default: None,
        };
        assert_eq!(param_no_default.name, "x");
        assert!(param_no_default.default.is_none());

        let param_with_default = Param {
            name: "y".to_string(),
            default: Some(Expr::Literal {
                value: Literal::Integer(42),
                span: Span::new(1, 1),
            }),
        };
        assert_eq!(param_with_default.name, "y");
        assert!(param_with_default.default.is_some());
    }

    #[test]
    fn test_expr_span() {
        let span = Span::new(5, 10);

        let lit = Expr::Literal { value: Literal::Integer(42), span };
        assert_eq!(lit.span(), span);

        let var = Expr::Variable { name: "x".to_string(), span };
        assert_eq!(var.span(), span);

        let assign = Expr::Assign {
            name: "x".to_string(),
            value: Box::new(Expr::Literal { value: Literal::Integer(1), span }),
            span,
        };
        assert_eq!(assign.span(), span);

        let binary = Expr::Binary {
            left: Box::new(Expr::Literal { value: Literal::Integer(1), span }),
            operator: BinaryOp::Add,
            right: Box::new(Expr::Literal { value: Literal::Integer(2), span }),
            span,
        };
        assert_eq!(binary.span(), span);

        let unary = Expr::Unary {
            operator: UnaryOp::Negate,
            operand: Box::new(Expr::Literal { value: Literal::Integer(1), span }),
            span,
        };
        assert_eq!(unary.span(), span);

        let logical = Expr::Logical {
            left: Box::new(Expr::Literal { value: Literal::Bool(true), span }),
            operator: LogicalOp::And,
            right: Box::new(Expr::Literal { value: Literal::Bool(false), span }),
            span,
        };
        assert_eq!(logical.span(), span);

        let call = Expr::Call {
            callee: Box::new(Expr::Variable { name: "f".to_string(), span }),
            arguments: vec![],
            span,
        };
        assert_eq!(call.span(), span);

        let get = Expr::Get {
            object: Box::new(Expr::Variable { name: "obj".to_string(), span }),
            property: "prop".to_string(),
            span,
        };
        assert_eq!(get.span(), span);

        let set = Expr::Set {
            object: Box::new(Expr::Variable { name: "obj".to_string(), span }),
            property: "prop".to_string(),
            value: Box::new(Expr::Literal { value: Literal::Integer(1), span }),
            span,
        };
        assert_eq!(set.span(), span);

        let index = Expr::Index {
            object: Box::new(Expr::Variable { name: "arr".to_string(), span }),
            index: Box::new(Expr::Literal { value: Literal::Integer(0), span }),
            span,
        };
        assert_eq!(index.span(), span);

        let index_set = Expr::IndexSet {
            object: Box::new(Expr::Variable { name: "arr".to_string(), span }),
            index: Box::new(Expr::Literal { value: Literal::Integer(0), span }),
            value: Box::new(Expr::Literal { value: Literal::Integer(1), span }),
            span,
        };
        assert_eq!(index_set.span(), span);

        let slice = Expr::Slice {
            object: Box::new(Expr::Variable { name: "arr".to_string(), span }),
            start: None,
            end: None,
            step: None,
            span,
        };
        assert_eq!(slice.span(), span);

        let list = Expr::List { elements: vec![], span };
        assert_eq!(list.span(), span);

        let dict = Expr::Dict { pairs: vec![], span };
        assert_eq!(dict.span(), span);

        let range = Expr::Range {
            start: Box::new(Expr::Literal { value: Literal::Integer(0), span }),
            end: Box::new(Expr::Literal { value: Literal::Integer(10), span }),
            inclusive: false,
            span,
        };
        assert_eq!(range.span(), span);

        let grouping = Expr::Grouping {
            expr: Box::new(Expr::Literal { value: Literal::Integer(1), span }),
            span,
        };
        assert_eq!(grouping.span(), span);

        let lambda = Expr::Lambda {
            params: vec!["x".to_string()],
            body: Box::new(Expr::Variable { name: "x".to_string(), span }),
            span,
        };
        assert_eq!(lambda.span(), span);

        let masel = Expr::Masel { span };
        assert_eq!(masel.span(), span);

        let input = Expr::Input {
            prompt: Box::new(Expr::Literal { value: Literal::String("?".to_string()), span }),
            span,
        };
        assert_eq!(input.span(), span);

        let fstring = Expr::FString { parts: vec![], span };
        assert_eq!(fstring.span(), span);

        let spread = Expr::Spread {
            expr: Box::new(Expr::Variable { name: "arr".to_string(), span }),
            span,
        };
        assert_eq!(spread.span(), span);

        let pipe = Expr::Pipe {
            left: Box::new(Expr::Literal { value: Literal::Integer(1), span }),
            right: Box::new(Expr::Variable { name: "f".to_string(), span }),
            span,
        };
        assert_eq!(pipe.span(), span);

        let ternary = Expr::Ternary {
            condition: Box::new(Expr::Literal { value: Literal::Bool(true), span }),
            then_expr: Box::new(Expr::Literal { value: Literal::Integer(1), span }),
            else_expr: Box::new(Expr::Literal { value: Literal::Integer(0), span }),
            span,
        };
        assert_eq!(ternary.span(), span);
    }

    #[test]
    fn test_stmt_span() {
        let span = Span::new(3, 5);

        let var_decl = Stmt::VarDecl {
            name: "x".to_string(),
            initializer: None,
            span,
        };
        assert_eq!(var_decl.span(), span);

        let expr_stmt = Stmt::Expression {
            expr: Expr::Literal { value: Literal::Integer(1), span },
            span,
        };
        assert_eq!(expr_stmt.span(), span);

        let block = Stmt::Block { statements: vec![], span };
        assert_eq!(block.span(), span);

        let if_stmt = Stmt::If {
            condition: Expr::Literal { value: Literal::Bool(true), span },
            then_branch: Box::new(Stmt::Block { statements: vec![], span }),
            else_branch: None,
            span,
        };
        assert_eq!(if_stmt.span(), span);

        let while_stmt = Stmt::While {
            condition: Expr::Literal { value: Literal::Bool(true), span },
            body: Box::new(Stmt::Block { statements: vec![], span }),
            span,
        };
        assert_eq!(while_stmt.span(), span);

        let for_stmt = Stmt::For {
            variable: "i".to_string(),
            iterable: Expr::Literal { value: Literal::Integer(0), span },
            body: Box::new(Stmt::Block { statements: vec![], span }),
            span,
        };
        assert_eq!(for_stmt.span(), span);

        let func = Stmt::Function {
            name: "foo".to_string(),
            params: vec![],
            body: vec![],
            span,
        };
        assert_eq!(func.span(), span);

        let ret = Stmt::Return { value: None, span };
        assert_eq!(ret.span(), span);

        let print = Stmt::Print {
            value: Expr::Literal { value: Literal::String("hi".to_string()), span },
            span,
        };
        assert_eq!(print.span(), span);

        let brk = Stmt::Break { span };
        assert_eq!(brk.span(), span);

        let cont = Stmt::Continue { span };
        assert_eq!(cont.span(), span);

        let class = Stmt::Class {
            name: "Foo".to_string(),
            superclass: None,
            methods: vec![],
            span,
        };
        assert_eq!(class.span(), span);

        let strct = Stmt::Struct {
            name: "Bar".to_string(),
            fields: vec![],
            span,
        };
        assert_eq!(strct.span(), span);

        let import = Stmt::Import {
            path: "lib".to_string(),
            alias: None,
            span,
        };
        assert_eq!(import.span(), span);

        let try_catch = Stmt::TryCatch {
            try_block: Box::new(Stmt::Block { statements: vec![], span }),
            error_name: "e".to_string(),
            catch_block: Box::new(Stmt::Block { statements: vec![], span }),
            span,
        };
        assert_eq!(try_catch.span(), span);

        let match_stmt = Stmt::Match {
            value: Expr::Literal { value: Literal::Integer(1), span },
            arms: vec![],
            span,
        };
        assert_eq!(match_stmt.span(), span);

        let assert = Stmt::Assert {
            condition: Expr::Literal { value: Literal::Bool(true), span },
            message: None,
            span,
        };
        assert_eq!(assert.span(), span);

        let destruct = Stmt::Destructure {
            patterns: vec![],
            value: Expr::List { elements: vec![], span },
            span,
        };
        assert_eq!(destruct.span(), span);
    }

    #[test]
    fn test_pattern_variants() {
        let lit_pattern = Pattern::Literal(Literal::Integer(42));
        let id_pattern = Pattern::Identifier("x".to_string());
        let wildcard = Pattern::Wildcard;
        let span = Span::new(1, 1);
        let range_pattern = Pattern::Range {
            start: Box::new(Expr::Literal { value: Literal::Integer(0), span }),
            end: Box::new(Expr::Literal { value: Literal::Integer(10), span }),
        };

        // Just verify they can be created and matched
        match lit_pattern {
            Pattern::Literal(Literal::Integer(42)) => {}
            _ => panic!("Expected integer literal pattern"),
        }
        match id_pattern {
            Pattern::Identifier(ref name) => assert_eq!(name, "x"),
            _ => panic!("Expected identifier pattern"),
        }
        match wildcard {
            Pattern::Wildcard => {}
            _ => panic!("Expected wildcard"),
        }
        match range_pattern {
            Pattern::Range { .. } => {}
            _ => panic!("Expected range pattern"),
        }
    }

    #[test]
    fn test_destruct_pattern_variants() {
        let var = DestructPattern::Variable("x".to_string());
        let rest = DestructPattern::Rest("remaining".to_string());
        let ignore = DestructPattern::Ignore;

        match var {
            DestructPattern::Variable(ref name) => assert_eq!(name, "x"),
            _ => panic!("Expected variable pattern"),
        }
        match rest {
            DestructPattern::Rest(ref name) => assert_eq!(name, "remaining"),
            _ => panic!("Expected rest pattern"),
        }
        match ignore {
            DestructPattern::Ignore => {}
            _ => panic!("Expected ignore pattern"),
        }
    }

    #[test]
    fn test_fstring_part_variants() {
        let text = FStringPart::Text("hello ".to_string());
        let span = Span::new(1, 1);
        let expr = FStringPart::Expr(Box::new(Expr::Variable {
            name: "name".to_string(),
            span,
        }));

        match text {
            FStringPart::Text(ref s) => assert_eq!(s, "hello "),
            _ => panic!("Expected text part"),
        }
        match expr {
            FStringPart::Expr(_) => {}
            _ => panic!("Expected expr part"),
        }
    }

    #[test]
    fn test_match_arm() {
        let span = Span::new(1, 1);
        let arm = MatchArm {
            pattern: Pattern::Wildcard,
            body: Stmt::Break { span },
            span,
        };
        assert_eq!(arm.span, span);
    }
}
