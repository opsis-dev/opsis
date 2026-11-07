# Packaging and licences

How a release build and its installers are made, what they ship, where the app reads its resources, the app icon, and the licence rules for dependencies, vendored code and the bundled FFmpeg. Paths are relative to the repository root. The engine and FFmpeg at run time are in [engine.md](engine.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                  | Where                                                                                                                                 |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------- |
| Build                    | `bun run make` → `scripts/make.ts`; `scripts/lib/release.ts` → `buildRelease`, `releaseBinary`                                        |
| Static Qt                | `scripts/lib/static-qt.ts` (`bun run qt:static`); `scripts/lib/trixie.ts` → `runInTrixie`                                             |
| Installers               | `crates/packager` (`opsis-packager`): `formats.rs` → `package`, `packager_config`; `rpm_package.rs` → `build_rpm`                     |
| Package checks           | `crates/packager/src/linking.rs` → `check_self_contained`; `crates/packager/src/linux_deps.rs` (library to package names)             |
| Staging                  | `scripts/stage.ts`; `scripts/lib/engine.ts` → `writeChromiumCredits`; `scripts/lib/unzip.ts`; `scripts/lib/ffmpeg.ts` → `stageFfmpeg` |
| Resource paths           | `crates/ui/src/folders.rs` → `host_folders`, `resources_for`; `crates/session/src/paths.rs` → `AppPaths`                              |
| FFmpeg beside the app    | `crates/engine/src/ffmpeg.rs` → `bundled_ffmpeg`                                                                                      |
| Release profile          | `[profile.release]` in `Cargo.toml`                                                                                                   |
| Icons                    | `brand/logo.svg` → `bun run brand:icons` → `brand/icons/`                                                                             |
| Crate licence allow-list | `deny.toml`                                                                                                                           |
| Notices                  | `THIRD_PARTY_NOTICES.md`, `packages/video/src/camera/LICENSE.programatic-demo`, `crates/capture/LICENSE.programatic-demo`             |
| Notice link check        | `bun run notice:links` (`scripts/notice-links.ts`)                                                                                    |

## Rules

### Building

- **`bun run make` runs `bun run stage`, builds the release binary against the static Qt (`buildRelease`), then `opsis-packager` builds the installers into `~/.cache/opsis-build`.** It builds the deb on Linux, the macOS app and dmg on macOS, NSIS and MSI on Windows; `--formats` names others from `deb`, `rpm`, `appimage`, `pacman`, `nsis`, `msi`, `app` and `dmg`, and the remaining arguments go to `cargo build`.
- **On Linux the release binary builds in a plain `debian:trixie` container, into `target/trixie`**, because linking the static Qt needs X11 and xcb development files a desktop doesn't have; it then needs glibc 2.41. With `CI=true` it builds on the machine itself, into `target/release`. The container runs as your user, with the repository, Cargo, rustup, Zig and both Qt folders mounted at their own paths.
- **`opsis-packager` refuses a program that loads Qt, ICU or KDDockWidgets, or looks for libraries in an absolute folder** (`check_self_contained`), so a development build can't be packaged.
- **Every Linux package installs the same files**: the programs in `/usr/bin`, the resources in `/usr/lib/opsis`, the desktop entry and the hicolor icons. That is cargo-packager's deb layout, which its AppImage and pacman formats reuse; `build_rpm` lays out the rpm the same way with the `rpm` crate, and a test compares the three packages' file lists.
- **Package dependencies come from the libraries the program loads**: `linux_deps.rs` names each one's deb and pacman package, the rpm requires the libraries themselves, and every format also needs `ca-certificates` for the downloads the app makes when asked. A library with no entry fails the build.
- **Check a deb by installing it in a plain `debian:trixie` container** (apt resolves its dependencies) and running the installed `/usr/bin/opsis` through the `views` scenario on the offscreen platform.
- **Cap cargo with `CARGO_BUILD_JOBS` locally.**
- **There is one build: it ships FFmpeg as a program beside the app and never the recording browser.**
- **`stage.ts` builds the film and terminal pages without source maps**, writes the browser's Chromium credits into the gitignored `resources/engine-licences/LICENSES.chromium.html`, and stages FFmpeg again.
  - The credits are read from `chrome://credits` through `playwright-core`, in Chrome for Testing of the version `install-engine` downloads into `~/.cache/opsis-stage-engine`, unpacked with `@zip.js/zip.js` into `~/.cache/opsis-chromium-credits` keeping the macOS app's symlinks and executable bits.
  - FFmpeg is staged for the host target (`rustc -vV`) as `target/ffmpeg/opsis-ffmpeg-<target triple>[.exe]`, checked against the pins, with its licence as `resources/LICENSE.ffmpeg.txt`.
- **A release build finds FFmpeg beside its own program as `opsis-ffmpeg`, never `ffmpeg`**, so a package can't clash with the distribution's `ffmpeg`; a debug build takes `OPSIS_FFMPEG`. The app starts it with its own process code.
- **Release builds use `opt-level = 3`, full LTO and one codegen unit, and are stripped**; expect a slow, memory-heavy link.
- **The app reads its resources from the folder of its program, `../lib/opsis` when it was installed from a Linux package, or `../Resources` on macOS (`resources_for`)**: the film page (whose `public/` files are the app's only copy of the `app:` files), the terminal page, `engine-licences`, `LICENSE.ffmpeg.txt`, `LICENSE`, `THIRD_PARTY_NOTICES.md`. In development everything comes from the repository.
- **Check the workflow with `actionlint`.**

### Icons

- **`brand/logo.svg` is the one source for the app icon.** `bun run brand:icons` renders `brand/icons/` with the recording browser (ImageMagick draws the gradients wrong).

### Licences

- **Opsis is GPL-3.0-or-later.** `LICENSE` is the official GPLv3 text from gnu.org, unchanged; the "or later" is declared in `package.json` and `Cargo.toml`.
- **Every dependency must be GPLv3-compatible**: MIT, MIT-0, BSD, 0BSD, ISC, Zlib, Apache-2.0 with or without the LLVM exception, BSL-1.0, bzip2-1.0.6, CC0-1.0, IJG, Unicode-3.0, Unlicense, MPL-2.0 without the "Incompatible With Secondary Licenses" notice, GPLv3, LGPLv3.
- **Qt and KDDockWidgets are used under the GPL-3.0, and a release links them statically, with Qt built without ICU**; development builds link the prebuilt shared Qt ([toolchain.md](toolchain.md)). Their notices, and the MIT ones of the libraries KDDockWidgets compiles in, are in `THIRD_PARTY_NOTICES.md`; the notices of the code the static Qt compiles in are still to come ([plans/native-editor.md](../plans/native-editor.md), packaging).
- **`webpki-roots` (Mozilla's certificate list, CDLA-Permissive-2.0) has a licence exception in `deny.toml`**: only `opsis-packager`'s downloader uses it, and the packaging tool isn't shipped.
- **`deny.toml` holds the crate allow-list, worked out for the five shipped platforms** (Linux x64 and arm64, Windows x64, macOS arm64 and Intel), so a licence only another platform pulls in never counts. It also denies yanked crates.
- **An advisory with no safe upgrade is an `ignore` entry with a `reason` saying where the crate comes from**, reviewed whenever dependencies move.
- **Vendored code and the icon data copied from the six glyph sets keep their notices**; `glyphs:bundle` refuses a set whose notice is missing.
- **FFmpeg ships as a separate program, the unchanged ffmpeg-static GPLv3 build**, with the release's licence as `LICENSE.ffmpeg.txt`, listed in Settings → Licences from the resource folder.
- **Its source notice in `THIRD_PARTY_NOTICES.md` links upstream only** (FFmpeg's source and the ffmpeg-static release, which a test keeps on the pinned release); no source tarballs are attached, and CI checks the links on `v*` tags (`bun run notice:links`).
- **Built-in sounds, songs and fonts carry their credits in `THIRD_PARTY_NOTICES.md`** ([audio.md](audio.md), [fonts.md](fonts.md)).

## Deliberate limits

- **Nothing is signed yet** ([plans/release.md](../plans/release.md), code signing), and the macOS and Windows packages and their static Qt are built only in CI, which doesn't build them yet.
- **There is no updater.**
- **The notices of the crates and npm packages compiled into the app are deliberately not shipped.**
