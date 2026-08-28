# Windows Phase 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `xtools-time`, `xtools-json`, and `xtools-trans` build and run on Windows while preserving Linux behavior and singleton activation.

**Architecture:** Keep Slint UI and tool logic shared. Hide singleton IPC and Linux-only startup/window behavior behind `cfg`-selected platform modules. Phase 1 excludes `xtools-host`, GTK layer-shell, Linux tray, and the Windows floating ball.

**Tech Stack:** Rust 1.85+, Cargo workspace, Slint 1.17.1, Windows named pipes, existing Unix abstract sockets on Linux.

## Global Constraints

- Phase 1 must not port `xtools-host`.
- Linux behavior and existing Linux IPC remain unchanged.
- Windows must not compile GTK layer-shell, X11, KWin, or Linux socket code.
- Commands remain `RAISE`, `RAISE <activation-token>`, and `QUIT`.
- Do not add a third-party dependency unless the Windows named-pipe implementation cannot be completed with the current dependency set.
- Every new observable singleton behavior gets a focused test.

---

### Task 1: Split singleton IPC into platform modules

**Files:**
- Modify: `crates/xtools-ui/src/instance.rs`
- Create: `crates/xtools-ui/src/instance/unix.rs`
- Create: `crates/xtools-ui/src/instance/windows.rs`
- Modify: `crates/xtools-ui/src/lib.rs`
- Test: platform module unit tests in the implementation files

**Interfaces:**
- Produces the existing public API: `InstanceCommand`, `claim_instance`, `terminate_instance`, `raise_instance`, `accept_command`, `accept_raise`.
- `claim_instance(name: &str) -> io::Result<Option<InstanceListener>>` returns a platform-neutral listener type.
- The listener type must remain owned by tool apps for the process lifetime and be clonable when passed to the polling timer.

- [x] **Step 1: Write failing tests**

Add tests for the existing command contract without changing command text:

```rust
#[test]
fn quit_round_trip_is_decoded() {
    // claim one test instance, call terminate_instance(name), then accept_command()
    // and assert Some(InstanceCommand::Quit).
}

#[test]
fn raise_token_round_trip_is_decoded() {
    // claim one test instance, call raise_instance(name, Some("token")), then
    // assert Some(InstanceCommand::Raise(Some("token".into()))).
}
```

Use unique per-test names to avoid collisions. On Windows use a unique named-pipe suffix; on Linux retain the current abstract socket test strategy.

- [x] **Step 2: Run focused tests and verify the new Windows abstraction fails before implementation**

Run:

```bash
cargo test -p xtools-ui instance -- --nocapture
```

Expected: the new abstraction does not compile or the new platform test fails because `InstanceListener` and the Windows implementation do not yet exist.

- [x] **Step 3: Move the current Linux implementation unchanged into `instance/unix.rs`**

Keep `SocketAddr::from_abstract_name`, nonblocking listener behavior, command parsing, and error handling unchanged. Define the Unix listener alias and expose the same functions through the parent module.

- [x] **Step 4: Implement Windows named-pipe ownership and polling**

Implement `InstanceListener` around a Windows named-pipe server. `claim_instance()` creates the stable per-user pipe name and returns `Some(listener)` only when no server exists; an existing server returns `None`. `raise_instance()` and `terminate_instance()` connect and write exactly the existing command lines. `accept_command()` performs one nonblocking/polling accept/read operation and returns `None` when no command is ready.

If the current dependency set lacks a usable named-pipe API, add one narrowly scoped Windows-only dependency in `crates/xtools-ui/Cargo.toml`; do not change Linux dependencies.

- [x] **Step 5: Run the focused tests and verify they pass**

Run:

```bash
cargo test -p xtools-ui instance -- --nocapture
```

On Linux, this proves the unchanged Unix behavior. On Windows, run the same command natively and require both round trips to pass.

- [x] **Step 6: Commit the IPC boundary**

```bash
git add crates/xtools-ui/src/instance.rs crates/xtools-ui/src/instance crates/xtools-ui/src/lib.rs crates/xtools-ui/Cargo.toml
git commit -m "refactor: abstract tool singleton IPC by platform"
```

---

### Task 2: Make shared UI startup compile on Windows

**Files:**
- Modify: `crates/xtools-ui/src/boot.rs`
- Modify: `crates/xtools-ui/src/slint_chrome.rs`
- Modify: `crates/xtools-ui/src/lib.rs`
- Modify: `crates/xtools-time/src/main.rs`
- Modify: `crates/xtools-json/src/main.rs`
- Modify: `crates/xtools-trans/src/main.rs`

**Interfaces:**
- Keep `capture_target_desktop()`, `target_desktop()`, `take_activation_token()`, and `prefer_x11_for_skip_taskbar()` callable by current binaries.
- On Windows, Linux desktop and X11 environment operations become safe no-ops.
- `setup_raise_timer()` continues to process `RAISE` and `QUIT` using the platform-neutral listener.

- [x] **Step 1: Add platform tests for Windows-safe startup behavior**

