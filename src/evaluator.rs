use crate::ast::*;
use crate::object::{Environment, Object};
use crate::stdlib::get_builtin;
use std::cell::RefCell;
use std::rc::Rc;

pub fn eval_program(program: &Program, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
    let mut result = Object::Nil;

    for stmt in &program.statements {
        result = eval_statement(stmt, env.clone())?;

        if let Object::ReturnValue(val) = result {
            return Ok(*val);
        }
    }

    Ok(result)
}

pub fn eval_statement(stmt: &Stmt, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
    match stmt {
        Stmt::Let { name, value, .. } => {
            let val = eval_expression(value, env.clone())?;
            env.borrow_mut().set(name.clone(), val, false);
            Ok(Object::Nil)
        }
        Stmt::Const { name, value, .. } => {
            let val = eval_expression(value, env.clone())?;
            env.borrow_mut().set(name.clone(), val, true);
            Ok(Object::Nil)
        }
        Stmt::Return { value, .. } => {
            let val = match value {
                Some(expr) => eval_expression(expr, env)?,
                None => Object::Nil,
            };
            Ok(Object::ReturnValue(Box::new(val)))
        }
        Stmt::Expr(expr) => eval_expression(expr, env),
        Stmt::Print { args, .. } => {
            let mut outputs = Vec::new();
            for arg in args {
                let val = eval_expression(arg, env.clone())?;
                let text = val.to_string();
                let shaped = crate::persian::reshape_persian(&text);
                outputs.push(shaped);
            }
            println!("{}", outputs.join(" "));
            Ok(Object::Nil)
        }
        Stmt::If { condition, consequence, alternative, .. } => {
            let cond_val = eval_expression(condition, env.clone())?;
            if cond_val.is_truthy() {
                eval_block_statement(consequence, env)
            } else if let Some(alt) = alternative {
                eval_block_statement(alt, env)
            } else {
                Ok(Object::Nil)
            }
        }
        Stmt::While { condition, body, .. } => {
            let mut result = Object::Nil;
            while eval_expression(condition, env.clone())?.is_truthy() {
                result = eval_block_statement(body, env.clone())?;
                if let Object::ReturnValue(_) = result {
                    return Ok(result);
                }
                if let Object::Break = result {
                    break;
                }
            }
            Ok(result)
        }
        Stmt::For { var, val_var, iterable, body, .. } => {
            let iter_val = eval_expression(iterable, env.clone())?;
            match iter_val {
                Object::Map(m) => {
                    let pairs: Vec<(String, Object)> = m.borrow().iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                    let mut result = Object::Nil;
                    for (k, v) in pairs {
                        env.borrow_mut().set(var.clone(), Object::Str(k.clone()), false);
                        if let Some(v_name) = val_var {
                            env.borrow_mut().set(v_name.clone(), v, false);
                        }
                        result = eval_block_statement(body, env.clone())?;
                        if let Object::ReturnValue(_) = result {
                            return Ok(result);
                        }
                        if let Object::Break = result {
                            break;
                        }
                    }
                    Ok(result)
                }
                Object::Array(arr) => {
                    let items = arr.borrow().clone();
                    let mut result = Object::Nil;
                    for (idx, item) in items.into_iter().enumerate() {
                        if let Some(v_name) = val_var {
                            // for [idx, item] in array
                            env.borrow_mut().set(var.clone(), Object::Int(idx as i64), false);
                            env.borrow_mut().set(v_name.clone(), item, false);
                        } else {
                            env.borrow_mut().set(var.clone(), item, false);
                        }
                        result = eval_block_statement(body, env.clone())?;
                        if let Object::ReturnValue(_) = result {
                            return Ok(result);
                        }
                        if let Object::Break = result {
                            break;
                        }
                    }
                    Ok(result)
                }
                Object::Str(s) => {
                    let items: Vec<Object> = s.chars().map(|c| Object::Str(c.to_string())).collect();
                    let mut result = Object::Nil;
                    for (idx, item) in items.into_iter().enumerate() {
                        if let Some(v_name) = val_var {
                            env.borrow_mut().set(var.clone(), Object::Int(idx as i64), false);
                            env.borrow_mut().set(v_name.clone(), item, false);
                        } else {
                            env.borrow_mut().set(var.clone(), item, false);
                        }
                        result = eval_block_statement(body, env.clone())?;
                        if let Object::ReturnValue(_) = result {
                            return Ok(result);
                        }
                        if let Object::Break = result {
                            break;
                        }
                    }
                    Ok(result)
                }
                _ => return Err("دستور for فقط روی آرایه‌ها، دیکشنری‌ها، بازه‌ها یا رشته‌ها کار می‌کند.".to_string()),
            }
        }
        Stmt::Break(_) => Ok(Object::Break),
        Stmt::Continue(_) => Ok(Object::Continue),
        Stmt::Import { path, alias, token } => {
            let code = std::fs::read_to_string(path)
                .map_err(|e| format!("خطای وارد کردن ماژول '{}' در سطر {}: {}", path, token.line, e))?;
            let lexer = crate::lexer::Lexer::new(&code);
            let mut parser = crate::parser::Parser::new(lexer);
            let program = parser.parse_program();
            if !parser.errors.is_empty() {
                return Err(format!("خطاهای نحوی در ماژول وارد شده '{}': {:?}", path, parser.errors));
            }
            if let Some(alias_name) = alias {
                let module_env = Rc::new(RefCell::new(Environment::new()));
                eval_program(&program, module_env.clone())?;
                let bindings = module_env.borrow().get_all_bindings();
                env.borrow_mut().set(alias_name.clone(), Object::Map(Rc::new(RefCell::new(bindings))), false);
            } else {
                eval_program(&program, env)?;
            }
            Ok(Object::Nil)
        }
        Stmt::TryCatch { try_block, err_var, catch_block, .. } => {
            match eval_block_statement(try_block, env.clone()) {
                Ok(val) => Ok(val),
                Err(e) => {
                    let catch_env = Rc::new(RefCell::new(Environment::new_enclosed(env)));
                    catch_env.borrow_mut().set(err_var.clone(), Object::Str(e), false);
                    eval_block_statement(catch_block, catch_env)
                }
            }
        }
        Stmt::Switch { condition, cases, default, .. } => {
            let target_val = eval_expression(condition, env.clone())?;
            let mut matched = false;
            let mut result = Object::Nil;

            for (case_expr, body) in cases {
                let case_val = eval_expression(case_expr, env.clone())?;
                if case_val == target_val {
                    matched = true;
                    result = eval_block_statement(body, env.clone())?;
                    break;
                }
            }

            if !matched {
                if let Some(def_block) = default {
                    result = eval_block_statement(def_block, env)?;
                }
            }

            Ok(result)
        }
        Stmt::Struct { name, fields, methods, .. } => {
            let mut method_map = std::collections::HashMap::new();
            for m in methods {
                method_map.insert(
                    m.name.clone(),
                    (m.params.clone(), m.body.clone(), env.clone()),
                );
            }
            let struct_obj = Object::Struct {
                name: name.clone(),
                fields: fields.clone(),
                methods: method_map,
            };
            env.borrow_mut().set(name.clone(), struct_obj, false);
            Ok(Object::Nil)
        }
        Stmt::Enum { name, variants, .. } => {
            let mut variant_map = std::collections::HashMap::new();
            for v in variants {
                variant_map.insert(v.clone(), Object::Str(format!("{}::{}", name, v)));
            }
            let enum_obj = Object::Map(Rc::new(RefCell::new(variant_map)));
            env.borrow_mut().set(name.clone(), enum_obj, false);
            Ok(Object::Nil)
        }
        Stmt::Match { condition, arms, default, .. } => {
            let target_val = eval_expression(condition, env.clone())?;
            let mut matched = false;
            let mut result = Object::Nil;

            for (pat_expr, body) in arms {
                let pat_val = eval_expression(pat_expr, env.clone())?;
                if pat_val == target_val {
                    matched = true;
                    result = eval_block_statement(body, env.clone())?;
                    break;
                }
            }

            if !matched {
                if let Some(def_block) = default {
                    result = eval_block_statement(def_block, env)?;
                }
            }

            Ok(result)
        }
    }
}

