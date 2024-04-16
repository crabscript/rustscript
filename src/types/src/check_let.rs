use crate::type_checker::{CheckResult, TypeChecker, TypeErrors};
use parser::structs::LetStmtData;

impl<'prog> TypeChecker<'prog> {
    pub(crate) fn check_let(&mut self, stmt: &LetStmtData) -> Result<CheckResult, TypeErrors> {
        let mut ty_errs = TypeErrors::new();

        let mut expr_type: Option<CheckResult> = None;
        match self.check_expr(&stmt.expr) {
            Ok(res) => {
                expr_type.replace(res);
            }
            Err(mut err) => {
                ty_errs.append(&mut err);
            }
        };

        match (expr_type, stmt.type_ann) {
            // type check expr has error + we have no type annotation: e.g let x = !2;
            // cannot proceed, error out with cont = false
            (None, None) => {
                ty_errs.cont = false;
                Err(ty_errs)
            }

            // type check expr has err + we have type ann: e.g let x : int = !2;
            // use type of annotation, continue
            (None, Some(ty_ann)) => {
                self.assign_ident(&stmt.ident.to_owned(), ty_ann)?;
                Err(ty_errs)
            }

            // expr is well-typed + no type annotation e.g let x = 2+2;
            // use expr type, no err
            (Some(expr_res), None) => {
                // assign ident, return checkresult propagated from expr
                self.assign_ident(&stmt.ident.to_owned(), expr_res.ty)?;

                let res = CheckResult {
                    ty: expr_res.ty,
                    must_break: expr_res.must_break,
                    must_return: expr_res.must_return,
                };

                Ok(res)
            }

            // expr is well-typed + have ty ann: e.g let x : int = true; or let x : int  = 2;
            // either way, insert type of binding = annotation so we can ty check rest. error out if mismatch
            (Some(expr_res), Some(ty_ann)) => {
                self.assign_ident(&stmt.ident.to_owned(), ty_ann)?;

                if !ty_ann.eq(&expr_res.ty) {
                    let string = format!(
                        "'{}' has declared type {} but assigned type {}",
                        stmt.ident, ty_ann, expr_res.ty
                    );
                    ty_errs.add(&string);
                    return Err(ty_errs);
                }

                let res = CheckResult {
                    ty: expr_res.ty,
                    must_break: expr_res.must_break,
                    must_return: expr_res.must_return,
                };

                Ok(res)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use parser::structs::Type;

    use crate::type_checker::{expect_err, expect_pass};

    #[test]
    fn test_type_check_sym_advanced() {
        // first has err but no type ann: we don't proceed
        expect_err(
            "let x = -true; let y : int = x + 2; let z : bool = !x;",
            "[TypeError]: Can't negate type bool",
            false,
        );

        // expr has err but we have ann: can proceed
        expect_err("let x : int = -true; let y : int = x + false;", 
        "[TypeError]: Can't negate type bool\n[TypeError]: Can't apply '+' to types 'int' and 'bool'", false);

        // expr is fine but no annotation: use inferred type
        expect_err(
            "let x = 2+2; let y : int = !x;",
            "Can't apply logical NOT to type int",
            true,
        );
        expect_pass("let x = 2+2; let y : int = -x*3; y", Type::Int);

        // expr is fine and we have annotation: check for mismatch, can proceed with binding type = annotation
        // here !y is fine so no error, since y is annotated bool
        expect_err("let x : int = !true; let y: bool = x + false; let z : bool = !y;", 
        "[TypeError]: 'x' has declared type int but assigned type bool\n[TypeError]: Can't apply '+' to types 'int' and 'bool'", false);

        expect_err("let x : int = !true; let y: bool = x + false; let z : bool = y + x;", 
        "[TypeError]: 'x' has declared type int but assigned type bool\n[TypeError]: Can't apply '+' to types 'int' and 'bool'\n[TypeError]: Can't apply '+' to types 'bool' and 'int'",
        false);
    }

    #[test]
    fn test_type_check_ident_decl() {
        // stops immediately because y has no annotation
        let t = "let y = x + 2; let z = y - false;";
        expect_err(t, "[TypeError]: Identifier 'x' not declared", false);

        // continues because y has type annotation
        let t = "let y : int = x + 2; let z = y - false;";
        expect_err(t, "[TypeError]: Identifier 'x' not declared\n[TypeError]: Can't apply '-' to types 'int' and 'bool'", false);

        // unit
        let t = "let x : () = {}; let y : () = { 2; 3; }; x";
        expect_pass(t, Type::Unit);

        let t = "let x : () = if true { 2; } else { 3; }; x";
        expect_pass(t, Type::Unit);

        let t = "let x : () = if true { 2 } else { 3 }; x";
        expect_err(t, "'x' has declared type () but assigned type int", true);
    }

    #[test]
    fn test_type_check_bigger() {
        let t = "let y : bool = 20; let x : int = y; let z : int = x*y + 3; z";
        expect_err(t, "[TypeError]: 'y' has declared type bool but assigned type int\n[TypeError]: 'x' has declared type int but assigned type bool\n[TypeError]: Can't apply '*' to types 'int' and 'bool'", false);
    }

    #[test]
    fn test_type_check_assign() {
        // don't continue since first one has err
        let t = "let x = !20; x = true; x";
        expect_err(t, "[TypeError]: Can't apply logical NOT to type int", false);

        let t = "let x = 20; x = true; x";
        expect_err(t, "'x' declared with type int but assigned type bool", true);

        let t = "let x : int = 20; x = !true; x";
        expect_err(t, "'x' declared with type int but assigned type bool", true);

        let t = "let x : int = !20; x = !true; x";
        expect_err(t,"[TypeError]: Can't apply logical NOT to type int\n[TypeError]: 'x' declared with type int but assigned type bool", false);

        let t = "let y = 2; x = 10;";
        expect_err(t, "Identifier 'x' not declared", true);

        // Assign before declaration (error)
        let t = "x = 10; let x = 5;";
        expect_err(
            t,
            "[TypeError]: Identifier 'x' assigned before declaration",
            false,
        );
    }
}