Test the command-independent behavior: missing Linux environment variables must not prevent startup, and the platform no-op functions must return without panic. Keep tests conditional where the behavior is OS-specific.

- [x] **Step 2: Run the focused tests before implementation**

Run:

```bash
cargo test -p xtools-ui boot -- --nocapture
```

Expected: the Windows target currently fails to compile because of Linux-only imports and listener types.

- [x] **Step 3: Gate Linux-only boot logic**

Use `#[cfg(unix)]`/`#[cfg(windows)]` around KWin desktop capture, environment removal, and X11 preference. Preserve the public helper names so the three tool mains do not duplicate platform conditionals.

- [x] **Step 4: Update Slint chrome to use the platform-neutral listener type**

Change `setup_raise_timer()` and each tool app’s stored lock field from `UnixListener` to the shared `InstanceListener` type. Preserve the 50ms repeated polling and the existing `RAISE`/`QUIT` behavior.

- [x] **Step 5: Run Linux tests and checks**

Run:

```bash
cargo test -p xtools-ui
cargo check --workspace
```

Expected: PASS with no Linux behavior changes.

- [x] **Step 6: Commit the cross-platform startup boundary**

```bash
git add crates/xtools-ui crates/xtools-time/src/main.rs crates/xtools-json/src/main.rs crates/xtools-trans/src/main.rs
git commit -m "feat: make Slint tools platform-neutral at startup"
```

---

### Task 3: Remove Linux-only tool features from Windows builds

**Files:**
- Modify: `crates/xtools-ui/Cargo.toml`
- Modify: `crates/xtools-time/Cargo.toml`
- Modify: `crates/xtools-json/Cargo.toml`
- Modify: `crates/xtools-trans/Cargo.toml`
- Modify: `crates/xtools-time/src/app.rs`
- Modify: `crates/xtools-json/src/app.rs`
- Modify: `crates/xtools-trans/src/app.rs`

**Interfaces:**
- Linux continues to use X11 skip-taskbar and focus-loss timers.
- Windows builds use Slint’s normal window backend and retain raise/quit polling.
- Tool UI callbacks and business logic are unchanged.

- [x] **Step 1: Identify all feature-gated imports and fields**

Make each `x11-skip-taskbar` import, timer field, and timer construction conditional on the feature/platform. Do not remove the functionality from Linux.

- [x] **Step 2: Run Linux package tests before changes**

Run:

```bash
cargo test -p xtools-time -p xtools-json -p xtools-trans
```

Expected: current Linux tests pass, establishing the baseline.

- [x] **Step 3: Gate X11-only timer setup**

On Windows, store no X11 skip-taskbar or focus-loss timer and keep only the shared raise timer. Ensure the struct fields and constructor return remain valid in both configurations.

- [x] **Step 4: Build the three packages for a Windows target**

On a Windows host or Windows CI runner, run:

```bash
cargo build -p xtools-time -p xtools-json -p xtools-trans --release
```

Expected: all three packages compile without GTK, X11, KWin, Unix socket, or Linux environment code.

- [x] **Step 5: Run Linux regression checks**

Run:

```bash
cargo test --workspace
cargo build --workspace --release
```

- [x] **Step 6: Commit Windows tool build support**

```bash
git add crates/xtools-ui/Cargo.toml crates/xtools-time crates/xtools-json crates/xtools-trans
 git commit -m "feat: support Windows builds for Slint tools"
```

---

### Task 4: Verify Windows singleton and tool behavior

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: `README.md` only if the supported build command or platform list changes

**Interfaces:**
- CI must run Linux regression checks and Windows tool-package checks.
- Phase 1 does not claim Windows floating-ball support.

- [x] **Step 1: Add a Windows CI job**

Use a Windows runner with the stable Rust toolchain. Run:

```bash
cargo test -p xtools-ui
cargo build -p xtools-time -p xtools-json -p xtools-trans --release
```

Do not build `xtools-host` on Windows in Phase 1.

- [x] **Step 2: Verify singleton behavior natively on Windows**

For each tool, launch it twice and assert that the second launch exits after sending `RAISE`, while the first window remains the only visible instance. Close the first window and launch again; assert a new instance can claim the name.

- [x] **Step 3: Run the complete Phase 1 verification matrix**

Linux:

```bash
cargo test --workspace
cargo build --workspace --release
```

Windows:

```bash
cargo test -p xtools-ui
cargo build -p xtools-time -p xtools-json -p xtools-trans --release
```

- [x] **Step 4: Commit CI and documentation changes**

```bash
git add .github/workflows/ci.yml README.md
git commit -m "ci: verify Windows Slint tools"
```

## Self-Review

- Spec coverage: Phase 1 scope, platform IPC, shared Slint startup, Linux preservation, Windows package build, singleton behavior, and Phase 2 exclusion are covered by Tasks 1–4.
- Placeholder scan: no TODO/TBD implementation placeholders; Windows-native execution is an explicit environment requirement, not an unfilled implementation step.
- Type consistency: `InstanceListener` is the shared listener type consumed by `setup_raise_timer()` and all three tool app structs.
- Scope: `xtools-host` and Windows tray remain explicitly outside Phase 1.
