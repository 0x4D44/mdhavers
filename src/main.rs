use std::fs;
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use colored::*;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

mod ast;
mod compiler;
mod error;
mod formatter;
mod interpreter;
mod lexer;
mod parser;
mod token;
mod value;

use crate::compiler::compile;
use crate::error::{format_error_context, random_scots_exclamation};
use crate::interpreter::Interpreter;
use crate::parser::parse;

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
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Run { file }) => run_file(&file),
        Some(Commands::Compile { file, output }) => compile_file(&file, output),
        Some(Commands::Repl) => run_repl(),
        Some(Commands::Check { file }) => check_file(&file),
        Some(Commands::Format { file, check }) => format_file(&file, check),
        Some(Commands::Tokens { file }) => show_tokens(&file),
        Some(Commands::Ast { file }) => show_ast(&file),
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
    let program = parse(&source).map_err(|e| format_parse_error(&source, e))?;
    let mut interpreter = Interpreter::new();

    // Set the current directory tae the file's directory fer module resolution
    if let Some(parent) = path.parent() {
        if parent.as_os_str().len() > 0 {
            interpreter.set_current_dir(parent);
        }
    }

    // Load the prelude (standard utility functions)
    interpreter
        .load_prelude()
        .map_err(|e| format!("Error loading prelude: {}", e))?;

    interpreter
        .interpret(&program)
        .map_err(|e| format_runtime_error(&source, e))?;

    Ok(())
}

fn compile_file(path: &PathBuf, output: Option<PathBuf>) -> Result<(), String> {
    let source = read_file(path)?;
    let js_code = compile(&source).map_err(|e| format_parse_error(&source, e))?;

    let output_path = output.unwrap_or_else(|| {
        let mut p = path.clone();
        p.set_extension("js");
        p
    });

    fs::write(&output_path, &js_code)
        .map_err(|e| format!("Cannae write tae {}: {}", output_path.display(), e))?;

    println!(
        "{} Compiled {} tae {}",
        "Bonnie!".green().bold(),
        path.display(),
        output_path.display()
    );

    Ok(())
}

fn run_repl() -> Result<(), String> {
    println!("{}", "═".repeat(50).cyan());
    println!(
        "{}",
        "  mdhavers REPL - A Scots Programming Language".cyan().bold()
    );
    println!("{}", "  Pure havers, but working havers!".cyan());
    println!("{}", "═".repeat(50).cyan());
    println!();
    println!(
        "{}",
        "Type 'help' fer help, 'quit' or 'haud yer wheesht' tae exit.".dimmed()
    );
    println!();

    let mut rl = DefaultEditor::new().map_err(|e| e.to_string())?;
    let mut interpreter = Interpreter::new();

    // Load the prelude fer REPL users
    if let Err(e) = interpreter.load_prelude() {
        eprintln!(
            "{}: Couldnae load prelude: {}",
            "Warning".yellow(),
            e
        );
    }

    loop {
        let readline = rl.readline(&format!("{} ", "mdhavers>".green().bold()));

        match readline {
            Ok(line) => {
                let line = line.trim();

                if line.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(line);

                // Handle special commands
                match line.to_lowercase().as_str() {
                    "quit" | "exit" | "haud yer wheesht" | "bye" | "cheers" => {
                        println!("{}", "Haste ye back! Slàinte!".cyan());
                        break;
                    }
                    "help" | "halp" => {
                        print_repl_help();
                        continue;
                    }
                    "clear" => {
                        print!("\x1B[2J\x1B[1;1H");
                        continue;
                    }
                    _ => {}
                }

                // Try to parse and execute
                match parse(line) {
                    Ok(program) => match interpreter.interpret(&program) {
                        Ok(value) => {
                            // Only print non-nil values
                            if !matches!(value, value::Value::Nil) {
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

    Ok(())
}

fn print_repl_help() {
    println!();
    println!("{}", "═══════════════════════════════════════════════════".cyan());
    println!("{}", "  mdhavers Help - Yer Guide tae Scots Coding".cyan().bold());
    println!("{}", "═══════════════════════════════════════════════════".cyan());
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
    println!("  {}           - show this help", "help".green());
    println!("  {} - exit the REPL", "quit / haud yer wheesht".green());
    println!("  {}          - clear the screen", "clear".green());
    println!();
}

fn check_file(path: &PathBuf) -> Result<(), String> {
    let source = read_file(path)?;

    // Lex
    let tokens = lexer::lex(&source).map_err(|e| format_parse_error(&source, e))?;
    println!(
        "{} Lexing passed ({} tokens)",
        "✓".green(),
        tokens.len()
    );

    // Parse
    let _program = parse(&source).map_err(|e| format_parse_error(&source, e))?;
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
    let formatted = formatter::format_source(&source)
        .map_err(|e| format_parse_error(&source, e))?;

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
            println!(
                "{} {} needs formattin'!",
                "✗".red(),
                path.display()
            );
            Err("File needs formattin'".to_string())
        }
    } else {
        // Write back to file
        fs::write(path, &formatted)
            .map_err(|e| format!("Cannae write tae {}: {}", path.display(), e))?;

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
    let tokens = lexer::lex(&source).map_err(|e| format_parse_error(&source, e))?;

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
    let program = parse(&source).map_err(|e| format_parse_error(&source, e))?;

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

    fs::read_to_string(path).map_err(|e| {
        format!(
            "Dinnae be daft! Cannae read '{}': {}",
            path.display(),
            e
        )
    })
}

fn format_parse_error(source: &str, error: error::HaversError) -> String {
    let mut msg = format!("{}", error);

    if let Some(line) = error.line() {
        msg.push_str("\n\n");
        msg.push_str(&format_error_context(source, line));
    }

    msg
}

fn format_runtime_error(source: &str, error: error::HaversError) -> String {
    let mut msg = format!("{}", error);

    if let Some(line) = error.line() {
        msg.push_str("\n\n");
        msg.push_str(&format_error_context(source, line));
    }

    msg
}
