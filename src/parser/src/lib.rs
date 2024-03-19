use std::fmt::Display;
use std::iter::Peekable;
use std::rc::Rc;

use lexer::Token;
use logos::Lexer;

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


// Different from bytecode Value because values on op stack might be different (e.g fn call)
#[derive(Debug)]
pub enum Expr {
    Integer(i64),
    Float(f64),
    Bool(bool),
    Block(BlockSeq) // expr can be a block
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Expr::Integer(val) => val.to_string(),
            Expr::Float(val) => val.to_string(),
            Expr::Bool(val) => val.to_string(),
            Expr::Block(seq) => seq.to_string()
        };

        write!(f, "{}", string)
    }
}

#[derive(Debug)]
pub struct LetStmt {
    ident:String, 
    expr:Expr
}

impl Display for LetStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "let {} = {}", self.ident, self.expr) 
    }
}

// Later: LetStmt, IfStmt, FnDef, etc.
#[derive(Debug)]
pub enum Decl {
    LetStmt(LetStmt),
    ExprStmt(Expr),
    Block(BlockSeq),
}

impl Decl {
    fn to_expr(self) -> Expr {
        match self {
            LetStmt(expr) => expr.expr,
            ExprStmt(expr) => expr,
            Block(seq) => Expr::Block(seq)
        }
    }
}

impl Display for Decl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Decl::ExprStmt(expr) => expr.to_string(),
            Decl::LetStmt(stmt) => stmt.to_string(),
            _ => unimplemented!(),
        };

        write!(f, "{}", string)
    }
}

// Last expression is value of program semantics (else Unit type)
// Program is either one declaration or a sequence of declarations with optional last expression
#[derive(Debug)]
pub struct BlockSeq {
    decls: Vec<Decl>,
    last_expr: Option<Rc<Expr>>,
}

impl Display for BlockSeq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let decls = self
            .decls
            .iter()
            .map(|d| format!("{};", d))
            .collect::<String>();
        let expr = match &self.last_expr {
            Some(expr) => expr.to_string(),
            None => String::from(""),
        };

        write!(f, "{}{}", decls, expr)
    }
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
    msg: String,
}

impl ParseError {
    pub fn new(err: &str) -> ParseError {
        ParseError {
            msg: err.to_owned(),
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ParseError]: {}", self.msg)
    }
}

pub struct Parser<'inp> {
    prev_tok: Option<Token>,
    lexer: Peekable<Lexer<'inp, Token>>,
}

