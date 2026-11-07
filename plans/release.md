# Release

Everything still missing between today's app and a public release, and what comes after it: size work, builds and checks on Windows and macOS, signing, updates, the release test pass, and node packs. Each section says first what is already in place, then what is left.

Status: the builds, signing and updates (sections 2, 5 and 6) were written for the Tauri app and are redone after the packaging step of [native-editor.md](native-editor.md); the rest is current for the native editor.

## How to read this plan

- Sections 1–6 come before a first public release. Section 7 runs before every release. Section 8 is later work.
- Each section has the same parts where they apply: **In place**, **Work** (numbered, only what is left), **Decided**, **Decision for the user**, **Risks**. Rules from `AGENTS.md` and `agent-docs/` hold throughout; when they disagree with this plan, they win.
- Local builds: `bun run make` builds the deb unless `--formats` names more (all four Linux formats build here, in the trixie container), `CARGO_BUILD_JOBS=8`, output in `~/.cache/opsis-build`, checked with `bun run package:check`. The Windows and macOS packages are built in CI only.

## Summary

| Topic                       | What users get                                                       | Left to do                                             | Status                           |
| --------------------------- | -------------------------------------------------------------------- | ------------------------------------------------------ | -------------------------------- |
| 1. Size work                | A smaller download and install                                       | act on the findings, record CI sizes                   | measured 2026-11-08              |
| 2. Windows and macOS builds | Installers that have actually run on each system                     | push, make the repo public, first CI run, fixes        | configured and linted, never run |
| 3. Platform checks          | The editor working the same on Windows and macOS                     | runner checks, the macOS shell, sign-in end to end     | not started                      |
| 4. Local API on Windows     | A named pipe only the user's account can open                        | the access list and its test                           | not started                      |
| 5. Code signing             | No "unknown publisher" or "can't be opened" warnings                 | identities, entitlements, CI signing                   | only the hardened runtime set    |
| 6. Updates                  | Check for updates, ask first, install signed updates                 | the updater plugin, the About button, the release feed | signing key and secrets in place |
| 7. Release test pass        | Fewer surprises: long takes, odd paths, every template, every system | the manual pass before each release                    | e2e in place (4 scenarios)       |
| 8. Node packs (long term)   | Nodes that install and update apart from the app                     | all of it, after sections 1–7                          | not started                      |

## 1. Size work

**In place.**

- **Size.** Measured 2026-11-08 on the native app's release build (`bun run make` on Linux x64, linked against the static Qt without ICU). The program is 101,795,856 bytes and needs no Qt files. The deb is 104,964,770 bytes and installs 224,746,225 bytes: the program, the FFmpeg program 79,826,272 and its licence 35,147, the film page 37,352,329, the terminal page 253,517, the browser's licences 5,345,493, `LICENSE`, `THIRD_PARTY_NOTICES.md`, the desktop entry and the icons. The rpm of the same files is 87,222,351 bytes: cargo-packager compresses the deb with gzip, `build_rpm` the rpm with zstd. Against the developer's shared Qt the program is 74,732,040 bytes but needs 71,802,192 bytes of Qt libraries, 39,084,760 of them ICU.
- **What ships.** One build, with FFmpeg inside and no recording browser: the app binary (`opt-level = 3`, full LTO, one codegen unit, stripped), FFmpeg as the `opsis-ffmpeg` program beside it with `LICENSE.ffmpeg.txt` (`plans/ffmpeg.md`), the film page (`film-bundle/`, one JavaScript file with no source maps and the repository's `public/`, the app's only copy of the `app:` files, which the app reads through `AppPaths::public_dir`), the terminal page (`terminal-bundle/`), the recording browser's licences, and Qt with KDDockWidgets linked into the program.
- **The film page.** 2,722,599 bytes of JavaScript in one file (760,496 gzipped) and 34,627,964 bytes of `public/` files: fonts 20,249,083 (the 16 families 1,078,938, the terminal stack 19,170,145, of which Noto Color Emoji is 10,730,124), music 7,923,531, backdrops 4,810,439, placeholders 1,292,763, sounds 352,148.

**Work.**

1. **Act on the findings.** Each is its own decision: turn off the `image` formats Opsis never reads; keep Qt Test out of the release binary, which links it for the scenario mode; whether every terminal take needs the 10.7 MB colour emoji font shipped rather than downloaded on first use; and a smaller deb, which cargo-packager always compresses with gzip.
2. **Record** each installer's size per platform from the CI artifacts.

**Decided.** The notices of the crates and npm packages compiled into the app are not shipped: not worth the size. Code copied into the repo and a few bundled components keep theirs (`THIRD_PARTY_NOTICES.md`).

## 2. Windows and macOS builds

**In place.**

