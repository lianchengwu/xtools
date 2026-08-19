# Phase 1: Host Orb and Orbital Menu - Research

**Researched:** 2026-08-19
**Domain:** GTK4 layer-shell Overlay host orb + orbital menu on KDE Plasma Wayland
**Confidence:** HIGH (crate APIs, protocol, workspace). MEDIUM (live KWin Overlay + input-region probe still required).

## User Constraints (from CONTEXT.md)

**CRITICAL:** Locked decisions are NON-NEGOTIABLE. Planner must honor these verbatim.

### Locked Decisions

#### Locked before this discussion
- **D-01:** Host is GTK4 + gtk4-layer-shell Overlay. Tool windows stay eframe. Do not revisit.
- **D-02:** Menu is three hardcoded entries (timestamp, JSON, translate). No plugin directory. No search box.
- **D-03:** Click main orb toggles expand/collapse. Not a hotkey. Not hover-to-open.

#### 主球外观
- **D-04:** Main orb is a ~40 logical-px dark disk with a light letter **x** centered in it.
- **D-05:** Not a solid unmarked disk, not a colored accent button, not translucent.

#### 三球怎么排
- **D-06:** Three function orbs fan in an arc **above** the main orb (not a 120° triangle, not a vertical list on the right).
- **D-07:** Each function orb is ~32 logical-px, slightly smaller than the main orb.
- **D-08:** Identify tools with a simple mark on the disk, no full name, no side label: clock (timestamp), `{}` (JSON), text/文 (translate).
- **D-09:** ROADMAP still requires expanded disks stay on the output so every ball remains clickable. If the fan would clip at the top edge, rotate or shrink just enough to keep every disk fully visible. Do not persist position.

#### 展开收起手感
- **D-10:** Expand is a short pop from the main orb to the fan seats (~100–180 ms). Collapse is the reverse. Not instant, not a long animation.
- **D-11:** Click-only. Hover does not expand or collapse.
- **D-12:** Collapse on: second click of the main orb, click of a function orb, or click anywhere that is not a function orb (including the transparent overlay / desktop). Any of those closes the menu.
- **D-13:** Outside-click dismiss requires the expanded overlay to receive those clicks. Collapsed state must keep the input region to the main disk only so the rest of the desktop stays click-through.

#### 拖和点怎么分
- **D-14:** Movement under ~6–8 logical px is a click. Beyond that is a drag.
- **D-15:** If a drag starts while the menu is open, collapse first, then drag only the main orb. Function orbs do not travel with the drag.
- **D-16:** First launch places the main orb at the **middle of the right screen edge** (vertically centered, inset so the full disk is on-screen).
- **D-17:** The entire main orb must stay inside the output while dragging. Clamp; never allow it off-screen.
- **D-18:** Do not persist orb position in this phase. Restart returns to D-16.

### the agent's Discretion
None — user picked a concrete option on every question.

### Deferred Ideas (OUT OF SCOPE)
- Persist orb position across restarts — HOST-05, v2
- Dedicated edge-aware constellation beyond the minimum “stay on output” clamp — HOST-04, v2
- Hover highlight on function orbs — not chosen; click-only
- Function orbs traveling with an in-menu drag — rejected; drag collapses first

## Project Constraints (from AGENTS.md)

Treat these with the same authority as CONTEXT.md locks.

- **Tech stack:** Rust. Host and tools are Rust window programs.
- **Process model:** Tools are independent window processes. Host owns the orb and later spawn/focus only. Phase 1 does **not** spawn tools.
- **Platform:** Linux desktop, personal use. This session is KDE Plasma Wayland.
- **v1 surface:** Three hardcoded entries. No plugin directory scan.
- **UI:** Shared theme tokens so later tool windows match. Phase 1 lands tokens; host paints from them.
- **Stack override:** AGENTS.md still embeds STACK.md's eframe-everywhere host. That is **overruled** by D-01 / SUMMARY.md. Do not put eframe, egui, or winit in `xtools-host`.
- **GSD workflow:** Implementation goes through GSD commands (`/gsd-execute-phase`). This research file is the planner input only.

## Summary

Phase 1 is a walking skeleton: a virtual Cargo workspace, an `xtools-ui` token crate, and an `xtools-host` GTK4 binary that is a single `zwlr_layer_shell_v1` Overlay surface. The host draws a 40 px dark disk with a light **x**, click-toggles three hardcoded function disks in a fan above it, and paints every disk from shared tokens. Function orbs are stubs. They must not open windows. A function-orb click must still stash the GDK pointer event so Phase 2 can mint an `xdg-activation-v1` token.

The only implementation path: one output-sized layer surface, `Layer::Overlay`, `exclusive_zone = -1`, `KeyboardMode::None`, cairo disks on a `DrawingArea`, compositor input region rebuilt on every state change, 8 logical-px slop, 120 ms tick-callback pop. Do not copy the official gtk4-layer-shell example's `Layer::Top` + `auto_exclusive_zone_enable()` — that is a panel sample and would steal work-area geometry.

This machine is already `XDG_SESSION_TYPE=wayland` / `XDG_CURRENT_DESKTOP=KDE`. KWin 6.6 implements layer-shell v5. Runtime `libgtk-4-1` 4.22.4 is installed; **`gtk4-devel` and `gtk4-layer-shell-devel` are not**. Wave 0 must install those before `cargo build`.

**Primary recommendation:** Build `crates/xtools-ui` tokens + `ToolId`, then `crates/xtools-host` as one GTK4 Overlay `ApplicationWindow` that fills the output, draws circles in cairo, and switches input region between "main disk only" (collapsed) and "entire surface" (expanded).

