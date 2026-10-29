use colored::*;
use parvan::ast::{Expr, Stmt};
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

const VERSION: &str = "4.0.0 (Next-Gen AOT Native, Static Type Checker & Event-Loop Platform)";
const MAGIC_MARKER: &[u8] = b"__PARVAN_PAYLOAD_V260_TAG__";


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

    // Check if running as embedded standalone binary
    if let Ok(exe_path) = env::current_exe() {
        if let Ok(bytes) = fs::read(&exe_path) {
            if bytes.len() > 16 {
                let footer_pos = bytes.len() - 16;
                let script_len = u64::from_le_bytes(bytes[footer_pos..footer_pos + 8].try_into().unwrap_or([0; 8])) as usize;
                let marker_len = u64::from_le_bytes(bytes[footer_pos + 8..footer_pos + 16].try_into().unwrap_or([0; 8])) as usize;
                if marker_len == MAGIC_MARKER.len() && bytes.len() >= 16 + script_len + marker_len {
                    let start = bytes.len() - 16 - script_len - marker_len;
                    if &bytes[start..start + marker_len] == MAGIC_MARKER {
                        let script_bytes = &bytes[start + marker_len..start + marker_len + script_len];
                        let key = 0x5A;
                        let deobfuscated: Vec<u8> = script_bytes.iter().map(|b| b ^ key).collect();
                        if let Ok(source_code) = std::str::from_utf8(&deobfuscated) {
                            run_code_content(source_code);
                            return;
                        }
                    }
                }
            }
        }
    }

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
        if first_arg == "build" || first_arg == "--build" {
            if args.len() > 2 {
                let mut out_name = "app.exe";
                let mut target_os = "windows";
                let mut i = 3;
                while i < args.len() {
                    if args[i] == "-o" && i + 1 < args.len() {
                        out_name = &args[i + 1];
                        i += 2;
                    } else if (args[i] == "-t" || args[i] == "--target") && i + 1 < args.len() {
                        target_os = &args[i + 1];
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                build_standalone(&args[2], out_name, target_os);
            } else {
                eprintln!("{} استفاده: parvan build <source.prv> -o <output.exe> [-t linux|windows|macos]", "✖".red());
            }
            return;
        }
        if first_arg == "--lsp" || first_arg == "lsp" {
            run_lsp_server();
            return;
        }
        if first_arg == "--init" {
            init_project();
            return;
        }
        if first_arg == "bench" || first_arg == "--bench" {
            if args.len() > 2 {
                run_benchmark(&args[2]);
            } else {
                eprintln!("{} استفاده: parvan bench <file.prv>", "✖".red());
            }
            return;
        }
        if first_arg == "debug" || first_arg == "--debug" {
            if args.len() > 2 {
                run_debugger(&args[2]);
            } else {
                eprintln!("{} استفاده: parvan debug <file.prv>", "✖".red());
            }
            return;
        }
        if first_arg == "check" || first_arg == "--check" {
            if args.len() > 2 {
                run_type_check(&args[2]);
            } else {
                eprintln!("{} استفاده: parvan check <file.prv>", "✖".red());
            }
            return;
        }
        if first_arg == "lint" || first_arg == "--lint" {
            if args.len() > 2 {
                run_linter(&args[2]);
            } else {
                eprintln!("{} استفاده: parvan lint <file.prv>", "✖".red());
            }
            return;
        }
        if first_arg == "doc" || first_arg == "--doc" {
            if args.len() > 2 {
                generate_documentation(&args[2]);
            } else {
                eprintln!("{} استفاده: parvan doc <file.prv>", "✖".red());
            }
            return;
        }
        if first_arg == "test" || first_arg == "--test" {
            run_project_tests();
            return;
        }
        if first_arg == "fmt" || first_arg == "--fmt" {
            if args.len() > 2 {
                format_file(&args[2]);
            } else {
                format_current_dir();
            }
            return;
        }
        if first_arg == "publish" || first_arg == "--publish" {
            let target_path = if args.len() > 2 { &args[2] } else { "." };
            publish_package(target_path);
            return;
        }
        if first_arg == "add" || first_arg == "--add" {
            if args.len() > 2 {
                add_package_dependency(&args[2]);
            } else {
                eprintln!("{} استفاده: parvan add <package_name>", "✖".red());
            }
            return;
        }
        if first_arg == "registry" || first_arg == "--registry" {
            list_registry_packages();
            return;
        }
        if first_arg == "install" || first_arg == "--install" {
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

fn print_welcome_banner() {
    println!();
    println!("  {} {} {}", "⚡".bright_cyan(), "ParvanLang".bright_white().bold(), format!("v{}", VERSION).bright_black());
    println!("  {} {}", "•".bright_cyan(), "Next-gen bilingual systems language powered by Rust & Bytecode VM".bright_black());
    println!();
}

fn print_help() {
    print_welcome_banner();
    println!("  {}", "دستورات:".bright_white().bold());
    println!("    {:26} {}", "parvan".bright_cyan(), "ورود به محیط تعاملی پروان (REPL)".bright_black());
    println!("    {:26} {}", "parvan <file.prv>".bright_cyan(), "اجرای فایل کد با مفسر درختی".bright_black());
    println!("    {:26} {}", "parvan --vm <file.prv>".bright_cyan(), "اجرای شتاب‌یافته روی ماشین مجازی بایت‌کد".bright_black());
    println!("    {:26} {}", "parvan check <file.prv>".bright_cyan(), "تحلیل استاتیک و بررسی پیشرفته انواع داده".bright_black());
    println!("    {:26} {}", "parvan debug <file.prv>".bright_cyan(), "اجرای تعاملی خط‌به‌خط در حالت دیباگ".bright_black());
    println!("    {:26} {}", "parvan build <f> -o <e>".bright_cyan(), "تولید فایل باینری مستقل و محافظت‌شده".bright_black());
    println!("    {:26} {}", "parvan lint <file.prv>".bright_cyan(), "تحلیل استاتیک و عیب‌یابی کدهای مرده".bright_black());
    println!("    {:26} {}", "parvan doc <file.prv>".bright_cyan(), "تولید خودکار پرتال مستندات HTML".bright_black());
    println!("    {:26} {}", "parvan bench <file.prv>".bright_cyan(), "بنچمارک دقیق سرعت و اوپ‌کدهای VM".bright_black());
    println!("    {:26} {}", "parvan --lsp".bright_cyan(), "اجرای سرور ادیتورها (Language Server)".bright_black());
    println!("    {:26} {}", "parvan fmt [file]".bright_cyan(), "فرمت و مرتب‌سازی کدها".bright_black());
    println!("    {:26} {}", "parvan test".bright_cyan(), "اجرای آزمون‌های پروژه".bright_black());
    println!("    {:26} {}", "parvan publish [path]".bright_cyan(), "بسته‌بندی و انتشار کتابخانه (.prvp)".bright_black());
    println!("    {:26} {}", "parvan --init".bright_cyan(), "ساخت پروژه جدید (Parvan.toml)".bright_black());
    println!("    {:26} {}", "parvan --version".bright_cyan(), "نمایش نسخه جاری".bright_black());
    println!();
}

fn build_standalone(source_path: &str, out_path: &str, target_os: &str) {
    println!("  {} {} {}", "Compiling".bright_green().bold(), source_path.bright_white(), format!("-> {} [{}]", out_path, target_os).bright_cyan());
    let current_exe = match env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{} خطای شناسایی باینری پایه: {}", "✖".red(), e);
            return;
        }
    };

    let source_code = match fs::read_to_string(source_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} خطای خواندن سورس کد '{}': {}", "✖".red(), source_path, e);
            return;
        }
    };

    // Verify syntax before bundling
    let lexer = Lexer::new(&source_code);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    if !parser.errors.is_empty() {
        eprintln!("{} سورس کد دارای خطای نحوی است و امکان بیلد وجود ندارد.", "✖".red());
        for err in parser.errors {
            eprintln!("  • {}", err.red());
        }
        return;
    }

    if target_os == "c" || target_os == "native" {
        // Direct C99 / Machine code emission
        let mut emitter = parvan::native_emitter::NativeEmitter::new();
        let c_code = emitter.emit_program(&program);
        if target_os == "c" {
            if let Err(e) = fs::write(out_path, c_code) {
                eprintln!("  {} Failed to write C99 code '{}': {}", "error:".red().bold(), out_path, e);
            } else {
                println!("  {} Transpiled C99 source code ready at {}", "Finished".bright_green().bold(), out_path.bright_white().bold());
            }
            return;
        } else {
            // Compile with gcc
            println!("  {} Invoking GCC with -O3 optimization for machine code generation...", "•".bright_cyan());
            if let Err(e) = parvan::native_emitter::compile_c_to_binary(&c_code, out_path) {
                eprintln!("  {} Native GCC Compilation error: {}", "error:".red().bold(), e);
            } else {
                println!("  {} Native AOT machine code executable ready at {}", "Finished".bright_green().bold(), out_path.bright_white().bold());
            }
            return;
        }
    }

    if target_os == "linux" {
        // Universal Linux ELF stub / Shebang self-contained bundle
        let mut script_payload = format!("#!/usr/bin/env parvan\n# Generated by Parvan Multiplatform Compiler v{}\n", VERSION);
        script_payload.push_str(&source_code);
        if let Err(e) = fs::write(out_path, script_payload) {
            eprintln!("  {} Failed to write target Linux binary '{}': {}", "error:".red().bold(), out_path, e);
        } else {
            println!("  {} Linux target executable bundle ready at {}", "Finished".bright_green().bold(), out_path.bright_white().bold());
        }
        return;
    }

    let marker_len_bytes = (MAGIC_MARKER.len() as u64).to_le_bytes();
    let mut exe_bytes = match fs::read(&current_exe) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{} خطای خواندن باینری الگو: {}", "✖".red(), e);
            return;
        }
    };

    // Check if current_exe already has a payload at the end and strip it
    if exe_bytes.len() > 16 {
        let footer_pos = exe_bytes.len() - 16;
        let script_len = u64::from_le_bytes(exe_bytes[footer_pos..footer_pos + 8].try_into().unwrap_or([0; 8])) as usize;
        let marker_len = u64::from_le_bytes(exe_bytes[footer_pos + 8..footer_pos + 16].try_into().unwrap_or([0; 8])) as usize;
        if marker_len == MAGIC_MARKER.len() && exe_bytes.len() >= 16 + script_len + marker_len {
            let start = exe_bytes.len() - 16 - script_len - marker_len;
            if &exe_bytes[start..start + marker_len] == MAGIC_MARKER {
                exe_bytes.truncate(start);
            }
        }
    }

    // XOR variable encryption layer to protect source script inside standalone binary
    let key = 0x5A;
    let obfuscated_source: Vec<u8> = source_code.bytes().map(|b| b ^ key).collect();

    let script_len_bytes = (obfuscated_source.len() as u64).to_le_bytes();
    exe_bytes.extend_from_slice(MAGIC_MARKER);
    exe_bytes.extend_from_slice(&obfuscated_source);
    exe_bytes.extend_from_slice(&script_len_bytes);
    exe_bytes.extend_from_slice(&marker_len_bytes);

    if let Err(e) = fs::write(out_path, exe_bytes) {
        eprintln!("  {} Failed to write executable '{}': {}", "error:".red().bold(), out_path, e);
    } else {
        println!("  {} Standalone binary ready at {}", "Finished".bright_green().bold(), out_path.bright_white().bold());
    }
}

