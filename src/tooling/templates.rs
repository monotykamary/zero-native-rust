use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frontend {
    Next,
    Vite,
    React,
    Svelte,
    Vue,
}

impl Frontend {
    pub fn parse(value: &str) -> Option<Frontend> {
        match value {
            "next" => Some(Frontend::Next),
            "vite" => Some(Frontend::Vite),
            "react" => Some(Frontend::React),
            "svelte" => Some(Frontend::Svelte),
            "vue" => Some(Frontend::Vue),
            _ => None,
        }
    }

    pub fn dist_dir(self) -> &'static str {
        match self {
            Frontend::Next => "frontend/out",
            Frontend::Vite | Frontend::React | Frontend::Svelte | Frontend::Vue => "frontend/dist",
        }
    }

    pub fn dev_port(self) -> &'static str {
        match self {
            Frontend::Next => "3000",
            Frontend::Vite | Frontend::React | Frontend::Svelte | Frontend::Vue => "5173",
        }
    }

    pub fn dev_url(self) -> String {
        format!("http://127.0.0.1:{}/", self.dev_port())
    }
}

#[derive(Debug, Clone)]
pub struct InitOptions {
    pub app_name: String,
    pub framework_path: String,
    pub frontend: Frontend,
}

impl Default for InitOptions {
    fn default() -> Self {
        Self {
            app_name: "zero-native-app".into(),
            framework_path: ".".into(),
            frontend: Frontend::Vite,
        }
    }
}

#[derive(Debug)]
pub struct TemplateNames {
    pub package_name: String,
    pub module_name: String,
    pub display_name: String,
    pub app_id: String,
}

impl TemplateNames {
    pub fn init(app_name: &str) -> Self {
        let package_name = normalize_package_name(app_name);
        let module_name = normalize_module_name(&package_name);
        let display_name = make_display_name(&package_name);
        let app_id = format!("dev.zero_native.{}", package_name);
        Self {
            package_name,
            module_name,
            display_name,
            app_id,
        }
    }
}

pub fn write_default_app(destination: &str, options: &InitOptions) -> Result<(), String> {
    let names = TemplateNames::init(&options.app_name);
    let framework_path = default_framework_path(destination, &options.framework_path);

    let dest = Path::new(destination);
    fs::create_dir_all(dest.join("src")).map_err(|e| e.to_string())?;
    fs::create_dir_all(dest.join("assets")).map_err(|e| e.to_string())?;

    fs::write(dest.join("build.zig"), build_zig(&names, &framework_path, options.frontend))
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("build.zig.zon"), build_zon(&names))
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("src/main.zig"), main_zig(&names, options.frontend))
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("src/runner.zig"), runner_zig())
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("app.zon"), app_zon(&names, options.frontend))
        .map_err(|e| e.to_string())?;

    let icon_bytes = fs::read("assets/icon.icns").unwrap_or_else(|_| FALLBACK_ICON_ICNS.to_vec());
    fs::write(dest.join("assets/icon.icns"), &icon_bytes)
        .map_err(|e| e.to_string())?;

    fs::write(dest.join("README.md"), readme_md(&names, &framework_path, options.frontend))
        .map_err(|e| e.to_string())?;

    write_frontend_files(dest, &names, options.frontend)?;

    Ok(())
}

const FALLBACK_ICON_ICNS: &[u8] = b"icns\x00\x00\x00\x08";

fn write_frontend_files(
    dest: &Path,
    names: &TemplateNames,
    frontend: Frontend,
) -> Result<(), String> {
    match frontend {
        Frontend::Next => write_next_frontend(dest, names),
        Frontend::Vite => write_vite_frontend(dest, names),
        Frontend::React => write_react_frontend(dest, names),
        Frontend::Svelte => write_svelte_frontend(dest, names),
        Frontend::Vue => write_vue_frontend(dest, names),
    }
}

