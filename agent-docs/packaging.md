# Packaging and licences

How a release build is made, what it ships, where the app reads its resources, the app icon, and the licence rules for dependencies, vendored code and the bundled FFmpeg. Paths are relative to the repository root. The engine and FFmpeg at run time are in [engine.md](engine.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                  | Where                                                                                                                                 |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------- |
| Build                    | `bun run make` → `scripts/make.ts`                                                                                                    |
| Staging                  | `scripts/stage.ts`; `scripts/lib/engine.ts` → `writeChromiumCredits`; `scripts/lib/unzip.ts`; `scripts/lib/ffmpeg.ts` → `stageFfmpeg` |
| Resource paths           | `crates/ui/src/folders.rs` → `host_folders`; `crates/session/src/paths.rs` → `AppPaths`                                               |
| FFmpeg beside the app    | `crates/engine/src/ffmpeg.rs` → `bundled_ffmpeg`                                                                                      |
| Release profile          | `[profile.release]` in `Cargo.toml`                                                                                                   |
| Icons                    | `brand/logo.svg` → `bun run brand:icons` → `brand/icons/`                                                                             |
| Crate licence allow-list | `deny.toml`                                                                                                                           |
| Notices                  | `THIRD_PARTY_NOTICES.md`, `packages/video/src/camera/LICENSE.programatic-demo`, `crates/capture/LICENSE.programatic-demo`             |
| Notice link check        | `bun run notice:links` (`scripts/notice-links.ts`)                                                                                    |

## Rules

### Building

- **`bun run make` runs `bun run stage`, then builds the release binary (`cargo build --release -p opsis-ui`, `target/release/opsis`)**; extra arguments go to `cargo build`. It makes no installer: the formats and how they bundle Qt are decided in the packaging step of [plans/native-editor.md](../plans/native-editor.md).
- **Cap cargo with `CARGO_BUILD_JOBS` locally.**
- **There is one build: it ships FFmpeg as a program beside the app and never the recording browser.**
- **`stage.ts` builds the film and terminal pages without source maps**, writes the browser's Chromium credits into the gitignored `resources/engine-licences/LICENSES.chromium.html`, and stages FFmpeg again.
  - The credits are read from `chrome://credits` through `playwright-core`, in Chrome for Testing of the version `install-engine` downloads into `~/.cache/opsis-stage-engine`, unpacked with `@zip.js/zip.js` into `~/.cache/opsis-chromium-credits` keeping the macOS app's symlinks and executable bits.
  - FFmpeg is staged for the host target (`rustc -vV`) as `target/ffmpeg/opsis-ffmpeg-<target triple>[.exe]`, checked against the pins, with its licence as `resources/LICENSE.ffmpeg.txt`.
- **A release build finds FFmpeg beside its own program as `opsis-ffmpeg`, never `ffmpeg`**, so a package can't clash with the distribution's `ffmpeg`; a debug build takes `OPSIS_FFMPEG`. The app starts it with its own process code.
- **Release builds use `opt-level = 3`, full LTO and one codegen unit, and are stripped**; expect a slow, memory-heavy link.
- **The app reads its resources from the folder of its program, or `../Resources` on macOS (`host_folders`)**: the film page (whose `public/` files are the app's only copy of the `app:` files), the terminal page, `engine-licences`, `LICENSE.ffmpeg.txt`, `LICENSE`, `THIRD_PARTY_NOTICES.md`. In development everything comes from the repository.
- **Check the workflow with `actionlint`.**

### Icons

- **`brand/logo.svg` is the one source for the app icon.** `bun run brand:icons` renders `brand/icons/` with the recording browser (ImageMagick draws the gradients wrong).

### Licences

- **Opsis is GPL-3.0-or-later.** `LICENSE` is the official GPLv3 text from gnu.org, unchanged; the "or later" is declared in `package.json` and `Cargo.toml`.
- **Every dependency must be GPLv3-compatible**: MIT, MIT-0, BSD, 0BSD, ISC, Zlib, Apache-2.0 with or without the LLVM exception, BSL-1.0, bzip2-1.0.6, CC0-1.0, IJG, Unicode-3.0, Unlicense, MPL-2.0 without the "Incompatible With Secondary Licenses" notice, GPLv3, LGPLv3.
- **Qt and KDDockWidgets are used under the GPL-3.0, and a release links them statically, with Qt built without ICU**; development builds link the prebuilt shared Qt ([toolchain.md](toolchain.md)). Their notices, and the MIT ones of the libraries KDDockWidgets compiles in, are in `THIRD_PARTY_NOTICES.md`; the static Qt builds come with the packaging step of [plans/native-editor.md](../plans/native-editor.md).
- **`deny.toml` holds the crate allow-list, worked out for the five shipped platforms** (Linux x64 and arm64, Windows x64, macOS arm64 and Intel), so a licence only another platform pulls in never counts. It also denies yanked crates.
- **An advisory with no safe upgrade is an `ignore` entry with a `reason` saying where the crate comes from**, reviewed whenever dependencies move.
- **Vendored code and the icon data copied from the six glyph sets keep their notices**; `glyphs:bundle` refuses a set whose notice is missing.
- **FFmpeg ships as a separate program, the unchanged ffmpeg-static GPLv3 build**, with the release's licence as `LICENSE.ffmpeg.txt`, listed in Settings → Licences from the resource folder.
- **Its source notice in `THIRD_PARTY_NOTICES.md` links upstream only** (FFmpeg's source and the ffmpeg-static release, which a test keeps on the pinned release); no source tarballs are attached, and CI checks the links on `v*` tags (`bun run notice:links`).
- **Built-in sounds, songs and fonts carry their credits in `THIRD_PARTY_NOTICES.md`** ([audio.md](audio.md), [fonts.md](fonts.md)).

## Deliberate limits

- **No installer is built and nothing is signed until the packaging step**, and CI builds only on Linux until then.
- **There is no updater.**
- **The notices of the crates and npm packages compiled into the app are deliberately not shipped.**
