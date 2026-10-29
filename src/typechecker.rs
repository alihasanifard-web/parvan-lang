// ==============================================================================
// 🔬 PARVAN TYPE CHECKER & STATIC ANALYZER (بررسی‌کننده انواع و تحلیل استاتیک پروان)
// معماران: علی حسنی‌فرد، احسان فضلی‌نژاد، امیرمهدی شهبازی
// ==============================================================================

use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    List(Box<Type>),
    Map,
    Function(Vec<Type>, Box<Type>),
    Struct(String),
    Any,
    Void,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "عدد (int)"),
            Type::Float => write!(f, "اعشار (float)"),
            Type::String => write!(f, "رشته (string)"),
            Type::Bool => write!(f, "بولین (bool)"),
            Type::List(inner) => write!(f, "لیست[{}]", inner),
            Type::Map => write!(f, "نگاشت (map)"),
            Type::Function(params, ret) => {
                let p_str = params.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ");
                write!(f, "تابع({}) -> {}", p_str, ret)
            }
            Type::Struct(name) => write!(f, "ساختار({})", name),
            Type::Any => write!(f, "هرنوع (any)"),
            Type::Void => write!(f, "خالی (void)"),
        }
    }
}

pub struct TypeChecker {
    env: Vec<HashMap<String, Type>>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut globals = HashMap::new();
        globals.insert("print".to_string(), Type::Any);
        globals.insert("println".to_string(), Type::Any);
        globals.insert("چاپ".to_string(), Type::Any);
        globals.insert("چاپ_سطر".to_string(), Type::Any);
        globals.insert("len".to_string(), Type::Function(vec![Type::Any], Box::new(Type::Int)));
        globals.insert("طول".to_string(), Type::Function(vec![Type::Any], Box::new(Type::Int)));

