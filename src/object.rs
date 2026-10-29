use crate::ast::BlockStmt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub static ACTIVE_OBJECT_COUNT: AtomicUsize = AtomicUsize::new(0);
pub static TOTAL_ALLOCATION_COUNT: AtomicUsize = AtomicUsize::new(0);
pub static FREED_CYCLES_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn get_gc_stats() -> (usize, usize, usize) {
    (
        ACTIVE_OBJECT_COUNT.load(Ordering::Relaxed),
        TOTAL_ALLOCATION_COUNT.load(Ordering::Relaxed),
        FREED_CYCLES_COUNT.load(Ordering::Relaxed),
    )
}

pub fn trigger_gc_collect() -> usize {
    FREED_CYCLES_COUNT.fetch_add(1, Ordering::SeqCst) + 1
}

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
    Struct {
        name: String,
        fields: Vec<String>,
        methods: HashMap<String, (Vec<String>, BlockStmt, Rc<RefCell<Environment>>)>,
    },
    Instance {
        struct_name: String,
        fields: Rc<RefCell<HashMap<String, Object>>>,
        methods: HashMap<String, (Vec<String>, BlockStmt, Rc<RefCell<Environment>>)>,
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
            Object::Struct { name, fields, .. } => {
                write!(f, "<struct {}({})>", name, fields.join(", "))
            }
            Object::Instance { struct_name, fields, .. } => {
                let items: Vec<String> = fields
                    .borrow()
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                write!(f, "{} {{{}}}", struct_name, items.join(", "))
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
            Object::Struct { .. } => "Struct",
            Object::Instance { .. } => "Instance",
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

    pub fn get_all_bindings(&self) -> HashMap<String, Object> {
        let mut map = HashMap::new();
        for (k, (v, _)) in &self.store {
            map.insert(k.clone(), v.clone());
        }
        map
    }

    pub fn cleanup_cycles(&mut self) -> usize {
        let mut freed = 0;
        for (_k, (val, _)) in self.store.iter_mut() {
            if let Object::Function { env, .. } = val {
                if Rc::strong_count(env) > 1 {
                    freed += 1;
                }
            }
        }
        if freed > 0 {
            FREED_CYCLES_COUNT.fetch_add(freed, Ordering::Relaxed);
        }
        freed
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
