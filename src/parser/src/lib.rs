use lexer::{lex, Token};
use logos::Lexer;
use std::iter::Peekable;
use structs::*;

pub mod blk;
pub mod expr;
pub mod ident;
pub mod if_else;
pub mod let_stmt;
pub mod parse_loop;
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
}

impl<'inp> Parser<'inp> {
    pub fn new(lexer: Lexer<'_, Token>) -> Parser<'_> {
        Parser {
            prev_tok: None,
            lexer: lexer.peekable(),
            is_loop: false,
        }
    }

    pub fn new_from_string(inp: &str) -> Parser<'_> {
        Parser {
            prev_tok: None,
            lexer: lex(inp).peekable(),
            is_loop: false,
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
    fn string_from_ident(token: Option<&Result<Token, ()>>) -> String {
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

    /// Parse and return type annotation. Expect lexer.peek() to be at Colon before call
    fn parse_type_annotation(&mut self) -> Result<Type, ParseError> {
        self.consume_token_type(Token::Colon, "Expected a colon")?;
        // expect_token_body!(self.lexer.peek(), Ident, "identifier")?;
        Parser::expect_token_for_type_ann(self.lexer.peek())?;

        // if ident, get the string and try to convert type. else, handle specially
        let peek = self
            .lexer
            .peek()
            .unwrap()
            .to_owned()
            .expect("Lexer should not fail"); // would have erred earlier

        let type_ann = match peek {
            Token::Ident(id) => Type::from_string(&id),
            Token::OpenParen => {
                self.advance();
                if let Some(Ok(Token::CloseParen)) = self.lexer.peek() {
                    Ok(Type::Unit)
                } else {
                    Err(ParseError::new("Expected '()' for unit type annotation"))
                }
            }
            _ => unreachable!(),
        }?;

        // Peek should be at equals at the end, so we advance
        self.advance();

        Ok(type_ann)
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

    // fn parse_ident(&mut self, ident: String, min_bp: u8) -> Result<Decl, ParseError> {
    //     let sym = Expr::Symbol(ident.to_string());

    //     // Handle assignment, fn call
    //     if let Some(tok) = self.lexer.peek() {
    //         let tok = tok.as_ref().expect("Lexer should not fail");

    //         // Assignment x = 2
    //         if tok.eq(&Token::Eq) {
    //             self.consume_token_type(Token::Eq, "Expected '='")?;
    //             self.advance();

    //             // now prev_tok has the start of the expr
    //             let expr = self.parse_expr(min_bp)?.to_expr()?;

    //             let assign = AssignStmtData { ident, expr };

    //             return Ok(AssignStmt(assign));
    //         } else if tok.eq(&Token::OpenParen) {
    //             // Fn call
    //             self.consume_token_type(Token::OpenParen, "Expected '('")?;
    //             dbg!("tok after:", &self.lexer.peek());

    //             // self.advance(); // put first token of param list
    //         }
    //     }

    //     Ok(ExprStmt(sym))
    // }

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
            | Token::If => self.parse_expr(0),
            // if not is_loop, error
            Token::Break => {
                if !self.is_loop {
                    return Err(ParseError::new("break outside of loop"));
                }
                Ok(Decl::BreakStmt)
            }
            Token::Let => self.parse_let(),
            Token::Loop => self.parse_loop(),
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
    fn test_parse_type_annotations() {
        test_parse("let x : int = 2;", "let x : int = 2;");
        test_parse("let x : bool = true;", "let x : bool = true;");
        test_parse("let x : float = true;", "let x : float = true;");
        test_parse("let x : () = true;", "let x : () = true;");
    }

    #[test]
    fn test_parse_type_annotations_errs() {
        // test_parse("let x : int = 2;", "");
        test_parse_err(
            "let x : let ",
            "Expected identifier or '(' for type annotation, got 'let'",
            true,
        );
        test_parse_err(
            "let x : 2 ",
            "Expected identifier or '(' for type annotation, got '2'",
            true,
        );
        test_parse_err(
            "let x : ",
            "Expected identifier or '(' for type annotation, got end of input",
            true,
        );
        test_parse_err(
            "let x : (2 ",
            "Expected '()' for unit type annotation",
            true,
        );
    }
}
