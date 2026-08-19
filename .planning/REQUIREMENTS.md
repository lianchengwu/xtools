# Requirements: xtools

**Defined:** 2026-08-19
**Core Value:** 点击主球，功能球围绕它弹出；再点功能球，打开或聚焦对应独立窗口。这一条必须成立。

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Host

- [ ] **HOST-01**: User can see a always-on-top main orb and drag it to a convenient spot
- [ ] **HOST-02**: User can click the main orb and see three function orbs appear around it (timestamp, JSON, translate)
- [ ] **HOST-03**: User can click the main orb again and the function orbs collapse

### Launch

- [ ] **LAUNCH-01**: User can click a function orb and get that tool as an independent Rust window
- [ ] **LAUNCH-02**: User can click a function orb whose window is already open and the existing window is focused; a second copy is not created
- [ ] **LAUNCH-03**: User sees exactly three hardcoded entries (timestamp, JSON, translate); the menu does not scan a plugin directory

### Timestamp

- [ ] **TIME-01**: User can convert Unix seconds or milliseconds to datetime and the other way around
- [ ] **TIME-02**: User can one-click copy a result as 10-digit, 13-digit, RFC3339, or a custom format

### JSON

- [ ] **JSON-01**: User can format pasted JSON
- [ ] **JSON-02**: User can minify pasted JSON
- [ ] **JSON-03**: User can see validation failure at a line and column, not only pass/fail

### Translate

- [ ] **TRANS-01**: User can type or paste text, pick languages, and see output in a dedicated window
- [ ] **TRANS-02**: User can get a real translation from one swappable engine; API keys are not stored in the repo

### Appearance

- [ ] **UI-01**: User sees the same colors, type size, and spacing in all three tool windows
- [ ] **UI-02**: User sees the same window chrome and control rhythm (title area, buttons, fields) in all three tool windows

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Host

- **HOST-04**: Function orbs rotate or shrink so they stay on the output when the main orb sits on an edge
- **HOST-05**: Main orb position persists across restarts

### Tools

- **TIME-03**: Last custom timestamp format is remembered
- **TRANS-03**: Last language pair is remembered
- **TRANS-04**: User can configure the translate engine (endpoint / key / command) in the UI
- **LAUNCH-04**: Drop-in plugin binaries appear in the menu without a rebuild

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Search box / command palette | User chose orbital balls; a box turns this into a uTools clone |
| Super panel / selection context menu | Different product; needs global hooks |
| Clipboard listen / auto-detect | Surprising; not the entry model |
| Global hotkeys | User rejected hotkeys as the entry |
| Click main orb opens all three windows | User negated this; menu step is required |
| Always-visible three balls, no main | User wants one quiet orb until expanded |
| Multi-instance tool windows | User chose focus-existing |
| Offline dictionary / local translation model | User wants a swappable engine, not a bound local model |
| jq / JSONPath | Format + validate is the v1 job |
| Embed tool UI in the host | Violates independent-process contract |
| Plugin marketplace | Personal v1; no trust surface |
| Installer, autostart, multi-DE packaging | Personal Linux use |
| Windows / macOS | Current machine is Linux |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| HOST-01 | — | Pending |
| HOST-02 | — | Pending |
| HOST-03 | — | Pending |
| LAUNCH-01 | — | Pending |
| LAUNCH-02 | — | Pending |
| LAUNCH-03 | — | Pending |
| TIME-01 | — | Pending |
| TIME-02 | — | Pending |
| JSON-01 | — | Pending |
| JSON-02 | — | Pending |
| JSON-03 | — | Pending |
| TRANS-01 | — | Pending |
| TRANS-02 | — | Pending |
| UI-01 | — | Pending |
| UI-02 | — | Pending |

**Coverage:**
- v1 requirements: 15 total
- Mapped to phases: 0
- Unmapped: 15

---
*Requirements defined: 2026-08-19*
*Last updated: 2026-08-19 after initial definition*
