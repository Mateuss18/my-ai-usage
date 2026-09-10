@C:\Users\mateu\.codex\RTK.md

# Project scope

- This is a personal Windows-only MVP for the repository owner.
- Optimize for correct behavior on the owner's PC, not for thousands of users, enterprise scale, or cross-platform support.
- Prefer the shortest native Windows/Tauri solution that works; reuse existing dependencies and avoid speculative abstractions.
- Do not add scalability, portability, extensibility, telemetry, or production-hardening work unless an issue explicitly requires it.
- Keep essential correctness, safe cleanup, and focused regression checks; MVP does not mean knowingly shipping a broken requested flow.
- Never leave unexplained vertical or horizontal empty space in a UI: size windows and panels to the largest visible state, keep both overflow axes closed, and keep Tauri native height synchronized between `src-tauri/src/lib.rs` and `src-tauri/tauri.conf.json`.
- Never open a console window with the app: keep `#![windows_subsystem = "windows"]` unconditional in `src-tauri/src/main.rs` and use `CREATE_NO_WINDOW` for child CLI processes; a terminal explicitly used to run development commands is separate from the app.
