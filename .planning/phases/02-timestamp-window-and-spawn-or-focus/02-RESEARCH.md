# Phase 2: Timestamp Window and Spawn-or-Focus - Research

**Researched:** 2026-08-22
**Domain:** GTK4 host spawn-or-focus of an eframe 0.36.1 timestamp window on KDE Plasma Wayland
**Confidence:** HIGH (crate APIs, jiff convert, instance bind, spawn path). MEDIUM (KWin honors a socket-delivered token into an already-mapped eframe toplevel).

## User Constraints

**CRITICAL:** Locked decisions are NON-NEGOTIABLE. Planner must honor these verbatim.

### Locked Decisions

#### Carried from Phase 1 / project lock
- **D-01:** Host remains GTK4 + gtk4-layer-shell. This window is eframe, not Overlay, not always-on-top.
- **D-02:** Clock orb is `ToolId::Time`. Host already stashes `gdk::Event` on function-orb click. Use that for spawn/focus.
- **D-03:** Single instance per tool. Abstract socket + raise. Second process must not open a second window.
- **D-04:** Shared colors/type/spacing come from `xtools-ui` tokens. This phase adds the egui chrome (`apply_theme`, title area, fields, buttons) that Phase 3 will copy.

#### 窗口里怎么排
- **D-05:** Vertical stack. Top: Unix seconds field and Unix milliseconds field. Bottom: one editable local datetime field. Edit any one field; the other two update immediately.
- **D-06:** Seconds and milliseconds are two separate boxes, kept in sync (ms = s × 1000).
- **D-07:** Datetime is local timezone, editable. No extra UTC row. No RFC3339 editor.
- **D-08:** Copy buttons sit beside the timestamp fields only (seconds, milliseconds). Not beside datetime.

#### 打开时默认填什么
- **D-09:** First launch (and launch after the process exited) fills all three fields from the current instant.
- **D-10:** If the window is already open, a later clock-orb click focuses it and does **not** overwrite fields.
- **D-11:** A 「现在」 button sits next to the local datetime field. Clicking it writes the current instant into all three fields.

#### 自定义格式 / 复制范围
- **D-12:** Do not ship a custom format box. Do not ship RFC3339 or datetime copy buttons. TIME-02 in this phase means: one-click copy of the 10-digit seconds value and the 13-digit milliseconds value.
- **D-13:** RFC3339 copy and custom strftime copy are deferred.

#### 窗口出现在哪
- **D-14:** First show: centered on the current output. About 560×480. Ordinary decorated eframe toplevel.
- **D-15:** Closing the window exits the timestamp process. Next clock-orb click starts a new process and fills "now" (D-09).
- **D-16:** The timestamp window must **not** appear on the taskbar (skip-taskbar / Utility hint). Host already does not. Phase 3 tools inherit this unless revisited.
- **D-17:** Title bar: `xtools · 时间戳` (agent discretion; suite prefix for later tools).

### the agent's Discretion
- Exact egui widget spacing beyond tokens.
- How skip-taskbar is spelled on Wayland vs X11 (Utility, `skip_taskbar`, app_id `dev.xtools.timestamp`).
- Invalid input: keep the last good peer fields and show a short inline error; do not crash.

### Deferred Ideas (OUT OF SCOPE)
- RFC3339 one-click copy — user declined for this phase
- Custom strftime format + copy — user declined
- Clipboard-auto-fill on open — not chosen
- Persist last values / last window position — v2
- UTC companion row — not chosen

**ROADMAP override:** Phase 2 success criterion 4 still names “10-digit, 13-digit, RFC3339, or a custom format”. CONTEXT D-12/D-13 and 02-UI-SPEC narrow TIME-02 for this phase to seconds + milliseconds only. Planner MUST honor CONTEXT, not the older ROADMAP sentence. Do not add RFC3339 or custom-format copy UI.

## Project Constraints (from AGENTS.md)

Treat these with the same authority as CONTEXT.md locks, except the stack-override note.

- **Tech stack:** Rust. Host and tools are Rust window programs.
- **Process model:** Tools are independent window processes. Host owns the orb and spawn/focus only.
- **Platform:** Linux desktop, personal use. This session is KDE Plasma Wayland.
- **v1 surface:** Three hardcoded entries. No plugin directory scan. This phase launches **only** `ToolId::Time`. JSON and translate orbs still collapse the menu and do not spawn.
- **UI:** Shared theme/controls. Chrome lives in `xtools-ui`. `xtools-time` does not invent a second look.
- **Stack override:** AGENTS.md still embeds STACK.md’s eframe-everywhere host and “four small windows”. That is **overruled** by D-01 / SUMMARY.md / Phase 1. Do not put eframe, egui, or winit in `xtools-host`. Do not make the timestamp window layer-shell or always-on-top.
- **Raise path (AGENTS.md, still valid for tools):** Unix socket + `ViewportCommand::Focus` + `XDG_ACTIVATION_TOKEN`. Not `single-instance`. Not `wmctrl` / `_NET_ACTIVE_WINDOW` on this Wayland session.
- **Time crate (AGENTS.md, still valid):** jiff 0.2.35. Not chrono.
- **Copy (AGENTS.md, still valid):** `ctx.copy_text`. Not arboard unless a path is outside egui.
- **GSD workflow:** Implementation goes through GSD commands. This file is planner input only.

## Summary

Phase 2 is the first satellite process. Clicking the clock orb (`ToolId::Time`) must either start `xtools-time` next to the host binary or raise the live instance. The window is an ordinary decorated eframe 0.36.1 toplevel that converts Unix seconds / milliseconds ↔ local datetime with jiff and copies only the two integer timestamp fields. Closing it must quit the process so the abstract socket dies and the next click is a fresh “now” fill.

There is one spawn path and one convert path. Host locates `xtools-time` as a sibling of `std::env::current_exe()`, mints an activation token from the stashed `gdk::Event` via `DisplayExt::app_launch_context` + `set_timestamp` + `startup_notify_id`, then **connects** `\0xtools-time` first. Connected: write `RAISE` + token and return. Not connected: `Command::new(sibling)` inheriting the session env plus `XDG_ACTIVATION_TOKEN`. The child calls `claim_instance("xtools-time")`. Bind wins → show the window. Bind loses → forward RAISE and **exit 0 without mapping a viewport**. A failed token or ignored `ViewportCommand::Focus` is not a license to spawn a second window.

