# The editor

The native editor's windows, docks and panels, the title bar and menus, the canvas, its look and motion, starting and closing, the Settings window and per-project view state. C++ paths are relative to `crates/qt/cpp/`, Rust paths to `crates/ui/src/` unless they name another crate. How C++ and Rust talk is in [ui.md](ui.md), how windows share the document in [document.md](document.md), the Timeline in [timeline.md](timeline.md), the database in [app-state.md](app-state.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                     | Where                                                                                                                               |
| --------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| Main window and its screens | `main_window.cc` → `MainWindow` (start screen, setup, editor), `StartAsChosen`; `start_screen.cc`, `engine_card.cc` → `SetupScreen` |
| Docks and panels            | `main_window.cc` → `BuildDocks`, `MakeDock`; `node_list.cc` (`NodeList`, `CategoryRail`); `dock_style.cc`                           |
| Title bar and window frame  | `window_chrome.cc` → `AppBar`, `WindowChrome`; `top_bar.cc` → `TopBar`                                                              |
| Menus                       | `crates/session/src/menu.rs` → `app_menu`; `menu.rs` → `menu`; `main_window.cc` → `BuildMenus`, `Dispatch`, `RunRole`               |
| Canvas                      | `graph_canvas.cc` → `GraphCanvas`, `CardItem`, `EdgeItem`, `Minimap`; `graph.rs` (cards, edges, lanes, active nodes)                |
| Tidy up                     | `crates/compile/src/layout.rs` → `auto_layout`; `ui.rs` → `tidy_layout`                                                             |
| Edge lanes                  | `crates/compile/src/lanes.rs` → `step_positions`                                                                                    |
| Look and motion             | `crates/ui/resources/style.qss`, `look.rs` → `TOKENS`; `look_style.cc`; `motion.cc` → `ReducedMotion`, `FadeIn`, `FadeOut`          |
| Players and pictures        | `file_player.cc` → `FilePlayer`, `ImagePreview`; `player.rs`                                                                        |
| Closing                     | `main_window.cc` → `ShowCloseDialog`; `crates/session/src/exit.rs` (`close_shown`, `decide_close`, `close_on_signals`)              |
| Settings window             | `settings_window.cc` → `SettingsWindow`; `fonts_page.cc`, `engine_card.cc`                                                          |
| View state                  | `ui.rs` → `load_layout`, `save_layout`, `project_window`, `save_window`; `crates/session/src/windows.rs`                            |

## Rules

### Windows and docks

- **The main window holds three screens in one stack**: the start screen, setup (shown while the engine is missing and setup wasn't skipped) and the editor. Settings, Replay, the streamed views, the Live window and the terminal window are windows of their own.
- **The panels are KDDockWidgets docks**: the node list on the left, the Inspector on the right, and Timeline, Mixer and Run as tabs of one bottom dock (40% of the height), Timeline first.
- **Any dock pops out into its own window and docks back**; the arrangement is saved per project when the project closes.
- **The node list, Inspector and bottom panel collapse from the View menu, its shortcuts and the top bar**; a closed node list leaves a 44 px category rail (`CategoryRail::kWidth`) whose categories open as popovers.
- **Files play in `FilePlayer`**: videos and sounds in the Run panel and Replay, and a made sound below its node's settings in the Inspector. Images open full size in `ImagePreview`.

### Title bar and menus

- **The window has no system title bar** (`Qt::FramelessWindowHint`, `WindowChrome`): `AppBar` is the drag handle, starting the system move once the pointer has moved 3 px (`kDragStartPx`), and a double-click toggles maximize. A 1 px border shows while the window is neither maximized nor full screen.
- **Menus come from one list, `app_menu` in `opsis-session`**, which `menu.rs` turns into the bar's sections and shortcuts. A command goes to `Dispatch`, which handles the window's own and sends the rest to `menu_command`; standard edits go to `RunRole`.

### Canvas

- **`settings.editor.canvasMode` is `mouse` (the wheel zooms, dragging the background pans) or `touchpad` (two fingers pan, a pinch zooms)**, chosen in the canvas toolbar and kept for every project.
- **The grid and minimap toggles sit in their own toolbar cards** and show their state with the selected overlay.
- **The canvas mirrors the session's whole selection**: a selection made elsewhere (the Run panel, Replay, the Local API) selects exactly those cards, so clicking a card always changes what is selected.
- **Tidy up lays the cards out in Rust** (`auto_layout`) from each card's shape, as one undo step.
- **Edges entering one column turn in their own lane where their runs overlap** (`step_positions`).
- **The nodes playing at the playhead are ringed and their edges move as dashes**; picking one node moves the playhead to the start of its span. The dashes stand still with reduced motion.

### Look and motion

- **Every colour is a token in `look.rs`**, resolved into `style.qss`; panels, menus and the top bar sit on the sidebar colour, the canvas on the background colour.
- **Motion respects the system's reduced motion setting** (`ReducedMotion`): fades and slides become instant.

### Starting and closing

- **Settings → General picks what launch shows**: the start screen, the last project or New project (`StartAsChosen`), once per launch and only when setup isn't showing.
- **Closing the main window asks first unless the user said not to**: "Are you sure you want to exit?" with "Don't ask again". Unfinished work (a run, engine setup, a font download, unsaved changes, a value still being typed) is always listed, and the button becomes "Close anyway".
- **Closing hands held values over before saving**: the session asks for them (`FlushDrafts`), the Inspector ends its typing, and Rust saves.
- **The dialog tells Rust it is up as soon as it shows (`close_shown`); if nothing does within 2 s, Rust saves and closes without asking**, so a broken window can always close.
- **SIGTERM, SIGINT and SIGHUP save and close like a quit** (`close_on_signals`, Unix only).

### Settings window

- **Settings is a window of its own with a page list**, opened from the menu, its shortcut and any Settings button; it is kept between openings and closes with the main window.
- **Settings are written one section at a time (`patch_settings`)**, so windows never overwrite each other, and every view follows `SettingsChanged`.
- **A notice from a Settings page shows in the Settings window** (`ShowToast`), not over the canvas.
- **Saved results (clear cache) belong to the project, in the project Inspector.**

### View state

- **View state is per project, on this computer, in the database's `view_state` table**: the dock arrangement (`nativeDockLayout`) and the main window's place and size (`mainWindow`).
- **A project with no saved window opens maximized; the start screen leaves the window as it is.**
- **Nothing is saved while no project is open.**
