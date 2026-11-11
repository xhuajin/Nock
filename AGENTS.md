# Repository Guidelines

## Project Structure & Module Organization

Nock is a native Rust desktop app for Windows. `win_main.rs` contains the entry point and window procedure; `app.rs`, `ui.rs`, and `sys.rs` hold the panel state, drawing, and Windows integration. Shared modules include `gfx.rs`, `cal.rs`, `store.rs`, and `i18n.rs`. `assets/` holds app icons, `docs/` holds the website and demos, and `installer/` and `winget/` provide distribution files.

## Build, Test, and Development Commands

- `cargo build --release` builds `target/release/nock.exe`; `build.bat` runs the same Windows build.
- `cargo test` runs unit tests and validates scene scripts, window message IDs, and notes persistence.
- `cargo clippy` checks Rust lint warnings.
- `ISCC.exe /DMyAppVersion=1.2.3 installer\nock.iss` builds the Windows installer after a release build.

## Coding Style & Naming Conventions

Use standard Rust formatting and run `cargo fmt`; keep indentation and layout rustfmt-compatible. Follow Rust naming (`snake_case` functions/modules, `CamelCase` types, `SCREAMING_SNAKE_CASE` constants). Keep platform-neutral text and file helpers in standard Rust modules. Do not introduce polling loops, long-lived background threads, or heavy dependencies; preserve event-driven behavior and near-zero idle CPU. Windows UI must remain native (Win32/Direct2D/WinRT), with no webview.

## Testing Guidelines

Tests are colocated in `#[cfg(test)]` modules with descriptive `snake_case` names. Run all tests with `cargo test` and lint with `cargo clippy`. For visual verification, use ignored preview tests:

```text
set PREVIEW_DIR=C:\temp\nock-preview
cargo test preview::render_pages -- --ignored
```


## Commit & Pull Request Guidelines

Recent history favors imperative, sentence-style summaries such as `Add Chinese UI language with tray switcher`; use a short title and explain the why in the body for larger changes. Keep pull requests small and focused. Describe behavior changes, list verification commands, link related issues, and include before/after screenshots for UI updates. Confirm the installer or package smoke test when changing distribution files.

## Local Release Build and Deployment

After changing the application, run these steps from the repository root:

1. Build the release executable:

   ```powershell
   cargo build --release
   ```

2. Stop any running `nock.exe` process before replacing the installed file:

   ```powershell
   Get-Process -Name nock -ErrorAction SilentlyContinue | Stop-Process -Force
   ```

3. Replace the installed executable with the release build:

   ```powershell
   Copy-Item -LiteralPath 'target\release\nock.exe' -Destination 'C:\D\Software\Nock\nock.exe' -Force
   ```

4. Start the updated application:

   ```powershell
   Start-Process -FilePath 'C:\D\Software\Nock\nock.exe' -WorkingDirectory 'C:\D\Software\Nock'
   ```

Verify that the process is running from `C:\D\Software\Nock\nock.exe` and that the installed file matches `target\release\nock.exe`. Writing to the install directory may require elevated permissions.
## Agent-Specific Instructions

Read `CONTRIBUTING.md` before changing behavior. Keep `WM_APP + n` message numbers unique and add only original artwork. Bug reports should include the Windows version and logs from `%APPDATA%\Nock` when available.