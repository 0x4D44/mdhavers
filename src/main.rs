use std::fs;
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use colored::*;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use mdhavers::compiler::compile;
use mdhavers::error::{format_error_context, random_scots_exclamation};
use mdhavers::formatter;
use mdhavers::lexer;
use mdhavers::parser::parse;
use mdhavers::wasm_compiler;
use mdhavers::Interpreter;
use mdhavers::Value;

// Crash handler helpers are excluded from source-based coverage runs.
#[cfg(not(coverage))]
use mdhavers::interpreter::{is_crash_handling_enabled, print_stack_trace};

/// Initialize crash handlers for graceful error reporting
#[cfg(not(coverage))]
fn setup_crash_handlers() {
    // Set up panic hook to print stack trace
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        if is_crash_handling_enabled() {
            eprintln!("\n{}", "🏴󠁧󠁢󠁳󠁣󠁴󠁿 Och noo! The program crashed!".red().bold());
            if let Some(location) = panic_info.location() {
                eprintln!(
                    "{}",
                    format!("Panic at {}:{}", location.file(), location.line()).red()
                );
            }
            if let Some(message) = panic_info.payload().downcast_ref::<&str>() {
                eprintln!("{}", format!("Message: {}", message).red());
            } else if let Some(message) = panic_info.payload().downcast_ref::<String>() {
                eprintln!("{}", format!("Message: {}", message).red());
            }
            print_stack_trace();
            eprintln!(
                "\n{}",
                "This shouldnae hae happened! Please report this issue.".yellow()
            );
        }
        default_panic(panic_info);
    }));
}

/// No-op under source-based coverage: we don't deliberately test panic-reporting UX.
#[cfg(coverage)]
fn setup_crash_handlers() {}

/// mdhavers - A Scots programming language
/// Pure havers, but working havers!
#[derive(Parser)]
#[command(name = "mdhavers")]
#[command(author = "Arthur")]
#[command(version = "0.1.0")]
#[command(about = "A Scots programming language - pure havers, but working havers!", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Run a .braw file directly
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a .braw program
    Run {
        /// The .braw file to run
        file: PathBuf,
    },

    /// Compile a .braw program to JavaScript
    Compile {
        /// The .braw file to compile
        file: PathBuf,

        /// Output file (defaults to <input>.js)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Start the interactive REPL
    Repl,

    /// Check a .braw file for errors without running it
    Check {
        /// The .braw file to check
        file: PathBuf,
    },

    /// Format a .braw file (pretty print)
    #[command(name = "fmt")]
    Format {
        /// The .braw file to format
        file: PathBuf,

        /// Just check if formatting is needed (dinnae modify)
        #[arg(long)]
        check: bool,
    },

    /// Show tokens from lexer (for debugging)
    Tokens {
        /// The .braw file to tokenize
        file: PathBuf,
    },

    /// Show AST from parser (for debugging)
    Ast {
        /// The .braw file to parse
        file: PathBuf,
    },

    /// Trace a .braw program (step-by-step execution wi' Scottish commentary)
    Trace {
        /// The .braw file to trace
        file: PathBuf,

        /// Verbose mode - shows expressions and values too
        #[arg(short, long)]
        verbose: bool,
    },

    /// Compile a .braw program to WebAssembly (WAT format)
    Wasm {
        /// The .braw file to compile
        file: PathBuf,

        /// Output file (defaults to <input>.wat)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Run a .wat or .wasm file using the built-in host runner
    #[cfg(feature = "wasm_runner")]
    WasmRun {
        /// The .wat or .wasm file to run
        file: PathBuf,
    },

    /// Build a native executable using LLVM
    Build {
        /// The .braw file to compile
        file: PathBuf,

        /// Output file (defaults to <input> without extension)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Optimization level (0-3)
        #[arg(short = 'O', long, default_value = "2")]
        opt_level: u8,

        /// Emit LLVM IR instead of native binary
        #[arg(long)]
        emit_llvm: bool,
    },
}

fn main() {
    // Set up crash handlers for graceful error reporting
    setup_crash_handlers();

    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Run { file }) => run_file(&file),
        Some(Commands::Compile { file, output }) => compile_file(&file, output),
        Some(Commands::Repl) => run_repl(),
        Some(Commands::Check { file }) => check_file(&file),
        Some(Commands::Format { file, check }) => format_file(&file, check),
        Some(Commands::Tokens { file }) => show_tokens(&file),
        Some(Commands::Ast { file }) => show_ast(&file),
        Some(Commands::Trace { file, verbose }) => trace_file(&file, verbose),
        Some(Commands::Wasm { file, output }) => compile_wasm(&file, output),
        #[cfg(feature = "wasm_runner")]
        Some(Commands::WasmRun { file }) => mdhavers::wasm_runner::run_wasm_file(&file),
        Some(Commands::Build {
            file,
            output,
            opt_level,
            emit_llvm,
        }) => build_native(&file, output, opt_level, emit_llvm),
        None => {
            // If a file is provided directly, run it
            if let Some(file) = cli.file {
                run_file(&file)
            } else {
                // Otherwise, start REPL
                run_repl()
            }
        }
    };

    if let Err(e) = result {
        eprintln!("{}: {}", random_scots_exclamation().red().bold(), e);
        process::exit(1);
    }
}

