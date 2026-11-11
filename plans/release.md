# Release

Everything still missing between today's app and a public release, and what comes after it: size work, builds and checks on Windows and macOS, signing, updates, the release test pass, and node packs. Each section says first what is already in place, then what is left.

Status: the Linux packages are built and checked ([packaging.md](../agent-docs/packaging.md)); the Windows and macOS builds, signing and updates (sections 2, 5 and 6) build on `opsis-packager`; the rest is current for the native editor.

## How to read this plan

- Sections 1–6 come before a first public release. Section 7 runs before every release. Section 8 is later work.
- Each section has the same parts where they apply: **In place**, **Work** (numbered, only what is left), **Decided**, **Decision for the user**, **Risks**. Rules from `AGENTS.md` and `agent-docs/` hold throughout; when they disagree with this plan, they win.
- Local builds: `bun run make` builds the deb unless `--formats` names more (all four Linux formats build here, in the trixie container), cargo capped at 8 jobs unless `CARGO_BUILD_JOBS` says otherwise, output in `~/.cache/opsis-dev/build`, checked with `bun run package:check`. The Windows and macOS packages are built in CI only.

## Summary

| Topic                       | What users get                                                       | Left to do                                         | Status                           |
| --------------------------- | -------------------------------------------------------------------- | -------------------------------------------------- | -------------------------------- |
| 1. Size work                | A smaller download and install                                       | act on the findings, record CI sizes               | measured 2026-11-08              |
| 2. Windows and macOS builds | Installers that have actually run on each system                     | push, first CI run, fixes                          | Linux packages built and checked |
| 3. Platform checks          | The editor working the same on Windows and macOS                     | runner checks, the macOS shell, sign-in end to end | not started                      |
| 4. Local API on Windows     | A named pipe only the user's account can open                        | the access list and its test                       | not started                      |
| 5. Code signing             | No "unknown publisher" or "can't be opened" warnings                 | identities, entitlements, CI signing               | not started                      |
| 6. Updates                  | Check for updates, ask first, install signed updates                 | check a real update on each system                 | built, not yet run               |
| 7. Release test pass        | Fewer surprises: long takes, odd paths, every template, every system | the manual pass before each release                | e2e in place (4 scenarios)       |
| 8. Node packs (long term)   | Nodes that install and update apart from the app                     | all of it, after sections 1–7                      | not started                      |

## 1. Size work

**In place.**

- **Size.** Measured 2026-11-08 on the native app's release build (`bun run make` on Linux x64, linked against the static Qt without ICU). The program is 101,795,856 bytes and needs no Qt files. Built with each format's own tool (same program and files): the deb (`dpkg-deb -Zzstd`) is 101,233,244 bytes, the rpm (`rpmbuild`, zstd) 87,346,674, the pacman package (`makepkg`, zstd) 101,293,373 and the AppImage 100,567,544. A Linux package installs 224,746,225 bytes: the program, the FFmpeg program 79,826,272 and its licence 35,147, the film page 37,352,329, the terminal page 253,517, the browser's licences 5,345,493, `LICENSE`, `THIRD_PARTY_NOTICES.md`, the desktop entry and the icons. Against the developer's shared Qt the program is 74,732,040 bytes but needs 71,802,192 bytes of Qt libraries, 39,084,760 of them ICU.
- **What ships.** One build, with FFmpeg inside and no recording browser: the app binary (`opt-level = 3`, full LTO, one codegen unit, stripped), FFmpeg as the `opsis-ffmpeg` program beside it with `LICENSE.ffmpeg.txt` (`plans/ffmpeg.md`), the film page (`film-bundle/`, one JavaScript file with no source maps and the repository's `public/`, the app's only copy of the `app:` files, which the app reads through `AppPaths::public_dir`), the terminal page (`terminal-bundle/`), the recording browser's licences, and Qt with KDDockWidgets linked into the program.
- **The film page.** 2,722,599 bytes of JavaScript in one file (760,496 gzipped) and 34,627,964 bytes of `public/` files: fonts 20,249,083 (the 16 families 1,078,938, the terminal stack 19,170,145, of which Noto Color Emoji is 10,730,124), music 7,923,531, backdrops 4,810,439, placeholders 1,292,763, sounds 352,148.

