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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_name_validation() {
        assert!(validate_app_name("MyApp").is_ok());
        assert!(validate_app_name("my-app").is_ok());
        assert!(validate_app_name("").is_err());
        assert!(validate_app_name(".").is_err());
        assert!(validate_app_name("..").is_err());
        assert!(validate_app_name("app/name").is_err());
        assert!(validate_app_name("app\\name").is_err());
    }

    #[test]
    fn linux_xdg_paths_use_explicit_env() {
        let env = Env {
            home: Some("/home/user".into()),
            xdg_config_home: Some("/custom/config".into()),
            xdg_cache_home: Some("/custom/cache".into()),
            xdg_data_home: Some("/custom/data".into()),
            xdg_state_home: Some("/custom/state".into()),
            ..Default::default()
        };
        let app = AppInfo { name: "test".into(), organization: None, qualifier: None };
        let config = resolve_one(&app, Platform::Linux, &env, DirKind::Config).unwrap();
        assert!(config.contains("/custom/config/test"));
    }

    #[test]
    fn linux_falls_back_to_home_defaults() {
        let env = Env {
            home: Some("/home/user".into()),
            ..Default::default()
        };
        let app = AppInfo { name: "test".into(), organization: None, qualifier: None };
        let config = resolve_one(&app, Platform::Linux, &env, DirKind::Config).unwrap();
        assert!(config.contains(".config"));
    }

    #[test]
    fn macos_library_paths_resolve_from_home() {
        let env = Env { home: Some("/Users/alice".into()), ..Default::default() };
        let app = AppInfo { name: "test".into(), organization: None, qualifier: None };
        let data = resolve_one(&app, Platform::MacOS, &env, DirKind::Data).unwrap();
        assert!(data.contains("Library/Application Support"));
    }

    #[test]
    fn windows_paths_resolve_from_appdata() {
        let env = Env {
            local_app_data: Some("C:\\Users\\alice\\AppData\\Local".into()),
            app_data: Some("C:\\Users\\alice\\AppData\\Roaming".into()),
            ..Default::default()
        };
        let app = AppInfo { name: "test".into(), organization: None, qualifier: None };
        let config = resolve_one(&app, Platform::Windows, &env, DirKind::Config).unwrap();
        assert!(config.contains("AppData"));
    }

    #[test]
    fn missing_required_env_produces_error() {
        let env = Env::default(); // no home
        let app = AppInfo { name: "test".into(), organization: None, qualifier: None };
        assert!(resolve_one(&app, Platform::Linux, &env, DirKind::Config).is_err());
    }

    #[test]
    fn ios_android_unsupported() {
        let env = Env { home: Some("/home".into()), ..Default::default() };
        let app = AppInfo { name: "test".into(), organization: None, qualifier: None };
        assert!(resolve_one(&app, Platform::IOS, &env, DirKind::Config).is_err());
        assert!(resolve_one(&app, Platform::Android, &env, DirKind::Config).is_err());
    }
}