fn run_file(path: &PathBuf) -> Result<(), String> {
    let source = read_file(path)?;
    let program = match parse(&source) {
        Ok(p) => p,
        Err(e) => return Err(format_parse_error(&source, e)),
    };
    let mut interpreter = Interpreter::new();

    // Set the current file name fer logging
    let filename = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or(path.display().to_string());
    interpreter.set_current_file(&filename);

    // Set the current directory tae the file's directory fer module resolution
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            interpreter.set_current_dir(parent);
        }
    }

    // Load the prelude (standard utility functions)
    if let Err(e) = interpreter.load_prelude() {
        return Err(format!("Error loading prelude: {}", e));
    }

    if let Err(e) = interpreter.interpret(&program) {
        return Err(format_runtime_error(&source, e));
    }

    Ok(())
}

fn trace_file(path: &PathBuf, verbose: bool) -> Result<(), String> {
    use mdhavers::interpreter::TraceMode;

    let source = read_file(path)?;
    let program = match parse(&source) {
        Ok(p) => p,
        Err(e) => return Err(format_parse_error(&source, e)),
    };
    let mut interpreter = Interpreter::new();

    // Set the trace mode
    interpreter.set_trace_mode(if verbose {
        TraceMode::Verbose
    } else {
        TraceMode::Statements
    });

    // Set the current directory fer module resolution
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            interpreter.set_current_dir(parent);
        }
    }

    println!("{}", "═".repeat(60).yellow());
    println!(
        "{}",
        "  🏴󠁧󠁢󠁳󠁣󠁴󠁿 mdhavers Tracer - Watchin' Yer Code Like a Hawk!"
            .yellow()
            .bold()
    );
    if verbose {
        println!("{}", "  Mode: Verbose (showin' everything)".yellow());
    } else {
        println!("{}", "  Mode: Statements only".yellow());
    }
    println!("{}", "═".repeat(60).yellow());
    println!();

    // Load the prelude (but without tracing it - too noisy)
    let saved_mode = interpreter.trace_mode();
    interpreter.set_trace_mode(TraceMode::Off);
    if let Err(e) = interpreter.load_prelude() {
        return Err(format!("Error loading prelude: {}", e));
    }
    interpreter.set_trace_mode(saved_mode);

    // Now run with tracing
    if let Err(e) = interpreter.interpret(&program) {
        return Err(format_runtime_error(&source, e));
    }

    println!();
    println!("{}", "═".repeat(60).yellow());
    println!(
        "{}",
        "  🏴󠁧󠁢󠁳󠁣󠁴󠁿 Trace complete - Pure dead brilliant!".yellow().bold()
    );
    println!("{}", "═".repeat(60).yellow());

    Ok(())
}

fn compile_file(path: &PathBuf, output: Option<PathBuf>) -> Result<(), String> {
    let source = read_file(path)?;
    let js_code = match compile(&source) {
        Ok(js) => js,
        Err(e) => return Err(format_parse_error(&source, e)),
    };

    let output_path = output.unwrap_or_else(|| {
        let mut p = path.clone();
        p.set_extension("js");
        p
    });

    if let Err(e) = fs::write(&output_path, &js_code) {
        return Err(format!("Cannae write tae {}: {}", output_path.display(), e));
    }

    println!(
        "{} Compiled {} tae {}",
        "Bonnie!".green().bold(),
        path.display(),
        output_path.display()
    );

    Ok(())
}

