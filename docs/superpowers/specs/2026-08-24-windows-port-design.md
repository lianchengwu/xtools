# Windows Port Design

## Scope

Use a phased port:

- Phase 1: make `xtools-time`, `xtools-json`, and `xtools-trans` build and run on Windows.
- Phase 2: add the Windows floating-ball host and tray.
- Linux behavior remains unchanged throughout Phase 1.

## Phase 1 Architecture

Keep the Slint UI and tool business logic shared. Move Linux-only startup and window behavior behind platform modules.

### Single-instance IPC

Replace direct Linux abstract Unix socket usage with a platform-neutral instance API:

- Linux implementation: existing abstract Unix socket.
- Windows implementation: named pipe under a stable per-user name.
- Commands remain `RAISE`, `RAISE <activation-token>`, and `QUIT`.

The three binaries retain the existing singleton flow: attempt to claim, raise the existing process when claim fails, otherwise create the UI and poll commands.

### Window behavior

- Keep the shared Slint components and callbacks.
- Linux keeps X11 skip-taskbar and KWin behavior.
- Windows uses the Slint/winit window handle for activation and equivalent visibility behavior.
- Linux-only environment handling becomes a no-op or platform implementation on Windows.

### Build configuration

Use conditional dependencies/features so Windows does not compile GTK layer-shell, X11, KWin, or Linux socket code. Keep the workspace package layout and release profile.

## Phase 2 Boundary

Do not port `xtools-host` in Phase 1. The future Windows host will provide:

- borderless transparent always-on-top window;
- drag and click handling;
- screen/work-area boundary clamping;
- circular input hit region;
- Windows tray integration;
- launching and raising the three tool binaries.

Shared layout, animation, painting, and tool identifiers should be reused where practical; host window and tray implementations remain platform-specific.

## Verification

Phase 1 acceptance:

1. `cargo build --workspace --release --target <windows-target>` succeeds on a Windows-capable toolchain.
2. Each tool launches on Windows.
3. Launching the same tool twice leaves one visible process/window and raises the existing window.
4. Closing a tool releases its singleton claim.
5. Existing Linux focused tests and release build remain passing.

Phase 2 is explicitly out of scope for this phase.
