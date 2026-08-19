# Pitfalls Research

**Domain:** Rust Linux desktop orbital-launcher toolbox
**Researched:** 2026-08-19
**Confidence:** MEDIUM

Roadmap phases used below:

- **Host/Orb phase** — always-on-top main ball, orbital 3-ball menu, hit-testing, drag, edges, compositor path
- **Tool-window phase** — independent Rust processes, focus-existing, shared theme, clipboard, timestamp/JSON UX
- **Translation-engine phase** — pluggable engine + secrets (after the translation window shell exists)

## Critical Pitfalls

### Pitfall 1: Building the host ball as an xdg-shell / winit AlwaysOnTop window

**What goes wrong:**
The main ball is a normal application window. On Wayland it sits in the stacking order with every other toplevel, gets tiled or maximized away, and `AlwaysOnTop` is a no-op. On X11 it may work by accident. The core value ("always floating, click to expand") dies on the user's daily compositor.

**Why it happens:**
winit 0.30 documents `WindowLevel` as **unsupported on Wayland**. iced/egui sit on winit. `xdg-shell` clients cannot set absolute position or force z-order; that is compositor policy. Developers copy the Windows/X11 "always on top" flag and assume it is portable.

**How to avoid:**
Use **layer-shell for the host only**. On Wayland: `zwlr_layer_shell_v1` via `gtk4-layer-shell` 0.8.x. Call `gtk4_layer_shell::is_supported()` first; `init_layer_shell()` **before** the window is mapped; `set_layer(Layer::Overlay)`; `set_exclusive_zone(-1)` so panels do not shove the ball; `set_keyboard_mode(KeyboardMode::None)`. Do not implement the host in iced/egui/winit. Tool windows stay ordinary `xdg-shell` toplevels — they must not be layer-shell surfaces.

X11 fallback for the host: override-redirect + 32-bit ARGB + `ShapeInput`, or a managed window with `_NET_WM_STATE_ABOVE`. Prefer native Wayland layer-shell over XWayland.

**Warning signs:**
`WindowLevel::AlwaysOnTop` / `with_window_level` in host code. Host crate depends on iced/egui but not `gtk4-layer-shell` / layer-shell bindings. Ball disappears under a maximized browser. Ball cannot be placed at an arbitrary (x, y) on Wayland.

**Phase to address:**
Host/Orb phase — first vertical slice. If this is wrong, every later phase is built on a dead overlay.

---

### Pitfall 2: Transparent overlay without a tight input region (desktop click-through)

**What goes wrong:**
The reliable way to place a free-floating ball on Wayland is a large (often output-sized) transparent layer-shell surface with the circle drawn inside it. The default input region is the **entire surface**. Clicks on "empty" pixels never reach the window underneath. The desktop is bricked wherever the overlay exists.

Alpha ≠ input. A fully transparent pixel still receives pointer events until `wl_surface.set_input_region` / `gdk_surface_set_input_region` excludes it.

**Why it happens:**
Layer-shell protocol: layer surfaces receive pointer/touch/tablet events normally; "If you do not want to receive them, set the input region on your surface to an empty region." GTK4: `gdk_surface_set_input_region`; `NULL` region means the **whole** surface is reactive. Developers set RGBA and stop.

**How to avoid:**
Keep the input region equal to the **union of visible circles** (main ball, plus function orbs only while expanded), in **surface-local** coordinates. Rebuild and commit the region on every move, expand, collapse, and scale change. Use circle (or tight bounding disks), not the window rectangle. On X11 use `ShapeInput` separately from the visual shape. Probe `gdk_display_supports_input_shapes()`.

**Warning signs:**
Cannot click terminals/browser "around" the ball. Hover cursors change over empty glass. Input region is set once at startup and never updated when orbs appear.

**Phase to address:**
Host/Orb phase. Treat input-region updates as part of the draw/layout loop, not a polish item.

---

### Pitfall 3: Exclusive zone or exclusive keyboard grab

