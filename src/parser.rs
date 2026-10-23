use crate::ast::*;
use crate::lexer::Lexer;
use crate::token::{Token, TokenKind};

#[derive(PartialEq, PartialOrd)]
enum Precedence {
    Lowest = 1,
    Pipe = 2,       // |>
    Or = 3,         // ||
    And = 4,        // &&
    Equals = 5,     // ==, !=
    LessGreater = 6,// >, <, >=, <=
    Sum = 7,        // +, -
    Product = 8,    // *, /, %
    Prefix = 9,     // -X, !X
    Call = 10,      // my_func(X)
    Index = 11,     // array[index], obj.property
}

impl Precedence {
    fn from_token(token: &TokenKind) -> Precedence {
        match token {
            TokenKind::Pipe => Precedence::Pipe,
            TokenKind::Or => Precedence::Or,
            TokenKind::And => Precedence::And,
            TokenKind::Eq | TokenKind::NotEq => Precedence::Equals,
            TokenKind::Lt | TokenKind::Lte | TokenKind::Gt | TokenKind::Gte => Precedence::LessGreater,
            TokenKind::Plus | TokenKind::Minus => Precedence::Sum,
            TokenKind::Asterisk | TokenKind::Slash | TokenKind::Percent => Precedence::Product,
            TokenKind::LParen => Precedence::Call,
            TokenKind::LBracket | TokenKind::Dot => Precedence::Index,
            _ => Precedence::Lowest,
        }
    }
}

