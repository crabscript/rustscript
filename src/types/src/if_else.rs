use crate::type_checker::{CheckResult, TypeChecker, TypeErrors};
use parser::structs::{IfElseData, Type};

impl<'prog> TypeChecker<'prog> {
    /*
    0. Check cond is bool type
    1. Get errors from if_blk and combine with else, if present
    2. No errs: check type. Types must be equal
    3. IfOnly: just check if_blk. If everything ok, always return Unit because all ifs are decl even if last
    */
    pub(crate) fn check_if_else(
        &mut self,
        if_else: &IfElseData,
    ) -> Result<CheckResult, TypeErrors> {
        let mut ty_errs = TypeErrors::new();
        let check_cond = self.check_expr(&if_else.cond);

        // check predicate for errs and add. no errs -> check type is bool
        if let Err(mut errs) = check_cond {
            ty_errs.append(&mut errs);
        } else {
            let check_cond = check_cond.unwrap();
            if !check_cond.ty.eq(&Type::Bool) {
                // add cond is not bool err
                let e = format!(
                    "Expected type '{}' for if condition, got '{}'",
                    Type::Bool,
                    check_cond.ty
                );
                ty_errs.add(&e);
            }
        }

        // add if blk errs
        let mut check_if = self.check_block(&if_else.if_blk);
        if let Err(ref mut errs) = check_if {
            ty_errs.append(errs);
        }

        // no else: stop here and return
        // condition may not run, so doesn't matter
        if if_else.else_blk.is_none() {
            return if ty_errs.is_ok() {
                // Ok(Type::Unit)
                Ok(CheckResult {
                    ty: Type::Unit,
                    must_break: false,
                    must_return: false,
                })
            } else {
                Err(ty_errs)
            };
        }

        let else_blk = if_else.else_blk.as_ref().unwrap();

        // Have else: check for errs and add. No errs, and if_blk also no errs: check for type mismatch
        let mut check_else = self.check_block(else_blk);
        if let Err(ref mut errs) = check_else {
            ty_errs.append(errs);
        }

        if let (Ok(if_ty), Ok(else_ty)) = (check_if, check_else) {
            // dbg!(&if_ty, &else_ty);
            // if one block has must_return or must_break, take the type of the other block. if both blks must_return || must_break,
            // overall type is Unit
            let if_terms = if_ty.must_break || if_ty.must_return;
            let else_terms = else_ty.must_break || else_ty.must_return;

            let overall_ty = match (if_terms, else_terms) {
                // no terminate: return out
                (false, false) => {
                    if if_ty.ty.eq(&else_ty.ty) {
                        if ty_errs.is_ok() {
                            return Ok(if_ty);
                        } else {
                            return Err(ty_errs);
                        }
                    }

                    let e = format!(
                        "if-else has type mismatch - consequent:{}, alt :{}",
                        if_ty.ty, else_ty.ty
                    );
                    ty_errs.add(&e);
                    // this would be the last error so we can return
                    return Err(ty_errs);
                }
                // if terms: take else type
                (true, false) => else_ty.ty,
                // else terms: take if type
                (false, true) => if_ty.ty,
                // both term: unit type
                (true, true) => Type::Unit,
            };

            // if-else: both branches must terminate for this to terminate as well
            return Ok(CheckResult {
                ty: overall_ty,
                must_break: if_ty.must_break && else_ty.must_break,
                must_return: if_ty.must_return && else_ty.must_return,
            });

            // if if_ty.ty.eq(&else_ty.ty) {
            //     if ty_errs.is_ok() {
            //         return Ok(if_ty);
            //     } else {
            //         return Err(ty_errs);
            //     }
            // }

            // let e = format!(
            //     "if-else has type mismatch - consequent:{}, alt :{}",
            //     if_ty.ty, else_ty.ty
            // );
            // ty_errs.add(&e);
        }

        Err(ty_errs)
    }
}

#[cfg(test)]
mod tests {
    use parser::structs::Type;

    use crate::type_checker::{expect_err, expect_pass};