**What goes wrong:**
`exclusive_zone > 0` tells the compositor to reserve a strip like a panel. Maximized windows shrink. A 64px floating toy has stolen a dock-sized chunk of every workspace.

`keyboard_interactivity = exclusive` on overlay/top steals **all** keyboard focus from the desktop. Super/Alt shortcuts die while the ball exists. The protocol documents this mode for lock screens and password prompts, not widgets.

**Why it happens:**
`gtk4-layer-shell::auto_exclusive_zone_enable()` looks convenient. Copy-paste from panel/dock samples. Default exclusive zone is 0 (surface may be **moved** to avoid real panels). Developers then "fix" that by setting exclusive instead of `-1`.

**How to avoid:**
Host: `exclusive_zone = -1` (stay put; do not reserve space). Never call `auto_exclusive_zone_enable` on the ball. Keyboard mode **None** for the host. Function orbs do not need keys. Tool windows use normal xdg-shell focus. If a future popup on the layer surface needs keys, use `OnDemand` (protocol v4+), never Exclusive.

**Warning signs:**
Maximized windows leave a gap the size of the overlay. Typing in a terminal does nothing until the ball is killed. `KeyboardMode::Exclusive` or `auto_exclusive_zone_enable` in host code.

**Phase to address:**
Host/Orb phase.

---

### Pitfall 4: Assuming layer-shell exists on every Linux desktop (GNOME/Mutter)

**What goes wrong:**
`get_layer_surface` never appears in the registry. The host fails to map, or the process exits. GNOME on Wayland is a hard miss.

wayland.app compositor table (protocol v5): `zwlr_layer_shell_v1` is **absent** on Mutter 49.2, Weston, Cage; present on KWin 6.6, Sway, Hyprland, niri, COSMIC, Labwc, river, Wayfire. GNOME issue mutter#973 requested the protocol; Mutter still does not implement it.

**Why it happens:**
wlroots-centric tutorials treat layer-shell as "the Wayland way." Personal-use scope hides the compositor question until the first GNOME session.

**How to avoid:**
Call `gtk4_layer_shell::is_supported()` at startup. If false: either run the host on X11 (`GDK_BACKEND=x11` + `_NET_WM_STATE_ABOVE` / override-redirect) or refuse with a one-line "needs layer-shell or X11" message. Do **not** invent a GNOME Shell extension for v1. Pick the daily compositor (KDE / wlroots / COSMIC) or accept the X11 fallback. Tool windows do not need layer-shell and work on Mutter.

**Warning signs:**
No `is_supported()` branch. CI/dev only on Hyprland/Sway. Host crashes on GNOME Wayland with a protocol/role error.

**Phase to address:**
Host/Orb phase (detect + fallback). Re-verify on the user's actual session before calling the overlay done.

---

### Pitfall 5: Single-instance via lock file without xdg-activation

**What goes wrong:**
Second click of a function orb starts another process, sees a lock, exits. The existing tool window stays buried. User thinks the orb is broken. Or the second process opens a second window, violating the locked decision "already open → focus, do not spawn."

On Wayland an app **cannot take focus**. It can only **receive** it with a valid `xdg-activation-v1` token. Compositors reject tokens that lack a recent input serial / requesting surface. A lock file does not raise a window.

**Why it happens:**
`single-instance` 0.3.3 is a 2021 lock-file crate. X11 `XSetInputFocus` / `_NET_ACTIVE_WINDOW` habits. Tokens look optional in the protocol; compositors treat incomplete tokens as spam (Broulik, 2025-08: token may be ignored; Extreme focus-stealing prevention activates only with a valid token).

**How to avoid:**
Per tool binary: own a well-known D-Bus name (`org.xtools.Timestamp` etc.) implementing `org.freedesktop.Application`. On orb click the **host** (which has the pointer serial) requests a token (`set_serial` + `set_surface` + `set_app_id`), then:

- first launch: `Command` with `XDG_ACTIVATION_TOKEN=<token>`; child unsets the env var immediately after read and calls `activate`
- already running: D-Bus `Activate` with `platform_data["activation-token"]`

