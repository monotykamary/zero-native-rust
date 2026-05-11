use std::process::Command as StdCommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidArguments,
    UnsupportedPlatform,
    MissingLayout,
    CommandFailed,
    WrapperBuildFailed,
}

pub const DEFAULT_VERSION: &str = "144.0.6+g5f7e671+chromium-144.0.7559.59";
pub const DEFAULT_PREPARED_DOWNLOAD_URL: &str =
    "https://github.com/vercel-labs/zero-native/releases/download";
pub const DEFAULT_MACOS_DIR: &str = "third_party/cef/macos";
pub const DEFAULT_LINUX_DIR: &str = "third_party/cef/linux";
pub const DEFAULT_WINDOWS_DIR: &str = "third_party/cef/windows";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    MacOSX64,
    MacOSArm64,
    Linux64,
    LinuxArm64,
    Windows64,
    WindowsArm64,
}

impl Platform {
    pub fn current() -> Result<Platform, Error> {
        let os = if cfg!(target_os = "macos") {
            "macos"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "windows") {
            "windows"
        } else {
            return Err(Error::UnsupportedPlatform);
        };
        let arch = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            return Err(Error::UnsupportedPlatform);
        };
        Ok(match (os, arch) {
            ("macos", "x86_64") => Platform::MacOSX64,
            ("macos", "aarch64") => Platform::MacOSArm64,
            ("linux", "x86_64") => Platform::Linux64,
            ("linux", "aarch64") => Platform::LinuxArm64,
            ("windows", "x86_64") => Platform::Windows64,
            ("windows", "aarch64") => Platform::WindowsArm64,
            _ => return Err(Error::UnsupportedPlatform),
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Platform::MacOSX64 => "macosx64",
            Platform::MacOSArm64 => "macosarm64",
            Platform::Linux64 => "linux64",
            Platform::LinuxArm64 => "linuxarm64",
            Platform::Windows64 => "windows64",
            Platform::WindowsArm64 => "windowsarm64",
        }
    }

    pub fn default_dir(self) -> &'static str {
        match self {
            Platform::MacOSX64 | Platform::MacOSArm64 => DEFAULT_MACOS_DIR,
            Platform::Linux64 | Platform::LinuxArm64 => DEFAULT_LINUX_DIR,
            Platform::Windows64 | Platform::WindowsArm64 => DEFAULT_WINDOWS_DIR,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Prepared,
    Official,
}

impl Source {
    pub fn parse(value: &str) -> Option<Source> {
        match value {
            "prepared" => Some(Source::Prepared),
            "official" => Some(Source::Official),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InstallOptions {
    pub dir: String,
    pub version: String,
    pub source: Source,
    pub force: bool,
    pub allow_build_tools: bool,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            dir: String::new(),
            version: DEFAULT_VERSION.into(),
            source: Source::Prepared,
            force: false,
            allow_build_tools: false,
        }
    }
}

struct RequiredEntry {
    path: &'static str,
    is_dir: bool,
}

fn required_entries(platform: Platform) -> Vec<RequiredEntry> {
    match platform {
        Platform::MacOSX64 | Platform::MacOSArm64 => vec![
            RequiredEntry { path: "include/cef_app.h", is_dir: false },
            RequiredEntry { path: "Release/Chromium Embedded Framework.framework", is_dir: true },
            RequiredEntry { path: "libcef_dll_wrapper/libcef_dll_wrapper.a", is_dir: false },
        ],
        Platform::Linux64 | Platform::LinuxArm64 => vec![
            RequiredEntry { path: "include/cef_app.h", is_dir: false },
            RequiredEntry { path: "Release/libcef.so", is_dir: false },
            RequiredEntry { path: "libcef_dll_wrapper/libcef_dll_wrapper.a", is_dir: false },
        ],
        Platform::Windows64 | Platform::WindowsArm64 => vec![
            RequiredEntry { path: "include/cef_app.h", is_dir: false },
            RequiredEntry { path: "Release/libcef.dll", is_dir: false },
            RequiredEntry { path: "libcef_dll_wrapper/libcef_dll_wrapper.lib", is_dir: false },
        ],
    }
}

pub fn verify_layout(dir: &str) -> bool {
    let platform = match Platform::current() {
        Ok(p) => p,
        Err(_) => return false,
    };
    verify_layout_for(platform, dir)
}

pub fn verify_layout_for(platform: Platform, dir: &str) -> bool {
    for entry in required_entries(platform) {
        let path = std::path::Path::new(dir).join(entry.path);
        let exists = if entry.is_dir { path.is_dir() } else { path.is_file() };
        if !exists {
            return false;
        }
    }
    true
}

pub fn ensure_layout(dir: &str) -> Result<(), Error> {
    if !verify_layout(dir) {
        return Err(Error::MissingLayout);
    }
    Ok(())
}

pub fn parse_options(args: &[String]) -> Result<InstallOptions, Error> {
    let mut options = InstallOptions::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dir" => {
                i += 1;
                if i >= args.len() {
                    return Err(Error::InvalidArguments);
                }
                options.dir = args[i].clone();
            }
            "--version" => {
                i += 1;
                if i >= args.len() {
                    return Err(Error::InvalidArguments);
                }
                options.version = args[i].clone();
            }
            "--source" => {
                i += 1;
                if i >= args.len() {
                    return Err(Error::InvalidArguments);
                }
                options.source = Source::parse(&args[i]).ok_or(Error::InvalidArguments)?;
            }
            "--force" => {
                options.force = true;
            }
            "--allow-build-tools" => {
                options.allow_build_tools = true;
            }
            _ => return Err(Error::InvalidArguments),
        }
        i += 1;
    }
    Ok(options)
}

