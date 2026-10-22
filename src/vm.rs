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
}

impl Vm {
    pub fn new(bytecode: Bytecode) -> Self {
        Self {
            instructions: bytecode.instructions,
            constants: bytecode.constants,
            stack: Vec::with_capacity(STACK_SIZE),
            globals: vec![Object::Nil; GLOBALS_SIZE],
            ip: 0,
        }
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
                x if x == Opcode::OpAdd as u8 => self.execute_binary_op(|a, b| match (a, b) {
                    (Object::Int(x), Object::Int(y)) => Ok(Object::Int(x + y)),
                    (Object::Float(x), Object::Float(y)) => Ok(Object::Float(x + y)),
                    (Object::Str(x), Object::Str(y)) => Ok(Object::Str(format!("{}{}", x, y))),
                    _ => Err("Unsupported types for Add".to_string()),
                })?,
                x if x == Opcode::OpSub as u8 => self.execute_binary_op(|a, b| match (a, b) {
                    (Object::Int(x), Object::Int(y)) => Ok(Object::Int(x - y)),
                    (Object::Float(x), Object::Float(y)) => Ok(Object::Float(x - y)),
                    _ => Err("Unsupported types for Sub".to_string()),
                })?,
                x if x == Opcode::OpMul as u8 => self.execute_binary_op(|a, b| match (a, b) {
                    (Object::Int(x), Object::Int(y)) => Ok(Object::Int(x * y)),
                    (Object::Float(x), Object::Float(y)) => Ok(Object::Float(x * y)),
                    _ => Err("Unsupported types for Mul".to_string()),
                })?,
                x if x == Opcode::OpDiv as u8 => self.execute_binary_op(|a, b| match (a, b) {
                    (Object::Int(x), Object::Int(y)) => {
                        if y == 0 { return Err("Division by zero in VM".to_string()); }
                        Ok(Object::Int(x / y))
                    }
                    (Object::Float(x), Object::Float(y)) => Ok(Object::Float(x / y)),
                    _ => Err("Unsupported types for Div".to_string()),
                })?,
                x if x == Opcode::OpEqual as u8 => self.execute_binary_op(|a, b| Ok(Object::Bool(a == b)))?,
                x if x == Opcode::OpNotEqual as u8 => self.execute_binary_op(|a, b| Ok(Object::Bool(a != b)))?,
                x if x == Opcode::OpTrue as u8 => self.push(Object::Bool(true))?,
                x if x == Opcode::OpFalse as u8 => self.push(Object::Bool(false))?,
                x if x == Opcode::OpNil as u8 => self.push(Object::Nil)?,
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
                x if x == Opcode::OpReturn as u8 => {
                    return Ok(self.pop().unwrap_or(Object::Nil));
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
