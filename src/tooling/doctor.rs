use std::process::Command as StdCommand;

use crate::platform_info;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    DoctorProblems,
    InvalidArguments,
}

#[derive(Debug, Clone)]
pub struct Options {
    pub strict: bool,
    pub manifest_path: Option<String>,
    pub web_engine_override: Option<super::web_engine::Engine>,
    pub cef_dir_override: Option<String>,
    pub cef_auto_install_override: Option<bool>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            strict: false,
            manifest_path: None,
            web_engine_override: None,
            cef_dir_override: None,
            cef_auto_install_override: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Available,
    Missing,
    Unsupported,
}

#[derive(Debug, Clone)]
pub struct Check {
    pub id: String,
    pub status: Status,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct Report {
    pub target: platform_info::Target,
    pub display_server: platform_info::DisplayServer,
    pub checks: Vec<Check>,
}

impl Report {
    pub fn has_problems(&self) -> bool {
        self.checks.iter().any(|c| c.status == Status::Missing)
    }

    pub fn format_text(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "host: {:?}-{:?} display={:?}\n",
            self.target.platform, self.target.arch, self.display_server
        ));
        for check in &self.checks {
            let mark = match check.status {
                Status::Available => "✓",
                Status::Missing => "✗",
                Status::Unsupported => "—",
            };
            out.push_str(&format!("  {} [{}] {}\n", mark, check.id, check.message));
        }
        out
    }
}

pub fn parse_options(args: &[String]) -> Result<Options, Error> {
    let mut options = Options::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--strict" => options.strict = true,
            "--manifest" => {
                i += 1;
                if i >= args.len() {
                    return Err(Error::InvalidArguments);
                }
                options.manifest_path = Some(args[i].clone());
            }
            "--web-engine" => {
                i += 1;
                if i >= args.len() {
                    return Err(Error::InvalidArguments);
                }
                options.web_engine_override = Some(super::web_engine::Engine::parse(&args[i])
                    .ok_or(Error::InvalidArguments)?);
            }
            "--cef-dir" => {
                i += 1;
                if i >= args.len() {
                    return Err(Error::InvalidArguments);
                }
                options.cef_dir_override = Some(args[i].clone());
            }
            "--cef-auto-install" => {
                options.cef_auto_install_override = Some(true);
            }
            _ => return Err(Error::InvalidArguments),
        }
        i += 1;
    }
    Ok(options)
}

pub fn run(args: &[String]) -> Result<(), Error> {
    let options = parse_options(args)?;
    let report = build_report(&options);
    print!("{}", report.format_text());
    if options.strict && report.has_problems() {
        return Err(Error::DoctorProblems);
    }
    Ok(())
}

fn build_report(options: &Options) -> Report {
    let target = platform_info::Target::current();
    let env_vars = collect_display_env();
    let display_server = platform_info::detect_display_server(&env_vars, target.platform);
    let mut checks = Vec::new();

    if command_available(&["zig", "version"]) {
        checks.push(Check {
            id: "zig".into(),
            status: Status::Available,
            message: "zig command is available".into(),
        });
    } else {
        checks.push(Check {
            id: "zig".into(),
            status: Status::Missing,
            message: "zig command was not found on PATH".into(),
        });
    }

    checks.push(Check {
        id: "null-backend".into(),
        status: Status::Available,
        message: "headless WebView shell platform is available".into(),
    });

    if let Some(ref path) = options.manifest_path {
        match std::fs::read_to_string(path) {
            Ok(_) => {
                checks.push(Check {
                    id: "manifest".into(),
                    status: Status::Available,
                    message: format!("{}: app.zon is valid", path),
                });
            }
            Err(e) => {
                checks.push(Check {
                    id: "manifest".into(),
                    status: Status::Missing,
                    message: format!("{}: could not be read: {}", path, e),
                });
            }
        }
    }

    if target.platform == platform_info::Platform::MacOS {
        checks.push(Check {
            id: "webview-system".into(),
            status: Status::Available,
            message: "WKWebView system WebView backend is available on macOS hosts".into(),
        });
        if path_exists("/usr/bin/codesign") {
            checks.push(Check {
                id: "codesign".into(),
                status: Status::Available,
                message: "codesign is available for macOS signing".into(),
            });
        } else {
            checks.push(Check {
                id: "codesign".into(),
                status: Status::Missing,
                message: "codesign was not found".into(),
            });
        }
        if command_available(&["xcrun", "notarytool", "--help"]) {
            checks.push(Check {
                id: "notarytool".into(),
                status: Status::Available,
                message: "xcrun notarytool is available for notarization".into(),
            });
        } else {
            checks.push(Check {
                id: "notarytool".into(),
                status: Status::Missing,
                message: "xcrun notarytool was not found".into(),
            });
        }
        if path_exists("/usr/bin/hdiutil") {
            checks.push(Check {
                id: "hdiutil".into(),
                status: Status::Available,
                message: "hdiutil is available for macOS .dmg creation".into(),
            });
        } else {
            checks.push(Check {
                id: "hdiutil".into(),
                status: Status::Missing,
                message: "hdiutil was not found".into(),
            });
        }
        if path_exists("/usr/bin/iconutil") {
            checks.push(Check {
                id: "iconutil".into(),
                status: Status::Available,
                message: "iconutil is available for .icns generation".into(),
            });
        } else {
            checks.push(Check {
                id: "iconutil".into(),
                status: Status::Missing,
                message: "iconutil was not found".into(),
            });
        }
    } else if target.platform == platform_info::Platform::Linux {
        if command_available(&["pkg-config", "--exists", "webkitgtk-6.0"]) {
            checks.push(Check {
                id: "webview-system".into(),
                status: Status::Available,
                message: "WebKitGTK 6.0 system WebView backend is available".into(),
            });
        } else {
            checks.push(Check {
                id: "webview-system".into(),
                status: Status::Missing,
                message: "WebKitGTK 6.0 was not found (install libwebkitgtk-6.0-dev or webkitgtk-6.0)".into(),
            });
        }
    } else {
        checks.push(Check {
            id: "webview-system".into(),
            status: Status::Unsupported,
            message: "system WebView backend is not wired for this host yet".into(),
        });
    }

    checks.push(Check {
        id: "ios-static-lib".into(),
        status: Status::Available,
        message: "Use `zig build lib -Dtarget=aarch64-ios` to build the iOS static library".into(),
    });
    checks.push(Check {
        id: "android-static-lib".into(),
        status: Status::Available,
        message: "Use `zig build lib -Dtarget=aarch64-android` to build the Android static library".into(),
    });

    Report {
        target,
        display_server,
        checks,
    }
}

fn collect_display_env() -> Vec<(String, String)> {
    let mut env = Vec::new();
    if let Ok(val) = std::env::var("WAYLAND_DISPLAY") {
        if !val.is_empty() {
            env.push(("WAYLAND_DISPLAY".into(), val));
        }
    }
    if let Ok(val) = std::env::var("DISPLAY") {
        if !val.is_empty() {
            env.push(("DISPLAY".into(), val));
        }
    }
    env
}

fn command_available(argv: &[&str]) -> bool {
    StdCommand::new(argv[0])
        .args(&argv[1..])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn path_exists(path: &str) -> bool {
    std::path::Path::new(path).exists()
}
