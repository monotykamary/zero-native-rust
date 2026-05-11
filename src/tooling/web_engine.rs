#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Engine {
    System,
    Chromium,
}

impl Engine {
    pub fn parse(value: &str) -> Option<Engine> {
        match value {
            "system" => Some(Engine::System),
            "chromium" => Some(Engine::Chromium),
            _ => None,
        }
    }
}

pub const DEFAULT_CEF_DIR: &str = "third_party/cef/macos";

#[derive(Debug, Clone)]
pub struct CefConfig {
    pub dir: String,
    pub auto_install: bool,
}

impl Default for CefConfig {
    fn default() -> Self {
        Self {
            dir: DEFAULT_CEF_DIR.into(),
            auto_install: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Overrides {
    pub web_engine: Option<Engine>,
    pub cef_dir: Option<String>,
    pub cef_auto_install: Option<bool>,
}

impl Default for Overrides {
    fn default() -> Self {
        Self {
            web_engine: None,
            cef_dir: None,
            cef_auto_install: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueSource {
    Default,
    Manifest,
    Override,
}

#[derive(Debug, Clone)]
pub struct Resolved {
    pub engine: Engine,
    pub cef_dir: String,
    pub cef_auto_install: bool,
    pub engine_source: ValueSource,
    pub cef_dir_source: ValueSource,
    pub cef_auto_install_source: ValueSource,
}

pub fn resolve(manifest_engine: &str, cef_config: &CefConfig, overrides: &Overrides) -> Result<Resolved, String> {
    let parsed_engine = Engine::parse(manifest_engine).ok_or("invalid web engine")?;
    let engine = overrides.web_engine.unwrap_or(parsed_engine);
    let cef_dir = overrides.cef_dir.clone().unwrap_or_else(|| cef_config.dir.clone());
    let cef_auto_install = overrides.cef_auto_install.unwrap_or(cef_config.auto_install);

    let engine_source = if overrides.web_engine.is_some() {
        ValueSource::Override
    } else if manifest_engine == "system" {
        ValueSource::Default
    } else {
        ValueSource::Manifest
    };

    let cef_dir_source = if overrides.cef_dir.is_some() {
        ValueSource::Override
    } else if cef_config.dir == DEFAULT_CEF_DIR {
        ValueSource::Default
    } else {
        ValueSource::Manifest
    };

    let cef_auto_install_source = if overrides.cef_auto_install.is_some() {
        ValueSource::Override
    } else if !cef_config.auto_install {
        ValueSource::Default
    } else {
        ValueSource::Manifest
    };

    Ok(Resolved {
        engine,
        cef_dir,
        cef_auto_install,
        engine_source,
        cef_dir_source,
        cef_auto_install_source,
    })
}
