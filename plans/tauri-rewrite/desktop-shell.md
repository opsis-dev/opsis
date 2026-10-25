# Desktop shell

What `src/main` and `src/preload` do today, on Tauri: windows, the title bar, menus, IPC, file protocols, the app database, keys, file watching, the exit flow, fonts and settings.

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
4. **IPC.** Each of the 45 `INVOKE` calls becomes a `#[tauri::command]` typed through `tauri-specta`; each of the 15 subscriptions becomes an event or, for live frames and compile results, a `Channel`. A `window.opsis` facade in `packages/ui` implements today's `OpsisApi` over `invoke`/`listen`, so the 83 call sites in 49 renderer files keep working. Tauri capabilities limit which commands each window label may call, replacing `assertTrustedSender`.
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

## Risks

- **WebKitGTK and WKWebView differ from Chromium** for the editor UI itself: CSS, focus and pointer events in React Flow, the colour picker and drag-and-drop. Run the editor on all three engines early (phase 2) and list differences.
- **Linux keyrings.** Users without a Secret Service can't save keys, as today with `basic_text`.

## Sources

- crates.io API for `tauri`, `rusqlite`, `keyring`, `notify`, `fontdb`, `zip` (2026-10-25).
- [tauri `Builder`: `register_uri_scheme_protocol`, `register_asynchronous_uri_scheme_protocol`](https://docs.rs/tauri/latest/tauri/struct.Builder.html), [`WindowConfig`](https://docs.rs/tauri-utils/latest/tauri_utils/config/struct.WindowConfig.html) (`traffic_light_position` needs Overlay and decorations; `background_throttling` macOS 14+), [Tauri CSP](https://v2.tauri.app/security/csp/), [calling the frontend](https://v2.tauri.app/develop/calling-frontend/) (2026-10-25).
- [keyring on docs.rs](https://docs.rs/crate/keyring/latest) (stores in separate crates), [fontdb `Database`](https://docs.rs/fontdb/latest/fontdb/struct.Database.html) (2026-10-25).
- `src/main/*`, `src/preload/preload.ts`, `src/shared/ipc.ts` (read 2026-10-25).