fn write_next_frontend(dest: &Path, names: &TemplateNames) -> Result<(), String> {
    fs::create_dir_all(dest.join("frontend/app")).map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/package.json"), next_package_json(names))
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/next.config.js"), next_config())
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/tsconfig.json"), next_tsconfig())
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/app/layout.tsx"), next_layout(names))
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/app/page.tsx"), next_page(names))
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/app/globals.css"), frontend_styles_css())
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn write_vite_frontend(dest: &Path, names: &TemplateNames) -> Result<(), String> {
    fs::create_dir_all(dest.join("frontend/src")).map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/package.json"), vite_package_json(names))
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/index.html"), vite_index_html(names))
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/src/main.js"), vite_main_js())
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/src/styles.css"), frontend_styles_css())
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn write_react_frontend(dest: &Path, names: &TemplateNames) -> Result<(), String> {
    fs::create_dir_all(dest.join("frontend/src")).map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/package.json"), react_package_json(names))
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/vite.config.js"), react_vite_config())
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/index.html"), react_index_html(names))
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/src/main.tsx"), react_main_tsx())
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/src/App.tsx"), react_app_tsx(names))
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/src/index.css"), frontend_styles_css())
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn write_svelte_frontend(dest: &Path, names: &TemplateNames) -> Result<(), String> {
    fs::create_dir_all(dest.join("frontend/src")).map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/package.json"), svelte_package_json(names))
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/svelte.config.js"), svelte_config())
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/vite.config.js"), svelte_vite_config())
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/index.html"), svelte_index_html(names))
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/src/main.js"), svelte_main_js())
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/src/App.svelte"), svelte_app_component())
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/src/app.css"), frontend_styles_css())
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn write_vue_frontend(dest: &Path, names: &TemplateNames) -> Result<(), String> {
    fs::create_dir_all(dest.join("frontend/src")).map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/package.json"), vue_package_json(names))
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/vite.config.js"), vue_vite_config())
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/index.html"), vue_index_html(names))
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/src/main.js"), vue_main_js())
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/src/App.vue"), vue_app_component())
        .map_err(|e| e.to_string())?;
    fs::write(dest.join("frontend/src/style.css"), frontend_styles_css())
        .map_err(|e| e.to_string())?;
    Ok(())
}

// Template content generation functions

fn build_zig(names: &TemplateNames, framework_path: &str, _frontend: Frontend) -> String {
    format!(
        r#"const std = @import("std");

const default_zero_native_path = "{framework_path}";
const app_exe_name = "{package_name}";

pub fn build(b: *std.Build) void {{
    const target = b.standardTargetOptions(.{{}});
    const optimize = b.standardOptimizeOption(.{{}});
    const platform_option = b.option(PlatformOption, "platform", "Desktop backend") orelse .auto;
    const web_engine_override = b.option(WebEngineOption, "web-engine", "Override web engine");
    const cef_dir_override = b.option([]const u8, "cef-dir", "Override CEF directory");
    const cef_auto_install_override = b.option(bool, "cef-auto-install", "Auto-install CEF");
    const zero_native_path = b.option([]const u8, "zero-native-path", "Path to zero-native") orelse default_zero_native_path;

    const selected_platform: PlatformOption = switch (platform_option) {{
        .auto => if (target.result.os.tag == .macos) .macos else if (target.result.os.tag == .linux) .linux else if (target.result.os.tag == .windows) .windows else .@"null",
        else => platform_option,
    }};

    const zero_native_mod = zeroNativeModule(b, target, optimize, zero_native_path);
    const options = b.addOptions();
    options.addOption([]const u8, "platform", switch (selected_platform) {{
        .auto => unreachable,
        .@"null" => "null",
        .macos => "macos",
        .linux => "linux",
        .windows => "windows",
    }});
    options.addOption([]const u8, "web_engine", "system");
    const options_mod = options.createModule();

    const runner_mod = localModule(b, target, optimize, "src/runner.zig");
    runner_mod.addImport("zero-native", zero_native_mod);
    runner_mod.addImport("build_options", options_mod);

    const app_mod = localModule(b, target, optimize, "src/main.zig");
    app_mod.addImport("zero-native", zero_native_mod);
    app_mod.addImport("runner", runner_mod);
    const exe = b.addExecutable(.{{ .name = app_exe_name, .root_module = app_mod }});
    b.installArtifact(exe);

    const run = b.addRunArtifact(exe);
    const run_step = b.step("run", "Run the app");
    run_step.dependOn(&run.step);

    const tests = b.addTest(.{{ .root_module = app_mod }});
    const test_step = b.step("test", "Run tests");
    test_step.dependOn(&b.addRunArtifact(tests).step);
}}

const PlatformOption = enum {{ auto, @"null", macos, linux, windows }};
const WebEngineOption = enum {{ system, chromium }};

fn localModule(b: *std.Build, target: std.Build.ResolvedTarget, optimize: std.builtin.OptimizeMode, path: []const u8) *std.Build.Module {{
    return b.createModule(.{{ .root_source_file = b.path(path), .target = target, .optimize = optimize }});
}}

fn zeroNativeModule(b: *std.Build, target: std.Build.ResolvedTarget, optimize: std.builtin.OptimizeMode, zero_native_path: []const u8) *std.Build.Module {{
    return b.createModule(.{{ .root_source_file = b.path(b.pathJoin(&.{{ zero_native_path, "src/root.zig" }})), .target = target, .optimize = optimize }});
}}
"#,
        framework_path = framework_path,
        package_name = names.package_name,
    )
}

fn build_zon(names: &TemplateNames) -> String {
    format!(
        r#".{{
    .name = .{module_name},
    .version = "0.1.0",
    .minimum_zig_version = "0.16.0",
    .dependencies = .{{}},
    .paths = .{{ "build.zig", "build.zig.zon", "src", "assets", "frontend", "app.zon", "README.md" }},
}}
"#,
        module_name = names.module_name,
    )
}

fn main_zig(names: &TemplateNames, frontend: Frontend) -> String {
    format!(
        r#"const std = @import("std");
const runner = @import("runner");
const zero_native = @import("zero-native");

pub const panic = std.debug.FullPanic(zero_native.debug.capturePanic);

const App = struct {{
    env_map: *std.process.Environ.Map,

    fn app(self: *@This()) zero_native.App {{
        return .{{
            .context = self,
            .name = "{package_name}",
            .source = zero_native.frontend.productionSource(.{{ .dist = "{dist_dir}" }}),
            .source_fn = source,
        }};
    }}

    fn source(context: *anyopaque) anyerror!zero_native.WebViewSource {{
        const self: *@This() = @ptrCast(@alignCast(context));
        return zero_native.frontend.sourceFromEnv(self.env_map, .{{ .dist = "{dist_dir}", .entry = "index.html" }});
    }}
}};

const dev_origins = [_][]const u8{{ "zero://app", "zero://inline", "{dev_origin}" }};

pub fn main(init: std.process.Init) !void {{
    var app = App{{ .env_map = init.environ_map }};
    try runner.runWithOptions(app.app(), .{{
        .app_name = "{display_name}",
        .window_title = "{display_name}",
        .bundle_id = "{app_id}",
        .icon_path = "assets/icon.icns",
        .security = .{{ .navigation = .{{ .allowed_origins = &dev_origins }} }},
    }}, init);
}}
"#,
        package_name = names.package_name,
        dist_dir = frontend.dist_dir(),
        dev_origin = format!("http://127.0.0.1:{}", frontend.dev_port()),
        display_name = names.display_name,
        app_id = names.app_id,
    )
}

fn runner_zig() -> &'static str {
    r#"const std = @import("std");
const build_options = @import("build_options");
const zero_native = @import("zero-native");

pub const RunOptions = struct {
    app_name: []const u8,
    window_title: []const u8 = "",
    bundle_id: []const u8,
    icon_path: []const u8 = "assets/icon.icns",
    security: zero_native.SecurityPolicy = .{},
};

pub fn runWithOptions(app: zero_native.App, options: RunOptions, init: std.process.Init) !void {
    _ = app;
    _ = options;
    _ = init;
}
"#
}

fn app_zon(names: &TemplateNames, frontend: Frontend) -> String {
    let dev_command = if frontend == Frontend::Next {
        format!(r#"            .command = .{{ "npm", "--prefix", "frontend", "run", "dev" }},"#)
    } else {
        format!(r#"            .command = .{{ "npm", "--prefix", "frontend", "run", "dev", "--", "--host", "127.0.0.1" }},"#)
    };
    format!(
        r#".{{
    .id = "{app_id}",
    .name = "{package_name}",
    .display_name = "{display_name}",
    .version = "0.1.0",
    .icons = .{{ "assets/icon.icns" }},
    .platforms = .{{ "macos", "linux" }},
    .permissions = .{{}},
    .capabilities = .{{ "webview" }},
    .frontend = .{{
        .dist = "{dist_dir}",
        .entry = "index.html",
        .spa_fallback = true,
        .dev = .{{
            .url = "{dev_url}",
{dev_command}
            .ready_path = "/",
            .timeout_ms = 30000,
        }},
    }},
    .security = .{{
        .navigation = .{{
            .allowed_origins = .{{ "zero://app", "zero://inline", "http://127.0.0.1:{dev_port}" }},
            .external_links = .{{ .action = "deny" }},
        }},
    }},
    .web_engine = "system",
    .cef = .{{ .dir = "third_party/cef/macos", .auto_install = false }},
    .windows = .{{
        .{{ .label = "main", .title = "{display_name}", .width = 720, .height = 480, .restore_state = true }},
    }},
}}
"#,
        app_id = names.app_id,
        package_name = names.package_name,
        display_name = names.display_name,
        dist_dir = frontend.dist_dir(),
        dev_url = frontend.dev_url(),
        dev_port = frontend.dev_port(),
        dev_command = dev_command,
    )
}

fn readme_md(names: &TemplateNames, framework_path: &str, frontend: Frontend) -> String {
    format!(
        r#"# {display_name}

A minimal zero-native desktop app with a web frontend.

## Setup

```sh
npm install --prefix frontend
```

The generated build defaults to this zero-native framework path:

```text
{framework_path}
```

## Commands

```sh
zig build dev
zig build run
zig build test
zig build package
zero-native doctor --manifest app.zon
```

Frontend:
- Type: {frontend_tag}
- Production assets: `{dist_dir}`
- Dev URL: `{dev_url}`
"#,
        display_name = names.display_name,
        framework_path = framework_path,
        frontend_tag = match frontend {
            Frontend::Next => "next",
            Frontend::Vite => "vite",
            Frontend::React => "react",
            Frontend::Svelte => "svelte",
            Frontend::Vue => "vue",
        },
        dist_dir = frontend.dist_dir(),
        dev_url = frontend.dev_url(),
    )
}

fn next_package_json(names: &TemplateNames) -> String {
    format!(
        r#"{{
  "name": "{package_name}",
  "private": true,
  "version": "0.1.0",
  "scripts": {{
    "dev": "next dev",
    "build": "next build",
    "start": "next start"
  }},
  "dependencies": {{
    "next": "^16.2.6",
    "react": "^19.2.6",
    "react-dom": "^19.2.6"
  }},
  "devDependencies": {{
    "@types/node": "^25.6.2",
    "@types/react": "^19.2.14",
    "@types/react-dom": "^19.2.3",
    "typescript": "^6.0.3"
  }}
}}
"#,
        package_name = names.package_name,
    )
}

