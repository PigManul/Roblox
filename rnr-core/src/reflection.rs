use std::any::Any;

/// Property types for reflection system
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyType {
    Bool,
    Int,
    Float,
    String,
    Instance,
    Vector2,
    Vector3,
    CFrame,
}

/// Access permissions for properties
#[derive(Debug, Clone, PartialEq)]
pub enum AccessType {
    None,
    Read,
    Write,
    ReadWrite,
}

/// Operation types for properties
#[derive(Debug, Clone, PartialEq)]
pub enum OperationType {
    Read,
    ReadWrite,
}

/// A reflected property with getter/setter functions
pub struct ReflectionProperty {
    pub name: String,
    pub description: String,
    pub access: AccessType,
    pub operation: OperationType,
    pub property_type: PropertyType,
    // In Rust, we'll use closures for getters/setters
    pub getter: Option<Box<dyn Fn(&dyn Any) -> Box<dyn Any + 'static> + Send + Sync>>,
    pub setter: Option<Box<dyn Fn(&mut dyn Any, Box<dyn Any>) + Send + Sync>>,
}

impl ReflectionProperty {
    pub fn new(
        name: &str,
        description: &str,
        access: AccessType,
        operation: OperationType,
        property_type: PropertyType,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            access,
            operation,
            property_type,
            getter: None,
            setter: None,
        }
    }

    pub fn with_getter<F>(mut self, getter: F) -> Self
    where
        F: Fn(&dyn Any) -> Box<dyn Any + 'static> + Send + Sync + 'static,
    {
        self.getter = Some(Box::new(getter));
        self
    }

    pub fn with_setter<F>(mut self, setter: F) -> Self
    where
        F: Fn(&mut dyn Any, Box<dyn Any>) + Send + Sync + 'static,
    {
        self.setter = Some(Box::new(setter));
        self
    }
}

/// A reflected function/method
pub struct ReflectionFunction {
    pub name: String,
    pub description: String,
    pub function: Box<dyn Fn(&mut dyn Any) + Send + Sync>,
}

impl ReflectionFunction {
    pub fn new<F>(name: &str, description: &str, function: F) -> Self
    where
        F: Fn(&mut dyn Any) + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            function: Box::new(function),
        }
    }
}
