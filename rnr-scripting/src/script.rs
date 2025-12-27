use std::rc::Rc;
use std::cell::RefCell;
use std::time::{Duration, Instant};
use rnr_core::instance::Instance;
use crate::context::ScriptContext;
use crate::bridge::{ScriptValue, ScriptError, Scriptable};

/// Script execution state
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptState {
    Stopped,
    Running,
    Paused,
    Error(String),
}

/// Script instance - represents a script in the game world
pub struct Script {
    instance: Rc<RefCell<Instance>>,
    context: Option<Rc<RefCell<ScriptContext>>>,
    source: String,
    compiled_bytecode: Option<Vec<u8>>,
    state: ScriptState,
    resume_time: Option<Instant>,
    execution_time: Duration,
    error_message: Option<String>,
}

impl Script {
    /// Create a new script with source code
    pub fn new(source: String) -> Rc<RefCell<Self>> {
        let instance = Instance::new();
        instance.borrow_mut().set_name("Script");
        instance.borrow_mut().set_class_name("Script");

        Rc::new(RefCell::new(Self {
            instance,
            context: None,
            source,
            compiled_bytecode: None,
            state: ScriptState::Stopped,
            resume_time: None,
            execution_time: Duration::ZERO,
            error_message: None,
        }))
    }

    /// Create a script with a specific context
    pub fn with_context(source: String, context: Rc<RefCell<ScriptContext>>) -> Rc<RefCell<Self>> {
        let script = Self::new(source);
        script.borrow_mut().set_context(context);
        script
    }

    /// Get the instance
    pub fn instance(&self) -> &Rc<RefCell<Instance>> {
        &self.instance
    }

    /// Set the script context
    pub fn set_context(&mut self, context: Rc<RefCell<ScriptContext>>) {
        self.context = Some(context);
    }

    /// Get the script context
    pub fn context(&self) -> Option<&Rc<RefCell<ScriptContext>>> {
        self.context.as_ref()
    }

    /// Get the script source code
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Set the script source code
    pub fn set_source(&mut self, source: String) {
        self.source = source;
        self.compiled_bytecode = None; // Invalidate compiled bytecode
        self.state = ScriptState::Stopped;
    }

    /// Compile the script
    pub fn compile(&mut self) -> Result<(), ScriptError> {
        if let Some(context) = &self.context {
            let script_name = format!("Script_{}", self.instance.borrow().name());
            context.borrow_mut().load_script(&script_name, &self.source)?;
            self.state = ScriptState::Stopped;
            Ok(())
        } else {
            Err(ScriptError::RuntimeError("No script context set".to_string()))
        }
    }

    /// Execute the script
    pub fn execute(&mut self) -> Result<(), ScriptError> {
        let start_time = Instant::now();

        if let Some(context) = &self.context {
            let script_name = format!("Script_{}", self.instance.borrow().name());
            context.borrow_mut().execute_script(&script_name)?;

            self.execution_time = start_time.elapsed();
            self.state = ScriptState::Running;
            Ok(())
        } else {
            Err(ScriptError::RuntimeError("No script context set".to_string()))
        }
    }

    /// Pause script execution (for coroutines)
    pub fn pause(&mut self) {
        self.state = ScriptState::Paused;
    }

    /// Resume script execution
    pub fn resume(&mut self) -> Result<(), ScriptError> {
        if self.state == ScriptState::Paused {
            self.state = ScriptState::Running;
            Ok(())
        } else {
            Err(ScriptError::RuntimeError("Script is not paused".to_string()))
        }
    }

    /// Stop script execution
    pub fn stop(&mut self) {
        self.state = ScriptState::Stopped;
        self.resume_time = None;
    }

    /// Get the current script state
    pub fn state(&self) -> &ScriptState {
        &self.state
    }

    /// Check if the script should resume (for timed waits)
    pub fn should_resume(&self, current_time: Instant) -> bool {
        if let Some(resume_time) = self.resume_time {
            current_time >= resume_time
        } else {
            false
        }
    }

    /// Set resume time for timed waits
    pub fn set_resume_time(&mut self, resume_time: Instant) {
        self.resume_time = Some(resume_time);
    }

    /// Get resume time
    pub fn resume_time(&self) -> Option<Instant> {
        self.resume_time
    }

    /// Get execution time
    pub fn execution_time(&self) -> Duration {
        self.execution_time
    }

    /// Get error message (if any)
    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    /// Check if script is disabled
    pub fn disabled(&self) -> bool {
        !self.instance.borrow().archivable()
    }

    /// Set disabled state
    pub fn set_disabled(&mut self, disabled: bool) {
        self.instance.borrow_mut().set_archivable(!disabled);
        if disabled {
            self.stop();
        }
    }

    /// Update script (handle timing, etc.)
    pub fn update(&mut self, current_time: Instant) {
        if self.state == ScriptState::Paused && self.should_resume(current_time) {
            let _ = self.resume();
        }
    }

    /// Clone the script
    pub fn clone_script(&self) -> Rc<RefCell<Script>> {
        let cloned = Rc::new(RefCell::new(Script {
            instance: Instance::new(),
            context: self.context.clone(),
            source: self.source.clone(),
            compiled_bytecode: self.compiled_bytecode.clone(),
            state: ScriptState::Stopped,
            resume_time: None,
            execution_time: Duration::ZERO,
            error_message: None,
        }));

        cloned.borrow().instance.borrow_mut().set_name(self.instance.borrow().name());
        cloned
    }
}

impl Scriptable for Script {
    fn script_class_name(&self) -> &str {
        "Script"
    }

