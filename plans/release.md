# Release

Everything still missing between today's app and a public release, and what comes after it: size work, builds and checks on Windows and macOS, signing, updates, the FFmpeg source offer, the release test pass, profiling, and node packs. Each section says first what is already in place, then what is left.

## How to read this plan

- Sections 1–7 come before a first public release. Section 8 runs before every release. Sections 9 and 10 are later work.
- Each section has the same parts where they apply: **In place**, **Work** (numbered, only what is left), **Decided**, **Decision for the user**, **Risks**. Rules from `AGENTS.md` hold throughout; when the two disagree, `AGENTS.md` wins.
- Local builds: deb only, `CARGO_BUILD_JOBS=8`, output in `~/.cache/opsis-build`. The rpm and AppImage, and the Windows and macOS packages, are built in CI only.

## Summary

| Topic                           | What users get                                                       | Left to do                                             | Status                                  |
| ------------------------------- | -------------------------------------------------------------------- | ------------------------------------------------------ | --------------------------------------- |
| 1. Size work                    | A smaller download and install                                       | measure again, act on the findings, record CI sizes    | measured; nothing cut since             |
| 2. Windows and macOS builds     | Installers that have actually run on each system                     | push, make the repo public, first CI run, fixes        | configured and linted, never run        |
| 3. Platform checks              | The editor working the same on WKWebView and WebView2                | runner checks, the macOS shell, sign-in end to end     | not started                             |
| 4. Local API on Windows         | A named pipe only the user's account can open                        | the access list and its test                           | not started                             |
| 5. Code signing                 | No "unknown publisher" or "can't be opened" warnings                 | identities, entitlements, CI signing                   | only the hardened runtime set           |
| 6. Updates                      | Check for updates, ask first, install signed updates                 | the updater plugin, the About button, the release feed | signing key and secrets in place        |
| 7. FFmpeg source offer          | The GPL's source obligation met for the shipped FFmpeg               | attach the sources to each release                     | in place                                |
| 8. Release test pass            | Fewer surprises: long takes, odd paths, every template, every system | the manual pass before each release                    | e2e in place (6 spec files)             |
| 9. Profiling (development only) | Measured costs before optimizing                                     | the process sampler script                             | size tools in place; no runtime tooling |
| 10. Node packs (long term)      | Nodes that install and update apart from the app                     | all of it, after sections 1–8                          | not started                             |

## 1. Size work

**In place.**

