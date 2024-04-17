use std::{
    collections::{HashMap, HashSet, VecDeque},
    time::{Duration, Instant},
};

use bytecode::{weak_clone, ByteCode, EnvStrong, Environment, Semaphore, ThreadID, W};

use crate::Thread;
pub use run::*;

mod gc;
mod run;

pub const DEFAULT_TIME_QUANTUM: Duration = Duration::from_millis(100);
pub const DEFAULT_GC_INTERVAL: Duration = Duration::from_secs(1);
pub const MAIN_THREAD_ID: i64 = 1;

/// The runtime of the virtual machine.
/// It contains the instructions to execute, the current thread, and the ready and blocked threads.
/// The instructions are the bytecode instructions to execute.
/// The ready queue is a queue of threads that are ready to run.
/// The blocked queue is a queue of threads that are waiting for some event to occur.
/// The zombie threads are threads that have finished executing and are waiting to be joined.
pub struct Runtime {
    /// If the program is done.
    pub done: bool,
    /// If the program is in debug mode.
    pub debug: bool,
    /// The time the program started, used for calculating the time quantum.
    pub time: Instant,
    /// The maximum amount of time a thread can run before it is preempted.
    pub time_quantum: Duration,
    /// The interval at which to run the mark and sweep garbage collector.
    pub gc_interval: Duration,
    /// The instructions to execute.
    pub instrs: Vec<ByteCode>,
    /// The environment registry, holds strong references to environments.
    pub env_registry: HashSet<EnvStrong>,
    /// The number of threads that have been created.
    pub thread_count: i64,
    /// The current thread that is executing.
    pub current_thread: Thread,
    /// The threads that are ready to run.
    pub ready_queue: VecDeque<Thread>,
    /// The threads that are blocked.
    pub blocked_queue: VecDeque<(Thread, Semaphore)>,
    /// The threads that have finished executing, waiting to be joined.
    pub zombie_threads: HashMap<ThreadID, Thread>,
}

/// Constructors for the runtime.
impl Runtime {
    pub fn new(instrs: Vec<ByteCode>) -> Self {
        let global_env = Environment::new_global_wrapped();
        let global_env_weak = weak_clone(&global_env);
        let mut envs = HashSet::new();
        envs.insert(W(global_env));

        Runtime {
            debug: false,
            done: false,
            time: Instant::now(),
            time_quantum: DEFAULT_TIME_QUANTUM,
            gc_interval: DEFAULT_GC_INTERVAL,
            instrs,
            env_registry: envs,
            thread_count: 1,
            current_thread: Thread::new(MAIN_THREAD_ID, global_env_weak),
            ready_queue: VecDeque::new(),
            blocked_queue: VecDeque::new(),
            zombie_threads: HashMap::new(),
        }
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime::new(vec![])
    }
}

/// Configuration for the runtime.
impl Runtime {
    pub fn set_time_quantum(&mut self, time_quantum: Duration) {
        self.time_quantum = time_quantum;
    }

    pub fn set_gc_interval(&mut self, gc_interval: Duration) {
        self.gc_interval = gc_interval;
    }

    pub fn set_debug_mode(&mut self) {
        self.debug = true;
    }
}