Chrome (`apply_theme`, title strip, labeled field, copy button, inline error) is compiled into the tool from `xtools-ui` behind an `egui-chrome` feature so the host does not link egui. TIME-02 this phase is seconds + milliseconds only. No RFC3339 copy. No custom format box.

**Primary recommendation:** Extend `xtools-ui` with `TIME_INSTANCE`, `raise_instance`, and egui chrome (feature-gated). Add `crates/xtools-time`. On clock-orb click, host mint-token → raise-or-spawn sibling `xtools-time`. Child claim-or-forward. Convert with `Timestamp::from_second` / `from_millisecond` / `to_zoned(TimeZone::system())`. Close quits.

## Requirement Map

| ID | Behavior this phase must make true | Implementation owner |
|----|------------------------------------|----------------------|
| **LAUNCH-01** | Clock-orb click opens an independent Rust timestamp window | Host `Command::new(sibling "xtools-time")` after raise-connect fails |
| **LAUNCH-02** | Second clock-orb click focuses the live window; no second copy | `claim_instance("xtools-time")` + socket `RAISE` + `ViewportCommand::Focus`. Never spawn when connect succeeds |
| **TIME-01** | Convert Unix s or ms ↔ local datetime, and the other two fields follow | jiff `Timestamp` + `TimeZone::system()` in `xtools-time` |
| **TIME-02** | One-click copy of the seconds digits and the milliseconds digits | `ctx.copy_text` on the two timestamp rows only. No RFC3339. No custom format |
| **UI-02** | Same chrome rhythm Phase 3 will copy (title area, fields, buttons) | `xtools-ui` egui chrome + 02-UI-SPEC tokens. `xtools-time` only composes |

Out of this phase: JSON/translate windows, plugin scan, persist last values, RFC3339/custom copy, UTC row.

## Architectural Responsibility Map

Split-toolkit desktop app. No network. No database.

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Locate + spawn `xtools-time` (LAUNCH-01) | Host process | Cargo target dir / install prefix | Sibling of `current_exe()`. Not `$PATH` |
| Mint `XDG_ACTIVATION_TOKEN` from click | Host + GDK `AppLaunchContext` | Compositor xdg-activation-v1 | Only the host has the pointer serial. gdk4 `Event` has no public `serial()` |
| Single-instance lock (LAUNCH-02) | `xtools-ui::claim_instance("xtools-time")` | Abstract `AF_UNIX` | Bind is instance truth. Same helper the host already uses |
| Raise existing window (LAUNCH-02) | Live `xtools-time` | Host writes `RAISE` + token | Child cannot take focus; it can only `activate` / `Focus` with a token |
| Convert s/ms ↔ local datetime (TIME-01) | `xtools-time` + jiff | `TimeZone::system()` | One crate, one path. Not chrono. Not hand-rolled epoch math |
| Copy seconds / milliseconds (TIME-02) | `xtools-time` via `ctx.copy_text` | Wayland clipboard owner = this process | Keep the process alive after copy |
| Shared chrome (UI-02) | `xtools-ui` (`egui-chrome` feature) | `xtools-time` composition | Phase 3 clones these widgets. Host must not enable the feature |
| Close = quit (D-15) | `xtools-time` process exit | Socket drop | Next click is a new process filled with now |
| Skip-taskbar (D-16) | eframe `ViewportBuilder` | Compositor policy | Wayland `app_id`; X11 `Utility`. Accept if a dock still lists an xdg-shell toplevel |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust / cargo | rustc **1.97.1**, edition **2024** | Language / workspace | Already on the machine. [VERIFIED: `rustc --version`] |
| **eframe** | **0.36.1** (default features: wayland + x11 + wgpu) | `xtools-time` native window | Locked. `NativeOptions.viewport` is a `ViewportBuilder`. Do not mix 0.31/0.32 widgets. [VERIFIED: `cargo search eframe`; docs.rs/eframe/0.36.1] |
| **egui** | **0.36.1** | Immediate-mode form + `TextEdit` + `copy_text` + `ViewportCommand::Focus` | Same release train as eframe. [VERIFIED: `cargo search egui`; docs.rs/egui/0.36.1] |
| **jiff** | **0.2.35** (default features, including `tz-system`) | Unix s/ms ↔ local `Zoned` | Locked. `from_second` / `from_millisecond` return `Result`. [VERIFIED: `cargo search jiff`; docs.rs/jiff/0.2.35] |
| **gtk4** | **0.11.4** (already in host) | Stashed `gdk::Event`, `DisplayExt::app_launch_context` | Do not add a second GTK stack. [VERIFIED: crates/xtools-host/Cargo.toml] |
| std `UnixListener` / `UnixStream` | std | Abstract bind + RAISE | Already used by `claim_instance`. Not `single-instance`. [VERIFIED: crates/xtools-ui/src/instance.rs] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| **egui** via `xtools-ui` feature `egui-chrome` | **0.36.1** | `apply_theme`, title strip, labeled field, copy button, inline error | Only `xtools-time` enables the feature. Host must not. |
| **gdk4** | **0.11.4** (via gtk4) | `Event::time` / `display` / `surface`; `GdkAppLaunchContextExt::set_timestamp` | Token mint in the host. [VERIFIED: docs.rs/gdk4/0.11.4] |
| **gio** | **0.22.8** (via gtk4) | `AppLaunchContextExt::startup_notify_id` → `XDG_ACTIVATION_TOKEN` | Host token mint. GLib ≥ 2.76 documents this as the activation token. [CITED: docs.gtk.org/gio/method.AppLaunchContext.get_startup_notify_id.html] |
| **winit** | **0.30.13** (via eframe, do not depend directly) | `ActivationToken`, `read_token_from_env`, `reset_activation_token_env`, `with_activation_token` | Child first-map only. Do not take a second winit version. [VERIFIED: docs.rs/winit/0.30.13] |

