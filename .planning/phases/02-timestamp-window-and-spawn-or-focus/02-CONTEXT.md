# Phase 2: Timestamp Window and Spawn-or-Focus - Context

**Gathered:** 2026-08-19
**Status:** Ready for planning

<domain>
## Phase Boundary

User can click the clock function orb and get an independent eframe timestamp window. If that process is already running, the existing window is focused and its contents are left alone. First open (and reopen after quit) fills the fields with "now". The window is the chrome template for JSON and translate.

In scope: LAUNCH-01, LAUNCH-02, TIME-01, TIME-02 (narrowed: copy seconds and milliseconds only), UI-02.
Out of scope this phase: JSON window, translate window, plugin scan, persist last values, RFC3339/custom-format copy buttons.

</domain>

<decisions>
## Implementation Decisions

### Carried from Phase 1 / project lock
- **D-01:** Host remains GTK4 + gtk4-layer-shell. This window is eframe, not Overlay, not always-on-top.
- **D-02:** Clock orb is `ToolId::Time`. Host already stashes `gdk::Event` on function-orb click. Use that for spawn/focus.
- **D-03:** Single instance per tool. Abstract socket + raise. Second process must not open a second window.
- **D-04:** Shared colors/type/spacing come from `xtools-ui` tokens. This phase adds the egui chrome (`apply_theme`, title area, fields, buttons) that Phase 3 will copy.

### 窗口里怎么排
- **D-05:** Vertical stack. Top: Unix seconds field and Unix milliseconds field. Bottom: one editable local datetime field. Edit any one field; the other two update immediately.
- **D-06:** Seconds and milliseconds are two separate boxes, kept in sync (ms = s × 1000).
- **D-07:** Datetime is local timezone, editable. No extra UTC row. No RFC3339 editor.
- **D-08:** Copy buttons sit beside the timestamp fields only (seconds, milliseconds). Not beside datetime.

### 打开时默认填什么
- **D-09:** First launch (and launch after the process exited) fills all three fields from the current instant.
- **D-10:** If the window is already open, a later clock-orb click focuses it and does **not** overwrite fields.
- **D-11:** A 「现在」 button sits next to the local datetime field. Clicking it writes the current instant into all three fields.

### 自定义格式 / 复制范围
- **D-12:** Do not ship a custom format box. Do not ship RFC3339 or datetime copy buttons. TIME-02 in this phase means: one-click copy of the 10-digit seconds value and the 13-digit milliseconds value.
- **D-13:** RFC3339 copy and custom strftime copy are deferred.

### 窗口出现在哪
- **D-14:** First show: centered on the current output. About 560×480. Ordinary decorated eframe toplevel.
- **D-15:** Closing the window exits the timestamp process. Next clock-orb click starts a new process and fills "now" (D-09).
- **D-16:** The timestamp window must **not** appear on the taskbar (skip-taskbar / Utility hint). Host already does not. Phase 3 tools inherit this unless revisited.
- **D-17:** Title bar: `xtools · 时间戳` (agent discretion; suite prefix for later tools).

### the agent's Discretion
- Exact egui widget spacing beyond tokens.
- How skip-taskbar is spelled on Wayland vs X11 (Utility, `skip_taskbar`, app_id `dev.xtools.timestamp`).
- Invalid input: keep the last good peer fields and show a short inline error; do not crash.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Product lock
- `.planning/PROJECT.md` — independent processes, single-instance focus, Linux personal use
- `.planning/REQUIREMENTS.md` — LAUNCH-01, LAUNCH-02, TIME-01, TIME-02, UI-02
- `.planning/ROADMAP.md` — Phase 2 goal, mode mvp, eframe tools, chrome template
- `.planning/phases/01-host-orb-and-orbital-menu/01-CONTEXT.md` — D-01 host/toolkit split; function-orb click stashes pointer event

### Research
- `.planning/research/SUMMARY.md` — eframe 0.36.1 tools; Unix socket raise; jiff 0.2.35; xdg-activation from click serial
- `.planning/research/STACK.md` — jiff, eframe pins; do not put eframe in the host
- `.planning/research/PITFALLS.md` — lock file without activation token; keep process alive after copy (Wayland clipboard); no wmctrl

### Code to extend
- `crates/xtools-host/src/main.rs` — `last_pointer_event` on function-orb click; spawn Time from here
- `crates/xtools-ui/src/ids.rs` — `ToolId::Time`
- `crates/xtools-ui/src/instance.rs` — `claim_instance` (reuse for `xtools-time`)
- `crates/xtools-ui/src/theme.rs` — tokens for chrome

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `xtools_ui::claim_instance` — abstract socket lock; timestamp binary should claim `\0xtools-time`
- `xtools_ui::theme` — ORB_* unused in the tool window; use the Color type and add chrome tokens if needed rather than hex in the tool
- `xtools_ui::ToolId` — host maps Time → `xtools-time` binary
- Host `last_pointer_event: Option<gdk::Event>` — mint activation token / pass to child env

### Established Patterns
- Virtual workspace, `default-members` still host
- No eframe/egui/winit in `xtools-host`
- Single-instance: bind abstract name or exit 0 (today the second host process just exits; Phase 2 must **raise** the live timestamp window)

### Integration Points
- Clock orb click currently only collapses the menu and stashes the event. This phase adds `Command::new` of `xtools-time` (same cargo target dir / `CARGO_BIN` / next to host) with inherited session env + `XDG_ACTIVATION_TOKEN`.
- If `claim_instance("xtools-time")` fails, write Raise + token on the socket and exit 0; the live process focuses.
- Closing the eframe viewport must quit the process (D-15), which drops the socket so the next click spawns fresh.

</code_context>

<specifics>
## Specific Ideas

- Two timestamp boxes + one local datetime + Now beside datetime + copy beside each timestamp box.
- Window title `xtools · 时间戳`.
- User was explicit: "只要复制时间戳" — do not add extra copy targets "while we are here."

</specifics>

<deferred>
## Deferred Ideas

- RFC3339 one-click copy — user declined for this phase
- Custom strftime format + copy — user declined
- Clipboard-auto-fill on open — not chosen
- Persist last values / last window position — v2
- UTC companion row — not chosen

</deferred>

---

*Phase: 2-Timestamp Window and Spawn-or-Focus*
*Context gathered: 2026-08-19*