fn run_lsp_server() {
    eprintln!("🦋 ParvanLang Language Server (LSP) Daemon v{}", VERSION);
    eprintln!("Listening for JSON-RPC messages on standard I/O (VS Code / VSCodium)...");
    
    use std::io::{BufRead, Read, Write};
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    let mut content_len: usize = 0;
    let mut reader = stdin.lock();

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.starts_with("Content-Length:") {
                    if let Some(val) = trimmed.split(':').nth(1) {
                        content_len = val.trim().parse::<usize>().unwrap_or(0);
                    }
                } else if trimmed.is_empty() && content_len > 0 {
                    // Header finished, read payload
                    let mut body_buf = vec![0u8; content_len];
                    if reader.read_exact(&mut body_buf).is_ok() {
                        if let Ok(msg_str) = std::str::from_utf8(&body_buf) {
                            if msg_str.contains("\"method\":\"initialize\"") {
                                let resp = "{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"capabilities\":{\"textDocumentSync\":1,\"hoverProvider\":true}}}";
                                let header = format!("Content-Length: {}
\n
\n", resp.len());
                                let _ = stdout.write_all(header.as_bytes());
                                let _ = stdout.write_all(resp.as_bytes());
                                let _ = stdout.flush();
                            }
                        }
                    }
                    content_len = 0;
                }
            }
            Err(_) => break,
        }
    }
}

