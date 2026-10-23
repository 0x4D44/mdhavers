//! Pretty printer fer mdhavers code
//! Makes yer code look braw and proper!
//!
//! Safety first: [`format_source_checked`] only ever hands back output that
//! parses tae the same AST as the original (spans aside) and keeps every
//! comment, in order. If it cannae guarantee that, it refuses.

use std::collections::HashMap;
use std::fmt;

use crate::ast::*;
use crate::error::HaversError;
use crate::lexer::{collect_comments, SourceComment};
use crate::token::{Token, TokenKind};

/// Configuration fer the formatter
#[allow(dead_code)]
pub struct FormatterConfig {
    /// Number o' spaces fer indentation
    pub indent_size: usize,
    /// Maximum line width before we wrap
    pub max_line_width: usize,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        FormatterConfig {
            indent_size: 4,
            max_line_width: 100,
        }
    }
}

/// Why formattin' didnae produce output
#[derive(Debug)]
pub enum FormatError {
    /// The source didnae parse in the first place
    Parse(HaversError),
    /// The formatter refused, because the result wouldnae be safe
    Refused { line: Option<usize>, reason: String },
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::Parse(e) => write!(f, "{}", e),
            FormatError::Refused {
                line: Some(line),
                reason,
            } => write!(
                f,
                "Haud on! The formatter cannae tidy this safely (line {}): {}",
                line, reason
            ),
            FormatError::Refused { line: None, reason } => write!(
                f,
                "Haud on! The formatter cannae tidy this safely: {}",
                reason
            ),
        }
    }
}

/// Token-level facts aboot the original source, used tae place comments
/// and blank lines. Spans in the AST are token start positions.
struct SourceInfo {
    tokens: Vec<Token>,
    by_pos: HashMap<(usize, usize), usize>,
    /// Matchin' bracket fer each ( [ { ) ] } token
    partner: Vec<Option<usize>>,
}

impl SourceInfo {
    fn new(source: &str) -> Option<Self> {
        let tokens = crate::lexer::lex(source).ok()?;
        let mut by_pos = HashMap::new();
        let mut partner = vec![None; tokens.len()];
        let mut stack: Vec<usize> = Vec::new();
        for (i, t) in tokens.iter().enumerate() {
            by_pos.entry((t.line, t.column)).or_insert(i);
            match t.kind {
                TokenKind::LeftParen | TokenKind::LeftBracket | TokenKind::LeftBrace => {
                    stack.push(i)
                }
                TokenKind::RightParen | TokenKind::RightBracket | TokenKind::RightBrace => {
                    if let Some(open) = stack.pop() {
                        partner[open] = Some(i);
                        partner[i] = Some(open);
                    }
                }
                _ => {}
            }
        }
        Some(SourceInfo {
            tokens,
            by_pos,
            partner,
        })
    }

    fn index_at(&self, span: Span) -> Option<usize> {
        self.by_pos.get(&(span.line, span.column)).copied()
    }

    /// Last source line covered by token `i` (strings can span lines)
    fn token_end_line(&self, i: usize) -> usize {
        let t = &self.tokens[i];
        if t.kind == TokenKind::Newline {
            t.line
        } else {
            t.line + t.lexeme.matches('\n').count()
        }
    }

    /// Last source line o' the construct startin' at token `start`: scan
    /// forward until a newline/`;` (or `,` when askit) at bracket depth 0,
    /// or a closer that belongs tae an enclosin' construct.
    fn scan_end_line(&self, start: usize, stop_at_comma: bool) -> usize {
        let mut depth = 0usize;
        let mut last = self.tokens[start].line;
        for (i, token) in self.tokens.iter().enumerate().skip(start) {
            match token.kind {
                TokenKind::Eof => break,
                TokenKind::Newline | TokenKind::Semicolon if depth == 0 => break,
                TokenKind::Comma if depth == 0 && stop_at_comma => break,
                TokenKind::LeftParen | TokenKind::LeftBracket | TokenKind::LeftBrace => depth += 1,
                TokenKind::RightParen | TokenKind::RightBracket | TokenKind::RightBrace => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                }
                _ => {}
            }
            if token.kind != TokenKind::Newline {
                last = last.max(self.token_end_line(i));
            }
        }
        last
    }

    fn find_from(&self, from: usize, kind: &TokenKind) -> Option<usize> {
        let wanted = std::mem::discriminant(kind);
        (from..self.tokens.len()).find(|&i| std::mem::discriminant(&self.tokens[i].kind) == wanted)
    }

    /// (open line, close line) o' the bracket token `open`
    fn bracket_lines(&self, open: usize) -> Option<(usize, usize)> {
        let close = self.partner[open]?;
        Some((self.tokens[open].line, self.tokens[close].line))
    }

    /// (open line, close line) o' the bracket token o' `kind` at `span`
    fn bracket_lines_at(&self, span: Span, kind: &TokenKind) -> Option<(usize, usize)> {
        let i = self.index_at(span)?;
        if std::mem::discriminant(&self.tokens[i].kind) != std::mem::discriminant(kind) {
            return None;
        }
        self.bracket_lines(i)
    }

    fn is_single_quoted_at(&self, span: Span) -> bool {
        self.index_at(span)
            .map(|i| matches!(self.tokens[i].kind, TokenKind::SingleQuoteString(_)))
            .unwrap_or(false)
    }

    /// The body brace o' a function, class or match statement
    fn body_brace_lines(&self, stmt: &Stmt) -> Option<(usize, usize)> {
        let i = self.index_at(stmt.span())?;
        match stmt {
            Stmt::Function { .. } => {
                let lp = self.find_from(i, &TokenKind::LeftParen)?;
                let rp = self.partner[lp]?;
                let lb = self.find_from(rp + 1, &TokenKind::LeftBrace)?;
                self.bracket_lines(lb)
            }
            Stmt::Class { .. } => {
                let lb = self.find_from(i, &TokenKind::LeftBrace)?;
                self.bracket_lines(lb)
            }
            Stmt::Match { .. } => {
                // The body brace is the first `{` followed by `whan` or `}`
                let mut j = i;
                loop {
                    let lb = self.find_from(j, &TokenKind::LeftBrace)?;
                    let mut k = lb + 1;
                    while k < self.tokens.len() && self.tokens[k].kind == TokenKind::Newline {
                        k += 1;
                    }
                    if k < self.tokens.len()
                        && matches!(self.tokens[k].kind, TokenKind::Whan | TokenKind::RightBrace)
                    {
                        return self.bracket_lines(lb);
                    }
                    j = lb + 1;
                }
            }
            _ => None,
        }
    }
}

/// The formatter itself
pub struct Formatter {
    config: FormatterConfig,
    output: String,
    indent_level: usize,
    /// Present when formattin' wi' the original source (comments, blank lines)
    src: Option<SourceInfo>,
    comments: Vec<SourceComment>,
    next_comment: usize,
    /// Last source line emitted in the current statement sequence (0 = none yet)
    last_line: usize,
    /// A blank line is owed afore the next item (efter a top-level function or class)
    pending_decl_blank: bool,
    /// Formattin' an expression inside an f-string hole
    in_fstring_hole: bool,
    refusal: Option<(usize, String)>,
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Formatter {
    pub fn new() -> Self {
        Formatter::with_config(FormatterConfig::default())
    }

    pub fn with_config(config: FormatterConfig) -> Self {
        Formatter {
            config,
            output: String::new(),
            indent_level: 0,
            src: None,
            comments: Vec::new(),
            next_comment: 0,
            last_line: 0,
            pending_decl_blank: false,
            in_fstring_hole: false,
            refusal: None,
        }
    }

    /// Format a whole program fae its AST alone (nae comments ken'd)
    pub fn format(&mut self, program: &Program) -> String {
        self.src = None;
        self.comments.clear();
        self.format_program(program);
        self.output.clone()
    }

    /// Format a program parsed fae `source`, keepin' its comments and
    /// single blank lines. Refuses when a comment cannae be placed.
    /// This does not re-parse the output: use [`format_source_checked`] fer that.
    pub fn format_with_source(
        &mut self,
        program: &Program,
        source: &str,
    ) -> Result<String, FormatError> {
        self.src = SourceInfo::new(source);
        self.comments = if self.src.is_some() {
            collect_comments(source)
        } else {
            Vec::new()
        };
        self.format_program(program);
        self.src = None;
        if let Some((line, reason)) = self.refusal.take() {
            return Err(FormatError::Refused {
                line: Some(line),
                reason,
            });
        }
        Ok(self.output.clone())
    }

    fn format_program(&mut self, program: &Program) {
        self.output.clear();
        self.indent_level = 0;
        self.next_comment = 0;
        self.last_line = 0;
        self.pending_decl_blank = false;
        self.in_fstring_hole = false;
        self.refusal = None;

        self.format_seq(&program.statements, true);
        self.flush_comments_before(usize::MAX);

        // Ensure file ends with newline
        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }
    }

