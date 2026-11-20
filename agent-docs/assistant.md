# Assistant

How Opsis works with coding agents: the Assistant switch that serves the Local API, and the headless mode agents drive. Rust paths are relative to `crates/` unless they name another folder. The Local API's routes are in [api.md](api.md), the scripts agents run in [scripting.md](scripting.md), starting and closing the editor in [editor.md](editor.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern            | Where                                                                                                                                                                                          |
| ------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| The switch         | `types/src/settings.rs` → `AssistantSection`; `session/src/api.rs` → `follow_setting`; `qt/cpp/settings_window.cc` → `AssistantPage`; `qt/cpp/window_chrome.cc` → `AppBar::SyncAssistantBadge` |
| Headless arguments | `ui/src/main.rs` → `headless_folder`, `USAGE`                                                                                                                                                  |
| Headless start     | `qt/cpp/app.cc` → `App::Run`; `session/src/headless.rs` → `open_project`; `session/src/api.rs` → `open_and_show`                                                                               |
| Headless answers   | `session/src/platform.rs` → `Platform::headless`; `ui/src/platform.rs` → `QtPlatform`; `session/src/document.rs` → `ask_about_disk`; `session/src/exit.rs` → `confirm`; `App::ShowError`       |
| Tests              | `session/tests/headless.rs`; `ui/tests/views.rs` → `a_headless_start_serves_the_project_and_ends_on_a_signal`; `ui/src/platform.rs` and `ui/src/main.rs` tests                                 |

## Rules

- **One switch, Settings → Assistant (`settings.assistant.enabled`, off by default), serves the Local API with every route** ([api.md](api.md)); agents change the project through Lua, so there is no separate switch for scripts.
- **`opsis --headless <project folder>` is the whole app with no window, for agents to drive**: `App::Run` sets Qt's `offscreen` platform from code before `QApplication` is made, `MainWindow::StartAsChosen` skips the start choice, and the session opens the given folder once it begins (`open_project`). A folder that can't be opened ends the app with code 1; any other argument list prints the usage and exits with code 2.
  - Headless always serves the Local API, whatever the switch says (`follow_setting`), since an agent is its only user.
  - It runs until a signal ends it; the signal saves the project and stops a run (`close_on_signals`).
- **The flag reaches the session as `Platform::headless`, so everything that would wait for a person answers by itself**:
  - The disk-change question answers **Load the file** (Undo brings the app's version back), or **Wait** when the file isn't a valid project yet (`ask_about_disk`).
  - Closing while work runs stops the work and exits, saving first (`confirm`).
  - File and folder pickers reply as cancelled, and notices are logged (`QtPlatform`).
  - Error message boxes are written to the log (`App::ShowError`).

## Deliberate limits

- **Headless on Windows ends only by ending the process**: `close_on_signals` listens on Unix only, so nothing saves the project first.
- **Dialogs a person opens with a click (Quick add, New project, renaming a script) have no headless answer**: with no window to click, nothing opens them.