- `.github/workflows/build.yml`: checks, the offscreen view tests and the `wayland` scenario on Ubuntu 22.04, the other e2e scenarios with the engine, the source notice's links on `v*` tags (`bun run notice:links`), and `workflow_dispatch`. The Windows and macOS build jobs went with the Tauri bundler and return with the packaging step.
- `actionlint` passes on the workflow (2026-10-26).
- The repository is `github.com/bismitpanda/opsis`. It is private and empty: nothing has been pushed, so it has no branches, workflows, runs or releases. The updater's signing secrets are already set on it (section 6).

**Work.**

1. **Push and make it public.** Push `main`, then switch the repository to public (decided below).
2. **First run.** Start the workflow with `workflow_dispatch`; expect every build job to need fixes at least once (`fail-fast: false` is set).
3. **Fix failures one system at a time.** Likely places: `install-engine` on Windows (Chrome for Testing download and unzip), begin-frame control on each platform's browser (the driver falls back to screenshots and says so), the MSI (WiX runs only on Windows and needs the VBSCRIPT feature), long paths on Windows (the downloaded engine sits several folders deep).
4. **AppImage.** Tauri packs WebKitGTK and its GStreamer plugins into it; check on a clean Ubuntu that it starts, plays an exported video in the Run panel and keeps the webview's sandbox working.
5. **Fedora.** The rpm can't fix H.264 playback: Fedora's `ffmpeg-free` has no H.264 decoder, and OpenH264 decodes Constrained Baseline only, while exports are High profile. The install notes say: enable RPM Fusion and swap `ffmpeg-free` for `ffmpeg`.
6. **Keep the builds** in `~/.cache/opsis-ci/<run id>/` when checking them; sizes go to section 1.

**Decided.** Public repo; CI runners only for Windows and macOS (no manual pass), plus a person on Linux; x64 Windows and separate Apple Silicon and Intel Mac builds, no Windows on Arm or universal build; one build with FFmpeg inside and no recording browser; NSIS and MSI on Windows, the zipped app (for updates and notarization) and a DMG (for first installs) on macOS.

## 3. Platform checks

The native editor has run only on Linux: X11 in a virtual display, and Wayland on headless sway.

**In place.** The document, undo, gestures and autosave live in Rust (`opsis-document`, `schedule_autosave`), and Local API edits apply whether or not an editor window is open, so a minimized or throttled editor can't hold up an API edit or a save.

**Work.**

1. **Windows and macOS.** On their CI runners, run `cargo test -p opsis-ui` (the `views` scenario on the offscreen platform), and start the packaged app with a test data folder (`OPSIS_USER_DATA`, the API on) and drive it through the Local API as the `api` scenario does: open each example, run the changelog, read the export back with `opsis-probe`. List what a runner can't judge (how the window controls look, first-open prompts) as not covered in the release notes.
2. **macOS shell.** The menu bar is the app's own, inside the window (`AppBar`), and the window has no system title bar, but neither has run on macOS: check the Cmd shortcuts, that Cmd+, opens Settings, that Cmd+Q goes through the exit dialog, moving and maximizing the window, and full screen.
3. **Timeline sound drift.** The Timeline's sound plays through the sound output (`opsis-audio`), and the playhead follows its clock while the film plays; measure drift over a long film on each system.
4. **Engine checks.** On each runner, download the recording browser through Settings' installer and run `check_browser` on it: Chrome for Testing must open a page. The bundled FFmpeg is already checked on every build runner (`check_ffmpeg` on the staged program, the CI step after the build).
5. **Sign-in end to end.** Not in the e2e scenarios, because saving a sign-in writes to the real keyring. `keyring-core`'s mock store is used only by the store's unit tests (`crates/store/src/keys.rs`); add a test-only store for scenario runs and a step in `runs` that signs in and records a take with it.

## 4. Local API on Windows

The named pipe is created with the default security descriptor, which grants read access to Everyone and the anonymous account.

**Work.**

1. Create the pipe for the current user only with tokio's `ServerOptions::create_with_security_attributes_raw` and a security descriptor built from the user's SID, in one small module with the only `unsafe` code, and a test that reads the pipe's ACL back.
2. In the Windows CI job, create a second local user and check it can't connect.

## 5. Code signing

**In place.** Nothing in the build: the Tauri bundler's hardened runtime and the workflow's Apple secrets and `codesign -dv` check went with it, and return with the packaging step.

**Work.**