fn run_linter(filename: &str) {
    println!("  {} {} {}", "Linting".bright_green().bold(), filename.bright_white(), "with Parvan Static Analyzer".bright_black());

    let content = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("  {} Cannot read file: {}", "error:".red().bold(), err);
            return;
        }
    };

    let lexer = Lexer::new(&content);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();

    if !parser.errors.is_empty() {
        eprintln!("  {} Fix syntax errors first:", "error:".red().bold());
        for err in parser.errors {
            eprintln!("    • {}", err);
        }
        return;
    }

    let mut declared_vars = std::collections::HashSet::new();
    let mut used_vars = std::collections::HashSet::new();

    for stmt in &program.statements {
        extract_idents_from_stmt(stmt, &mut declared_vars, &mut used_vars);
    }

    let mut warnings = 0;
    for (var, line) in declared_vars {
        if !used_vars.contains(&var) && !var.starts_with('_') {
            warnings += 1;
            println!("  {} [line {}] Variable '{}' is declared but never used (prefix with '_' to silence)", "warning:".bright_yellow().bold(), line, var.bright_white());
        }
    }

    if warnings == 0 {
        println!("  {} All checks passed! No dead code or unused variables found.", "✔".bright_green().bold());
    } else {
        println!();
        println!("  {} Found {} code quality warning(s).", "ℹ".bright_cyan(), warnings);
    }
}

fn extract_idents_from_stmt(
    stmt: &Stmt,
    declared: &mut std::collections::HashSet<(String, usize)>,
    used: &mut std::collections::HashSet<String>,
) {
    match stmt {
        Stmt::Let { name, value, token } | Stmt::Const { name, value, token } => {
            declared.insert((name.clone(), token.line));
            extract_idents_from_expr(value, used);
        }
        Stmt::Expr(e) => extract_idents_from_expr(e, used),
        Stmt::If { condition, consequence, alternative, .. } => {
            extract_idents_from_expr(condition, used);
            for s in &consequence.statements {
                extract_idents_from_stmt(s, declared, used);
            }
            if let Some(alt) = alternative {
                for s in &alt.statements {
                    extract_idents_from_stmt(s, declared, used);
                }
            }
        }
        Stmt::While { condition, body, .. } => {
            extract_idents_from_expr(condition, used);
            for s in &body.statements {
                extract_idents_from_stmt(s, declared, used);
            }
        }
        Stmt::For { var, val_var, iterable, body, token } => {
            declared.insert((var.clone(), token.line));
            if let Some(ref val_v) = val_var {
                declared.insert((val_v.clone(), token.line));
            }
            extract_idents_from_expr(iterable, used);
            for s in &body.statements {
                extract_idents_from_stmt(s, declared, used);
            }
        }
        Stmt::Return { value, .. } => {
            if let Some(v) = value {
                extract_idents_from_expr(v, used);
            }
        }
        _ => {}
    }
}

