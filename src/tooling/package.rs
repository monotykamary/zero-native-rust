use std::fs;
use std::path::Path;
use std::process::Command as StdCommand;

use super::manifest;
use super::bundle_assets;
use super::cef;
use super::codesign;
use super::web_engine;
use crate::diagnostics;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageTarget {
    MacOS,
    Windows,
    Linux,
    IOS,
    Android,
}

impl PackageTarget {
    pub fn parse(value: &str) -> Option<PackageTarget> {
        match value {
            "macos" => Some(PackageTarget::MacOS),
            "windows" => Some(PackageTarget::Windows),
            "linux" => Some(PackageTarget::Linux),
            "ios" => Some(PackageTarget::IOS),
            "android" => Some(PackageTarget::Android),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PackageOptions {
    pub metadata: manifest::Metadata,
    pub target: PackageTarget,
    pub optimize: String,
    pub output_path: String,
    pub binary_path: Option<String>,
    pub assets_dir: String,
    pub web_engine: web_engine::Engine,
    pub cef_dir: String,
    pub signing: codesign::SigningConfig,
    pub archive: bool,
}

#[derive(Debug, Clone)]
pub struct PackageStats {
    pub path: String,
    pub target: PackageTarget,
    pub signing_mode: codesign::SigningMode,
    pub asset_count: usize,
    pub web_engine: web_engine::Engine,
    pub archive_path: Option<String>,
}

pub fn create_package(options: &PackageOptions) -> Result<PackageStats, String> {
    let mut stats = match options.target {
        PackageTarget::MacOS => create_macos_app(options)?,
        PackageTarget::Windows | PackageTarget::Linux => create_desktop_artifact(options)?,
        PackageTarget::IOS => create_ios_artifact(options)?,
        PackageTarget::Android => create_android_artifact(options)?,
    };

    if options.archive {
        if let Some(archive_path) = create_archive(options)? {
            stats.archive_path = Some(archive_path);
        }
    }

    Ok(stats)
}

pub fn print_diagnostic(stats: &PackageStats) {
    let diagnostic = diagnostics::Diagnostic {
        severity: diagnostics::Severity::Info,
        code: diagnostics::DiagnosticCode {
            namespace: "package".into(),
            value: "created".into(),
        },
        message: format!("created {:?} artifact at {}", stats.target, stats.path),
        labels: vec![],
        notes: vec![],
    };
    println!("{}", diagnostics::format_short(&diagnostic));
    if let Some(ref archive) = stats.archive_path {
        println!("  archive: {}", archive);
    }
}

fn create_macos_app(options: &PackageOptions) -> Result<PackageStats, String> {
    let output = Path::new(&options.output_path);
    fs::create_dir_all(output.join("Contents/MacOS")).map_err(|e| e.to_string())?;
    fs::create_dir_all(output.join("Contents/Resources")).map_err(|e| e.to_string())?;

    let executable_name = Path::new(&options.metadata.name)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    if let Some(ref binary_path) = options.binary_path {
        let bytes = fs::read(binary_path).map_err(|e| e.to_string())?;
        fs::write(output.join(format!("Contents/MacOS/{}", executable_name)), bytes)
            .map_err(|e| e.to_string())?;
    } else {
        fs::write(
            output.join("Contents/MacOS/README.txt"),
            "No app binary was supplied for this local package.\n",
        )
        .map_err(|e| e.to_string())?;
    }

    let info_plist = macos_info_plist(&options.metadata, &executable_name)?;
    fs::write(output.join("Contents/Info.plist"), &info_plist)
        .map_err(|e| e.to_string())?;
    fs::write(output.join("Contents/PkgInfo"), "APPL????")
        .map_err(|e| e.to_string())?;

    let assets_output = output
        .join("Contents/Resources")
        .join(options.metadata.frontend.as_ref().map(|f| f.dist.as_str()).unwrap_or("assets"));
    let assets_output_str = assets_output.to_string_lossy().to_string();
    let bundle_stats = bundle_assets::bundle(&options.assets_dir, &assets_output_str)?;

    if let Some(ref icon_path) = options.metadata.icons.first() {
        let dest = format!("Contents/Resources/{}", Path::new(icon_path).file_name().unwrap_or_default().to_string_lossy());
        match fs::read(icon_path) {
            Ok(bytes) => {
                let _ = fs::write(output.join(&dest), bytes);
            }
            Err(_) => {
                let _ = fs::write(
                    output.join(&dest),
                    "placeholder: configured app icon was not found\n",
                );
            }
        }
    } else {
        let _ = fs::write(
            output.join("Contents/Resources/AppIcon.icns"),
            "placeholder: replace with a real macOS .icns before distributing\n",
        );
    }

    if options.web_engine == web_engine::Engine::Chromium {
        cef::ensure_layout(&options.cef_dir).map_err(|_| "CEF layout is missing".to_string())?;
        copy_macos_cef_runtime(output, &options.cef_dir)?;
    }

    run_signing(output, options)?;

    Ok(PackageStats {
        path: options.output_path.clone(),
        target: PackageTarget::MacOS,
        signing_mode: options.signing.mode,
        asset_count: bundle_stats.asset_count,
        web_engine: options.web_engine,
        archive_path: None,
    })
}

fn create_desktop_artifact(options: &PackageOptions) -> Result<PackageStats, String> {
    let output = Path::new(&options.output_path);
    fs::create_dir_all(output.join("bin")).map_err(|e| e.to_string())?;
    fs::create_dir_all(output.join("resources")).map_err(|e| e.to_string())?;

    let executable_name = if options.target == PackageTarget::Windows {
        format!("{}.exe", options.metadata.name)
    } else {
        options.metadata.name.clone()
    };

    if let Some(ref binary_path) = options.binary_path {
        let bytes = fs::read(binary_path).map_err(|e| e.to_string())?;
        fs::write(output.join(format!("bin/{}", executable_name)), bytes)
            .map_err(|e| e.to_string())?;
    } else {
        fs::write(
            output.join("bin/README.txt"),
            "Build the app binary separately and place it here for this target.\n",
        )
        .map_err(|e| e.to_string())?;
    }

    let assets_output = output
        .join("resources")
        .join(options.metadata.frontend.as_ref().map(|f| f.dist.as_str()).unwrap_or("assets"));
    let assets_output_str = assets_output.to_string_lossy().to_string();
    let bundle_stats = bundle_assets::bundle(&options.assets_dir, &assets_output_str)?;

    let readme_text = match options.target {
        PackageTarget::Windows => "Windows zero-native artifact directory.\n",
        PackageTarget::Linux => "Linux zero-native artifact directory.\n",
        _ => "zero-native artifact directory.\n",
    };
    fs::write(output.join("README.txt"), readme_text).map_err(|e| e.to_string())?;

    if options.target == PackageTarget::Linux {
        let entry = format!(
            "[Desktop Entry]\nType=Application\nName={}\nExec={}\nIcon=app-icon\nCategories=Utility;\n",
            options.metadata.display_name_or_name(),
            options.metadata.name,
        );
        let _ = fs::create_dir_all(output.join("share/applications"));
        let _ = fs::write(output.join(format!("share/applications/{}.desktop", options.metadata.name)), entry);
    }

    Ok(PackageStats {
        path: options.output_path.clone(),
        target: options.target,
        signing_mode: options.signing.mode,
        asset_count: bundle_stats.asset_count,
        web_engine: options.web_engine,
        archive_path: None,
    })
}

fn create_ios_artifact(options: &PackageOptions) -> Result<PackageStats, String> {
    let output = Path::new(&options.output_path);
    fs::create_dir_all(output.join("zero-nativeHost")).map_err(|e| e.to_string())?;
    fs::write(output.join("README.md"), "iOS zero-native host skeleton.\n").map_err(|e| e.to_string())?;
    fs::write(output.join("Info.plist"), ios_info_plist()).map_err(|e| e.to_string())?;

    if let Some(ref binary_path) = options.binary_path {
        let bytes = fs::read(binary_path).map_err(|e| e.to_string())?;
        fs::write(output.join("Libraries/libzero-native.a"), bytes).map_err(|e| e.to_string())?;
    }

    Ok(PackageStats {
        path: options.output_path.clone(),
        target: PackageTarget::IOS,
        signing_mode: codesign::SigningMode::None,
        asset_count: 0,
        web_engine: options.web_engine,
        archive_path: None,
    })
}

fn create_android_artifact(options: &PackageOptions) -> Result<PackageStats, String> {
    let output = Path::new(&options.output_path);
    fs::create_dir_all(output.join("app/src/main/java/dev/zero_native")).map_err(|e| e.to_string())?;
    fs::create_dir_all(output.join("app/src/main/cpp")).map_err(|e| e.to_string())?;
    fs::write(output.join("README.md"), "Android zero-native host skeleton.\n").map_err(|e| e.to_string())?;

    if let Some(ref binary_path) = options.binary_path {
        let bytes = fs::read(binary_path).map_err(|e| e.to_string())?;
        fs::write(output.join("app/src/main/cpp/lib/libzero-native.a"), bytes).map_err(|e| e.to_string())?;
    }

    Ok(PackageStats {
        path: options.output_path.clone(),
        target: PackageTarget::Android,
        signing_mode: codesign::SigningMode::None,
        asset_count: 0,
        web_engine: options.web_engine,
        archive_path: None,
    })
}

fn macos_info_plist(metadata: &manifest::Metadata, executable_name: &str) -> Result<String, String> {
    let icon_name = metadata
        .icons
        .first()
        .map(|p| Path::new(p).file_name().unwrap_or_default().to_string_lossy().to_string())
        .unwrap_or_else(|| "AppIcon.icns".into());

    Ok(format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleIdentifier</key>
  <string>{}</string>
  <key>CFBundleName</key>
  <string>{}</string>
  <key>CFBundleDisplayName</key>
  <string>{}</string>
  <key>CFBundleExecutable</key>
  <string>{}</string>
  <key>CFBundleIconFile</key>
  <string>{}</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>{}</string>
  <key>CFBundleVersion</key>
  <string>{}</string>
</dict>
</plist>
"#,
        xml_escape(&metadata.id),
        xml_escape(&metadata.name),
        xml_escape(metadata.display_name_or_name()),
        xml_escape(executable_name),
        xml_escape(&icon_name),
        xml_escape(&metadata.version),
        xml_escape(&metadata.version),
    ))
}

fn ios_info_plist() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict><key>CFBundleIdentifier</key><string>dev.zero_native.ios</string><key>CFBundleName</key><string>zero-nativeHost</string></dict></plist>
"#
}

fn xml_escape(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c if (c as u32) < 0x20 && c != '\t' && c != '\n' && c != '\r' => {}
            _ => out.push(ch),
        }
    }
    out
}

