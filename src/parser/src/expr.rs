use crate::Decl;
use crate::Decl::*;
use crate::Expr;
use crate::ParseError;
use crate::Parser;
use crate::{BinOpType, UnOpType};
use lexer::Token;

impl<'inp> Parser<'inp> {
    // Parses and returns an expression (something that is definitely an expression)
    // Return as Decl for consistency
    // Invariant: prev_tok should contain the start of the expr before call
    pub(crate) fn parse_expr(&mut self, min_bp: u8) -> Result<Decl, ParseError> {
        let prev_tok = self.expect_prev_tok()?;
        let mut lhs = match prev_tok {
            Token::OpenParen => {
                self.advance();
                let lhs = self.parse_expr(0)?;
                self.consume_token_type(Token::CloseParen, "Expected closing parenthesis")?;
                Ok(lhs)
            }
            Token::Integer(val) => Ok(ExprStmt(Expr::Integer(*val))),
            Token::Float(val) => Ok(ExprStmt(Expr::Float(*val))),
            Token::Bool(val) => Ok(ExprStmt(Expr::Bool(*val))),
            // Unary
            Token::Minus => {
                let ((), r_bp) = Parser::get_prefix_bp(&UnOpType::Negate);
                self.advance();
                let rhs = self.parse_expr(r_bp)?;
                let res = Expr::UnOpExpr(UnOpType::Negate, Box::new(rhs.to_expr()?));
                Ok(ExprStmt(res))
            }
            Token::Bang => {
                let ((), r_bp) = Parser::get_prefix_bp(&UnOpType::Not);
                self.advance();
                let rhs = self.parse_expr(r_bp)?;
                let res = Expr::UnOpExpr(UnOpType::Not, Box::new(rhs.to_expr()?));
                Ok(ExprStmt(res))
            }
            Token::Ident(id) => {
                // Three cases: id, id = ..., id() => load var, assignment, func call
                // Handle just id first
                // dbg!(&self.lexer.peek());
                self.parse_ident(id.to_string(), min_bp)
            }
            Token::OpenBrace => self.parse_blk(min_bp),
            Token::If => self.parse_if_else(min_bp),
            _ => Err(ParseError::new(&format!(
                "Unexpected token - not an expression: '{}'",
                prev_tok
            ))),
        }?;

        // dbg!("LHS:", &lhs);
        loop {
            if self.lexer.peek().is_none()
                || self.is_peek_token_type(Token::Semi)
                || self.is_peek_token_type(Token::CloseBrace)
                || self.is_peek_token_type(Token::CloseParen)
                // to deal with if and bracket
                || self.is_peek_token_type(Token::OpenBrace)
            {
                break;
            }

            let tok = self
                .lexer
                .peek()
                .expect("Should have token")
                .clone()
                .expect("Lexer should not fail");

            // dbg!("Prev_tok before from_token:", &self.prev_tok);
            let binop = BinOpType::from_token(&tok);

            if let (&Some(Token::CloseBrace), &Err(_)) = (&self.prev_tok, &binop) {
                break;
            }

            let binop = binop?;

            let (l_bp, r_bp) = Parser::get_infix_bp(&binop);
            // self.advance();
            if l_bp < min_bp {
                break;
            }

            // only advance after the break
            // before adv: peek is at infix op
            // after adv: peek crosses infix op, then reaches the next infix op and prev_tok = next atom
            // e.g 2+3*4: before adv peek is at +, after adv peek is at *
            self.advance();
            self.advance();
            let rhs = self.parse_expr(r_bp)?;

            // dbg!(&lhs, &rhs);

            lhs = ExprStmt(Expr::BinOpExpr(
                binop,
                Box::new(lhs.to_expr()?),
                Box::new(rhs.to_expr()?),
            ));
        }

        Ok(lhs)
    }
}
