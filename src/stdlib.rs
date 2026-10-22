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
        "keys" | "کلیدها" => Some(Object::Builtin(builtin_keys)),
        "values" | "مقادیر" => Some(Object::Builtin(builtin_values)),
        "contains" | "شامل_است" => Some(Object::Builtin(builtin_contains)),
        "read_file" | "خواندن_فایل" => Some(Object::Builtin(builtin_read_file)),
        "write_file" | "نوشتن_فایل" => Some(Object::Builtin(builtin_write_file)),
        "split" | "تفکیک" => Some(Object::Builtin(builtin_split)),
        "join" | "پیوستن" => Some(Object::Builtin(builtin_join)),
        "trim" | "پیرایش" => Some(Object::Builtin(builtin_trim)),
        "uppercase" | "بزرگ" => Some(Object::Builtin(builtin_uppercase)),
        "lowercase" | "کوچک" => Some(Object::Builtin(builtin_lowercase)),
        "reshape" | "جورچین" | "راست_به_چپ" => Some(Object::Builtin(builtin_reshape)),
        "reverse" | "معکوس" => Some(Object::Builtin(builtin_reverse)),
        "sort" | "مرتب_سازی" => Some(Object::Builtin(builtin_sort)),
        "replace" | "جایگزینی" => Some(Object::Builtin(builtin_replace)),
        "starts_with" | "شروع_با" => Some(Object::Builtin(builtin_starts_with)),
        "ends_with" | "پایان_با" => Some(Object::Builtin(builtin_ends_with)),
        "floor" | "کف" => Some(Object::Builtin(builtin_floor)),
        "ceil" | "سقف" => Some(Object::Builtin(builtin_ceil)),
        "round" | "گرد" => Some(Object::Builtin(builtin_round)),
        "exit" | "خروج" => Some(Object::Builtin(builtin_exit)),
        "http_get" | "دریافت_وب" => Some(Object::Builtin(builtin_http_get)),
        "http_post" | "ارسال_وب" => Some(Object::Builtin(builtin_http_post)),
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
        Object::Map(m) => Ok(Object::Int(m.borrow().len() as i64)),
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
    let prompt = if let Some(p) = args.first() {
        let p_str = p.to_string();
        crate::persian::reshape_persian(&p_str)
    } else {
        "".to_string()
    };

    match crate::rtl_input::RtlLineReader::read_line(&prompt) {
        Ok(Some(line)) => Ok(Object::Str(line.trim_end_matches(&['\r', '\n'][..]).to_string())),
        Ok(None) => Ok(Object::Nil),
        Err(_) => {
            // Fallback to standard input
            print!("{}", prompt);
            io::stdout().flush().unwrap_or(());
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).map_err(|e| e.to_string())?;
            Ok(Object::Str(buffer.trim_end_matches(&['\r', '\n'][..]).to_string()))
        }
    }
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

fn builtin_keys(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err("تابع keys یک آرگومان (دیکشنری) می‌پذیرد.".to_string());
    }
    match &args[0] {
        Object::Map(m) => {
            let list: Vec<Object> = m.borrow().keys().map(|k| Object::Str(k.clone())).collect();
            Ok(Object::Array(Rc::new(RefCell::new(list))))
        }
        _ => Err("آرگومان تابع keys باید دیکشنری باشد.".to_string()),
    }
}

fn builtin_values(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err("تابع values یک آرگومان (دیکشنری) می‌پذیرد.".to_string());
    }
    match &args[0] {
        Object::Map(m) => {
            let list: Vec<Object> = m.borrow().values().cloned().collect();
            Ok(Object::Array(Rc::new(RefCell::new(list))))
        }
        _ => Err("آرگومان تابع values باید دیکشنری باشد.".to_string()),
    }
}

fn builtin_contains(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 2 {
        return Err("تابع contains دو آرگومان می‌پذیرد: (مجموعه، آیتم)".to_string());
    }
    match (&args[0], &args[1]) {
        (Object::Map(m), Object::Str(k)) => Ok(Object::Bool(m.borrow().contains_key(k))),
        (Object::Array(arr), target) => {
            let found = arr.borrow().iter().any(|item| item == target);
            Ok(Object::Bool(found))
        }
        (Object::Str(s), Object::Str(sub)) => Ok(Object::Bool(s.contains(sub))),
        _ => Err("تابع contains روی این نوع داده‌ها پشتیبانی نمی‌شود.".to_string()),
    }
}

fn builtin_read_file(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err("تابع read_file نیازمند یک مسیر فایل متنی است.".to_string());
    }
    match &args[0] {
        Object::Str(path) => {
            let content = std::fs::read_to_string(path)
                .map_err(|e| format!("خطای خواندن فایل '{}': {}", path, e))?;
            Ok(Object::Str(content))
        }
        _ => Err("مسیر فایل باید رشته باشد.".to_string()),
    }
}

