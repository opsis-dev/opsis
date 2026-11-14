# End-to-end tests

The scenarios that drive the real editor: what each covers, the virtual display or headless Wayland it runs in, the data folder it gets, the offscreen view tests, and how a step acts and checks. Paths are relative to the repository root. See also [AGENTS.md](../AGENTS.md) §4 and [ui.md](ui.md).

## Code map

| Concern                 | Where                                                                                                                                                                                                                              |
| ----------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Runner                  | `scripts/e2e-qt.ts` (`bun run e2e:qt`)                                                                                                                                                                                             |
| Scenario engine         | `crates/qt/cpp/scenario.cc` → `Scenario` (`Add`, `Until`, `Next`, `Finish`, `Overflow`); `scenario.h` → `Miss`; `e2e_steps.cc` → `Find`, `Click`, `ClickLater`, `ClickAt`, `ClickMenu`                                             |
| Steps                   | `crates/qt/cpp/e2e_*.cc`: `e2e_steps.cc` (tour) with its parts `e2e_events.cc`, `e2e_player.cc`, `e2e_settings.cc`; `e2e_runs.cc` with `e2e_ports.cc` and `e2e_terminal.cc` (runs); `e2e_api.cc`; `e2e_views.cc`; `e2e_wayland.cc` |
| Native dialogs off      | `crates/qt/cpp/app.cc` → `App::Run`                                                                                                                                                                                                |
| CI jobs                 | `.github/workflows/build.yml` ([packaging.md](packaging.md))                                                                                                                                                                       |
| The views in packages   | `scripts/package-check.ts`                                                                                                                                                                                                         |
| The app by hand         | `.claude/skills/check-app`                                                                                                                                                                                                         |
| API checks from outside | `scripts/lib/e2e-api.ts` → `runApiChecks`                                                                                                                                                                                          |
| Real Wayland input      | `crates/e2e-input` (`opsis-e2e-input`); the requests it takes in `crates/e2e-input/src/main.rs`                                                                                                                                    |
| Offscreen view tests    | `crates/ui/tests/views.rs`                                                                                                                                                                                                         |
| Data folder seed        | `scripts/lib/user-data.ts` → `seedUserData`                                                                                                                                                                                        |

## Rules

- **`bun run e2e:qt` drives the editor like a person.**
  - It runs the app on its own Xvfb on a free display (headless sway for `wayland`), with the staged FFmpeg (`OPSIS_FFMPEG`) and sound sent to the system's null output (`OPSIS_AUDIO_DEVICE=alsa:null`, so nothing plays on the speakers while playback still keeps time).
  - Each scenario gets its data folder through `OPSIS_USER_DATA`; `OPSIS_QT_PROJECTS` names the projects folder of `runs`, `wayland` and `views`, `OPSIS_QT_FIXTURES` the tour's generated clip and sample font, and `OPSIS_QT_PAGE` the test page on localhost that `tour`, `runs` and `api` open.
  - It takes no screenshots unless `--shots` is given (`bun run e2e:qt tour --shots` empties `~/.cache/opsis-dev/qt-shots/<scenario>` and writes one picture per step there), by setting `OPSIS_QT_SHOTS` to `off` or that folder. A scenario started by hand without `OPSIS_QT_SHOTS` writes its shots to `~/.cache/opsis-dev/qt-shots/<name>` (`scenario.cc`).
  - It fails on any failed check, crash, non-zero exit or timeout (a scenario still running after `RUN_TIMEOUT_MS` is stopped), and writes each scenario's full log to `target/e2e-qt/<scenario>.log`.
  - It needs `Xvfb` for every scenario but `wayland`, `sway` for `wayland`, `fc-match` for the tour's sample font and, with `OPSIS_QT_E2E_HANG_MS`, `gdb`.
  - It builds `opsis-ui`, `opsis-e2e-input` and the renderer program (`opsis-effects`) first.
  - A scenario passes only if its log has the `scenario <name> done` line (`Scenario::Next`).
- **The runner takes the first free X display from `FIRST_DISPLAY` to `LAST_DISPLAY` (`freeDisplay`)** and stops when none is free.
- **The runner has four scenarios, all by default or the ones named** (`bun run e2e:qt tour`), and refuses any other name:
  - `tour` (`e2e_steps.cc` and the parts it builds) starts a fresh `~/.cache/opsis-dev/qt-e2e` with the dev engine linked, the API off and a generated test clip, and walks every panel, field and window.
  - `runs` (`e2e_runs.cc`, `e2e_ports.cc`, `e2e_terminal.cc`) starts `~/.cache/opsis-dev/qt-e2e-runs` without an engine, so it checks the setup screen, skips it and chooses the dev engine's browser on the Engine settings page through the file dialog (`OPSIS_QT_BROWSER`), then records, exports, replays and records terminal takes against a test page on localhost (`OPSIS_QT_PAGE`).
  - `api` (`e2e-api.ts`, `e2e_api.cc`) starts `~/.cache/opsis-dev/qt-e2e-api` with the API and Run Lua scripts on and drives the app from outside over its socket, through every route in `CALLED_ROUTES` (`scripts/lib/e2e-api.ts`), and checks the OpenAPI document lists each of them.
  - `wayland` (`e2e_wayland.cc`) starts `~/.cache/opsis-dev/qt-e2e-wayland` with the API off, needs no engine and runs on headless sway with real input.
