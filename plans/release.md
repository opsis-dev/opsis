# Release

Everything still missing between today's app and a public release, and what comes after it: size work, builds and checks on Windows and macOS, signing, updates, the FFmpeg source offer, the release test pass, profiling, and node packs.

Status: plan only, except where a section says what is already in place.

## How to read this plan

- Sections 1–7 come before a first public release. Section 8 runs before every release. Sections 9 and 10 are later work.
- Each section has the same parts where they apply: **In place**, **Work** (numbered), **Decisions**, **Risks**. Rules from `AGENTS.md` hold throughout; when the two disagree, `AGENTS.md` wins.
- Local builds: deb only, `CARGO_BUILD_JOBS=8`, output in `~/.cache/opsis-build`. The rpm and AppImage, and the Windows and macOS packages, are built in CI only.

## Summary

| Topic                           | What users get                                                       | Needs from the user                       | Status                       |
| ------------------------------- | -------------------------------------------------------------------- | ----------------------------------------- | ---------------------------- |
| 1. Size work                    | A smaller download and install                                       | nothing                                   | not started                  |
| 2. Windows and macOS builds     | Installers that have actually run on each system                     | a push to start CI                        | configured, never run        |
| 3. Platform checks              | The editor working the same on WKWebView and WebView2                | nothing (CI runners)                      | not started                  |
| 4. Local API on Windows         | A named pipe only the user's account can open                        | nothing                                   | not started                  |
| 5. Code signing                 | No "unknown publisher" or "can't be opened" warnings                 | Apple account, a Windows signing route    | not started                  |
| 6. Updates                      | Check for updates, ask first, install signed updates                 | the signing key pair, CI secrets, answers | not started                  |
| 7. FFmpeg source offer          | The GPL's source obligation met for the shipped FFmpeg               | nothing (release assets)                  | in place, attach to releases |
| 8. Release test pass            | Fewer surprises: long takes, odd paths, every template, every system | nothing                                   | partly in place (e2e)        |
| 9. Profiling (development only) | Measured costs before optimizing                                     | nothing                                   | not started                  |
| 10. Node packs (long term)      | Nodes that install and update apart from the app                     | decisions when phase 3 starts             | not started                  |

## 1. Size work

**In place.** The deb is 99,605,920 bytes (95.0 MiB) and installs 225,424 KiB on Linux x64 (measured 2026-10-26, from 111.6 MB and 237,273 KiB the build before, and 122 MB and 292 MB before LTO): the app binary 64.4 MB (`opt-level = 3`, full LTO, one codegen unit, stripped), the compiled sidecar 91.3 MB (`--minify` for the host's Bun target; about 89 MB of it is the Bun runtime), the compositor 27.4 MB (about 25 MB of it FFmpeg), the video bundle 33.2 MB with no source maps and with `public/` (6.5 MB), which is the app's only copy of the `app:` files, the preview page 6.4 MB and the recording browser's licences 5.4 MB; that build also held 4.5 MB of sources for Export as code, which has since been removed. There is one build: the recording browser and our FFmpeg are never shipped. `public/` used to ship three times (on its own, embedded in the binary through the editor's build, and in the bundle); a packaged app now reads it from the bundle (`AppPaths::public_dir`), the editor loads `app:` files through the desktop file server, and the editor's build carries only `page-errors.js`. `--bytecode` would start the sidecar in 20 ms instead of 56 ms for 4 MB more; it's left out.

Built-in music (`public/music`, three 128 kbps AAC tracks) adds 7,923,531 bytes (7.6 MiB) to `public/`, so to the video bundle; it came after the measurement above.

**Work.**

1. **Measure.** `cargo bloat --release -p opsis-desktop --crates` for the binary and `source-map-explorer` on the editor and video bundles (neither tool is installed yet), so the next cuts have a before and after.
2. **Record** each installer's size per platform from the CI artifacts.