fn compile_wasm(path: &PathBuf, output: Option<PathBuf>) -> Result<(), String> {
    let source = read_file(path)?;
    let wat_code = match wasm_compiler::compile_to_wat(&source) {
        Ok(wat) => wat,
        Err(e) => return Err(format_parse_error(&source, e)),
    };

    let output_path = output.unwrap_or_else(|| {
        let mut p = path.clone();
        p.set_extension("wat");
        p
    });

    if let Err(e) = fs::write(&output_path, &wat_code) {
        return Err(format!("Cannae write tae {}: {}", output_path.display(), e));
    }

    println!(
        "{} Compiled {} tae WebAssembly (WAT)",
        "Braw!".green().bold(),
        path.display()
    );
    println!("  {} {}", "Output:".dimmed(), output_path.display());
    println!();
    println!("{}", "Tae convert tae binary WASM, use:".dimmed());
    println!("  {} wat2wasm {}", "$".dimmed(), output_path.display());

    Ok(())
}

#[cfg(not(feature = "llvm"))]
fn build_native(
    _path: &PathBuf,
    _output: Option<PathBuf>,
    _opt_level: u8,
    _emit_llvm: bool,
) -> Result<(), String> {
    use colored::Colorize;
    eprintln!("{}", "═".repeat(60).yellow());
    eprintln!(
        "{}",
        "  LLVM Native Compilation Not Available".yellow().bold()
    );
    eprintln!("{}", "═".repeat(60).yellow());
    eprintln!();
    eprintln!("The 'build' command requires LLVM support, which wasnae");
    eprintln!("included in this build o' mdhavers.");
    eprintln!();
    eprintln!("{}", "Tae enable native compilation:".cyan().bold());
    eprintln!();
    eprintln!("  1. Install LLVM dependencies:");
    eprintln!(
        "     {}",
        "sudo apt install llvm-15 libpolly-15-dev lld-15 libzstd-dev".green()
    );
    eprintln!();
    eprintln!("  2. Rebuild mdhavers with LLVM:");
    eprintln!("     {}", "make release".green());
    eprintln!(
        "     {} {}",
        "or:".dimmed(),
        "cargo build --release --features llvm".green()
    );
    eprintln!();
    eprintln!(
        "{}",
        "Alternative compilation targets available now:"
            .cyan()
            .bold()
    );
    eprintln!(
        "  {} - Compile tae JavaScript",
        "mdhavers compile <file>".green()
    );
    eprintln!(
        "  {} - Compile tae WebAssembly",
        "mdhavers wasm <file>".green()
    );
    eprintln!();
    Err("LLVM support not available".to_string())
}

#[cfg(feature = "llvm")]
fn build_native(
    path: &PathBuf,
    output: Option<PathBuf>,
    opt_level: u8,
    emit_llvm: bool,
) -> Result<(), String> {
    let source = read_file(path)?;
    let program = match parse(&source) {
        Ok(p) => p,
        Err(e) => return Err(format_parse_error(&source, e)),
    };

    if emit_llvm {
        // Emit LLVM IR
        let compiler = mdhavers::LLVMCompiler::new().with_optimization(opt_level);
        let ir = match compiler.compile_to_ir(&program) {
            Ok(ir) => ir,
            Err(e) => return Err(format!("{}", e)),
        };

        let output_path = output.unwrap_or_else(|| {
            let mut p = path.clone();
            p.set_extension("ll");
            p
        });

        if let Err(e) = fs::write(&output_path, &ir) {
            return Err(format!("Cannae write tae {}: {}", output_path.display(), e));
        }

        println!(
            "{} Compiled {} tae LLVM IR",
            "Braw!".green().bold(),
            path.display()
        );
        println!("  {} {}", "Output:".dimmed(), output_path.display());
    } else {
        // Build native executable
        let output_path = output.unwrap_or_else(|| {
            let mut p = path.clone();
            p.set_extension("");
            p
        });

        let compiler = mdhavers::LLVMCompiler::new();
        if let Err(e) =
            compiler.compile_to_native_with_source(&program, &output_path, opt_level, Some(path))
        {
            return Err(format!("{}", e));
        }

        println!(
            "{} Built native executable from {}",
            "Braw!".green().bold(),
            path.display()
        );
        println!("  {} {}", "Output:".dimmed(), output_path.display());
        println!();
        println!("{}", "Tae run:".dimmed());
        println!("  {} ./{}", "$".dimmed(), output_path.display());
    }

    Ok(())
}

