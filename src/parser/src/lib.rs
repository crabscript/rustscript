pub fn add(left: usize, right: usize) -> usize {
    left + right
}

enum Expr {
    Integer(i64)
}

enum Decl {
    ExprStmt(Expr)
}

// Last expression is value of program semantics (else Unit type)
// Program is either one declaration or a sequence of declarations with optional last expression
    // Decl is there to avoid needing to do recursive Seq for a bunch of stmts in order
enum ASTNode {
    Decl,
    Seq(Vec<ASTNode>, Option<Expr>)
}


#[cfg(test)]
mod tests {
    #[test]
    fn can_lex() {
        let m = String::from("let x = 2;");
        let res = lexer::lex(m.as_str());
        let res = res.collect::<Vec<_>>();
        dbg!(res);
    }
}
