use crate::type_checker::{CheckResult, TypeChecker, TypeErrors};
use parser::structs::{FnCallData, Type};

// Ideally these constants should be shared across type checker and VM but I don't want to waste time refactoring
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

    fn get_type_string(arg_types: &[Type]) -> String {
        let r = arg_types
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        format!("({})", r)
    }

    /// Just checks for length mismatch
    fn check_arg_params_len(
        fn_name: &str,
        arg_len: usize,
        exp_len: usize,
    ) -> Result<(), TypeErrors> {
        if arg_len != exp_len {
            let e = format!(
                "Function '{}' takes {} arguments but {} were supplied",
                fn_name, exp_len, arg_len
            );
            return Err(TypeErrors::new_err(&e));
        }

        Ok(())
    }

    /// Check if a arg type match given vector of param types. If not, throw a suitable error - report length mismatch or
    /// type mismatch.
    pub(crate) fn check_arg_params_match(
        fn_name: &str,
        arg_types: &[Type],
        param_types: &[Type],
    ) -> Result<(), TypeErrors> {
        TypeChecker::check_arg_params_len(fn_name, arg_types.len(), param_types.len())?;

        let mut mismatch = false;
        for (arg, param) in arg_types.iter().zip(param_types.iter()) {
            if *arg != *param {
                mismatch = true;
                break;
            }
        }

        if mismatch {
            let error_msg = format!(
                "Mismatched types in function call: got ({}) but expected ({})",
                TypeChecker::get_type_string(arg_types),
                TypeChecker::get_type_string(param_types),
            );
            return Err(TypeErrors::new_err(&error_msg));
        }

        Ok(())
    }

    // Passed in CheckResult is the accumulated CheckResult from arg checks with ty = Type::Unit
    // arg can be a block, block can have break/return
    pub(crate) fn check_builtin_fn_call(
        &mut self,
        name: &str,
        arg_types: Vec<Type>,
        mut check_res: CheckResult,
    ) -> Result<CheckResult, TypeErrors> {
        check_res.ty = match name {
            // () -> string
            READ_LINE => {
                TypeChecker::check_arg_params_match(name, &arg_types, &[])?;
                Type::String
            }
            // (any) -> ()
            PRINT => {
                TypeChecker::check_arg_params_len(name, arg_types.len(), 1)?;
                Type::Unit
            }
            // (any) -> ()
            PRINTLN => {
                TypeChecker::check_arg_params_len(name, arg_types.len(), 1)?;
                Type::Unit
            }
            // (string) => int
            STRING_LEN => {
                TypeChecker::check_arg_params_match(name, &arg_types, &[Type::String])?;
                Type::Int
            }
            // (int, int) => int or (float, float) => float
            MIN => {
                TypeChecker::check_arg_params_len(name, arg_types.len(), 2)?;
                match (arg_types.first().unwrap(), arg_types.get(1).unwrap()) {
                    (Type::Int, Type::Int) => Type::Int,
                    (Type::Float, Type::Float) => Type::Float,
                    _ => {
                        let e = format!(
                            "Expected (int, int) or (float, float) but got {}",
                            TypeChecker::get_type_string(&arg_types)
                        );
                        return Err(TypeErrors::new_err(&e));
                    }
                }
            }
            // Same as min
            MAX => {
                TypeChecker::check_arg_params_len(name, arg_types.len(), 2)?;
                match (arg_types.first().unwrap(), arg_types.get(1).unwrap()) {
                    (Type::Int, Type::Int) => Type::Int,
                    (Type::Float, Type::Float) => Type::Float,
                    _ => {
                        let e = format!(
                            "Expected (int, int) or (float, float) but got {}",
                            TypeChecker::get_type_string(&arg_types)
                        );
                        return Err(TypeErrors::new_err(&e));
                    }
                }
            }
            // int or float => same type
            ABS => {
                TypeChecker::check_arg_params_len(name, arg_types.len(), 1)?;
                match arg_types.first().unwrap() {
                    Type::Int => Type::Int,
                    Type::Float => Type::Float,
                    _ => {
                        let e = format!(
                            "Expected int or float but got {}",
                            TypeChecker::get_type_string(&arg_types)
                        );
                        return Err(TypeErrors::new_err(&e));
                    }
                }
            }
            // float -> float
            COS => {
                TypeChecker::check_arg_params_len(name, arg_types.len(), 1)?;
                match arg_types.first().unwrap() {
                    Type::Float => Type::Float,
                    _ => {
                        let e = format!(
                            "Expected float but got {}",
                            TypeChecker::get_type_string(&arg_types)
                        );
                        return Err(TypeErrors::new_err(&e));
                    }
                }
            }
            // float -> float
            SIN => {
                TypeChecker::check_arg_params_len(name, arg_types.len(), 1)?;
                match arg_types.first().unwrap() {
                    Type::Float => Type::Float,
                    _ => {
                        let e = format!(
                            "Expected float but got {}",
                            TypeChecker::get_type_string(&arg_types)
                        );
                        return Err(TypeErrors::new_err(&e));
                    }
                }
            }
            // float -> float
            TAN => {
                TypeChecker::check_arg_params_len(name, arg_types.len(), 1)?;
                match arg_types.first().unwrap() {
                    Type::Float => Type::Float,
                    _ => {
                        let e = format!(
                            "Expected float but got {}",
                            TypeChecker::get_type_string(&arg_types)
                        );
                        return Err(TypeErrors::new_err(&e));
                    }
                }
            }
            // float -> float
            SQRT => {
                TypeChecker::check_arg_params_len(name, arg_types.len(), 1)?;
                match arg_types.first().unwrap() {
                    Type::Float => Type::Float,
                    _ => {
                        let e = format!(
                            "Expected float but got {}",
                            TypeChecker::get_type_string(&arg_types)
                        );
                        return Err(TypeErrors::new_err(&e));
                    }
                }
            }
            // float -> float
            LOG => {
                TypeChecker::check_arg_params_len(name, arg_types.len(), 1)?;
                match arg_types.first().unwrap() {
                    Type::Float => Type::Float,
                    _ => {
                        let e = format!(
                            "Expected float but got {}",
                            TypeChecker::get_type_string(&arg_types)
                        );
                        return Err(TypeErrors::new_err(&e));
                    }
                }
            }
            // float, float => float
            POW => {
                TypeChecker::check_arg_params_len(name, arg_types.len(), 2)?;
                match (arg_types.first().unwrap(), arg_types.get(1).unwrap()) {
                    (Type::Float, Type::Float) => Type::Float,
                    _ => {
                        let e = format!(
                            "Expected (float, float) but got {}",
                            TypeChecker::get_type_string(&arg_types)
                        );
                        return Err(TypeErrors::new_err(&e));
                    }
                }
            }
            // int -> string
            ITOA => {
                TypeChecker::check_arg_params_len(name, arg_types.len(), 1)?;
                match arg_types.first().unwrap() {
                    Type::Int => Type::String,
                    _ => {
                        let e = format!(
                            "Expected int but got {}",
                            TypeChecker::get_type_string(&arg_types)
                        );
                        return Err(TypeErrors::new_err(&e));
                    }
                }
            }
            // string -> int
            ATOI => {
                TypeChecker::check_arg_params_len(name, arg_types.len(), 1)?;
                match arg_types.first().unwrap() {
                    Type::String => Type::Int,
                    _ => {
                        let e = format!(
                            "Expected string but got {}",
                            TypeChecker::get_type_string(&arg_types)
                        );
                        return Err(TypeErrors::new_err(&e));
                    }
                }
            }
            // float -> int
            FLOAT_TO_INT => {
                TypeChecker::check_arg_params_len(name, arg_types.len(), 1)?;
                match arg_types.first().unwrap() {
                    Type::Float => Type::Int,
                    _ => {
                        let e = format!(
                            "Expected float but got {}",
                            TypeChecker::get_type_string(&arg_types)
                        );
                        return Err(TypeErrors::new_err(&e));
                    }
                }
            }
            // int -> float
            INT_TO_FLOAT => {
                TypeChecker::check_arg_params_len(name, arg_types.len(), 1)?;
                match arg_types.first().unwrap() {
                    Type::Int => Type::Float,
                    _ => {
                        let e = format!(
                            "Expected int but got {}",
                            TypeChecker::get_type_string(&arg_types)
                        );
                        return Err(TypeErrors::new_err(&e));
                    }
                }
            }
            SEM_CREATE => {
                // Fill out this block
                todo!()
            }
            SEM_SET => {
                // Fill out this block
                todo!()
            }
            _ => todo!(),
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

    use super::BUILTINS;

    #[test]
    fn test_type_check_builtin_sym() {
        for &builtin in BUILTINS.iter() {
            expect_pass(builtin, Type::BuiltInFn);
        }
    }

    #[test]
    fn test_type_check_builtin_functions() {
        expect_pass("let x : () = print(2); x", Type::Unit);

        // Test min
        expect_pass("let x : int = min(2, 3); x", Type::Int);
        expect_pass("let x : float = min(2.0, 3.0); x", Type::Float);

        // Test max
        expect_pass("let x : int = max(2, 3); x", Type::Int);
        expect_pass("let x : float = max(2.0, 3.0); x", Type::Float);

        // Test abs
        expect_pass("let x : int = abs(-5); x", Type::Int);
        expect_pass("let x : float = abs(-5.0); x", Type::Float);

        // Test cos
        expect_pass("let x : float = cos(0.0); x", Type::Float);

        // Test sin
        expect_pass("let x : float = sin(0.0); x", Type::Float);

        // Test tan
        expect_pass("let x : float = tan(0.0); x", Type::Float);

        // Test sqrt
        expect_pass("let x : float = sqrt(4.0); x", Type::Float);

        // Test log
        expect_pass("let x : float = log(1.0); x", Type::Float);

        // Test pow
        expect_pass("let x : float = pow(2.0, 3.0); x", Type::Float);

        // Test itoa
        // expect_pass("let x : string = itoa(123); x", Type::String);

        // Test atoi
        // expect_pass("let x : int = atoi(\"123\"); x", Type::Int);

        // Test float_to_int
        expect_pass("let x : int = float_to_int(3.5); x", Type::Int);

        // Test int_to_float
        expect_pass("let x : float = int_to_float(3); x", Type::Float);
    }
}
