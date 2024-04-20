use lexer::{lex, Token};
use logos::Lexer;
use std::iter::Peekable;
use structs::*;

pub mod blk;
pub mod expr;
pub mod fn_decl;
pub mod ident;
pub mod if_else;
pub mod let_stmt;
pub mod parse_loop;
pub mod parse_type_ann;
pub mod seq;
pub mod structs;

// To expect token types that have a value inside (for Ident and primitives)
macro_rules! expect_token_body {
    ($peek:expr, $token:ident, $expected:expr) => {{
        let err = Err(ParseError::new(concat!("Expected ", $expected)));
        let pk = $peek;

        if pk.is_none() {
            err
        } else {
            let pk = pk
                .expect("Peek has something")
                .as_ref()
                .expect("Expect lexer to succeed");
            match pk {
                Token::$token(_) => Ok(()),
                _ => err,
            }
        }
    }};
}

pub(crate) use expect_token_body;

pub struct Parser<'inp> {
    prev_tok: Option<Token>,
    lexer: Peekable<Lexer<'inp, Token>>,
    pub is_loop: bool,
    pub is_fn: bool,
}

impl<'inp> Parser<'inp> {
    pub fn new(lexer: Lexer<'_, Token>) -> Parser<'_> {
        Parser {
            prev_tok: None,
            lexer: lexer.peekable(),
            is_loop: false,
            is_fn: false,
        }
    }

    pub fn new_from_string(inp: &str) -> Parser<'_> {
        Parser {
            prev_tok: None,
            lexer: lex(inp).peekable(),
            is_loop: false,
            is_fn: false,
        }
    }

    // Check if peek is a specific token type
    fn is_peek_token_type(&mut self, token: Token) -> bool {
        let pk = self.lexer.peek();
        if pk.is_none() {
            false
        } else {
            let pk = pk.unwrap();
            match pk {
                Ok(prev) => prev.eq(&token),
                _ => false,
            }
        }
    }

    /// To expect token types at peek that have no value (most of them)
    fn expect_token_type(&mut self, token: Token, expected_msg: &str) -> Result<(), ParseError> {
        if !self.is_peek_token_type(token) {
            Err(ParseError::new(expected_msg))
        } else {
            Ok(())
        }
    }

    /// Expect token type at peek and advance if it was there
    fn consume_token_type(&mut self, token: Token, expected_msg: &str) -> Result<(), ParseError> {
        if !self.is_peek_token_type(token) {
            Err(ParseError::new(expected_msg))
        } else {
            self.advance();
            Ok(())
        }
    }

