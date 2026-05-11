#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    MacOS,
    Windows,
    Linux,
    IOS,
    Android,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirKind {
    Config,
    Cache,
    Data,
    State,
    Logs,
    Temp,
}

#[derive(Debug, Clone)]
pub struct AppInfo {
    pub name: String,
    pub organization: Option<String>,
    pub qualifier: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Env {
    pub home: Option<String>,
    pub xdg_config_home: Option<String>,
    pub xdg_cache_home: Option<String>,
    pub xdg_data_home: Option<String>,
    pub xdg_state_home: Option<String>,
    pub local_app_data: Option<String>,
    pub app_data: Option<String>,
    pub temp: Option<String>,
    pub tmp: Option<String>,
    pub tmpdir: Option<String>,
}

impl Default for Env {
    fn default() -> Self {
        Self {
            home: None,
            xdg_config_home: None,
            xdg_cache_home: None,
            xdg_data_home: None,
            xdg_state_home: None,
            local_app_data: None,
            app_data: None,
            temp: None,
            tmp: None,
            tmpdir: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedDirs {
    pub config: String,
    pub cache: String,
    pub data: String,
    pub state: String,
    pub logs: String,
    pub temp: String,
}

pub fn resolve_one(
    app: &AppInfo,
    platform: Platform,
    env: &Env,
    kind: DirKind,
) -> Result<String, AppDirError> {
    validate_app_name(&app.name)?;
    match platform {
        Platform::Linux => resolve_linux(&app.name, env, kind),
        Platform::MacOS => resolve_macos(&app.name, env, kind),
        Platform::Windows => resolve_windows(&app.name, env, kind),
        _ => Err(AppDirError::UnsupportedPlatform),
    }
}

fn resolve_linux(app_name: &str, env: &Env, kind: DirKind) -> Result<String, AppDirError> {
    let home = env.home.as_deref().ok_or(AppDirError::MissingHome)?;
    match kind {
        DirKind::Config => Ok(match &env.xdg_config_home {
            Some(root) => join(&[root, app_name]),
            None => join(&[home, ".config", app_name]),
        }),
        DirKind::Cache => Ok(match &env.xdg_cache_home {
            Some(root) => join(&[root, app_name]),
            None => join(&[home, ".cache", app_name]),
        }),
        DirKind::Data => Ok(match &env.xdg_data_home {
            Some(root) => join(&[root, app_name]),
            None => join(&[home, ".local", "share", app_name]),
        }),
        DirKind::State => Ok(match &env.xdg_state_home {
            Some(root) => join(&[root, app_name]),
            None => join(&[home, ".local", "state", app_name]),
        }),
        DirKind::Logs => Ok(match &env.xdg_state_home {
            Some(root) => join(&[root, app_name, "logs"]),
            None => join(&[home, ".local", "state", app_name, "logs"]),
        }),
        DirKind::Temp => Ok(join(&[env.tmpdir.as_deref().unwrap_or("/tmp"), app_name])),
    }
}

fn resolve_macos(app_name: &str, env: &Env, kind: DirKind) -> Result<String, AppDirError> {
    let home = env.home.as_deref().ok_or(AppDirError::MissingHome)?;
    match kind {
        DirKind::Config => Ok(join(&[home, "Library", "Preferences", app_name])),
        DirKind::Cache => Ok(join(&[home, "Library", "Caches", app_name])),
        DirKind::Data => Ok(join(&[home, "Library", "Application Support", app_name])),
        DirKind::State => Ok(join(&[
            home,
            "Library",
            "Application Support",
            app_name,
            "State",
        ])),
        DirKind::Logs => Ok(join(&[home, "Library", "Logs", app_name])),
        DirKind::Temp => Ok(join(&[env.tmpdir.as_deref().unwrap_or("/tmp"), app_name])),
    }
}

fn resolve_windows(app_name: &str, env: &Env, kind: DirKind) -> Result<String, AppDirError> {
    match kind {
        DirKind::Config => Ok(join(&[
            env.app_data.as_deref().ok_or(AppDirError::MissingRequiredEnv)?,
            app_name,
        ])),
        DirKind::Cache => Ok(join(&[
            env.local_app_data
                .as_deref()
                .ok_or(AppDirError::MissingRequiredEnv)?,
            app_name,
            "Cache",
        ])),
        DirKind::Data => Ok(join(&[
            env.local_app_data
                .as_deref()
                .ok_or(AppDirError::MissingRequiredEnv)?,
            app_name,
            "Data",
        ])),
        DirKind::State => Ok(join(&[
            env.local_app_data
                .as_deref()
                .ok_or(AppDirError::MissingRequiredEnv)?,
            app_name,
            "State",
        ])),
        DirKind::Logs => Ok(join(&[
            env.local_app_data
                .as_deref()
                .ok_or(AppDirError::MissingRequiredEnv)?,
            app_name,
            "Logs",
        ])),
        DirKind::Temp => Ok(join(&[
            env.temp
                .as_deref()
                .or(env.tmp.as_deref())
                .ok_or(AppDirError::MissingRequiredEnv)?,
            app_name,
        ])),
    }
}

fn join(parts: &[&str]) -> String {
    parts.join(std::path::MAIN_SEPARATOR.to_string().as_str())
}

pub fn validate_app_name(name: &str) -> Result<(), AppDirError> {
    if name.is_empty() || name == "." || name == ".." {
        return Err(AppDirError::InvalidAppName);
    }
    if name.contains('\0') || name.contains('/') || name.contains('\\') {
        return Err(AppDirError::InvalidAppName);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppDirError {
    MissingHome,
    MissingRequiredEnv,
    InvalidAppName,
    UnsupportedPlatform,
    NoSpaceLeft,
}