## Requirement Map

| ID | Behavior this phase must make true | Implementation owner |
|----|------------------------------------|----------------------|
| **HOST-01** | Always-on-top main orb; user can drag it | Overlay layer + exclusive_zone −1 + drag + clamp |
| **HOST-02** | Click main orb → three function orbs appear around it | Fan-above layout + 120 ms pop + hardcoded `ToolId` |
| **HOST-03** | Click main orb again → function orbs collapse | Toggle + reverse pop; also collapse on function-orb / outside click (D-12) |
| **LAUNCH-03** | Exactly three hardcoded entries; no plugin scan | `ToolId::{Time, Json, Trans}` in `xtools-ui`. No `plugins/` |
| **UI-01** | Same colors, type size, spacing later tools will use | `xtools-ui` token module. Host paints from tokens. No egui chrome yet |

Out of this phase: LAUNCH-01/02 spawn-or-focus, TIME/JSON/TRANS windows, UI-02 chrome, HOST-04/05.

## Architectural Responsibility Map

Single-process desktop overlay. No network, no backend, no database.

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Always-on-top orb (HOST-01) | Host process / compositor Overlay | — | Only layer-shell Overlay floats above a maximized window on KWin |
| Drag + on-screen clamp (HOST-01, D-16/17) | Host process | GDK Monitor geometry | Position is surface-local; compositor does not move a free-floating ball |
| Expand / collapse three orbs (HOST-02/03) | Host process, same surface | — | Menu is painted geometry, not three windows or three processes |
| Hardcoded menu entries (LAUNCH-03) | `xtools-ui` `ToolId` | Host paint/hit-test | Compile-time enum. No directory scan |
| Theme tokens (UI-01) | `xtools-ui` library | Host paint; later eframe tools | Tokens compile into every binary. No theme IPC |
| Outside-click dismiss (D-12/13) | Host input region + hit-test | Compositor | Expanded region must be large enough to receive the click |
| Pointer serial for later activation | Host event stash | Phase 2 launcher | Function orbs are stubs; serial still required |
| X11 fallback | Host, only if `is_supported() == false` | gdk4-x11 | Not the daily path on this KDE Wayland box |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust / cargo | rustc **1.97.1**, edition **2024** | Language / workspace | Already on the machine. [VERIFIED: `rustc --version`] |
| Cargo virtual workspace | resolver **3** | `xtools-ui` + `xtools-host`; `default-members = ["crates/xtools-host"]` | Cargo book: virtual manifest has no root `[package]`; resolver must be explicit. [CITED: doc.rust-lang.org/cargo/reference/workspaces.html] |
| **gtk4** | **0.11.4** (features `v4_12`) | Application, Window, DrawingArea, gestures, tick callback | Official gtk-rs. System GTK is 4.22.4 so `v4_12` (`Surface::scale`) is safe. [VERIFIED: `cargo info gtk4`] |
| **gtk4-layer-shell** | **0.8.1** (feature `v1_3`) | `is_supported`, `init_layer_shell`, `Layer::Overlay`, exclusive zone, keyboard mode | Safe gir wrapper for the C library this host requires. [VERIFIED: `cargo info gtk4-layer-shell`] |
| **gdk4** | **0.11.4** (via gtk4) | `SurfaceExt::set_input_region`, `DisplayExt::supports_input_shapes`, monitors, events | Input region is the click-through contract. [VERIFIED: docs.rs/gdk4/0.11.4] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| **cairo-rs** | **0.22.0** (via gtk4) | `Context::arc` / `fill` / `set_source_rgba`; `Region` of rectangles | Draw disks; build input region. Do not add a second graphics crate. [VERIFIED: `cargo info cairo-rs`] |
| **pango** | **0.22.8** (via gtk4) | Centered **x** / `{}` / 文 marks | `WidgetExt::create_pango_layout`. [VERIFIED: `cargo info pango`] |
| **glib** | **0.22.8** (via gtk4) | `ControlFlow` for tick callback | Transitive. [VERIFIED: `cargo info glib`] |
| **gdk4-wayland** | **0.11.4** | Optional Wayland extras for Phase 2 token mint | Phase 1: stash `gdk::Event` only. Add the crate now only if you need a Wayland-typed surface; otherwise defer. [VERIFIED: `cargo info gdk4-wayland`] |
| **gdk4-x11** | **0.11.4** | X11 fallback when `is_supported() == false` | Thin branch only. Not the daily path. [VERIFIED: `cargo info gdk4-x11`] |

Do **not** add to this phase: eframe, egui, winit, iced, slint, tauri, tokio, serde_json, jiff, ureq, keyring.

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Output-sized Overlay + shaped region | Four small layer surfaces (main + 3 orbs) | Extra protocol chatter, stacking fights. PITFALLS: one surface. |
| cairo `DrawingArea` | Custom widget `snapshot()` + GSK path | GSK is fine on GTK 4.22; cairo is the documented draw_func and is enough for disks + marks. |
| `WidgetExtManual::add_tick_callback` | `GtkTimedAnimation` / `CallbackAnimationTarget` | **Those types are not in gtk4 0.11.4 rustdocs** (docs.rs 404). Tick callback is the bound API. |
| `exclusive_zone = -1` | `auto_exclusive_zone_enable()` | Official example enables auto exclusive zone. That reserves panel geometry. Forbidden for a floating ball. |
| Layer-shell Overlay | eframe `with_always_on_top` | winit `WindowLevel` is unsupported on Wayland. Locked out by D-01. |

**Installation (after system devel packages):**