fn next_config() -> &'static str {
    r#"/** @type {import('next').NextConfig} */
const nextConfig = {
  output: "export",
};

module.exports = nextConfig;
"#
}

fn next_tsconfig() -> &'static str {
    r#"{
  "compilerOptions": {
    "target": "ES2017",
    "lib": ["dom", "dom.iterable", "esnext"],
    "allowJs": true,
    "skipLibCheck": true,
    "strict": true,
    "noEmit": true,
    "esModuleInterop": true,
    "module": "esnext",
    "moduleResolution": "bundler",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "jsx": "react-jsx",
    "incremental": true,
    "plugins": [{ "name": "next" }],
    "paths": { "@/*": ["./app/*"] }
  },
  "include": ["next-env.d.ts", "**/*.ts", "**/*.tsx"],
  "exclude": ["node_modules"]
}
"#
}

fn next_layout(names: &TemplateNames) -> String {
    format!(
        r#"import "./globals.css";

export const metadata = {{
  title: "{display_name}",
}};

export default function RootLayout({{ children }}: {{ children: React.ReactNode }}) {{
  return (
    <html lang="en">
      <body>{{children}}</body>
    </html>
  );
}}
"#,
        display_name = names.display_name,
    )
}

fn next_page(names: &TemplateNames) -> String {
    format!(
        r#""use client";

import {{ useEffect, useState }} from "react";

export default function Home() {{
  const [bridge, setBridge] = useState("checking...");

  useEffect(() => {{
    setBridge((window as any).zero ? "available" : "not enabled");
  }}, []);

  return (
    <main>
      <p className="eyebrow">zero-native + Next.js</p>
      <h1>{display_name}</h1>
      <p className="lede">A Next.js frontend running inside the system WebView.</p>
      <div className="card">
        <span>Native bridge</span>
        <strong>{{bridge}}</strong>
      </div>
    </main>
  );
}}
"#,
        display_name = names.display_name,
    )
}

