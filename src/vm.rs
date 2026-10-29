use crate::code::{Bytecode, Opcode};
use crate::object::Object;

const STACK_SIZE: usize = 2048;
const GLOBALS_SIZE: usize = 65536;

pub struct Vm {
    instructions: Vec<u8>,
    constants: Vec<Object>,
    stack: Vec<Object>,
    globals: Vec<Object>,
    ip: usize, // Instruction Pointer
    pub hot_loops: std::collections::HashMap<usize, usize>,
}

impl Vm {
    pub fn new(bytecode: Bytecode) -> Self {
        Self {
            instructions: bytecode.instructions,
            constants: bytecode.constants,
            stack: Vec::with_capacity(STACK_SIZE),
            globals: vec![Object::Nil; GLOBALS_SIZE],
            ip: 0,
            hot_loops: std::collections::HashMap::new(),
        }
    }

    pub fn hot_loop_count(&self) -> usize {
        self.hot_loops.values().filter(|&&v| v >= 5).count()
    }

    pub fn run(&mut self) -> Result<Object, String> {
        while self.ip < self.instructions.len() {
            let op_byte = self.instructions[self.ip];
            self.ip += 1;

            match op_byte {
                x if x == Opcode::OpConstant as u8 => {
                    let const_idx = self.read_u16();
                    let obj = self.constants[const_idx as usize].clone();
                    self.push(obj)?;
                }
                x if x == Opcode::OpAdd as u8 => {
                    let len = self.stack.len();
                    if len >= 2 {
                        if let (Object::Int(a), Object::Int(b)) = (&self.stack[len - 2], &self.stack[len - 1]) {
                            let res = a + b;
                            self.stack.pop();
                            self.stack[len - 2] = Object::Int(res);
                            continue;
                        }
                    }
                    self.execute_binary_op(|a, b| match (a, b) {
                        (Object::Int(x), Object::Int(y)) => Ok(Object::Int(x + y)),
                        (Object::Float(x), Object::Float(y)) => Ok(Object::Float(x + y)),
                        (Object::Int(x), Object::Float(y)) => Ok(Object::Float(x as f64 + y)),
                        (Object::Float(x), Object::Int(y)) => Ok(Object::Float(x + y as f64)),
                        (Object::Str(x), Object::Str(y)) => Ok(Object::Str(format!("{}{}", x, y))),
                        (Object::Str(x), other) => Ok(Object::Str(format!("{}{}", x, other))),
                        (other, Object::Str(y)) => Ok(Object::Str(format!("{}{}", other, y))),
                        _ => Err("Unsupported types for Add".to_string()),
                    })?
                }
                x if x == Opcode::OpSub as u8 => {
                    let len = self.stack.len();
                    if len >= 2 {
                        if let (Object::Int(a), Object::Int(b)) = (&self.stack[len - 2], &self.stack[len - 1]) {
                            let res = a - b;
                            self.stack.pop();
                            self.stack[len - 2] = Object::Int(res);
                            continue;
                        }
                    }
                    self.execute_binary_op(|a, b| match (a, b) {
                        (Object::Int(x), Object::Int(y)) => Ok(Object::Int(x - y)),
                        (Object::Float(x), Object::Float(y)) => Ok(Object::Float(x - y)),
                        _ => Err("Unsupported types for Sub".to_string()),
                    })?
                }
                x if x == Opcode::OpMul as u8 => {
                    let len = self.stack.len();
                    if len >= 2 {
                        if let (Object::Int(a), Object::Int(b)) = (&self.stack[len - 2], &self.stack[len - 1]) {
                            let res = a * b;
                            self.stack.pop();
                            self.stack[len - 2] = Object::Int(res);
                            continue;
                        }
                    }
                    self.execute_binary_op(|a, b| match (a, b) {
                        (Object::Int(x), Object::Int(y)) => Ok(Object::Int(x * y)),
                        (Object::Float(x), Object::Float(y)) => Ok(Object::Float(x * y)),
                        _ => Err("Unsupported types for Mul".to_string()),
                    })?
                }
                x if x == Opcode::OpDiv as u8 => self.execute_binary_op(|a, b| match (a, b) {
                    (Object::Int(x), Object::Int(y)) => {
                        if y == 0 { return Err("Division by zero in VM".to_string()); }
                        Ok(Object::Int(x / y))
                    }
                    (Object::Float(x), Object::Float(y)) => Ok(Object::Float(x / y)),
                    _ => Err("Unsupported types for Div".to_string()),
                })?,
                x if x == Opcode::OpMod as u8 => self.execute_binary_op(|a, b| match (a, b) {
                    (Object::Int(x), Object::Int(y)) => {
                        if y == 0 { return Err("Modulo by zero in VM".to_string()); }
                        Ok(Object::Int(x % y))
                    }
                    _ => Err("Unsupported types for Modulo".to_string()),
                })?,
                x if x == Opcode::OpEqual as u8 => self.execute_binary_op(|a, b| Ok(Object::Bool(a == b)))?,
                x if x == Opcode::OpNotEqual as u8 => self.execute_binary_op(|a, b| Ok(Object::Bool(a != b)))?,
                x if x == Opcode::OpGreaterThan as u8 => self.execute_binary_op(|a, b| match (a, b) {
                    (Object::Int(x), Object::Int(y)) => Ok(Object::Bool(x > y)),
                    (Object::Float(x), Object::Float(y)) => Ok(Object::Bool(x > y)),
                    _ => Err("Invalid types for > comparison".to_string()),
                })?,
                x if x == Opcode::OpGreaterEqual as u8 => self.execute_binary_op(|a, b| match (a, b) {
                    (Object::Int(x), Object::Int(y)) => Ok(Object::Bool(x >= y)),
                    (Object::Float(x), Object::Float(y)) => Ok(Object::Bool(x >= y)),
                    _ => Err("Invalid types for >= comparison".to_string()),
                })?,
                x if x == Opcode::OpLessThan as u8 => self.execute_binary_op(|a, b| match (a, b) {
                    (Object::Int(x), Object::Int(y)) => Ok(Object::Bool(x < y)),
                    (Object::Float(x), Object::Float(y)) => Ok(Object::Bool(x < y)),
                    _ => Err("Invalid types for < comparison".to_string()),
                })?,
                x if x == Opcode::OpLessEqual as u8 => self.execute_binary_op(|a, b| match (a, b) {
                    (Object::Int(x), Object::Int(y)) => Ok(Object::Bool(x <= y)),
                    (Object::Float(x), Object::Float(y)) => Ok(Object::Bool(x <= y)),
                    _ => Err("Invalid types for <= comparison".to_string()),
                })?,
                x if x == Opcode::OpTrue as u8 => self.push(Object::Bool(true))?,
                x if x == Opcode::OpFalse as u8 => self.push(Object::Bool(false))?,
                x if x == Opcode::OpNil as u8 => self.push(Object::Nil)?,
                x if x == Opcode::OpMinus as u8 => {
                    let val = self.pop()?;
                    match val {
                        Object::Int(i) => self.push(Object::Int(-i))?,
                        Object::Float(f) => self.push(Object::Float(-f))?,
                        _ => return Err("Unary minus requires a numeric operand in VM".to_string()),
                    }
                }
                x if x == Opcode::OpBang as u8 => {
                    let val = self.pop()?;
                    self.push(Object::Bool(!val.is_truthy()))?;
                }
                x if x == Opcode::OpSetGlobal as u8 => {
                    let global_idx = self.read_u16() as usize;
                    let val = self.pop()?;
                    self.globals[global_idx] = val;
                }
                x if x == Opcode::OpGetGlobal as u8 => {
                    let global_idx = self.read_u16() as usize;
                    let val = self.globals[global_idx].clone();
                    self.push(val)?;
                }
                x if x == Opcode::OpJump as u8 => {
                    let target = self.read_u16() as usize;
                    if target <= self.ip {
                        *self.hot_loops.entry(target).or_insert(0) += 1;
                    }
                    self.ip = target;
                }
                x if x == Opcode::OpJumpIfFalse as u8 => {
                    let target = self.read_u16() as usize;
                    let cond = self.pop()?;
                    if !cond.is_truthy() {
                        self.ip = target;
                    }
                }
                x if x == Opcode::OpReturn as u8 => {
                    return Ok(self.pop().unwrap_or(Object::Nil));
                }
                x if x == Opcode::OpCall as u8 => {
                    let num_args = self.instructions[self.ip] as usize;
                    self.ip += 1;

                    let callee = self.pop()?;
                    let mut args = Vec::with_capacity(num_args);
                    for _ in 0..num_args {
                        args.push(self.pop()?);
                    }
                    args.reverse();

                    match callee {
                        Object::Builtin(builtin) => {
                            let result = builtin(args)?;
                            self.push(result)?;
                        }
                        Object::Function { params, body, env } => {
                            if params.len() != args.len() {
                                return Err(format!("تعداد آرگومان نامعتبر: انتظار {} می‌رفت اما {} ارسال شد", params.len(), args.len()));
                            }
                            let func_env = std::rc::Rc::new(std::cell::RefCell::new(crate::object::Environment::new_enclosed(env)));
                            for (p, a) in params.iter().zip(args.into_iter()) {
                                func_env.borrow_mut().set(p.clone(), a, false);
                            }
                            let res = crate::evaluator::eval_block_statement(&body, func_env)?;
                            if let Object::ReturnValue(v) = res {
                                self.push(*v)?;
                            } else {
                                self.push(res)?;
                            }
                        }
                        other => return Err(format!("Callable object of type '{}' not yet supported on VM", other.type_name())),
                    }
                }
                unknown => return Err(format!("Unknown opcode byte: {}", unknown)),
            }
        }

        Ok(self.stack.last().cloned().unwrap_or(Object::Nil))
    }

    fn push(&mut self, obj: Object) -> Result<(), String> {
        if self.stack.len() >= STACK_SIZE {
            return Err("Stack overflow in Parvan VM".to_string());
        }
        self.stack.push(obj);
        Ok(())
    }

    fn pop(&mut self) -> Result<Object, String> {
        self.stack.pop().ok_or_else(|| "Stack underflow in Parvan VM".to_string())
    }

    fn read_u16(&mut self) -> u16 {
        let b1 = self.instructions[self.ip] as u16;
        let b2 = self.instructions[self.ip + 1] as u16;
        self.ip += 2;
        (b1 << 8) | b2
    }

    fn execute_binary_op<F>(&mut self, op: F) -> Result<(), String>
    where
        F: FnOnce(Object, Object) -> Result<Object, String>,
    {
        let right = self.pop()?;
        let left = self.pop()?;
        let res = op(left, right)?;
        self.push(res)
    }
}
