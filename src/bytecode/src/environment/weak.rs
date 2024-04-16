use std::{
    cell::RefCell,
    fmt::Debug,
    hash::{Hash, Hasher},
    rc::{Rc, Weak},
};

use crate::{Environment, W};

pub type EnvWeak = W<Weak<RefCell<Environment>>>;

impl Clone for EnvWeak {
    fn clone(&self) -> Self {
        W(self.0.clone())
    }
}

impl Debug for EnvWeak {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let this = self.0.upgrade();

        match this {
            Some(this) => write!(f, "{:?}", this),
            None => write!(f, "Weak::None"),
        }
    }
}

impl Default for EnvWeak {
    fn default() -> Self {
        W(Weak::new())
    }
}

impl PartialEq for EnvWeak {
    fn eq(&self, other: &Self) -> bool {
        let this = self.0.upgrade();
        let other = other.0.upgrade();

        match (this, other) {
            (Some(this), Some(other)) => Rc::ptr_eq(&this, &other),
            _ => false,
        }
    }
}

impl Eq for EnvWeak {}

impl Hash for EnvWeak {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let this = self.0.upgrade();

        match this {
            Some(this) => Rc::as_ptr(&this).hash(state),
            None => state.write_usize(0),
        }
    }
}
