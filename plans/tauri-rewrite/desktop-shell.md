# Desktop shell

What `src/main` and `src/preload` do today, on Tauri: windows, the title bar, menus, IPC, file protocols, the app database, keys, file watching, the exit flow, fonts and settings.

Crates: `opsis-store`, `opsis-project` and `opsis-fonts` (phase 3, steps 7–9, 11 and 12) and `opsis-desktop` (phase 6, the rest).

## What it adds

- The same app with no Electron: 286.8 MB of Electron leaves the package (`sizes.md`).
- All windows in one process: pop-out panels and Settings no longer mirror state over a `BroadcastChannel`.

## Fit with Opsis

Already in place (all in `src/main` unless noted):

- Windows: `window-registry.ts`, `window-bounds.ts`, `project-window.ts`, `panel-windows.ts`, `app-windows.ts`, `settings-window.ts`, `window-roles.ts`, `window-placement.ts`, `raise-window.ts`; the main window's size and maximized state saved per project (key `mainWindow`).
- One title bar: macOS `titleBarStyle: "hiddenInset"` with traffic lights in our bar; Windows and Linux `frame: false` with our `WindowControls` and `WindowBorder` (`src/renderer/app/app-bar.tsx`).
- Menus from `appMenu()` (`src/shared/menu.ts`): native on macOS (`menu.ts`), in-app elsewhere, accelerators handled by `useMenuShortcuts`.
- IPC: 45 request/response calls declared in `INVOKE` (`src/shared/ipc.ts`), handled in `ipc-handlers.ts`; 15 event subscriptions in `src/preload/preload.ts` (`onProjectFileChanged`, `onProjectFilesChanged`, `onRunEvent`, `onEngineInstall`, `onSettingsChanged`, `onViewPrefChanged`, `onApiStatus`, `onApiRequest`, `onLive`, `onMenuCommand`, `onPanelWindows`, `onConfirmClose`, `onFlushSave`, `onWindowState`, `onSettingsPage`).
- Protocols: `opsis-app://app/` serves the packaged UI with its CSP as a response header; `opsis-file://` serves project files and fonts (`protocol.ts`).
- App state in `<userData>/opsis.db` through `node:sqlite` and drizzle; `schema.generated.sql` from `drizzle-kit export`; the schema hash in the `meta` table; `encryptedText()` (`db/crypto.ts`).
- Project sign-ins encrypted with `safeStorage` into `<project>/secrets/` (`secrets.ts`).
- File watching of the project file and of the files the editor loads (`projects.ts`, `loaded-files-watch.ts`).
- The exit flow (`exit.ts`): `ExitDialog` in the page, `track(label, promise)`, the 2 s answer timeout and the 3 s unload timeout (`closeEvenIfPageDies`).
- Fonts: the library in `<userData>/fonts` with `fontkit` (`fonts.ts`), online fonts (`fonts-online.ts`), and fonts on the computer through Chromium's `queryLocalFonts`.

Missing:

- Tauri. `queryLocalFonts` is a Chromium API, so the computer's fonts need another source in WebKit.

## Work

