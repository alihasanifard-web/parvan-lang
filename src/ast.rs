use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum InfixOp {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrefixOp {
    Minus,
    Bang,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Ident(String, Token),
    Int(i64, Token),
    Float(f64, Token),
    Str(String, Token),
    Bool(bool, Token),
    Nil(Token),
    Prefix(PrefixOp, Box<Expr>, Token),
    Infix(Box<Expr>, InfixOp, Box<Expr>, Token),
    Call(Box<Expr>, Vec<Expr>, Token),
    Array(Vec<Expr>, Token),
    Map(Vec<(Expr, Expr)>, Token),
    Index(Box<Expr>, Box<Expr>, Token),
    Property(Box<Expr>, String, Token),
    Assign(String, Box<Expr>, Token),
    IndexAssign(Box<Expr>, Box<Expr>, Box<Expr>, Token),
    Function {
        name: Option<String>,
        params: Vec<String>,
        body: BlockStmt,
        token: Token,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockStmt {
    pub statements: Vec<Stmt>,
    pub token: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        value: Expr,
        token: Token,
    },
    Const {
        name: String,
        value: Expr,
        token: Token,
    },
    Return {
        value: Option<Expr>,
        token: Token,
    },
    Expr(Expr),
    If {
        condition: Expr,
        consequence: BlockStmt,
        alternative: Option<BlockStmt>,
        token: Token,
    },
    While {
        condition: Expr,
        body: BlockStmt,
        token: Token,
    },
    For {
        var: String,
        iterable: Expr,
        body: BlockStmt,
        token: Token,
    },
    Break(Token),
    Continue(Token),
    Print {
        args: Vec<Expr>,
        token: Token,
    },
    Import {
        path: String,
        token: Token,
    },
    TryCatch {
        try_block: BlockStmt,
        err_var: String,
        catch_block: BlockStmt,
        token: Token,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
