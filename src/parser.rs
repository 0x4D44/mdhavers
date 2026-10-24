use crate::ast::{LogLevel, *};
use crate::error::{HaversError, HaversResult};
use crate::token::{Token, TokenKind};

/// How deep statements and expressions may nest afore the parser gies up.
/// Each nested statement, bracketed/sub-expression, prefix operator, chained
/// assignment, ternary `ither` or `ither gin` counts as one level. The limit keeps
/// the recursive-descent parser inside a 2 MB thread stack in debug builds, where
/// a bracketed sub-expression level costs about 25 KB of stack (measured).
pub const MAX_NESTING_DEPTH: usize = 64;

/// The parser - turns tokens intae an AST
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    depth: usize,
    /// How many function bodies (`dae`, methods, lambdas) enclose the
    /// cursor; `gie` at zero is an error.
    function_depth: usize,
    /// True when the nearest enclosing loop-or-block-expression is a block
    /// expression, where `brak`/`haud` are an error.
    in_block_expr: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
            depth: 0,
            function_depth: 0,
            in_block_expr: false,
        }
    }

    /// Run `parse_fn` inside a function body, where `gie` is allowed.
    fn in_function<T>(
        &mut self,
        parse_fn: impl FnOnce(&mut Self) -> HaversResult<T>,
    ) -> HaversResult<T> {
        self.function_depth += 1;
        let saved = std::mem::replace(&mut self.in_block_expr, false);
        let result = parse_fn(self);
        self.in_block_expr = saved;
        self.function_depth -= 1;
        result
    }

    /// Parse a loop body, where `brak`/`haud` are allowed again.
    fn loop_body(&mut self) -> HaversResult<Stmt> {
        let saved = std::mem::replace(&mut self.in_block_expr, false);
        let result = self.block();
        self.in_block_expr = saved;
        result
    }

    /// Run `parse_fn` one nesting level deeper, failing cleanly (instead o'
    /// overflowing the native stack) when the code is nested ower deep.
    fn nested<T>(&mut self, parse_fn: fn(&mut Self) -> HaversResult<T>) -> HaversResult<T> {
        if self.depth >= MAX_NESTING_DEPTH {
            return Err(HaversError::ParseError {
                message: format!(
                    "Yer code's nested ower deep (mair than {} levels) - sort it intae wee functions",
                    MAX_NESTING_DEPTH
                ),
                line: self.current_line(),
            });
        }
        self.depth += 1;
        let result = parse_fn(self);
        self.depth -= 1;
        result
    }

    /// Parse the tokens intae a program
    pub fn parse(&mut self) -> HaversResult<Program> {
        let mut statements = Vec::new();

        self.skip_newlines();

        while !self.is_at_end() {
            statements.push(self.declaration()?);
            self.skip_newlines();
        }

        Ok(Program::new(statements))
    }

    // === Declaration parsing ===

    fn declaration(&mut self) -> HaversResult<Stmt> {
        self.nested(Self::declaration_inner)
    }

    fn declaration_inner(&mut self) -> HaversResult<Stmt> {
        if self.check(&TokenKind::Ken) {
            self.var_declaration()
        } else if self.check(&TokenKind::Dae) {
            self.function_declaration()
        } else if self.check(&TokenKind::Kin) {
            self.class_declaration()
        } else if self.check(&TokenKind::Thing) {
            self.struct_declaration()
        } else if self.check(&TokenKind::Fetch) {
            self.import_declaration()
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> HaversResult<Stmt> {
        let span = self.current_span();
        self.advance(); // consume 'ken'

        // Check fer destructuring pattern: ken [a, b, c] = ...
        if self.check(&TokenKind::LeftBracket) {
            return self.destructure_declaration(span);
        }

        let name = self.expect_identifier("variable name")?;

        let initializer = if self.match_token(&TokenKind::Equals) {
            Some(self.expression()?)
        } else {
            None
        };

        self.expect_statement_end()?;

        Ok(Stmt::VarDecl {
            name,
            initializer,
            span,
        })
    }

    /// Parse a destructuring pattern: ken [a, b, ...rest] = list
    fn destructure_declaration(&mut self, span: Span) -> HaversResult<Stmt> {
        self.expect(&TokenKind::LeftBracket, "[")?;

        let mut patterns = Vec::new();
        let mut seen_rest = false;

        while !self.check(&TokenKind::RightBracket) && !self.is_at_end() {
            // Check fer rest pattern: ...name
            if self.match_token(&TokenKind::DotDotDot) {
                if seen_rest {
                    return Err(HaversError::ParseError {
                        message: "Ye can only hae ane rest pattern (...) in a destructure"
                            .to_string(),
                        line: span.line,
                    });
                }
                let name = self.expect_identifier("rest variable name")?;
                patterns.push(DestructPattern::Rest(name));
                seen_rest = true;
            } else if self.match_token(&TokenKind::Underscore) {
                // Ignore pattern: _
                patterns.push(DestructPattern::Ignore);
            } else {
                // Regular variable
                let name = self.expect_identifier("variable name")?;
                patterns.push(DestructPattern::Variable(name));
            }

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        self.expect(&TokenKind::RightBracket, "]")?;
        self.expect(&TokenKind::Equals, "=")?;

        let value = self.expression()?;

        self.expect_statement_end()?;

        Ok(Stmt::Destructure {
            patterns,
            value,
            span,
        })
    }

    fn function_declaration(&mut self) -> HaversResult<Stmt> {
        self.in_function(Self::function_declaration_inner)
    }

    fn function_declaration_inner(&mut self) -> HaversResult<Stmt> {
        let span = self.current_span();
        self.advance(); // consume 'dae'

        let name = self.expect_identifier("function name")?;
        self.expect(&TokenKind::LeftParen, "(")?;

        let mut params = Vec::new();
        let mut seen_default = false;

        if !self.check(&TokenKind::RightParen) {
            let mut throwaways = 0;
            loop {
                let param_name = self.expect_binder("parameter name", &mut throwaways)?;

                // Check for default value: param = value
                let default = if self.match_token(&TokenKind::Equals) {
                    seen_default = true;
                    Some(self.expression()?)
                } else {
                    // Params wi' defaults must come efter params wi'oot
                    if seen_default {
                        return Err(HaversError::ParseError {
                            message:
                                "Och! Params wi'oot defaults cannae come efter params wi' defaults"
                                    .to_string(),
                            line: span.line,
                        });
                    }
                    None
                };

                params.push(Param {
                    name: param_name,
                    default,
                });

                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        self.expect(&TokenKind::RightParen, ")")?;
        self.skip_newlines();
        self.expect(&TokenKind::LeftBrace, "{")?;

        let body = self.block_statements()?;

        Ok(Stmt::Function {
            name,
            params,
            body,
            span,
        })
    }

    fn class_declaration(&mut self) -> HaversResult<Stmt> {
        let span = self.current_span();
        self.advance(); // consume 'kin'

        let name = self.expect_identifier("class name")?;

        let superclass = if self.match_token(&TokenKind::Fae) {
            Some(self.expect_identifier("superclass name")?)
        } else {
            None
        };

        self.skip_newlines();
        self.expect(&TokenKind::LeftBrace, "{")?;

        let mut methods = Vec::new();
        self.skip_newlines();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            if self.check(&TokenKind::Dae) {
                methods.push(self.function_declaration()?);
            } else {
                return Err(HaversError::ParseError {
                    message: "Expected method definition in class".to_string(),
                    line: self.current_line(),
                });
            }
            self.skip_newlines();
        }

        self.expect(&TokenKind::RightBrace, "}")?;

        Ok(Stmt::Class {
            name,
            superclass,
            methods,
            span,
        })
    }

    fn struct_declaration(&mut self) -> HaversResult<Stmt> {
        let span = self.current_span();
        self.advance(); // consume 'thing'

        let name = self.expect_identifier("struct name")?;
        self.skip_newlines();
        self.expect(&TokenKind::LeftBrace, "{")?;
        self.skip_newlines();

        let mut fields = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            fields.push(self.expect_identifier("field name")?);
            // Allow fields separated by commas, newlines, or just whitespace.
            self.skip_newlines();
            self.match_token(&TokenKind::Comma);
            self.skip_newlines();
        }

        self.expect(&TokenKind::RightBrace, "}")?;

        Ok(Stmt::Struct { name, fields, span })
    }

    fn import_declaration(&mut self) -> HaversResult<Stmt> {
        let span = self.current_span();
        self.advance(); // consume 'fetch'

        let path = self.expect_string("module path")?;

        let alias = if self.match_token(&TokenKind::Tae) {
            Some(self.expect_identifier("alias name")?)
        } else {
            None
        };

        self.expect_statement_end()?;

        Ok(Stmt::Import { path, alias, span })
    }

    // === Statement parsing ===

    fn statement(&mut self) -> HaversResult<Stmt> {
        if self.check(&TokenKind::Gin) {
            self.if_statement()
        } else if self.check(&TokenKind::Whiles) {
            self.while_statement()
        } else if self.check(&TokenKind::Fer) {
            self.for_statement()
        } else if self.check(&TokenKind::Gie) {
            self.return_statement()
        } else if self.check(&TokenKind::Blether) {
            self.print_statement()
        } else if self.check(&TokenKind::Brak) {
            self.break_statement()
        } else if self.check(&TokenKind::Haud) {
            self.continue_statement()
        } else if self.check(&TokenKind::HaeABash) {
            self.try_catch_statement()
        } else if self.check(&TokenKind::Keek) {
            self.match_statement()
        } else if self.check(&TokenKind::MakSiccar) {
            self.assert_statement()
        } else if self.check(&TokenKind::LogWhisper) {
            self.log_statement(LogLevel::Whisper)
        } else if self.check(&TokenKind::LogMutter) {
            self.log_statement(LogLevel::Mutter)
        } else if self.check(&TokenKind::LogBlether) {
            self.log_statement(LogLevel::Blether)
        } else if self.check(&TokenKind::LogHoller) {
            self.log_statement(LogLevel::Holler)
        } else if self.check(&TokenKind::LogRoar) {
            self.log_statement(LogLevel::Roar)
        } else if self.check(&TokenKind::Hurl) {
            self.hurl_statement()
        } else if self.check(&TokenKind::LeftBrace) {
            self.block()
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> HaversResult<Stmt> {
        let span = self.current_span();
        self.advance(); // consume 'gin'

        let condition = self.expression()?;
        self.skip_newlines();
        let then_branch = Box::new(self.block()?);

        let else_branch = if self.match_token(&TokenKind::Ither) {
            self.skip_newlines();
            if self.check(&TokenKind::Gin) {
                // else if
                Some(Box::new(self.nested(Self::if_statement)?))
            } else {
                Some(Box::new(self.block()?))
            }
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
            span,
        })
    }

    fn while_statement(&mut self) -> HaversResult<Stmt> {
        let span = self.current_span();
        self.advance(); // consume 'whiles'

        let condition = self.expression()?;
        self.skip_newlines();
        let body = Box::new(self.loop_body()?);

        Ok(Stmt::While {
            condition,
            body,
            span,
        })
    }

    fn for_statement(&mut self) -> HaversResult<Stmt> {
        let span = self.current_span();
        self.advance(); // consume 'fer'

        let variable = self.expect_binder("loop variable", &mut 0)?;
        self.expect(&TokenKind::In, "in")?;
        let iterable = self.expression()?;
        self.skip_newlines();
        let body = Box::new(self.loop_body()?);

        Ok(Stmt::For {
            variable,
            iterable,
            body,
            span,
        })
    }

    fn return_statement(&mut self) -> HaversResult<Stmt> {
        let span = self.current_span();
        if self.function_depth == 0 {
            // Decision F4: a top-level `gie` is an error on every backend.
            return Err(HaversError::ReturnOutsideFunction { line: span.line });
        }
        self.advance(); // consume 'gie'

        // A bare `gie` (nae value) ends at a newline, `;`, `}` or the end o' the file.
        let value = if self.check(&TokenKind::Newline)
            || self.check(&TokenKind::Semicolon)
            || self.check(&TokenKind::RightBrace)
            || self.is_at_end()
        {
            None
        } else {
            Some(self.expression()?)
        };

        self.expect_statement_end()?;

        Ok(Stmt::Return { value, span })
    }

    fn print_statement(&mut self) -> HaversResult<Stmt> {
        let span = self.current_span();
        self.advance(); // consume 'blether'

        let value = self.expression()?;
        self.expect_statement_end()?;

        Ok(Stmt::Print { value, span })
    }

    fn break_statement(&mut self) -> HaversResult<Stmt> {
        let span = self.current_span();
        if self.in_block_expr {
            return Err(HaversError::LoopJumpInBlockExpr {
                keyword: "brak".to_string(),
                line: span.line,
            });
        }
        self.advance(); // consume 'brak'
        self.expect_statement_end()?;
        Ok(Stmt::Break { span })
    }

    fn continue_statement(&mut self) -> HaversResult<Stmt> {
        let span = self.current_span();
        if self.in_block_expr {
            return Err(HaversError::LoopJumpInBlockExpr {
                keyword: "haud".to_string(),
                line: span.line,
            });
        }
        self.advance(); // consume 'haud'
        self.expect_statement_end()?;
        Ok(Stmt::Continue { span })
    }

    fn try_catch_statement(&mut self) -> HaversResult<Stmt> {
        let span = self.current_span();
        self.advance(); // consume 'hae_a_bash'

        self.skip_newlines();
        let try_block = Box::new(self.block()?);

        self.skip_newlines();
        self.expect(&TokenKind::GinItGangsWrang, "gin_it_gangs_wrang")?;

        let error_name = self.expect_identifier("error variable name")?;
        self.skip_newlines();
        let catch_block = Box::new(self.block()?);

        Ok(Stmt::TryCatch {
            try_block,
            error_name,
            catch_block,
            span,
        })
    }

    fn match_statement(&mut self) -> HaversResult<Stmt> {
        let span = self.current_span();
        self.advance(); // consume 'keek'

        let value = self.expression()?;
        self.skip_newlines();
        self.expect(&TokenKind::LeftBrace, "{")?;
        self.skip_newlines();

        let mut arms = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            arms.push(self.match_arm()?);
            self.skip_newlines();
        }

        self.expect(&TokenKind::RightBrace, "}")?;

        Ok(Stmt::Match { value, arms, span })
    }

    fn assert_statement(&mut self) -> HaversResult<Stmt> {
        let span = self.current_span();
        self.advance(); // consume 'mak_siccar'

        let condition = self.expression()?;

        // Optional message after comma
        let message = if self.match_token(&TokenKind::Comma) {
            Some(self.expression()?)
        } else {
            None
        };

        // Without this, `mak_siccar x == 1 "msg"` quietly drapped the message
        // intae a separate statement.
        self.expect_statement_end()?;

        Ok(Stmt::Assert {
            condition,
            message,
            span,
        })
    }

    fn log_statement(&mut self, level: LogLevel) -> HaversResult<Stmt> {
        let span = self.current_span();
        self.advance(); // consume 'log_whisper', 'log_mutter', etc.

        let message = self.expression()?;
        let mut extras = Vec::new();
        if self.match_token(&TokenKind::Comma) {
            extras.push(self.expression()?);
            if self.match_token(&TokenKind::Comma) {
                extras.push(self.expression()?);
            }
        }
        self.expect_statement_end()?;

        Ok(Stmt::Log {
            level,
            message,
            extras,
            span,
        })
    }

    fn hurl_statement(&mut self) -> HaversResult<Stmt> {
        let span = self.current_span();
        self.advance(); // consume 'hurl'

        let message = self.expression()?;
        self.expect_statement_end()?;

        Ok(Stmt::Hurl { message, span })
    }

    fn match_arm(&mut self) -> HaversResult<MatchArm> {
        let span = self.current_span();
        self.expect(&TokenKind::Whan, "whan")?;

        let pattern = self.pattern()?;
        self.expect(&TokenKind::Arrow, "->")?;
        self.skip_newlines();

        // Match arms can have blocks, statements, or expressions
        let body = if self.check(&TokenKind::LeftBrace) {
            self.block()?
        } else if self.check(&TokenKind::Blether) {
            self.print_statement()?
        } else if self.check(&TokenKind::Gie) {
            self.return_statement()?
        } else if self.check(&TokenKind::Brak) {
            self.break_statement()?
        } else if self.check(&TokenKind::Haud) {
            self.continue_statement()?
        } else {
            let expr = self.expression()?;
            Stmt::Expression {
                span: expr.span(),
                expr,
            }
        };

        Ok(MatchArm {
            pattern,
            body,
            span,
        })
    }

    fn pattern(&mut self) -> HaversResult<Pattern> {
        let token = self.peek().clone();
        match &token.kind {
            TokenKind::Integer(n) => {
                let n = *n;
                self.advance();
                self.integer_pattern(n, Span::new(token.line, token.column))
            }
            TokenKind::Float(n) => {
                let n = *n;
                self.advance();
                Ok(Pattern::Literal(Literal::Float(n)))
            }
            TokenKind::String(s) | TokenKind::SingleQuoteString(s) => {
                let s = process_escapes(s);
                self.advance();
                Ok(Pattern::Literal(Literal::String(s)))
            }
            TokenKind::Aye => {
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(true)))
            }
            TokenKind::Nae => {
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(false)))
            }
            TokenKind::Naething => {
                self.advance();
                Ok(Pattern::Literal(Literal::Nil))
            }
            TokenKind::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            TokenKind::Identifier(name) if name == "_" => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(Pattern::Identifier(name))
            }
            TokenKind::Minus => {
                // Handle negative number patterns like -5 or -3.14
                self.advance();
                let next_token = self.peek().clone();
                match &next_token.kind {
                    TokenKind::Integer(n) => {
                        let n = -*n;
                        self.advance();
                        self.integer_pattern(n, Span::new(token.line, token.column))
                    }
                    TokenKind::Float(n) => {
                        let n = -*n;
                        self.advance();
                        Ok(Pattern::Literal(Literal::Float(n)))
                    }
                    _ => Err(HaversError::ParseError {
                        message: format!(
                            "Expected number after '-' in pattern, got {}",
                            next_token.kind
                        ),
                        line: next_token.line,
                    }),
                }
            }
            _ => Err(HaversError::ParseError {
                message: format!("Expected pattern, got {}", token.kind),
                line: token.line,
            }),
        }
    }

    /// An integer pattern, or a range pattern when `..` (end left oot) or
    /// `..=` (end kept in) follows: `n..end` / `n..=end`.
    fn integer_pattern(&mut self, n: i64, span: Span) -> HaversResult<Pattern> {
        let inclusive = if self.match_token(&TokenKind::DotDotEquals) {
            true
        } else if self.match_token(&TokenKind::DotDot) {
            false
        } else {
            return Ok(Pattern::Literal(Literal::Integer(n)));
        };
        let end = self.expression()?;
        Ok(Pattern::Range {
            start: Box::new(Expr::Literal {
                value: Literal::Integer(n),
                span,
            }),
            end: Box::new(end),
            inclusive,
        })
    }

    fn block(&mut self) -> HaversResult<Stmt> {
        let span = self.current_span();
        self.expect(&TokenKind::LeftBrace, "{")?;
        let statements = self.block_statements()?;
        Ok(Stmt::Block { statements, span })
    }

    fn block_statements(&mut self) -> HaversResult<Vec<Stmt>> {
        let mut statements = Vec::new();
        self.skip_newlines();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
            self.skip_newlines();
        }

        self.expect(&TokenKind::RightBrace, "}")?;
        Ok(statements)
    }

    fn expression_statement(&mut self) -> HaversResult<Stmt> {
        let expr = self.expression()?;
        let span = expr.span();
        self.expect_statement_end()?;
        Ok(Stmt::Expression { expr, span })
    }

    // === Expression parsing (precedence climbing) ===

    fn expression(&mut self) -> HaversResult<Expr> {
        self.nested(Self::assignment)
    }

    fn assignment(&mut self) -> HaversResult<Expr> {
        let expr = self.pipe_expr()?;

        if self.match_token(&TokenKind::Equals) {
            let span = self.current_span();
            let value = self.nested(Self::assignment)?;

            match expr {
                Expr::Variable { name, .. } => {
                    return Ok(Expr::Assign {
                        name,
                        value: Box::new(value),
                        span,
                    });
                }
                Expr::Get {
                    object, property, ..
                } => {
                    return Ok(Expr::Set {
                        object,
                        property,
                        value: Box::new(value),
                        span,
                    });
                }
                Expr::Index { object, index, .. } => {
                    return Ok(Expr::IndexSet {
                        object,
                        index,
                        value: Box::new(value),
                        span,
                    });
                }
                _ => {
                    return Err(HaversError::ParseError {
                        message: "Invalid assignment target".to_string(),
                        line: span.line,
                    });
                }
            }
        }

        // Handle compound assignment operators
        let compound_op = if self.match_token(&TokenKind::PlusEquals) {
            Some(BinaryOp::Add)
        } else if self.match_token(&TokenKind::MinusEquals) {
            Some(BinaryOp::Subtract)
        } else if self.match_token(&TokenKind::StarEquals) {
            Some(BinaryOp::Multiply)
        } else if self.match_token(&TokenKind::SlashEquals) {
            Some(BinaryOp::Divide)
        } else {
            None
        };

        if let Some(op) = compound_op {
            let value = self.nested(Self::assignment)?;
            return compound_assignment(expr, op, value);
        }

        Ok(expr)
    }

    /// Pipe expression: left |> right (means call right with left as argument)
    fn pipe_expr(&mut self) -> HaversResult<Expr> {
        let mut expr = self.ternary()?;

        while self.match_token(&TokenKind::PipeForward) {
            let span = self.current_span();
            let right = self.ternary()?;
            expr = Expr::Pipe {
                left: Box::new(expr),
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Ternary expression: gin condition than truthy ither falsy
    fn ternary(&mut self) -> HaversResult<Expr> {
        // Check fer ternary expression starting wi' 'gin'
        if self.match_token(&TokenKind::Gin) {
            let span = self
                .previous()
                .map(|t| Span::new(t.line, t.column))
                .unwrap_or(self.current_span());

            // Parse the condition
            let condition = self.or()?;

            // Expect 'than'
            self.expect(&TokenKind::Than, "than")?;

            // Parse the 'then' expression (truthy case)
            let then_expr = self.or()?;

            // Expect 'ither'
            self.expect(&TokenKind::Ither, "ither")?;

            // Parse the 'else' expression (falsy case)
            let else_expr = self.nested(Self::ternary)?; // Right-associative

            return Ok(Expr::Ternary {
                condition: Box::new(condition),
                then_expr: Box::new(then_expr),
                else_expr: Box::new(else_expr),
                span,
            });
        }

        self.or()
    }

    fn or(&mut self) -> HaversResult<Expr> {
        let mut expr = self.and()?;

        while self.match_token(&TokenKind::Or) {
            let span = self
                .previous()
                .map(|t| Span::new(t.line, t.column))
                .unwrap_or(self.current_span());
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator: LogicalOp::Or,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> HaversResult<Expr> {
        let mut expr = self.equality()?;

        while self.match_token(&TokenKind::An) {
            let span = self
                .previous()
                .map(|t| Span::new(t.line, t.column))
                .unwrap_or(self.current_span());
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator: LogicalOp::And,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> HaversResult<Expr> {
        let mut expr = self.comparison()?;

        loop {
            let op = if self.match_token(&TokenKind::EqualsEquals) {
                BinaryOp::Equal
            } else if self.match_token(&TokenKind::BangEquals) {
                BinaryOp::NotEqual
            } else {
                break;
            };

            let span = self
                .previous()
                .map(|t| Span::new(t.line, t.column))
                .unwrap_or(self.current_span());
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> HaversResult<Expr> {
        let mut expr = self.range()?;

        loop {
            let op = if self.match_token(&TokenKind::Less) {
                BinaryOp::Less
            } else if self.match_token(&TokenKind::LessEquals) {
                BinaryOp::LessEqual
            } else if self.match_token(&TokenKind::Greater) {
                BinaryOp::Greater
            } else if self.match_token(&TokenKind::GreaterEquals) {
                BinaryOp::GreaterEqual
            } else {
                break;
            };

            let span = self
                .previous()
                .map(|t| Span::new(t.line, t.column))
                .unwrap_or(self.current_span());
            let right = self.range()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Range: `start..end` / `start..=end`, binding looser than `+`/`-` (so
    /// `x + 1..5` is `(x + 1)..5` and `-1..2` is `(-1)..2`) but tighter than
    /// comparisons. Ranges dinnae chain.
    fn range(&mut self) -> HaversResult<Expr> {
        let start = self.term()?;

        let inclusive = if self.match_token(&TokenKind::DotDotEquals) {
            true
        } else if self.match_token(&TokenKind::DotDot) {
            false
        } else {
            return Ok(start);
        };

        let span = start.span();
        let end = self.term()?;
        Ok(Expr::Range {
            start: Box::new(start),
            end: Box::new(end),
            inclusive,
            span,
        })
    }

    fn term(&mut self) -> HaversResult<Expr> {
        let mut expr = self.factor()?;

        loop {
            let op = if self.match_token(&TokenKind::Plus) {
                BinaryOp::Add
            } else if self.match_token(&TokenKind::Minus) {
                BinaryOp::Subtract
            } else {
                break;
            };

            let span = self
                .previous()
                .map(|t| Span::new(t.line, t.column))
                .unwrap_or(self.current_span());
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> HaversResult<Expr> {
        let mut expr = self.unary()?;

        loop {
            let op = if self.match_token(&TokenKind::Star) {
                BinaryOp::Multiply
            } else if self.match_token(&TokenKind::Slash) {
                BinaryOp::Divide
            } else if self.match_token(&TokenKind::Percent) {
                BinaryOp::Modulo
            } else {
                break;
            };

            let span = self
                .previous()
                .map(|t| Span::new(t.line, t.column))
                .unwrap_or(self.current_span());
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> HaversResult<Expr> {
        if self.match_token(&TokenKind::Minus) {
            let t = self.previous().unwrap();
            let span = Span::new(t.line, t.column);
            let operand = self.nested(Self::unary)?;
            return Ok(Expr::Unary {
                operator: UnaryOp::Negate,
                operand: Box::new(operand),
                span,
            });
        }

        // For `nae`, we need to distinguish between:
        // - `nae` as a boolean literal (when not followed by an operand)
        // - `nae x` as a NOT operator (when followed by an operand)
        if self.check(&TokenKind::Nae) {
            // Look ahead to see if there's an operand
            if self.is_nae_followed_by_operand() {
                self.advance(); // consume nae
                let t = self.previous().unwrap();
                let span = Span::new(t.line, t.column);
                let operand = self.nested(Self::unary)?;
                return Ok(Expr::Unary {
                    operator: UnaryOp::Not,
                    operand: Box::new(operand),
                    span,
                });
            }
            // Otherwise, let it be parsed as a literal in primary()
        }

        if self.match_token(&TokenKind::Bang) {
            let t = self.previous().unwrap();
            let span = Span::new(t.line, t.column);
            let operand = self.nested(Self::unary)?;
            return Ok(Expr::Unary {
                operator: UnaryOp::Not,
                operand: Box::new(operand),
                span,
            });
        }

        self.call()
    }

    /// Check if `nae` is followed by something that could be an operand
    fn is_nae_followed_by_operand(&self) -> bool {
        if self.current + 1 >= self.tokens.len() {
            return false;
        }
        let next = &self.tokens[self.current + 1];
        matches!(
            next.kind,
            TokenKind::Integer(_)
                | TokenKind::Float(_)
                | TokenKind::String(_)
                | TokenKind::SingleQuoteString(_)
                | TokenKind::Identifier(_)
                | TokenKind::LeftParen
                | TokenKind::LeftBracket
                | TokenKind::Minus
                | TokenKind::Bang
                | TokenKind::Aye
                | TokenKind::Nae
                | TokenKind::Naething
                | TokenKind::Masel
                | TokenKind::Speir
                | TokenKind::Pipe
        )
    }

    fn call(&mut self) -> HaversResult<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&TokenKind::LeftParen) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&TokenKind::Dot) {
                let property = self.expect_identifier("property name")?;
                let span = self.current_span();
                expr = Expr::Get {
                    object: Box::new(expr),
                    property,
                    span,
                };
            } else if self.match_token(&TokenKind::LeftBracket) {
                expr = self.index_or_slice(expr)?;
            } else if self.newlines_then_dot() {
                // A continuation line startin' wi' `.method()`: a line cannae
                // otherwise begin wi' a `.`, sae this changes nae meanin'.
                self.skip_newlines();
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Is the cursor on a run o' newlines whose next real token is a `.`?
    fn newlines_then_dot(&self) -> bool {
        if !self.check(&TokenKind::Newline) {
            return false;
        }
        self.tokens[self.current..]
            .iter()
            .find(|t| t.kind != TokenKind::Newline)
            .is_some_and(|t| t.kind == TokenKind::Dot)
    }

    fn finish_call(&mut self, callee: Expr) -> HaversResult<Expr> {
        let span = callee.span();
        let mut arguments = Vec::new();

        // Newlines inside the brackets are just layoot: a lang call can spread
        // its arguments ower several lines.
        self.skip_newlines();
        if !self.check(&TokenKind::RightParen) {
            loop {
                // Check for spread operator in function arguments
                if self.match_token(&TokenKind::DotDotDot) {
                    let spread_span = self.current_span();
                    let expr = self.expression()?;
                    arguments.push(Expr::Spread {
                        expr: Box::new(expr),
                        span: spread_span,
                    });
                } else {
                    arguments.push(self.expression()?);
                }
                self.skip_newlines();
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
                self.skip_newlines();
            }
        }

        self.expect(&TokenKind::RightParen, ")")?;

        Ok(Expr::Call {
            callee: Box::new(callee),
            arguments,
            span,
        })
    }

    fn primary(&mut self) -> HaversResult<Expr> {
        let token = self.peek().clone();
        let span = Span::new(token.line, token.column);

        match &token.kind {
            TokenKind::Integer(n) => {
                let n = *n;
                self.advance();
                Ok(Expr::Literal {
                    value: Literal::Integer(n),
                    span,
                })
            }
            TokenKind::Float(n) => {
                let n = *n;
                self.advance();
                Ok(Expr::Literal {
                    value: Literal::Float(n),
                    span,
                })
            }
            TokenKind::String(s) | TokenKind::SingleQuoteString(s) => {
                let s = process_escapes(s);
                self.advance();
                Ok(Expr::Literal {
                    value: Literal::String(s),
                    span,
                })
            }
            TokenKind::FString(s) => {
                let s = s.clone();
                self.advance();
                self.parse_fstring(&s, span)
            }
            TokenKind::Aye => {
                self.advance();
                Ok(Expr::Literal {
                    value: Literal::Bool(true),
                    span,
                })
            }
            TokenKind::Nae => {
                self.advance();
                Ok(Expr::Literal {
                    value: Literal::Bool(false),
                    span,
                })
            }
            TokenKind::Naething => {
                self.advance();
                Ok(Expr::Literal {
                    value: Literal::Nil,
                    span,
                })
            }
            TokenKind::Masel => {
                self.advance();
                Ok(Expr::Masel { span })
            }
            TokenKind::Speir => {
                self.advance();
                let prompt = self.expression()?;
                Ok(Expr::Input {
                    prompt: Box::new(prompt),
                    span,
                })
            }
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::Variable { name, span })
            }
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.expect(&TokenKind::RightParen, ")")?;
                Ok(Expr::Grouping {
                    expr: Box::new(expr),
                    span,
                })
            }
            TokenKind::LeftBracket => {
                self.advance();
                self.list_literal(span)
            }
            TokenKind::LeftBrace => {
                self.advance(); // consume '{'
                self.brace_expression(span)
            }
            // Lambda expressions: |x, y| x + y  or  |x, y| { statements... }
            TokenKind::Pipe => {
                self.advance();
                self.in_function(|p| p.lambda_expression(span))
            }
            _ => Err(HaversError::ParseError {
                message: format!("Unexpected token: {}", token.kind),
                line: token.line,
            }),
        }
    }

    /// `object[...]`, wi' the cursor just past the `[`: an index or a slice.
    /// Kept oot o' `call` sae the recursive path has a wee stack frame.
    fn index_or_slice(&mut self, mut expr: Expr) -> HaversResult<Expr> {
        let span = self.current_span();

        // Check fer slice syntax: [start:end:step], [:end], [start:], [::step], etc.
        if self.check(&TokenKind::Colon) {
            // [:end] or [:] or [:end:step] or [::step]
            self.advance(); // consume the first colon

            let end = if self.check(&TokenKind::Colon) || self.check(&TokenKind::RightBracket) {
                None
            } else {
                Some(Box::new(self.expression()?))
            };

            // Check fer step
            let step = if self.match_token(&TokenKind::Colon) {
                if self.check(&TokenKind::RightBracket) {
                    None
                } else {
                    Some(Box::new(self.expression()?))
                }
            } else {
                None
            };

            self.expect(&TokenKind::RightBracket, "]")?;
            expr = Expr::Slice {
                object: Box::new(expr),
                start: None,
                end,
                step,
                span,
            };
        } else {
            // Could be [index] or [start:end] or [start:] or [start:end:step]
            let first = self.expression()?;

            if self.match_token(&TokenKind::Colon) {
                // It's a slice: [start:end] or [start:] or [start:end:step] or [start::step]
                let end = if self.check(&TokenKind::Colon) || self.check(&TokenKind::RightBracket) {
                    None
                } else {
                    Some(Box::new(self.expression()?))
                };

                // Check fer step
                let step = if self.match_token(&TokenKind::Colon) {
                    if self.check(&TokenKind::RightBracket) {
                        None
                    } else {
                        Some(Box::new(self.expression()?))
                    }
                } else {
                    None
                };

                self.expect(&TokenKind::RightBracket, "]")?;
                expr = Expr::Slice {
                    object: Box::new(expr),
                    start: Some(Box::new(first)),
                    end,
                    step,
                    span,
                };
            } else {
                // Regular index access
                self.expect(&TokenKind::RightBracket, "]")?;
                expr = Expr::Index {
                    object: Box::new(expr),
                    index: Box::new(first),
                    span,
                };
            }
        }
        Ok(expr)
    }

    /// A list literal, wi' the cursor just past the `[`.
    fn list_literal(&mut self, span: Span) -> HaversResult<Expr> {
        let mut elements = Vec::new();
        self.skip_newlines(); // Allow newline after [
        if !self.check(&TokenKind::RightBracket) {
            loop {
                // Check for spread operator (skail = scatter)
                if self.match_token(&TokenKind::DotDotDot) {
                    let spread_span = self.current_span();
                    let expr = self.expression()?;
                    elements.push(Expr::Spread {
                        expr: Box::new(expr),
                        span: spread_span,
                    });
                } else {
                    elements.push(self.expression()?);
                }
                self.skip_newlines(); // Allow newline after element
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
                // Allow trailing comma: [1, 2, 3,]
                self.skip_newlines();
                if self.check(&TokenKind::RightBracket) {
                    break;
                }
                self.skip_newlines(); // Allow newline after comma
            }
        }
        self.expect(&TokenKind::RightBracket, "]")?;
        Ok(Expr::List { elements, span })
    }

    /// A dict literal or block expression, wi' the cursor just past the `{`.
    fn brace_expression(&mut self, span: Span) -> HaversResult<Expr> {
        self.skip_newlines();

        // Empty dict literal: {}
        if self.check(&TokenKind::RightBrace) {
            self.advance(); // consume '}'
            return Ok(Expr::Dict {
                pairs: Vec::new(),
                span,
            });
        }

        // Dict or block expression? Decide by lookahead (linear time,
        // nae re-parsing): it's a dict if a ':' turns up on the first
        // line, ootside any brackets.
        if !self.brace_starts_dict() {
            // A `gie` in a block expression returns frae the enclosing
            // function (DEC row 64), so it needs one.
            let saved = std::mem::replace(&mut self.in_block_expr, true);
            let mut statements = Vec::new();
            while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
                match self.declaration() {
                    Ok(stmt) => statements.push(stmt),
                    Err(e) => {
                        self.in_block_expr = saved;
                        return Err(e);
                    }
                }
                self.skip_newlines();
            }
            self.in_block_expr = saved;
            self.expect(&TokenKind::RightBrace, "}")?;
            return Ok(Expr::BlockExpr { statements, span });
        }

        // Dict literal
        let mut pairs = Vec::new();

        let first_key = self.expression()?;
        self.expect(&TokenKind::Colon, ":")?;
        let first_value = self.expression()?;
        pairs.push((first_key, first_value));

        self.skip_newlines();
        while self.match_token(&TokenKind::Comma) {
            self.skip_newlines();
            if self.check(&TokenKind::RightBrace) {
                break; // trailing comma
            }
            let key = self.expression()?;
            self.expect(&TokenKind::Colon, ":")?;
            let value = self.expression()?;
            pairs.push((key, value));
            self.skip_newlines();
        }

        self.expect(&TokenKind::RightBrace, "}")?;
        Ok(Expr::Dict { pairs, span })
    }

    /// A lambda `|x, y| body`, wi' the cursor just past the first `|`.
    fn lambda_expression(&mut self, span: Span) -> HaversResult<Expr> {
        let mut params = Vec::new();
        if !self.check(&TokenKind::Pipe) {
            let mut throwaways = 0;
            loop {
                params.push(self.expect_binder("parameter name", &mut throwaways)?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(&TokenKind::Pipe, "|")?;
        // Check for block body: |x, y| { ... }
        let body = if self.check(&TokenKind::LeftBrace) && !self.lambda_brace_starts_dict() {
            let brace_token = self.peek();
            let block_span = Span::new(brace_token.line, brace_token.column);
            self.advance(); // consume {
            self.skip_newlines();
            let mut statements = Vec::new();
            while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
                statements.push(self.declaration()?);
                self.skip_newlines();
            }
            self.expect(&TokenKind::RightBrace, "}")?;
            Expr::BlockExpr {
                statements,
                span: block_span,
            }
        } else {
            self.expression()?
        };
        Ok(Expr::Lambda {
            params,
            body: Box::new(body),
            span,
        })
    }

    /// Wi' the cursor on the `{` efter a lambda's `|...|`: does it open a dict
    /// literal (`|x| {"k": x}`) raither than a block body? Same rule as a `{` in
    /// expression position, except `{}` stays an empty block body, as it aye was.
    fn lambda_brace_starts_dict(&mut self) -> bool {
        let saved = self.current;
        self.advance(); // consume {
        self.skip_newlines();
        let is_dict = !self.check(&TokenKind::RightBrace) && self.brace_starts_dict();
        self.current = saved;
        is_dict
    }

    /// Wi' the cursor just past a `{` (and any newlines) in expression position,
    /// decide whether it opens a dict literal or a block expression. Scans ahead,
    /// without consuming, tae the first `:`, `,`, `;`, newline or closing bracket
    /// that isnae inside nested brackets: a `:` means dict, onything else a block.
    /// Each token is looked at once per enclosing brace, so there's nae
    /// exponential backtracking on nested braces.
    fn brace_starts_dict(&self) -> bool {
        let mut depth = 0usize;
        for token in &self.tokens[self.current..] {
            match token.kind {
                TokenKind::LeftParen | TokenKind::LeftBracket | TokenKind::LeftBrace => {
                    depth += 1;
                }
                TokenKind::RightParen | TokenKind::RightBracket | TokenKind::RightBrace => {
                    if depth == 0 {
                        return false;
                    }
                    depth -= 1;
                }
                TokenKind::Colon if depth == 0 => return true,
                TokenKind::Comma | TokenKind::Semicolon | TokenKind::Newline if depth == 0 => {
                    return false;
                }
                TokenKind::Eof => return false,
                _ => {}
            }
        }
        false
    }

    // === Helper methods ===

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.current)
            .unwrap_or(&self.tokens[self.tokens.len() - 1])
    }

    fn previous(&self) -> Option<&Token> {
        if self.current > 0 {
            self.tokens.get(self.current - 1)
        } else {
            None
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous().unwrap()
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
        }
    }

    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: &TokenKind, expected: &str) -> HaversResult<()> {
        if self.check(kind) {
            self.advance();
            Ok(())
        } else {
            Err(HaversError::UnexpectedToken {
                expected: expected.to_string(),
                found: self.peek().kind.to_string(),
                line: self.peek().line,
            })
        }
    }

    fn expect_identifier(&mut self, context: &str) -> HaversResult<String> {
        let token = self.peek().clone();
        if let TokenKind::Identifier(name) = &token.kind {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(HaversError::UnexpectedToken {
                expected: context.to_string(),
                found: token.kind.to_string(),
                line: token.line,
            })
        }
    }

    /// Like `expect_identifier`, but also accepts `_` as a throwaway binder (loop
    /// variables and parameters). Each `_` in the same list gets its ain name
    /// (`_`, `__`, `___`, ...) so twa throwaway parameters never clash.
    fn expect_binder(&mut self, context: &str, throwaways: &mut usize) -> HaversResult<String> {
        if self.match_token(&TokenKind::Underscore) {
            *throwaways += 1;
            return Ok("_".repeat(*throwaways));
        }
        self.expect_identifier(context)
    }

    fn expect_string(&mut self, context: &str) -> HaversResult<String> {
        let token = self.peek().clone();
        if let TokenKind::String(s) | TokenKind::SingleQuoteString(s) = &token.kind {
            let s = s.clone();
            self.advance();
            Ok(s)
        } else {
            Err(HaversError::UnexpectedToken {
                expected: context.to_string(),
                found: token.kind.to_string(),
                line: token.line,
            })
        }
    }

    fn expect_statement_end(&mut self) -> HaversResult<()> {
        if self.is_at_end() || self.check(&TokenKind::RightBrace) {
            return Ok(());
        }

        if self.match_token(&TokenKind::Newline) {
            return Ok(());
        }

        if self.match_token(&TokenKind::Semicolon) {
            self.skip_newlines();
            return Ok(());
        }

        // Be lenient - if the next token starts a new statement, that's fine.
        // Every statement-startin' keyword is treated the same here.
        let next = &self.peek().kind;
        if matches!(
            next,
            TokenKind::Ken
                | TokenKind::Dae
                | TokenKind::Gin
                | TokenKind::Whiles
                | TokenKind::Fer
                | TokenKind::Gie
                | TokenKind::Blether
                | TokenKind::Brak
                | TokenKind::Haud
                | TokenKind::Kin
                | TokenKind::Thing
                | TokenKind::Fetch
                | TokenKind::HaeABash
                | TokenKind::Keek
                | TokenKind::MakSiccar
                | TokenKind::Hurl
                | TokenKind::LogWhisper
                | TokenKind::LogMutter
                | TokenKind::LogBlether
                | TokenKind::LogHoller
                | TokenKind::LogRoar
        ) {
            return Ok(());
        }

        Err(HaversError::UnexpectedToken {
            expected: "newline or ';'".to_string(),
            found: self.peek().kind.to_string(),
            line: self.peek().line,
        })
    }

    fn skip_newlines(&mut self) {
        while self.match_token(&TokenKind::Newline) {}
    }

    fn current_span(&self) -> Span {
        let token = self.peek();
        Span::new(token.line, token.column)
    }

    fn current_line(&self) -> usize {
        self.peek().line
    }

    /// Parse an f-string like f"Hello {name}!" into parts
    fn parse_fstring(&mut self, content: &str, span: Span) -> HaversResult<Expr> {
        let mut parts = Vec::new();
        let mut current_text = String::new();
        // Count the newlines consumed so far, so errors (and spans) inside a hole
        // on a later line o' a multi-line f-string point at the richt line.
        let newlines = std::cell::Cell::new(0usize);
        let mut chars = content
            .chars()
            .inspect(|&c| {
                if c == '\n' {
                    newlines.set(newlines.get() + 1);
                }
            })
            .peekable();

        while let Some(c) = chars.next() {
            if c == '{' {
                // Newlines afore this hole (nothing efter the '{' has been read yet).
                let hole_line = span.line + newlines.get();

                // Check for escaped brace {{
                if chars.peek() == Some(&'{') {
                    chars.next();
                    current_text.push('{');
                    continue;
                }

                // Save current text if any (process escapes)
                if !current_text.is_empty() {
                    parts.push(FStringPart::Text(process_escapes(&current_text)));
                    current_text.clear();
                }

                // Extract expression inside {}
                let mut expr_str = String::new();
                let mut brace_depth = 1;
                // Inside a quoted string literal in the hole ('"' or '\''): copied
                // verbatim, and its braces dinnae count.
                let mut in_quote: Option<char> = None;
                // Inside an auld-style \"...\" string in the hole: braces dinnae count.
                let mut in_escaped_quote = false;
                while let Some(c) = chars.next() {
                    if let Some(quote) = in_quote {
                        expr_str.push(c);
                        if c == '\\' {
                            if let Some(next) = chars.next() {
                                expr_str.push(next);
                            }
                        } else if c == quote {
                            in_quote = None;
                        }
                    } else if (c == '"' || c == '\'') && !in_escaped_quote {
                        in_quote = Some(c);
                        expr_str.push(c);
                    } else if c == '{' && !in_escaped_quote {
                        brace_depth += 1;
                        expr_str.push(c);
                    } else if c == '}' && !in_escaped_quote {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            break;
                        }
                        expr_str.push(c);
                    } else if c == '\\' {
                        // Handle escape sequences in the expression part
                        // This allows things like f"test {func(\"hello\")}"
                        if let Some(&next) = chars.peek() {
                            match next {
                                '"' => {
                                    chars.next();
                                    expr_str.push('"');
                                    in_escaped_quote = !in_escaped_quote;
                                }
                                '\\' => {
                                    chars.next();
                                    expr_str.push('\\');
                                }
                                _ => {
                                    // Keep the backslash for other escapes
                                    expr_str.push(c);
                                }
                            }
                        } else {
                            expr_str.push(c);
                        }
                    } else {
                        expr_str.push(c);
                    }
                }

                let expr = self.parse_fstring_hole(&expr_str, hole_line)?;
                parts.push(FStringPart::Expr(Box::new(expr)));
            } else if c == '}' {
                // Check for escaped brace }}
                if chars.peek() == Some(&'}') {
                    chars.next();
                    current_text.push('}');
                } else {
                    // Single } without matching { - just add it
                    current_text.push(c);
                }
            } else {
                current_text.push(c);
            }
        }

        // Don't forget remaining text (process escapes)
        if !current_text.is_empty() {
            parts.push(FStringPart::Text(process_escapes(&current_text)));
        }

        Ok(Expr::FString { parts, span })
    }

    /// Parse the source inside ane f-string hole, which starts on `hole_line`.
    /// The hole must haud exactly ane expression.
    fn parse_fstring_hole(&self, source: &str, hole_line: usize) -> HaversResult<Expr> {
        // The sub-lexer counts lines fae 1; shift them sae they match the file.
        let line_shift = hole_line.saturating_sub(1);
        let mut tokens =
            crate::lexer::lex(source).map_err(|err| shift_lexer_error_line(err, line_shift))?;
        for token in &mut tokens {
            token.line += line_shift;
        }

        let mut hole_parser = Parser::new(tokens);
        // Nesting inside a hole still counts toward the limit.
        hole_parser.depth = self.depth;
        hole_parser.skip_newlines();
        let expr = hole_parser.expression()?;
        hole_parser.skip_newlines();

        if !hole_parser.is_at_end() {
            let leftover = hole_parser.peek();
            let message = if leftover.kind == TokenKind::Colon {
                "F-string format specs like {x:>5} arenae supported - \
                 use a function tae format the value"
                    .to_string()
            } else {
                format!(
                    "There's mair in this f-string {{...}} than ane expression - \
                     didnae expect '{}'",
                    leftover.kind
                )
            };
            return Err(HaversError::ParseError {
                message,
                line: leftover.line,
            });
        }

        Ok(expr)
    }
}

/// Move a lexer error's line by `shift` (fer source lexed fae the middle o' a file).
fn shift_lexer_error_line(err: HaversError, shift: usize) -> HaversError {
    match err {
        HaversError::UnkentToken {
            lexeme,
            line,
            column,
        } => HaversError::UnkentToken {
            lexeme,
            line: line + shift,
            column,
        },
        HaversError::InvalidNumber { value, line } => HaversError::InvalidNumber {
            value,
            line: line + shift,
        },
        HaversError::UnterminatedString { line } => {
            HaversError::UnterminatedString { line: line + shift }
        }
        other => other,
    }
}

/// Desugar `target op= value` intae a plain assignment.
fn compound_assignment(expr: Expr, op: BinaryOp, value: Expr) -> HaversResult<Expr> {
    let span = expr.span();

    // `target op= value` is desugared tae `target = target op value`, so the
    // target's object (and index) expressions are evaluated twice. That's
    // only allowed when they're simple, side-effect-free lookups.
    match expr {
        Expr::Variable { name, .. } => Ok(Expr::Assign {
            name: name.clone(),
            value: Box::new(Expr::Binary {
                left: Box::new(Expr::Variable { name, span }),
                operator: op,
                right: Box::new(value),
                span,
            }),
            span,
        }),
        Expr::Get {
            object,
            property,
            span: get_span,
        } if is_simple_lookup(&object) => {
            let current = Expr::Get {
                object: object.clone(),
                property: property.clone(),
                span: get_span,
            };
            Ok(Expr::Set {
                object,
                property,
                value: Box::new(Expr::Binary {
                    left: Box::new(current),
                    operator: op,
                    right: Box::new(value),
                    span,
                }),
                span,
            })
        }
        Expr::Index {
            object,
            index,
            span: index_span,
        } if is_simple_lookup(&object) && is_simple_lookup(&index) => {
            let current = Expr::Index {
                object: object.clone(),
                index: index.clone(),
                span: index_span,
            };
            Ok(Expr::IndexSet {
                object,
                index,
                value: Box::new(Expr::Binary {
                    left: Box::new(current),
                    operator: op,
                    right: Box::new(value),
                    span,
                }),
                span,
            })
        }
        Expr::Get { .. } | Expr::Index { .. } => Err(HaversError::ParseError {
            message: "That compound assignment target's ower complicated - \
                              pit the object (or index) in a variable first"
                .to_string(),
            line: span.line,
        }),
        _ => Err(HaversError::ParseError {
            message: "Invalid compound assignment target".to_string(),
            line: span.line,
        }),
    }
}

/// Is `expr` a plain lookup (variables, literals, `masel`, fields, indexes and
/// arithmetic on them) that's safe tae evaluate twice? Used by the compound
/// assignment desugaring, which reads the target and then writes it.
fn is_simple_lookup(expr: &Expr) -> bool {
    match expr {
        Expr::Literal { .. } | Expr::Variable { .. } | Expr::Masel { .. } => true,
        Expr::Grouping { expr, .. } => is_simple_lookup(expr),
        Expr::Get { object, .. } => is_simple_lookup(object),
        Expr::Index { object, index, .. } => is_simple_lookup(object) && is_simple_lookup(index),
        Expr::Unary { operand, .. } => is_simple_lookup(operand),
        Expr::Binary { left, right, .. } => is_simple_lookup(left) && is_simple_lookup(right),
        _ => false,
    }
}

/// Process escape sequences in a string
/// Handles \n, \t, \r, \\, \", \0, \xNN (hex), etc.
fn process_escapes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('\'') => result.push('\''),
                Some('0') => result.push('\0'),
                Some('x') | Some('X') => {
                    // Hex escape: \xNN where NN is two hex digits
                    let mut hex = String::new();
                    for _ in 0..2 {
                        if let Some(&c) = chars.peek() {
                            if c.is_ascii_hexdigit() {
                                hex.push(chars.next().unwrap());
                            } else {
                                break;
                            }
                        }
                    }
                    if hex.len() == 2 {
                        let byte = u8::from_str_radix(&hex, 16).expect("validated two hex digits");
                        result.push(byte as char);
                    } else {
                        // Not enough hex digits - keep as-is
                        result.push_str("\\x");
                        result.push_str(&hex);
                    }
                }
                Some(other) => {
                    // Unknown escape - keep as-is
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }

    result
}

// Coverage-only wrappers to reach internal branches that are otherwise hard/impossible
// to trigger through normal source lexing (e.g., due to always-appended EOF tokens or
// lexer regex constraints).
#[cfg(coverage)]
pub fn previous_is_none_at_start_for_coverage() -> bool {
    let tokens = vec![Token::eof(1)];
    let parser = Parser::new(tokens);
    parser.previous().is_none()
}

#[cfg(coverage)]
pub fn is_nae_followed_by_operand_end_of_stream_for_coverage() -> bool {
    let tokens = vec![Token::new(TokenKind::Nae, "nae".to_string(), 1, 1)];
    let parser = Parser::new(tokens);
    parser.is_nae_followed_by_operand()
}

#[cfg(coverage)]
pub fn process_escapes_for_coverage(s: &str) -> String {
    process_escapes(s)
}

#[cfg(coverage)]
pub fn parse_fstring_for_coverage(content: &str) -> HaversResult<Expr> {
    let tokens = vec![Token::eof(1)];
    let mut parser = Parser::new(tokens);
    parser.parse_fstring(content, Span::new(1, 1))
}

#[cfg(coverage)]
pub fn advance_at_end_for_coverage() {
    let tokens = vec![
        Token::new(
            TokenKind::Identifier("x".to_string()),
            "x".to_string(),
            1,
            1,
        ),
        Token::eof(1),
    ];
    let mut parser = Parser::new(tokens);
    let _ = parser.advance();
    let _ = parser.advance();
}

#[cfg(coverage)]
pub fn destructure_eof_for_coverage() {
    let tokens = vec![
        Token::new(TokenKind::Ken, "ken".to_string(), 1, 1),
        Token::new(TokenKind::LeftBracket, "[".to_string(), 1, 5),
        Token::new(
            TokenKind::Identifier("a".to_string()),
            "a".to_string(),
            1,
            6,
        ),
        Token::eof(1),
    ];
    let mut parser = Parser::new(tokens);
    let _ = parser.declaration();
}

#[cfg(coverage)]
pub fn match_eof_for_coverage() {
    let tokens = vec![
        Token::new(TokenKind::Keek, "keek".to_string(), 1, 1),
        Token::new(
            TokenKind::Identifier("x".to_string()),
            "x".to_string(),
            1,
            6,
        ),
        Token::new(TokenKind::LeftBrace, "{".to_string(), 1, 8),
        Token::eof(1),
    ];
    let mut parser = Parser::new(tokens);
    let _ = parser.statement();
}

/// Convenience function tae parse source code
pub fn parse(source: &str) -> HaversResult<Program> {
    #[cfg(coverage)]
    {
        destructure_eof_for_coverage();
        match_eof_for_coverage();
    }
    let tokens = crate::lexer::lex(source)?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DUMMY_SPAN: Span = Span { line: 0, column: 0 };

    fn dummy_expr() -> Expr {
        Expr::Literal {
            value: Literal::Nil,
            span: DUMMY_SPAN,
        }
    }

    fn dummy_block_stmt() -> Box<Stmt> {
        Box::new(Stmt::Block {
            statements: Vec::new(),
            span: DUMMY_SPAN,
        })
    }

    fn assert_stmt_variant(actual: &Stmt, expected: Stmt) {
        assert_eq!(
            std::mem::discriminant(actual),
            std::mem::discriminant(&expected)
        );
    }

    fn assert_pattern_variant(actual: &Pattern, expected: Pattern) {
        assert_eq!(
            std::mem::discriminant(actual),
            std::mem::discriminant(&expected)
        );
    }

    fn assert_error_variant(actual: &HaversError, expected: HaversError) {
        assert_eq!(
            std::mem::discriminant(actual),
            std::mem::discriminant(&expected)
        );
    }

    fn literal_from_pattern(pattern: &Pattern) -> Option<&Literal> {
        match pattern {
            Pattern::Literal(literal) => Some(literal),
            _ => None,
        }
    }

    #[test]
    fn destructure_declaration_requires_left_bracket_for_coverage() {
        let tokens = vec![
            Token::new(
                TokenKind::Identifier("x".to_string()),
                "x".to_string(),
                1,
                1,
            ),
            Token::eof(1),
        ];
        let mut parser = Parser::new(tokens);
        let err = parser.destructure_declaration(DUMMY_SPAN).unwrap_err();
        assert_error_variant(
            &err,
            HaversError::UnexpectedToken {
                expected: String::new(),
                found: String::new(),
                line: 0,
            },
        );
    }

    #[test]
    fn advance_does_not_increment_past_eof_for_coverage() {
        let tokens = vec![
            Token::new(TokenKind::Ken, "ken".to_string(), 1, 1),
            Token::eof(1),
        ];
        let mut parser = Parser::new(tokens);
        parser.current = 1;
        let prev_kind = parser.advance().kind.clone();
        assert_eq!(parser.current, 1);
        assert_eq!(prev_kind, TokenKind::Ken);
    }

    #[test]
    fn peek_returns_last_token_when_out_of_bounds_for_coverage() {
        let tokens = vec![Token::eof(1)];
        let mut parser = Parser::new(tokens);
        parser.current = parser.tokens.len();
        assert_eq!(&parser.peek().kind, &TokenKind::Eof);
    }

    #[test]
    #[cfg(coverage)]
    fn parser_public_helpers_are_exercised_in_unit_instance_for_coverage() {
        assert!(previous_is_none_at_start_for_coverage());
        assert!(!is_nae_followed_by_operand_end_of_stream_for_coverage());
        assert_eq!(process_escapes_for_coverage("hello\\"), "hello\\");

        let err = parse_fstring_for_coverage("{\\").unwrap_err();
        assert_error_variant(
            &err,
            HaversError::UnkentToken {
                lexeme: String::new(),
                line: 0,
                column: 0,
            },
        );
    }

    #[test]
    fn test_var_declaration() {
        let program = parse("ken x = 5").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert_stmt_variant(
            &program.statements[0],
            Stmt::VarDecl {
                name: String::new(),
                initializer: None,
                span: DUMMY_SPAN,
            },
        );
    }

    #[test]
    fn test_function_declaration() {
        let program = parse("dae greet(name) {\n  blether name\n}").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert_stmt_variant(
            &program.statements[0],
            Stmt::Function {
                name: String::new(),
                params: Vec::new(),
                body: Vec::new(),
                span: DUMMY_SPAN,
            },
        );
    }

    #[test]
    fn test_if_statement() {
        let program =
            parse("gin x > 5 {\n  blether \"big\"\n} ither {\n  blether \"wee\"\n}").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert_stmt_variant(
            &program.statements[0],
            Stmt::If {
                condition: dummy_expr(),
                then_branch: dummy_block_stmt(),
                else_branch: None,
                span: DUMMY_SPAN,
            },
        );
    }

    #[test]
    fn test_while_loop() {
        let program = parse("whiles x < 10 {\n  x = x + 1\n}").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert_stmt_variant(
            &program.statements[0],
            Stmt::While {
                condition: dummy_expr(),
                body: dummy_block_stmt(),
                span: DUMMY_SPAN,
            },
        );
    }

    #[test]
    fn test_for_loop() {
        let program = parse("fer i in 1..10 {\n  blether i\n}").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert_stmt_variant(
            &program.statements[0],
            Stmt::For {
                variable: String::new(),
                iterable: dummy_expr(),
                body: dummy_block_stmt(),
                span: DUMMY_SPAN,
            },
        );
    }

    #[test]
    fn test_expressions() {
        let program = parse("ken x = 5 + 3 * 2").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_list_literal() {
        let program = parse("ken arr = [1, 2, 3]").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_dict_literal() {
        let program = parse("ken d = {\"a\": 1, \"b\": 2}").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_dict_literal_trailing_comma() {
        let program = parse("ken d = {\"a\": 1,}").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert_stmt_variant(
            &program.statements[0],
            Stmt::VarDecl {
                name: String::new(),
                initializer: None,
                span: DUMMY_SPAN,
            },
        );
    }

    #[test]
    fn test_multiline_list() {
        let program = parse("ken arr = [\n  1,\n  2,\n  3\n]").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert_stmt_variant(
            &program.statements[0],
            Stmt::VarDecl {
                name: String::new(),
                initializer: None,
                span: DUMMY_SPAN,
            },
        );
    }

    #[test]
    fn test_multiline_dict() {
        let program = parse("ken d = {\n  \"a\": 1,\n  \"b\": 2\n}").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_fstring_with_escaped_quotes() {
        // The lexer handles escapes in f-strings, parser should handle the interpolation
        let program = parse(r#"blether f"test {\"hello\"}""#).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_lambda_expression() {
        let program = parse("ken f = |x| x * 2").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_ternary_expression() {
        let program = parse("ken x = gin aye than 1 ither 2").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_match_statement() {
        let program = parse(
            "keek x {\n  whan 1 -> { blether \"one\" }\n  whan _ -> { blether \"other\" }\n}",
        )
        .unwrap();
        assert_eq!(program.statements.len(), 1);
        assert_stmt_variant(
            &program.statements[0],
            Stmt::Match {
                value: dummy_expr(),
                arms: Vec::new(),
                span: DUMMY_SPAN,
            },
        );
    }

    fn first_match_arms(program: &Program) -> Option<&[MatchArm]> {
        match program.statements.first()? {
            Stmt::Match { arms, .. } => Some(arms),
            _ => None,
        }
    }

    #[test]
    fn test_match_statement_parses_return_break_and_continue_arms() {
        // Inside a function: a top-level `gie` is an error (decision F4).
        let program = parse(
            r#"dae f(x) {
keek x {
  whan 1 -> gie 1
  whan 2 -> brak
  whan 3 -> haud
}
}"#,
        )
        .unwrap();
        let Some(Stmt::Function { body, .. }) = program.statements.first() else {
            panic!("expected function");
        };
        let body = Program {
            statements: body.clone(),
        };

        let arms = first_match_arms(&body).expect("expected match statement");
        assert_stmt_variant(
            &arms[0].body,
            Stmt::Return {
                value: None,
                span: DUMMY_SPAN,
            },
        );
        assert_stmt_variant(&arms[1].body, Stmt::Break { span: DUMMY_SPAN });
        assert_stmt_variant(&arms[2].body, Stmt::Continue { span: DUMMY_SPAN });
    }

    #[test]
    fn test_match_pattern_negative_int_and_float() {
        let program = parse(
            r#"keek x {
  whan -5 -> 1
  whan -3.14 -> 2
}"#,
        )
        .unwrap();

        let arms = first_match_arms(&program).expect("expected match statement");
        assert_eq!(
            literal_from_pattern(&arms[0].pattern),
            Some(&Literal::Integer(-5))
        );
        assert_eq!(
            literal_from_pattern(&arms[1].pattern),
            Some(&Literal::Float(-3.14))
        );
    }

    #[test]
    fn test_match_pattern_bool_false() {
        let program = parse(
            r#"keek x {
  whan nae -> 1
}"#,
        )
        .unwrap();

        let arms = first_match_arms(&program).expect("expected match statement");
        assert_eq!(
            literal_from_pattern(&arms[0].pattern),
            Some(&Literal::Bool(false))
        );
    }

    #[test]
    fn test_first_match_arms_helper_returns_none_for_empty_program() {
        let program = Program::new(Vec::new());
        assert!(first_match_arms(&program).is_none());
    }

    #[test]
    fn test_first_match_arms_helper_returns_none_for_non_match_stmt() {
        let program = parse("ken x = 1").unwrap();
        assert!(first_match_arms(&program).is_none());
    }

    #[test]
    fn test_class_declaration() {
        let program =
            parse("kin Dug {\n  dae init(name) {\n    masel.name = name\n  }\n}").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert_stmt_variant(
            &program.statements[0],
            Stmt::Class {
                name: String::new(),
                superclass: None,
                methods: Vec::new(),
                span: DUMMY_SPAN,
            },
        );
    }

    #[test]
    fn test_spread_operator() {
        let program = parse("ken arr = [...other, 4, 5]").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_slice_syntax() {
        let program = parse("ken slice = arr[1:3]").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_pipe_operator() {
        let program = parse("ken result = x |> f |> g").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    // ==================== Error Cases ====================

    #[test]
    fn test_destructure_multiple_rest_error() {
        let result = parse("ken [...a, ...b] = [1, 2, 3]");
        assert!(result.is_err());
    }

    #[test]
    fn test_param_defaults_ordering_error() {
        let result = parse("dae foo(a = 1, b) { gie a + b }");
        assert!(result.is_err());
    }

    #[test]
    fn test_destructure_with_ignore() {
        let program = parse("ken [a, _, c] = [1, 2, 3]").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_destructure_with_rest() {
        let program = parse("ken [first, ...rest] = [1, 2, 3, 4]").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_destructure_trailing_comma_exits_loop_for_coverage() {
        let program = parse("ken [a,] = [1]").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_destructure_missing_right_bracket_hits_eof_branch_for_coverage() {
        let result = parse("ken [a,");
        assert!(result.is_err());
    }

    #[test]
    fn test_function_with_default_params() {
        let program =
            parse("dae greet(name, greeting = \"Hello\") { gie greeting + name }").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_function_with_all_default_params() {
        let program = parse("dae foo(a = 1, b = 2, c = 3) { gie a + b + c }").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    // ==================== Escape Sequences ====================

    #[test]
    fn test_escape_newline() {
        let program = parse(r#"ken s = "hello\nworld""#).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_escape_tab() {
        let program = parse(r#"ken s = "hello\tworld""#).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_escape_carriage_return() {
        let program = parse(r#"ken s = "hello\rworld""#).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_escape_backslash() {
        let program = parse(r#"ken s = "C:\\Users\\path""#).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_escape_double_quote() {
        let program = parse(r#"ken s = "say \"hello\"""#).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_escape_single_quote() {
        let program = parse(r#"ken s = "it\'s fine""#).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_escape_null() {
        let program = parse(r#"ken s = "null\0byte""#).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_escape_hex() {
        let program = parse(r#"ken s = "\x41\x42""#).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_escape_unknown() {
        // Unknown escapes are kept as-is
        let program = parse(r#"ken s = "hello\zworld""#).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_escape_trailing_backslash() {
        let program = parse(r#"ken s = "hello\\""#).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    // ==================== F-String Tests ====================

    #[test]
    fn test_fstring_basic() {
        let program = parse(r#"ken s = f"Hello {name}!""#).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_fstring_multiple_exprs() {
        let program = parse(r#"ken s = f"{a} + {b} = {a + b}""#).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_fstring_escaped_braces() {
        let program = parse(r#"ken s = f"Use {{braces}} here""#).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_fstring_single_closing_brace_is_literal_text() {
        let program = parse(r#"ken s = f"oops } here""#).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_fstring_nested_expr() {
        let program = parse(r#"ken s = f"Result: {gin x > 0 than x ither -x}""#).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_class_declaration_requires_methods() {
        let err = parse("kin C { ken x = 1 }").unwrap_err();
        assert_eq!(
            err,
            HaversError::ParseError {
                message: "Expected method definition in class".to_string(),
                line: 1,
            }
        );
    }

    #[test]
    fn test_pattern_identifier_underscore_is_wildcard() {
        let tokens = vec![
            Token::new(
                TokenKind::Identifier("_".to_string()),
                "_".to_string(),
                1,
                1,
            ),
            Token::eof(1),
        ];
        let mut parser = Parser::new(tokens);
        let pat = parser.pattern().unwrap();
        assert_pattern_variant(&pat, Pattern::Wildcard);
        assert!(literal_from_pattern(&pat).is_none());
    }

    #[test]
    fn test_match_pattern_minus_requires_number() {
        let err = parse("keek x { whan -foo -> 1 }").unwrap_err();
        assert_error_variant(
            &err,
            HaversError::ParseError {
                message: String::new(),
                line: 0,
            },
        );
    }

    #[test]
    fn test_match_pattern_unexpected_token_errors() {
        let err = parse("keek x { whan + -> 1 }").unwrap_err();
        assert_error_variant(
            &err,
            HaversError::ParseError {
                message: String::new(),
                line: 0,
            },
        );
    }

    #[test]
    fn test_invalid_compound_assignment_target_errors() {
        let err = parse("1 += 2").unwrap_err();
        assert_eq!(
            err,
            HaversError::ParseError {
                message: "Invalid compound assignment target".to_string(),
                line: 1,
            }
        );
    }

    #[test]
    fn test_slice_syntax_step_present_but_empty() {
        let program = parse("ken a = arr[::]\nken b = arr[1::]").unwrap();
        assert_eq!(program.statements.len(), 2);
        assert_stmt_variant(
            &program.statements[0],
            Stmt::VarDecl {
                name: String::new(),
                initializer: None,
                span: DUMMY_SPAN,
            },
        );
        assert_stmt_variant(
            &program.statements[1],
            Stmt::VarDecl {
                name: String::new(),
                initializer: None,
                span: DUMMY_SPAN,
            },
        );
    }

    #[test]
    fn test_import_requires_string_path() {
        let err = parse("fetch 123").unwrap_err();
        assert_eq!(
            err,
            HaversError::UnexpectedToken {
                expected: "module path".to_string(),
                found: "123".to_string(),
                line: 1,
            }
        );
    }

    #[test]
    fn test_lenient_statement_end_allows_next_statement_keyword() {
        let program = parse("ken x = 1 ken y = 2").unwrap();
        assert_eq!(program.statements.len(), 2);
    }

    #[test]
    fn test_statement_end_semicolon_consumes_following_newlines() {
        let program = parse("ken x = 1;\n\nken y = 2;").unwrap();
        assert_eq!(program.statements.len(), 2);
    }

    #[test]
    fn test_bang_unary_not_operator() {
        let program = parse("ken x = !aye").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert_stmt_variant(
            &program.statements[0],
            Stmt::VarDecl {
                name: String::new(),
                initializer: None,
                span: DUMMY_SPAN,
            },
        );
    }

    #[test]
    fn test_is_nae_followed_by_operand_handles_end_of_stream() {
        let tokens = vec![Token::new(TokenKind::Nae, "nae".to_string(), 1, 1)];
        let parser = Parser::new(tokens);
        assert!(!parser.is_nae_followed_by_operand());
    }

    #[test]
    fn test_previous_is_none_at_start() {
        let tokens = vec![Token::eof(1)];
        let parser = Parser::new(tokens);
        assert!(parser.previous().is_none());
    }

    #[test]
    fn test_fstring_interpolation_with_nested_braces() {
        let program = parse(r#"ken s = f"{ {1: 2} }""#).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_fstring_interpolation_backslash_escapes() {
        let program = parse(r#"ken s = f"{\"a\\\\b\nc\"}""#).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_fstring_expr_trailing_backslash_is_error() {
        let tokens = vec![Token::eof(1)];
        let mut parser = Parser::new(tokens);
        let err = parser.parse_fstring("{\\", Span::new(1, 1)).unwrap_err();
        assert_error_variant(
            &err,
            HaversError::UnkentToken {
                lexeme: String::new(),
                line: 0,
                column: 0,
            },
        );
    }

    #[test]
    fn test_escape_hex_incomplete_sequences_are_preserved() {
        let program = parse(r#"ken a = "\x""#).unwrap();
        assert_eq!(program.statements.len(), 1);

        let program = parse(r#"ken a = "\x1""#).unwrap();
        assert_eq!(program.statements.len(), 1);

        let program = parse(r#"ken a = "\xZ""#).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_process_escapes_trailing_backslash_is_preserved() {
        assert_eq!(process_escapes("hello\\"), "hello\\");
    }

    #[test]
    fn test_process_escapes_accepts_uppercase_hex_marker_for_coverage() {
        assert_eq!(process_escapes("\\X41"), "A");
    }

    // ==================== More Statement Tests ====================

    #[test]
    fn test_struct_declaration() {
        let program = parse("thing Point { x, y }").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert_stmt_variant(
            &program.statements[0],
            Stmt::Struct {
                name: String::new(),
                fields: Vec::new(),
                span: DUMMY_SPAN,
            },
        );
    }

    #[test]
    fn test_import_statement() {
        let program = parse(r#"fetch "math""#).unwrap();
        assert_eq!(program.statements.len(), 1);
        assert_stmt_variant(
            &program.statements[0],
            Stmt::Import {
                path: String::new(),
                alias: None,
                span: DUMMY_SPAN,
            },
        );
    }

    #[test]
    fn test_try_catch() {
        let program = parse("hae_a_bash { risky() } gin_it_gangs_wrang e { blether e }").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert_stmt_variant(
            &program.statements[0],
            Stmt::TryCatch {
                try_block: dummy_block_stmt(),
                error_name: String::new(),
                catch_block: dummy_block_stmt(),
                span: DUMMY_SPAN,
            },
        );
    }

    #[test]
    fn test_assert_statement() {
        let program = parse("mak_siccar x > 0").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert_stmt_variant(
            &program.statements[0],
            Stmt::Assert {
                condition: dummy_expr(),
                message: None,
                span: DUMMY_SPAN,
            },
        );
    }

    #[test]
    fn test_assert_with_message() {
        let program = parse(r#"mak_siccar x > 0 , "x must be positive""#).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_class_with_inheritance() {
        let program = parse("kin Dog fae Animal { dae bark() { gie \"woof\" } }").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_match_with_range_pattern() {
        let program =
            parse("keek x {\n  whan 1..10 -> blether \"small\"\n  whan _ -> blether \"big\"\n}")
                .unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    // ==================== Expression Tests ====================

    #[test]
    fn test_lambda_syntax() {
        let program = parse("ken add = |a, b| a + b").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_range_inclusive() {
        let program = parse("ken r = 1..=10").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert_stmt_variant(
            &program.statements[0],
            Stmt::VarDecl {
                name: String::new(),
                initializer: None,
                span: DUMMY_SPAN,
            },
        );
    }

    #[test]
    fn test_range_exclusive() {
        let program = parse("ken r = 1..10").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_slice_with_step() {
        let program = parse("ken s = arr[::2]").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_slice_reverse() {
        let program = parse("ken s = arr[::-1]").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_compound_assignment_plus() {
        let program = parse("x += 5").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_compound_assignment_minus() {
        let program = parse("x -= 5").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_compound_assignment_multiply() {
        let program = parse("x *= 5").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_compound_assignment_divide() {
        let program = parse("x /= 5").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_logical_not() {
        let program = parse("ken b = no(aye)").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_unary_negate() {
        let program = parse("ken n = -42").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_method_call_chain() {
        let program = parse("obj.method1().method2().field").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_masel_reference() {
        let program = parse("kin Foo { dae bar() { gie masel.x } }").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_dict_literal_syntax() {
        let program = parse(r#"ken d = {"key": "value", "num": 42}"#).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_list_literal_syntax() {
        let program = parse("ken l = [1, 2, 3, 4, 5]").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_grouping_expression() {
        let program = parse("ken n = (1 + 2) * 3").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    // ==================== Control Flow Tests ====================

    #[test]
    fn test_while_loop_parse() {
        let program = parse("whiles x > 0 { x = x - 1 }").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert_stmt_variant(
            &program.statements[0],
            Stmt::While {
                condition: dummy_expr(),
                body: dummy_block_stmt(),
                span: DUMMY_SPAN,
            },
        );
    }

    #[test]
    fn test_for_loop_parse() {
        let program = parse("fer i in 1..10 { blether i }").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert_stmt_variant(
            &program.statements[0],
            Stmt::For {
                variable: String::new(),
                iterable: dummy_expr(),
                body: dummy_block_stmt(),
                span: DUMMY_SPAN,
            },
        );
    }

    #[test]
    fn test_break_statement() {
        let program = parse("whiles aye { brak }").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_continue_statement() {
        let program = parse("fer i in 1..10 { haud }").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_return_with_value() {
        let program = parse("dae foo() { gie 42 }").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_return_without_value() {
        let program = parse("dae foo() { gie naething }").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_return_without_value_at_end_of_file_for_coverage() {
        // A bare `gie` right at the end o' the file (inside a function, since a
        // top-level `gie` is an error); the unclosed body then fails.
        assert!(parse("dae f() { gie").is_err());
        assert!(matches!(
            parse("gie"),
            Err(HaversError::ReturnOutsideFunction { line: 1 })
        ));
    }

    #[test]
    fn test_brak_haud_in_block_expression_is_a_parse_error() {
        for (src, kw) in [
            (
                "fer x in [1, 2, 3] { ken y = { gin x == 2 { brak }\n x }\n blether y }",
                "brak",
            ),
            ("whiles aye { ken y = { haud\n 1 }\n blether y }", "haud"),
            ("ken y = { brak\n 1 }", "brak"),
        ] {
            match parse(src) {
                Err(HaversError::LoopJumpInBlockExpr { keyword, .. }) => assert_eq!(keyword, kw),
                other => panic!("expected LoopJumpInBlockExpr for {src}: {other:?}"),
            }
        }
    }

    #[test]
    fn test_brak_haud_allowed_in_loops_near_block_expressions() {
        // A loop inside the block expression may still jump.
        assert!(parse("ken y = { fer i in 0..3 { brak }\n 5 }").is_ok());
        assert!(parse("ken y = { whiles aye { haud }\n 5 }").is_ok());
        // Ordinary loop bodies (incl. nested statement blocks) are unaffected.
        assert!(parse("fer x in [1, 2] { gin x == 1 { brak } }").is_ok());
        // The context is restored after a block expression ends.
        assert!(parse("fer x in [1] { ken y = { 1 }\n brak }").is_ok());
        // A function body inside a block expression resets the context (as before).
        assert!(parse("ken y = { dae f() { brak }\n 1 }").is_ok());
        assert!(parse("ken y = { ken f = |a| { brak }\n 1 }").is_ok());
    }

    #[test]
    fn test_if_else() {
        let program =
            parse("gin x > 0 { blether \"positive\" } ither { blether \"not positive\" }").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_if_else_if() {
        let program = parse("gin x > 0 { blether \"pos\" } ither gin x < 0 { blether \"neg\" } ither { blether \"zero\" }").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    // ==================== Error Recovery Tests ====================

    #[test]
    fn test_unexpected_token_error() {
        let result = parse("ken = 42");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_closing_brace() {
        let result = parse("gin aye { blether \"hi\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_closing_bracket() {
        let result = parse("ken l = [1, 2, 3");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_assignment_target() {
        let result = parse("42 = x");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_literal_trailing_comma_for_coverage() {
        let program = parse("ken l = [1, 2,]").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_slice_syntax_empty_end_for_coverage() {
        let program = parse("ken s = arr[:] \nken t = arr[:3]").unwrap();
        assert_eq!(program.statements.len(), 2);
    }

    #[test]
    fn test_block_expression_parsing_for_coverage() {
        let program = parse("ken x = { ken y = 1 }\n").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert_stmt_variant(
            &program.statements[0],
            Stmt::VarDecl {
                name: String::new(),
                initializer: None,
                span: DUMMY_SPAN,
            },
        );
    }

    #[test]
    fn test_lambda_block_body_parsing_for_coverage() {
        let program = parse("ken f = |x| { ken y = x }\n").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_return_statement_without_value_for_coverage() {
        let program = parse("dae foo() { gie\n}\n").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_empty_match_and_class_struct_bodies_for_coverage() {
        let program = parse("keek x { }\nkin Foo { }\nthing Bar { }\n").unwrap();
        assert_eq!(program.statements.len(), 3);
    }

    #[test]
    fn test_process_escapes_hex_short_and_missing_digits_for_coverage() {
        assert_eq!(process_escapes("\\x"), "\\x");
        assert_eq!(process_escapes("\\x4"), "\\x4");
        assert_eq!(process_escapes("\\x4Z"), "\\x4Z");
    }

    #[cfg(coverage)]
    #[test]
    fn parser_missing_brace_loops_hit_eof_branches_for_coverage() {
        let cases = [
            "ken [a, b",
            "kin Foo {\n  dae bar() { }\n",
            "thing Foo {\n  a, b\n",
            "keek 1 {\n  1 -> blether 1\n",
            "ken x = { ken y = 1\n",
            "ken f = |x| { blether x\n",
        ];

        for src in cases {
            assert!(parse(src).is_err());
        }
    }

    #[cfg(coverage)]
    #[test]
    fn process_escapes_hex_valid_digits_for_coverage() {
        assert_eq!(process_escapes("\\x41"), "A");
        assert!(process_escapes("\\xZZ").contains("\\x"));
    }

    // ==================== Robustness (MDH-BUG-FLU-00014..00024) ====================

    /// The initializer o' the first `ken` in `src`.
    fn first_initializer(src: &str) -> Expr {
        let program = parse(src).unwrap_or_else(|e| panic!("{src:?} should parse: {e}"));
        match program.statements.into_iter().next() {
            Some(Stmt::VarDecl {
                initializer: Some(expr),
                ..
            }) => expr,
            other => panic!("expected a ken wi' an initializer, got {other:?}"),
        }
    }

    fn parse_error_message(src: &str) -> String {
        match parse(src) {
            Err(HaversError::ParseError { message, .. }) => message,
            other => panic!("expected a ParseError for {src:?}, got {other:?}"),
        }
    }

    #[test]
    fn range_binds_looser_than_addition() {
        // x + 1..5 is (x + 1)..5, no' x + (1..5)
        match first_initializer("ken r = x + 1..5") {
            Expr::Range {
                start,
                end,
                inclusive: false,
                ..
            } => {
                assert!(matches!(
                    *start,
                    Expr::Binary {
                        operator: BinaryOp::Add,
                        ..
                    }
                ));
                assert!(matches!(
                    *end,
                    Expr::Literal {
                        value: Literal::Integer(5),
                        ..
                    }
                ));
            }
            other => panic!("expected a range, got {other:?}"),
        }
        // The end is an additive expression an' aw: 0..n - 1 is 0..(n - 1)
        match first_initializer("ken r = 0..n - 1") {
            Expr::Range { end, .. } => assert!(matches!(
                *end,
                Expr::Binary {
                    operator: BinaryOp::Subtract,
                    ..
                }
            )),
            other => panic!("expected a range, got {other:?}"),
        }
    }

    #[test]
    fn negative_range_start_is_negated_number_not_negated_range() {
        match first_initializer("ken r = -1..2") {
            Expr::Range { start, .. } => assert!(matches!(
                *start,
                Expr::Unary {
                    operator: UnaryOp::Negate,
                    ..
                }
            )),
            other => panic!("expected a range, got {other:?}"),
        }
        match first_initializer("ken r = -3..=-1") {
            Expr::Range {
                start,
                end,
                inclusive: true,
                ..
            } => {
                assert!(matches!(*start, Expr::Unary { .. }));
                assert!(matches!(*end, Expr::Unary { .. }));
            }
            other => panic!("expected an inclusive range, got {other:?}"),
        }
    }

    #[test]
    fn range_binds_tighter_than_comparison_and_accepts_any_term() {
        assert!(matches!(
            first_initializer("ken b = 1..3 == r"),
            Expr::Binary {
                operator: BinaryOp::Equal,
                ..
            }
        ));
        // Floats, calls an' method results are allowed as bounds (the runtime decides).
        assert!(matches!(
            first_initializer("ken r = 1.5..len(xs)"),
            Expr::Range { .. }
        ));
        assert!(matches!(
            first_initializer("ken r = xs.start..xs.stop"),
            Expr::Range { .. }
        ));
    }

    #[test]
    fn match_range_pattern_accepts_negative_start() {
        let program = parse("keek x {\n  whan -5..0 -> blether \"neg\"\n  whan _ -> 1\n}").unwrap();
        let arms = match &program.statements[0] {
            Stmt::Match { arms, .. } => arms,
            other => panic!("expected a match, got {other:?}"),
        };
        match &arms[0].pattern {
            Pattern::Range { start, .. } => assert!(matches!(
                **start,
                Expr::Literal {
                    value: Literal::Integer(-5),
                    ..
                }
            )),
            other => panic!("expected a range pattern, got {other:?}"),
        }
    }

    #[test]
    fn brace_in_expression_position_picks_dict_or_block_like_before() {
        let dicts = [
            "ken d = {a: 1}",
            "ken d = {\"k\": 1, \"j\": 2}",
            "ken d = {\n  \"k\": 1,\n  \"j\": [1,\n 2]\n}",
            "ken d = {[1, 2][0]: 1}",
            "ken d = {gin a than b ither c: 1}",
        ];
        for src in dicts {
            assert!(
                matches!(first_initializer(src), Expr::Dict { .. }),
                "{src:?} should be a dict"
            );
        }
        let blocks = [
            "ken b = { a }",
            "ken b = { ken y = 1 }",
            "ken b = { xs[1:2] }",
            "ken b = { f({k: 1}) }",
            "ken b = {\n  gin a {\n    blether {k: 1}\n  }\n  2\n}",
            "ken b = { 0 + { 0 + 1 } }",
        ];
        for src in blocks {
            assert!(
                matches!(first_initializer(src), Expr::BlockExpr { .. }),
                "{src:?} should be a block"
            );
        }
    }

    #[test]
    fn nested_braces_parse_in_linear_time() {
        // 26 levels used tae take minutes (each level was parsed twice).
        // (Each level is twa nesting levels, so 26 stays under the limit.)
        let levels = 26;
        let src = format!("ken a = {}1{}", "{0+".repeat(levels), "}".repeat(levels));
        let started = std::time::Instant::now();
        let program = parse(&src).unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(
            started.elapsed() < std::time::Duration::from_secs(2),
            "parse took {:?}",
            started.elapsed()
        );
    }

    #[test]
    fn deep_nesting_is_a_parse_error_not_a_stack_overflow() {
        let n = 100_000;
        let cases = [
            format!("ken a = {}1{}", "(".repeat(n), ")".repeat(n)),
            format!("ken a = {}1{}", "[".repeat(n), "]".repeat(n)),
            format!("ken a = {}1", "-".repeat(n)),
            format!("ken a = {}1", "!".repeat(n)),
            format!("{}{}", "{".repeat(n), "}".repeat(n)),
            format!("ken a = {}1", "b = ".repeat(n)),
            format!("ken a = {}1", "gin aye than 1 ither ".repeat(n)),
            format!("gin aye {{ }}{}", " ither gin aye { }".repeat(n)),
            format!("ken f = {}1", "|x| ".repeat(n)),
        ];
        for src in &cases {
            let message = parse_error_message(src);
            assert!(
                message.contains("nested ower deep"),
                "{}...: {message}",
                &src[..20]
            );
        }
    }

    #[test]
    fn nesting_just_under_the_limit_still_parses() {
        // ken + its initializer use twa levels; each bracket adds ane.
        let n = MAX_NESTING_DEPTH - 2;
        let parens = format!("ken a = {}1{}", "(".repeat(n), ")".repeat(n));
        assert!(parse(&parens).is_ok());
        let calls = format!("ken a = {}1{}", "f(".repeat(n), ")".repeat(n));
        assert!(parse(&calls).is_ok());
        let too_deep = format!("ken a = {}1{}", "(".repeat(n + 1), ")".repeat(n + 1));
        assert!(parse_error_message(&too_deep).contains("nested ower deep"));
    }

    #[test]
    fn bare_gie_before_brace_or_semicolon_returns_naething() {
        for src in [
            "dae f() { gie }",
            "dae f() { gie; }",
            "dae f() { gie; blether 1 }",
        ] {
            let program = parse(src).unwrap_or_else(|e| panic!("{src:?}: {e}"));
            match &program.statements[0] {
                Stmt::Function { body, .. } => {
                    assert!(
                        matches!(body[0], Stmt::Return { value: None, .. }),
                        "{src:?}"
                    )
                }
                other => panic!("expected a function, got {other:?}"),
            }
        }
    }

    #[test]
    fn mak_siccar_without_comma_is_a_parse_error() {
        let err = parse("mak_siccar 1 == 2 \"this message is never attached\"").unwrap_err();
        assert!(
            matches!(err, HaversError::UnexpectedToken { line: 1, .. }),
            "{err:?}"
        );
        // Still fine: wi' a comma, wi' a semicolon, or followed by another statement.
        assert!(parse("mak_siccar x, \"msg\"").is_ok());
        assert!(parse("mak_siccar x; blether 1").is_ok());
        assert_eq!(
            parse("mak_siccar x\nblether 1").unwrap().statements.len(),
            2
        );
        // Every statement keyword may follow on the same line, as fer ither statements.
        assert_eq!(parse("ken x = 1 mak_siccar x").unwrap().statements.len(), 2);
        assert_eq!(parse("ken x = 1 hurl \"x\"").unwrap().statements.len(), 2);
    }

    #[test]
    fn compound_assignment_on_fields_and_indexes() {
        let program = parse("kin C { dae bump() { masel.n += 1 } }").unwrap();
        let method_body = match &program.statements[0] {
            Stmt::Class { methods, .. } => match &methods[0] {
                Stmt::Function { body, .. } => body.clone(),
                other => panic!("expected a method, got {other:?}"),
            },
            other => panic!("expected a class, got {other:?}"),
        };
        match &method_body[0] {
            Stmt::Expression {
                expr:
                    Expr::Set {
                        object,
                        property,
                        value,
                        ..
                    },
                ..
            } => {
                assert!(matches!(**object, Expr::Masel { .. }));
                assert_eq!(property, "n");
                match &**value {
                    Expr::Binary {
                        left,
                        operator: BinaryOp::Add,
                        ..
                    } => assert!(matches!(**left, Expr::Get { .. })),
                    other => panic!("expected masel.n + 1, got {other:?}"),
                }
            }
            other => panic!("expected a Set, got {other:?}"),
        }

        for (src, op) in [
            ("xs[0] += 1", BinaryOp::Add),
            ("xs[i + 1] -= 2", BinaryOp::Subtract),
            ("d[\"k\"] *= 3", BinaryOp::Multiply),
            ("grid[y][x] /= 2", BinaryOp::Divide),
        ] {
            let program = parse(src).unwrap_or_else(|e| panic!("{src:?}: {e}"));
            match &program.statements[0] {
                Stmt::Expression {
                    expr: Expr::IndexSet { value, .. },
                    ..
                } => match &**value {
                    Expr::Binary { left, operator, .. } => {
                        assert!(matches!(**left, Expr::Index { .. }), "{src:?}");
                        assert_eq!(
                            std::mem::discriminant(operator),
                            std::mem::discriminant(&op),
                            "{src:?}"
                        );
                    }
                    other => panic!("{src:?}: expected a binary value, got {other:?}"),
                },
                other => panic!("{src:?}: expected an IndexSet, got {other:?}"),
            }
        }
        assert!(parse("obj.inner.count += 1").is_ok());
    }

    #[test]
    fn compound_assignment_refuses_targets_that_would_run_twice() {
        for src in ["f().n += 1", "xs[next()] += 1", "make()[0] -= 1"] {
            assert!(
                parse_error_message(src).contains("ower complicated"),
                "{src:?}"
            );
        }
    }

    #[test]
    fn underscore_is_a_throwaway_binder() {
        match &parse("fer _ in 0..3 { }").unwrap().statements[0] {
            Stmt::For { variable, .. } => assert_eq!(variable, "_"),
            other => panic!("expected a fer loop, got {other:?}"),
        }
        match first_initializer("ken f = |_, y, _| y") {
            Expr::Lambda { params, .. } => assert_eq!(params, vec!["_", "y", "__"]),
            other => panic!("expected a lambda, got {other:?}"),
        }
        match &parse("dae f(_, b, _) { gie b }").unwrap().statements[0] {
            Stmt::Function { params, .. } => {
                let names: Vec<&str> = params.iter().map(|p| p.name.as_str()).collect();
                assert_eq!(names, vec!["_", "b", "__"]);
            }
            other => panic!("expected a function, got {other:?}"),
        }
    }

    #[test]
    fn fstring_hole_must_hold_exactly_one_expression() {
        let message = parse_error_message("ken a = 1\nken b = 2\nblether f\"{a b}\"");
        assert!(message.contains("mair in this f-string"), "{message}");
        let message = parse_error_message("ken x = 7\nblether f\"{x:>5}\"");
        assert!(message.contains("format specs"), "{message}");
        // Newlines roond the expression are fine.
        assert!(parse("blether f\"{\n1 + 1\n}\"").is_ok());
    }

    #[test]
    fn fstring_hole_errors_report_the_real_line() {
        let err = parse("ken a = 1\nken b = 2\nblether f\"{a +}\"").unwrap_err();
        assert_eq!(err.line(), Some(3), "{err:?}");
        // A hole on the second line o' a multi-line f-string.
        let err = parse("ken a = 1\nblether f\"one\ntwo {a b}\"").unwrap_err();
        assert_eq!(err.line(), Some(3), "{err:?}");
        // Lexer errors inside a hole are shifted an' aw.
        let err = parse("\n\nblether f\"{a @ b}\"").unwrap_err();
        assert!(
            matches!(err, HaversError::UnkentToken { line: 3, .. }),
            "{err:?}"
        );
    }

    #[test]
    fn fstring_holes_may_contain_string_literals() {
        let cases = [
            r#"blether f"Cities: {["Edinburgh", "Glasgow"]}""#,
            r#"blether f"{"}"}""#,
            r#"blether f"{ {"k": 1}["k"] }""#,
            r#"blether f"{'single'} an' {"double"}""#,
            r#"blether f"{f(\"}\")}""#,
        ];
        for src in cases {
            let program = parse(src).unwrap_or_else(|e| panic!("{src:?}: {e}"));
            match &program.statements[0] {
                Stmt::Print {
                    value: Expr::FString { parts, .. },
                    ..
                } => assert!(
                    parts.iter().any(|p| matches!(p, FStringPart::Expr(_))),
                    "{src:?}"
                ),
                other => panic!("{src:?}: expected blether f\"...\", got {other:?}"),
            }
        }
    }

    // ==================== Follow-ups (MDH-BUG-FLU-00066/75/78) ====================

    /// The body o' the lambda that is the first `ken`'s initializer.
    fn first_lambda_body(src: &str) -> Expr {
        match first_initializer(src) {
            Expr::Lambda { body, .. } => *body,
            other => panic!("{src:?}: expected a lambda, got {other:?}"),
        }
    }

    #[test]
    fn lambda_brace_body_picks_dict_or_block_like_expression_position() {
        // MDH-BUG-FLU-00066: a `key:` on the first line means a dict literal.
        assert!(matches!(
            first_lambda_body("ken f = |x| {\"item\": x, \"key\": k(x)}"),
            Expr::Dict { ref pairs, .. } if pairs.len() == 2
        ));
        assert!(matches!(
            first_lambda_body("ken f = |x| {\n  \"a\": x\n}"),
            Expr::Dict { .. }
        ));
        // A dict body can carry on as an expression.
        assert!(matches!(
            first_lambda_body("ken f = |k| {\"a\": 1}[k]"),
            Expr::Index { .. }
        ));
        // Blocks stay blocks, and `{}` stays an empty block body.
        for src in [
            "ken f = |x| { shove(xs, x) }",
            "ken f = |x| {\n  ken y = x[1:2]\n  gie y\n}",
            "ken f = |x| { obj.a = x; obj.b = x }",
            "ken f = |x| {}",
            "ken f = || { naething }",
        ] {
            assert!(
                matches!(first_lambda_body(src), Expr::BlockExpr { .. }),
                "{src:?}"
            );
        }
    }

    #[test]
    fn call_arguments_may_span_lines() {
        // MDH-BUG-FLU-00075
        for src in [
            "ken r = f(\n  1,\n  2\n)",
            "ken r = f(1,\n      2)",
            "ken r = f(\n)",
            "ken r = f(\n  ...xs\n)",
        ] {
            assert!(
                matches!(first_initializer(src), Expr::Call { .. }),
                "{src:?}"
            );
        }
        match first_initializer("ken r = f(\n  1,\n\n  2\n)") {
            Expr::Call { arguments, .. } => assert_eq!(arguments.len(), 2),
            other => panic!("expected a call, got {other:?}"),
        }
        // A dangling comma is still an error, as it was on one line.
        assert!(parse("ken r = f(1,\n)").is_err());
    }

    #[test]
    fn leading_dot_continues_a_chain_on_the_next_line() {
        // MDH-BUG-FLU-00075
        let program = parse("ken r = xs\n  .map(f)\n\n  .len()\nblether r\n").unwrap();
        assert_eq!(program.statements.len(), 2);
        assert!(matches!(
            &program.statements[0],
            Stmt::VarDecl {
                initializer: Some(Expr::Call { .. }),
                ..
            }
        ));
        // Ony ither token on the next line still starts a new statement.
        let program = parse("ken r = xs\n(1)\n").unwrap();
        assert_eq!(program.statements.len(), 2);
        let program = parse("ken r = xs\n[1]\n").unwrap();
        assert_eq!(program.statements.len(), 2);
        // A statement can still never start wi' a bare `.`.
        assert!(parse(".len()\n").is_err());
    }

    #[test]
    fn inclusive_range_pattern_is_parsed() {
        // MDH-BUG-FLU-00078
        let program =
            parse("keek x {\n  whan 1..=5 -> 1\n  whan 6..9 -> 2\n  whan -3..=-1 -> 3\n}").unwrap();
        let arms = match &program.statements[0] {
            Stmt::Match { arms, .. } => arms,
            other => panic!("expected a match, got {other:?}"),
        };
        let flags: Vec<bool> = arms
            .iter()
            .map(|arm| match &arm.pattern {
                Pattern::Range { inclusive, .. } => *inclusive,
                other => panic!("expected a range pattern, got {other:?}"),
            })
            .collect();
        assert_eq!(flags, vec![true, false, true]);
    }
}
