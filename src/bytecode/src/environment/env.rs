use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    fmt::Debug,
    rc::{Rc, Weak},
};

use anyhow::Result;

use crate::{builtin, ByteCodeError, Symbol, Value};

#[derive(Debug, Clone, Default)]
pub struct Environment {
    pub parent: Option<Weak<RefCell<Environment>>>,
    pub env: HashMap<Symbol, Value>,
}

impl PartialEq for Environment {
    fn eq(&self, other: &Self) -> bool {
        self.env == other.env
    }
}

impl Environment {
    /// Create a new frame with no parent, i.e. the root frame.
    pub fn new() -> Self {
        Environment {
            parent: None,
            env: HashMap::new(),
        }
    }

    /// Create the global environment.
    ///
    /// Constants are added to the global environment.
    /// - Logical constants: true, false
    /// - Math constants: PI, E
    /// - Environment constants: MAX_INT, MIN_INT, MAX_FLOAT, MIN_FLOAT, EPSILON
    ///
    /// Built in functions are added to the global environment.
    /// - Math functions: abs, ceil, floor, round, sqrt, sin, cos, tan, log10, pow
    /// - String functions: len
    /// - Type conversion functions: int_to_float, float_to_int, atoi, atoi
    /// - Comparison functions: min, max
    ///
    /// # Returns
    ///
    /// A wrapped reference to the global environment.
    pub fn new_global_wrapped() -> Rc<RefCell<Self>> {
        let env = Environment::new_wrapped();

        // Global constants
        // Logical constants
        env.borrow_mut().set(builtin::TRUE_SYM, true);
        env.borrow_mut().set(builtin::FALSE_SYM, false);

        // Math constants
        env.borrow_mut().set(builtin::PI_SYM, std::f64::consts::PI);
        env.borrow_mut().set(builtin::E_SYM, std::f64::consts::E);

        //Environment constants
        env.borrow_mut().set(builtin::MAX_INT_SYM, std::i64::MAX);
        env.borrow_mut().set(builtin::MIN_INT_SYM, std::i64::MIN);
        env.borrow_mut().set(builtin::MAX_FLOAT_SYM, std::f64::MAX);
        env.borrow_mut().set(builtin::MIN_FLOAT_SYM, std::f64::MIN);
        env.borrow_mut()
            .set(builtin::EPSILON_SYM, std::f64::EPSILON);

        // Built in functions
        // Math functions
        env.borrow_mut().set(builtin::ABS_SYM, builtin::abs());
        env.borrow_mut().set(builtin::COS_SYM, builtin::cos());
        env.borrow_mut().set(builtin::SIN_SYM, builtin::sin());
        env.borrow_mut().set(builtin::TAN_SYM, builtin::tan());
        env.borrow_mut().set(builtin::LOG_SYM, builtin::log());
        env.borrow_mut().set(builtin::POW_SYM, builtin::pow());
        env.borrow_mut().set(builtin::SQRT_SYM, builtin::sqrt());
        env.borrow_mut().set(builtin::MAX_SYM, builtin::max());
        env.borrow_mut().set(builtin::MIN_SYM, builtin::min());

        // String functions
        env.borrow_mut()
            .set(builtin::STRING_LEN_SYM, builtin::string_len());

        // Type conversion functions
        env.borrow_mut()
            .set(builtin::INT_TO_FLOAT_SYM, builtin::int_to_float());
        env.borrow_mut()
            .set(builtin::FLOAT_TO_INT_SYM, builtin::float_to_int());
        env.borrow_mut().set(builtin::ATOI_SYM, builtin::atoi());
        env.borrow_mut().set(builtin::ITOA_SYM, builtin::itoa());

        // stdin, stdout
        env.borrow_mut()
            .set(builtin::READ_LINE_SYM, builtin::read_line());
        env.borrow_mut().set(builtin::PRINT_SYM, builtin::print());
        env.borrow_mut()
            .set(builtin::PRINTLN_SYM, builtin::println());

        // Semaphore functions
        env.borrow_mut()
            .set(builtin::SEM_CREATE_SYM, builtin::sem_create());
        env.borrow_mut()
            .set(builtin::SEM_SET_SYM, builtin::sem_set());

        env
    }

    /// Create a wrapped frame with no parent, i.e. the root frame.
    pub fn new_wrapped() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Environment::new()))
    }
}

impl Environment {
    /// Set the parent of the frame.
    pub fn set_parent(&mut self, parent: Weak<RefCell<Environment>>) {
        self.parent = Some(parent);
    }