pub fn run(args: &[String]) -> Result<(), Error> {
    if args.is_empty() {
        print_cef_usage();
        return Err(Error::InvalidArguments);
    }
    match args[0].as_str() {
        "install" => {
            let options = parse_options(&args[1..])?;
            let platform = Platform::current()?;
            let dir = if options.dir.is_empty() {
                platform.default_dir().to_string()
            } else {
                options.dir.clone()
            };

            if verify_layout_for(platform, &dir) && !options.force {
                println!("CEF already installed at {}", dir);
                return Ok(());
            }

            run_command(&[
                "curl",
                "--fail",
                "--location",
                "--output",
                &format!("{}/cef-archive.tar.gz", cache_dir()),
                &format!(
                    "{}/cef-{}/zero-native-cef-{}-{}.tar.gz",
                    DEFAULT_PREPARED_DOWNLOAD_URL,
                    options.version,
                    options.version,
                    platform.name()
                ),
            ]).map_err(|_| Error::CommandFailed)?;

            let _ = std::fs::create_dir_all(&dir);
            run_command(&["tar", "-xzf", &format!("{}/cef-archive.tar.gz", cache_dir()), "-C", &dir])
                .map_err(|_| Error::CommandFailed)?;

            if verify_layout_for(platform, &dir) {
                println!("CEF installed at {}", dir);
                Ok(())
            } else {
                Err(Error::MissingLayout)
            }
        }
        "path" => {
            let options = parse_options(&args[1..])?;
            let platform = Platform::current()?;
            let dir = if options.dir.is_empty() {
                platform.default_dir().to_string()
            } else {
                options.dir
            };
            println!("{}", dir);
            Ok(())
        }
        "doctor" => {
            let options = parse_options(&args[1..])?;
            let platform = Platform::current()?;
            let dir = if options.dir.is_empty() {
                platform.default_dir().to_string()
            } else {
                options.dir
            };
            if verify_layout_for(platform, &dir) {
                println!("CEF layout is ready at {}", dir);
            } else {
                println!("CEF layout is missing required files under {}", dir);
                return Err(Error::MissingLayout);
            }
            Ok(())
        }
        _ => {
            print_cef_usage();
            Err(Error::InvalidArguments)
        }
    }
}

fn cache_dir() -> String {
    if let Ok(val) = std::env::var("XDG_CACHE_HOME") {
        format!("{}/zero-native/cef", val)
    } else if let Ok(home) = std::env::var("HOME") {
        if cfg!(target_os = "macos") {
            format!("{}/Library/Caches/zero-native/cef", home)
        } else {
            format!("{}/.cache/zero-native/cef", home)
        }
    } else {
        ".zig-cache/zero-native-cef".into()
    }
}

fn run_command(argv: &[&str]) -> Result<(), String> {
    let status = StdCommand::new(argv[0])
        .args(&argv[1..])
        .status()
        .map_err(|e| format!("{}: {}", argv[0], e))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{} failed", argv[0]))
    }
}

fn print_cef_usage() {
    eprintln!(
        "usage: zero-native cef <command>

commands:
  install [--dir path] [--version version] [--source prepared|official] [--allow-build-tools] [--force]
  path [--dir path]
  doctor [--dir path]"
    );
}