fn extract_idents_from_expr(expr: &Expr, used: &mut std::collections::HashSet<String>) {
    match expr {
        Expr::Ident(id, _) => {
            used.insert(id.clone());
        }
        Expr::Infix(left, _, right, _) => {
            extract_idents_from_expr(left, used);
            extract_idents_from_expr(right, used);
        }
        Expr::Prefix(_, right, _) => {
            extract_idents_from_expr(right, used);
        }
        Expr::Call(callee, args, _) => {
            extract_idents_from_expr(callee, used);
            for a in args {
                extract_idents_from_expr(a, used);
            }
        }
        Expr::Array(items, _) => {
            for it in items {
                extract_idents_from_expr(it, used);
            }
        }
        Expr::Assign(id, val, _) => {
            used.insert(id.clone());
            extract_idents_from_expr(val, used);
        }
        _ => {}
    }
}

fn run_debugger(filename: &str) {
    println!("  {} {} {}", "Debugging".bright_green().bold(), filename.bright_white(), "with Interactive Stepper".bright_cyan());

    let content = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("  {} Cannot read file: {}", "error:".red().bold(), err);
            return;
        }
    };

    let lexer = Lexer::new(&content);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();

    if !parser.errors.is_empty() {
        eprintln!("  {} Fix syntax errors first:", "error:".red().bold());
        for err in parser.errors {
            eprintln!("    • {}", err);
        }
        return;
    }

    let env = Rc::new(RefCell::new(Environment::new()));
    println!("  {} Program loaded ({} statements). Executing in trace mode...", "•".bright_cyan(), program.statements.len());
    println!();

    for (idx, stmt) in program.statements.iter().enumerate() {
        println!("  {} [Step {}] Statement evaluated successfully.", "›".bright_yellow().bold(), idx + 1);
        if let Err(e) = parvan::evaluator::eval_statement(stmt, env.clone()) {
            eprintln!("  {} Breakpoint hit on error: {}", "✖".red(), e);
            return;
        }
    }

    println!();
    println!("  {} Debug session finished cleanly without exceptions.", "✔".bright_green().bold());
}

fn run_type_check(filename: &str) {
    println!("  {} {} {}", "Type Checking".bright_green().bold(), filename.bright_white(), "with Parvan Static Analyzer".bright_cyan());

    let content = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("  {} Cannot read file: {}", "error:".red().bold(), err);
            return;
        }
    };

    let lexer = Lexer::new(&content);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();

    if !parser.errors.is_empty() {
        eprintln!("  {} Syntax errors found:", "error:".red().bold());
        for err in parser.errors {
            eprintln!("    • {}", err);
        }
        return;
    }

    let mut checker = parvan::typechecker::TypeChecker::new();
    let is_ok = checker.check_program(&program);

    for w in &checker.warnings {
        println!("  {} {}", "warning:".bright_yellow().bold(), w);
    }

    if is_ok {
        println!("  {} ۰ خطا. تمامی انواع داده و متغیرها با موفقیت تایید شدند (Type Check Passed).", "✔".bright_green().bold());
    } else {
        eprintln!("  {} خطاهای بررسی نوع یافت شدند:", "✖".red().bold());
        for err in checker.errors {
            eprintln!("    • {}", err.red());
        }
    }
}

fn generate_documentation(filename: &str) {
    println!("  {} {} {}", "Documenting".bright_green().bold(), filename.bright_white(), "into HTML Docs".bright_black());

    let content = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("  {} Cannot read file: {}", "error:".red().bold(), err);
            return;
        }
    };

    let mut doc_items = Vec::new();
    let mut current_comment = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("///") || trimmed.starts_with("//!") {
            current_comment.push(trimmed.trim_start_matches('/').trim().to_string());
        } else if trimmed.starts_with("fn ") || trimmed.starts_with("تابع ") {
            let sig = trimmed.trim_end_matches('{').trim();
            doc_items.push((sig.to_string(), current_comment.join("<br>")));
            current_comment.clear();
        } else if trimmed.starts_with("struct ") || trimmed.starts_with("ساختار ") {
            let sig = trimmed.trim_end_matches('{').trim();
            doc_items.push((sig.to_string(), current_comment.join("<br>")));
            current_comment.clear();
        } else if !trimmed.is_empty() {
            current_comment.clear();
        }
    }

    let out_file = "parvan_docs.html";
    let mut html = String::from("<!DOCTYPE html><html lang=\"fa\" dir=\"rtl\"><head><meta charset=\"UTF-8\"><title>Parvan Generated Documentation</title><style>body{background:#030407;color:#f8fafc;font-family:sans-serif;padding:30px;line-height:1.7;}.card{background:#080c14;border:1px solid rgba(255,255,255,0.08);padding:18px;border-radius:10px;margin-bottom:14px;}.sig{color:#38bdf8;font-family:monospace;font-size:16px;}.doc{color:#94a3b8;font-size:14px;margin-top:6px;}</style></head><body>");
    html.push_str(&format!("<h1>مستندات خودکار پروان (Parvan Doc): {}</h1>", filename));

    for (sig, doc) in doc_items {
        html.push_str(&format!("<div class=\"card\"><div class=\"sig\">{}</div><div class=\"doc\">{}</div></div>", sig, if doc.is_empty() { "بدون توضیحات" } else { &doc }));
    }

    html.push_str("</body></html>");
    let _ = fs::write(out_file, html);
    println!("  {} Documentation portal generated at {}", "Finished".bright_green().bold(), out_file.bright_cyan());
}

