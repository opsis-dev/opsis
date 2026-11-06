# End-to-end tests

The suite that drives the real app through WebdriverIO: how the app is built for it, the virtual display it requires, the data folder each run gets, and how specs share one project. Paths are relative to the repository root. See also [AGENTS.md](../AGENTS.md) §4 and §6.

## Code map

| Concern                    | Where                                                                                                        |
| -------------------------- | ------------------------------------------------------------------------------------------------------------ |
| Suite                      | `e2e/` (own `package.json`, `bun.lock`, `node_modules`; `bun install` inside it once)                        |
| Config                     | `e2e/wdio.conf.ts`                                                                                           |
| Specs and support          | `e2e/src/specs/`, `e2e/src/support/`                                                                         |
| Data folder seed           | `e2e/src/support/user-data.ts`                                                                               |
| Close check                | `e2e/src/close-check.ts`                                                                                     |
| e2e feature and capability | `crates/desktop/src/lib.rs` (`compile_error!`), `crates/desktop/capabilities/e2e`, `crates/desktop/build.rs` |
| Shared configs by path     | `tsconfig.base.json`, `scripts/lib/oxlint-base.ts`, `scripts/lib/oxfmt-base.ts`                              |

## Rules

- **`bun run test:e2e` runs `e2e:app`, which builds the app with the `e2e` feature, then the suite.**
- **The app is driven through `@wdio/tauri-service` with its embedded driver, `tauri-plugin-wdio-webdriver`**, compiled in only with the `e2e` Cargo feature. The feature refuses to build without debug assertions and adds the `wdio-webdriver` capability.
- **`build.rs` picks the e2e capabilities and tells cargo to watch the `capabilities` folder**, because with its own pattern `tauri_build` watches only `tauri.conf.json` and an edited capability isn't compiled in.
- **The build goes to `target/e2e`, so `cargo test` never overwrites it.**
- **The tests run in a virtual display only**: `xvfb-run` with `WAYLAND_DISPLAY` unset and `GDK_BACKEND=x11` (otherwise GTK opens the window on the desktop's Wayland display); `wdio.conf.ts` and `close-check.ts` refuse to start without it.
- **`bun run e2e:qt` drives the native editor like a person** (`scripts/e2e-qt.ts`, steps in `crates/qt/cpp/e2e_steps.cc`): its own Xvfb on a free display, a fresh data folder in `~/.cache/opsis-qt-e2e`, the staged FFmpeg (`OPSIS_FFMPEG`), no screenshots (`OPSIS_QT_SHOTS=off`), and a failure on any failed check, crash or timeout. CI runs it in the Checks job.
  - A scenario run uses Qt's own file dialogs (`Qt::AA_DontUseNativeDialogs`, `App::Run` in `crates/qt/cpp/app.cc`), since a desktop's native dialog can't be driven; every other run keeps the native ones.
  - Every step fails when a panel is wider than its dock (`Scenario::Overflow`, `crates/qt/cpp/scenario.cc`): text that should wrap and controls sized to their longest choice show up there.
  - A step finds what it acts on as a person would (a button's text or tooltip, a menu's label, the card or port under the pointer) and acts with real mouse and key events (`QTest`), never by calling the code behind the control. The one exception is a drop from the file manager, sent as a drop event.
  - A step's check reads the result from the session (the graph, the Inspector, the settings, the run), not from the widget it touched; a step that can't find its control fails by name (`Scenario::Miss`).
  - A click that opens a modal dialog goes through `ClickLater`, or the step blocks in the dialog's own loop; a shortcut step activates the window first, since Xvfb has no window manager to hand focus back after a dialog.
- **A Qt program in the virtual display needs `env -u WAYLAND_DISPLAY QT_QPA_PLATFORM=xcb` besides `DISPLAY`**: Qt picks its Wayland plugin whenever `WAYLAND_DISPLAY` is set and ignores `DISPLAY`, so the window opens on the desktop. The Qt editor's scenario mode (`OPSIS_QT_SCENARIO`, `crates/qt/cpp/scenario.cc`) refuses to run on any plugin but `xcb` or `offscreen`.
- **Each run gets a fresh data folder in `~/.cache/opsis-e2e`** (`OPSIS_USER_DATA`): the database seeded with the API on and the dev engine's recording browser as the `settings.engine` path, `examples/` copied, and `OPSIS_FFMPEG` set to the staged FFmpeg. Failure screenshots go to `~/.cache/opsis-e2e/failures`.
- **The app checks the engine at start, so specs that record or export wait for it (`engineReady`).**
- **The virtual display hides the window but not the sound, so a spec mutes any player it presses Play on.**
- **The specs share one project folder and the app's selection, so a spec deletes the nodes it adds** (the Run panel lists the selected node's files, not the exports).
- **The close check runs on its own after the suite**, because closing the app ends the driver.
- **The suite is checked only by its own `typecheck`, `lint` and `format` scripts**, which CI runs after the root checks; the root checks skip it.
