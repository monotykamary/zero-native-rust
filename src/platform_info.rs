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

pub fn detect_display_server(_env: &[(String, String)], platform: Platform) -> DisplayServer {
    match platform {
        Platform::MacOS => DisplayServer::AppKit,
        Platform::Windows => DisplayServer::Win32,
        Platform::IOS => DisplayServer::UIKit,
        Platform::Linux => DisplayServer::None, // would check env vars
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