fn repl_needs_more_input(buffer: &str) -> bool {
    let mut in_string = false;
    let mut escape = false;
    let mut in_comment = false;
    let mut braces: i32 = 0;
    let mut brackets: i32 = 0;
    let mut parens: i32 = 0;

    for ch in buffer.chars() {
        if in_comment {
            if ch == '\n' {
                in_comment = false;
            }
            continue;
        }

        if in_string {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '#' => in_comment = true,
            '"' => in_string = true,
            '{' => braces += 1,
            '}' => braces -= 1,
            '[' => brackets += 1,
            ']' => brackets -= 1,
            '(' => parens += 1,
            ')' => parens -= 1,
            _ => {}
        }

        // If nesting goes negative, treat as "not incomplete" so we surface a parse error.
        if braces < 0 || brackets < 0 || parens < 0 {
            return false;
        }
    }

    in_string || braces > 0 || brackets > 0 || parens > 0
}

fn run_repl() -> Result<(), String> {
    use mdhavers::interpreter::TraceMode;

    println!("{}", "═".repeat(50).cyan());
    println!(
        "{}",
        "  mdhavers REPL - A Scots Programming Language"
            .cyan()
            .bold()
    );
    println!("{}", "  Pure havers, but working havers!".cyan());
    println!("{}", "═".repeat(50).cyan());
    println!();
    println!(
        "{}",
        "Type 'help' fer help, 'quit' or 'haud yer wheesht' tae exit.".dimmed()
    );
    println!(
        "{}",
        "Multiline input works for blocks/strings; use ':cancel' tae abandon an unfinished entry."
            .dimmed()
    );
    println!();

    let mut rl = match DefaultEditor::new() {
        Ok(rl) => rl,
        Err(e) => return Err(e.to_string()),
    };

    // Try to load history from file
    let history_path = dirs::home_dir()
        .map(|h| h.join(".mdhavers_history"))
        .unwrap_or(std::path::PathBuf::from(".mdhavers_history"));

    if history_path.exists() {
        let _ = rl.load_history(&history_path);
    }

    let mut interpreter = Interpreter::new();
    let mut trace_enabled = false;
    let mut verbose_trace = false;
    let mut buffer = String::new();

    // Load the prelude fer REPL users
    if let Err(e) = interpreter.load_prelude() {
        eprintln!("{}: Couldnae load prelude: {}", "Warning".yellow(), e);
    }

    loop {
        // Update prompt to show trace mode
        let prompt = if !buffer.is_empty() {
            if trace_enabled {
                if verbose_trace {
                    format!("{} ", "mdhavers[trace:v]...>".yellow().bold())
                } else {
                    format!("{} ", "mdhavers[trace]...>".yellow().bold())
                }
            } else {
                format!("{} ", "mdhavers...>".green().bold())
            }
        } else if trace_enabled {
            if verbose_trace {
                format!("{} ", "mdhavers[trace:v]>".yellow().bold())
            } else {
                format!("{} ", "mdhavers[trace]>".yellow().bold())
            }
        } else {
            format!("{} ", "mdhavers>".green().bold())
        };
        let readline = rl.readline(&prompt);

        match readline {
            Ok(line) => {
                let trimmed = line.trim();

                if trimmed.is_empty() && buffer.is_empty() {
                    continue;
                }

                if !trimmed.is_empty() {
                    let _ = rl.add_history_entry(line.as_str());
                }

                // Handle special commands.
                // While collecting multiline input, only `:`-prefixed commands are recognized so
                // that plain text like "help" can be used inside multiline strings.
                let lower = trimmed.to_lowercase();
                let allow_plain_commands = buffer.is_empty();
                if allow_plain_commands || lower.starts_with(':') {
                    match lower.as_str() {
                        "quit" | "exit" | "haud yer wheesht" | "bye" | "cheers" | ":quit"
                        | ":exit" => {
                            println!("{}", "Haste ye back! Slàinte!".cyan());
                            break;
                        }
                        "help" | "halp" | ":help" => {
                            print_repl_help();
                            continue;
                        }
                        "clear" | "cls" | ":clear" => {
                            print!("\x1B[2J\x1B[1;1H");
                            continue;
                        }
                        ":cancel" | "cancel" => {
                            if !buffer.is_empty() {
                                buffer.clear();
                                println!("{}", "Cancelled multiline input.".dimmed());
                            }
                            continue;
                        }
                        ":reset" | "reset" => {
                            interpreter = Interpreter::new();
                            if let Err(e) = interpreter.load_prelude() {
                                eprintln!("{}: Couldnae load prelude: {}", "Warning".yellow(), e);
                            }
                            buffer.clear();
                            trace_enabled = false;
                            verbose_trace = false;
                            interpreter.set_trace_mode(TraceMode::Off);
                            println!("{}", "Interpreter reset - fresh as a daisy!".green());
                            continue;
                        }
                        ":wisdom" | "wisdom" => {
                            // Print a wee bit of Scots wisdom
                            print_scots_wisdom();
                            continue;
                        }
                        ":codewisdom" | "codewisdom" => {
                            // Print programming-specific Scottish wisdom
                            print_programming_wisdom();
                            continue;
                        }
                        ":examples" | "examples" => {
                            print_repl_examples();
                            continue;
                        }
                        ":trace" | "trace" => {
                            trace_enabled = !trace_enabled;
                            verbose_trace = false;
                            interpreter.set_trace_mode(if trace_enabled {
                                TraceMode::Statements
                            } else {
                                TraceMode::Off
                            });
                            if trace_enabled {
                                println!(
                                    "{}",
                                    "🏴󠁧󠁢󠁳󠁣󠁴󠁿 Trace mode ON - watchin' yer code like a hawk!".yellow()
                                );
                            } else {
                                println!("{}", "Trace mode OFF - back tae normal.".dimmed());
                            }
                            continue;
                        }
                        ":trace v" | "trace v" | ":trace verbose" | "trace verbose" => {
                            trace_enabled = true;
                            verbose_trace = true;
                            interpreter.set_trace_mode(TraceMode::Verbose);
                            println!(
                                "{}",
                                "🏴󠁧󠁢󠁳󠁣󠁴󠁿 Verbose trace mode ON - showin' ye EVERYTHING!".yellow()
                            );
                            continue;
                        }
                        ":vars" | "vars" | ":env" | "env" => {
                            print_environment(&interpreter);
                            continue;
                        }
                        _ => {}
                    }
                }

                // Accumulate input until it looks complete.
                buffer.push_str(&line);
                buffer.push('\n');

                if repl_needs_more_input(&buffer) {
                    continue;
                }

                // Try to parse and execute
                match parse(&buffer) {
                    Ok(program) => match interpreter.interpret(&program) {
                        Ok(value) => {
                            // Only print non-nil values
                            if !matches!(value, Value::Nil) {
                                println!("{} {}", "=>".dimmed(), format!("{}", value).yellow());
                            }
                        }
                        Err(e) => {
                            eprintln!("{}: {}", "Och!".red().bold(), e);
                        }
                    },
                    Err(e) => {
                        eprintln!("{}: {}", "Parse error".red().bold(), e);
                    }
                }

                buffer.clear();
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", "Interrupted! Use 'quit' tae leave.".yellow());
            }
            Err(ReadlineError::Eof) => {
                println!("{}", "Haste ye back! Slàinte!".cyan());
                break;
            }
            Err(err) => {
                eprintln!("{}: {:?}", "Error".red(), err);
                break;
            }
        }
    }

    // Save history on exit
    if let Err(e) = rl.save_history(&history_path) {
        eprintln!("{}: Couldnae save history: {}", "Warning".yellow(), e);
    }

    Ok(())
}

