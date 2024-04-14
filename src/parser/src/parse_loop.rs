use lexer::Token;

use crate::Decl;
use crate::Expr;
use crate::LoopData;
use crate::ParseError;
use crate::Parser;

// Loops are only statements, not expressions
/*
// inf
loop {
   2;
   3;
   4
}

let x = 0;
loop x < 3 {
    x = x + 1;
}

// break
let x = 0;
loop x < 5 {
    x = x + 1;

    if x == 2 {
        break;
    }
}
*/
impl<'inp> Parser<'inp> {
    /*

    */
    pub(crate) fn parse_loop(&mut self) -> Result<Decl, ParseError> {
        // If token not consumed (no open paren), advance so first token of expr goes into prev_tok
        // allows loop (x < 3) - condition in brackets
        if !self.consume_opt_token_type(Token::OpenParen) {
            self.advance();
        }

        // dbg!("prev_tok after loop:", &self.prev_tok);

        let cond = self.parse_expr(0)?.to_expr()?;

        // Differentiate condition is a blk vs the thing we parsed as cond is actually the body and there is no cond
        if let Expr::BlockExpr(ref blk) = cond {
            // dbg!("peek after parsing blk:", &self.lexer.peek());
            // next token is NOT OpenBrace: we just parsed body, there is no condition
            if !matches!(&self.lexer.peek(), Some(Ok(Token::OpenBrace))) {
                let lp = LoopData {
                    cond: None,
                    body: blk.to_owned(),
                };

                return Ok(Decl::LoopStmt(lp));
            }
        }

        // go past OpenBrace, put in prev_tok
        self.consume_token_type(
            Token::OpenBrace,
            &format!("Expected {} for loop block", Token::OpenBrace),
        )?;

        let loop_blk = self.parse_blk()?.to_block()?;

        // Ok(Decl::ExprStmt(Expr::Bool(true)))
        let lp = LoopData {
            cond: Some(cond),
            body: loop_blk,
        };

        Ok(Decl::LoopStmt(lp))
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{test_parse, test_parse_err};

    #[test]
    fn test_parse_loop_nocond() {
        // always stmt
        let t = r"
        loop { 
            2; 
        }
        ";
        test_parse(t, "loop  { 2; };");

        let t = r"
        loop { 
            2;
            if x == 3 {
                5;
            }
            100
        }
        3;
        ";
        test_parse(t, "loop  { 2;if (x==3) { 5; };100 };3;");

        // can't assign as expr
        let t = "
        let x = loop {

        };
        ";
        test_parse_err(t, "loop is not an expression", true);
    }

    #[test]
    fn test_parse_loop_with_cond() {
        // cond not a blk
        let t = r"
        loop x < 5 { 
            3; 
            4
        }
        ";
        test_parse(t, "loop (x<5) { 3;4 };");

        let t = r"
        loop {2; false} { 
            3; 
            4
        }
        3;
        ";
        test_parse(t, "loop { 2;false } { 3;4 };3;");

        // can parse any expression as cond
        let t = r"
        loop if x && y { false } else { true } {
            2;
            3
        }
        ";
        test_parse(t, "loop if (x&&y) { false } else { true } { 2;3 };");

        // can't use just the loop as cond
        let t = r"
        loop loop {} {

        }
        ";
        test_parse_err(t, "not an expression: 'loop'", true);
    }

    #[test]
    fn test_parse_loop_multiple() {
        let t = r"
        loop {
            200;
        }

        let x = 0;
        loop x < 5 {
            x = x + 1;
        }
        ";
        test_parse(t, "loop  { 200; };let x = 0;loop (x<5) { x = (x+1); };");
    }

    #[test]
    fn test_parse_loop_nested() {
        let t = r"
        loop {
            loop {

            }
        }
        ";
        test_parse(t, "loop  { loop  {  }; };");

        let t = r"
        let i = 0;
        loop i < 5 {
            let j = 0;
            loop {
                j = j+ 1;
            }

            i = i + 1;
        }
        ";
        test_parse(
            t,
            "let i = 0;loop (i<5) { let j = 0;loop  { j = (j+1); };i = (i+1); };",
        );
    }

    #[test]
    fn test_parse_loop_break() {
        let t = r"
        loop {
            break;
        }
        ";
        // test_parse(t, "");
    }
}
