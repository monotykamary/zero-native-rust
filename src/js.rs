#[derive(Debug, Clone)]
pub enum ValueKind {
    Null,
    Boolean,
    Number,
    String,
}

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
}

#[derive(Debug, Clone)]
pub struct Call {
    pub module: String,
    pub function: String,
    pub args: Vec<Value>,
}

pub struct RuntimeHooks {
    pub call_fn: Option<fn(&mut dyn std::any::Any, Call) -> Result<Value, JsError>>,
}

pub struct Bridge {
    pub hooks: RuntimeHooks,
}

impl Bridge {
    pub fn call(&self, value: Call) -> Result<Value, JsError> {
        if value.module.is_empty() || value.function.is_empty() {
            return Err(JsError::InvalidCall);
        }
        match self.hooks.call_fn {
            Some(call_fn) => Err(JsError::EngineUnavailable), // would need context
            None => Err(JsError::EngineUnavailable),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsError {
    EngineUnavailable,
    InvalidCall,
}