fn print_repl_help() {
    println!();
    println!(
        "{}",
        "═══════════════════════════════════════════════════".cyan()
    );
    println!(
        "{}",
        "  mdhavers Help - Yer Guide tae Scots Coding".cyan().bold()
    );
    println!(
        "{}",
        "═══════════════════════════════════════════════════".cyan()
    );
    println!();

    println!("{}", "Keywords:".yellow().bold());
    println!("  {}  - declare a variable (I know)", "ken".green());
    println!("  {}  - if statement", "gin".green());
    println!("  {}  - else", "ither".green());
    println!("  {}  - while loop", "whiles".green());
    println!("  {}  - for loop", "fer".green());
    println!("  {}  - return from function (give)", "gie".green());
    println!("  {}  - print output (chat)", "blether".green());
    println!("  {}  - get user input (ask)", "speir".green());
    println!("  {}  - define a function (do)", "dae".green());
    println!("  {}  - define a class (family)", "kin".green());
    println!("  {}  - define a struct", "thing".green());
    println!("  {}  - true", "aye".green());
    println!("  {}  - false", "nae".green());
    println!("  {}  - null/none", "naething".green());
    println!("  {}  - logical and", "an".green());
    println!("  {}  - logical or", "or".green());
    println!("  {}  - break from loop", "brak".green());
    println!("  {}  - continue in loop", "haud".green());
    println!();

    println!("{}", "Examples:".yellow().bold());
    println!("  {}", "ken x = 42".dimmed());
    println!("  {}", "gin x > 10 { blether \"big\" }".dimmed());
    println!("  {}", "dae add(a, b) { gie a + b }".dimmed());
    println!("  {}", "fer i in 1..10 { blether i }".dimmed());
    println!();

    println!("{}", "Built-in Functions:".yellow().bold());
    println!("  {}    - length of list/string", "len(x)".green());
    println!("  {}  - type of value", "whit_kind(x)".green());
    println!("  {}  - convert to string", "tae_string(x)".green());
    println!("  {}  - convert to integer", "tae_int(x)".green());
    println!("  {}  - add to list", "shove(list, x)".green());
    println!("  {}  - remove from list", "yank(list)".green());
    println!();

    println!("{}", "REPL Commands:".yellow().bold());
    println!("  {}           - show this help", "help / :help".green());
    println!(
        "  {} - exit the REPL",
        "quit / haud yer wheesht / :quit".green()
    );
    println!("  {}          - clear the screen", "clear / :clear".green());
    println!(
        "  {}          - reset the interpreter",
        "reset / :reset".green()
    );
    println!("  {}        - cancel multiline input", ":cancel".green());
    println!("  {}         - get some Scots wisdom", "wisdom".green());
    println!("  {}     - get programming wisdom", "codewisdom".green());
    println!("  {}       - see example code", "examples".green());
    println!(
        "  {}          - toggle trace mode (debugger)",
        "trace".green()
    );
    println!("  {}       - verbose trace mode", "trace v".green());
    println!("  {}     - show defined variables", "vars / env".green());
    println!();
}

