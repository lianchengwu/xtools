# Architecture Research

**Domain:** Rust Linux desktop orbital-launcher toolbox
**Researched:** 2026-08-19
**Confidence:** MEDIUM

Seam-capped (verified websearch = MEDIUM; webfetch of official specs = LOW). Process model, workspace layout, lock, and activation are taken from current official specs, not from training recall.

## Standard Architecture

Use a **host process + three independent tool processes + one shared UI crate**. The host owns the always-on-top orb, the orbital 3-ball menu, and spawn-or-focus. Each tool is its own Rust window binary. Looks stay consistent because every binary compiles the same theme/widgets crate — not because the host embeds tool UI or ships theme bytes over IPC.

This machine is KDE Plasma Wayland (`XDG_SESSION_TYPE=wayland`, `XDG_CURRENT_DESKTOP=KDE`). Design for that first. X11 EWMH is a fallback, not the primary path.

### System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Linux compositor                            │
│              KWin (Wayland) / X11 WM as fallback                    │
├─────────────────────────────────────────────────────────────────────┤
│  HOST PROCESS  (xtools-host)                                        │
│  ┌────────────┐   click    ┌──────────────────────────────────┐     │
│  │  Orb       │───────────▶│  Orbital menu (same process)     │     │
│  │  layer-    │  toggle    │  Time / JSON / Trans balls       │     │
│  │  shell     │◀───────────│  layout around orb, not windows  │     │
│  └─────┬──────┘            └────────────────┬─────────────────┘     │
│        │                                    │ click ball            │
│        │                                    ▼                       │
│        │                       ┌────────────────────────┐           │
│        │                       │  Launcher              │           │
│        │                       │  ToolId hardcoded v1   │           │
│        │                       │  token + spawn/focus   │           │
│        │                       └───────────┬────────────┘           │
├────────┼───────────────────────────────────┼────────────────────────┤
│        │   abstract AF_UNIX  Focus{token}  │                        │
│        │   \0xtools-{tool}                 │ exec + XDG_ACTIVATION  │
│        ▼                                   ▼                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐               │
│  │ xtools-time  │  │ xtools-json  │  │ xtools-trans │               │
│  │ toplevel     │  │ toplevel     │  │ toplevel     │               │
│  │ flock/socket │  │ flock/socket │  │ + Engine     │               │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘               │
│         │                 │                 │                       │
├─────────┴─────────────────┴─────────────────┴───────────────────────┤
│  SHARED CRATE  xtools-ui  (compiled into host and every tool)       │
│  theme · widgets · window chrome · instance lock/wakeup             │
└─────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| `xtools-host` | Always-on-top orb, drag, expand/collapse three balls, hardcoded `ToolId` menu, spawn-or-focus | One binary. Layer-shell overlay surface. No timestamp/JSON/translate widgets. |
| Orbital menu | Three function balls laid out around the orb | Same host process and same surface (or child popups parented to the layer surface). Not three processes. |
| Launcher | Given a `ToolId`, either spawn that binary or wake the existing one | Abstract Unix socket connect; on failure `Command::new` with `XDG_ACTIVATION_TOKEN`. |
| `xtools-ui` | Theme tokens, shared widgets, window chrome, instance lock + wakeup protocol | Library crate path-depended by host and all three tools. |
| `xtools-time` | Unix s/ms ↔ datetime, one-click copy of common formats | Independent toplevel process. Single-instance. |
| `xtools-json` | Format, minify, validate, show error location | Independent toplevel process. Single-instance. |
| `xtools-trans` | Input / output / language UI; swap engines without rewriting the window | Independent toplevel process. `TranslateEngine` trait + one v1 impl. |
| Compositor | Z-order of the orb, activation of tool windows | `zwlr_layer_shell_v1` overlay + `xdg_activation_v1`. X11: `_NET_WM_STATE_ABOVE` + `_NET_ACTIVE_WINDOW`. |

## Recommended Project Structure

Use a **virtual workspace**. Do not put `[package]` in the root `Cargo.toml`. The host is the default member so `cargo run` starts the orb, but it is not a Cargo root package mixed with workspace keys.

