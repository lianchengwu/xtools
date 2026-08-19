---
phase: 01-host-orb-and-orbital-menu
plan: 01
subsystem: ui
tags: [gtk4, gtk4-layer-shell, cairo, wayland, kwin]

requires: []
provides:
  - Virtual Cargo workspace with xtools-ui + xtools-host
  - Shared theme tokens and ToolId
  - Runnable GTK Overlay orb
affects: [01-02, phase-2]

tech-stack:
  added: [gtk4 0.11.4, gtk4-layer-shell 0.8.1]
  patterns: [virtual workspace, Overlay exclusive_zone -1, token-driven paint]

key-files:
  created:
    - Cargo.toml
    - crates/xtools-ui/src/theme.rs
    - crates/xtools-ui/src/ids.rs
    - crates/xtools-host/src/overlay.rs
    - crates/xtools-host/src/main.rs
  modified: []

key-decisions:
  - "Host is GTK4 Overlay, not eframe"
  - "pkg-config module on this box is gtk4, not gtk+-4.0"

patterns-established:
  - "Paint and layout numbers come from xtools-ui::theme"
  - "init_layer_shell before present; exclusive_zone -1"

requirements-completed: [HOST-01, UI-01]

coverage:
  - id: D1
    description: Always-on-top draggable main orb
    requirement: HOST-01
    verification:
      - kind: other
        ref: timeout 8 ./target/debug/xtools-host exits 124
        status: pass
    human_judgment: true
    rationale: Overlay z-order and drag must be confirmed on this KWin session
  - id: D2
    description: Shared theme tokens compiled into xtools-ui
    requirement: UI-01
    verification:
      - kind: other
        ref: cargo check -p xtools-ui && grep MAIN_D crates/xtools-ui/src/theme.rs
        status: pass
    human_judgment: false

duration: 25min
completed: 2026-08-19
status: complete
---

# Phase 1: Host Orb and Orbital Menu — Plan 01 Summary

**GTK4 layer-shell Overlay host with a 40px dark x orb, shared tokens, and a virtual workspace.**

## Performance

- **Duration:** ~25 min
- **Tasks:** 2
- **Files modified:** 12+

## Accomplishments

- Installed gtk4-devel, gtk4-layer-shell-devel, libgtk4-layer-shell0
- Virtual workspace; `cargo run -p xtools-host`
- Overlay orb: Layer::Overlay, exclusive_zone -1, KeyboardMode::None
- Tokens in xtools-ui; host paints from them
- Binary stayed up 8s (timeout 124), no crash

## Deviations from Plan

- Text marks use cairo toy fonts, not pango layouts (no extra pangocairo dep)
- openSUSE pkg-config name is `gtk4`, not `gtk+-4.0`

## Issues Encountered

None after devel packages installed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 01-02 expand/collapse is implemented in the same host binary.

---
*Phase: 01-host-orb-and-orbital-menu*
*Completed: 2026-08-19*
