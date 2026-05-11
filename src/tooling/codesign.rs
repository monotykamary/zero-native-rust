use std::process::Command as StdCommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SigningMode {
    None,
    Adhoc,
    Identity,
}

impl SigningMode {
    pub fn parse(value: &str) -> Option<SigningMode> {
        match value {
            "none" => Some(SigningMode::None),
            "adhoc" | "ad-hoc" => Some(SigningMode::Adhoc),
            "identity" => Some(SigningMode::Identity),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SigningConfig {
    pub mode: SigningMode,
    pub identity: Option<String>,
    pub entitlements: Option<String>,
    pub team_id: Option<String>,
}

impl Default for SigningConfig {
    fn default() -> Self {
        Self {
            mode: SigningMode::None,
            identity: None,
            entitlements: None,
            team_id: None,
        }
    }
}

#[derive(Debug)]
pub struct SignResult {
    pub ok: bool,
    pub message: String,
}

pub fn build_sign_command(
    app_path: &str,
    identity: &str,
    entitlements: Option<&str>,
    hardened_runtime: bool,
    deep: bool,
) -> String {
    let mut cmd = format!("codesign --sign {} --force", identity);
    if deep {
        cmd.push_str(" --deep");
    }
    if hardened_runtime {
        cmd.push_str(" --options runtime");
    }
    if let Some(ent) = entitlements {
        cmd.push_str(&format!(" --entitlements {}", ent));
    }
    cmd.push(' ');
    cmd.push_str(app_path);
    cmd
}

pub fn sign_ad_hoc(app_path: &str) -> Result<SignResult, String> {
    let cmd = build_sign_command(app_path, "-", None, false, true);
    run_shell(&cmd)
}

pub fn sign_identity(app_path: &str, identity: &str, entitlements: Option<&str>) -> Result<SignResult, String> {
    let cmd = build_sign_command(app_path, identity, entitlements, true, true);
    run_shell(&cmd)
}

pub fn build_notarize_submit_command(
    zip_path: &str,
    team_id: &str,
    apple_id: Option<&str>,
    password_keychain_item: Option<&str>,
) -> String {
    let mut cmd = format!("xcrun notarytool submit {} --team-id {}", zip_path, team_id);
    if let Some(id) = apple_id {
        cmd.push_str(&format!(" --apple-id {}", id));
    }
    if let Some(item) = password_keychain_item {
        cmd.push_str(&format!(" --password @keychain:{}", item));
    }
    cmd.push_str(" --wait");
    cmd
}

pub fn build_staple_command(app_path: &str) -> String {
    format!("xcrun stapler staple {}", app_path)
}

pub fn build_zip_command(app_path: &str, zip_path: &str) -> String {
    format!("ditto -c -k --keepParent {} {}", app_path, zip_path)
}

pub fn notarize(app_path: &str, team_id: &str, apple_id: Option<&str>, password_keychain_item: Option<&str>) -> Result<SignResult, String> {
    let zip_path = format!("{}.zip", app_path);

    let zip_cmd = build_zip_command(app_path, &zip_path);
    if run_shell(&zip_cmd).is_err() {
        return Ok(SignResult {
            ok: false,
            message: "failed to zip app for notarization".into(),
        });
    }

    let submit_cmd = build_notarize_submit_command(&zip_path, team_id, apple_id, password_keychain_item);
    if run_shell(&submit_cmd).is_err() {
        return Ok(SignResult {
            ok: false,
            message: "notarytool submit failed".into(),
        });
    }

    let staple_cmd = build_staple_command(app_path);
    if run_shell(&staple_cmd).is_err() {
        return Ok(SignResult {
            ok: false,
            message: "stapler staple failed".into(),
        });
    }

    Ok(SignResult {
        ok: true,
        message: "notarization complete".into(),
    })
}

fn run_shell(cmd: &str) -> Result<SignResult, String> {
    let output = StdCommand::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|e| format!("shell command failed: {}", e))?;
    Ok(SignResult {
        ok: output.status.success(),
        message: if output.status.success() {
            "signed".into()
        } else {
            "codesign failed".into()
        },
    })
}
