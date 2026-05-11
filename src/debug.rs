use crate::trace;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceMode {
    Off,
    Events,
    Runtime,
    All,
}

impl TraceMode {
    pub fn includes(self, category: TraceMode) -> bool {
        self == TraceMode::All || self == category
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Text,
    JsonLines,
}

impl LogFormat {
    pub fn parse(value: &str) -> Option<LogFormat> {
        match value {
            "text" => Some(LogFormat::Text),
            "jsonl" | "json_lines" => Some(LogFormat::JsonLines),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub trace: TraceMode,
    pub debug_overlay: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            trace: TraceMode::Events,
            debug_overlay: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogPaths {
    pub log_dir: String,
    pub log_file: String,
    pub panic_file: String,
}

pub fn resolve_log_paths(
    app_name: &str,
    env: &crate::app_dirs::Env,
    override_dir: Option<&str>,
) -> Result<LogPaths, crate::app_dirs::AppDirError> {
    let platform = crate::app_dirs::current_platform();
    let log_dir = match override_dir {
        Some(dir) => dir.to_string(),
        None => crate::app_dirs::resolve_one(
            &crate::app_dirs::AppInfo {
                name: app_name.to_string(),
                organization: None,
                qualifier: None,
            },
            platform,
            env,
            crate::app_dirs::DirKind::Logs,
        )?,
    };
    let log_file = std::path::Path::new(&log_dir)
        .join("zero-native.jsonl")
        .to_string_lossy()
        .to_string();
    let panic_file = std::path::Path::new(&log_dir)
        .join("last-panic.txt")
        .to_string_lossy()
        .to_string();
    Ok(LogPaths {
        log_dir,
        log_file,
        panic_file,
    })
}

pub fn setup_logging(
    env: &std::collections::HashMap<String, String>,
    app_name: &str,
) -> LogSetup {
    let app_dirs_env = env_from_map(env);
    let override_dir = env.get("ZERO_NATIVE_LOG_DIR").map(|s| s.as_str());
    let paths = resolve_log_paths(app_name, &app_dirs_env, override_dir).unwrap_or_else(|_| {
        let fallback = std::env::temp_dir().join(app_name);
        LogPaths {
            log_dir: fallback.to_string_lossy().to_string(),
            log_file: fallback.join("zero-native.jsonl").to_string_lossy().to_string(),
            panic_file: fallback.join("last-panic.txt").to_string_lossy().to_string(),
        }
    });
    let format = env
        .get("ZERO_NATIVE_LOG_FORMAT")
        .and_then(|v| LogFormat::parse(v))
        .unwrap_or(LogFormat::JsonLines);
    LogSetup { paths, format }
}

#[derive(Debug, Clone)]
pub struct LogSetup {
    pub paths: LogPaths,
    pub format: LogFormat,
}

pub fn env_from_map(env: &std::collections::HashMap<String, String>) -> crate::app_dirs::Env {
    crate::app_dirs::Env {
        home: env.get("HOME").cloned(),
        xdg_config_home: env.get("XDG_CONFIG_HOME").cloned(),
        xdg_cache_home: env.get("XDG_CACHE_HOME").cloned(),
        xdg_data_home: env.get("XDG_DATA_HOME").cloned(),
        xdg_state_home: env.get("XDG_STATE_HOME").cloned(),
        local_app_data: env.get("LOCALAPPDATA").cloned(),
        app_data: env.get("APPDATA").cloned(),
        temp: env.get("TEMP").cloned(),
        tmp: env.get("TMP").cloned(),
        tmpdir: env.get("TMPDIR").cloned(),
    }
}

pub fn parse_trace_mode(value: &str) -> Option<TraceMode> {
    match value {
        "off" => Some(TraceMode::Off),
        "events" => Some(TraceMode::Events),
        "runtime" => Some(TraceMode::Runtime),
        "all" => Some(TraceMode::All),
        _ => None,
    }
}

pub fn append_trace_record(
    log_dir: &str,
    path: &str,
    format: LogFormat,
    record: &trace::Record,
) -> std::io::Result<()> {
    let _ = std::fs::create_dir_all(log_dir);
    let line = match format {
        LogFormat::Text => {
            let mut text = trace::format_text(record);
            text.push('\n');
            text
        }
        LogFormat::JsonLines => trace::format_json_line(record),
    };
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    file.write_all(line.as_bytes())?;
    Ok(())
}

pub fn diagnose(_sink: &mut dyn trace::Sink) {
    // Debug diagnostics would walk the runtime state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_mode_parsing_and_matching() {
        assert_eq!(Some(TraceMode::Events), parse_trace_mode("events"));
        assert_eq!(Some(TraceMode::Off), parse_trace_mode("off"));
        assert!(TraceMode::All.includes(TraceMode::Runtime));
        assert!(!TraceMode::Events.includes(TraceMode::Runtime));
        assert!(TraceMode::Events.includes(TraceMode::Events));
        assert!(TraceMode::Off.includes(TraceMode::Off));
    }

    #[test]
    fn log_format_parsing() {
        assert_eq!(Some(LogFormat::Text), LogFormat::parse("text"));
        assert_eq!(Some(LogFormat::JsonLines), LogFormat::parse("jsonl"));
        assert_eq!(Some(LogFormat::JsonLines), LogFormat::parse("json_lines"));
        assert_eq!(None, LogFormat::parse("xml"));
    }

    #[test]
    fn log_path_resolution_uses_override() {
        let env = crate::app_dirs::Env {
            home: Some("/Users/alice".into()),
            tmpdir: Some("/tmp".into()),
            ..Default::default()
        };
        let paths = resolve_log_paths("dev.zero_native.test", &env, Some("/tmp/zero-native-logs")).unwrap();
        assert_eq!("/tmp/zero-native-logs", paths.log_dir);
        assert!(paths.log_file.contains("zero-native.jsonl"));
        assert!(paths.panic_file.contains("last-panic.txt"));
    }

    #[test]
    fn env_from_hashmap() {
        let mut map = std::collections::HashMap::new();
        map.insert("HOME".into(), "/home/user".into());
        map.insert("TMPDIR".into(), "/tmp".into());
        let env = env_from_map(&map);
        assert_eq!(Some("/home/user".to_string()), env.home);
        assert_eq!(Some("/tmp".to_string()), env.tmpdir);
        assert!(env.xdg_config_home.is_none());
    }
}
