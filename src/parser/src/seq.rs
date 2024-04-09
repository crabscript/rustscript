use crate::BlockSeq;
use crate::Decl;
use crate::Expr;
use crate::ParseError;
use crate::Parser;
use lexer::Token;
use std::rc::Rc;

impl<'inp> Parser<'inp> {
    pub fn parse_seq(&mut self) -> Result<BlockSeq, ParseError> {
        let mut decls: Vec<Decl> = vec![];
        let mut last_expr: Option<Expr> = None;

        while self.lexer.peek().is_some() {
            // parsing a block: break so parse_blk can consume CloseBrace
            if self.is_peek_token_type(Token::CloseBrace) {
                break;
            }

            self.advance();
            // dbg!("prev_tok:", &self.prev_tok);

            let expr = self.parse_decl()?;
            dbg!("Got expr:", &expr);
            // dbg!("Peek:", &self.lexer.peek());

            // end of block: lexer empty OR curly brace (TODO add curly later)
            if self.lexer.peek().is_none() || self.is_peek_token_type(Token::CloseBrace) {
                last_expr.replace(expr.to_expr()?);
                break;
            }
            // semicolon: parse as stmt
            // let semi = expect_token_body!(Semi, "semicolon");
            // TODO: handle block as expr stmt here - block as last expr was already handled above and we break
            else if self.is_peek_token_type(Token::Semi) {
                decls.push(expr);
                self.advance();
                // dbg!("Peek after semi:", &self.lexer.peek());
            }
            // check if expr is a block-like expression (if so, treat as statement)
            // if it was the tail expr it should be handled at the first branch
            else if self
                .prev_tok
                .as_ref()
                .map(|tok| tok.eq(&Token::CloseBrace))
                .unwrap_or(false)
            {
                decls.push(expr);
            }
            // Syntax error
            else {
                dbg!("prev_tok:", &self.prev_tok);
                let is_brace = &self
                    .prev_tok
                    .as_ref()
                    .map(|tok| tok.eq(&Token::CloseBrace))
                    .unwrap_or(false);
                dbg!("prev_tok is close brace:", is_brace);
                dbg!("peek:", &self.lexer.peek());
                return Err(ParseError::new("Expected semicolon"));
            }
        }
        // dbg!(&last_expr, &decls);
        Ok(BlockSeq {
            decls,
            last_expr: last_expr.map(Rc::new),
        })
    }
}