**Work.**

1. **Act on the findings.** Each is its own decision: turn off the `image` formats Opsis never reads; keep Qt Test out of the release binary, which links it for the scenario mode; whether every terminal take needs the 10.7 MB colour emoji font shipped rather than downloaded on first use; and building Qt with `-no-feature-timezone_locale`, which leaves out the table of time-zone names Qt compiles in without ICU (3,990,504 bytes in the spike).
2. **Record** each installer's size per platform from the CI artifacts.

**Decided.** The notices of the crates and npm packages compiled into the app are not shipped: not worth the size. Code copied into the repo and a few bundled components keep theirs (`THIRD_PARTY_NOTICES.md`).

## 2. Windows and macOS builds

**In place.**

- `.github/workflows/build.yml`, on every push, on Ubuntu 24.04: the checks (the offscreen view tests among them) and the `wayland` scenario, and the other e2e scenarios with the engine.
- `.github/workflows/release.yml`, on `v*` tags or by hand (`workflow_dispatch`): the source notice's links (`bun run notice:links`), a package job for each system (Linux in the trixie container, Windows with the pinned NSIS and WiX v7, both Macs) and, on tags, the draft release with the update feed ([packaging.md](../agent-docs/packaging.md)). None has run. On macOS the job installs Ninja when the runner lacks it; on Windows `qt:static` loads Visual Studio's environment itself.
- The repository is `github.com/opsis-dev/opsis`. It is private; its `main` is behind the local one, its one workflow run (Build, 2026-11-05) failed, and it has no releases. The update signing secrets are set on it (section 6).

**Work.**

1. **Push and make it public.** Push `main`, then switch the repository to public (decided below).
2. **First run.** Start the workflow with `workflow_dispatch`; expect every build job to need fixes at least once (`fail-fast: false` is set).
3. **Fix failures one system at a time.** Likely places: the static Qt build and the editor's static link on Windows and macOS (written from Qt's docs and the macOS `.prl` files, never run there: the `.prl` text on MSVC and the Windows plugin's system libraries are the least certain), `bun`'s `windowsVerbatimArguments` in `msvcEnv`, `install-engine` on Windows (Chrome for Testing download and unzip), begin-frame control on each platform's browser (the driver falls back to screenshots and says so), the MSI (WiX runs only on Windows and needs the VBSCRIPT feature), long paths on Windows (the downloaded engine sits several folders deep).
4. **Keep the builds** in `~/.cache/opsis-ci/<run id>/` when checking them; sizes go to section 1.

**Decided.** Public repo; CI runners only for Windows and macOS (no manual pass), plus a person on Linux; x64 Windows and separate Apple Silicon and Intel Mac builds, no Windows on Arm or universal build; one build with FFmpeg inside and no recording browser; NSIS and MSI on Windows, the zipped app (for updates and notarization) and a DMG (for first installs) on macOS.

## 3. Platform checks

The native editor has run only on Linux: X11 in a virtual display, and Wayland on headless sway.

**In place.** The document, undo, gestures and autosave live in Rust (`opsis-document`, `schedule_autosave`), and Local API edits apply whether or not an editor window is open, so a minimized or throttled editor can't hold up an API edit or a save.

**Work.**

1. **Windows and macOS.** On their CI runners, run `cargo test -p opsis-ui` (the `views` scenario on the offscreen platform), and start the packaged app with a test data folder (`OPSIS_USER_DATA`, the API on) and drive it through the Local API as the `api` scenario does: open each example, run the changelog, read the export back with `opsis-probe`. List what a runner can't judge (how the window controls look, first-open prompts) as not covered in the release notes.
2. **macOS shell.** The menu bar is the app's own, inside the window (`AppBar`), and the window has no system title bar, but neither has run on macOS: check the Cmd shortcuts, that Cmd+, opens Settings, that Cmd+Q goes through the exit dialog, moving and maximizing the window, and full screen.
3. **Timeline sound drift.** The Timeline's sound plays through the sound output (`opsis-audio`), and the playhead follows its clock while the film plays; measure drift over a long film on each system.
4. **Engine checks.** On each runner, download the recording browser through Settings' installer and run `check_browser` on it: Chrome for Testing must open a page. The release workflow checks the bundled FFmpeg on every package runner (`check_ffmpeg` on the staged program, the step after the build), but it has not run on Windows or macOS yet.
5. **Sign-in end to end.** Not in the e2e scenarios, because saving a sign-in writes to the real keyring. `keyring-core`'s mock store is used only by the store's unit tests (`crates/store/src/keys.rs`); add a test-only store for scenario runs and a step in `runs` that signs in and records a take with it.

