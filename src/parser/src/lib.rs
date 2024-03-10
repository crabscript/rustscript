use std::fmt::Display;

use lexer::Token;
use logos::{Lexer, Logos};


pub fn add(left: usize, right: usize) -> usize {
    left + right
}

enum Expr {
    Integer(i64)
}

enum Decl {
    ExprStmt(Expr)
}

// Last expression is value of program semantics (else Unit type)
// Program is either one declaration or a sequence of declarations with optional last expression
    // Decl is there to avoid needing to do recursive Seq for a bunch of stmts in order
pub enum ASTNode {
    Decl,
    Seq(Vec<ASTNode>, Option<Expr>)
}

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

    pub fn parse(self)->Result<ASTNode, ParseError> {
        Ok(ASTNode::Decl)
    }

}

#[cfg(test)]
mod tests {
    #[test]
    fn can_lex() {
        let m = String::from("20; 30");
        let res = lexer::lex(m.as_str());
        let res = res.collect::<Vec<_>>();
        dbg!(res);
    }
}
