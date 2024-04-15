use crate::type_checker::{TypeChecker, TypeErrors};
use parser::structs::{LoopData, Type};

impl<'prog> TypeChecker<'prog> {
    // if loop cond present, must be bool. else just check blks.
    // break in a blk is a stmt, is unit type.
    pub(crate) fn check_loop(&mut self, loop_data: &LoopData) -> Result<(), TypeErrors> {
        let mut ty_errs = TypeErrors::new();

        // if condition: check has type bool. add errs if any
        if let Some(expr) = &loop_data.cond {
            let check_cond = self.check_expr(expr);

            match check_cond {
                Ok(Type::Bool) => (),
                Ok(ty) => {
                    let e = format!(
                        "Expected type '{}' for loop predicate but got '{}'",
                        Type::Bool,
                        ty
                    );
                    ty_errs.add(&e);
                }
                Err(mut errs) => ty_errs.append(&mut errs),
            }
        }

        let mut check_blk = self.check_block(&loop_data.body);
        if let Err(ref mut errs) = check_blk {
            ty_errs.append(errs);
        }

        if ty_errs.is_ok() {
            Ok(())
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
}
