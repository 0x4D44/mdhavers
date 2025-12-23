use logos::Logos;

use crate::error::{HaversError, HaversResult};
use crate::token::{Token, TokenKind};

/// The lexer - turns source code intae tokens
pub struct Lexer<'source> {
    source: &'source str,
    logos: logos::Lexer<'source, TokenKind>,
    line: usize,
    column: usize,
    cursor: usize,
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source str) -> Self {
        Lexer {
            source,
            logos: TokenKind::lexer(source),
            line: 1,
            column: 1,
            cursor: 0,
        }
    }

    fn advance_to(&mut self, pos: usize) {
        let slice = &self.source[self.cursor..pos];
        for ch in slice.chars() {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        self.cursor = pos;
    }

    /// Tokenize the whole source intae a vector
    pub fn tokenize(&mut self) -> HaversResult<Vec<Token>> {
        let mut tokens = Vec::new();

        while let Some(result) = self.logos.next() {
            let span = self.logos.span();
            self.advance_to(span.start);
            let token_line = self.line;
            let token_column = self.column;
            let lexeme = self.logos.slice().to_string();

            match result {
                Ok(kind) => {
                    tokens.push(Token::new(kind, lexeme, token_line, token_column));
                }
                Err(_) => {
                    return Err(HaversError::UnkentToken {
                        lexeme,
                        line: token_line,
                        column: token_column,
                    });
                }
            }

            self.advance_to(span.end);
        }

        // Add EOF token
        tokens.push(Token::eof(self.line));

        Ok(tokens)
    }
}

/// Convenience function tae lex a string
pub fn lex(source: &str) -> HaversResult<Vec<Token>> {
    let mut lexer = Lexer::new(source);
    lexer.tokenize()
}

#[cfg(test)]
#[allow(clippy::approx_constant)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let source = "ken gin ither whiles fer gie blether";
        let tokens = lex(source).unwrap();

        assert!(matches!(tokens[0].kind, TokenKind::Ken));
        assert!(matches!(tokens[1].kind, TokenKind::Gin));
        assert!(matches!(tokens[2].kind, TokenKind::Ither));
        assert!(matches!(tokens[3].kind, TokenKind::Whiles));
        assert!(matches!(tokens[4].kind, TokenKind::Fer));
        assert!(matches!(tokens[5].kind, TokenKind::Gie));
        assert!(matches!(tokens[6].kind, TokenKind::Blether));
    }

    #[test]
    fn test_numbers() {
        let source = "42 3.14";
        let tokens = lex(source).unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Integer(42));
        assert_eq!(tokens[1].kind, TokenKind::Float(3.14));
    }

    #[test]
    fn test_strings() {
        let source = r#""Hello, Scotland!""#;
        let tokens = lex(source).unwrap();

        assert_eq!(
            tokens[0].kind,
            TokenKind::String("Hello, Scotland!".to_string())
        );
    }

    #[test]
    fn test_operators() {
        let source = "+ - * / == != < > <= >=";
        let tokens = lex(source).unwrap();

        assert!(matches!(tokens[0].kind, TokenKind::Plus));
        assert!(matches!(tokens[1].kind, TokenKind::Minus));
        assert!(matches!(tokens[2].kind, TokenKind::Star));
        assert!(matches!(tokens[3].kind, TokenKind::Slash));
        assert!(matches!(tokens[4].kind, TokenKind::EqualsEquals));
        assert!(matches!(tokens[5].kind, TokenKind::BangEquals));
        assert!(matches!(tokens[6].kind, TokenKind::Less));
        assert!(matches!(tokens[7].kind, TokenKind::Greater));
        assert!(matches!(tokens[8].kind, TokenKind::LessEquals));
        assert!(matches!(tokens[9].kind, TokenKind::GreaterEquals));
    }

    #[test]
    fn test_identifiers() {
        let source = "foo bar_baz _private";
        let tokens = lex(source).unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Identifier("foo".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Identifier("bar_baz".to_string()));
        assert_eq!(
            tokens[2].kind,
            TokenKind::Identifier("_private".to_string())
        );
    }

    #[test]
    fn test_comments_are_skipped() {
        let source = "ken x = 5 # this is a comment\nken y = 10";
        let tokens = lex(source).unwrap();

        // Should have: ken, x, =, 5, newline, ken, y, =, 10, eof
        assert_eq!(tokens.len(), 10);
    }

    #[test]
    fn test_multiline() {
        let source = "ken x = 5\nken y = 10";
        let tokens = lex(source).unwrap();

        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[5].line, 2); // second ken
    }

    #[test]
    fn test_braw_program() {
        let source = r#"
# A wee program
dae greet(name) {
    blether "Hullo, " + name + "!"
}

ken message = "Scotland"
greet(message)
"#;
        let tokens = lex(source).unwrap();
        assert!(tokens.len() > 10);
        // Just check it parses without error
    }

    #[test]
    fn test_invalid_token_error() {
        let err = lex("@").unwrap_err();
        assert!(matches!(
            err,
            HaversError::UnkentToken {
                lexeme,
                line: 1,
                column: 1
            } if lexeme == "@"
        ));
    }

    #[test]
    fn test_multiline_string_updates_line_tracking() {
        let source = "\"Hello\nWorld\" ken x = 1";
        let tokens = lex(source).unwrap();

        // "World\" ken ..." so `ken` begins on line 2 at column 8.
        assert_eq!(tokens[1].kind, TokenKind::Ken);
        assert_eq!(tokens[1].line, 2);
        assert_eq!(tokens[1].column, 8);
    }

    #[test]
    fn test_unicode_columns_count_chars_not_bytes() {
        let err = lex("\"é\" @").unwrap_err();
        assert!(matches!(
            err,
            HaversError::UnkentToken {
                lexeme,
                line: 1,
                column: 5
            } if lexeme == "@"
        ));
    }
}