**Decided.** The notices of the crates and npm packages compiled into the app are not shipped: not worth the size.

## 2. Windows and macOS builds

**In place.** `.github/workflows/build.yml`: checks on Ubuntu 22.04; `bun run make` on Ubuntu 22.04 (deb, rpm, AppImage), `windows-latest` (NSIS, per-user install, and MSI) and `macos-latest` and `macos-26-intel` (the app, zipped, and a DMG); the browser that the Chromium credits are read from cached; the compositor's FFmpeg source collected on `v*` tags; artifacts uploaded; an end-to-end job. It passes `actionlint` and has never run. The repo's remote is `github.com/bismitpanda/opsis`.

**Work.**

1. **First run.** Push a branch and start the workflow with `workflow_dispatch`; expect every build job to need fixes at least once (`fail-fast: false` is set).
2. **Fix failures one system at a time.** Likely places: `install-engine` on Windows (Chrome for Testing download and unzip), the compositor package for `win32-x64-msvc` and `darwin-x64` in `scripts/stage.ts`, `bun build --compile` of the sidecar per platform, the MSI (WiX runs only on Windows and needs the VBSCRIPT feature), long paths on Windows (the downloaded engine sits several folders deep).
3. **AppImage.** Tauri packs WebKitGTK and its GStreamer plugins into it; check on a clean Ubuntu that it starts, plays an exported video in the Run panel and keeps the webview's sandbox working.
4. **Fedora.** The rpm can't fix H.264 playback: Fedora's `ffmpeg-free` has no H.264 decoder, and OpenH264 decodes Constrained Baseline only, while exports are High profile. The install notes say: enable RPM Fusion and swap `ffmpeg-free` for `ffmpeg`.
5. **Keep the builds** in `~/.cache/opsis-ci/<run id>/` when checking them; sizes go to section 1.

**Decided.** Public repo; CI runners only for Windows and macOS (no manual pass), plus a person on Linux; x64 Windows and separate Apple Silicon and Intel Mac builds, no Windows on Arm or universal build; one build with no engine inside; NSIS and MSI on Windows, the zipped app (for updates and notarization) and a DMG (for first installs) on macOS.

## 3. Platform checks

The editor has only run on WebKitGTK.

**Work.**

1. **WKWebView and WebView2.** In the CI build jobs, start the packaged app with a test data folder (`OPSIS_USER_DATA`, the API on) and drive it through the Local API: open each example, run the changelog, `ffprobe` the export. The e2e suite (WebdriverIO, embedded driver) also covers macOS through `@wdio/tauri-service`; run it on the macOS runners. List what a runner can't judge (how the window controls look, first-open prompts) as not covered in the release notes.
2. **macOS shell.** The native menu (`crates/desktop/src/native_menu.rs`) is built and its ids are tested, but it has never run: check the app menu, Edit roles, Cmd+, opens Settings, Cmd+Q goes through the exit dialog, and the traffic lights sit inside the app bar.
3. **Minimized editor.** The editor window disables background throttling, which only macOS 14+ honours. Check on WebKitGTK and WebView2 whether a minimized editor still answers a Local API edit and autosaves within the 20 s rule; if not, move the workflow and undo into Rust, so API edits never wait on a window.
4. **Preview sound drift.** The preview's sound plays in the webview, kept within two frames of the page's clock; measure drift over a long film on each system.
5. **Engine checks.** On each runner, download the engine through Settings' installer and run the checks on it (`check_browser`, `check_ffmpeg`, `check_ffprobe`): ffmpeg-static's Windows and macOS builds must have every required encoder, decoder, format and filter, and Chrome for Testing must open a page.
6. **Sign-in end to end.** Not in the e2e suite, because saving a sign-in writes to the real keyring. Add it with a keyring the test owns (a test-only store behind the `e2e` feature).

## 4. Local API on Windows

The named pipe is created with the default security descriptor, which grants read access to Everyone and the anonymous account.

