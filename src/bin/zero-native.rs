use std::env;
use std::process;

const VERSION: &str = "0.1.9";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        usage();
        return;
    }

    let command = &args[1];
    match command.as_str() {
        "--help" | "-h" | "help" => usage(),
        "--version" | "version" => println!("zero-native {}", VERSION),
        "init" => run_init(&args[2..]),
        "doctor" => run_doctor(&args[2..]),
        "validate" => run_validate(&args[2..]),
        "bundle-assets" => run_bundle_assets(&args[2..]),
        "package" => run_package(&args[2..]),
        "package-windows" => run_package_shortcut(&args[2..], "windows"),
        "package-linux" => run_package_shortcut(&args[2..], "linux"),
        "package-ios" => run_package_ios(&args[2..]),
        "package-android" => run_package_android(&args[2..]),
        "cef" => run_cef(&args[2..]),
        "dev" => run_dev(&args[2..]),
        "automate" => run_automate(&args[2..]),
        _ => {
            eprintln!("unknown command: {}", command);
            usage();
            process::exit(1);
        }
    }
}

fn usage() {
    eprintln!(
        "usage: zero-native <command>

commands:
  init [path] --frontend <next|vite|react|svelte|vue>
  cef install|path|doctor [--dir path] [--version version] [--source prepared|official] [--force]
  doctor [--strict] [--manifest app.zon] [--web-engine system|chromium] [--cef-dir path] [--cef-auto-install]
  validate [app.zon]
  bundle-assets [app.zon] [assets] [output]
  package [--target macos] [--output path] [--binary path] [--assets path] [--web-engine system|chromium] [--cef-dir path] [--cef-auto-install] [--signing none|adhoc|identity] [--identity name] [--entitlements path] [--team-id id] [--archive]
  dev [--manifest app.zon] --binary path [--url http://127.0.0.1:5173/] [--command \"npm run dev\"] [--timeout-ms 30000]
  package-windows [--output path] [--binary path]
  package-linux [--output path] [--binary path]
  package-ios [--output path] [--binary path]
  package-android [--output path] [--binary path]
  automate <command>
  version"
    );
}

fn fail(message: &str) -> ! {
    eprintln!("{}", message);
    process::exit(1);
}

fn flag_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    for (i, arg) in args.iter().enumerate() {
        if arg == name && i + 1 < args.len() {
            return Some(&args[i + 1]);
        }
    }
    None
}

fn flag_bool(args: &[String], name: &str) -> bool {
    args.iter().any(|arg| arg == name)
}

fn positional_arg(args: &[String]) -> Option<&str> {
    let flag_names = [
        "--frontend", "--manifest", "--target", "--output", "--binary",
        "--assets", "--web-engine", "--cef-dir", "--signing", "--identity",
        "--entitlements", "--team-id", "--command", "--url", "--timeout-ms",
        "--optimize", "--download-url",
    ];
    let mut skip_next = false;
    for arg in args {
        if skip_next {
            skip_next = false;
            continue;
        }
        if arg.starts_with("--") {
            if flag_names.contains(&arg.as_str()) {
                skip_next = true;
            }
            continue;
        }
        return Some(arg);
    }
    None
}

fn run_init(args: &[String]) {
    let destination = positional_arg(args).unwrap_or(".");
    let frontend_str = flag_value(args, "--frontend")
        .unwrap_or_else(|| fail("--frontend is required: next, vite, react, svelte, vue"));
    let frontend = zero_native::tooling::templates::Frontend::parse(frontend_str)
        .unwrap_or_else(|| fail("invalid --frontend value: use next, vite, react, svelte, or vue"));

    let app_name = if destination == "." {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "zero-native-app".into())
    } else {
        std::path::Path::new(destination)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "zero-native-app".into())
    };

    let options = zero_native::tooling::templates::InitOptions {
        app_name,
        framework_path: ".".into(),
        frontend,
    };

    if let Err(e) = zero_native::tooling::templates::write_default_app(destination, &options) {
        eprintln!("init failed: {}", e);
        process::exit(1);
    }

    println!("created zero-native app at {} (frontend: {})", destination, frontend_str);

    println!("\nNext steps:");
    if destination != "." {
        println!("  cd {}", destination);
    }
    println!("  zig build run");
}

fn run_doctor(args: &[String]) {
    match zero_native::tooling::doctor::run(args) {
        Ok(()) => {}
        Err(zero_native::tooling::doctor::Error::DoctorProblems) => {
            process::exit(1);
        }
        Err(zero_native::tooling::doctor::Error::InvalidArguments) => {
            eprintln!("invalid arguments for doctor");
            process::exit(1);
        }
    }
}

fn run_validate(args: &[String]) {
    let path = if args.is_empty() { "app.zon" } else { &args[0] };
    let result = zero_native::tooling::manifest::validate_file(path);
    zero_native::tooling::manifest::print_diagnostic(&result);
    if !result.ok {
        process::exit(1);
    }
}