    fn indent(&self) -> String {
        " ".repeat(self.config.indent_size * self.indent_level)
    }

    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn writeln(&mut self, s: &str) {
        self.output.push_str(&self.indent());
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn refuse(&mut self, line: usize, reason: &str) {
        if self.refusal.is_none() {
            self.refusal = Some((line, reason.to_string()));
        }
    }

    /// Run `f` wi' a fresh output buffer and hand back what it wrote
    fn with_buffer<F: FnOnce(&mut Self)>(&mut self, f: F) -> String {
        let saved_output = std::mem::take(&mut self.output);
        let saved_last = self.last_line;
        let saved_pending = self.pending_decl_blank;
        f(&mut *self);
        self.last_line = saved_last;
        self.pending_decl_blank = saved_pending;
        std::mem::replace(&mut self.output, saved_output)
    }

    // ==================== Comments and blank lines ====================

    /// Emit, as own-line comments, aw pendin' comments afore `line`
    fn flush_comments_before(&mut self, line: usize) {
        if self.src.is_none() {
            return;
        }
        while self.next_comment < self.comments.len()
            && self.comments[self.next_comment].line < line
        {
            let c = self.comments[self.next_comment].clone();
            self.next_comment += 1;
            if c.trailing {
                self.refuse(
                    c.line,
                    "a comment at the end o' a line cannae be kept in its place",
                );
            }
            self.blank_before(Some(c.line));
            self.writeln(&c.text);
            self.last_line = c.line;
        }
    }

    /// Finish an output line that covers source lines up tae `line`: hand
    /// back a trailin' comment tae append, or refuse if comments sit inside.
    fn close_line(&mut self, line: usize) -> String {
        if self.src.is_none() {
            return String::new();
        }
        let first = self.next_comment;
        while self.next_comment < self.comments.len()
            && self.comments[self.next_comment].line <= line
        {
            self.next_comment += 1;
        }
        let count = self.next_comment - first;
        if count == 0 {
            return String::new();
        }
        let c = &self.comments[first];
        if count == 1 && c.trailing && c.line == line {
            return format!("  {}", c.text);
        }
        let at = c.line;
        self.refuse(
            at,
            "there's a comment stuck in the middle o' a statement and \
             ah cannae find a safe hame fer it",
        );
        String::new()
    }

    fn close_line_opt(&mut self, line: Option<usize>) -> String {
        match line {
            Some(l) => self.close_line(l),
            None => String::new(),
        }
    }

    /// Emit one blank line afore an item startin' at `line` when the source
    /// had one (or a top-level function/class came just afore)
    fn blank_before(&mut self, line: Option<usize>) {
        let gap = match line {
            Some(l) => self.src.is_some() && self.last_line != 0 && l > self.last_line + 1,
            None => false,
        };
        if (gap || self.pending_decl_blank)
            && self.output.ends_with('\n')
            && !self.output.ends_with("\n\n")
        {
            self.output.push('\n');
        }
        self.pending_decl_blank = false;
    }

    fn stmt_start_line(&self, stmt: &Stmt) -> Option<usize> {
        self.src.as_ref()?;
        Some(stmt_start(stmt).line)
    }

    /// Last source line o' a statement that prints on one line
    fn simple_end_line(&self, stmt: &Stmt) -> Option<usize> {
        let src = self.src.as_ref()?;
        let i = src.index_at(stmt_start(stmt))?;
        Some(src.scan_end_line(i, false))
    }

    // ==================== Statements ====================

    /// Format a sequence o' statements at the current indent
    fn format_seq(&mut self, stmts: &[Stmt], blank_after_decls: bool) {
        for stmt in stmts {
            let start = self.stmt_start_line(stmt);
            if let Some(line) = start {
                self.flush_comments_before(line);
            }
            self.blank_before(start);
            if let Some(end) = self.format_stmt(stmt) {
                self.last_line = end;
            }
            self.pending_decl_blank =
                blank_after_decls && matches!(stmt, Stmt::Function { .. } | Stmt::Class { .. });
        }
    }

    /// Format the inside o' a braced body, one level deeper
    fn format_block_body(&mut self, stmts: &[Stmt], close: Option<usize>, blank_after_decls: bool) {
        self.indent_level += 1;
        self.last_line = 0;
        self.pending_decl_blank = false;
        self.format_seq(stmts, blank_after_decls);
        if let Some(close) = close {
            self.flush_comments_before(close);
        }
        self.pending_decl_blank = false;
        self.indent_level -= 1;
    }

    /// Write `{`, the body and `}` (nae newline efter). Returns the `}` line.
    fn format_braced_body(
        &mut self,
        stmts: &[Stmt],
        lines: Option<(usize, usize)>,
        blank_after_decls: bool,
    ) -> Option<usize> {
        let trailing = self.close_line_opt(lines.map(|l| l.0));
        self.write("{");
        self.write(&trailing);
        self.output.push('\n');
        self.format_block_body(stmts, lines.map(|l| l.1), blank_after_decls);
        self.write(&self.indent());
        self.write("}");
        lines.map(|l| l.1)
    }

    /// Write a block statement as `{ ... }`; ony other statement gets wrapped
    fn format_braced(&mut self, stmt: &Stmt) -> Option<usize> {
        match stmt {
            Stmt::Block { statements, span } => {
                let lines = self
                    .src
                    .as_ref()
                    .and_then(|s| s.bracket_lines_at(*span, &TokenKind::LeftBrace));
                self.format_braced_body(statements, lines, false)
            }
            other => self.format_braced_body(std::slice::from_ref(other), None, false),
        }
    }

    /// Format one statement at the current indent, endin' wi' a newline.
    /// Returns the last source line it covered (when the source is ken'd).
    fn format_stmt(&mut self, stmt: &Stmt) -> Option<usize> {
        if !is_compound(stmt) {
            let text = self.format_simple_stmt(stmt);
            let end = self.simple_end_line(stmt);
            let trailing = self.close_line_opt(end);
            self.writeln(&format!("{}{}", text, trailing));
            return end;
        }
        self.write(&self.indent());
        let end = self.format_compound(stmt);
        let trailing = self.close_line_opt(end);
        self.write(&trailing);
        self.output.push('\n');
        end
    }

    /// Write a compound statement fae the current position (nae indent, nae newline)
    fn format_compound(&mut self, stmt: &Stmt) -> Option<usize> {
        match stmt {
            Stmt::Block { .. } => self.format_braced(stmt),

            Stmt::If { .. } => self.format_if(stmt),

            Stmt::While {
                condition, body, ..
            } => {
                let cond = self.format_expr(condition);
                self.write(&format!("whiles {} ", cond));
                self.format_braced(body)
            }

            Stmt::For {
                variable,
                iterable,
                body,
                ..
            } => {
                let iter = self.format_expr(iterable);
                self.write(&format!("fer {} in {} ", variable, iter));
                self.format_braced(body)
            }

            Stmt::Function {
                name, params, body, ..
            } => {
                let lines = self.src.as_ref().and_then(|s| s.body_brace_lines(stmt));
                let params_str = self.format_params(params);
                self.write(&format!("dae {}({}) ", name, params_str));
                self.format_braced_body(body, lines, false)
            }

            Stmt::Class {
                name,
                superclass,
                methods,
                ..
            } => {
                let lines = self.src.as_ref().and_then(|s| s.body_brace_lines(stmt));
                let inheritance = if let Some(parent) = superclass {
                    format!(" fae {}", parent)
                } else {
                    String::new()
                };
                self.write(&format!("kin {}{} ", name, inheritance));
                // Blank line between methods
                self.format_braced_body(methods, lines, true)
            }

            Stmt::TryCatch {
                try_block,
                error_name,
                catch_block,
                ..
            } => {
                self.write("hae_a_bash ");
                self.format_braced(try_block);
                self.write(&format!(" gin_it_gangs_wrang {} ", error_name));
                self.format_braced(catch_block)
            }

            Stmt::Match { value, arms, .. } => {
                let lines = self.src.as_ref().and_then(|s| s.body_brace_lines(stmt));
                let val = self.format_expr(value);
                let trailing = self.close_line_opt(lines.map(|l| l.0));
                self.write(&format!("keek {} {{{}\n", val, trailing));
                self.indent_level += 1;
                self.last_line = 0;
                self.pending_decl_blank = false;
                for arm in arms {
                    self.format_match_arm(arm);
                }
                if let Some((_, close)) = lines {
                    self.flush_comments_before(close);
                }
                self.indent_level -= 1;
                self.write(&self.indent());
                self.write("}");
                lines.map(|l| l.1)
            }

            // Simple statements are handled by format_simple_stmt
            other => {
                let text = self.format_simple_stmt(other);
                self.write(&text);
                self.simple_end_line(other)
            }
        }
    }

    /// Write an if statement, followin' `ither gin` chains
    fn format_if(&mut self, stmt: &Stmt) -> Option<usize> {
        match stmt {
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let cond = self.format_expr(condition);
                self.write(&format!("gin {} ", cond));
                let mut end = self.format_braced(then_branch);
                if let Some(else_stmt) = else_branch {
                    self.write(" ither ");
                    end = match else_stmt.as_ref() {
                        nested @ Stmt::If { .. } => self.format_if(nested),
                        other => self.format_braced(other),
                    };
                }
                end
            }
            other => self.format_braced(other),
        }
    }

