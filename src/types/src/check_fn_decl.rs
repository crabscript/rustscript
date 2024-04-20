use parser::structs::{FnDeclData, FnTypeData, Type};

use crate::type_checker::{CheckResult, TypeChecker, TypeErrors};

impl<'prog> TypeChecker<'prog> {
    pub(crate) fn check_fn_decl(
        &mut self,
        fn_decl: &FnDeclData,
    ) -> Result<CheckResult, TypeErrors> {
        self.fn_type_stack.push(fn_decl.ret_type.clone());
        let res = self.check_fn_decl_inner(fn_decl);
        self.fn_type_stack.pop();
        res
    }

    // 1. all nested returns belonging to fn should have same type as annotated ret type: use fn_stack to track this
    // 2. Last expr (if it exists) must have same type as annotated, unless there was must_return before

    // 3. Fn decl well-typed iff - all nested return stmts belonging to the function return the same type as the ty_ann,
    // AND (somewhere in the block we encounter a terminating decl/ last_expr OR the
    // last expression of the block has the same type as the ty_ann)
    // Everything after a must_return is ignored. function returns unit => don't need must_return, but nested ret cannot return anything else
    fn check_fn_decl_inner(&mut self, fn_decl: &FnDeclData) -> Result<CheckResult, TypeErrors> {
        // Assert all params have type ann and add their types
        let mut param_types: Vec<Type> = vec![];

        for param in fn_decl.params.iter() {
            if let Some(ty) = &param.type_ann {
                param_types.push(ty.to_owned());
            } else {
                let e = format!("Parameter '{}' has no type annotation", param.name);
                return Err(TypeErrors::new_err(&e));
            }
        }

        let fn_ty = FnTypeData {
            params: param_types,
            ret_type: fn_decl.ret_type.clone(),
        };

        let fn_ty = Type::UserFn(Box::new(fn_ty));
        // let mut ty_errs = TypeErrors::new();

        let fn_res = CheckResult {
            ty: fn_ty.clone(),
            must_break: false,
            must_return: false,
        };

        // Before checking block, add this fn to env to support recursion
        self.assign_ident(&fn_decl.name, fn_ty.clone())?; // should work because of enterscope

        // dbg!("FN_PARAMS:", &fn_decl.params, &fn_decl.name);

        let blk_res = self.check_block(&fn_decl.body, fn_decl.params.clone())?;
        // dbg!("FN BLK TYPE:", &blk_res);

        // If must_return encountered in block, we assume nested returns are correct type so just stop here
        if blk_res.must_return {
            return Ok(fn_res);
        }

        // check blk_ty matches overall ret type only if last_expr exists
        if fn_decl.body.last_expr.is_some() {
            if blk_res.ty.eq(&fn_decl.ret_type) {
                return Ok(fn_res);
            } else {
                let e = format!(
                    "Function '{}' has return type '{}' but found block type '{}'",
                    fn_decl.name, fn_decl.ret_type, blk_res.ty
                );
                return Err(TypeErrors::new_err(&e));
            }
        }

        // if no must_return, and no last_expr, and overall type is not Unit, err
        if !fn_decl.ret_type.eq(&Type::Unit) {
            let e = format!(
                "Function '{}' might not return '{}'",
                fn_decl.name, fn_decl.ret_type
            );
            return Err(TypeErrors::new_err(&e));
        }

        Ok(fn_res)

        // If everything is ok, return the annotated types
        // Fn decl doesn't contribute to overall must_ret / must_break of the outer block
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
        // should fail because blk has type bool
        let t = r"
        fn f(x: int) -> int {
            true
        }
        f
        ";
        expect_err(t, "has return type 'int' but found block type 'bool'", true);

        // last expr ret
        let t = r"
        fn f() -> int {
            20+30
        }
        ";
        expect_pass(t, Type::Unit);

        let t = r"
        fn f() -> int {
            return 20;
        }
        ";
        expect_pass(t, Type::Unit);

        let t = r"
        fn f() -> int {
            {
                return 30;
            }
            true
        }
        ";
        expect_pass(t, Type::Unit);

        let t = r"
        fn f() -> int {
            if true {
                return 20;
            } else {
                return 30;
            }
        }
        ";
        expect_pass(t, Type::Unit);

        // // if only, loop are not must_ret
        //     // although inf loop that would definitely return here, we are conservative
        let t = r"
        fn f() -> int {
            if true {
                return 20;
            } 

            loop {
                return 30;
            }
        }
        ";
        expect_err(t, "might not return", true);

        // unit - don't have to must_return
        let t = r"
        fn f() {
            if true {
                return;
            } 

            loop {
                return;
            }

            fn g() -> int {
                200
            }
        }
        ";
        expect_pass(t, Type::Unit);

        // if else
        let t = r"
        fn f() -> int {
            if true {
                return 20;
            } else {
                30
            }
        }
        ";
        expect_pass(t, Type::Unit);

        let t = r"
        fn f(x: int) -> int {
            2;
        }
        f
        ";
        expect_err(t, "might not return", true);
    }

    #[test]
    fn test_type_check_fn_ret_stmt() {
        let t = r"
        fn f() -> int {
            if true {
                return true;
            } else {
                return 2.56;
            }

            return 5;
        }
        ";
        expect_err(t, "[TypeError]: Expected function return type 'int' but return statement has type 'bool'\n[TypeError]: Expected function return type 'int' but return statement has type 'float'", false);

        // check that it ignores inner return for hof
        let t = r"
        fn f() -> int {
            fn g() -> bool {
                return true;
            }

            return 20;
        }
        f
        ";
        expect_pass_str(t, "fn() -> int");

        // when return expr has error - keeps checking rest
        let t = r"
        fn f() -> int {
            if true {
                return 2+ !2;
            }
            return !true;
        }
        ";
        expect_err(t, "[TypeError]: Can't apply logical NOT to type int\n[TypeError]: Expected function return type 'int' but return statement has type 'bool'", false);
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

        let t = r"
        fn fac(n: int, b: bool) {
            n + b
        } 
        fac(1)
        ";
        expect_err(t, "Can't apply '+' to types 'int' and 'bool'", true);
    }
}
