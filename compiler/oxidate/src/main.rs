use lexer::lex;
use parser::Parser;
use bytecode::ByteCode;

use crate::compiler::Compiler;

mod compiler;

fn main() {
    println!("Hello, world!");
    let inp = "42;";
    let lex = lex(inp);
    let p = Parser::new(lex);
    let res = p.parse().unwrap();

    let comp = Compiler::new(res);
    let res = comp.compile().unwrap();
    dbg!(res);
    // compiler::compile();
}
