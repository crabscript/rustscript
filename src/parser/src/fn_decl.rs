use std::collections::HashSet;

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

        let mut params: Vec<FnParam> = vec![];
        // to prevent duplicate params e.g f(x,x). HashSet doesn't preserve order so I need a separate one
        let mut seen_ident: HashSet<String> = HashSet::new();

        // Parse params
        while let Some(tok) = self.lexer.peek() {
            let tok = tok.clone();
            // stop at )
            if tok.clone().unwrap().eq(&Token::CloseParen) {
                break;
            }

            // param name
            dbg!("peek:", &self.lexer.peek());

            // Invariant: at start peek is a param identifier
            let param_name = Parser::string_from_ident(self.lexer.peek());
            let mut param_ty: Option<Type> = None;

            self.advance(); // go past ident

            if self.is_peek_token_type(Token::Colon) {
                // Parse type annotation if any
                self.advance(); // put colon in advance so at type_ann first tok = first token for type
                let ty = self.parse_type_annotation()?;
                param_ty.replace(ty);

                // to go past last token of type_ann, so peek is at comma or close paren
                dbg!("AFTER TY_ANN:", &self.lexer.peek());
                self.advance();
            }

            dbg!("Peek here:", &self.lexer.peek());
            dbg!("Param: ", &param_name, &param_ty);

            // Comma or CloseParen
            if !self.lexer.peek().eq(&Some(&Ok(Token::CloseParen))) {
                self.consume_token_type(
                    Token::Comma,
                    "Expected ',' to separate function arguments",
                )?;
            }

            if seen_ident.contains(&param_name) {
                let e = format!(
                    "Parameter '{}' bound more than once for function {}",
                    param_name, fn_name
                );
                return Err(ParseError::new(&e));
            }

            seen_ident.insert(param_name.clone());

            params.push(FnParam {
                name: param_name,
                type_ann: param_ty,
            })
        }

        self.advance(); // skip past close paren, peek is at OpenBrace or ret type first token

        let mut ret_ty = Type::Unit;
        // Parse return type: expect -> first
        dbg!("PEEK AT PARSE RET:", &self.lexer.peek());
        // if its there parse ret type, else keep it as Unit
        if self.consume_opt_token_type(Token::FnDeclReturn) {
            // peek is now at type_ann first token
            let ret_ty_ann = self.parse_type_annotation()?;
            self.advance(); // go past last token of ty_ann

            ret_ty = ret_ty_ann;
        }

        // self.consume_opt_token_type(token)
        // if self.lexer.peek().eq(&Some(&Ok(Token::FnDeclReturn))) {
        //     self.advance();

        // }

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
            ret_type: ret_ty,
            body,
        };

        Ok(Decl::FnDeclStmt(fn_decl))
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{test_parse, test_parse_err};

    #[test]
    fn test_parse_fn_decl_basic() {
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
        fn f(x: int) {
            let y = 2;
        }
        ";
        test_parse(t, "fn f (x:int) { let y = 2; };");

        let t = r"
        fn f(x: int, y : bool) {
            let y = 2;
        }
        ";
        test_parse(t, "fn f (x:int, y:bool) { let y = 2; };");

        let t = r"
        fn f(x,y) {
            let y = 2;
        }
        ";
        test_parse(t, "fn f (x, y) { let y = 2; };");

        let t = r"
        fn f(x,y: int, z: bool, g) {
            let y = 2;
        }
        ";
        test_parse(t, "fn f (x, y:int, z:bool, g) { let y = 2; };");
    }

    #[test]
    fn test_parse_fn_decl_with_retype() {
        let t = r"
        fn f() -> int {
            2
        }
        ";
        test_parse(t, "fn f () -> int { 2 };");

        let t = r"
        fn f(x: bool, y: int) -> int {
            2
        }
        ";
        test_parse(t, "fn f (x:bool, y:int) -> int { 2 };");

        // many
        let t = r"
        let x = 20;

        fn f(x: int) -> bool {
            true
        }

        fn g(y : bool) -> float {
            2.56
        }

        200
        ";
        test_parse(
            t,
            "let x = 20;fn f (x:int) -> bool { true };fn g (y:bool) -> float { 2.56 };200",
        );
    }

    #[test]
    fn test_parse_fn_decl_return() {
        // let t = r"
        // fn f() {
        //     return 3;
        // }
        // ";
    }

    #[test]
    fn test_parse_fn_decl_edges() {
        // can parse before/after
        let t = r"
        300;

        fn hi() {

        }

        200
        ";
        test_parse(t, "300;fn hi () {  };200");

        // multiple fns
        let t = r"
        fn g(x: int) {
            let x = 2;
            loop x < 5 {
                x = x + 1;
                break;
            }
        }

        fn f(x: bool) {
            let x = 2;
        }
        ";
        test_parse(t, "fn g (x:int) { let x = 2;loop (x<5) { x = (x+1);break; }; };fn f (x:bool) { let x = 2; };");

        // arg clash - can throw at parser
        let t = r"
        fn f(x : int, x : bool) {

        }
        ";
        test_parse_err(t, "'x' bound more than once for function f", true)
    }
}
