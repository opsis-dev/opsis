# FFmpeg

Ship FFmpeg inside the app as a Tauri sidecar instead of downloading it at setup, then replace the ffmpeg-static program with Opsis's own build. The media code doesn't change: it keeps starting the `ffmpeg` program with the same arguments, so every argument builder, filter graph and log reader stays as it is. Building FFmpeg into the app through its C API was looked at and is not planned (section 4).

## How to read this plan

- Sections 1 and 2 ship together: the sidecar and its licence and source notice. Section 3, Opsis's own build, comes after and swaps the program without touching the app. Section 4 records the option not chosen.
- Each section has the same parts where they apply: **In place**, **Work** (numbered, only what is left), **Decided**.
- Each piece of work ends with the usual checks: `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, and the tests of the crates that start FFmpeg (`bun run test:rust -p opsis-media -p opsis-footage -p opsis-render -p opsis-capture -- --ignored`, which sets `OPSIS_FFMPEG`).
- This plan changes `AGENTS.md` on purpose (One engine, Packaging, Licences, Cache keys, End-to-end tests, Tooling); every such change updates `AGENTS.md`, `README.md`, `THIRD_PARTY_NOTICES.md` and `plans/release.md` in the same commit.

## Summary

| Topic                         | What users get                                                     | Left to do                                            | Status      |
| ----------------------------- | ------------------------------------------------------------------ | ----------------------------------------------------- | ----------- |
| 1. Sidecar with ffmpeg-static | Exports work without setting up FFmpeg; only the browser downloads | macOS untested                                        | built       |
| 2. Licence and source notice  | FFmpeg's licence and where its source is                           | release notes, once releases are made                 | built       |
| 3. Own build                  | One FFmpeg version on every platform, only what Opsis uses         | pinned sources, the build binary, CI builds, the swap | not started |
| 4. FFmpeg inside the app      | Nothing                                                            | not planned                                           | not chosen  |

## 1. Sidecar with ffmpeg-static

**In place.** Built on 2026-11-01.

- **Pins and staging.** `scripts/lib/ffmpeg.generated.json` pins the ffmpeg-static b6.1.1 program and licence per target triple (`bun run ffmpeg:pin`, `scripts/ffmpeg-pin.ts`); `downloads.generated.json` pins only the browser. `stageFfmpeg` (`scripts/lib/ffmpeg.ts`) downloads a target's pins into `~/.cache/opsis-ffmpeg`, checks sha256 and size, and writes `crates/desktop/binaries/opsis-ffmpeg-<target triple>[.exe]` (gitignored) and `resources/LICENSE.ffmpeg.txt`. `scripts/lib/ffmpeg.ts` takes the target from `rustc -vV`; `bun run stage` always stages, while `bun start` (`scripts/start.ts`), `bun run test:rust` (`scripts/test-rust.ts`), the e2e runner (`wdio.conf.ts`) and `bun run ffmpeg:stage` stage only when the program or licence is missing, then set `OPSIS_FFMPEG`.
- **Config.** `"externalBin": ["binaries/opsis-ffmpeg"]` and the licence resource are in `tauri.release.conf.json` only. The deb installs `/usr/bin/opsis-ffmpeg` beside `/usr/bin/opsis-desktop`, and `LICENSE.ffmpeg.txt` in `/usr/lib/Opsis`.
- **Lookup.** `bundled_ffmpeg` (`crates/engine/src/ffmpeg.rs`): `OPSIS_FFMPEG` in builds with debug assertions (an absolute path to a file), otherwise `opsis-ffmpeg` beside `current_exe()`. When neither is there it returns `FfmpegMissing`, "Opsis's video tools are missing. Install Opsis again." (`EngineMissing` would send people to Settings → Engine, which can no longer fix it.) `EngineFiles.ffmpeg` and, through it, `FootageTools.ffmpeg` take it.
- **The engine is the browser.** `EngineTool`, `EnginePart` and `EngineInstallUpdate.part` are gone (one value each was left), as are `settings.engine.ffmpeg`, `EngineStatus.ffmpeg`, `install_ffmpeg` and the FFmpeg start-up check. `choose_engine_tool` became `choose_browser`. Engine status is complete when the browser is ready. Setup, Settings → Engine, About and the Licences page copy name only the recording browser, and `ENGINE_MISSING_MESSAGE` says "The recording browser isn't set up yet." Analysis no longer reports the engine as missing: `AnalysisResult::Error.engine_missing`, the editor's `unavailable` media state and the Mixer's "Set up the engine" line are gone, because mixing and analysis need no setup.
- **The swap contract.** `check_ffmpeg` also refuses a configuration line without `--enable-gpl --enable-version3` or with `--enable-nonfree`. It runs as a test on the staged program in `bun run test`, and after the build on every CI build runner (`bun run test:rust -p opsis-media --lib capabilities`).
- **Cache keys.** `Release.engine` holds only the browser's hash. `ENGINE_VERSION` is unchanged.
- **Tests.** Tests that start FFmpeg read `OPSIS_FFMPEG` through `opsis_golden::ffmpeg()` or `EngineFiles`. The probe parity test reads ffprobe from `~/.cache/opsis-ffmpeg`. The e2e and check-app data folders seed only the browser.
- **Size.** Measured 2026-11-01 on Linux x64: the deb grew from 48,625,200 to 78,800,484 bytes, and the install by the 79,826,272-byte program (`plans/release.md` section 1).

**Work.**

1. **macOS.** The sidecar is signed and notarized with the app. That it runs under the hardened runtime without extra entitlements is unchecked; nothing tests it in CI. Look at it if exports fail on macOS.

**Decided.**

- ffmpeg-static b6.1.1 is the sidecar on every platform, macOS included, until section 3's build replaces it.
- No shell plugin: the app keeps starting FFmpeg with its own process code, only the path changes.
- Users can no longer choose their own FFmpeg; `OPSIS_FFMPEG` is for development and tests only.
- About shows nothing about FFmpeg.
- No CI export run on macOS: the sidecar ships there untested.

## 2. Licence and source notice

**In place.** Opsis is GPL-3.0-or-later. The ffmpeg-static programs are GPLv3 builds (`--enable-gpl --enable-version3`, no `--enable-nonfree`). The release's licence ships as `LICENSE.ffmpeg.txt` and Settings → Licences lists it from the resource folder (`shipped` in `crates/engine/src/licences.rs`). `THIRD_PARTY_NOTICES.md` links FFmpeg's source and the ffmpeg-static release; a test in `opsis-engine` checks that the release link is the pinned one. `bun run notice:links` (`scripts/notice-links.ts`) fetches every link in the section, and CI runs it on `v*` tags.

**Work.**

1. **Release notes.** Put the source notice in each GitHub release's notes once a workflow creates releases (none does yet).

**Decided.** Upstream links only; no source tarballs attached to releases. Until releases are made by CI, the notice lives in `THIRD_PARTY_NOTICES.md` only.

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
