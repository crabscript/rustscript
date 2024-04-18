use parser::structs::{FnDeclData, Type};

use crate::type_checker::{CheckResult, TypeChecker, TypeErrors};

impl<'prog> TypeChecker<'prog> {
    pub(crate) fn check_fn_decl(
        &mut self,
        fn_decl: &FnDeclData,
    ) -> Result<CheckResult, TypeErrors> {
        dbg!("Got decl:", fn_decl);
        // Fn decl doesn't contribute to overall must_ret / must_break of the outer block
        let res = CheckResult {
            ty: Type::Unit,
            must_break: false,
            must_return: false,
        };

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use parser::structs::{FnDeclData, Type};

    use crate::type_checker::expect_pass;

    #[test]
    fn test_type_check_fn_decl() {
        let t = r"
        fn f() {

        }
        ";
        expect_pass(t, Type::Unit);
    }
}