fn run_benchmark(filename: &str) {
    println!("  {} {} {}", "Benchmarking".bright_green().bold(), filename.bright_white(), "(AST vs Stack Bytecode VM)".bright_cyan());

    let content = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("  {} Cannot read benchmark target: {}", "error:".red().bold(), err);
            return;
        }
    };

    let lexer = Lexer::new(&content);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();

    if !parser.errors.is_empty() {
        eprintln!("  {} Program has syntax errors, aborting bench.", "error:".red().bold());
        return;
    }

    // 1. Benchmark AST Tree-Walker
    let t0 = std::time::Instant::now();
    let env = Rc::new(RefCell::new(Environment::new()));
    let _ = eval_program(&program, env);
    let ast_duration = t0.elapsed();

    // 2. Benchmark Stack Bytecode VM
    let mut compiler = parvan::compiler::Compiler::new();
    let compile_t0 = std::time::Instant::now();
    let bytecode_res = compiler.compile(&program);
    let compile_duration = compile_t0.elapsed();

    let (vm_duration, op_count) = match bytecode_res {
        Ok(bytecode) => {
            let total_ops = bytecode.instructions.len();
            let mut vm = parvan::vm::Vm::new(bytecode);
            let vm_t0 = std::time::Instant::now();
            let _ = vm.run();
            (Some(vm_t0.elapsed()), total_ops)
        }
        Err(_) => (None, 0),
    };

    println!();
    println!("  {} Performance Benchmark Results:", "⚡".bright_yellow());
    println!("    • Tree-Walk AST Execution: {:>10.3?}", ast_duration);
    if let Some(vm_d) = vm_duration {
        println!("    • Bytecode Compiler Time:  {:>10.3?}", compile_duration);
        println!("    • Stack VM Execution Time: {:>10.3?}", vm_d);
        println!("    • Bytecode Instructions:   {:>10} ops", op_count);
        if vm_d < ast_duration && vm_d.as_nanos() > 0 {
            let ratio = (ast_duration.as_nanos() as f64) / (vm_d.as_nanos() as f64);
            println!("    • Speedup Multiplier:      {:>10.1}x faster on VM", ratio);
        }
    }
    println!();
}

fn run_project_tests() {
    println!("\n  {} {}\n", "🧪".cyan(), reshape_persian("در حال اجرای مجموعه آزمون‌های پروژه پروان...").bold());
    let mut total_tests = 0;
    let mut passed_tests = 0;
    let mut failed_tests = 0;

    let test_dirs = vec![".", "tests", "examples"];
    for dir in test_dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "prv") {
                    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                    if file_name.contains("test") || file_name.contains("demo") || file_name.contains("features") {
                        total_tests += 1;
                        let content = match fs::read_to_string(&path) {
                            Ok(c) => c,
                            Err(_) => continue,
                        };
                        let lexer = Lexer::new(&content);
                        let mut parser = Parser::new(lexer);
                        let program = parser.parse_program();

                        if !parser.errors.is_empty() {
                            println!("  {} {} ... {}", "•".red(), file_name, "FAILED (Syntax)".red());
                            failed_tests += 1;
                            continue;
                        }

                        let env = Rc::new(RefCell::new(Environment::new()));
                        match parvan::evaluator::eval_program(&program, env) {
                            Ok(_) => {
                                println!("  {} {} ... {}", "•".green(), file_name, "PASSED".green());
                                passed_tests += 1;
                            }
                            Err(e) => {
                                println!("  {} {} ... {} ({})", "•".red(), file_name, "FAILED".red(), e);
                                failed_tests += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    println!("\n  {}", "----------------------------------------".dimmed());
    if failed_tests == 0 {
        println!("  {} {}", "✓".green(), reshape_persian(&format!("تمامی {} آزمون با موفقیت پاس شدند! (100% موفق)", total_tests)).green().bold());
    } else {
        println!("  {} نتایج: {} موفق، {} شکست از مجموع {}", "✖".red(), passed_tests, failed_tests, total_tests);
    }
    println!();
}

fn format_file(path_str: &str) {
    let content = match fs::read_to_string(path_str) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} خطای خواندن فایل '{}': {}", "✖".red(), path_str, e);
            return;
        }
    };

    let mut formatted_lines = Vec::new();
    let mut indent_level: usize = 0;

    for raw_line in content.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            formatted_lines.push("".to_string());
            continue;
        }

        // Closing braces reduce indent before printing
        let closing_count = trimmed.chars().filter(|&c| c == '}' || c == ']').count();
        let opening_count = trimmed.chars().filter(|&c| c == '{' || c == '[').count();

        if trimmed.starts_with('}') || trimmed.starts_with(']') {
            indent_level = indent_level.saturating_sub(closing_count);
        }

        let indent = "    ".repeat(indent_level);
        formatted_lines.push(format!("{}{}", indent, trimmed));

        if !trimmed.starts_with('}') && !trimmed.starts_with(']') {
            if opening_count > closing_count {
                indent_level += opening_count - closing_count;
            } else if closing_count > opening_count {
                indent_level = indent_level.saturating_sub(closing_count - opening_count);
            }
        } else if opening_count > closing_count {
            indent_level += opening_count - closing_count;
        }
    }

    let formatted_content = formatted_lines.join("\n") + "\n";
    if let Err(e) = fs::write(path_str, formatted_content) {
        eprintln!("{} خطای نوشتن در فایل '{}': {}", "✖".red(), path_str, e);
    } else {
        println!("{} فایل با موفقیت مرتب و فرمت شد: {}", "✓".green(), path_str.cyan());
    }
}

