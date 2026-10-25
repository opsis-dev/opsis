# Tauri rewrite

Opsis moves from Electron to Tauri. Rust owns the app: windows, the project, compile, runs, capture, the Local API and everything the worker does today. The editor stays React, running in the system webview. Remotion renders run in a Bun sidecar. One Chrome Headless Shell does capture, renders and the streamed views (preview, sign-in, element picker).

Status: plan only. Nothing here is built yet. Measurements are from the tree at `df01ff0` (card stills already removed) on Linux x64, taken 2026-10-25.

## How to read this plan

- This file holds the decisions, the shape, the phases and the conventions. Each other file is one part of the move, with the same sections as the other plans: **What it adds**, **Fit with Opsis** (Already in place / Missing), **Work**, **Decisions for the user** where a choice is open, **Risks**, **Sources**.
- `code-split.md` has the line counts: what is converted to Rust, what stays as it is, what changes, and what is duplicated. Effort is given in lines, not days.
- External facts carry the date they were checked. Things that can only be known by building (frame rates, binary sizes) are Work steps that measure them, never assumptions.

## Decisions so far

- **Tauri, not Electron (decided).** The system webview draws the editor: WebView2 on Windows, WKWebView on macOS, WebKitGTK on Linux.
- **No editor-side video (decided).** The Remotion Player leaves the webview. The preview, sign-in and the element picker are streamed from the headless shell (`streamed-views.md`). Card stills are already gone (`df01ff0`).
- **Rust owns compile (decided).** Node definitions, compile, cards, validation, the catalogue and the timeline flattener move to Rust. The UI asks for compile over IPC; there is no compile code in the webview and no WebAssembly build (`compile-in-rust.md`).
- **The sidecar only renders (decided).** Bun runs `@remotion/renderer` and nothing else; it gets a spec as JSON and imports nothing from the other packages (`sidecar.md`). Bun was tested: renders are byte-identical to Node's.
- **Rust and the sidecar talk over stdin/stdout (decided)**, one JSON message per line, with stdout reserved for messages (tested).
- **Capture over CDP in Rust, no Playwright (decided)** (`capture-cdp.md`).
- **One ONNX Runtime (decided).** All on-device models run through the `ort` crate with one ONNX Runtime build per platform. No sherpa-onnx (its static libraries carry their own ONNX Runtime) and no whisper.cpp (a second inference engine) (`plans/voice-and-captions.md`, **Rust**).
- **Workspace (decided):** a Bun-workspace turborepo with `ui`, `video`, `sidecar` and the Rust crate. There is no `shared` package (`workspace.md`).
- **Rust is the source of types (decided).** TS types are generated from Rust; JSON Schema for forms and the API comes from Rust (`types-and-contracts.md`).
- **Linux ships deb and rpm only (decided).** No AppImage; anyone can build one from source (`packaging.md`).
- **FFmpeg is unchanged (decided).** Same two ffmpeg-static programs, run the same way (see **Not in this plan**).

## Shape

```
┌──────────────────────── Tauri app (Rust, one process) ────────────────────────┐
│ windows · menus · protocols · app database · keys · file watching · exit       │
│ node definitions · compile · cards · flattener · catalogue · validation        │
│ runner (tokio) · media and mix executors · traces · asset preparation          │
│ capture over CDP · streamed views · Local API (socket / named pipe)            │
└───────┬──────────────────────┬──────────────────────────┬─────────────────────┘
        │ invoke / events /     │ stdin/stdout JSON lines  │ CDP (pipe)
        │ channels              │                          │
┌───────▼────────┐   ┌──────────▼─────────┐   ┌────────────▼───────────────────┐
│ ui (webview)   │   │ sidecar (Bun)      │   │ Chrome Headless Shell          │
│ React editor   │   │ renderMedia /      ├──►│ capture · render tabs ·        │
│ store + undo   │   │ renderStill only   │   │ preview page · sign-in · picker│
└────────────────┘   └────────────────────┘   └────────────────────────────────┘
                      video bundle (Remotion, built at build time) is loaded by
                      the shell for renders and for the streamed preview
FFmpeg / ffprobe: separate programs, started by Rust (and by Remotion's compositor)
```

## Files

| File                      | Part                                                                                        |
| ------------------------- | ------------------------------------------------------------------------------------------- |
| `sizes.md`                | What ships today and after, measured                                                        |
| `workspace.md`            | The turborepo packages, where every `src/shared` file goes, the build pipeline              |
| `code-split.md`           | Line counts: converted, deleted, unchanged, changed, duplicated                             |
| `types-and-contracts.md`  | Rust as the type source, generated TS and JSON Schema, the small logic kept on both sides   |
| `compile-in-rust.md`      | What compile does, where it runs today, the Rust version, the Inspector from JSON Schema    |
| `desktop-shell.md`        | Tauri windows, title bar, menus, IPC, protocols, database, keys, file watching, exit, fonts |
| `runner-and-executors.md` | The tokio runner, cache keys, the 20 executors, media and mix, traces, run events           |
| `sidecar.md`              | Bun and Remotion, the tested stdio protocol, building and shipping the sidecar              |
| `capture-cdp.md`          | Capture without Playwright: CDP client, selectors, screencast, sign-in steps, traces        |
| `streamed-views.md`       | Preview, sign-in and element picker streamed from the headless shell                        |
| `local-api.md`            | The Local API in Rust                                                                       |
| `asset-preparation.md`    | 3D text, models, textures, SVG logos and font inspection prepared in Rust                   |
| `image-nodes.md`          | New image nodes on image-rs                                                                 |
| `packaging.md`            | Tauri bundles, deb and rpm, Windows and macOS, signing, updates, sizes                      |
| `agents-md.md`            | What `AGENTS.md` loses, keeps and gains when this ships                                     |