```bash
# Wave 0 — system deps (openSUSE, this machine). Not installed today.
sudo zypper in gtk4-devel gtk4-layer-shell-devel libgtk4-layer-shell0

# Workspace crates (planner writes manifests; do not cargo-add eframe)
# crates/xtools-host/Cargo.toml:
#   gtk4 = { version = "0.11.4", features = ["v4_12"] }
#   gtk4-layer-shell = { version = "0.8.1", features = ["v1_3"] }
#   xtools-ui = { workspace = true }
```

**Version verification (2026-08-19):**

| Package | `cargo info` version | crates.io published | Weekly downloads | Official docs |
|---------|----------------------|---------------------|------------------|---------------|
| gtk4 | 0.11.4 | 2019-07-25 | 50,104 | https://docs.rs/gtk4/0.11.4 |
| gtk4-layer-shell | 0.8.1 | 2023-04-12 | 5,632 | https://docs.rs/gtk4-layer-shell/0.8.1 |
| gdk4 | 0.11.4 | 2019-07-25 | 51,196 | https://docs.rs/gdk4/0.11.4 |
| cairo-rs | 0.22.0 | 2015-05-12 | 757,012 | https://docs.rs/cairo-rs/0.22.0 |
| pango | 0.22.8 | 2015-05-12 | 746,103 | https://gtk-rs.org |
| glib | 0.22.8 | 2015-05-12 | 993,433 | https://gtk-rs.org |

## Package Legitimacy Audit

> Ran `gsd-tools query package-legitimacy check --ecosystem crates` on 2026-08-19.

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| gtk4 | crates | since 2019 | 50k/wk | github.com/gtk-rs/gtk4-rs | OK | Approved |
| gtk4-layer-shell | crates | since 2023 | 5.6k/wk | github.com/pentamassiv/gtk4-layer-shell-gir | OK | Approved |
| gdk4 | crates | since 2019 | 51k/wk | github.com/gtk-rs/gtk4-rs | OK | Approved |
| gdk4-wayland | crates | since 2021 | 8.0k/wk | github.com/gtk-rs/gtk4-rs | OK | Approved (optional) |
| gdk4-x11 | crates | since 2021 | 7.3k/wk | github.com/gtk-rs/gtk4-rs | OK | Approved (fallback only) |
| cairo-rs | crates | since 2015 | 757k/wk | github.com/gtk-rs/gtk-rs-core | OK | Approved (transitive) |
| pango | crates | since 2015 | 746k/wk | github.com/gtk-rs/gtk-rs-core | OK | Approved (transitive) |
| glib | crates | since 2015 | 993k/wk | github.com/gtk-rs/gtk-rs-core | OK | Approved (transitive) |