```
xtools/
├── Cargo.toml                 # virtual [workspace], resolver, members, default-members
├── Cargo.lock
├── crates/
│   ├── xtools-ui/             # shared theme, widgets, chrome, instance lock
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── theme.rs       # colors, type, spacing, radii
│   │       ├── widgets.rs     # field, button, split, error gutter, copy row
│   │       ├── chrome.rs      # window padding / title rhythm
│   │       └── instance.rs    # ToolId, abstract socket lock, Wake::Focus
│   ├── xtools-host/           # orb + orbital menu + launcher
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── orb.rs         # layer-shell surface, drag, always-on-top
│   │       ├── menu.rs        # 3-ball layout around orb
│   │       └── launch.rs      # spawn-or-focus
│   ├── xtools-time/           # timestamp window binary
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   ├── xtools-json/           # JSON window binary
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   └── xtools-trans/          # translate window + engine trait
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs        # window only talks to the trait
│           ├── engine.rs      # pub trait TranslateEngine
│           └── engines/
│               └── v1.rs      # one working impl
└── target/                    # single shared build dir
```

Root manifest shape:

```toml
[workspace]
resolver = "3"
members = [
    "crates/xtools-ui",
    "crates/xtools-host",
    "crates/xtools-time",
    "crates/xtools-json",
    "crates/xtools-trans",
]
default-members = ["crates/xtools-host"]

[workspace.package]
edition = "2024"
license = "MIT"
rust-version = "1.85"

[workspace.dependencies]
xtools-ui = { path = "crates/xtools-ui" }
```

### Structure Rationale

- **Virtual workspace:** Cargo book: a workspace without a root package keeps each binary in its own directory, shares one `Cargo.lock` and one `target/`, and avoids rebuilding the UI crate four times. `resolver` must be set explicitly on a virtual manifest.
- **`default-members = [xtools-host]`:** `cargo run` from the repo root launches the orb. Tools are `cargo run -p xtools-time` (and friends). Use `cargo test --workspace` so default-members does not hide crate tests.
- **`crates/xtools-ui`:** Visual identity is code. Compile the same theme, widgets, and chrome into every process. Put lock/wakeup here too so host and tools share one `ToolId` and one `Wake` message without a sixth crate.
- **One binary crate per window:** Matches the locked process model. A tool crash cannot take down the orb or the other two windows.
- **`TranslateEngine` stays in `xtools-trans`:** Only that window needs engines. Do not leak HTTP clients or language lists into `xtools-ui`.
- **No `plugins/` directory:** v1 menu is a hardcoded `ToolId` enum. Scanning a folder is out of scope.

### Build Order

Build in this order because each step is the style/contract template for the next. Do not start three tool windows in parallel before chrome exists.

1. **`xtools-ui` theme + widgets + chrome, then `xtools-host` orb.** Prove always-on-top, drag, click-to-expand three balls, click-to-collapse. Balls may be stubs that do not launch yet. This is the product core value.
2. **One tool window as the style template — `xtools-time`.** Simplest domain (pure functions, no network). Wire instance lock + wakeup. Copy the window chrome, spacing, and copy-row widgets until it looks like the same product as the orb, not a different app.
3. **The other two.** Clone the time-window chrome into `xtools-json` (error gutter is the new widget) and `xtools-trans` (engine trait + one impl). Do not invent a second visual system.

## Architectural Patterns

### Pattern 1: Host / satellite processes

**What:** The host is a launcher shell. Satellites are full window processes. The host never constructs a timestamp converter, a JSON tree, or a translate form.
**When to use:** Always, for this project. The user locked independent window processes.
**Trade-offs:** Extra processes and a tiny wakeup protocol. Isolation and “one tool dies, orb stays” are the point. A single-process multi-window app would be simpler and is the wrong product.

**Example:**

```rust
// crates/xtools-ui/src/instance.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolId { Time, Json, Trans }

pub enum Wake { Focus { token: Option<String> } }

impl ToolId {
    pub fn bin(self) -> &'static str {
        match self {
            ToolId::Time => "xtools-time",
            ToolId::Json => "xtools-json",
            ToolId::Trans => "xtools-trans",
        }
    }
    pub fn socket_name(self) -> &'static str {
        match self {
            ToolId::Time => "\0xtools-time",
            ToolId::Json => "\0xtools-json",
            ToolId::Trans => "\0xtools-trans",
        }
    }
}
```

### Pattern 2: Abstract Unix socket as lock + wakeup