Do not use the lock-file crate as the focus path. X11 may still use `_NET_ACTIVE_WINDOW`; keep the same D-Bus Activate so both backends share one policy.

**Warning signs:**
Two JSON windows. Orb click no-ops while the window exists on another workspace. Host never reads a pointer serial. Child never sees `XDG_ACTIVATION_TOKEN`. Demand-attention hint in the task bar instead of raise.

**Phase to address:**
Tool-window phase. The host must collect the serial in the Host/Orb phase so the token can be minted.

---

### Pitfall 6: Spawning GUI children with a cleaned environment or no session sockets

**What goes wrong:**
`Command::env_clear()`, custom `env`, systemd-from-root, or `setsid` + dropped `XDG_RUNTIME_DIR` produces a child that cannot connect to the compositor: "no DISPLAY", missing `wayland-0`. Or the child starts but cannot activate because the token was never passed, or was inherited by a **grandchild** and consumed twice.

**Why it happens:**
"Sanitize the environment" advice from CLI tools. Forgetting that GUI children need `WAYLAND_DISPLAY`, `XDG_RUNTIME_DIR`, `XDG_SESSION_TYPE`, `DISPLAY` (X11/XWayland), and often `DBUS_SESSION_BUS_ADDRESS`. Protocol: child must unset `XDG_ACTIVATION_TOKEN` after reading so it does not propagate.

**How to avoid:**
Use `std::process::Command` **without** `env_clear`. Inherit the host session. Set only `XDG_ACTIVATION_TOKEN` (and maybe `XDG_CURRENT_DESKTOP` if already present). Resolve tool binaries from the same install prefix, not a login-shell `PATH` rebuild. Reap children (`try_wait` / a dedicated reaper) so zombies do not leak. Do not daemonize tools out of the graphical session. Host crash must not SIGKILL tools — they are independent windows.

**Warning signs:**
Works when launched from a terminal, fails from the orb. `WAYLAND_DISPLAY` empty in the child. Activation works only on the first spawn. Defunct tool PIDs accumulate.

**Phase to address:**
Tool-window phase (spawn contract). Host/Orb phase must not clear env "for safety."

---

### Pitfall 7: HiDPI / fractional scale applied to the wrong coordinate space

**What goes wrong:**
Orbs draw at the right visual size but the input region is 2× too small (or 2× too large). Clicks on the visible disk miss; clicks on empty glass hit. Dragging jumps. Saved positions restore at the wrong place after a scale change or monitor move.

**Why it happens:**
Wayland input regions and layer-shell sizes are **surface-local**. Buffers are scaled with `wl_surface.set_buffer_scale` (integer) or `wp_fractional_scale_v1`. GTK widget sizes are already surface-local; raw pixel math from `gdk_surface_get_scale_factor()` applied a second time double-scales. Mixed 1×/2×/125% outputs make "use physical pixels everywhere" fail.

**How to avoid:**
Store the ball position in **surface-local logical units relative to the current output**. Layout, hit-test, and `set_input_region` all use that space. Let GTK own buffer scale. On `scale` / `enter` / monitor change: recompute layout and input region; clamp position to the new output. Never persist raw device pixels.

**Warning signs:**
Works on 1×, broken on 2× laptop panel. Hit box is a quarter of the disk. After dragging to an external monitor the ball is off-screen or tiny.

**Phase to address:**
Host/Orb phase. Tool windows: let the toolkit handle scale; do not hand-roll DPI in shared widgets.

---

### Pitfall 8: Orbital menu clipped at screen edges

**What goes wrong:**
Main ball sits near a corner. Expand places one or two function orbs off-screen or under a panel. Those tools are unreachable. Fitts' Law edge advantage is destroyed because the target no longer exists.

**Why it happens:**
Fixed angles (e.g. −90° / −30° / +30°) around a center that the user is allowed to drag anywhere. No pre-flight of the expanded bounding box against the output work area.

