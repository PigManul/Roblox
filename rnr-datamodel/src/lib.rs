use std::rc::Rc;
use std::cell::RefCell;
use rnr_core::instance::Instance;

pub mod datamodel;

pub use datamodel::*;

/// Instance factory for creating instances by class name
pub trait InstanceFactory {
    fn create_instance(&self, class_name: &str) -> Option<Rc<RefCell<Instance>>>;
}

/// Create an instance by class name (requires factory to be set)
pub fn create_instance(_class_name: &str) -> Option<Rc<RefCell<Instance>>> {
    // TODO: Implement instance factory system
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
