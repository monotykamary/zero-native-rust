#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform { MacOS, Windows, Linux, IOS, Android, Web, Unknown }

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.name()) }
}

impl Platform {
    pub fn name(self) -> &'static str {
        match self { Self::MacOS => "macos", Self::Windows => "windows", Self::Linux => "linux", Self::IOS => "ios", Self::Android => "android", Self::Web => "web", Self::Unknown => "unknown" }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch { X86_64, AArch64, Arm, Wasm32, Unknown }

impl Arch {
    pub fn name(self) -> &'static str {
        match self { Self::X86_64 => "x86_64", Self::AArch64 => "aarch64", Self::Arm => "arm", Self::Wasm32 => "wasm32", Self::Unknown => "unknown" }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Abi { Gnu, Msvc, Musl, Eabi, Unknown }

impl Abi {
    pub fn name(self) -> &'static str {
        match self { Self::Gnu => "gnu", Self::Msvc => "msvc", Self::Musl => "musl", Self::Eabi => "eabi", Self::Unknown => "unknown" }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServer { Wayland, X11, Quartz, Windows, Unknown }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuApi { Metal, Vulkan_1_1, Vulkan_1_2, OpenGl, Direct3D_11, Direct3D_12, WebGpu, Unknown }

impl GpuApi {
    pub fn name(self) -> &'static str {
        match self { Self::Metal => "metal", Self::Vulkan_1_1 => "vulkan-1.1", Self::Vulkan_1_2 => "vulkan-1.2", Self::OpenGl => "opengl", Self::Direct3D_11 => "d3d11", Self::Direct3D_12 => "d3d12", Self::WebGpu => "webgpu", Self::Unknown => "unknown" }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuStatus { Available, Unavailable, NotTested }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdkKind { Cef, Msvc, Ios, Android, Unknown }

impl SdkKind {
    pub fn name(self) -> &'static str {
        match self { Self::Cef => "cef", Self::Msvc => "msvc", Self::Ios => "ios", Self::Android => "android", Self::Unknown => "unknown" }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status { Installed, NotInstalled, Outdated, Unknown }

#[derive(Debug, Clone)]
pub struct Target {
    pub platform: Platform,
    pub arch: Arch,
    pub abi: Abi,
    pub display_server: DisplayServer,
}

impl Target {
    pub fn current() -> Self {
        let (platform, arch, abi, display_server) = detect_target();
        Self { platform, arch, abi, display_server }
    }

    pub fn triple(&self) -> String {
        format!("{}-{}-{}", self.arch.name(), self.platform.name(), self.abi.name())
    }
}

impl Default for Target {
    fn default() -> Self { Self::current() }
}

#[derive(Debug, Clone)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct SdkRecord {
    pub kind: SdkKind,
    pub version: String,
    pub path: Option<String>,
    pub status: Status,
}

#[derive(Debug, Clone)]
pub struct GpuApiRecord {
    pub api: GpuApi,
    pub status: GpuStatus,
    pub version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HostProbeInputs {
    pub target: Target,
    pub env_vars: Vec<EnvVar>,
    pub sdks: Vec<SdkRecord>,
    pub gpus: Vec<GpuApiRecord>,
}

#[derive(Debug, Clone)]
pub struct HostInfo {
    pub target: Target,
    pub display_servers: Vec<DisplayServer>,
    pub available_gpus: Vec<GpuApiRecord>,
    pub installed_sdks: Vec<SdkRecord>,
    pub env_vars: Vec<EnvVar>,
}

impl HostInfo {
    pub fn detect(target: Target) -> Self {
        let sdks = detect_sdks(&target);
        let gpus = detect_gpus(&target);
        let display_servers = detect_display_servers_inner(&target);
        Self {
            target,
            display_servers,
            available_gpus: gpus,
            installed_sdks: sdks,
            env_vars: Vec::new(),
        }
    }

