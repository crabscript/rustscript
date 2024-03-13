use std::fmt::Display;
use std::iter::Peekable;

use lexer::Token;
use logos::Lexer;

// Different from bytecode Value because values on op stack might be different (e.g fn call)
#[derive(Debug, PartialEq)]
pub enum Expr {
    Integer(i64),
    Float(f64),
    Bool(bool),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Expr::Integer(val) => val.to_string(),
            Expr::Float(val) => val.to_string(),
            Expr::Bool(val) => val.to_string(),
        };

        write!(f, "{}", string)
    }
}

// Later: LetStmt, IfStmt, FnDef, etc.
#[derive(Debug, PartialEq)]
pub enum Decl {
    ExprStmt(Expr),
    Block,
}

impl Display for Decl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Decl::ExprStmt(expr) => expr.to_string(),
            Decl::Block => unimplemented!(),
        };

        write!(f, "{}", string)
    }
}

// Last expression is value of program semantics (else Unit type)
// Program is either one declaration or a sequence of declarations with optional last expression
#[derive(Debug, PartialEq)]
pub struct Program {
    decls: Vec<Decl>,
    last_expr: Option<Expr>,
}

impl Display for Program {
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
    decls: Vec<Decl>,
    prev_tok: Option<Token>,
    lexer: Peekable<Lexer<'inp, Token>>,
}

impl<'inp> Parser<'inp> {
    pub fn new<'src>(lexer: Lexer<'src, Token>) -> Parser<'src> {
        Parser {
            decls: vec![],
            prev_tok: None,
            lexer: lexer.peekable(),
        }
    }

    // expect peek to be semicolon
    fn expect_semicolon(&mut self) -> Result<(), ParseError> {
        let err = Err(ParseError::new("Expected semicolon"));
        let pk = self.lexer.peek();

        if pk.is_none() {
            err
        } else {
            let pk = pk
                .expect("Peek has something")
                .as_ref()
                .expect("Expect lexer to suceed");
            match pk {
                Token::Semi => Ok(()),
                _ => err,
            }
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

    // So that we can reuse this for last expr
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let prev_tok = self
            .prev_tok
            .as_ref()
            .expect("prev_tok should not be empty");
        match prev_tok {
            Token::Integer(val) => Ok(Expr::Integer(*val)),
            Token::Float(val) => Ok(Expr::Float(*val)),
            Token::Bool(val) => Ok(Expr::Bool(*val)),
            _ => unimplemented!(),
        }
    }

    // Program is a sequence of declarations
    fn parse_decl(&mut self) -> Result<(), ParseError> {
        let val = self.parse_expr()?;
        self.decls.push(Decl::ExprStmt(val));

        self.expect_semicolon()?; // invariant: after parsing previous leave peek at where semicolon would be
        self.advance(); // should be called once done (consume semicolon)
        Ok(())
    }

    pub fn parse(mut self) -> Result<Program, ParseError> {
        self.advance();

        while let Some(_) = self.lexer.peek() {
            self.parse_decl()?;

            // need to call lexer.next() at least once somewhere so the loop breaks
            self.advance();
        }

        let mut last_expr: Option<Expr> = None;
        if let Some(ref val) = self.prev_tok {
            match val {
                Token::Integer(_) | Token::Float(_) | Token::Bool(_) => {
                    let v = self.parse_expr()?;
                    last_expr.replace(v);
                }
                _ => (),
            }
        }

        Ok(Program {
            decls: self.decls,
            last_expr,
        })
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
        assert_eq!(res.to_string(), expected);
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
    fn test_parse_ints() {
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
    fn test_parse_mixed() {
        test_parse("true; 2; 4.5; false; 200; 7.289; 90; true", "true;2;4.5;false;200;7.289;90;true");
        test_parse("true; 2; 4.5; false; 200; 7.289; 90; true; 2.1;", "true;2;4.5;false;200;7.289;90;true;2.1;")
    }

    #[test]
    fn test_errs_for_consecutive_exprs() {
        test_parse_err("20 30", "Expected semicolon", true);
    }
}