fn builtin_write_file(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 2 {
        return Err("تابع write_file دو آرگومان می‌پذیرد: (مسیر_فایل، متن)".to_string());
    }
    match (&args[0], &args[1]) {
        (Object::Str(path), Object::Str(content)) => {
            std::fs::write(path, content)
                .map_err(|e| format!("خطای نوشتن در فایل '{}': {}", path, e))?;
            Ok(Object::Bool(true))
        }
        _ => Err("آرگومان‌های write_file باید هر دو از نوع رشته باشند.".to_string()),
    }
}

fn builtin_split(args: Vec<Object>) -> Result<Object, String> {
    if args.len() < 1 || args.len() > 2 {
        return Err("تابع split یک یا دو آرگومان می‌پذیرد: (متن, جداکننده)".to_string());
    }
    let sep = if args.len() == 2 {
        match &args[1] {
            Object::Str(s) => s.clone(),
            _ => return Err("جداکننده باید رشته باشد.".to_string()),
        }
    } else {
        " ".to_string()
    };

    match &args[0] {
        Object::Str(s) => {
            let parts: Vec<Object> = s.split(&sep).map(|p| Object::Str(p.to_string())).collect();
            Ok(Object::Array(Rc::new(RefCell::new(parts))))
        }
        _ => Err("آرگومان تابع split باید رشته باشد.".to_string()),
    }
}

fn builtin_join(args: Vec<Object>) -> Result<Object, String> {
    if args.len() < 1 || args.len() > 2 {
        return Err("تابع join یک یا دو آرگومان می‌پذیرد: (آرایه, جداکننده)".to_string());
    }
    let sep = if args.len() == 2 {
        match &args[1] {
            Object::Str(s) => s.clone(),
            _ => return Err("جداکننده باید رشته باشد.".to_string()),
        }
    } else {
        "".to_string()
    };

    match &args[0] {
        Object::Array(arr) => {
            let str_parts: Vec<String> = arr.borrow().iter().map(|item| item.to_string()).collect();
            Ok(Object::Str(str_parts.join(&sep)))
        }
        _ => Err("آرگومان اول تابع join باید آرایه باشد.".to_string()),
    }
}

fn builtin_trim(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err("تابع trim یک آرگومان رشته می‌پذیرد.".to_string());
    }
    match &args[0] {
        Object::Str(s) => Ok(Object::Str(s.trim().to_string())),
        _ => Err("آرگومان تابع trim باید رشته باشد.".to_string()),
    }
}

fn builtin_uppercase(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err("تابع uppercase یک آرگومان می‌پذیرد.".to_string());
    }
    match &args[0] {
        Object::Str(s) => Ok(Object::Str(s.to_uppercase())),
        _ => Err("آرگومان تابع uppercase باید رشته باشد.".to_string()),
    }
}

fn builtin_lowercase(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err("تابع lowercase یک آرگومان می‌پذیرد.".to_string());
    }
    match &args[0] {
        Object::Str(s) => Ok(Object::Str(s.to_lowercase())),
        _ => Err("آرگومان تابع lowercase باید رشته باشد.".to_string()),
    }
}

fn builtin_reshape(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err("تابع جورچین (reshape) یک آرگومان می‌پذیرد: متن برای چیدمان راست‌به‌چپ.".to_string());
    }
    let text = match &args[0] {
        Object::Str(s) => s.clone(),
        other => other.to_string(),
    };
    let reshaped = crate::persian::reshape_persian(&text);
    Ok(Object::Str(reshaped))
}

fn builtin_reverse(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err("تابع reverse یک آرگومان (آرایه یا رشته) می‌پذیرد.".to_string());
    }
    match &args[0] {
        Object::Str(s) => {
            let rev: String = s.chars().rev().collect();
            Ok(Object::Str(rev))
        }
        Object::Array(arr) => {
            let mut list = arr.borrow().clone();
            list.reverse();
            Ok(Object::Array(Rc::new(RefCell::new(list))))
        }
        _ => Err("تابع reverse فقط روی رشته و آرایه کار می‌کند.".to_string()),
    }
}

fn builtin_sort(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err("تابع sort یک آرگومان (آرایه) می‌پذیرد.".to_string());
    }
    match &args[0] {
        Object::Array(arr) => {
            let mut list = arr.borrow().clone();
            list.sort_by(|a, b| {
                match (a, b) {
                    (Object::Int(x), Object::Int(y)) => x.cmp(y),
                    (Object::Float(x), Object::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                    (Object::Str(x), Object::Str(y)) => x.cmp(y),
                    _ => std::cmp::Ordering::Equal,
                }
            });
            Ok(Object::Array(Rc::new(RefCell::new(list))))
        }
        _ => Err("تابع sort فقط روی آرایه کار می‌کند.".to_string()),
    }
}

fn builtin_replace(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 3 {
        return Err("تابع replace سه آرگومان می‌پذیرد: (متن_اصلی, متن_قدیم, متن_جدید).".to_string());
    }
    match (&args[0], &args[1], &args[2]) {
        (Object::Str(src), Object::Str(from), Object::Str(to)) => {
            Ok(Object::Str(src.replace(from, to)))
        }
        _ => Err("آرگومان‌های replace باید هر سه از نوع رشته باشند.".to_string()),
    }
}