    fn get_script_property(&self, name: &str) -> Option<ScriptValue> {
        match name {
            "Name" => Some(ScriptValue::String(self.instance.borrow().name().to_string())),
            "Source" => Some(ScriptValue::String(self.source.clone())),
            "Disabled" => Some(ScriptValue::Bool(self.disabled())),
            "LinkedSource" => Some(ScriptValue::String(String::new())), // Placeholder
            _ => None,
        }
    }

    fn set_script_property(&mut self, name: &str, value: ScriptValue) -> Result<(), ScriptError> {
        match name {
            "Name" => {
                if let ScriptValue::String(name_str) = value {
                    self.instance.borrow_mut().set_name(&name_str);
                    Ok(())
                } else {
                    Err(ScriptError::TypeMismatch("Expected string for Name".to_string()))
                }
            }
            "Source" => {
                if let ScriptValue::String(source_str) = value {
                    self.set_source(source_str);
                    Ok(())
                } else {
                    Err(ScriptError::TypeMismatch("Expected string for Source".to_string()))
                }
            }
            "Disabled" => {
                if let ScriptValue::Bool(disabled) = value {
                    self.set_disabled(disabled);
                    Ok(())
                } else {
                    Err(ScriptError::TypeMismatch("Expected bool for Disabled".to_string()))
                }
            }
            _ => Err(ScriptError::PropertyNotFound(name.to_string())),
        }
    }

    fn call_script_method(&mut self, name: &str, args: Vec<ScriptValue>) -> Result<ScriptValue, ScriptError> {
        match name {
            "Clone" => Ok(ScriptValue::Instance(self.instance.clone())),
            "Destroy" => {
                // Remove from parent
                rnr_core::instance::Instance::set_parent(&self.instance, None);
                Ok(ScriptValue::Nil)
            }
            _ => Err(ScriptError::MethodNotFound(name.to_string())),
        }
    }
}

/// Script service - manages scripts in the data model
pub struct ScriptService {
    instance: Rc<RefCell<Instance>>,
    scripts: Vec<Rc<RefCell<Script>>>,
}

impl ScriptService {
    /// Create a new script service
    pub fn new() -> Rc<RefCell<Self>> {
        let instance = Instance::new();
        instance.borrow_mut().set_name("ScriptService");
        instance.borrow_mut().set_class_name("ScriptService");

        Rc::new(RefCell::new(Self {
            instance,
            scripts: Vec::new(),
        }))
    }

    /// Get the instance
    pub fn instance(&self) -> &Rc<RefCell<Instance>> {
        &self.instance
    }

    /// Add a script to the service
    pub fn add_script(&mut self, script: Rc<RefCell<Script>>) {
        self.scripts.push(script);
    }

    /// Remove a script from the service
    pub fn remove_script(&mut self, script: &Rc<RefCell<Script>>) {
        self.scripts.retain(|s| !Rc::ptr_eq(s, script));
    }

    /// Get all scripts
    pub fn scripts(&self) -> &[Rc<RefCell<Script>>] {
        &self.scripts
    }

    /// Update all scripts
    pub fn update_scripts(&mut self, current_time: Instant) {
        for script in &self.scripts {
            script.borrow_mut().update(current_time);
        }
    }

    /// Compile all scripts
    pub fn compile_all_scripts(&mut self) -> Result<(), ScriptError> {
        for script in &self.scripts {
            script.borrow_mut().compile()?;
        }
        Ok(())
    }

    /// Stop all scripts
    pub fn stop_all_scripts(&mut self) {
        for script in &self.scripts {
            script.borrow_mut().stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::NullScriptingBridge;
    use crate::context::ScriptContext;

    #[test]
    fn test_script_creation() {
        let script = Script::new("print('Hello, World!')".to_string());

        assert_eq!(script.borrow().source(), "print('Hello, World!')");
        assert_eq!(*script.borrow().state(), ScriptState::Stopped);
        assert!(!script.borrow().disabled());
    }

    #[test]
    fn test_script_properties() {
        let script = Script::new("test script".to_string());

        // Test getting properties
        assert_eq!(script.borrow().get_script_property("Source"),
                   Some(ScriptValue::String("test script".to_string())));
        assert_eq!(script.borrow().get_script_property("Disabled"),
                   Some(ScriptValue::Bool(false)));
        assert!(script.borrow().get_script_property("Nonexistent").is_none());

        // Test setting properties
        assert!(script.borrow_mut().set_script_property("Source",
            ScriptValue::String("new source".to_string())).is_ok());
        assert_eq!(script.borrow().source(), "new source");

        assert!(script.borrow_mut().set_script_property("Disabled",
            ScriptValue::Bool(true)).is_ok());
        assert!(script.borrow().disabled());
    }

    #[test]
    fn test_script_execution() {
        let bridge = Box::new(NullScriptingBridge);
        let context = Rc::new(RefCell::new(ScriptContext::new(bridge)));
        context.borrow_mut().initialize().unwrap();

        let script = Script::with_context("test".to_string(), context);

        // Without proper context, execution should fail
        assert!(script.borrow_mut().compile().is_err());
        assert!(script.borrow_mut().execute().is_err());
    }

    #[test]
    fn test_script_service() {
        let service = ScriptService::new();
        let script1 = Script::new("script1".to_string());
        let script2 = Script::new("script2".to_string());

        service.borrow_mut().add_script(script1.clone());
        service.borrow_mut().add_script(script2.clone());

        assert_eq!(service.borrow().scripts().len(), 2);

        service.borrow_mut().remove_script(&script1);
        assert_eq!(service.borrow().scripts().len(), 1);
    }
}
