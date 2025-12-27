use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use rnr_core::instance::Instance;

/// Trait for objects that can be exposed to scripts
pub trait Scriptable {
    /// Get the script class name
    fn script_class_name(&self) -> &str;

    /// Get a property value by name
    fn get_script_property(&self, name: &str) -> Option<ScriptValue>;

    /// Set a property value by name
    fn set_script_property(&mut self, name: &str, value: ScriptValue) -> Result<(), ScriptError>;

    /// Call a method by name
    fn call_script_method(&mut self, name: &str, args: Vec<ScriptValue>) -> Result<ScriptValue, ScriptError>;
}

/// Script value types
#[derive(Debug, Clone)]
pub enum ScriptValue {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Instance(Rc<RefCell<Instance>>),
    Array(Vec<ScriptValue>),
    Table(HashMap<String, ScriptValue>),
}

/// Script execution errors
#[derive(Debug, Clone)]
pub enum ScriptError {
    PropertyNotFound(String),
    MethodNotFound(String),
    InvalidArguments(String),
    TypeMismatch(String),
    RuntimeError(String),
}

impl std::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptError::PropertyNotFound(name) => write!(f, "Property '{}' not found", name),
            ScriptError::MethodNotFound(name) => write!(f, "Method '{}' not found", name),
            ScriptError::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
            ScriptError::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            ScriptError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
        }
    }
}

impl std::error::Error for ScriptError {}

/// Scripting bridge trait - abstracts the underlying scripting engine
pub trait ScriptingBridge {
    /// Initialize the scripting engine
    fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Load and compile a script
    fn load_script(&mut self, name: &str, source: &str) -> Result<ScriptHandle, ScriptError>;

    /// Execute a loaded script
    fn execute_script(&mut self, handle: &ScriptHandle) -> Result<(), ScriptError>;

    /// Call a global function
    fn call_global_function(&mut self, name: &str, args: Vec<ScriptValue>) -> Result<ScriptValue, ScriptError>;

    /// Get a global variable
    fn get_global(&self, name: &str) -> Option<ScriptValue>;

    /// Set a global variable
    fn set_global(&mut self, name: &str, value: ScriptValue) -> Result<(), ScriptError>;

    /// Register a scriptable object
    fn register_object(&mut self, name: &str, object: Rc<RefCell<dyn Scriptable>>) -> Result<(), ScriptError>;

    /// Update the scripting engine (handle coroutines, etc.)
    fn update(&mut self, delta_time: f32);

    /// Shutdown the scripting engine
    fn shutdown(&mut self);
}

/// Script handle for loaded scripts
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ScriptHandle {
    pub id: u64,
    pub name: String,
}

/// Null scripting bridge for when scripting is disabled
pub struct NullScriptingBridge;

impl ScriptingBridge for NullScriptingBridge {
    fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn load_script(&mut self, _name: &str, _source: &str) -> Result<ScriptHandle, ScriptError> {
        Err(ScriptError::RuntimeError("Scripting is disabled".to_string()))
    }

    fn execute_script(&mut self, _handle: &ScriptHandle) -> Result<(), ScriptError> {
        Err(ScriptError::RuntimeError("Scripting is disabled".to_string()))
    }

    fn call_global_function(&mut self, _name: &str, _args: Vec<ScriptValue>) -> Result<ScriptValue, ScriptError> {
        Err(ScriptError::RuntimeError("Scripting is disabled".to_string()))
    }

    fn get_global(&self, _name: &str) -> Option<ScriptValue> {
        None
    }

    fn set_global(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
        Err(ScriptError::RuntimeError("Scripting is disabled".to_string()))
    }

    fn register_object(&mut self, _name: &str, _object: Rc<RefCell<dyn Scriptable>>) -> Result<(), ScriptError> {
        Err(ScriptError::RuntimeError("Scripting is disabled".to_string()))
    }

    fn update(&mut self, _delta_time: f32) {}

    fn shutdown(&mut self) {}
}

/// Instance bridge - provides Lua/Rust interop for Instances
pub struct InstanceBridge {
    registered_instances: HashMap<u64, Rc<RefCell<Instance>>>,
    next_id: u64,
}

impl InstanceBridge {
    pub fn new() -> Self {
        Self {
            registered_instances: HashMap::new(),
            next_id: 1,
        }
    }

    /// Register an instance for script access
    pub fn register_instance(&mut self, instance: Rc<RefCell<Instance>>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.registered_instances.insert(id, instance);
        id
    }