    /// Format a statement that prints on a single line (bar nested blocks
    /// inside expressions), without indent or newline
    fn format_simple_stmt(&mut self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::VarDecl {
                name, initializer, ..
            } => match initializer {
                Some(expr) => {
                    let init = self.format_expr(expr);
                    format!("ken {} = {}", name, init)
                }
                None => format!("ken {}", name),
            },

            Stmt::Expression { expr, .. } => self.format_expr(expr),

            Stmt::Return { value, .. } => match value {
                Some(expr) => {
                    let v = self.format_expr(expr);
                    format!("gie {}", v)
                }
                None => "gie".to_string(),
            },

            Stmt::Print { value, .. } => {
                let v = self.format_expr(value);
                format!("blether {}", v)
            }

            Stmt::Break { .. } => "brak".to_string(),

            Stmt::Continue { .. } => "haud".to_string(),

            Stmt::Struct { name, fields, .. } => {
                if fields.is_empty() {
                    format!("thing {} {{}}", name)
                } else {
                    format!("thing {} {{ {} }}", name, fields.join(", "))
                }
            }

            Stmt::Import { path, alias, .. } => {
                // The parser keeps import paths raw (nae escape processing)
                let quoted = quote_raw(path);
                if let Some(a) = alias {
                    format!("fetch {} tae {}", quoted, a)
                } else {
                    format!("fetch {}", quoted)
                }
            }

            Stmt::Assert {
                condition, message, ..
            } => {
                let cond = self.format_expr(condition);
                if let Some(msg) = message {
                    let msg_str = self.format_expr(msg);
                    format!("mak_siccar {}, {}", cond, msg_str)
                } else {
                    format!("mak_siccar {}", cond)
                }
            }

            Stmt::Destructure {
                patterns, value, ..
            } => {
                let patterns_str = self.format_destruct_patterns(patterns);
                let val_str = self.format_expr(value);
                format!("ken [{}] = {}", patterns_str, val_str)
            }

            Stmt::Log {
                level,
                message,
                extras,
                ..
            } => {
                let keyword = match level {
                    crate::ast::LogLevel::Wheesht => "log_wheesht",
                    crate::ast::LogLevel::Roar => "log_roar",
                    crate::ast::LogLevel::Holler => "log_holler",
                    crate::ast::LogLevel::Blether => "log_blether",
                    crate::ast::LogLevel::Mutter => "log_mutter",
                    crate::ast::LogLevel::Whisper => "log_whisper",
                };
                let msg = self.format_expr(message);
                if extras.is_empty() {
                    format!("{} {}", keyword, msg)
                } else {
                    let mut parts = Vec::with_capacity(extras.len());
                    for expr in extras {
                        parts.push(self.format_expr(expr));
                    }
                    format!("{} {}, {}", keyword, msg, parts.join(", "))
                }
            }

            Stmt::Hurl { message, .. } => {
                let msg = self.format_expr(message);
                format!("hurl {}", msg)
            }

            // Compound statements: format intae a buffer an' splice it in
            compound => {
                let text = self.with_buffer(|me| {
                    me.format_stmt(compound);
                });
                text.trim().to_string()
            }
        }
    }

    /// Format destructuring patterns
    fn format_destruct_patterns(&self, patterns: &[DestructPattern]) -> String {
        patterns
            .iter()
            .map(|p| match p {
                DestructPattern::Variable(name) => name.clone(),
                DestructPattern::Rest(name) => format!("...{}", name),
                DestructPattern::Ignore => "_".to_string(),
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn format_match_arm(&mut self, arm: &MatchArm) {
        let start = self.src.as_ref().map(|_| arm.span.line);
        if let Some(line) = start {
            self.flush_comments_before(line);
        }
        self.blank_before(start);
        let pattern = self.format_pattern(&arm.pattern);
        self.write(&self.indent());
        self.write(&format!("whan {} -> ", pattern));
        // Arms take a block or a bare blether/gie/brak/haud/expression
        let end = match &arm.body {
            Stmt::Print { .. }
            | Stmt::Return { .. }
            | Stmt::Break { .. }
            | Stmt::Continue { .. }
            | Stmt::Expression { .. } => {
                let text = self.format_simple_stmt(&arm.body);
                self.write(&text);
                self.simple_end_line(&arm.body)
            }
            other => self.format_braced(other),
        };
        let trailing = self.close_line_opt(end);
        self.write(&trailing);
        self.output.push('\n');
        if let Some(end) = end {
            self.last_line = end;
        }
    }

    fn format_pattern(&mut self, pattern: &Pattern) -> String {
        match pattern {
            Pattern::Literal(lit) => self.format_literal(lit, None),
            Pattern::Identifier(name) => name.clone(),
            Pattern::Wildcard => "_".to_string(),
            Pattern::Range {
                start,
                end,
                inclusive,
            } => {
                let s = self.format_expr(start);
                let e = self.format_expr(end);
                let op = if *inclusive { "..=" } else { ".." };
                format!("{}{}{}", s, op, e)
            }
        }
    }

    /// Format function parameters, handling default values
    fn format_params(&mut self, params: &[Param]) -> String {
        let mut parts = Vec::with_capacity(params.len());
        for p in params {
            if let Some(default) = &p.default {
                let d = self.format_expr(default);
                parts.push(format!("{} = {}", p.name, d));
            } else {
                parts.push(p.name.clone());
            }
        }
        parts.join(", ")
    }

    fn format_expr_list(&mut self, exprs: &[Expr]) -> Vec<String> {
        let mut out = Vec::with_capacity(exprs.len());
        for e in exprs {
            out.push(self.format_expr(e));
        }
        out
    }

    // ==================== Expressions ====================

    fn format_literal(&self, lit: &Literal, span: Option<Span>) -> String {
        match lit {
            Literal::Integer(n) => n.to_string(),
            Literal::Float(f) => format_float(*f),
            Literal::String(s) => {
                let single = if self.in_fstring_hole {
                    // Inside f-string holes single quotes lex under the current lexer
                    true
                } else {
                    match (span, self.src.as_ref()) {
                        (Some(sp), Some(src)) => src.is_single_quoted_at(sp),
                        _ => false,
                    }
                };
                quote_string(s, if single { '\'' } else { '"' })
            }
            Literal::Bool(true) => "aye".to_string(),
            Literal::Bool(false) => "nae".to_string(),
            Literal::Nil => "naething".to_string(),
        }
    }

    /// The bracket lines o' a list/dict literal that spanned several source lines
    fn multiline_bracket(&self, span: Span, kind: &TokenKind) -> Option<(usize, usize)> {
        let lines = self.src.as_ref()?.bracket_lines_at(span, kind)?;
        if lines.1 > lines.0 {
            Some(lines)
        } else {
            None
        }
    }

    /// Format a list or dict one item per line, keepin' comments between items
    fn format_multiline_items<T>(
        &mut self,
        brackets: (&str, &str),
        items: &[T],
        lines: (usize, usize),
        start_of: fn(&T) -> Span,
        format_item: fn(&mut Self, &T) -> String,
    ) -> String {
        self.with_buffer(|me| {
            let trailing = me.close_line(lines.0);
            me.write(brackets.0);
            me.write(&trailing);
            me.output.push('\n');
            me.indent_level += 1;
            me.last_line = 0;
            me.pending_decl_blank = false;
            for item in items {
                let start = start_of(item);
                me.flush_comments_before(start.line);
                me.blank_before(Some(start.line));
                let text = format_item(me, item);
                let end = me
                    .src
                    .as_ref()
                    .and_then(|s| s.index_at(start).map(|i| s.scan_end_line(i, true)));
                let trailing = me.close_line_opt(end);
                me.writeln(&format!("{},{}", text, trailing));
                if let Some(end) = end {
                    me.last_line = end;
                }
            }
            me.flush_comments_before(lines.1);
            me.indent_level -= 1;
            me.write(&me.indent());
            me.write(brackets.1);
        })
    }

    fn format_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Literal { value, span } => self.format_literal(value, Some(*span)),

            Expr::Variable { name, .. } => name.clone(),

            Expr::Assign { name, value, span } => {
                // `x += y` is parsed as `x = x + y` wi' aw three spans the same;
                // print it back compound (an' always when `y` would need brackets)
                if let Expr::Binary {
                    left,
                    operator,
                    right,
                    span: bin_span,
                } = value.as_ref()
                {
                    if let (Expr::Variable { name: lname, .. }, Some(op)) =
                        (left.as_ref(), compound_op(*operator))
                    {
                        if lname == name
                            && (bin_span == span || needs_parens_on_right(*operator, right))
                        {
                            let r = self.format_expr(right);
                            return format!("{} {} {}", name, op, r);
                        }
                    }
                }
                let v = self.format_expr(value);
                format!("{} = {}", name, v)
            }

            Expr::Binary {
                left,
                operator,
                right,
                ..
            } => {
                let l = self.format_expr(left);
                let r = self.format_expr(right);
                format!("{} {} {}", l, operator, r)
            }

            Expr::Unary {
                operator, operand, ..
            } => {
                let o = self.format_expr(operand);
                match operator {
                    // `nae` is only read as "not" when an operand-lookin' token follows
                    UnaryOp::Not if o.starts_with("f\"") || o.starts_with('{') => {
                        format!("!{}", o)
                    }
                    UnaryOp::Not => format!("nae {}", o),
                    UnaryOp::Negate => format!("-{}", o),
                }
            }

            Expr::Logical {
                left,
                operator,
                right,
                ..
            } => {
                let l = self.format_expr(left);
                let r = self.format_expr(right);
                format!("{} {} {}", l, operator, r)
            }

            Expr::Call {
                callee, arguments, ..
            } => {
                let c = self.format_expr(callee);
                let args = self.format_expr_list(arguments);
                format!("{}({})", c, args.join(", "))
            }

            Expr::Get {
                object, property, ..
            } => {
                let o = self.format_expr(object);
                format!("{}.{}", o, property)
            }

            Expr::Set {
                object,
                property,
                value,
                ..
            } => {
                let o = self.format_expr(object);
                let v = self.format_expr(value);
                format!("{}.{} = {}", o, property, v)
            }

            Expr::Index { object, index, .. } => {
                let o = self.format_expr(object);
                let i = self.format_expr(index);
                format!("{}[{}]", o, i)
            }

            Expr::IndexSet {
                object,
                index,
                value,
                ..
            } => {
                let o = self.format_expr(object);
                let i = self.format_expr(index);
                let v = self.format_expr(value);
                format!("{}[{}] = {}", o, i, v)
            }

            Expr::Slice {
                object,
                start,
                end,
                step,
                ..
            } => {
                let o = self.format_expr(object);
                let start_str = match start {
                    Some(s) => self.format_expr(s),
                    None => String::new(),
                };
                let end_str = match end {
                    Some(e) => self.format_expr(e),
                    None => String::new(),
                };
                if let Some(st) = step {
                    let step_str = self.format_expr(st);
                    format!("{}[{}:{}:{}]", o, start_str, end_str, step_str)
                } else {
                    format!("{}[{}:{}]", o, start_str, end_str)
                }
            }

            Expr::List { elements, span } => {
                if !elements.is_empty() {
                    if let Some(lines) = self.multiline_bracket(*span, &TokenKind::LeftBracket) {
                        return self.format_multiline_items(
                            ("[", "]"),
                            elements,
                            lines,
                            leftmost_span,
                            |me, e| me.format_expr(e),
                        );
                    }
                }
                let elems = self.format_expr_list(elements);
                format!("[{}]", elems.join(", "))
            }

            Expr::Dict { pairs, span } => {
                if !pairs.is_empty() {
                    if let Some(lines) = self.multiline_bracket(*span, &TokenKind::LeftBrace) {
                        return self.format_multiline_items(
                            ("{", "}"),
                            pairs,
                            lines,
                            |pair: &(Expr, Expr)| leftmost_span(&pair.0),
                            |me, pair: &(Expr, Expr)| {
                                let k = me.format_expr(&pair.0);
                                let v = me.format_expr(&pair.1);
                                format!("{}: {}", k, v)
                            },
                        );
                    }
                }
                let mut kvs = Vec::with_capacity(pairs.len());
                for (k, v) in pairs {
                    let k = self.format_expr(k);
                    let v = self.format_expr(v);
                    kvs.push(format!("{}: {}", k, v));
                }
                format!("{{{}}}", kvs.join(", "))
            }

            Expr::Range {
                start,
                end,
                inclusive,
                ..
            } => {
                let op = if *inclusive { "..=" } else { ".." };
                let s = self.format_expr(start);
                let e = self.format_expr(end);
                format!("{}{}{}", s, op, e)
            }

            Expr::Grouping { expr, .. } => {
                let inner = self.format_expr(expr);
                format!("({})", inner)
            }

            Expr::Lambda { params, body, .. } => {
                let params_str = params.join(", ");
                let b = self.format_expr(body);
                format!("|{}| {}", params_str, b)
            }

            Expr::Masel { .. } => "masel".to_string(),

            Expr::Input { prompt, .. } => {
                let p = self.format_expr(prompt);
                format!("speir {}", p)
            }

            Expr::FString { parts, .. } => {
                let mut result = String::from("f\"");
                for part in parts {
                    match part {
                        FStringPart::Text(s) => result.push_str(&escape_fstring_text(s)),
                        FStringPart::Expr(e) => {
                            // Hole spans are relative tae the hole, so nae source info here
                            let saved_src = self.src.take();
                            let saved_hole = self.in_fstring_hole;
                            self.in_fstring_hole = true;
                            let inner = self.format_expr(e);
                            self.in_fstring_hole = saved_hole;
                            self.src = saved_src;
                            result.push('{');
                            result.push_str(&encode_fstring_hole(&inner));
                            result.push('}');
                        }
                    }
                }
                result.push('"');
                result
            }

            Expr::Spread { expr, .. } => {
                let inner = self.format_expr(expr);
                format!("...{}", inner)
            }

            Expr::Pipe { left, right, .. } => {
                let l = self.format_expr(left);
                let r = self.format_expr(right);
                format!("{} |> {}", l, r)
            }

            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                let c = self.format_expr(condition);
                let t = self.format_expr(then_expr);
                let e = self.format_expr(else_expr);
                format!("gin {} than {} ither {}", c, t, e)
            }

            Expr::BlockExpr { statements, span } => {
                let lines = self
                    .src
                    .as_ref()
                    .and_then(|s| s.bracket_lines_at(*span, &TokenKind::LeftBrace));
                self.with_buffer(|me| {
                    me.format_braced_body(statements, lines, false);
                })
            }
        }
    }
}