pub fn eval_block_statement(block: &BlockStmt, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
    let mut result = Object::Nil;
    // Scoped block
    let block_env = Rc::new(RefCell::new(Environment::new_enclosed(env)));

    for stmt in &block.statements {
        result = eval_statement(stmt, block_env.clone())?;
        match result {
            Object::ReturnValue(_) | Object::Break | Object::Continue => return Ok(result),
            _ => {}
        }
    }

    Ok(result)
}

pub fn eval_expression(expr: &Expr, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
    match expr {
        Expr::Int(val, _) => Ok(Object::Int(*val)),
        Expr::Float(val, _) => Ok(Object::Float(*val)),
        Expr::Str(val, _) => Ok(Object::Str(val.clone())),
        Expr::Bool(val, _) => Ok(Object::Bool(*val)),
        Expr::Nil(_) => Ok(Object::Nil),
        Expr::Ident(name, tok) => {
            if let Some(val) = env.borrow().get(name) {
                return Ok(val);
            }
            if let Some(builtin) = get_builtin(name) {
                return Ok(builtin);
            }
            Err(format!("خطا در سطر {}: متغیر یا تابع '{}' یافت نشد.", tok.line, name))
        }
        Expr::Assign(name, val_expr, tok) => {
            let val = eval_expression(val_expr, env.clone())?;
            env.borrow_mut().assign(name, val.clone())
                .map_err(|e| format!("خطا در سطر {}: {}", tok.line, e))?;
            Ok(val)
        }
        Expr::Prefix(op, right_expr, tok) => {
            let right = eval_expression(right_expr, env)?;
            eval_prefix_op(op, right, tok)
        }
        Expr::Infix(left_expr, op, right_expr, tok) => {
            // Short-circuit logical operators
            if *op == InfixOp::And {
                let left = eval_expression(left_expr, env.clone())?;
                if !left.is_truthy() {
                    return Ok(Object::Bool(false));
                }
                let right = eval_expression(right_expr, env)?;
                return Ok(Object::Bool(right.is_truthy()));
            }
            if *op == InfixOp::Or {
                let left = eval_expression(left_expr, env.clone())?;
                if left.is_truthy() {
                    return Ok(Object::Bool(true));
                }
                let right = eval_expression(right_expr, env)?;
                return Ok(Object::Bool(right.is_truthy()));
            }

            let left = eval_expression(left_expr, env.clone())?;
            let right = eval_expression(right_expr, env)?;
            eval_infix_op(left, op, right, tok)
        }
        Expr::Array(elements, _) => {
            let mut list = Vec::new();
            for el in elements {
                list.push(eval_expression(el, env.clone())?);
            }
            Ok(Object::Array(Rc::new(RefCell::new(list))))
        }
        Expr::Map(pairs, _) => {
            let mut map = std::collections::HashMap::new();
            for (k_expr, v_expr) in pairs {
                let k_obj = eval_expression(k_expr, env.clone())?;
                let v_obj = eval_expression(v_expr, env.clone())?;
                let k_str = match k_obj {
                    Object::Str(s) => s,
                    Object::Int(i) => i.to_string(),
                    other => other.to_string(),
                };
                map.insert(k_str, v_obj);
            }
            Ok(Object::Map(Rc::new(RefCell::new(map))))
        }
        Expr::Index(left_expr, index_expr, tok) => {
            let left = eval_expression(left_expr, env.clone())?;
            let idx = eval_expression(index_expr, env)?;
            eval_index_op(left, idx, tok)
        }
        Expr::Property(left_expr, prop_name, tok) => {
            let left = eval_expression(left_expr, env.clone())?;
            match left {
                Object::Map(m) => {
                    let borrowed = m.borrow();
                    match borrowed.get(prop_name) {
                        Some(val) => Ok(val.clone()),
                        None => Ok(Object::Nil),
                    }
                }
                Object::Instance { ref fields, ref methods, ref struct_name } => {
                    // Check field first
                    if let Some(val) = fields.borrow().get(prop_name) {
                        return Ok(val.clone());
                    }
                    // Check method: bind self to instance!
                    if let Some((params, body, def_env)) = methods.get(prop_name) {
                        let method_env = Rc::new(RefCell::new(Environment::new_enclosed(def_env.clone())));
                        method_env.borrow_mut().set("self".to_string(), left.clone(), false);
                        method_env.borrow_mut().set("این".to_string(), left.clone(), false);
                        method_env.borrow_mut().set("خود".to_string(), left.clone(), false);
                        return Ok(Object::Function {
                            params: params.clone(),
                            body: body.clone(),
                            env: method_env,
                        });
                    }
                    Err(format!("فیلد یا متد '{}' در ساختار '{}' تعریف نشده است در سطر {}", prop_name, struct_name, tok.line))
                }
                _ => Err(format!("دسترسی با نقطه فقط روی دیکشنری یا نمونه ساختار مجاز است در سطر {}", tok.line)),
            }
        }
        Expr::PropertyAssign(left_expr, prop_name, val_expr, tok) => {
            let left = eval_expression(left_expr, env.clone())?;
            let val = eval_expression(val_expr, env)?;
            match left {
                Object::Map(m) => {
                    m.borrow_mut().insert(prop_name.clone(), val.clone());
                    Ok(val)
                }
                Object::Instance { ref fields, .. } => {
                    fields.borrow_mut().insert(prop_name.clone(), val.clone());
                    Ok(val)
                }
                _ => Err(format!("تغییر ویژگی با نقطه فقط روی دیکشنری یا شیء مجاز است در سطر {}", tok.line)),
            }
        }
        Expr::IndexAssign(left_expr, idx_expr, val_expr, tok) => {
            let left = eval_expression(left_expr, env.clone())?;
            let idx = eval_expression(idx_expr, env.clone())?;
            let val = eval_expression(val_expr, env)?;

            match (left, idx) {
                (Object::Array(arr), Object::Int(i)) => {
                    let mut borrowed = arr.borrow_mut();
                    let len = borrowed.len();
                    let real_idx = if i < 0 {
                        (len as i64 + i) as usize
                    } else {
                        i as usize
                    };
                    if real_idx >= len {
                        return Err(format!("ایندکس {} خارج از محدوده آرایه با طول {} است در سطر {}", i, len, tok.line));
                    }
                    borrowed[real_idx] = val.clone();
                    Ok(val)
                }
                (Object::Map(m), Object::Str(key)) => {
                    m.borrow_mut().insert(key, val.clone());
                    Ok(val)
                }
                (Object::Map(m), other) => {
                    m.borrow_mut().insert(other.to_string(), val.clone());
                    Ok(val)
                }
                (other, _) => Err(format!("عملیات تغییر ایندکس روی نوع '{}' مجاز نیست در سطر {}", other.type_name(), tok.line)),
            }
        }
        Expr::Function { name, params, body, .. } => {
            let func_obj = Object::Function {
                params: params.clone(),
                body: body.clone(),
                env: env.clone(),
            };
            if let Some(fn_name) = name {
                env.borrow_mut().set(fn_name.clone(), func_obj.clone(), false);
            }
            Ok(func_obj)
        }
        Expr::Call(func_expr, arg_exprs, tok) => {
            let func = eval_expression(func_expr, env.clone())?;
            let mut args = Vec::new();
            for arg_e in arg_exprs {
                args.push(eval_expression(arg_e, env.clone())?);
            }
            apply_function(func, args, tok)
        }
    }
}

