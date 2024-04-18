use crate::AssignStmtData;
use crate::Decl;
use crate::Expr;
use crate::FnCallData;
use crate::ParseError;
use crate::Parser;
use lexer::Token;

impl<'inp> Parser<'inp> {
    pub fn parse_ident(&mut self, ident: String, min_bp: u8) -> Result<Decl, ParseError> {
        let sym = Expr::Symbol(ident.to_string());

        // Handle assignment, fn call
        if let Some(tok) = self.lexer.peek() {
            let tok = tok.as_ref().expect("Lexer should not fail");

            // Assignment x = 2
            if tok.eq(&Token::Eq) {
                self.consume_token_type(Token::Eq, "Expected '='")?;
                self.advance();

                // now prev_tok has the start of the expr
                let expr = self.parse_expr(min_bp)?.to_expr()?;

                let assign = AssignStmtData { ident, expr };

                return Ok(Decl::AssignStmt(assign));
            } else if tok.eq(&Token::OpenParen) {
                // Fn call
                self.consume_token_type(Token::OpenParen, "Expected '('")?;
                // dbg!("tok after:", &self.lexer.peek());

                let mut args: Vec<Expr> = vec![];

                while let Some(tok) = self.lexer.peek() {
                    let tok = tok.clone();
                    // stop at )
                    if tok.clone().unwrap().eq(&Token::CloseParen) {
                        break;
                    }

                    self.advance(); // put next tok into prev_tok so parse_expr can use it

                    // let expr = self.parse_expr(min_bp)?.to_expr()?;
                    // need to reset min_bp when parsing each expr, shouldnt depend on prev
                    let expr = self.parse_expr(0)?.to_expr()?;

                    // dbg!("Peek after parsing:", &self.lexer.peek(), &expr);

                    args.push(expr);

                    if !self.lexer.peek().eq(&Some(&Ok(Token::CloseParen))) {
                        self.consume_token_type(
                            Token::Comma,
                            "Expected ',' to separate function arguments",
                        )?;
                    }
                }

                self.consume_token_type(Token::CloseParen, "Expected ')'")?;

                let data = FnCallData { name: ident, args };

                let fn_call = Expr::FnCallExpr(data);

                return Ok(Decl::ExprStmt(fn_call));
            }
        }

        Ok(Decl::ExprStmt(sym))
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{test_parse, test_parse_err};

    #[test]
    fn test_parse_fn_call() {
        let t = "print()";
        test_parse(t, "print()");

        let t = "print();";
        test_parse(t, "print();");

        let t = "print(2);";
        test_parse(t, "print(2);");

        let t = "print(2,3)";
        test_parse(t, "print(2,3)");

        let t = "print(2,3);";
        test_parse(t, "print(2,3);");

        let t = "print(2, 3, 4);";
        test_parse(t, "print(2,3,4);");

        let t = "print(2,3,); 2";
        test_parse(t, "print(2,3);2");
    }

    #[test]
    fn test_parse_fn_call_mixed() {
        // in stmts
        let t = "let x = f(2,3);";
        test_parse(t, "let x = f(2,3);");

        let t = "x = f(2,3);";
        test_parse(t, "x = f(2,3);");

        let t = r"
        if true {
            f(2,3,);
        } else {
            g(5,6)
        }
        ";
        test_parse(t, "if true { f(2,3); } else { g(5,6) }");

        let t = r"
        loop {
            print(2);
            f(g,3);
            foo() + bar(6)
        }
        ";
        test_parse(t, "loop  { print(2);f(g,3);(foo()+bar(6)) };");

        let t = r"
        let x : int = {
            let y = 2;
            // 3 arguments
            foo(g(2) + f(6), (bar(7,3) * y) + func(7), func(8))
        };
        ";
        test_parse(
            t,
            "let x : int = { let y = 2;foo((g(2)+f(6)),((bar(7,3)*y)+func(7)),func(8)) };",
        );
    }

    #[test]
    fn test_parse_fn_call_err() {
        test_parse_err("print(", "Expected ')'", true);
        test_parse_err("print(}", "Unexpected token - not an expression", true);
        test_parse_err("print(,)", "Unexpected token - not an expression", true);
    }
}
