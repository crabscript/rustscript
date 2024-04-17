use std::{cell::RefCell, collections::HashMap, rc::Weak};

use bytecode::{weak_clone, EnvWeak, Environment, StackFrame, Value, W};

use crate::{Runtime, Thread};

/// Runtime methods at runtime.
impl Runtime {
    /// Mark and sweep the environment registry.
    /// This will remove all environments that are no longer referenced.
    ///
    /// - Mark environment x -> env_registry.get(x) = true
    /// - Sweep environment x -> env_registry.remove(x) if env_registry.get(x) = false
    /// - Clean up -> reset env_registry.get(x) = false
    ///
    /// Traverse through all the threads, for each thread:
    ///   - Mark its current environment and the environment of closure values in the current environment,
    ///     and the chain of parent environments.
    ///   - Go through the runtime stack and mark all the environments and environment of closure values in
    ///     their respective environment, and the chain of parent environments
    ///   - Go through the operand stack and mark all the environments of closure values, and the chain of parent environments
    pub fn mark_and_weep(self) -> Self {
        let marked = mark(&self);
        sweep(self, marked)
    }
}

fn mark(rt: &Runtime) -> HashMap<EnvWeak, bool> {
    let mut marked = env_hashmap(rt);

    // Mark the current thread
    marked = mark_thread(marked, &rt.current_thread);

    // Mark the ready queue
    for thread in rt.ready_queue.iter() {
        marked = mark_thread(marked, thread);
    }

    // Mark the blocked queue
    for (thread, _) in rt.blocked_queue.iter() {
        marked = mark_thread(marked, thread);
    }

    // Zombie threads will be ignored

    marked
}

fn sweep(mut rt: Runtime, m: HashMap<EnvWeak, bool>) -> Runtime {
    todo!()
}

fn env_hashmap(rt: &Runtime) -> HashMap<EnvWeak, bool> {
    let mut envs = HashMap::new();
    for env in rt.env_registry.iter() {
        envs.insert(W(weak_clone(env)), false);
    }
    envs
}

fn mark_thread(mut m: HashMap<EnvWeak, bool>, t: &Thread) -> HashMap<EnvWeak, bool> {
    todo!()
}

fn mark_env(
    mut m: HashMap<EnvWeak, bool>,
    env: &Weak<RefCell<Environment>>,
) -> HashMap<EnvWeak, bool> {
    todo!()
}

fn mark_operand_stack(mut m: HashMap<EnvWeak, bool>, os: &Vec<Value>) -> HashMap<EnvWeak, bool> {
    todo!()
}

fn mark_runtime_stack(
    mut m: HashMap<EnvWeak, bool>,
    rs: &Vec<StackFrame>,
) -> HashMap<EnvWeak, bool> {
    todo!()
}