fn eval_prefix_op(op: &PrefixOp, right: Object, tok: &crate::token::Token) -> Result<Object, String> {
    match op {
        PrefixOp::Bang => Ok(Object::Bool(!right.is_truthy())),
        PrefixOp::Minus => match right {
            Object::Int(i) => Ok(Object::Int(-i)),
            Object::Float(f) => Ok(Object::Float(-f)),
            _ => Err(format!("خطا در سطر {}: عملگر منفی روی نوع '{}' مجاز نیست.", tok.line, right.type_name())),
        },
    }
}

fn eval_infix_op(left: Object, op: &InfixOp, right: Object, tok: &crate::token::Token) -> Result<Object, String> {
    // String Concatenation
    if *op == InfixOp::Plus {
        if let (Object::Str(a), b) = (&left, &right) {
            return Ok(Object::Str(format!("{}{}", a, b)));
        }
        if let (a, Object::Str(b)) = (&left, &right) {
            return Ok(Object::Str(format!("{}{}", a, b)));
        }
    }

    match (left, right) {
        (Object::Int(a), Object::Int(b)) => match op {
            InfixOp::Plus => Ok(Object::Int(a + b)),
            InfixOp::Minus => Ok(Object::Int(a - b)),
            InfixOp::Multiply => Ok(Object::Int(a * b)),
            InfixOp::Divide => {
                if b == 0 { return Err(format!("خطای تقسیم بر صفر در سطر {}", tok.line)); }
                Ok(Object::Int(a / b))
            }
            InfixOp::Modulo => {
                if b == 0 { return Err(format!("خطای باقیمانده بر صفر در سطر {}", tok.line)); }
                Ok(Object::Int(a % b))
            }
            InfixOp::Equal => Ok(Object::Bool(a == b)),
            InfixOp::NotEqual => Ok(Object::Bool(a != b)),
            InfixOp::LessThan => Ok(Object::Bool(a < b)),
            InfixOp::LessEqual => Ok(Object::Bool(a <= b)),
            InfixOp::GreaterThan => Ok(Object::Bool(a > b)),
            InfixOp::GreaterEqual => Ok(Object::Bool(a >= b)),
            _ => Err(format!("عملگر نامعتبر برای دو عدد صحیح در سطر {}", tok.line)),
        },
        (Object::Float(a), Object::Float(b)) => match op {
            InfixOp::Plus => Ok(Object::Float(a + b)),
            InfixOp::Minus => Ok(Object::Float(a - b)),
            InfixOp::Multiply => Ok(Object::Float(a * b)),
            InfixOp::Divide => Ok(Object::Float(a / b)),
            InfixOp::Equal => Ok(Object::Bool(a == b)),
            InfixOp::NotEqual => Ok(Object::Bool(a != b)),
            InfixOp::LessThan => Ok(Object::Bool(a < b)),
            InfixOp::LessEqual => Ok(Object::Bool(a <= b)),
            InfixOp::GreaterThan => Ok(Object::Bool(a > b)),
            InfixOp::GreaterEqual => Ok(Object::Bool(a >= b)),
            _ => Err(format!("عملگر نامعتبر برای اعداد اعشاری در سطر {}", tok.line)),
        },
        (Object::Int(a), Object::Float(b)) => eval_infix_op(Object::Float(a as f64), op, Object::Float(b), tok),
        (Object::Float(a), Object::Int(b)) => eval_infix_op(Object::Float(a), op, Object::Float(b as f64), tok),
        (Object::Bool(a), Object::Bool(b)) => match op {
            InfixOp::Equal => Ok(Object::Bool(a == b)),
            InfixOp::NotEqual => Ok(Object::Bool(a != b)),
            _ => Err(format!("عملگر نامعتبر روی مقادیر بولی در سطر {}", tok.line)),
        },
        (Object::Str(a), Object::Str(b)) => match op {
            InfixOp::Equal => Ok(Object::Bool(a == b)),
            InfixOp::NotEqual => Ok(Object::Bool(a != b)),
            _ => Err(format!("عملگر نامعتبر روی رشته‌ها در سطر {}", tok.line)),
        },
        (Object::Bool(_a), Object::Str(_b)) => match op {
            InfixOp::Equal => Ok(Object::Bool(false)),
            InfixOp::NotEqual => Ok(Object::Bool(true)),
            _ => Err(format!("عملگر نامعتبر در سطر {}", tok.line)),
        },
        (Object::Str(_a), Object::Bool(_b)) => match op {
            InfixOp::Equal => Ok(Object::Bool(false)),
            InfixOp::NotEqual => Ok(Object::Bool(true)),
            _ => Err(format!("عملگر نامعتبر در سطر {}", tok.line)),
        },
        (a, b) => match op {
            InfixOp::Equal => Ok(Object::Bool(a == b)),
            InfixOp::NotEqual => Ok(Object::Bool(a != b)),
            _ => Err(format!(
                "خطا در سطر {}: عملگر نامعتبر بین نوع '{}' و '{}'.",
                tok.line, a.type_name(), b.type_name()
            )),
        },
    }
}

