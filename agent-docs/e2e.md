# End-to-end tests

The scenarios that drive the real editor: what each covers, the virtual display or headless Wayland it runs in, the data folder it gets, the offscreen view tests, and how a step acts and checks. Paths are relative to the repository root. See also [AGENTS.md](../AGENTS.md) §4 and [ui.md](ui.md).

## Code map

| Concern                 | Where                                                                                                                                                |
| ----------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| Runner                  | `scripts/e2e-qt.ts` (`bun run e2e:qt`)                                                                                                               |
| Scenario engine         | `crates/qt/cpp/scenario.cc` → `Scenario` (`Add`, `Until`, `Miss`, `Overflow`, `ClickLater`)                                                          |
| Steps                   | `crates/qt/cpp/e2e_*.cc`: `e2e_steps.cc` (tour), `e2e_runs.cc`, `e2e_ports.cc`, `e2e_api.cc`, `e2e_views.cc`, `e2e_wayland.cc`, and the tour's parts |
| API checks from outside | `scripts/lib/e2e-api.ts` → `runApiChecks`                                                                                                            |
| Real Wayland input      | `crates/e2e-input` (`opsis-e2e-input`)                                                                                                               |
| Offscreen view tests    | `crates/ui/tests/views.rs`                                                                                                                           |
| Data folder seed        | `scripts/lib/user-data.ts` → `seedUserData`                                                                                                          |

## Rules

- **`bun run e2e:qt` drives the editor like a person**: its own Xvfb on a free display (headless sway for `wayland`), the staged FFmpeg (`OPSIS_FFMPEG`), sound sent to the system's null output (`OPSIS_AUDIO_DEVICE=alsa:null`, so nothing plays on the speakers while playback still keeps time), no screenshots unless `--shots` is given (`bun run e2e:qt tour --shots` empties `~/.cache/opsis-qt-shots/<scenario>` and writes one picture per step there), and a failure on any failed check, crash, non-zero exit or timeout. Each scenario's full log is written to `target/e2e-qt/<scenario>.log`.
- **Four scenarios, all by default or the ones named** (`bun run e2e:qt tour`):
  - `tour` (`e2e_steps.cc` and the parts it builds) starts a fresh `~/.cache/opsis-qt-e2e` with the dev engine linked and a generated test clip, and walks every panel, field and window.
  - `runs` (`e2e_runs.cc`, `e2e_ports.cc`) starts `~/.cache/opsis-qt-e2e-runs` without an engine, so it checks the setup screen, skips it and chooses the dev engine's browser on the Engine settings page through the file dialog (`OPSIS_QT_BROWSER`), then records, exports, replays and records terminal takes against a test page on localhost (`OPSIS_QT_PAGE`).
  - `api` (`e2e-api.ts`, `e2e_api.cc`) starts `~/.cache/opsis-qt-e2e-api` with the API on and drives the app from outside over its socket, through every route a client uses (open, edits, undo and redo, settings, variables, save, problems, validate, a run followed through its events and a stopped run).
  - `wayland` (`e2e_wayland.cc`) needs no engine and runs on headless sway with real input.
- **`tour`, `runs` and `api` need the dev engine in `~/.config/Opsis/engine`**; CI runs `wayland` in the Checks job (with `sway` installed) and the other three in the E2E job, which installs the engine.
- **The `api` scenario ends with the close check**: once the script's checks are done it writes the file named by `OPSIS_QT_HANDOFF`, and the in-app half checks the canvas shows what the API changed, then closes through the exit dialog with Don't ask again ticked. The script then requires exit code 0 and `confirmExit: false` in the database.
- **`wayland` sends real input through the compositor, not `QTest`**, because on Wayland a popup (a menu, a combo's list, the colour picker) opens only for input the compositor delivered, and focus, the clipboard and hover need a seat with devices.
  - The script starts headless sway (`WLR_BACKENDS=headless`, no libinput devices) in a private runtime folder, `$XDG_RUNTIME_DIR/opsis-e2e-<pid>`, then `opsis-e2e-input`, which holds one virtual pointer and one virtual keyboard for the whole run and carries out the requests a step appends to `OPSIS_QT_INPUT` (`click`, `move`, `press`, `release`, `key ctrl+z`, `type …`).
  - The devices must stay attached: a client gets pointer and key events only after binding them, so a pointer made and dropped per click, as `wlrctl` does, never reaches the app.
  - A step reports the last press and key the app received when it fails, so a lost event shows apart from a broken control.
- **`cargo test -p opsis-ui` runs the `views` scenario on Qt's offscreen platform**: widget checks that need no display, engine or network, so `bun run test` covers them.
- **`OPSIS_QT_E2E_HANG_MS` runs the app under gdb**: when the app is still going after that many milliseconds, the script interrupts it and gdb writes every thread's backtrace into the scenario's log, which is how a freeze is caught.
- **A scenario never runs on the desktop.** Qt picks its Wayland plugin whenever `WAYLAND_DISPLAY` is set and ignores `DISPLAY`, so a Qt program in the virtual display needs `env -u WAYLAND_DISPLAY QT_QPA_PLATFORM=xcb` besides `DISPLAY`; the scenario mode (`OPSIS_QT_SCENARIO`) refuses any plugin but `xcb`, `offscreen`, or `wayland` inside a private `opsis-e2e-` runtime folder.
- **A step finds what it acts on as a person would** (a button's text or tooltip, a menu's label, the card or port under the pointer) and acts with real mouse and key events (`QTest`), never by calling the code behind the control. The one exception is a drop from the file manager, sent as a drop event.
- **A step's check reads the result from the session** (the graph, the Inspector, the settings, the run), not from the widget it touched; a step that can't find its control fails by name (`Scenario::Miss`).
- **A step waits a fixed time (`Scenario::Add`) or polls a condition until a timeout (`Scenario::Until`)** for anything that takes as long as the machine makes it: an engine check, a streamed page, a run, a menu opening.
- **A click that opens a modal dialog goes through `ClickLater`**, or the step blocks in the dialog's own loop; a shortcut step activates the window first, since Xvfb has no window manager to hand focus back after a dialog.
- **A scenario run uses Qt's own file dialogs** (`Qt::AA_DontUseNativeDialogs`, `App::Run`), since a desktop's native dialog can't be driven; every other run keeps the native ones.
- **Every step fails when a panel is wider than its dock** (`Scenario::Overflow`): text that should wrap and controls sized to their longest choice show up there.

## Deliberate limits

- **`wayland` clicks only in the main window**: sway tiles it at the output's origin, while a Wayland client can't learn where a dialog or floating window sits, so those are driven by keys.
