---
phase: quick
id: 260822-uts
slug: fix-timestamp-window-cjk-tofu-fonts-bord
status: active
created: 2026-08-22
---

# Fix timestamp window chrome

User cannot read or use the timestamp window:

1. CJK glyphs render as tofu (egui default Ubuntu/Hack have no Han).
2. System decorations look cheap; user wants frameless.
3. Window still appears on the KWin taskbar (`with_window_type(Utility)` is X11-only and ignored on native Wayland).

## Tasks

### 1. Shared chrome: CJK fonts + frameless shell

Files:
- `crates/xtools-ui/src/chrome.rs`

- Load system Source Han Sans (fallback list) once via `Context::set_fonts`. Put CJK first on Proportional, last on Monospace.
- Replace the in-content title strip with a frameless `tool_shell`: rounded card, drag title, painted close, 20px pad.
- Keep copy / 现在 / field / error widgets; tighten rounding to match the card.

### 2. Timestamp window: no chrome, skip taskbar

Files:
- `crates/xtools-time/src/main.rs`
- `crates/xtools-time/src/app.rs`
- `crates/xtools-time/src/skip_taskbar.rs` (new)
- `crates/xtools-time/Cargo.toml`

- `with_decorations(false)`, `with_transparent(true)`, `clear_color` fully transparent.
- Before eframe init: if `DISPLAY` is set, drop `WAYLAND_DISPLAY` so Utility + EWMH apply (session is Wayland, XWayland is `:1`).
- After map: set `_NET_WM_STATE_SKIP_TASKBAR` on this pid's X11 windows.
- Wire UI through `tool_shell`. Install fonts in `CreationContext`.

## Done when

- Chinese labels (`时间戳` / `复制` / `现在` / errors) render as real glyphs.
- Window has no compositor title bar; title row drags; × closes.
- Timestamp does not appear on the KWin taskbar.
- `cargo check -p xtools-time` passes.

## Out of scope

- JSON / translate windows
- RFC3339 / custom-format copy
- Updating 02-UI-SPEC.md (user overrode D-14 decorations)
