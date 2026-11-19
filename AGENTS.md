# Repository instructions

## Project overview

Neon Cat Lab is a standalone Rust 2024 desktop exercise for Windows, Linux on composited X11, and macOS. It uses eframe for the transparent overlay and global-hotkey for the instructor's exit shortcut. `src/pointer.rs` reads pointer position, `src/display.rs` selects and positions the overlay, `src/flight.rs` owns movement, and `src/art.rs` draws the cat and burst. `build.rs` validates `neon-cat.yaml` and embeds its settings. The build helper produces the configured executable name in `target/lab`.

## Required commands

| Task | Command |
| --- | --- |
| Install dependencies | `cargo fetch --locked` |
| Format | `cargo fmt` |
| Lint | `cargo clippy --locked --all-targets -- -D warnings` |
| Typecheck | `cargo check --locked --all-targets` |
| Unit tests | `cargo test --locked` |
| Integration test | Run the desktop smoke test below |
| Full verification | `cargo fmt --check && cargo clippy --locked --all-targets -- -D warnings && cargo check --locked --all-targets && cargo test --locked && cargo run --locked --release --bin build-lab`, followed by the desktop smoke test |
| Production build | `cargo run --locked --release --bin build-lab` |
| Database migration | Not applicable |

## Repository conventions

Use `main` as the integration branch and `codex/` for agent-created work branches. The repository has no configured issue labels, milestones, database, external API, or CI service. If CI is introduced, run the automated verification commands on Windows, Linux, and macOS. For a local release, build from the intended revision on each target operating system and architecture and smoke-test that executable before use.

## Definition of done

- All automated commands in the full verification row pass.
- Run the release executable on every desktop platform affected by a change to windowing or input.
- Move the mouse over another app. Verify that the cat follows and pausing produces a burst at the pointer. Move to a second display and back. Verify the handoff in both directions, including mixed scaling when available.
- Click a control underneath the cat and verify that the underlying app receives the click and keeps focus.
- Verify that the overlay has no Windows taskbar button or macOS Dock icon.
- Verify that the shortcut in the instructor guide closes the overlay. Relaunch, identify the lab process in Resource Monitor or `ps aux`, end that PID, and verify that the overlay and the process both disappear.
- Relaunch and verify that the program exits at the configured session limit.
