use crate::type_checker::{CheckResult, TypeChecker, TypeErrors};
use parser::structs::{LoopData, Type};

impl<'prog> TypeChecker<'prog> {
    // if loop cond present, must be bool. else just check blks.
    // break in a blk is a stmt, is unit type.
    pub(crate) fn check_loop(&mut self, loop_data: &LoopData) -> Result<CheckResult, TypeErrors> {
        let mut ty_errs = TypeErrors::new();

        // if condition: check has type bool. add errs if any
        if let Some(expr) = &loop_data.cond {
            let check_cond = self.check_expr(expr);

            match check_cond {
                Ok(CheckResult {
                    ty: Type::Bool,
                    must_break: _,
                    must_return: _,
                }) => (),
                Ok(ty) => {
                    let e = format!(
                        "Expected type '{}' for loop predicate but got '{}'",
                        Type::Bool,
                        ty.ty
                    );
                    ty_errs.add(&e);
                }
                Err(mut errs) => ty_errs.append(&mut errs),
            }
        }

        let mut check_blk = self.check_block(&loop_data.body, vec![]);
        if let Err(ref mut errs) = check_blk {
            ty_errs.append(errs);
        }

        // TODO: a loop with no cond and no must_break in its block has must_return = true
        if ty_errs.is_ok() {
            Ok(CheckResult {
                ty: Type::Unit,
                must_break: false, // loop never contributes to must_break of outer
                must_return: false,
            })
        } else {
            Err(ty_errs)
        }
    }
}

#[cfg(test)]
mod tests {
    use parser::structs::Type;

    use crate::type_checker::{expect_err, expect_pass};

    #[test]
    fn test_type_check_loop() {
        let t = r"
        loop {

        }
        ";
        expect_pass(t, Type::Unit);

        let t = r"
        loop {
            let x = 0;
            loop x < 5 {
                x = x + 1;
            }
        }
        ";
        expect_pass(t, Type::Unit);

        let t = r"
        let y = 3;
        let z = true;

        loop y < 3 && z == true {
            let x = 0;
            loop x < 5 {
                x = x + 1;
                break;
            }
            y = y + 1;
        }
        ";
        expect_pass(t, Type::Unit);

        // if we break in the branch its ok because it will just jump past loop + ident was not loaded anyway
        let t = r"
        loop {
            let y = if true {
                break;
                3
            } else { 4 };
        }
        ";
        expect_pass(t, Type::Unit);
    }

    #[test]
    fn test_type_check_errs() {
        // cond has errs
        let t = r"
        loop !2 {

        }
        ";
        expect_err(t, "Can't apply logical NOT to type int", true);

        // cond ok but not type bool
        let t = r"
        loop 2+2 - 3 {

        }
        ";
        expect_err(
            t,
            "Expected type 'bool' for loop predicate but got 'int'",
            true,
        );

        // cond and blk have errs
        let t = r"
        loop x < 5 {
            2+false;
        }
        ";
        expect_err(t,  "[TypeError]: Identifier 'x' not declared\n[TypeError]: Can't apply '+' to types 'int' and 'bool'", false);

        // cond ok but not type bool, body has errs
        let t = r"
        loop 2.2 - 3.6 {
            2+false;
        }
        ";
        expect_err(t,  "[TypeError]: Expected type 'bool' for loop predicate but got 'float'\n[TypeError]: Can't apply '+' to types 'int' and 'bool'", false);
    }

    #[test]
    fn test_type_check_loop_edges() {
        // when in loop, break in if else is accepted and the type of the other branch is taken as overall type
        let t = "
        loop {
            let x : int = if true {
                break;
            } else {
                3
            };
        }
       ";
        expect_pass(t, Type::Unit);

        // else break
        let t = "
        loop {
            let x : int = if true {
                3
            } else {
                break;
            };
        }
       ";
        expect_pass(t, Type::Unit);

        // both break
        let t = "
        loop {
            let x : () = if true {
                break;
            } else {
                break;
            };
        }
       ";
        expect_pass(t, Type::Unit);

        // loop stmt doesn't contribute to must_break, so this is checked for mismatch across branches
        let t = r"
        loop {
            let x : int = if true {
               loop { break; }
               
            } else {
               3
            };
       }
        ";
        expect_err(
            t,
            "if-else has type mismatch - consequent: (), alt: int",
            true,
        );
    }
}