    /// If token type there, consume and advance. Otherwise do nothing.
    /// Return true if the token was consumed, else false
    fn consume_opt_token_type(&mut self, token: Token) -> bool {
        if self.is_peek_token_type(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    // Store current lexer token as prev_tok and move up lexer
    fn advance(&mut self) {
        if let Some(val) = self.lexer.peek() {
            self.prev_tok
                .replace(val.clone().expect("Expect lexer to succeed"));
            self.lexer.next();
        }
    }

    // Expect prev_tok to be there (helper method)
    fn expect_prev_tok(&self) -> Result<&Token, ParseError> {
        match &self.prev_tok {
            Some(tok) => Ok(tok),
            None => Err(ParseError::new("Expected previous token")),
        }
    }

    // Pass in self.lexer.peek() => get String out for Ident, String in quotes
    pub(crate) fn string_from_ident(token: Option<&Result<Token, ()>>) -> String {
        // dbg!("string from ident token:", &token);
        let tok = token.unwrap();
        let tok = tok.clone().unwrap();
        tok.to_string()
    }

    /// Expect one of Ident, (, or fn to start type annotation
    fn expect_token_for_type_ann(token: Option<&Result<Token, ()>>) -> Result<(), ParseError> {
        if let Some(Ok(tok)) = token {
            match tok {
                Token::Ident(_) | Token::OpenParen => Ok(()),
                _ => {
                    let e = format!(
                        "Expected identifier or '(' for type annotation, got '{}'",
                        tok
                    );
                    Err(ParseError::new(&e))
                }
            }
        } else {
            Err(ParseError::new(
                "Expected identifier or '(' for type annotation, got end of input",
            ))
        }
    }
    /* Precedence */

    // Return (left bp, right bp)
    // Adapted from: https://doc.rust-lang.org/reference/expressions.html
    // (left, right) => left < right means left associative. left > right means right associative. equal => no associativity (error)
    fn get_infix_bp(binop: &BinOpType) -> (u8, u8) {
        match binop {
            BinOpType::Mul | BinOpType::Div => (8, 9),
            BinOpType::Add | BinOpType::Sub => (6, 7),
            // no associativity for comparison ops
            BinOpType::LogicalEq | BinOpType::Gt | BinOpType::Lt => (5, 5),
            BinOpType::LogicalAnd => (3, 4),
            BinOpType::LogicalOr => (1, 2),
        }
    }

    // Unary negation must have a higher precedence than binops
    fn get_prefix_bp(unop: &UnOpType) -> ((), u8) {
        match unop {
            UnOpType::Negate | UnOpType::Not => ((), 10),
        }
    }

    // Parses and returns a declaration. At this stage "declaration" includes values, let assignments, fn declarations, etc
    // Because treatment of something as an expression can vary based on whether it is last value or not, whether semicolon comes after, etc.
    fn parse_decl(&mut self) -> Result<Decl, ParseError> {
        let prev_tok = self.expect_prev_tok()?;
        match prev_tok {
            Token::Integer(_)
            | Token::Float(_)
            | Token::Bool(_)
            | Token::Minus
            | Token::Ident(_)
            | Token::OpenParen
            | Token::Bang
            | Token::OpenBrace
            | Token::If
            | Token::String(_) => self.parse_expr(0),
            Token::Spawn => {
                self.advance();
                let fn_call = self.parse_expr(0)?.to_expr()?;
                if let Expr::FnCallExpr(fn_data) = fn_call {
                    let sp = Expr::SpawnExpr(fn_data);
                    Ok(Decl::ExprStmt(sp))
                } else {
                    Err(ParseError::new("spawn expected function call"))
                }
            }
            // join t;
            Token::Join => {
                self.advance();
                let join_id = self.parse_expr(0)?.to_expr()?;
                if let Expr::Symbol(tid) = join_id {
                    let j = Expr::JoinExpr(tid);
                    Ok(Decl::ExprStmt(j))
                } else {
                    Err(ParseError::new("join expected variable for thread to join"))
                }
            }
            // wait sem;
            Token::Wait => {
                self.advance();
                let sem = self.parse_expr(0)?.to_expr()?;
                if let Expr::Symbol(sem_sym) = sem {
                    Ok(Decl::WaitStmt(sem_sym))
                } else {
                    Err(ParseError::new("wait expected semaphore variable"))
                }
            }
            Token::Post => {
                self.advance();
                let sem = self.parse_expr(0)?.to_expr()?;
                if let Expr::Symbol(sem_sym) = sem {
                    Ok(Decl::PostStmt(sem_sym))
                } else {
                    Err(ParseError::new("post expected semaphore variable"))
                }
            }
            // if not is_loop, error
            Token::Break => {
                if !self.is_loop {
                    return Err(ParseError::new("break outside of loop"));
                }
                Ok(Decl::BreakStmt)
            }
            Token::Yield => Ok(Decl::YieldStmt),
            // if not is_fn, err
            Token::Return => {
                if !self.is_fn {
                    return Err(ParseError::new("return outside of fn"));
                }

                // parse expr if not semicolon
                let mut ret_expr: Option<Expr> = None;
                if !self.is_peek_token_type(Token::Semi) {
                    self.advance();
                    let expr = self.parse_expr(0)?.to_expr()?;
                    ret_expr.replace(expr);
                }

                Ok(Decl::ReturnStmt(ret_expr))
            }
            Token::Let => self.parse_let(),
            Token::Loop => self.parse_loop(),
            Token::Fn => self.parse_fn_decl(),
            _ => Err(ParseError::new(&format!(
                "Unexpected token: '{}'",
                prev_tok
            ))),
        }
    }

    // Implicit block
    pub fn parse(mut self) -> Result<BlockSeq, ParseError> {
        self.parse_seq()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;

    pub fn test_parse(inp: &str, expected: &str) {
        let lex = Token::lexer(inp);
        let parser = Parser::new(lex);
        let res = parser.parse().expect("Should parse");
        dbg!(&res);
        assert_eq!(res.to_string(), expected);
    }

    pub fn test_parse_err(inp: &str, exp_err: &str, contains: bool) {
        let lex = Token::lexer(inp);
        let parser = Parser::new(lex);
        let res = parser.parse().expect_err("Should err");

        dbg!(&res.to_string());

        if contains {
            assert!(res.to_string().contains(exp_err))
        } else {
            assert_eq!(res.to_string(), exp_err);
        }
    }

    #[test]
    fn test_parse_ints() {
        test_parse("", "");
        test_parse(" 20\n ", "20"); // expr only
        test_parse(" 20;\n ", "20;"); // one exprstmt
        test_parse(" 20 ;30 \n ", "20;30"); // exprstmt, expr
        test_parse(" 20 ;30; \n ", "20;30;"); // exprstmt, exprsmt
        test_parse(" 20 ;30; \n40 \n ", "20;30;40"); // two exprstmt + expr
    }

    #[test]
    fn test_parse_floats() {
        test_parse(" 2.2\n ", "2.2");
        test_parse(" 2.23\n ", "2.23");
        test_parse(" 2.23; 4.5\n ", "2.23;4.5");
        test_parse(" 2.23; 4.5; 4.6\n ", "2.23;4.5;4.6");
    }

    #[test]
    fn test_parse_bools() {
        test_parse("true\n ", "true");
        test_parse("true; false\n ", "true;false");
        test_parse("true; false; true;\n ", "true;false;true;");
        test_parse("true; false; true; false\n ", "true;false;true;false");
    }

    #[test]
    fn test_parse_mixed_primitives() {
        test_parse(
            "true; 2; 4.5; false; 200; 7.289; 90; true",
            "true;2;4.5;false;200;7.289;90;true",
        );
        test_parse(
            "true; 2; 4.5; false; 200; 7.289; 90; true; 2.1;",
            "true;2;4.5;false;200;7.289;90;true;2.1;",
        )
    }

    #[test]
    fn test_errs_for_consecutive_exprs() {
        test_parse_err("20 30", "infix operator", true);
    }

    #[test]
    fn test_parse_assignment() {
        test_parse_err("x = y = 2", "not an expression", true);
        test_parse_err("let x = y = 2;", "not an expression", true);

        test_parse("x = 3+2;", "x = (3+2);");
        // not type checked yet - allows for us to do dynamic typing if we want
        test_parse(
            "let x : int = 20; x = true; x",
            "let x : int = 20;x = true;x",
        );
    }

    #[test]
    fn test_parse_concurrency() {
        let t = r"
        let t = spawn func();
        spawn f2();
        spawn f3()
        ";
        test_parse(t, "let t = spawn func();spawn f2();spawn f3()");

        let t = r"
        spawn 2+2;
        ";
        test_parse_err(t, "spawn expected function call", true);

        // join
        let t = r"
        let t = spawn func();
        let res = join t;
        ";
        test_parse(t, "let t = spawn func();let res = join t;");

        // wait and post
        let t = r"
        let sem = sem_create();
        wait sem;
        post sem;
        ";
        test_parse(t, "let sem = sem_create();wait sem;post sem;");

        let t = r"
        wait 2+2;
        ";
        test_parse_err(t, "expected semaphore variable", true);

        let t = r"
        post 2+2;
        ";
        test_parse_err(t, "expected semaphore variable", true);

        // can't assign wait/post
        let t = r"
        let x = wait sem;
        ";
        test_parse_err(t, "wait is not an expression", true);

        // can't assign wait/post
        let t = r"
        let x = post sem;
        ";
        test_parse_err(t, "post is not an expression", true);

        // must be stmt with semi
        let t = r"
         wait sem
         ";
        test_parse_err(t, "Expected semicolon", true);

        let t = r"
         post sem
         ";
        test_parse_err(t, "Expected semicolon", true);
    }

    #[test]
    fn test_parse_string() {
        let t = r#""hello" + "world""#;
        test_parse(t, "(hello+world)");

        let t = r#"let t = "hello world"; println(t);"#;
        test_parse(t, "let t = hello world;println(t);");
    }

    #[test]
    fn test_parse_type_annotations_more() {
        let t = r"
        let g : fn(int) -> int = f;
        ";
    }
}
