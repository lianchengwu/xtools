# Project Research Summary

**Project:** xtools
**Domain:** Rust Linux desktop orbital-launcher toolbox
**Researched:** 2026-08-19
**Confidence:** MEDIUM

## Executive Summary

xtools is a personal Linux toolbox, not a search-box launcher and not a DevToys mega-window. The product is one always-on-top host orb that expands into three orbital function balls; each ball spawn-or-focuses an independent Rust window (timestamp, JSON, translate). Experts build this as a **host + satellite processes** app: the host owns overlay chrome and launch, tools own their own toplevels, and visual unity comes from a compiled-in theme crate — not from embedding tool UI or shipping theme bytes over IPC.

**Recommended approach:** split the GUI stack on purpose. The host orb is **GTK4 + gtk4-layer-shell Overlay** (`zwlr_layer_shell_v1`). The three tool windows stay **eframe 0.36.1 + egui 0.36.1**. Shared look is `xtools-ui` theme tokens plus egui chrome/widgets compiled into the tools; the host only paints circles from those tokens. Single-instance is an abstract Unix socket plus an `xdg-activation-v1` token minted from the ball-click serial. Translate is a `Box<dyn TranslateEngine>` plus one working v1 impl. Menu is a hardcoded `ToolId` enum.

The key risk is treating the host as a normal always-on-top window. This machine is **KDE Plasma Wayland**. winit 0.30 documents `WindowLevel` as **unsupported on Wayland**, so eframe `with_always_on_top()` is a no-op on the daily compositor and the core value dies. Layer-shell Overlay is the path that actually floats above a maximized window on KWin. Secondary risks: a full-surface input region bricks the desktop; a lock file without an activation token looks like a dead orb; mixing three independently styled tool UIs before a chrome template exists. Mitigate in that order — overlay + input region + slop first, then one template tool with raise, then clone chrome into JSON and translate.

## Key Findings

### Recommended Stack

See [STACK.md](STACK.md) for crate pins. Use its versions; **reject its "one toolkit for host and tools" decision.**

STACK is right that four independently styled apps would fail the "one suite" requirement, and right that eframe 0.36.1 is the cheapest consistent form-UI story in Rust. It is wrong that the host can be that same eframe window. Always-on-top via winit is an X11/Windows flag. On this Plasma Wayland session it will not keep the orb above a maximized browser. ARCHITECTURE.md and PITFALLS.md both treat that as a product-killing mistake, and they are correct for *this* machine (KWin implements layer-shell; Mutter does not, but this is not a GNOME box).

**Host-window decision: GTK4 + gtk4-layer-shell Overlay for `xtools-host` only. Tools stay eframe. Do not revisit this during roadmap.**

Why this split, not "eframe everywhere" and not "GTK everywhere":

- The host is painted disks + hit-tests, not forms. It needs compositor-owned z-order and a shaped input region. gtk4-layer-shell 0.8.x gives `Layer::Overlay`, `exclusive_zone = -1`, `KeyboardMode::None`, and `gdk_surface_set_input_region`. eframe does not.
- The tools are ordinary decorated toplevels (fields, copy rows, error gutter). They must **not** be layer-shell surfaces. eframe/egui 0.36.1 is the right toolkit there: shared `Visuals`, cheap custom widgets, `ViewportCommand::Focus`.
- Mixing toolkits is acceptable because the host never draws tool chrome. `xtools-ui` is tokens + instance protocol for everyone, and egui widgets/chrome for the three tools only. Do not put egui inside the host "to keep one toolkit." Recovery cost of a winit host is HIGH: throw the host away and keep the tools.

**Core technologies:**

