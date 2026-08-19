# Walking Skeleton — xtools

**Phase:** 1
**Generated:** 2026-08-19

## Capability Proven End-to-End

Click the floating x orb and three marked balls pop above it.

## Architectural Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Framework | Rust 2024 virtual workspace; host is GTK4 0.11.4 + gtk4-layer-shell 0.8.1 Overlay | D-01. winit AlwaysOnTop is unsupported on this KDE Wayland session. Tool windows stay eframe in later phases; they are not the host. |
| Data layer | N/A — no database | This is a local desktop overlay. Menu entries are compile-time `ToolId` values. Orb position is in-process only (D-18). No network, no persistence, no rows to read or write. |
| Auth | N/A — no accounts | Personal local app. Single-instance abstract socket (NUL + `xtools-host`) is same-uid process exclusion, not user authentication. |
| Deployment target | Local `cargo run -p xtools-host` on this openSUSE KDE Wayland session | v1 is personal Linux use. No installer, no autostart, no multi-DE packaging. |
| Directory layout | `crates/xtools-ui` (tokens + `ToolId`) + `crates/xtools-host` (one Overlay surface) | Tokens compile into every later binary. Host owns orb + orbital menu on one surface. Do not stub tool crates in Phase 1. |

## Stack Touched in Phase 1

- [ ] Project scaffold — virtual Cargo workspace, resolver 3, edition 2024, `default-members = ["crates/xtools-host"]`
- [ ] Routing — N/A (not a web app; one binary entry `xtools-host`)
- [ ] Database — N/A (no DB: overlay state is process memory; three tools are an enum, not rows)
- [ ] UI — click the main x disk; three marked function disks (clock / `{}` / 文) pop above it
- [ ] Deployment — documented local full-stack run: `cargo run -p xtools-host` (requires `gtk4-devel`, `gtk4-layer-shell-devel`, `libgtk4-layer-shell0`)

## Out of Scope (Deferred to Later Slices)

- Spawn or focus of timestamp / JSON / translate windows (Phase 2 / 3; LAUNCH-01, LAUNCH-02)
- eframe tool chrome and shared control rhythm (UI-02, Phase 2)
- Persist orb position across restarts (HOST-05, v2)
- Dedicated edge-aware constellation beyond stay-on-output rotate/shrink (HOST-04, v2)
- Query field / command palette
- Plugin directory scan
- Hover-to-open, global hotkeys
- Function orbs traveling with an in-menu drag
- Network, API keys, D-Bus exports

## Subsequent Slice Plan

Each later phase adds one vertical slice on top of this skeleton without altering its architectural decisions:

- Phase 2: Click a function orb to spawn or focus the timestamp window (eframe chrome template, xdg-activation from the stashed pointer event)
- Phase 3: Clone that chrome into JSON format/minify/validate and a translate shell with one swappable engine
