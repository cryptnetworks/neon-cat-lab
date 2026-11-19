# Neon Cat process hunt

Neon Cat is a Rust classroom exercise. A cat follows the real pointer between connected displays and draws a neon burst when it catches up. When the pointer changes displays, the cat appears at the nearest edge of the new display and resumes the chase. The overlay passes clicks through to the desktop. The program runs under the signed-in student's account.

## Instructor setup

Install stable Rust and Cargo. Edit [neon-cat.yaml](neon-cat.yaml) before building on each target operating system and architecture.

| Setting | Default | Meaning |
| --- | --- | --- |
| `process_name` | `neon-cat-lab` | Executable filename without an extension. Use letters, digits, hyphens, or underscores. |
| `session_seconds` | `300` | Time before the exercise quits automatically. |
| `quit_shortcut` | `Ctrl+Alt+Shift+K` | Instructor shortcut. On macOS, `Alt` means Option. |
| `cat_speed` | `800.0` | Maximum flight speed in screen points per second. |

```sh
cargo run --locked --release --bin build-lab
```

Launch the executable printed by the build command from the student's graphical session. With the default configuration, this is `target/lab/neon-cat-lab.exe` on Windows and `target/lab/neon-cat-lab` on Linux and macOS. The cat is the only content drawn on the desktop. The overlay skips the Windows taskbar and macOS Dock. Rebuild after changing the YAML. The executable contains the settings and can run on its own.

Windows MSVC builds require the Visual Studio C++ build tools and an OpenGL driver. Linux requires a composited X11 session with OpenGL and the X Shape extension. Native Wayland is unsupported. macOS builds require the Xcode Command Line Tools.

## Student prompt

> Something unexpected is moving across your desktop. Use Resource Monitor on Windows or `ps aux` on Linux to identify the process responsible. Record the process name and PID, stop only that process, then verify that the desktop activity ended and the PID is gone.

## Instructor answer key

- **Windows**. Run `resmon`, open the **CPU** tab, and look for the configured executable name with `.exe` in **Processes**. Record the PID, right-click that row, and choose **End Process**.
- **Linux**. Find the row whose command ends in the configured executable name in the `ps aux` output. Confirm the owner and PID, send `kill -TERM <PID>`, and confirm that the PID is absent with `ps -p <PID>`.

The learner has finished when the cat disappears, the matching PID is gone, and they can explain how they verified the process before ending it.
