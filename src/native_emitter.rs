// ==============================================================================
// ⚡ PARVAN NATIVE C99 EMITTER & MACHINE CODE BACKEND (مولد کد ماشین و ترنسپایلر C99)
// معماران: علی حسنی‌فرد، احسان فضلی‌نژاد، امیرمهدی شهبازی
// ==============================================================================

use crate::ast::*;
use std::process::Command;
use std::fs;

pub struct NativeEmitter {
    output: String,
    indent_level: usize,
}

impl NativeEmitter {
    pub fn new() -> Self {
        let mut em = Self {
            output: String::new(),
            indent_level: 0,
        };
        em.emit_prelude();
        em
    }

    fn emit_prelude(&mut self) {
        self.output.push_str("/* Generated automatically by Parvan Native AOT Engine v4.0 */\n");
        self.output.push_str("#include <stdio.h>\n");
        self.output.push_str("#include <stdlib.h>\n");
        self.output.push_str("#include <string.h>\n");
        self.output.push_str("#include <stdint.h>\n");
        self.output.push_str("#include <stdbool.h>\n");
        self.output.push_str("#include <math.h>\n\n");
        self.output.push_str("typedef int64_t prv_int;\n");
        self.output.push_str("typedef double prv_float;\n");
        self.output.push_str("typedef const char* prv_str;\n\n");
        self.output.push_str("void prv_print_int(prv_int x) { printf(\"%lld\\n\", (long long)x); }\n");
        self.output.push_str("void prv_print_str(prv_str s) { printf(\"%s\\n\", s); }\n");
        self.output.push_str("void prv_print_float(prv_float f) { printf(\"%g\\n\", f); }\n\n");
    }

    fn indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }

    pub fn emit_program(&mut self, program: &Program) -> String {
        self.output.push_str("int main(int argc, char** argv) {\n");
        self.indent_level += 1;

        for stmt in &program.statements {
            self.emit_statement(stmt);
        }

        self.indent();
        self.output.push_str("return 0;\n");
        self.indent_level -= 1;
        self.output.push_str("}\n");
        self.output.clone()
    }

    fn emit_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, value, .. } => {
                self.indent();
                let clean_name = self.sanitize_ident(name);
                let val_expr = self.emit_expression(value);
                self.output.push_str(&format!("prv_int {} = {};\n", clean_name, val_expr));
            }
            Stmt::Const { name, value, .. } => {
                self.indent();
                let clean_name = self.sanitize_ident(name);
                let val_expr = self.emit_expression(value);
                self.output.push_str(&format!("const prv_int {} = {};\n", clean_name, val_expr));
            }
            Stmt::Expr(expr) => {
                self.indent();
                let e = self.emit_expression(expr);
                self.output.push_str(&format!("{};\n", e));
            }
            Stmt::Print { args, .. } => {
                self.indent();
                if let Some(arg) = args.first() {
                    let arg_s = self.emit_expression(arg);
                    self.output.push_str(&format!("prv_print_int({});\n", arg_s));
                } else {
                    self.output.push_str("prv_print_str(\"\");\n");
                }
            }
            Stmt::While { condition, body, .. } => {
                self.indent();
                let c = self.emit_expression(condition);
                self.output.push_str(&format!("while ({}) {{\n", c));
                self.indent_level += 1;
                for s in &body.statements {
                    self.emit_statement(s);
                }
                self.indent_level -= 1;
                self.indent();
                self.output.push_str("}\n");
            }
            Stmt::Return { value, .. } => {
                self.indent();
                if let Some(e) = value {
                    let ret = self.emit_expression(e);
                    self.output.push_str(&format!("return {};\n", ret));
                } else {
                    self.output.push_str("return 0;\n");
                }
            }
            _ => {}
        }
    }

    fn emit_expression(&self, expr: &Expr) -> String {
        match expr {
            Expr::Int(i, ..) => format!("{}", i),
            Expr::Float(f, ..) => format!("{}", f),
            Expr::Str(s, ..) => format!("\"{}\"", s),
            Expr::Bool(b, ..) => format!("{}", if *b { 1 } else { 0 }),
            Expr::Ident(id, ..) => self.sanitize_ident(id),
            Expr::Infix(left, op, right, ..) => {
                let l = self.emit_expression(left);
                let r = self.emit_expression(right);
                let op_str = match op {
                    InfixOp::Plus => "+",
                    InfixOp::Minus => "-",
                    InfixOp::Multiply => "*",
                    InfixOp::Divide => "/",
                    InfixOp::Modulo => "%",
                    InfixOp::Equal => "==",
                    InfixOp::NotEqual => "!=",
                    InfixOp::GreaterThan => ">",
                    InfixOp::LessThan => "<",
                    InfixOp::GreaterEqual => ">=",
                    InfixOp::LessEqual => "<=",
                    InfixOp::And => "&&",
                    InfixOp::Or => "||",
                };
                format!("({} {} {})", l, op_str, r)
            }
            Expr::Call(func, args, ..) => {
                if let Expr::Ident(name, ..) = func.as_ref() {
                    if name == "print" || name == "println" || name == "چاپ" || name == "چاپ_سطر" {
                        if let Some(arg) = args.first() {
                            let arg_s = self.emit_expression(arg);
                            return format!("prv_print_int({})", arg_s);
                        }
                    }
                }
                "0".to_string()
            }
            Expr::Assign(name, val, ..) => {
                let s_name = self.sanitize_ident(name);
                let v = self.emit_expression(val);
                format!("{} = {}", s_name, v)
            }
            _ => "0".to_string(),
        }
    }

    fn sanitize_ident(&self, name: &str) -> String {
        let mut res = String::new();
        for ch in name.chars() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                res.push(ch);
            } else {
                res.push_str(&format!("_u{:x}", ch as u32));
            }
        }
        res
    }
}

pub fn compile_c_to_binary(c_code: &str, output_binary_path: &str) -> Result<(), String> {
    let tmp_c = format!("{}.c", output_binary_path);
    if let Err(e) = fs::write(&tmp_c, c_code) {
        return Err(format!("خطا در ذخیره فایل میانی C: {}", e));
    }

    let gcc_cmd = "gcc";
    let status = Command::new(gcc_cmd)
        .arg("-O3")
        .arg(&tmp_c)
        .arg("-o")
        .arg(output_binary_path)
        .arg("-lm")
        .status();

    let _ = fs::remove_file(&tmp_c);

    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("کامپایلر gcc با کد خروج {} مواجه شد.", s)),
        Err(e) => Err(format!("عدم امکان اجرای کامپایلر gcc: {}", e)),
    }
}
