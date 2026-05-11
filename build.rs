use std::env;
use std::path::PathBuf;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    let src_dir = PathBuf::from("../zero-native/src/platform");
    let header_dir = src_dir.join(&target_os);

    match target_os.as_str() {
        "macos" => {
            cc::Build::new()
                .file(src_dir.join("macos/appkit_host.m"))
                .file(src_dir.join("macos/automation_host.m"))
                .flag("-framework")
                .flag("AppKit")
                .flag("-framework")
                .flag("WebKit")
                .flag("-framework")
                .flag("CoreFoundation")
                .flag("-framework")
                .flag("UniformTypeIdentifiers")
                .compile("appkit_host");

            let bindings = bindgen::Builder::default()
                .header(src_dir.join("macos/appkit_host.h").to_str().unwrap())
                .header(src_dir.join("macos/automation_host.h").to_str().unwrap())
                .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
                .generate()
                .expect("unable to generate appkit bindings");
            bindings
                .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("appkit_bindings.rs"))
                .expect("couldn't write appkit bindings");
        }
        "linux" => {
            cc::Build::new()
                .file(src_dir.join("linux/gtk_host.c"))
                .flag("`pkg-config --cflags gtk4 webkitgtk-6.0`")
                .compile("gtk_host");

            let bindings = bindgen::Builder::default()
                .header(src_dir.join("linux/gtk_host.h").to_str().unwrap())
                .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
                .generate()
                .expect("unable to generate gtk bindings");
            bindings
                .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("gtk_bindings.rs"))
                .expect("couldn't write gtk bindings");
        }
        "windows" => {
            cc::Build::new()
                .file(src_dir.join("windows/webview2_host.cpp"))
                .cpp(true)
                .flag("/std:c++20")
                .compile("webview2_host");

            let bindings = bindgen::Builder::default()
                .header(src_dir.join("windows/webview2_host.cpp").to_str().unwrap())
                .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
                .generate()
                .expect("unable to generate windows bindings");
            bindings
                .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("windows_bindings.rs"))
                .expect("couldn't write windows bindings");
        }
        _ => {}
    }
}
