use crate::ast::BlockStmt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Clone)]
pub enum Object {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Array(Rc<RefCell<Vec<Object>>>),
    Map(Rc<RefCell<HashMap<String, Object>>>),
    Function {
        params: Vec<String>,
        body: BlockStmt,
        env: Rc<RefCell<Environment>>,
    },
    Builtin(fn(Vec<Object>) -> Result<Object, String>),
    ReturnValue(Box<Object>),
    Break,
    Continue,
    Nil,
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Int(a), Object::Int(b)) => a == b,
            (Object::Float(a), Object::Float(b)) => a == b,
            (Object::Int(a), Object::Float(b)) => (*a as f64) == *b,
            (Object::Float(a), Object::Int(b)) => *a == (*b as f64),
            (Object::Str(a), Object::Str(b)) => a == b,
            (Object::Bool(a), Object::Bool(b)) => a == b,
            (Object::Nil, Object::Nil) => true,
            (Object::Array(a), Object::Array(b)) => *a.borrow() == *b.borrow(),
            (Object::Map(a), Object::Map(b)) => *a.borrow() == *b.borrow(),
            _ => false,
        }
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Int(v) => write!(f, "{}", v),
            Object::Float(v) => write!(f, "{}", v),
            Object::Str(v) => write!(f, "{}", v),
            Object::Bool(v) => write!(f, "{}", v),
            Object::Nil => write!(f, "nil"),
            Object::ReturnValue(v) => write!(f, "{}", v),
            Object::Break => write!(f, "<break>"),
            Object::Continue => write!(f, "<continue>"),
            Object::Builtin(_) => write!(f, "<builtin function>"),
            Object::Function { params, .. } => {
                write!(f, "<fn({})>", params.join(", "))
            }
            Object::Array(elements) => {
                let items: Vec<String> = elements.borrow().iter().map(|e| e.to_string()).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Object::Map(pairs) => {
                let items: Vec<String> = pairs
                    .borrow()
                    .iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, v))
                    .collect();
                write!(f, "{{{}}}", items.join(", "))
            }
        }
    }
}

impl Object {
    pub fn is_truthy(&self) -> bool {
        match self {
            Object::Bool(b) => *b,
            Object::Nil => false,
            Object::Int(i) => *i != 0,
            Object::Float(fl) => *fl != 0.0,
            Object::Str(s) => !s.is_empty(),
            Object::Array(arr) => !arr.borrow().is_empty(),
            _ => true,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Object::Int(_) => "Integer",
            Object::Float(_) => "Float",
            Object::Str(_) => "String",
            Object::Bool(_) => "Boolean",
            Object::Array(_) => "Array",
            Object::Map(_) => "Map",
            Object::Function { .. } => "Function",
            Object::Builtin(_) => "Builtin",
            Object::ReturnValue(_) => "ReturnValue",
            Object::Break => "Break",
            Object::Continue => "Continue",
            Object::Nil => "Nil",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Environment {
    store: HashMap<String, (Object, bool)>, // (value, is_const)
    outer: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            outer: None,
        }
    }

    pub fn new_enclosed(outer: Rc<RefCell<Environment>>) -> Self {
        Self {
            store: HashMap::new(),
            outer: Some(outer),
        }
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        match self.store.get(name) {
            Some((val, _)) => Some(val.clone()),
            None => match &self.outer {
                Some(outer) => outer.borrow().get(name),
                None => None,
            },
        }
    }

    pub fn set(&mut self, name: String, val: Object, is_const: bool) {
        self.store.insert(name, (val, is_const));
    }

    pub fn assign(&mut self, name: &str, val: Object) -> Result<(), String> {
        if let Some((target, is_const)) = self.store.get_mut(name) {
            if *is_const {
                return Err(format!("خطای انتساب: متغیر ثابت '{}' قابل تغییر نیست (const).", name));
            }
            *target = val;
            return Ok(());
        }

        if let Some(outer) = &self.outer {
            return outer.borrow_mut().assign(name, val);
        }

        Err(format!("خطای مرجع: متغیر '{}' تعریف نشده است.", name))
    }
}