Do **not** add this phase: chrono, arboard, egui_extras, serde_json, ureq, tokio, zbus, `single-instance`, libc 1.x, gdk4-wayland (unless the live token probe fails; see Open Questions).

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Sibling of `current_exe()` | `$PATH` / `which xtools-time` | Breaks `cargo run` and a non-login orb session. Forbidden |
| Abstract socket RAISE | D-Bus `org.freedesktop.Application` | PITFALLS prefers D-Bus. SUMMARY/ARCHITECTURE lock: not v1. Revisit only if KWin ignores socket-delivered tokens |
| Abstract socket RAISE | Lock file / `single-instance` 0.3.3 | Lock ≠ focus. Stale crate. Forbidden |
| `ViewportCommand::Focus` | `wmctrl` / `_NET_ACTIVE_WINDOW` | Ignored on this Wayland session. Forbidden |
| jiff | chrono 0.4 | Locked out. New code uses jiff |
| `ctx.copy_text` | arboard / `wl-copy` + exit | Wayland clipboard has no server store. Process must stay owner |
| eframe in host | — | D-01. Forbidden |
| Spawn when raise/Focus fails | Second window | Violates LAUNCH-02 / D-03. Forbidden |

**Installation (planner writes manifests):**

```toml
# workspace Cargo.toml — add member only
members = ["crates/xtools-ui", "crates/xtools-host", "crates/xtools-time"]

# crates/xtools-ui/Cargo.toml
[features]
default = []
egui-chrome = ["dep:egui"]

[dependencies]
egui = { version = "0.36.1", optional = true }

# crates/xtools-time/Cargo.toml
[dependencies]
eframe = "0.36.1"
egui = "0.36.1"
jiff = "0.2.35"
xtools-ui = { workspace = true, features = ["egui-chrome"] }
```

Host `Cargo.toml` stays gtk4 + gtk4-layer-shell + `xtools-ui` **without** `egui-chrome`.

**Version verification (2026-08-22):**

| Package | `cargo search` / `cargo info` | Official docs |
|---------|-------------------------------|---------------|
| eframe | 0.36.1 latest | https://docs.rs/eframe/0.36.1 |
| egui | 0.36.1 latest | https://docs.rs/egui/0.36.1 |
| jiff | 0.2.35 | https://docs.rs/jiff/0.2.35 |
| gtk4 | 0.11.4 already pinned in host | https://docs.rs/gtk4/0.11.4 |
| gdk4 | 0.11.4 via gtk4 | https://docs.rs/gdk4/0.11.4 |
| winit | 0.30.13 via eframe | https://docs.rs/winit/0.30.13 |

`cargo info eframe` reports a default yanked/older line `0.32.3 (latest 0.36.1)`. Pin **0.36.1**, the locked latest.

## Package Legitimacy Audit

> `gsd-tools` is not on this PATH. Audit used `cargo search` / `cargo info` plus official docs.rs and the already-pinned host crates. Package names come from CONTEXT / STACK / official docs, not an unverified web invention.

| Package | Registry | Age | Source Repo | Verdict | Disposition |
|---------|----------|-----|-------------|---------|-------------|
| eframe 0.36.1 | crates | egui train since 2020 | github.com/emilk/egui | OK | Approved — locked |
| egui 0.36.1 | crates | same | github.com/emilk/egui | OK | Approved — locked |
| jiff 0.2.35 | crates | 2024– | github.com/BurntSushi/jiff | OK | Approved — locked |
| gtk4 0.11.4 | crates | since 2019 | github.com/gtk-rs/gtk4-rs | OK | Already in host |
| gdk4 / gio | crates | via gtk4 | github.com/gtk-rs/gtk4-rs | OK | Transitive host |
| libc 1.0.0-alpha.4 | crates | alpha | — | SUS | **REMOVED** — do not add. Personal-use socket; no new libc crate |

