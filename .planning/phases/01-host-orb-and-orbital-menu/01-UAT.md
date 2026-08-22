---
status: complete
phase: 01-host-orb-and-orbital-menu
source: [01-01-SUMMARY.md, 01-02-SUMMARY.md]
updated: 2026-08-19T16:40:00Z
---

## Current Test

[testing complete]

## Tests

### 1. See and drag the main orb
expected: Dark x orb mid-right, always on top, draggable, stays on screen
result: pass
reported: "yes — visible mid-right after red fill + top-right layer-shell placement"
coverage_id: 01-01-D1

### 2. Click main orb expands three function orbs
expected: Short pop; three smaller disks fan above the main orb (clock / {} / 文)
result: pass
coverage_id: 01-02-D1

### 3. Click again or outside collapses
expected: Second click on main, click on a function orb, or click on empty overlay glass folds the three balls back
result: pass
coverage_id: 01-02-D2

### 4. Shared theme tokens compiled into xtools-ui
expected: Shared theme tokens compiled into xtools-ui
result: pass
source: automated
coverage_id: 01-01-D2

### 5. Menu is ToolId Time/Json/Trans only
expected: Menu is ToolId Time/Json/Trans only
result: pass
source: automated
coverage_id: 01-02-D3

## Summary

total: 5
passed: 5
issues: 0
pending: 0
skipped: 0

## Gaps

[none yet]
