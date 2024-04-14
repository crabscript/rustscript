use crate::BlockSeq;
use crate::Decl;
// use crate::Decl::*;
use crate::Expr;
use crate::IfElseData;
use crate::ParseError;
use crate::Parser;
// use crate::{BinOpType, UnOpType};
use lexer::Token;

impl<'inp> Parser<'inp> {
    pub(crate) fn parse_if_else(&mut self, min_bp: u8) -> Result<Decl, ParseError> {
        // condition - in parens
        // self.consume_token_type(Token::OpenParen, "Expected open parenthesis")?;

        // If token not consumed (no open paren), advance so first token of expr goes into prev_tok
        if !self.consume_opt_token_type(Token::OpenParen) {
            self.advance();
        }

        let cond = self.parse_expr(min_bp)?.to_expr()?;

        // go past OpenBrace, put in prev_tok
        self.consume_token_type(
            Token::OpenBrace,
            &format!("Expected {} for if block", Token::OpenBrace),
        )?;

        let if_blk = self.parse_blk(min_bp)?.to_block()?;

        // check else
        let mut else_blk: Option<BlockSeq> = None;

        if self.expect_token_type(Token::Else, "").is_ok() {
            self.consume_token_type(Token::Else, "Expected 'else' for if")?;
            self.consume_token_type(
                Token::OpenBrace,
                &format!("Expected {} for else block", Token::OpenBrace),
            )?;

            let blk = self.parse_blk(min_bp)?.to_block()?;

            else_blk.replace(blk);
        }

        let has_else = else_blk.is_some();

        let stmt = IfElseData {
            cond,
            if_blk,
            else_blk,
        };

        if has_else {
            let exp = Expr::IfElseExpr(Box::new(stmt));
            let decl = Decl::ExprStmt(exp);
            Ok(decl)
        } else {
            Ok(Decl::IfOnlyStmt(stmt))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::*;

    #[test]
    fn test_parse_if_basic() {
        let t = r"
        if (true) {
            30;
            40;
        }
        ";
        test_parse(t, "if true { 30;40; };");

        // with semi is same
        let t = r"
        if (true) {
            30;
            40;
        };
        ";
        test_parse(t, "if true { 30;40; };");

        // if-else expr last
        let t = r"
        if (true) {
            30;
        } else {
            40;
            50
        }
        ";
        test_parse(t, "if true { 30; } else { 40;50 }");

        // above with semi becomes decl
        let t = r"
        if (true) {
            30;
        } else {
            40;
            50
        };
        ";
        test_parse(t, "if true { 30; } else { 40;50 };");

        // without brackets for cond
        let t = r"
        if true {
            30;
            40;
        }
        ";
        test_parse(t, "if true { 30;40; };");

        let t = r"
        if (!true) {
            30;
            40;
        }
        ";
        test_parse(t, "if (!true) { 30;40; };");
    }

    #[test]
    fn test_parse_if_consec() {
        // if-only becomes stmt (2stmts, no last expr)
        let t = r"
        if true {
            30;
        }

        if !true {
            40;
        }
        ";

        test_parse(t, "if true { 30; };if (!true) { 40; };");

        // with semicolon - same as above
        let t = r"
        if true {
            30;
        }

        if !true {
            40;
        };
        ";

        test_parse(t, "if true { 30; };if (!true) { 40; };");

        // if, if-else
        let t = r"
        if true {
            30;
        }

        if some_cond {
            40; 60
        } else {
            50; 70;
        }
        ";

        test_parse(t, "if true { 30; };if some_cond { 40;60 } else { 50;70; }");

        // // if, if-else (stmt)
        let t = r"
        if true {
            30;
        }

        if some_cond {
            40; 60
        } else {
            50; 70;
        };
        ";

        test_parse(t, "if true { 30; };if some_cond { 40;60 } else { 50;70; };");

        // if-else, if
        let t = r"
        if true {
            30;
        } else {
            x
        }

        if some_cond {
            40; 60
        }
        ";

        test_parse(t, "if true { 30; } else { x };if some_cond { 40;60 };");

        // if-else, if-else
        let t = r"
        if true {
            30;
        } else {
            x
        }

        if some_cond {
            40; 60
        } else {
            y;
        }
        ";

        test_parse(
            t,
            "if true { 30; } else { x };if some_cond { 40;60 } else { y; }",
        );

        let t = r"
        if true {
            x;
        }

        if y {
            200;
        } else {
            300;
        }

        if false {
            400;
        }
        ";

        test_parse(
            t,
            "if true { x; };if y { 200; } else { 300; };if false { 400; };",
        );
    }

    #[test]
    fn test_parse_if_expr() {
        // can parse but fails type check - if blk produces Unit, else blk produces 3
        let t = r"
        let x = if true { 2; } else { 3 };
        ";
        test_parse(t, "let x = if true { 2; } else { 3 };");

        // if-only can't be expr
        let t = r"
        let x = if true { 2; };
        ";
        test_parse_err(t, "if without else branch is not an expression", true);

        // nested in blk
        let t = r"
        let x = {
            if false {
                20;
            }

            if true {
                2
            } else {
                3
            }
        };
        ";
        test_parse(t, "let x = { if false { 20; };if true { 2 } else { 3 } };");
    }
}