    /// Get an instance by its script ID
    pub fn get_instance(&self, id: u64) -> Option<Rc<RefCell<Instance>>> {
        self.registered_instances.get(&id).cloned()
    }

    /// Unregister an instance
    pub fn unregister_instance(&mut self, id: u64) {
        self.registered_instances.remove(&id);
    }
}

impl Default for InstanceBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Global instance bridge
static mut INSTANCE_BRIDGE: Option<InstanceBridge> = None;

/// Initialize the global instance bridge
pub fn initialize_instance_bridge() {
    unsafe {
        INSTANCE_BRIDGE = Some(InstanceBridge::new());
    }
}

/// Get the global instance bridge
pub fn get_instance_bridge() -> Option<&'static mut InstanceBridge> {
    unsafe {
        INSTANCE_BRIDGE.as_mut()
    }
}

/// Helper functions for common script operations
pub mod helpers {
    use super::*;

    /// Convert a ScriptValue to a string for debugging
    pub fn script_value_to_string(value: &ScriptValue) -> String {
        match value {
            ScriptValue::Nil => "nil".to_string(),
            ScriptValue::Bool(b) => b.to_string(),
            ScriptValue::Int(i) => i.to_string(),
            ScriptValue::Float(f) => f.to_string(),
            ScriptValue::String(s) => format!("\"{}\"", s),
            ScriptValue::Instance(_) => "<Instance>".to_string(),
            ScriptValue::Array(arr) => format!("[{} items]", arr.len()),
            ScriptValue::Table(tbl) => format!("{{{}}} entries", tbl.len()),
        }
    }

    /// Create a ScriptValue from a primitive
    pub fn from_bool(value: bool) -> ScriptValue {
        ScriptValue::Bool(value)
    }

    pub fn from_int(value: i64) -> ScriptValue {
        ScriptValue::Int(value)
    }

    pub fn from_float(value: f64) -> ScriptValue {
        ScriptValue::Float(value)
    }

    pub fn from_string(value: String) -> ScriptValue {
        ScriptValue::String(value)
    }

    pub fn from_instance(instance: Rc<RefCell<Instance>>) -> ScriptValue {
        ScriptValue::Instance(instance)
    }

    /// Extract values from ScriptValue
    pub fn to_bool(value: &ScriptValue) -> Option<bool> {
        match value {
            ScriptValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn to_int(value: &ScriptValue) -> Option<i64> {
        match value {
            ScriptValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn to_float(value: &ScriptValue) -> Option<f64> {
        match value {
            ScriptValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn to_string(value: &ScriptValue) -> Option<&str> {
        match value {
            ScriptValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn to_instance(value: &ScriptValue) -> Option<Rc<RefCell<Instance>>> {
        match value {
            ScriptValue::Instance(inst) => Some(inst.clone()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rnr_core::instance::Instance;

    #[test]
    fn test_script_value_conversions() {
        assert_eq!(helpers::to_bool(&ScriptValue::Bool(true)), Some(true));
        assert_eq!(helpers::to_int(&ScriptValue::Int(42)), Some(42));
        assert_eq!(helpers::to_float(&ScriptValue::Float(3.14)), Some(3.14));
        assert_eq!(helpers::to_string(&ScriptValue::String("hello".to_string())), Some("hello"));

        let instance = Instance::new();
        let value = helpers::from_instance(instance.clone());
        assert!(helpers::to_instance(&value).is_some());
    }

    #[test]
    fn test_instance_bridge() {
        let mut bridge = InstanceBridge::new();
        let instance = Instance::new();

        let id = bridge.register_instance(instance.clone());
        assert!(bridge.get_instance(id).is_some());
        assert_eq!(bridge.get_instance(id + 1), None);

        bridge.unregister_instance(id);
        assert!(bridge.get_instance(id).is_none());
    }

    #[test]
    fn test_null_bridge() {
        let mut bridge = NullScriptingBridge;

        // Should initialize without error
        assert!(bridge.initialize().is_ok());

        // All operations should fail gracefully
        assert!(bridge.load_script("test", "print('hello')").is_err());
        assert!(bridge.get_global("test").is_none());
        assert!(bridge.set_global("test", ScriptValue::Nil).is_err());

        // Should not panic on update/shutdown
        bridge.update(0.016);
        bridge.shutdown();
    }
}