- **Package size.** Measured 2026-10-27 with `bun run make -- --bundles deb`: the deb is 103,769,842 bytes and installs 218,670 KiB on Linux x64. Against the 2026-10-26 build (99,605,920 bytes, 225,424 KiB) the install is 6,754 KiB smaller, because Export as code and its 4.5 MB of sources are gone, and the deb is 4.16 MB larger, because of the built-in music (`public/music`, three 128 kbps AAC tracks, 7,923,531 bytes) and a larger binary.
- **What ships.** One build, with no recording browser and no FFmpeg of ours inside: the app binary (`opt-level = 3`, full LTO, one codegen unit, stripped), the compiled sidecar (`--minify`; about 89 MB of it is the Bun runtime), the compositor (about 25 MB of it FFmpeg), the video bundle with no source maps and with `public/`, the app's only copy of the `app:` files (a packaged app reads them through `AppPaths::public_dir`, and the editor's build carries only `page-errors.js`), the preview page and the recording browser's licences. `--bytecode` would start the sidecar in 20 ms instead of 56 ms for 4 MB more; it's left out.
- **Tools.** `cargo-bloat` is installed and `source-map-explorer` is a dev dependency (`a7f7e13`); both ran on 2026-10-26.
- **The binary by crate** (`cargo bloat --release -p opsis-desktop --crates`, stripping off for the measurement): 72.4 MiB unstripped, 45.8 MiB of it code. The largest are std 4.3 MiB, unattributed 3.9 MiB, opsis_compile 2.9 MiB, serde_core 2.8 MiB, jsonschema 1.9 MiB, tokio, rav1e (the AV1 encoder, used only by Image format's AVIF) and tauri at 1.5 MiB each, aws_lc_sys 1.3 MiB, and zbus 1.1 MiB. `exr` (251 KiB) and `image_webp` (218 KiB) come with the `image` crate's default formats.
- **The video bundle** (a development build with maps, 150 chunks, 25.6 MB of mapped JavaScript): `@remotion/studio` 7.17 MB (28%), which renders never show but Remotion's bundler puts in every bundle; the registry elements 3.39 MB, one file of which (`elements/scene/launch-anything/assets.ts`, then `elements/remocn/templates/launch-anything/assets.ts`) is 1.7 MB; mediabunny 3.38 MB; three 2.48 MB; zod 1.32 MB; `@mediabunny/aac-encoder` 1.00 MB.
- **The editor** (53 chunks, 3.58 MB of mapped JavaScript): elkjs 1.43 MB (40%, the engine behind Tidy up layout), the editor's own code 0.39 MB, remotion 0.25 MB (pulled in through an `@opsis/video` import, though the editor renders no video), `@base-ui/react` 0.22 MB.

**Work.**

1. **Measure again.** The numbers above predate Konva and react-konva in the editor, and nnnoiseless (with its trained weights), fundsp, ab_glyph and wuff in the binary.
2. **Act on the findings.** Each is its own decision: keep Remotion Studio out of the shipped bundle, find what pulls `remotion` into the editor, load elkjs only when Tidy up layout runs, shrink the 1.7 MB `assets.ts`, and turn off the `image` formats Opsis never reads.
3. **Record** each installer's size per platform from the CI artifacts.

**Decided.** The notices of the crates and npm packages compiled into the app are not shipped: not worth the size. Code copied into the repo and a few bundled components keep theirs (`THIRD_PARTY_NOTICES.md`).

## 2. Windows and macOS builds

**In place.**

- `.github/workflows/build.yml`: checks on Ubuntu 22.04; `bun run make` on Ubuntu 22.04 (deb, rpm, AppImage), `windows-latest` (NSIS, per-user install, and MSI) and `macos-latest` and `macos-26-intel` (the app, zipped, and a DMG); the browser that the Chromium credits are read from, cached; the compositor's FFmpeg source collected on `v*` tags; artifacts uploaded; an end-to-end job; `workflow_dispatch`. The release config (`tauri.release.conf.json`) has the same targets.
- `actionlint` passes on the workflow (2026-10-26).
- The repository is `github.com/bismitpanda/opsis`. It is private and empty: nothing has been pushed, so it has no branches, workflows, runs or releases. The updater's signing secrets are already set on it (section 6).

**Work.**

1. **Push and make it public.** Push `main`, then switch the repository to public (decided below).
2. **First run.** Start the workflow with `workflow_dispatch`; expect every build job to need fixes at least once (`fail-fast: false` is set).
3. **Fix failures one system at a time.** Likely places: `install-engine` on Windows (Chrome for Testing download and unzip), the compositor package for `win32-x64-msvc` and `darwin-x64` in `scripts/stage.ts`, `bun build --compile` of the sidecar per platform, the MSI (WiX runs only on Windows and needs the VBSCRIPT feature), long paths on Windows (the downloaded engine sits several folders deep).
4. **AppImage.** Tauri packs WebKitGTK and its GStreamer plugins into it; check on a clean Ubuntu that it starts, plays an exported video in the Run panel and keeps the webview's sandbox working.
5. **Fedora.** The rpm can't fix H.264 playback: Fedora's `ffmpeg-free` has no H.264 decoder, and OpenH264 decodes Constrained Baseline only, while exports are High profile. The install notes say: enable RPM Fusion and swap `ffmpeg-free` for `ffmpeg`.
6. **Keep the builds** in `~/.cache/opsis-ci/<run id>/` when checking them; sizes go to section 1.

**Decided.** Public repo; CI runners only for Windows and macOS (no manual pass), plus a person on Linux; x64 Windows and separate Apple Silicon and Intel Mac builds, no Windows on Arm or universal build; one build with no engine inside; NSIS and MSI on Windows, the zipped app (for updates and notarization) and a DMG (for first installs) on macOS.

## 3. Platform checks

The editor has only run on WebKitGTK.

**In place.** The document, undo, gestures and autosave live in Rust (`opsis-document`, `schedule_autosave`), and Local API edits apply whether or not an editor window is open, so a minimized or throttled editor can't hold up an API edit or a save.

**Work.**

1. **WKWebView and WebView2.** In the CI build jobs, start the packaged app with a test data folder (`OPSIS_USER_DATA`, the API on) and drive it through the Local API: open each example, run the changelog, `ffprobe` the export. The e2e suite (WebdriverIO, embedded driver) also covers macOS through `@wdio/tauri-service`; run it on the macOS runners. List what a runner can't judge (how the window controls look, first-open prompts) as not covered in the release notes.
2. **macOS shell.** The native menu (`crates/desktop/src/native_menu.rs`) is built and its ids are tested, but it has never run: check the app menu, Edit roles, Cmd+, opens Settings, Cmd+Q goes through the exit dialog, and the traffic lights sit inside the app bar.
3. **Preview sound drift.** The preview's sound plays in the webview, kept within two frames of the page's clock; measure drift over a long film on each system.
4. **Engine checks.** On each runner, download the engine through Settings' installer and run the checks on it (`check_browser`, `check_ffmpeg`, `check_ffprobe`): ffmpeg-static's Windows and macOS builds must have every required encoder, decoder, format and filter, and Chrome for Testing must open a page.
5. **Sign-in end to end.** Not in the e2e suite, because saving a sign-in writes to the real keyring. `keyring-core`'s mock store is used only by the store's unit tests (`crates/store/src/keys.rs`); add a test-only store behind the `e2e` feature and a spec that signs in and records a take with it.

## 4. Local API on Windows

The named pipe is created with the default security descriptor, which grants read access to Everyone and the anonymous account.

**Work.**

1. Create the pipe for the current user only with tokio's `ServerOptions::create_with_security_attributes_raw` and a security descriptor built from the user's SID, in one small module with the only `unsafe` code, and a test that reads the pipe's ACL back.
2. In the Windows CI job, create a second local user and check it can't connect.

## 5. Code signing

**In place.** `hardenedRuntime` is set for macOS in `tauri.release.conf.json`. The workflow passes the Apple secrets when they exist and checks every Mach-O with `codesign -dv`.

**Work.**

1. **macOS.** `bundle.macOS.signingIdentity` and an entitlements file; CI signs with `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY` and notarizes with `APPLE_API_ISSUER`, `APPLE_API_KEY`, `APPLE_API_KEY_PATH`. The sidecar and the compositor's programs must all be signed, or notarization fails; the recording browser and our FFmpeg are downloaded or chosen on the user's Mac, not shipped.
2. **Windows.** `bundle.windows.certificateThumbprint`, `digestAlgorithm` and `timestampUrl` for a certificate in the store, or `signCommand` for another tool (Azure Artifact Signing has a `signCommand` route). Sign the sidecar and the compositor's programs too.
3. **Check** on clean machines: the publisher name shows; macOS opens the downloaded zip with one confirmation.

**Decided.** The signer is the user's own name.

**Decision for the user.** Signing routes and costs, checked 2026-10-16: the Apple Developer Program is 99 USD a year and is needed for macOS users to open the app normally and for macOS updates. Windows: Azure Artifact Signing (about 10 USD a month, individuals only in the US or Canada), an OV certificate (about 220–440 USD a year, key on a token or cloud service), or not signing yet ("More info → Run anyway").

## 6. Updates

**How it works.** `tauri-plugin-updater`:

1. The app fetches a small JSON feed from GitHub Releases, only when the user clicks "Check for updates" or has switched on checking at start. Nothing goes online by default.
2. It shows "Version X is available" with the notes; nothing downloads until the user clicks.
3. `download()` fetches the update and checks its signature against the public key built into the app; `install()` installs it: the new installer on Windows, the app replaced on macOS, the AppImage replaced on Linux. deb and rpm can't update themselves, so they show a button that opens the release page.
4. CI signs each build with the private key and writes the feed (`version`, `notes`, `pub_date`, `platforms.<os-arch>.url` and `signature`) to a draft release on a `v*` tag; a person publishes it after section 8.

**In place** (2026-10-26).

- The key pair was made with `tauri signer generate`: the private key is `.tauri/opsis.key`, ignored by git (`/.tauri/opsis.key` in `.gitignore`), with its password in the ignored `.env` as `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`; the public key is committed as `.tauri/opsis.key.pub` (`4b0af6d`). A test signature with the values from `.env` succeeded. If both copies of the private key are lost, installed copies can never update again.
- Both values are set as secrets on `bismitpanda/opsis`.

**Work.**

1. Add `tauri-plugin-updater`, with the contents of `.tauri/opsis.key.pub` as `plugins.updater.pubkey` and the feed on the releases of `bismitpanda/opsis`.
2. Settings → About gets the button and the switch (an `updates` section in the settings); the exit dialog lists a running download (`AppState.tasks`).
3. The release job gets `createUpdaterArtifacts`, signs with the secrets and writes the feed to the draft release.

**Decision for the user.** Check on start with a Settings switch to turn it off, or only on the button.

**Decided.** The feed address is the GitHub repo URL (GitHub forwards a renamed repo; never reuse the old name). deb: no apt repository, the release-page button.

## 7. FFmpeg source offer

**In place.** The only FFmpeg Opsis distributes is the one in Remotion's compositor: FFmpeg 7.1 built by Remotion with `--enable-gpl` (GPLv2 or later; Remotion's own page says so and points at `remotion-dev/rust-ffmpeg-splitter`). Our ffmpeg-static programs (GPLv3) are downloaded from that project's release on the user's request, or chosen, and never shipped. The stage writes `COPYING.GPLv2` and `SOURCES.md` next to the compositor, Settings → Licences lists both, and `THIRD_PARTY_NOTICES.md` names them. `licences/compositor-sources.json` pins the exact source, traced from Remotion 4.0.537's `packages/compositor/Cargo.toml` (`rust-ffmpeg` 0579107 → `rust-ffmpeg-sys` e7f864a, whose binaries came from `rust-ffmpeg-splitter` 50b6a0c): the splitter (scripts, the five FFmpeg patches and the bundled Opus 1.3.1, libvpx and LAME), FFmpeg 7.1, x264 at b35605a (the `stable` head when the binaries were built), both pinned x265 commits, dav1d 1.2.1, libaom 3.9.1, fdk-aac-free 2.0.0 and zimg 2.9.3. `bun run compositor:sources` downloads them with SHA256SUMS, CI's `compositor-sources` job does it on `v*` tags, and the stage and a test fail when the manifest names another Remotion version than the one installed (both are 4.0.537 today). Decided: keep the GPL builds.

