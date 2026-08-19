# Phase 1: Host Orb and Orbital Menu - Context

**Gathered:** 2026-08-19
**Status:** Ready for planning

<domain>
## Phase Boundary

User can drag an always-on-top main orb and click it to expand or collapse three hardcoded function orbs painted from shared theme tokens. Function orbs are menu geometry — they do not launch tool windows in this phase. A click must still produce a pointer serial later phases can mint an activation token from.

In scope: HOST-01, HOST-02, HOST-03, LAUNCH-03, UI-01.
Out of scope this phase: spawn/focus (Phase 2), timestamp/JSON/translate windows, plugin scan, search box, persist orb position.

</domain>

<decisions>
## Implementation Decisions

### Locked before this discussion
- **D-01:** Host is GTK4 + gtk4-layer-shell Overlay. Tool windows stay eframe. Do not revisit.
- **D-02:** Menu is three hardcoded entries (timestamp, JSON, translate). No plugin directory. No search box.
- **D-03:** Click main orb toggles expand/collapse. Not a hotkey. Not hover-to-open.

### 主球外观
- **D-04:** Main orb is a ~40 logical-px dark disk with a light letter **x** centered in it.
- **D-05:** Not a solid unmarked disk, not a colored accent button, not translucent.

### 三球怎么排
- **D-06:** Three function orbs fan in an arc **above** the main orb (not a 120° triangle, not a vertical list on the right).
- **D-07:** Each function orb is ~32 logical-px, slightly smaller than the main orb.
- **D-08:** Identify tools with a simple mark on the disk, no full name, no side label: clock (timestamp), `{}` (JSON), text/文 (translate).
- **D-09:** ROADMAP still requires expanded disks stay on the output so every ball remains clickable. If the fan would clip at the top edge, rotate or shrink just enough to keep every disk fully visible. Do not persist position.

### 展开收起手感
- **D-10:** Expand is a short pop from the main orb to the fan seats (~100–180 ms). Collapse is the reverse. Not instant, not a long animation.
- **D-11:** Click-only. Hover does not expand or collapse.
- **D-12:** Collapse on: second click of the main orb, click of a function orb, or click anywhere that is not a function orb (including the transparent overlay / desktop). Any of those closes the menu.
- **D-13:** Outside-click dismiss requires the expanded overlay to receive those clicks. Collapsed state must keep the input region to the main disk only so the rest of the desktop stays click-through.

### 拖和点怎么分
- **D-14:** Movement under ~6–8 logical px is a click. Beyond that is a drag.
- **D-15:** If a drag starts while the menu is open, collapse first, then drag only the main orb. Function orbs do not travel with the drag.
- **D-16:** First launch places the main orb at the **middle of the right screen edge** (vertically centered, inset so the full disk is on-screen).
- **D-17:** The entire main orb must stay inside the output while dragging. Clamp; never allow it off-screen.
- **D-18:** Do not persist orb position in this phase. Restart returns to D-16.

### the agent's Discretion
None — user picked a concrete option on every question.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Product lock
- `.planning/PROJECT.md` — core value, out-of-scope list, Linux personal use
- `.planning/REQUIREMENTS.md` — HOST-01, HOST-02, HOST-03, LAUNCH-03, UI-01
- `.planning/ROADMAP.md` — Phase 1 goal, mode mvp, host-window lock, UI hint

### Research (host overlay)
- `.planning/research/SUMMARY.md` — GTK4 + gtk4-layer-shell Overlay for host; eframe for tools; input region = visible disks
- `.planning/research/ARCHITECTURE.md` — host owns orb + orbital menu; one surface; no per-ball process
- `.planning/research/PITFALLS.md` — do not use winit AlwaysOnTop; exclusive_zone −1; KeyboardMode::None; circle hit-tests; slop; no buffer before configure
- `.planning/research/STACK.md` — crate pins; **ignore** its eframe-everywhere host decision (overruled by SUMMARY)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None. Greenfield. No `src/`, no crates yet.

### Established Patterns
- None in-repo. Follow research: virtual Cargo workspace, `xtools-ui` theme tokens first, `xtools-host` as gtk4-layer-shell Overlay, `default-members = ["crates/xtools-host"]`.

### Integration Points
- Phase 2 will spawn/focus from a function-orb click. Phase 1 must emit a pointer serial on that click even though the balls are stubs.
- Shared theme tokens in `xtools-ui` are the only visual contract Phase 2/3 clone.

</code_context>

<specifics>
## Specific Ideas

- Main mark is the letter **x**, not a toolbox glyph.
- Function marks: clock / curly braces / 文. Keep them as simple vector strokes on the disk.
- Fan lives above the main orb so the area under the orb stays readable.
- Dismiss-on-outside-click is a real menu, not a sticky launcher.

</specifics>

<deferred>
## Deferred Ideas

- Persist orb position across restarts — HOST-05, v2
- Dedicated edge-aware constellation beyond the minimum “stay on output” clamp — HOST-04, v2
- Hover highlight on function orbs — not chosen; click-only
- Function orbs traveling with an in-menu drag — rejected; drag collapses first

None — discussion stayed within phase scope

</deferred>

---

*Phase: 1-Host Orb and Orbital Menu*
*Context gathered: 2026-08-19*