**Work.**

1. Create the pipe for the current user only with tokio's `ServerOptions::create_with_security_attributes_raw` and a security descriptor built from the user's SID, in one small module with the only `unsafe` code, and a test that reads the pipe's ACL back.
2. In the Windows CI job, create a second local user and check it can't connect.

## 5. Code signing

**Work.**

1. **macOS.** `bundle.macOS.signingIdentity`, `hardenedRuntime` (set) and an entitlements file; CI signs with `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY` and notarizes with `APPLE_API_ISSUER`, `APPLE_API_KEY`, `APPLE_API_KEY_PATH` (the workflow passes them when the secrets exist and checks every Mach-O with `codesign -dv`). The sidecar and the compositor's programs must all be signed, or notarization fails; the recording browser and our FFmpeg are downloaded or chosen on the user's Mac, not shipped.
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

**Needs from the user.**

1. Run `bunx tauri signer generate -w ~/.tauri/opsis.key` (with a password if wanted) and keep the private key safe: if it's lost, installed copies can never update again.
2. Add `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` to the GitHub repository's secrets.
3. Send the public key; it goes into `plugins.updater.pubkey` and is safe to commit.
4. Confirm the feed lives on the releases of `bismitpanda/opsis`.
5. Choose: check on start with a Settings switch to turn it off, or only on the button.

**Work.** Settings → About gets the button and the switch (`updates` section in the settings); the release job gets `createUpdaterArtifacts` and the feed; the exit dialog lists a running download (`AppState.tasks`).

**Decided.** The feed address is the GitHub repo URL (GitHub forwards a renamed repo; never reuse the old name). deb: no apt repository, the release-page button.

## 7. FFmpeg source offer

**In place.** The only FFmpeg Opsis distributes is the one in Remotion's compositor: FFmpeg 7.1 built by Remotion with `--enable-gpl` (GPLv2 or later; Remotion's own page says so and points at `remotion-dev/rust-ffmpeg-splitter`). Our ffmpeg-static programs (GPLv3) are downloaded from that project's release on the user's request, or chosen, and never shipped. The stage writes `COPYING.GPLv2` and `SOURCES.md` next to the compositor, Settings → Licences lists both, and `THIRD_PARTY_NOTICES.md` names them. `licences/compositor-sources.json` pins the exact source, traced from Remotion 4.0.537's `packages/compositor/Cargo.toml` (`rust-ffmpeg` 0579107 → `rust-ffmpeg-sys` e7f864a, whose binaries came from `rust-ffmpeg-splitter` 50b6a0c): the splitter (scripts, the five FFmpeg patches and the bundled Opus 1.3.1, libvpx and LAME), FFmpeg 7.1, x264 at b35605a (the `stable` head when the binaries were built), both pinned x265 commits, dav1d 1.2.1, libaom 3.9.1, fdk-aac-free 2.0.0 and zimg 2.9.3. `bun run compositor:sources` downloads them with SHA256SUMS, CI's `compositor-sources` job does it on `v*` tags, and the stage and a test fail when the manifest names another Remotion version than the one installed. Decided: keep the GPL builds.

**Work.**

1. Attach the collected sources to each GitHub release, in the release job of section 6, so `SOURCES.md`'s link (`releases/tag/v<version>`) holds them.
2. When Remotion moves, trace the chain again (the compositor's `Cargo.toml` → `rust-ffmpeg` → `rust-ffmpeg-sys`'s last binaries commit, which names the splitter commit) and update the manifest; check that x264's `stable` head hasn't moved past the build date.

**Risk.** Patents are a separate question: H.264 and H.265 patent pools apply whatever the copyright licence. The `remotion` program in the compositor is under Remotion's own licence and links that FFmpeg; Remotion publishes no written offer of its own.

## 8. Release test pass

