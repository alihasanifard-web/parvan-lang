use crate::object::Object;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_builtin(name: &str) -> Option<Object> {
    match name {
        "len" | "طول" => Some(Object::Builtin(builtin_len)),
        "push" | "افزودن" => Some(Object::Builtin(builtin_push)),
        "pop" | "حذف_آخر" => Some(Object::Builtin(builtin_pop)),
        "range" | "بازه" => Some(Object::Builtin(builtin_range)),
        "input" | "ورودی" => Some(Object::Builtin(builtin_input)),
        "time" | "زمان" => Some(Object::Builtin(builtin_time)),
        "type" | "نوع" => Some(Object::Builtin(builtin_type)),
        "to_int" | "به_عدد" => Some(Object::Builtin(builtin_to_int)),
        "to_str" | "به_متن" => Some(Object::Builtin(builtin_to_str)),
        "abs" | "قدر_مطلق" => Some(Object::Builtin(builtin_abs)),
        "sqrt" | "جذر" => Some(Object::Builtin(builtin_sqrt)),
        _ => None,
    }
}

fn builtin_len(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err(format!("تابع len فقط ۱ آرگومان می‌پذیرد، اما {} دریافت شد.", args.len()));
    }
    match &args[0] {
        Object::Str(s) => Ok(Object::Int(s.chars().count() as i64)),
        Object::Array(arr) => Ok(Object::Int(arr.borrow().len() as i64)),
        _ => Err(format!("تابع len روی نوع داده '{}' پشتیبانی نمی‌شود.", args[0].type_name())),
    }
}

fn builtin_push(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 2 {
        return Err(format!("تابع push دو آرگومان می‌پذیرد (آرایه، مقدار)، اما {} دریافت شد.", args.len()));
    }
    match &args[0] {
        Object::Array(arr) => {
            arr.borrow_mut().push(args[1].clone());
            Ok(Object::Array(arr.clone()))
        }
        _ => Err("آرگومان اول تابع push باید آرایه باشد.".to_string()),
    }
}

fn builtin_pop(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err("تابع pop دقیقاً ۱ آرگومان (آرایه) می‌پذیرد.".to_string());
    }
    match &args[0] {
        Object::Array(arr) => {
            let last = arr.borrow_mut().pop();
            Ok(last.unwrap_or(Object::Nil))
        }
        _ => Err("آرگومان تابع pop باید آرایه باشد.".to_string()),
    }
}

fn builtin_range(args: Vec<Object>) -> Result<Object, String> {
    if args.is_empty() || args.len() > 3 {
        return Err("تابع range بین ۱ تا ۳ آرگومان می‌پذیرد: range(start, end, step).".to_string());
    }

    let (start, end, step) = match args.len() {
        1 => {
            let end = match args[0] {
                Object::Int(n) => n,
                _ => return Err("آرگومان range باید عدد صحیح باشد.".to_string()),
            };
            (0, end, 1)
        }
        2 => {
            let start = match args[0] {
                Object::Int(n) => n,
                _ => return Err("آرگومان range باید عدد صحیح باشد.".to_string()),
            };
            let end = match args[1] {
                Object::Int(n) => n,
                _ => return Err("آرگومان range باید عدد صحیح باشد.".to_string()),
            };
            (start, end, 1)
        }
        3 => {
            let start = match args[0] {
                Object::Int(n) => n,
                _ => return Err("آرگومان range باید عدد صحیح باشد.".to_string()),
            };
            let end = match args[1] {
                Object::Int(n) => n,
                _ => return Err("آرگومان range باید عدد صحیح باشد.".to_string()),
            };
            let step = match args[2] {
                Object::Int(n) => n,
                _ => return Err("گام حرکت (step) باید عدد صحیح باشد.".to_string()),
            };
            if step == 0 {
                return Err("گام حرکت در range نمی‌تواند صفر باشد.".to_string());
            }
            (start, end, step)
        }
        _ => unreachable!(),
    };

    let mut list = Vec::new();
    let mut cur = start;
    if step > 0 {
        while cur < end {
            list.push(Object::Int(cur));
            cur += step;
        }
    } else {
        while cur > end {
            list.push(Object::Int(cur));
            cur += step;
        }
    }

    Ok(Object::Array(Rc::new(RefCell::new(list))))
}

fn builtin_input(args: Vec<Object>) -> Result<Object, String> {
    use std::io::{self, Write};
    if let Some(prompt) = args.first() {
        print!("{}", prompt);
        io::stdout().flush().unwrap_or(());
    }

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).map_err(|e| e.to_string())?;
    Ok(Object::Str(buffer.trim_end_matches(&['\r', '\n'][..]).to_string()))
}

fn builtin_time(_args: Vec<Object>) -> Result<Object, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs_f64();
    Ok(Object::Float(now))
}

fn builtin_type(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err("تابع type یک آرگومان می‌پذیرد.".to_string());
    }
    Ok(Object::Str(args[0].type_name().to_string()))
}

fn builtin_to_int(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err("تابع to_int یک آرگومان می‌پذیرد.".to_string());
    }
    match &args[0] {
        Object::Int(i) => Ok(Object::Int(*i)),
        Object::Float(f) => Ok(Object::Int(*f as i64)),
        Object::Str(s) => s.trim().parse::<i64>().map(Object::Int).map_err(|_| format!("رشته '{}' به عدد صحیح تبدیل نمی‌شود.", s)),
        _ => Err("امکان تبدیل این نوع داده به عدد صحیح وجود ندارد.".to_string()),
    }
}

fn builtin_to_str(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err("تابع to_str یک آرگومان می‌پذیرد.".to_string());
    }
    Ok(Object::Str(args[0].to_string()))
}

fn builtin_abs(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err("تابع abs یک آرگومان می‌پذیرد.".to_string());
    }
    match &args[0] {
        Object::Int(i) => Ok(Object::Int(i.abs())),
        Object::Float(f) => Ok(Object::Float(f.abs())),
        _ => Err("تابع abs فقط روی اعداد کار می‌کند.".to_string()),
    }
}

fn builtin_sqrt(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err("تابع sqrt یک آرگومان می‌پذیرد.".to_string());
    }
    match &args[0] {
        Object::Int(i) => {
            if *i < 0 { return Err("جذر عدد منفی تعریف نشده است.".to_string()); }
            Ok(Object::Float((*i as f64).sqrt()))
        }
        Object::Float(f) => {
            if *f < 0.0 { return Err("جذر عدد منفی تعریف نشده است.".to_string()); }
            Ok(Object::Float(f.sqrt()))
        }
        _ => Err("تابع sqrt فقط روی اعداد کار می‌کند.".to_string()),
    }
}