1. **macOS.** `bundle.macOS.signingIdentity` and an entitlements file; CI signs with `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY` and notarizes with `APPLE_API_ISSUER`, `APPLE_API_KEY`, `APPLE_API_KEY_PATH`. Every program inside the app must be signed, or notarization fails: that includes the `opsis-ffmpeg` sidecar in `Contents/MacOS`, which is signed with the hardened runtime and notarized with the app. Whether ffmpeg-static runs under the hardened runtime without extra entitlements is untested. The recording browser is downloaded or chosen on the user's Mac, not shipped.
2. **Windows.** `bundle.windows.certificateThumbprint`, `digestAlgorithm` and `timestampUrl` for a certificate in the store, or `signCommand` for another tool (Azure Artifact Signing has a `signCommand` route). Sign every program inside the app.
3. **Check** on clean machines: the publisher name shows; macOS opens the downloaded zip with one confirmation.

**Decided.** The signer is the user's own name.

**Decision for the user.** Signing routes and costs, checked 2026-10-16: the Apple Developer Program is 99 USD a year and is needed for macOS users to open the app normally and for macOS updates. Windows: Azure Artifact Signing (about 10 USD a month, individuals only in the US or Canada), an OV certificate (about 220–440 USD a year, key on a token or cloud service), or not signing yet ("More info → Run anyway").

## 6. Updates

**How it works.** `tauri-plugin-updater`:

1. The app fetches a small JSON feed from GitHub Releases, only when the user clicks "Check for updates" or has switched on checking at start. Nothing goes online by default.
2. It shows "Version X is available" with the notes; nothing downloads until the user clicks.
3. `download()` fetches the update and checks its signature against the public key built into the app; `install()` installs it: the new installer on Windows, the app replaced on macOS, the AppImage replaced on Linux. deb and rpm can't update themselves, so they show a button that opens the release page.
4. CI signs each build with the private key and writes the feed (`version`, `notes`, `pub_date`, `platforms.<os-arch>.url` and `signature`) to a draft release on a `v*` tag; a person publishes it after section 7.

**In place** (2026-10-26).

- The key pair was made with `tauri signer generate`: the private key is `.tauri/opsis.key`, ignored by git (`/.tauri/opsis.key` in `.gitignore`), with its password in the ignored `.env` as `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`; the public key is committed as `.tauri/opsis.key.pub` (`4b0af6d`). A test signature with the values from `.env` succeeded. If both copies of the private key are lost, installed copies can never update again.
- Both values are set as secrets on `bismitpanda/opsis`.

**Work.**

1. Add `tauri-plugin-updater`, with the contents of `.tauri/opsis.key.pub` as `plugins.updater.pubkey` and the feed on the releases of `bismitpanda/opsis`.
2. Settings → About gets the button and the switch (an `updates` section in the settings); the exit dialog lists a running download (`AppState.tasks`).
3. The release job gets `createUpdaterArtifacts`, signs with the secrets and writes the feed to the draft release.

**Decision for the user.** Check on start with a Settings switch to turn it off, or only on the button.

**Decided.** The feed address is the GitHub repo URL (GitHub forwards a renamed repo; never reuse the old name). deb: no apt repository, the release-page button.

## 7. Release test pass

**In place.** `bun run e2e:qt` runs four scenarios in the real app: `tour` (every panel, field and window), `runs` (setup, recording, export, Replay, the element picker, terminal takes recorded, played, taped live and imported from a cast), `api` (every Local API route from outside, ending with the close check) and `wayland` (real input on headless sway); `cargo test -p opsis-ui` runs the `views` scenario offscreen. They carry the nine WebdriverIO spec files the Tauri app had. The Tauri app's deb was checked off-screen through the Local API on 2026-10-31: from empty caches it recorded and exported a film with footage, a grade and Redact, and exported the Changelog template in both shapes.

**Work, before each release** (Linux by hand, Windows and macOS through the runner checks in section 3):

1. **Every e2e scenario** (`bun run e2e:qt`) and the offscreen view tests.
2. **Long takes:** 10 and 20 minute recordings; memory of the app and the headless shell, the take's size, Timeline responsiveness, export time, trace size.
3. **Paths:** project folders with spaces and non-English letters, another drive on Windows, assets imported from such folders, fonts copied into projects.
4. **Moving and renaming** a project folder while closed and while open.
5. **Canvas sizes** beyond the presets: 21:9, 3:2, 60 fps, a very small and a 4K canvas.
6. **Broken font files** and ZIPs without fonts: a plain message, never a crash.
7. **Every template end to end,** run and exported, each export watched through.
8. **Stopping a run** at every stage, and killing the headless shell or FFmpeg mid-run: the run reports it and the next one starts fresh.
9. **Pop-out windows** editing at once: edits and undo from each, closing one mid-drag.
10. **Keyboard only:** create, connect, edit, run and export without a mouse.
11. **Renders as recorded:** `crates/render/tests/frames.rs` (the same bytes) and `three_stills.rs` against their golden files, rerun on every browser or FFmpeg bump.

## 8. Node packs (long term)

Nodes that install and update apart from the app; build after sections 1–7. The phases, the WebAssembly contract and the work are in [node-packs.md](node-packs.md).