**Packages removed due to [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** libc 1.x alpha — do not install

Authoritative confirmation: docs.rs + `cargo search` + existing workspace pins. Not `[ASSUMED]` for the approved row.

## Architecture Patterns

### System Architecture Diagram

```
pointer click on clock disk
    │
    ▼
xtools-host  (GTK Overlay, already running)
    │  last_pointer_event = gdk::Event  (already stashed)
    │  begin_collapse()
    │
    ├─ mint_token(event)
    │     display.app_launch_context()
    │     set_timestamp(event.time())
    │     startup_notify_id(None, &[])  → Option<XDG_ACTIVATION_TOKEN>
    │
    ├─ raise_instance("xtools-time", token)
    │     UnixStream connect \0xtools-time
    │        ok  → write "RAISE <token>\n"  → return   ★ no spawn
    │        err → sibling_bin("xtools-time")
    │               Command::new(bin) inherit env
    │               .env("XDG_ACTIVATION_TOKEN", token)
    │               spawn                        ★ LAUNCH-01
    │
    ▼
xtools-time
    claim_instance("xtools-time")
       Ok(None)  → raise_instance(...) → exit 0     ★ never map a window
       Ok(Some(listener))
            unset XDG_ACTIVATION_TOKEN / reset_activation_token_env
            eframe::run_native  560×480  title xtools · 时间戳
            fill fields from Timestamp::now()          ★ D-09
            own listener until process exit            ★ D-15
            on socket RAISE → ViewportCommand::Focus   ★ D-10, no field write
            on copy → ctx.copy_text ; stay alive
            on close → process ends ; socket gone
```

Trace LAUNCH-02: second clock click → host connect succeeds → RAISE → live `update` sees the line → `Focus`. Fields unchanged. `ps` shows one `xtools-time`.

### Recommended Project Structure

```
xtools/
├── Cargo.toml                         # add crates/xtools-time member
└── crates/
    ├── xtools-ui/
    │   └── src/
    │       ├── lib.rs                 # re-export TIME_INSTANCE, raise_instance, chrome
    │       ├── ids.rs                 # ToolId + TIME_INSTANCE + binary_name()
    │       ├── instance.rs            # claim_instance + raise_instance
    │       ├── theme.rs               # existing tokens + chrome Color helpers
    │       └── chrome.rs              # NEW, cfg(feature = "egui-chrome")
    ├── xtools-host/
    │   └── src/main.rs                # Time orb → mint + raise-or-spawn
    └── xtools-time/                   # NEW binary
        ├── Cargo.toml
        └── src/
            ├── main.rs                # claim / forward / run_native
            ├── app.rs                 # fields, now, convert, copy, socket poll
            └── convert.rs             # jiff only
```

Do not stub `xtools-json` / `xtools-trans` this phase.

`ToolId` gains:

```rust
pub const TIME_INSTANCE: &str = "xtools-time";

impl ToolId {
    pub fn binary_name(self) -> &'static str {
        match self {
            ToolId::Time => "xtools-time",
            ToolId::Json => "xtools-json",
            ToolId::Trans => "xtools-trans",
        }
    }
    pub fn instance_name(self) -> &'static str {
        match self {
            ToolId::Time => TIME_INSTANCE,
            ToolId::Json => "xtools-json",
            ToolId::Trans => "xtools-trans",
        }
    }
}
```

Host this phase calls `binary_name` / `instance_name` only for `ToolId::Time`.

### Pattern 1: One spawn path — sibling of the host binary

**What:** Resolve `xtools-time` as `current_exe()?.parent()?.join("xtools-time")`. That is the argv0-relative / same-directory path. `cargo build -p xtools-host -p xtools-time` places both under `target/debug/`.
**When to use:** Always, for LAUNCH-01.
**Do not** search `$PATH`. **Do not** `env_clear`. If the sibling is missing, `eprintln` and return — do not look elsewhere.

```rust
// Source: https://doc.rust-lang.org/std/env/fn.current_exe.html
fn sibling_bin(name: &str) -> Option<std::path::PathBuf> {
    let exe = std::env::current_exe().ok()?;
    Some(exe.parent()?.join(name))
}
```

### Pattern 2: Raise first, spawn only if the socket is absent

**What:** Host and losing child both call `raise_instance`. Spawn is the fallback when connect fails, never the fallback when Focus is ignored.
**When to use:** Every clock-orb click and every `claim_instance == None`.

```text
connect(\0xtools-time)?
    yes → RAISE [token] → done          // LAUNCH-02
    no  → spawn sibling (host only)     // LAUNCH-01
          child claim
              Some → window
              None → RAISE → exit 0     // still LAUNCH-02
```

**If the token is missing or `ViewportCommand::Focus` is ignored:** the live process still owns the socket. The host still must not spawn. The user may need to find the existing window. That is the correct failure mode. A second window is never the boring fallback.

### Pattern 3: Mint the token from the stashed GDK event

**What:** Phase 1 already stores `last_pointer_event: Option<gdk::Event>` (`Event::time`, `display`, `surface`, `seat`). gdk4 0.11.4 has **no public `serial()`**. Recover the compositor token through GDK’s launch context, which is the documented “timestamp from the triggering event” API.
**When to use:** Immediately on `ToolId::Time` click, using the event just written (or `last_pointer_event.clone()`).

```rust
// Source: https://docs.rs/gdk4/0.11.4/gdk4/prelude/trait.DisplayExt.html
//         https://gtk-rs.org/gtk4-rs/stable/latest/docs/gdk4/prelude/trait.GdkAppLaunchContextExt.html
//         https://docs.gtk.org/gio/method.AppLaunchContext.get_startup_notify_id.html
//         https://docs.gtk.org/gdk4/method.AppLaunchContext.set_timestamp.html
fn mint_token(event: &gtk4::gdk::Event) -> Option<String> {
    use gtk4::gdk::prelude::{DisplayExt, GdkAppLaunchContextExt};
    use gtk4::gio::prelude::AppLaunchContextExt;
    let display = event.display()?;
    let ctx = display.app_launch_context();
    ctx.set_timestamp(event.time());
    ctx.startup_notify_id(None::<&gtk4::gio::AppInfo>, &[])
        .map(|s| s.to_string())
}
```

Pass the string as `XDG_ACTIVATION_TOKEN` on spawn and as the RAISE payload. If mint returns `None`, still raise-or-spawn — token is best-effort. Do not block launch on a missing token.

### Pattern 4: Child consumes the token once, then unsets it

**What:** First map uses the inherited token. Then the env var must die so a grandchild cannot steal focus later.
**When to use:** `xtools-time` `main`, before `run_native`.

```rust
// Source: https://docs.rs/winit/0.30.13/winit/platform/startup_notify/index.html
//         https://wayland.app/protocols/xdg-activation-v1
let token = std::env::var("XDG_ACTIVATION_TOKEN").ok();
std::env::remove_var("XDG_ACTIVATION_TOKEN");
// also call winit::platform::startup_notify::reset_activation_token_env()
// inside NativeOptions.window_builder after read_token_from_env, if used.
```

First-map: `WindowAttributesExtStartupNotify::with_activation_token` via `NativeOptions.window_builder` if a token was present. eframe `run_native("dev.xtools.timestamp", …)` plus `ViewportBuilder::with_app_id("dev.xtools.timestamp")`.

winit 0.30 documents `with_activation_token` as a **create-time** attribute. There is no public “activate this already-mapped window with token T” on `WindowExtStartupNotify` (only `request_activation_token`). Existing-window raise is therefore: socket RAISE → `ctx.send_viewport_cmd(ViewportCommand::Focus)`. If KWin ignores Focus without an `xdg_activation_v1.activate` on that surface, the window stays single. Do not open another.

### Pattern 5: One convert path — jiff Timestamp ↔ local Zoned

**What:** Every fill and every valid edit goes through `jiff::Timestamp` plus `TimeZone::system()`. Display format is exactly `YYYY-MM-DD HH:MM:SS.mmm` (space, no `T`, no `Z`, no offset).
**When to use:** D-09 now-fill, D-11 「现在」, and any field that parses (TIME-01).

```rust
// Source: https://docs.rs/jiff/0.2.35/jiff/struct.Timestamp.html
//         https://docs.rs/jiff/0.2.35/jiff/tz/struct.TimeZone.html
//         https://docs.rs/jiff/0.2.35/jiff/fmt/strtime/index.html
use jiff::{civil::DateTime, tz::TimeZone, Timestamp};

const LOCAL_FMT: &str = "%Y-%m-%d %H:%M:%S%.3f";

fn from_now() -> (i64, i64, String) {
    let ts = Timestamp::now();
    let z = ts.to_zoned(TimeZone::system());
    (ts.as_second(), ts.as_millisecond(), z.strftime(LOCAL_FMT).to_string())
}

fn from_seconds(s: i64) -> jiff::Result<(i64, i64, String)> {
    let ts = Timestamp::from_second(s)?;
    let z = ts.to_zoned(TimeZone::system());
    Ok((ts.as_second(), ts.as_millisecond(), z.strftime(LOCAL_FMT).to_string()))
}

fn from_millis(ms: i64) -> jiff::Result<(i64, i64, String)> {
    let ts = Timestamp::from_millisecond(ms)?;
    let z = ts.to_zoned(TimeZone::system());
    Ok((ts.as_second(), ts.as_millisecond(), z.strftime(LOCAL_FMT).to_string()))
}

fn from_local(text: &str) -> jiff::Result<(i64, i64, String)> {
    let dt = DateTime::strptime(LOCAL_FMT, text)?;
    let z = dt.to_zoned(TimeZone::system())?; // Compatible disambiguation on DST gaps/folds
    let ts = z.timestamp();
    Ok((ts.as_second(), ts.as_millisecond(), z.strftime(LOCAL_FMT).to_string()))
}
```

Integer fields: parse trimmed decimal `i64` with no thousands separators. On success, rewrite all three strings from the functions above (that is how ms stays `s × 1000` for whole seconds). On empty/invalid: leave the edited text as typed, keep the last good peer strings, show the matching 02-UI-SPEC error under **that** field only. Do not crash. Do not touch peers.

### Pattern 6: UI-02 chrome in `xtools-ui`, not in the binary

**What:** 02-UI-SPEC is the contract. Implement `apply_theme`, title strip `时间戳`, labeled `TextEdit` (Hack / monospace values), 「复制」 64×32, 「现在」 on the datetime caption row, inline error 4 px under the edited field. Convert `theme::Color` floats to `Color32` in chrome — no raw hex in `xtools-time`.
**When to use:** Every tool frame this phase and Phase 3.

Copy: `ctx.copy_text(seconds_text)` / `ctx.copy_text(millis_text)` of the **current field digits** when that field is valid. Button label → `已复制` for 1 s (`ctx.request_repaint_after`). Process stays alive. No datetime copy. No RFC3339 copy.

`TextEdit::singleline` + `font(TextStyle::Monospace)` for the three values. [CITED: docs.rs/egui/0.36.1/egui/widgets/struct.TextEdit.html]

### Pattern 7: Close quits; listener lives with the window

**What:** Store the `UnixListener` on the eframe `App`. When the decorated close button ends `run_native`, `main` returns, the listener drops, the abstract name vanishes. Next clock click is LAUNCH-01 + D-09.
**When to use:** Always. Do not daemonize. Do not `persist_window: true`.

```rust
// Source: https://docs.rs/eframe/0.36.1/eframe/struct.NativeOptions.html
let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
        .with_title("xtools · 时间戳")
        .with_inner_size([560.0, 480.0])
        .with_min_inner_size([480.0, 360.0])
        .with_decorations(true)
        .with_app_id("dev.xtools.timestamp")
        .with_window_type(egui::viewport::X11WindowType::Utility),
    persist_window: false,
    centered: true, // documented as unsupported on Wayland; still set it
    ..Default::default()
};
```

`with_taskbar(false)` is **Windows-only** in egui 0.36.1. Do not expect it to hide a KDE task manager entry. Wayland skip-taskbar is compositor policy keyed by `app_id`. X11: `Utility` first; if it still lists, a `window_builder` hook may set `_NET_WM_STATE_SKIP_TASKBAR`. Accept residual dock presence on Wayland (02-UI-SPEC).

### Anti-Patterns to Avoid

- **Spawn when Focus fails:** Violates LAUNCH-02. Raise via socket then Focus; if Focus is ignored, still one process.
- **`env_clear` on `Command`:** Child loses `WAYLAND_DISPLAY` / `XDG_RUNTIME_DIR`. Inherit, set only the token.
- **`$PATH` lookup for `xtools-time`:** Wrong binary, or none from a desktop-started host.
- **Putting egui in `xtools-ui` default features:** Host would link eframe’s toolkit. Use `egui-chrome`.
- **RFC3339 / custom / datetime copy “while we are here”:** D-12, D-13, 02-UI-SPEC out-of-chrome list.
- **`wmctrl`, `_NET_ACTIVE_WINDOW`, `single-instance` crate:** Locked out.
- **Closing after copy:** Wayland paste is empty. PITFALLS clipboard.
- **`persist_window: true` / restore last values:** Deferred.
- **Always-on-top or layer-shell on the tool:** D-01.
- **Hand-rolled epoch ↔ datetime:** DST and leap-second-adjacent civil times are jiff’s job.
- **Logging `last_pointer_event` coordinates or the raw token at info/warn.**

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Unix s/ms ↔ local civil time | Manual `1970` + local offset | jiff `Timestamp` + `TimeZone::system()` | DST gaps/folds, range checks. `from_second` is `Result` |
| Single-instance | pidfile / flock / `single-instance` | `claim_instance` + abstract bind | Bind is the lock; death releases it |
| Focus existing on Wayland | `wmctrl` / X11 EWMH from a second process | Token + socket RAISE + `ViewportCommand::Focus` | Compositor must grant focus |
| Clipboard | `wl-copy` + exit, arboard then quit | `ctx.copy_text` on a live process | No server-side clipboard store |
| Theme / chrome | Hex and padding in `xtools-time` | `xtools-ui` tokens + chrome widgets | UI-02 / Phase 3 clone |
| Activation serial | gdk4-wayland raw `wl_seat` in v1 | `AppLaunchContext::set_timestamp` + `startup_notify_id` | Documented GTK path from the click event |
| Binary location | `$PATH` rebuild | `current_exe()` parent join | Same cargo target dir / install prefix |

**Key insight:** The hard part is not the form. It is “one process, raised from a GTK layer-shell click, without a second map.” Treat the socket as the instance, the token as the raise hint, and Focus-ignored as a buried-but-unique window.

## Common Pitfalls

### Pitfall 1: Second window when raise looks dead
**What goes wrong:** Token mint fails or Focus is a no-op, so the host spawns again. Two timestamp windows. LAUNCH-02 dies.
**Why it happens:** “Spawn is the boring fallback” habit. On Wayland, focus-stealing prevention looks like a dead orb.
**How to avoid:** Connect-or-spawn is ordered. Connect success ⇒ never spawn. `claim_instance` None ⇒ RAISE and exit 0, never `run_native`.
**Warning signs:** `ps -C xtools-time` count > 1. Two titles `xtools · 时间戳`.

### Pitfall 2: Lock without raise
**What goes wrong:** Second click exits; buried window stays buried. User thinks the clock orb is broken.
**Why it happens:** Host copies today’s host `claim_instance` (`Ok(None) => return`) without writing RAISE.
**How to avoid:** Host does not claim the tool socket. It only connects. The tool claims. Host on connect writes RAISE + token.
**Warning signs:** Clock click no-ops while `xtools-time` is in `ps`.

### Pitfall 3: `env_clear` or PATH spawn
**What goes wrong:** Child has no `WAYLAND_DISPLAY`. Or a stray `xtools-time` on PATH opens.
**Why it happens:** CLI hygiene. Login-shell assumptions.
**How to avoid:** Inherit env. Sibling path only. Set only `XDG_ACTIVATION_TOKEN`.
**Warning signs:** Works from a terminal `cargo run`, fails from the orb.

### Pitfall 4: Token left in the environment
**What goes wrong:** A later child of `xtools-time` steals focus with a stale token.
**Why it happens:** Protocol requires unset after first read; easy to forget.
**How to avoid:** `remove_var` + `reset_activation_token_env` before `run_native`.
**Warning signs:** Unrelated windows jump forward after a timestamp copy or a Phase 3 spawn from a helper.

### Pitfall 5: Closing after copy
**What goes wrong:** Paste into the terminal is empty on Wayland.
**Why it happens:** Clipboard is owned by the source process.
**How to avoid:** `ctx.copy_text`; keep the window up. D-15 is title-bar close, not copy.
**Warning signs:** `已复制` then an empty paste once the process is gone.

### Pitfall 6: eframe in the host, or egui as a default `xtools-ui` dep
**What goes wrong:** Host grows a winit event loop or a second toolkit. Overlay contract frays.
**Why it happens:** “Share chrome” read as “one crate, all features on.”
**How to avoid:** `egui-chrome` optional. Host does not enable it. `cargo tree -p xtools-host` must not list eframe/egui/winit.
**Warning signs:** `WindowLevel` or `eframe::` under `crates/xtools-host`.

### Pitfall 7: RFC3339 / custom copy creep
**What goes wrong:** TIME-02 grows buttons CONTEXT declined. UI-02 chrome is no longer the template the user approved.
**Why it happens:** ROADMAP criterion 4 is stale relative to D-12.
**How to avoid:** Two copy buttons. Planner tasks that add a third copy target are out of scope.
**Warning signs:** A `RFC3339` label or a format `TextEdit`.

### Pitfall 8: Dropping the listener while the window lives
**What goes wrong:** Socket gone ⇒ next click spawns a second process ⇒ second window.
**Why it happens:** `claim_instance` result stored in `main` and dropped before `run_native`.
**How to avoid:** Move the `UnixListener` into the `App`. Poll `accept` (nonblocking) each `update`.
**Warning signs:** Two windows after the first has been open the whole time.

### Pitfall 9: `centered: true` assumed on Wayland
**What goes wrong:** First show is not on the current output.
**Why it happens:** eframe documents `NativeOptions.centered` as unsupported on Wayland.
**How to avoid:** Set `centered: true` anyway. If first-show is wrong on this KWin, use `ViewportCommand::center_on_screen` once on first frame. Do not persist position.
**Warning signs:** Window opens on the wrong monitor.

## Code Examples

### Sibling spawn + token (host)

```rust
// Source: https://doc.rust-lang.org/std/env/fn.current_exe.html
//         https://docs.gtk.org/gio/method.AppLaunchContext.get_startup_notify_id.html
fn launch_time(event: Option<&gtk4::gdk::Event>) {
    let token = event.and_then(mint_token);
    if xtools_ui::raise_instance(xtools_ui::TIME_INSTANCE, token.as_deref()).unwrap_or(false) {
        return;
    }
    let Some(bin) = sibling_bin("xtools-time") else { return };
    if !bin.is_file() {
        eprintln!("xtools-host: missing {}", bin.display());
        return;
    }
    let mut cmd = std::process::Command::new(bin);
    if let Some(t) = token {
        cmd.env("XDG_ACTIVATION_TOKEN", t);
    }
    if let Err(err) = cmd.spawn() {
        eprintln!("xtools-host: spawn xtools-time: {err}");
    }
}
```

### claim-or-forward (child)

```rust
// Source: crates/xtools-ui/src/instance.rs
//         https://docs.rs/egui/0.36.1/egui/viewport/enum.ViewportCommand.html
fn main() -> eframe::Result {
    let token = std::env::var("XDG_ACTIVATION_TOKEN").ok();
    std::env::remove_var("XDG_ACTIVATION_TOKEN");
    match xtools_ui::claim_instance(xtools_ui::TIME_INSTANCE) {
        Ok(Some(listener)) => run_window(listener, token),
        Ok(None) => {
            let _ = xtools_ui::raise_instance(xtools_ui::TIME_INSTANCE, token.as_deref());
            Ok(())
        }
        Err(err) => {
            eprintln!("xtools-time: instance lock failed: {err}");
            std::process::exit(1);
        }
    }
}
```

### RAISE line protocol

```rust
// One line, no serde. Token is optional and must not contain spaces or NULs.
// RAISE\n
// RAISE <token>\n
```

`raise_instance` connects with `UnixStream::connect_addr(&SocketAddr::from_abstract_name(name.as_bytes())?)`, writes the line, returns `Ok(true)`. Connect failure is `Ok(false)` (no live instance), not an error that should panic the host.

Live `App::update`: `listener.set_nonblocking(true)` (already done in `claim_instance`). `while let Ok((stream, _)) = listener.accept()` read one line. On `RAISE`, set a flag; do **not** rewrite fields. Then `ctx.send_viewport_cmd(egui::viewport::ViewportCommand::Focus)`.

### Copy

```rust
// Source: https://docs.rs/egui/0.36.1/egui/struct.Context.html#method.copy_text
if copy_clicked && seconds_valid {
    ui.ctx().copy_text(seconds.clone());
    copied_at = Some(ui.ctx().input(|i| i.time));
    ui.ctx().request_repaint_after(std::time::Duration::from_secs(1));
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `single-instance` 0.3.3 lock file | Abstract `AF_UNIX` bind + RAISE | Project lock 2026-08-19 | Focus is a message, not a file |
| X11 `_NET_ACTIVE_WINDOW` / wmctrl | `xdg-activation-v1` token from click | Wayland default on this machine | Second process cannot steal focus |
| chrono UnixTime | jiff `Timestamp::from_second` / `from_millisecond` | jiff 0.2 | `Result`, zone-aware `Zoned` |
| `GdkEvent` serial field | GDK `AppLaunchContext` + event timestamp | GTK4 / gdk4 0.11 — no public `serial()` | Stash the whole `Event` (Phase 1 A3) |
| eframe-everywhere host | GTK host + eframe tool | D-01 | Do not put eframe in the host |
| ROADMAP TIME-02 four copy formats | CONTEXT D-12 two copy formats | Discuss-phase 2026-08-19 | No RFC3339 UI this phase |
| `NativeOptions.centered` as portable | Documented unsupported on Wayland | eframe 0.36.1 | May need one-shot `center_on_screen` |
| `with_taskbar(false)` as Linux skip-taskbar | Windows-only in egui 0.36.1 | egui 0.36 ViewportBuilder | Use `app_id` + X11 `Utility` |

**Deprecated/outdated:**
- RFC3339 / custom-format copy as Phase 2 TIME-02 (CONTEXT narrowed it)
- D-Bus `org.freedesktop.Application` as the v1 raise path (SUMMARY lock)
- `run_and_return` loops that keep a hidden process after close (D-15)
- libc 1.0 alpha for `SO_PEERCRED` (do not add)

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `startup_notify_id(None, &[])` after `set_timestamp(event.time())` is enough for KWin to issue a usable xdg-activation token from a layer-shell surface | Pattern 3 | If KWin needs a raw wl serial, add `gdk4-wayland` and recover serial from the seat. Do not change the raise-or-spawn order |
| A2 | `ViewportCommand::Focus` on an already-mapped eframe window is the right existing-window activate; winit 0.30 has no public apply-token-to-live-window API | Pattern 4 | Window may stay buried. Still one process. D-Bus only if a later phase reopens this |
| A3 | `DateTime::strptime("%Y-%m-%d %H:%M:%S%.3f")` accepts the UI-SPEC local string including `.mmm` | Pattern 5 | If `%.3f` parse is picky about trailing zeros, accept `DateTime` `FromStr` of the same text as a fallback **inside convert.rs**, not a second UI format |
| A4 | `NativeOptions.centered` plus optional first-frame `center_on_screen` satisfies D-14 on this KWin | Pattern 7 | Window may open on the wrong output; fix with one-shot center, not persist |
| A5 | Skipping `SO_PEERCRED` is acceptable for a personal single-uid box | Security | Another local uid could RAISE. Do not pull libc 1.x alpha to close this |

A3 is an implementation detail, not a product fork. A1/A2/A4 are live compositor probes. A5 is residual personal-use risk.

## Open Questions

1. **Does a socket-delivered token plus `ViewportCommand::Focus` actually raise the eframe window on this KWin?**
   - What we know: protocol + KWin 6.6 implement `xdg_activation_v1`. winit applies tokens at **window creation**. Focus on a live window may be ignored.
   - What's unclear: whether KWin treats `Focus` from the already-mapped client as enough when the token never reaches `activate(token, surface)`.
   - Recommendation: implement RAISE + Focus. Verify with a buried window. If ignored, do **not** spawn. Optional later: D-Bus or a winit activate hook. Not this phase’s default.

2. **Does GDK `set_timestamp(event.time())` recover a recent enough serial from a layer-shell click?**
   - What we know: Phase 1 A3; gdk4 has no public `serial()`; GTK docs say use the event timestamp.
   - What's unclear: layer-shell vs xdg-shell requesting surface for `set_surface`.
   - Recommendation: ship Pattern 3. If tokens come back empty, then add gdk4-wayland.

3. **First-show centering on Wayland**
   - What we know: eframe `centered` is documented unsupported on Wayland.
   - What's unclear: this KWin + eframe 0.36.1 actual placement.
   - Recommendation: set `centered: true`; if wrong, one `ViewportCommand::center_on_screen` on the first frame.

These are verification probes, not design forks. Planner should write human-check steps, not alternative architectures.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| rustc / cargo | all crates | ✓ | 1.97.1 | — |
| eframe / egui 0.36.1 | `xtools-time`, chrome | ✓ crates.io | 0.36.1 | none — locked |
| jiff 0.2.35 | TIME-01 | ✓ crates.io | 0.2.35 | none — locked |
| gtk4 0.11.4 + system GTK | host token mint | ✓ (Phase 1) | 0.11.4 / system 4.22 | none |
| Wayland session + KWin xdg-activation | LAUNCH-02 raise | ✓ `XDG_SESSION_TYPE=wayland` `XDG_CURRENT_DESKTOP=KDE` | KWin implements v1 | Focus may be ignored; still no second window |
| `/usr/share/zoneinfo` | `TimeZone::system()` | ✓ typical openSUSE | — | jiff falls back to `TimeZone::unknown` (civil times still convert; WARN log) |
| `gsd-tools` | legitimacy seam | ✗ | — | cargo search + official docs (this file) |

**Missing dependencies with no fallback:** none for implementation.

**Missing dependencies with fallback:** `gsd-tools` (audit done via cargo + docs.rs).

## Security Domain

`security_enforcement` is true (ASVS level 1). Desktop personal toolbox; no accounts, no HTTP this phase.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | No users |
| V3 Session Management | no | No sessions |
| V4 Access Control | yes (local) | Abstract socket is the instance. Do not follow attacker-controlled paths from `current_exe` beyond “sibling name `xtools-time`” |
| V5 Input Validation | yes | jiff `Result` on s/ms/datetime. Socket line: `RAISE` + optional token; reject NULs, newlines inside the token, and lines > 4 KiB |
| V6 Cryptography | no | No secrets, no hashes of time |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Second process maps a window | Tampering / Elevation | `claim_instance` None → RAISE + exit. Never `run_native` on a lost bind |
| Forged RAISE from another uid | Spoofing | Personal-use residual (A5). Do not add libc 1.x. Token is not a secret but ignore garbage |
| Stale `XDG_ACTIVATION_TOKEN` inherited by a grandchild | Elevation (focus steal) | Unset after first read |
| Path injection via `current_exe` / argv0 | Elevation | Join a hardcoded `"xtools-time"` only; `is_file()` before exec; no shell |
| Huge / non-numeric field text | Denial of service | `i64` parse; jiff range `Result`; keep last good peers |
| Logging pointer coordinates or tokens | Information disclosure | Do not log event position, time, or token at info/warn (Phase 1 T-01-08 continues) |
| Clipboard contents after process death | — (UX, not threat) | Keep process alive after copy |

## Sources

### Primary (HIGH confidence)

- crates/xtools-host/src/main.rs — `last_pointer_event`, `handle_click` collapses function orbs and does not spawn
- crates/xtools-ui/src/instance.rs — `claim_instance` abstract bind, `AddrInUse` → `Ok(None)`
- crates/xtools-ui/src/ids.rs — `ToolId::{Time, Json, Trans}`, `HOST_INSTANCE` only
- crates/xtools-ui/src/theme.rs — `ORB_FILL` / `ORB_MARK` / `MARK_PX`
- 02-CONTEXT.md, 02-UI-SPEC.md (approved 2026-08-22), REQUIREMENTS.md, ROADMAP.md
- https://docs.rs/eframe/0.36.1/eframe/struct.NativeOptions.html — viewport, persist_window, centered (Wayland unsupported), run_and_return
- https://docs.rs/egui/0.36.1/egui/viewport/struct.ViewportBuilder.html — with_app_id, with_window_type, with_taskbar (Windows)
- https://docs.rs/egui/0.36.1/egui/viewport/enum.ViewportCommand.html — Focus, center_on_screen
- https://docs.rs/egui/0.36.1/egui/viewport/enum.X11WindowType.html — Utility
- https://docs.rs/egui/0.36.1/egui/struct.Context.html — copy_text, send_viewport_cmd
- https://docs.rs/egui/0.36.1/egui/widgets/struct.TextEdit.html — singleline
- https://docs.rs/jiff/0.2.35/jiff/struct.Timestamp.html — now, from_second, from_millisecond, as_second, as_millisecond, to_zoned
- https://docs.rs/jiff/0.2.35/jiff/tz/struct.TimeZone.html — system, to_zoned Compatible
- https://docs.rs/jiff/0.2.35/jiff/civil/struct.DateTime.html — FromStr space-separated civil, strptime
- https://docs.rs/jiff/0.2.35/jiff/fmt/strtime/index.html — `%Y` `%m` `%d` `%H` `%M` `%S` `%.3f`
- https://docs.rs/gdk4/0.11.4/gdk4/prelude/trait.DisplayExt.html — app_launch_context
- https://docs.rs/gdk4/0.11.4/gdk4/struct.Event.html — time, display, surface, seat; no serial()
- https://gtk-rs.org/gtk4-rs/stable/latest/docs/gdk4/prelude/trait.GdkAppLaunchContextExt.html — set_timestamp
- https://docs.gtk.org/gdk4/class.AppLaunchContext.html — timestamp from triggering event
- https://docs.gtk.org/gdk4/method.AppLaunchContext.set_timestamp.html
- https://docs.gtk.org/gio/method.AppLaunchContext.get_startup_notify_id.html — returns XDG_ACTIVATION_TOKEN (GLib 2.76+)
- https://docs.rs/gio/0.22.8/gio/prelude/trait.AppLaunchContextExt.html — startup_notify_id
- https://wayland.app/protocols/xdg-activation-v1 — token from click serial; child unsets env; KWin 6.6 implements v1
- https://docs.rs/winit/0.30.13/winit/platform/startup_notify/index.html — read_token_from_env, reset_activation_token_env, with_activation_token (create-time)
- https://docs.rs/winit/0.30.13/winit/window/struct.ActivationToken.html
- https://doc.rust-lang.org/std/env/fn.current_exe.html
- `cargo search` 2026-08-22 — eframe/egui 0.36.1, jiff 0.2.35
- `rustc --version` — 1.97.1
- `XDG_SESSION_TYPE=wayland` `XDG_CURRENT_DESKTOP=KDE`

### Secondary (MEDIUM confidence)

- .planning/research/SUMMARY.md — Phase 2 spawn-or-focus, socket + token, chrome template
- .planning/research/PITFALLS.md — lock without activation; env_clear; clipboard owner; no wmctrl
- AGENTS.md STACK chapter — eframe tool patterns; host chapter overruled
- Kai Uwe Broulik, On Window Activation (cited in SUMMARY) — Extreme FSP needs a valid token

### Tertiary (LOW confidence)

- Whether KWin will `activate` an eframe xdg-shell toplevel from Focus after a GTK layer-shell click token that never reached winit `with_activation_token` on that live surface
- NVIDIA/wgpu black rectangle on the tool window — glow fallback only if it happens; do not change host toolkit

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — locked crates, cargo search 0.36.1 / 0.2.35, docs.rs APIs read this session
- Architecture: HIGH — one spawn path, one convert path, claim-or-forward is already in-tree
- Raise-on-Wayland: MEDIUM — token mint + Focus prescribed; live KWin probe still required
- Pitfalls: HIGH — copied from PITFALLS + eframe/winit/jiff docs, mapped to the five REQ IDs

**Research date:** 2026-08-22
**Valid until:** 2026-09-21 (30 days; eframe/jiff stable pins)

---
*Phase: 2-Timestamp Window and Spawn-or-Focus*
*Ready for planning: yes*
