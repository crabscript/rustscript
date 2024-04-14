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
    // Ensure is_loop flag is saved and restored as long as valid return. Error crashes the whole parser so it's fine
    pub(crate) fn parse_loop(&mut self) -> Result<Decl, ParseError> {
        let prev_is_loop = self.is_loop;
        let lp = self.parse_loop_inner()?;
        self.is_loop = prev_is_loop;
        Ok(lp)
    }

    fn parse_loop_inner(&mut self) -> Result<Decl, ParseError> {
        // If token not consumed (no open paren), advance so first token of expr goes into prev_tok
        // allows loop (x < 3) - condition in brackets
        if !self.consume_opt_token_type(Token::OpenParen) {
            self.advance();
        }

        // dbg!("prev_tok after loop:", &self.prev_tok);

        // save and restore prev is_loop e.g when coming from global
        let prev_is_loop = self.is_loop;
        dbg!("prev is loop:", prev_is_loop);
        self.is_loop = true;

        let cond = self.parse_expr(0)?.to_expr()?;

        // If the thing we parsed is a block, this is a loop with just a body and no cond
        if let Expr::BlockExpr(ref blk) = cond {
            // dbg!("peek after parsing blk:", &self.lexer.peek());
            // next token is NOT OpenBrace: we just parsed body, there is no condition
            let lp = LoopData {
                cond: None,
                body: blk.to_owned(),
            };

            return Ok(Decl::LoopStmt(lp));
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

        self.is_loop = prev_is_loop;

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

        // if a loop has blk after it, treated as loop body. blk as condition is not allowed
        let t = r"
        loop { 
            2; false
        } 
        
        { 
            3; 
            4
        }
        3;
        ";
        test_parse(t, "loop  { 2;false };{ 3;4 };3;");

        // can parse any expression as cond
        let t = r"
        loop if x && y { false } else { true } {
            2;
            3
        }
        ";
        test_parse(t, "loop if (x&&y) { false } else { true } { 2;3 };");
    }

    #[test]
    fn test_parse_loop_cond_err() {
        // can't use just the loop as cond
        let t = r"
         loop loop {} {
 
         }
         ";
        test_parse_err(t, "not an expression: 'loop'", true);

        let t = "loop x < 5";
        test_parse_err(t, " Expected { for loop block", true);
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
    fn test_parse_loop_break_errs() {
        let t = r"
        loop {
            break
        }
        ";
        test_parse_err(t, "Expected semicolon", true);

        // break not allowed outside  loop
        let t = r"
        break;
        ";
        test_parse_err(t, "break outside of loop", true);

        let t = r"
        {
            break;
        }
        ";
        test_parse_err(t, "break outside of loop", true);

        let t = r"
        if true {
            break;
        }
        ";
        test_parse_err(t, "break outside of loop", true);

        let t = r"
        if true {
            2;
        } else {
            break;
        }
        ";
        test_parse_err(t, "break outside of loop", true);

        let t = r"
        loop {
            let x = 0;
            loop {
                break;
            }
            break;
        }
        {
            loop {
                break;
            }
            break;
        };
        ";
        test_parse_err(t, "break outside of loop", true);
    }

    #[test]
    fn test_parse_break_inloop() {
        let t = r"
        loop {
            break;
        }
        ";
        test_parse(t, "loop  { break; };");

        let t = r"
        loop x < 5 {
            if x == 3 {
                break;
            } else {
                30;
            }
        }
        ";
        test_parse(t, "loop (x<5) { if (x==3) { break; } else { 30; } };");

        let t = r"
        loop {
            break;
        }
        break;
        ";
        test_parse_err(t, "break outside of loop", true);

        // nested
        let t = r"
        loop {
            let x = 0;
            loop {
                break;
            }
            break;
        }
        ";
        test_parse(t, "loop  { let x = 0;loop  { break; };break; };");
    }
}
