# AGENTS.md after the move

What `AGENTS.md` loses, keeps and gains when each phase ships. Edit `AGENTS.md` in the phase that changes the rule, not before; the code is the authority until then.

## What it adds

- A checklist so no Electron-era rule survives the move by accident, and every new rule (Rust types, the sidecar's stdout, streamed views, prepared files) is written down when it starts to hold.

## Work

### Hard rules

| Rule                           | Change                                                                                                                                                                                                                                                                                       | Phase |
| ------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----- |
| No comments                    | Extend to Rust: no `//`, `///`, `//!`, `/* */`; attributes are allowed. Add `--glob '*.rs'` to the check command                                                                                                                                                                             | 2     |
| File names are kebab-case      | TS unchanged; Rust module files are snake_case (a module name must be an identifier)                                                                                                                                                                                                         | 2     |
| Never hand-edit generated code | Add the generated TS bindings (`types-and-contracts.md`) and the element metadata the ingest writes for Rust; move paths to `packages/*`; drop `public/three/draco/**` and `three:decoders` (`asset-preparation.md`); drop `src/main/db/schema.generated.sql`'s Node import (Rust embeds it) | 1–6   |
| Dependencies                   | Add `cargo add` (latest release); keep the Remotion pin; the drizzle pin's reason (`node:sqlite` driver) goes, only `drizzle-kit` stays for `db:schema`; the TypeScript pin stays while Remotion's bundler needs it                                                                          | 2, 4  |
| Tooling                        | "Nothing ever runs under Bun's runtime" becomes "only the sidecar runs on Bun"; Electron and Forge lines go                                                                                                                                                                                  | 4, 7  |
| Verify with                    | Add `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`                                                                                                                                                                                                                   | 2     |

### Layout

Replace the `src/*` tree with `packages/ui`, `packages/video`, `packages/sidecar`, the `app` crate and `scripts/` (`workspace.md`). Phase 1.

### Invariants that go or change

| Section                                              | Change                                                                                                                                                                        |
| ---------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Runtime boundaries                                   | Package dependencies replace the oxlint import bans for `shared`; the video rules stay in `packages/video`                                                                    |
| Nodes never generate Remotion code                   | Same rule; node definitions and compile are Rust, executors are Rust except renders                                                                                           |
| Params carry their own inspector UI                  | zod `.meta()` becomes `schemars` `x-` keywords; `FIELD_WIDGETS` checked by a Rust test                                                                                        |
| Node cards                                           | `card()` is Rust; `NODE_ICON_NAMES` is a Rust enum, `NODE_ICONS` still maps every name in the UI                                                                              |
| Every value a user would want to change              | The ingest writes element metadata for Rust; its rules are unchanged                                                                                                          |
| Editor panels                                        | The Preview tab is a streamed view; "Player runs with `controls={false}`" and `player.play(e)` go                                                                             |
| View state, starting and closing                     | Same behaviour, implemented in Rust; the 2 s and 3 s exit rules stay                                                                                                          |
| App state is one database                            | `rusqlite` instead of `node:sqlite`/drizzle queries; `drizzle-kit export` still writes the SQL; `encryptedText()` and `safeStorage` become `keyring` (+ AES-GCM for sign-ins) |
| Settings is its own window, Windows share one editor | `BroadcastChannel` mirroring (`sync.ts`) becomes Rust events; window roles become the Tauri window API                                                                        |
| Local API                                            | Served by Rust; the Windows pipe is created for the current user only                                                                                                         |
| Run state                                            | Rust owns progress; the UI renders events and progress it receives                                                                                                            |
| Capture timing, Drag action, Screenshots             | Same rules, in Rust over CDP; "Never fall back to Playwright's own browsers" becomes "no Playwright"                                                                          |
| Interactive browsing happens in Electron windows     | Becomes: sign-in and the picker are streamed from the headless shell (`streamed-views.md`)                                                                                    |
| Traces                                               | Our own trace format; the "Playwright's Trace Viewer" rule goes                                                                                                               |
| One engine, two editions                             | Playwright gone; the installer is Rust; `TESTED_VERSION` still comes from the installed `@remotion/renderer`                                                                  |
| Fonts                                                | `fontkit` → `skrifa`/`read-fonts`; `queryLocalFonts` → `fontdb`; 3D text reads prepared typefaces                                                                             |
| 3D elements                                          | Models and typefaces are prepared files (`asset-preparation.md`); `PREPARED_VERSION` joins `PICTURE_VERSION` and `MIX_VERSION`                                                |
| Mixer                                                | Stem levels streamed in Rust; loudness from `ebur128`; the no-stem fallback goes once every film plays its stem                                                               |
| Packaging                                            | Forge, fuses, `packageAfterCopy`, `binariesDirectory` in `app.asar.unpacked`, Squirrel and the AppImage launcher go; Tauri bundler rules from `packaging.md` come in          |
| Canvas transitions preview as they export            | The editor-window switch goes (no video in the webview); the rule for renders stays                                                                                           |
| One title bar, Renderer hosting, Linux dev sandbox   | Rewritten for Tauri (`desktop-shell.md`); the `opsis-app` protocol and the dev `--no-sandbox` wrapper go                                                                      |
| Worker temp folders                                  | Rust gives each child program its temp folder                                                                                                                                 |

### New invariants

- **Rust is the source of types** that cross a boundary; TS is generated and never edited by hand; JSON Schema for forms and the API comes from the same types.
- **The sidecar renders only**, gets specs as JSON, and keeps stdout for messages: `console.*` and `process.stdout.write` go to stderr.
- **Streamed views**: frames go to the webview as raw channel payloads; the page in the shell is the clock for preview sound.
- **Prepared files** are keyed by `PREPARED_VERSION`; bump it whenever a preparer's output changes.

## Sources

- `AGENTS.md` at `df01ff0` (read 2026-10-25).
