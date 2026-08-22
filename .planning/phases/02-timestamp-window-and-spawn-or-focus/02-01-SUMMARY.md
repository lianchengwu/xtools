---
phase: 02-timestamp-window-and-spawn-or-focus
plan: 01
subsystem: ui
tags: [eframe, egui, spawn, single-instance]

requires:
  - phase: 01
    provides: Host orb, ToolId::Time click, last_pointer_event
provides:
  - xtools-time eframe binary
  - raise_instance + launch_time
  - egui-chrome feature
affects: [02-02, phase-3]

tech-stack:
  added: [eframe 0.36.1, egui 0.36.1, jiff 0.2.35]
  patterns: [raise-before-spawn, feature-gated chrome]

key-files:
  created:
    - crates/xtools-time/src/main.rs
    - crates/xtools-time/src/app.rs
    - crates/xtools-ui/src/chrome.rs
  modified:
    - crates/xtools-host/src/main.rs
    - crates/xtools-ui/src/instance.rs
    - crates/xtools-ui/src/ids.rs

requirements-completed: [LAUNCH-01, LAUNCH-02, UI-02]

coverage:
  - id: D1
    description: Clock orb opens independent timestamp window
    requirement: LAUNCH-01
    verification: []
    human_judgment: true
    rationale: Must click the live overlay
  - id: D2
    description: Second click focuses; no second process
    requirement: LAUNCH-02
    verification: []
    human_judgment: true
    rationale: Process count and focus are live compositor behavior
  - id: D3
    description: Shared chrome title/fields/buttons
    requirement: UI-02
    verification:
      - kind: other
        ref: grep apply_theme crates/xtools-ui/src/chrome.rs
        status: pass
    human_judgment: false

duration: 40min
completed: 2026-08-22
status: complete
---

# Phase 2 Plan 01 Summary

**Clock orb raise-or-spawns a skip-taskbar eframe window titled xtools · 时间戳.**

## Accomplishments

- `xtools-time` binary, eframe 0.36.1
- Host `launch_time`: raise_instance first, sibling spawn only if absent
- `egui-chrome` feature; host does not enable it
- Close quits; next click is a new now-filled process

## Deviations from Plan

- eframe 0.36 App API is `ui(&mut Ui)` not `update(&mut Context)`
- Center via `ViewportCommand::center_on_screen`

## Next Phase Readiness

Convert/copy landed in the same implementation pass (02-02).