fn print_scots_wisdom() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as usize;

    let proverbs = [
        (
            "Mony a mickle maks a muckle",
            "Many small things add up tae something big",
        ),
        (
            "Lang may yer lum reek",
            "May ye always hae fuel fer yer fire (prosperity)",
        ),
        (
            "Whit's fer ye willnae go by ye",
            "What's meant fer ye will find ye",
        ),
        (
            "A nod's as guid as a wink tae a blind horse",
            "Some hints are pointless",
        ),
        (
            "Dinnae teach yer granny tae suck eggs",
            "Dinnae give advice tae experts",
        ),
        (
            "He wha daes the deil's wark gets the deil's wage",
            "Bad deeds bring bad consequences",
        ),
        (
            "Better a wee fire that warms than a muckle fire that burns",
            "Moderation is best",
        ),
        (
            "Guid gear comes in sma' bulk",
            "Good things come in wee packages",
        ),
        (
            "A blate cat maks a prood moose",
            "Shyness invites boldness in others",
        ),
        (
            "Facts are chiels that winna ding",
            "Ye cannae argue wi' facts",
        ),
        (
            "Ae man's meat is anither man's poison",
            "What works fer one may no' work fer anither",
        ),
        (
            "It's a lang road that has nae turnin'",
            "Things will improve eventually",
        ),
        (
            "Better bend than brek",
            "It's better tae compromise than tae break",
        ),
        ("Frae savin' comes havin'", "Save now, prosper later"),
        (
            "They that dance maun pay the fiddler",
            "Ye must pay fer yer pleasures",
        ),
        ("Oot o' sicht, oot o' mind", "We forget whit we dinnae see"),
        ("A fool an' his money are soon parted", "Dinnae be wasteful"),
        (
            "There's nae fool like an auld fool",
            "Age doesnae always bring wisdom",
        ),
        (
            "Ye cannae mak a silk purse oot o' a soo's lug",
            "Ye cannae improve on poor materials",
        ),
        ("Birds o' a feather flock thegither", "Like attracts like"),
    ];

    let (proverb, meaning) = proverbs[seed % proverbs.len()];
    println!();
    println!(
        "{}",
        "═══════════════════════════════════════════════════".cyan()
    );
    println!("{}", "  A Wee Bit o' Scots Wisdom".cyan().bold());
    println!(
        "{}",
        "═══════════════════════════════════════════════════".cyan()
    );
    println!();
    println!("  \"{}\"", proverb.yellow().italic());
    println!();
    println!("  {}: {}", "Meaning".dimmed(), meaning.dimmed());
    println!();
}

