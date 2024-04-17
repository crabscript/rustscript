use std::{
    cell::RefCell,
    hash::{Hash, Hasher},
    rc::Rc,
};

use crate::{Environment, W};

pub type EnvStrong = W<Rc<RefCell<Environment>>>;

impl PartialEq for EnvStrong {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for EnvStrong {}

impl Hash for EnvStrong {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state)
    }
}
