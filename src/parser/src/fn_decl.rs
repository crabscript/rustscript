use crate::Decl;
use crate::FnDeclData;
use crate::FnParam;
use crate::ParseError;
use crate::Parser;
use crate::Type;
use lexer::Token;

// FnDecl is only statement, not expression
// return stmt is only allowed inside a function
impl<'inp> Parser<'inp> {
    pub(crate) fn parse_fn_decl(&mut self) -> Result<Decl, ParseError> {
        // Get name
        crate::expect_token_body!(self.lexer.peek(), Ident, "identifier")?;
        let fn_name = Parser::string_from_ident(self.lexer.peek());
        self.advance();

        self.consume_token_type(
            Token::OpenParen,
            &format!("Expected {} for function parameters", Token::OpenBrace),
        )?;

        // dbg!("After paren, peek:", &self.lexer.peek());

        let params: Vec<FnParam> = vec![];

        // Parse params
        while let Some(tok) = self.lexer.peek() {
            let tok = tok.clone();
            // stop at )
            if tok.clone().unwrap().eq(&Token::CloseParen) {
                break;
            }

            // param name
            dbg!("peek:", &self.lexer.peek());

            let param_name = Parser::string_from_ident(self.lexer.peek());
            let param_ty: Option<Type> = None;

            // if self.is_peek_token_type(Token::Colon) {
            //     // Parse type annotation if any
            //     self.advance(); // put colon in advance so at type_ann first tok = first token for type
            //     let ty = self.parse_type_annotation()?;
            //     param_ty.replace(ty);
            // }

            dbg!("Param: ", &param_name, &param_ty);

            self.advance(); // put next tok into prev_tok so parse_expr can use it
        }

        self.advance(); // skip past close paren, peek is at OpenBRace

        // Parse return type

        // Parse body
        self.consume_token_type(
            Token::OpenBrace,
            &format!("Expected {} for function body", Token::OpenBrace),
        )?;
        dbg!("Got open brace:", &self.lexer.peek());

        let body = self.parse_blk()?.to_block()?;
        dbg!("Got body", &body);

        let fn_decl = FnDeclData {
            params,
            name: fn_name,
            ret_type: None,
            body,
        };

        Ok(Decl::FnDeclStmt(fn_decl))
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::test_parse;

    #[test]
    fn test_parse_fn_decl() {
        let t = r"
        fn f() {
            
        }
        ";
        test_parse(t, "fn f () {  };");

        let t = r"
        fn f() {
            let x = 2;
        }
        ";
        test_parse(t, "fn f () { let x = 2; };");

        let t = r"
        fn f(x) {
            let y = 2;
        }
        ";
        // test_parse(t, "");

        // let t = r"
        // fn f() {
        //     return 3;
        // }
        // ";
    }
}
