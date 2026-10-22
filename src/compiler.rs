use crate::ast::*;
use crate::code::{Bytecode, Opcode};
use crate::object::Object;
use std::collections::HashMap;

pub struct Compiler {
    instructions: Vec<u8>,
    constants: Vec<Object>,
    symbol_table: HashMap<String, u16>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            symbol_table: HashMap::new(),
        }
    }

    pub fn compile(&mut self, program: &Program) -> Result<Bytecode, String> {
        for stmt in &program.statements {
            self.compile_statement(stmt)?;
        }

        Ok(Bytecode {
            instructions: self.instructions.clone(),
            constants: self.constants.clone(),
        })
    }

    fn compile_statement(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Let { name, value, .. } | Stmt::Const { name, value, .. } => {
                self.compile_expression(value)?;
                let next_idx = self.symbol_table.len() as u16;
                let symbol_idx = *self.symbol_table.entry(name.clone()).or_insert(next_idx);
                self.emit(Opcode::OpSetGlobal);
                self.emit_u16(symbol_idx);
                Ok(())
            }
            Stmt::Expr(expr) => {
                self.compile_expression(expr)?;
                Ok(())
            }
            Stmt::Return { value, .. } => {
                if let Some(v) = value {
                    self.compile_expression(v)?;
                } else {
                    self.emit(Opcode::OpNil);
                }
                self.emit(Opcode::OpReturn);
                Ok(())
            }
            _ => Err("Statement not yet supported in Bytecode compiler".to_string()),
        }
    }

    fn compile_expression(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Int(val, _) => {
                let const_idx = self.add_constant(Object::Int(*val));
                self.emit(Opcode::OpConstant);
                self.emit_u16(const_idx);
                Ok(())
            }
            Expr::Float(val, _) => {
                let const_idx = self.add_constant(Object::Float(*val));
                self.emit(Opcode::OpConstant);
                self.emit_u16(const_idx);
                Ok(())
            }
            Expr::Str(val, _) => {
                let const_idx = self.add_constant(Object::Str(val.clone()));
                self.emit(Opcode::OpConstant);
                self.emit_u16(const_idx);
                Ok(())
            }
            Expr::Bool(val, _) => {
                if *val {
                    self.emit(Opcode::OpTrue);
                } else {
                    self.emit(Opcode::OpFalse);
                }
                Ok(())
            }
            Expr::Nil(_) => {
                self.emit(Opcode::OpNil);
                Ok(())
            }
            Expr::Ident(name, tok) => {
                let idx_opt = self.symbol_table.get(name).copied();
                if let Some(idx) = idx_opt {
                    self.emit(Opcode::OpGetGlobal);
                    self.emit_u16(idx);
                    Ok(())
                } else {
                    Err(format!("Undefined variable '{}' at line {}", name, tok.line))
                }
            }
            Expr::Infix(left, op, right, _) => {
                self.compile_expression(left)?;
                self.compile_expression(right)?;

                match op {
                    InfixOp::Plus => self.emit(Opcode::OpAdd),
                    InfixOp::Minus => self.emit(Opcode::OpSub),
                    InfixOp::Multiply => self.emit(Opcode::OpMul),
                    InfixOp::Divide => self.emit(Opcode::OpDiv),
                    InfixOp::Modulo => self.emit(Opcode::OpMod),
                    InfixOp::Equal => self.emit(Opcode::OpEqual),
                    InfixOp::NotEqual => self.emit(Opcode::OpNotEqual),
                    InfixOp::GreaterThan => self.emit(Opcode::OpGreaterThan),
                    InfixOp::GreaterEqual => self.emit(Opcode::OpGreaterEqual),
                    InfixOp::LessThan => self.emit(Opcode::OpLessThan),
                    InfixOp::LessEqual => self.emit(Opcode::OpLessEqual),
                    _ => return Err("Operator not supported in compiler".to_string()),
                };
                Ok(())
            }
            Expr::Prefix(op, right, _) => {
                self.compile_expression(right)?;
                match op {
                    PrefixOp::Bang => self.emit(Opcode::OpBang),
                    PrefixOp::Minus => self.emit(Opcode::OpMinus),
                };
                Ok(())
            }
            _ => Err("Expression not supported in compiler yet".to_string()),
        }
    }

    fn add_constant(&mut self, obj: Object) -> u16 {
        self.constants.push(obj);
        (self.constants.len() - 1) as u16
    }

    fn emit(&mut self, op: Opcode) -> usize {
        let pos = self.instructions.len();
        self.instructions.push(op as u8);
        pos
    }

    fn emit_u16(&mut self, val: u16) {
        self.instructions.push((val >> 8) as u8);
        self.instructions.push((val & 0xFF) as u8);
    }
}