fn print_programming_wisdom() {
    let wisdom = mdhavers::error::scots_programming_wisdom();
    println!();
    println!(
        "{}",
        "═══════════════════════════════════════════════════".cyan()
    );
    println!("{}", "  Scottish Programming Wisdom".cyan().bold());
    println!(
        "{}",
        "═══════════════════════════════════════════════════".cyan()
    );
    println!();
    println!("  \"{}\"", wisdom.yellow().italic());
    println!();
}

fn print_repl_examples() {
    println!();
    println!(
        "{}",
        "═══════════════════════════════════════════════════".cyan()
    );
    println!("{}", "  mdhavers Examples".cyan().bold());
    println!(
        "{}",
        "═══════════════════════════════════════════════════".cyan()
    );
    println!();

    println!("{}", "Variables:".yellow().bold());
    println!("  {}", "ken name = \"Hamish\"".green());
    println!("  {}", "ken age = 42".green());
    println!("  {}", "ken is_braw = aye".green());
    println!();

    println!("{}", "Conditionals:".yellow().bold());
    println!(
        "  {}",
        "gin age > 18 { blether \"Ye're auld enough!\" }".green()
    );
    println!(
        "  {}",
        "gin score > 90 { \"A\" } ither gin score > 70 { \"B\" } ither { \"C\" }".green()
    );
    println!();

    println!("{}", "Loops:".yellow().bold());
    println!("  {}", "fer i in 1..5 { blether i }".green());
    println!("  {}", "whiles x < 10 { x = x + 1 }".green());
    println!();

    println!("{}", "Functions:".yellow().bold());
    println!(
        "  {}",
        "dae greet(name) { gie \"Hullo, \" + name + \"!\" }".green()
    );
    println!("  {}", "greet(\"Scotland\")".green());
    println!();

    println!("{}", "Lists & Dicts:".yellow().bold());
    println!(
        "  {}",
        "ken fruits = [\"apple\", \"banana\", \"cherry\"]".green()
    );
    println!(
        "  {}",
        "ken person = {\"name\": \"Morag\", \"age\": 28}".green()
    );
    println!();

    println!("{}", "Functional:".yellow().bold());
    println!("  {}", "gaun([1, 2, 3], |x| x * 2)".green());
    println!("  {}", "sieve([1, 2, 3, 4], |x| x % 2 == 0)".green());
    println!("  {}", "tumble([1, 2, 3, 4], 0, |acc, x| acc + x)".green());
    println!();
}

fn print_environment(interpreter: &Interpreter) {
    let vars = interpreter.get_user_variables();

    println!();
    println!(
        "{}",
        "═══════════════════════════════════════════════════".cyan()
    );
    println!("{}", "  Yer Variables (Environment)".cyan().bold());
    println!(
        "{}",
        "═══════════════════════════════════════════════════".cyan()
    );
    println!();

    if vars.is_empty() {
        println!(
            "  {}",
            "Nae variables defined yet - use 'ken x = 42' tae create one!".dimmed()
        );
    } else {
        // Separate user values from prelude functions
        let user_vars: Vec<_> = vars.iter().filter(|(_, t, _)| t != "function").collect();
        let user_funcs: Vec<_> = vars.iter().filter(|(_, t, _)| t == "function").collect();

        if !user_vars.is_empty() {
            println!("{}", "  Values:".yellow().bold());
            for (name, type_name, value) in &user_vars {
                // Truncate long values
                let display_value = if value.len() > 50 {
                    format!("{}...", &value[..47])
                } else {
                    value.clone()
                };
                println!(
                    "    {} : {} = {}",
                    name.green(),
                    type_name.dimmed(),
                    display_value.yellow()
                );
            }
            println!();
        }

        if !user_funcs.is_empty() {
            // Only show first few user functions, hide prelude
            let show_funcs: Vec<_> = user_funcs.iter().take(10).collect();
            let hidden = user_funcs.len().saturating_sub(10);

            println!("{}", "  Functions:".yellow().bold());
            for (name, _, _) in show_funcs {
                println!("    {}", name.green());
            }
            if hidden > 0 {
                println!("    {} ... and {} more functions", "".dimmed(), hidden);
            }
        }
    }
    println!();
}

