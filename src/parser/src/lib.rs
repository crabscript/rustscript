use lexer::{lex, Token};
use logos::Lexer;
use std::iter::Peekable;
use structs::*;

pub mod blk;
pub mod expr;
pub mod seq;
pub mod structs;

// To expect token types that have a value inside (for Ident and primitives)
macro_rules! expect_token_body {
    ($peek:expr, $token:ident, $expected:expr) => {{
        let err = Err(ParseError::new(concat!("Expected ", $expected)));
        let pk = $peek;

        if pk.is_none() {
            err
        } else {
            let pk = pk
                .expect("Peek has something")
                .as_ref()
                .expect("Expect lexer to succeed");
            match pk {
                Token::$token(_) => Ok(()),
                _ => err,
            }
        }
    }};
}

pub struct Parser<'inp> {
    prev_tok: Option<Token>,
    lexer: Peekable<Lexer<'inp, Token>>,
}

use Decl::*;
impl<'inp> Parser<'inp> {
    pub fn new(lexer: Lexer<'_, Token>) -> Parser<'_> {
        Parser {
            prev_tok: None,
            lexer: lexer.peekable(),
        }
    }

    pub fn new_from_string(inp: &str) -> Parser<'_> {
        Parser {
            prev_tok: None,
            lexer: lex(inp).peekable(),
        }
    }

    // Check if peek is a specific token type
    fn is_peek_token_type(&mut self, token: Token) -> bool {
        let pk = self.lexer.peek();
        if pk.is_none() {
            false
        } else {
            let pk = pk.unwrap();
            match pk {
                Ok(prev) => prev.eq(&token),
                _ => false,
            }
        }
    }

    // To expect token types at peek that have no value (most of them)
    fn expect_token_type(&mut self, token: Token, expected_msg: &str) -> Result<(), ParseError> {
        if !self.is_peek_token_type(token) {
            Err(ParseError::new(expected_msg))
        } else {
            Ok(())
        }
    }

    // Expect token type at peek and advance if it was there
    fn consume_token_type(&mut self, token: Token, expected_msg: &str) -> Result<(), ParseError> {
        if !self.is_peek_token_type(token) {
            Err(ParseError::new(expected_msg))
        } else {
            self.advance();
            Ok(())
        }
    }

    // If token type there, consume and advance. Otherwise do nothing
    // Return true if the token was consumed, else false
    // fn consume_opt_token_type(&mut self, token: Token) -> bool {
    //     if self.is_peek_token_type(token) {
    //         self.advance();
    //         true
    //     } else {
    //         false
    //     }
    // }

    // Store current lexer token as prev_tok and move up lexer
    fn advance(&mut self) {
        if let Some(val) = self.lexer.peek() {
            self.prev_tok
                .replace(val.clone().expect("Expect lexer to succeed"));
            self.lexer.next();
        }
    }

    // Expect prev_tok to be there (helper method)
    fn expect_prev_tok(&self) -> Result<&Token, ParseError> {
        match &self.prev_tok {
            Some(tok) => Ok(tok),
            None => Err(ParseError::new("Expected previous token")),
        }
    }

    // Pass in self.lexer.peek() => get String out for Ident, String in quotes
    fn string_from_ident(token: Option<&Result<Token, ()>>) -> String {
        let tok = token.unwrap();
        let tok = tok.clone().unwrap();
        tok.to_string()
    }

    /// Parse and return type annotation. Expect lexer.peek() to be at Colon before call
    fn parse_type_annotation(&mut self) -> Result<Type, ParseError> {
        self.consume_token_type(Token::Colon, "Expected a colon")?;
        expect_token_body!(self.lexer.peek(), Ident, "identifier")?;
        let ident = Parser::string_from_ident(self.lexer.peek());

        // Primitive types for now. Compound types: build using primitives within parser
        let type_ann = Type::from_string(&ident)?;
        // dbg!("TYPE ANNOTATION:", &type_ann);

        // Peek should be at equals at the end, so we advance
        self.advance();
        // dbg!(&self.lexer.peek());

        Ok(type_ann)
    }

