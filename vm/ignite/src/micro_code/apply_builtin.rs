use anyhow::Result;
use bytecode::{builtin, Value};

use crate::{Runtime, VmError};

pub fn apply_builtin(mut rt: Runtime, sym: &str, args: Vec<Value>) -> Result<Runtime> {
    match sym {
        builtin::READ_LINE_SYM => {
            let input = builtin::read_line_impl()?;
            rt.current_thread.operand_stack.push(Value::String(input));
        }
        builtin::PRINT_SYM => {
            for arg in args {
                builtin::print_impl(&arg);
            }
        }
        builtin::PRINTLN_SYM => {
            for arg in args[..args.len() - 1].iter() {
                builtin::print_impl(arg);
            }
            if let Some(arg) = args.last() {
                builtin::println_impl(arg);
            }
        }
        builtin::STRING_LEN_SYM => {
            let s = args.first().ok_or(VmError::InsufficientArguments {
                expected: 1,
                got: args.len(),
            })?;

            let len = builtin::string_len_impl(s)?;
            rt.current_thread.operand_stack.push(Value::Int(len as i64));
        }
        builtin::MIN_SYM => {
            let v1 = args.first().ok_or(VmError::InsufficientArguments {
                expected: 2,
                got: args.len(),
            })?;
            let v2 = args.get(1).ok_or(VmError::InsufficientArguments {
                expected: 2,
                got: args.len(),
            })?;

            let min = builtin::min_impl(v1, v2)?;
            rt.current_thread.operand_stack.push(min);
        }
        builtin::MAX_SYM => {
            let v1 = args.first().ok_or(VmError::InsufficientArguments {
                expected: 2,
                got: args.len(),
            })?;
            let v2 = args.get(1).ok_or(VmError::InsufficientArguments {
                expected: 2,
                got: args.len(),
            })?;

            let max = builtin::max_impl(v1, v2)?;
            rt.current_thread.operand_stack.push(max);
        }
        builtin::ABS_SYM => {
            let x = args.first().ok_or(VmError::InsufficientArguments {
                expected: 1,
                got: args.len(),
            })?;

            let abs = builtin::abs_impl(x)?;
            rt.current_thread.operand_stack.push(abs);
        }
        builtin::COS_SYM => {
            let x = args.first().ok_or(VmError::InsufficientArguments {
                expected: 1,
                got: args.len(),
            })?;

            let cos = builtin::cos_impl(x)?;
            rt.current_thread.operand_stack.push(cos);
        }
        builtin::SIN_SYM => {
            let x = args.first().ok_or(VmError::InsufficientArguments {
                expected: 1,
                got: args.len(),
            })?;

            let sin = builtin::sin_impl(x)?;
            rt.current_thread.operand_stack.push(sin);
        }
        builtin::TAN_SYM => {
            let x = args.first().ok_or(VmError::InsufficientArguments {
                expected: 1,
                got: args.len(),
            })?;

            let tan = builtin::tan_impl(x)?;
            rt.current_thread.operand_stack.push(tan);
        }
        builtin::SQRT_SYM => {
            let x = args.first().ok_or(VmError::InsufficientArguments {
                expected: 1,
                got: args.len(),
            })?;

            let sqrt = builtin::sqrt_impl(x)?;
            rt.current_thread.operand_stack.push(sqrt);
        }
        builtin::LOG_SYM => {
            let x = args.first().ok_or(VmError::InsufficientArguments {
                expected: 1,
                got: args.len(),
            })?;

            let log = builtin::log_impl(x)?;
            rt.current_thread.operand_stack.push(log);
        }
        builtin::POW_SYM => {
            dbg!(&args);
            let x = args.first().ok_or(VmError::InsufficientArguments {
                expected: 2,
                got: args.len(),
            })?;
            let y = args.get(1).ok_or(VmError::InsufficientArguments {
                expected: 2,
                got: args.len(),
            })?;

            let pow = builtin::pow_impl(x, y)?;
            rt.current_thread.operand_stack.push(pow);
        }
        builtin::ITOA_SYM => {
            let x = args.first().ok_or(VmError::InsufficientArguments {
                expected: 1,
                got: args.len(),
            })?;

            let itoa = builtin::itoa_impl(x)?;
            rt.current_thread.operand_stack.push(itoa);
        }
        builtin::ATOI_SYM => {
            let s = args.first().ok_or(VmError::InsufficientArguments {
                expected: 1,
                got: args.len(),
            })?;

            let atoi = builtin::atoi_impl(s)?;
            rt.current_thread.operand_stack.push(atoi);
        }
        builtin::FLOAT_TO_INT_SYM => {
            let x = args.first().ok_or(VmError::InsufficientArguments {
                expected: 1,
                got: args.len(),
            })?;

            let float_to_int = builtin::float_to_int_impl(x)?;
            rt.current_thread.operand_stack.push(float_to_int);
        }
        builtin::INT_TO_FLOAT_SYM => {
            let x = args.first().ok_or(VmError::InsufficientArguments {
                expected: 1,
                got: args.len(),
            })?;

            let int_to_float = builtin::int_to_float_impl(x)?;
            rt.current_thread.operand_stack.push(int_to_float);
        }
        builtin::SEM_CREATE_SYM => {
            let sem = builtin::sem_create_impl();
            rt.current_thread.operand_stack.push(sem);
        }
        builtin::SEM_SET_SYM => {
            let sem = args.first().ok_or(VmError::InsufficientArguments {
                expected: 2,
                got: args.len(),
            })?;
            let val = args.get(1).ok_or(VmError::InsufficientArguments {
                expected: 2,
                got: args.len(),
            })?;

            builtin::sem_set_impl(sem, val)?;
        }
        _ => {
            return Err(VmError::UnknownBuiltin {
                sym: sym.to_string(),
            }
            .into());
        }
    }

    Ok(rt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Ok;
    use bytecode::{builtin::*, type_of, Semaphore};

    #[test]
    fn test_apply_builtin() -> Result<()> {
        let mut rt = Runtime::default();
        let hello_world = "Hello, world!".to_string();

        // Stdout
        let sym = PRINT_SYM;
        let args = vec![Value::String(hello_world.clone())];
        println!("Expect to see 'Hello, world!':");
        rt = apply_builtin(rt, sym, args)?;
        println!();

        let sym = PRINTLN_SYM;
        let args = vec![Value::String(hello_world.clone())];
        println!("Expect to see 'Hello, world!':");
        rt = apply_builtin(rt, sym, args)?;

        let sym = STRING_LEN_SYM;
        let args = vec![Value::String(hello_world.clone())];
        rt = apply_builtin(rt, sym, args)?;
        assert_eq!(
            Value::Int(hello_world.clone().len() as i64),
            rt.current_thread.operand_stack.pop().unwrap()
        );

        // Conv
        let sym = INT_TO_FLOAT_SYM;
        let args = vec![Value::Int(42)];
        rt = apply_builtin(rt, sym, args)?;

        let expected = Value::Float(42.0);
        let actual = rt.current_thread.operand_stack.pop().unwrap();
        assert_eq!(expected, actual);

        let sym = FLOAT_TO_INT_SYM;
        let args = vec![Value::Float(42.0)];
        rt = apply_builtin(rt, sym, args)?;

        let expected = Value::Int(42);
        let actual = rt.current_thread.operand_stack.pop().unwrap();
        assert_eq!(expected, actual);

        let sym = ATOI_SYM;
        let args = vec![Value::String("42".to_string())];
        rt = apply_builtin(rt, sym, args)?;
        assert_eq!(
            Value::Int(42),
            rt.current_thread.operand_stack.pop().unwrap()
        );

        let args: Vec<Value> = vec![Value::String("forty-two".to_string())];
        let result = apply_builtin(rt, sym, args);
        assert!(result.is_err());

        let mut rt = Runtime::default();
        let sym = ITOA_SYM;
        let args = vec![Value::Int(42)];
        rt = apply_builtin(rt, sym, args)?;
        assert_eq!(
            Value::String("42".to_string()),
            rt.current_thread.operand_stack.pop().unwrap()
        );

        // Math
        let sym = MIN_SYM;
        let args = vec![Value::Int(42), Value::Int(24)];
        rt = apply_builtin(rt, sym, args)?;
        assert_eq!(
            Value::Int(24),
            rt.current_thread.operand_stack.pop().unwrap()
        );

        let args = vec![Value::Float(42.0), Value::Float(24.0)];
        rt = apply_builtin(rt, sym, args)?;
        assert_eq!(
            Value::Float(24.0),
            rt.current_thread.operand_stack.pop().unwrap()
        );

        let sym = MAX_SYM;
        let args = vec![Value::Int(42), Value::Int(24)];
        rt = apply_builtin(rt, sym, args)?;
        assert_eq!(
            Value::Int(42),
            rt.current_thread.operand_stack.pop().unwrap()
        );

        let args = vec![Value::Float(42.0), Value::Float(24.0)];
        rt = apply_builtin(rt, sym, args)?;
        assert_eq!(
            Value::Float(42.0),
            rt.current_thread.operand_stack.pop().unwrap()
        );

        let sym = ABS_SYM;
        let args = vec![Value::Int(-42)];
        rt = apply_builtin(rt, sym, args)?;
        assert_eq!(
            Value::Int(42),
            rt.current_thread.operand_stack.pop().unwrap()
        );

        let args = vec![Value::Float(-42.0)];
        rt = apply_builtin(rt, sym, args)?;
        assert_eq!(
            Value::Float(42.0),
            rt.current_thread.operand_stack.pop().unwrap()
        );

        let sym = COS_SYM;
        let args = vec![Value::Float(0.0)];
        rt = apply_builtin(rt, sym, args)?;
        assert_eq!(
            Value::Float(0.0_f64.cos()),
            rt.current_thread.operand_stack.pop().unwrap()
        );

        let args = vec![Value::Float(std::f64::consts::PI)];
        rt = apply_builtin(rt, sym, args)?;
        assert_eq!(
            Value::Float(std::f64::consts::PI.cos()),
            rt.current_thread.operand_stack.pop().unwrap()
        );

        let sym = SIN_SYM;
        let args = vec![Value::Float(0.0)];
        rt = apply_builtin(rt, sym, args)?;
        assert_eq!(
            Value::Float(0.0),
            rt.current_thread.operand_stack.pop().unwrap()
        );

        let args = vec![Value::Float(std::f64::consts::PI)];
        rt = apply_builtin(rt, sym, args)?;
        assert_eq!(
            Value::Float(std::f64::consts::PI.sin()),
            rt.current_thread.operand_stack.pop().unwrap()
        );

        let sym = TAN_SYM;
        let args = vec![Value::Float(0.0)];
        rt = apply_builtin(rt, sym, args)?;
        assert_eq!(
            Value::Float(0.0),
            rt.current_thread.operand_stack.pop().unwrap()
        );

        let args = vec![Value::Float(std::f64::consts::PI)];
        rt = apply_builtin(rt, sym, args)?;
        assert_eq!(
            Value::Float(std::f64::consts::PI.tan()),
            rt.current_thread.operand_stack.pop().unwrap()
        );

        let sym = SQRT_SYM;
        let args = vec![Value::Float(42.0)];
        rt = apply_builtin(rt, sym, args)?;
        assert_eq!(
            Value::Float(42.0_f64.sqrt()),
            rt.current_thread.operand_stack.pop().unwrap()
        );

        let args = vec![Value::Float(102934.0)];
        rt = apply_builtin(rt, sym, args)?;
        assert_eq!(
            Value::Float(102934.0_f64.sqrt()),
            rt.current_thread.operand_stack.pop().unwrap()
        );

        let sym = POW_SYM;
        let args = vec![Value::Float(2.0), Value::Float(3.0)];
        rt = apply_builtin(rt, sym, args)?;
        assert_eq!(
            Value::Float(2.0_f64.powf(3.0)),
            rt.current_thread.operand_stack.pop().unwrap()
        );

        let args = vec![Value::Float(2.0), Value::Int(3)];
        let result = apply_builtin(rt, sym, args);
        assert!(result.is_err());

        let mut rt = Runtime::default();
        let sym = LOG_SYM;
        let args = vec![Value::Float(42.0)];
        rt = apply_builtin(rt, sym, args)?;
        assert_eq!(
            Value::Float(42.0_f64.log(10.0)),
            rt.current_thread.operand_stack.pop().unwrap()
        );

        let sym = SEM_CREATE_SYM;
        let args = vec![];
        rt = apply_builtin(rt, sym, args)?;
        assert_eq!(
            type_of(&Value::Semaphore(Semaphore::default())),
            type_of(&rt.current_thread.operand_stack.pop().unwrap())
        );

        let sym = SEM_SET_SYM;
        let sem = Semaphore::default();
        let args = vec![sem.clone().into(), Value::Int(42)];
        _ = apply_builtin(rt, sym, args)?;
        let sem_guard = sem.lock().unwrap();
        assert_eq!(42, *sem_guard);

        Ok(())
    }
}