fn vite_package_json(names: &TemplateNames) -> String {
    format!(
        r#"{{
  "name": "{package_name}",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {{
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  }},
  "devDependencies": {{
    "vite": "^8.0.11"
  }}
}}
"#,
        package_name = names.package_name,
    )
}

fn vite_index_html(names: &TemplateNames) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{display_name}</title>
  </head>
  <body>
    <main id="app">
      <p class="eyebrow">zero-native + Vite</p>
      <h1>{display_name}</h1>
      <p class="lede">A minimal web frontend running inside the system WebView.</p>
      <div class="card">
        <span>Native bridge</span>
        <strong id="bridge-status">checking...</strong>
      </div>
    </main>
    <script type="module" src="/src/main.js"></script>
  </body>
</html>
"#,
        display_name = names.display_name,
    )
}

fn vite_main_js() -> String {
    r##"import "./styles.css";

const bridgeStatus = document.querySelector("#bridge-status");
const hasBridge = typeof window !== "undefined" && Boolean(window.zero);

bridgeStatus.textContent = hasBridge ? "available" : "not enabled";
bridgeStatus.dataset.ready = "true";
"##.to_string()
}

fn react_package_json(names: &TemplateNames) -> String {
    format!(
        r#"{{
  "name": "{package_name}",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {{
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  }},
  "dependencies": {{
    "react": "^19.2.6",
    "react-dom": "^19.2.6"
  }},
  "devDependencies": {{
    "@types/react": "^19.2.14",
    "@types/react-dom": "^19.2.3",
    "@vitejs/plugin-react": "^6.0.1",
    "vite": "^8.0.11"
  }}
}}
"#,
        package_name = names.package_name,
    )
}

