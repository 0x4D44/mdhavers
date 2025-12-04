use crate::ast::*;
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

    fn function_declaration(&mut self) -> HaversResult<Stmt> {
        let span = self.current_span();
        self.advance(); // consume 'dae'

        let name = self.expect_identifier("function name")?;
        self.expect(&TokenKind::LeftParen, "(")?;

        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                params.push(self.expect_identifier("parameter name")?);
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
            if !self.match_token(&TokenKind::Comma) {
                self.skip_newlines();
                break;
            }
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

    fn match_arm(&mut self) -> HaversResult<MatchArm> {
        let span = self.current_span();
        self.expect(&TokenKind::Whan, "whan")?;

        let pattern = self.pattern()?;
        self.expect(&TokenKind::Arrow, "->")?;
        self.skip_newlines();

        let body = if self.check(&TokenKind::LeftBrace) {
            self.block()?
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
            TokenKind::String(s) => {
                let s = s.clone();
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
            TokenKind::Identifier(name) if name == "_" => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(Pattern::Identifier(name))
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
        let expr = self.or()?;

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
                Expr::Get { object, property, .. } => {
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
                            left: Box::new(Expr::Variable {
                                name,
                                span,
                            }),
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

    fn or(&mut self) -> HaversResult<Expr> {
        let mut expr = self.and()?;

        while self.match_token(&TokenKind::Or) {
            let span = self.previous().map(|t| Span::new(t.line, t.column)).unwrap_or(self.current_span());
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
            let span = self.previous().map(|t| Span::new(t.line, t.column)).unwrap_or(self.current_span());
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

            let span = self.previous().map(|t| Span::new(t.line, t.column)).unwrap_or(self.current_span());
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

            let span = self.previous().map(|t| Span::new(t.line, t.column)).unwrap_or(self.current_span());
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

            let span = self.previous().map(|t| Span::new(t.line, t.column)).unwrap_or(self.current_span());
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

            let span = self.previous().map(|t| Span::new(t.line, t.column)).unwrap_or(self.current_span());
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
            let span = self.previous().map(|t| Span::new(t.line, t.column)).unwrap_or(self.current_span());
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
                let span = self.previous().map(|t| Span::new(t.line, t.column)).unwrap_or(self.current_span());
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
            let span = self.previous().map(|t| Span::new(t.line, t.column)).unwrap_or(self.current_span());
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
                | TokenKind::Identifier(_)
                | TokenKind::LeftParen
                | TokenKind::LeftBracket
                | TokenKind::LeftBrace
                | TokenKind::Minus
                | TokenKind::Bang
                | TokenKind::Aye
                | TokenKind::Naething
                | TokenKind::Masel
                | TokenKind::Speir
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
                let index = self.expression()?;
                self.expect(&TokenKind::RightBracket, "]")?;
                expr = Expr::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                    span,
                };
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
                arguments.push(self.expression()?);
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
            TokenKind::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Literal {
                    value: Literal::String(s),
                    span,
                })
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
                Ok(Expr::Grouping {
                    expr: Box::new(expr),
                    span,
                })
            }
            TokenKind::LeftBracket => {
                self.advance();
                let mut elements = Vec::new();
                if !self.check(&TokenKind::RightBracket) {
                    loop {
                        elements.push(self.expression()?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(&TokenKind::RightBracket, "]")?;
                Ok(Expr::List { elements, span })
            }
            TokenKind::LeftBrace => {
                self.advance();
                let mut pairs = Vec::new();
                self.skip_newlines();
                if !self.check(&TokenKind::RightBrace) {
                    loop {
                        let key = self.expression()?;
                        self.expect(&TokenKind::Colon, ":")?;
                        let value = self.expression()?;
                        pairs.push((key, value));
                        self.skip_newlines();
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                        self.skip_newlines();
                    }
                }
                self.expect(&TokenKind::RightBrace, "}")?;
                Ok(Expr::Dict { pairs, span })
            }
            _ => Err(HaversError::ParseError {
                message: format!("Unexpected token: {}", token.kind),
                line: token.line,
            }),
        }
    }

    fn maybe_range(&mut self, start_expr: Expr) -> HaversResult<Expr> {
        if self.match_token(&TokenKind::DotDot) {
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
        self.tokens.get(self.current).unwrap_or(&self.tokens[self.tokens.len() - 1])
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
        if let TokenKind::String(s) = &token.kind {
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
        let program = parse("gin x > 5 {\n  blether \"big\"\n} ither {\n  blether \"wee\"\n}")
            .unwrap();
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
}