## 4. Local API on Windows

The named pipe is created with the default security descriptor, which grants read access to Everyone and the anonymous account.

**Work.**

1. Create the pipe for the current user only with tokio's `ServerOptions::create_with_security_attributes_raw` and a security descriptor built from the user's SID, in one small module with the only `unsafe` code, and a test that reads the pipe's ACL back.
2. In the Windows CI job, create a second local user and check it can't connect.

## 5. Code signing

**In place.** `opsis-packager` signs the macOS app ad hoc with `codesign` (`crates/packager/src/macos.rs`); nothing else is code signed.

**Work.**

1. **macOS.** `codesign` with the Developer ID identity, the hardened runtime (`--options runtime`) and entitlements, then `notarytool submit --wait` and `stapler staple`, in `crates/packager/src/macos.rs`; CI signs with `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY` and notarizes with `APPLE_API_ISSUER`, `APPLE_API_KEY`, `APPLE_API_KEY_PATH`. Every program inside the app must be signed, or notarization fails: that includes the `opsis-ffmpeg` sidecar in `Contents/MacOS`, which is signed with the hardened runtime and notarized with the app. Whether ffmpeg-static runs under the hardened runtime without extra entitlements is untested. The recording browser is downloaded or chosen on the user's Mac, not shipped.
2. **Windows.** `signtool` (Windows SDK) with the certificate, SHA-256 and a timestamp server, or Azure Artifact Signing's own tool, on the program, FFmpeg, the NSIS installer and its uninstaller, and the MSI, in `crates/packager/src/windows.rs`.
3. **Check** on clean machines: the publisher name shows; macOS opens the downloaded zip with one confirmation.

**Decided.** The signer is the user's own name.

**Decision for the user.** Signing routes and costs, checked 2026-10-16: the Apple Developer Program is 99 USD a year and is needed for macOS users to open the app normally and for macOS updates. Windows: Azure Artifact Signing (about 10 USD a month, individuals only in the US or Canada), an OV certificate (about 220–440 USD a year, key on a token or cloud service), or not signing yet ("More info → Run anyway").

## 6. Updates

**In place.** The updater, the About page's Updates card, the installer marker, update signing and the draft release with its feed are built ([updates.md](../agent-docs/updates.md), [packaging.md](../agent-docs/packaging.md)). The key pair is `signing/opsis.key` (ignored by git) and `signing/opsis.key.pub` (committed); the key and its password are in the ignored `.env` and the repository's secrets as `OPSIS_UPDATE_KEY` and `OPSIS_UPDATE_KEY_PASSWORD`, and the ignored test `the_update_key_signs_what_the_committed_public_key_verifies` (`crates/packager/src/sign.rs`) passed with them on 2026-11-08.

**Work.**

1. **Check a real update on each system** once the release workflow runs: install a published version, publish a newer one, and update from it on the AppImage, the NSIS and MSI installs and both Macs (Restart to update and an ordinary close); the deb, rpm and pacman installs open the release page.

**Risks.** An ad-hoc-signed macOS app that replaces itself may meet Gatekeeper on first open until section 5 signs and notarizes it.

## 7. Release test pass

**In place.** `bun run e2e:qt` runs four scenarios in the real app: `tour` (every panel, field and window), `runs` (setup, recording, export, Replay, the element picker, terminal takes recorded, played, taped live and imported from a cast), `api` (every Local API route from outside, ending with the close check) and `wayland` (real input on headless sway); `cargo test -p opsis-ui` runs the `views` scenario offscreen. `bun run package:check` installs each Linux package in a clean container and opens and exports the changelog example in it ([packaging.md](../agent-docs/packaging.md)).

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
