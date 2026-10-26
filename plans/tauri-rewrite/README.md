# Tauri rewrite

Opsis moves from Electron to Tauri. Rust owns the app: windows, the project, compile, runs, capture, the Local API and everything the worker does today. The editor stays React, running in the system webview. Remotion renders run in a Bun sidecar. One Chrome Headless Shell does capture, renders and the streamed views (preview, sign-in, element picker).

Status: phases 1 to 6 are done on branch `feat/tauri` (1 `b11b013`, 1.5 `6181b36`, 2 `2b457a8`, 3 `3e8562c`, 4 `bd4181e` and `8932925`, 5 `cfd70d5`, 6 the commit after it): the app is the Tauri app and Electron is gone. Measurements before phase 6 are from the tree at `df01ff0` (card stills already removed) on Linux x64, taken 2026-10-25.

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
- **Workspace (decided):** JS/TS apps in `apps/` (`ui`, the editor; `sidecar`), JS/TS packages in `packages/` (`video`, and `core` until the switch), and all Rust in `crates/`, as in `ctf-platform`: a Cargo workspace whose root `Cargo.toml` only lists `crates/*`, with `Cargo.lock` at the root. Bun workspaces are `apps/*` and `packages/*`; turbo runs both. There is no `shared` package (`workspace.md`).
- **Crates by domain (decided).** The Tauri app is one crate among several, each owning one domain (`workspace.md`, **The crates**). Nothing Rust lives in `apps/`; there is no `src-tauri`.
- **Rspack builds the video bundle (decided, phase 1.5).** Remotion's `bundle({ rspack: true })` with our `rspackOverride`; all 109 registry elements rendered byte-identically from webpack and Rspack bundles before the switch.
- **No interim app (decided).** The Electron app isn't kept working while Rust is built. The TS code stays in the tree as the reference the Rust ports are tested against, and is deleted when the editor switches to the Tauri app (phase 6). Nothing is built only to bridge phases: no TS worker run as a child of Rust, no stop-gap sign-in or picker windows, no Local API routes forwarded to TS.
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
│ apps/ui        │   │ apps/sidecar (Bun) │   │ Chrome Headless Shell          │
│ React editor   │   │ renderMedia /      ├──►│ capture · render tabs ·        │
│ store + undo   │   │ renderStill only   │   │ preview page · sign-in · picker│
└────────────────┘   └────────────────────┘   └────────────────────────────────┘
                      video bundle (packages/video, built by Rspack at build
                      time) is loaded by the shell for renders and the preview
FFmpeg / ffprobe: separate programs, started by Rust (and by Remotion's compositor)
```

## Files

| File                      | Part                                                                                        |
| ------------------------- | ------------------------------------------------------------------------------------------- |
| `sizes.md`                | What ships today and after, measured                                                        |
| `workspace.md`            | Apps, packages and crates, where every `src/shared` file goes, the build pipeline           |
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

Each phase is checked before the next starts. Line counts are from `code-split.md`. Phases 2 to 5 build crates next to the TS code they replace and test them against it; the app keeps running on Electron only because nothing in those phases touches it, not because it has to. Phase 6 switches the editor to the Tauri app and deletes what Rust replaced.

- **Phase 1: workspace split (done, `b11b013`).** Code moved into packages with no behaviour change; the remaining `shared` files split by their consumers (`workspace.md`).
- **Phase 1.5: apps, imports, installs, Rspack (done, `6181b36`).** The editor and the sidecar became apps; own-file imports through `#core/`, `#video/`, `#sidecar/`; bun's isolated installs; Rspack for the video bundle (`workspace.md`, **Phase 1.5 as built**).
- **Phase 2: types and compile** (`opsis-types`, `opsis-compile`). The types that cross a boundary with generated TS and JSON Schema; node definitions, settings, compile, cards, flattener, mix plan, catalogue and validation. About 8,700 lines of `packages/core`. Check: compile every example and template with both implementations and compare the JSON (`compile-in-rust.md`, `types-and-contracts.md`).
- **Phase 3: app state, projects, fonts, engine, traces** (`opsis-store`, `opsis-project`, `opsis-fonts`, `opsis-engine`, `opsis-traces`). The database, settings, view state, keys and sealed sign-ins, project files and watching, asset and model import, the font library, the engine installer and paths, the trace store. Check: the ported `src/main`, `src/engine` and `src/traces` tests (`desktop-shell.md` steps 7–9, 11, 12; `runner-and-executors.md` step 7).
- **Phase 4: runs, media and renders** (`opsis-runner`, `opsis-media`, `opsis-render`, `apps/sidecar` on Bun). The runner, cache keys and run events; the media and mix executors; the sidecar protocol, the asset server, the picture mux and export as code. About 7,300 lines. Check: FFmpeg argument strings equal the TS ones; renders through the sidecar byte-identical to the Node renders (`runner-and-executors.md`, `sidecar.md`).
- **Phase 5: capture and streamed views** (`opsis-capture`, `opsis-stream`, `apps/preview`; done, `cfd70d5`). CDP client, sessions, selectors, actions, screencast encoder, sign-in steps; the stream transport and the view for preview, sign-in, picker and live frames. Check: the TodoMVC take's action log and frame count against a take from before the move (`capture-cdp.md`, `streamed-views.md`).
- **Phase 6: the switch** (`opsis-desktop`, `opsis-api`; done, `desktop-shell.md` **Phase 6 as built**). The Tauri app: windows, title bar, menus, IPC commands and channels, protocols, exit; the Local API in Rust; the editor on the `window.opsis` facade, the Inspector on JSON Schema, the preview, sign-in and picker as streamed views, pop-out windows over Rust events. Electron, Forge, `src/`, `packages/core` and every TS duplicate of a Rust port are deleted, and `playwright-core` becomes a dev dependency (only `playwright:scripts` reads it). Check: the examples open, run and export on Linux with the same output as before (`desktop-shell.md`, `local-api.md`).
- **Phase 7: asset preparation and image nodes** (`opsis-prepare`, `opsis-image`; `asset-preparation.md`, `image-nodes.md`).
- **Phase 8: packaging** (`packaging.md`), then `AGENTS.md` (`agents-md.md`). Until here there is no packaged build: Forge can't package the Tauri app, so phases 6 and 7 are checked in development.

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
- Phases 2 to 5: the crates' own tests, and the parity check named in the phase against the TS code still in the tree.
- From phase 6: the examples in `examples/` open, run and export on Linux, with the same output as before where output is meant to be unchanged (compare files byte for byte, as in the sidecar test in `sidecar.md`). The 16:9 `changelog` export differs between two runs of the same code in its last 75 frames (the bar chart), so compare its 1:1 export, or fix that first.
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
