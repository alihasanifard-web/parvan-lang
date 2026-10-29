pub mod ast;
pub mod code;
pub mod compiler;
pub mod evaluator;
pub mod lexer;
pub mod object;
pub mod parser;
pub mod persian;
pub mod rtl_input;
pub mod stdlib;
pub mod token;
pub mod vm;
pub mod typechecker;
pub mod native_emitter;
pub mod event_loop;

use crate::lexer::Lexer;
use crate::parser::Parser;

pub fn eval_parvan_code(code: &str) -> String {
    let lexer = Lexer::new(code);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();

    if !parser.errors.is_empty() {
        return format!("Syntax Errors: {:?}", parser.errors);
    }

    let env = std::rc::Rc::new(std::cell::RefCell::new(object::Environment::new()));
    match evaluator::eval_program(&program, env) {
        Ok(obj) => obj.to_string(),
        Err(e) => format!("Runtime Error: {}", e),
    }
}
