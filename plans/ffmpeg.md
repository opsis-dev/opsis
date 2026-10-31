# FFmpeg

Ship FFmpeg inside the app as a Tauri sidecar instead of downloading it at setup, then replace the ffmpeg-static program with Opsis's own build. The media code doesn't change: it keeps starting the `ffmpeg` program with the same arguments, so every argument builder, filter graph and log reader stays as it is. Building FFmpeg into the app through its C API was looked at and is not planned (section 4).

## How to read this plan

- Sections 1 and 2 ship together: the sidecar and its licence and source notice. Section 3, Opsis's own build, comes after and swaps the program without touching the app. Section 4 records the option not chosen.
- Each section has the same parts where they apply: **In place**, **Work** (numbered, only what is left), **Decided**.
- Each piece of work ends with the usual checks: `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, and the tests of the crates that start FFmpeg (`cargo test -p opsis-media -p opsis-footage -p opsis-render -p opsis-capture -- --ignored`).
- This plan changes `AGENTS.md` on purpose (One engine, Packaging, Licences, Cache keys, End-to-end tests, Tooling); every such change updates `AGENTS.md`, `README.md`, `THIRD_PARTY_NOTICES.md` and `plans/release.md` in the same commit.

## Summary

| Topic                         | What users get                                                     | Left to do                                                      | Status      |
| ----------------------------- | ------------------------------------------------------------------ | --------------------------------------------------------------- | ----------- |
| 1. Sidecar with ffmpeg-static | Exports work without setting up FFmpeg; only the browser downloads | staging, config, lookup, removing the FFmpeg half of the engine | not started |
| 2. Licence and source notice  | FFmpeg's licence and where its source is                           | the licence in resources, the source notice                     | not started |
| 3. Own build                  | One FFmpeg version on every platform, only what Opsis uses         | pinned sources, the build binary, CI builds, the swap           | not started |
| 4. FFmpeg inside the app      | Nothing                                                            | not planned                                                     | not chosen  |

## 1. Sidecar with ffmpeg-static

**In place.**

- **One path for every FFmpeg run.** `Ffmpeg::new(&EngineFiles)` (`crates/media/src/ffmpeg.rs`) runs and pipes for the media executors, mix stems, analysis, the render encoder (`crates/render/src/frames.rs`) and the take encoder (`crates/capture/src/encoder.rs`). The footage decoder starts its own process from `FootageTools { ffmpeg, temp }` (`crates/footage/src/decoder.rs`, built in `crates/render/src/footage.rs`). Both take the path from `EngineFiles.ffmpeg` (`crates/engine/src/tools.rs`), which comes from `settings.engine.ffmpeg`.
- **Downloaded today.** `install_ffmpeg` (`crates/engine/src/install.rs`) fetches the ffmpeg-static b6.1.1 program and licence for five targets, pinned in `crates/engine/src/downloads.generated.json` (`ffmpegRelease`, the `ffmpeg` entries; `pin.rs`). `check_ffmpeg` (`crates/media/src/capabilities.rs`) checks every `REQUIRED_*` encoder, decoder, format and filter before the path is saved, `verify_on_start` hashes it again when it changed, and its hash goes into every node key (`Release.engine`, `crates/runner/src/keys.rs`). Its licence is listed from the engine folder (`ffmpeg_licence` in `crates/desktop/src/engine.rs`, `crates/engine/src/licences.rs`, the FFmpeg entry in `crates/desktop/src/settings.rs`). Setup and About show it as "Video tools" (`apps/frontend/src/app/setup.tsx`, `settings/about-page.tsx`), and `apps/frontend/src/store/environment.ts` counts it as missing engine.
- **How Tauri places a sidecar.** `bundle.externalBin` names files `<name>-<target triple>[.exe]`. tauri-build copies each next to the program on every build of a config that declares it, and fails the build when one is missing (`copy_binaries` in tauri-build's `lib.rs`). The bundles put it beside the main program with the triple removed: the deb in `/usr/bin/<name>` (tauri-bundler `bundle/linux/debian.rs`, `copy_binaries` in `bundle/settings.rs`), the macOS app in `Contents/MacOS`, Windows in the install folder.
- **Size.** The Linux x64 program is 79,826,272 bytes. Compressed, the five b6.1.1 programs are 19,246,198 (macOS Arm) to 29,581,307 bytes (Windows x64).

**Work.**

1. **Staging.** A Rust binary in `opsis-engine`, beside `install-engine` (scripts reach the engine only through Rust binaries, `scripts/lib/engine.ts`), downloads the pinned program and licence for a target, checks sha256 and size, and writes `crates/desktop/binaries/opsis-ffmpeg-<target triple>[.exe]` with `LICENSE.ffmpeg.txt` into `resources/`. Downloads are cached in `~/.cache/opsis-ffmpeg`. `bun run stage` runs it for packaging; `bun start`, the e2e runner and the test setup run it when the program is missing. `crates/desktop/binaries/` is gitignored and added to the generated-code table in `AGENTS.md`.
2. **The name is `opsis-ffmpeg`, never `ffmpeg`.** The deb would install `/usr/bin/ffmpeg`, which belongs to the distribution's `ffmpeg` package, and the install would fail on the file conflict.
3. **Config.** `"externalBin": ["binaries/opsis-ffmpeg"]` in `tauri.release.conf.json` only, so the base config stays bundle-free and dev and e2e builds compile without the file.
4. **Lookup.** One function in `opsis-engine`: `OPSIS_FFMPEG` (an absolute path) when it is set, read by code under `#[cfg(debug_assertions)]`; otherwise `current_exe()`'s folder joined with `opsis-ffmpeg` and the platform's executable suffix; `EngineMissing` when neither exists. Release builds don't contain the variable's code. `tauri dev` and `cargo test` build with debug assertions, and the `e2e` feature refuses to build without them (`compile_error!` in `crates/desktop/src/lib.rs`), so all three read it. `bun start`, the e2e runner and the test setup set `OPSIS_FFMPEG` to the staged program. `EngineFiles.ffmpeg` and `FootageTools.ffmpeg` take the result. Windows keeps `CREATE_NO_WINDOW`, and every run keeps its child temp folder.
5. **Remove the FFmpeg half of the engine.** `EngineTool::Ffmpeg`, `settings.engine.ffmpeg`, `install_ffmpeg` and `EnginePart::Ffmpeg`, the FFmpeg entries and `ffmpegRelease` in `downloads.generated.json` (the pin moves to the staging binary's own pinned JSON), the FFmpeg targets in `pin.rs`, the start-up check of the FFmpeg path, `ffmpeg_licence` reading the engine folder, the Video tools rows in Setup and About, and the FFmpeg check in `environment.ts` and `crates/desktop/src/analysis.rs`. Engine status is complete when the browser is ready. Settings → Engine sets up the browser only, and its copy says so. FFmpeg appears only under Settings → Licences.
6. **Keep the capability check as the swap contract.** `check_ffmpeg` and the `REQUIRED_*` lists stay, run as a test on the staged program on every CI runner. The same test fails when the configuration line lacks `--enable-gpl --enable-version3` or has `--enable-nonfree`.
7. **Cache keys.** `Release` drops the FFmpeg hash: the bundled program only changes with a release, and the app version is already in every key. `ENGINE_VERSION` stays as it is; clear local test caches instead.
8. **Tests.** Tests that start FFmpeg read `OPSIS_FFMPEG`. The ffprobe the probe parity test reads (`crates/probe/tests/ffprobe.rs`) moves beside the staged program in `~/.cache/opsis-ffmpeg`. The e2e data folder seeds only the browser in `settings.engine`, and `engineReady` waits for the browser only.
9. **macOS.** The sidecar is signed with the hardened runtime and notarized with the app; the CI step that checks every Mach-O with `codesign` covers it. Run an export on the macOS runners to confirm the ffmpeg-static program works under the hardened runtime without extra entitlements.
10. **Size.** Measure the deb again and update `plans/release.md` section 1 (What ships, Package size).
11. **Docs.** `AGENTS.md` (One engine, Packaging, Licences, Cache keys, End-to-end tests, and `OPSIS_FFMPEG` beside `OPSIS_USER_DATA` in Tooling), `README.md` (setup, packaging, the FFmpeg row of the stack table), `THIRD_PARTY_NOTICES.md` (the FFmpeg section) and `plans/release.md` ("one build with no engine inside").

**Decided.**

- ffmpeg-static b6.1.1 is the sidecar on every platform, macOS included, until section 3's build replaces it.
- No shell plugin: the app keeps starting FFmpeg with its own process code, only the path changes.
- Users can no longer choose their own FFmpeg; `OPSIS_FFMPEG` is for development and tests only.
- About shows nothing about FFmpeg.

## 2. Licence and source notice

**In place.** Opsis is GPL-3.0-or-later. The ffmpeg-static programs are GPLv3 builds (`--enable-gpl --enable-version3`, no `--enable-nonfree`).

**Work.**

1. **The licence.** Ship the release's `LICENSE` file in `resources/` and list it in Settings → Licences from the resource folder (`AppPaths`), not from the engine folder.
2. **The source notice.** Links to the upstream source, in `THIRD_PARTY_NOTICES.md` and each GitHub release's notes: FFmpeg's source and the ffmpeg-static release the programs come from.
3. **Links stay working.** CI checks every link in the notice before each release.

**Decided.** Upstream links only; no source tarballs attached to releases.

## 3. Own build

The aim is one FFmpeg version on every platform, configured with only what the `REQUIRED_*` lists name, from sources Opsis pins. A separate program needs none of the work FFmpeg inside the app would: no bindings, no link order, no matching C runtime.

**Work.**

1. **Pinned sources.** A generated JSON with the URL, sha256 and size of FFmpeg, x264, x265, libvpx, LAME, Opus, Rubber Band (its built-in FFT and resampler, so no FFTW or libsamplerate) and zlib, regenerated by a command like `bun run engine:downloads`.
2. **The build binary.** A Rust binary builds every library and then FFmpeg for one target into `~/.cache/opsis-ffmpeg/<target>/<hash of the pins and flags>/`. FFmpeg is configured with `--enable-gpl --enable-version3 --disable-autodetect --disable-everything --disable-network --disable-doc --disable-ffplay --disable-ffprobe`, then only the encoders, decoders, muxers, demuxers, filters, parsers, bitstream filters and protocols Opsis uses: the `REQUIRED_*` lists, plus what they need without naming it (the `file` and `pipe` protocols, the `concat` demuxer, the `movie` source behind `amovie`, the parsers and bitstream filters stream copies need). The full test suite with the ignored tests is what proves the list complete.
3. **Platforms.**
   - Linux x64 and arm64: fully static with musl on native runners, so one program runs on any distribution.
   - macOS Arm and Intel: on each macOS runner, with the deployment target matching the app's minimum.
   - Windows x64: cross-compiled from Linux with llvm-mingw. A separate program never shares the app's C runtime, so MSVC isn't needed.
4. **Releases.** CI builds the programs and attaches them to an Opsis GitHub release of their own, with the build binary's source. The staging binary pins those programs the way it pinned ffmpeg-static, and the source notice links to the pinned sources.
5. **The swap.** Change the pin, run the capability test and every test with the ignored ones, then record the golden files and render hashes again: another FFmpeg version changes the output.
6. **Size.** Measure the trimmed program on every platform and update `plans/release.md`.

## 4. Not chosen: FFmpeg inside the app

Linking FFmpeg into the Rust program (owned bindgen bindings and a small C file built with `cc` for the `va_list` log callback and the C macros; `cxx` isn't needed, since FFmpeg's API is C) keeps the filter graph strings but means rewriting what the `ffmpeg` program does: reading inputs, feeding many inputs into one graph, stream copies, seeks and trims, `concat`, chapters and metadata, `faststart`, encoder options and progress, for the ~50 options Opsis passes. It also needs every library built statically against MSVC's C runtime on Windows, routing FFmpeg's process-wide log callback per job, and it gives up crash isolation: a crash in an encoder or filter would end the app. The sidecar gets the same shipped FFmpeg without any of it. Look at it again only if starting processes is measured to cost real time, for example footage decoders restarting on every seek.

## 5. Risks

- **Different FFmpeg per platform.** ffmpeg-static b6.1.1 is FFmpeg 7.0.2 on Linux and 6.1.1 on Windows, so until section 3 exports and golden results can differ between platforms.
- **Warnings on Windows.** An unsigned `opsis-ffmpeg.exe` inside the installer may be flagged by SmartScreen or virus scanners; sign it with the app once code signing exists (`plans/release.md` section 5).
- **Package size.** The download grows by the compressed program (about 19–30 MB) and the install by the program (about 80 MB on Linux x64) until section 3 trims it.

## 6. Sources

- Tauri: `crates/tauri-build/src/lib.rs` (`copy_binaries`: sidecars copied next to the program on every build), `crates/tauri-bundler/src/bundle/settings.rs` (`copy_binaries`: the target triple removed) and `crates/tauri-bundler/src/bundle/linux/debian.rs` (sidecars into `usr/bin`), on the `tauri-apps/tauri` default branch.
- ffmpeg-static b6.1.1 release assets: `linux-x64.README`, `win32-x64.README`; `ffmpeg -version` of the downloaded Linux program.
