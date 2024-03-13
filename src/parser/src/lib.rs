use std::fmt::Display;

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
    lexer: Lexer<'inp, Token>,
}

impl<'inp> Parser<'inp> {
    pub fn new<'src>(lexer: Lexer<'src, Token>) -> Parser<'src> {
        Parser { lexer }
    }

    // Assume input is valid: no double semi or double expr. semicolon means prev_tok has a value
    pub fn parse(mut self) -> Result<Program, ParseError> {
        let mut decls: Vec<Decl> = vec![];
        let mut prev_tok: Option<Token> = None;

        for tok_res in self.lexer.into_iter() {
            let tok = tok_res.expect("Expect token");
    
            match tok {
                Token::Integer(_) => {
                    if prev_tok.is_some() {
                        return Err(ParseError::new("Consecutive expressions not allowed"));
                    }
                    prev_tok.replace(tok);
                }
                Token::Semi => {
                    let tok = prev_tok
                        .clone()
                        .expect("Expected expression before semicolon");
                    match tok {
                        Token::Integer(val) => decls.push(Decl::ExprStmt(Expr::Integer(val))),
                        _ => return Err(ParseError::new("Unexpected token type")),
                    }
                    prev_tok.take();
                }
                _ => unimplemented!()
            }
        }

        let mut ret_expr: Option<Expr> = None;
        if let Some(tok) = prev_tok {
            match tok {
                Token::Integer(val) => {
                    ret_expr.replace(Expr::Integer(val));
                }
                _ => return Err(ParseError::new("Unexpected token type")),
            }
        }

        Ok(Program {
            decls,
            last_expr: ret_expr,
        })
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
    fn test_parse_ints() {
        test_parse(" 20\n ", "20"); // expr only
        test_parse(" 20;\n ", "20;"); // one exprstmt
        test_parse(" 20 ;30 \n ", "20;30"); // exprstmt, expr
        test_parse(" 20 ;30; \n ", "20;30;"); // exprstmt, exprsmt
        test_parse(" 20 ;30; \n40 \n ", "20;30;40"); // two exprstmt + expr

    }

    #[test]
    fn test_errs_for_consecutive_exprs() {
        test_parse_err("20 30", "Consecutive expressions not allowed", true);
    }
}