    /// Get a snapshot of the value of a symbol in the frame at the time of the call.
    pub fn get(&self, sym: &Symbol) -> Result<Value> {
        // If the symbol is found in the current environment, return the value.
        if let Some(val) = self.env.get(sym) {
            return Ok(val.clone());
        }

        // If the symbol is not found in the current environment, search the parent environment.
        let Some(parent) = &self.parent else {
            // If the parent environment is not found, return an error.
            return Err(ByteCodeError::UnboundedName { name: sym.clone() }.into());
        };

        // If the parent environment is found, search the parent environment.
        let Some(parent) = parent.upgrade() else {
            // If the parent environment is dropped prematurely, return an error.
            return Err(ByteCodeError::EnvironmentDroppedError.into());
        };

        let parent_ref = parent.borrow();
        parent_ref.get(sym)
    }

    /// Set the value of a symbol in the current environment.
    ///
    /// # Arguments
    ///
    /// * `sym` - The symbol whose value is to be set.
    /// * `val` - The value to be set.
    pub fn set(&mut self, sym: impl Into<Symbol>, val: impl Into<Value>) {
        self.env.insert(sym.into(), val.into());
    }

    /// Update the value of a symbol in the current environment.
    /// If the symbol is not found in the current environment, the parent environment is searched.
    /// If the symbol is not found in the environment chain, an error is returned.
    ///
    /// # Arguments
    ///
    /// * `sym` - The symbol whose value is to be updated.
    /// * `val` - The new value to be set.
    ///
    /// # Returns
    ///
    /// An error if the symbol is not found in the environment chain.
    ///
    /// # Errors
    ///
    /// * `ByteCodeError::UnboundedName` - If the symbol is not found in the environment chain.
    pub fn update(&mut self, sym: impl Into<Symbol>, val: impl Into<Value>) -> Result<()> {
        let sym = sym.into();

        // If the symbol is found in the current environment, update the value.
        if let Entry::Occupied(mut entry) = self.env.entry(sym.clone()) {
            entry.insert(val.into());
            return Ok(());
        }

        // If the symbol is not found in the current environment, search the parent environment.
        let Some(parent) = &self.parent else {
            // If the parent environment is not found, return an error.
            return Err(ByteCodeError::UnboundedName { name: sym }.into());
        };

        // If the parent environment is found, search the parent environment.
        let Some(parent) = parent.upgrade() else {
            // If the parent environment is dropped prematurely, return an error.
            return Err(ByteCodeError::EnvironmentDroppedError.into());
        };

        let mut parent_ref = parent.borrow_mut();
        parent_ref.update(sym, val)
    }
}

pub fn weak_clone(env: &Rc<RefCell<Environment>>) -> Weak<RefCell<Environment>> {
    let env = Rc::clone(env);
    weak_clone(&env)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment() {
        let env = Environment::new_wrapped();
        env.borrow_mut().set("x", 42);
        assert_eq!(env.borrow().get(&"x".to_string()).unwrap(), Value::Int(42));
    }

    #[test]
    fn test_set_environment() {
        let parent_env = Environment::new_wrapped();
        parent_env.borrow_mut().set("x", 42);
        let parent_env_weak = weak_clone(&parent_env);

        let child_env = Environment::new_wrapped();
        child_env.borrow_mut().set_parent(parent_env_weak);
        child_env.borrow_mut().set("y", 43);

        assert_eq!(
            child_env.borrow().get(&"x".to_string()).unwrap(),
            Value::Int(42)
        );
        assert_eq!(
            child_env.borrow().get(&"y".to_string()).unwrap(),
            Value::Int(43)
        );
    }

    #[test]
    fn test_update_environment() {
        let parent_env = Environment::new_wrapped();
        parent_env.borrow_mut().set("x", 42);
        let parent_env_weak = weak_clone(&parent_env);

        let child_env = Environment::new_wrapped();
        child_env.borrow_mut().set_parent(parent_env_weak);
        child_env.borrow_mut().set("y", 43);
        child_env.borrow_mut().update("x", 44).unwrap();

        assert_eq!(
            child_env.borrow().get(&"x".to_string()).unwrap(),
            Value::Int(44)
        );
        assert_eq!(
            child_env.borrow().get(&"y".to_string()).unwrap(),
            Value::Int(43)
        );
        assert!(!child_env.borrow().env.contains_key(&"x".to_string()));
    }
}
