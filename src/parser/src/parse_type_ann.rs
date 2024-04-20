use crate::ParseError;
use crate::Parser;
use crate::Type;
use lexer::Token;

impl<'inp> Parser<'inp> {
    /// Parse and return type annotation. Expect lexer.peek() to be at Colon before call
    // Should only consume tokens belonging to the annotation, starting peek at first token and ending
    // peek at the last token of the annotation
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
            Token::Ident(id) => Type::from_string(&id),
            Token::OpenParen => {
                self.advance();
                if let Some(Ok(Token::CloseParen)) = self.lexer.peek() {
                    Ok(Type::Unit)
                } else {
                    Err(ParseError::new("Expected '()' for unit type annotation"))
                }
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
}
