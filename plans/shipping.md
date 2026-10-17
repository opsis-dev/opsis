# Shipping

What it takes to hand Demoflow to other people on Windows, macOS and Linux: builds that have actually been run on each system, app icons, signed installers, a way to update, a settled FFmpeg licence, a test pass before each release, and, later, node packs that ship and update apart from the app. Each topic has its own section with the same parts, so the plan can grow one section at a time.

Status: plan only. Nothing here is built yet.

This covers milestone M6 of the original spec ("installers per OS via Forge makers, CI matrix, first-run downloads screen, auto-update, code signing") plus a later design discussion about node packs. The first-run engine screen from M6 already exists (Settings → Engine, `src/engine/install.ts`).

## How to read this plan

- Sections 1–5 come before a first public release, in order: a build that has never run can't be signed, and a signed build is what updates need. Section 6 runs before every release. Section 7, node packs, is the long-term part, built after the rest.
- **Status** is one of: `not started`, `defined, never run` (config exists but nobody has run it), `partly in place`, `decision pending` (the work depends on the user's choice).
- Each section has the same parts: **What it adds**, **Fit with Demoflow** (Already in place / Missing, every file and function named was checked in the repo), **Work** (numbered), **Decisions for the user** where a choice, a cost or an account is the user's, **Risks**, **Sources**.
- Prices and terms change; each one says when it was checked.

## Conventions

These restate rules from `AGENTS.md`; when the two disagree, `AGENTS.md` wins.

- No comments of any kind, kebab-case file names, dependencies only through `bun add` / `bun add -d` (latest releases; `remotion` / `@remotion/*` stay at exact `4.0.535`, `typescript` at `6.0.3`).
- Packaging rules in `AGENTS.md` **Packaging** hold: Forge's Vite plugin ships only `.vite/`, runtime packages are staged in `packageAfterCopy`, the Remotion bundle is prebuilt, the engine is staged for the host only, `LICENSES.chromium.html` always ships with the engine.
- **Nothing uses the internet unless the user asks.** Checking for updates, downloading them and downloading node packs each sit behind a button or a setting that is off by default, and say what they will download before they do.
- **User-facing copy** never names the tools behind it: "Check for updates", not "Squirrel"; "This app isn't signed yet", not "notarization failed".
- Certificates, keys and passwords live only in GitHub Actions secrets or the signing service, never in the repo or in `forge.config.ts`.
- Large downloads (CI artifacts, test builds, long test recordings) go to `~/.cache/demoflow-*`.
- Don't commit or push unless the user asks; the first CI run (section 1) needs a push, so it waits for the user.

**Checks when a section is done.** `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, the no-comments check from `AGENTS.md`, and `actionlint .github/workflows/build.yml` for any workflow change.

## Summary

| Topic                       | What users get                                                                                              | Cost to the user                                                      | Status             |
| --------------------------- | ----------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------- | ------------------ |
| 1. Windows and macOS builds | Installers for Windows and macOS that someone has actually run                                              | CI minutes if the repo is private; access to a Windows PC and Macs    | defined, never run |
| 2. App icons                | The Demoflow icon in the Windows taskbar, installer and Programs list, and the macOS Dock and Finder        | none                                                                  | partly in place    |
| 3. Code signing             | No "unknown publisher" or "can't be opened" warnings; required for updates on macOS                         | Apple: 99 USD/year; Windows: about 10 USD/month or about 220 USD/year | not started        |
| 4. Updating the app         | **Check for updates** in Settings, opt-in, asks before downloading                                          | none beyond signing; hosting is GitHub Releases                       | not started        |
| 5. FFmpeg licence           | The GPL notices and the matching FFmpeg source, so the app is distributed correctly                         | hosting the source tarballs                                           | settled: keep GPL  |
| 6. Release test pass        | Fewer surprises: long takes, odd paths, old projects, every template, a smoke test in CI                    | none                                                                  | not started        |
| 7. Node packs (long term)   | Nodes that install and update on their own, reusable groups of nodes, and projects that name what's missing | none                                                                  | not started        |

## 1. Windows and macOS builds

### What it adds

A Windows installer (Squirrel `Setup.exe`) and macOS zips (Apple Silicon and Intel), each in the full and lite editions, built by CI and checked by hand on each system. Today Demoflow has only been built and run on Linux (README: "Windows and macOS builds haven't been made or tested yet").

### Fit with Demoflow

Already in place:

- `.github/workflows/build.yml` runs the checks, then `make` on `ubuntu-latest`, `windows-latest`, `macos-latest` and `macos-26-intel` for `full` and `lite`, caches `resources/engine` per OS and arch, checks the licence files and uploads `out/make/**`.
- `forge.config.ts`: `MakerSquirrel({})`, `MakerZIP({}, ["darwin"])`; `stageEngine` stages the engine for the host only; `stageCompositorPackages` handles both darwin compositor packages; `asar.unpackDir` keeps the compositor outside the archive.
- `src/main/main.ts`: Squirrel's install events are handled (`electron-squirrel-startup`); macOS gets `titleBarStyle: "hiddenInset"` with `trafficLightPosition`, other systems `frame: false`. `src/main/menu.ts` sets the native menu from `appMenu()` on macOS and none elsewhere; `WindowControls`, `WindowBorder` and `useMenuShortcuts` live in `src/renderer/app/app-bar.tsx`.
- `src/engine/paths.ts`: `remotionPlatform` and `enginePaths` add `.exe` on Windows; FFmpeg comes from the pinned `FFMPEG_RELEASE` (`b6.1.1`).
- `src/main/api.ts`: `apiAddress()` gives `\\.\pipe\demoflow-api-<user>` on Windows; the socket file's `chmod 0600` runs only on other systems.
- `watchProject` (`src/main/projects.ts`) watches the project folder, because saves write a temporary file and `rename` it over the project file.
- Project refs are built with forward slashes (`.split(path.sep).join("/")` in `src/worker/engine/runner.ts`, `src/worker/executors/output.ts`, `src/worker/mix/render.ts`, `src/worker/media/analysis.ts`).

Missing:

- The repo has no git remote, so the workflow has never been pushed or run.
- No `app.setAppUserModelId`. Squirrel's shortcut uses `com.squirrel.<package id>.<exe name>`, and the app must use the same id for taskbar pinning and notifications.
- Nobody has checked who can open the Windows pipe. Without explicit security, a named pipe's default descriptor grants full control to LocalSystem, administrators and the creator owner, and **read access to Everyone and the anonymous account**. README says only your user account can open it; that is unverified on Windows.
- The paths that turn refs back into files (`src/main/protocol.ts`, `startAssetServer` in `src/worker/render/asset-server.ts`) and the FFmpeg concat lists (`src/worker/executors/media.ts`, `-f concat -safe 0`) have only met Linux paths.

### Work

1. **Workflow check.** Run `actionlint` on `build.yml`. Keep `bun run lint` (not `lint:fix`) in CI.
2. **First run.** Once the user has made the GitHub repo and added the remote (see **Decisions**), push a branch and start the workflow with `workflow_dispatch`. Expect all eight build jobs to be needed at least once; `fail-fast: false` is already set.
3. **Fix failures one system at a time.** Likely places: `stageEngine` and `installHeadlessShell` on Windows (Chrome for Testing download and unzip), `stageCompositorPackages` for `@remotion/compositor-win32-x64-msvc` and `-darwin-x64`, the Squirrel maker on a ~400 MB app (the Linux engine alone is 393 MB on disk), and `bun install --frozen-lockfile` picking each OS's optional compositor package. Keep fixes in `forge.config.ts`, `src/engine/` and the workflow.
4. **Keep the builds.** Download each job's artifact to `~/.cache/demoflow-ci/<run id>/` and note the installer sizes in this plan.
5. **App id.** Call `app.setAppUserModelId("com.squirrel.demoflow.demoflow")` on Windows (check the exact id against the shortcut Squirrel makes) before any window opens.
6. **Checks on the Windows runner** (decided: CI runners only, no manual pass; `windows-latest`, x64). Automated through the Local API, which works on the packaged app where Playwright can't attach (see section 6, **Risks**): install with `Setup.exe --silent`, write a test profile's `settings.json` with `api.enabled: true`, start the installed app, wait for the pipe, then `/status`, `/open` on a copy of `examples/feature-launch`, `POST /project/nodes` for a title and an export, `POST /project/run`, and `ffprobe` on the output; then the second-user pipe check below with a local account the job creates. What a runner can't judge (how the window controls look, SmartScreen prompts) is listed as not covered in the release notes. The checks:
   - Setup.exe installs per user and starts the app; Start menu entry; uninstall from Settings → Apps removes it.
   - Frameless window: `WindowControls` minimize, maximize/restore (icon follows `windowState`), close through the exit dialog; `WindowBorder` shows only when not maximized; dragging the `AppBar` moves the window, double-click maximizes, Win+arrows snap; the in-app menu as a full menubar from 1280 px and as one Menu button below; Ctrl shortcuts.
   - Lite: Settings → Engine → Set up engine downloads and the Licences page lists the browser's licences. Full: the engine is found in `resources/engine` with no download.
   - Record the "Sample: to-do app" template, export 16:9 and 9:16, card stills appear, Mixer stems render, Fonts from the computer and from a ZIP.
   - File watching: edit `project.demoflow.json` in Notepad and in VS Code (which replaces the file) while the project is open: with nothing unsaved the editor reloads it; with unsaved edits `DiskChangeDialog` asks. Save while another program has the file open (Windows can refuse `rename` with `EPERM`/`EBUSY`): the save must retry or report, never lose the edit.
   - Paths: a project under `C:\Users\<name>\Demo Projects\` with spaces; import media from another drive; a Concat/Join node over clips in folders with spaces and an apostrophe.
   - Local API: switch it on, then call `/status` from a Node script with `http.request({ socketPath: "\\\\.\\pipe\\demoflow-api-<user>" })`. Then, signed in as a second standard user on the same PC, try the same pipe; it must refuse. Read the pipe's ACL (Sysinternals `accesschk \pipe\demoflow-api-<user>`). Check a user name with a space and with non-English letters.
7. **If the pipe is open to anyone else,** make it the current user's only: either set the pipe's security descriptor at creation (Node has no option for that; `listen({ readableAll, writableAll })` only widens it), or require a random token from `<userData>/api-token` (under `%APPDATA%`, readable only by the user) on every request, and say so in Settings → API and `openapi.ts`. Re-test with the second user. Update README's sentence to match what was verified.
8. **Checks on the macOS runners** (`macos-latest` for Apple Silicon, `macos-26-intel` for Intel), automated the same way through the Local API socket; Gatekeeper's first-open prompts and how the traffic lights look can't be judged on a runner and are listed as not covered. The checks:
   - First open of an unsigned zip: expect "can't be opened"; record the exact steps (System Settings → Privacy & Security → Open Anyway) for the release notes until section 3 is done.
   - Traffic lights sit inside the `AppBar` at `trafficLightPosition` and clear its controls; full screen hides them and the bar adjusts.
   - Native menu: the app menu, Edit roles, Cmd+, opens the Settings window, Cmd+Q goes through the exit dialog.
   - Same engine, recording, export, fonts and file-watching checks as Windows; the Local API socket file is mode 0600.
9. **Write down** what failed and what was fixed in this section, then update README's "Early preview" note and `AGENTS.md` **Packaging** once both systems pass.

### Decisions for the user

- **Public repo (decided).** Demoflow is Apache-2.0 open source, so the repo is public: standard GitHub runners are free on Linux, Windows and macOS, and the hosted update service (section 4) can be used.
- **Test machines (decided): CI runners only.** No manual Windows or macOS pass; the runner checks above, plus a person on Linux.
- **Windows on Arm and a universal macOS build (decided): left out of the first release.** x64 Windows, and separate Apple Silicon and Intel Mac builds; `osxUniversal` stays configured but unused.

### Risks

- **Long paths on Windows.** Squirrel installs to `%LocalAppData%\demoflow\app-<version>\`, and the engine goes several folders deeper (`resources\engine\chrome-headless-shell\win64\chrome-headless-shell-win64\…`). Check that nothing passes 260 characters, especially in the lite edition's `<userData>\engine`.
- **Squirrel's first run.** Right after install the app starts with `--squirrel-firstrun` and Squirrel holds a lock; anything that touches the install folder then (or a section 4 update check) can fail.
- **Security software.** Unsigned `chrome-headless-shell.exe` and `ffmpeg.exe` started from a per-user folder may be scanned slowly or blocked; time the first recording.

### Sources

- [GitHub Docs: GitHub Actions billing](https://docs.github.com/en/billing/concepts/product-billing/github-actions) (free minutes and per-minute rates; checked 2026-10-16)
- [GitHub changelog: macOS 26 generally available for GitHub-hosted runners](https://github.blog/changelog/2026-02-26-macos-26-is-now-generally-available-for-github-hosted-runners/) (`macos-26-intel` label; checked 2026-10-16)
- [Electron: autoUpdater, Windows notes](https://www.electronjs.org/docs/latest/api/auto-updater) (Squirrel launch events, `--squirrel-firstrun`, `setAppUserModelId`; checked 2026-10-16)
- [Microsoft Learn: CreateNamedPipeA, `lpSecurityAttributes`](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createnamedpipea) (default pipe security; checked 2026-10-16)
- `du -sh` of the installed Linux engine (`~/.config/Demoflow/engine`, 393 MB; measured 2026-10-16)

## 2. App icons

### What it adds

The Demoflow icon on Windows (installer, Start menu, taskbar, Programs list) and macOS (Dock, Finder, the zip's app). Today both show Electron's icon.

### Fit with Demoflow

Already in place:

- `brand/logo.svg` is the one source. `bun run brand:icons` (`scripts/brand-icons.ts`) renders it with the recording browser to `brand/icons/<size>.png` for `ICON_SIZES` (16, 24, 32, 48, 64, 128, 256, 512, 1024). `AGENTS.md` says why: ImageMagick draws the gradients wrong.
- `forge.config.ts` ships `brand/icons` as an extra resource; the window icon, deb and AppImage use the PNGs.

Missing:

- No `.ico` or `.icns`; `packagerConfig` has no `icon`; `MakerSquirrel({})` has no `setupIcon` or `iconUrl`.

### Work

1. **Pack the PNGs.** In `scripts/brand-icons.ts`, after rendering, write `brand/icons/icon.ico` (16, 24, 32, 48, 64, 256; ICO entries can hold PNG data directly) and `brand/icons/icon.icns` (PNG-based types: `icp4` 16, `icp5` 32, `icp6` 64, `ic07` 128, `ic08` 256, `ic09` 512, `ic10` 1024, plus the `@2x` types). Both formats are a small header plus the PNG files, so a few lines of `Buffer` code do it; no new dependency, no `iconutil` (macOS only) and no ImageMagick. Unit-test the headers: entry count, sizes, offsets.
2. **Wire them.** `packagerConfig.icon: path.join(ICONS_DIR, "icon")` (Forge picks `.ico` / `.icns` by platform). `MakerSquirrel({ setupIcon: …icon.ico, iconUrl: … })`. `iconUrl` is documented as a URL for the Programs list; check whether a `file:` path works so neither building nor installing fetches it, and if it must be a URL, use the release's own copy on GitHub.
3. **Regenerate** with `bun run brand:icons` and check the generated-code table in `AGENTS.md` (row `brand/icons/*.png`) now lists `.ico` and `.icns` too; update the `AGENTS.md` **Packaging** icon bullet.
4. **Check** on each system in section 1's pass: small sizes are sharp (16 px in Explorer, 32 px in the Programs list), macOS Dock at Retina size.

### Risks

- `brand:icons` needs a headless shell in `resources/engine` or `DEMOFLOW_HEADLESS_SHELL`; on a fresh checkout run `make` once first, as the script's error says.
- Windows caches icons; reinstall or clear the icon cache before judging a change.

### Sources

- [Electron Forge: Squirrel.Windows maker](https://www.electronforge.io/config/makers/squirrel.windows) (`setupIcon`, `iconUrl`)
- [Electron Forge: custom app icons](https://www.electronforge.io/guides/create-and-add-icons)

## 3. Code signing and notarization

### What it adds

Installers that open without "Windows protected your PC" or "can't be opened because Apple cannot check it", with Demoflow's verified publisher name. On macOS it's also required for updates (section 4).

### Fit with Demoflow

Already in place:

- Forge signs at package (`packagerConfig.osxSign`, `osxNotarize`) and make time (Squirrel `windowsSign`); nothing else in the build needs to change shape.
- `FusesPlugin` already turns on `EnableEmbeddedAsarIntegrityValidation` and `OnlyLoadAppFromAsar`, which are meant for signed apps.

Missing:

- No accounts, certificates or CI secrets; no `osxSign` / `osxNotarize` / `windowsSign` in `forge.config.ts`; no entitlements file.

### Work

1. **macOS.** Enrol in the Apple Developer Program, create a Developer ID Application certificate, and add to CI (macOS jobs only): the certificate as a base64 `.p12` imported into a temporary keychain, and an App Store Connect API key for notarization. In `forge.config.ts`: `osxSign: {}` with an entitlements file allowing what Electron needs under the hardened runtime, and `osxNotarize` with `appleApiKey`, `appleApiKeyId`, `appleApiIssuer` from `process.env`. Only sign when those variables exist, so local and fork builds still work.
2. **Everything inside the app is signed.** The full edition carries `chrome-headless-shell`, `ffmpeg` and `ffprobe` in `resources/engine`, and the compositor's `remotion` binary and libraries in `app.asar.unpacked`. Check with `codesign --verify --deep --strict` and `spctl -a -vv` that every executable is signed with the hardened runtime and that notarization accepts them.
3. **Lite edition engine.** The engine downloaded into `<userData>/engine` by `installEngine` isn't part of the signed app. Check on a notarized build that it still starts (downloads made by the app itself aren't quarantined, but check, don't assume).
4. **Windows.** Pick a signing route (see **Options and suggestion**). Pass `windowsSign` to both `packagerConfig` and `MakerSquirrel` so `demoflow.exe`, `Update.exe` and `Setup.exe` are signed, with a timestamp server. Sign the bundled `chrome-headless-shell.exe`, `ffmpeg.exe` and `ffprobe.exe` in the full edition too.
5. **Check** on clean machines: Windows shows the publisher name (a SmartScreen "unrecognized app" prompt can still appear until reputation builds); macOS opens the downloaded zip with a single "downloaded from the internet" confirmation.

### Options and suggestion

These are suggestions with their costs; nothing is assumed until the user picks.

**Suggested:** sign macOS before giving the app to macOS users, because without notarization most of them can't open it normally and macOS updates (section 4) don't work. On Windows, start unsigned (it runs after "More info → Run anyway", and Squirrel updates still work), then add Azure Artifact Signing if the user is eligible, or an OV certificate once downloads justify it.

- **Apple Developer Program: 99 USD per year** (individual or organization; an organization needs a D-U-N-S number). There is no cheaper way to notarize, and without it macOS 15 and later block the app until the user goes to System Settings → Privacy & Security → Open Anyway.
- **Windows: one of**
  - **Azure Artifact Signing** (formerly Trusted Signing): 9.99 USD/month for 5,000 signatures (Basic). No hardware token, works from GitHub Actions. Public certificates are only for **organizations** in the US, Canada, the EU, the UK, Australia, New Zealand, Japan, South Korea, Singapore, Switzerland, Norway and Israel, and **individuals in the US or Canada**. Identity validation takes 1–20 business days. Needs a paid Azure subscription.
  - **An OV certificate from a certificate authority**, about 220 USD/year for Sectigo through resellers, up to about 440 USD/year for DigiCert. Since June 2023 the key must live on a hardware token or the authority's cloud signing service, which affects how CI signs.
  - **Not signing Windows yet**: users click "More info → Run anyway". Squirrel updates still work on Windows without signing.
- EV certificates no longer skip SmartScreen; Microsoft says paying for EV only for that "is no longer justified". Every route builds reputation over downloads.
- **Signer (decided): the user's own name**, on an individual Apple Developer account and, when Windows is signed, an individual certificate. It's shown to every user and is hard to change later.

### Risks

- **Reputation resets with a new identity.** Changing the certificate or the publisher name starts SmartScreen reputation again; pick the name once.
- **Notarization is slow and strict.** Any unsigned binary or a missing hardened runtime fails the whole submission; the engine and compositor are the likely culprits.
- **Smart App Control** on Windows 11 blocks unsigned files with no reputation outright, not just with a prompt.

### Sources

- [Apple: Enroll in the Apple Developer Program](https://developer.apple.com/programs/enroll/) (99 USD/year, D-U-N-S for organizations; checked 2026-10-16)
- [Electron Forge: Signing a macOS app](https://www.electronforge.io/guides/code-signing/code-signing-macos) (`osxSign`, `osxNotarize`, credentials; checked 2026-10-16)
- [Electron Forge: Signing a Windows app](https://www.electronforge.io/guides/code-signing/code-signing-windows) (`windowsSign`, hardware key rule since June 2023; checked 2026-10-16)
- [Microsoft Learn: Artifact Signing quickstart](https://learn.microsoft.com/en-us/azure/artifact-signing/quickstart) (eligible countries, individuals, validation time; checked 2026-10-16)
- [Microsoft Learn: Artifact Signing FAQ](https://learn.microsoft.com/en-us/azure/artifact-signing/faq) (paid subscription required; checked 2026-10-16)
- [Azure: Artifact Signing pricing](https://azure.microsoft.com/en-us/pricing/details/artifact-signing/) (5,000 / 100,000 signatures; the page showed no price when fetched) and [Microsoft Learn: SmartScreen reputation](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation) ("Starts at $9.99/month", EV no longer bypasses SmartScreen, Smart App Control; checked 2026-10-16)
- [SSL Insights: DigiCert vs Sectigo code signing (2026)](https://sslinsights.com/digicert-vs-sectigo-code-signing/) (reseller prices; checked 2026-10-16)
- [The Hacker News: macOS Sequoia tightens Gatekeeper](https://thehackernews.com/2024/08/apples-new-macos-sequoia-tightens.html) (Control-click override removed)

## 4. Updating the app

### What it adds

**Check for updates** in Settings: one click asks whether a newer version exists, shows the version and its download size, and installs only when the user says so. An optional switch checks when the app starts. Nothing is checked by default.

### Fit with Demoflow

Already in place:

- The Settings window has an About page (`AboutPage` in `src/renderer/app/settings/pages.tsx`) showing the version and edition; settings are a zod schema (`appSettingsSchema`, `src/shared/settings.ts`) written section by section with `patchSettings`.
- Squirrel's install events are handled in `src/main/main.ts`; `track(label, promise)` (`src/main/exit.ts`) lists long work in the exit dialog.

Missing:

- No updater at all. `update-electron-app` is not installed, which is right: it checks at startup and every ten minutes, breaking the offline rule.
- Electron's `autoUpdater` has no Linux support.
- The two editions share one version number and platform, so a feed keyed only by platform and arch can't tell a full install from a lite one.

### Work

1. **Settings.** Add `updates: { checkOnStart: false, lastChecked: "" }` to `appSettingsSchema`. On the About page add a "Check for updates" button and a switch, "Check for updates when Demoflow starts", with the line "Demoflow only goes online for this when you ask." Use the existing `Card` / `Field` / `Switch` pattern from `GeneralPage`.
2. **Two steps, both asked for.** Step one fetches a small manifest (latest version, notes, size per platform and edition) from the release host when the user clicks the button (or at start, if they switched that on). Step two, after the user clicks "Download and install", hands the feed to `autoUpdater`. Never call `autoUpdater.checkForUpdates()` as the "check": on Windows and macOS it starts downloading as soon as it finds an update.
3. **Windows and macOS.** `autoUpdater.setFeedURL` with a feed per edition; wrap the download in `track("Downloading an update", …)`; on `update-downloaded` offer "Restart now" or "Later" (installed on next quit). Skip everything while `--squirrel-firstrun` is set. macOS requires the app to be signed (section 3).
4. **Feed.** Either `update.electronjs.org` (free, needs a public GitHub repo and releases named with `-darwin` / `-win32` and the arch; it has no edition in the URL, so only one edition can use it) or static files on GitHub Releases: Forge's `macUpdateManifestBaseUrl` writes the macOS `RELEASES.json`, Squirrel writes `RELEASES`. Static files per edition (`…/full/`, `…/lite/`) solve the edition problem; recommended.
5. **Linux AppImage.** Embed update information (`gh-releases-zsync|<owner>|<repo>|latest|Demoflow-*-x86_64.AppImage.zsync`) and upload the `.zsync` file. `@reforged/maker-appimage` 6.0.0 has no option for it (its source has no `zsync`), so either post-process with `appimagetool -u` in CI or offer the download link only. The in-app button runs the same two steps: show the version, then download the delta with `libappimageupdate`-compatible tooling or open the release page.
6. **Linux deb.** No self-update. Either publish an apt repository (signing key, hosting) or, simpler, show "Version X is available" with a button that opens the release page.
7. **Release workflow.** On a `v*` tag, the workflow (already triggered by tags) publishes the signed artifacts, the manifests and `.zsync` files to a draft GitHub Release; a person publishes it after section 6's pass.
8. **Tests.** Unit-test the manifest parsing and version comparison; manually test an update from one tagged build to the next on each system, including declining it.

### Decisions for the user

- **Full edition updates are big (open; the user is undecided).** Squirrel installs each version into a new `app-<version>` folder, so a full-edition update downloads the engine again (about 400 MB on Linux). Options: let the full edition update in full; offer the full edition only as an offline installer and point updaters to the lite edition, which keeps its engine in `<userData>/engine` across updates; or move the full edition's engine to `<userData>/engine` on first run.
- **Feed address (decided): the GitHub repo URL.** Installed copies ask `github.com/bismitpanda/demoflow` (or whatever the repo is called at release). If the repo is renamed, GitHub forwards the old address (web, git, API and release downloads) to the new name, so installed copies keep finding updates. The forwarding stops if a new repo is ever created under the old name, so never reuse it.
- **deb (decided):** no apt repository; the app shows "Version X is available" with a button that opens the release page.

### Risks

- **The offline rule.** Any library that checks on its own (including `update-electron-app` defaults) must not be used; review every new dependency in this section for timers.
- **Squirrel.Windows is old.** Forge's Squirrel maker still works, but it's the least maintained part of the stack; an MSIX or NSIS route would change the installer and the update path.
- **A broken update can't fix itself.** Keep the manifest check independent of the rest of the app so even a buggy release can still offer the next one.

### Sources

- [Electron: Updating applications](https://www.electronjs.org/docs/latest/tutorial/updates) (update.electronjs.org, static storage, `RELEASES` / `releases.json`; checked 2026-10-16)
- [Electron: autoUpdater](https://www.electronjs.org/docs/latest/api/auto-updater) (no Linux support, macOS must be signed, Squirrel first run; checked 2026-10-16)
- [update.electronjs.org](https://github.com/electron/update.electronjs.org) (public repo, asset naming; checked 2026-10-16)
- [AppImage docs: making AppImages updateable](https://docs.appimage.org/packaging-guide/optional/updates.html) (update information, `appimagetool -u`, "never download updates without the user's explicit consent"; checked 2026-10-16)

## 5. FFmpeg licence

### What it adds

The notices and source the GPL asks for when Demoflow ships FFmpeg. GPL is not a problem for Demoflow: the app is open source under Apache-2.0, and it runs FFmpeg as separate programs over command-line arguments and pipes, which the GPL FAQ treats as separate programs, so the GPL doesn't reach Demoflow's own code (Apache-2.0 is also compatible with GPLv3). What the GPL does require for the binaries Demoflow distributes is their licence text and their exact source. Decided: keep the GPL builds.

### Fit with Demoflow

What ships today (checked 2026-10-16):

- **ffmpeg-static** (`FFMPEG_RELEASE` `b6.1.1`, both 76 MB binaries): the Linux binary reports `7.0.2-static` from johnvansickle.com, configured with `--enable-gpl --enable-version3`, `libx264`, `libx265`, `libvpx`, `libmp3lame`, `libopus` and many more. macOS builds come from evermeet.cx (Intel) and osxexperts.net (Apple Silicon), Windows from gyan.dev. `LICENSE.txt` next to them is GPLv3. Full edition: bundled in `resources/engine`. Lite: downloaded by the user's copy from the ffmpeg-static GitHub release.
- **Remotion's compositor** (`@remotion/compositor-*`, in `app.asar.unpacked`) carries its own FFmpeg 7.1, configured with `--enable-gpl`, `libx264`, `libx265`. Remotion documents it as GPLv2+ and uses fdk-aac-free for AAC. Renders use it whatever happens to ffmpeg-static, so **switching ffmpeg-static to LGPL builds doesn't make the app GPL-free**.
- Demoflow runs both as separate programs over command-line arguments and pipes; nothing links them.

GPL-only parts Demoflow uses (searched `src/worker` and `src/shared/mix`):

- `libx264`: the take encoder (`src/worker/capture/encoder.ts`), `src/worker/executors/capture.ts`, and most media nodes in `src/worker/executors/media.ts` (trim, join, speed, reverse, resize, web-ready).
- `libx265`: the H.265 option in `src/worker/executors/media.ts`.
- Everything else is LGPL or BSD: `libvpx-vp9`, `libopus`, `libmp3lame`, native `aac`, `prores_ks`, `gif`, `png`; the mixer's filters (`sidechaincompress`, `acompressor`, `alimiter`, `loudnorm`, `ebur128`, `equalizer`, `highpass`, `pan`, `amix`, `atempo`, `areverse`, `palettegen` / `paletteuse`) and `muxArgs` in `src/worker/render/picture.ts`. None of FFmpeg's GPL-only filters (`vidstab*`, `frei0r`, `rubberband`, `delogo`, `hqdn3d`…) appear.

### Work

1. **Notices.** Keep `LICENSE.txt` beside the binaries (already checked by the workflow); add a section to `THIRD_PARTY_NOTICES.md` and the Licences page that FFmpeg ships as separate GPL programs, with versions, and that Remotion's compositor contains a GPLv2+ FFmpeg.
2. **Source offer.** GPLv3 §6 requires the Corresponding Source with each binary distribution, or a written offer. For each platform's build, record where its exact source lives and mirror the source tarballs as assets of each Demoflow release (full edition) so availability doesn't depend on third-party sites. Link them from the Licences page.
3. **Lite edition.** The binaries come straight from the ffmpeg-static release; the notice and source links still apply, since the app installs them.

### Considered and set aside

- **LGPL FFmpeg builds:** they'd lose `libx264` and `libx265` (H.264 would need OpenH264 or hardware encoders, changing every H.264 path and the look of takes), there's no ready macOS LGPL build, and Remotion's compositor ships a GPL FFmpeg anyway, so the app would still distribute GPL code.
- **Relicensing Demoflow as GPLv3:** not needed, since separate programs don't require it.
- This is not legal advice; for a commercial release, have someone qualified read the final notice.

### Risks

- **Patents are a separate question.** H.264/H.265 patent pools apply regardless of the copyright licence; the GPL builds don't grant patent rights either.
- **Keeping source in step.** Whenever `FFMPEG_RELEASE` (`src/engine/paths.ts`) or the Remotion version changes, the mirrored source and the notice must change with it; the CI licence check should fail when they don't match.

### Sources

- [FFmpeg: License and legal considerations](https://ffmpeg.org/legal.html) (LGPL checklist, "If those parts get used the GPL applies to all of FFmpeg", libx264; checked 2026-10-16)
- [GNU: GPL FAQ, MereAggregation](https://www.gnu.org/licenses/gpl-faq.html#MereAggregation) (pipes, sockets, command-line arguments; checked 2026-10-16)
- [Remotion: FFmpeg licence](https://www.remotion.dev/docs/miscellaneous/ffmpeg-license) (GPLv2+, x264/x265, fdk-aac-free; checked 2026-10-16)
- [ffmpeg-static README](https://github.com/eugeneware/ffmpeg-static) (sources of the binaries per platform; checked 2026-10-16)
- [BtbN/FFmpeg-Builds](https://github.com/BtbN/FFmpeg-Builds) (`lgpl` variants for Windows and Linux only, without libx264/libx265; checked 2026-10-16)
- [OpenH264 FAQ](https://www.openh264.org/faq.html) (BSD source, royalties covered only for Cisco's binary; checked 2026-10-16)
- `ffmpeg -version` of the installed ffmpeg-static binary and of `@remotion/compositor-linux-x64-gnu@4.0.535` (configure lines; run 2026-10-16)

## 6. Release test pass

### What it adds

A list of what has never been tested, run by hand before each release, and a small automated smoke test of the real app in CI.

### Fit with Demoflow

Already in place:

- Unit tests with vitest (`src/**/*.test.ts`): node registry, cards, compile, timeline edits, mix graphs.
- `playwright-core` (already a dependency) exports `_electron`, which launches and drives an Electron app.
- Seven templates in `src/shared/templates/index.ts` (Blank plus six, with copies of the six in `examples/`); only `todomvc` has been run end to end.

Missing:

- No end-to-end UI tests at all; no checklist for a release.

### Work

1. **Release checks,** on Linux by hand every release, and on Windows and macOS through the CI runners only (decided), noting results in the release's draft notes:
   - **Long takes:** a 10 and a 20 minute recording; memory of the worker and the editor, `FrameEncoder` output size, Timeline and Preview responsiveness, export time, trace size in `.demoflow/traces`.
   - **Paths:** a project folder with spaces, with non-English letters (`Démo`, `デモ`), and on another drive (Windows); asset import from such folders; fonts copied into `assets/fonts/<id>/`.
   - **Moving a project folder** while closed, and renaming it while open: recent projects, `watchProject`, cached results and stills still resolve (refs are relative), the Local API's `/open`.
   - **Custom canvas sizes** beyond 16:9, 1:1, 4:5, 9:16: e.g. 21:9, 3:2, 1080×1080 at 60 fps, a very small and a 4K canvas; `variantSize` (`src/shared/nodes/output.ts`), overlays and `shapeInsets` (`src/video/core/frame.ts`).
   - **Broken font files:** a truncated `.ttf`, a ZIP with no fonts, a font that claims weights it lacks; the library import in `src/main/fonts.ts` must refuse with a plain message, never crash.
   - **Every template end to end:** Feature launch, Changelog clip, Onboarding walkthrough, Social teaser (9:16), Before / after: create, run, export, compare Preview and export.
   - **Stopping a run** at every stage (engine setup, browser start, recording, encoding, rendering, mixing) with `cancelRun` (`src/main/workers.ts`), then running again; and **killing the worker process** mid-run: the editor must report it and the next run must start a fresh worker.
   - **Two windows editing at once:** the editor plus the Timeline and Mixer popped out (`src/renderer/app/sync.ts`): edits in both, undo from each, closing one mid-drag.
   - **Keyboard only:** create a project, add and connect nodes, edit settings in the Inspector, run and export without a mouse; focus is always visible.
2. **Smoke suite.** `e2e/smoke.test.ts` with its own `vitest` project (or `vitest.e2e.config.ts`, so `bun run test` stays fast) and a script `bun run test:e2e`. It builds the development Vite output (`.vite/build/main.cjs`, see **Risks**), then uses `_electron.launch` from `playwright-core` with a fresh `--user-data-dir` in a temporary folder:
   - the start screen appears; create a project from the Blank template in a temporary folder;
   - add a Title node from the node list, connect it to an export, delete another node, undo;
   - open Settings (Ctrl+,), switch pages, close it;
   - export a title-only clip (no recording, so no website is needed) with the engine from the CI cache, and check the file exists and `ffprobe` reports the expected length.
3. **CI.** A Linux job after `checks` that installs the engine from the same cache, runs the suite under `xvfb-run`, and uploads screenshots and the trace on failure.
4. **Packaged-app check on every OS.** After `make`, each build job starts the packaged app with a test profile that has the Local API on, and drives it through the socket or pipe (create from a template, add a title and an export, run, `ffprobe` the file). This is how the packaged builds get tested on Windows and macOS, since the fuses stop `_electron` from attaching to them.

### Risks

- **The fuses block attaching.** `forge.config.ts` turns off `EnableNodeCliInspectArguments` and `RunAsNode`, which `_electron.launch` relies on to connect, so the suite can't drive the packaged app. Run it against the development build (Electron from `node_modules` with `.vite/build/main.cjs`), and keep a short manual check of the packaged app in the pass above.
- **`_electron` is marked experimental** in Playwright; pin nothing, but keep the suite small so an API change is a quick fix.
- **The Linux sandbox.** CI needs `--no-sandbox` like `scripts/start.ts` adds in development.

### Sources

- [Playwright: Electron](https://playwright.dev/docs/api/class-electron) (`_electron.launch`, experimental)
- [Electron: Fuses](https://www.electronjs.org/docs/latest/tutorial/fuses) (`EnableNodeCliInspectArguments`, `RunAsNode`)

## 7. Node packs (long term)

### What it adds

Built-in and third-party nodes that install and update on their own, separate from the app:

- **App shell** (Electron, main, preload, worker host, engine) updates rarely and is signed (sections 3–4).
- **Node packs**: a documented format with a manifest and node definitions. Built-in nodes become first-party packs.
- **No-code custom nodes**: save a selection of nodes as a reusable node, plus templates and presets. These are data, so they're safe to share.
- **Projects name what they need**: opening a project without a pack says "This project needs the _Charts_ pack (1.2 or later)" instead of "This node type isn't available".

Build this after sections 1–6.

### Fit with Demoflow

Already in place:

- A node is data plus optional pure code: `NodeDef` (`src/shared/nodes/define.ts`) has ports, a zod schema with `.meta()` for the Inspector, an optional pure `compile()` and `card()`. `NODE_DEFS` (`src/shared/nodes/registry.ts`) lists them; executors are keyed by type (`getExecutor`, `src/worker/executors/index.ts`).
- Video components already load lazily by key: `generatedElementLoaders` (`src/video/registry/components.generated.ts`) holds `import()` loaders, `loadElement` / `isKnownElement` (`src/video/registry/index.ts`) and `delayRender` wait for them.
- Unknown types are already handled, with one message: `src/shared/graph.ts`, `src/shared/compile/compile.ts`, `src/renderer/editor/inspector.tsx` ("This node type isn't available").
- Cache keys hash `ENGINE_VERSION` (`src/worker/engine/cache.ts`), type, params and upstream hashes.
- Groups exist (`core.group`, `src/shared/nodes/utility.ts`, created in `src/renderer/store/workflow.ts`), but only as a visual frame around nodes.
- Untrusted page code already runs only in sandboxed windows or the recording browser; the editor windows use `sandbox: true`, `contextIsolation: true`, and the CSP in `src/main/protocol.ts` allows `script-src 'self'` only.

Missing:

- **The Remotion bundle is fixed at package time.** `bundleRemotionProject` (`src/worker/render/bundle.ts`) runs in `packageAfterCopy`; a packaged app never bundles (`getServeUrl` only reads the prebuilt bundle). A pack's video component can't be added to that bundle later.
- No pack format, no place for packs on disk, no trust model, no version in projects beyond `version: z.literal(1)` (`src/shared/schema/workflow.ts`).
- No reusable subgraph node.

### Work

Build it in four phases, each checked before the next:

- **Phase 1, no-code nodes:** items 1–2.
- **Phase 2, packs as data and first-party packs:** items 3–6.
- **Phase 3, pack code (compile, executors, video):** items 7–9.
- **Phase 4, third-party packs:** items 10–11.

1. **Saved nodes.** "Save as reusable node" on a selection: stores the nodes, inner edges, which inner ports and settings are exposed, a name, icon and description in `<userData>/nodes/<id>.json` (or in the project, to share it). A new `core.subgraph` node type compiles by expanding its contents (pure, in `src/shared/compile`); its card and Inspector show only the exposed settings. Cache keys come from the expanded nodes, so they stay correct.
2. **Templates and presets.** "Save project as template" and per-node "Save settings as preset", both JSON in `<userData>`; the New project dialog and Inspector list them next to the built-ins.
3. **Manifest.** `pack.json`: `id` (reverse-domain), `name`, `version` (semver), `api` (the pack API version it needs), `permissions` (`network`, `files`, `browser`, `render`), and lists of nodes, elements and their entry files. A zod schema in `src/shared/packs/manifest.ts`, unit-tested.
4. **First-party packs.** Group today's `NODE_DEFS` and elements into packs with ids like `demoflow.browser`, `demoflow.camera`, `demoflow.audio`, still compiled into the app. Node types keep their ids, so no project changes.
5. **Projects record packs.** On save, the workflow lists `packs: { [id]: version }` for the node types it uses (optional field; old files load as today). `checkWorkflow` (`src/shared/api/check.ts`) and the three "isn't available" places name the missing pack and version. `normalizeWorkflow` stays the place for migrations.
6. **Cache keys.** Include the owning pack's id and version in the node hash next to `ENGINE_VERSION`, so updating a pack re-runs only its nodes.
7. **Compile and settings from a pack.** A pack's node definitions and pure `compile()` run in a locked-down context (a worker with no Node APIs, or the renderer's sandbox) and return plain data; settings schemas come as JSON Schema and are turned into the same `FieldMeta` UI.
8. **Executors from a pack.** Run in the worker's process tree as a separate `utilityProcess` per pack with only the permissions in its manifest; the worker passes files in and reads files out, like today's executors.
9. **Video components from a pack.** Since the prebuilt bundle can't grow, a pack ships its component prebuilt as an ES module that imports `react` and `remotion` from shared globals the app exposes (the same versions as the prebuilt bundle). `loadElement` gains a third source next to core and `generatedElementLoaders`: `import()` from `demoflow-file://packs/<id>/<version>/…` in the editor and from `startAssetServer` in renders, waiting with `delayRender` as now. The CSP must allow that one origin for scripts. Same rule as today: every animation is a pure function of `useCurrentFrame()`; the pack API docs say so and a test render compares two runs of the same frame.
10. **Installing.** Packs install from a file the user picks, or from a pack list fetched only when the user opens it ("Get more nodes" behind a button that says it goes online). Before any pack code runs, the app checks its signature against keys it knows or asks the user to trust it, listing its permissions. Packs live in `<userData>/packs/<id>/<version>/`; projects can pin a version.
11. **Docs.** A pack guide in `docs/` (written only when this phase starts) with the manifest, the node API and one example pack.

### Decisions for the user

- **Where third-party packs come from (decided): files only at first.** No online catalogue yet. Previously listed options: files only, a list hosted on GitHub, or nothing until later.
- **Who signs packs (decided): unsigned packs may run code after a clear warning** naming what the pack can do; signed packs (Demoflow's key or an author the user trusts) skip the warning. Previously listed options: only the user's key (first-party and reviewed packs), any author key the user trusts, or unsigned packs allowed after a warning.
- **Pack licence terms:** packs that use Remotion APIs fall under the Remotion licence for their users, like the app.

### Risks

- **Pack video code runs in the editor window.** The Player renders in the same renderer that holds `window.demoflow`; a pack component could call it. Either render pack components in a sandboxed frame without the preload bridge, or allow only signed packs to ship video code.
- **Version skew.** A pack built against one `remotion` / `react` version breaks with another; the pack `api` version must change whenever the app's pinned `remotion` changes.
- **Preview equals render** must hold for pack components too, in both Fast and Exact preview.
- **Size of the change.** Moving built-ins into packs touches every node file; do it only after the format has carried one real third-party pack.

### Sources

- Repo files named above (checked 2026-10-16). No external sources; this section is a design from an earlier discussion.
