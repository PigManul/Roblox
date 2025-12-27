use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use rnr_core::instance::{Instance, InstanceListener};

/// The DataModel is the root of the instance tree and manages services
pub struct DataModel {
    instance: Rc<RefCell<Instance>>,
    /// GUID to instance mapping for network replication
    guid_map: HashMap<String, Rc<RefCell<Instance>>>,
    /// Services provided by the DataModel
    services: HashMap<String, Rc<RefCell<Instance>>>,
}

impl DataModel {
    /// Create a new DataModel
    pub fn new() -> Rc<RefCell<Self>> {
        let instance = Instance::new();
        instance.borrow_mut().set_name("DataModel");
        instance.borrow_mut().set_class_name("DataModel");

        Rc::new(RefCell::new(Self {
            instance,
            guid_map: HashMap::new(),
            services: HashMap::new(),
        }))
    }

    /// Get the underlying instance
    pub fn instance(&self) -> &Rc<RefCell<Instance>> {
        &self.instance
    }

    /// Get a service by name
    pub fn get_service(&self, service_name: &str) -> Option<Rc<RefCell<Instance>>> {
        self.services.get(service_name).cloned()
    }

    /// Register a service
    pub fn register_service(&mut self, service_name: &str, service: Rc<RefCell<Instance>>) {
        // Set parent to DataModel
        rnr_core::instance::Instance::set_parent(&service, Some(self.instance.clone()));
        self.services.insert(service_name.to_string(), service);
    }

    /// Get instance by GUID
    pub fn get_instance_by_guid(&self, guid: &str) -> Option<Rc<RefCell<Instance>>> {
        self.guid_map.get(guid).cloned()
    }

    /// Register instance with GUID
    pub fn register_instance_guid(&mut self, instance: Rc<RefCell<Instance>>, guid: String) {
        self.guid_map.insert(guid, instance);
    }

    /// Remove instance by GUID
    pub fn remove_instance_guid(&mut self, guid: &str) {
        self.guid_map.remove(guid);
    }

    /// Get GUID for instance
    pub fn get_guid_for_instance(&self, instance: &Rc<RefCell<Instance>>) -> Option<String> {
        for (guid, inst) in &self.guid_map {
            if Rc::ptr_eq(inst, instance) {
                return Some(guid.clone());
            }
        }
        None
    }
}

impl InstanceListener for DataModel {
    fn on_child_added(&mut self, child: Rc<RefCell<Instance>>) {
        // Services are automatically registered when added as children
        // In a real implementation, this would check if the child is a service
        // and register it in the services map
    }

    fn on_child_removed(&mut self, child: Rc<RefCell<Instance>>) {
        // Unregister services when removed
        let child_name = child.borrow().name().to_string();
        self.services.remove(&child_name);
    }

    fn on_descendant_added(&mut self, _descendant: Rc<RefCell<Instance>>) {
        // Handle descendant additions (for GUID registration, etc.)
    }

    fn on_descendant_removed(&mut self, _descendant: Rc<RefCell<Instance>>) {
        // Handle descendant removals (for GUID cleanup, etc.)
    }

    fn on_parent_changed(&mut self, _new_parent: Option<Rc<RefCell<Instance>>>) {
        // DataModel should not have a parent
    }
}

/// Extend Instance to add DataModel-specific properties
pub trait DataModelInstanceExt {
    fn get_datamodel(&self) -> Option<Rc<RefCell<DataModel>>>;
}

impl DataModelInstanceExt for Instance {
    fn get_datamodel(&self) -> Option<Rc<RefCell<DataModel>>> {
        let mut current = self.parent();
        while let Some(parent) = current {
            if parent.borrow().is_a("DataModel") {
                // In a real implementation, we'd need to downcast to DataModel
                // For now, return None as we can't safely downcast
                return None;
            }
            current = parent.borrow().parent();
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_datamodel_creation() {
        let datamodel = DataModel::new();
        assert_eq!(datamodel.borrow().instance().borrow().name(), "DataModel");
        assert_eq!(datamodel.borrow().instance().borrow().class_name(), "DataModel");
    }

    #[test]
    fn test_service_registration() {
        let datamodel = DataModel::new();
        let service = Instance::new();
        service.borrow_mut().set_name("TestService");

        datamodel.borrow_mut().register_service("TestService", service.clone());

        assert!(datamodel.borrow().get_service("TestService").is_some());
        assert_eq!(datamodel.borrow().get_service("TestService").unwrap().borrow().name(), "TestService");
    }

    #[test]
    fn test_guid_registration() {
        let datamodel = DataModel::new();
        let instance = Instance::new();

        datamodel.borrow_mut().register_instance_guid(instance.clone(), "test-guid".to_string());

        assert!(datamodel.borrow().get_instance_by_guid("test-guid").is_some());
        assert_eq!(datamodel.borrow().get_guid_for_instance(&instance), Some("test-guid".to_string()));
    }
}