The existing plans each gained a **Rust** section saying how they are built after the move: `plans/voice-and-captions.md`, `plans/ai-assist.md`, `plans/motion-library.md`, `plans/shipping.md`.

## Phases

Each phase is checked before the next starts. Line counts are from `code-split.md`.

1. **Workspace split, still Electron.** Move code into `ui`, `video`, `sidecar` with no behaviour change; the remaining `shared` files are split by their consumers (`workspace.md`). Check: the app runs and every test passes.
2. **Tauri shell and the main process in Rust.** Windows, IPC facade, app database, settings, keys, projects, file watching, exit, fonts, Local API transport (`desktop-shell.md`, `local-api.md`). About 2,800 lines of `src/main` become Rust. During this phase the old TS worker keeps running as a child process so runs still work; it is replaced in phase 4.
3. **Compile in Rust.** Node definitions, compile, cards, flattener, catalogue, validation; the Inspector reads JSON Schema; TS types are generated (`compile-in-rust.md`, `types-and-contracts.md`). About 8,700 lines of `src/shared`.
4. **Runner, executors and the sidecar.** The tokio runner, media, mix, traces; renders through the Bun sidecar (`runner-and-executors.md`, `sidecar.md`). About 7,300 lines.
5. **Capture over CDP and streamed views.** Capture, sign-in, picker, preview (`capture-cdp.md`, `streamed-views.md`). Capture is about 1,800 converted lines plus what Playwright did for us.
6. **Asset preparation and image nodes** (`asset-preparation.md`, `image-nodes.md`).
7. **Packaging** (`packaging.md`), then `AGENTS.md` (`agents-md.md`).

## Conventions

These restate rules from `AGENTS.md`; when the two disagree, `AGENTS.md` wins until `agents-md.md` is applied.

- **No comments of any kind**, in TS and in Rust: no `//`, `/* */`, `///` or `//!`. Attributes (`#[allow(…)]`, `#[cfg(…)]`) are not comments and are allowed. Constraints that span the codebase go in `AGENTS.md`.
- **File names.** TS files stay kebab-case. Rust module files are snake_case, because a module name must be a Rust identifier; `agents-md.md` adds this exception.
- **Dependencies through the package managers only:** `bun add` / `bun add -d` for TS, `cargo add` for Rust, latest releases unless a pin is written down with its reason. Remotion packages keep their exact shared pin.
- **Rust checks:** `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`. TS checks stay `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`.
- **Nothing uses the internet unless the user asks**, unchanged. Build steps that download (ONNX Runtime through `ort`, crates) happen at build time only, never in the app.
- **User-facing copy** never names Tauri, Rust, Bun, CDP or any crate.
- **Large files** go to `~/.cache/opsis-*`. **Don't commit or push** unless asked.

## Checks when a phase is done

- Rust and TS checks above, plus the no-comments check from `AGENTS.md` extended to `*.rs` (`rg -n '^\s*//|/\*|\*/' --glob '*.rs'`).
- The examples in `examples/` open, run and export on Linux, with the same output as before the phase where output is meant to be unchanged (compare files byte for byte, as in the sidecar test in `sidecar.md`).
- Don't commit or push.

## Not in this plan

- **FFmpeg.** Unchanged. Findings from the discussion, kept for a later plan:
  - The pinned ffmpeg-static `b6.1.1` mixes versions: its download script pins 6.1.1 for Windows (gyan.dev) and macOS (evermeet.cx, osxexperts.net) but takes John Van Sickle's latest release on Linux, which is why the Linux binary reports `7.0.2-static`. John Van Sickle doesn't publish his build scripts.
  - Each of `ffmpeg` (79.8 MB) and `ffprobe` (79.7 MB) statically links the same libav libraries plus about 30 outside libraries; the app's code asks only for x264, x265, libvpx (VP9), opus and lame, plus FFmpeg's built-in codecs and filters (and dav1d for AV1 input).
  - One binary for both is possible: a shared build (`--enable-shared`), or a multicall binary. FFmpegKitNext (FFmpeg 9.0.2, last commit 2026-10-24) already builds both tools into one library with `ffmpeg_execute` and `ffprobe_execute`, with GitHub workflows for Linux, Windows and macOS. BtbN/FFmpeg-Builds builds Linux and Windows in Docker on GitHub Actions; markus-perl/ffmpeg-build-script covers macOS.
- **Replacing Remotion.** Remotion stays, inside the sidecar. Its licence (a company licence above three employees) and its pins remain until a later plan replaces the renderer.
- **A native UI (GPUI).** Not part of this move.

## Sources

- Measurements on this machine, 2026-10-25: sizes (`sizes.md`), the Bun render test and the stdio test (`sidecar.md`), compile timing and result sizes (`compile-in-rust.md`), import-graph line counts (`code-split.md`).
- ffmpeg-static download script (`download-binaries/index.sh` in `eugeneware/ffmpeg-static`), `ffmpeg -version` of the installed binary, FFmpegKitNext and BtbN sources, cloned into `~/.cache/opsis-research` (2026-10-25).
