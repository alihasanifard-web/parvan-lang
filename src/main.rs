mod ast;
mod evaluator;
mod lexer;
mod object;
mod parser;
mod stdlib;
mod token;

use colored::*;
use evaluator::eval_program;
use lexer::Lexer;
use object::Environment;
use parser::Parser;
use std::cell::RefCell;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;
use std::rc::Rc;

const VERSION: &str = "1.0.0 (Cyberpunk Edition)";
const BANNER: &str = r#"
  ██████╗  █████╗ ██████╗ ██╗   ██╗ █████╗ ███╗   ██╗
  ██╔══██╗██╔══██╗██╔══██╗██║   ██║██╔══██╗████╗  ██║
  ██████╔╝███████║██████╔╝██║   ██║███████║██╔██╗ ██║
  ██╔═══╝ ██╔══██║██╔══██╗╚██╗ ██╔╝██╔══██║██║╚██╗██║
  ██║     ██║  ██║██║  ██║ ╚████╔╝ ██║  ██║██║ ╚████║
  ╚═╝     ╚═╝  ╚═╝╚═╝  ╚═╝  ╚═══╝  ╚═╝  ╚═╝╚═╝  ╚═══╝
"#;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let first_arg = &args[1];
        if first_arg == "--help" || first_arg == "-h" {
            print_help();
            return;
        }
        if first_arg == "--version" || first_arg == "-v" {
            println!("ParvanLang v{}", VERSION);
            return;
        }
        // Run file
        run_file(first_arg);
    } else {
        // Run interactive REPL
        run_repl();
    }
}

fn print_help() {
    println!("{}", BANNER.cyan().bold());
    println!("{}", "زبان برنامه‌نویسی پروان (Parvan Programming Language)".green().bold());
    println!("سازندگان: علی حسنی‌فرد، احسان فضلی‌نژاد، امیرمهدی شهبازی\n");
    println!("{}", "نحوه استفاده:".yellow().bold());
    println!("  parvan                   ورود به محیط تعاملی و زنده (REPL)");
    println!("  parvan <file.prv>        اجرای فایل کد پروان");
    println!("  parvan --help            نمایش این راهنما");
    println!("  parvan --version         نمایش نسخه مفسر");
}

fn run_file(filename: &str) {
    let content = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("{} خطای خواندن فایل '{}': {}", "❌".red(), filename, err);
            process::exit(1);
        }
    };

    let lexer = Lexer::new(&content);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();

    if !parser.errors.is_empty() {
        eprintln!("{}", "خطاهای نحو در برنامه:".red().bold());
        for err in parser.errors {
            eprintln!("  {} {}", "•".red(), err);
        }
        process::exit(1);
    }

    let env = Rc::new(RefCell::new(Environment::new()));
    if let Err(err) = eval_program(&program, env) {
        eprintln!("{} {}", "خطای اجرا:".red().bold(), err);
        process::exit(1);
    }
}

fn run_repl() {
    println!("{}", BANNER.cyan().bold());
    println!(
        "{} {} - {}",
        "⚡ ParvanLang".bright_cyan().bold(),
        format!("v{}", VERSION).yellow(),
        "محیط تعاملی فرمان پروان".white()
    );
    println!(
        "توسعه‌یافته توسط: {} | {} | {}",
        "علی حسنی‌فرد".bright_green(),
        "احسان فضلی‌نژاد".bright_green(),
        "امیرمهدی شهبازی".bright_green()
    );
    println!("برای خروج عبارت {} یا {} را بنویسید.\n", "exit".red(), "خروج".red());

    let env = Rc::new(RefCell::new(Environment::new()));

    loop {
        print!("{} ", "parvan>".bright_magenta().bold());
        io::stdout().flush().unwrap_or(());

        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed == "exit" || trimmed == "خروج" || trimmed == "quit" {
                    println!("{}", "خداحافظ! 🚀".green());
                    break;
                }

                let lexer = Lexer::new(trimmed);
                let mut parser = Parser::new(lexer);
                let program = parser.parse_program();

                if !parser.errors.is_empty() {
                    for err in parser.errors {
                        println!("  {} {}", "•".red(), err);
                    }
                    continue;
                }

                match eval_program(&program, env.clone()) {
                    Ok(obj) => {
                        if obj != object::Object::Nil {
                            println!("{} {}", "=>".bright_black(), obj.to_string().bright_yellow());
                        }
                    }
                    Err(err) => {
                        println!("{} {}", "خطا:".red().bold(), err);
                    }
                }
            }
            Err(err) => {
                eprintln!("خطای ورودی: {}", err);
                break;
            }
        }
    }
}
