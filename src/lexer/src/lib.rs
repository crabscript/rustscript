use logos::{Lexer, Logos, Skip};

/// Update the line count and the char index.
fn newline_callback(lex: &mut Lexer<Token>) -> Skip {
    lex.extras.0 += 1;
    lex.extras.1 = lex.span().end;
    Skip
}

// Just skip comments
fn comment_callback(_lex: &mut Lexer<Token>) -> Skip {
    Skip
}

#[derive(Debug, Logos, PartialEq, Clone)]
#[logos(skip r"[ \t\r\f]+", extras=(usize, usize))]
// #[logos(extras = (usize, usize))]
pub enum Token {
    #[regex(r"\n", newline_callback)]
    Newline,

    #[token(";")]
    Semi,

    #[token(":")]
    Colon,

    #[token(".")]
    Dot,

    #[token(",")]
    Comma,

    #[token("(")]
    OpenParen,

    #[token(")")]
    CloseParen,

    #[token("{")]
    OpenBrace,

    #[token("}")]
    CloseBrace,

    #[token("[")]
    OpenBracket,

    #[token("]")]
    CloseBracket,

    #[token("@")]
    At,

    #[token("#")]
    Pound,

    #[token("~")]
    Tilde,

    #[token("?")]
    Question,

    #[token("$")]
    Dollar,

    #[token("=")]
    Eq,

    #[token("==")]
    LogEq,

    #[token("!")]
    Bang,

    #[token("<")]
    Lt,

    #[token(">")]
    Gt,

    #[token("-")]
    Minus,

    #[token("&")]
    And,

    #[token("&&")]
    LogAnd,

    #[token("|")]
    Or,

    #[token("||")]
    LogOr,

    #[token("+")]
    Plus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("^")]
    Caret,

    #[token("%")]
    Percent,

    #[token("let")]
    Let,

    #[token("if")]
    If,

    #[token("else")]
    Else,

    #[token("fn")]
    Fn,

    #[token("->")]
    FnDeclReturn,

    #[token("return")]
    Return,

    #[regex(r#"[a-zA-Z_][a-zA-Z0-9_]*"#, |lex| lex.slice().to_owned())]
    Ident(String),

    #[regex(r#"//[^\n]*"#, comment_callback)]
    Comment,

    #[token("loop")]
    Loop,

    #[token("break")]
    Break,

    #[token("spawn")]
    Spawn,

    #[token("join")]
    Join,

    #[token("false", |_| false)]
    #[token("true", |_| true)]
    Bool(bool),

    // issue: negative numbers should be dealt with at parser level instead of lexer level (causes issue with minus operator)
    // https://stackoverflow.com/questions/58910659/how-to-properly-lex-negative-numbers
    // so we don't put -? at the front
    #[regex(r"\d+", |lex| lex.slice().parse::<i64>().unwrap())]
    Integer(i64),

    #[regex(r"\d*\.\d+", |lex| lex.slice().parse::<f64>().unwrap())]
    Float(f64),

    #[regex(r#""([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#, |lex| {
      let slice = lex.slice();
      let stripped = &slice[1..slice.len() - 1];
      stripped.to_owned()
  })]
    String(String),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl Token {
    pub fn repr(&self) -> String {
        match self {
            Self::Ident(id) => id.to_string(),
            Self::String(str) => str.to_string(),
            Self::Semi => ";".to_string(),
            Self::Colon => ":".to_string(),
            Self::Dot => ".".to_string(),
            Self::Comma => ",".to_string(),
            Self::OpenParen => "(".to_string(),
            Self::CloseParen => ")".to_string(),
            Self::OpenBrace => "{".to_string(),
            Self::CloseBrace => "}".to_string(),
            Self::OpenBracket => "[".to_string(),
            Self::CloseBracket => "]".to_string(),
            Self::At => "@".to_string(),
            Self::Pound => "#".to_string(),
            Self::Tilde => "~".to_string(),
            Self::Question => "?".to_string(),
            Self::Dollar => "$".to_string(),
            Self::Eq => "=".to_string(),
            Self::Bang => "!".to_string(),
            Self::Lt => "<".to_string(),
            Self::Gt => ">".to_string(),
            Self::Minus => "-".to_string(),
            Self::And => "&".to_string(),
            Self::Or => "|".to_string(),
            Self::Plus => "+".to_string(),
            Self::Star => "*".to_string(),
            Self::Slash => "/".to_string(),
            Self::Caret => "^".to_string(),
            Self::Percent => "%".to_string(),
            Self::Let => "let".to_string(),
            Self::Bool(val) => val.to_string(),
            Self::Integer(val) => val.to_string(),
            Self::Float(val) => val.to_string(),
            Self::If => "if".to_string(),
            Self::Else => "else".to_string(),
            Self::LogEq => "==".to_string(),
            Self::LogAnd => "&&".to_string(),
            Self::LogOr => "||".to_string(),
            Self::Loop => "loop".to_string(),
            Self::Break => "break".to_string(),
            Self::Comment => "//".to_string(),
            Self::Newline => "\n".to_string(),
            Self::Fn => "fn".to_string(),
            Self::Return => "return".to_string(),
            Self::FnDeclReturn => "->".to_string(),
            Self::Spawn => "spawn".to_string(),
            Self::Join => "join".to_string(),
        }
    }
}

