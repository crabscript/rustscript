use std::fmt::Display;
use std::iter::Peekable;

use lexer::Token;
use logos::Lexer;

// Different from bytecode Value because values on op stack might be different (e.g fn call)
#[derive(Debug, PartialEq)]
pub enum Expr {
    Integer(i64),
    Float(i64),
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
        let decls = self.decls.iter().map(|d| format!("{};",d)).collect::<String>();
        let expr = match &self.last_expr {
            Some(expr) => expr.to_string(),
            None => String::from("")
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
    prev_tok:Option<Token>,
    lexer: Peekable<Lexer<'inp, Token>>,
}

impl<'inp> Parser<'inp> {
    pub fn new<'src>(lexer: Lexer<'src, Token>) -> Parser<'src> {
        Parser { decls:vec![], prev_tok:None, lexer: lexer.peekable() }
    }

    // Assume input is valid: no double semi or double expr. semicolon means prev_tok has a value
    // pub fn parse(mut self) -> Result<Program, ParseError> {
    //     let mut decls: Vec<Decl> = vec![];
    //     let mut prev_tok: Option<Token> = None;

    //     for tok_res in self.lexer.into_iter() {
    //         let tok = tok_res.expect("Expect token");
    
    //         match tok {
    //             Token::Integer(_) => {
    //                 if prev_tok.is_some() {
    //                     return Err(ParseError::new("Consecutive expressions not allowed"));
    //                 }
    //                 prev_tok.replace(tok);
    //             }
    //             Token::Semi => {
    //                 let tok = prev_tok
    //                     .clone()
    //                     .expect("Expected expression before semicolon");
    //                 match tok {
    //                     Token::Integer(val) => decls.push(Decl::ExprStmt(Expr::Integer(val))),
    //                     _ => return Err(ParseError::new("Unexpected token type")),
    //                 }
    //                 prev_tok.take();
    //             }
    //             _ => unimplemented!()
    //         }
    //     }

    //     let mut ret_expr: Option<Expr> = None;
    //     if let Some(tok) = prev_tok {
    //         match tok {
    //             Token::Integer(val) => {
    //                 ret_expr.replace(Expr::Integer(val));
    //             }
    //             _ => return Err(ParseError::new("Unexpected token type")),
    //         }
    //     }

    //     Ok(Program {
    //         decls,
    //         last_expr: ret_expr,
    //     })
    // }

    // expect peek to be semicolon
    fn expect_semicolon(&mut self)->Result<(), ParseError> {
        let err = Err(ParseError::new("Expected semicolon")); 
        let pk = self.lexer.peek();

        if pk.is_none() {
            err
        } else {
            let pk = pk.expect("Expect lexer to succeed").as_ref().expect("Peek has something");
            match pk {
                Token::Semi => Ok(()),
                _ => err
            }
        }
    }

    fn advance(&mut self) {
        if let Some(val) = self.lexer.peek() {
            self.prev_tok.replace(val.clone().expect("Expect lexer to succeed"));
            self.lexer.next();
        }
    }

    fn parse_atomic(&mut self)->Result<(), ParseError> {
        dbg!("atomic:", &self.prev_tok);
        
        Ok(())

    }   

    fn parse_expr(&mut self)->Result<(), ParseError>  {
        let prev_tok = self.prev_tok.as_ref().expect("prev_tok should not be empty");
        match prev_tok {
            Token::Integer(_) | Token::Float(_) | Token::Bool(_) => {
                self.parse_atomic()?
            },
            _ => unimplemented!()
        }

        Ok(())
    }

    fn parse_decl(&mut self)->Result<(), ParseError> {
        self.parse_expr()?;
        self.expect_semicolon()?; // invariant: after parsing previous leave peek at where semicolon would be
        self.advance(); // should be called once done (consume semicolon)
        Ok(())
    }

    pub fn parse(mut self)->Result<Program,ParseError> {
        self.advance();

        while let Some(_) = self.lexer.peek() {
            self.parse_decl()?;

            // need to call lexer.next() at least once somewhere so the loop breaks
            self.advance();
        }

        let last_expr:Option<Expr> = None;
        Ok(
            Program {
                decls: self.decls,
                last_expr
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use logos::Logos;
    use super::*;

    fn test_parse(inp:&str, expected:&str) {
        let lex = Token::lexer(inp);
        let parser = Parser::new(lex);
        let res = parser.parse().expect("Should parse");
        assert_eq!(res.to_string(), expected);
    }

    fn test_parse_err(inp:&str, exp_err:&str, contains:bool) {
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
        let mut lexer = Token::lexer("20; 30;");
        let p = Parser::new(lexer);
        dbg!(p.parse());
    }

    #[test]
    fn test_parse_ints() {
        test_parse(" 20\n ", "20"); // expr only
        test_parse(" 20;\n ", "20;"); // one exprstmt
        test_parse(" 20 ;30 \n ", "20;30"); // exprstmt, expr
        test_parse(" 20 ;30; \n ", "20;30;"); // exprstmt, exprsmt
        test_parse(" 20 ;30; \n40 \n ", "20;30;40"); // two exprstmt + expr
    }

    // #[test]
    // fn test_parse_floats() {
    //     test_parse(" 2.2\n ", "2.2"); // expr only
    //     test_parse(" 2.23\n ", "2.23;"); // expr only
    // }

    #[test]
    fn test_errs_for_consecutive_exprs() {
        test_parse_err("20 30", "Consecutive expressions not allowed", true);
    }
}
