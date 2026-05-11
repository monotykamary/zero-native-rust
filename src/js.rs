#[derive(Debug, Clone)]
pub enum ValueKind {
    Null,
    Boolean,
    Number,
    String,
}

#[derive(Debug, Clone, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_bridge_reports_unavailable_engine() {
        let bridge = Bridge { hooks: RuntimeHooks { call_fn: None } };
        assert_eq!(Err(JsError::EngineUnavailable), bridge.call(Call {
            module: "app".into(),
            function: "main".into(),
            args: vec![],
        }));
    }

    #[test]
    fn bridge_validates_call_names() {
        let bridge = Bridge { hooks: RuntimeHooks { call_fn: None } };
        assert_eq!(Err(JsError::InvalidCall), bridge.call(Call {
            module: "".into(),
            function: "main".into(),
            args: vec![],
        }));
        assert_eq!(Err(JsError::InvalidCall), bridge.call(Call {
            module: "app".into(),
            function: "".into(),
            args: vec![],
        }));
    }
}