use Decl::*;
impl<'inp> Parser<'inp> {
    pub fn new<'src>(lexer: Lexer<'src, Token>) -> Parser<'src> {
        Parser {
            prev_tok: None,
            lexer: lexer.peekable(),
        }
    }

    // Check if peek is a specific token type
    fn is_peek_token_type(&mut self, token: Token)-> bool{
        let pk = self.lexer.peek();
        if pk.is_none() {
            return false;
        } else {
            let pk = pk.unwrap();
            match pk {
                Ok(prev) => prev.eq(&token),
                _ => false
            }
        }
    }

    // To expect token types that have no value (most of them)
    fn expect_token_type(&mut self, token:Token, expected_msg: &str) -> Result<(), ParseError> {
        if !self.is_peek_token_type(token) {
            Err(ParseError::new(expected_msg))
        } else {
            Ok(())
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

    // Pass in self.lexer.peek() => get String out for Ident, String in quotes
    fn string_from_ident(token: Option<&Result<Token, ()>>) -> String {
        let tok = token.unwrap();
        let tok = tok.clone().unwrap();
        tok.to_string()
    }

    // Parse let statement
        // let x = 2;
    fn parse_let(&mut self) -> Result<Decl, ParseError> {
        // self.advance();

        expect_token_body!(self.lexer.peek(), Ident, "Expected identifier")?;
        let ident = Parser::string_from_ident(self.lexer.peek());

        dbg!("IDENT:", ident);

        // let ident = self.lexer.peek().expect("Expected identifier").clone().expect("Expected identifier");
        // let ident = ident.

        // self.expect_token_type(Token::Eq, "Expected '='")?;
        
        // let expr = self.parse_expr()?;

        // // Error if assigning to an actual declaration (let, fn)
        // match expr {
        //     Decl::LetStmt(_) => return Err(ParseError::new("Can't assign to a let statement")),
        //     _ => ()
        // }
        
        // let let_stmt:LetStmt = {
        //     ide
        // }
        Ok(ExprStmt(Expr::Bool(true)))
    }

    // Parses and returns an expression. At this stage "expression" includes values, let assignments, fn declarations, etc
        // Because treatment of something as an expression can vary based on whether it is last value or not, whether semicolon comes after, etc.
    fn parse_expr(&mut self) -> Result<Decl, ParseError> {
        let prev_tok = self
            .prev_tok
            .as_ref()
            .expect("prev_tok should not be empty");
        match prev_tok {
            Token::Integer(val) => Ok(ExprStmt(Expr::Integer(*val))),
            Token::Float(val) => Ok(ExprStmt(Expr::Float(*val))),
            Token::Bool(val) => Ok(ExprStmt(Expr::Bool(*val))),
            Token::Let => self.parse_let(),
            _ => unimplemented!(),
        }
    }

    

    pub fn parse_seq(&mut self)->Result<BlockSeq, ParseError> {
        let mut decls:Vec<Decl> = vec![];
        let mut last_expr:Option<Expr> = None;    

        while let Some(_) = self.lexer.peek() {
            self.advance();

            let expr = self.parse_expr()?;

            // end of block: lexer empty OR curly brace (TODO add curly later)
            if self.lexer.peek().is_none() || self.is_peek_token_type(Token::CloseBrace) {
                last_expr.replace(expr.to_expr());
                break;
            }

            // semicolon: parse as stmt
            // let semi = expect_token_body!(Semi, "semicolon");
            else if self.is_peek_token_type(Token::Semi) {
                decls.push(expr);
                self.advance();
            }

            // TODO: check if expr is a block-like expression (if so, treat as statement)
                // if it was the tail it should be handled at the first branch


            // Syntax error
             
            else {
                return Err(ParseError::new("Expected semicolon"))
            }
            
        }
        dbg!(&last_expr, &decls);
        Ok(BlockSeq {
            decls,
            last_expr: last_expr.map(|x| Rc::new(x))
        })
    }

    // Implicit block
    pub fn parse(mut self)->Result<BlockSeq,ParseError> {
        self.parse_seq()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;

    fn test_parse(inp: &str, expected: &str) {
        let lex = Token::lexer(inp);
        let parser = Parser::new(lex);
        let res = parser.parse().expect("Should parse");
        // assert_eq!(res.to_string(), expected);
    }

    fn test_parse_err(inp: &str, exp_err: &str, contains: bool) {
        let lex = Token::lexer(inp);
        let parser = Parser::new(lex);
        let res = parser.parse().expect_err("Should err");

        if contains {
            assert!(res.to_string().contains(exp_err))
        } else {
            assert_eq!(res.to_string(), exp_err);
        }
    }

    #[test] 
    fn play() {
        let mut lex = Token::lexer("}");
        let mut p = Parser::new(lex);
        // p.advance();
        dbg!(p.is_peek_token_type(Token::CloseBrace));
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
        test_parse("true; 2; 4.5; false; 200; 7.289; 90; true", "true;2;4.5;false;200;7.289;90;true");
        test_parse("true; 2; 4.5; false; 200; 7.289; 90; true; 2.1;", "true;2;4.5;false;200;7.289;90;true;2.1;")
    }

    #[test]
    fn test_parse_let() {
        test_parse("let x = 2;", "let x = 2;")
    }

    #[test]
    fn test_errs_for_consecutive_exprs() {
        test_parse_err("20 30", "Expected semicolon", true);
    }
}