**How to avoid:**
Keep the **main ball where the user put it**. Do not jump the host on expand. Before mapping orbs, compute the three disk rects. If any disk leaves the output (or overlaps a known exclusive panel strip), **rotate the constellation** toward the interior, or shrink radius, until all three disks are fully visible. Prefer rotation over moving the main ball. On startup, clamp the saved main position so a later expand can still fit. Multi-monitor: bind the layer surface to the output the ball is on; if that output disappears, relocate to the primary output center.

**Warning signs:**
Only two orbs appear in a corner. An orb is half-visible and not clickable (input region clipped). Expand near the top panel hides the timestamp ball.

**Phase to address:**
Host/Orb phase. Write an edge-case matrix (center, each edge, each corner, next to panel) into the phase verification.

---

### Pitfall 9: Drag vs click without slop (or orbs that cover the main ball)

**What goes wrong:**
Every press-move of one pixel starts a drag, so the user can never toggle the menu. Or every drag is interpreted as a click, so the ball teleports and then expands. Function orbs spawn on top of the main disk; the next click hits a tool instead of collapse. Rectangular hit-tests on circular art make "empty" corners steal clicks (and the real disk miss).

**Why it happens:**
No press-slop. Orbital radius ≤ `main_r + func_r`. Hit-testing uses the layer-shell / widget rectangle because that is what the event gives you.

**How to avoid:**
Press → if movement ≤ slop (about 6–8 logical px) and release is inside the main disk, it is a **click** (toggle menu). If movement exceeds slop, it is a **drag**; do not toggle on release. Keep `radius >= main_r + func_r + gap` (gap ≥ 8 logical px) so disks never overlap. Hit-test **circles** (`dx²+dy² ≤ r²`) for both compositor input region and in-process events. Expanded state: main disk remains a full target for collapse; function orbs are separate targets; the glass between them is click-through.

**Warning signs:**
Cannot open the menu without moving the ball. Cannot move the ball without opening tools. Clicking the main ball after expand launches a tool. Clicks in the square corners of a circular sprite do something.

**Phase to address:**
Host/Orb phase. This is the core interaction; do not defer slop or disk math.

---

### Pitfall 10: Translation API keys (and other secrets) in source or plaintext config

**What goes wrong:**
A DeepL/Google/OpenAI key is committed, screenshotted, or copied with the repo. Personal-use is not a mitigation once the tree is pushed or synced.

**Why it happens:**
v1 "just make translate work." `.env` next to the binary. Hardcoded default "so it runs on my machine."

**How to avoid:**
Never put keys in git, `include_str!`, or world-readable config. Translation window: empty engine until the user pastes a key; store it in **Secret Service** via `keyring` (v1 feature) or `keyring-core` + a Linux secret-service backend (`oo7` / libsecret). Load into memory only for the request; do not log it. Config may store engine **name** and endpoint, not the secret. If Secret Service is missing, refuse with "set up a keyring" rather than writing `~/.config/xtools/api_key`.

**Warning signs:**
`api_key = "` in the repo. Translate works on a fresh clone with no prompt. Debug logs print `Authorization: Bearer`.

**Phase to address:**
Translation-engine phase. The translation **window shell** can ship without a key; the first real engine must not.

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| iced/egui host with `AlwaysOnTop` | Fast prototype on X11 | Wayland overlay rewrite | Never for the host; fine for throwaway spikes |
| One output-sized surface, input region "later" | Ball is visible | Desktop is unusable; hard to retrofit | Never ship; empty region on first map |
| Lock file instead of D-Bus + activation | Single-instance in one afternoon | Focus-existing never works on Wayland | Never as the raise path |
| Copy-paste theme constants into each binary | Three windows look OK today | Drift on the next color/spacing change | Never — one shared theme crate from the first tool |
| `auto_exclusive_zone_enable` | Ball not covered by panels | Reserves panel geometry | Never on a floating overlay |
| XWayland-only host | Overlay works on GNOME | Blurry, no layer-shell, future Wayland-only sessions break | Only as explicit fallback when `is_supported() == false` |
| Hardcoded translate key | Demo works | Secret in git history forever | Never |
| Plugin directory "while we are here" | Feels architectural | Out of scope; breaks v1 | Never in v1 |