/// Statements that own a braced body
fn is_compound(stmt: &Stmt) -> bool {
    matches!(
        stmt,
        Stmt::Block { .. }
            | Stmt::If { .. }
            | Stmt::While { .. }
            | Stmt::For { .. }
            | Stmt::Function { .. }
            | Stmt::Class { .. }
            | Stmt::TryCatch { .. }
            | Stmt::Match { .. }
    )
}

/// Position o' the first token o' an expression
fn leftmost_span(expr: &Expr) -> Span {
    match expr {
        Expr::Binary { left, .. } | Expr::Logical { left, .. } | Expr::Pipe { left, .. } => {
            leftmost_span(left)
        }
        Expr::Call { callee, .. } => leftmost_span(callee),
        Expr::Get { object, .. }
        | Expr::Set { object, .. }
        | Expr::Index { object, .. }
        | Expr::IndexSet { object, .. }
        | Expr::Slice { object, .. } => leftmost_span(object),
        Expr::Range { start, .. } => leftmost_span(start),
        Expr::Spread { expr, .. } => leftmost_span(expr),
        // Assign's span is its value, on the same line as the name
        other => other.span(),
    }
}

/// Position o' the first token o' a statement
fn stmt_start(stmt: &Stmt) -> Span {
    match stmt {
        Stmt::Expression { expr, .. } => leftmost_span(expr),
        other => other.span(),
    }
}

fn compound_op(op: BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::Add => Some("+="),
        BinaryOp::Subtract => Some("-="),
        BinaryOp::Multiply => Some("*="),
        BinaryOp::Divide => Some("/="),
        _ => None,
    }
}

