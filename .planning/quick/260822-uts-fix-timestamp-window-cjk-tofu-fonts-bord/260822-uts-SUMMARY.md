---
id: 260822-uts
status: complete
date: 2026-08-22
commit: 0b53e23
---

# Quick 260822-uts — timestamp chrome

CJK tofu, system decorations, and KWin taskbar entry are gone.

## Done

- `install_fonts` loads `/usr/share/fonts/truetype/SourceHanSansCN-Regular.otf` first on Proportional.
- Frameless 440×400 card: drag title, painted close, transparent clear color.
- Prefer X11 when `DISPLAY` is set; Utility + `_NET_WM_STATE_SKIP_TASKBAR` on this pid.

## Evidence

- `cargo test -p xtools-ui --features egui-chrome` — `finds_system_cjk_face` pass
- Live window `0x2000003`: `_NET_WM_WINDOW_TYPE_UTILITY`, `_NET_WM_STATE_SKIP_TASKBAR`
- Screenshot `/tmp/xtools-time.png`: 时间戳 / 复制 / 现在 / 本地时间 render; no compositor title bar

## Decision override

User overrode D-14 / UI-SPEC decorations:yes. Tools are now frameless. Phase 3 should clone this shell, not a decorated eframe chrome.