**Work.**

1. Attach the collected sources to each GitHub release, in the release job of section 6, so `SOURCES.md`'s link (`releases/tag/v<version>`) holds them.
2. When Remotion moves, trace the chain again (the compositor's `Cargo.toml` → `rust-ffmpeg` → `rust-ffmpeg-sys`'s last binaries commit, which names the splitter commit) and update the manifest; check that x264's `stable` head hasn't moved past the build date.

## 8. Release test pass

**In place.** `bun run test:e2e` (WebdriverIO, embedded driver, in a virtual display) has six spec files: the start screen (`start`), every example opening (`examples`), the streamed preview and the element picker (`views`), the Timeline (`timeline`), a run and export of the changelog with the export played back (`export`), and drawing a mark in Annotate's marks editor (`annotate`), then the close check. The whole suite passed on 2026-10-27 with the first five; `annotate` was added and passed on its own on 2026-10-28. The packaged deb was checked off-screen through the Local API: it opened the changelog and exported it with the packed sidecar.

**Work, before each release** (Linux by hand, Windows and macOS through the runner checks in section 3):

1. **The whole e2e suite**, all six spec files together.
2. **Long takes:** 10 and 20 minute recordings; memory of the app, the sidecar and the headless shell, the take's size, Timeline and Preview responsiveness, export time, trace size.
3. **Paths:** project folders with spaces and non-English letters, another drive on Windows, assets imported from such folders, fonts copied into projects.
4. **Moving and renaming** a project folder while closed and while open.
5. **Canvas sizes** beyond the presets: 21:9, 3:2, 60 fps, a very small and a 4K canvas.
6. **Broken font files** and ZIPs without fonts: a plain message, never a crash.
7. **Every template end to end,** comparing the Preview with the export.
8. **Stopping a run** at every stage, and killing the sidecar or the headless shell mid-run: the run reports it and the next one starts fresh.
9. **Pop-out windows** editing at once: edits and undo from each, closing one mid-drag.
10. **Keyboard only:** create, connect, edit, run and export without a mouse.
11. **Renders as recorded:** `crates/render/tests/renders.rs` (the same bytes) and `three_stills.rs` against their golden files, rerun on every Bun or Remotion bump.