pub fn lex(input: &str) -> Lexer<'_, Token> {
    Token::lexer(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::f64;
    use std::i64;

    #[test]
    fn test_bool() {
        let mut lexer = Token::lexer("true\n\tfalse");
        assert_eq!(
            Token::Bool(true),
            lexer.next().unwrap().expect("Expected token")
        );
        assert_eq!(
            Token::Bool(false),
            lexer.next().unwrap().expect("Expected token")
        );
    }

    #[test]
    fn test_lexer_integer() {
        let input = "0 1 42 1234567890 -1 -42 -1234567890";
        let mut tokens = Token::lexer(input);

        let expected = vec![
            Token::Integer(0),
            Token::Integer(1),
            Token::Integer(42),
            Token::Integer(1234567890),
            Token::Minus,
            Token::Integer(1),
            Token::Minus,
            Token::Integer(42),
            Token::Minus,
            Token::Integer(1234567890),
        ];

        for e in expected {
            assert_eq!(e, tokens.next().unwrap().expect("Expected token"));
        }
    }

    #[test]
    fn test_lexer_number_leading_zero() {
        let input = "02 003 00401.02";
        let mut tokens = Token::lexer(input);

        let expected = [Token::Integer(2), Token::Integer(3), Token::Float(401.02)];

        for e in expected {
            assert_eq!(e, tokens.next().unwrap().expect("Expected token"));
        }
    }

    #[test]
    fn test_lexer_integer_max() {
        // NOTE: Because of minus lexing issue the range of -ve numbers we can handle is reduced by one
        let max_int = i64::MAX.to_string();
        let min_int = (i64::MIN + 1).to_string();

        let input = format!("{} {}", max_int, min_int);
        let mut tokens = Token::lexer(&input);

        let expected = vec![
            Token::Integer(i64::MAX),
            Token::Minus,
            Token::Integer(i64::MAX),
        ];

        for e in expected {
            assert_eq!(e, tokens.next().unwrap().expect("Expected token"));
        }
    }

    #[test]
    fn test_lexer_float() {
        let input = "1.23 -4.56";
        let mut tokens = Token::lexer(input);

        let expected = vec![Token::Float(1.23), Token::Minus, Token::Float(4.56)];

        for e in expected {
            assert_eq!(e, tokens.next().unwrap().expect("Expected token"));
        }
    }

    #[test]
    fn test_lexer_float_max() {
        // NOTE: Because of minus lexing issue the range of -ve numbers we can handle is reduced by one
        let max_float = f64::MAX.to_string();
        let min_float = (f64::MIN + 1.0).to_string();

        // Add .0 to the end of the floats to make them float tokens
        let input = format!("{}.0 {}.0", max_float, min_float);
        let mut tokens = Token::lexer(&input);

        let expected = vec![Token::Float(f64::MAX), Token::Minus, Token::Float(f64::MAX)];

        for e in expected {
            assert_eq!(e, tokens.next().unwrap().expect("Expected token"));
        }
    }

    #[test]
    fn test_float_special_cases() {
        let input = "0.0 -0.0 0.1 1.0 1.1 .0 .1";
        let mut tokens = Token::lexer(input);

        let expected = vec![
            Token::Float(0.0),
            Token::Minus,
            Token::Float(0.0),
            Token::Float(0.1),
            Token::Float(1.0),
            Token::Float(1.1),
            Token::Float(0.0),
            Token::Float(0.1),
        ];

        for e in expected {
            assert_eq!(e, tokens.next().unwrap().expect("Expected token"));
        }
    }

    #[test]
    fn test_string() {
        let mut lexer = Token::lexer(r#""hello" "world""#);
        assert_eq!(
            Token::String("hello".to_string()),
            lexer.next().unwrap().expect("Expected token")
        );
        assert_eq!(
            Token::String("world".to_string()),
            lexer.next().unwrap().expect("Expected token")
        );
    }

    #[test]
    fn test_single_char_symbols() {
        let input = ";:.,{}()@#~?$=-&|+*/^%";
        let mut lexer = Token::lexer(input);

        let expected = vec![
            Token::Semi,
            Token::Colon,
            Token::Dot,
            Token::Comma,
            Token::OpenBrace,
            Token::CloseBrace,
            Token::OpenParen,
            Token::CloseParen,
            Token::At,
            Token::Pound,
            Token::Tilde,
            Token::Question,
            Token::Dollar,
            Token::Eq,
            Token::Minus,
            Token::And,
            Token::Or,
            Token::Plus,
            Token::Star,
            Token::Slash,
            Token::Caret,
            Token::Percent,
        ];

        for e in expected {
            assert_eq!(e, lexer.next().unwrap().expect("Expected token"));
        }
    }

    #[test]
    fn test_identifiers() {
        let input = "foo bar baz _john _ fn let mut continue break struct";
        let mut lexer = Token::lexer(input);

        let expected = vec![
            Token::Ident("foo".to_string()),
            Token::Ident("bar".to_string()),
            Token::Ident("baz".to_string()),
            Token::Ident("_john".to_string()),
            Token::Ident("_".to_string()),
            Token::Fn,
            Token::Let,
            Token::Ident("mut".to_string()),
            Token::Ident("continue".to_string()),
            Token::Break,
        ];

        for e in expected {
            assert_eq!(e, lexer.next().unwrap().expect("Expected token"));
        }
    }

    #[test]
    fn test_normal_code_1() {
        let input = r#"let x = 42; let y = 4.0;"#;
        let mut lexer = Token::lexer(input);

        let expected = vec![
            Token::Let,
            Token::Ident("x".to_string()),
            Token::Eq,
            Token::Integer(42),
            Token::Semi,
            Token::Let,
            Token::Ident("y".to_string()),
            Token::Eq,
            Token::Float(4.0),
        ];

        for e in expected {
            assert_eq!(e, lexer.next().unwrap().expect("Expected token"));
        }
    }

    #[test]
    fn test_normal_code_2() {
        let input = r#"while (x < 10) { x = x + 1; }"#;
        let mut lexer = Token::lexer(input);

        let expected = vec![
            Token::Ident("while".to_string()),
            Token::OpenParen,
            Token::Ident("x".to_string()),
            Token::Lt,
            Token::Integer(10),
            Token::CloseParen,
            Token::OpenBrace,
            Token::Ident("x".to_string()),
            Token::Eq,
            Token::Ident("x".to_string()),
            Token::Plus,
            Token::Integer(1),
            Token::Semi,
            Token::CloseBrace,
        ];

        for e in expected {
            assert_eq!(e, lexer.next().unwrap().expect("Expected token"));
        }
    }

    #[test]
    fn test_normal_code_3() {
        let input = r#"fn add(x: i64, y: i64) -> i64 { return x + y; }"#;
        let mut lexer = Token::lexer(input);

        let expected = vec![
            Token::Fn,
            Token::Ident("add".to_string()),
            Token::OpenParen,
            Token::Ident("x".to_string()),
            Token::Colon,
            Token::Ident("i64".to_string()),
            Token::Comma,
            Token::Ident("y".to_string()),
            Token::Colon,
            Token::Ident("i64".to_string()),
            Token::CloseParen,
            Token::FnDeclReturn,
            Token::Ident("i64".to_string()),
            Token::OpenBrace,
            Token::Return,
            Token::Ident("x".to_string()),
            Token::Plus,
            Token::Ident("y".to_string()),
            Token::Semi,
            Token::CloseBrace,
        ];

        for e in expected {
            assert_eq!(e, lexer.next().unwrap().expect("Expected token"));
        }
    }

    #[test]
    fn test_nonsense_input() {
        let input = "  frij34ij33 \n \t wrjijeritj  ";
        let mut lexer = Token::lexer(input);
        let expected = vec![
            Token::Ident(String::from("frij34ij33")),
            Token::Ident(String::from("wrjijeritj")),
        ];

        for e in expected {
            assert_eq!(e, lexer.next().unwrap().expect("Expected token"));
        }
    }

    #[test]
    fn test_lex_comp_ops() {
        // ==, <, >, &&, ||
        let t = "== = < > && ||";
        let mut lexer = Token::lexer(t);
        let exp: Vec<Token> = vec![
            Token::LogEq,
            Token::Eq,
            Token::Lt,
            Token::Gt,
            Token::LogAnd,
            Token::LogOr,
        ];
        for e in exp {
            assert_eq!(e, lexer.next().unwrap().expect("Expected token"));
        }

        // usage
        let t = "x = x < 10 && x > 3 || y == 4; ";
        let mut lexer = Token::lexer(t);

        let exp: Vec<Token> = vec![
            Token::Ident("x".to_string()),
            Token::Eq,
            Token::Ident("x".to_string()),
            Token::Lt,
            Token::Integer(10),
            Token::LogAnd,
            Token::Ident("x".to_string()),
            Token::Gt,
            Token::Integer(3),
            Token::LogOr,
            Token::Ident("y".to_string()),
            Token::LogEq,
            Token::Integer(4),
        ];
        for e in exp {
            assert_eq!(e, lexer.next().unwrap().expect("Expected token"));
        }
    }

    #[test]
    fn test_lex_loop() {
        let t = r"
        loop {
            break;
        }
        ";
        let exp = vec![
            Token::Loop,
            Token::OpenBrace,
            Token::Break,
            Token::Semi,
            Token::CloseBrace,
        ];
        let mut lexer = Token::lexer(t);
        for e in exp {
            assert_eq!(e, lexer.next().unwrap().expect("Expected token"));
        }
    }

    #[test]
    fn test_lex_comments() {
        let t = r"
        // comment
        1
        // comment2
        2
        // c3
        // c4
        3;
        ";
        let mut lexer = Token::lexer(t);
        // skips comment but adds to newline
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Integer(1));
        assert_eq!(lexer.extras.0, 2);

        assert_eq!(lexer.next().unwrap().unwrap(), Token::Integer(2));
        assert_eq!(lexer.extras.0, 4);

        assert_eq!(lexer.next().unwrap().unwrap(), Token::Integer(3));
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Semi);
        assert_eq!(lexer.extras.0, 7);

        assert_eq!(lexer.next(), None);

        // comment on same line
        let t = r"
        // ignored, but adds to line counter since next line is 2
        2; // int
        3 // int2
        // more comments
        // blah
        // ignored
        ";
        let mut lexer = Token::lexer(t);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Integer(2));
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Semi);
        assert_eq!(lexer.extras.0, 2);

        assert_eq!(lexer.next().unwrap().unwrap(), Token::Integer(3));
        assert_eq!(lexer.extras.0, 3);
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_lex_spawn_join() {
        let t = r"
        spawn join
        ";
        let mut lexer = Token::lexer(t);

        assert_eq!(lexer.next().unwrap().unwrap(), Token::Spawn);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Join);
    }
}