- **Rust edition 2024 / rustc 1.97.1:** User-locked language; already on the machine.
- **Cargo virtual workspace:** Four binaries + `xtools-ui`. Host is `default-members`. No root package.
- **gtk4 0.11.4 + gtk4-layer-shell 0.8.1:** Host overlay only. `is_supported()` at startup; `init_layer_shell()` before map.
- **eframe / egui / egui_extras 0.36.1:** All three tool windows. Default features (wayland + x11 + wgpu). Pin one winit (0.30.13 via eframe). Glow only if wgpu transparency is a black rectangle on this NVIDIA laptop — tools, not host.
- **std `UnixListener` (abstract `AF_UNIX`):** Single-instance lock + `Wake::Focus { token }`. Not `single-instance` 0.3.3. Not a pidfile.
- **jiff 0.2.35:** Timestamp s/ms ↔ datetime, RFC3339, `strftime`. Not chrono.
- **serde_json 1.0.151:** Format, minify, `Error::line()` / `column()`. No second JSON crate.
- **ureq 3.4.0 (`json`):** Sync HTTPS behind `TranslateEngine`. Worker thread + channel. No tokio in every binary.
- **thiserror 2.0.20 / anyhow 1.0.104:** Typed engine/parse errors in UI; anyhow at `main` only.

**Critical version / session requirements:**

- Do not mix egui 0.31 widgets with eframe 0.36.
- Do not disable eframe Wayland features.
- Host must not depend on eframe/egui/winit. Warning sign: `WindowLevel::AlwaysOnTop` in host code.
- Confirm `XDG_SESSION_TYPE=wayland` + `XDG_CURRENT_DESKTOP=KDE` before calling the overlay done. X11 `_NET_WM_STATE_ABOVE` is fallback only.

### Expected Features

See [FEATURES.md](FEATURES.md). Table stakes are the locked click path plus daily-tool depth, not Alfred/Raycast coverage.

**Must have (table stakes):**

- Always-on-top draggable main orb — no hotkey, always visible
- Click main → Timestamp / JSON / Translate orbs around it; click again → collapse
- Click function orb → spawn or focus that independent Rust window; never a second copy
- Shared theme / controls / layout rhythm across the three windows
- Timestamp: Unix s/ms ↔ datetime; one-click copy of 10-digit, 13-digit, RFC3339, custom
- JSON: format, minify, validate with line/column (not a boolean)
- Translate: input / output / language shell + one working swappable engine
- Hardcoded three-entry menu — no directory scan

**Should have (competitive):**

- Orbital 3-ball menu instead of a search box (this *is* the product)
- Persistent mouse-first orb (no bind to remember)
- Independent window processes as the default (a tool crash cannot kill the orb)
- Three first-class windows that stay open while you work

**Defer (v1.x / v2+):**

- Persist orb position, remember last formats/languages, translate engine config UI — v1.x after daily use
- Plugin-directory scan, search box, super panel, clipboard listen, global hotkeys — rejected / different product
- jq / JSONPath, offline translation model, extra DevToys catalog, installer / cross-platform, multi-instance windows

Edge-aware orbit is listed as P2 in FEATURES.md. Treat a **minimum rotate/shrink so no disk leaves the output** as Phase 1, not polish — clipped balls make tools unreachable. Persist-across-restart can wait.

### Architecture Approach

See [ARCHITECTURE.md](ARCHITECTURE.md). Host process + three satellite processes + one shared crate. Host never constructs a timestamp converter, a JSON tree, or a translate form. IPC payload in v1 is `Wake::Focus { token }` only.

**Major components:**

1. **`xtools-host`** — GTK layer-shell overlay: orb, drag, expand/collapse three host-drawn balls, hardcoded `ToolId`, spawn-or-focus. One surface; do not give each ball its own process or layer surface.
2. **`xtools-ui`** — Toolkit-agnostic theme tokens + `ToolId` / abstract-socket instance protocol for everyone; egui widgets + window chrome for tools only.
3. **`xtools-time` / `xtools-json` / `xtools-trans`** — Independent eframe toplevels. Time is the chrome template. JSON adds an error gutter. Trans holds `Box<dyn TranslateEngine + Send + Sync>` and one v1 impl; engines stay out of `xtools-ui`.

**Key patterns:**

- Virtual workspace, `default-members = ["crates/xtools-host"]`, `cargo test --workspace`.
- Abstract Unix socket bind = instance truth. Check `SO_PEERCRED` (same uid). Pathname + `flock` under `$XDG_RUNTIME_DIR/xtools/` only if abstract names get in the way. Host also single-instance (`\0xtools-host`).
- Activation token from the **click serial**, not a leftover or a timer. Child unsets `XDG_ACTIVATION_TOKEN` after first read. No `wmctrl` / `_NET_ACTIVE_WINDOW` on a Wayland session.
- Inherit the graphical session on spawn. Never `env_clear`. Host crash must not SIGKILL tools.
- No plugin folder, no theme IPC, no PID table as source of truth.

