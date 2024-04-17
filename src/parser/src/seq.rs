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

            // Include function names in list of symbols to be used for ENTERSCOPE
            if let Decl::FnDeclStmt(ref data) = expr {
                symbols.push(data.name.to_owned());
            }

            // if ends with semicolon: statement, advance past semi
            if self.is_peek_token_type(Token::Semi) {
                // parse_let doesn't consume the semicolon but does check peek for Semi, so we will definitely run this if expr was let

                // push declared symbols from let or fn declarations so that they can be put in ENTERSCOPE
                if let Decl::LetStmt(ref stmt) = expr {
                    symbols.push(stmt.ident.to_owned());
                }

                decls.push(expr);

                self.advance();
                continue;
                // dbg!("Peek after semi:", &self.lexer.peek());
            } else if self.lexer.peek().is_none() || self.is_peek_token_type(Token::CloseBrace) {
                // reached end of block / program: treat as last_expr, UNLESS it can't be converted to expr
                // e.g: if with no else, fn decl - these are handled in the next branch (which also handles them when not at last)
                let to_expr = expr.to_expr();
                if to_expr.is_ok() {
                    last_expr.replace(to_expr?);
                    break;
                }
            }

            // check if expr is a block-like expression AND we are in the middle, we know because
            // prev branch failed. if so, add as decl.
            if self
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
