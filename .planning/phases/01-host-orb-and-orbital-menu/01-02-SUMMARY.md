---
phase: 01-host-orb-and-orbital-menu
plan: 02
subsystem: ui
tags: [gtk4, orbital-menu, input-region, tick-callback]

requires:
  - phase: 01-01
    provides: Overlay host + tokens + ToolId
provides:
  - Fan-above three-ball menu
  - 120ms pop / collapse
  - Two-state input region
  - Pointer event stash
affects: [phase-2]

tech-stack:
  added: []
  patterns: [GestureDrag 8px slop, scanline disk region, tick pop]

key-files:
  created:
    - crates/xtools-host/src/anim.rs
    - crates/xtools-host/src/layout.rs
    - crates/xtools-host/src/input.rs
    - crates/xtools-host/src/paint.rs
  modified:
    - crates/xtools-host/src/main.rs

key-decisions:
  - "Expanded input region is the whole surface so outside click dismisses"
  - "Drag while open snap-collapses; orbs do not travel"

patterns-established:
  - "ToolId::ALL is the only menu source"
  - "Stash gdk::Event on function-orb click; do not spawn"

requirements-completed: [HOST-02, HOST-03, LAUNCH-03]

coverage:
  - id: D1
    description: Click main orb expands three hardcoded function orbs
    requirement: HOST-02
    verification: []
    human_judgment: true
    rationale: Visual fan + marks need a look on this session
  - id: D2
    description: Click again or outside collapses
    requirement: HOST-03
    verification: []
    human_judgment: true
    rationale: Dismiss paths are pointer UX
  - id: D3
    description: Menu is ToolId Time/Json/Trans only
    requirement: LAUNCH-03
    verification:
      - kind: other
        ref: grep enum ToolId crates/xtools-ui/src/ids.rs
        status: pass
    human_judgment: false

duration: 20min
completed: 2026-08-19
status: complete
---

# Phase 1: Host Orb and Orbital Menu — Plan 02 Summary

**Click the x orb and three marked balls pop above it; click anywhere to fold them back.**

## Accomplishments

- Fan seats at -150/-90/-30 deg with rotate/shrink to stay on output
- 120ms ease-out-cubic tick pop
- Collapse on main re-click, function-orb click, glass click
- Drag-while-open snap-collapses then moves only the main disk
- Collapsed input region is a scanline disk; expanded is the full surface
- Function-orb click stores last_pointer_event and does not spawn

## Deviations from Plan

- Plans 01-01 and 01-02 landed in one host implementation pass

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Host can stash a pointer event. Phase 2 should spawn/focus xtools-time using that event for xdg-activation.

---
*Phase: 01-host-orb-and-orbital-menu*
*Completed: 2026-08-19*
