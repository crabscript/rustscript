use parser::structs::{FnDeclData, FnTypeData, Type};

use crate::type_checker::{CheckResult, TypeChecker, TypeErrors};

impl<'prog> TypeChecker<'prog> {
    pub(crate) fn check_fn_decl(
        &mut self,
        fn_decl: &FnDeclData,
    ) -> Result<CheckResult, TypeErrors> {
        // If everything is ok, return the annotated types
        // Fn decl doesn't contribute to overall must_ret / must_break of the outer block

        let fn_ty = FnTypeData {
            params: fn_decl.params.clone(),
            ret_type: fn_decl.ret_type.clone(),
        };

        let res = CheckResult {
            ty: Type::UserFn(Box::new(fn_ty)),
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
