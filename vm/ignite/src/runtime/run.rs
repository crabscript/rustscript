use std::time::Instant;

use anyhow::Result;
use bytecode::ByteCode;

use crate::{micro_code, Runtime, VmError};

/// Runtime methods at runtime.
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
    /// Check if the time quantum has expired.
    /// The time quantum is the maximum amount of time a thread can run before it is preempted.
    pub fn time_quantum_expired(&self) -> bool {
        self.time.elapsed() >= self.time_quantum
    }

    pub fn should_garbage_collect(&self) -> bool {
        self.time.elapsed() >= self.gc_interval
    }

    pub fn garbage_collect(mut self) -> Self {
        self = self.mark_and_weep();
        self.time = Instant::now();
        self
    }

    /// The program is done if the current thread is the main thread and the current thread is done.
    pub fn is_done(&self) -> bool {
        self.done
    }

    pub fn debug_print(&self) {
        let thread_id = self.current_thread.thread_id;
        let pc = self.current_thread.pc;
        let instruction = self.instrs.get(pc).expect("PC out of bounds");
        println!("Thread: {}, PC: {}, {:?}", thread_id, pc, instruction);
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
        if rt.is_done() {
            break;
        }

        if rt.should_garbage_collect() {
            rt = rt.garbage_collect();
        }

        if rt.time_quantum_expired() {
            rt = micro_code::yield_(rt)?;
            continue;
        }

        if rt.debug {
            rt.debug_print();
        }

        let instr = rt.fetch_instr()?;

        rt = execute(rt, instr)?;
    }

    Ok(rt)
}

