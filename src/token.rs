#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    // End of file / Illegal
    Eof,
    Illegal(char),

    // Identifiers & Literals
    Ident(String),
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Nil,

    // Operators
    Plus,       // +
    PlusPlus,   // ++
    Minus,      // -
    MinusMinus, // --
    Asterisk,   // *
    Slash,      // /
    Percent,    // %
    Bang,       // !
    Assign,     // =
    PlusAssign, // +=
    MinusAssign,// -=
    Pipe,       // |> (Pipeline Operator)

    // Comparisons
    Eq,     // ==
    NotEq,  // !=
    Lt,     // <
    Lte,    // <=
    Gt,     // >
    Gte,    // >=
    And,    // &&
    Or,     // ||

    // Delimiters
    Comma,      // ,
    Colon,      // :
    Semicolon,  // ;
    Dot,        // .
    LParen,     // (
    RParen,     // )
    LBrace,     // {
    RBrace,     // }
    LBracket,   // [
    RBracket,   // ]
    Arrow,      // ->
    FatArrow,   // =>

    // Keywords (English + Persian Aliases!)
    Let,        // let / بگذار / متغیر
    Const,      // const / ثابت
    Fn,         // fn / func / تابع
    Return,     // return / بازگردان
    If,         // if / اگر
    Else,       // else / وگرنه
    While,      // while / تا_زمانی_که
    For,        // for / برای
    In,         // in / در
    Break,      // break / شکست
    Continue,   // continue / ادامه
    Print,      // print / چاپ
    Import,     // import / وارد_کن
    As,         // as / به_عنوان
    Try,        // try / تلاش
    Catch,      // catch / خطا
    Switch,     // switch / انتخاب
    Case,       // case / حالت
    Async,      // async / ناهمگام
    Await,      // await / صبر / منتظر
    Default,    // default / پیش‌فرض
    Enum,       // enum / شمارش
    Match,      // match / تطبیق
    Struct,     // struct / ساختار
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub literal: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, literal: String, line: usize, column: usize) -> Self {
        Self { kind, literal, line, column }
    }
}
