use crate::Decl;
use crate::Decl::*;
use crate::LetStmtData;
use crate::ParseError;
use crate::Parser;
use crate::Type;
use lexer::Token;

impl<'inp> Parser<'inp> {
    // Parse let statement
    // let x = 2;
    pub(crate) fn parse_let(&mut self) -> Result<Decl, ParseError> {
        crate::expect_token_body!(self.lexer.peek(), Ident, "identifier")?;
        let ident = Parser::string_from_ident(self.lexer.peek());
        self.advance();

        let mut type_ann: Option<Type> = None;

        // Do nothing if not colon: allow no annotation to let prev tests pass (for now)
        if self.is_peek_token_type(Token::Colon) {
            // Parse type annotation if any
            self.advance(); // put colon in advance so at type_ann first tok = first token for type

            let ty = self.parse_type_annotation()?;
            type_ann.replace(ty);

            // call advance so peek is at equals
            // self.advance();
        }

        self.consume_token_type(Token::Eq, "Expected '='")?;

        self.advance(); // store the start tok of the next expr as prev_tok

        // ensure we are assigning to an expression
        let expr = self.parse_decl()?.to_expr()?;

        self.expect_token_type(Token::Semi, "Expected semicolon after let")?;

        let stmt = LetStmtData {
            ident,
            expr,
            type_ann,
        };

        Ok(LetStmt(stmt))
    }
}

#[cfg(test)]
pub mod tests {
    use crate::tests::*;
    #[test]
    fn test_parse_let() {
        test_parse("let x = 2;", "let x = 2;");
        test_parse("let x = 2; let y = 3;", "let x = 2;let y = 3;"); // both treated as decls
        test_parse("let x = 2; let y = 3; 30;", "let x = 2;let y = 3;30;"); // 30 is decl
        test_parse("let x = 2; let y = 3; 30", "let x = 2;let y = 3;30"); // 30 is expr

        test_parse(
            "let x = 2; let y = 3; 30; 40; 50",
            "let x = 2;let y = 3;30;40;50",
        );
        test_parse(
            "let x = 2; let y = 3; 30; 40; 50; let z = 60;",
            "let x = 2;let y = 3;30;40;50;let z = 60;",
        );

        test_parse(
            "let x = 2; let y = 3; 30; 40; 50; let z = 60; true",
            "let x = 2;let y = 3;30;40;50;let z = 60;true",
        );

        // // other types
        test_parse(
            "let x = true; let y = 200; let z = 2.2; 3.14159",
            "let x = true;let y = 200;let z = 2.2;3.14159",
        );

        // Identifiers
        test_parse("let x = 20; let y = x; y", "let x = 20;let y = x;y");
        test_parse(
            "let x = 20; let y = x; let z = x + y * 2;",
            "let x = 20;let y = x;let z = (x+(y*2));",
        );
    }

    #[test]
    fn test_parse_let_err() {
        test_parse_err("let", "Expected identifier", true);
        test_parse_err("let 2 = 3", "Expected identifier", true);
        test_parse_err("let x 2", "Expected '='", true);
        test_parse_err("let x = 2", "Expected semicolon", true);
        test_parse_err("let x = let y = 3;", "not an expression", true);
        test_parse_err(";", "Unexpected token", true);
        test_parse_err("=", "Unexpected token", true);
    }

    #[test]
    fn test_parse_let_type() {
        test_parse("let x : int = 2;", "let x : int = 2;");
        test_parse("let x : bool = true;", "let x : bool = true;");
        test_parse("let x : float = 3.25;", "let x : float = 3.25;");

        // Doesn't check types yet - just a parser
        test_parse("let x : int = true;", "let x : int = true;");
        test_parse("let x : bool = 2.3;", "let x : bool = 2.3;");
        test_parse("let x : float = 5;", "let x : float = 5;");

        // basic err cases
        test_parse_err("let x : u32 = true;", "Unknown primitive type", true);
        test_parse_err("let x : = true;", "Expected identifier", true);
    }

    #[test]
    fn test_parse_let_type_many() {
        test_parse(
            "let x : int = 2; let y : bool = true; let z : float = 2.33;",
            "let x : int = 2;let y : bool = true;let z : float = 2.33;",
        );
        test_parse(
            "let x : int = 2; let y : bool = true; let z : float = 2.33; x",
            "let x : int = 2;let y : bool = true;let z : float = 2.33;x",
        );
        // Not affected by parens, ops etc
        test_parse(
            "let x : int = (2 * 3 + 4 - (5 + 6)); let y : bool = !!(true);",
            "let x : int = (((2*3)+4)-(5+6));let y : bool = (!(!true));",
        );
    }
}