PITFALLS.md prefers D-Bus `org.freedesktop.Application` for raise. Do **not** start there. Abstract socket + token is enough for a personal Linux toolbox and matches ARCHITECTURE. Revisit D-Bus only if KWin ignores socket-delivered tokens.

### Critical Pitfalls

See [PITFALLS.md](PITFALLS.md). Top risks, in the order they will kill the product:

1. **eframe / winit AlwaysOnTop host** — No-op on Wayland; orb sinks under maximized windows. **Avoid:** GTK4 + gtk4-layer-shell Overlay for host only. Verify with a maximized app *over* the ball on this KWin session.
2. **Transparent overlay with a rectangular / full-surface input region** — Alpha ≠ input; the desktop is bricked. **Avoid:** input region = union of visible disks, rebuilt on move/expand/collapse/scale, surface-local.
3. **Exclusive zone or exclusive keyboard** — Steals panel geometry or every key. **Avoid:** `exclusive_zone = -1`, never `auto_exclusive_zone_enable`, `KeyboardMode::None`.
4. **Lock file without xdg-activation** — Second click exits; buried window stays buried. **Avoid:** socket `Wake::Focus` + token from the orb-click serial. Host must plumb the serial in Phase 1 even if tools come later.
5. **Theme copy-paste / three tools in parallel before chrome exists** — Three apps, not one suite. **Avoid:** land tokens + one template window (timestamp), then clone.
6. **Translate API keys in source or plaintext config** — History leak. **Avoid:** Secret Service; window shell can ship without a key; first real engine must not.

Also: 6–8 logical px drag/click slop; `radius >= r_main + r_orb + gap`; circle hit-tests; keep the main ball where the user parked it; keep the timestamp process alive after copy (Wayland clipboard has no server store); JSON parse off-thread for large pastes; no buffer on the layer surface before the first configure.

## Implications for Roadmap

Based on research, suggested phase structure. Coarse: three phases. Do not start tools until the host stays on top, click-through glass works, and a pointer serial can be emitted.

