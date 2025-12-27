use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::bridge::{ScriptingBridge, ScriptHandle, ScriptValue, ScriptError, Scriptable};

/// Script execution context
pub struct ScriptContext {
    bridge: Box<dyn ScriptingBridge>,
    loaded_scripts: HashMap<String, ScriptHandle>,
    registered_objects: HashMap<String, Rc<RefCell<dyn Scriptable>>>,
    is_running: bool,
}

impl ScriptContext {
    /// Create a new script context with the given scripting bridge
    pub fn new(bridge: Box<dyn ScriptingBridge>) -> Self {
        Self {
            bridge,
            loaded_scripts: HashMap::new(),
            registered_objects: HashMap::new(),
            is_running: false,
        }
    }

    /// Initialize the script context
    pub fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.bridge.initialize()?;
        self.is_running = true;
        Ok(())
    }

    /// Load a script from source code
    pub fn load_script(&mut self, name: &str, source: &str) -> Result<(), ScriptError> {
        let handle = self.bridge.load_script(name, source)?;
        self.loaded_scripts.insert(name.to_string(), handle);
        Ok(())
    }

    /// Execute a loaded script
    pub fn execute_script(&mut self, name: &str) -> Result<(), ScriptError> {
        if let Some(handle) = self.loaded_scripts.get(name) {
            self.bridge.execute_script(handle)
        } else {
            Err(ScriptError::RuntimeError(format!("Script '{}' not found", name)))
        }
    }

    /// Call a global function
    pub fn call_global_function(&mut self, name: &str, args: Vec<ScriptValue>) -> Result<ScriptValue, ScriptError> {
        self.bridge.call_global_function(name, args)
    }

    /// Get a global variable
    pub fn get_global(&self, name: &str) -> Option<ScriptValue> {
        self.bridge.get_global(name)
    }

    /// Set a global variable
    pub fn set_global(&mut self, name: &str, value: ScriptValue) -> Result<(), ScriptError> {
        self.bridge.set_global(name, value)
    }

    /// Register a scriptable object
    pub fn register_object(&mut self, name: &str, object: Rc<RefCell<dyn Scriptable>>) -> Result<(), ScriptError> {
        self.registered_objects.insert(name.to_string(), object.clone());
        self.bridge.register_object(name, object)
    }

    /// Get a registered object
    pub fn get_registered_object(&self, name: &str) -> Option<Rc<RefCell<dyn Scriptable>>> {
        self.registered_objects.get(name).cloned()
    }

    /// Update the script context
    pub fn update(&mut self, delta_time: f32) {
        if self.is_running {
            self.bridge.update(delta_time);
        }
    }

    /// Shutdown the script context
    pub fn shutdown(&mut self) {
        self.is_running = false;
        self.bridge.shutdown();
    }

    /// Check if the context is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Get all loaded script names
    pub fn loaded_scripts(&self) -> Vec<String> {
        self.loaded_scripts.keys().cloned().collect()
    }

    /// Unload a script
    pub fn unload_script(&mut self, name: &str) {
        self.loaded_scripts.remove(name);
    }

    /// Reload all scripts (useful for development)
    pub fn reload_all_scripts(&mut self) -> Result<(), ScriptError> {
        // In a real implementation, this would reload scripts from disk
        // For now, just re-execute all loaded scripts
        let script_names: Vec<String> = self.loaded_scripts.keys().cloned().collect();

        for name in script_names {
            self.execute_script(&name)?;
        }

        Ok(())
    }
}

impl Drop for ScriptContext {
    fn drop(&mut self) {
        if self.is_running {
            self.shutdown();
        }
    }
}

/// Wait function for scripts (coroutine support)
pub async fn wait(seconds: f32) {
    tokio::time::sleep(std::time::Duration::from_secs_f32(seconds)).await;
}

/// Spawn a script task (simplified for now)
pub fn spawn_script<F, Fut>(_context: Rc<RefCell<ScriptContext>>, _future: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    // TODO: Implement proper script task spawning
    // For now, just a placeholder to avoid compilation issues
}

/// Script runner for managing multiple script contexts
pub struct ScriptRunner {
    contexts: HashMap<String, Rc<RefCell<ScriptContext>>>,
}

impl ScriptRunner {
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
        }
    }

    /// Create a new script context
    pub fn create_context(&mut self, name: &str, bridge: Box<dyn ScriptingBridge>) -> Rc<RefCell<ScriptContext>> {
        let context = Rc::new(RefCell::new(ScriptContext::new(bridge)));
        self.contexts.insert(name.to_string(), context.clone());
        context
    }

    /// Get a context by name
    pub fn get_context(&self, name: &str) -> Option<Rc<RefCell<ScriptContext>>> {
        self.contexts.get(name).cloned()
    }

    /// Update all contexts
    pub fn update_all(&mut self, delta_time: f32) {
        for context in self.contexts.values() {
            context.borrow_mut().update(delta_time);
        }
    }

    /// Shutdown all contexts
    pub fn shutdown_all(&mut self) {
        for context in self.contexts.values() {
            context.borrow_mut().shutdown();
        }
        self.contexts.clear();
    }

    /// Get all context names
    pub fn context_names(&self) -> Vec<String> {
        self.contexts.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::NullScriptingBridge;

    #[test]
    fn test_script_context_creation() {
        let bridge = Box::new(NullScriptingBridge);
        let mut context = ScriptContext::new(bridge);

        assert!(!context.is_running());
        assert!(context.loaded_scripts().is_empty());
    }

    #[test]
    fn test_script_context_lifecycle() {
        let bridge = Box::new(NullScriptingBridge);
        let mut context = ScriptContext::new(bridge);

        // Initialize
        context.initialize().unwrap();
        assert!(context.is_running());

        // Load script (should fail with null bridge)
        assert!(context.load_script("test", "print('hello')").is_err());

        // Shutdown
        context.shutdown();
        assert!(!context.is_running());
    }

    #[test]
    fn test_script_runner() {
        let mut runner = ScriptRunner::new();

        assert!(runner.context_names().is_empty());

        let bridge = Box::new(NullScriptingBridge);
        let _context = runner.create_context("test", bridge);

        assert_eq!(runner.context_names(), vec!["test"]);
        assert!(runner.get_context("test").is_some());
        assert!(runner.get_context("nonexistent").is_none());

        runner.update_all(0.016);
        runner.shutdown_all();
        assert!(runner.context_names().is_empty());
    }
}
