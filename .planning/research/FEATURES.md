# Feature Research

**Domain:** Rust Linux desktop orbital-launcher toolbox
**Researched:** 2026-08-19
**Confidence:** MEDIUM

Provider seam rates `webfetch` LOW even `--verified` and verified `websearch` MEDIUM. Competitor UX claims below are grounded in official pages fetched directly and cross-checked across products. Personal-use scope means "table stakes" are the locked interaction in `PROJECT.md`, not a public marketplace.

## Feature Landscape

The category splits into two entry models. **Search-box launchers** (Alfred, Raycast, Wox, Albert, Ulauncher, PowerToys Run, Rubick, and uTools' primary surface) hide tools behind type-to-find. **Tool suites** (DevToys, most PowerToys utilities) put many tools inside one app window. **xtools is neither.** Use a persistent always-on-top main orb that expands into three orbital function orbs, each spawning or focusing an independent Rust window. Do not ship a search box or a plugin-directory scan in v1.

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = this product feels incomplete. For xtools that means the locked click path plus daily-tool depth, not Alfred/Raycast coverage.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Always-on-top draggable main orb | Core Value: the entry must be visible without a hotkey | MEDIUM | Linux always-on-top + input region is the hard part; keep the orb small and clickable, not a full-screen overlay |
| Click main orb → three function orbs orbit it | User locked this menu; a list or immediate windows is a miss | MEDIUM | Hardcode Timestamp / JSON / Translate around the main orb. Collapse on second click |
| Click function orb → open or focus independent window | Tools are processes, not embedded panels | MEDIUM | Host only launches and focuses. Re-click focuses the existing window; never spawn a second copy |
| Single-instance per tool | Same-function multi-open was rejected | LOW | CopyQ-style: one process, later invoke shows/focuses it |
| Shared theme / controls / layout rhythm | Three windows must look like one suite | MEDIUM | Share one theme crate. Do not let each window invent chrome |
| Timestamp: Unix s/ms ↔ datetime + one-click copy | Daily log/API work; result must leave immediately | LOW | Ship 10-digit, 13-digit, RFC3339, and one custom format. Bidirectional. No timezone encyclopedia in v1 |
| JSON: format, minify, validate with error location | Boolean valid/invalid is incomplete | MEDIUM | Point at line/column (and a snippet). Do not add jq/JSONPath in v1 — DevToys splits that into a separate tester |
| Translate shell: input / output / language, swappable engine | User refused a hard-wired offline dictionary | MEDIUM | v1 is the window + one working engine adapter. Engine config, not model training |
| Hardcoded three-entry menu | Plugin folder-drop was deferred | LOW | Architecture may reserve a spawn API. Do not scan directories in v1 |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required by the search-box market, but they *are* this product.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Orbital 3-ball menu instead of a search box | Muscle-memory click: main → direction → tool. No keyword to remember | MEDIUM | uTools has a floating ball, but official docs say click = show search box. Invert that: the balls *are* the menu |
| Persistent mouse-first orb (no hotkey required) | Hands stay on the mouse; nothing to bind or forget | MEDIUM | Search-box family is keyboard-first. Gnome-Pie is radial but hotkey-triggered and not always on screen |
| Independent window processes as the default | A tool crash or hang cannot take down the orb or sibling tools | MEDIUM | uTools/Rubick can *detach* a plugin (`Ctrl+D` on uTools). xtools never embeds tool UI in the host |
| Three first-class tool windows, not palette results | Timestamp/JSON/Translate stay open and usable while you work | LOW | DevToys keeps tools inside one mega-window (plus multi-instance). PowerToys Run keeps them as launcher hits |
| Personal Linux, three tools done well | No marketplace, no account, no plugin trust surface | LOW | Rubick/uTools grow via npm/plugin stores. That is the opposite of v1 |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems. Do not build these in v1.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Search box / command palette | Every competitor has one; feels like "the" toolbox | User chose orbital balls. A box turns the orb into a uTools clone and steals the differentiator | Orbital 3-ball expand/collapse only |
| Plugin-directory scan (drop a file, it appears) | Easy extensibility; uTools/Rubick/Wox/Albert/Ulauncher/DevToys all grow this way | Trust, discovery, versioning, and menu layout explode before the three windows work | Hardcode three spawn targets; reserve a process API for later |
| Super panel / selection context menu | uTools official mouse path; inline timestamp + translate on selected text | Out of scope. Needs global input hooks and content sniffing | Open the matching tool window; user pastes |
| Global hotkeys | Alfred/Raycast/Albert/Ulauncher default | User rejected hotkeys as the entry. Conflicts with every compositor and existing bind | Click the always-on-top orb |
| Clipboard listen / Smart Detection | DevToys and uTools auto-route copied text | Surprising, privacy-adjacent, and not the entry model | Manual paste into the focused tool |
| Click main orb opens all three windows | "Faster" | User negated this. Destroys the menu step | Expand balls first; one window per later click |
| Always-visible three balls, no main | "Fewer clicks" | User wants one quiet orb until needed | One main orb; satellites only when expanded |
| Multi-instance tool windows | DevToys advertises multiple instances | User chose focus-existing | Single-instance lock + raise |
| Offline dictionary / local translation model | "Works without network" | Heavy, quality-variable, binds the product to one engine | Swappable engine adapter; ship one that works |
| jq / JSONPath query in v1 | Power users; DevToys has a JSONPath tester | Scope leak. Format/minify/error-location is the job | Validate with a caret; query later if ever |
| Embed tool UI inside the host | Simpler process model | Violates "host only launches." One crash kills the orb | Independent Rust window processes |
| Plugin marketplace / npm install | Rubick/uTools growth loop | Not personal-v1. Supply-chain and UI chrome for nobody else | Add a fourth hardcoded binary later if needed |
| Installer, autostart, multi-DE packaging | "Real app" | Personal Linux use. Packaging is a distribution project | Run from the repo/target dir |
| Windows / macOS | Competitors are cross-platform | Current machine is Linux; Wayland/X11 is already enough surface | Linux only |
| Auto-paste clipboard into the search/tool | uTools "自动粘贴到搜索框" | There is no search box; auto-paste is surprising | Explicit paste in the tool window |

## Feature Dependencies

```
Always-on-top draggable main orb
    └──requires──> Linux overlay / always-on-top window
    └──requires──> Drag vs click hit-testing

Orbital 3-ball expand / collapse
    └──requires──> Always-on-top draggable main orb
    └──requires──> Fixed 3-entry menu (hardcoded)
    └──enhances──> Edge-aware orbit placement (v1.x)

Spawn or focus independent tool window
    └──requires──> Orbital 3-ball expand / collapse
    └──requires──> Per-tool single-instance lock
    └──requires──> Process spawn with inherited display env

Shared theme crate
    └──requires──> Same GUI toolkit for all four binaries
    └──enhances──> Timestamp window
    └──enhances──> JSON window
    └──enhances──> Translate window

Timestamp window
    └──requires──> Spawn or focus independent tool window
    └──requires──> Shared theme crate

JSON window
    └──requires──> Spawn or focus independent tool window
    └──requires──> Shared theme crate

Translate window (shell + one engine)
    └──requires──> Spawn or focus independent tool window
    └──requires──> Shared theme crate
    └──requires──> Engine adapter trait
    └──conflicts──> Hard-wired offline model

Search box ──conflicts──> Orbital 3-ball menu
Plugin directory scan ──conflicts──> Hardcoded 3-entry menu
Super panel / clipboard listen / global hotkeys ──conflicts──> Mouse-orb-only entry
Multi-instance ──conflicts──> Single-instance focus
```

### Dependency Notes

- **Orbital menu requires the main orb:** Satellites are positioned around the live main-orb origin. If the orb is not always-on-top and draggable first, the menu has nothing to orbit.
- **Spawn/focus requires the menu plus a single-instance lock:** Without the lock, a second click opens a second window (rejected). Without spawn, the balls are decoration.
- **Shared theme enhances all three tools:** Visual "one suite" is a Validated-path requirement. Land the theme crate before polishing individual windows, or the three UIs will diverge.
- **Translate shell requires an engine trait, not a model:** The window is the product; the engine is a plug. Binding v1 to an offline dictionary conflicts with "engine swappable."
- **Search box conflicts with orbital menu:** uTools proves the failure mode — once a box exists, the ball becomes a shortcut to the box. Do not add it "just in case."
- **Plugin scan conflicts with hardcoded three:** A scan implies dynamic layout, icons, crash isolation policy, and a trust story. That is a later milestone, not a v1 stretch.

## MVP Definition

### Launch With (v1)

Minimum viable product — what is needed to validate the locked path. Nothing else.

- [ ] Always-on-top draggable main orb — without this there is no product
- [ ] Click main → Timestamp / JSON / Translate orbs appear around it — the menu contract
- [ ] Click main again → satellites collapse — toggle, not a one-way pop
- [ ] Click a function orb → start that Rust window process; if already running, focus it — host does not embed UI
- [ ] Three windows share one theme, controls, and layout rhythm — looks like one suite
- [ ] Timestamp: Unix seconds/milliseconds ↔ datetime; one-click copy of 10-digit, 13-digit, RFC3339, custom — daily log/API use
- [ ] JSON: format, minify, validate, mark error location — not just pass/fail
- [ ] Translate: unified input / output / language UI plus one working swappable engine — shell first
- [ ] Menu entries hardcoded to those three binaries — no directory scan

### Add After Validation (v1.x)

Features to add once the click path is daily-usable.

- [ ] Persist main-orb position across restarts — trigger: first week of real use, orb reset is annoying
- [ ] Edge-aware orbit (flip/shift satellites when the main orb sits on a screen edge) — trigger: dragging to a corner clips a ball
- [ ] Remember last timestamp custom format and last translate language pair — trigger: re-entering the same values every open
- [ ] Translate engine config UI (endpoint / key / command) — trigger: swapping the v1 engine for another
- [ ] Focus-steal robustness on the running compositor — trigger: "already open" does not raise on Wayland

### Future Consideration (v2+)

Features to defer until the three-window path is proven.

- [ ] Plugin-directory scan / drop-in binaries — why defer: explicit later work; do not invent a marketplace
- [ ] Fourth+ tools (Base64, hash, regex, color picker) — why defer: DevToys already is that catalog
- [ ] Search box / command palette — why defer: user rejected; would erase the differentiator
- [ ] Super panel / clipboard listen / global hotkeys — why defer: out of scope; different product
- [ ] Offline translation model — why defer: user refused a bound local engine
- [ ] jq / JSONPath — why defer: format/validate is the v1 job
- [ ] Installer, autostart, multi-DE packaging, Windows/macOS — why defer: personal Linux only
- [ ] Multi-instance windows — why defer: user chose focus-existing

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Always-on-top draggable main orb | HIGH | MEDIUM | P1 |
| Orbital 3-ball expand / collapse | HIGH | MEDIUM | P1 |
| Spawn or focus independent tool window | HIGH | MEDIUM | P1 |
| Single-instance per tool | HIGH | LOW | P1 |
| Shared theme crate | HIGH | MEDIUM | P1 |
| Timestamp convert + one-click copy | HIGH | LOW | P1 |
| JSON format / minify / error location | HIGH | MEDIUM | P1 |
| Translate shell + one engine | HIGH | MEDIUM | P1 |
| Hardcoded three-entry menu | HIGH | LOW | P1 |
| Persist orb position | MEDIUM | LOW | P2 |
| Edge-aware orbit | MEDIUM | MEDIUM | P2 |
| Remember last formats / languages | MEDIUM | LOW | P2 |
| Translate engine config UI | MEDIUM | LOW | P2 |
| Plugin directory scan | LOW | HIGH | P3 |
| Search box / command palette | LOW | HIGH | P3 |
| Super panel / clipboard / hotkeys | LOW | HIGH | P3 |
| Extra DevToys-style tools | LOW | HIGH | P3 |
| Installer / cross-platform | LOW | HIGH | P3 |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

## Competitor Feature Analysis

Two families. **Search-box** products treat the keyboard query as the product. **Orbital / mouse** products treat direction or a persistent widget as the product. xtools is the second family, and must not grow a box to "catch up."

| Feature | Search-box family (Alfred, Raycast, Wox, Albert, Ulauncher, PowerToys Run, Rubick) | uTools (search + ball + panel) | DevToys / PowerToys utilities | Gnome-Pie | xtools approach |
|---------|-------------------------------------------------------------------------------------|--------------------------------|-------------------------------|-----------|-----------------|
| Primary entry | Global hotkey → type in a box | Alt+Space search box; ball click *opens the box* | App window + in-app search | Hotkey → pie at cursor | Always-on-top main orb; click expands three function orbs |
| Floating ball | Absent | Official: click = search, long-press = screenshot | Absent (DevToys has compact overlay, not an orb) | Absent (pie is ephemeral) | The ball *is* the menu. Never open a search box |
| Tool surface | Result row or embedded extension view | Plugin UI in the box; `Ctrl+D` detaches a window | Tools live inside one suite window; multi-instance allowed | Launches apps/files/keystrokes; no tool windows | Independent Rust window process per tool; focus if live |
| Extensibility | Workflows / extensions / plugin store | 3000+ plugin market | 30 built-ins + NuGet extensions | User-defined pies/slices | v1 hardcoded three binaries. No scan, no store |
| Timestamp | Occasional workflow/plugin | Super panel can decode a selected timestamp inline | Official Date / timestamp converter | Not a tool | Dedicated window: s/ms ↔ datetime + copy 10 / 13 / RFC3339 / custom |
| JSON | Occasional extension | Plugin | Official formatter; JSONPath is a separate tester | Not a tool | Dedicated window: format, minify, validate with error location. No JSONPath in v1 |
| Translate | Occasional extension | Super panel inline English translation | Not a first-class DevToys tool | Not a tool | Dedicated window: input / output / languages; engine swappable |
| Clipboard / selection | Alfred clipboard history; many auto-detect | Auto-paste into box; super panel on selection | Smart Detection | Optional clipboard slices | None in v1. User pastes |
| Single instance | One launcher process; tools vary | One host; plugins may detach | Multiple instances advertised | One daemon + pies | One host orb + one window per tool |
| Platform | Mac / Win / Linux split by product | Win / mac / Linux | Win / mac / Linux | Linux | Linux personal only |

**Prescription:** Clone uTools' *job* (reach a tiny daily tool in one or two gestures) and Gnome-Pie's *geometry* (remember a direction). Do not clone uTools' search box, super panel, or plugin market. Do not clone DevToys' catalog or clipboard brain. Do not clone Alfred/Raycast/Wox/Albert/Ulauncher/PowerToys Run at all.

## Sources

- uTools homepage: https://www.u-tools.cn/index.html (redirected from https://u.tools/)
- uTools preferences (search box, 悬浮球, 超级面板, `Ctrl+D` detach): https://www.u-tools.cn/docs/guide/preferences.html
- uTools super panel: https://www.u-tools.cn/docs/guide/uTools-super-panel.html
- Alfred 5: https://www.alfredapp.com/
- Raycast: https://www.raycast.com/
- Albert: https://albertlauncher.github.io/
- Ulauncher: https://ulauncher.io/
- Wox: https://wox-launcher.github.io/Wox/
- Microsoft PowerToys overview (updated 2026-06-13): https://learn.microsoft.com/en-us/windows/powertoys/
- DevToys: https://devtoys.app/ and https://github.com/DevToys-app/DevToys
- CopyQ: https://hluk.github.io/CopyQ/ and https://copyq.readthedocs.io/en/latest/basic-usage.html
- Gnome-Pie: http://schneegans.github.io/gnome-pie and https://github.com/Schneegans/Gnome-Pie
- Rubick (Electron uTools-like toolbox): https://github.com/rubickCenter/rubick
- Project lock: `/home/perfect/Desktop/xtools/.planning/PROJECT.md`

---
*Feature research for: Rust Linux desktop orbital-launcher toolbox*
*Researched: 2026-08-19*