**What:** Each tool (and the host) `bind()`s an abstract `AF_UNIX` name. `bind()` failure means an instance already owns the name. The same socket accepts `Wake::Focus { token }`.
**When to use:** Linux-only single-instance desktop tools that also need to talk to the live process. That is this project.
**Trade-offs:** Linux-only (already locked). Abstract names have no file permissions — check `SO_PEERCRED` and require the same uid. Pathname sockets under `$XDG_RUNTIME_DIR/xtools/` plus `flock(LOCK_EX|LOCK_NB)` are the fallback if abstract names ever get in the way of debugging; they can go stale after a crash and need unlink-and-retry.

Do not use pidfiles. `flock(2)` is advisory and released when the last fd closes; a pid file is neither.

**Example:**

```rust
pub fn claim_or_wake(id: ToolId, token: Option<String>) -> std::io::Result<Option<std::os::unix::net::UnixListener>> {
    let sock = std::os::unix::net::UnixListener::bind(id.socket_name());
    match sock {
        Ok(listener) => Ok(Some(listener)),
        Err(_) => {
            let conn = std::os::unix::net::UnixStream::connect(id.socket_name())?;
            // write Wake::Focus { token }; then the existing process raises itself
            let _ = (conn, token);
            Ok(None)
        }
    }
}
```

### Pattern 3: Shared UI crate, not theme IPC

**What:** Colors, type, spacing, field/button/gutter widgets, and window padding live in `xtools-ui` and are compiled into every binary.
**When to use:** Multiple processes that must look like one product and have no live theme switcher.
**Trade-offs:** Changing a color requires a rebuild of four binaries. That is correct for a personal toolbox. Runtime theme broadcast (host → tools over IPC or shared memory) solves a problem v1 does not have and couples processes that should stay strangers.

### Pattern 4: Activation token, not focus stealing

**What:** On ball click the host asks the compositor for an `xdg_activation_v1` token (serial from that click, host surface, target `app_id`). New process: set `XDG_ACTIVATION_TOKEN` and unset it in the child after read. Existing process: send the token in `Wake::Focus`. The tool calls `activate(token, surface)`.
**When to use:** Any “focus the existing window” path on Wayland. KWin 6 implements the protocol.
**Trade-offs:** Without a recent serial the compositor may ignore the request (focus-stealing prevention). That is expected. Do not compensate with `wmctrl` / `_NET_ACTIVE_WINDOW` on a Wayland session. Those are X11 EWMH messages. `winit` `WindowLevel` is unsupported on Wayland — do not use it for the orb either.

### Pattern 5: Translate engine trait object

**What:** The translate window holds `Box<dyn TranslateEngine + Send + Sync>`. The window draws input, output, and language controls. It never names a vendor.
**When to use:** The set of engines is open at runtime (user: engine must be swappable). Rust book ch18: use `dyn Trait` when impls are not a closed compile-time set; use generics only for a homogeneous collection.
**Trade-offs:** Dynamic dispatch is irrelevant at human typing speed. A generic `Window<E: TranslateEngine>` would freeze the engine at compile time and fight the “replace the engine” requirement.

**Example:**

```rust
// crates/xtools-trans/src/engine.rs
pub struct Request<'a> {
    pub text: &'a str,
    pub source: Lang,
    pub target: Lang,
}

pub trait TranslateEngine: Send + Sync {
    fn id(&self) -> &'static str;
    fn languages(&self) -> &[Lang];
    fn translate(&self, req: Request<'_>) -> Result<String, EngineError>;
}

// crates/xtools-trans/src/main.rs
struct TransApp {
    engine: Box<dyn TranslateEngine>,
    input: String,
    output: String,
}
```

## Data Flow

### Request Flow

```
Click orb
    ↓
Host menu state: Collapsed ↔ Expanded
    ↓
Click function ball
    ↓
Host launch.rs → xdg-activation token from click serial
    ↓
connect(\0xtools-{tool})
    ├─ success → write Wake::Focus { token } → existing tool activate()
    └─ fail    → Command::new(bin)
                      env XDG_ACTIVATION_TOKEN
                      spawn
                      child bind()s socket, shows toplevel
```

### State Management

```
Host-only state (never shared with tools):
    orb_pos, expanded: bool, drag_offset

Per-tool process state (never shared with host):
    window contents, scroll, last error, selected format / langs

Instance truth:
    abstract socket bind success == this process is the instance
    host does not keep a PID table as source of truth
```