fn run_bundle_assets(args: &[String]) {
    let positional: Vec<&str> = args.iter()
        .filter(|a| !a.starts_with('-'))
        .map(|a| a.as_str())
        .collect();

    let manifest_path = flag_value(args, "--manifest")
        .map(String::from)
        .unwrap_or_else(|| positional.first().unwrap_or(&"app.zon").to_string());

    let metadata = match zero_native::tooling::manifest::read_metadata(&manifest_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error reading manifest: {}", e);
            process::exit(1);
        }
    };

    let assets_dir = positional.get(1).copied()
        .unwrap_or_else(|| metadata.frontend.as_ref().map(|f| f.dist.as_str()).unwrap_or("assets"));
    let output_dir = positional.get(2).copied().unwrap_or("zig-out/assets");

    match zero_native::tooling::bundle_assets::bundle(assets_dir, output_dir) {
        Ok(stats) => println!("bundled {} assets into {}", stats.asset_count, output_dir),
        Err(e) => {
            eprintln!("bundle-assets failed: {}", e);
            process::exit(1);
        }
    }
}

fn run_package(args: &[String]) {
    let manifest_path = flag_value(args, "--manifest").unwrap_or("app.zon");
    let metadata = match zero_native::tooling::manifest::read_metadata(manifest_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error reading manifest: {}", e);
            process::exit(1);
        }
    };

    let target_name = flag_value(args, "--target").unwrap_or("macos");
    let target = zero_native::tooling::package::PackageTarget::parse(target_name)
        .unwrap_or_else(|| fail("invalid package target"));

    let web_engine_override = flag_value(args, "--web-engine")
        .map(|v| zero_native::tooling::web_engine::Engine::parse(v)
            .unwrap_or_else(|| fail("invalid web engine")));

    let cef_dir = flag_value(args, "--cef-dir")
        .map(String::from)
        .unwrap_or_else(|| metadata.cef.dir.clone());

    let web_engine = web_engine_override.unwrap_or(
        zero_native::tooling::web_engine::Engine::parse(&metadata.web_engine)
            .unwrap_or(zero_native::tooling::web_engine::Engine::System)
    );

    let signing_name = flag_value(args, "--signing").unwrap_or("none");
    let signing = zero_native::tooling::codesign::SigningConfig {
        mode: zero_native::tooling::codesign::SigningMode::parse(signing_name)
            .unwrap_or_else(|| fail("invalid signing mode")),
        identity: flag_value(args, "--identity").map(String::from),
        entitlements: flag_value(args, "--entitlements").map(String::from),
        team_id: flag_value(args, "--team-id").map(String::from),
    };

    let output_dir = flag_value(args, "--output").map(String::from)
        .unwrap_or_else(|| "zig-out/package/zero-native-local.app".into());

    let archive = flag_bool(args, "--archive");

    if web_engine == zero_native::tooling::web_engine::Engine::Chromium && flag_bool(args, "--cef-auto-install") {
        if let Err(e) = zero_native::tooling::cef::run(&[
            "install".into(),
            "--dir".into(),
            cef_dir.clone(),
        ]) {
            eprintln!("CEF auto-install failed: {:?}", e);
        }
    }

    let options = zero_native::tooling::package::PackageOptions {
        metadata,
        target,
        optimize: flag_value(args, "--optimize").unwrap_or("Debug").to_string(),
        output_path: output_dir,
        binary_path: flag_value(args, "--binary").map(String::from),
        assets_dir: flag_value(args, "--assets").map(String::from)
            .unwrap_or_else(|| "assets".into()),
        web_engine,
        cef_dir,
        signing,
        archive,
    };

    match zero_native::tooling::package::create_package(&options) {
        Ok(stats) => zero_native::tooling::package::print_diagnostic(&stats),
        Err(e) => {
            eprintln!("package failed: {}", e);
            process::exit(1);
        }
    }
}

fn run_package_shortcut(args: &[String], target_str: &str) {
    let manifest_path = flag_value(args, "--manifest").unwrap_or("app.zon");
    let metadata = match zero_native::tooling::manifest::read_metadata(manifest_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error reading manifest: {}", e);
            process::exit(1);
        }
    };

    let target = zero_native::tooling::package::PackageTarget::parse(target_str)
        .unwrap_or_else(|| fail("invalid package target"));
    let cef_dir = flag_value(args, "--cef-dir")
        .map(String::from)
        .unwrap_or_else(|| metadata.cef.dir.clone());

    let default_output = format!("zig-out/package/{}", target_str);
    let options = zero_native::tooling::package::PackageOptions {
        metadata,
        target,
        optimize: "Debug".into(),
        output_path: flag_value(args, "--output").map(String::from)
            .unwrap_or(default_output),
        binary_path: flag_value(args, "--binary").map(String::from),
        assets_dir: flag_value(args, "--assets").map(String::from)
            .unwrap_or_else(|| "assets".into()),
        web_engine: zero_native::tooling::web_engine::Engine::System,
        cef_dir,
        signing: zero_native::tooling::codesign::SigningConfig::default(),
        archive: false,
    };

    match zero_native::tooling::package::create_package(&options) {
        Ok(stats) => zero_native::tooling::package::print_diagnostic(&stats),
        Err(e) => {
            eprintln!("package failed: {}", e);
            process::exit(1);
        }
    }
}

