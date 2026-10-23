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
            Stmt::If { condition, consequence, alternative, .. } => {
                self.compile_expression(condition)?;

                // Emit jump-if-false with placeholder address
                let jump_if_false_pos = self.emit(Opcode::OpJumpIfFalse);
                self.emit_u16(9999);

                // Compile consequence
                for s in &consequence.statements {
                    self.compile_statement(s)?;
                }

                // Emit jump over alternative
                let jump_pos = self.emit(Opcode::OpJump);
                self.emit_u16(9999);

                // Patch jump_if_false target
                let alt_start = self.instructions.len() as u16;
                self.patch_u16(jump_if_false_pos + 1, alt_start);

                if let Some(alt) = alternative {
                    for s in &alt.statements {
                        self.compile_statement(s)?;
                    }
                }

                // Patch jump target
                let after_alt = self.instructions.len() as u16;
                self.patch_u16(jump_pos + 1, after_alt);

                Ok(())
            }
            Stmt::While { condition, body, .. } => {
                let loop_start = self.instructions.len() as u16;

                // Evaluate condition
                self.compile_expression(condition)?;

                // If condition is false, jump out of loop
                let jump_out_pos = self.emit(Opcode::OpJumpIfFalse);
                self.emit_u16(9999);

                // Compile loop body
                for s in &body.statements {
                    self.compile_statement(s)?;
                }

                // Loop back to condition
                self.emit(Opcode::OpJump);
                self.emit_u16(loop_start);

                // Patch exit jump address
                let loop_end = self.instructions.len() as u16;
                self.patch_u16(jump_out_pos + 1, loop_end);

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
            Expr::Function { params, body, .. } => {
                let func_obj = Object::Function {
                    params: params.clone(),
                    body: body.clone(),
                    env: std::rc::Rc::new(std::cell::RefCell::new(crate::object::Environment::new())),
                };
                let const_idx = self.add_constant(func_obj);
                self.emit(Opcode::OpConstant);
                self.emit_u16(const_idx);
                Ok(())
            }
            Expr::Ident(name, tok) => {
                let idx_opt = self.symbol_table.get(name).copied();
                if let Some(idx) = idx_opt {
                    self.emit(Opcode::OpGetGlobal);
                    self.emit_u16(idx);
                    Ok(())
                } else if let Some(builtin) = crate::stdlib::get_builtin(name) {
                    let const_idx = self.add_constant(builtin);
                    self.emit(Opcode::OpConstant);
                    self.emit_u16(const_idx);
                    Ok(())
                } else {
                    Err(format!("Undefined variable '{}' at line {}", name, tok.line))
                }
            }
            Expr::Call(func_expr, args, _) => {
                for arg in args {
                    self.compile_expression(arg)?;
                }
                self.compile_expression(func_expr)?;
                self.emit(Opcode::OpCall);
                self.instructions.push(args.len() as u8);
                Ok(())
            }
            Expr::Infix(left, op, right, tok) => {
                // Constant Folding Optimization!
                if let (Expr::Int(a, _), Expr::Int(b, _)) = (&**left, &**right) {
                    match op {
                        InfixOp::Plus => {
                            let folded = Expr::Int(a + b, tok.clone());
                            return self.compile_expression(&folded);
                        }
                        InfixOp::Minus => {
                            let folded = Expr::Int(a - b, tok.clone());
                            return self.compile_expression(&folded);
                        }
                        InfixOp::Multiply => {
                            let folded = Expr::Int(a * b, tok.clone());
                            return self.compile_expression(&folded);
                        }
                        InfixOp::Divide if *b != 0 => {
                            let folded = Expr::Int(a / b, tok.clone());
                            return self.compile_expression(&folded);
                        }
                        InfixOp::Modulo if *b != 0 => {
                            let folded = Expr::Int(a % b, tok.clone());
                            return self.compile_expression(&folded);
                        }
                        _ => {}
                    }
                }

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
            Expr::Assign(name, val_expr, _tok) => {
                self.compile_expression(val_expr)?;
                let next_idx = self.symbol_table.len() as u16;
                let symbol_idx = *self.symbol_table.entry(name.clone()).or_insert(next_idx);
                self.emit(Opcode::OpSetGlobal);
                self.emit_u16(symbol_idx);
                // Also get global back on stack so expression returns value
                self.emit(Opcode::OpGetGlobal);
                self.emit_u16(symbol_idx);
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

    fn patch_u16(&mut self, pos: usize, val: u16) {
        self.instructions[pos] = (val >> 8) as u8;
        self.instructions[pos + 1] = (val & 0xFF) as u8;
    }
}
