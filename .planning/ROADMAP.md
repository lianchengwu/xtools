# Roadmap: xtools

## Overview

xtools ships as a personal Linux orbital toolbox. Phase 1 puts a GTK4 layer-shell Overlay orb on the desktop, expands and collapses three hardcoded function balls, and paints them from shared theme tokens. Phase 2 opens the timestamp window as the eframe chrome template and proves spawn-or-focus. Phase 3 clones that chrome into JSON and translate so the three independent windows feel like one suite.

## Phases

**Phase Numbering:**

- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

- [ ] **Phase 1: Host Orb and Orbital Menu** - Always-on-top draggable orb expands and collapses three hardcoded function balls
- [ ] **Phase 2: Timestamp Window and Spawn-or-Focus** - Click a function orb to open or focus the timestamp window and convert/copy times
- [ ] **Phase 3: JSON and Translate Tools** - JSON format/minify/validate and a translate shell with one swappable engine

## Phase Details

### Phase 1: Host Orb and Orbital Menu

**Goal:** User can drag an always-on-top main orb and click it to expand or collapse three hardcoded function orbs painted from shared theme tokens.
**Mode:** mvp
**Depends on:** Nothing (first phase)
**Context:** Host-window decision is locked: `xtools-host` is GTK4 + gtk4-layer-shell Overlay; tool windows stay eframe. Do not revisit. Function orbs are menu geometry — they need not launch a tool yet, but a click must produce a pointer serial later phases can mint an activation token from. Expanded disks stay on the output so every ball remains clickable. No search box. No plugin directory scan.
**Requirements:** HOST-01, HOST-02, HOST-03, LAUNCH-03, UI-01
**Success Criteria** (what must be TRUE):

  1. User can see an always-on-top main orb and drag it to a convenient spot
  2. User can click the main orb and see exactly three function orbs appear around it (timestamp, JSON, translate) — not scanned from a plugin directory
  3. User can click the main orb again and the function orbs collapse
  4. User sees the host orb and function orbs painted from the shared color, type size, and spacing tokens that later tool windows will use

**Plans:** TBD
**UI hint**: yes

### Phase 2: Timestamp Window and Spawn-or-Focus

**Goal:** User can click a function orb to spawn or focus the timestamp window, convert Unix time both ways, and copy results through the shared chrome.
**Mode:** mvp
**Depends on:** Phase 1
**Context:** Timestamp is the chrome template. JSON and translate clone this window rhythm in Phase 3; do not invent a second look. Tools are ordinary eframe toplevels, not layer-shell surfaces.
**Requirements:** LAUNCH-01, LAUNCH-02, TIME-01, TIME-02, UI-02
**Success Criteria** (what must be TRUE):

  1. User can click a function orb and get that tool as an independent Rust window
  2. User can click a function orb whose window is already open and the existing window is focused; a second copy is not created
  3. User can convert Unix seconds or milliseconds to datetime and the other way around
  4. User can one-click copy a result as 10-digit, 13-digit, RFC3339, or a custom format
  5. User sees the timestamp window use the shared chrome and control rhythm (title area, buttons, fields) that later tools will copy

**Plans:** 2 plans
**Wave 1**

- [ ] 02-01-PLAN.md — Clock orb spawn-or-focus of the timestamp chrome window

**Wave 2** *(blocked on Wave 1 completion)*

- [ ] 02-02-PLAN.md — Live convert and copy seconds/milliseconds only

**UI hint**: yes

### Phase 3: JSON and Translate Tools

**Goal:** User can format and validate JSON with line/column errors, and translate text in a dedicated window backed by one swappable engine.
**Mode:** mvp
**Depends on:** Phase 2
**Context:** Clone Phase 2 chrome. Translate talks only to a `TranslateEngine` trait; ship one working engine. API keys are not stored in the repo. Menu stays three hardcoded binaries.
**Requirements:** JSON-01, JSON-02, JSON-03, TRANS-01, TRANS-02
**Success Criteria** (what must be TRUE):

  1. User can format pasted JSON
  2. User can minify pasted JSON
  3. User can see a validation failure at a line and column, not only pass/fail
  4. User can type or paste text, pick languages, and see output in a dedicated translate window
  5. User can get a real translation from one swappable engine; API keys are not stored in the repo

**Plans:** TBD
**UI hint**: yes

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Host Orb and Orbital Menu | 0/TBD | Not started | - |
| 2. Timestamp Window and Spawn-or-Focus | 0/TBD | Not started | - |
| 3. JSON and Translate Tools | 0/TBD | Not started | - |
