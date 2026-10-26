# Packaging

How the Tauri app is built and shipped: deb and rpm on Linux (no AppImage), NSIS on Windows, an app bundle on macOS, signing, updates, icons and end-to-end tests. This replaces Electron Forge; `plans/shipping.md` keeps the release process and gets a **Rust** section pointing here.

Configuration in `crates/desktop` (`opsis-desktop`'s `tauri.conf.json`) and CI (phase 8).

## What it adds

- Installers built by Tauri's bundler on every OS from one config.
- An rpm without `rpmbuild`, and a deb that pulls in what the editor's video playback needs.

## Fit with Opsis

Already in place:

- `forge.config.ts` with Squirrel, ZIP, deb and AppImage makers, `stageEngine`, `stageCompositorPackages`, the prebuilt Remotion bundle in `packageAfterCopy`, fuses, and the full and lite editions.
- `.github/workflows/build.yml` running checks and `make` on Linux, Windows and two macOS runners.
- `brand/logo.svg` as the one icon source, rendered to PNGs by the recording browser (`bun run brand:icons`).
- The engine and `LICENSES.chromium.html` staged for the full edition; the lite edition's licence file in `resources/engine-licences/`.

Missing:

- Tauri bundling, signing and updater configuration.

## Work

1. **Linux: deb, rpm and AppImage** (the AppImage was added in phase 8; CI builds all three, a local build only the deb). Tauri's bundler (tauri-bundler 2.10.4) builds the rpm itself with the pure-Rust `rpm` crate (0.30), so no `rpmbuild` is needed and both build on Ubuntu runners. Both `bundle.linux.deb` and `bundle.linux.rpm` take `depends`, `recommends`, `provides`, `conflicts`, `files` and `desktopTemplate`, plus `replaces` (deb) and `obsoletes` (rpm). The deb already depends on `libwebkit2gtk-4.1-0` and `libgtk-3-0`. Build on Ubuntu 22.04 or Debian 12 (they ship `libwebkit2gtk-4.1-dev`); building on a newer base raises the minimum glibc. Forge's AppImage maker and `scripts/appimage-launcher.sh` go; the AppImage comes from Tauri's bundler, which packs WebKitGTK and its GStreamer plugins into it, so check in CI that it starts and plays video on a clean Ubuntu.
2. **Video playback in the webview on Linux.** The Run panel and Replay play exports and takes with `<video>`, which WebKitGTK plays through GStreamer; H.264 needs the libav plugin. Add `gstreamer1.0-libav` to the deb's `depends`.
3. **Fedora.** An rpm dependency can't fix H.264 there. Fedora's `ffmpeg-free` enables no `h264` or `hevc` decoder, only `libopenh264`, and OpenH264's decoder handles Constrained Baseline only. Opsis's H.264 exports are High profile with B-frames (an encode with `h264Args`' settings reports `profile=High`, `has_b_frames=2`). The install docs say: enable RPM Fusion and swap `ffmpeg-free` for `ffmpeg`.
4. **Windows.** NSIS `-setup.exe`, installed per user into `%LOCALAPPDATA%` without administrator rights (the default `installMode`), like Squirrel today. MSI is left out: WiX only runs on Windows and per-user installs are NSIS's default.
5. **macOS.** The app bundle in a zip (and a DMG if wanted), with `bundle.macOS.signingIdentity`, `hardenedRuntime` and `entitlements`. CI signs with `APPLE_CERTIFICATE` (base64 `.p12`), `APPLE_CERTIFICATE_PASSWORD` and `APPLE_SIGNING_IDENTITY`, and notarizes with `APPLE_API_ISSUER`, `APPLE_API_KEY` and `APPLE_API_KEY_PATH`. The bundler signed sidecars and frameworks on macOS in the v1 line (tauri-bundler 1.4.0, "Code sign sidecars and frameworks on macOS"); in CI, check every executable (app, sidecar, compositor, headless shell, FFmpeg) with `codesign -dv` and sign what the bundler left unsigned before notarizing.
6. **Bundled programs.** The sidecar through `bundle.externalBin` (`sidecar.md`); the compositor next to it; for the full edition the engine (headless shell, FFmpeg, `LICENSES.chromium.html`) through `bundle.resources`. The lite edition keeps installing the engine into `<userData>/engine` from Settings.
7. **Icons.** `tauri icon` takes a square PNG or an SVG and writes `icon.icns`, `icon.ico` and the PNGs. Run it on the 1024 px PNG the recording browser renders (`brand:icons`), not on the SVG, unless a comparison shows its SVG drawing gets the gradients right (`AGENTS.md`: ImageMagick draws them wrong).
8. **Updates.** `tauri-plugin-updater` 2.14.0: `check()` reports an update without downloading it, and `download()` and `install()` are separate calls, which fits the two-step, ask-first rule in `plans/shipping.md` section 4. Updates are signed with Tauri's own key pair (`TAURI_SIGNING_PRIVATE_KEY` in CI; signing can't be turned off). The feed is a static JSON file (`version`, `notes`, `pub_date`, `platforms.<os-arch>.url` and `signature`) on GitHub Releases, one per edition. On Linux the updater covers AppImage only, so deb and rpm users get "Version X is available" with a button to the release page, as `plans/shipping.md` already decided for the deb.
9. **End-to-end tests.** `tauri-driver` drives Windows and Linux only ("macOS has no WKWebView driver tool"); WebdriverIO with `@wdio/tauri-service` covers macOS through its embedded WebDriver server. The packaged-app checks through the Local API (`plans/shipping.md`, sections 1 and 6) work unchanged.
10. **CI.** Replace `make` with Tauri's GitHub Action (`tauri-apps/tauri-action`) or `bun run tauri build` per runner; cache Cargo, the engine and the Chromium credits per OS and arch as today; run `actionlint`.
11. **Sizes.** Record every installer's size in `sizes.md`.

## Phase 8 as built

Phase 8 (2026-10-26, branch `feat/tauri`).

- **Staging.** `bun run stage` (`scripts/stage.ts`) fills the gitignored `resources/`: the video bundle, the sidecar compiled with `bun build --compile` into `resources/binaries/opsis-render-<target triple>`, the compositor package for the platform, the Export as code sources from `source-layout.generated.json`, and the engine installed by `install-engine` (`crates/engine/src/bin`) into `~/.cache/opsis-stage-engine` with its Chromium credits; the full edition copies the engine into `resources/engine`, the lite one only the credits into `resources/engine-licences`. `bun run make` stages and runs `tauri build --config tauri.<edition>.conf.json` (`OPSIS_EDITION`, default full). The bundle settings live only in the edition configs, so dev and e2e builds don't need a staged `resources/`.
- **Sidecar.** The compiled sidecar renders the stills and videos of `renders.rs` byte-identically to `bun run`, and is about the size of Bun plus a JS folder (91.7 MB against 89.0 MB of Bun and about 3 MB of JS) while being one file. `bundle.externalBin` installs it beside the app (`/usr/bin/opsis-render` in the deb); a packaged app starts it from there and passes the staged compositor as `binariesDirectory`.
- **Linux formats.** CI builds deb, rpm and AppImage; a local build makes the deb. The deb depends on `gstreamer1.0-libav`.
- **Icons.** `bun run app:icons` runs `tauri icon` on `brand/icons/1024.png` into `crates/desktop/icons` and keeps the desktop sizes.
- **Only the app is packed.** Tauri packs every binary target of `opsis-desktop`; the types exporter is behind the `export-bindings` feature so it isn't built or packed (it was 100 MB of the first deb). Release builds are stripped.
- **Checked.** The lite deb, extracted and started off-screen with a fresh data folder, opened the changelog through the Local API and exported it with the packed sidecar; the 1:1 export is byte-identical to the Electron app's. Sizes are in `sizes.md`.
- **CI.** `.github/workflows/build.yml`: checks on Ubuntu 22.04 with the webview's build packages; `bun run make` for both editions on Ubuntu 22.04 (deb, rpm, AppImage), Windows (NSIS), macOS arm64 and x64 (app, zipped), with the engine and credits cached; a signing check on macOS when the Apple secrets are set; an end-to-end job that installs the dev engine and runs `bun run test:e2e`. It passes `actionlint`; it hasn't run on GitHub yet, so Windows and macOS builds are untested.
- **Not done.** The updater (step 8) needs your signing key pair (`TAURI_SIGNING_PRIVATE_KEY`, and its public key in the config) and the release feed of `plans/shipping.md`, so it waits for the release work. Notices of the crates and npm packages compiled into the app aren't shipped yet: MIT and Apache-2.0 ask for them in binary distributions, so generate them before the first public release. Size work beyond stripping (LTO, `opt-level`, dropping the bundle's source maps, `public/` shipped twice).