- **The app has two more scenarios the runner doesn't run**: `views` (`e2e_views.cc`), run by `cargo test -p opsis-ui`, and `bench` (`OPSIS_QT_SCENARIO=bench`, with `OPSIS_QT_BENCH_NODES` nodes), which builds a generated project and logs how long generating, dragging and undoing take. Set by hand, any other name runs the tour (`scenario.cc`).
- **`runs` stops at once without the dev engine in `~/.config/Opsis/engine`, and `tour` and `api` link it when it is there.**
- **CI runs `wayland` in the Checks job (with `sway` installed) and the other three in the E2E job, which installs the engine.**
- **The `api` scenario ends with the close check**: once the script's checks are done it writes the file named by `OPSIS_QT_HANDOFF`, and the in-app half checks the canvas shows what the API changed, then closes through the exit dialog with Don't ask again ticked. The script then requires exit code 0 and `confirmExit: false` in the database.
- **`wayland` sends real input through the compositor, not `QTest`, to every control it tests**, because on Wayland a popup (a menu, a combo's list, the colour picker) opens only for input the compositor delivered, and focus, the clipboard and hover need a seat with devices.
  - Setup it isn't testing goes the usual way: skipping setup is a `QTest` click, and making the project, adding a node and selecting go through the session.
  - The script starts headless sway (`WLR_BACKENDS=headless`, no libinput devices) in a private runtime folder, `$XDG_RUNTIME_DIR/opsis-e2e-<pid>`, then `opsis-e2e-input`, which holds one virtual pointer and one virtual keyboard for the whole run and carries out the requests a step appends to `OPSIS_QT_INPUT`.
  - The devices must stay attached: a client gets pointer and key events only after binding them, so a pointer made and dropped per click, as `wlrctl` does, never reaches the app.
  - A step reports the last press and key the app received when it fails, so a lost event shows apart from a broken control.
- **`cargo test -p opsis-ui` runs the `views` and `reopen` scenarios on Qt's offscreen platform**: widget checks that need no display, engine or network, so `bun run test` covers them.
  - `reopen` (`BuildReopen`, `e2e_views.cc`) is the one check across two launches: the test runs the app twice on one data folder, and the scenario tells the launches apart by whether its projects folder has a project yet. The first creates one, pops the Timeline out, saves that arrangement with the Script tab taken out of it, as an arrangement saved before the tab existed, and closes the project through the session so nothing saves over it; the second opens it again and checks the Timeline is still floating and the Script tab is back in the bottom group.
  - The test gives the app `LIMIT` to finish and its own data and projects folders, no screenshots and the null sound output, removes `WAYLAND_DISPLAY` and `DISPLAY`, and fails unless the app exits cleanly, logs `scenario <name> done` and logs no failed check (`run_scenario`, `crates/ui/tests/views.rs`); `scripts/package-check.ts` runs the same scenario in the installed packages.
- **`OPSIS_QT_E2E_HANG_MS` runs the app under gdb**: when the app is still going after that many milliseconds, the script interrupts it and gdb writes every thread's backtrace into the scenario's log, which is how a freeze is caught.
- **A scenario never runs on the desktop**: the scenario mode (`OPSIS_QT_SCENARIO`) refuses any plugin but `xcb`, `offscreen`, or `wayland` inside a private `opsis-e2e-` runtime folder (`scenario.cc`).
- **A Qt program in the virtual display needs `env -u WAYLAND_DISPLAY QT_QPA_PLATFORM=xcb` besides `DISPLAY`**, because Qt picks its Wayland plugin whenever `WAYLAND_DISPLAY` is set and ignores `DISPLAY`; the runner does it in `startXvfb`, the check-app skill by hand.
- **A step finds what it acts on as a person would** (a button's text or tooltip, a menu's label, the card or port under the pointer) and acts with real mouse and key events (`QTest`, or the compositor's in `wayland`), never by calling the code behind the control. The exceptions are a drop from the file manager, sent as a drop event, and setup a step isn't testing (making a project, setting the selection, opening a Settings page), which calls the session or window directly.
- **A step checks what it changed in the project, the settings or a run through the session** (the graph, the Inspector, the settings, the run), not through the widget it touched; a view's own state (a popup, a dialog, a label, a checked button) is read from the widget.
- **A step that can't find its control fails by name (`Scenario::Miss`).**
- **A step waits a fixed time (`Scenario::Add`) or polls a condition until a timeout (`Scenario::Until`)** for anything that takes as long as the machine makes it: an engine check, a streamed page, a run, a menu opening.
- **A click that opens a modal dialog goes through `ClickLater`**, or the step blocks in the dialog's own loop; a shortcut step activates the window first, since Xvfb has no window manager to hand focus back after a dialog.
- **A scenario run uses Qt's own file dialogs** (`Qt::AA_DontUseNativeDialogs`, `App::Run`), since a desktop's native dialog can't be driven; every other run keeps the native ones.
- **Every step fails when a panel is wider than its dock** (`Scenario::Overflow`, which checks the scroll areas that hide their horizontal bar): text that should wrap and controls sized to their longest choice show up there.

## Deliberate limits

- **`wayland` clicks only in the main window**: sway tiles it at the output's origin, while a Wayland client can't learn where a dialog or floating window sits, so those are driven by keys.