### Phase 1: Shared tokens + host orb + orbital expand/collapse
**Rationale:** Core value is "click the floating ball, three balls appear around it." If the overlay is wrong, every later phase sits on a dead host. Tokens must exist before the host paints, or the orb and the later tool chrome will diverge.
**Delivers:** Virtual workspace; `xtools-ui` theme tokens + `ToolId` / instance types; `xtools-host` as a gtk4-layer-shell Overlay; draggable main disk; click toggles three host-drawn balls; collapse does not close tools (none yet); host single-instance; input region = visible disks; exclusive_zone −1; keyboard None; slop + circle hit-test + no disk overlap; minimum rotate/shrink so expanded disks stay on the output; `is_supported()` with a one-line refusal or X11 fallback (not a GNOME extension). Balls are stubs — they need not launch yet, but the click must produce a serial the launcher can later mint a token from.
**Addresses:** Always-on-top orb; orbital 3-ball expand/collapse; hardcoded three-entry menu geometry; shared theme *tokens*.
**Avoids:** winit AlwaysOnTop host; full-surface input region; exclusive zone / exclusive keyboard; Mutter assumption (detect, don't crash); drag/click without slop; orbs covering the main disk; rectangular hit-tests; HiDPI double-scale; attaching a buffer before configure.
**Host-window decision (locked here):** GTK4 + gtk4-layer-shell Overlay. Not eframe. Not four small eframe viewports. Not XWayland unless `is_supported() == false`.

### Phase 2: Timestamp window as chrome template + single-instance focus
**Rationale:** Architecture build order: one tool as the style/contract template before cloning. Timestamp is pure functions, no network, and exercises copy + raise — the two Wayland-sensitive tool behaviors.
**Delivers:** `xtools-ui` egui chrome/widgets (`apply_theme`, `tool_frame`, `copy_button`); `xtools-time` eframe toplevel (not always-on-top); Unix s/ms ↔ datetime; one-click copy of 10 / 13 / RFC3339 / custom; process stays alive as clipboard owner; abstract socket claim-or-wake; host spawn-or-focus with inherited session env + `XDG_ACTIVATION_TOKEN`; child unsets the token after read; second click focuses, does not spawn.
**Uses:** eframe/egui 0.36.1, jiff 0.2.35, `xtools-ui` instance protocol, xdg-activation-v1.
**Implements:** Launcher in `xtools-host`; first satellite process; shared chrome that JSON and translate will copy.
**Addresses:** Spawn-or-focus; single-instance; shared theme/controls; timestamp convert + copy.
**Avoids:** Lock-file-only single-instance; `env_clear` on spawn; `wmctrl` on Wayland; closing the window immediately after copy; inventing a second visual system.

### Phase 3: Remaining tools — JSON + translate shell + pluggable engine
**Rationale:** Clone the timestamp chrome. Do not invent a second look. JSON and translate are independent of each other; they share the Phase 2 contract. Translate *shell* and one engine belong together so the window is not a dead form, but the engine is a trait object — not a hard-wired offline model.
**Delivers:** `xtools-json` — format, minify, validate, mark line/column in the editor (serde_json errors; parse off-thread if the paste is large). `xtools-trans` — input / output / language UI talking only to `TranslateEngine`; one working ureq-backed impl on a worker thread; no key in git; Secret Service before the first real request (empty engine / prompt if the keyring is missing). Same instance lock + focus path as timestamp. Menu stays three hardcoded binaries.
**Addresses:** JSON format/minify/error location; translate shell + one swappable engine.
**Avoids:** jq/JSONPath; boolean-only validation; API keys in source; blocking the UI thread; binding v1 to an offline dictionary; plugin directory "while we are here."

### Phase Ordering Rationale

- Overlay before tools: a beautiful timestamp window does not prove the product if the orb sinks or steals the desktop.
- Tokens before host paint, chrome before the second tool: visual unity is a compile-time crate, not a later restyle pass.
- Timestamp before JSON/translate: cheapest domain that still proves spawn, raise, clipboard, and chrome.
- JSON and translate after the template: clone, don't design twice. Engine secrets wait until the translate window exists.
- This order matches ARCHITECTURE build order and PITFALLS phase mapping. It refuses FEATURES anti-features (search box, plugin scan, multi-instance, hotkeys) by never scheduling them.

### Research Flags

Phases likely needing deeper research during planning:

- **Phase 1:** gtk4-layer-shell on *this* KWin: Overlay + shaped input region + exclusive_zone −1 + no keyboard + fractional scale. Sparse, compositor-specific. Use `/gsd-plan-phase --research-phase 1`.
- **Phase 2:** xdg-activation from a GTK host into an eframe child (token mint, env pass, `ViewportCommand::Focus` / winit activate). Cross-toolkit raise is the uncertain seam. Use `/gsd-plan-phase --research-phase 2`.

Phases with standard patterns (skip research-phase):

- **Phase 3:** serde_json pretty/minify/`Error::line()` is documented. `TranslateEngine` trait + ureq worker is a normal Rust pattern. Secret Service is a known crate (`keyring` / `oo7`). Plan from STACK + FEATURES; only spike if the chosen v1 engine's HTTP shape is unclear.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH (crates) / MEDIUM (host GUI) | Crate versions and eframe viewport APIs verified on crates.io + docs.rs. Host must *not* follow STACK's eframe-everywhere call; that part is overruled by compositor facts. |
| Features | MEDIUM | Locked by PROJECT.md and competitor official pages. Personal-use "table stakes" are the click path, not marketplace coverage. |
| Architecture | MEDIUM | Process model, workspace, socket lock, and activation taken from current specs. Seam-capped fetches. Split-toolkit host is the opinionated resolution of the STACK vs ARCHITECTURE conflict. |
| Pitfalls | MEDIUM | Protocol text (layer-shell, activation, input region, winit WindowLevel) is solid. KWin token behavior and NVIDIA/wgpu tool transparency need a live probe. |

**Overall confidence:** MEDIUM

The product shape and phase order are clear. The remaining uncertainty is live compositor behavior (does Overlay stay above a maximized window; does a click-serial token raise an eframe window), not what to build.

### Gaps to Address

- **KWin Overlay probe:** Phase 1 verification must put a maximized window over the orb on this session. Docs are not enough.
- **Cross-toolkit activation:** GTK host mints token → eframe tool focuses. If KWin ignores it, add an urgent/taskbar hint; only then consider D-Bus `org.freedesktop.Application`.
- **Which v1 translate engine:** Unspecified. Phase 3 picks one HTTP adapter that works; do not block the shell on the vendor.
- **wgpu vs glow on tools:** Only if a tool window is a black rectangle on this NVIDIA laptop. Do not change host toolkit for that.
- **Abstract socket vs pathname+flock:** Start abstract; switch if debugging needs a visible path.
- **GNOME:** Out of personal-v1 scope. `is_supported() == false` is a message or X11 fallback, not a Shell extension.

## Sources

### Primary (HIGH confidence)

- crates.io API (2026-08-19) — eframe/egui/egui_extras 0.36.1, gtk4 0.11.4, gtk4-layer-shell 0.8.1, jiff 0.2.35, serde_json 1.0.151, ureq 3.4.0, winit 0.30.13
- https://docs.rs/eframe/0.36.1/eframe/struct.NativeOptions.html — viewport, persist_window, wgpu default
- https://docs.rs/egui/0.36.1/egui/viewport/struct.ViewportBuilder.html — always_on_top, transparent, app_id
- https://docs.rs/egui/0.36.1/egui/viewport/enum.ViewportCommand.html — Focus, StartDrag, MousePassthrough
- https://docs.rs/winit/0.30.13/winit/window/enum.WindowLevel.html — **unsupported on Wayland**
- https://docs.rs/gtk4-layer-shell/0.8.1/gtk4_layer_shell/trait.LayerShell.html — `is_supported`, `init_layer_shell`, layer, exclusive zone, keyboard mode
- https://docs.gtk.org/gdk4/method.Surface.set_input_region.html — region vs alpha
- https://wayland.app/protocols/wlr-layer-shell-unstable-v1 — Overlay, exclusive_zone −1, configure-before-buffer, compositor table (KWin yes, Mutter no)
- https://wayland.app/protocols/xdg-activation-v1 — token from click serial; child must unset env
- https://doc.rust-lang.org/cargo/reference/workspaces.html — virtual manifest, default-members
- https://docs.rs/serde_json/1.0.151/serde_json/struct.Error.html — line(), column()
- https://docs.rs/jiff/0.2.35/jiff/struct.Timestamp.html — from_second / from_millisecond, RFC3339
- man7 `unix(7)` / `flock(2)` — abstract namespace, advisory lock
- XDG Base Directory Spec — `$XDG_RUNTIME_DIR` for sockets

### Secondary (MEDIUM confidence)

- uTools / Alfred / Raycast / Albert / Ulauncher / Wox / PowerToys / DevToys / Gnome-Pie / Rubick / CopyQ official pages — category split (search-box vs orbital); do not clone the box
- Kai Uwe Broulik, On Window Activation (2025-08) — cannot take focus; Extreme FSP needs a valid token
- GNOME mutter#973 — layer-shell absent on Mutter (irrelevant to daily KDE, relevant to `is_supported`)
- Desktop Entry Spec D-Bus Activation — fallback raise path, not v1 default
- Arch Wiki Clipboard — Wayland selection owned by the source process
- PROJECT.md — locked click path and out-of-scope list

### Tertiary (LOW confidence)

- NVIDIA + wgpu transparent tool windows on this laptop — probe if a tool is a black rectangle; glow fallback
- Whether KWin will honor an activation token delivered over an abstract socket from a layer-shell surface into an eframe xdg-shell toplevel — live test in Phase 2
- iced_layershell / raw smithay client as host alternatives — not needed if gtk4-layer-shell maps on this session

---
*Research completed: 2026-08-19*
*Ready for roadmap: yes*
