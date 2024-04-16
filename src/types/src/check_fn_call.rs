use crate::type_checker::{CheckResult, TypeChecker, TypeErrors};
use parser::structs::{FnCallData, Type};

// const BUILTINS: [&str; 19] = ["read_line", "print", "println", "string_len", "min", "max", "abs", "cos",
// "sin", "tan", "sqrt", "log", "pow", "itoa", "atoi", "float_to_int", "int_to_float", "sem_create", "sem_set"];

const READ_LINE: &str = "read_line";
const PRINT: &str = "print";
const PRINTLN: &str = "println";
const STRING_LEN: &str = "string_len";
const MIN: &str = "min";
const MAX: &str = "max";
const ABS: &str = "abs";
const COS: &str = "cos";
const SIN: &str = "sin";
const TAN: &str = "tan";
const SQRT: &str = "sqrt";
const LOG: &str = "log";
const POW: &str = "pow";
const ITOA: &str = "itoa";
const ATOI: &str = "atoi";
const FLOAT_TO_INT: &str = "float_to_int";
const INT_TO_FLOAT: &str = "int_to_float";
const SEM_CREATE: &str = "sem_create";
const SEM_SET: &str = "sem_set";

const BUILTINS: [&str; 19] = [
    READ_LINE,
    PRINT,
    PRINTLN,
    STRING_LEN,
    MIN,
    MAX,
    ABS,
    COS,
    SIN,
    TAN,
    SQRT,
    LOG,
    POW,
    ITOA,
    ATOI,
    FLOAT_TO_INT,
    INT_TO_FLOAT,
    SEM_CREATE,
    SEM_SET,
];

impl<'prog> TypeChecker<'prog> {
    /// Check if name is a builtin function
    pub(crate) fn is_builtin_fn(name: &str) -> bool {
        BUILTINS.contains(&name)
    }

    // Passed in CheckResult is the accumulated CheckResult from arg checks with ty = Type::Unit
    // arg can be a block, block can have break/return
    pub(crate) fn check_builtin_fn_call(
        &mut self,
        name: &str,
        arg_types: Vec<Type>,
        check_res: CheckResult,
    ) -> Result<CheckResult, TypeErrors> {
        dbg!("BUILTIN_CHK:", &arg_types);
        match name {
            READ_LINE => {
                // Fill out this block
            }
            PRINT => {
                // Fill out this block
            }
            PRINTLN => {
                // Fill out this block
            }
            STRING_LEN => {
                // Fill out this block
            }
            MIN => {
                // Fill out this block
            }
            MAX => {
                // Fill out this block
            }
            ABS => {
                // Fill out this block
            }
            COS => {
                // Fill out this block
            }
            SIN => {
                // Fill out this block
            }
            TAN => {
                // Fill out this block
            }
            SQRT => {
                // Fill out this block
            }
            LOG => {
                // Fill out this block
            }
            POW => {
                // Fill out this block
            }
            ITOA => {
                // Fill out this block
            }
            ATOI => {
                // Fill out this block
            }
            FLOAT_TO_INT => {
                // Fill out this block
            }
            INT_TO_FLOAT => {
                // Fill out this block
            }
            SEM_CREATE => {
                // Fill out this block
            }
            SEM_SET => {
                // Fill out this block
            }
            _ => (),
        };

        Ok(check_res)
    }

    // Accumulate errors from the expressions. Propagate must_break, must_return
    pub(crate) fn check_fn_call(
        &mut self,
        fn_call: &FnCallData,
    ) -> Result<CheckResult, TypeErrors> {
        let mut ty_errs = TypeErrors::new();

        let mut check_res = CheckResult {
            ty: Type::Unit,
            must_break: false,
            must_return: false,
        };

        // types of the args in order
        let mut arg_types: Vec<Type> = vec![];

        // collect errors and keep mutating check_res
        for arg in fn_call.args.iter() {
            let check_arg = self.check_expr(arg);
            match check_arg {
                Ok(arg_res) => {
                    check_res = CheckResult::combine(&check_res, &arg_res);
                    arg_types.push(arg_res.ty);
                }
                // add errors for each expr if any
                Err(mut errs) => {
                    ty_errs.append(&mut errs);
                }
            }
        }

        // if errs for args, return out. can't check func call is correct
        if !ty_errs.is_ok() {
            return Err(ty_errs);
        }

        if TypeChecker::is_builtin_fn(&fn_call.name) {
            return self.check_builtin_fn_call(&fn_call.name, arg_types, check_res);
        }

        Ok(check_res)
    }
}

#[cfg(test)]
mod tests {
    use parser::structs::Type;

    use crate::type_checker::expect_pass;

    use super::{BUILTINS, READ_LINE};

    #[test]
    fn test_builtin_sym() {
        for &builtin in BUILTINS.iter() {
            expect_pass(builtin, Type::BuiltInFn);
        }
    }
}