pub struct Parser {
    lexer: Lexer,
    cur_token: Token,
    peek_token: Token,
    pub errors: Vec<String>,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let cur = lexer.next_token();
        let peek = lexer.next_token();
        Self {
            lexer,
            cur_token: cur,
            peek_token: peek,
            errors: Vec::new(),
        }
    }

    fn next_token(&mut self) {
        self.cur_token = std::mem::replace(&mut self.peek_token, self.lexer.next_token());
    }

    fn cur_token_is(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.cur_token.kind) == std::mem::discriminant(kind)
    }

    fn peek_token_is(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek_token.kind) == std::mem::discriminant(kind)
    }

    fn expect_peek(&mut self, kind: TokenKind) -> bool {
        if self.peek_token_is(&kind) {
            self.next_token();
            true
        } else {
            self.peek_error(&kind);
            false
        }
    }

    fn peek_error(&mut self, expected: &TokenKind) {
        let msg = format!(
            "خطای نحو [سطر {}, ستون {}]: انتظار می‌رفت توکن {:?} باشد، اما {:?} ('{}') دریافت شد.",
            self.peek_token.line, self.peek_token.column, expected, self.peek_token.kind, self.peek_token.literal
        );
        self.errors.push(msg);
    }

    pub fn parse_program(&mut self) -> Program {
        let mut program = Program {
            statements: Vec::new(),
        };

        while self.cur_token.kind != TokenKind::Eof {
            if let Some(stmt) = self.parse_statement() {
                program.statements.push(stmt);
            }
            self.next_token();
        }

        program
    }

    fn parse_statement(&mut self) -> Option<Stmt> {
        match &self.cur_token.kind {
            TokenKind::Let => self.parse_let_statement(false),
            TokenKind::Const => self.parse_let_statement(true),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::For => self.parse_for_statement(),
            TokenKind::Break => {
                let tok = self.cur_token.clone();
                if self.peek_token_is(&TokenKind::Semicolon) {
                    self.next_token();
                }
                Some(Stmt::Break(tok))
            }
            TokenKind::Continue => {
                let tok = self.cur_token.clone();
                if self.peek_token_is(&TokenKind::Semicolon) {
                    self.next_token();
                }
                Some(Stmt::Continue(tok))
            }
            TokenKind::Print => self.parse_print_statement(),
            TokenKind::Import => self.parse_import_statement(),
            TokenKind::Try => self.parse_try_catch_statement(),
            TokenKind::Switch => self.parse_switch_statement(),
            TokenKind::Struct => self.parse_struct_statement(),
            TokenKind::Enum => self.parse_enum_statement(),
            TokenKind::Match => self.parse_match_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_struct_statement(&mut self) -> Option<Stmt> {
        let tok = self.cur_token.clone();
        self.next_token(); // on struct name

        let name = if let TokenKind::Ident(ref n) = self.cur_token.kind {
            n.clone()
        } else {
            self.errors.push(format!("انتظار نام ساختار بعد از کلیدواژه struct در سطر {}", self.cur_token.line));
            return None;
        };

        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }

        self.next_token(); // on first item inside '{'

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        while !self.cur_token_is(&TokenKind::RBrace) && !self.cur_token_is(&TokenKind::Eof) {
            if self.cur_token_is(&TokenKind::Fn) {
                // Parse method: fn method_name(params) { body }
                let fn_tok = self.cur_token.clone();
                self.next_token(); // on method name
                let m_name = if let TokenKind::Ident(ref mn) = self.cur_token.kind {
                    mn.clone()
                } else {
                    self.errors.push(format!("انتظار نام متد در سطر {}", self.cur_token.line));
                    return None;
                };

                if !self.expect_peek(TokenKind::LParen) {
                    return None;
                }

                let params = self.parse_function_params()?;

                if !self.expect_peek(TokenKind::LBrace) {
                    return None;
                }

                let body = self.parse_block_statement();
                methods.push(crate::ast::StructMethod {
                    name: m_name,
                    params,
                    body,
                    token: fn_tok,
                });
            } else if let TokenKind::Ident(ref field_name) = self.cur_token.kind.clone() {
                fields.push(field_name.clone());
                if self.peek_token_is(&TokenKind::Comma) {
                    self.next_token();
                } else if self.peek_token_is(&TokenKind::Semicolon) {
                    self.next_token();
                }
            } else {
                self.errors.push(format!("عنصر نامعتبر درون تعریف ساختار در سطر {}", self.cur_token.line));
                return None;
            }

            self.next_token();
        }

        Some(Stmt::Struct {
            name,
            fields,
            methods,
            token: tok,
        })
    }

    fn parse_enum_statement(&mut self) -> Option<Stmt> {
        let tok = self.cur_token.clone();
        self.next_token(); // on enum name

        let name = if let TokenKind::Ident(ref n) = self.cur_token.kind {
            n.clone()
        } else {
            self.errors.push(format!("انتظار نام enum در سطر {}", self.cur_token.line));
            return None;
        };

        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }

        self.next_token(); // inside '{'
        let mut variants = Vec::new();

        while !self.cur_token_is(&TokenKind::RBrace) && !self.cur_token_is(&TokenKind::Eof) {
            if let TokenKind::Ident(ref v_name) = self.cur_token.kind.clone() {
                variants.push(v_name.clone());
                if self.peek_token_is(&TokenKind::Comma) {
                    self.next_token();
                }
            }
            self.next_token();
        }

        Some(Stmt::Enum {
            name,
            variants,
            token: tok,
        })
    }

    fn parse_match_statement(&mut self) -> Option<Stmt> {
        let tok = self.cur_token.clone();
        self.next_token(); // on condition

        let condition = self.parse_expression(Precedence::Lowest)?;

        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }

        self.next_token(); // inside '{'
        let mut arms = Vec::new();
        let mut default_branch = None;

        while !self.cur_token_is(&TokenKind::RBrace) && !self.cur_token_is(&TokenKind::Eof) {
            if self.cur_token_is(&TokenKind::Default) {
                self.next_token(); // on '=>' or '{'
                if self.cur_token_is(&TokenKind::FatArrow) || self.cur_token_is(&TokenKind::Arrow) {
                    self.next_token();
                }
                if self.cur_token_is(&TokenKind::LBrace) {
                    default_branch = Some(self.parse_block_statement());
                }
            } else {
                let pat_expr = self.parse_expression(Precedence::Lowest)?;
                if !self.expect_peek(TokenKind::FatArrow) && !self.expect_peek(TokenKind::Arrow) {
                    return None;
                }
                if !self.expect_peek(TokenKind::LBrace) {
                    return None;
                }
                let body = self.parse_block_statement();
                arms.push((pat_expr, body));
            }
            self.next_token();
        }

        Some(Stmt::Match {
            condition,
            arms,
            default: default_branch,
            token: tok,
        })
    }

    fn parse_switch_statement(&mut self) -> Option<Stmt> {
        let tok = self.cur_token.clone();
        self.next_token(); // on condition

        let condition = self.parse_expression(Precedence::Lowest)?;
        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }

        let mut cases = Vec::new();
        let mut default_branch = None;

        self.next_token(); // now on first token inside '{' (e.g. Case or Default or '}')

        while !self.cur_token_is(&TokenKind::RBrace) && !self.cur_token_is(&TokenKind::Eof) {
            if self.cur_token_is(&TokenKind::Case) {
                self.next_token(); // on case value
                let case_val = self.parse_expression(Precedence::Lowest)?;
                if !self.expect_peek(TokenKind::Colon) {
                    return None;
                }
                if !self.expect_peek(TokenKind::LBrace) {
                    return None;
                }
                let body = self.parse_block_statement();
                cases.push((case_val, body));
                self.next_token(); // advance past '}' to next case/default
            } else if self.cur_token_is(&TokenKind::Default) {
                if !self.expect_peek(TokenKind::Colon) {
                    return None;
                }
                if !self.expect_peek(TokenKind::LBrace) {
                    return None;
                }
                let body = self.parse_block_statement();
                default_branch = Some(body);
                self.next_token(); // advance past '}'
            } else {
                self.next_token();
            }
        }

        Some(Stmt::Switch {
            condition,
            cases,
            default: default_branch,
            token: tok,
        })
    }

    fn parse_import_statement(&mut self) -> Option<Stmt> {
        let tok = self.cur_token.clone();
        self.next_token();
        if let TokenKind::Str(ref path) = self.cur_token.kind.clone() {
            let p = path.clone();
            let mut alias = None;
            if self.peek_token_is(&TokenKind::As) {
                self.next_token(); // consume 'as'
                if let TokenKind::Ident(ref id) = self.peek_token.kind.clone() {
                    self.next_token(); // consume alias
                    alias = Some(id.clone());
                }
            }
            if self.peek_token_is(&TokenKind::Semicolon) {
                self.next_token();
            }
            Some(Stmt::Import { path: p, alias, token: tok })
        } else {
            self.errors.push(format!("دستور import نیازمند مسیر فایلی به شکل رشته است در سطر {}", tok.line));
            None
        }
    }

    fn parse_try_catch_statement(&mut self) -> Option<Stmt> {
        let tok = self.cur_token.clone();
        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }
        let try_block = self.parse_block_statement();

        if !self.expect_peek(TokenKind::Catch) {
            return None;
        }

        let mut err_var = "err".to_string();
        if self.peek_token_is(&TokenKind::LParen) {
            self.next_token(); // on '('
            if let TokenKind::Ident(ref name) = self.peek_token.kind.clone() {
                self.next_token();
                err_var = name.clone();
            }
            if !self.expect_peek(TokenKind::RParen) {
                return None;
            }
        }

        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }
        let catch_block = self.parse_block_statement();

        Some(Stmt::TryCatch {
            try_block,
            err_var,
            catch_block,
            token: tok,
        })
    }

    fn parse_let_statement(&mut self, is_const: bool) -> Option<Stmt> {
        let let_token = self.cur_token.clone();

        if let TokenKind::Ident(ref name) = self.peek_token.kind.clone() {
            self.next_token();
            let var_name = name.clone();

            // Optional type annotation: let x: int = 10;
            if self.peek_token_is(&TokenKind::Colon) {
                self.next_token(); // on ':'
                self.next_token(); // on type name
            }

            if !self.expect_peek(TokenKind::Assign) {
                return None;
            }

            self.next_token();
            let val = self.parse_expression(Precedence::Lowest)?;

            if self.peek_token_is(&TokenKind::Semicolon) {
                self.next_token();
            }

            if is_const {
                Some(Stmt::Const {
                    name: var_name,
                    value: val,
                    token: let_token,
                })
            } else {
                Some(Stmt::Let {
                    name: var_name,
                    value: val,
                    token: let_token,
                })
            }
        } else {
            self.peek_error(&TokenKind::Ident("".to_string()));
            None
        }
    }

    fn parse_return_statement(&mut self) -> Option<Stmt> {
        let tok = self.cur_token.clone();
        self.next_token();

        if self.cur_token_is(&TokenKind::Semicolon) {
            return Some(Stmt::Return {
                value: None,
                token: tok,
            });
        }

        let expr = self.parse_expression(Precedence::Lowest);
        if self.peek_token_is(&TokenKind::Semicolon) {
            self.next_token();
        }

        Some(Stmt::Return {
            value: expr,
            token: tok,
        })
    }

    fn parse_print_statement(&mut self) -> Option<Stmt> {
        let tok = self.cur_token.clone();
        if !self.expect_peek(TokenKind::LParen) {
            return None;
        }

        let args = self.parse_expression_list(TokenKind::RParen)?;
        if self.peek_token_is(&TokenKind::Semicolon) {
            self.next_token();
        }

        Some(Stmt::Print { args, token: tok })
    }

    fn parse_if_statement(&mut self) -> Option<Stmt> {
        let tok = self.cur_token.clone();
        self.next_token();

        let condition = self.parse_expression(Precedence::Lowest)?;

        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }

        let consequence = self.parse_block_statement();
        let mut alternative = None;

        if self.peek_token_is(&TokenKind::Else) {
            self.next_token();

            if self.peek_token_is(&TokenKind::If) {
                self.next_token();
                let nested_if = self.parse_if_statement()?;
                alternative = Some(BlockStmt {
                    token: tok.clone(),
                    statements: vec![nested_if],
                });
            } else if self.expect_peek(TokenKind::LBrace) {
                alternative = Some(self.parse_block_statement());
            }
        }

        Some(Stmt::If {
            condition,
            consequence,
            alternative,
            token: tok,
        })
    }

    fn parse_while_statement(&mut self) -> Option<Stmt> {
        let tok = self.cur_token.clone();
        self.next_token();

        let condition = self.parse_expression(Precedence::Lowest)?;
        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }

        let body = self.parse_block_statement();
        Some(Stmt::While {
            condition,
            body,
            token: tok,
        })
    }

    fn parse_for_statement(&mut self) -> Option<Stmt> {
        let tok = self.cur_token.clone();
        let var_name: String;
        let mut val_var = None;

        if self.peek_token_is(&TokenKind::LBracket) {
            // for [k, v] in map
            self.next_token(); // on '['
            self.next_token(); // on k
            if let TokenKind::Ident(ref k) = self.cur_token.kind.clone() {
                var_name = k.clone();
            } else {
                return None;
            }
            if !self.expect_peek(TokenKind::Comma) {
                return None;
            }
            self.next_token(); // on v
            if let TokenKind::Ident(ref v) = self.cur_token.kind.clone() {
                val_var = Some(v.clone());
            } else {
                return None;
            }
            if !self.expect_peek(TokenKind::RBracket) {
                return None;
            }
        } else if let TokenKind::Ident(ref name) = self.peek_token.kind.clone() {
            self.next_token();
            var_name = name.clone();
            // check if followed by comma: for k, v in map
            if self.peek_token_is(&TokenKind::Comma) {
                self.next_token(); // on ','
                self.next_token(); // on v
                if let TokenKind::Ident(ref v) = self.cur_token.kind.clone() {
                    val_var = Some(v.clone());
                } else {
                    return None;
                }
            }
        } else {
            return None;
        }

        if !self.expect_peek(TokenKind::In) {
            return None;
        }

        self.next_token();
        let iterable = self.parse_expression(Precedence::Lowest)?;

        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }

        let body = self.parse_block_statement();
        Some(Stmt::For {
            var: var_name,
            val_var,
            iterable,
            body,
            token: tok,
        })
    }

    fn parse_block_statement(&mut self) -> BlockStmt {
        let tok = self.cur_token.clone();
        let mut statements = Vec::new();

        self.next_token();

        while !self.cur_token_is(&TokenKind::RBrace) && !self.cur_token_is(&TokenKind::Eof) {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }

        BlockStmt {
            token: tok,
            statements,
        }
    }

    fn parse_expression_statement(&mut self) -> Option<Stmt> {
        // Check for assignment: ident = expr;
        if let TokenKind::Ident(ref name) = self.cur_token.kind.clone() {
            if self.peek_token_is(&TokenKind::Assign) {
                let id_tok = self.cur_token.clone();
                let var_name = name.clone();
                self.next_token(); // on '='
                self.next_token(); // on start of expr
                let val = self.parse_expression(Precedence::Lowest)?;
                if self.peek_token_is(&TokenKind::Semicolon) {
                    self.next_token();
                }
                return Some(Stmt::Expr(Expr::Assign(var_name, Box::new(val), id_tok)));
            } else if self.peek_token_is(&TokenKind::PlusPlus) {
                let id_tok = self.cur_token.clone();
                let var_name = name.clone();
                self.next_token(); // on '++'
                if self.peek_token_is(&TokenKind::Semicolon) {
                    self.next_token();
                }
                let add_expr = Expr::Infix(
                    Box::new(Expr::Ident(var_name.clone(), id_tok.clone())),
                    InfixOp::Plus,
                    Box::new(Expr::Int(1, id_tok.clone())),
                    id_tok.clone(),
                );
                return Some(Stmt::Expr(Expr::Assign(var_name, Box::new(add_expr), id_tok)));
            } else if self.peek_token_is(&TokenKind::MinusMinus) {
                let id_tok = self.cur_token.clone();
                let var_name = name.clone();
                self.next_token(); // on '--'
                if self.peek_token_is(&TokenKind::Semicolon) {
                    self.next_token();
                }
                let sub_expr = Expr::Infix(
                    Box::new(Expr::Ident(var_name.clone(), id_tok.clone())),
                    InfixOp::Minus,
                    Box::new(Expr::Int(1, id_tok.clone())),
                    id_tok.clone(),
                );
                return Some(Stmt::Expr(Expr::Assign(var_name, Box::new(sub_expr), id_tok)));
            } else if self.peek_token_is(&TokenKind::PlusAssign) {
                let id_tok = self.cur_token.clone();
                let var_name = name.clone();
                self.next_token();
                self.next_token();
                let right = self.parse_expression(Precedence::Lowest)?;
                if self.peek_token_is(&TokenKind::Semicolon) {
                    self.next_token();
                }
                let add_expr = Expr::Infix(
                    Box::new(Expr::Ident(var_name.clone(), id_tok.clone())),
                    InfixOp::Plus,
                    Box::new(right),
                    id_tok.clone(),
                );
                return Some(Stmt::Expr(Expr::Assign(var_name, Box::new(add_expr), id_tok)));
            } else if self.peek_token_is(&TokenKind::MinusAssign) {
                let id_tok = self.cur_token.clone();
                let var_name = name.clone();
                self.next_token();
                self.next_token();
                let right = self.parse_expression(Precedence::Lowest)?;
                if self.peek_token_is(&TokenKind::Semicolon) {
                    self.next_token();
                }
                let sub_expr = Expr::Infix(
                    Box::new(Expr::Ident(var_name.clone(), id_tok.clone())),
                    InfixOp::Minus,
                    Box::new(right),
                    id_tok.clone(),
                );
                return Some(Stmt::Expr(Expr::Assign(var_name, Box::new(sub_expr), id_tok)));
            }
        }

        let expr = self.parse_expression(Precedence::Lowest)?;

        // Check for index assignment: arr[idx] = val;
        if let Expr::Index(left, idx, tok) = &expr {
            if self.peek_token_is(&TokenKind::Assign) {
                self.next_token(); // on '='
                self.next_token(); // on start of right expr
                let val = self.parse_expression(Precedence::Lowest)?;
                if self.peek_token_is(&TokenKind::Semicolon) {
                    self.next_token();
                }
                return Some(Stmt::Expr(Expr::IndexAssign(
                    left.clone(),
                    idx.clone(),
                    Box::new(val),
                    tok.clone(),
                )));
            }
        }

        // Check for property assignment: obj.prop = val;
        if let Expr::Property(left, prop, tok) = &expr {
            if self.peek_token_is(&TokenKind::Assign) {
                self.next_token(); // on '='
                self.next_token(); // on start of right expr
                let val = self.parse_expression(Precedence::Lowest)?;
                if self.peek_token_is(&TokenKind::Semicolon) {
                    self.next_token();
                }
                return Some(Stmt::Expr(Expr::PropertyAssign(
                    left.clone(),
                    prop.clone(),
                    Box::new(val),
                    tok.clone(),
                )));
            }
        }
        if self.peek_token_is(&TokenKind::Semicolon) {
            self.next_token();
        }
        Some(Stmt::Expr(expr))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expr> {
        let mut left = self.parse_prefix()?;

        while !self.peek_token_is(&TokenKind::Semicolon) && precedence < Precedence::from_token(&self.peek_token.kind) {
            match self.peek_token.kind {
                TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Asterisk
                | TokenKind::Slash
                | TokenKind::Percent
                | TokenKind::Eq
                | TokenKind::NotEq
                | TokenKind::Lt
                | TokenKind::Lte
                | TokenKind::Gt
                | TokenKind::Gte
                | TokenKind::And
                | TokenKind::Or => {
                    self.next_token();
                    left = self.parse_infix(left)?;
                }
                TokenKind::Pipe => {
                    self.next_token(); // on |>
                    let pipe_tok = self.cur_token.clone();
                    self.next_token(); // on target function/callable
                    let right = self.parse_expression(Precedence::Call)?;
                    match right {
                        Expr::Call(func, mut args, tok) => {
                            args.insert(0, left);
                            left = Expr::Call(func, args, tok);
                        }
                        func_expr => {
                            left = Expr::Call(Box::new(func_expr), vec![left], pipe_tok);
                        }
                    }
                }
                TokenKind::LParen => {
                    self.next_token();
                    left = self.parse_call_expression(left)?;
                }
                TokenKind::LBracket => {
                    self.next_token();
                    left = self.parse_index_expression(left)?;
                }
                TokenKind::Dot => {
                    self.next_token(); // on '.'
                    self.next_token(); // on property ident
                    if let TokenKind::Ident(ref prop) = self.cur_token.kind.clone() {
                        let dot_tok = self.cur_token.clone();
                        left = Expr::Property(Box::new(left), prop.clone(), dot_tok);
                    } else {
                        self.errors.push(format!("انتظار نام فیلد بعد از نقطه در سطر {}", self.cur_token.line));
                        return None;
                    }
                }
                _ => return Some(left),
            }
        }

        Some(left)
    }

    fn parse_prefix(&mut self) -> Option<Expr> {
        match &self.cur_token.kind {
            TokenKind::Ident(name) => Some(Expr::Ident(name.clone(), self.cur_token.clone())),
            TokenKind::Int(val) => Some(Expr::Int(*val, self.cur_token.clone())),
            TokenKind::Float(val) => Some(Expr::Float(*val, self.cur_token.clone())),
            TokenKind::Str(val) => {
                let s = val.clone();
                let tok = self.cur_token.clone();
                if s.contains('{') && s.contains('}') {
                    self.parse_interpolated_string(&s, tok)
                } else {
                    Some(Expr::Str(s, tok))
                }
            }
            TokenKind::Bool(val) => Some(Expr::Bool(*val, self.cur_token.clone())),
            TokenKind::Nil => Some(Expr::Nil(self.cur_token.clone())),
            TokenKind::Bang => {
                let tok = self.cur_token.clone();
                self.next_token();
                let right = self.parse_expression(Precedence::Prefix)?;
                Some(Expr::Prefix(PrefixOp::Bang, Box::new(right), tok))
            }
            TokenKind::Minus => {
                let tok = self.cur_token.clone();
                self.next_token();
                let right = self.parse_expression(Precedence::Prefix)?;
                Some(Expr::Prefix(PrefixOp::Minus, Box::new(right), tok))
            }
            TokenKind::LParen => {
                let _start_tok = self.cur_token.clone();
                // Check if this is an arrow lambda like () -> expr or (x, y) -> expr
                if let Some(lambda) = self.try_parse_arrow_lambda() {
                    Some(lambda)
                } else {
                    self.next_token();
                    let expr = self.parse_expression(Precedence::Lowest);
                    if !self.expect_peek(TokenKind::RParen) {
                        return None;
                    }
                    expr
                }
            }
            TokenKind::LBracket => self.parse_array_literal(),
            TokenKind::LBrace => self.parse_map_literal(),
            TokenKind::Fn => self.parse_function_literal(),
            _ => {
                self.errors.push(format!(
                    "خطای عبارت: توکن غیرمنتظره {:?} در سطر {}",
                    self.cur_token.kind, self.cur_token.line
                ));
                None
            }
        }
    }

    fn parse_infix(&mut self, left: Expr) -> Option<Expr> {
        let tok = self.cur_token.clone();
        let op = match &tok.kind {
            TokenKind::Plus => InfixOp::Plus,
            TokenKind::Minus => InfixOp::Minus,
            TokenKind::Asterisk => InfixOp::Multiply,
            TokenKind::Slash => InfixOp::Divide,
            TokenKind::Percent => InfixOp::Modulo,
            TokenKind::Eq => InfixOp::Equal,
            TokenKind::NotEq => InfixOp::NotEqual,
            TokenKind::Lt => InfixOp::LessThan,
            TokenKind::Lte => InfixOp::LessEqual,
            TokenKind::Gt => InfixOp::GreaterThan,
            TokenKind::Gte => InfixOp::GreaterEqual,
            TokenKind::And => InfixOp::And,
            TokenKind::Or => InfixOp::Or,
            _ => return None,
        };

        let precedence = Precedence::from_token(&tok.kind);
        self.next_token();
        let right = self.parse_expression(precedence)?;

        Some(Expr::Infix(Box::new(left), op, Box::new(right), tok))
    }

    fn parse_call_expression(&mut self, function: Expr) -> Option<Expr> {
        let tok = self.cur_token.clone();
        let args = self.parse_expression_list(TokenKind::RParen)?;
        Some(Expr::Call(Box::new(function), args, tok))
    }

    fn parse_index_expression(&mut self, left: Expr) -> Option<Expr> {
        let tok = self.cur_token.clone();
        self.next_token();
        let index = self.parse_expression(Precedence::Lowest)?;
        if !self.expect_peek(TokenKind::RBracket) {
            return None;
        }
        Some(Expr::Index(Box::new(left), Box::new(index), tok))
    }

    fn parse_array_literal(&mut self) -> Option<Expr> {
        let tok = self.cur_token.clone();
        let elements = self.parse_expression_list(TokenKind::RBracket)?;
        Some(Expr::Array(elements, tok))
    }

    fn parse_map_literal(&mut self) -> Option<Expr> {
        let tok = self.cur_token.clone();
        let mut pairs = Vec::new();

        if self.peek_token_is(&TokenKind::RBrace) {
            self.next_token();
            return Some(Expr::Map(pairs, tok));
        }

        self.next_token();
        let key = self.parse_expression(Precedence::Lowest)?;
        if !self.expect_peek(TokenKind::Colon) {
            return None;
        }
        self.next_token();
        let val = self.parse_expression(Precedence::Lowest)?;
        pairs.push((key, val));

        while self.peek_token_is(&TokenKind::Comma) {
            self.next_token();
            if self.peek_token_is(&TokenKind::RBrace) {
                break; // trailing comma
            }
            self.next_token();
            let k = self.parse_expression(Precedence::Lowest)?;
            if !self.expect_peek(TokenKind::Colon) {
                return None;
            }
            self.next_token();
            let v = self.parse_expression(Precedence::Lowest)?;
            pairs.push((k, v));
        }

        if !self.expect_peek(TokenKind::RBrace) {
            return None;
        }

        Some(Expr::Map(pairs, tok))
    }

    fn parse_expression_list(&mut self, end: TokenKind) -> Option<Vec<Expr>> {
        let mut list = Vec::new();

        if self.peek_token_is(&end) {
            self.next_token();
            return Some(list);
        }

        self.next_token();
        list.push(self.parse_expression(Precedence::Lowest)?);

        while self.peek_token_is(&TokenKind::Comma) {
            self.next_token();
            self.next_token();
            list.push(self.parse_expression(Precedence::Lowest)?);
        }

        if !self.expect_peek(end) {
            return None;
        }

        Some(list)
    }

    fn try_parse_arrow_lambda(&mut self) -> Option<Expr> {
        let tok = self.cur_token.clone();
        let clone_lexer = self.lexer.clone();
        let clone_cur = self.cur_token.clone();
        let clone_peek = self.peek_token.clone();

        // Tentatively parse parameter list: (param1, param2, ...) ->
        let mut params = Vec::new();
        let mut is_lambda = false;

        self.next_token(); // consume '('
        if self.cur_token_is(&TokenKind::RParen) {
            // empty params ()
            if self.peek_token_is(&TokenKind::Arrow) {
                is_lambda = true;
                self.next_token(); // on '->'
            }
        } else if let TokenKind::Ident(ref p1) = self.cur_token.kind.clone() {
            params.push(p1.clone());
            let mut valid_param_list = true;
            while self.peek_token_is(&TokenKind::Comma) {
                self.next_token(); // on ','
                self.next_token(); // on next ident
                if let TokenKind::Ident(ref p) = self.cur_token.kind.clone() {
                    params.push(p.clone());
                } else {
                    valid_param_list = false;
                    break;
                }
            }
            if valid_param_list && self.peek_token_is(&TokenKind::RParen) {
                self.next_token(); // on ')'
                if self.peek_token_is(&TokenKind::Arrow) {
                    is_lambda = true;
                    self.next_token(); // on '->'
                }
            }
        }

        if is_lambda {
            self.next_token(); // on start of lambda body expression or block '{'
            let body = if self.cur_token_is(&TokenKind::LBrace) {
                self.parse_block_statement()
            } else {
                let expr = self.parse_expression(Precedence::Lowest)?;
                BlockStmt {
                    token: tok.clone(),
                    statements: vec![Stmt::Return {
                        value: Some(expr),
                        token: tok.clone(),
                    }],
                }
            };

            Some(Expr::Function {
                name: None,
                params,
                body,
                token: tok,
            })
        } else {
            // Rollback parser state
            self.lexer = clone_lexer;
            self.cur_token = clone_cur;
            self.peek_token = clone_peek;
            None
        }
    }

    fn parse_interpolated_string(&mut self, s: &str, tok: Token) -> Option<Expr> {
        let mut segments: Vec<Expr> = Vec::new();
        let mut cur_literal = String::new();
        let mut chars = s.chars().peekable();
        let mut has_interpolation = false;

        while let Some(ch) = chars.next() {
            if ch == '{' {
                // If escaped {{, treat as literal {
                if chars.peek() == Some(&'{') {
                    chars.next();
                    cur_literal.push('{');
                    continue;
                }

                // Check if this looks like a valid expression identifier or calculation
                // If it starts with a quote, e.g. {"name": ...}, it's JSON or a literal object string, not interpolation!
                let next_ch = chars.peek().copied();
                if next_ch == Some('"') || next_ch == Some('\'') {
                    cur_literal.push('{');
                    continue;
                }
                let mut expr_str = String::new();
                let mut brace_depth = 1;
                let mut found_closing = false;
                while let Some(c) = chars.next() {
                    if c == '{' {
                        brace_depth += 1;
                        expr_str.push(c);
                    } else if c == '}' {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            found_closing = true;
                            break;
                        } else {
                            expr_str.push(c);
                        }
                    } else {
                        expr_str.push(c);
                    }
                }

                if !found_closing {
                    // Not a closed interpolation, treat as literal
                    cur_literal.push('{');
                    cur_literal.push_str(&expr_str);
                    continue;
                }

                let trimmed = expr_str.trim();
                // Try parsing trimmed as an expression
                let sub_lexer = crate::lexer::Lexer::new(trimmed);
                let mut sub_parser = Parser::new(sub_lexer);
                if let Some(parsed_expr) = sub_parser.parse_expression(Precedence::Lowest) {
                    if sub_parser.errors.is_empty() {
                        if !cur_literal.is_empty() {
                            segments.push(Expr::Str(cur_literal.clone(), tok.clone()));
                            cur_literal.clear();
                        }
                        has_interpolation = true;
                        segments.push(parsed_expr);
                        continue;
                    }
                }

                // If parsing failed (e.g. JSON syntax like {"name": ...}), treat as literal!
                cur_literal.push('{');
                cur_literal.push_str(&expr_str);
                cur_literal.push('}');
            } else if ch == '}' && chars.peek() == Some(&'}') {
                chars.next();
                cur_literal.push('}');
            } else {
                cur_literal.push(ch);
            }
        }

        if !has_interpolation {
            return Some(Expr::Str(s.to_string(), tok));
        }

        if !cur_literal.is_empty() {
            segments.push(Expr::Str(cur_literal, tok.clone()));
        }

        if segments.is_empty() {
            return Some(Expr::Str("".to_string(), tok));
        }

        let mut res = segments.remove(0);
        for next_expr in segments {
            res = Expr::Infix(
                Box::new(res),
                crate::ast::InfixOp::Plus,
                Box::new(next_expr),
                tok.clone(),
            );
        }

        Some(res)
    }

    fn parse_function_literal(&mut self) -> Option<Expr> {
        let tok = self.cur_token.clone();
        let mut func_name = None;

        // Named function e.g. fn add(a, b) { ... }
        if let TokenKind::Ident(ref name) = self.peek_token.kind.clone() {
            self.next_token();
            func_name = Some(name.clone());
        }

        if !self.expect_peek(TokenKind::LParen) {
            return None;
        }

        let params = self.parse_function_params()?;

        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }

        let body = self.parse_block_statement();

        Some(Expr::Function {
            name: func_name,
            params,
            body,
            token: tok,
        })
    }

    fn parse_function_params(&mut self) -> Option<Vec<String>> {
        let mut params = Vec::new();

        if self.peek_token_is(&TokenKind::RParen) {
            self.next_token();
            return Some(params);
        }

        self.next_token();
        if let TokenKind::Ident(ref name) = self.cur_token.kind {
            params.push(name.clone());
            if self.peek_token_is(&TokenKind::Colon) {
                self.next_token(); // on ':'
                self.next_token(); // on type
            }
        }

        while self.peek_token_is(&TokenKind::Comma) {
            self.next_token();
            self.next_token();
            if let TokenKind::Ident(ref name) = self.cur_token.kind {
                params.push(name.clone());
                if self.peek_token_is(&TokenKind::Colon) {
                    self.next_token(); // on ':'
                    self.next_token(); // on type
                }
            }
        }

        if !self.expect_peek(TokenKind::RParen) {
            return None;
        }

        Some(params)
    }
}
