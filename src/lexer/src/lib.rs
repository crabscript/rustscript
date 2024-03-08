#![allow(unused)]

use token::Token;
use token::TokenKind;

mod token;

pub fn tokenize(s: &mut str) -> Vec<Token> {
    let mut tokens = Vec::new();

    loop {
        let next_token = next(s);
        match next_token.kind {
            TokenKind::Eof => break,
            _ => tokens.push(next_token),
        }
    }

    tokens
}

fn next(s: &mut str) -> Token {
    todo!()
}
