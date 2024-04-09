use crate::Decl;
use crate::Expr;
use crate::ParseError;
use crate::Parser;
use lexer::Token;

impl<'inp> Parser<'inp> {
    pub fn parse_blk(&mut self, _min_bp: u8) -> Result<Decl, ParseError> {
        // BlockSeq - vec decls, last expr
        // self.advance(); // put first tok of block into prev_tok
        let blk = self.parse_seq()?;
        let res = Decl::ExprStmt(Expr::Block(blk));
        let err = format!("Expected '{}' to close block", Token::CloseBrace);
        self.consume_token_type(Token::CloseBrace, &err)?;

        dbg!("prev_tok after blk:", &self.prev_tok);
        // dbg!("peek after blk:", &self.lexer.peek());

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::*;
    #[test]
    fn test_parse_blk_simple() {
        //     test_parse("{ 2 }", "{ 2 }");
        //     test_parse("{ 2+3 }", "{ (2+3) }");
        test_parse("{ 2; }", "{ 2; }");
        // test_parse("{ 2; 3; }", "{ 2;3; }");
        // test_parse("{ 2; 3 }", "{ 2;3 }");
        // test_parse("{ 2; 3; 4 }", "{ 2;3;4 }");
    }

    #[test]
    fn test_parse_blk_more() {
        // blk expr at the end
        let t = r"
        let x = 2;
        {
            let x = 3;
            x
        }
        ";
        test_parse(t, "let x = 2;{ let x = 3;x }");

        // blk stmt at the end
        let t = r"
        let x = 2;
        {
            let x = 3;
            x
        };
        ";
        test_parse(t, "let x = 2;{ let x = 3;x };");

        // blk in middle with semi
        let t = r"
        let x = 2;
        {
            let x = 3;
            x
        };
        x
        ";
        test_parse(t, "let x = 2;{ let x = 3;x };x");

        // blk in the middle without semi - we allow this to parse but type checker will reject (blk stmt should have unit) (?)
        let t = r"
        {
            let x = 3;
            x
        }
        y+2
        ";
        test_parse(t, "{ let x = 3;x };(y+2)");

        let t = r"
        let y = 10;
        {
            let x = 3;
            x
        };
        y+2;
        ";
        test_parse(t, "let y = 10;{ let x = 3;x };(y+2);");
    }
}
