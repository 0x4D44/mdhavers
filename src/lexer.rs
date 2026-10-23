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
                    // An all-digit lexeme only fails when it overflows an i64.
                    if !lexeme.is_empty() && lexeme.bytes().all(|b| b.is_ascii_digit()) {
                        return Err(HaversError::InvalidNumber {
                            value: format!(
                                "'{}' is ower muckle fer a whole number (column {})",
                                lexeme, token_column
                            ),
                            line: token_line,
                        });
                    }
                    // `f"` matched but the f-string body never closed.
                    if lexeme == "f\"" {
                        return Err(HaversError::UnterminatedString { line: token_line });
                    }
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

/// A comment found in the source, wi' where it sits.
/// The token stream skips comments, so the formatter gets them fae here.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceComment {
    /// 1-based line o' the comment's first character
    pub line: usize,
    /// 1-based column (in chars) o' the comment's first character
    pub column: usize,
    /// The comment text, includin' its `#` or `//`, wi' trailin' whitespace trimmed
    pub text: String,
    /// True when code comes afore the comment on the same line
    pub trailing: bool,
}

/// Collect aw the `#` and `//` comments in the source, in order.
///
/// This disnae change the token stream: it runs the same logos lexer and
/// looks at the gaps between tokens, which only ever haud whitespace and
/// comments. It stops quietly at the first lexing error.
pub fn collect_comments(source: &str) -> Vec<SourceComment> {
    fn advance_position(
        source: &str,
        cursor: &mut usize,
        line: &mut usize,
        column: &mut usize,
        pos: usize,
    ) {
        for ch in source[*cursor..pos].chars() {
            if ch == '\n' {
                *line += 1;
                *column = 1;
            } else {
                *column += 1;
            }
        }
        *cursor = pos;
    }

    let mut comments = Vec::new();
    let mut logos = TokenKind::lexer(source);
    let mut prev_end = 0usize;
    let mut prev_is_code = false;
    let mut cursor = 0usize;
    let mut line = 1usize;
    let mut column = 1usize;

    loop {
        let next = logos.next();
        let token_span = next.as_ref().map(|_| logos.span());
        let gap_end = token_span.as_ref().map(|s| s.start).unwrap_or(source.len());

        let gap = &source[prev_end..gap_end];
        let trimmed = gap.trim_start_matches([' ', '\t', '\r']);
        if trimmed.starts_with('#') || trimmed.starts_with("//") {
            let offset = prev_end + (gap.len() - trimmed.len());
            advance_position(source, &mut cursor, &mut line, &mut column, offset);
            comments.push(SourceComment {
                line,
                column,
                text: trimmed.trim_end().to_string(),
                trailing: prev_is_code,
            });
        }

        match (next, token_span) {
            (Some(Ok(kind)), Some(span)) => {
                prev_is_code = kind != TokenKind::Newline;
                prev_end = span.end;
            }
            _ => break,
        }
    }

    comments
}

#[cfg(test)]
#[allow(clippy::approx_constant)]
mod tests {
    use super::*;

    fn is_unkent_token(err: &HaversError, lexeme: &str, line: usize, column: usize) -> bool {
        match err {
            HaversError::UnkentToken {
                lexeme: found,
                line: found_line,
                column: found_column,
            } => {
                let lexeme_ok = found == lexeme;
                let line_ok = *found_line == line;
                let column_ok = *found_column == column;
                lexeme_ok && line_ok && column_ok
            }
            _ => false,
        }
    }

    #[test]
    fn test_keywords() {
        let source = "ken gin ither whiles fer gie blether";
        let tokens = lex(source).unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Ken);
        assert_eq!(tokens[1].kind, TokenKind::Gin);
        assert_eq!(tokens[2].kind, TokenKind::Ither);
        assert_eq!(tokens[3].kind, TokenKind::Whiles);
        assert_eq!(tokens[4].kind, TokenKind::Fer);
        assert_eq!(tokens[5].kind, TokenKind::Gie);
        assert_eq!(tokens[6].kind, TokenKind::Blether);
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

