<!-- GSD:project-start source:PROJECT.md -->

## Project

**xtools**

Linux 桌面上给自己用的工具箱。一颗始终置顶的主悬浮球，点击后时间戳 / JSON / 翻译三颗功能球围绕主球弹出；再点某颗功能球，打开对应的独立 Rust 窗口。三个功能是独立窗口进程，主程序只负责球和唤起。

**Core Value:** 点击主球，功能球围绕它弹出；再点功能球，打开或聚焦对应独立窗口。这一条必须成立。

### Constraints

- **Tech stack**: Rust — 用户指定；主程序和三个功能都是 Rust 窗口程序
- **Process model**: 功能 = 独立进程窗口；主程序只负责悬浮球和启动/聚焦
- **Platform**: Linux 桌面，自用
- **v1 surface**: 三个写死入口，不做动态插件发现
- **UI**: 窗口风格必须一致，需要共享主题/控件，而不是三个窗口各画一套

<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->

## Technology Stack

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust (edition 2024) | rustc 1.97.1 present | Language / workspace | User-locked. Edition 2024 is available on this rustc. |
| Cargo workspace | — | Four binaries + two libs | Launcher + timestamp + json + translate are sibling processes; `xtools-ui` and `xtools-ipc` are shared. |
| **eframe** | **0.36.1** | Native window host (winit + wgpu) | `NativeOptions.viewport` is an egui `ViewportBuilder` with first-class `with_always_on_top()`, `with_transparent(true)`, `with_decorations(false)`, `with_mouse_passthrough(bool)`, `with_app_id` (Wayland), `with_window_type` (X11). Extra windows via viewports. Default features already enable **wayland + x11 + wgpu**. Pins `winit ^0.30.13` and `egui ^0.36.1`. |
| **egui** | **0.36.1** | Immediate-mode UI + custom paint | Orbital balls are circles + click hit-tests (`Painter::circle` + `Sense::click`), not native widgets. Tool UIs are forms. Shared widgets are plain functions — the cheapest consistent-theme story in Rust. |
| **winit** (via eframe) | **0.30.13** | Windowing | `Window::set_cursor_hittest` is what mouse-passthrough maps to. `focus_window` + `ActivationToken` are the raise/focus primitives. Do not take a second winit version. |
| std `UnixListener` + `XDG_RUNTIME_DIR` | std | Single-instance + raise | Bind `$XDG_RUNTIME_DIR/xtools-<tool>.sock`. Second process sends `Raise { token }` and exits. First process runs `ViewportCommand::Focus`. No abandoned lock crate. |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| **egui_extras** | **0.36.1** | Table / image / extra widgets | JSON output table or image icons. Same version as egui. |
| **jiff** | **0.2.35** | Unix s/ms ↔ datetime, RFC3339, custom formats | Timestamp tool only. `Timestamp::from_second` / `from_millisecond`, `as_second` / `as_millisecond`, `FromStr`/`Display` for RFC3339, `strftime` for custom copy formats. |
| **serde** | **1.0.229** | Serialize engine requests / config | All tools if they persist settings; required by serde_json. |
| **serde_json** | **1.0.151** | Format, minify, validate JSON | JSON tool. `Value` + `to_string_pretty` / `to_string`. `Error::line()` + `Error::column()` (1-based) mark the bad token in the editor. Do not add a second JSON crate. |
| **ureq** | **3.4.0** (`json`) | Sync HTTPS for pluggable translate | Translate process only. One POST, rustls, no tokio. Wrap behind a `TranslateEngine` trait (`fn translate(&self, text, src, dst) -> Result<String>`). Call off the egui thread (`std::thread` + channel) so the window stays live. |
| **thiserror** | **2.0.20** | Typed errors | Engine + parse errors shown in the UI. |
| **anyhow** | **1.0.104** | Binary-level error glue | `main` / spawn helpers only. |
| **dirs** | **6.0.0** | Config/data dirs | Optional later (`~/.config/xtools`). Not required for v1 hardcoded menu. |
| **arboard** | **3.6.1** | System clipboard | Only if a path is outside egui. Prefer `ctx.copy_text(...)` for one-click timestamp formats. |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| rustc / cargo 1.97.1 | Build | Already on the machine. `edition = "2024"`. |
| rust-analyzer | IDE | Workspace-aware. |
| `cargo clippy` / `cargo fmt` | Local quality | Do not add CI/packaging in v1 (out of scope). |
| `echo $XDG_SESSION_TYPE` | Session check | First launcher spike: confirm X11 vs Wayland before treating AlwaysOnTop as done. |