fn builtin_starts_with(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 2 {
        return Err("تابع starts_with دو آرگومان می‌پذیرد: (متن, پیشوند).".to_string());
    }
    match (&args[0], &args[1]) {
        (Object::Str(s), Object::Str(prefix)) => Ok(Object::Bool(s.starts_with(prefix))),
        _ => Err("آرگومان‌های starts_with باید رشته باشند.".to_string()),
    }
}

fn builtin_ends_with(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 2 {
        return Err("تابع ends_with دو آرگومان می‌پذیرد: (متن, پسوند).".to_string());
    }
    match (&args[0], &args[1]) {
        (Object::Str(s), Object::Str(suffix)) => Ok(Object::Bool(s.ends_with(suffix))),
        _ => Err("آرگومان‌های ends_with باید رشته باشند.".to_string()),
    }
}

fn builtin_floor(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err("تابع floor یک آرگومان عددی می‌پذیرد.".to_string());
    }
    match &args[0] {
        Object::Int(i) => Ok(Object::Int(*i)),
        Object::Float(f) => Ok(Object::Int(f.floor() as i64)),
        _ => Err("تابع floor فقط روی اعداد کار می‌کند.".to_string()),
    }
}

fn builtin_ceil(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err("تابع ceil یک آرگومان عددی می‌پذیرد.".to_string());
    }
    match &args[0] {
        Object::Int(i) => Ok(Object::Int(*i)),
        Object::Float(f) => Ok(Object::Int(f.ceil() as i64)),
        _ => Err("تابع ceil فقط روی اعداد کار می‌کند.".to_string()),
    }
}

fn builtin_round(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err("تابع round یک آرگومان عددی می‌پذیرد.".to_string());
    }
    match &args[0] {
        Object::Int(i) => Ok(Object::Int(*i)),
        Object::Float(f) => Ok(Object::Int(f.round() as i64)),
        _ => Err("تابع round فقط روی اعداد کار می‌کند.".to_string()),
    }
}

fn builtin_exit(args: Vec<Object>) -> Result<Object, String> {
    let code = if let Some(Object::Int(c)) = args.first() {
        *c as i32
    } else {
        0
    };
    std::process::exit(code);
}

fn builtin_http_get(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err("تابع http_get نیازمند ۱ آرگومان (آدرس URL) است.".to_string());
    }
    let url = match &args[0] {
        Object::Str(u) => u.clone(),
        _ => return Err("آدرس URL باید رشته باشد.".to_string()),
    };

    use std::io::Read;
    use std::net::TcpStream;

    let parsed = url.strip_prefix("http://").ok_or_else(|| "فقط پروتکل http:// پشتیبانی می‌شود.".to_string())?;
    let mut parts = parsed.splitn(2, '/');
    let host_port = parts.next().unwrap_or("");
    let path = format!("/{}", parts.next().unwrap_or(""));

    let host = if host_port.contains(':') {
        host_port.to_string()
    } else {
        format!("{}:80", host_port)
    };

    let mut stream = TcpStream::connect(&host).map_err(|e| format!("خطای اتصال به سرور '{}': {}", host, e))?;
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: ParvanLang/2.0\r\nConnection: close\r\n\r\n",
        path, host_port
    );

    use std::io::Write;
    stream.write_all(request.as_bytes()).map_err(|e| e.to_string())?;

    let mut response = String::new();
    stream.read_to_string(&mut response).map_err(|e| e.to_string())?;

    // Split headers and body
    let body = if let Some(pos) = response.find("\r\n\r\n") {
        &response[pos + 4..]
    } else {
        &response
    };

    Ok(Object::Str(body.to_string()))
}

fn builtin_http_post(args: Vec<Object>) -> Result<Object, String> {
    if args.len() < 2 {
        return Err("تابع http_post حداقل ۲ آرگومان می‌پذیرد: (url, body).".to_string());
    }
    let url = match &args[0] {
        Object::Str(u) => u.clone(),
        _ => return Err("آدرس URL باید رشته باشد.".to_string()),
    };
    let body_str = args[1].to_string();

    use std::io::{Read, Write};
    use std::net::TcpStream;

    let parsed = url.strip_prefix("http://").ok_or_else(|| "فقط پروتکل http:// پشتیبانی می‌شود.".to_string())?;
    let mut parts = parsed.splitn(2, '/');
    let host_port = parts.next().unwrap_or("");
    let path = format!("/{}", parts.next().unwrap_or(""));

    let host = if host_port.contains(':') {
        host_port.to_string()
    } else {
        format!("{}:80", host_port)
    };

    let mut stream = TcpStream::connect(&host).map_err(|e| format!("خطای اتصال به سرور '{}': {}", host, e))?;
    let request = format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: ParvanLang/2.0\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        path, host_port, body_str.len(), body_str
    );

    stream.write_all(request.as_bytes()).map_err(|e| e.to_string())?;

    let mut response = String::new();
    stream.read_to_string(&mut response).map_err(|e| e.to_string())?;

    let resp_body = if let Some(pos) = response.find("\r\n\r\n") {
        &response[pos + 4..]
    } else {
        &response
    };

    Ok(Object::Str(resp_body.to_string()))
}