        assert_eq!(tokens[0].kind, TokenKind::Plus);
        assert_eq!(tokens[1].kind, TokenKind::Minus);
        assert_eq!(tokens[2].kind, TokenKind::Star);
        assert_eq!(tokens[3].kind, TokenKind::Slash);
        assert_eq!(tokens[4].kind, TokenKind::EqualsEquals);
        assert_eq!(tokens[5].kind, TokenKind::BangEquals);
        assert_eq!(tokens[6].kind, TokenKind::Less);
        assert_eq!(tokens[7].kind, TokenKind::Greater);
        assert_eq!(tokens[8].kind, TokenKind::LessEquals);
        assert_eq!(tokens[9].kind, TokenKind::GreaterEquals);
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
    fn test_collect_comments_finds_whole_line_and_trailing_comments() {
        let source = "# heid  \nken x = 5 // tail\n  \"# no a comment\"\n\t# indented\r\nken y = 'a#b' # last";
        let comments = collect_comments(source);
        let expected = vec![
            SourceComment {
                line: 1,
                column: 1,
                text: "# heid".to_string(),
                trailing: false,
            },
            SourceComment {
                line: 2,
                column: 11,
                text: "// tail".to_string(),
                trailing: true,
            },
            SourceComment {
                line: 4,
                column: 2,
                text: "# indented".to_string(),
                trailing: false,
            },
            SourceComment {
                line: 5,
                column: 15,
                text: "# last".to_string(),
                trailing: true,
            },
        ];
        assert_eq!(comments, expected);
        // The token stream is unchanged: comments are still skipped
        assert_eq!(lex("ken x = 5 // tail").unwrap().len(), 5);
    }

    #[test]
    fn test_collect_comments_after_multiline_string_is_trailing() {
        let comments = collect_comments("ken s = \"a\nb\" # efter\n");
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].line, 2);
        assert_eq!(comments[0].column, 4);
        assert!(comments[0].trailing);
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
        assert!(is_unkent_token(&err, "@", 1, 1));
        let mismatch = HaversError::UnkentToken {
            lexeme: "#".to_string(),
            line: 1,
            column: 1,
        };
        assert!(!is_unkent_token(&mismatch, "@", 1, 1));
        let mismatch = HaversError::UnkentToken {
            lexeme: "@".to_string(),
            line: 2,
            column: 1,
        };
        assert!(!is_unkent_token(&mismatch, "@", 1, 1));
        let mismatch = HaversError::UnkentToken {
            lexeme: "@".to_string(),
            line: 1,
            column: 2,
        };
        assert!(!is_unkent_token(&mismatch, "@", 1, 1));
        let other = HaversError::UnexpectedToken {
            expected: "expression".to_string(),
            found: "@".to_string(),
            line: 1,
        };
        assert!(!is_unkent_token(&other, "@", 1, 1));
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
        assert!(is_unkent_token(&err, "@", 1, 5));
        let other = HaversError::UnexpectedToken {
            expected: "expression".to_string(),
            found: "@".to_string(),
            line: 1,
        };
        assert!(!is_unkent_token(&other, "@", 1, 5));
    }

    #[test]
    fn test_integer_literal_overflow_is_invalid_number() {
        // MDH-BUG-FLU-00020: i64::MAX + 1 used tae be reported as an unkent token.
        let err = lex("ken big = 9223372036854775808").unwrap_err();
        match err {
            HaversError::InvalidNumber { value, line } => {
                assert_eq!(line, 1);
                assert!(value.contains("9223372036854775808"), "{value}");
                assert!(value.contains("column 11"), "{value}");
            }
            other => panic!("expected InvalidNumber, got {other:?}"),
        }
        // i64::MAX itself is still fine.
        let tokens = lex("9223372036854775807").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Integer(i64::MAX));
    }

    #[test]
    fn test_unterminated_fstring_is_unterminated_string() {
        let err = lex("ken x = 1\nblether f\"abc {x}").unwrap_err();
        assert_eq!(err, HaversError::UnterminatedString { line: 2 });
    }

    #[test]
    fn test_more_keywords_operators_and_strings_are_lexed() {
        let source = r#"
speir fae tae an or nae aye naething nil nowt
ken gin ither than whiles fer gie blether
dae thing fetch kin brak haud haud_yer_wheesht gang_on in is masel hae_a_bash gin_it_gangs_wrang keek whan mak_siccar
log_whisper log_mutter log_blether log_holler log_roar hurl

42 3.14 1e3 1.0e-3
"double" 'single' f"Hello {name}!"

+ - * / % = == != < <= > >= ! += -= *= /= ... ..= .. . _ ( ) { } [ ] , : ; -> |> |
# comment
// comment
"#;

        let tokens = lex(source).unwrap();

        assert!(tokens.iter().any(|t| t.kind == TokenKind::Speir));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Ken));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Gin));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Ither));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Than));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Whiles));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Fer));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Gie));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Blether));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::MakSiccar));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::PipeForward));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Arrow));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Underscore));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Newline));
        assert!(tokens
            .iter()
            .any(|t| { t.kind == TokenKind::SingleQuoteString("single".to_string()) }));
        assert!(tokens
            .iter()
            .any(|t| { t.kind == TokenKind::FString("Hello {name}!".to_string()) }));
    }
}