## Installation

# Workspace root

# then convert to a workspace; add sibling bins + libs

# Core GUI (all four binaries)

# Timestamp binary

# JSON binary

# Translate binary

# Optional

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| eframe/egui 0.36.1 | iced 0.14.0 | If you later want a retained Elm UI and accept more boilerplate for custom orbital paint. iced `window::Settings` does have `transparent`, `decorations: false`, `level: AlwaysOnTop`. `iced_layershell` 0.19.1 is the only iced-specific reason to switch: wlroots overlay protocol. Do not mix iced tools with an egui launcher. |
| eframe/egui 0.36.1 | slint 1.17.1 | If the three tools become large declarative UIs *and* you drop the custom overlay. Official `Window` has `always-on-top` and `no-frame`; it does **not** have a first-class transparent surface. Sharing `.slint` files is good; the orbital launcher is not Slint's strength. |
| eframe/egui 0.36.1 | gtk4 0.11.4 (+ gtk4-layer-shell 0.8.1) | If you are on a **wlroots** compositor (Sway/Hyprland) *and* AlwaysOnTop is ignored *and* you need a real layer-shell overlay + `GdkSurface::set_input_region` circle. GTK4 removed `keep_above`. Layer-shell does **not** work on GNOME Mutter. `GtkWindowExt::present` is the best raise API in this comparison. Heavier system deps; custom balls are Cairo, not widgets. |
| eframe/egui 0.36.1 | tauri 2.11.5 | Never for this product. WebView + WebKitGTK for a 72px ball and three form tools is the wrong process/weight. User asked for Rust windows. |
| ureq 3.4.0 | reqwest 0.13.4 (`blocking` + `json`) | If an engine needs HTTP/2, cookie jars, or streaming. Keep it behind the same `TranslateEngine` trait. Do not put `reqwest` async on the egui thread. |
| jiff 0.2.35 | chrono 0.4.45 | Only if you must copy-paste chrono examples. New code should use jiff. |
| Unix socket IPC | zbus 5.19.0 | If you later want `org.freedesktop.Application` desktop activation. Overkill for v1 personal use. |
| Unix socket IPC | single-instance 0.3.3 | Never — last crates.io release 2021-12-16. |

### Toolkit comparison (this product)

| Criterion | eframe/egui 0.36.1 | iced 0.14.0 | slint 1.17.1 | gtk4 0.11.4 | tauri 2.11.5 |
|-----------|--------------------|-------------|--------------|-------------|--------------|
| Always-on-top API | `with_always_on_top` / `WindowLevel` | `window::Level::AlwaysOnTop` | `always-on-top: true` | **removed**; X11 hint or layer-shell | WebView window flag |
| Transparent undecorated | first-class | first-class | no-frame yes; true alpha is a backend hack | `set_decorated(false)` + CSS/alpha | possible, heavy |
| Circular hit-test | `MousePassthrough` + paint, or small windows | custom canvas | poor fit | `set_input_region` (best Linux shape API) | DOM hacks |
| Orbital child widgets | extra viewports in one process | multi-window messages | extra Windows | extra GtkWindows | extra webviews |
| Shared theme crate | Visuals + fn widgets | theme + widget crates | shared `.slint` | GTK CSS | HTML/CSS |
| Focus existing | `ViewportCommand::Focus` + activation token | window command | `show()` | `present()` (best) | window API |
| Linux deps | wgpu/Vulkan or GL | wgpu | winit backend | libgtk-4 + (layer-shell) | WebKitGTK |
| Fit for this app | **Use this** | fallback | tools-only | overlay-only fallback | do not use |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| tauri / wry / dioxus-desktop / Electron | WebView for a floating ball + three form tools. Extra WebKitGTK, larger RAM, not a Rust-native window. | eframe 0.36.1 |
| Mixing GUI toolkits (egui launcher + gtk tools, etc.) | Breaks the "one look" requirement. Two theme systems, two font stacks. | One toolkit, `xtools-ui` crate |
| gtk3 / gdk3 / `gtk` 0.18 | Old bindings. No keep_above story worth reviving. | gtk4 0.11.4 only if abandoning egui entirely |
| `gtk_window_set_keep_above` | Does not exist in GTK4. | eframe AlwaysOnTop, or layer-shell on wlroots |
| gtk4-layer-shell on GNOME | Mutter does not implement `wlr-layer-shell`. | eframe AlwaysOnTop; accept compositor limits |
| One huge transparent overlay for the orbit | Click-through of the empty donut is the hard problem (`set_cursor_hittest` / X11 Shape). Once passthrough is on, you may not get move events to turn it off. | **Four small windows**: 72×72 main + three orbital viewports. Corners are tiny. |
| `single-instance` 0.3.3 / `fslock` as the raise mechanism | Lock file ≠ focus. `single-instance` is stale (2021). | Unix socket + `ViewportCommand::Focus` + `XDG_ACTIVATION_TOKEN` |
| xdotool / wmctrl / `_NET_ACTIVE_WINDOW` from a second process on Wayland | Wayland compositors ignore X11 focus-steal. Tokens expire. | Pass activation token from the click (launcher) into the tool |
| chrono as the timestamp crate | New project; jiff is the current Temporal-style crate with `from_second` / `from_millisecond`. | jiff 0.2.35 |
| ariadne / codespan-reporting | Terminal span printers. The JSON UI is a window. | `serde_json::Error::line()` / `column()` → select that range in `TextEdit` |
| json-spanned-value 0.2.2 / justjson | Stale or extra parser. serde_json already reports line/col. | serde_json 1.0.151 |
| tokio in every binary | Launcher and timestamp/json do not need an async runtime. | ureq on a worker thread in translate only |
| reqwest async on the egui update thread | Will stall the ball/window. | Worker thread + channel; or reqwest `blocking` if you swap HTTP crates |
| Plugin `libloading` / directory scan | Explicitly out of scope. | Hardcoded three `Command::new` paths |
| Raw winit + wgpu without egui | You would reimplement widgets, text, IME, clipboard. | eframe |
| iced < 0.13 / egui < 0.28 | Old window APIs; viewport/level support is what we need. | 0.14 / 0.36.1 |