**Packages removed due to [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

Authoritative confirmation: gtk-rs official docs + `cargo info` + legitimacy OK. Not `[ASSUMED]`.

## Architecture Patterns

### System Architecture Diagram

```
pointer / button
    │
    ▼
KWin (zwlr_layer_shell_v1 Overlay, this session)
    │  configure (size) → ack handled by gtk4-layer-shell
    ▼
xtools-host  (single process, single GtkWindow / GdkSurface)
    │
    ├─ is_supported()?
    │     no  → X11 fallback or one-line refuse (not a GNOME extension)
    │     yes → init_layer_shell → Overlay / exclusive_zone=-1 / KeyboardMode::None
    │
    ├─ DrawingArea.set_draw_func
    │     tokens from xtools-ui → cairo_arc disks + pango marks
    │
    ├─ GestureDrag + GestureClick (8 px slop)
    │     collapsed + click on main     → expand (120 ms tick)
    │     expanded  + click on main     → collapse
    │     expanded  + click on function → stash gdk::Event, collapse (no spawn)
    │     expanded  + click on glass    → collapse
    │     movement > 8 px               → if expanded, snap-collapse; drag main; clamp
    │
    └─ SurfaceExt::set_input_region
          collapsed → scanline union of main disk
          expanded  → None (entire surface reactive) so outside click arrives
```

Trace HOST-02: click main disk → slop not exceeded → `MenuState::Expanding` → tick callback interpolates three seats on the upper arc → input region becomes full surface → user sees three marked disks.

### Recommended Project Structure

Greenfield. Phase 1 creates **only** these two crates. Do not stub `xtools-time` / `xtools-json` / `xtools-trans`.

```
xtools/
├── Cargo.toml                      # virtual [workspace], resolver = "3"
├── crates/
│   ├── xtools-ui/                  # tokens + ToolId (+ host instance name)
│   │   ├── Cargo.toml              # no gtk4, no eframe
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── theme.rs            # colors, type size, spacing, radii
│   │       └── ids.rs              # ToolId::{Time, Json, Trans}
│   └── xtools-host/                # Overlay binary
│       ├── Cargo.toml              # gtk4, gtk4-layer-shell, xtools-ui
│       └── src/
│           ├── main.rs             # Application, is_supported, activate
│           ├── overlay.rs          # layer-shell init, anchors, exclusive zone
│           ├── paint.rs            # cairo disks + marks from tokens
│           ├── layout.rs           # fan-above seats, rotate/shrink, clamp
│           ├── input.rs            # slop, hit-test, input region, event stash
│           └── anim.rs             # 120 ms tick pop
└── target/
```

Root manifest (prescribe this shape):

```toml
[workspace]
resolver = "3"
members = ["crates/xtools-ui", "crates/xtools-host"]
default-members = ["crates/xtools-host"]

[workspace.package]
edition = "2024"
license = "MIT"
rust-version = "1.85"

[workspace.dependencies]
xtools-ui = { path = "crates/xtools-ui" }
```

`cargo run` from the repo root starts the orb. `cargo test --workspace` so default-members does not hide `xtools-ui` tests.

### Pattern 1: Overlay host, one surface

**What:** `gtk::ApplicationWindow`, `LayerShell::init_layer_shell()` **before** `present()`, then Overlay + fill the output.
**When to use:** Always, for `xtools-host`.
**Do not** copy the crate example's `Layer::Top` or `auto_exclusive_zone_enable()`.

```rust
// Source: https://github.com/pentamassiv/gtk4-layer-shell-gir/.../simple-example.rs
// Adapted: Overlay, exclusive_zone -1, KeyboardMode::None, all four anchors.
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

fn attach_overlay(window: &gtk::ApplicationWindow) {
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_namespace(Some("xtools"));
    window.set_keyboard_mode(KeyboardMode::None);
    for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
        window.set_anchor(edge, true);
    }
    window.set_exclusive_zone(-1);
}
```

`set_exclusive_zone` has no effect unless the surface is anchored. Anchor all four edges so the compositor assigns the output size (`set_size(0,0)` is the protocol default when opposite edges are anchored). gtk4-layer-shell performs the initial no-buffer commit; do not attach your own `wl_buffer` first. [CITED: wayland.app/protocols/wlr-layer-shell-unstable-v1]

### Pattern 2: Tokens first, host paints from them

**What:** `xtools-ui::theme` exports logical sizes and colors. Host never hardcodes `#1a1a1a` or `40.0`.
**When to use:** Every paint and layout number in Phase 1.

Prescribe these token names (planner may rename fields but not the values):

| Token | Value | Used by |
|-------|-------|---------|
| `color.orb_fill` | dark opaque disk (not accent, not translucent) | main + function disks |
| `color.orb_mark` | light mark | **x**, clock, `{}`, 文 |
| `type.mark_px` | ~16–18 logical px | pango font size |
| `space.main_d` | 40 | main diameter |
| `space.func_d` | 32 | function diameter |
| `space.gap` | ≥ 8 | disk-to-disk gap |
| `space.slop` | 8 | click vs drag |
| `space.pop_ms` | 120 | tick animation |

UI-01 is satisfied for this phase when later eframe tools can `use xtools_ui::theme` and get the same numbers. Do not add egui `Visuals` yet.

### Pattern 3: Collapsed vs expanded input region

**What:** Alpha is not input. `gdk_surface_set_input_region(NULL)` makes the **entire** surface reactive. [CITED: docs.gtk.org/gdk4/method.Surface.set_input_region.html]

| Menu state | Input region | Why |
|------------|--------------|-----|
| Collapsed / collapsing-finished | Scanline `cairo::Region` of the **main disk only** | Desktop click-through (D-13) |
| Expanding / expanded | `set_input_region(None)` — whole surface | Outside click can dismiss (D-12/13) |
| Dragging after snap-collapse | Main disk only, rebuilt on drag **end** (and on scale/monitor change) | Do not rebuild every motion pixel |

`cairo::Region` is rectangles only (`create_rectangle` / `union_rectangle`). [CITED: docs.rs/cairo-rs/0.22.0/cairo/struct.Region.html] A bounding square would steal the disk's corners. Build the disk as horizontal scanlines (`for y in -r..=r { w = sqrt(r²−y²); union row }`). In-process hit-test is still `dx*dx + dy*dy <= r*r`.

Call `NativeExt::surface()` after realize; if `DisplayExt::supports_input_shapes()` is false, refuse (modern KWin returns true). [CITED: docs.rs/gdk4/0.11.4 SurfaceExt / DisplayExt]

Rebuild region on: expand start, collapse end, drag end, `scale` notify, enter/leave monitor.

### Pattern 4: 8 px slop, then click or drag

**What:** One `GestureDrag` (begin / update / end) plus circle hit-test. Do not trust `GestureClick` alone — a 1 px jitter would never toggle.
**When to use:** All pointer interaction on the overlay.

```
press on a disk:
  record origin, last_event
  if movement <= 8 logical px and release still on that disk → CLICK
  if movement > 8 → DRAG
```

`WidgetExt::drag_check_threshold` exists but uses the GTK setting, not D-14. Implement **8.0** logical px explicitly.

Click routing:

1. Main disk, collapsed → expand.
2. Main disk, expanded → collapse.
3. Function disk → clone `GestureExt::last_event()`, collapse, **do not spawn**.
4. Glass (expanded only) → collapse.

Drag routing:

1. If menu is open when slop is exceeded: **snap-collapse** (skip reverse pop) then move only the main orb (D-15).
2. `main_x/y += delta`; clamp so the full main disk stays inside the monitor geometry (D-17).
3. Function orbs do not travel.

### Pattern 5: Fan above, then rotate/shrink to stay on output

**What:** Seats live on an arc above the main center. This is Phase 1 minimum stay-on-output (D-09), not HOST-04.

Prescribe:

- Orbit radius `R = r_main + r_func + gap` with `gap >= 8` so disks never overlap.
- Rest angles (radians, 0 = +x, CCW): `−150°`, `−90°`, `−30°` (left-up, up, right-up). Map `ToolId::Time / Json / Trans` left-to-right.
- If any function disk would leave the output: rotate the whole constellation toward the interior in 15° steps; if still clipped, shrink `R` down to `r_main + r_func + gap` minimum then further only until all disks fit. **Never move the main orb to make the fan fit.**
- First launch: monitor geometry from `DisplayExt::monitor_at_surface` (or first monitor). `cx = geo.x + geo.width - r_main - inset`, `cy = geo.y + geo.height / 2`. `inset` ≥ 0 so the disk is fully inside.

### Pattern 6: 120 ms tick pop — not TimedAnimation

**What:** `WidgetExtManual::add_tick_callback`. Drive `t` from `FrameClock::frame_time()`. Ease-out cubic. 120 ms (inside D-10's 100–180). Collapse is the same curve reversed.
**When to use:** Expand and click-collapse. Not for drag-initiated snap-collapse.

`GtkTimedAnimation` / `CallbackAnimationTarget` are **not present** in gtk4 0.11.4 rustdocs. Do not plan them. [CITED: docs.rs/gtk4/0.11.4 — struct 404; docs.rs/gtk4/0.11.4/gtk4/prelude/trait.WidgetExtManual.html `add_tick_callback`]

Tick must `queue_draw()`. It does not paint by itself. [CITED: docs.gtk.org/gtk4/method.Widget.add_tick_callback.html]

### Pattern 7: Stash the pointer event, do not launch

**What:** On function-orb **click** (after slop), store `Option<gdk::Event>` from `GestureExt::last_event(None)`. `gdk::Event` exposes `time()`, `surface()`, `seat()`, `position()` — not a public Wayland serial. Phase 2 will mint via `DisplayExt::app_launch_context()` using that event. [CITED: docs.rs/gdk4/0.11.4/gdk4/struct.Event.html]

Do not `Command::new`. Do not `env_clear`. Do not implement socket wake of tools.

Host itself should still be single-instance: bind abstract `\0xtools-host` in `main` before creating the application. Second process exits. Implementation can live in `xtools-ui` as a tiny helper so Phase 2 reuses the pattern.

### Anti-Patterns to Avoid

- **eframe / winit AlwaysOnTop host:** no-op on Wayland. Locked out.
- **Official layer-shell example as-is:** `Layer::Top` + `auto_exclusive_zone_enable()` steals a panel strip.
- **`set_input_region` once at startup:** expanded glass then bricks the desktop, or outside click never arrives.
- **Bounding-box input region for a circle:** transparent corners steal clicks when collapsed.
- **Four layer surfaces / four processes for orbs:** stacking fights; D-15 collapse-then-drag becomes protocol hell.
- **`KeyboardMode::Exclusive`:** steals Super/Alt and terminal typing.
- **Hover-to-open, hotkey, plugin scan, persist position, tool windows.**
- **ARCHITECTURE.md "orbs travel with the drag":** overruled by D-15.
- **Double-scaling with `scale_factor`:** layout and region stay in surface-local logical units. Let GTK own the buffer scale. Use `SurfaceExt::scale()` (v4_12) only to detect change and rebuild.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Always-on-top on Wayland | winit `WindowLevel`, `_NET_WM_STATE_ABOVE` on the daily path | `gtk4_layer_shell` Overlay | Protocol-owned z-order. KWin implements v5 |
| Click-through glass | CSS opacity tricks, `MousePassthrough` | `SurfaceExt::set_input_region` | Alpha ≠ input |
| Click vs drag | Pixel-perfect press==release | 8 px slop on `GestureDrag` | D-14 |
| 120 ms pop | `std::thread::sleep`, glib timeout redraw hacks | `add_tick_callback` + frame clock | vsync; official animation hook |
| Circle paint | Pixel loops, PNG sprites | `cairo::Context::arc` + `fill` | One call, antialiased |
| Circle input mask | Custom compositor protocol | `cairo::Region` scanlines | GDK takes a cairo region |
| Theme consistency | Hex literals in host | `xtools-ui::theme` | UI-01 |
| Virtual workspace | Root package + bins mixed | Virtual manifest, `default-members` | Cargo book |

**Key insight:** The hard problems are compositor contracts (layer, exclusive zone, input region, keyboard mode). GTK already binds them. Custom window managers and custom physics are how this phase fails.

## Common Pitfalls

### Pitfall 1: Panel sample settings on a floating orb
**What goes wrong:** Maximized windows leave a ball-sized gap; orb sits in the Top layer under fullscreen chrome.
**Why it happens:** Official example calls `auto_exclusive_zone_enable()` and `set_layer(Layer::Top)`.
**How to avoid:** `Layer::Overlay`, `set_exclusive_zone(-1)`, never `auto_exclusive_zone_enable`. Verify: maximize a browser; work area is full width/height; orb still visible on top.
**Warning signs:** `auto_exclusive_zone_enable` in host code; `Layer::Top`.

### Pitfall 2: Full-surface input region while collapsed
**What goes wrong:** Desktop is unclickable around the orb. Or the opposite: expanded glass is click-through so outside-click dismiss never fires.
**Why it happens:** `set_input_region(None)` means the whole surface. Collapsed and expanded have opposite requirements (D-13).
**How to avoid:** Two-state region (Pattern 3). Rebuild on expand start and collapse end.
**Warning signs:** Cannot click a terminal next to the collapsed orb; cannot dismiss by clicking the desktop.

### Pitfall 3: Rectangular hit-test / bounding-box region
**What goes wrong:** Clicks in the square corners of a circular sprite steal the desktop or miss the disk.
**Why it happens:** cairo regions are rectangles; widgets are rectangles.
**How to avoid:** Scanline disk region + `dx²+dy² ≤ r²` in process.
**Warning signs:** Hover cursor changes over empty corners.

### Pitfall 4: No slop, or orbs covering the main disk
**What goes wrong:** Cannot toggle without moving; or cannot move without toggling; next click hits a function orb.
**Why it happens:** 1 px drag; `R ≤ r_main + r_func`.
**How to avoid:** slop = 8; `R >= r_main + r_func + 8`.
**Warning signs:** Menu opens when the user meant to drag.

### Pitfall 5: `init_layer_shell` after map / buffer before configure
**What goes wrong:** Protocol error `already_constructed`; surface never maps.
**Why it happens:** `present()` then init. Protocol: no buffer until first configure.
**How to avoid:** `init_layer_shell` + layer/zone/keyboard/anchors **before** `present()`, as the official example does. Let gtk4-layer-shell ack configure.
**Warning signs:** Crash on first run; empty window.

### Pitfall 6: `is_supported()` ignored
**What goes wrong:** Host dies on Mutter / Weston / Cage. Those compositors have no `zwlr_layer_shell_v1` (wayland.app table: Mutter 49.2 = x, KWin 6.6 = 5).
**Why it happens:** wlroots-centric tutorials skip the check.
**How to avoid:** Call `gtk4_layer_shell::is_supported()` first (may roundtrip). False → one-line stderr and X11 fallback or exit. No GNOME Shell extension.
**Warning signs:** No `is_supported` branch.

### Pitfall 7: HiDPI double-scale
**What goes wrong:** Region is 2× too small; clicks miss; drag jumps.
**Why it happens:** Multiplying logical layout by `scale_factor` after GTK already used surface-local units.
**How to avoid:** Store position in logical surface-local units. Rebuild on `connect_scale_notify`.
**Warning signs:** Works at 1×, broken on this laptop's scaled panel.

### Pitfall 8: Exclusive keyboard
**What goes wrong:** Super/Alt and terminal typing die while the orb exists.
**Why it happens:** Overlay + Exclusive is documented for lock screens.
**How to avoid:** `KeyboardMode::None` always in Phase 1.
**Warning signs:** `KeyboardMode::Exclusive` or `OnDemand`.

### Pitfall 9: Missing system devel packages
**What goes wrong:** `pkg-config` cannot find `gtk+-4.0` / `gtk4-layer-shell-0`. `cargo build` fails.
**Why it happens:** This machine has `libgtk-4-1` runtime only. `gtk4-devel` and `gtk4-layer-shell-devel` are in repo-oss but **not installed**.
**How to avoid:** Wave 0 installs them. Do not vendor the C library.
**Warning signs:** `Package gtk+-4.0 was not found`.

### Pitfall 10: Planning tool windows or persist
**What goes wrong:** Scope explodes; chrome invented twice; position file fights D-18.
**Why it happens:** Core value mentions spawn-or-focus.
**How to avoid:** Function orbs stash an event and collapse. Restart returns to the right-edge seat.

## Code Examples

### Detect layer-shell and refuse or continue

```rust
// Source: https://docs.rs/gtk4-layer-shell/0.8.1/gtk4_layer_shell/fn.is_supported.html
if !gtk4_layer_shell::is_supported() {
    eprintln!("xtools-host: zwlr_layer_shell_v1 not available; need layer-shell or X11");
    // X11 fallback branch only. Do not crash with a protocol error.
    return;
}
```

### Init Overlay before map

```rust
// Source: https://github.com/pentamassiv/gtk4-layer-shell-gir/.../simple-example.rs
// (layer / exclusive zone changed — do not copy Top + auto_exclusive_zone)
let window = gtk::ApplicationWindow::new(app);
window.set_decorated(false);
window.set_resizable(false);
window.init_layer_shell(); // BEFORE present
window.set_layer(gtk4_layer_shell::Layer::Overlay);
window.set_exclusive_zone(-1);
window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);
```

### Draw a disk

```rust
// Source: https://docs.rs/cairo-rs/0.22.0/cairo/struct.Context.html
//          https://docs.rs/gtk4/0.11.4/gtk4/prelude/trait.DrawingAreaExtManual.html
area.set_draw_func(move |_, cr, _w, _h| {
    cr.set_source_rgba(fill.r, fill.g, fill.b, 1.0);
    cr.arc(cx, cy, radius, 0.0, std::f64::consts::TAU);
    let _ = cr.fill();
});
```

### Set a circular-ish input region

```rust
// Source: https://docs.rs/gdk4/0.11.4/gdk4/prelude/trait.SurfaceExt.html
//          https://docs.rs/cairo-rs/0.22.0/cairo/struct.Region.html
fn disk_region(cx: f64, cy: f64, r: f64) -> cairo::Region {
    let region = cairo::Region::create();
    let r_i = r.ceil() as i32;
    for dy in -r_i..=r_i {
        let w = ((r * r) - (dy as f64 * dy as f64)).sqrt().floor() as i32;
        let _ = region.union_rectangle(&cairo::RectangleInt::new(
            cx as i32 - w,
            cy as i32 + dy,
            w * 2 + 1,
            1,
        ));
    }
    region
}
```

### Tick-callback pop

```rust
// Source: https://docs.rs/gtk4/0.11.4/gtk4/prelude/trait.WidgetExtManual.html
//          https://docs.gtk.org/gtk4/method.Widget.add_tick_callback.html
widget.add_tick_callback(move |w, clock| {
    let now = clock.frame_time(); // µs
    let t = ((now - start) as f64 / 120_000.0).clamp(0.0, 1.0);
    let eased = 1.0 - (1.0 - t).powi(3);
    // interpolate orb positions; w.queue_draw();
    if t >= 1.0 {
        glib::ControlFlow::Break
    } else {
        glib::ControlFlow::Continue
    }
});
```

### Stash the click event for Phase 2

```rust
// Source: https://docs.rs/gtk4/0.11.4/gtk4/prelude/trait.GestureExt.html
//          https://docs.rs/gdk4/0.11.4/gdk4/struct.Event.html
if let Some(ev) = gesture.last_event(None) {
    state.last_pointer_event = Some(ev); // clone; keep time() + surface()
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| GTK3 `gtk_window_set_keep_above` | Removed in GTK4; use layer-shell Overlay | GTK 4.0 | Host cannot be a normal toplevel |
| winit `WindowLevel::AlwaysOnTop` | Unsupported on Wayland | winit 0.30 docs | eframe host is a dead end here |
| Exclusive zone 0 (default) | `exclusive_zone = -1` for overlays that must not move | wlr-layer-shell | Default 0 lets KWin shove the ball |
| `GtkTimedAnimation` in C | **Not bound** in gtk4 0.11.4 | gtk-rs 0.11 | Use `add_tick_callback` |
| Integer `scale_factor` only | `SurfaceExt::scale()` fractional | GTK 4.12 / gtk4 `v4_12` | Enable the feature; listen for scale notify |
| eframe-everywhere (STACK.md) | Split toolkit (SUMMARY.md, D-01) | 2026-08-19 project lock | Ignore STACK host chapter |

**Deprecated/outdated:**
- `auto_exclusive_zone_enable` on a floating toy
- `KeyboardMode::Exclusive` on the orb
- `single-instance` crate / pidfile (not this phase's raise path anyway)
- Root Cargo package mixed with workspace members

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Snap-collapse (skip reverse pop) on drag-start is the right reading of D-10 + D-15 | Pattern 4 | If user wants the reverse pop even while dragging, add 120 ms before the orb follows the pointer — worse feel |
| A2 | Rest angles −150/−90/−30 are the intended “fan above” | Pattern 5 | Visual only; D-06 does not name degrees |
| A3 | `DisplayExt::app_launch_context()` plus a stashed `gdk::Event` is enough for Phase 2 to recover a serial | Pattern 7 | If KWin needs a raw wl serial, Phase 2 adds gdk4-wayland. Phase 1 still must stash the Event |
| A4 | Transparent window via CSS `window { background: transparent; }` + `set_decorated(false)` is sufficient on KWin | Overlay setup | If the surface is opaque black, add `css` probe; do not switch toolkit |
| A5 | First monitor / `monitor_at_surface` after map is the “right screen edge” | D-16 | Multi-monitor: bind `set_monitor` to the output the user is on; if unclear, primary/current |

A1–A2 are visual/interaction taste inside locked decisions — planner should use them, not re-ask. A3–A5 are live-session probes, not product forks.

## Open Questions

1. **Does Overlay + shaped region actually stay above a maximized window on this KWin?**
   - What we know: protocol + KWin 6.6 compositor table say yes; session is KDE Wayland.
   - What's unclear: NVIDIA + this laptop has not been live-probed.
   - Recommendation: Phase verification, not a design fork. Put a maximized browser over the orb.

2. **X11 fallback depth if `is_supported()` is false on some future session**
   - What we know: GTK4 removed `keep_above`. gdk4-x11 exists at 0.11.4.
   - What's unclear: exact `_NET_WM_STATE_ABOVE` helper in gdk4-x11 0.11.4 (not fetched line-by-line).
   - Recommendation: Daily path is Wayland. Fallback is a one-line refuse **or** a thin X11 branch — do not block Overlay work on a perfect X11 shape.

3. **Raw Wayland serial vs `gdk::Event::time()`**
   - What we know: Event has `time()`, `surface()`, `seat()`. No public `serial()` on gdk4 0.11.4 Event.
   - What's unclear: whether `AppLaunchContext` reads the serial from a cloned Event in Phase 2.
   - Recommendation: Phase 1 stores the whole `gdk::Event`. Phase 2 researches minting.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| rustc / cargo | workspace | ✓ | 1.97.1 | — |
| Wayland session | Overlay | ✓ | `wayland` / KDE / `wayland-0` | — |
| libgtk-4 runtime | host | ✓ | 4.22.4 (`libgtk-4-1`) | — |
| **gtk4-devel** | `pkg-config gtk+-4.0` | **✗** | repo-oss 4.22.4+29-1.2 | Install Wave 0 |
| **gtk4-layer-shell-devel** + `libgtk4-layer-shell0` | gtk4-layer-shell crate | **✗** | repo-oss 1.3.0+git21 | Install Wave 0 |
| cairo-devel / glib2-devel | gtk4-devel deps | ✓ | installed | — |
| KWin layer-shell | Overlay | ✓ (compositor table v5) | KWin 6.6 listed | Live probe |
| Knowledge graph | research | ✗ | no `.planning/graphs/graph.json` | Skip |

**Missing dependencies with no fallback:**
- `gtk4-devel` and `gtk4-layer-shell-devel` — `cargo build` cannot succeed until installed.

**Missing dependencies with fallback:**
- gdk4-x11 path if a non-KDE session lacks layer-shell.

## Security Domain

`security_enforcement: true`, ASVS level 1. This phase is a local always-on-top widget with no network and no accounts.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | No accounts |
| V3 Session Management | no | No sessions |
| V4 Access Control | yes (local) | Host single-instance abstract socket; if used, check `SO_PEERCRED` same uid. Do not world-listen |
| V5 Input Validation | yes | Pointer coordinates only. Clamp to output. Circle hit-test. No text fields |
| V6 Cryptography | no | No secrets, no TLS. Do not add a keyring “while here” |

### Known Threat Patterns for a layer-shell overlay

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Exclusive keyboard grab captures passwords meant for other apps | Information disclosure / Elevation | `KeyboardMode::None` |
| Overlay input region bricks the desktop (availability) | Denial of service | Collapsed region = main disk only |
| Second host instance confuse / spoof the orb | Spoofing | Abstract `\0xtools-host`; same-uid only |
| Logging pointer events forever | Information disclosure | Do not log coordinates at info level |
| Future spawn with `env_clear` drops session sockets | Denial of service | Do not touch process env this phase |

No user content crosses a trust boundary. Do not add HTTP, plugin loading, or D-Bus exported methods in Phase 1.

## Sources

### Primary (HIGH confidence)
- `cargo info` 2026-08-19 — gtk4 0.11.4, gtk4-layer-shell 0.8.1, gdk4 0.11.4, cairo-rs 0.22.0, pango 0.22.8, glib 0.22.8, gdk4-wayland/x11 0.11.4
- `gsd-tools query package-legitimacy check --ecosystem crates` — all OK
- https://docs.rs/gtk4-layer-shell/0.8.1/gtk4_layer_shell/ — `is_supported`, `LayerShell`, `Layer::Overlay`, `KeyboardMode::None`
- https://docs.rs/gtk4-layer-shell/0.8.1/gtk4_layer_shell/fn.is_supported.html
- https://github.com/pentamassiv/gtk4-layer-shell-gir/blob/main/gtk4-layer-shell/examples/simple-example.rs — init-before-present order (do not copy Top / auto exclusive)
- https://docs.rs/gdk4/0.11.4/gdk4/prelude/trait.SurfaceExt.html — `set_input_region`
- https://docs.gtk.org/gdk4/method.Surface.set_input_region.html — NULL = whole surface; alpha ≠ input
- https://docs.gtk.org/gdk4/method.Display.supports_input_shapes.html
- https://docs.rs/gdk4/0.11.4/gdk4/prelude/trait.DisplayExt.html — `supports_input_shapes`, `app_launch_context`, `monitor_at_surface`
- https://docs.rs/gtk4/0.11.4/gtk4/prelude/trait.DrawingAreaExtManual.html — `set_draw_func`
- https://docs.rs/gtk4/0.11.4/gtk4/prelude/trait.WidgetExtManual.html — `add_tick_callback`
- https://docs.gtk.org/gtk4/method.Widget.add_tick_callback.html
- https://docs.rs/gtk4/0.11.4/gtk4/prelude/trait.WidgetExt.html — `drag_check_threshold`, `queue_draw`, `add_controller`
- https://docs.rs/gtk4/0.11.4/gtk4/prelude/trait.GestureExt.html — `last_event`
- https://docs.rs/gtk4/0.11.4/gtk4/struct.GestureClick.html / `GestureDrag`
- https://docs.rs/gtk4/0.11.4/gtk4/prelude/trait.GtkWindowExt.html — `set_decorated`, `present`
- https://docs.rs/gtk4/0.11.4/gtk4/prelude/trait.NativeExt.html — `surface()`
- https://docs.rs/cairo-rs/0.22.0/cairo/struct.Context.html — `arc`, `fill`, `set_source_rgba`
- https://docs.rs/cairo-rs/0.22.0/cairo/struct.Region.html — rectangle union only
- https://docs.rs/gdk4/0.11.4/gdk4/struct.Event.html — `time`, `surface`, `seat` (no serial)
- https://wayland.app/protocols/wlr-layer-shell-unstable-v1 — Overlay, exclusive_zone −1, keyboard none, configure-before-buffer, compositor table (KWin 6.6 = v5, Mutter = absent)
- https://doc.rust-lang.org/cargo/reference/workspaces.html — virtual workspace, resolver, default-members
- `pkg-config` / `rpm` / `zypper se` 2026-08-19 — GTK runtime present; devel + layer-shell missing
- `printenv` — `wayland`, `KDE`, `wayland-0`

### Secondary (MEDIUM confidence)
- `.planning/research/SUMMARY.md` — split toolkit lock; ignore STACK eframe host
- `.planning/research/PITFALLS.md` — exclusive zone, input region, slop, no buffer before configure
- `.planning/research/ARCHITECTURE.md` — virtual workspace; **ignore** “orbs travel with drag” (D-15 wins)
- `.planning/research/STACK.md` — crate pins only
- gtk4-layer-shell README — needs system `gtk4` + `gtk4-layer-shell` C library ≥ matching 1.1/1.3

### Tertiary (LOW confidence — validate during implementation)
- Live KWin Overlay above a maximized window on this NVIDIA laptop
- Whether CSS transparency is enough or the first frame is an opaque black rectangle
- Whether a cloned `gdk::Event` still carries a usable activation serial in Phase 2

## Metadata

**Research scope:**
- Core technology: GTK4 0.11.4 + gtk4-layer-shell 0.8.1 Overlay on KDE Plasma Wayland
- Ecosystem: gtk-rs, cairo-rs, Cargo virtual workspaces
- Patterns: one surface, shaped input region, slop, tick pop, token crate
- Pitfalls: example exclusive zone, full-surface region, missing devel packages, eframe host

**Confidence breakdown:**
- Standard stack: HIGH — cargo info + official docs + legitimacy OK
- Architecture: HIGH for the prescribed path; MEDIUM until KWin Overlay is live-verified
- Pitfalls: HIGH — protocol text + official example anti-patterns
- Code examples: HIGH for cited APIs; MEDIUM for scanline region helper (composed from Region API, not copied from a sample)

**Research date:** 2026-08-19
**Valid until:** 2026-09-18 (30 days; gtk-rs 0.11 / layer-shell 0.8 are stable)

---

*Phase: 01-host-orb-and-orbital-menu*
*Research completed: 2026-08-19*
*Ready for planning: yes*
