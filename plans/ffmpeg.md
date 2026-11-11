# FFmpeg

Replace the ffmpeg-static program the app ships as its `opsis-ffmpeg` sidecar with Opsis's own build, and finish the sidecar's two loose ends: macOS and the release notes. The media code doesn't change: it keeps starting the same program with the same arguments, so every argument builder, filter graph and log reader stays as it is. Building FFmpeg into the app through its C API was looked at and is not planned (section 4).

## How to read this plan

- Sections 1 and 2 are the sidecar's loose ends. Section 3, Opsis's own build, swaps the program without touching the app. Section 4 records the option not chosen.
- Each section has the same parts where they apply: **Work** (numbered, only what is left), **Decided**.
- Each piece of work ends with the usual checks: `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, and the tests of the crates that start FFmpeg (`bun run test:rust -p opsis-media -p opsis-footage -p opsis-render -p opsis-capture -- --ignored`, which sets `OPSIS_FFMPEG`).
- This plan changes the docs on purpose (the generated-code table in `AGENTS.md`, `agent-docs/engine.md`, `agent-docs/packaging.md`); every such change updates those docs, `README.md`, `THIRD_PARTY_NOTICES.md` and `plans/release.md` in the same commit.

## Summary

| Topic                        | What users get                                             | Left to do                                            | Status             |
| ---------------------------- | ---------------------------------------------------------- | ----------------------------------------------------- | ------------------ |
| 1. The sidecar on macOS      | Exports that work on a signed Mac app                      | check it under the hardened runtime                   | waiting on signing |
| 2. Source notice in releases | Where FFmpeg's source is, in each release's notes          | the release notes                                     | not built          |
| 3. Own build                 | One FFmpeg version on every platform, only what Opsis uses | pinned sources, the build script, CI builds, the swap | not started        |
| 4. FFmpeg inside the app     | Nothing                                                    | not planned                                           | not chosen         |

## 1. The sidecar on macOS

**Work.**

1. **Hardened runtime.** Once macOS signing is in place (`plans/release.md` section 4), the sidecar is signed with the hardened runtime and notarized with the app. Whether ffmpeg-static runs under it without extra entitlements is unchecked, and nothing tests it in CI: the capability check on the macOS runners starts the staged program, not the signed one in `Contents/MacOS`. Look at it if exports fail on macOS.

**Decided.**

- ffmpeg-static b6.1.1 is the sidecar on every platform, macOS included, until section 3's build replaces it.
- About shows nothing about FFmpeg.
- No CI export run on macOS: the sidecar ships there untested.

## 2. Source notice in releases

**Work.**

1. **Release notes.** Put the source notice in each GitHub release's notes: the `draft` job in `.github/workflows/release.yml` writes them from `latest.json`, whose notes `scripts/release-feed.ts` takes from `CHANGELOG.md` only.

**Decided.** Upstream links only; no source tarballs attached to releases. Until the release notes carry it, the notice lives in `THIRD_PARTY_NOTICES.md` only.

## 3. Own build

The aim is one FFmpeg version on every platform, configured with only what the `REQUIRED_*` lists name, from sources Opsis pins. A separate program needs none of the work FFmpeg inside the app would: no bindings, no link order, no matching C runtime.

**Work.**

1. **Pinned sources.** A generated JSON in `scripts/lib/` with the URL, sha256 and size of FFmpeg, x264, x265, libvpx, LAME, Opus, Rubber Band (its built-in FFT and resampler, so no FFTW or libsamplerate) and zlib, written by a pin script like `scripts/ffmpeg-pin.ts` (`pinned` and `writeGeneratedJson` in `scripts/lib/pins.ts`).
2. **The build script.** A Bun script builds every library and then FFmpeg for one target into `~/.cache/opsis-dev/ffmpeg/<target>/<hash of the pins and flags>/`. FFmpeg is configured with `--enable-gpl --enable-version3 --disable-autodetect --disable-everything --disable-network --disable-doc --disable-ffplay --disable-ffprobe`, then only the encoders, decoders, muxers, demuxers, filters, parsers, bitstream filters and protocols Opsis uses: the `REQUIRED_*` lists, plus what they need without naming it (the `file` and `pipe` protocols, the `concat` demuxer, the `movie` source behind `amovie`, the parsers and bitstream filters stream copies need). The full test suite with the ignored tests is what proves the list complete.
3. **Platforms.**
   - Linux x64 and arm64: fully static with musl on native runners, so one program runs on any distribution.
   - macOS Arm and Intel: on each macOS runner, with the deployment target at the app's minimum (14.4, the oldest macOS Qt 6.12 runs on).
   - Windows x64: cross-compiled from Linux with llvm-mingw. A separate program never shares the app's C runtime, so MSVC isn't needed.
4. **Releases.** CI builds the programs and attaches them to an Opsis GitHub release of their own, each with a licence file covering FFmpeg and the libraries in it, and with the build script and its pins. `scripts/lib/ffmpeg.ts` points `FFMPEG_RELEASE`, `releaseUrl` (which builds ffmpeg-static's URL), the asset names in `FFMPEG_TARGETS` and the `ffmpegProgramUrl`, `ffprobeProgramUrl` and `ffmpegLicenceUrl` helpers at that release and `bun run ffmpeg:pin` pins its programs and licences the way it pins ffmpeg-static's. The source notice links to the pinned sources, and the notice test in `scripts/lib/ffmpeg.test.ts` checks that link instead of the ffmpeg-static release.
5. **The swap.** Change the pins, then stage again with `bun run stage` or after deleting `target/ffmpeg/`: `bun start`, `bun run test:rust` and the e2e runner stage only when the program is missing, so they would keep the old one. Run the capability test and every test with the ignored ones, then record the golden files and render hashes again: another FFmpeg version changes the output.
6. **Size.** Measure the trimmed program on every platform and update `plans/release.md`.
7. **ffprobe (undecided).** The build disables ffprobe, but today each target's pins in `scripts/lib/ffmpeg.generated.json` include a `probe` (required by the schema in `scripts/lib/ffmpeg.ts`), `stageFfprobe` stages it for `bun run ffprobe:stage`, and the parity test `crates/probe/tests/ffprobe.rs` compares `opsis-probe` with it ([engine.md](../agent-docs/engine.md)). Either keep pinning ffprobe from ffmpeg-static for that test only, or build ffprobe too; the schema, `scripts/ffprobe-stage.ts` and engine.md follow the choice.

## 4. Not chosen: FFmpeg inside the app

Linking FFmpeg into the Rust program (owned bindgen bindings and a small C file built with `cc` for the `va_list` log callback and the C macros; `cxx` isn't needed, since FFmpeg's API is C) keeps the filter graph strings but means rewriting what the `ffmpeg` program does: reading inputs, feeding many inputs into one graph, stream copies, seeks and trims, `concat`, chapters and metadata, `faststart`, encoder options and progress, for the ~50 options Opsis passes. It also needs every library built statically against MSVC's C runtime on Windows, routing FFmpeg's process-wide log callback per job, and it gives up crash isolation: a crash in an encoder or filter would end the app. The sidecar gets the same shipped FFmpeg without any of it. Look at it again only if starting processes is measured to cost real time, for example footage decoders restarting on every seek.

## 5. Risks

- **Different FFmpeg per platform.** ffmpeg-static b6.1.1 is FFmpeg 7.0.2 on Linux and 6.1.1 on Windows, so until section 3 exports and golden results can differ between platforms.
- **Warnings on Windows.** An unsigned `opsis-ffmpeg.exe` inside the installer may be flagged by SmartScreen or virus scanners; sign it with the app once code signing exists (`plans/release.md` section 4).
- **Package size.** The download grows by the compressed program (about 19–30 MB) and the install by the program (about 80 MB on Linux x64) until section 3 trims it.

## 6. Sources

- ffmpeg-static b6.1.1 release assets: `linux-x64.README`, `win32-x64.README`; `ffmpeg -version` of the downloaded Linux program.
