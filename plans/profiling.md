# Profiling

What Opsis costs in size, build time, CPU and memory, measured with development tools only. No code goes into the app: everything runs from outside it, as build settings or as a dev script.

## How to read this plan

- Each section has the same parts where they apply: **In place**, **Work** (numbered, only what is left), **Decision for the user**, **Risks**. Rules from `AGENTS.md` hold throughout; when the two disagree, `AGENTS.md` wins.
- Every measurement carries its date; a newer one replaces the older.
- Measured on Linux x64 (20 cores, 14 GiB of memory).

## 1. The profiling build

**In place.**

- Release builds use `opt-level = 3`, full LTO and one codegen unit, and are stripped (`[profile.release]` in `Cargo.toml`).
- `tauri build` turns on the `tauri/custom-protocol` feature, which embeds `apps/frontend/dist` in the binary; a plain `cargo build` leaves it off and the editor's files out.

**Work.**

1. **One build for every tool.** The shipped profile with its symbols kept, made once and read by `cargo bloat`, samply, perf and heaptrack, with the compile timings written alongside:

   ```sh
   bun run --cwd apps/frontend build
   CARGO_BUILD_JOBS=8 CARGO_PROFILE_RELEASE_STRIP=false CARGO_PROFILE_RELEASE_DEBUG=true \
   cargo build --release -p opsis-desktop --features tauri/custom-protocol --timings
   ```

   `cargo bloat` takes the same settings, `-p` and `--features`, so it uses this build instead of making another. Stripping removes only names, so the shipped size is this binary stripped.

**Decision for the user.** Where the build goes: `~/.cache/opsis-build` (shared with `bun run make`, which then rebuilds crates whenever the two switch settings), its own `~/.cache/opsis-profile`, or `./target`.

**Risks.** Full LTO with debug info is the slowest link and the largest in memory, never measured on this machine. If it runs out of memory, `CARGO_PROFILE_RELEASE_DEBUG=line-tables-only` keeps function names and lines for the profilers with far less debug info; `cargo bloat` needs only the symbols.

## 2. Size

**In place.**

- **Tools.** `cargo-bloat` and `cargo-machete` are installed and `source-map-explorer` is a dev dependency (`a7f7e13`); bloat and source-map-explorer ran on 2026-10-26.
- **The binary by crate** (`cargo bloat --release -p opsis-desktop --crates`, stripping off for the measurement, 2026-10-26): 72.4 MiB unstripped, 45.8 MiB of it code. The largest are std 4.3 MiB, unattributed 3.9 MiB, opsis_compile 2.9 MiB, serde_core 2.8 MiB, jsonschema 1.9 MiB, tokio, rav1e (the AV1 encoder, used only by Image format's AVIF) and tauri at 1.5 MiB each, aws_lc_sys 1.3 MiB, and zbus 1.1 MiB. `exr` (251 KiB) and `image_webp` (218 KiB) come with the `image` crate's default formats.
- **The editor** (2026-10-26, 53 chunks, 3.58 MB of mapped JavaScript): elkjs 1.43 MB (40%, the engine behind Tidy up layout), the editor's own code 0.39 MB, `@base-ui/react` 0.22 MB.

**Work.**

1. **Measure again.** The binary by crate and the editor by package predate Konva and react-konva in the editor, nnnoiseless (with its trained weights), fundsp, ab_glyph and wuff in the binary, and the renderer; the film page has never been measured by package.
2. **The binary.** `cargo bloat --release -p opsis-desktop --features tauri/custom-protocol --crates -n 40` on the build of section 1, then the largest crates function by function with `--filter <crate>`.
3. **Generic code copied many times.** `cargo llvm-lines` (not installed: `cargo install cargo-llvm-lines`), one crate at a time on the dev profile, starting with `opsis-compile`; it explains code size and compile time together, and it is the one tool here that makes its own build.
4. **Crates compiled twice and crates never used.** `cargo tree -d --workspace` lists every crate built in more than one version; `cargo machete` lists declared dependencies nothing uses.
5. **The film page by package.** `bun run film:bundle` (without `--mode release`, which drops the source maps), then `bunx source-map-explorer film-bundle/assets/*.js`.
6. **The editor by package.** Its build has no source maps; `bunx vite build --sourcemap` in `apps/frontend` adds them for one build, then `bunx source-map-explorer apps/frontend/dist/assets/*.js`.

**Decision for the user.** Unused TS files, exports and dependencies: `knip` finds them across the Bun workspace, but needs a config naming the entry points (the two apps, the packages' `exports`, `scripts`, the tool configs) before its results mean anything. Add it as a dev dependency with that config, run it once with `bunx`, or skip it.

## 3. Build and test time

**In place.** `cargo build --timings` and `cargo-nextest` (installed).

**Work.**

1. **Compile time by crate.** `--timings` on the build of section 1 and on a debug build writes an HTML report of each crate's compile time and how many build at once.
2. **Rust tests.** `cargo nextest run --workspace` times each test (it skips doc tests).
3. **Every check.** `bunx turbo run test --summarize` times each turbo task.

## 4. CPU

**Work.**

1. **Per-component CPU and memory.** A dev script starts the app off-screen the way the e2e suite does (`xvfb-run`, Wayland unset, a seeded data folder with the API on), opens an example and runs it over the Local API, and samples every process in the app's tree once a second from `/proc`, tagged by command line: the app, WebKit's web and network processes, each headless shell (recording, streamed views, render tabs), FFmpeg. It writes a CSV and a chart. Node timings come from the run events the API streams.
2. **Where Rust spends time.** `samply record -p <pid>` (not installed: `cargo install --locked samply`) or `perf record -g` (installed; `kernel.perf_event_paranoid` may need lowering) on the app, on the build of section 1.
3. **The editor UI.** WebKitGTK's remote inspector through `WEBKIT_INSPECTOR_HTTP_SERVER=127.0.0.1:9222` (timelines, memory, layout); dev builds also open the inspector from the View menu.

**Decision for the user.** Time per operation. The crates log through `log` and `env_logger`, so there are no spans to time; seeing each compile, run step or save on a timeline needs `tracing` with `tracing-chrome` (opened in Perfetto) behind a dev-only Cargo feature, like `e2e`. That is a code change; take it only if samply's flame graphs aren't enough.

## 5. Memory

**Work.**

1. **The app.** `heaptrack` (not installed: `apt install heaptrack heaptrack-gui`) on the binary of section 1: every allocation with its stack, as flame graphs, with no code change.
2. **Peak memory of one program.** `/usr/bin/time -v` on the binaries that run alone (`render-elements`, `editor-data`).