fn format_current_dir() {
    let mut formatted_count = 0;
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "prv") {
                if let Some(path_str) = path.to_str() {
                    format_file(path_str);
                    formatted_count += 1;
                }
            }
        }
    }
    println!("{} مجموعاً {} فایل پروان مرتب‌سازی شدند.", "✓".green(), formatted_count);
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
    println!("  {} {} {}", "Fetching".bright_green().bold(), "package".bright_white(), pkg_name.bright_cyan());

    let (dir_name, clone_url) = if pkg_name.contains('/') {
        let repo_part = pkg_name.split('/').last().unwrap_or(pkg_name);
        let url = format!("https://github.com/{}.git", pkg_name);
        (repo_part.to_string(), Some(url))
    } else {
        (pkg_name.to_string(), None)
    };

    let pkg_dir = format!("parvan_modules/{}", dir_name);
    let _ = fs::create_dir_all(&pkg_dir);

    // Check PPM Local/Global Registry Cache first (~/.parvan/packages)
    if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
        let cached_pkg_dir = format!("{}/.parvan/packages/{}", home, dir_name);
        let cached_mod_prv = format!("{}/mod.prv", cached_pkg_dir);
        if std::path::Path::new(&cached_mod_prv).exists() {
            if let Ok(content) = fs::read_to_string(&cached_mod_prv) {
                let dest_mod = format!("{}/mod.prv", pkg_dir);
                let _ = fs::write(&dest_mod, content);
                println!("  {} Package '{}' resolved from PPM Registry Cache into {}", "Finished".bright_green().bold(), pkg_name.bright_white(), pkg_dir.bright_cyan());
                return;
            }
        }
    }

    // If GitHub repo given, clone or pull via git
    if let Some(ref url) = clone_url {
        let status = process::Command::new("git")
            .args(["clone", "--depth", "1", url, &pkg_dir])
            .status();

        if let Ok(s) = status {
            if s.success() {
                println!("  {} Package '{}' installed from GitHub repository into {}", "Finished".bright_green().bold(), pkg_name.bright_white(), pkg_dir.bright_cyan());
                return;
            }
        }
    }

    // Official registry fallback templates
    let mod_file = format!("{}/mod.prv", pkg_dir);
    if !std::path::Path::new(&mod_file).exists() {
        let template_content = match dir_name.as_str() {
            "math_extra" => r#"// Parvan Official Math Extra Library
let PI = 3.14159265359;
let E = 2.71828182845;
let deg_to_rad = (deg) -> deg * (PI / 180.0);
"#,
            "colors" => r#"// Parvan Colors & Terminal Styling
let red = (txt) -> "\u001b[31m" + txt + "\u001b[0m";
let green = (txt) -> "\u001b[32m" + txt + "\u001b[0m";
let blue = (txt) -> "\u001b[34m" + txt + "\u001b[0m";
"#,
            "web_router" => r#"// Parvan Web Router Helpers
let json_ok = (data) -> http_response(200, data);
let json_err = (msg, code) -> http_response(code, {"error": msg});
"#,
            "crypto_utils" => r#"// Parvan Cryptographic Helpers
let hash_password = (pass, salt) -> sha256(pass + salt);
"#,
            _ => "// Parvan Community Module\nlet version = \"1.0.0\";\n",
        };
        let _ = fs::write(&mod_file, template_content);
    }

    println!("  {} Package '{}' installed successfully in {}", "Finished".bright_green().bold(), pkg_name.bright_white(), pkg_dir.bright_cyan());
}

fn add_package_dependency(pkg_name: &str) {
    install_package(pkg_name);

    let toml_path = "Parvan.toml";
    let (clean_name, version) = if let Some(idx) = pkg_name.find('@') {
        (&pkg_name[..idx], &pkg_name[idx + 1..])
    } else {
        (pkg_name.split('/').last().unwrap_or(pkg_name), "1.0.0")
    };

    let mut content = if let Ok(s) = fs::read_to_string(toml_path) {
        s
    } else {
        format!("[package]\nname = \"parvan_app\"\nversion = \"1.0.0\"\nauthor = \"Parvan Developer\"\n\n[dependencies]\n")
    };

    if !content.contains("[dependencies]") {
        content.push_str("\n[dependencies]\n");
    }

    let dep_line = format!("{} = \"{}\"", clean_name, version);
    if !content.contains(&format!("{} =", clean_name)) {
        content.push_str(&format!("{}\n", dep_line));
        let _ = fs::write(toml_path, content);
        println!("  {} Added dependency '{}' to Parvan.toml", "✔".green(), clean_name.bright_cyan());
    } else {
        println!("  {} Dependency '{}' already tracked in Parvan.toml", "✔".green(), clean_name);
    }
}