fn react_vite_config() -> &'static str {
    r#"import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
});
"#
}

fn react_index_html(names: &TemplateNames) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{display_name}</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
"#,
        display_name = names.display_name,
    )
}

fn react_main_tsx() -> &'static str {
    r#"import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import "./index.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>
);
"#
}

fn react_app_tsx(names: &TemplateNames) -> String {
    format!(
        r#"import {{ useEffect, useState }} from "react";

export default function App() {{
  const [bridge, setBridge] = useState("checking...");

  useEffect(() => {{
    setBridge((window as any).zero ? "available" : "not enabled");
  }}, []);

  return (
    <main>
      <p className="eyebrow">zero-native + React</p>
      <h1>{display_name}</h1>
      <p className="lede">A React frontend running inside the system WebView.</p>
      <div className="card">
        <span>Native bridge</span>
        <strong>{{bridge}}</strong>
      </div>
    </main>
  );
}}
"#,
        display_name = names.display_name,
    )
}

fn svelte_package_json(names: &TemplateNames) -> String {
    format!(
        r#"{{
  "name": "{package_name}",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {{
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  }},
  "dependencies": {{
    "svelte": "^5.55.5"
  }},
  "devDependencies": {{
    "@sveltejs/vite-plugin-svelte": "^7.1.2",
    "vite": "^8.0.11"
  }}
}}
"#,
        package_name = names.package_name,
    )
}

fn svelte_vite_config() -> &'static str {
    r#"import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
});
"#
}

fn svelte_config() -> &'static str {
    "export default {};\n"
}

fn svelte_index_html(names: &TemplateNames) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{display_name}</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.js"></script>
  </body>
</html>
"#,
        display_name = names.display_name,
    )
}

fn svelte_main_js() -> &'static str {
    r#"import App from "./App.svelte";
import "./app.css";

const app = new App({ target: document.getElementById("app") });

export default app;
"#
}

fn svelte_app_component() -> &'static str {
    r#"<script>
  import { onMount } from "svelte";

  let bridge = $state("checking...");

  onMount(() => {
    bridge = window.zero ? "available" : "not enabled";
  });
</script>

<main>
  <p class="eyebrow">zero-native + Svelte</p>
  <h1>App</h1>
  <p class="lede">A Svelte frontend running inside the system WebView.</p>
  <div class="card">
    <span>Native bridge</span>
    <strong>{bridge}</strong>
  </div>
</main>
"#
}

fn vue_package_json(names: &TemplateNames) -> String {
    format!(
        r#"{{
  "name": "{package_name}",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {{
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  }},
  "dependencies": {{
    "vue": "^3.5.34"
  }},
  "devDependencies": {{
    "@vitejs/plugin-vue": "^6.0.6",
    "vite": "^8.0.11"
  }}
}}
"#,
        package_name = names.package_name,
    )
}