### Key Data Flows

1. **Expand:** User clicks the orb. Host flips `expanded = true` and draws three balls on a circle around the orb (same process). No child processes. No compositor toplevels for the balls. Balls move with the orb while dragging.

2. **Spawn-or-focus:** User clicks a function ball. Host maps the ball to a hardcoded `ToolId`. It requests an activation token from the click, then `connect()`s `\0xtools-{tool}`. Connected: send `Focus { token }` and return. Not connected: `exec` the matching binary with `XDG_ACTIVATION_TOKEN`. The new process `bind()`s the abstract name; if `bind` loses a race, it forwards `Focus` and exits. The tool window raises itself. Menu stays expanded until the user clicks the orb again.

3. **Collapse:** User clicks the orb while expanded. Host sets `expanded = false` and stops drawing the three balls. Running tool windows are not closed, hidden, or signaled. The orb remains.

4. **Host single-instance:** A second `xtools-host` binds `\0xtools-host`, fails, and exits (or pokes the live orb). Do not start a second orb.

5. **Translate call:** UI collects text + langs → `engine.translate(...)` → output pane or inline error. Swap engines by constructing a different `Box<dyn TranslateEngine>` at startup. No host involvement.

## Scaling Considerations

This is a personal Linux toolbox, not a multi-tenant service. Scale is “how many tools / how many compositors,” not user count.

| Scale | Architecture Adjustments |
|-------|--------------------------|
| 3 hardcoded tools (v1) | Host enum + three binaries. No plugin loader. No D-Bus daemon. |
| ~10 personal tools | Still hardcoded or a static table in the host. Same lock/wakeup. Do not add directory scan until the three-window path is boring. |
| Other machines / compositors | Keep layer-shell + xdg-activation as the Wayland path. Keep EWMH `_NET_WM_STATE_ABOVE` / `_NET_ACTIVE_WINDOW` behind an X11 fallback. Do not assume GNOME and KWin treat layer-shell overlays the same. |

### Scaling Priorities

1. **First bottleneck:** Wayland activation without a click serial. Fix by always minting the token from the ball-click event, never from a timer or a host-startup leftover.
2. **Second bottleneck:** Host surface toolkit that cannot speak layer-shell. `winit` `WindowLevel::AlwaysOnTop` is unsupported on Wayland. Pick a host stack that can create `zwlr_layer_surface_v1` (STACK.md). Tools can stay ordinary toplevels.

## Anti-Patterns

### Anti-Pattern 1: Embed tool UI in the host

**What people do:** One process, orb plus timestamp/JSON/translate widgets in extra windows or panels.
**Why it's wrong:** Violates the locked process model. A JSON panic takes down the orb. “Independent window programs, host only launches” becomes a comment, not an architecture.
**Do this instead:** Host draws balls. Tools are separate binaries. Shared look comes from `xtools-ui`.

### Anti-Pattern 2: Theme or business data over IPC

**What people do:** Host is a theme server; tools pull colors and maybe even formatted JSON over a socket.
**Why it's wrong:** Couples processes that should only exchange `Focus`. Makes the host a runtime dependency of every tool. Shared memory theming adds cross-process mutexes for no v1 feature.
**Do this instead:** Compile `xtools-ui` into each binary. IPC payload is `Wake::Focus` only.

### Anti-Pattern 3: PID files / “is the PID alive?”

**What people do:** Write `~/.cache/xtools-json.pid`, then `kill -0` on next launch.
**Why it's wrong:** Stale after crash; PID reuse can target the wrong process. `flock(2)` and abstract `bind()` already solve this.
**Do this instead:** Abstract Unix socket bind, or `flock` on `$XDG_RUNTIME_DIR/xtools/{tool}.lock`.

### Anti-Pattern 4: Three processes for the three balls

**What people do:** Treat each orbital ball as its own always-on-top window process.
**Why it's wrong:** Drag, expand, and collapse become distributed layout. Focus and layer-shell setup multiply by four. The menu is a host widget, not a tool.
**Do this instead:** Balls are host-drawn hit targets around the orb.

### Anti-Pattern 5: `winit` AlwaysOnTop or `wmctrl` on Wayland