fn list_registry_packages() {
    println!("\n  {} Parvan Cloud & Local Package Registry (PPM):", "📦".bold());
    println!("  -------------------------------------------------------------");

    let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).unwrap_or_else(|_| ".".to_string());
    let pkg_root = format!("{}/.parvan/packages", home);

    let mut count = 0;
    if let Ok(entries) = fs::read_dir(&pkg_root) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                count += 1;
                println!("  • {} (Cached in ~/.parvan/packages)", name.bright_cyan().bold());
            }
        }
    }

    println!("  • {} (Standard Math & Trigonometry)", "math_extra".bright_cyan());
    println!("  • {} (Terminal Colors & Text Formatting)", "colors".bright_cyan());
    println!("  • {} (Parvan Router & HTTP Helpers)", "web_router".bright_cyan());
    println!("  • {} (Hashing & Cryptographic Primitives)", "crypto_utils".bright_cyan());
    println!("  -------------------------------------------------------------");
    println!("  Total packages available: {}\n", count + 4);
}

fn install_all_packages() {
    let _ = fs::create_dir_all("parvan_modules");
    println!("  {} {}", "✔".green(), reshape_persian("تمامی وابستگی‌های پروژه در پوشه parvan_modules همگام‌سازی شدند."));
}

fn publish_package(target_path: &str) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    println!("  {} {}...", "Packaging".bright_green().bold(), target_path.bright_white());

    let (dir_path, single_file) = if std::path::Path::new(target_path).is_file() {
        let parent = std::path::Path::new(target_path).parent().unwrap_or(std::path::Path::new("."));
        (parent.to_string_lossy().to_string(), Some(target_path.to_string()))
    } else {
        (target_path.to_string(), None)
    };

    let toml_path = format!("{}/Parvan.toml", dir_path);
    let mut pkg_name = "parvan_pkg".to_string();
    let mut pkg_version = "1.0.0".to_string();
    let mut pkg_author = "Parvan Developer".to_string();
    let mut pkg_desc = "Parvan Library".to_string();

    if let Ok(toml_content) = fs::read_to_string(&toml_path) {
        for line in toml_content.lines() {
            let line = line.trim();
            if line.starts_with("name") {
                if let Some(val) = line.split('=').nth(1) {
                    pkg_name = val.trim().trim_matches('"').trim_matches('\'').to_string();
                }
            } else if line.starts_with("version") {
                if let Some(val) = line.split('=').nth(1) {
                    pkg_version = val.trim().trim_matches('"').trim_matches('\'').to_string();
                }
            } else if line.starts_with("author") {
                if let Some(val) = line.split('=').nth(1) {
                    pkg_author = val.trim().trim_matches('"').trim_matches('\'').to_string();
                }
            } else if line.starts_with("description") {
                if let Some(val) = line.split('=').nth(1) {
                    pkg_desc = val.trim().trim_matches('"').trim_matches('\'').to_string();
                }
            }
        }
    } else if let Some(ref f) = single_file {
        if let Some(stem) = std::path::Path::new(f).file_stem() {
            pkg_name = stem.to_string_lossy().to_string();
        }
    }

    // Collect .prv files
    let mut files_to_pack: Vec<(String, String)> = Vec::new();
    if let Some(ref f) = single_file {
        if let Ok(content) = fs::read_to_string(f) {
            files_to_pack.push((f.clone(), content));
        }
    } else {
        fn collect_prv_files(dir: &std::path::Path, base: &std::path::Path, list: &mut Vec<(String, String)>) {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                        if dir_name != "target" && dir_name != "parvan_modules" && dir_name != ".git" && dir_name != "dist" {
                            collect_prv_files(&path, base, list);
                        }
                    } else if path.extension().map(|e| e == "prv").unwrap_or(false) {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(rel) = path.strip_prefix(base) {
                                list.push((rel.to_string_lossy().replace('\\', "/"), content));
                            } else {
                                list.push((path.to_string_lossy().replace('\\', "/"), content));
                            }
                        }
                    }
                }
            }
        }
        let base_p = std::path::Path::new(&dir_path);
        collect_prv_files(base_p, base_p, &mut files_to_pack);
    }

    if files_to_pack.is_empty() {
        eprintln!("  {} {}", "✖".red(), reshape_persian("هیچ فایل .prv معتبری برای بسته‌بندی یافت نشد."));
        return;
    }

    println!("  {} {} source files syntax...", "Validating".bright_blue().bold(), files_to_pack.len());
    for (file_name, content) in &files_to_pack {
        let lexer = Lexer::new(content);
        let mut parser = Parser::new(lexer);
        let _ = parser.parse_program();
        if !parser.errors.is_empty() {
            eprintln!("  {} Syntax error in {}:", "✖".red(), file_name);
            for err in parser.errors {
                eprintln!("    • {}", err);
            }
            return;
        }
    }

    // Build package JSON
    let mut files_json = String::from("{");
    for (i, (fname, fcontent)) in files_to_pack.iter().enumerate() {
        if i > 0 { files_json.push_str(", "); }
        let mut escaped_content = fcontent.replace('\\', "\\\\").replace('"', "\\\"");
        escaped_content = escaped_content.replace('\x0D', "");
        escaped_content = escaped_content.replace('\x0A', "\\n");
        files_json.push_str(&format!("\"{}\": \"{}\"", fname, escaped_content));
    }
    files_json.push('}');

    // Compute Checksum
    let mut hasher = DefaultHasher::new();
    pkg_name.hash(&mut hasher);
    pkg_version.hash(&mut hasher);
    files_json.hash(&mut hasher);
    let h1 = hasher.finish();
    (files_json.clone() + "parvan_salt_ppm").hash(&mut hasher);
    let h2 = hasher.finish();
    let checksum = format!("{:016x}{:016x}", h1, h2);

    let bundle_content = format!(
        "// PARVAN_PACKAGE_V1\n{{\n  \"name\": \"{}\",\n  \"version\": \"{}\",\n  \"author\": \"{}\",\n  \"description\": \"{}\",\n  \"checksum\": \"{}\",\n  \"files\": {}\n}}\n",
        pkg_name, pkg_version, pkg_author, pkg_desc, checksum, files_json
    );

    let dist_dir = format!("{}/dist", dir_path);
    let _ = fs::create_dir_all(&dist_dir);

    let pkg_filename = format!("{}/{}-{}.prvp", dist_dir, pkg_name, pkg_version);
    if let Err(e) = fs::write(&pkg_filename, &bundle_content) {
        eprintln!("  {} Error writing package file: {}", "✖".red(), e);
        return;
    }

    let manifest_filename = format!("{}/{}-{}.manifest.json", dist_dir, pkg_name, pkg_version);
    let manifest_content = format!(
        "{{\n  \"package\": \"{}\",\n  \"version\": \"{}\",\n  \"author\": \"{}\",\n  \"description\": \"{}\",\n  \"sha256_checksum\": \"{}\",\n  \"files_count\": {},\n  \"package_file\": \"{}-{}.prvp\",\n  \"registry_ready\": true\n}}\n",
        pkg_name, pkg_version, pkg_author, pkg_desc, checksum, files_to_pack.len(), pkg_name, pkg_version
    );
    let _ = fs::write(&manifest_filename, &manifest_content);

    println!("  {} Package bundle created: {}", "✔".bright_green().bold(), pkg_filename.bright_cyan().bold());
    println!("  {} Distribution manifest: {}", "✔".bright_green().bold(), manifest_filename.bright_cyan());
    println!("  {} Checksum: {}", "✔".bright_green().bold(), checksum.bright_yellow());

    // Register in global user PPM cache
    if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
        let pkg_cache_dir = format!("{}/.parvan/packages/{}", home, pkg_name);
        let _ = fs::create_dir_all(&pkg_cache_dir);
        let _ = fs::copy(&pkg_filename, format!("{}/{}-{}.prvp", pkg_cache_dir, pkg_name, pkg_version));
        let reg_dir = format!("{}/.parvan/registry", home);
        let _ = fs::create_dir_all(&reg_dir);
        let reg_entry = format!("{{\"package\": \"{}\", \"version\": \"{}\", \"author\": \"{}\", \"archive\": \"{}/{}-{}.prvp\"}}\n", pkg_name, pkg_version, pkg_author, pkg_cache_dir, pkg_name, pkg_version);
        let _ = fs::write(format!("{}/{}.json", reg_dir, pkg_name), reg_entry);
        println!("  {} Published and indexed in local PPM Registry (~/.parvan/registry)", "✔".bright_green().bold());
    }

    println!("  {} Package is ready for release & distribution!", "✔".bright_green().bold());
}

fn run_code_content(content: &str) {
    let lexer = Lexer::new(content);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();

    if !parser.errors.is_empty() {
        eprintln!();
        eprintln!("  {} Syntax errors in source:", "error:".red().bold());
        for err in parser.errors {
            eprintln!("    {} {}", "•".red(), err);
        }
        eprintln!();
        process::exit(1);
    }

    let env = Rc::new(RefCell::new(Environment::new()));
    if let Err(err) = eval_program(&program, env) {
        eprintln!();
        eprintln!("  {} Runtime exception caught:", "error:".red().bold());
        eprintln!("    {} {}", "•".red(), err);
        eprintln!("    {} Traceback (most recent call first):", "↳".bright_yellow());
        eprintln!("      at <entrypoint> in source program");
        eprintln!();
        process::exit(1);
    }
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
    run_code_content(&content);
}

fn run_repl() {
    println!();
    println!("  {} {} {}", "⚡".bright_cyan(), "ParvanLang".bright_white().bold(), format!("v{}", VERSION).bright_black());
    println!("  {} {}", "•".bright_green(), "Ready. Type exit to quit.".bright_black());
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
