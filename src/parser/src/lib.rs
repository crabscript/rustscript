use std::fmt::Display;

use lexer::Token;
use logos::Lexer;


#[derive(Debug, PartialEq)]
pub enum Expr {
    Integer(i64)
}

// Later: LetStmt, IfStmt, FnDef, etc.
#[derive(Debug, PartialEq)]
pub enum Decl {
    ExprStmt(Expr),
    Block
}

// Last expression is value of program semantics (else Unit type)
// Program is either one declaration or a sequence of declarations with optional last expression
    // Decl is there to avoid needing to do recursive Seq for a bunch of stmts in order
// Seq is for top level. Use Decl::Block for block scoping
#[derive(Debug, PartialEq)]
pub struct Program {
    decls:Vec<Decl>,
    last_expr: Option<Expr>
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
    msg:String
}

impl ParseError {
    pub fn new(err:&str)->ParseError {
        ParseError {
            msg:err.to_owned()
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ParseError]: {}", self.msg)
    }
}

pub struct Parser<'inp> {
    lexer: Lexer<'inp, Token>
}

impl <'inp> Parser<'inp> {
    pub fn new<'src>(lexer: Lexer<'src, Token>)->Parser<'src> {
        Parser {
            lexer
        }
    }

    // Assume input is valid: no double semi or double expr. semicolon means prev_tok has a value
    pub fn parse(mut self)->Result<Program, ParseError> {
        let mut decls:Vec<Decl> = vec![];
        let mut prev_tok:Option<Token> = None;
        

        for tok_res in self.lexer.into_iter() {
            let tok = tok_res.expect("Expect token");
            // if prev_tok.is_none() {
            //     prev_tok.replace(tok.clone());
            //     continue;
            // }

            match tok {
                Token::Integer(_) => {
                    if prev_tok.is_some() {
                        return Err(ParseError::new("Consecutive expressions not allowed"))
                    }
                    prev_tok.replace(tok);
                },
                Token::Semi => {
                    let tok = prev_tok.clone().expect("Expected expression before semicolon");
                    match tok {
                        Token::Integer(val) => decls.push(Decl::ExprStmt(Expr::Integer(val))),
                        _ => return Err(ParseError::new("Unexpected token type"))
                    }
                    prev_tok.take();
                    
                },
                _ => {}
            }
        }

        let mut ret_expr:Option<Expr> = None;
        if let Some(tok) = prev_tok {
            match tok {
                Token::Integer(val) => { 
                    ret_expr.replace(Expr::Integer(val));
                },
                _ => return Err(ParseError::new("Unexpected token type"))
            }
        }

        Ok(Program { decls, last_expr: ret_expr })
    }

}

#[cfg(test)]
mod tests {
    use logos::Logos;

    use super::*;
    #[test]
    fn can_parse_ints() {
        let inp = " 20 ";
        let lex = Token::lexer(inp);
        let parser = Parser::new(lex);
        let res = parser.parse();
    }
}