    // Parse let statement
    // let x = 2;
    fn parse_let(&mut self) -> Result<Decl, ParseError> {
        expect_token_body!(self.lexer.peek(), Ident, "identifier")?;
        let ident = Parser::string_from_ident(self.lexer.peek());
        self.advance();

        let mut type_ann: Option<Type> = None;

        // Do nothing if not colon: allow no annotation to let prev tests pass (for now)
        if self.is_peek_token_type(Token::Colon) {
            // Parse type annotation if any
            let ty = self.parse_type_annotation()?;
            type_ann.replace(ty);
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

    /* Precedence */

    // Return (left bp, right bp)
    fn get_infix_bp(binop: &BinOpType) -> (u8, u8) {
        match binop {
            BinOpType::Add | BinOpType::Sub => (1, 2),
            BinOpType::Mul | BinOpType::Div => (3, 4),
        }
    }

    // Unary negation has a higher precedence than binops
    fn get_prefix_bp(unop: &UnOpType) -> ((), u8) {
        match unop {
            UnOpType::Negate | UnOpType::Not => ((), 5),
        }
    }

    fn parse_ident(&mut self, ident: String, min_bp: u8) -> Result<Decl, ParseError> {
        let sym = Expr::Symbol(ident.to_string());

        // Handle assignment
        if let Some(tok) = self.lexer.peek() {
            let tok = tok.as_ref().expect("Lexer should not fail");
            if tok.eq(&Token::Eq) {
                self.consume_token_type(Token::Eq, "Expected '='")?;
                self.advance();

                // now prev_tok has the start of the expr
                let expr = self.parse_expr(min_bp)?.to_expr()?;

                let assign = AssignStmt { ident, expr };

                return Ok(Assign(assign));
            }
        }
        Ok(ExprStmt(sym))
    }

    // Parses and returns a declaration. At this stage "declaration" includes values, let assignments, fn declarations, etc
    // Because treatment of something as an expression can vary based on whether it is last value or not, whether semicolon comes after, etc.
    fn parse_decl(&mut self) -> Result<Decl, ParseError> {
        let prev_tok = self.expect_prev_tok()?;
        match prev_tok {
            Token::Integer(_)
            | Token::Float(_)
            | Token::Bool(_)
            | Token::Minus
            | Token::Ident(_)
            | Token::OpenParen
            | Token::Bang
            | Token::OpenBrace => self.parse_expr(0),
            Token::Let => self.parse_let(),
            _ => Err(ParseError::new(&format!(
                "Unexpected token: '{}'",
                prev_tok
            ))),
        }
    }

    // Implicit block
    pub fn parse(mut self) -> Result<BlockSeq, ParseError> {
        self.parse_seq()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;

    pub fn test_parse(inp: &str, expected: &str) {
        let lex = Token::lexer(inp);
        let parser = Parser::new(lex);
        let res = parser.parse().expect("Should parse");
        dbg!(&res);
        assert_eq!(res.to_string(), expected);
    }

    pub fn test_parse_err(inp: &str, exp_err: &str, contains: bool) {
        let lex = Token::lexer(inp);
        let parser = Parser::new(lex);
        let res = parser.parse().expect_err("Should err");

        dbg!(&res.to_string());

        if contains {
            assert!(res.to_string().contains(exp_err))
        } else {
            assert_eq!(res.to_string(), exp_err);
        }
    }

    #[test]
    fn test_parse_ints() {
        test_parse("", "");
        test_parse(" 20\n ", "20"); // expr only
        test_parse(" 20;\n ", "20;"); // one exprstmt
        test_parse(" 20 ;30 \n ", "20;30"); // exprstmt, expr
        test_parse(" 20 ;30; \n ", "20;30;"); // exprstmt, exprsmt
        test_parse(" 20 ;30; \n40 \n ", "20;30;40"); // two exprstmt + expr
    }

    #[test]
    fn test_parse_floats() {
        test_parse(" 2.2\n ", "2.2");
        test_parse(" 2.23\n ", "2.23");
        test_parse(" 2.23; 4.5\n ", "2.23;4.5");
        test_parse(" 2.23; 4.5; 4.6\n ", "2.23;4.5;4.6");
    }

    #[test]
    fn test_parse_bools() {
        test_parse("true\n ", "true");
        test_parse("true; false\n ", "true;false");
        test_parse("true; false; true;\n ", "true;false;true;");
        test_parse("true; false; true; false\n ", "true;false;true;false");
    }

    #[test]
    fn test_parse_mixed_primitives() {
        test_parse(
            "true; 2; 4.5; false; 200; 7.289; 90; true",
            "true;2;4.5;false;200;7.289;90;true",
        );
        test_parse(
            "true; 2; 4.5; false; 200; 7.289; 90; true; 2.1;",
            "true;2;4.5;false;200;7.289;90;true;2.1;",
        )
    }

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
    fn test_errs_for_consecutive_exprs() {
        test_parse_err("20 30", "infix operator", true);
    }

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

    #[test]
    fn test_parse_assignment() {
        test_parse_err("x = y = 2", "not an expression", true);
        test_parse_err("let x = y = 2;", "not an expression", true);

        test_parse("x = 3+2;", "x = (3+2);");
        // not type checked yet - allows for us to do dynamic typing if we want
        test_parse(
            "let x : int = 20; x = true; x",
            "let x : int = 20;x = true;x",
        );
    }
}
