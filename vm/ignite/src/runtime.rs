use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use anyhow::Result;
use bytecode::ByteCode;

use crate::{micro_code, Thread, ThreadState, VmError};

const DEFAULT_TIME_QUANTUM: Duration = Duration::from_millis(100);
const MAIN_THREAD_ID: i64 = 1;

/// The runtime of the virtual machine.
/// It contains the instructions to execute, the current thread, and the ready and suspended threads.
/// The instructions are the bytecode instructions to execute.
/// The ready queue is a queue of threads that are ready to run.
/// The suspended queue is a queue of threads that are waiting for some event to occur.
/// The running thread is the thread that is currently executing.
pub struct Runtime {
    time: Instant,
    time_quantum: Duration,
    pub instrs: Vec<ByteCode>,
    pub thread_count: i64,
    pub current_thread: Thread,
    pub ready_queue: VecDeque<Thread>,
    pub suspended_queue: VecDeque<Thread>,
}

impl Runtime {
    pub fn new(instrs: Vec<ByteCode>) -> Self {
        Runtime {
            time: Instant::now(),
            time_quantum: DEFAULT_TIME_QUANTUM,
            instrs,
            thread_count: 1,
            current_thread: Thread::new(MAIN_THREAD_ID),
            ready_queue: VecDeque::new(),
            suspended_queue: VecDeque::new(),
        }
    }

    pub fn set_time_quantum(&mut self, time_quantum: Duration) {
        self.time_quantum = time_quantum;
    }
}

/// Run the program until it is done.
///
/// # Arguments
///
/// * `rt` - The runtime to run.
///
/// # Returns
///
/// The runtime after the program has finished executing.
///
/// # Errors
///
/// If an error occurs during execution.
pub fn run(mut rt: Runtime) -> Result<Runtime> {
    loop {
        if rt.time_quantum_expired() {
            rt = rt.yield_current_thread();
        }

        if rt.should_yield_current_thread() {
            rt = rt.yield_current_thread();
        }

        if rt.is_current_thread_joining() {
            rt = rt.join_current_thread();
        }

        let instr = rt.fetch_instr()?;

        execute(&mut rt, instr)?;

        if !rt.is_current_thread_done() {
            continue;
        }

        if !rt.is_current_main_thread() {
            rt = rt.drop_current_thread();
            continue;
        }

        // If the main thread is done, then the program is done.
        break;
    }

    Ok(rt)
}

/// Execute a single instruction, returning whether the program is done.
///
/// # Arguments
///
/// * `rt` - The runtime to execute the instruction on.
///
/// * `instr` - The instruction to execute.
///
/// # Returns
///
/// Whether the program is done executing.
///
/// # Errors
///
/// If an error occurs during execution.
pub fn execute(rt: &mut Runtime, instr: ByteCode) -> Result<()> {
    match instr {
        ByteCode::DONE => micro_code::done(rt)?,
        ByteCode::ASSIGN(sym) => micro_code::assign(rt, sym)?,
        ByteCode::LD(sym) => micro_code::ld(rt, sym)?,
        ByteCode::LDC(val) => micro_code::ldc(rt, val)?,
        ByteCode::LDF(addr, prms) => micro_code::ldf(rt, addr, prms)?,
        ByteCode::POP => micro_code::pop(rt)?,
        ByteCode::UNOP(op) => micro_code::unop(rt, op)?,
        ByteCode::BINOP(op) => micro_code::binop(rt, op)?,
        ByteCode::JOF(pc) => micro_code::jof(rt, pc)?,
        ByteCode::GOTO(pc) => micro_code::goto(rt, pc)?,
        ByteCode::RESET(ft) => micro_code::reset(rt, ft)?,
        ByteCode::ENTERSCOPE(syms) => micro_code::enter_scope(rt, syms)?,
        ByteCode::EXITSCOPE => micro_code::exit_scope(rt)?,
        ByteCode::CALL(arity) => micro_code::call(rt, arity)?,
        ByteCode::SPAWN => micro_code::spawn(rt)?,
        ByteCode::JOIN(tid) => micro_code::join(rt, tid)?,
        ByteCode::YIELD => micro_code::yield_(rt)?,
    }
    Ok(())
}

impl Runtime {
    /// Check if the time quantum has expired.
    /// The time quantum is the maximum amount of time a thread can run before it is preempted.
    pub fn time_quantum_expired(&self) -> bool {
        self.time.elapsed() >= self.time_quantum
    }

