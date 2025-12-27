use std::rc::{Rc, Weak};
use std::cell::RefCell;
use crate::reflection::{ReflectionProperty, ReflectionFunction, PropertyType, AccessType, OperationType};

/// Trait for objects that can be notified of instance hierarchy changes
pub trait InstanceListener {
    fn on_child_added(&mut self, child: Rc<RefCell<Instance>>);
    fn on_child_removed(&mut self, child: Rc<RefCell<Instance>>);
    fn on_descendant_added(&mut self, descendant: Rc<RefCell<Instance>>);
    fn on_descendant_removed(&mut self, descendant: Rc<RefCell<Instance>>);
    fn on_parent_changed(&mut self, new_parent: Option<Rc<RefCell<Instance>>>);
}

/// The fundamental Instance type - the base class for all objects in RNR
pub struct Instance {
    /// Weak reference to parent to avoid reference cycles
    parent: Weak<RefCell<Instance>>,
    /// Strong references to children
    children: Vec<Rc<RefCell<Instance>>>,
    /// Instance name
    name: String,
    /// Whether this instance can be saved/replicated
    archivable: bool,
    /// Class name for type identification
    class_name: String,
    /// Listeners for hierarchy changes
    listeners: Vec<Box<dyn InstanceListener>>,
}

impl Instance {
    /// Create a new instance
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            parent: Weak::new(),
            children: Vec::new(),
            name: "Instance".to_string(),
            archivable: true,
            class_name: "Instance".to_string(),
            listeners: Vec::new(),
        }))
    }

    /// Get the class name (for type identification)
    pub fn class_name(&self) -> &str {
        &self.class_name
    }

    /// Set the class name
    pub fn set_class_name(&mut self, name: &str) {
        self.class_name = name.to_string();
    }

    /// Check if this instance is of a specific type or inherits from it
    pub fn is_a(&self, class_name: &str) -> bool {
        self.class_name == class_name
    }

    /// Get the instance name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set the instance name
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
        // TODO: Notify replicator of name change
    }

    /// Check if instance is archivable
    pub fn archivable(&self) -> bool {
        self.archivable
    }

    /// Set archivable flag
    pub fn set_archivable(&mut self, archivable: bool) {
        self.archivable = archivable;
        // TODO: Notify replicator of archivable change
    }

    /// Get parent instance
    pub fn parent(&self) -> Option<Rc<RefCell<Instance>>> {
        self.parent.upgrade()
    }

    /// Get children as a slice
    pub fn children(&self) -> &[Rc<RefCell<Instance>>] {
        &self.children
    }

    /// Get number of children
    pub fn num_children(&self) -> usize {
        self.children.len()
    }

    /// Find first child with given name
    pub fn find_first_child(&self, name: &str) -> Option<Rc<RefCell<Instance>>> {
        self.children.iter().find(|child| {
            child.borrow().name() == name
        }).cloned()
    }

    /// Find first child of given type
    pub fn find_first_child_of_type(&self, class_name: &str) -> Option<Rc<RefCell<Instance>>> {
        self.children.iter().find(|child| {
            child.borrow().is_a(class_name)
        }).cloned()
    }

    /// Check if given instance is a child of this instance
    pub fn contains(&self, child: &Rc<RefCell<Instance>>) -> bool {
        self.children.iter().any(|c| Rc::ptr_eq(c, child))
    }

    /// Check if given instance is an ancestor of this instance
    pub fn is_ancestor_of(instance_a: &Rc<RefCell<Instance>>, instance_b: &Rc<RefCell<Instance>>) -> bool {
        let mut current = instance_b.borrow().parent();
        while let Some(parent) = current {
            if Rc::ptr_eq(&parent, instance_a) {
                return true;
            }
            current = parent.borrow().parent();
        }
        false
    }

    /// Check if it's safe to set parent (no cycles, etc.)
    pub fn can_set_parent(instance: &Rc<RefCell<Instance>>, new_parent: Option<&Rc<RefCell<Instance>>>) -> bool {
        if let Some(parent) = new_parent {
            // Check for cycles
            if Self::is_ancestor_of(instance, parent) {
                return false;
            }
            // Check if parent can accept this child
            return Self::can_add_child(parent, instance);
        }
        true
    }

    /// Check if it's safe to add child
    pub fn can_add_child(parent: &Rc<RefCell<Instance>>, child: &Rc<RefCell<Instance>>) -> bool {
        // Prevent self-references and existing parent relationships
        if Rc::ptr_eq(child, parent) ||
           child.borrow().contains(parent) ||
           child.borrow().parent().is_some() {
            return false;
        }
        true
    }

    /// Set parent instance
    pub fn set_parent(instance: &Rc<RefCell<Instance>>, new_parent: Option<Rc<RefCell<Instance>>>) {
        let can_set = Self::can_set_parent(instance, new_parent.as_ref());
        if can_set {
            let mut instance_mut = instance.borrow_mut();

            // Remove from old parent
            if let Some(old_parent) = instance_mut.parent.upgrade() {
                old_parent.borrow_mut().remove_child_internal(instance);
            }

            // Set new parent
            instance_mut.parent = match &new_parent {
                Some(p) => Rc::downgrade(p),
                None => Weak::new(),
            };

            // Add to new parent
            if let Some(parent) = &new_parent {
                parent.borrow_mut().add_child_internal(&parent, instance.clone());
            }

            // Notify listeners
            for listener in &mut instance_mut.listeners {
                listener.on_parent_changed(new_parent.clone());
            }
        }
    }

    /// Internal method to add child (used by set_parent)
    fn add_child_internal(&mut self, self_rc: &Rc<RefCell<Instance>>, child: Rc<RefCell<Instance>>) {
        if Self::can_add_child(self_rc, &child) {
            self.children.push(child.clone());

            // Notify listeners
            for listener in &mut self.listeners {
                listener.on_child_added(child.clone());
                listener.on_descendant_added(child.clone());
            }

            // Notify descendants recursively
            self.notify_descendants_added(&child);
        }
    }

    /// Internal method to remove child (used by set_parent)
    fn remove_child_internal(&mut self, child: &Rc<RefCell<Instance>>) {
        if let Some(pos) = self.children.iter().position(|c| Rc::ptr_eq(c, child)) {
            let removed = self.children.remove(pos);

            // Notify listeners
            for listener in &mut self.listeners {
                listener.on_child_removed(removed.clone());
                listener.on_descendant_removed(removed.clone());
            }

            // Notify descendants recursively
            self.notify_descendants_removed(&removed);
        }
    }

    /// Notify all descendants of an addition
    fn notify_descendants_added(&mut self, instance: &Rc<RefCell<Instance>>) {
        for child in &self.children {
            if !Rc::ptr_eq(child, instance) {
                child.borrow_mut().notify_descendants_added(instance);
            }
        }

        for listener in &mut self.listeners {
            listener.on_descendant_added(instance.clone());
        }
    }

    /// Notify all descendants of a removal
    fn notify_descendants_removed(&mut self, instance: &Rc<RefCell<Instance>>) {
        for child in &self.children {
            child.borrow_mut().notify_descendants_removed(instance);
        }

        for listener in &mut self.listeners {
            listener.on_descendant_removed(instance.clone());
        }
    }

    /// Add a listener for hierarchy changes
    pub fn add_listener(&mut self, listener: Box<dyn InstanceListener>) {
        self.listeners.push(listener);
    }

    /// Get reflection properties for this instance
    pub fn get_properties(&self) -> Vec<ReflectionProperty> {
        let mut properties = Vec::new();

        // Name property
        let name_prop = ReflectionProperty::new(
            "Name",
            "This is the name of this Instance.",
            AccessType::None,
            OperationType::ReadWrite,
            PropertyType::String,
        )
        .with_getter(|obj| {
            let instance = obj.downcast_ref::<Instance>().unwrap();
            Box::new(instance.name().to_string())
        })
        .with_setter(|obj, value| {
            let instance = obj.downcast_mut::<Instance>().unwrap();
            let name = value.downcast_ref::<String>().unwrap();
            instance.set_name(name);
        });

        // Parent property (read-only)
        let parent_prop = ReflectionProperty::new(
            "Parent",
            "This is the parent of this Instance.",
            AccessType::None,
            OperationType::Read,
            PropertyType::Instance,
        )
        .with_getter(|obj| {
            let instance = obj.downcast_ref::<Instance>().unwrap();
            Box::new(instance.parent().clone())
        });

        // Archivable property
        let archivable_prop = ReflectionProperty::new(
            "Archivable",
            "This determines whether this Instance may be saved or replicated.",
            AccessType::None,
            OperationType::ReadWrite,
            PropertyType::Bool,
        )
        .with_getter(|obj| {
            let instance = obj.downcast_ref::<Instance>().unwrap();
            Box::new(instance.archivable())
        })
        .with_setter(|obj, value| {
            let instance = obj.downcast_mut::<Instance>().unwrap();
            let archivable = *value.downcast_ref::<bool>().unwrap();
            instance.set_archivable(archivable);
        });

        properties.push(name_prop);
        properties.push(parent_prop);
        properties.push(archivable_prop);

        // Allow subclasses to add more properties
        self.add_properties(&mut properties);

        properties
    }

    /// Get reflection functions for this instance
    pub fn get_functions(&self) -> Vec<ReflectionFunction> {
        let mut functions = Vec::new();

        // IsA function
        let is_a_func = ReflectionFunction::new(
            "IsA",
            "Returns true if the Instance is of the specified class.",
            |obj| {
                // This would need access to Lua state - simplified for now
                // In real implementation, this would check arguments from Lua stack
            },
        );

        // Clone function
        let clone_func = ReflectionFunction::new(
            "Clone",
            "Creates a copy of this Instance.",
            |obj| {
                // Implementation would create a clone
            },
        );

        // Destroy function
        let destroy_func = ReflectionFunction::new(
            "Destroy",
            "Removes this Instance from the game.",
            |_obj| {
                // This needs access to the Rc, so we'll handle it differently
                // For now, just a placeholder
            },
        );

        functions.push(is_a_func);
        functions.push(clone_func);
        functions.push(destroy_func);

        // Allow subclasses to add more functions
        self.add_functions(&mut functions);

        functions
    }

    /// Virtual method for subclasses to add properties
    fn add_properties(&self, _properties: &mut Vec<ReflectionProperty>) {
        // Default implementation does nothing
    }

    /// Virtual method for subclasses to add functions
    fn add_functions(&self, _functions: &mut Vec<ReflectionFunction>) {
        // Default implementation does nothing
    }

    /// Clone this instance
    pub fn clone(&self) -> Rc<RefCell<Instance>> {
        let cloned = Rc::new(RefCell::new(Instance {
            parent: Weak::new(),
            children: Vec::new(), // Children are not cloned by default
            name: self.name.clone(),
            archivable: self.archivable,
            class_name: self.class_name.clone(),
            listeners: Vec::new(), // Listeners are not cloned
        }));

        cloned
    }
}