    pub fn probe() -> Self {
        Self::detect(Target::current())
    }
}

#[derive(Debug, Clone)]
pub struct DoctorCheck {
    pub name: String,
    pub status: Status,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct DoctorReport {
    pub checks: Vec<DoctorCheck>,
}

impl DoctorReport {
    pub fn is_healthy(&self) -> bool { self.checks.iter().all(|c| c.status == Status::Installed) }
}

pub fn run_doctor(target: &Target) -> DoctorReport {
    let mut checks = Vec::new();
    let sdks = detect_sdks(target);
    for sdk in &sdks {
        checks.push(DoctorCheck {
            name: format!("SDK: {}", sdk.kind.name()),
            status: sdk.status,
            message: match &sdk.path { Some(p) => format!("v{} at {}", sdk.version, p), None => format!("v{} (no path)", sdk.version) },
        });
    }
    let gpus = detect_gpus(target);
    for gpu in &gpus {
        checks.push(DoctorCheck {
            name: format!("GPU: {}", gpu.api.name()),
            status: match gpu.status { GpuStatus::Available => Status::Installed, GpuStatus::Unavailable => Status::NotInstalled, GpuStatus::NotTested => Status::Unknown },
            message: gpu.version.as_deref().unwrap_or("unknown version").to_string(),
        });
    }
    DoctorReport { checks }
}

fn detect_target() -> (Platform, Arch, Abi, DisplayServer) {
    let platform = if cfg!(target_os = "macos") { Platform::MacOS }
        else if cfg!(target_os = "windows") { Platform::Windows }
        else if cfg!(target_os = "linux") { Platform::Linux }
        else { Platform::Unknown };
    let arch = if cfg!(target_arch = "x86_64") { Arch::X86_64 }
        else if cfg!(target_arch = "aarch64") { Arch::AArch64 }
        else if cfg!(target_arch = "arm") { Arch::Arm }
        else { Arch::Unknown };
    let abi = if cfg!(target_env = "gnu") { Abi::Gnu }
        else if cfg!(target_env = "msvc") { Abi::Msvc }
        else if cfg!(target_env = "musl") { Abi::Musl }
        else { Abi::Unknown };
    let display_server = match platform {
        Platform::MacOS => DisplayServer::Quartz,
        Platform::Windows => DisplayServer::Windows,
        Platform::Linux => {
            if std::env::var("WAYLAND_DISPLAY").is_ok() { DisplayServer::Wayland } else { DisplayServer::X11 }
        }
        _ => DisplayServer::Unknown,
    };
    (platform, arch, abi, display_server)
}

pub fn detect_display_server(env_vars: &[(String, String)], platform: Platform) -> DisplayServer {
    match platform {
        Platform::MacOS => DisplayServer::Quartz,
        Platform::Windows => DisplayServer::Windows,
        Platform::Linux => {
            if env_vars.iter().any(|(k, _)| k == "WAYLAND_DISPLAY") { DisplayServer::Wayland } else { DisplayServer::X11 }
        }
        _ => DisplayServer::Unknown,
    }
}

fn detect_display_servers_inner(target: &Target) -> Vec<DisplayServer> {
    match target.platform {
        Platform::MacOS => vec![DisplayServer::Quartz],
        Platform::Windows => vec![DisplayServer::Windows],
        Platform::Linux => {
            let mut servers = Vec::new();
            if std::env::var("WAYLAND_DISPLAY").is_ok() { servers.push(DisplayServer::Wayland); }
            if std::env::var("DISPLAY").is_ok() || servers.is_empty() { servers.push(DisplayServer::X11); }
            servers
        }
        _ => vec![DisplayServer::Unknown],
    }
}

fn detect_sdks(target: &Target) -> Vec<SdkRecord> {
    let mut sdks = Vec::new();
    match target.platform {
        Platform::MacOS => {
            sdks.push(SdkRecord { kind: SdkKind::Cef, version: "0".into(), path: None, status: Status::NotInstalled });
        }
        Platform::Windows => {
            sdks.push(SdkRecord { kind: SdkKind::Msvc, version: "0".into(), path: None, status: Status::NotInstalled });
        }
        Platform::Linux => {
            sdks.push(SdkRecord { kind: SdkKind::Cef, version: "0".into(), path: None, status: Status::NotInstalled });
        }
        _ => {}
    }
    sdks
}

fn detect_gpus(target: &Target) -> Vec<GpuApiRecord> {
    let mut gpus = Vec::new();
    match target.platform {
        Platform::MacOS => {
            gpus.push(GpuApiRecord { api: GpuApi::Metal, status: GpuStatus::NotTested, version: None });
        }
        Platform::Windows => {
            gpus.push(GpuApiRecord { api: GpuApi::Direct3D_11, status: GpuStatus::NotTested, version: None });
        }
        Platform::Linux => {
            gpus.push(GpuApiRecord { api: GpuApi::Vulkan_1_1, status: GpuStatus::NotTested, version: None });
        }
        _ => {}
    }
    gpus
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_target_is_valid() {
        let target = Target::current();
        assert!(target.platform != Platform::Unknown || target.arch != Arch::Unknown);
    }

    #[test]
    fn target_triple_format() {
        let target = Target { platform: Platform::MacOS, arch: Arch::AArch64, abi: Abi::Unknown, display_server: DisplayServer::Quartz };
        assert_eq!("aarch64-macos-unknown", target.triple());
    }

    #[test]
    fn detect_host_info() {
        let target = Target::current();
        let info = HostInfo::detect(target);
        assert!(!info.available_gpus.is_empty() || !info.installed_sdks.is_empty());
    }

    #[test]
    fn doctor_report_runs() {
        let target = Target::current();
        let report = run_doctor(&target);
        assert!(!report.checks.is_empty() || target.platform == Platform::Unknown);
    }

    #[test]
    fn gpu_api_names() {
        assert_eq!("metal", GpuApi::Metal.name());
        assert_eq!("vulkan-1.2", GpuApi::Vulkan_1_2.name());
    }

    #[test]
    fn platform_names() {
        assert_eq!("macos", Platform::MacOS.name());
        assert_eq!("linux", Platform::Linux.name());
    }
}