    #[test]
    fn test_type_check_if_basic() {
        let t = r"
        if true {
            20
        }
        ";
        expect_pass(t, Type::Unit);

        // if-else as expr gives type of blks if matching
        let t = r"
        if true {
            20
        } else {
            30
        }
        ";
        expect_pass(t, Type::Int);

        // if-else as expr gives type of blks if matching
        let t = r"
        let x : int = if true {
            20
        } else {
            30
        };
        x
        ";
        expect_pass(t, Type::Int);

        // both unit ok
        let t = r"
        if true {
            20;
        } else {
            30;
        }
        ";
        expect_pass(t, Type::Unit);
    }

    #[test]
    fn test_type_check_if_conderr() {
        let t = r"
        if !2 {
            20;
        }
        ";
        expect_err(t, "Can't apply logical NOT to type int", true);

        // cond ok but wrong type
        let t = r"
        if 2+2 {
            20;
        }
        ";
        expect_err(t, "Expected type 'bool' for if condition, got 'int'", true);

        // cond has err when if-else types match
        let t = r"let x = 2; let y = 3; if !x == y { 20 } else { 30 }";
        expect_err(t, "Can't apply logical NOT to type int", true);
    }

    #[test]
    fn test_type_check_if_multiple() {
        // all three error (cond, two blks)
        let t = r"
        if 2+2 {
            let x : bool = 2.46;
        } else {
            30+false;
        }
        ";
        expect_err(t,  "[TypeError]: Expected type 'bool' for if condition, got 'int'\n[TypeError]: 'x' has declared type bool but assigned type float\n[TypeError]: Can't apply '+' to types 'int' and 'bool'", false);

        // multiple errs in blks
        let t = r"
        if 2+2 {
            let x : bool = 2.46;
            2+true;
        } else {
            30+false;
            2.56+2;
        }
        ";
        expect_err(t,  "[TypeError]: Expected type 'bool' for if condition, got 'int'\n[TypeError]: 'x' has declared type bool but assigned type float\n[TypeError]: Can't apply '+' to types 'int' and 'bool'\n[TypeError]: Can't apply '+' to types 'int' and 'bool'\n[TypeError]: Can't apply '+' to types 'float' and 'int'", false);

        // cond + else err
        let t = r"
        if 2+2 {
            let x : int = 2;
        } else {
            30+false;
        }
        ";
        expect_err(t, "[TypeError]: Expected type 'bool' for if condition, got 'int'\n[TypeError]: Can't apply '+' to types 'int' and 'bool'", false);

        // cond + if err
        let t = r"
         if 2+2 {
             30+2.56;
         } else {
             300;
         }
         ";
        expect_err(t, "[TypeError]: Expected type 'bool' for if condition, got 'int'\n[TypeError]: Can't apply '+' to types 'int' and 'float'", false);
    }

    #[test]
    fn test_type_check_if_else_blksonly() {
        // if + else err
        let t = r"
         if true {
            30+2.56;
         } else {
            300+false;
         }
         ";
        expect_err(t,  "[TypeError]: Can't apply '+' to types 'int' and 'float'\n[TypeError]: Can't apply '+' to types 'int' and 'bool'", false);

        // if only
        let t = r"
         if true {
            30+2.56;
         } else {
            300;
         }
         ";
        expect_err(t, "Can't apply '+' to types 'int' and 'float'", true);

        // else only
        let t = r"
         if true {
            300;
         } else {
            30+true;
         }
         ";
        expect_err(t, "Can't apply '+' to types 'int' and 'bool'", true);

        // no errs but type mismatch
        let t = r"
         if true {
            300
         } else {
            true
         }
         ";
        expect_err(
            t,
            "if-else has type mismatch - consequent:int, alt :bool",
            true,
        );

        // no errs but type mismatch - when if else is stmt
        let t = r"
         if true {
            300
         } else {
            true
         };
         ";
        expect_err(
            t,
            "if-else has type mismatch - consequent:int, alt :bool",
            true,
        );

        // works when if-else is stmt as long as types are same - just like Rust
        let t = r"
         if true {
            300
         } else {
            500
         }

         if true {
            true
         } else {
            false
         }

         300;
         if false {
            200
         }
         ";
        expect_pass(t, Type::Unit);
    }
}
