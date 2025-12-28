#![cfg(coverage)]

use logos::Logos;
use mdhavers::token::{Token, TokenKind};

#[test]
fn token_display_is_covered_in_dependency_crate_instance() {
    let token = Token::new(TokenKind::Ken, "ken".to_string(), 5, 1);
    assert_eq!(format!("{token}"), "ken at line 5");
}

#[test]
fn tokenkind_logos_state_machine_is_exercised_for_coverage() {
    fn exercise(source: &str) {
        let mut lexer = TokenKind::lexer(source);
        while lexer.next().is_some() {}
    }

    // This is a deterministic “corpus” designed to drive Logos’ generated DFA through more
    // transitions (including partial keyword/operator matches and error recovery), improving
    // instantiation coverage without relying on external IO.
    let keywords = [
        "ken",
        "gin",
        "ither",
        "than",
        "whiles",
        "fer",
        "gie",
        "blether",
        "speir",
        "fae",
        "tae",
        "an",
        "or",
        "nae",
        "aye",
        "naething",
        "nil",
        "nowt",
        "dae",
        "thing",
        "fetch",
        "kin",
        "brak",
        "haud",
        "haud_yer_wheesht",
        "gang_on",
        "in",
        "is",
        "masel",
        "hae_a_bash",
        "gin_it_gangs_wrang",
        "keek",
        "whan",
        "mak_siccar",
        "log_whisper",
        "log_mutter",
        "log_blether",
        "log_holler",
        "log_roar",
        "hurl",
    ];

    let operators = [
        "+", "-", "*", "/", "%", "=", "==", "!=", "<", "<=", ">", ">=", "!", "+=", "-=", "*=",
        "/=", "...", "..=", "..", ".", "_", "(", ")", "{", "}", "[", "]", ",", ":", ";", "->",
        "|>", "|",
    ];

    let mut corpus = String::new();

    // Full tokens (keywords/operators), plus near-misses and prefixes to exercise fallback paths.
    for kw in keywords {
        corpus.push_str(kw);
        corpus.push(' ');
        corpus.push_str(&format!("{kw}x "));
        for i in 1..kw.len() {
            corpus.push_str(&kw[..i]);
            corpus.push(' ');
            corpus.push_str(&kw[..i]);
            corpus.push_str("x ");
        }
    }
    for op in operators {
        corpus.push_str(op);
        corpus.push(' ');
        corpus.push_str(op);
        corpus.push_str(op);
        corpus.push(' ');
    }

    // Numbers and string forms (including escapes and incomplete prefixes).
    corpus.push_str("0 1 42 3.14 1e3 1.0e-3 999999999 ");
    corpus.push_str("1e+3 1e-3 1E+3 1E-3 1.0e+3 1.0E+3 ");
    corpus.push_str("1e 1e+ 1e- 1.0e 1.0e+ 1.0e- 1.0E 1.0E+ 1.0E- ");
    corpus.push_str("\"\" \"a\" \"Hello, Scotland!\" \"Hello\\nWorld\" ");
    corpus.push_str("\"multi\nline\" \"a\\\nb\" ");
    corpus.push_str("\"\\\\\" \"\\t\" \"\\r\" \"\\0\" ");
    corpus.push_str("'a' 'single' ");
    corpus.push_str("'multi\nline' 'a\\\nb' ");
    corpus.push_str("'\\t' '\\r' '\\0' ");
    corpus.push_str("f\"\" f\"Hello {name}!\" f\"\\\"\" f\"\\t\" f\"\\r\" f\"\\0\" ");
    corpus.push_str("f\"multi\nline\" f\"a\\\nb\" ");

    // Comments and line breaks.
    corpus.push_str("# comment\n// comment\n");
    corpus.push_str("# eof_comment // eof_comment");
    // Additional whitespace forms covered by the lexer skip regex.
    corpus.push_str("\t\r\r\n");

    // Broad ASCII punctuation to drive error recovery and additional transitions.
    for b in 0x20u8..=0x7Eu8 {
        corpus.push(b as char);
        corpus.push(' ');
    }

    // Some unicode to exercise non-ASCII paths.
    corpus.push_str("é £ 🏴 ");

    // Large string/f-string literals that include a wide range of ASCII chars (including escaped
    // quotes and backslashes) to drive deeper DFA states in Logos' generated regex engines.
    fn push_wide_string_literal(corpus: &mut String, quote: char, prefix: &str) {
        corpus.push_str(prefix);
        corpus.push(quote);
        for b in 0x20u8..=0x7Eu8 {
            let ch = b as char;
            if ch == quote {
                corpus.push('\\');
                corpus.push(quote);
            } else if ch == '\\' {
                corpus.push('\\');
                corpus.push('\\');
            } else {
                corpus.push(ch);
            }
        }
        corpus.push(quote);
        corpus.push(' ');
    }
    push_wide_string_literal(&mut corpus, '"', "");
    push_wide_string_literal(&mut corpus, '"', "f");
    push_wide_string_literal(&mut corpus, '\'', "");

    // Unterminated literals to exercise error/partial-match paths.
    corpus.push_str("\"unterminated\\");
    corpus.push(' ');
    corpus.push_str("f\"unterminated");
    corpus.push(' ');
    corpus.push_str("'unterminated");
    corpus.push(' ');

    exercise(&corpus);

    // A small deterministic exploration of short character sequences that are particularly
    // relevant to the Logos regexes (quotes, backslashes, exponent markers, etc.). This tends to
    // reach DFA states that a "token list" corpus won't hit (like unterminated literals and
    // escape-heavy paths).
    let alphabet: &[char] = &[
        '"', '\'', 'f', '\\', 'n', 't', '0', '1', 'e', 'E', '+', '-', '.', '_', '#', '/', '\n',
        '\t', '\r',
    ];
    fn explore(exercise: &dyn Fn(&str), alphabet: &[char], max_len: usize) {
        fn rec(exercise: &dyn Fn(&str), alphabet: &[char], max_len: usize, cur: &mut String) {
            exercise(cur);
            if cur.len() >= max_len {
                return;
            }
            for &c in alphabet {
                cur.push(c);
                rec(exercise, alphabet, max_len, cur);
                cur.pop();
            }
        }
        let mut cur = String::new();
        rec(exercise, alphabet, max_len, &mut cur);
    }
    explore(&exercise, alphabet, 4);

    // A slightly broader exploration (small max length) over a wider alphabet to hit DFA states
    // that only show up for uncommon short prefixes / partial matches.
    let mut broad_alphabet = Vec::new();
    broad_alphabet.extend('a'..='z');
    broad_alphabet.extend('A'..='Z');
    broad_alphabet.extend('0'..='9');
    broad_alphabet.extend([
        ' ', '\n', '\t', '\r', '_', '#', '/', '\\', '\'', '"', '.', '+', '-', '*', '%', '=', '!',
        '<', '>', '|', '(', ')', '{', '}', '[', ']', ',', ':', ';',
    ]);
    broad_alphabet.sort_unstable();
    broad_alphabet.dedup();
    explore(&exercise, &broad_alphabet, 3);
}