fn check_file(path: &PathBuf) -> Result<(), String> {
    let source = read_file(path)?;

    // Lex
    let tokens = match lexer::lex(&source) {
        Ok(t) => t,
        Err(e) => return Err(format_parse_error(&source, e)),
    };
    println!("{} Lexing passed ({} tokens)", "✓".green(), tokens.len());

    // Parse
    let _program = match parse(&source) {
        Ok(p) => p,
        Err(e) => return Err(format_parse_error(&source, e)),
    };
    println!("{} Parsing passed", "✓".green());

    println!(
        "\n{} {} looks braw!",
        "Bonnie!".green().bold(),
        path.display()
    );

    Ok(())
}

fn format_file(path: &PathBuf, check_only: bool) -> Result<(), String> {
    let source = read_file(path)?;

    // Format the code
    let formatted = match formatter::format_source(&source) {
        Ok(s) => s,
        Err(e) => return Err(format_parse_error(&source, e)),
    };

    if check_only {
        // Just check if formatting would change anything
        if source == formatted {
            println!(
                "{} {} is already formatted braw!",
                "✓".green(),
                path.display()
            );
            Ok(())
        } else {
            println!("{} {} needs formattin'!", "✗".red(), path.display());
            Err("File needs formattin'".to_string())
        }
    } else {
        // Write back to file
        if let Err(e) = fs::write(path, &formatted) {
            return Err(format!("Cannae write tae {}: {}", path.display(), e));
        }

        println!(
            "{} Formatted {} - lookin' braw!",
            "Bonnie!".green().bold(),
            path.display()
        );

        Ok(())
    }
}

fn show_tokens(path: &PathBuf) -> Result<(), String> {
    let source = read_file(path)?;
    let tokens = match lexer::lex(&source) {
        Ok(t) => t,
        Err(e) => return Err(format_parse_error(&source, e)),
    };

    println!("{}", "Tokens:".cyan().bold());
    println!("{}", "─".repeat(50));

    for token in &tokens {
        println!(
            "{:4}:{:2}  {:20} {:?}",
            token.line,
            token.column,
            format!("{}", token.kind).green(),
            token.lexeme.dimmed()
        );
    }

    println!("{}", "─".repeat(50));
    println!("Total: {} tokens", tokens.len());

    Ok(())
}

fn show_ast(path: &PathBuf) -> Result<(), String> {
    let source = read_file(path)?;
    let program = match parse(&source) {
        Ok(p) => p,
        Err(e) => return Err(format_parse_error(&source, e)),
    };

    println!("{}", "AST:".cyan().bold());
    println!("{}", "─".repeat(50));

    for (i, stmt) in program.statements.iter().enumerate() {
        println!("{}. {:?}", i + 1, stmt);
    }

    println!("{}", "─".repeat(50));
    println!("Total: {} top-level statements", program.statements.len());

    Ok(())
}

fn read_file(path: &PathBuf) -> Result<String, String> {
    // Check extension
    if let Some(ext) = path.extension() {
        if ext != "braw" {
            eprintln!(
                "{}: File should have .braw extension, but got .{}",
                "Warning".yellow(),
                ext.to_string_lossy()
            );
        }
    }

    match fs::read_to_string(path) {
        Ok(s) => Ok(s),
        Err(e) => Err(format!(
            "Dinnae be daft! Cannae read '{}': {}",
            path.display(),
            e
        )),
    }
}

fn format_parse_error(source: &str, error: mdhavers::HaversError) -> String {
    let mut msg = format!("{}", error);

    if let Some(line) = error.line() {
        msg.push_str("\n\n");
        msg.push_str(&format_error_context(source, line));
    }

    // Add helpful suggestion if available
    if let Some(suggestion) = mdhavers::error::get_error_suggestion(&error) {
        msg.push('\n');
        msg.push_str(suggestion);
    }

    msg
}

fn format_runtime_error(source: &str, error: mdhavers::HaversError) -> String {
    let mut msg = format!("{}", error);

    if let Some(line) = error.line() {
        msg.push_str("\n\n");
        msg.push_str(&format_error_context(source, line));
    }

    // Add helpful suggestion if available
    if let Some(suggestion) = mdhavers::error::get_error_suggestion(&error) {
        msg.push('\n');
        msg.push_str(suggestion);
    }

    msg
}
