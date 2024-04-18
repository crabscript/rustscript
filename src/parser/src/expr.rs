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
            Token::String(str) => Ok(ExprStmt(Expr::StringLiteral(str.to_owned()))),
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
            Token::OpenBrace => self.parse_blk(),
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
                // to deal with if and bracket e.g if { .. } else { .. } when it reaches last bracket
                || self.is_peek_token_type(Token::OpenBrace)
                // to deal with comma in func call e.g print(2,3);
                || self.is_peek_token_type(Token::Comma)
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
            // comparison ops have no associativity (this is how Rust works) so left/right prec are same
            if l_bp == min_bp {
                return Err(ParseError::new(
                    "Comparison operators can't be chained. Use parentheses to disambiguate.",
                ));
            }
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

#[cfg(test)]
mod tests {
    use crate::tests::{test_parse, test_parse_err};

    #[test]
    fn test_parse_binop() {
        test_parse("2+3;", "(2+3);");
        test_parse("2*3;", "(2*3);");
        test_parse("2+2*3", "(2+(2*3))");
        test_parse("2*3+4", "((2*3)+4)");
        test_parse("2*3+4/2", "((2*3)+(4/2))");

        test_parse("2*3+4; 2+4*3; 20/200*2", "((2*3)+4);(2+(4*3));((20/200)*2)");

        test_parse("2-3", "(2-3)");
        test_parse("2-3+4/5*6", "((2-3)+((4/5)*6))");
        test_parse("2-3+4/5*6-8+9; 2+2;", "((((2-3)+((4/5)*6))-8)+9);(2+2);");

        test_parse("let x = 2+3*4-5; 300", "let x = ((2+(3*4))-5);300");
    }

    #[test]
    fn test_parse_negation() {
        test_parse("-2;", "(-2);");
        test_parse("-2+3;", "((-2)+3);");
        test_parse("3+-2;", "(3+(-2));");
        test_parse("--2;", "(-(-2));");
        test_parse("---2;", "(-(-(-2)));");
        test_parse("-1*2+3-4", "((((-1)*2)+3)-4)");
        test_parse(
            "let x = -1.23; -1+2*3; 3*-2/5",
            "let x = (-1.23);((-1)+(2*3));((3*(-2))/5)",
        );

        // no type checking yet - leave type checking to one distinct phase
        test_parse("let x = -true+false;", "let x = ((-true)+false);");
    }

    #[test]
    fn test_parse_ident() {
        test_parse("x", "x");
        test_parse("x;", "x;");
        test_parse("x; y;", "x;y;");
        test_parse("x; y; z", "x;y;z");

        test_parse("x; y; x+y*2", "x;y;(x+(y*2))");
        test_parse("x; y; -y+x/3", "x;y;((-y)+(x/3))");
        test_parse("x; y; -y+x/3", "x;y;((-y)+(x/3))");
    }

    #[test]
    fn test_parse_parens() {
        test_parse("(2)", "2");
        test_parse("((((20))));", "20;");
        test_parse("(2+3)", "(2+3)");
        test_parse("(2+3)*4", "((2+3)*4)");
        test_parse("2+3*(4-5)", "(2+(3*(4-5)))");
        test_parse("2+3*(4-(5*6/(7-3)))", "(2+(3*(4-((5*6)/(7-3)))))");
        test_parse(
            "(2*3+(4-(6*5)))*(10-(20)*(3+2))",
            "(((2*3)+(4-(6*5)))*(10-(20*(3+2))))",
        );

        // Err cases
        test_parse_err("((2+3)*5", "closing paren", true);
        test_parse_err("(2*3+(4-(6*5)))*(10-(20)*(3+2)", "closing paren", true);
    }

    #[test]
    fn test_parse_not() {
        test_parse("!true", "(!true)");
        test_parse("!false", "(!false)");
        test_parse("!!true;", "(!(!true));");
        test_parse("!!!true", "(!(!(!true)))");

        // No type check, but we will use same prec for mul as for logical and/or
        test_parse("!2*3", "((!2)*3)");
        test_parse("!(2*3)", "(!(2*3))");
    }

    #[test]
    fn test_parse_comp_ops() {
        // ==, <, >
        test_parse("2 > 3", "(2>3)");
        test_parse_err("2 > 3 > 4", "Comparison operators can't be chained", true);
        test_parse_err(
            "false == 3 > 5",
            "Comparison operators can't be chained",
            true,
        );

        // can chain if brackets provided
        test_parse("(2 > 3) > true", "((2>3)>true)");
        test_parse("false == (3 > 5)", "(false==(3>5))");
        test_parse("(false == 3) > 5", "((false==3)>5)"); // can parse but not well-typed
    }

    #[test]
    fn test_parse_logical_ops() {
        // &&, || - left assoc
        test_parse("x && y && z", "((x&&y)&&z)");
        test_parse("x || y || z", "((x||y)||z)");

        // override
        test_parse("x && (y && z)", "(x&&(y&&z))");
        test_parse("x || (y || z)", "(x||(y||z))");

        // both
        test_parse("x && y || z", "((x&&y)||z)");

        // and is stronger - e.g 2+2*3 => 2+(2*3)
        test_parse("true || false && false", "(true||(false&&false))");
        // but brackets can override precedence - (2+2)*3
        test_parse("(true || false) && false", "((true||false)&&false)");

        // with not
        // becomes (!x && y) || (!z == false)
        test_parse("!x && y || !z == false", "(((!x)&&y)||((!z)==false))");

        // can override
        test_parse("!(x && y) || !z == false", "((!(x&&y))||((!z)==false))");
    }
}
