use crate::ast::{LogLevel, *};
use crate::error::{HaversError, HaversResult};
use crate::token::{Token, TokenKind};

/// The parser - turns tokens intae an AST
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
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
        let span = self.current_span();
        self.advance(); // consume 'dae'

        let name = self.expect_identifier("function name")?;
        self.expect(&TokenKind::LeftParen, "(")?;

        let mut params = Vec::new();
        let mut seen_default = false;

        if !self.check(&TokenKind::RightParen) {
            loop {
                let param_name = self.expect_identifier("parameter name")?;

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
                Some(Box::new(self.if_statement()?))
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
        let body = Box::new(self.block()?);

        Ok(Stmt::While {
            condition,
            body,
            span,
        })
    }

    fn for_statement(&mut self) -> HaversResult<Stmt> {
        let span = self.current_span();
        self.advance(); // consume 'fer'

        let variable = self.expect_identifier("loop variable")?;
        self.expect(&TokenKind::In, "in")?;
        let iterable = self.expression()?;
        self.skip_newlines();
        let body = Box::new(self.block()?);

        Ok(Stmt::For {
            variable,
            iterable,
            body,
            span,
        })
    }

    fn return_statement(&mut self) -> HaversResult<Stmt> {
        let span = self.current_span();
        self.advance(); // consume 'gie'

        let value = if self.check(&TokenKind::Newline) || self.check(&TokenKind::Eof) {
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
        self.advance(); // consume 'brak'
        self.expect_statement_end()?;
        Ok(Stmt::Break { span })
    }

    fn continue_statement(&mut self) -> HaversResult<Stmt> {
        let span = self.current_span();
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
                if self.match_token(&TokenKind::DotDot) {
                    let end = self.expression()?;
                    Ok(Pattern::Range {
                        start: Box::new(Expr::Literal {
                            value: Literal::Integer(n),
                            span: Span::new(token.line, token.column),
                        }),
                        end: Box::new(end),
                    })
                } else {
                    Ok(Pattern::Literal(Literal::Integer(n)))
                }
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
                        Ok(Pattern::Literal(Literal::Integer(n)))
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
        self.assignment()
    }

    fn assignment(&mut self) -> HaversResult<Expr> {
        let expr = self.pipe_expr()?;

        if self.match_token(&TokenKind::Equals) {
            let span = self.current_span();
            let value = self.assignment()?;

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
            let span = expr.span();
            let value = self.assignment()?;

            match expr {
                Expr::Variable { name, .. } => {
                    return Ok(Expr::Assign {
                        name: name.clone(),
                        value: Box::new(Expr::Binary {
                            left: Box::new(Expr::Variable { name, span }),
                            operator: op,
                            right: Box::new(value),
                            span,
                        }),
                        span,
                    });
                }
                _ => {
                    return Err(HaversError::ParseError {
                        message: "Invalid compound assignment target".to_string(),
                        line: span.line,
                    });
                }
            }
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
            let else_expr = self.ternary()?; // Right-associative

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
        let mut expr = self.term()?;

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
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
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
            let span = self
                .previous()
                .map(|t| Span::new(t.line, t.column))
                .unwrap_or(self.current_span());
            let operand = self.unary()?;
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
                let span = self
                    .previous()
                    .map(|t| Span::new(t.line, t.column))
                    .unwrap_or(self.current_span());
                let operand = self.unary()?;
                return Ok(Expr::Unary {
                    operator: UnaryOp::Not,
                    operand: Box::new(operand),
                    span,
                });
            }
            // Otherwise, let it be parsed as a literal in primary()
        }

        if self.match_token(&TokenKind::Bang) {
            let span = self
                .previous()
                .map(|t| Span::new(t.line, t.column))
                .unwrap_or(self.current_span());
            let operand = self.unary()?;
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
                let span = self.current_span();

                // Check fer slice syntax: [start:end:step], [:end], [start:], [::step], etc.
                if self.check(&TokenKind::Colon) {
                    // [:end] or [:] or [:end:step] or [::step]
                    self.advance(); // consume the first colon

                    let end =
                        if self.check(&TokenKind::Colon) || self.check(&TokenKind::RightBracket) {
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
                        let end = if self.check(&TokenKind::Colon)
                            || self.check(&TokenKind::RightBracket)
                        {
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
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> HaversResult<Expr> {
        let span = callee.span();
        let mut arguments = Vec::new();

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
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
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
                self.maybe_range(Expr::Literal {
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
                let expr = Expr::Variable { name, span };
                self.maybe_range(expr)
            }
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.expect(&TokenKind::RightParen, ")")?;
                // Check for range after grouping: (x+1)..10
                self.maybe_range(Expr::Grouping {
                    expr: Box::new(expr),
                    span,
                })
            }
            TokenKind::LeftBracket => {
                self.advance();
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
            TokenKind::LeftBrace => {
                self.advance(); // consume '{'
                self.skip_newlines();

                // Empty dict literal: {}
                if self.check(&TokenKind::RightBrace) {
                    self.advance(); // consume '}'
                    return Ok(Expr::Dict {
                        pairs: Vec::new(),
                        span,
                    });
                }

                // If it looks like a statement, parse as block expression.
                // Otherwise, try dict-first and fall back to block expr if no ':' appears.
                let checkpoint = self.current;
                let key_attempt = self.expression();
                let is_dict = key_attempt.is_ok() && self.check(&TokenKind::Colon);

                if !is_dict {
                    // Rewind and parse block expression statements until '}'
                    self.current = checkpoint;
                    let mut statements = Vec::new();
                    self.skip_newlines();
                    while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
                        statements.push(self.declaration()?);
                        self.skip_newlines();
                    }
                    self.expect(&TokenKind::RightBrace, "}")?;
                    return Ok(Expr::BlockExpr { statements, span });
                }

                // Dict literal
                let mut pairs = Vec::new();

                // Safe because is_dict implies key_attempt is Ok
                let first_key = key_attempt.expect("dict key already parsed");
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
            // Lambda expressions: |x, y| x + y  or  |x, y| { statements... }
            TokenKind::Pipe => {
                self.advance();
                let mut params = Vec::new();
                if !self.check(&TokenKind::Pipe) {
                    loop {
                        params.push(self.expect_identifier("parameter name")?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(&TokenKind::Pipe, "|")?;
                // Check for block body: |x, y| { ... }
                let body = if self.check(&TokenKind::LeftBrace) {
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
            _ => Err(HaversError::ParseError {
                message: format!("Unexpected token: {}", token.kind),
                line: token.line,
            }),
        }
    }

    fn maybe_range(&mut self, start_expr: Expr) -> HaversResult<Expr> {
        if self.match_token(&TokenKind::DotDotEquals) {
            let span = start_expr.span();
            let end = self.term()?;
            Ok(Expr::Range {
                start: Box::new(start_expr),
                end: Box::new(end),
                inclusive: true,
                span,
            })
        } else if self.match_token(&TokenKind::DotDot) {
            let span = start_expr.span();
            let end = self.term()?;
            Ok(Expr::Range {
                start: Box::new(start_expr),
                end: Box::new(end),
                inclusive: false,
                span,
            })
        } else {
            Ok(start_expr)
        }
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

        // Be lenient - if the next token starts a new statement, that's fine
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
        let mut chars = content.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '{' {
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
                while let Some(c) = chars.next() {
                    if c == '{' {
                        brace_depth += 1;
                        expr_str.push(c);
                    } else if c == '}' {
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

                // Parse the expression
                let expr_tokens = crate::lexer::lex(&expr_str)?;
                let mut expr_parser = Parser::new(expr_tokens);
                let expr = expr_parser.expression()?;
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

/// Convenience function tae parse source code
pub fn parse(source: &str) -> HaversResult<Program> {
    let tokens = crate::lexer::lex(source)?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_declaration() {
        let program = parse("ken x = 5").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0], Stmt::VarDecl { .. }));
    }

    #[test]
    fn test_function_declaration() {
        let program = parse("dae greet(name) {\n  blether name\n}").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0], Stmt::Function { .. }));
    }

    #[test]
    fn test_if_statement() {
        let program =
            parse("gin x > 5 {\n  blether \"big\"\n} ither {\n  blether \"wee\"\n}").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0], Stmt::If { .. }));
    }

    #[test]
    fn test_while_loop() {
        let program = parse("whiles x < 10 {\n  x = x + 1\n}").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0], Stmt::While { .. }));
    }

    #[test]
    fn test_for_loop() {
        let program = parse("fer i in 1..10 {\n  blether i\n}").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0], Stmt::For { .. }));
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
        assert!(matches!(
            &program.statements[0],
            Stmt::VarDecl { initializer: Some(expr), .. }
                if matches!(expr, Expr::Dict { pairs, .. } if pairs.len() == 1)
        ));
    }

    #[test]
    fn test_multiline_list() {
        let program = parse("ken arr = [\n  1,\n  2,\n  3\n]").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(
            &program.statements[0],
            Stmt::VarDecl { initializer: Some(expr), .. }
                if matches!(expr, Expr::List { elements, .. } if elements.len() == 3)
        ));
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
	        assert!(matches!(program.statements[0], Stmt::Match { .. }));
	    }

	    fn first_match_arms(program: &Program) -> Option<&[MatchArm]> {
	        match program.statements.first()? {
	            Stmt::Match { arms, .. } => Some(arms),
	            _ => None,
	        }
	    }

		    #[test]
		    fn test_match_statement_parses_return_break_and_continue_arms() {
		        let program = parse(
		            r#"keek x {
			  whan 1 -> gie 1
		  whan 2 -> brak
		  whan 3 -> haud
		}"#,
		        )
		        .unwrap();

		        let arms = first_match_arms(&program).expect("expected match statement");
		        assert!(matches!(arms[0].body, Stmt::Return { .. }));
		        assert!(matches!(arms[1].body, Stmt::Break { .. }));
		        assert!(matches!(arms[2].body, Stmt::Continue { .. }));
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
		        assert!(matches!(
		            arms[0].pattern,
		            Pattern::Literal(Literal::Integer(-5))
	        ));
	        assert!(matches!(
            arms[1].pattern,
            Pattern::Literal(Literal::Float(f)) if (f + 3.14).abs() < 1e-6
        ));
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
		        assert!(matches!(
		            arms[0].pattern,
		            Pattern::Literal(Literal::Bool(false))
	        ));
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
        assert!(matches!(program.statements[0], Stmt::Class { .. }));
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
        assert!(matches!(
            err,
            HaversError::ParseError { message, line: 1 }
                if message == "Expected method definition in class"
        ));
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
        assert!(matches!(pat, Pattern::Wildcard));
    }

    #[test]
    fn test_match_pattern_minus_requires_number() {
        let err = parse("keek x { whan -foo -> 1 }").unwrap_err();
        assert!(matches!(err, HaversError::ParseError { .. }));
    }

    #[test]
    fn test_match_pattern_unexpected_token_errors() {
        let err = parse("keek x { whan + -> 1 }").unwrap_err();
        assert!(matches!(err, HaversError::ParseError { .. }));
    }

    #[test]
    fn test_invalid_compound_assignment_target_errors() {
        let err = parse("1 += 2").unwrap_err();
        assert!(matches!(
            err,
            HaversError::ParseError { message, .. } if message == "Invalid compound assignment target"
        ));
    }

    #[test]
    fn test_slice_syntax_step_present_but_empty() {
        let program = parse("ken a = arr[::]\nken b = arr[1::]").unwrap();
        assert_eq!(program.statements.len(), 2);

        assert!(matches!(
            &program.statements[0],
            Stmt::VarDecl { initializer: Some(expr), .. }
                if matches!(expr, Expr::Slice { start: None, end: None, step: None, .. })
        ));
        assert!(matches!(
            &program.statements[1],
            Stmt::VarDecl { initializer: Some(expr), .. }
                if matches!(expr, Expr::Slice { start: Some(_), end: None, step: None, .. })
        ));
    }

    #[test]
    fn test_import_requires_string_path() {
        let err = parse("fetch 123").unwrap_err();
        assert!(matches!(
            err,
            HaversError::UnexpectedToken { expected, .. } if expected == "module path"
        ));
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
        assert!(matches!(
            &program.statements[0],
            Stmt::VarDecl { initializer: Some(expr), .. }
                if matches!(expr, Expr::Unary { operator: UnaryOp::Not, .. })
        ));
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
        assert!(matches!(err, HaversError::UnkentToken { .. }));
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

    // ==================== More Statement Tests ====================

    #[test]
    fn test_struct_declaration() {
        let program = parse("thing Point { x, y }").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0], Stmt::Struct { .. }));
    }

    #[test]
    fn test_import_statement() {
        let program = parse(r#"fetch "math""#).unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0], Stmt::Import { .. }));
    }

    #[test]
    fn test_try_catch() {
        let program = parse("hae_a_bash { risky() } gin_it_gangs_wrang e { blether e }").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0], Stmt::TryCatch { .. }));
    }

    #[test]
    fn test_assert_statement() {
        let program = parse("mak_siccar x > 0").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0], Stmt::Assert { .. }));
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
        assert!(matches!(
            program.statements[0],
            Stmt::VarDecl {
                initializer: Some(Expr::Range { inclusive: true, .. }),
                ..
            }
        ));
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
        assert!(matches!(program.statements[0], Stmt::While { .. }));
    }

    #[test]
    fn test_for_loop_parse() {
        let program = parse("fer i in 1..10 { blether i }").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0], Stmt::For { .. }));
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
}