1. **App and windows.** Tauri 2.12.5 (current on crates.io, 2026-10-21). The main window, the Settings window (route `#/settings?page=…`) and pop-out panels (route `#/panel/<name>`) as `WebviewWindow`s with the same labels and saved bounds. Port the window registry, bounds and placement logic to Rust.
2. **Title bar.** Windows and Linux: `decorations: false`, our `WindowControls` calling the Tauri window API (minimize, toggle maximize, close) instead of window roles, `WindowBorder` unchanged. macOS: `titleBarStyle: "Overlay"` with `trafficLightPosition` (both need `decorations: true`, per `WindowConfig`), `hiddenTitle: true`.
3. **Menus.** Rust owns the menu (today's `appMenu()` data, 180 lines, ported): the native macOS menu with `tauri::menu`, and the same list sent to the UI for the in-app menu on Windows and Linux. Menu commands reach the editor as events, as `onMenuCommand` does now.
4. **IPC.** Each of the 45 `INVOKE` calls becomes a `#[tauri::command]` typed through `tauri-specta`; each of the 15 subscriptions becomes an event or, for live frames and compile results, a `Channel`. A `window.opsis` facade in `apps/ui` implements today's `OpsisApi` over `invoke`/`listen`, so the 83 call sites in 49 renderer files keep working. Tauri capabilities limit which commands each window label may call, replacing `assertTrustedSender`.
5. **Pop-out windows.** `sync.ts` (301 lines) is replaced: pop-out windows send edits and gestures to the editor window through Rust events and receive the store snapshot the same way; the editor window still owns the store and undo.
6. **Protocols and CSP.** The UI is served by Tauri from `frontendDist`. `opsis-file` is registered with `register_asynchronous_uri_scheme_protocol` (it reads files off the UI thread). The CSP moves to the Tauri config, which appends its own hashes and nonces at build time; list `opsis-file:` in `img-src`, `media-src`, `font-src` and `connect-src` (on Windows custom schemes are served from `http://<scheme>.localhost`, as the Tauri CSP example shows for `asset:`).
7. **App database.** `rusqlite` 0.41.0 (MIT) with the bundled SQLite, WAL and foreign keys on. Keep `src/main/db/schema.ts` as the drizzle schema and `drizzle-kit export` as the generator of `schema.generated.sql`; Rust embeds that SQL and keeps today's rule (hash in `meta`, a database for another schema is replaced). Queries are hand-written SQL in Rust; the drizzle query builder leaves with Node.
8. **Keys and sign-ins.** `keyring` 4.2.0 (MIT OR Apache-2.0) with its per-platform stores: the Secret Service store on Linux, the Apple native store, the Windows native store. Small secrets (service and model keys, `plans/voice-and-captions.md` and `plans/ai-assist.md`) live in the keyring directly. Sign-in storage states are larger: encrypt them with AES-256-GCM (`aes-gcm`) using one key kept in the keyring, written to `<project>/secrets/` as today. When the platform store is missing (Linux without a Secret Service), refuse to save with today's message, as `encryptedText()` refuses on `basic_text`.
9. **File watching.** `notify` 8.2.1 (CC0-1.0) for the project folder (saves replace the file) and the loaded files; ignore the app's own writes; same `projectFilesChanged` payload (size and modified time).
10. **Exit.** `CloseRequested` is prevented and the UI is asked to show `ExitDialog`; the 2 s no-answer rule and the 3 s rule for a page that dies are reimplemented with tokio timers; `track` becomes a Rust registry of labelled tasks.
11. **Fonts.** The computer's fonts come from `fontdb` 0.24.0 (`Database::load_system_fonts`, Windows, Linux and macOS), read with `skrifa`/`read-fonts` for names, weights and italics (replacing `fontkit` in `fonts.ts`; `asset-preparation.md`). Online fonts and ZIP import (`zip` 9.1.0) port as they are.
12. **Settings, view state, recent projects, licences, model import, engine setup.** Straight ports of `settings.ts`, `view-state.ts`, `projects.ts`, `licences.ts`, `model-files.ts`, `engine-setup.ts` and `import-assets.ts`, behind the same commands.
13. **Hidden windows.** Compile no longer runs in the webview, so throttling doesn't touch compile or runs. It still touches the three jobs the editor window keeps:
    - it owns the store and undo, and autosaves 800 ms after an edit with a timer (`AUTOSAVE_DELAY_MS`, `use-autosave.ts`);
    - Local API edits under `/project` are applied by it;
    - it shows `ExitDialog` within the exit rule's 2 s.

    `WindowConfig` documents that, by default, a minimized view gets its timers throttled and can be unloaded after about 5 minutes. Set `backgroundThrottling: "disabled"` on the editor window (documented for macOS 14+ and iOS 17+), and measure on WebKitGTK whether a minimized editor still answers an API edit and saves.

    If it doesn't, move the workflow and undo into Rust. Rust already receives every change for compile; with it owning the store, the UI becomes a view, API edits never wait on a window, and nothing depends on the webview being awake.

14. **What goes away.** Canvas Draw Element switch (no video in the webview), the Linux dev `--no-sandbox` start wrapper (`scripts/start.ts`; the app has no Chromium), `electron-squirrel-startup`, Forge fuses.
15. **Tests.** Port `src/main` tests (1,002 lines); IPC contract tests check every `OpsisApi` method has a command.

## Phase 3 as built

Phase 3 (2026-10-25, branch `feat/tauri`) built the crates behind steps 7 to 9, 11 and 12, plus `capture-cdp.md` step 1 and `runner-and-executors.md` steps 7 and 8. The commands, dialogs, windows and the calls that join the crates (recent projects and view state on open and create, `lastDir`, engine setup progress) belong to `opsis-desktop` in phase 6.

- **`opsis-store`.** The app database on `rusqlite` with `src/main/db/schema.generated.sql` as its schema and its sha256 in `meta` (a database made for another schema is replaced), settings (merged per section, checked with `garde`, unreadable ones fall back to defaults), recent projects, view state, window bounds, font rows. Sign-ins are sealed with AES-256-GCM (`aes-gcm`) under a key kept in the OS keyring (`keyring-core` with the Apple, Windows and Secret Service stores; the mock store in tests).
- **`opsis-engine`.** Engine paths and status, the installer (`reqwest` streams each file and checks it against the sha256 and size in `downloads.generated.json`, written once by `bun run engine:downloads`, because neither Remotion nor Google publishes hashes; Remotion's build on macOS arm64 and on Linux with glibc 2.35 or later, Chrome for Testing elsewhere; `zip`, `flate2`; cancel with a `CancellationToken`), licence copying, child temp folders (`sysinfo` for process liveness), and the shared file and hash helpers (`tempfile` atomic writes, stable JSON for keys). Download sizes come from `humansize` ("3.0 MB of 29.0 MB", "1.5 kB").
- **`opsis-project`.** Create from `templates.generated.json` (written by `bun run examples` from the TS templates; new workflow ids from `nanoid`), open, and save with versions (`SaveResult` is `{ outcome: "saved" | "changed" }`); folder names from `slug`, `~` through `shellexpand`, paths compared lexically with `path-clean`; watching the project file and the loaded files with `notify` through `notify-debouncer-mini` (150 ms); asset and model import (`gltf`, validated, so a damaged model says so instead of failing later); file stamps, sign-in stamps, the last run, clearing the cache. The secrets folder is made with the first sign-in, not with the project.
- **`opsis-fonts`.** Review and add from files and ZIPs, replace and remove, copy into a project; `wuff` unwraps WOFF and WOFF2 and `skrifa` reads names, weights, italics and the weight axis; the computer's fonts from `fontdb` (single-face files; collections are counted as unreadable); online fonts with `reqwest`, tested against `wiremock`.
- **`opsis-traces`.** Our own format: per take a zstd JSON manifest (header, viewport, actions with their node ids, console, page errors, requests) and its frames as content-addressed JPEG blobs, retention per node as before. The model drops Playwright's call collapsing (an action has no `count`), and a request's unknown status or size is `null` rather than −1. `inject.ts` is not ported: the capture (phase 5) records frames itself.
- **Added by phase 4's audit.** `opsis-engine` serves files over HTTP with `tower-http`'s `ServeDir` behind a CORS layer (`http_files`: ranges, types, path safety, the render asset server now and the app protocol later), writes through a temporary file (`write_via_temp`), holds `PICTURE_VERSION` and lists licences from sources the desktop names (`list_licences`, `is_licence_file`). `opsis-project` works out the projects folders (`project_locations`). The store tests refusing to seal without secure storage. Errors no longer vanish: the store reports a database side file it can't remove, and pruning logs a file it can't delete.
- **Tests.** The `projects`, `loaded-files-watch`, `model-files`, `file-stamps`, `project-paths`, `fonts`, `fonts-online`, `settings`, `window-bounds`, `worker-temp`, `licences` and trace `store` tests are ported (store 19, engine 23, project 23, fonts 13, traces 7). The `examples/` were stale since node cards lost their pictures (positions, and the Title's mark settings); they and the fixtures were written again.
- **Not done here.** The phase 3 types are not in `bindings.ts` yet; they join it with their commands in phase 6. `chromium-credits.ts` stays in TS until packaging (phase 8). The trace `model.ts` parser of Playwright's events is not ported, since nothing will write that format.

## Phase 6 as built

Phase 6 (2026-10-26, branch `feat/tauri`) built `opsis-desktop` and `opsis-api` (`local-api.md`, **Phase 6 as built**), moved the editor onto them and deleted Electron, Forge, `src/`, `packages/core` and every TS duplicate of a Rust port.

- **Commands and events.** Every desktop call is a `#[tauri::command]` with tauri-specta (`crates/desktop/src/bindings.rs`), exported with its types to `apps/ui/src/generated/bindings.ts`; the `window.opsis` facade (`apps/ui/src/app/opsis.ts`) keeps the editor's call sites. Capabilities name the windows that may call them (`main`, `settings`, `panel-*`, `view-*`). Live frames, streamed views and the preview use raw-byte `Channel`s; the rest are events sent to one window or all.
- **Compile over IPC** (`compile-in-rust.md` steps 7 and 8). The editor sends each change to the async `compile` command and gets the spec, issues, the flattened timeline, the mix plan and levels, cards, the Inspector context and each bar's clock, numbered so an older answer never replaces a newer one (`seq`); panel windows ask with `send_compiled` and follow `Compiled`. The catalogue, menus and templates load once before the editor renders. The Inspector reads JSON Schema with `x-` keywords.
- **Windows** (steps 1, 2, 5). Main, Settings, pop-out panels and streamed views as `WebviewWindow`s with saved bounds and the per-project main window size; undecorated on Windows and Linux with our controls; macOS Overlay with traffic lights at 16, 16. Pop-out windows sync through `relay` events instead of a `BroadcastChannel`. The editor window disables background throttling, which only macOS 14+ honours; whether a minimized editor on WebKitGTK still answers an API edit couldn't be measured, because the tests run in a virtual display with no window manager (minimizing leaves the page visible there), and the real screen is off limits for tests.
- **Menus** (step 3). One list in Rust (`menu.rs`), sent to the editor for the in-app menu and built as the native macOS menu (`native_menu.rs`: app menu, the shared sections, Window and Help, Quit through the exit flow). It compiles and its ids are tested on Linux; it hasn't run on macOS yet.
- **Files for the editor** (step 6, changed). Not an `opsis-file` scheme: WebKitGTK can't play video or audio from a custom scheme (tauri-apps/tauri#3725, open; the GStreamer plugin from tauri-apps/tauri#14402 moved to its own repository and needs installing per machine). The desktop serves the open project and the font library over HTTP on 127.0.0.1 under a token made at each launch (`file_server.rs`, `ServeDir` with ranges), and the CSP allows `http://127.0.0.1:*`. The e2e suite plays an exported video in the Run panel.
- **Exit** (step 10). As planned, plus logs at each step. Holding the database lock while `announce` took it again hung the close after "Don't ask again"; the result is now taken out of the guard first.
- **Streamed preview.** The preview's shell gets the render's `CanvasDrawElement` feature (merged into Playwright's `--enable-features`, since Chromium keeps only the last), so canvas transitions preview as they export; an ignored test checks the page has `drawElementImage`. The master stem plays in the webview through a Web Audio chain that applies mixer drags at once.
- **Page errors.** `public/page-errors.js` and `boot.ts` send uncaught errors, rejections and console errors and warnings from every window to the Rust log. A module that read `window.opsis` while loading ran before boot in the bundled build and left the window black; the platform now comes from the user agent.
- **End-to-end tests.** `apps/e2e`: WebdriverIO 9 with `@wdio/tauri-service` and the embedded driver (`tauri-plugin-wdio-webdriver`, only with the `e2e` feature, which refuses release builds), in `xvfb-run` with Wayland unset. They open the start screen and every example, run and export the changelog, play the export, and check closing with "Don't ask again".
- **Same output.** The changelog's 1:1 export from the Tauri app is byte-identical to the Electron app's at `79d773d` (both run off-screen through the Local API, 2026-10-26). The 16:9 export matched in one run and not in the next: the bar chart's known run-to-run difference (README, **Checks when a phase is done**). Exports are written with the umask's mode (0664 here) like before; atomic writes had made them 0600.
- **Sizes.** The release binary is 93.4 MB on Linux x64 (69.8 MB stripped) with the editor and `public/` embedded (`sizes.md`).
- **Not done here.** WebKitGTK is the only engine the editor has run on; WKWebView and WebView2 wait for the phase 8 CI jobs, as does the Windows pipe ACL (`local-api.md`). The trace `model.ts` parser of Playwright's events was not ported (nothing writes that format). Packaging is phase 8.

## Risks

- **WebKitGTK and WKWebView differ from Chromium** for the editor UI itself: CSS, focus and pointer events in React Flow, the colour picker and drag-and-drop. Run the editor on all three engines as soon as `opsis-desktop` opens a window and list differences; phase 6 ran it on WebKitGTK only (the e2e suite), so WKWebView and WebView2 come with the phase 8 CI jobs.
- **Linux keyrings.** Users without a Secret Service can't save keys, as today with `basic_text`.

## Sources

- crates.io API for `tauri`, `rusqlite`, `keyring`, `notify`, `fontdb`, `zip` (2026-10-25).
- [tauri `Builder`: `register_uri_scheme_protocol`, `register_asynchronous_uri_scheme_protocol`](https://docs.rs/tauri/latest/tauri/struct.Builder.html), [`WindowConfig`](https://docs.rs/tauri-utils/latest/tauri_utils/config/struct.WindowConfig.html) (`traffic_light_position` needs Overlay and decorations; `background_throttling` macOS 14+), [Tauri CSP](https://v2.tauri.app/security/csp/), [calling the frontend](https://v2.tauri.app/develop/calling-frontend/) (2026-10-25).
- [keyring on docs.rs](https://docs.rs/crate/keyring/latest) (stores in separate crates), [fontdb `Database`](https://docs.rs/fontdb/latest/fontdb/struct.Database.html) (2026-10-25).
- `src/main/*`, `src/preload/preload.ts`, `src/shared/ipc.ts` (read 2026-10-25).