## Stack Patterns by Variant

- Root viewport: ~72×72, `with_decorations(false)`, `with_transparent(true)`, `with_always_on_top()`, `with_resizable(false)`, `with_app_id("dev.xtools.launcher")`, X11 `with_window_type(X11WindowType::Utility)`.
- `persist_window: false` so the ball does not restore as a huge rectangle.
- Implement `App::clear_color` as fully transparent. Do not wrap the ball in an opaque `CentralPanel`.
- Drag: `ViewportCommand::StartDrag` on pointer-down inside the circle.
- Click main ball → spawn/show three **deferred viewports** (not processes) at 120° around the main center; click again → hide them.
- Each orbital viewport is another small undecorated always-on-top window. Moving the main ball updates `ViewportCommand::OuterPosition` on the children.
- Hit-testing: treat the window as a circle (`pos.distance(center) <= radius`). Optional: `ViewportCommand::MousePassthrough(outside)`. Prefer small windows over a large donut.
- Normal decorated eframe window, **not** always-on-top.
- `with_app_id("dev.xtools.timestamp")` (etc.) so Wayland grouping and the socket name match.
- On startup: try bind the Unix socket. If bind fails, write `Raise` + inherited `XDG_ACTIVATION_TOKEN` to the socket and exit 0.
- On `Raise`: `ctx.send_viewport_cmd(ViewportCommand::Focus)`.
- Launcher spawn: `Command::new(bin)` with the token from the click event copied into the child env (`XDG_ACTIVATION_TOKEN`). Do not spawn if you can first connect and raise.
- Export one `Visuals` (dark, one accent, one font size scale, same margin/rounding).
- Export `apply_theme(ctx)`, `tool_frame()`, `copy_button(ui, label, text)`, `ball(ui, label) -> Response`.
- All four binaries call `apply_theme` first thing in `update`.
- No runtime theme server. No CSS. No `.slint` compile step.
- AlwaysOnTop + `X11WindowType::Utility` is the expected path. Skip-taskbar via `NativeOptions.window_builder` / winit X11 ext if the ball appears in the taskbar.
- AlwaysOnTop often works. If the ball sinks, *then* evaluate gtk4-layer-shell **only for the launcher**, or `iced_layershell`. Do not rewrite the tools.
- AlwaysOnTop may be ignored. Layer-shell is not available. Personal workaround: XWayland (`WAYLAND_DISPLAY=` unset for the launcher) or live with normal stacking. Do not write a GNOME Shell extension for v1.
- Switch that binary to eframe `glow` renderer. Do not change toolkit.

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| eframe 0.36.1 | egui 0.36.1, egui_extras 0.36.1, winit 0.30.13 | Same release train. Do not mix 0.31 widgets with 0.36 eframe. |
| eframe 0.36.1 default | wgpu (not glow) | glow is optional. Default Linux features: `wayland` + `x11`. |
| slint 1.17.1 | winit 0.30.13 | Same winit major; still do not link slint and eframe in one binary. |
| iced 0.14.0 | winit 0.30 | Same. |
| gtk4 0.11.4 | glib/gio 0.22, gtk4-layer-shell 0.8.1 | Needs system libgtk-4. Layer-shell needs libgtk4-layer-shell and a wlroots compositor. |
| ureq 3.4.0 | serde_json 1.0.x | `json` feature. API is not ureq 2. |
| reqwest 0.13.4 | serde_json 1.0.x | `json` / `form` / `blocking` are optional now — enable what you use. Do not mix 0.12 and 0.13. |
| jiff 0.2.35 | — | 0.2 line. `from_second` / `from_millisecond` return `Result`. |
| thiserror 2.0.20 | — | Not thiserror 1. |
| serde 1.0.229 | serde_json 1.0.151 | Semver 1.x. |

