#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityKind {
    NativeModule,
    Webview,
    JsBridge,
    Filesystem,
    Network,
    Clipboard,
    Custom,
}

#[derive(Debug, Clone)]
pub struct Capability {
    pub kind: CapabilityKind,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct RuntimeContext {
    pub platform_name: String,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub target: Option<u64>,
}

pub type ModuleId = u64;

#[derive(Debug, Clone)]
pub struct ModuleHooks {
    pub start_fn: Option<fn(runtime: RuntimeContext) -> Result<(), ModuleError>>,
    pub stop_fn: Option<fn(runtime: RuntimeContext) -> Result<(), ModuleError>>,
    pub command_fn: Option<fn(runtime: RuntimeContext, Command) -> Result<(), ModuleError>>,
}

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub id: ModuleId,
    pub name: String,
    pub dependencies: Vec<ModuleId>,
    pub capabilities: Vec<Capability>,
}

#[derive(Debug)]
pub struct Module {
    pub info: ModuleInfo,
    pub context: Box<dyn std::any::Any>,
    pub hooks: ModuleHooks,
}

#[derive(Debug)]
pub struct ModuleRegistry {
    pub modules: Vec<Module>,
}

impl ModuleRegistry {
    pub fn validate(&self) -> Result<(), ModuleError> {
        for (index, module) in self.modules.iter().enumerate() {
            for prev in &self.modules[..index] {
                if prev.info.id == module.info.id {
                    return Err(ModuleError::DuplicateModule);
                }
            }
            for dep in &module.info.dependencies {
                if !self.modules.iter().any(|m| m.info.id == *dep) {
                    return Err(ModuleError::MissingDependency);
                }
            }
        }
        Ok(())
    }

    pub fn start_all(&self, runtime: RuntimeContext) -> Result<(), ModuleError> {
        self.validate()?;
        for module in &self.modules {
            if let Some(start_fn) = module.hooks.start_fn {
                start_fn(runtime.clone()).map_err(|_| ModuleError::ModuleFailed)?;
            }
        }
        Ok(())
    }

    pub fn stop_all(&self, runtime: RuntimeContext) -> Result<(), ModuleError> {
        for module in self.modules.iter().rev() {
            if let Some(stop_fn) = module.hooks.stop_fn {
                stop_fn(runtime.clone()).map_err(|_| ModuleError::ModuleFailed)?;
            }
        }
        Ok(())
    }

    pub fn dispatch_command(&self, runtime: RuntimeContext, command: Command) -> Result<(), ModuleError> {
        if let Some(target) = command.target {
            let module = self.find_by_id(target).ok_or(ModuleError::MissingDependency)?;
            if let Some(command_fn) = module.hooks.command_fn {
                command_fn(runtime, command).map_err(|_| ModuleError::ModuleFailed)?;
            }
            return Ok(());
        }
        for module in &self.modules {
            if let Some(command_fn) = module.hooks.command_fn {
                command_fn(runtime.clone(), command.clone()).map_err(|_| ModuleError::ModuleFailed)?;
            }
        }
        Ok(())
    }

    pub fn has_capability(&self, kind: CapabilityKind) -> bool {
        self.modules.iter().any(|m| {
            m.info.capabilities.iter().any(|c| std::mem::discriminant(&c.kind) == std::mem::discriminant(&kind))
        })
    }

    pub fn find_by_id(&self, id: ModuleId) -> Option<&Module> {
        self.modules.iter().find(|m| m.info.id == id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleError {
    DuplicateModule,
    MissingDependency,
    ModuleFailed,
}
