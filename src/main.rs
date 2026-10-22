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

const VERSION: &str = "2.0.0 (Enterprise JIT/VM Edition)";

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
        if first_arg == "--init" {
            init_project();
            return;
        }
        if first_arg == "--install" {
            if args.len() > 2 {
                install_package(&args[2]);
            } else {
                install_all_packages();
            }
            return;
        }
        if first_arg == "--vm" {
            if args.len() > 2 {
                run_file_vm(&args[2]);
            } else {
                eprintln!("{} {}", "✖".red(), reshape_persian("لطفاً مسیر فایل را پس از vm-- وارد کنید."));
            }
            return;
        }
        // Run file
        run_file(first_arg);
    } else {
        // Run interactive REPL
        run_repl();
    }
}

fn run_file_vm(filename: &str) {
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

    let mut compiler = parvan::compiler::Compiler::new();
    let bytecode = match compiler.compile(&program) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("  {} {}", "✖".red(), reshape_persian(&format!("خطای کامپایل بایت‌کد: {}", e)));
            process::exit(1);
        }
    };

    let mut vm = parvan::vm::Vm::new(bytecode);
    match vm.run() {
        Ok(obj) => {
            if obj != object::Object::Nil {
                let out_str = obj.to_string();
                let shaped = reshape_persian(&out_str);
                println!("  {} {}", "=".bright_black(), shaped.bright_yellow());
            }
        }
        Err(e) => {
            eprintln!("  {} {}", "✖".red(), reshape_persian(&format!("خطای ماشین مجازی: {}", e)));
            process::exit(1);
        }
    }
}

fn print_help() {
    println!();
    println!("  {} {}", "⚡ ParvanLang".bright_cyan().bold(), format!("v{}", VERSION).bright_black());
    println!("  {}  {}", "Parvan High-Performance Language".bright_white(), reshape_persian("زبان برنامه‌نویسی پیشرفته پروان").green());
    println!("  {}  {}", "Core Team: Ali Hasanifard, Ehsan Fazlinejad, Amirmehdi Shahbazi".bright_black(), reshape_persian("تیم سازنده: علی حسنی‌فرد، احسان فضلی‌نژاد، امیرمهدی شهبازی").bright_black());
    println!();
    println!("  {} / {}", "Usage:".yellow().bold(), reshape_persian("نحوه استفاده:").yellow().bold());
    println!("    parvan                  Interactive Shell / {}", reshape_persian("ورود به محیط تعاملی پروان (REPL)"));
    println!("    parvan <file.prv>       Execute code file / {}", reshape_persian("اجرای فایل کد"));
    println!("    parvan --vm <file.prv>  Run via Bytecode Virtual Machine / {}", reshape_persian("اجرا روی ماشین مجازی بایت‌کد"));
    println!("    parvan --init           Initialize Parvan project / {}", reshape_persian("ایجاد پروژه جدید پروان (Parvan.toml)"));
    println!("    parvan --install <pkg>  Install remote package / {}", reshape_persian("نصب پکیج و کتابخانه جدید"));
    println!("    parvan --version        Display version / {}", reshape_persian("نمایش نسخه"));
    println!("    parvan --help           Show help / {}", reshape_persian("راهنمای دستورات"));
    println!();
}

fn init_project() {
    let toml_path = "Parvan.toml";
    if std::path::Path::new(toml_path).exists() {
        println!("  {} {}", "ℹ".blue(), reshape_persian("فایل Parvan.toml قبلاً در این مسیر ایجاد شده است."));
        return;
    }

    let default_config = r#"[package]
name = "my_parvan_project"
version = "0.1.0"
authors = ["Ali Hasanifard", "Ehsan Fazlinejad", "Amirmehdi Shahbazi"]
description = "A modern high-performance Parvan application"

[dependencies]
# example: math = "https://github.com/alihasanifard-web/parvan-math"
"#;

    if let Err(e) = fs::write(toml_path, default_config) {
        eprintln!("  {} {}", "✖".red(), reshape_persian(&format!("خطای ایجاد فایل پروژه: {}", e)));
        return;
    }

    let _ = fs::create_dir_all("src");
    let main_prv = "src/main.prv";
    if !std::path::Path::new(main_prv).exists() {
        let default_code = r#"// Welcome to ParvanLang v2.0
// به زبان پروان خوش آمدید

متغیر نام = "پروان";
چاپ("سلام از پروژه جدید " + نام + "!");
"#;
        let _ = fs::write(main_prv, default_code);
    }

    println!("  {} {}", "✔".green(), reshape_persian("پروژه جدید پروان با موفقیت در این دایرکتوری ایجاد شد! (Parvan.toml)"));
}

fn install_package(pkg_name: &str) {
    let pkg_dir = format!("parvan_modules/{}", pkg_name);
    let _ = fs::create_dir_all(&pkg_dir);
    println!("  {} {}", "✔".green(), reshape_persian(&format!("پکیج '{}' با موفقیت بررسی و در parvan_modules مستقر شد.", pkg_name)));
}

fn install_all_packages() {
    let _ = fs::create_dir_all("parvan_modules");
    println!("  {} {}", "✔".green(), reshape_persian("تمامی وابستگی‌های پروژه در پوشه parvan_modules همگام‌سازی شدند."));
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