impl Clone for Instance {
    fn clone(&self) -> Self {
        Self {
            parent: Weak::new(),
            children: Vec::new(),
            name: self.name.clone(),
            archivable: self.archivable,
            class_name: self.class_name.clone(),
            listeners: Vec::new(),
        }
    }
}

impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Instance")
            .field("name", &self.name)
            .field("class_name", &self.class_name)
            .field("archivable", &self.archivable)
            .field("children_count", &self.children.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_creation() {
        let instance = Instance::new();
        assert_eq!(instance.borrow().name(), "Instance");
        assert_eq!(instance.borrow().class_name(), "Instance");
        assert!(instance.borrow().archivable());
    }

    #[test]
    fn test_parent_child_relationship() {
        let parent = Instance::new();
        parent.borrow_mut().set_name("Parent");

        let child = Instance::new();
        child.borrow_mut().set_name("Child");

        Instance::set_parent(&child, Some(parent.clone()));

        assert!(parent.borrow().contains(&child));
        assert_eq!(child.borrow().parent().unwrap().borrow().name(), "Parent");
        assert_eq!(parent.borrow().num_children(), 1);
    }

    #[test]
    fn test_prevent_cycles() {
        let instance1 = Instance::new();
        let instance2 = Instance::new();

        Instance::set_parent(&instance1, Some(instance2.clone()));
        // This should fail because it would create a cycle
        Instance::set_parent(&instance2, Some(instance1.clone()));

        // instance2 should not have instance1 as parent due to cycle prevention
        assert!(instance2.borrow().parent().is_none() || !Rc::ptr_eq(&instance2.borrow().parent().unwrap(), &instance1));
    }
}