## Sources

- crates.io API (`curl`, 2026-08-19) — max stable versions: eframe/egui/egui_extras 0.36.1, iced 0.14.0, slint 1.17.1, gtk4 0.11.4, tauri 2.11.5, winit 0.30.13, jiff 0.2.35, serde_json 1.0.151, serde 1.0.229, ureq 3.4.0, reqwest 0.13.4, gtk4-layer-shell 0.8.1, iced_layershell 0.19.1, arboard 3.6.1, thiserror 2.0.20, anyhow 1.0.104, dirs 6.0.0, zbus 5.19.0, single-instance 0.3.3 (2021-12-16), json-spanned-value 0.2.2 (2020-10-10)
- https://docs.rs/eframe/0.36.1/eframe/struct.NativeOptions.html — viewport, persist_window, window_builder, default wgpu
- https://docs.rs/egui/0.36.1/egui/viewport/struct.ViewportBuilder.html — always_on_top, transparent, mouse_passthrough, app_id, window_type
- https://docs.rs/egui/0.36.1/egui/viewport/enum.ViewportCommand.html — MousePassthrough, Focus, StartDrag, OuterPosition
- https://docs.rs/egui/0.36.1/egui/viewport/enum.X11WindowType.html — Utility / Notification / Dock
- https://github.com/emilk/egui/blob/0.36.1/crates/eframe/Cargo.toml — default features wayland+x11+wgpu; winit ^0.30.13
- https://docs.rs/winit/0.30.13/winit/window/struct.Window.html — set_cursor_hittest, focus_window, set_window_level
- https://docs.rs/winit/0.30.13/winit/window/struct.WindowAttributes.html — transparent, decorations, window_level
- https://docs.rs/iced/0.14.0/iced/window/struct.Settings.html — transparent, decorations, level, blur
- https://docs.slint.dev/latest/docs/slint/reference/window/window/ — always-on-top, no-frame (no transparent property)
- https://docs.rs/gtk4/0.11.4/gtk4/prelude/trait.GtkWindowExt.html — present, set_decorated, set_hide_on_close
- https://docs.rs/gtk4-layer-shell/0.8.1/gtk4_layer_shell/trait.LayerShell.html — init_layer_shell, set_layer
- https://docs.gtk.org/gdk4/method.Surface.set_input_region.html — shaped hit-testing
- https://docs.rs/serde_json/1.0.151/serde_json/struct.Error.html — line(), column()
- https://docs.rs/jiff/0.2.35/jiff/struct.Timestamp.html — from_second, from_millisecond, RFC3339 Display
- https://docs.rs/ureq/3.4.0/ureq/ — sync rustls Agent, optional json
- https://docs.rs/reqwest/0.13.4/reqwest/ — async + blocking, optional json
- https://docs.rs/tauri/2.11.5/tauri/ — WebView runtime (rejected)
- Wayland xdg-activation / focus-steal — MEDIUM (protocol + winit ActivationToken; compositor-dependent)

<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->

## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->

## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->

## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, `.github/skills/`, or `.codex/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->

## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:

- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->

<!-- GSD:profile-start -->

## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
