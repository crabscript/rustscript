use std::rc::Rc;

use anyhow::Result;
use bytecode::{type_of, FnType, FrameType, StackFrame, Value};

use crate::{runtime::extend_environment, Runtime, VmError};

use super::apply_builtin::apply_builtin;

/// Call a function with the given number of arguments.
/// First it pops n values from the operand stack where n is the arity of the function.
/// Then it pops the closure from the operand stack.
/// It checks that the closure is a closure and that the arity of the closure matches the number of arguments.
/// It creates a new stack frame with the environment of the closure and the address of the closure.
/// It extends the environment with the parameters and arguments.
/// It sets the program counter to the address of the closure. Essentially calling the function.
///
/// # Arguments
///
/// * `rt` - The runtime to execute the instruction in.
///
/// * `arity` - The number of arguments to pass to the function.
///
/// # Errors
///
/// If the operand stack does not contain enough values to pop (arity + 1).
/// If the closure is not of type closure or the arity of the closure does not match the number of arguments.
pub fn call(rt: &mut Runtime, arity: usize) -> Result<()> {
    let mut args = Vec::new();

    for _ in 0..arity {
        args.push(
            rt.operand_stack
                .pop()
                .ok_or(VmError::OperandStackUnderflow)?,
        );
    }

    let closure = rt
        .operand_stack
        .pop()
        .ok_or(VmError::OperandStackUnderflow)?;

    let Value::Closure {
        fn_type,
        sym,
        prms,
        addr,
        env,
    } = closure
    else {
        return Err(VmError::BadType {
            expected: "Closure".to_string(),
            found: type_of(&closure).to_string(),
        }
        .into());
    };

    if prms.len() != arity {
        return Err(VmError::ArityParamsMismatch {
            arity: prms.len(),
            params: arity,
        }
        .into());
    }

    if let FnType::Builtin = fn_type {
        return apply_builtin(rt, sym.as_str(), args);
    }

    let frame = StackFrame {
        frame_type: FrameType::CallFrame,
        env: Rc::clone(&env.0),
        address: Some(rt.pc),
    };

    rt.runtime_stack.push(frame);
    extend_environment(rt, prms, args)?;
    rt.pc = addr;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytecode::{ByteCode, Environment, FnType, W};

    #[test]
    fn test_call() {
        let mut rt = Runtime::new(vec![ByteCode::CALL(0), ByteCode::DONE]);
        let result = call(&mut rt, 0);

        assert!(result.is_err());

        rt.operand_stack.push(Value::Closure {
            fn_type: FnType::User,
            sym: "Closure".to_string(),
            prms: vec![],
            addr: 123,
            env: W(Environment::new_wrapped()),
        });

        let result = call(&mut rt, 0);
        assert!(result.is_ok());
        assert_eq!(rt.pc, 123);
    }
}
