# zero-native (Rust)

Build native desktop apps with web UI. Tiny binaries. Minimal memory. Instant rebuilds.

This is a Rust port of [zero-native](https://github.com/vercel-labs/zero-native), which was originally written in Zig. The C/ObjC/C++ platform host layer is carried over unchanged — compiled via `cc` and `bindgen` in `build.rs`.

## Architecture

```
┌─────────────────────────────────────┐
│  Rust runtime (bridge, security,    │
│  window mgmt, extensions, events)   │
│  ~2.8k lines of Rust                │
└──────────────┬──────────────────────┘
               │ extern "C" via bindgen
               │ (23 functions on macOS, 19 on Linux, 16 on Windows)
               │ + 2-3 callback function pointers
               ▼
┌───────────────────────────────────────┐
│  C/ObjC/C++ platform hosts            │
│  (carried over from the Zig project)  │
│  appkit_host.m  •  gtk_host.c         │
│  cef_host.mm    •  webview2_host.cpp  │
└───────────────────────────────────────┘
```

## Quick start

```bash
cargo build
cargo run --bin zero-native -- --version
cargo test
```

## Commands

| Command | Description |
|---------|-------------|
| `doctor` | Print platform diagnostics |
| `validate` | Validate app.zon manifest |
| `init` | Create a new app (stub) |
| `bundle-assets` | Bundle app assets (stub) |
| `package` | Create packaged artifact (stub) |
| `cef` | Manage Chromium Embedded Framework (stub) |
| `dev` | Run dev server + native shell (stub) |

## Modules

| Module | Zig source | Rust source | Status |
|--------|-----------|-------------|--------|
| `geometry` | 450 lines | 250 lines | ✅ Complete |
| `trace` | 300 lines | 170 lines | ✅ Complete |
| `assets` | 450 lines | 220 lines | ✅ Complete |
| `app_dirs` | 300 lines | 195 lines | ✅ Complete |
| `app_manifest` | 500 lines | 220 lines | ✅ Complete |
| `diagnostics` | 350 lines | 80 lines | ✅ Complete |
| `platform_info` | 350 lines | 85 lines | ✅ Complete |
| `security` | 50 lines | 70 lines | ✅ Complete |
| `bridge` | 400 lines | 370 lines | ✅ Complete (with proper JSON parser) |
| `platform` | 600 lines | 300 lines | ✅ Core types + NullPlatform |
| `runtime` | 1200 lines | 250 lines | ✅ Core event loop + window management |
| `extensions` | 200 lines | 125 lines | ✅ Complete |
| `automation` | 100 lines | 50 lines | ✅ Complete |
| `window_state` | 400 lines | 45 lines | ✅ Serialization |
| `json` | 200 lines | 115 lines | ✅ Complete |
| `js` | 60 lines | 50 lines | ✅ Complete |
| `frontend` | 50 lines | 40 lines | ✅ Complete |
| `embed` | 120 lines | 55 lines | ✅ C ABI exports |
| CLI tooling | 5000+ lines | 100 lines | 🚧 Stub only |

## FFI bindings

Generated automatically by `bindgen` from the existing C headers:

- **macOS**: `appkit_host.h` → 23 `extern fn` bindings → `appkit_host.m` compiled via `cc`
- **Linux**: `gtk_host.h` → 19 `extern fn` bindings → `gtk_host.c` compiled via `cc`
- **Windows**: header → 16 `extern fn` bindings → `webview2_host.cpp` compiled via `cc`

## Mobile embedding

The `embed` module exports the same C ABI as the Zig version:

```c
zero_native_app_create()
zero_native_app_destroy()
zero_native_app_start()
zero_native_app_stop()
```

iOS and Android host apps can link `libzero_native.a` and call through this interface, exactly as in the Zig version.

## License

Apache-2.0