## Integration Gotchas

Common mistakes when connecting to compositors, session services, and engines.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| `zwlr_layer_shell_v1` | Attach a buffer before the first `configure` | Initial commit with **no** buffer; `ack_configure`; then attach |
| `gtk4-layer-shell` | `init_layer_shell` after present | Init before map; check `is_supported()` |
| `xdg-activation-v1` | Child requests its own token with no serial | Host mints token from the orb-click serial and passes it |
| D-Bus `org.freedesktop.Application` | Ignore `platform_data` | Read `activation-token` (and `desktop-startup-id` on X11) |
| Secret Service | Write keys to `~/.config` | `keyring` / `oo7`; prompt in the translation UI |
| Clipboard (timestamp copy) | `wl-copy` and exit, or close window immediately | Keep the tool process alive as the data source; use GTK clipboard on a live window. Wayland has no server-side clipboard store |
| Child `Command` | `env_clear` + reconstructed PATH | Inherit session env; set only the activation token |
| Translate HTTP | Block the UI thread | Async request; timeout; show engine errors in the output pane |

## Performance Traps

Patterns that work at small scale but fail as usage grows. This project is personal-use; thresholds are desktop-local, not user-count.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Full-output ARGB buffer every frame | GPU fan, compositor stutter | Damage only the disks; idle with no redraw | 4K@2× overlay + vsync already |
| JSON parse + format on UI thread | Window freeze on paste | Cap size; parse off-thread; stream error spans | Multi-megabyte logs |
| Rebuilding input region every pointer-motion during drag | Janky drag | Update region on release / end of move, not every pixel, unless hit-testing requires it | High polling rate mice |
| One layer surface per orb | Stacking/anchor fights, 4× protocol chatter | One host surface; draw all disks into it | First expand |
| Unbounded translate retries | Rate-limit / billed API | Single in-flight request; visible cancel | First paid engine |

## Security Mistakes

Domain-specific issues beyond generic web OWASP.

| Mistake | Risk | Prevention |
|---------|------|------------|
| API keys in source / `.env` committed | Key leak via git, backups, screenshots | Secret Service only; never a default key |
| Exclusive keyboard grab on the overlay | Keylogger-shaped UX; captures passwords meant for other apps | `KeyboardMode::None` on host |
| Logging request URLs with tokens | Secrets in journald | Redact; do not `Debug` the engine client |
| World-readable config in `/tmp` lock paths | Another user on the box impersonates the instance | Session D-Bus name + `XDG_RUNTIME_DIR` sockets, mode 0700 |
| Passing the activation token to every child forever | Grandchild steals focus later | Unset `XDG_ACTIVATION_TOKEN` after the first read |
| Translating selection without user action | Accidental exfil of secrets in the input box | Only send on explicit Translate; no clipboard listener (out of scope anyway) |

## UX Pitfalls