**What people do:** Set `WindowLevel::AlwaysOnTop` for the orb; focus tools with `wmctrl -a` / `_NET_ACTIVE_WINDOW`.
**Why it's wrong:** Official winit 0.30 docs: WindowLevel is unsupported on Wayland. `_NET_ACTIVE_WINDOW` is an X11 EWMH client message. This session is Plasma Wayland.
**Do this instead:** Layer-shell overlay for the orb. `xdg_activation_v1` token into the tool. X11 hints only on an actual X11 session.

### Anti-Pattern 6: Plugin directory scan in v1

**What people do:** Drop a binary in `~/.local/share/xtools/plugins` and have it appear as a fourth ball.
**Why it's wrong:** Explicitly out of scope. Adds discovery, trust, and layout problems before the three-window path exists.
**Do this instead:** `ToolId` enum with three variants. Architecture already isolates processes so a later static table can grow without a rewrite.

### Anti-Pattern 7: Multi-instance tool windows

**What people do:** Every ball click `spawn()`s another window.
**Why it's wrong:** User chose focus-existing. Clipboard/log workflows want one timestamp window, not five.
**Do this instead:** `claim_or_wake`. Second launch is a focus message.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| KWin / Wayland compositor | layer-shell overlay for host; xdg-activation for tools | Current desktop. Token must carry the click serial. |
| X11 WM (fallback) | `_NET_WM_STATE_ABOVE` for orb; `_NET_ACTIVE_WINDOW` for focus | Only when `XDG_SESSION_TYPE=x11`. Do not send these to a Wayland compositor. |
| Translation HTTP API | Inside one `TranslateEngine` impl | Window does not know the vendor. No offline model in v1. |
| Clipboard | Tool-local copy buttons | Host does not listen to the clipboard. |
| `$XDG_RUNTIME_DIR` | Locks/sockets if not using abstract names | Spec: user-owned 0700 tmpfs, gone after logout. Not for large files. |
| `$XDG_CONFIG_HOME` | Optional later config | Not required to prove the orb + three windows. |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| host ↔ tool | `Wake::Focus { token }` on abstract Unix socket; or `exec` + env | No other messages in v1. |
| host ↔ orbital balls | In-process event/state | Not IPC. |
| tool ↔ `xtools-ui` | Direct Rust calls | Same address space, compiled in. |
| translate UI ↔ engine | `Box<dyn TranslateEngine>` | Constructed at process start. |
| tool ↔ tool | None | Timestamp does not talk to JSON. |
| host ↔ host | Abstract socket `\0xtools-host` | Second host exits. |

## Sources

- Cargo workspaces (virtual manifest, members, default-members, shared lock/target): https://doc.rust-lang.org/cargo/reference/workspaces.html
- Rust book — Cargo workspaces: https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html
- Rust book — trait objects (`Box<dyn Trait>` when impls are open): https://doc.rust-lang.org/book/ch18-02-trait-objects.html
- `flock(2)` — advisory lock, released when fds close, `LOCK_EX|LOCK_NB`: https://man7.org/linux/man-pages/man2/flock.2.html
- `unix(7)` — abstract namespace (`sun_path[0] == '\0'`), auto-cleanup, `SO_PEERCRED`: https://man7.org/linux/man-pages/man7/unix.7.html
- XDG Base Directory Spec 0.8 — `$XDG_RUNTIME_DIR` for sockets/locks: https://specifications.freedesktop.org/basedir-spec/latest/
- EWMH 1.5 — `_NET_ACTIVE_WINDOW`: https://specifications.freedesktop.org/wm-spec/latest/ar01s03.html
- EWMH 1.5 — `_NET_WM_STATE_ABOVE`, window types: https://specifications.freedesktop.org/wm-spec/latest/ar01s05.html
- xdg-activation-v1 — token from click serial; `XDG_ACTIVATION_TOKEN` for children; KWin implements v1: https://wayland.app/protocols/xdg-activation-v1
- wlr-layer-shell-unstable-v1 — overlay/top layers, exclusive zone 0: https://wayland.app/protocols/wlr-layer-shell-unstable-v1
- winit 0.30 `WindowLevel` — **unsupported on Wayland**: https://docs.rs/winit/latest/winit/window/enum.WindowLevel.html
- GTK4 `gtk_window_present` — raise/unminimize/focus, compositor may still refuse: https://docs.gtk.org/gtk4/method.Window.present.html

---
*Architecture research for: Rust Linux desktop orbital-launcher toolbox*
*Researched: 2026-08-19*
