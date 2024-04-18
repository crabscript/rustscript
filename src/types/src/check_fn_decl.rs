use parser::structs::{FnDeclData, FnTypeData, Type};

use crate::type_checker::{CheckResult, TypeChecker, TypeErrors};

impl<'prog> TypeChecker<'prog> {
    // 1. all nested returns should have same type as annotated ret type: use fn_stack to track this
    pub(crate) fn check_fn_decl(
        &mut self,
        fn_decl: &FnDeclData,
    ) -> Result<CheckResult, TypeErrors> {
        let fn_ty = FnTypeData {
            params: fn_decl.params.clone(),
            ret_type: fn_decl.ret_type.clone(),
        };

        let fn_ty = Type::UserFn(Box::new(fn_ty));
        // let mut ty_errs = TypeErrors::new();

        // Before checking block, add this fn to env to support recursion
        self.assign_ident(&fn_decl.name, fn_ty.clone())?; // should work because of enterscope

        dbg!("FN_PARAMS:", &fn_decl.params, &fn_decl.name);
        self.check_block(&fn_decl.body, fn_decl.params.clone())?;
        // If everything is ok, return the annotated types
        // Fn decl doesn't contribute to overall must_ret / must_break of the outer block
        let res = CheckResult {
            ty: fn_ty,
            must_break: false,
            must_return: false,
        };
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use parser::structs::Type;

    use crate::type_checker::{expect_err, expect_pass, expect_pass_str};

    #[test]
    fn test_type_check_fn_decl_simple() {
        let t = r"
        fn f() {

        }
        ";
        expect_pass(t, Type::Unit);

        let t = r"
        fn f() {

        }
        f
        ";
        expect_pass_str(t, "fn()");

        // with annotation
        let t = r"
        fn f(x: int) -> bool {
            true
        }
        f
        ";
        expect_pass_str(t, "fn(int) -> bool");

        let t = r"
        fn f(x: int) {
            
        }
        f
        ";
        expect_pass_str(t, "fn(int)");
    }

    #[test]
    fn test_type_check_fn_decl_fails() {
        // param has no ty ann
        let t = r"
        fn f(x : int, y) {

        }
        ";
        expect_err(t, "[TypeError]: Parameter 'y' has no type annotation", true);

        let t = r"
        fn fac(n) {

        }
        ";
        expect_err(t, "Parameter 'n' has no type annotation", true);
    }

    #[test]
    fn test_type_check_fn_rettype() {
        // should fail because blk has unit
        // let t = r"
        // fn f(x: int) -> int {
        //     f(x-1);
        // }
        // f
        // ";
    }

    #[test]
    fn test_type_check_fn_decl_edges() {
        // Recursive
        let t = r"
        fn f(x: int) -> int {
            f(x-1)
        }
        f
        ";
        expect_pass_str(t, "fn(int) -> int");

        // should fail bc n has type int but x has type bool
        // need to add type assignments for params before going in
        let t = r"
        fn fac(n: int) {
            let x :bool = n;
            2 + n
        } 
        fac(1)
        ";
        expect_err(t, "'x' has declared type bool but assigned type int", true);
    }
}