fn copy_macos_cef_runtime(app_dir: &Path, cef_dir: &str) -> Result<(), String> {
    let frameworks_src = Path::new(cef_dir).join("Release/Chromium Embedded Framework.framework");
    let frameworks_dest = app_dir.join("Contents/Frameworks/Chromium Embedded Framework.framework");
    copy_tree(&frameworks_src, &frameworks_dest)?;

    let resources_src = Path::new(cef_dir).join("Resources");
    let resources_dest = app_dir.join("Contents/Resources/cef");
    if resources_src.exists() {
        let _ = copy_tree(&resources_src, &resources_dest);
    }

    Ok(())
}

fn copy_tree(source: &Path, dest: &Path) -> Result<(), String> {
    if !source.exists() {
        return Err(format!("source does not exist: {}", source.display()));
    }
    if source.is_dir() {
        let _ = fs::create_dir_all(dest);
        for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let src_path = entry.path();
            let file_name = entry.file_name();
            let dest_path = dest.join(&file_name);
            if src_path.is_dir() {
                copy_tree(&src_path, &dest_path)?;
            } else {
                fs::copy(&src_path, &dest_path).map_err(|e| e.to_string())?;
            }
        }
    } else {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::copy(source, dest).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn run_signing(app_dir: &Path, options: &PackageOptions) -> Result<(), String> {
    match options.signing.mode {
        codesign::SigningMode::None => {
            let _ = fs::write(
                app_dir.join("Contents/Resources/signing-plan.txt"),
                "signing=none\nunsigned local package\n",
            );
        }
        codesign::SigningMode::Adhoc => {
            match codesign::sign_ad_hoc(&options.output_path) {
                Ok(result) if result.ok => {
                    let _ = fs::write(
                        app_dir.join("Contents/Resources/signing-plan.txt"),
                        "signing=adhoc\nad-hoc signed\n",
                    );
                }
                _ => {
                    let _ = fs::write(
                        app_dir.join("Contents/Resources/signing-plan.txt"),
                        "signing=adhoc\ncodesign --sign - failed; bundle is unsigned\n",
                    );
                }
            }
        }
        codesign::SigningMode::Identity => {
            if let Some(ref identity) = options.signing.identity {
                match codesign::sign_identity(
                    &options.output_path,
                    identity,
                    options.signing.entitlements.as_deref(),
                ) {
                    Ok(result) if result.ok => {
                        let _ = fs::write(
                            app_dir.join("Contents/Resources/signing-plan.txt"),
                            &format!("signing=identity\nsigned with {}\n", identity),
                        );
                    }
                    _ => {
                        let _ = fs::write(
                            app_dir.join("Contents/Resources/signing-plan.txt"),
                            "signing=identity\ncodesign failed; bundle is unsigned\n",
                        );
                    }
                }
            } else {
                let _ = fs::write(
                    app_dir.join("Contents/Resources/signing-plan.txt"),
                    "signing=identity\nno identity provided; bundle is unsigned\n",
                );
            }
        }
    }
    Ok(())
}

fn create_archive(options: &PackageOptions) -> Result<Option<String>, String> {
    let (suffix, cmd) = match options.target {
        PackageTarget::MacOS => (".dmg", format!("hdiutil create -volname \"{}\" -srcfolder \"{}\" -ov -format UDZO", options.metadata.display_name_or_name(), options.output_path)),
        PackageTarget::Windows => (".zip", format!("cd \"{}\" && zip -r \"../{}-{}-{}-{}-archive.zip\" .", options.output_path, options.metadata.name, options.metadata.version, package_target_tag(options.target), options.optimize)),
        PackageTarget::Linux => (".tar.gz", format!("tar czf \"{}-{}-{}-{}-archive.tar.gz\" -C \"{}\" .", options.metadata.name, options.metadata.version, package_target_tag(options.target), options.optimize, options.output_path)),
        PackageTarget::IOS | PackageTarget::Android => return Ok(None),
    };

    let archive_path = format!(
        "{}-{}-{}-{}{}",
        options.metadata.name,
        options.metadata.version,
        package_target_tag(options.target),
        options.optimize,
        suffix,
    );

    let status = StdCommand::new("sh")
        .arg("-c")
        .arg(&cmd)
        .status()
        .map_err(|e| format!("archive failed: {}", e))?;

    if status.success() {
        Ok(Some(archive_path))
    } else {
        eprintln!("warning: archive creation failed for {}", archive_path);
        Ok(None)
    }
}

fn package_target_tag(target: PackageTarget) -> &'static str {
    match target {
        PackageTarget::MacOS => "macos",
        PackageTarget::Windows => "windows",
        PackageTarget::Linux => "linux",
        PackageTarget::IOS => "ios",
        PackageTarget::Android => "android",
    }
}