    /// Check if the current thread should yield.
    /// This is set by the `YIELD` instruction.
    pub fn should_yield_current_thread(&self) -> bool {
        self.current_thread.state == ThreadState::Yielded
    }

    /// Yield the current thread. Set the state of the current thread to `Ready` and push it onto the ready queue.
    /// Pop the next thread from the ready queue and set it as the current thread.
    /// The timer is reset to the current time.
    pub fn yield_current_thread(mut self) -> Self {
        let mut current_thread = self.current_thread;
        current_thread.state = ThreadState::Ready; // Reset the state
        self.ready_queue.push_back(current_thread);

        let next_ready_thread = self
            .ready_queue
            .pop_front()
            .expect("No threads in ready queue");

        self.current_thread = next_ready_thread;
        self.time = Instant::now(); // Reset the time
        self
    }

    pub fn drop_current_thread(mut self) -> Self {
        let next_ready_thread = self
            .ready_queue
            .pop_front()
            .expect("No threads in ready queue");

        self.current_thread = next_ready_thread;
        self
    }

    pub fn is_current_thread_done(&self) -> bool {
        self.current_thread.state == ThreadState::Done
    }

    pub fn is_current_main_thread(&self) -> bool {
        self.current_thread.thread_id == MAIN_THREAD_ID
    }

    pub fn is_current_thread_joining(&self) -> bool {
        matches!(self.current_thread.state, ThreadState::Joining(_))
    }

    /// Join the current thread with the thread with the given ThreadID based on the current thread's state.
    /// If the thread to join is in the ready or suspended queue, then the current thread will yield.
    /// Otherwise, the current thread will be set to ready.
    pub fn join_current_thread(mut self) -> Self {
        if let ThreadState::Joining(tid) = self.current_thread.state {
            let thread_to_join = self
                .ready_queue
                .iter()
                .chain(self.suspended_queue.iter())
                .find(|t| t.thread_id == tid);

            if thread_to_join.is_some() {
                // If the thread to join in the ready or suspended queue, then we need to yield the current thread.
                let rt = self.yield_current_thread();
                return rt;
            };

            // Otherwise we can just set the current thread to ready.
            self.current_thread.state = ThreadState::Ready;
        } else {
            panic!("Current thread is not joining");
        }

        self
    }
}