## 9. Profiling (development only)

No code goes into the app; everything runs from outside it or as build settings.

**In place.** The size tools of section 1 (`cargo-bloat`, `source-map-explorer`) and `cargo build --timings`.

**Work.**

1. **Per-component CPU and memory.** A dev script starts the app off-screen the way the e2e suite does (`xvfb-run`, Wayland unset, a seeded data folder with the API on), opens an example and runs it over the Local API, and samples every process in the app's tree once a second from `/proc`, tagged by command line: the app, WebKit's web and network processes, each headless shell (preview, recording, render tabs), the sidecar, FFmpeg. It writes a CSV and a chart. Node timings come from the run events the API streams.
2. **Where Rust spends time.** `samply record -p <pid>` or `perf record -g` on the app, on a release build with symbols kept through build settings (`CARGO_PROFILE_RELEASE_DEBUG=true`, `CARGO_PROFILE_RELEASE_STRIP=false`).
3. **Memory.** `heaptrack` on the app binary.
4. **The editor UI.** WebKitGTK's remote inspector through `WEBKIT_INSPECTOR_HTTP_SERVER=127.0.0.1:9222` (timelines, memory, layout); dev builds also open the inspector from the View menu.

## 10. Node packs (long term)

Nodes that install and update apart from the app; build after sections 1–8.

- **Phase 1, no-code nodes.** "Save as reusable node" on a selection (nodes, inner edges, exposed ports and settings) into the data folder or the project; a `core.subgraph` node compiles by expanding its contents in `opsis-compile`. "Save project as template" and per-node presets.
- **Phase 2, packs as data.** A `pack.json` manifest (id, name, semver version, the pack API version, permissions, nodes and elements), first-party packs grouping today's nodes without changing their ids, projects listing the packs they use, and the "isn't available" messages naming the missing pack. Pack id and version join the node cache key.
- **Phase 3, pack code.** Node definitions and compile are Rust, so a pack can't ship TS compile code: settings come as JSON Schema, and compile and executors run as WebAssembly modules or as separate programs speaking the sidecar's JSON-lines protocol (decide when this phase starts). Video components ship prebuilt as ES modules importing `react` and `remotion` from globals the bundle exposes, loaded next to the registry's loaders under `delayRender`.
- **Phase 4, third-party packs.** Installed from a file (decided: no online catalogue at first); unsigned packs may run code after a clear warning naming what they can do, signed ones skip it (decided).
- **Risks.** Pack video code runs where the preview and renders run; version skew between a pack and the bundle's `remotion`/`react`; preview must still equal render.