Common user experience mistakes in this domain.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Orbs spawn off-screen at edges | Tools missing; feels broken | Rotate/shrink constellation; never clip a disk |
| No drag/click slop | Cannot separate move vs toggle | 6–8 logical px slop; click toggles, drag moves |
| Function orbs overlap the main ball | Collapse target gone; mis-fires a tool | `radius >= r_main + r_orb + gap` |
| Rectangular hit-test on a circle | Clicks in transparent corners steal desktop or miss the disk | Circle tests + matching input region |
| Second orb click does nothing visible | "Single-instance" looks like a dead control | Raise + focus via activation token; if compositor refuses, urgent/taskbar hint |
| Three tool windows look like three apps | Product does not feel like one toolbox | Shared theme crate: color, type, spacing, header pattern |
| JSON only says "invalid" | Cannot fix the paste | Error span: line/column + caret in the editor |
| Timestamp copy, then window closed instantly | Wayland paste is empty | Keep the window up; clipboard owner is this process |
| Host uses exclusive keyboard | Desktop shortcuts die | No keyboard on the overlay |
| Expand moves the main ball | User's parked position is lost | Main ball stays; only orbs rearrange |

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **Always-on-top ball:** Visible on X11 ≠ done — verify on Wayland with a maximized window **over** the ball; confirm layer-shell Overlay, not `WindowLevel`
- [ ] **Click-through glass:** Ball looks circular — click the transparent corners and the area between expanded orbs; those clicks must reach the window below
- [ ] **Input region after expand:** Region updated when orbs appear/disappear/move; not a single startup rectangle
- [ ] **No exclusive zone:** Maximize a window; it must use the full work area (no ball-sized gap)
- [ ] **No keyboard steal:** Overlay mapped; Super/Alt and typing in a terminal still work
- [ ] **GNOME/unsupported compositor:** `is_supported() == false` is a defined fallback, not a crash
- [ ] **Edge constellation:** Expand at all four corners and against the top panel; all three orbs fully on-screen and hittable
- [ ] **Drag vs click:** Nudge 2 px must not toggle; a still click must not move; slop is in logical px
- [ ] **Orb overlap:** Function disks do not cover the main disk; main click still collapses
- [ ] **Focus existing:** With JSON already open and buried, orb click raises it; `ps` shows one process
- [ ] **Activation token:** Host sets `XDG_ACTIVATION_TOKEN` or D-Bus `activation-token` from the **click serial**; child unsets env
- [ ] **Spawn env:** Child started from the orb (not a terminal) still sees `WAYLAND_DISPLAY` / `XDG_RUNTIME_DIR`
- [ ] **HiDPI:** On scale 2 and a fractional output, visual disk and hit disk coincide
- [ ] **Clipboard:** Timestamp "copy" then paste into another app **while the tool window is still open**; do not claim success if the process exited
- [ ] **JSON errors:** Invalid JSON reports location, not only a boolean
- [ ] **Theme crate:** Changing one token updates all three tool windows
- [ ] **Secrets:** Fresh clone cannot translate until a key is entered; `git grep` finds no provider keys
- [ ] **Layer-shell configure:** Host does not attach a buffer before the first configure (protocol error)

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Host built on winit/iced | HIGH | Throw away host windowing; keep tool crates. Reimplement host with GTK4 + gtk4-layer-shell |
| Missing input region shipped | LOW | Add region updates; no protocol change |
| Exclusive zone reserved | LOW | Set `-1`, disable auto exclusive, restart host |
| GNOME Wayland host dead | MEDIUM | Add `is_supported` + X11 fallback; do not rewrite tools |
| Lock-file single-instance | MEDIUM | Replace with D-Bus name + activation; keep the same app-ids |
| No activation tokens | MEDIUM | Thread pointer serial from orb click; mint token in host; pass to child |
| Env-cleared spawn | LOW | Delete `env_clear`; inherit session |
| Scale/hit mismatch | MEDIUM | Move all geometry to surface-local; invalidate on scale events |
| Edge-clipped orbs | LOW | Add rotate/shrink pre-pass; keep main position |
| Drag/click / overlap | LOW | Slop + minimum radius + circle hit-test |
| Key in git | HIGH | Rotate the provider key; `git filter-repo` / accept history leak; move to keyring |
| Clipboard empty after copy | LOW | Stop exiting after copy; hold GTK clipboard on the live window |

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| xdg-shell / winit AlwaysOnTop host | Host/Orb | Maximized app cannot cover the ball on a layer-shell compositor |
| Full-surface input region | Host/Orb | Clicks on glass reach the desktop; only disks receive events |
| Exclusive zone / exclusive keyboard | Host/Orb | Maximize fills the work area; desktop shortcuts still work |
| Mutter/GNOME has no layer-shell | Host/Orb | `is_supported()` false → X11 or explicit error; no crash |
| Edge-clipped orbs | Host/Orb | Corner/edge matrix: three full disks, all hittable |
| Drag vs click / orbs covering main | Host/Orb | Slop test + overlap geometry assert + collapse still works |
| HiDPI input/layout mismatch | Host/Orb | Scale 1, 2, and fractional: visual == hit |
| Spawn env / activation serial plumbing | Host/Orb (serial + inherit) + Tool-window | Child env dump; token present on first launch |
| Focus-existing without raise | Tool-window | Buried window comes forward on orb click; one PID |
| Theme drift across processes | Tool-window | One crate; screenshot three windows after a token change |
| Wayland clipboard owner | Tool-window | Copy from timestamp while window lives; paste elsewhere |
| JSON error without location | Tool-window | Fixture with a mid-file syntax error shows line/col |
| API keys in source | Translation-engine | Empty keyring → prompt; no secrets in tree |

