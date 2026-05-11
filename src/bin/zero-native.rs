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
        "doctor" => run_doctor(&args[2..]),
        "validate" => run_validate(&args[2..]),
        "init" => run_init(&args[2..]),
        "bundle-assets" => run_bundle_assets(&args[2..]),
        "package" => run_package(&args[2..]),
        "cef" => run_cef(&args[2..]),
        "dev" => run_dev(&args[2..]),
        _ => {
            eprintln!("unknown command: {}", command);
            usage();
            process::exit(1);
        }
    }
}

fn usage() {
    eprintln!(
        "zero-native {} — build native desktop apps with web UI

Usage: zero-native <command> [options]

Commands:
  init              Create a new zero-native app
  doctor            Print platform diagnostics
  validate          Validate app.zon manifest
  bundle-assets     Bundle app assets
  package           Create a packaged app artifact
  cef               Manage Chromium Embedded Framework
  dev               Run managed dev server and native shell
  version           Print version
  help              Print this help message",
        VERSION
    );
}

fn run_doctor(args: &[String]) {
    use zero_native::platform_info;
    let target = platform_info::Target::current();
    println!(
        "target: {:?}-{:?}",
        std::mem::discriminant(&target.os),
        std::mem::discriminant(&target.arch)
    );
    println!("platform: {:?}", target.os);
}

fn run_validate(args: &[String]) {
    let path = if args.is_empty() { "app.zon" } else { &args[0] };
    match std::fs::read_to_string(path) {
        Ok(_content) => println!("valid: {}", path),
        Err(e) => {
            eprintln!("error reading {}: {}", path, e);
            process::exit(1);
        }
    }
}

fn run_init(args: &[String]) {
    eprintln!("init: not yet implemented in Rust port");
    process::exit(1);
}

fn run_bundle_assets(args: &[String]) {
    eprintln!("bundle-assets: not yet implemented in Rust port");
    process::exit(1);
}

fn run_package(args: &[String]) {
    eprintln!("package: not yet implemented in Rust port");
    process::exit(1);
}

fn run_cef(args: &[String]) {
    eprintln!("cef: not yet implemented in Rust port");
    process::exit(1);
}

fn run_dev(args: &[String]) {
    eprintln!("dev: not yet implemented in Rust port");
    process::exit(1);
}