## Decisions for the user

- **DMG.** A zip is enough for updates and notarization; add a DMG only if wanted for first installs.

## Risks

- **WebKitGTK versions.** The editor runs on whatever WebKitGTK the distribution ships; test on the oldest supported Ubuntu LTS.
- **Unsigned helpers on macOS.** Notarization fails on any unsigned executable inside the bundle; step 5's `codesign` check is the guard.

## Sources

- tauri-bundler `Cargo.toml` and `src/bundle/linux/rpm.rs`, tauri-utils `config.rs` (deb and rpm fields), from the `tauri-apps/tauri` repository (2026-10-25).
- [Tauri RPM](https://v2.tauri.app/distribute/rpm/), [Tauri Debian](https://v2.tauri.app/distribute/debian/), [Windows installer](https://v2.tauri.app/distribute/windows-installer/), [macOS signing](https://v2.tauri.app/distribute/sign/macos/), [config reference](https://v2.tauri.app/reference/config/), [icons](https://v2.tauri.app/develop/icons/), [updater](https://v2.tauri.app/plugin/updater/), [WebDriver](https://v2.tauri.app/develop/tests/webdriver/), [tauri-bundler 1.4.0 notes](https://v2.tauri.app/release/tauri-bundler/v1.4.0/) (2026-10-25).
- [Fedora `ffmpeg` `enable_decoders`](https://src.fedoraproject.org/rpms/ffmpeg/raw/rawhide/f/enable_decoders), [OpenH264 README](https://github.com/cisco/openh264) (decoder: Constrained Baseline up to level 5.2) (2026-10-25); the Fedora quick-docs page refused automated access.
- Test encode with the engine's FFmpeg and `ffprobe` (2026-10-25).