fn binary_precedence(op: BinaryOp) -> u8 {
    match op {
        BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => 5,
        BinaryOp::Add | BinaryOp::Subtract => 4,
        BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual => 3,
        BinaryOp::Equal | BinaryOp::NotEqual => 2,
    }
}

/// Would `right` need brackets as the right operand o' `op`?
fn needs_parens_on_right(op: BinaryOp, right: &Expr) -> bool {
    match right {
        Expr::Binary { operator, .. } => binary_precedence(*operator) <= binary_precedence(op),
        Expr::Logical { .. }
        | Expr::Ternary { .. }
        | Expr::Pipe { .. }
        | Expr::Assign { .. }
        | Expr::Set { .. }
        | Expr::IndexSet { .. } => true,
        _ => false,
    }
}

/// Print a float so it lexes back tae the same value an' stays a float
fn format_float(f: f64) -> String {
    if f.is_infinite() {
        // A literal too muckle fer f64 lexes as infinity
        let text = if f > 0.0 { "1e999" } else { "-1e999" };
        return text.to_string();
    }
    // Debug is the shortest round-trip form and always has a `.` or an exponent
    format!("{:?}", f)
}

/// Push `c` escaped so the parser's escape processing gies back `c`
fn push_escaped(out: &mut String, c: char, quote: char) {
    match c {
        '\\' => out.push_str("\\\\"),
        '\n' => out.push_str("\\n"),
        '\t' => out.push_str("\\t"),
        '\r' => out.push_str("\\r"),
        '\0' => out.push_str("\\0"),
        c if c == quote => {
            out.push('\\');
            out.push(c);
        }
        c if (c as u32) < 0x20 || c as u32 == 0x7f => {
            out.push_str(&format!("\\x{:02x}", c as u32));
        }
        c => out.push(c),
    }
}

/// Quote a string value so it lexes back tae exactly the same value
fn quote_string(s: &str, quote: char) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push(quote);
    for c in s.chars() {
        push_escaped(&mut out, c, quote);
    }
    out.push(quote);
    out
}

/// Escape the literal text o' an f-string (braces doubled)
fn escape_fstring_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '{' => out.push_str("{{"),
            '}' => out.push_str("}}"),
            c => push_escaped(&mut out, c, '"'),
        }
    }
    out
}

/// Encode formatted hole source fer inside an f-string: the f-string token
/// ends at a bare `"`, and the hole scanner turns `\"` an' `\\` back intae
/// `"` an' `\` (ither backslashes are kept as they are).
fn encode_fstring_hole(inner: &str) -> String {
    let mut out = String::with_capacity(inner.len() + 1);
    // `{{` would read as an escaped brace
    if inner.starts_with('{') {
        out.push(' ');
    }
    let chars: Vec<char> = inner.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' if matches!(chars.get(i + 1), Some('"') | Some('\\') | None) => {
                out.push_str("\\\\")
            }
            c => out.push(c),
        }
    }
    out
}

/// Can `raw` sit between `quote`s as one string token?
fn fits_quotes(raw: &str, quote: char) -> bool {
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if chars.next().is_none() {
                return false;
            }
        } else if c == quote {
            return false;
        }
    }
    true
}

/// Quote a raw (unescaped) token body, as the parser keeps import paths
fn quote_raw(raw: &str) -> String {
    for q in ['"', '\''] {
        if fits_quotes(raw, q) {
            return format!("{}{}{}", q, raw, q);
        }
    }
    format!("\"{}\"", raw)
}

/// A span-free fingerprint o' a program: equal fingerprints mean equal
/// ASTs apart fae line/column positions.
pub fn ast_fingerprint(program: &Program) -> String {
    strip_spans(&format!("{:?}", program))
}

fn stmt_fingerprint(stmt: &Stmt) -> String {
    strip_spans(&format!("{:?}", stmt))
}