**Phase ordering implication:** Do not start tool windows until the host can (1) stay on top via layer-shell, (2) click-through glass, (3) emit a pointer serial. Do not start a real translate engine until the translation window can store a secret.

## Sources

Official / primary (MEDIUM via `classify-confidence --provider webfetch` / `--provider websearch --verified`; protocol text itself is current as of fetch 2026-08-19):

- [wlr-layer-shell-unstable-v1](https://wayland.app/protocols/wlr-layer-shell-unstable-v1) — layers, exclusive zone −1/0/>0, keyboard none/exclusive/on_demand, default pointer delivery, configure-before-buffer, compositor support table (Mutter = unsupported)
- [xdg-activation-v1](https://wayland.app/protocols/xdg-activation-v1) — token + serial + surface; `XDG_ACTIVATION_TOKEN`; child must unset; compositor may ignore
- [Desktop Entry Spec — D-Bus Activation](https://specifications.freedesktop.org/desktop-entry/latest/dbus.html) — `org.freedesktop.Application` + `activation-token` in `platform_data`
- [Gdk.Surface.set_input_region](https://docs.gtk.org/gdk4/method.Surface.set_input_region.html) — region vs alpha; `NULL` = fully reactive
- [Gdk.Display.supports_input_shapes](https://docs.gtk.org/gdk4/method.Display.supports_input_shapes.html)
- [gtk4-layer-shell 0.8.1](https://crates.io/crates/gtk4-layer-shell) / [LayerShell trait](https://docs.rs/gtk4-layer-shell/latest/gtk4_layer_shell/trait.LayerShell.html) — `is_supported`, `init_layer_shell` before map, exclusive zone, keyboard mode
- [winit 0.30 `WindowLevel`](https://docs.rs/winit/latest/winit/window/enum.WindowLevel.html) — **Wayland unsupported**
- [Wayland Book — HiDPI](https://wayland-book.com/surfaces-in-depth/hidpi.html) — surface-local vs `set_buffer_scale`
- [GNOME mutter#973](https://gitlab.gnome.org/GNOME/mutter/-/issues/973) — layer-shell request against Mutter
- [Kai Uwe Broulik, On Window Activation (2025-08)](https://blog.broulik.de/2025/08/on-window-activation/) — cannot take focus; Extreme FSP requires a valid token
- [keyring 4.1 docs](https://docs.rs/keyring/latest/keyring/) — Linux Secret Service via the keyring ecosystem
- [NN/g Fitts's Law](https://www.nngroup.com/articles/fitts-law/) — edge targets; clipped radial items lose the model
- [Arch Wiki: Clipboard](https://wiki.archlinux.org/title/Clipboard) — Wayland selection is owned by the source process

Community / corroboration (do not treat as stronger than protocol text):

- [single-instance 0.3.3](https://crates.io/crates/single-instance) — lock file only; last release 2021-12; insufficient for Wayland raise
- Layer-shell vs xdg-shell positioning is compositor-owned; floating widgets use layer-shell + margins or an input-shaped overlay, not `xdg-shell` x/y

---
*Pitfalls research for: Rust Linux desktop orbital-launcher toolbox*
*Researched: 2026-08-19*
