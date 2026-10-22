use crate::object::Object;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Opcode {
    OpConstant = 1,
    OpAdd,
    OpSub,
    OpMul,
    OpDiv,
    OpMod,
    OpEqual,
    OpNotEqual,
    OpGreaterThan,
    OpGreaterEqual,
    OpLessThan,
    OpLessEqual,
    OpTrue,
    OpFalse,
    OpNil,
    OpMinus,
    OpBang,
    OpPop,
    OpSetGlobal,
    OpGetGlobal,
    OpSetLocal,
    OpGetLocal,
    OpJump,
    OpJumpIfFalse,
    OpReturn,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bytecode {
    pub instructions: Vec<u8>,
    pub constants: Vec<Object>,
}

impl Bytecode {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
        }
    }
}