/// Replace every `Span { line: N, column: M }` ootside string literals wi' `Span`
fn strip_spans(debug: &str) -> String {
    const PAT: &str = "Span { line: ";
    let mut out = String::with_capacity(debug.len());
    let mut i = 0;
    while i < debug.len() {
        let rest = &debug[i..];
        if rest.starts_with('"') {
            // Copy a Debug string literal whole (its escapes are ASCII)
            let bytes = rest.as_bytes();
            let mut j = 1;
            while j < bytes.len() {
                if bytes[j] == b'\\' {
                    j += 2;
                    continue;
                }
                j += 1;
                if bytes[j - 1] == b'"' {
                    break;
                }
            }
            let j = j.min(rest.len());
            out.push_str(&rest[..j]);
            i += j;
        } else if rest.starts_with(PAT) {
            match span_debug_len(rest) {
                Some(len) => {
                    out.push_str("Span");
                    i += len;
                }
                None => {
                    out.push_str(PAT);
                    i += PAT.len();
                }
            }
        } else {
            let ch = rest.chars().next().expect("non-empty rest");
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

/// Length o' a `Span { line: N, column: M }` at the start o' `s`
fn span_debug_len(s: &str) -> Option<usize> {
    fn digits(s: &str, from: usize) -> usize {
        s[from..].bytes().take_while(|b| b.is_ascii_digit()).count()
    }
    let mut i = "Span { line: ".len();
    let n = digits(s, i);
    if n == 0 {
        return None;
    }
    i += n;
    let mid = ", column: ";
    if !s[i..].starts_with(mid) {
        return None;
    }
    i += mid.len();
    let n = digits(s, i);
    if n == 0 {
        return None;
    }
    i += n;
    if !s[i..].starts_with(" }") {
        return None;
    }
    Some(i + 2)
}

/// Check that `formatted` is a safe replacement fer `source` (parsed as
/// `original`): it must parse tae an equal AST an' keep every comment in order.
pub fn check_round_trip(
    original: &Program,
    source: &str,
    formatted: &str,
) -> Result<(), FormatError> {
    let reparsed = crate::parser::parse(formatted).map_err(|e| FormatError::Refused {
        line: None,
        reason: format!("the tidied code wouldnae parse ({})", e),
    })?;

    if ast_fingerprint(original) != ast_fingerprint(&reparsed) {
        let n = original.statements.len().max(reparsed.statements.len());
        let line = (0..n)
            .find(
                |&i| match (original.statements.get(i), reparsed.statements.get(i)) {
                    (Some(a), Some(b)) => stmt_fingerprint(a) != stmt_fingerprint(b),
                    _ => true,
                },
            )
            .and_then(|i| original.statements.get(i).or(original.statements.last()))
            .map(|s| s.span().line);
        return Err(FormatError::Refused {
            line,
            reason: "the tidied code wouldnae mean the same thing as yer original".to_string(),
        });
    }

    let before = collect_comments(source);
    let after = collect_comments(formatted);
    let mismatch =
        (0..before.len().max(after.len())).find(|&i| match (before.get(i), after.get(i)) {
            (Some(a), Some(b)) => a.text != b.text,
            _ => true,
        });
    if let Some(i) = mismatch {
        return Err(FormatError::Refused {
            line: before.get(i).or(before.last()).map(|c| c.line),
            reason: "a comment wid hae gone missin' or oot o' order".to_string(),
        });
    }

    Ok(())
}

/// Format source code, refusin' unless the result is provably safe:
/// it re-parses tae the same AST (spans aside) an' keeps every comment.
pub fn format_source_checked(source: &str) -> Result<String, FormatError> {
    let program = crate::parser::parse(source).map_err(FormatError::Parse)?;
    let mut formatter = Formatter::new();
    let formatted = formatter.format_with_source(&program, source)?;
    check_round_trip(&program, source, &formatted)?;
    Ok(formatted)
}

/// Format source code (convenience function)
pub fn format_source(source: &str) -> Result<String, crate::error::HaversError> {
    format_source_checked(source).map_err(|e| match e {
        FormatError::Parse(e) => e,
        refused => HaversError::InternalError(refused.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    // ==================== FormatterConfig Tests ====================

    #[test]
    fn test_formatter_config_default() {
        let config = FormatterConfig::default();
        assert_eq!(config.indent_size, 4);
        assert_eq!(config.max_line_width, 100);
    }

    #[test]
    fn test_formatter_with_config() {
        let config = FormatterConfig {
            indent_size: 2,
            max_line_width: 80,
        };
        let formatter = Formatter::with_config(config);
        assert_eq!(formatter.config.indent_size, 2);
    }

    #[test]
    fn test_formatter_default_formats_empty_program_for_coverage() {
        let program = parse("").unwrap();
        let mut formatter = Formatter::default();
        let result = formatter.format(&program);
        assert_eq!(result, "\n");
    }

    // ==================== Variable Declaration Tests ====================

    #[test]
    fn test_format_variable() {
        let source = "ken   x   =    42";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert_eq!(result.trim(), "ken x = 42");
    }

    #[test]
    fn test_format_variable_no_initializer() {
        let source = "ken x";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert_eq!(result.trim(), "ken x");
    }

    // ==================== Function Tests ====================

    #[test]
    fn test_format_function() {
        let source = "dae greet(name){blether name}";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("dae greet(name) {"));
        assert!(result.contains("    blether name"));
        assert!(result.contains("}"));
    }

    #[test]
    fn test_format_function_with_defaults() {
        let source = "dae foo(a, b = 10, c = \"test\") { gie a + b }";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("dae foo(a, b = 10, c = \"test\") {"));
    }

    #[test]
    fn test_format_multiple_functions_extra_newline() {
        let source = r#"dae foo() { gie 1 }
dae bar() { gie 2 }"#;
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        // Should have extra blank line between functions
        assert!(result.contains("}\n\ndae bar"));
    }

    // ==================== Control Flow Tests ====================

    #[test]
    fn test_format_if_else() {
        let source = "gin x > 5 {blether \"big\"} ither {blether \"wee\"}";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("gin x > 5 {"));
        assert!(result.contains("} ither {"));
    }

    #[test]
    fn test_format_if_no_else() {
        let source = "gin x > 5 { blether \"big\" }";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("gin x > 5 {"));
        assert!(!result.contains("ither"));
    }

    #[test]
    fn test_format_while() {
        let source = "whiles x < 10 { ken x = x + 1 }";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("whiles x < 10 {"));
    }

    #[test]
    fn test_format_for() {
        let source = "fer i in 0..10 { blether i }";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("fer i in 0..10 {"));
    }

    #[test]
    fn test_format_break() {
        let source = "whiles aye { brak }";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("brak"));
    }

    #[test]
    fn test_format_continue() {
        let source = "whiles aye { haud }";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("haud"));
    }

    // ==================== Return Statement Tests ====================

    #[test]
    fn test_format_return_with_value() {
        let source = "dae foo() { gie 42 }";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("gie 42"));
    }

    #[test]
    fn test_format_return_no_value() {
        // Return without value needs newline before closing brace
        let source = "dae foo() {\n    gie\n}";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("    gie\n"));
    }

    // ==================== Class Tests ====================

    #[test]
    fn test_format_class() {
        let source = "kin Animal {dae init(name){masel.name = name}}";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("kin Animal {"));
        assert!(result.contains("dae init(name) {"));
    }

    #[test]
    fn test_format_class_with_inheritance() {
        let source = "kin Dog fae Animal { dae bark() { blether \"woof\" } }";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("kin Dog fae Animal {"));
    }

    #[test]
    fn test_format_class_multiple_methods() {
        let source = "kin Calc { dae add(a,b) { gie a+b } dae sub(a,b) { gie a-b } }";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("dae add(a, b)"));
        assert!(result.contains("dae sub(a, b)"));
        // Blank line between methods
        assert!(result.contains("}\n\n    dae sub"));
    }

    #[test]
    fn test_format_multiple_classes_extra_newline() {
        let source = r#"kin A { dae foo() { gie 1 } }
kin B { dae bar() { gie 2 } }"#;
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        // Extra blank line between classes
        assert!(result.contains("}\n\nkin B"));
    }

    // ==================== Struct Tests ====================

    #[test]
    fn test_format_struct() {
        let source = "thing Point { x, y }";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("thing Point { x, y }"));
    }

    // ==================== Import Tests ====================

    #[test]
    fn test_format_import() {
        let source = "fetch \"math\"";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("fetch \"math\""));
    }

    #[test]
    fn test_format_import_with_alias() {
        // MDH-BUG-FLU-00028: the alias keyword is `tae`; `as` doesnae parse
        let source = "fetch \"math\" tae m";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert_eq!(result, "fetch \"math\" tae m\n");
        assert_eq!(format_source(source).unwrap(), "fetch \"math\" tae m\n");
    }

    // ==================== Try-Catch Tests ====================

    #[test]
    fn test_format_try_catch() {
        let source = "hae_a_bash { ken x = 1 / 0 } gin_it_gangs_wrang e { blether e }";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("hae_a_bash {"));
        assert!(result.contains("gin_it_gangs_wrang e {"));
    }

    #[test]
    fn test_format_log_and_hurl_statements_for_coverage() {
        let source = r#"
log_roar "err", 1, 2
log_holler "warn"
log_blether "info"
log_mutter "debug"
log_whisper "trace"
hurl "boom"
"#;
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);

        assert!(result.contains("log_roar \"err\", 1, 2"));
        assert!(result.contains("log_holler \"warn\""));
        assert!(result.contains("log_blether \"info\""));
        assert!(result.contains("log_mutter \"debug\""));
        assert!(result.contains("log_whisper \"trace\""));
        assert!(result.contains("hurl \"boom\""));
    }

    #[test]
    fn test_format_log_wheesht_from_ast_for_coverage() {
        use crate::ast::{Expr, Literal, LogLevel, Program, Span, Stmt};

        let span = Span::new(1, 1);
        let stmt = Stmt::Log {
            level: LogLevel::Wheesht,
            message: Expr::Literal {
                value: Literal::String("silent".to_string()),
                span,
            },
            extras: vec![],
            span,
        };

        let program = Program::new(vec![stmt]);
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);

        assert!(result.contains("log_wheesht \"silent\""));
    }

    // ==================== Match Statement Tests ====================

    #[test]
    fn test_format_match() {
        let source = r#"keek x {
            whan 1 -> blether "one"
            whan 2 -> blether "two"
            whan _ -> blether "other"
        }"#;
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("keek x {"));
        assert!(result.contains("whan 1 ->"));
        assert!(result.contains("whan 2 ->"));
        assert!(result.contains("whan _ ->"));
    }

    #[test]
    fn test_format_match_range_pattern() {
        let source = r#"keek x {
            whan 1..10 -> blether "range"
            whan _ -> blether "other"
        }"#;
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("whan 1..10 ->"));
    }

    #[test]
    fn test_format_match_identifier_pattern_for_coverage() {
        let source = r#"keek x {
            whan y -> blether y
        }"#;
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("whan y ->"));
        assert!(result.contains("blether y"));
    }

    // ==================== Assert Tests ====================

    #[test]
    fn test_format_assert() {
        let source = "mak_siccar x > 0";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("mak_siccar x > 0"));
    }

    #[test]
    fn test_format_assert_with_message() {
        let source = "mak_siccar x > 0, \"x must be positive\"";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("mak_siccar x > 0, \"x must be positive\""));
    }

    // ==================== Destructuring Tests ====================

    #[test]
    fn test_format_destructure_simple() {
        let source = "ken [a, b] = [1, 2]";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("ken [a, b] = [1, 2]"));
    }

    #[test]
    fn test_format_destructure_with_rest() {
        let source = "ken [first, ...rest] = [1, 2, 3, 4]";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("ken [first, ...rest] = [1, 2, 3, 4]"));
    }

    #[test]
    fn test_format_destructure_with_ignore() {
        let source = "ken [_, second, _] = [1, 2, 3]";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("ken [_, second, _] = [1, 2, 3]"));
    }

    // ==================== Expression Tests ====================

    #[test]
    fn test_format_assignment() {
        let source = "ken x = 1\nx = 42";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("x = 42"));
    }

    #[test]
    fn test_format_unary_not() {
        let source = "nae aye";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("nae aye"));
    }

    #[test]
    fn test_format_unary_negate() {
        let source = "-42";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("-42"));
    }

    #[test]
    fn test_format_logical_and() {
        let source = "aye an nae";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("aye an nae"));
    }

    #[test]
    fn test_format_logical_or() {
        let source = "aye or nae";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("aye or nae"));
    }

    #[test]
    fn test_format_call() {
        let source = "foo(1, 2, 3)";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("foo(1, 2, 3)"));
    }

    #[test]
    fn test_format_get_property() {
        let source = "obj.prop";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("obj.prop"));
    }

    #[test]
    fn test_format_set_property() {
        let source = "ken obj = {}\nobj.prop = 42";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("obj.prop = 42"));
    }

    #[test]
    fn test_format_index() {
        let source = "list[0]";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("list[0]"));
    }

    #[test]
    fn test_format_index_set() {
        let source = "ken list = [1,2,3]\nlist[0] = 99";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("list[0] = 99"));
    }

    #[test]
    fn test_format_slice() {
        let source = "list[1:3]";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("list[1:3]"));
    }

    #[test]
    fn test_format_slice_with_step() {
        let source = "list[::2]";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("list[::2]"));
    }

    #[test]
    fn test_format_list() {
        let source = "[1, 2, 3]";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("[1, 2, 3]"));
    }

    #[test]
    fn test_format_dict() {
        // Dict must be assigned to variable
        let source = r#"ken d = {"a": 1, "b": 2}"#;
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        // Dict order may vary
        assert!(result.contains("{"));
        assert!(result.contains("}"));
    }

    #[test]
    fn test_format_range_exclusive() {
        let source = "0..10";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("0..10"));
    }

    #[test]
    fn test_format_range_inclusive() {
        let source = "0..=10";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("0..=10"));
    }

    #[test]
    fn test_format_grouping() {
        let source = "(1 + 2) * 3";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("(1 + 2)"));
    }

    #[test]
    fn test_format_lambda() {
        let source = "|x, y| x + y";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("|x, y| x + y"));
    }

    #[test]
    fn test_format_masel() {
        let source = "kin Foo { dae test() { gie masel } }";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("masel"));
    }

    #[test]
    fn test_format_input() {
        let source = "speir \"What is your name? \"";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("speir \"What is your name? \""));
    }

    #[test]
    fn test_format_fstring() {
        let source = "ken name = \"world\"\nf\"Hello {name}!\"";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("f\"Hello {name}!\""));
    }

    #[test]
    fn test_format_spread() {
        let source = "[1, 2, ...[3, 4]]";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("...[3, 4]"));
    }

    #[test]
    fn test_format_pipe() {
        let source = "5 |> double |> triple";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("|>"));
    }

    #[test]
    fn test_format_ternary() {
        // Ternary expressions must be assigned to a variable
        let source = "ken result = gin x > 0 than \"positive\" ither \"non-positive\"";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("gin x > 0 than \"positive\" ither \"non-positive\""));
    }

    #[test]
    fn test_format_block_expr_for_coverage() {
        let source = r#"
ken x = {
    ken a = 1
    ken b
    brak
    haud
    gie 5
    gin aye { blether "x" }
    a
}
"#;
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);

        assert!(result.contains("ken x = {"));
        assert!(result.contains("ken a = 1"));
        assert!(result.contains("ken b"));
        assert!(result.contains("brak"));
        assert!(result.contains("haud"));
        assert!(result.contains("gie 5"));
        // MDH-BUG-FLU-00027: nested statements are printed, never "..."
        assert!(!result.contains("..."));
        assert!(result.contains("\n    gin aye {\n        blether \"x\"\n    }\n"));
        assert!(result.contains("\n    a\n"));
    }

    // ==================== Block Statement Tests ====================

    #[test]
    fn test_format_block() {
        let source = "{ ken x = 1\n ken y = 2 }";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("{\n"));
        assert!(result.contains("    ken x = 1"));
        assert!(result.contains("    ken y = 2"));
    }

    // ==================== Inline Statement Tests ====================
    // Note: The parser requires blocks after control flow statements,
    // so single inline statements are not testable via parsing.
    // These tests verify the formatter handles block statements correctly
    // when they contain single statements.

    #[test]
    fn test_format_single_stmt_in_if() {
        // Parser requires blocks, but formatter handles them gracefully
        let source = "gin aye { ken x = 1 }";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("gin aye {"));
        assert!(result.contains("ken x = 1"));
    }

    #[test]
    fn test_format_single_stmt_return_in_if() {
        let source = "dae f() { gin aye { gie 42 } }";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("gie 42"));
    }

    #[test]
    fn test_format_single_stmt_print_in_if() {
        let source = "gin aye { blether \"hi\" }";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("blether \"hi\""));
    }

    #[test]
    fn test_format_single_stmt_break_in_while() {
        let source = "whiles aye { brak }";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("brak"));
    }

    #[test]
    fn test_format_single_stmt_continue_in_while() {
        let source = "whiles aye { haud }";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("haud"));
    }

    // ==================== File Ending Tests ====================

    #[test]
    fn test_format_adds_trailing_newline() {
        let source = "ken x = 1";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.ends_with('\n'));
    }

    // ==================== Convenience Function Tests ====================

    #[test]
    fn test_format_source_function() {
        let result = format_source("ken x=42").unwrap();
        assert!(result.contains("ken x = 42"));
    }

    #[test]
    fn test_format_source_error() {
        let result = format_source("ken =");
        assert!(result.is_err());
    }

    // ==================== Round-trip safety (MDH-BUG-FLU-00025..00031) ====================

    /// Format `src`, assert the exact output, and that formatting is idempotent
    fn assert_formats_to(src: &str, expected: &str) {
        let out = format_source_checked(src).unwrap_or_else(|e| panic!("refused: {}", e));
        assert_eq!(out, expected);
        let again = format_source_checked(&out).unwrap_or_else(|e| panic!("refused: {}", e));
        assert_eq!(again, out, "formatting is no' idempotent");
    }

    fn refusal_line(src: &str) -> Option<usize> {
        match format_source_checked(src) {
            Err(FormatError::Refused { line, .. }) => line,
            other => panic!("expected a refusal, got {:?}", other),
        }
    }

    #[test]
    fn fmt_keeps_whole_line_and_trailing_comments() {
        let src = "# heid\nken x=1 # trailin'\n// slashes\nken y = 2\n";
        assert_formats_to(
            src,
            "# heid\nken x = 1  # trailin'\n// slashes\nken y = 2\n",
        );
    }

    #[test]
    fn fmt_keeps_single_blank_lines_and_collapses_runs() {
        let src = "ken a = 1\n\n\n\nken b = 2\nken c = 3\n\n# note\n\nken d = 4\n";
        assert_formats_to(
            src,
            "ken a = 1\n\nken b = 2\nken c = 3\n\n# note\n\nken d = 4\n",
        );
    }

    #[test]
    fn fmt_keeps_comments_inside_blocks_and_at_block_ends() {
        let src = "dae f(x) { # heid\n    # first\n    gin x > 1 {\n        gie 1 # one\n        # efter\n    }\n\n    gie 0\n    # tail\n} # done\nken z = 3\n";
        assert_formats_to(
            src,
            "dae f(x) {  # heid\n    # first\n    gin x > 1 {\n        gie 1  # one\n        # efter\n    }\n\n    gie 0\n    # tail\n}  # done\n\nken z = 3\n",
        );
    }

    #[test]
    fn fmt_keeps_comments_in_multiline_lists_and_dicts() {
        let src = "ken days = [\n    \"Mon\",   # Monday\n    \"Tue\"    # Tuesday\n]\nken d = {\n    # first\n    \"a\": 1,\n\n    \"b\": [1, 2]\n}\n";
        assert_formats_to(
            src,
            "ken days = [\n    \"Mon\",  # Monday\n    \"Tue\",  # Tuesday\n]\nken d = {\n    # first\n    \"a\": 1,\n\n    \"b\": [1, 2],\n}\n",
        );
    }

    #[test]
    fn fmt_keeps_comments_in_class_and_match() {
        let src = "kin A {\n    # a method\n    dae m() {\n        gie 1\n    }\n    dae n() {\n        gie 2\n    }\n}\nkeek x {\n    # one\n    whan 1 -> blether \"one\" # yin\n    whan _ -> {\n        blether \"other\"\n    }\n}\n";
        assert_formats_to(
            src,
            "kin A {\n    # a method\n    dae m() {\n        gie 1\n    }\n\n    dae n() {\n        gie 2\n    }\n}\n\nkeek x {\n    # one\n    whan 1 -> blether \"one\"  # yin\n    whan _ -> {\n        blether \"other\"\n    }\n}\n",
        );
    }

    #[test]
    fn fmt_refuses_comment_inside_an_expression() {
        // A thing body prints on one line, so its comments cannae be placed: refuse, don't drop it
        assert_eq!(refusal_line("thing P {\n    # odd\n    x\n}\n"), Some(2));
        assert_eq!(refusal_line("thing Q {\n    x, # odd\n    y\n}\n"), Some(2));
    }

    #[test]
    fn fmt_plain_format_without_source_still_works() {
        // The AST-only API knows nae comments but must still be faithful
        let program = parse("gin a { blether 1 } ither gin b { blether 2 }").unwrap();
        let out = Formatter::new().format(&program);
        assert_eq!(
            out,
            "gin a {\n    blether 1\n} ither gin b {\n    blether 2\n}\n"
        );
    }

    #[test]
    fn fmt_else_if_chains_are_kept() {
        let src = "gin a { blether 1 } ither gin b { blether 2 } ither gin c {\nblether 3 } ither { blether 4 }\n";
        assert_formats_to(
            src,
            "gin a {\n    blether 1\n} ither gin b {\n    blether 2\n} ither gin c {\n    blether 3\n} ither {\n    blether 4\n}\n",
        );
    }

    #[test]
    fn fmt_lambda_block_bodies_keep_nested_statements() {
        let src = "ken f = |n| {\n  # check\n  gin n > 0 { gie n }\n  fer i in 0..n { blether i }\n  whiles nae done { brak }\n  gie 0\n}\nblether map([1], |x| { gie x * 2 })\n";
        assert_formats_to(
            src,
            "ken f = |n| {\n    # check\n    gin n > 0 {\n        gie n\n    }\n    fer i in 0..n {\n        blether i\n    }\n    whiles nae done {\n        brak\n    }\n    gie 0\n}\nblether map([1], |x| {\n    gie x * 2\n})\n",
        );
    }

    #[test]
    fn fmt_block_bodies_respect_indent_config() {
        let program = parse("dae f() {\nken g = |x| {\ngin x { gie 1 }\n}\n}").unwrap();
        let mut formatter = Formatter::with_config(FormatterConfig {
            indent_size: 2,
            max_line_width: 80,
        });
        let out = formatter.format(&program);
        assert_eq!(
            out,
            "dae f() {\n  ken g = |x| {\n    gin x {\n      gie 1\n    }\n  }\n}\n"
        );
    }

    #[test]
    fn fmt_import_alias_uses_tae() {
        assert_formats_to("fetch \"lib/maths\" tae m\n", "fetch \"lib/maths\" tae m\n");
        assert_formats_to("fetch 'it\"s'\n", "fetch 'it\"s'\n");
    }

    #[test]
    fn fmt_strings_are_re_escaped() {
        let src = "ken a = \"say \\\"hi\\\"\"\nken b = \"back\\\\nslash\"\nken c = 'it\\'s \"quoted\"'\nken d = \"tab\\there\\r\\0\\x1b[0m\"\nken e = \"multi\nline\"\n";
        assert_formats_to(
            src,
            "ken a = \"say \\\"hi\\\"\"\nken b = \"back\\\\nslash\"\nken c = 'it\\'s \"quoted\"'\nken d = \"tab\\there\\r\\0\\x1b[0m\"\nken e = \"multi\\nline\"\n",
        );
        // The values survive exactly
        let out = format_source_checked(src).unwrap();
        let program = parse(&out).unwrap();
        let values: Vec<String> = program
            .statements
            .iter()
            .filter_map(|s| match s {
                Stmt::VarDecl {
                    initializer:
                        Some(Expr::Literal {
                            value: Literal::String(v),
                            ..
                        }),
                    ..
                } => Some(v.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            values,
            vec![
                "say \"hi\"".to_string(),
                "back\\nslash".to_string(),
                "it's \"quoted\"".to_string(),
                "tab\there\r\0\u{1b}[0m".to_string(),
                "multi\nline".to_string(),
            ]
        );
    }

    #[test]
    fn fmt_string_patterns_are_re_escaped() {
        assert_formats_to(
            "keek s {\n    whan \"a\\\"b\" -> blether 1\n    whan -2.5 -> blether 2\n}\n",
            "keek s {\n    whan \"a\\\"b\" -> blether 1\n    whan -2.5 -> blether 2\n}\n",
        );
    }

    #[test]
    fn fmt_floats_stay_floats() {
        assert_formats_to(
            "ken a = 1.0\nken b = 1e300\nken c = 2.5e-10\nken d = 0.1 + 3.0\n",
            "ken a = 1.0\nken b = 1e300\nken c = 2.5e-10\nken d = 0.1 + 3.0\n",
        );
        assert_eq!(format_float(f64::INFINITY), "1e999");
        assert_eq!("1e999".parse::<f64>().unwrap(), f64::INFINITY);
        assert_eq!(format_float(f64::NEG_INFINITY), "-1e999");
    }

    #[test]
    fn fmt_fstrings_round_trip() {
        assert_formats_to(
            "ken s = f\"{{braces}} {d['k']} \\\"q\\\" {n + 1}\"\n",
            "ken s = f\"{{braces}} {d['k']} \\\"q\\\" {n + 1}\"\n",
        );
        // Double-quoted inner literals come oot single-quoted
        assert_formats_to("ken s = f\"{d[\\\"k\\\"]}\"\n", "ken s = f\"{d['k']}\"\n");
    }

    #[test]
    fn fmt_compound_assignment_is_kept() {
        assert_formats_to(
            "x += 1\ny -= a - b\nz = z + 1\nw *= p or q\n",
            "x += 1\ny -= a - b\nz = z + 1\nw *= p or q\n",
        );
    }

    #[test]
    fn fmt_not_before_fstring_uses_bang() {
        assert_formats_to("ken t = !f\"x\"\n", "ken t = !f\"x\"\n");
        assert_formats_to("ken t = !x\n", "ken t = nae x\n");
    }

    #[test]
    fn fmt_round_trip_check_catches_changed_meaning_and_lost_comments() {
        let src = "ken x = 1\nken y = 2 # keep\n";
        let program = parse(src).unwrap();
        // Different meaning
        match check_round_trip(&program, src, "ken x = 1\nken y = 3  # keep\n") {
            Err(FormatError::Refused { line, .. }) => assert_eq!(line, Some(2)),
            other => panic!("expected refusal, got {:?}", other),
        }
        // Disnae parse
        assert!(matches!(
            check_round_trip(&program, src, "ken x = = 1\n"),
            Err(FormatError::Refused { line: None, .. })
        ));
        // Lost comment
        match check_round_trip(&program, src, "ken x = 1\nken y = 2\n") {
            Err(FormatError::Refused { line, .. }) => assert_eq!(line, Some(2)),
            other => panic!("expected refusal, got {:?}", other),
        }
        // Same AST, different spans an' layout, comment kept: fine
        assert!(check_round_trip(&program, src, "\nken x = 1\n\nken y = 2  # keep\n").is_ok());
    }

    #[test]
    fn fmt_fingerprint_ignores_spans_but_not_values() {
        let a = parse("ken x = \"Span { line: 1, column: 2 }\"").unwrap();
        let b = parse("\n\n   ken x = \"Span { line: 1, column: 2 }\"").unwrap();
        let c = parse("ken x = \"Span { line: 3, column: 4 }\"").unwrap();
        assert_eq!(ast_fingerprint(&a), ast_fingerprint(&b));
        assert_ne!(ast_fingerprint(&a), ast_fingerprint(&c));
    }

    #[test]
    fn fmt_format_source_reports_refusal_as_error() {
        let err = format_source("thing P {\n# odd\nx\n}\n").unwrap_err();
        assert!(err.to_string().contains("cannae tidy this safely"));
    }

    #[cfg(coverage)]
    #[test]
    fn formatter_helper_instantiations_are_exercised_for_coverage() {
        use std::hint::black_box;

        let span = Span::new(1, 1);
        let mut formatter = Formatter::new();

        // Call helper methods via fn pointers to discourage inlining under coverage.
        let format_params: fn(&mut Formatter, &[Param]) -> String = Formatter::format_params;
        let params = vec![
            Param {
                name: "a".to_string(),
                default: None,
            },
            Param {
                name: "b".to_string(),
                default: Some(Expr::Literal {
                    value: Literal::Integer(1),
                    span,
                }),
            },
        ];
        let params_str = black_box(format_params)(black_box(&mut formatter), black_box(&params));
        assert_eq!(params_str, "a, b = 1");

        let format_patterns: fn(&Formatter, &[DestructPattern]) -> String =
            Formatter::format_destruct_patterns;
        let patterns = vec![
            DestructPattern::Variable("x".to_string()),
            DestructPattern::Rest("rest".to_string()),
            DestructPattern::Ignore,
        ];
        let patterns_str = black_box(format_patterns)(black_box(&formatter), black_box(&patterns));
        assert_eq!(patterns_str, "x, ...rest, _");

        // Drive format_expr iterator closures (call args, slice start/end/step, list, dict)
        // and format_destruct_patterns via formatting a full program.
        let program = parse(
            r#"
ken [a, ...rest, _] = [1, 2, 3]
dae f(a, b = 1) { gie a }
blether f(1, 2)
blether [1, 2, 3][0:2:1]
blether {"a": 1, "b": 2}
"#,
        )
        .unwrap();

        let mut formatter = Formatter::new();
        let out = formatter.format(&program);
        assert!(out.contains("ken [a, ...rest, _] = [1, 2, 3]"));
        assert!(out.contains("blether f(1, 2)"));
        assert!(out.contains("blether [1, 2, 3][0:2:1]"));
        assert!(out.contains("blether {\"a\": 1, \"b\": 2}"));
    }
}