fn eval_index_op(left: Object, index: Object, tok: &crate::token::Token) -> Result<Object, String> {
    match (&left, &index) {
        (Object::Array(arr), Object::Int(i)) => {
            let borrowed = arr.borrow();
            let idx = if *i < 0 {
                (borrowed.len() as i64 + *i) as usize
            } else {
                *i as usize
            };

            if idx >= borrowed.len() {
                return Err(format!(
                    "خطا در سطر {}: ایندکس {} خارج از محدوده آرایه است (طول: {}).",
                    tok.line, i, borrowed.len()
                ));
            }
            Ok(borrowed[idx].clone())
        }
        (Object::Str(s), Object::Int(i)) => {
            let chars: Vec<char> = s.chars().collect();
            let idx = if *i < 0 {
                (chars.len() as i64 + *i) as usize
            } else {
                *i as usize
            };

            if idx >= chars.len() {
                return Err(format!("ایندکس رشته خارج از محدوده در سطر {}", tok.line));
            }
            Ok(Object::Str(chars[idx].to_string()))
        }
        (Object::Map(m), Object::Str(key)) => {
            let borrowed = m.borrow();
            match borrowed.get(key) {
                Some(val) => Ok(val.clone()),
                None => Ok(Object::Nil),
            }
        }
        (Object::Map(m), other) => {
            let key = other.to_string();
            let borrowed = m.borrow();
            match borrowed.get(&key) {
                Some(val) => Ok(val.clone()),
                None => Ok(Object::Nil),
            }
        }
        _ => Err(format!("عملیات ایندکس روی نوع '{}' با ایندکس '{}' مجاز نیست.", left.type_name(), index.type_name())),
    }
}