impl Runtime {
    /// Fetch the next instruction to execute.
    /// This will increment the program counter of the current thread.
    ///
    /// # Returns
    ///
    /// The next instruction to execute.
    ///
    /// # Errors
    ///
    /// If the program counter is out of bounds.
    pub fn fetch_instr(&mut self) -> Result<ByteCode> {
        let instr = self
            .instrs
            .get(self.current_thread.pc)
            .cloned()
            .ok_or(VmError::PcOutOfBounds(self.current_thread.pc))?;
        self.current_thread.pc += 1;
        Ok(instr)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use anyhow::{Ok, Result};
    use bytecode::{builtin, BinOp, ByteCode, FrameType, UnOp, Value};

    #[test]
    fn test_pc() {
        let instrs = vec![
            ByteCode::ldc(42),
            ByteCode::POP,
            ByteCode::ldc(42),
            ByteCode::POP,
            ByteCode::DONE,
        ];
        let rt = Runtime::new(instrs);
        let rt = run(rt).unwrap();
        assert_eq!(rt.current_thread.pc, 5);

        let rt = Runtime::new(vec![
            ByteCode::ldc(false),
            ByteCode::JOF(3),
            ByteCode::POP, // This will panic since there is no value on the stack
            ByteCode::DONE,
        ]);
        let rt = run(rt).unwrap();
        assert_eq!(rt.current_thread.pc, 4);

        let rt = Runtime::new(vec![
            ByteCode::ldc(true),
            ByteCode::JOF(3), // jump to pop instruction
            ByteCode::DONE,
            ByteCode::POP, // This will panic since there is no value on the stack
            ByteCode::DONE,
        ]);
        let rt = run(rt).unwrap();
        assert_eq!(rt.current_thread.pc, 3);

        let rt = Runtime::new(vec![
            ByteCode::GOTO(2),
            ByteCode::POP, // This will panic since there is no value on the stack
            ByteCode::DONE,
        ]);
        let rt = run(rt).unwrap();
        assert_eq!(rt.current_thread.pc, 3);
    }

    #[test]
    fn test_arithmetic() {
        // 42 + 42
        let instrs = vec![
            ByteCode::ldc(42),
            ByteCode::ldc(42),
            ByteCode::BINOP(BinOp::Add),
            ByteCode::DONE,
        ];
        let rt = Runtime::new(instrs);
        let rt = run(rt).unwrap();
        assert_eq!(rt.current_thread.operand_stack, vec![Value::Int(84)]);

        // -(42 - 123)
        let instrs = vec![
            ByteCode::ldc(42),
            ByteCode::ldc(123),
            ByteCode::BINOP(BinOp::Sub),
            ByteCode::UNOP(UnOp::Neg),
            ByteCode::DONE,
        ];
        let rt = Runtime::new(instrs);
        let rt = run(rt).unwrap();
        assert_eq!(rt.current_thread.operand_stack, vec![Value::Int(81)]);

        // (2 * 3) > 9
        let instrs = vec![
            ByteCode::ldc(2),
            ByteCode::ldc(3),
            ByteCode::BINOP(BinOp::Mul),
            ByteCode::ldc(9),
            ByteCode::BINOP(BinOp::Gt),
            ByteCode::DONE,
        ];
        let rt = Runtime::new(instrs);
        let rt = run(rt).unwrap();
        assert_eq!(rt.current_thread.operand_stack, vec![Value::Bool(false)]);
    }

    #[test]
    fn test_assignment() {
        let instrs = vec![
            ByteCode::ldc(42),
            ByteCode::assign("x"),
            ByteCode::ldc(43),
            ByteCode::assign("y"),
            ByteCode::ldc(44),
            ByteCode::assign("x"),
            ByteCode::DONE,
        ];

        let rt = Runtime::new(instrs);
        rt.current_thread
            .env
            .borrow_mut()
            .set("x", Value::Unitialized);
        rt.current_thread
            .env
            .borrow_mut()
            .set("y", Value::Unitialized);

        let rt = run(rt).unwrap();
        assert_eq!(
            rt.current_thread.env.borrow().get(&"x".to_string()),
            Some(Value::Int(44))
        );
        assert_eq!(
            rt.current_thread.env.borrow().get(&"y".to_string()),
            Some(Value::Int(43))
        );
    }

    #[test]
    fn test_fn_call() -> Result<()> {
        // fn simple(n) {
        //     return n;
        // }
        // simple(42)
        let instrs = vec![
            ByteCode::enterscope(vec!["simple"]),
            ByteCode::ldf(3, vec!["n"]),
            ByteCode::GOTO(5), // Jump to the end of the function
            // Body of simple
            ByteCode::ld("n"), // Load the value of n onto the stacks
            ByteCode::RESET(FrameType::CallFrame), // Return from the function
            ByteCode::assign("simple"), // Assign the function to the symbol
            ByteCode::ld("simple"), // Load the function onto the stack
            ByteCode::ldc(42), // Load the argument onto the stack
            ByteCode::CALL(1), // Call the function with 1 argument
            ByteCode::EXITSCOPE,
            ByteCode::DONE,
        ];

        let rt = Runtime::new(instrs);
        let mut rt = run(rt)?;

        let result = rt.current_thread.operand_stack.pop().unwrap();
        assert_eq!(result, Value::Int(42));
        assert_eq!(rt.current_thread.runtime_stack.len(), 0);

        Ok(())
    }

    #[test]
    fn test_global_constants() -> Result<()> {
        let instrs = vec![ByteCode::ld(builtin::PI_SYM), ByteCode::DONE];

        let rt = Runtime::new(instrs);
        let rt = run(rt)?;
        assert_eq!(
            rt.current_thread.operand_stack,
            vec![Value::Float(std::f64::consts::PI)]
        );

        let instrs = vec![ByteCode::ld(builtin::MAX_INT_SYM), ByteCode::DONE];

        let rt = Runtime::new(instrs);
        let rt = run(rt)?;

        assert_eq!(
            rt.current_thread.operand_stack,
            vec![Value::Int(std::i64::MAX)]
        );

        Ok(())
    }

    #[test]
    fn test_global_functions() -> Result<()> {
        let instrs = vec![
            ByteCode::ld(builtin::STRING_LEN_SYM),
            ByteCode::ldc("Hello, world!"),
            ByteCode::CALL(1),
            ByteCode::DONE,
        ];

        let rt = Runtime::new(instrs);
        let rt = run(rt)?;

        assert_eq!(rt.current_thread.operand_stack, vec![Value::Int(13)]);

        let instrs = vec![
            ByteCode::ld(builtin::ABS_SYM),
            ByteCode::ldc(-42),
            ByteCode::CALL(1),
            ByteCode::DONE,
        ];

        let rt = Runtime::new(instrs);
        let rt = run(rt)?;

        assert_eq!(rt.current_thread.operand_stack, vec![Value::Int(42)]);

        Ok(())
    }

    #[test]
    fn test_simple_concurrency() -> Result<()> {
        let instrs = vec![ByteCode::SPAWN, ByteCode::DONE];

        let mut rt = Runtime::new(instrs);
        rt.set_time_quantum(Duration::from_millis(u64::MAX)); // Set the time quantum to infinity
        let rt = run(rt)?;

        // There is one thread in the ready queue
        assert_eq!(rt.ready_queue.len(), 1);
        // The spawned instruction pushes 0 onto the operand stack of the child thread
        assert_eq!(rt.ready_queue[0].operand_stack, vec![Value::Int(0)]);
        // The spawn instruction pushes the child thread ID onto the parent thread's operand stack
        assert_eq!(
            rt.current_thread.operand_stack,
            vec![Value::Int(MAIN_THREAD_ID + 1)]
        );

        let instrs = vec![
            ByteCode::SPAWN, // Parent operand stack will have child tid 2, child operand stack will have 0
            ByteCode::enterscope(vec!["tid", "child", "parent"]), // all threads enters a new scope
            ByteCode::assign("tid"), // Parent thread assigns the child tid to the tid symbol (2), child tid is 0
            ByteCode::ld("tid"),
            ByteCode::ldc(0),
            ByteCode::BINOP(BinOp::Eq), // Check if the child operand stack has 0
            ByteCode::JOF(10),          // Parent jumps since (child_tid == 0) == false
            ByteCode::ldc("Child thread"), // Child thread loads the value onto its operand stack
            ByteCode::assign("child"), // Child thread sets the value of the child in the environment
            ByteCode::GOTO(13),        // Child jump to the DONE instruction
            ByteCode::ldc("Parent thread"), // Parent thread loads the value onto its operand stack
            ByteCode::assign("parent"), // Parent thread sets the value of the parent in the environment
            ByteCode::JOIN(MAIN_THREAD_ID + 1), // Parent thread joins the child thread
            ByteCode::DONE,
        ];

        let rt = Runtime::new(instrs);
        let rt = run(rt)?;

        assert_eq!(
            rt.current_thread.env.borrow().get(&"parent".to_string()),
            Some(Value::String("Parent thread".to_string()))
        );
        assert_eq!(
            rt.current_thread.env.borrow().get(&"child".to_string()),
            Some(Value::Unitialized) // The parent thread environment should be unchanged
        );

        let instrs = vec![
            ByteCode::enterscope(vec!["count"]), // Line 0
            ByteCode::ldc(0),                    // Line 1
            ByteCode::assign("count"),           // Line 2
            ByteCode::SPAWN, // Line 3: Parent operand stack will have child tid 2, child operand stack will have 0
            ByteCode::enterscope(vec!["tid"]), // Line 4
            ByteCode::assign("tid"), // Line 5: Parent thread assigns the child tid to the tid symbol (2), child tid is 0
            ByteCode::ld("tid"),     // Line 6
            ByteCode::ldc(0),        // Line 7
            ByteCode::BINOP(BinOp::Eq), // Line 8: Check if the child operand stack has 0
            ByteCode::JOF(15),       // Line 9: Parent jumps since (child_tid == 0) == false
            ByteCode::ldc(1),        // Line 10: Child thread loads the value onto its operand stack
            ByteCode::ld("count"), // Line 11: Child thread loads the value of count onto the stack
            ByteCode::BINOP(BinOp::Add), // Line 12: Child thread adds the value of count and 1
            ByteCode::assign("count"), // Line 13: Child thread sets the value of count in the environment
            ByteCode::GOTO(10), // Line 14: Child loops infinitely (until time quantum expires)
            ByteCode::YIELD,    // Line 15: Parent thread yields
            ByteCode::DONE,     // Line 16
        ];

        let mut rt = Runtime::new(instrs);
        rt.set_time_quantum(Duration::from_millis(1000)); // Set the time quantum to 1 second
        let rt = run(rt)?;

        let final_count: i64 = rt
            .current_thread
            .env
            .borrow()
            .get(&"count".to_string())
            .expect("Count not in environment")
            .try_into()?;

        assert!(final_count > 0);

        Ok(())
    }
}
