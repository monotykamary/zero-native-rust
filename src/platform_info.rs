#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    MacOS,
    Windows,
    Linux,
    IOS,
    Android,
    Unknown,
}

impl Platform {
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::MacOS
        } else if cfg!(target_os = "linux") {
            Self::Linux
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Unknown
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    X86_64,
    AArch64,
    Arm,
    Riscv64,
    Wasm32,
    Unknown,
}

impl Arch {
    pub fn current() -> Self {
        if cfg!(target_arch = "x86_64") {
            Self::X86_64
        } else if cfg!(target_arch = "aarch64") {
            Self::AArch64
        } else if cfg!(target_arch = "arm") {
            Self::Arm
        } else if cfg!(target_arch = "riscv64") {
            Self::Riscv64
        } else {
            Self::Unknown
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServer {
    None,
    AppKit,
    Win32,
    Wayland,
    X11,
    UIKit,
    Unknown,
}

pub fn detect_display_server(env: &[(String, String)], platform: Platform) -> DisplayServer {
    match platform {
        Platform::MacOS => DisplayServer::AppKit,
        Platform::Windows => DisplayServer::Win32,
        Platform::IOS => DisplayServer::UIKit,
        Platform::Linux => {
            for (key, value) in env {
                if key == "WAYLAND_DISPLAY" && !value.is_empty() {
                    return DisplayServer::Wayland;
                }
                if key == "DISPLAY" && !value.is_empty() {
                    return DisplayServer::X11;
                }
            }
            DisplayServer::None
        }
        _ => DisplayServer::Unknown,
    }
}

#[derive(Debug, Clone)]
pub struct Target {
    pub os: Platform,
    pub arch: Arch,
}

impl Target {
    pub fn current() -> Self {
        Self {
            os: Platform::current(),
            arch: Arch::current(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_target_maps_builtin_values() {
        let target = Target::current();
        assert_ne!(Platform::Unknown, target.os);
        assert_ne!(Arch::Unknown, target.arch);
    }

    #[test]
    fn display_server_detection() {
        assert_eq!(DisplayServer::Wayland, detect_display_server(
            &vec![("WAYLAND_DISPLAY".into(), "wayland-0".into())], Platform::Linux));
        assert_eq!(DisplayServer::X11, detect_display_server(
            &vec![("DISPLAY".into(), ":0".into())], Platform::Linux));
        assert_eq!(DisplayServer::None, detect_display_server(&[], Platform::Linux));
        assert_eq!(DisplayServer::AppKit, detect_display_server(&[], Platform::MacOS));
    }
}