        Self {
            env: vec![globals],
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn enter_scope(&mut self) {
        self.env.push(HashMap::new());
    }

    fn leave_scope(&mut self) {
        self.env.pop();
    }

    fn define_var(&mut self, name: &str, ty: Type) {
        if let Some(scope) = self.env.last_mut() {
            scope.insert(name.to_string(), ty);
        }
    }

    fn lookup_var(&self, name: &str) -> Option<Type> {
        for scope in self.env.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }

    pub fn check_program(&mut self, program: &Program) -> bool {
        for stmt in &program.statements {
            self.check_statement(stmt);
        }
        self.errors.is_empty()
    }

    fn check_block(&mut self, block: &BlockStmt) {
        self.enter_scope();
        for s in &block.statements {
            self.check_statement(s);
        }
        self.leave_scope();
    }

    fn check_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, value, token } | Stmt::Const { name, value, token } => {
                let val_ty = self.infer_expression(value);
                self.define_var(name, val_ty);
                if name.is_empty() {
                    self.errors.push(format!("سطر {}: نام متغیر نمی‌تواند خالی باشد.", token.line));
                }
            }
            Stmt::Return { value, .. } => {
                if let Some(e) = value {
                    self.infer_expression(e);
                }
            }
            Stmt::Expr(expr) => {
                self.infer_expression(expr);
            }
            Stmt::Print { args, .. } => {
                for a in args {
                    self.infer_expression(a);
                }
            }
            Stmt::If { condition, consequence, alternative, .. } => {
                self.infer_expression(condition);
                self.check_block(consequence);
                if let Some(alt) = alternative {
                    self.check_block(alt);
                }
            }
            Stmt::While { condition, body, .. } => {
                let cond_ty = self.infer_expression(condition);
                if cond_ty != Type::Bool && cond_ty != Type::Any && cond_ty != Type::Int {
                    self.warnings.push(format!("هشدار: شرط حلقه while باید از نوع بولین باشد، اما {} دریافت شد.", cond_ty));
                }
                self.check_block(body);
            }
            Stmt::For { var, val_var, iterable, body, .. } => {
                let iter_ty = self.infer_expression(iterable);
                self.enter_scope();
                if let Some(val_name) = val_var {
                    self.define_var(var, Type::String);
                    self.define_var(val_name, Type::Any);
                } else {
                    let elem_ty = match iter_ty {
                        Type::List(inner) => *inner,
                        _ => Type::Any,
                    };
                    self.define_var(var, elem_ty);
                }
                for s in &body.statements {
                    self.check_statement(s);
                }
                self.leave_scope();
            }
            Stmt::Struct { name, methods, .. } => {
                self.define_var(name, Type::Struct(name.clone()));
                self.enter_scope();
                self.define_var("self", Type::Struct(name.clone()));
                self.define_var("این", Type::Struct(name.clone()));
                for m in methods {
                    self.check_block(&m.body);
                }
                self.leave_scope();
            }
            Stmt::Enum { name, .. } => {
                self.define_var(name, Type::Any);
            }
            _ => {}
        }
    }

    pub fn infer_expression(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Int(_, _) => Type::Int,
            Expr::Float(_, _) => Type::Float,
            Expr::Str(_, _) => Type::String,
            Expr::Bool(_, _) => Type::Bool,
            Expr::Nil(_) => Type::Any,
            Expr::Ident(name, tok) => {
                if let Some(ty) = self.lookup_var(name) {
                    ty
                } else {
                    self.errors.push(format!("سطر {}: متغیر '{}' تعریف نشده است.", tok.line, name));
                    Type::Any
                }
            }
            Expr::Prefix(_op, right, _) => {
                self.infer_expression(right)
            }
            Expr::Infix(left, op, right, tok) => {
                let left_ty = self.infer_expression(left);
                let right_ty = self.infer_expression(right);

                match op {
                    InfixOp::Plus => {
                        if left_ty == Type::String || right_ty == Type::String {
                            Type::String
                        } else if left_ty == Type::Float || right_ty == Type::Float {
                            Type::Float
                        } else if left_ty == Type::Int && right_ty == Type::Int {
                            Type::Int
                        } else {
                            Type::Any
                        }
                    }
                    InfixOp::Minus | InfixOp::Multiply | InfixOp::Divide | InfixOp::Modulo => {
                        if (left_ty == Type::String || right_ty == Type::String) && left_ty != Type::Any && right_ty != Type::Any {
                            self.errors.push(format!("سطر {}: عملگر ریاضی روی نوع رشته مجاز نیست ({} و {}).", tok.line, left_ty, right_ty));
                        }
                        if left_ty == Type::Float || right_ty == Type::Float {
                            Type::Float
                        } else {
                            Type::Int
                        }
                    }
                    InfixOp::Equal | InfixOp::NotEqual | InfixOp::GreaterThan | InfixOp::LessThan | InfixOp::GreaterEqual | InfixOp::LessEqual => {
                        Type::Bool
                    }
                    InfixOp::And | InfixOp::Or => Type::Bool,
                }
            }
            Expr::Function { params, body, .. } => {
                self.enter_scope();
                for p in params {
                    self.define_var(p, Type::Any);
                }
                for s in &body.statements {
                    self.check_statement(s);
                }
                self.leave_scope();
                Type::Function(vec![Type::Any; params.len()], Box::new(Type::Any))
            }
            Expr::Call(func, args, tok) => {
                let func_ty = self.infer_expression(func);
                for arg in args {
                    self.infer_expression(arg);
                }
                match func_ty {
                    Type::Function(params, ret) => {
                        if params.len() != args.len() && !params.is_empty() {
                            self.warnings.push(format!("سطر {}: انتظار {} آرگومان، اما {} ارسال شد.", tok.line, params.len(), args.len()));
                        }
                        *ret
                    }
                    _ => Type::Any,
                }
            }
            Expr::Array(elements, _) => {
                if let Some(first) = elements.first() {
                    let inner = self.infer_expression(first);
                    for e in elements.iter().skip(1) {
                        self.infer_expression(e);
                    }
                    Type::List(Box::new(inner))
                } else {
                    Type::List(Box::new(Type::Any))
                }
            }
            Expr::Map(pairs, _) => {
                for (k, v) in pairs {
                    self.infer_expression(k);
                    self.infer_expression(v);
                }
                Type::Map
            }
            Expr::Index(left, idx, _) => {
                let left_ty = self.infer_expression(left);
                self.infer_expression(idx);
                match left_ty {
                    Type::List(inner) => *inner,
                    _ => Type::Any,
                }
            }
            Expr::Assign(name, val, tok) => {
                let val_ty = self.infer_expression(val);
                if let Some(existing_ty) = self.lookup_var(name) {
                    if existing_ty != Type::Any && val_ty != Type::Any && existing_ty != val_ty {
                        self.warnings.push(format!("سطر {}: انتساب نوع {} به متغیر با نوع {}.", tok.line, val_ty, existing_ty));
                    }
                } else {
                    self.errors.push(format!("سطر {}: متغیر '{}' قبل از مقداردهی تعریف نشده است.", tok.line, name));
                }
                val_ty
            }
            Expr::Await(inner, _) => {
                self.infer_expression(inner)
            }
            _ => Type::Any,
        }
    }
}
