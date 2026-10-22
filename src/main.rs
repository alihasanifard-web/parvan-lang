use colored::*;
use parvan::evaluator::eval_program;
use parvan::lexer::Lexer;
use parvan::object::{self, Environment};
use parvan::parser::Parser;
use parvan::persian::reshape_persian;
use std::cell::RefCell;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;
use std::rc::Rc;

const VERSION: &str = "1.2.0";

#[cfg(target_os = "windows")]
fn set_windows_console_font() {
    use std::mem;

    #[repr(C)]
    struct COORD {
        x: i16,
        y: i16,
    }

    #[repr(C)]
    struct CONSOLE_FONT_INFOEX {
        cb_size: u32,
        n_font: u32,
        dw_font_size: COORD,
        font_family: u32,
        font_weight: u32,
        face_name: [u16; 32],
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetStdHandle(nStdHandle: u32) -> *mut std::ffi::c_void;
        fn SetCurrentConsoleFontEx(hConsoleOutput: *mut std::ffi::c_void, bMaximumWindow: i32, lpConsoleCurrentFontEx: *const CONSOLE_FONT_INFOEX) -> i32;
    }

    unsafe {
        let handle = GetStdHandle(0xFFFFFFF5); // STD_OUTPUT_HANDLE
        let mut cfi: CONSOLE_FONT_INFOEX = mem::zeroed();
        cfi.cb_size = mem::size_of::<CONSOLE_FONT_INFOEX>() as u32;
        cfi.dw_font_size.x = 0;
        cfi.dw_font_size.y = 18; // 18pt font
        cfi.font_family = 54;
        cfi.font_weight = 500;

        // Try Vazirmatn font name
        let font_name = "Vazirmatn";
        let mut face_name = [0u16; 32];
        for (i, c) in font_name.encode_utf16().enumerate().take(31) {
            face_name[i] = c;
        }
        cfi.face_name = face_name;

        SetCurrentConsoleFontEx(handle, 0, &cfi);
    }
}

fn main() {
    #[cfg(target_os = "windows")]
    {
        // Try to set console font to Vazirmatn or Tahoma for beautiful Persian rendering
        set_windows_console_font();
    }

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
    println!();
    println!("  {} {}", "⚡ ParvanLang".bright_cyan().bold(), format!("v{}", VERSION).bright_black());
    println!("  {}  {}", "Parvan Programming Language".bright_white(), reshape_persian("زبان برنامه‌نویسی پروان").green());
    println!("  {}  {}", "Core Team: Ali Hasanifard, Ehsan Fazlinejad, Amirmehdi Shahbazi".bright_black(), reshape_persian("تیم سازنده: علی حسنی‌فرد، احسان فضلی‌نژاد، امیرمهدی شهبازی").bright_black());
    println!();
    println!("  {} / {}", "Usage:".yellow().bold(), reshape_persian("نحوه استفاده:").yellow().bold());
    println!("    parvan                  Interactive Shell / {}", reshape_persian("ورود به محیط تعاملی پروان (REPL)"));
    println!("    parvan <file.prv>       Execute code file / {}", reshape_persian("اجرای فایل کد"));
    println!("    parvan --version        Display version / {}", reshape_persian("نمایش نسخه"));
    println!("    parvan --help           Show help / {}", reshape_persian("راهنمای دستورات"));
    println!();
}

fn run_file(filename: &str) {
    let content = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(err) => {
            let err_msg = format!("خطای خواندن فایل '{}': {}", filename, err);
            eprintln!("  {} {}", "✖".red(), reshape_persian(&err_msg));
            process::exit(1);
        }
    };

    let lexer = Lexer::new(&content);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();

    if !parser.errors.is_empty() {
        eprintln!();
        eprintln!("  {}", reshape_persian("خطاهای نحوی در برنامه:").red().bold());
        for err in parser.errors {
            eprintln!("  {} {}", "•".red(), reshape_persian(&err));
        }
        eprintln!();
        process::exit(1);
    }

    let env = Rc::new(RefCell::new(Environment::new()));
    if let Err(err) = eval_program(&program, env) {
        eprintln!("  {} {}", "✖".red(), reshape_persian(&err));
        process::exit(1);
    }
}

fn run_repl() {
    println!();
    println!("  {} {}", "⚡ ParvanLang".bright_cyan().bold(), format!("v{}", VERSION).bright_black());
    println!(
        "  {} {} | {}",
        "•".bright_green(),
        "Ready - Type exit or خروج to quit".bright_white(),
        reshape_persian("آماده اجرا - برای خروج خروج یا exit بنویسید").bright_black()
    );
    println!();

    let env = Rc::new(RefCell::new(Environment::new()));
    let prompt = format!("  {} ", "›".bright_cyan().bold());

    loop {
        let input_opt = match parvan::rtl_input::RtlLineReader::read_line(&prompt) {
            Ok(opt) => opt,
            Err(_) => {
                // Fallback to standard input if not a tty / error
                print!("{}", prompt);
                io::stdout().flush().unwrap_or(());
                let mut line = String::new();
                if io::stdin().read_line(&mut line).is_err() {
                    break;
                }
                Some(line)
            }
        };

        match input_opt {
            None => break,
            Some(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed == "exit" || trimmed == "خروج" || trimmed == "quit" {
                    println!("  {} / {}", "Goodbye!".bright_green(), reshape_persian("خداحافظ!").bright_green());
                    break;
                }

                let lexer = Lexer::new(trimmed);
                let mut parser = Parser::new(lexer);
                let program = parser.parse_program();

                if !parser.errors.is_empty() {
                    for err in parser.errors {
                        println!("  {} {}", "•".red(), reshape_persian(&err));
                    }
                    continue;
                }

                match eval_program(&program, env.clone()) {
                    Ok(obj) => {
                        if obj != object::Object::Nil {
                            let out_str = obj.to_string();
                            let shaped = reshape_persian(&out_str);
                            println!("  {} {}", "=".bright_black(), shaped.bright_yellow());
                        }
                    }
                    Err(err) => {
                        println!("  {} {}", "✖".red(), reshape_persian(&err));
                    }
                }
            }
        }
    }
}
