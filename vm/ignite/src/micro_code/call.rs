use anyhow::Result;
use bytecode::{type_of, FnType, FrameType, StackFrame, Value};

use crate::{extend_environment, Runtime, VmError};

use super::apply_builtin;

/// Call a function with the given number of arguments.
/// First it pops n values from the operand stack where n is the arity of the function.
/// The values will be the arguments to the function and they are pushed to a vector and reversed.
/// i.e. the last argument is the top value of the operand stack.
/// Then it pops the closure from the operand stack.
/// It checks that the closure is a closure and that the arity of the closure matches the number of arguments.
/// If the closure is a builtin function it applies the builtin function and returns.
/// Otherwise it creates a new stack frame with the environment of the closure and the address of the closure.
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
pub fn call(mut rt: Runtime, arity: usize) -> Result<Runtime> {
    let mut args = Vec::new();
    args.reserve_exact(arity);

    for _ in 0..arity {
        args.push(
            rt.current_thread
                .operand_stack
                .pop()
                .ok_or(VmError::OperandStackUnderflow)?,
        );
    }

    args.reverse();

    let value = rt
        .current_thread
        .operand_stack
        .pop()
        .ok_or(VmError::OperandStackUnderflow)?;

    let Value::Closure {
        fn_type,
        sym,
        prms,
        addr,
        env,
    } = value
    else {
        return Err(VmError::BadType {
            expected: "Closure".to_string(),
            found: type_of(&value).to_string(),
        }
        .into());
    };

    if prms.len() != arity {
        return Err(VmError::ArityParamsMismatch {
            arity,
            params: prms.len(),
        }
        .into());
    }

    if let FnType::Builtin = fn_type {
        return apply_builtin(rt, sym.as_str(), args);
    }

    let frame = StackFrame {
        frame_type: FrameType::CallFrame,
        env: env.clone(),
        address: Some(rt.current_thread.pc),
    };

    rt.current_thread.runtime_stack.push(frame);
    rt = extend_environment(rt, env.0, prms, args)?;
    rt.current_thread.pc = addr;

    Ok(rt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytecode::{ByteCode, FnType};

    #[test]
    fn test_call() -> Result<()> {
        let rt = Runtime::new(vec![ByteCode::CALL(0), ByteCode::DONE]);
        let result = call(rt, 0);
        assert!(result.is_err());

        let mut rt = Runtime::new(vec![ByteCode::CALL(0), ByteCode::DONE]);
        rt.current_thread.operand_stack.push(Value::Closure {
            fn_type: FnType::User,
            sym: "Closure".to_string(),
            prms: vec![],
            addr: 123,
            env: Default::default(),
        });

        let rt = call(rt, 0)?;
        assert_eq!(rt.current_thread.pc, 123);

        Ok(())
    }
}
