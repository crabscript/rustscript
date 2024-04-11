use crate::Decl;
// use crate::Decl::*;
use crate::Expr;
use crate::ParseError;
use crate::Parser;
// use crate::{BinOpType, UnOpType};
use lexer::Token;

impl<'inp> Parser<'inp> {
    pub(crate) fn parse_if_else(&mut self, min_bp: u8) -> Result<Decl, ParseError> {
        dbg!("if else");
        dbg!(&self.lexer.peek());

        // condition - in parens
        self.consume_token_type(Token::OpenParen, "Expected open parenthesis")?;
        dbg!("OK", &self.prev_tok);

        let cond = self.parse_expr(min_bp);
        dbg!(&cond); // got cond

        dbg!("peek after:", &self.lexer.peek()); // OpenBrace
        Ok(Decl::ExprStmt(Expr::Bool(true)))
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::*;

    #[test]
    fn parse_if_basic() {
        let t = r"
        if (true) {

        }
        ";
        // test_parse(t, "");
    }
}
