use std::collections::HashSet;
use std::fs;

use crate::app_manifest;
use crate::diagnostics;

pub const DEFAULT_WEB_ENGINE: &str = "system";
pub const DEFAULT_CEF_DIR: &str = "third_party/cef/macos";

#[derive(Debug, Clone)]
pub struct Metadata {
    pub id: String,
    pub name: String,
    pub display_name: Option<String>,
    pub version: String,
    pub icons: Vec<String>,
    pub platforms: Vec<String>,
    pub permissions: Vec<String>,
    pub capabilities: Vec<String>,
    pub bridge_commands: Vec<BridgeCommandMetadata>,
    pub web_engine: String,
    pub cef: CefConfig,
    pub frontend: Option<FrontendMetadata>,
    pub security: SecurityMetadata,
    pub windows: Vec<WindowMetadata>,
}

impl Metadata {
    pub fn display_name_or_name(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.name)
    }
}

#[derive(Debug, Clone)]
pub struct BridgeCommandMetadata {
    pub name: String,
    pub permissions: Vec<String>,
    pub origins: Vec<String>,
}

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
pub struct WindowMetadata {
    pub label: String,
    pub title: Option<String>,
    pub width: f32,
    pub height: f32,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub restore_state: bool,
}

impl Default for WindowMetadata {
    fn default() -> Self {
        Self {
            label: "main".into(),
            title: None,
            width: 720.0,
            height: 480.0,
            x: None,
            y: None,
            restore_state: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrontendDevMetadata {
    pub url: String,
    pub command: Vec<String>,
    pub ready_path: String,
    pub timeout_ms: u32,
}

#[derive(Debug, Clone)]
pub struct FrontendMetadata {
    pub dist: String,
    pub entry: String,
    pub spa_fallback: bool,
    pub dev: Option<FrontendDevMetadata>,
}

#[derive(Debug, Clone)]
pub struct ExternalLinkMetadata {
    pub action: String,
    pub allowed_urls: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NavigationMetadata {
    pub allowed_origins: Vec<String>,
    pub external_links: ExternalLinkMetadata,
}

#[derive(Debug, Clone)]
pub struct SecurityMetadata {
    pub navigation: NavigationMetadata,
}

impl Default for SecurityMetadata {
    fn default() -> Self {
        Self {
            navigation: NavigationMetadata {
                allowed_origins: vec![],
                external_links: ExternalLinkMetadata {
                    action: "deny".into(),
                    allowed_urls: vec![],
                },
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub ok: bool,
    pub message: String,
}

pub fn validate_file(path: &str) -> ValidationResult {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => {
            return ValidationResult {
                ok: false,
                message: format!("{}: could not be read", path),
            }
        }
    };

    let metadata = match parse_text(&source) {
        Ok(m) => m,
        Err(_) => {
            return ValidationResult {
                ok: false,
                message: "app.zon metadata could not be parsed".into(),
            }
        }
    };

    for icon in &metadata.icons {
        if let Err(_) = validate_relative_path(icon) {
            return ValidationResult {
                ok: false,
                message: "app.zon icons are invalid".into(),
            };
        }
    }

    let mut seen_icons = HashSet::new();
    for icon in &metadata.icons {
        if !seen_icons.insert(icon.as_str()) {
            return ValidationResult {
                ok: false,
                message: "app.zon icons are invalid".into(),
            };
        }
    }

    if let Err(_) = parse_version(&metadata.version) {
        return ValidationResult {
            ok: false,
            message: "app.zon version is invalid".into(),
        };
    }

    match metadata.web_engine.as_str() {
        "system" | "chromium" => {}
        _ => {
            return ValidationResult {
                ok: false,
                message: "app.zon web engine is invalid".into(),
            }
        }
    }

    ValidationResult {
        ok: true,
        message: "app.zon is valid".into(),
    }
}

pub fn read_metadata(path: &str) -> Result<Metadata, String> {
    let source = fs::read_to_string(path).map_err(|e| format!("{}: {}", path, e))?;
    parse_text(&source)
}

pub fn parse_text(source: &str) -> Result<Metadata, String> {
    let id = extract_string_field(source, "id").ok_or("missing .id")?;
    let name = extract_string_field(source, "name").ok_or("missing .name")?;
    let version = extract_string_field(source, "version").ok_or("missing .version")?;
    let display_name = extract_string_field(source, "display_name");
    let web_engine = extract_string_field(source, "web_engine").unwrap_or(DEFAULT_WEB_ENGINE.to_string());
    let icons = extract_string_list(source, "icons");
    let platforms = extract_string_list(source, "platforms");
    let permissions = extract_string_list(source, "permissions");
    let capabilities = extract_string_list(source, "capabilities");

    let cef_section = extract_object_section(source, "cef");
    let cef = CefConfig {
        dir: cef_section
            .as_ref()
            .and_then(|s| extract_string_field(s, "dir"))
            .unwrap_or_else(|| DEFAULT_CEF_DIR.into()),
        auto_install: cef_section
            .as_ref()
            .and_then(|s| crate::json::bool_field(s, "auto_install"))
            .unwrap_or(false),
    };

    let frontend = extract_object_section(source, "frontend").map(|s| parse_frontend(&s)).transpose()?;

    let security_section = extract_object_section(source, "security");
    let security = security_section
        .as_ref()
        .map(|s| parse_security(s))
        .unwrap_or_default();

    let windows = parse_windows(source);

    let bridge_section = extract_object_section(source, "bridge");
    let bridge_commands = bridge_section
        .as_ref()
        .map(|s| parse_bridge_commands(s))
        .unwrap_or_default();

    Ok(Metadata {
        id,
        name,
        display_name,
        version,
        icons,
        platforms,
        permissions,
        capabilities,
        bridge_commands,
        web_engine,
        cef,
        frontend,
        security,
        windows,
    })
}

fn parse_frontend(source: &str) -> Result<FrontendMetadata, String> {
    let dist = extract_string_field(source, "dist").unwrap_or_else(|| "dist".into());
    let entry = extract_string_field(source, "entry").unwrap_or_else(|| "index.html".into());
    let spa_fallback = crate::json::bool_field(source, "spa_fallback").unwrap_or(true);

    let dev = extract_object_section(source, "dev").map(|s| {
        let url = extract_string_field(&s, "url").unwrap_or_else(|| "http://127.0.0.1:5173/".into());
        let command = extract_string_list(&s, "command");
        let ready_path = extract_string_field(&s, "ready_path").unwrap_or_else(|| "/".into());
        let timeout_ms = crate::json::unsigned_field::<u32>(&s, "timeout_ms").unwrap_or(30_000);
        FrontendDevMetadata {
            url,
            command,
            ready_path,
            timeout_ms,
        }
    });

    Ok(FrontendMetadata {
        dist,
        entry,
        spa_fallback,
        dev,
    })
}

fn parse_security(source: &str) -> SecurityMetadata {
    let nav_section = extract_object_section(source, "navigation");
    let (allowed_origins, external_links) = nav_section
        .as_ref()
        .map(|s| {
            let allowed_origins = extract_string_list(s, "allowed_origins");
            let ext_section = extract_object_section(s, "external_links");
            let external_links = ext_section
                .as_ref()
                .map(|es| {
                    let action = extract_string_field(es, "action").unwrap_or_else(|| "deny".into());
                    let allowed_urls = extract_string_list(es, "allowed_urls");
                    ExternalLinkMetadata { action, allowed_urls }
                })
                .unwrap_or_else(|| ExternalLinkMetadata {
                    action: "deny".into(),
                    allowed_urls: vec![],
                });
            (allowed_origins, external_links)
        })
        .unwrap_or((vec![], ExternalLinkMetadata {
            action: "deny".into(),
            allowed_urls: vec![],
        }));

    SecurityMetadata {
        navigation: NavigationMetadata {
            allowed_origins,
            external_links,
        },
    }
}

fn parse_windows(source: &str) -> Vec<WindowMetadata> {
    let windows_section = extract_object_section(source, "windows");
    match windows_section {
        Some(section) => {
            let mut windows = Vec::new();
            let entries = extract_object_entries(&section);
            for entry in entries {
                let label = extract_string_field(&entry, "label").unwrap_or_else(|| "main".into());
                let title = extract_string_field(&entry, "title");
                let width = crate::json::number_field(&entry, "width").unwrap_or(720.0);
                let height = crate::json::number_field(&entry, "height").unwrap_or(480.0);
                let x = crate::json::number_field(&entry, "x");
                let y = crate::json::number_field(&entry, "y");
                let restore_state = crate::json::bool_field(&entry, "restore_state").unwrap_or(true);
                windows.push(WindowMetadata {
                    label,
                    title,
                    width,
                    height,
                    x,
                    y,
                    restore_state,
                });
            }
            windows
        }
        None => vec![],
    }
}

fn parse_bridge_commands(source: &str) -> Vec<BridgeCommandMetadata> {
    let items = extract_string_list(source, "commands");
    items
        .iter()
        .map(|name| BridgeCommandMetadata {
            name: name.clone(),
            permissions: vec![],
            origins: vec![],
        })
        .collect()
}

pub fn parse_version(value: &str) -> Result<app_manifest::Version, String> {
    let parts: Vec<&str> = value.split('.').collect();
    if parts.len() != 3 {
        return Err("invalid version".into());
    }
    let major = parts[0].parse::<u32>().map_err(|_| "invalid major")?;
    let minor = parts[1].parse::<u32>().map_err(|_| "invalid minor")?;
    let patch = parts[2].parse::<u32>().map_err(|_| "invalid patch")?;
    Ok(app_manifest::Version {
        major,
        minor,
        patch,
        pre: None,
        build: None,
    })
}

pub fn print_diagnostic(result: &ValidationResult) {
    let severity = if result.ok {
        diagnostics::Severity::Info
    } else {
        diagnostics::Severity::Error
    };
    let code = diagnostics::DiagnosticCode {
        namespace: "manifest".into(),
        value: if result.ok { "valid" } else { "invalid" }.into(),
    };
    let diagnostic = diagnostics::Diagnostic {
        severity,
        code,
        message: result.message.clone(),
        labels: vec![],
        notes: vec![],
    };
    println!("{}", diagnostics::format_short(&diagnostic));
}

fn validate_relative_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("empty path".into());
    }
    if path.starts_with('/') || path.starts_with('\\') {
        return Err("absolute path".into());
    }
    if path.len() >= 3 && path.as_bytes()[0].is_ascii_alphabetic() && path.as_bytes()[1] == b':'
        && (path.as_bytes()[2] == b'/' || path.as_bytes()[2] == b'\\')
    {
        return Err("absolute windows path".into());
    }
    for segment in path.split(&['/', '\\']) {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err("invalid path segment".into());
        }
    }
    Ok(())
}

pub fn extract_string_field(source: &str, field: &str) -> Option<String> {
    let search = format!(".{} =", field);
    let pos = source.find(&search)?;
    let after = &source[pos + search.len()..];
    let trimmed = after.trim_start();
    let start = trimmed.find('"')?;
    let rest = &trimmed[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_string_list(source: &str, field: &str) -> Vec<String> {
    let search = format!(".{} = .{{", field);
    let pos = match source.find(&search) {
        Some(p) => p,
        None => return vec![],
    };
    let after = &source[pos + search.len()..];
    let end = matching_brace(after).unwrap_or(after.len());
    let inner = &after[..end];
    let mut items = Vec::new();
    let mut i = 0;
    while i < inner.len() {
        if inner.as_bytes()[i] == b'"' {
            let rest = &inner[i + 1..];
            if let Some(end) = rest.find('"') {
                items.push(rest[..end].to_string());
                i += end + 2;
                continue;
            }
        }
        i += 1;
    }
    items
}

fn extract_object_section(source: &str, field: &str) -> Option<String> {
    let search = format!(".{} = .{{", field);
    let pos = source.find(&search)?;
    let after = &source[pos + search.len()..];
    let end = matching_brace(after)?;
    Some(after[..end].to_string())
}

fn extract_object_entries(source: &str) -> Vec<String> {
    let mut entries = Vec::new();
    let mut depth = 0;
    let mut start = None;
    for (i, ch) in source.char_indices() {
        if ch == '{' {
            if depth == 0 {
                start = Some(i + 1);
            }
            depth += 1;
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                if let Some(s) = start {
                    entries.push(source[s..i].to_string());
                }
                start = None;
            }
        }
    }
    entries
}

fn matching_brace(source: &str) -> Option<usize> {
    let mut depth = 1;
    for (i, ch) in source.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}