fn apply_function(func: Object, args: Vec<Object>, tok: &crate::token::Token) -> Result<Object, String> {
    match func {
        Object::Builtin(builtin) => builtin(args).map_err(|e| format!("خطا در تابع در سطر {}: {}", tok.line, e)),
        Object::Struct { name, fields, methods } => {
            if fields.len() != args.len() {
                return Err(format!(
                    "خطا در ایجاد نمونه ساختار '{}' در سطر {}: انتظار {} فیلد ({}) می‌رفت اما {} آرگومان داده شد.",
                    name, tok.line, fields.len(), fields.join(", "), args.len()
                ));
            }
            let mut field_map = std::collections::HashMap::new();
            for (field_name, arg_val) in fields.iter().zip(args.into_iter()) {
                field_map.insert(field_name.clone(), arg_val);
            }
            Ok(Object::Instance {
                struct_name: name,
                fields: Rc::new(RefCell::new(field_map)),
                methods,
            })
        }
        Object::Function { params, body, env } => {
            if params.len() != args.len() {
                return Err(format!(
                    "خطا در فراخوانی تابع در سطر {}: انتظار {} آرگومان می‌رفت اما {} آرگومان ارسال شد.",
                    tok.line, params.len(), args.len()
                ));
            }

            let extended_env = Rc::new(RefCell::new(Environment::new_enclosed(env)));
            for (param, arg) in params.iter().zip(args.into_iter()) {
                extended_env.borrow_mut().set(param.clone(), arg, false);
            }

            let evaluated = eval_block_statement(&body, extended_env)?;
            if let Object::ReturnValue(val) = evaluated {
                Ok(*val)
            } else {
                Ok(evaluated)
            }
        }
        _ => Err(format!("خطا در سطر {}: شیء از نوع '{}' قابل فراخوانی نیست.", tok.line, func.type_name())),
    }
}
