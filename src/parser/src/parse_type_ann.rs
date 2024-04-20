use crate::FnTypeData;
use crate::ParseError;
use crate::Parser;
use crate::Type;
use lexer::Token;

impl<'inp> Parser<'inp> {
    /// Parse and return type annotation. Expect lexer.peek() to be at Colon before call
    // Should only consume tokens belonging to the annotation, starting peek at first token and ending
    // peek at token AFTER the last token of type annotation
    pub(crate) fn parse_type_annotation(&mut self) -> Result<Type, ParseError> {
        // self.consume_token_type(Token::Colon, "Expected a colon")?;
        // expect_token_body!(self.lexer.peek(), Ident, "identifier")?;
        Parser::expect_token_for_type_ann(self.lexer.peek())?;

        // if ident, get the string and try to convert type. else, handle specially
        let peek = self
            .lexer
            .peek()
            .unwrap()
            .to_owned()
            .expect("Lexer should not fail"); // would have erred earlier

        let type_ann = match peek {
            Token::Ident(id) => {
                let res = Type::from_string(&id);
                self.advance();
                res
            }
            Token::OpenParen => {
                self.advance();
                if let Some(Ok(Token::CloseParen)) = self.lexer.peek() {
                    self.advance();
                    Ok(Type::Unit)
                } else {
                    Err(ParseError::new("Expected '()' for unit type annotation"))
                }
            }
            Token::Fn => {
                self.advance(); // go past fn
                self.consume_token_type(
                    Token::OpenParen,
                    "Expected '(' for function type annotation",
                )?; // go past (

                let mut param_types: Vec<Type> = vec![];
                let mut ret_ty = Type::Unit;

                // Parse param types
                while let Some(tok) = self.lexer.peek() {
                    let tok = tok.clone();
                    // stop at )
                    if tok.clone().unwrap().eq(&Token::CloseParen) {
                        break;
                    }

                    let param_ty = self.parse_type_annotation()?;
                    param_types.push(param_ty);
                    // self.advance(); // go past token of last ty_an

                    // Comma or CloseParen
                    if !self.lexer.peek().eq(&Some(&Ok(Token::CloseParen))) {
                        self.consume_token_type(
                            Token::Comma,
                            "Expected ',' to separate function parameters",
                        )?;
                    }
                }

                dbg!("PEEK AFTER LOOP:", &self.lexer.peek());

                self.advance(); // skip past open paren, peek is at return arrow or equals

                if self.consume_opt_token_type(Token::FnDeclReturn) {
                    // peek is now at type_ann first token
                    let ret_ty_ann = self.parse_type_annotation()?;
                    // self.advance(); // go past last token of ty_ann
                    dbg!(&ret_ty_ann);
                    ret_ty = ret_ty_ann;
                }

                dbg!("PEEK AFTER:", &self.lexer.peek());

                let fn_ty_data = FnTypeData {
                    params: param_types,
                    ret_type: ret_ty,
                };

                Ok(Type::UserFn(Box::new(fn_ty_data)))
            }
            _ => unreachable!(),
        }?;

        Ok(type_ann)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{test_parse, test_parse_err};

    #[test]
    fn test_parse_type_annotations() {
        test_parse("let x : int = 2;", "let x : int = 2;");
        test_parse("let x : bool = true;", "let x : bool = true;");
        test_parse("let x : float = true;", "let x : float = true;");
        test_parse("let x : () = true;", "let x : () = true;");
    }

    #[test]
    fn test_parse_type_annotations_errs() {
        // test_parse("let x : int = 2;", "");
        test_parse_err(
            "let x : let ",
            "Expected identifier or '(' for type annotation, got 'let'",
            true,
        );
        test_parse_err(
            "let x : 2 ",
            "Expected identifier or '(' for type annotation, got '2'",
            true,
        );
        test_parse_err(
            "let x : ",
            "Expected identifier or '(' for type annotation, got end of input",
            true,
        );
        test_parse_err(
            "let x : (2 ",
            "Expected '()' for unit type annotation",
            true,
        );
    }

    #[test]
    fn test_parse_type_annotations_more() {
        // hof, semaphore, string

        // // empty
        let t = r"
        let g : fn() = f;
        ";
        test_parse(t, "let g : fn() = f;");

        // // has params
        let t = r"
        let g : fn(int) = f;
        ";
        test_parse(t, "let g : fn(int) = f;");

        let t = r"
        let g : fn(int, bool) = f;
        ";
        test_parse(t, "let g : fn(int, bool) = f;");

        // // // param is fn
        let t = r"
        let g : fn(int, fn(int)) = f;
        ";
        test_parse(t, "let g : fn(int, fn(int)) = f;");

        // ret type
        let t = r"
        let g : fn(int) -> bool = f;
        ";
        test_parse(t, "let g : fn(int) -> bool = f;");

        let t = r"
        let g : fn(int, float) -> bool = f;
        ";
        test_parse(t, "let g : fn(int, float) -> bool = f;");

        // returns fn
        let t = r"
        let g : fn(int, bool) -> fn(int) -> int = f;
        ";
        test_parse(t, "let g : fn(int, bool) -> fn(int) -> int = f;");
    }
}
