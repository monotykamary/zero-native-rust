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
pub const DEFAULT_OFFICIAL_DOWNLOAD_URL: &str = "https://cef-builds.spotifycdn.com";
pub const DEFAULT_DOWNLOAD_URL: &str = DEFAULT_PREPARED_DOWNLOAD_URL;
pub const DEFAULT_MACOS_DIR: &str = "third_party/cef/macos";
pub const DEFAULT_LINUX_DIR: &str = "third_party/cef/linux";
pub const DEFAULT_WINDOWS_DIR: &str = "third_party/cef/windows";
pub const DEFAULT_RELEASE_OUTPUT_DIR: &str = "zig-out/cef";

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
        let os = if cfg!(target_os = "macos") { "macos" }
            else if cfg!(target_os = "linux") { "linux" }
            else if cfg!(target_os = "windows") { "windows" }
            else { return Err(Error::UnsupportedPlatform); };
        let arch = if cfg!(target_arch = "x86_64") { "x86_64" }
            else if cfg!(target_arch = "aarch64") { "aarch64" }
            else { return Err(Error::UnsupportedPlatform); };
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

    pub fn wrapper_library_name(self) -> &'static str {
        match self {
            Platform::Windows64 | Platform::WindowsArm64 => "libcef_dll_wrapper.lib",
            _ => "libcef_dll_wrapper.a",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind { File, Directory }

#[derive(Debug, Clone)]
pub struct RequiredEntry {
    pub path: &'static str,
    pub kind: EntryKind,
}

fn required_entries(platform: Platform) -> Vec<RequiredEntry> {
    match platform {
        Platform::MacOSX64 | Platform::MacOSArm64 => vec![
            RequiredEntry { path: "include/cef_app.h", kind: EntryKind::File },
            RequiredEntry { path: "Release/Chromium Embedded Framework.framework", kind: EntryKind::Directory },
            RequiredEntry { path: "libcef_dll_wrapper/libcef_dll_wrapper.a", kind: EntryKind::File },
        ],
        Platform::Linux64 | Platform::LinuxArm64 => vec![
            RequiredEntry { path: "include/cef_app.h", kind: EntryKind::File },
            RequiredEntry { path: "Release/libcef.so", kind: EntryKind::File },
            RequiredEntry { path: "libcef_dll_wrapper/libcef_dll_wrapper.a", kind: EntryKind::File },
        ],
        Platform::Windows64 | Platform::WindowsArm64 => vec![
            RequiredEntry { path: "include/cef_app.h", kind: EntryKind::File },
            RequiredEntry { path: "Release/libcef.dll", kind: EntryKind::File },
            RequiredEntry { path: "libcef_dll_wrapper/libcef_dll_wrapper.lib", kind: EntryKind::File },
        ],
    }
}

#[derive(Debug, Clone)]
pub struct LayoutReport {
    pub ok: bool,
    pub missing_path: Option<String>,
}

pub fn verify_layout(dir: &str) -> LayoutReport {
    let platform = match Platform::current() {
        Ok(p) => p,
        Err(_) => return LayoutReport { ok: false, missing_path: None },
    };
    verify_layout_for(platform, dir)
}

pub fn verify_layout_for(platform: Platform, dir: &str) -> LayoutReport {
    for entry in required_entries(platform) {
        let path = std::path::Path::new(dir).join(entry.path);
        let exists = match entry.kind {
            EntryKind::File => path.is_file(),
            EntryKind::Directory => path.is_dir(),
        };
        if !exists {
            return LayoutReport { ok: false, missing_path: Some(entry.path.to_string()) };
        }
    }
    LayoutReport { ok: true, missing_path: None }
}

pub fn ensure_layout(dir: &str) -> Result<(), Error> {
    let report = verify_layout(dir);
    if !report.ok { return Err(Error::MissingLayout); }
    Ok(())
}

pub fn ensure_layout_for(platform: Platform, dir: &str) -> Result<(), Error> {
    let report = verify_layout_for(platform, dir);
    if !report.ok { return Err(Error::MissingLayout); }
    Ok(())
}

pub fn missing_message(dir: &str, report: &LayoutReport) -> String {
    if report.ok {
        format!("CEF layout is ready at {}", dir)
    } else {
        format!(
            "CEF layout is missing {} under {}",
            report.missing_path.as_deref().unwrap_or("required files"),
            dir
        )
    }
}

pub fn prepared_archive_name(version: &str, platform: Platform) -> String {
    format!("zero-native-cef-{}-{}.tar.gz", version, platform.name())
}

pub fn archive_name(version: &str, platform: Platform) -> String {
    format!("cef_binary_{}_{}.tar.bz2", version, platform.name())
}

pub fn prepared_archive_url(base_url: &str, version: &str, platform: Platform) -> String {
    let base = trim_trailing_slashes(base_url);
    let name = prepared_archive_name(version, platform);
    format!("{}/cef-{}/{}", base, version, name)
}

pub fn archive_url(base_url: &str, version: &str, platform: Platform) -> String {
    let base = trim_trailing_slashes(base_url);
    let name = archive_name(version, platform);
    format!("{}/{}", base, name)
}

fn trim_trailing_slashes(value: &str) -> &str {
    let end = value.trim_end_matches('/');
    if end.is_empty() { "/" } else { end }
}

pub fn cache_dir() -> String {
    if let Ok(val) = std::env::var("XDG_CACHE_HOME") {
        format!("{}/zero-native/cef", val)
    } else if cfg!(target_os = "windows") {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            return format!("{}/zero-native/cef", local);
        }
        if let Ok(home) = std::env::var("USERPROFILE") {
            return format!("{}/AppData/Local/zero-native/cef", home);
        }
        ".zig-cache/zero-native-cef".into()
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

#[derive(Debug, Clone)]
pub struct InstallOptions {
    pub dir: String,
    pub version: String,
    pub source: Source,
    pub download_url: Option<String>,
    pub force: bool,
    pub allow_build_tools: bool,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            dir: String::new(),
            version: DEFAULT_VERSION.into(),
            source: Source::Prepared,
            download_url: None,
            force: false,
            allow_build_tools: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrepareOptions {
    pub dir: String,
    pub output_dir: String,
    pub version: String,
}

impl Default for PrepareOptions {
    fn default() -> Self {
        Self {
            dir: String::new(),
            output_dir: DEFAULT_RELEASE_OUTPUT_DIR.into(),
            version: DEFAULT_VERSION.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InstallResult {
    pub dir: String,
    pub archive_path: String,
    pub platform: Platform,
    pub installed: bool,
}

pub fn parse_options(args: &[String]) -> Result<InstallOptions, Error> {
    let mut options = InstallOptions::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dir" => { i += 1; if i >= args.len() { return Err(Error::InvalidArguments); } options.dir = args[i].clone(); }
            "--version" => { i += 1; if i >= args.len() { return Err(Error::InvalidArguments); } options.version = args[i].clone(); }
            "--source" => { i += 1; if i >= args.len() { return Err(Error::InvalidArguments); } options.source = Source::parse(&args[i]).ok_or(Error::InvalidArguments)?; }
            "--download-url" => { i += 1; if i >= args.len() { return Err(Error::InvalidArguments); } options.download_url = Some(args[i].clone()); }
            "--force" => { options.force = true; }
            "--allow-build-tools" => { options.allow_build_tools = true; }
            _ => return Err(Error::InvalidArguments),
        }
        i += 1;
    }
    Ok(options)
}

pub fn parse_prepare_options(args: &[String]) -> Result<PrepareOptions, Error> {
    let mut options = PrepareOptions::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dir" => { i += 1; if i >= args.len() { return Err(Error::InvalidArguments); } options.dir = args[i].clone(); }
            "--output" => { i += 1; if i >= args.len() { return Err(Error::InvalidArguments); } options.output_dir = args[i].clone(); }
            "--version" => { i += 1; if i >= args.len() { return Err(Error::InvalidArguments); } options.version = args[i].clone(); }
            _ => return Err(Error::InvalidArguments),
        }
        i += 1;
    }
    Ok(options)
}

fn resolve_dir(dir: &str, platform: Platform) -> String {
    if dir.is_empty() { platform.default_dir().to_string() } else { dir.to_string() }
}

pub fn install(options: &InstallOptions) -> Result<InstallResult, Error> {
    let platform = Platform::current()?;
    let resolved_dir = resolve_dir(&options.dir, platform);
    let existing = verify_layout_for(platform, &resolved_dir);
    if existing.ok && !options.force {
        return Ok(InstallResult { dir: resolved_dir, archive_path: String::new(), platform, installed: false });
    }

    let cache_path = cache_dir();
    let _ = std::fs::create_dir_all(&cache_path);

    let download_url = options.download_url.as_deref().unwrap_or(DEFAULT_DOWNLOAD_URL);

    match options.source {
        Source::Prepared => install_prepared(options, platform, &resolved_dir, &cache_path, download_url, &existing),
        Source::Official => install_official(options, platform, &resolved_dir, &cache_path, download_url, &existing),
    }
}

fn install_prepared(options: &InstallOptions, platform: Platform, resolved_dir: &str, cache_path: &str, base_url: &str, _existing: &LayoutReport) -> Result<InstallResult, Error> {
    let archive_name = prepared_archive_name(&options.version, platform);
    let archive_path = format!("{}/{}", cache_path, archive_name);
    let url = prepared_archive_url(base_url, &options.version, platform);

    if options.force || !std::path::Path::new(&archive_path).exists() {
        run_command(&["curl", "--fail", "--location", "--output", &archive_path, &url])
            .map_err(|_| {
                eprintln!("Prepared CEF runtime is not available at {}", url);
                eprintln!("Maintainers can publish it with the CEF runtime release workflow.");
                eprintln!("Advanced users may run `zero-native cef install --source official --allow-build-tools`.");
                Error::CommandFailed
            })?;
    }

    let tmp_dir = format!("{}/extract-tmp", cache_path);
    let _ = run_command(&["rm", "-rf", &tmp_dir]);
    let layout_dir = format!("{}/layout", tmp_dir);
    let _ = std::fs::create_dir_all(&layout_dir);
    run_command(&["tar", "-xzf", &archive_path, "-C", &layout_dir])
        .map_err(|_| Error::CommandFailed)?;

    ensure_layout_for(platform, &layout_dir)?;

    if std::path::Path::new(resolved_dir).exists() {
        let _ = run_command(&["rm", "-rf", resolved_dir]);
    }
    run_command(&["mv", &layout_dir, resolved_dir])
        .map_err(|_| Error::CommandFailed)?;
    ensure_layout_for(platform, resolved_dir)?;

    Ok(InstallResult { dir: resolved_dir.to_string(), archive_path, platform, installed: true })
}

fn install_official(options: &InstallOptions, platform: Platform, resolved_dir: &str, cache_path: &str, base_url: &str, _existing: &LayoutReport) -> Result<InstallResult, Error> {
    if !options.allow_build_tools {
        eprintln!("Official CEF archives require building libcef_dll_wrapper locally. Use the prepared runtime with `zero-native cef install`, or opt in with `--source official --allow-build-tools`.");
        return Err(Error::WrapperBuildFailed);
    }

    let aname = archive_name(&options.version, platform);
    let archive_path = format!("{}/{}", cache_path, aname);
    let url = archive_url(base_url, &options.version, platform);

    if options.force || !std::path::Path::new(&archive_path).exists() {
        run_command(&["curl", "--fail", "--location", "--output", &archive_path, &url])
            .map_err(|_| Error::CommandFailed)?;
    }

    let tmp_dir = format!("{}/extract-tmp-official", cache_path);
    let _ = run_command(&["rm", "-rf", &tmp_dir]);
    let _ = std::fs::create_dir_all(&tmp_dir);
    run_command(&["tar", "-xjf", &archive_path, "-C", &tmp_dir])
        .map_err(|_| Error::CommandFailed)?;

    let extracted_name = aname.trim_end_matches(".tar.bz2");
    let extracted_root = format!("{}/{}", tmp_dir, extracted_name);
    if !std::path::Path::new(&extracted_root).exists() {
        return Err(Error::CommandFailed);
    }

    if std::path::Path::new(resolved_dir).exists() {
        let _ = run_command(&["rm", "-rf", resolved_dir]);
    }
    run_command(&["mv", &extracted_root, resolved_dir])
        .map_err(|_| Error::CommandFailed)?;
    ensure_layout_for(platform, resolved_dir)?;

    Ok(InstallResult { dir: resolved_dir.to_string(), archive_path, platform, installed: true })
}

pub fn prepare_release(options: &PrepareOptions) -> Result<String, Error> {
    let platform = Platform::current()?;
    let dir = resolve_dir(&options.dir, platform);
    ensure_layout_for(platform, &dir)?;
    let _ = std::fs::create_dir_all(&options.output_dir);

    let name = prepared_archive_name(&options.version, platform);
    let archive_path = format!("{}/{}", options.output_dir, name);

    let cmd = format!(
        "output_dir=$(cd {} && pwd) && cd {} && tar -czf \"$output_dir\"/{} include Release libcef_dll_wrapper $(test -d Resources && echo Resources) $(test -d locales && echo locales)",
        shell_quote(&options.output_dir),
        shell_quote(&dir),
        shell_quote(&name),
    );
    run_command(&["sh", "-c", &cmd]).map_err(|_| Error::CommandFailed)?;

    let sha_cmd = format!(
        "cd {} && shasum -a 256 {} | awk '{{print $1}}' > {}.sha256",
        shell_quote(&options.output_dir),
        shell_quote(&name),
        shell_quote(&name),
    );
    let _ = run_command(&["sh", "-c", &sha_cmd]);

    Ok(archive_path)
}

pub fn run(args: &[String]) -> Result<(), Error> {
    if args.is_empty() {
        print_cef_usage();
        return Err(Error::InvalidArguments);
    }
    match args[0].as_str() {
        "install" => {
            let options = parse_options(&args[1..])?;
            let result = install(&options)?;
            if result.installed {
                println!("CEF installed at {}", result.dir);
            } else {
                println!("CEF already installed at {}", result.dir);
            }
            Ok(())
        }
        "path" => {
            let options = parse_options(&args[1..])?;
            let platform = Platform::current()?;
            println!("{}", resolve_dir(&options.dir, platform));
            Ok(())
        }
        "doctor" => {
            let options = parse_options(&args[1..])?;
            let platform = Platform::current()?;
            let dir = resolve_dir(&options.dir, platform);
            let report = verify_layout_for(platform, &dir);
            println!("{}", missing_message(&dir, &report));
            if !report.ok { return Err(Error::MissingLayout); }
            Ok(())
        }
        "prepare-release" => {
            let options = parse_prepare_options(&args[1..])?;
            let path = prepare_release(&options)?;
            println!("prepared CEF runtime at {}", path);
            Ok(())
        }
        _ => {
            print_cef_usage();
            Err(Error::InvalidArguments)
        }
    }
}

fn run_command(argv: &[&str]) -> Result<(), String> {
    let status = StdCommand::new(argv[0])
        .args(&argv[1..])
        .status()
        .map_err(|e| format!("{}: {}", argv[0], e))?;
    if status.success() { Ok(()) } else { Err(format!("{} failed", argv[0])) }
}

fn shell_quote(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('\'');
    for ch in value.chars() {
        if ch == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

fn print_cef_usage() {
    eprintln!(
        "usage: zero-native cef <command>

commands:
  install [--dir path] [--version version] [--source prepared|official] [--download-url url] [--allow-build-tools] [--force]
  path [--dir path]
  doctor [--dir path]
  prepare-release [--dir path] [--output path] [--version version]"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_names_follow_convention() {
        assert_eq!(
            "zero-native-cef-1.2.3+gabc+chromium-4.5.6-macosarm64.tar.gz",
            prepared_archive_name("1.2.3+gabc+chromium-4.5.6", Platform::MacOSArm64)
        );
        assert_eq!(
            "cef_binary_1.2.3+gabc+chromium-4.5.6_macosarm64.tar.bz2",
            archive_name("1.2.3+gabc+chromium-4.5.6", Platform::MacOSArm64)
        );
        assert_eq!(
            "cef_binary_1.2.3+gabc+chromium-4.5.6_linux64.tar.bz2",
            archive_name("1.2.3+gabc+chromium-4.5.6", Platform::Linux64)
        );
        assert_eq!(
            "cef_binary_1.2.3+gabc+chromium-4.5.6_windows64.tar.bz2",
            archive_name("1.2.3+gabc+chromium-4.5.6", Platform::Windows64)
        );
    }

    #[test]
    fn archive_urls_trim_trailing_slash() {
        let url = prepared_archive_url("https://example.com/releases/", "1.2.3", Platform::MacOSArm64);
        assert_eq!(
            "https://example.com/releases/cef-1.2.3/zero-native-cef-1.2.3-macosarm64.tar.gz",
            url
        );
        let url2 = archive_url("https://example.com/", "1.2.3", Platform::MacOSX64);
        assert_eq!(
            "https://example.com/cef_binary_1.2.3_macosx64.tar.bz2",
            url2
        );
    }

    #[test]
    fn test_parse_install_options() {
        let args: Vec<String> = ["--dir", "vendor/cef", "--version", "1.2.3", "--source", "official", "--download-url", "https://example.com", "--allow-build-tools", "--force"].iter().map(|s| s.to_string()).collect();
        let options = parse_options(&args).unwrap();
        assert_eq!("vendor/cef", options.dir);
        assert_eq!("1.2.3", options.version);
        assert_eq!(Source::Official, options.source);
        assert_eq!(Some("https://example.com".to_string()), options.download_url);
        assert!(options.allow_build_tools);
        assert!(options.force);
    }

    #[test]
    fn test_parse_prepare_options() {
        let args: Vec<String> = ["--dir", "vendor/cef", "--output", "zig-out/cef", "--version", "1.2.3"].iter().map(|s| s.to_string()).collect();
        let options = parse_prepare_options(&args).unwrap();
        assert_eq!("vendor/cef", options.dir);
        assert_eq!("zig-out/cef", options.output_dir);
        assert_eq!("1.2.3", options.version);
    }

    #[test]
    fn layout_verifier_reports_missing() {
        let report = verify_layout(".zig-cache/does-not-exist-cef");
        assert!(!report.ok);
        assert!(report.missing_path.is_some());
    }

    #[test]
    fn missing_message_formats() {
        let ok_report = LayoutReport { ok: true, missing_path: None };
        assert_eq!("CEF layout is ready at /tmp/cef", missing_message("/tmp/cef", &ok_report));
        let bad_report = LayoutReport { ok: false, missing_path: Some("include/cef_app.h".into()) };
        assert!(missing_message("/tmp/cef", &bad_report).contains("include/cef_app.h"));
    }

    #[test]
    fn platform_current_and_names() {
        let p = Platform::current().unwrap();
        assert!(!p.name().is_empty());
        assert!(!p.default_dir().is_empty());
    }

    #[test]
    fn source_parse() {
        assert_eq!(Some(Source::Prepared), Source::parse("prepared"));
        assert_eq!(Some(Source::Official), Source::parse("official"));
        assert_eq!(None, Source::parse("unknown"));
    }

    #[test]
    fn cache_dir_is_nonempty() {
        let dir = cache_dir();
        assert!(!dir.is_empty());
    }

    #[test]
    fn shell_quote_escapes_single_quotes() {
        assert_eq!("'hello'", shell_quote("hello"));
        assert_eq!("'it'\\''s'", shell_quote("it's"));
    }

    #[test]
    fn wrapper_library_name_varies_by_platform() {
        assert_eq!("libcef_dll_wrapper.lib", Platform::Windows64.wrapper_library_name());
        assert_eq!("libcef_dll_wrapper.a", Platform::MacOSArm64.wrapper_library_name());
    }
}