/// Execute a single instruction, mutating the runtime.
///
/// # Arguments
///
/// * `rt` - The runtime to execute the instruction on.
///
/// * `instr` - The instruction to execute.
///
/// # Returns
///
/// The runtime after the instruction has been executed.
///
/// # Errors
///
/// If an error occurs during execution.
pub fn execute(rt: Runtime, instr: ByteCode) -> Result<Runtime> {
    match instr {
        ByteCode::DONE => micro_code::done(rt),
        ByteCode::ASSIGN(sym) => micro_code::assign(rt, sym),
        ByteCode::LD(sym) => micro_code::ld(rt, sym),
        ByteCode::LDC(val) => micro_code::ldc(rt, val),
        ByteCode::LDF(addr, prms) => micro_code::ldf(rt, addr, prms),
        ByteCode::POP => micro_code::pop(rt),
        ByteCode::UNOP(op) => micro_code::unop(rt, op),
        ByteCode::BINOP(op) => micro_code::binop(rt, op),
        ByteCode::JOF(pc) => micro_code::jof(rt, pc),
        ByteCode::GOTO(pc) => micro_code::goto(rt, pc),
        ByteCode::RESET(ft) => micro_code::reset(rt, ft),
        ByteCode::ENTERSCOPE(syms) => micro_code::enter_scope(rt, syms),
        ByteCode::EXITSCOPE => micro_code::exit_scope(rt),
        ByteCode::CALL(arity) => micro_code::call(rt, arity),
        ByteCode::SPAWN(addr) => micro_code::spawn(rt, addr),
        ByteCode::JOIN => micro_code::join(rt),
        ByteCode::YIELD => micro_code::yield_(rt),
        ByteCode::SEMCREATE => micro_code::sem_create(rt),
        ByteCode::WAIT => micro_code::wait(rt),
        ByteCode::POST => micro_code::post(rt),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::MAIN_THREAD_ID;

    use super::*;
    use anyhow::{Ok, Result};
    use bytecode::{builtin, BinOp, ByteCode, FrameType, Symbol, UnOp, Value};

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
    fn test_assignment() -> Result<()> {
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
            .upgrade()
            .unwrap()
            .borrow_mut()
            .set("x", Value::Unitialized);
        rt.current_thread
            .env
            .upgrade()
            .unwrap()
            .borrow_mut()
            .set("y", Value::Unitialized);

        let rt = run(rt).unwrap();
        assert_eq!(
            rt.current_thread
                .env
                .upgrade()
                .unwrap()
                .borrow()
                .get(&"x".to_string())?,
            Value::Int(44)
        );
        assert_eq!(
            rt.current_thread
                .env
                .upgrade()
                .unwrap()
                .borrow()
                .get(&"y".to_string())?,
            Value::Int(43)
        );

        Ok(())
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
    fn test_concurrency_01() -> Result<()> {
        let instrs = vec![ByteCode::SPAWN(1), ByteCode::DONE];

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

        Ok(())
    }

    #[test]
    fn test_concurrency_02() -> Result<()> {
        // fn simple(n) {
        //    return n;
        // }
        //
        // spawn simple(123);
        // join 2
        let instrs = vec![
            ByteCode::enterscope(vec!["simple"]),
            ByteCode::ldf(3, vec!["n"]),
            ByteCode::GOTO(5), // Jump past function body
            ByteCode::ld("n"),
            ByteCode::RESET(FrameType::CallFrame),
            ByteCode::assign("simple"),
            ByteCode::SPAWN(8), // Parent operand stack will have child tid 2, child operand stack will have
            ByteCode::GOTO(13), // Parent jump past CALL and DONE
            ByteCode::POP,
            ByteCode::ld("simple"),
            ByteCode::ldc(123),
            ByteCode::CALL(1),
            ByteCode::DONE,
            ByteCode::ldc(MAIN_THREAD_ID + 1), // Load the child tid onto the stack
            ByteCode::JOIN,
            ByteCode::DONE,
        ];

        let rt = Runtime::new(instrs);
        let mut rt = run(rt)?;

        println!("{:?}", rt.current_thread.operand_stack);

        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Int(123)
        );

        Ok(())
    }

    #[test]
    fn test_concurrency_03() -> Result<()> {
        // let count = 0;
        // fn infinite_increment() {
        //    loop {
        //        count = count + 1;
        //    }
        // }
        // spawn infinite_increment();
        // yield;
        // // no join

        let empty_str_arr: Vec<Symbol> = vec![];

        let instrs = vec![
            ByteCode::enterscope(vec!["count", "infinite_increment"]),
            ByteCode::ldc(0),
            ByteCode::assign("count"), // Set count to 0
            ByteCode::ldf(6, empty_str_arr),
            ByteCode::assign("infinite_increment"), // assign function
            ByteCode::GOTO(11),                     // Jump past function body
            ByteCode::ld("count"),                  // Start of function body
            ByteCode::ldc(1),
            ByteCode::BINOP(BinOp::Add),
            ByteCode::assign("count"),
            ByteCode::GOTO(6),   // End of function body
            ByteCode::SPAWN(13), // Parent operand stack will have child tid 2, child operand stack will have
            ByteCode::GOTO(17),  // Parent jump past CALL and DONE
            ByteCode::POP,
            ByteCode::ld("infinite_increment"),
            ByteCode::CALL(0),
            ByteCode::DONE,
            ByteCode::YIELD, // Parent thread yields to child thread
            ByteCode::DONE,
        ];

        let mut rt = Runtime::new(instrs);
        rt.set_time_quantum(Duration::from_millis(1000)); // Set the time quantum to 1 second
        let rt = run(rt)?;

        let final_count: i64 = rt
            .current_thread
            .env
            .upgrade()
            .unwrap()
            .borrow()
            .get(&"count".to_string())
            .expect("Count not in environment")
            .try_into()?;

        assert!(final_count > 0);
        Ok(())
    }

    #[test]
    fn test_concurrency_04() -> Result<()> {
        // let count = 0;
        //
        // fn increment(times: int) {
        //   let i = 0;
        //   loop i < times {
        //     count = count + 1;
        //     i = i + 1;
        //   }
        // }
        //
        // let tid_2 = spawn increment(l00);
        // let tid_3 = spawn increment(100);
        // let tid_4 = spawn increment(100);
        //
        // join tid_2;
        // join tid_3;
        // join tid_4;
        //
        // count

        let instrs = vec![
            // pc 0
            ByteCode::enterscope(vec!["count", "increment", "tid_2", "tid_3", "tid_4"]),
            // pc 1
            ByteCode::ldc(0),
            // pc 2
            ByteCode::assign("count"), // Set count to 0
            // pc 3
            ByteCode::ldf(6, vec!["times"]),
            // pc 4
            ByteCode::assign("increment"), // assign function
            // pc 5
            ByteCode::GOTO(25), // Jump past function body
            // pc 6
            ByteCode::enterscope(vec!["i"]),
            // pc 7
            ByteCode::ldc(0),
            // pc 8
            ByteCode::assign("i"),
            // pc 9
            ByteCode::ld("i"),
            // pc 10
            ByteCode::ld("times"),
            // pc 11
            ByteCode::BINOP(BinOp::Lt),
            // pc 12
            ByteCode::JOF(23), // Jump past the loop
            // pc 13
            ByteCode::ld("count"),
            // pc 14
            ByteCode::ldc(1),
            // pc 15
            ByteCode::BINOP(BinOp::Add),
            // pc 16
            ByteCode::YIELD, // Try to introduce race conditions
            // pc 17
            ByteCode::assign("count"),
            // pc 18
            ByteCode::ld("i"),
            // pc 19
            ByteCode::ldc(1),
            // pc 20
            ByteCode::BINOP(BinOp::Add),
            // pc 21
            ByteCode::assign("i"),
            // pc 22
            ByteCode::GOTO(9), // End of loop
            // pc 23
            ByteCode::EXITSCOPE,
            // pc 24
            ByteCode::RESET(FrameType::CallFrame), // End of function
            // pc 25
            ByteCode::SPAWN(28), // Parent operand stack will have child tid 2, child operand stack will have 0
            // pc 26
            ByteCode::assign("tid_2"), // Parent saves the child tid
            // pc 27
            ByteCode::GOTO(32), // Parent jumps past function call by the child
            // pc 28
            ByteCode::ld("increment"), // Child loads the function
            // pc 29
            ByteCode::ldc(100), // Child loads the argument
            // pc 30
            ByteCode::CALL(1), // Child calls the increment function with 100
            // pc 31
            ByteCode::DONE, // Child is done
            // pc 32
            ByteCode::SPAWN(35), // Parent operand stack will have child tid 3, child operand stack will have 0
            // pc 33
            ByteCode::assign("tid_3"), // Parent saves the child tid
            // pc 34
            ByteCode::GOTO(39), // Parent jumps past function call by the child
            // pc 35
            ByteCode::ld("increment"), // Child loads the function
            // pc 36
            ByteCode::ldc(100), // Child loads the argument
            // pc 37
            ByteCode::CALL(1), // Child calls the increment function with 100
            // pc 38
            ByteCode::DONE, // Child is done
            // pc 39
            ByteCode::SPAWN(42), // Parent operand stack will have child tid 4, child operand stack will have 0
            // pc 40
            ByteCode::assign("tid_4"), // Parent loads the child tid
            // pc 41
            ByteCode::GOTO(46), // Parent jumps past function call by the child
            // pc 42
            ByteCode::ld("increment"), // Child loads the function
            // pc 43
            ByteCode::ldc(100), // Child loads the argument
            // pc 44
            ByteCode::CALL(1), // Child calls the increment function with 100
            // pc 45
            ByteCode::DONE, // Child is done
            // pc 46
            ByteCode::ld("tid_2"),
            // pc 47
            ByteCode::JOIN, // Parent thread joins the child thread
            // pc 48
            ByteCode::ld("tid_3"), // Parent loads the child tid
            // pc 49
            ByteCode::JOIN, // Parent thread joins the child thread
            // pc 50
            ByteCode::ld("tid_4"), // Parent loads the child tid
            // pc 51
            ByteCode::JOIN, // Parent thread joins the child thread
            // pc 52
            ByteCode::ld("count"), // Parent loads the count
            // pc 53
            ByteCode::DONE, // Parent is done
        ];

        let mut rt = Runtime::new(instrs);

        // Set the time quantum to a short time, so that race conditions are more likely to occur
        rt.set_time_quantum(Duration::from_micros(1));
        let rt = run(rt)?;

        let final_count: i64 = rt
            .current_thread
            .env
            .upgrade()
            .unwrap()
            .borrow()
            .get(&"count".to_string())
            .expect("Count not in environment")
            .try_into()?;

        println!("Final count: {}", final_count);
        assert!(final_count < 300); // The count should be less than 300 due to race conditions

        Ok(())
    }

    #[test]
    fn test_concurrency_05() -> Result<()> {
        // let count = 0;
        // let sem = sem_create();
        //
        // fn increment(times: int) {
        //   let i = 0;
        //   loop i < times {
        //     wait sem;
        //     count = count + 1; // Critical section
        //     post sem;
        //     i = i + 1;
        //   }
        // }
        //
        // let tid_2 = spawn increment(l00);
        // let tid_3 = spawn increment(100);
        // let tid_4 = spawn increment(100);
        //
        // join tid_2;
        // join tid_3;
        // join tid_4;
        //
        // count

        let instrs = vec![
            // pc 0
            ByteCode::enterscope(vec!["count", "sem", "increment", "tid_2", "tid_3", "tid_4"]),
            // pc 1
            ByteCode::ldc(0),
            // pc 2
            ByteCode::assign("count"), // Set count to 0
            // pc 3
            ByteCode::ld(builtin::SEM_CREATE_SYM),
            // pc 4
            ByteCode::CALL(0),
            // pc 5
            ByteCode::assign("sem"), // Set sem to the semaphore
            // pc 6
            ByteCode::ldf(9, vec!["times"]),
            // pc 7
            ByteCode::assign("increment"), // assign function
            // pc 8
            ByteCode::GOTO(32), // Jump past function body
            // pc 9
            ByteCode::enterscope(vec!["i"]),
            // pc 10
            ByteCode::ldc(0),
            // pc 11
            ByteCode::assign("i"),
            // pc 12
            ByteCode::ld("i"),
            // pc 13
            ByteCode::ld("times"),
            // pc 14
            ByteCode::BINOP(BinOp::Lt),
            // pc 15
            ByteCode::JOF(30), // Jump past the loop
            // pc 16
            ByteCode::ld("sem"),
            // pc 17
            ByteCode::WAIT,
            // pc 18
            ByteCode::ld("count"),
            // pc 19
            ByteCode::ldc(1),
            // pc 20
            ByteCode::BINOP(BinOp::Add),
            // pc 21
            ByteCode::assign("count"),
            // pc 22
            ByteCode::ld("sem"),
            // pc 23
            ByteCode::POST,
            // pc 24
            ByteCode::ld("i"),
            // pc 25
            ByteCode::ldc(1),
            // pc 26
            ByteCode::YIELD, // Try to introduce race conditions
            // pc 27
            ByteCode::BINOP(BinOp::Add),
            // pc 28
            ByteCode::assign("i"),
            // pc 29
            ByteCode::GOTO(12), // End of loop
            // pc 30
            ByteCode::EXITSCOPE,
            // pc 31
            ByteCode::RESET(FrameType::CallFrame), // End of function
            // pc 32
            ByteCode::SPAWN(35), // Parent operand stack will have child tid 2, child operand stack will have 0
            // pc 33
            ByteCode::assign("tid_2"), // Parent saves the child tid
            // pc 34
            ByteCode::GOTO(39), // Parent jumps past function call by the child
            // pc 35
            ByteCode::ld("increment"), // Child loads the function
            // pc 36
            ByteCode::ldc(100), // Child loads the argument
            // pc 37
            ByteCode::CALL(1), // Child calls the increment function with 100
            // pc 38
            ByteCode::DONE, // Child is done
            // pc 39
            ByteCode::SPAWN(42), // Parent operand stack will have child tid 3, child operand stack will have 0
            // pc 40
            ByteCode::assign("tid_3"), // Parent saves the child tid
            // pc 41
            ByteCode::GOTO(46), // Parent jumps past function call by the child
            // pc 42
            ByteCode::ld("increment"), // Child loads the function
            // pc 43
            ByteCode::ldc(100), // Child loads the argument
            // pc 44
            ByteCode::CALL(1), // Child calls the increment function with 100
            // pc 45
            ByteCode::DONE, // Child is done
            // pc 46
            ByteCode::SPAWN(49), // Parent operand stack will have child tid 4, child operand stack will have 0
            // pc 47
            ByteCode::assign("tid_4"), // Parent loads the child tid
            // pc 48
            ByteCode::GOTO(53), // Parent jumps past function call by the child
            // pc 49
            ByteCode::ld("increment"), // Child loads the function
            // pc 50
            ByteCode::ldc(100), // Child loads the argument
            // pc 51
            ByteCode::CALL(1), // Child calls the increment function with 100
            // pc 52
            ByteCode::DONE, // Child is done
            // pc 53
            ByteCode::ld("tid_2"),
            // pc 54
            ByteCode::JOIN, // Parent thread joins the child thread
            // pc 55
            ByteCode::ld("tid_3"), // Parent loads the child tid
            // pc 56
            ByteCode::JOIN, // Parent thread joins the child thread
            // pc 57
            ByteCode::ld("tid_4"), // Parent loads the child tid
            // pc 58
            ByteCode::JOIN, // Parent thread joins the child thread
            // pc 59
            ByteCode::ld("count"), // Parent loads the count
            // pc 60
            ByteCode::DONE, // Parent is done
        ];

        let mut rt = Runtime::new(instrs.clone());
        // Set the time quantum to a short time, so that race conditions are more likely to occur
        rt.set_time_quantum(Duration::from_micros(10));
        let rt = run(rt)?;

        let final_count: i64 = rt
            .current_thread
            .env
            .upgrade()
            .unwrap()
            .borrow()
            .get(&"count".to_string())
            .expect("Count not in environment")
            .try_into()?;

        println!(
            "Time Quantum: {} microseconds, Final count: {}",
            10, final_count
        );
        assert_eq!(final_count, 300); // The count should be exactly 300

        let mut rt = Runtime::new(instrs.clone());
        rt.set_time_quantum(Duration::from_micros(1));
        let rt = run(rt)?;

        let final_count: i64 = rt
            .current_thread
            .env
            .upgrade()
            .unwrap()
            .borrow()
            .get(&"count".to_string())
            .expect("Count not in environment")
            .try_into()?;

        println!(
            "Time Quantum: {} microseconds, Final count: {}",
            1, final_count
        );
        assert_eq!(final_count, 300); // The count should be exactly 300

        Ok(())
    }
}