fn vue_vite_config() -> &'static str {
    r#"import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  plugins: [vue()],
});
"#
}

fn vue_index_html(names: &TemplateNames) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{display_name}</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.js"></script>
  </body>
</html>
"#,
        display_name = names.display_name,
    )
}

fn vue_main_js() -> String {
    r##"import { createApp } from "vue";
import App from "./App.vue";
import "./style.css";

createApp(App).mount("#app");
"##.to_string()
}

fn vue_app_component() -> &'static str {
    r#"<script setup>
import { ref, onMounted } from "vue";

const bridge = ref("checking...");

onMounted(() => {
  bridge.value = window.zero ? "available" : "not enabled";
});
</script>

<template>
  <main>
    <p class="eyebrow">zero-native + Vue</p>
    <h1>App</h1>
    <p class="lede">A Vue frontend running inside the system WebView.</p>
    <div class="card">
      <span>Native bridge</span>
      <strong>{{ bridge }}</strong>
    </div>
  </main>
</template>
"#
}

fn frontend_styles_css() -> &'static str {
    r#":root {
  color: #0f172a;
  background: #f8fafc;
  font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}

body {
  min-width: 320px;
  min-height: 100vh;
  margin: 0;
  display: grid;
  place-items: center;
}

main {
  width: min(560px, calc(100vw - 48px));
  padding: 32px;
  border-radius: 24px;
  background: white;
  box-shadow: 0 24px 60px rgba(15, 23, 42, 0.14);
}

h1 {
  margin: 0 0 12px;
  font-size: clamp(2rem, 8vw, 4rem);
  line-height: 1;
}

.eyebrow {
  margin: 0 0 12px;
  color: #2563eb;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.lede {
  margin: 0 0 24px;
  color: #475569;
  line-height: 1.6;
}

.card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 16px;
  border: 1px solid #e2e8f0;
  border-radius: 16px;
  background: #f8fafc;
}
"#
}

fn normalize_package_name(value: &str) -> String {
    let mut result = String::new();
    let mut last_separator = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            result.push(ch.to_ascii_lowercase());
            last_separator = false;
        } else if !last_separator && !result.is_empty() {
            result.push('-');
            last_separator = true;
        }
    }
    if result.ends_with('-') {
        result.pop();
    }
    if result.is_empty() {
        result = "zero-native-app".into();
    }
    result
}

fn normalize_module_name(value: &str) -> String {
    const MAX_LEN: usize = 32;
    let mut result = String::new();
    if value.is_empty() || value.as_bytes()[0].is_ascii_digit() {
        result.push_str("app_");
    }
    for ch in value.chars() {
        if result.len() >= MAX_LEN {
            break;
        }
        if ch.is_ascii_alphanumeric() {
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push('_');
        }
    }
    result
}

fn make_display_name(value: &str) -> String {
    let mut result = String::new();
    let mut start_word = true;
    for ch in value.chars() {
        if ch == '-' {
            if !result.is_empty() && !result.ends_with(' ') {
                result.push(' ');
            }
            start_word = true;
            continue;
        }
        if start_word && ch.is_ascii_alphabetic() {
            result.push(ch.to_ascii_uppercase());
        } else {
            result.push(ch);
        }
        start_word = false;
    }
    if result.is_empty() {
        result = "zero-native app".into();
    }
    result
}

fn default_framework_path(destination: &str, framework_path: &str) -> String {
    if std::path::Path::new(framework_path).is_absolute() {
        return framework_path.to_string();
    }
    if std::path::Path::new(destination).is_absolute() {
        if let Ok(cwd) = std::env::current_dir() {
            return cwd.join(framework_path).to_string_lossy().to_string();
        }
    }
    let mut out = String::new();
    for part in destination.split(&['/', '\\']) {
        if part == "." || part.is_empty() {
            continue;
        }
        if part == ".." {
            continue;
        }
        if !out.is_empty() {
            out.push('/');
        }
        out.push_str("..");
    }
    for part in framework_path.split(&['/', '\\']) {
        if part == "." || part.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push('/');
        }
        out.push_str(part);
    }
    if out.is_empty() {
        out.push('.');
    }
    out
}
