# Packaging and licences

How a release build is made, what it ships, where a packaged app reads its resources, the app icon, and the licence rules for dependencies, vendored code and the bundled FFmpeg. Paths are relative to the repository root. The engine and FFmpeg at run time are in [engine.md](engine.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                  | Where                                                                                                                                 |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------- |
| Build                    | `bun run make` → `scripts/make.ts`; `crates/desktop/tauri.release.conf.json`                                                          |
| Staging                  | `scripts/stage.ts`; `scripts/lib/engine.ts` → `writeChromiumCredits`; `scripts/lib/unzip.ts`; `scripts/lib/ffmpeg.ts` → `stageFfmpeg` |
| Resource paths           | `crates/session/src/paths.rs` → `AppPaths`                                                                                            |
| Release profile          | `[profile.release]` in `Cargo.toml`                                                                                                   |
| CI                       | `.github/workflows/build.yml`                                                                                                         |
| Icons                    | `brand/logo.svg` → `bun run brand:icons` → `brand/icons/` → `bun run app:icons` → `crates/desktop/icons`                              |
| Crate licence allow-list | `deny.toml`                                                                                                                           |
| Notices                  | `THIRD_PARTY_NOTICES.md`, `packages/video/src/camera/LICENSE.programatic-demo`, `crates/capture/LICENSE.programatic-demo`             |
| Notice link check        | `bun run notice:links` (`scripts/notice-links.ts`)                                                                                    |

## Rules

### Building

- **`bun run make` runs `bun run stage`, then `tauri build --config tauri.release.conf.json` in `crates/desktop`**; extra arguments go to `tauri build` (`-- --bundles deb`).
- **Locally, build the deb only, cap cargo with `CARGO_BUILD_JOBS` and put the output in the cache (`CARGO_TARGET_DIR=~/.cache/opsis-build`)**; CI builds every format.
- **There is one build: it ships FFmpeg as a sidecar and never the recording browser.**
- **`stage.ts` builds the film and terminal pages without source maps**, writes the browser's Chromium credits into the gitignored `resources/engine-licences/LICENSES.chromium.html`, and stages FFmpeg again.
  - The credits are read from `chrome://credits` through `playwright-core`, in Chrome for Testing of the version `install-engine` downloads into `~/.cache/opsis-stage-engine`, unpacked with `@zip.js/zip.js` into `~/.cache/opsis-chromium-credits` keeping the macOS app's symlinks and executable bits.
  - FFmpeg is staged for the host target (`rustc -vV`) as `crates/desktop/binaries/opsis-ffmpeg-<target triple>[.exe]`, checked against the pins, with its licence as `resources/LICENSE.ffmpeg.txt`.
- **FFmpeg is a Tauri sidecar declared only in `tauri.release.conf.json` (`"externalBin": ["binaries/opsis-ffmpeg"]`)**, so `tauri dev` and the e2e build compile without the file. The bundles put it beside the main program with the triple removed.
- **It is named `opsis-ffmpeg`, never `ffmpeg`**, because the deb would otherwise clash with the distribution's `ffmpeg` package. The app starts it with its own process code, no shell plugin.
- **The bundle settings live only in `tauri.release.conf.json`** (targets, resources, the deb's `gstreamer1.0-libav` dependency, NSIS per-user install, macOS hardened runtime); the base `tauri.conf.json` stays bundle-free so `tauri dev` and the e2e build need no staged files.
- **Tauri packs every binary target of `opsis-desktop`, so dev-only binaries sit behind a feature (`export-bindings`).**
- **Release builds use `opt-level = 3`, full LTO and one codegen unit, and are stripped**; expect a slow, memory-heavy link.
- **A packaged app reads its resources from Tauri's resource folder (`AppPaths`)**: the film page (whose `public/` files are the app's only copy of the `app:` files), the terminal page, `engine-licences`, `LICENSE.ffmpeg.txt`, `LICENSE`, `THIRD_PARTY_NOTICES.md`. In development everything comes from the repository.
- **Linux ships deb, rpm and AppImage (the last two in CI only); Windows NSIS; macOS the app, zipped, signed and notarized in CI** with the `APPLE_*` secrets, where every Mach-O inside is checked with `codesign`.
- **Check the workflow with `actionlint`.**

### Icons

- **`brand/logo.svg` is the one source for the app icon.** `bun run brand:icons` renders `brand/icons/` with the recording browser (ImageMagick draws the gradients wrong), and `bun run app:icons` makes `crates/desktop/icons` from the 1024 px PNG with `tauri icon`.

### Licences

- **Opsis is GPL-3.0-or-later.** `LICENSE` is the official GPLv3 text from gnu.org, unchanged; the "or later" is declared in `package.json`, `Cargo.toml` and `tauri.release.conf.json`.
- **Every dependency must be GPLv3-compatible**: MIT, MIT-0, BSD, 0BSD, ISC, Zlib, Apache-2.0 with or without the LLVM exception, BSL-1.0, bzip2-1.0.6, CC0-1.0, IJG, Unicode-3.0, Unlicense, MPL-2.0 without the "Incompatible With Secondary Licenses" notice, GPLv3, LGPLv3.
- **`deny.toml` holds the crate allow-list, worked out for the five shipped platforms** (Linux x64 and arm64, Windows x64, macOS arm64 and Intel), so a licence only another platform pulls in never counts. It also denies yanked crates.
- **An advisory with no safe upgrade is an `ignore` entry with a `reason` saying where the crate comes from**, reviewed whenever dependencies move.
- **Vendored code and the icon data copied from the six glyph sets keep their notices**, with morphicons and lucide-react, which ship in the editor bundle; `glyphs:bundle` refuses a set whose notice is missing.
- **FFmpeg ships as a separate program, the unchanged ffmpeg-static GPLv3 build**, with the release's licence as `LICENSE.ffmpeg.txt`, listed in Settings → Licences from the resource folder.
- **Its source notice in `THIRD_PARTY_NOTICES.md` links upstream only** (FFmpeg's source and the ffmpeg-static release, which a test keeps on the pinned release); no source tarballs are attached, and CI checks the links on `v*` tags (`bun run notice:links`).
- **Built-in sounds, songs and fonts carry their credits in `THIRD_PARTY_NOTICES.md`** ([audio.md](audio.md), [fonts.md](fonts.md)).

## Deliberate limits

- **There is no updater, nothing is signed outside the macOS CI step, and the Windows and macOS packages are built but not checked by running them.**
- **The notices of the crates and npm packages compiled into the app are deliberately not shipped.**