fn run_package_ios(args: &[String]) {
    let manifest_path = flag_value(args, "--manifest").unwrap_or("app.zon");
    let metadata = match zero_native::tooling::manifest::read_metadata(manifest_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error reading manifest: {}", e);
            process::exit(1);
        }
    };

    let options = zero_native::tooling::package::PackageOptions {
        metadata,
        target: zero_native::tooling::package::PackageTarget::IOS,
        optimize: "Debug".into(),
        output_path: flag_value(args, "--output").map(String::from)
            .unwrap_or_else(|| "zig-out/mobile/ios".into()),
        binary_path: flag_value(args, "--binary").map(String::from),
        assets_dir: flag_value(args, "--assets").map(String::from)
            .unwrap_or_else(|| "assets".into()),
        web_engine: zero_native::tooling::web_engine::Engine::System,
        cef_dir: String::new(),
        signing: zero_native::tooling::codesign::SigningConfig::default(),
        archive: false,
    };

    match zero_native::tooling::package::create_package(&options) {
        Ok(stats) => zero_native::tooling::package::print_diagnostic(&stats),
        Err(e) => {
            eprintln!("package failed: {}", e);
            process::exit(1);
        }
    }
}

fn run_package_android(args: &[String]) {
    let manifest_path = flag_value(args, "--manifest").unwrap_or("app.zon");
    let metadata = match zero_native::tooling::manifest::read_metadata(manifest_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error reading manifest: {}", e);
            process::exit(1);
        }
    };

    let options = zero_native::tooling::package::PackageOptions {
        metadata,
        target: zero_native::tooling::package::PackageTarget::Android,
        optimize: "Debug".into(),
        output_path: flag_value(args, "--output").map(String::from)
            .unwrap_or_else(|| "zig-out/mobile/android".into()),
        binary_path: flag_value(args, "--binary").map(String::from),
        assets_dir: flag_value(args, "--assets").map(String::from)
            .unwrap_or_else(|| "assets".into()),
        web_engine: zero_native::tooling::web_engine::Engine::System,
        cef_dir: String::new(),
        signing: zero_native::tooling::codesign::SigningConfig::default(),
        archive: false,
    };

    match zero_native::tooling::package::create_package(&options) {
        Ok(stats) => zero_native::tooling::package::print_diagnostic(&stats),
        Err(e) => {
            eprintln!("package failed: {}", e);
            process::exit(1);
        }
    }
}

fn run_cef(args: &[String]) {
    match zero_native::tooling::cef::run(args) {
        Ok(()) => {}
        Err(zero_native::tooling::cef::Error::InvalidArguments) => {
            process::exit(1);
        }
        Err(e) => {
            eprintln!("cef error: {:?}", e);
            process::exit(1);
        }
    }
}

fn run_dev(args: &[String]) {
    let manifest_path = flag_value(args, "--manifest").unwrap_or("app.zon");
    let metadata = match zero_native::tooling::manifest::read_metadata(manifest_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error reading manifest: {}", e);
            process::exit(1);
        }
    };

    let command_override = flag_value(args, "--command")
        .map(|v| v.split_whitespace().map(String::from).collect());

    let options = zero_native::tooling::dev::Options {
        metadata,
        binary_path: flag_value(args, "--binary").map(String::from),
        url_override: flag_value(args, "--url").map(String::from),
        command_override,
        timeout_ms: flag_value(args, "--timeout-ms")
            .map(|v| v.parse().unwrap_or(30_000)),
    };

    match zero_native::tooling::dev::run(&options) {
        Ok(()) => {}
        Err(zero_native::tooling::dev::Error::MissingFrontend) => {
            eprintln!("dev: app.zon does not define a frontend section");
            process::exit(1);
        }
        Err(zero_native::tooling::dev::Error::MissingDevConfig) => {
            eprintln!("dev: app.zon frontend does not define a dev section");
            process::exit(1);
        }
        Err(zero_native::tooling::dev::Error::MissingBinary) => {
            eprintln!("dev: --binary path is required");
            process::exit(1);
        }
        Err(zero_native::tooling::dev::Error::Timeout) => {
            eprintln!("dev: timed out waiting for frontend dev server");
            process::exit(1);
        }
        Err(zero_native::tooling::dev::Error::InvalidUrl) => {
            eprintln!("dev: invalid dev server URL");
            process::exit(1);
        }
    }
}

fn run_automate(args: &[String]) {
    if let Err(e) = zero_native::tooling::automation_cli::run(args) {
        match e {
            zero_native::tooling::automation_cli::Error::UnsupportedCommand => {
                process::exit(1);
            }
            zero_native::tooling::automation_cli::Error::AutomationFailed => {
                process::exit(1);
            }
            zero_native::tooling::automation_cli::Error::Timeout => {
                process::exit(1);
            }
        }
    }
}