**In place.** `bun run test:e2e` (WebdriverIO, embedded driver, in a virtual display): the start screen, every example opening, the streamed preview, the element picker, a run and export of the changelog, playing the export, and the close flow. The packaged deb was checked off-screen through the Local API: it opened the changelog and exported it with the packed sidecar.

**Work, before each release** (Linux by hand, Windows and macOS through the runner checks in section 3):

1. **Long takes:** 10 and 20 minute recordings; memory of the app, the sidecar and the headless shell, the take's size, Timeline and Preview responsiveness, export time, trace size.
2. **Paths:** project folders with spaces and non-English letters, another drive on Windows, assets imported from such folders, fonts copied into projects.
3. **Moving and renaming** a project folder while closed and while open.
4. **Canvas sizes** beyond the presets: 21:9, 3:2, 60 fps, a very small and a 4K canvas.
5. **Broken font files** and ZIPs without fonts: a plain message, never a crash.
6. **Every template end to end,** comparing the Preview with the export.
7. **Stopping a run** at every stage, and killing the sidecar or the headless shell mid-run: the run reports it and the next one starts fresh.
8. **Pop-out windows** editing at once: edits and undo from each, closing one mid-drag.
9. **Keyboard only:** create, connect, edit, run and export without a mouse.
10. **Renders as recorded:** `crates/render/tests/renders.rs` (the same bytes) and `three_stills.rs` against their golden files, rerun on every Bun or Remotion bump.

## 9. Profiling (development only)

No code goes into the app; everything runs from outside it or as build settings.

1. **Per-component CPU and memory.** A dev script starts the app off-screen the way the e2e suite does (`xvfb-run`, Wayland unset, a seeded data folder with the API on), opens an example and runs it over the Local API, and samples every process in the app's tree once a second from `/proc`, tagged by command line: the app, WebKit's web and network processes, each headless shell (preview, recording, render tabs), the sidecar, FFmpeg. It writes a CSV and a chart. Node timings come from the run events the API streams.
2. **Where Rust spends time.** `samply record -p <pid>` or `perf record -g` on the app, on a release build with symbols kept through build settings (`CARGO_PROFILE_RELEASE_DEBUG=true`, `CARGO_PROFILE_RELEASE_STRIP=false`).
3. **Memory.** `heaptrack` on the app binary.
4. **The editor UI.** WebKitGTK's remote inspector through `WEBKIT_INSPECTOR_HTTP_SERVER=127.0.0.1:9222` (timelines, memory, layout); dev builds also open the inspector from the View menu.
5. **Builds and sizes.** `cargo build --timings`, `cargo bloat`, `source-map-explorer`, `dust`.

## 10. Node packs (long term)

Nodes that install and update apart from the app; build after sections 1–8.

- **Phase 1, no-code nodes.** "Save as reusable node" on a selection (nodes, inner edges, exposed ports and settings) into the data folder or the project; a `core.subgraph` node compiles by expanding its contents in `opsis-compile`. "Save project as template" and per-node presets.
- **Phase 2, packs as data.** A `pack.json` manifest (id, name, semver version, the pack API version, permissions, nodes and elements), first-party packs grouping today's nodes without changing their ids, projects listing the packs they use, and the "isn't available" messages naming the missing pack. Pack id and version join the node cache key.
- **Phase 3, pack code.** Node definitions and compile are Rust, so a pack can't ship TS compile code: settings come as JSON Schema, and compile and executors run as WebAssembly modules or as separate programs speaking the sidecar's JSON-lines protocol (decide when this phase starts). Video components ship prebuilt as ES modules importing `react` and `remotion` from globals the bundle exposes, loaded next to the registry's loaders under `delayRender`.
- **Phase 4, third-party packs.** Installed from a file (decided: no online catalogue at first); unsigned packs may run code after a clear warning naming what they can do, signed ones skip it (decided).
- **Risks.** Pack video code runs where the preview and renders run; version skew between a pack and the bundle's `remotion`/`react`; preview must still equal render.
