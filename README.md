# zero-native (Rust)

Build native desktop apps with web UI. Tiny binaries. Minimal memory. Instant rebuilds.

> **⚠️ Experimental port** — This is a Rust port of [zero-native](https://github.com/vercel-labs/zero-native), which was originally written in Zig. It is an experiment to evaluate Rust as an alternative implementation language. The API surface is complete, but bugs, behavioral differences, and missing edge cases likely remain. It is not yet production-ready — use at your own risk, and prefer the [Zig original](https://github.com/vercel-labs/zero-native) for anything critical.

## Architecture

The Rust runtime re-implements every public module from the Zig original. The C/ObjC/C++ platform host layer is carried over unchanged — compiled via `cc` and `bindgen` in `build.rs` — so the native window, webview, and OS integration code is identical to the Zig version.

```
┌─────────────────────────────────────────────────────┐
│  Rust runtime                                       │
│  bridge • security • window mgmt • extensions       │
│  events • automation • app_manifest • diagnostics   │
│  geometry • json • frontend • embed • window_state  │
│  ~12.5k lines of Rust                               │
├──────────────────────┬──────────────────────────────┤
│  PlatformHost trait  │  NullPlatform • MacPlatform  │
│  (safe Rust membrane)│  LinuxPlatform • WinPlatform │
└──────────────────────┴──────────────────────────────┘
               │ unsafe FFI (isolated inside trait impls)
               ▼
┌──────────────────────────────────────────────────────┐
│  C/ObjC/C++ platform hosts (unchanged from Zig)      │
│  appkit_host.m  •  gtk_host.c  •  webview2_host.cpp  │
│  cef_host.mm                                         │
└──────────────────────────────────────────────────────┘
```

## Quick start

```bash
cargo build
cargo test          # 175 tests
cargo run --release # 783 KB binary
```

## Commands

| Command | Description |
|---------|-------------|
| `init` | Create a new app (`--frontend next\|vite\|react\|svelte\|vue`) |
| `doctor` | Print platform diagnostics (`--strict`, `--manifest`, `--web-engine`) |
| `validate` | Validate app.zon manifest |
| `bundle-assets` | Bundle app assets into output directory |
| `package` | Create packaged artifact (macOS .app, Windows, Linux, iOS, Android) |
| `cef` | Manage Chromium Embedded Framework (`install`, `path`, `doctor`, `prepare-release`) |
| `dev` | Run frontend dev server + native shell |
| `automate` | Automation protocol (`list`, `snapshot`, `reload`, `wait`, `bridge`) |
| `version` | Print version |

## Module coverage

| Module | Lines | Description |
|--------|------:|-------------|
| `geometry` | 1031 | PointF/D, SizeF/D, RectF/D/I/U, InsetsF/D/I/U, OffsetF/D/I/U, ScaleF/D/I/U, ConstraintsF/D/I/U, Rounding, Edge, split, snap, logical↔physical conversion |
| `trace` | 556 | Trace records (Duration, Counter, Gauge, Frame), Buffer/Fanout sinks, JSON/text formatting |
| `assets` | 379 | Asset discovery, SHA-256 hashing, media type inference, path normalization, RuntimeAssets, validate_id, validate manifest |
| `app_dirs` | 341 | Platform-aware app directories (macOS/Linux/Windows), path resolution from env vars |
| `app_manifest` | 668 | Manifest types (PackageKind, BridgeConfig, FrontendConfig, SecurityConfig, PlatformSettings, etc.), full validation suite, ZON parsing |
| `diagnostics` | 222 | Diagnostic records, SourceMap, primary/secondary/note/suggestion helpers, format_text/format_json_line, validate_diagnostic |
| `platform_info` | 300 | Arch, Abi, DisplayServer, GpuApi, SdkKind, Status, Target detection, HostInfo probing, DoctorReport |
| `debug` | 214 | TraceMode, LogFormat, LogPaths, LogSetup, FileTraceSink, FanoutTraceSink, setup_logging, install_panic_capture |
| `security` | 86 | ExternalLinkAction/Policy, NavigationPolicy, Policy, has_permission/allows_origin |
| `bridge` | 545 | AsyncHandler/AsyncResponder/AsyncRegistry, Request parser, write_success/error_response, write_json_string, is_valid_json_value, Dispatcher with policy checking |
| `platform` | 471 | PlatformHost trait (20 methods), NullPlatform, WindowState/Info/Options, Event, BridgeMessage, FileFilter, Dialog types (Open/Save/Message), Tray types, Backend enum |
| `runtime` | 863 | Runtime with complete window management, bridge message dispatch, builtin window/dialog bridge commands, automation integration, reload_windows, emit_window_event, respond_to_bridge, allows_builtin_bridge_command |
| `extensions` | 200 | ModuleRegistry with validate/start_all/stop_all/dispatch_command/has_capability/find_by_id |
| `automation` | 276 | Command parsing, snapshot (write_text/write_a11y_text), Server with publish/publish_bridge_response/take_command |
| `window_state` | 246 | Store with save/load, ZON parser (parse_window/parse_windows), default_paths, write_windows |
| `json` | 354 | Recursive-descent JSON parser, field extraction, string unescape with \\uXXXX, write_string, StringStorage, is_valid_value |
| `frontend` | 68 | source_from_env, production_source, Config |
| `embed` | 198 | EmbeddedApp with start/stop/resize/frame/touch/set_asset_root, all zero_native_app_* FFI exports |
| `js` | 78 | Bridge availability check, NullBridge |
| `policy_values` | 49 | join() with newline-separated buffer writing |

### Platform backends

| Backend | Lines | FFI bindings | Status |
|---------|------:|-------------|--------|
| `macos` | 443 | 23 `extern fn` → `appkit_host.m` compiled via `cc` | ✅ Full |
| `linux` | 253 | 19 `extern fn` → `gtk_host.c` compiled via `cc` | ✅ Full (tray returns UnsupportedService) |
| `windows` | 174 | 16 `extern fn` → `webview2_host.cpp` compiled via `cc` | ✅ Core (dialogs return UnsupportedService) |

### CLI tooling

| Module | Lines | Description |
|--------|------:|-------------|
| `templates` | 1068 | Scaffold new apps with 5 frontend frameworks (Next/Vite/React/Svelte/Vue) |
| `package` | 722 | Create macOS .app, Windows/Linux artifacts, iOS/Android skeletons, archive, signing |
| `cef` | 609 | Install/verify/prepare Chromium Embedded Framework, LayoutReport, archive name/URL helpers |
| `manifest` | 499 | Read/parse/validate app.zon, ZON field extraction |
| `doctor` | 294 | Platform diagnostics, SDK/GPU checks, web engine resolution |
| `codesign` | 150 | Ad-hoc/identity signing, notarization, command builders |
| `dev` | 169 | Frontend dev server + native shell |
| `bundle_assets` | 112 | Asset bundling with hash verification |
| `web_engine` | 106 | Engine resolution (system/chromium), CEF config, value source tracking |
| `automation_cli` | 117 | CLI entry point for automation commands |
| `raw_manifest` | 109 | Raw ZON manifest types |

## FFI bindings

Generated automatically by `bindgen` from the existing C headers:

- **macOS**: `appkit_host.h` → 23 `extern fn` bindings → `appkit_host.m` compiled via `cc`
- **Linux**: `gtk_host.h` → 19 `extern fn` bindings → `gtk_host.c` compiled via `cc`
- **Windows**: header → 16 `extern fn` bindings → `webview2_host.cpp` compiled via `cc`

All `unsafe` FFI code is isolated inside `PlatformHost` trait implementations. Callers never write `unsafe`.

## Mobile embedding

The `embed` module exports the same C ABI as the Zig version:

```c
zero_native_app_create()
zero_native_app_destroy()
zero_native_app_start()
zero_native_app_stop()
zero_native_app_resize()
zero_native_app_touch()
zero_native_app_frame()
zero_native_app_set_asset_root()
zero_native_app_last_command_count()
```

iOS and Android host apps can link `libzero_native.a` and call through this interface, exactly as in the Zig version. Package scaffolding for both platforms generates the full host project (Swift view controller, Kotlin/JNI bridge) with embedded header.

## Key design decisions

| Decision | Rationale |
|----------|-----------|
| **`PlatformHost` trait as safe membrane** | All `unsafe` FFI code isolated inside trait impl blocks; callers never write `unsafe`. `Box<dyn PlatformHost>` in Runtime. |
| **Fat pointer splitting in RunState** | `handler: (usize, usize)` — data pointer and vtable stored separately via `transmute` to avoid Rust lifetime capture on `*mut dyn FnMut(Event)`. |
| **Runtime collects events before dispatch** | Avoids closure-over-self borrow conflict in the main loop. |
| **Manual `Module` Clone** | `context: Box<dyn Any>` can't implement Clone; manual impl replaces with `Box::new(())` on clone. |
| **`build.rs` framework linking** | `cargo:rustc-link-lib=framework=...` directives for AppKit/WebKit/CoreFoundation/UniformTypeIdentifiers so the binary links macOS frameworks. |
| **Flat module structure** | Zig's `*/root.zig` pattern maps to Rust's `src/*.rs` flat files; platform module is `src/platform/` directory. |
| **ZON parser for window state** | Custom parser for Zig Object Notation format matching the Zig implementation. |

## License

Apache-2.0
