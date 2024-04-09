use crate::BlockSeq;
use crate::Decl;
use crate::Expr;
use crate::ParseError;
use crate::Parser;
use lexer::Token;
use std::rc::Rc;

impl<'inp> Parser<'inp> {
    pub(crate) fn parse_seq(&mut self) -> Result<BlockSeq, ParseError> {
        let mut decls: Vec<Decl> = vec![];
        let mut symbols: Vec<String> = vec![];
        let mut last_expr: Option<Expr> = None;

        while self.lexer.peek().is_some() {
            // parsing a block: break so parse_blk can consume CloseBrace
            if self.is_peek_token_type(Token::CloseBrace) {
                break;
            }

            self.advance();
            // dbg!("prev_tok:", &self.prev_tok);

            let expr = self.parse_decl()?;
            // dbg!("Got expr:", &expr);
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
                // parse_let doesn't consume the semicolon but does check peek for Semi, so we will definitely run this if expr was let
                if let Decl::LetStmt(ref stmt) = expr {
                    symbols.push(stmt.ident.to_owned());
                }

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
                return Err(ParseError::new("Expected semicolon"));
            }
        }
        // dbg!(&last_expr, &decls);
        Ok(BlockSeq {
            decls,
            last_expr: last_expr.map(Rc::new),
            symbols,
        })
    }
}
