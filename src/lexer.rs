use crate::token::{Token, TokenKind};

pub struct Lexer {
    input: Vec<char>,
    position: usize,      // current char position
    read_position: usize, // next char position
    ch: Option<char>,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let mut lexer = Self {
            input: input.chars().collect(),
            position: 0,
            read_position: 0,
            ch: None,
            line: 1,
            column: 0,
        };
        lexer.read_char();
        lexer
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = None;
        } else {
            self.ch = Some(self.input[self.read_position]);
        }
        self.position = self.read_position;
        self.read_position += 1;
        self.column += 1;
    }

    fn peek_char(&self) -> Option<char> {
        if self.read_position >= self.input.len() {
            None
        } else {
            Some(self.input[self.read_position])
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        while let Some(ch) = self.ch {
            if ch == '\n' {
                self.line += 1;
                self.column = 0;
                self.read_char();
            } else if ch.is_whitespace() {
                self.read_char();
            } else if ch == '/' && self.peek_char() == Some('/') {
                // Line comment
                while let Some(c) = self.ch {
                    if c == '\n' {
                        self.line += 1;
                        self.column = 0;
                        self.read_char();
                        break;
                    }
                    self.read_char();
                }
            } else {
                break;
            }
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        let line = self.line;
        let col = self.column;

        let token = match self.ch {
            None => Token::new(TokenKind::Eof, "".to_string(), line, col),
            Some('+') => {
                if self.peek_char() == Some('=') {
                    self.read_char();
                    self.read_char();
                    Token::new(TokenKind::PlusAssign, "+=".to_string(), line, col)
                } else {
                    self.read_char();
                    Token::new(TokenKind::Plus, "+".to_string(), line, col)
                }
            }
            Some('-') => {
                if self.peek_char() == Some('>') {
                    self.read_char();
                    self.read_char();
                    Token::new(TokenKind::Arrow, "->".to_string(), line, col)
                } else if self.peek_char() == Some('=') {
                    self.read_char();
                    self.read_char();
                    Token::new(TokenKind::MinusAssign, "-=".to_string(), line, col)
                } else {
                    self.read_char();
                    Token::new(TokenKind::Minus, "-".to_string(), line, col)
                }
            }
            Some('*') => {
                self.read_char();
                Token::new(TokenKind::Asterisk, "*".to_string(), line, col)
            }
            Some('/') => {
                self.read_char();
                Token::new(TokenKind::Slash, "/".to_string(), line, col)
            }
            Some('%') => {
                self.read_char();
                Token::new(TokenKind::Percent, "%".to_string(), line, col)
            }
            Some('!') => {
                if self.peek_char() == Some('=') {
                    self.read_char();
                    self.read_char();
                    Token::new(TokenKind::NotEq, "!=".to_string(), line, col)
                } else {
                    self.read_char();
                    Token::new(TokenKind::Bang, "!".to_string(), line, col)
                }
            }
            Some('=') => {
                if self.peek_char() == Some('=') {
                    self.read_char();
                    self.read_char();
                    Token::new(TokenKind::Eq, "==".to_string(), line, col)
                } else {
                    self.read_char();
                    Token::new(TokenKind::Assign, "=".to_string(), line, col)
                }
            }
            Some('<') => {
                if self.peek_char() == Some('=') {
                    self.read_char();
                    self.read_char();
                    Token::new(TokenKind::Lte, "<=".to_string(), line, col)
                } else {
                    self.read_char();
                    Token::new(TokenKind::Lt, "<".to_string(), line, col)
                }
            }
            Some('>') => {
                if self.peek_char() == Some('=') {
                    self.read_char();
                    self.read_char();
                    Token::new(TokenKind::Gte, ">=".to_string(), line, col)
                } else {
                    self.read_char();
                    Token::new(TokenKind::Gt, ">".to_string(), line, col)
                }
            }
            Some('&') => {
                if self.peek_char() == Some('&') {
                    self.read_char();
                    self.read_char();
                    Token::new(TokenKind::And, "&&".to_string(), line, col)
                } else {
                    let ch = self.ch.unwrap();
                    self.read_char();
                    Token::new(TokenKind::Illegal(ch), ch.to_string(), line, col)
                }
            }
            Some('|') => {
                if self.peek_char() == Some('|') {
                    self.read_char();
                    self.read_char();
                    Token::new(TokenKind::Or, "||".to_string(), line, col)
                } else {
                    let ch = self.ch.unwrap();
                    self.read_char();
                    Token::new(TokenKind::Illegal(ch), ch.to_string(), line, col)
                }
            }
            Some(';') => {
                self.read_char();
                Token::new(TokenKind::Semicolon, ";".to_string(), line, col)
            }
            Some('.') => {
                self.read_char();
                Token::new(TokenKind::Dot, ".".to_string(), line, col)
            }
            Some(':') => {
                self.read_char();
                Token::new(TokenKind::Colon, ":".to_string(), line, col)
            }
            Some(',') => {
                self.read_char();
                Token::new(TokenKind::Comma, ",".to_string(), line, col)
            }
            Some('(') => {
                self.read_char();
                Token::new(TokenKind::LParen, "(".to_string(), line, col)
            }
            Some(')') => {
                self.read_char();
                Token::new(TokenKind::RParen, ")".to_string(), line, col)
            }
            Some('{') => {
                self.read_char();
                Token::new(TokenKind::LBrace, "{".to_string(), line, col)
            }
            Some('}') => {
                self.read_char();
                Token::new(TokenKind::RBrace, "}".to_string(), line, col)
            }
            Some('[') => {
                self.read_char();
                Token::new(TokenKind::LBracket, "[".to_string(), line, col)
            }
            Some(']') => {
                self.read_char();
                Token::new(TokenKind::RBracket, "]".to_string(), line, col)
            }
            Some('"') => self.read_string(line, col),
            Some(ch) => {
                if is_letter(ch) {
                    self.read_identifier_or_keyword(line, col)
                } else if ch.is_ascii_digit() {
                    self.read_number(line, col)
                } else {
                    self.read_char();
                    Token::new(TokenKind::Illegal(ch), ch.to_string(), line, col)
                }
            }
        };

        token
    }

    fn read_identifier_or_keyword(&mut self, line: usize, col: usize) -> Token {
        let mut ident = String::new();
        while let Some(ch) = self.ch {
            if is_letter(ch) || ch.is_ascii_digit() || ch == '_' {
                ident.push(ch);
                self.read_char();
            } else {
                break;
            }
        }

        let kind = match ident.as_str() {
            // English Keywords
            "let" => TokenKind::Let,
            "const" => TokenKind::Const,
            "fn" | "func" => TokenKind::Fn,
            "return" => TokenKind::Return,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "print" => TokenKind::Print,
            "import" => TokenKind::Import,
            "try" => TokenKind::Try,
            "catch" => TokenKind::Catch,
            "true" => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),
            "nil" | "null" => TokenKind::Nil,

            // Persian Keywords! (دوزبانه بودن پاروان)
            "بگذار" | "متغیر" => TokenKind::Let,
            "ثابت" => TokenKind::Const,
            "تابع" => TokenKind::Fn,
            "بازگردان" => TokenKind::Return,
            "اگر" => TokenKind::If,
            "وگرنه" => TokenKind::Else,
            "تا_زمانی_که" => TokenKind::While,
            "برای" => TokenKind::For,
            "در" => TokenKind::In,
            "شکست" => TokenKind::Break,
            "ادامه" => TokenKind::Continue,
            "چاپ" => TokenKind::Print,
            "وارد_کن" | "پیوست" => TokenKind::Import,
            "تلاش" => TokenKind::Try,
            "خطا" | "گرفتن_خطا" => TokenKind::Catch,
            "درست" => TokenKind::Bool(true),
            "غلط" => TokenKind::Bool(false),
            "هیچ" => TokenKind::Nil,

            _ => TokenKind::Ident(ident.clone()),
        };

        Token::new(kind, ident, line, col)
    }

    fn read_number(&mut self, line: usize, col: usize) -> Token {
        let mut num_str = String::new();
        let mut is_float = false;

        while let Some(ch) = self.ch {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.read_char();
            } else if ch == '.' && !is_float && self.peek_char().map_or(false, |c| c.is_ascii_digit()) {
                is_float = true;
                num_str.push(ch);
                self.read_char();
            } else {
                break;
            }
        }

        if is_float {
            let val = num_str.parse::<f64>().unwrap_or(0.0);
            Token::new(TokenKind::Float(val), num_str, line, col)
        } else {
            let val = num_str.parse::<i64>().unwrap_or(0);
            Token::new(TokenKind::Int(val), num_str, line, col)
        }
    }

    fn read_string(&mut self, line: usize, col: usize) -> Token {
        self.read_char(); // skip opening quote
        let mut s = String::new();

        while let Some(ch) = self.ch {
            if ch == '"' {
                self.read_char(); // skip closing quote
                break;
            } else if ch == '\\' {
                self.read_char();
                match self.ch {
                    Some('n') => s.push('\n'),
                    Some('t') => s.push('\t'),
                    Some('r') => s.push('\r'),
                    Some('"') => s.push('"'),
                    Some('\\') => s.push('\\'),
                    Some(c) => s.push(c),
                    None => break,
                }
                self.read_char();
            } else {
                s.push(ch);
                self.read_char();
            }
        }

        Token::new(TokenKind::Str(s.clone()), format!("\"{}\"", s), line, col)
    }
}

fn is_letter(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_'
}
