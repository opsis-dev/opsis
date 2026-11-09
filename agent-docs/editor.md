# The editor

The native editor's windows, docks and panels, the title bar and menus, the canvas, its look and motion, starting and closing, the Settings window and per-project view state. C++ paths are relative to `crates/qt/cpp/`, Rust paths to `crates/ui/src/` unless they name another crate. How C++ and Rust talk is in [ui.md](ui.md), how windows share the document in [document.md](document.md), the Timeline in [timeline.md](timeline.md), the Inspector in [inspector.md](inspector.md), the About page's Updates card in [updates.md](updates.md), the database in [app-state.md](app-state.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                     | Where                                                                                                                                                                                                            |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Main window and its screens | `main_window.cc` → `MainWindow` (start screen, setup, editor), `StartAsChosen`; `start_screen.cc`, `engine_card.cc` → `SetupScreen`                                                                              |
| Docks and panels            | `main_window.cc` → `BuildDocks`, `MakeDock`; `node_list.cc` (`NodeList`, `CategoryRail`); `dock_style.cc`                                                                                                        |
| Title bar and window frame  | `window_chrome.cc` → `AppBar`, `WindowChrome`; `top_bar.cc` → `TopBar`                                                                                                                                           |
| Menus                       | `crates/session/src/menu.rs` → `app_menu`; `menu.rs` → `menu`; `main_window.cc` → `BuildMenus`, `Dispatch`, `RunRole`                                                                                            |
| Canvas                      | `graph_canvas.cc` → `GraphCanvas`, `CardItem`, `EdgeItem`, `Minimap`; `graph.rs` (cards, edges, lanes, active nodes)                                                                                             |
| Tidy up                     | `crates/compile/src/layout.rs` → `auto_layout`; `ui.rs` → `tidy_layout`                                                                                                                                          |
| Edge lanes                  | `crates/compile/src/lanes.rs` → `step_positions`                                                                                                                                                                 |
| Look and motion             | `crates/ui/resources/style.qss`, `look.rs` → `TOKENS`; `look_style.cc` → `LookStyle`, `FocusRing`; `motion.cc` → `ReducedMotion`, `MotionMs`, `EaseOut`, `FadeIn`, `FadeOut`, `GrowWidth`; `widgets.h` → `Combo` |
| Docks                       | `dock_style.cc` → `DockTitleBar`; KDDockWidgets flags in `app.cc`                                                                                                                                                |
| Players and pictures        | `file_player.cc` → `FilePlayer`, `ImagePreview`; `player.rs`                                                                                                                                                     |
| Closing                     | `main_window.cc` → `ShowCloseDialog`; `crates/session/src/exit.rs` (`close_shown`, `decide_close`, `close_on_signals`)                                                                                           |
| Settings window             | `settings_window.cc` → `SettingsWindow`; `fonts_page.cc`, `engine_card.cc`, `update_card.cc`                                                                                                                     |
| Project Inspector           | `project_panel.cc` → `ProjectPanel`                                                                                                                                                                              |
| View state                  | `ui.rs` → `load_layout`, `save_layout`, `project_window`, `save_window`; `main_window.cc` → `SaveLayout`, `SaveWindow`; `crates/session/src/windows.rs` → `visible_bounds`                                       |

## Rules

### Windows and docks

- **The main window holds three screens in one stack**: the start screen, setup (shown while the engine is missing and setup wasn't skipped) and the editor. Settings, Replay, the streamed views, the Live window and the terminal window are windows of their own.
- **The panels are KDDockWidgets docks**: the node list on the left, the Inspector on the right, and Timeline, Mixer and Run as tabs of one bottom dock (`kBottomShare` of the height), Timeline first.
- **Any dock pops out into its own window and docks back**; the arrangement is saved per project (`SaveLayout`) when the project closes (`CloseProject`), when the window goes back to the start screen and when the window asks Rust whether it may close (`closeEvent`).
- **The node list, Inspector and bottom panel collapse from the View menu, its shortcuts and the top bar**; a closed node list leaves a narrow category rail (`CategoryRail::kWidth`) whose categories open as popovers.
- **Docked panels show no title, and a lone panel no tab** (`dock_style.cc` → `DockTitleBar::setVisible`; `app.cc` sets no `Flag_AlwaysShowTabs`): the docked title bar is zero pixels tall, never hidden, because KDDockWidgets refuses to float a panel through a hidden title bar (`TitleBar::makeWindow`). A floating panel keeps its window's title bar to drag and close it by.
  - The style sheet styles `QTabWidget::pane`, or Fusion draws a second sunken frame inside every dock.
- **Files play in `FilePlayer`**: videos and sounds in the Run panel and Replay, and a made sound below its node's settings in the Inspector. Images open full size in `ImagePreview`.

### Title bar and menus

- **The window has no system title bar** (`Qt::FramelessWindowHint`, `WindowChrome`): `AppBar` is the drag handle, starting the system move once the pointer has moved `kDragStartPx`, and a double-click toggles maximize. A border in the `window-border` colour shows while the window is neither maximized nor full screen (`Framed`).
- **Menus come from one list, `app_menu` in `opsis-session`**, which `menu.rs` turns into the bar's sections and shortcuts. A command goes to `Dispatch`, which handles the window's own and sends the rest to `menu_command`; standard edits go to `RunRole`.

### Canvas

- **`settings.editor.canvasMode` is `mouse` (the wheel zooms, dragging the background pans) or `touchpad` (two fingers pan, a pinch zooms)**, chosen in the canvas toolbar or Settings → Editor and kept for every project.
- **The grid and minimap toggles sit in their own toolbar cards** and show their state with the selected overlay.
- **The canvas mirrors the session's whole selection**: a selection made elsewhere (the Run panel, Replay, the Local API) selects exactly those cards, so clicking a card always changes what is selected.
- **Tidy up lays the cards out in Rust** (`auto_layout`) from each card's shape, as one undo step.
- **Edges entering one column turn in their own lane where their runs overlap** (`step_positions`).
- **The nodes playing at the playhead are ringed and their edges move as dashes**; picking one node moves the playhead to the start of its span. The dashes stand still with reduced motion.

### Look and motion

- **Every colour is a token in `look.rs`**, resolved into `style.qss`; panels, menus and the top bar sit on the sidebar colour, the canvas on the background colour.
- **Motion respects the system's reduced motion setting** (`ReducedMotion`, from `QAccessibilityHints::motionPreference`): fades and slides become instant.
  - Widget animations take their length through `MotionMs` and their curve from `EaseOut` (`motion.cc`, which also holds `FadeIn`, `FadeOut` and `GrowWidth`); the canvas's moving edge dashes and the sweep on a loading card's ring check `ReducedMotion` themselves.
- **The focus ring shows only after Tab or Backtab** (`FocusRing` and `LookStyle` in `look_style.cc`): a `QFocusFrame` draws the `ring` colour outside the widget, which a style sheet can't, and Fusion's dotted focus rectangle is never drawn.
- **Drop-down lists are `Combo` (`widgets.h`), never a bare `QComboBox`**: the style sheet draws a combo without asking the style for its arrow, so `Combo` paints its own chevron.

### Starting and closing

- **Settings → General picks what launch shows**: the start screen, the last project or New project (`StartAsChosen`), once per launch and only when setup isn't showing.
- **Closing the main window asks first unless the user said not to**: "Are you sure you want to exit?" with "Don't ask again". Unfinished work is always listed, and the button becomes "Close anyway": a run, the labels in `AppState.tasks` (engine setup, a font or update download), and unsaved changes or a value still being typed (`ongoing`, `crates/session/src/exit.rs`).
- **Closing hands held values over before saving**: the session asks for them (`FlushSave`, which the editor receives as `FlushDrafts`), the Inspector ends its typing, and Rust saves.
- **The dialog tells Rust it is up as soon as it shows (`close_shown`); if nothing does within `SHOWN_WITHIN`, Rust saves and closes without asking**, so a broken window can always close.
- **SIGTERM, SIGINT and SIGHUP save the project and close without asking** (`close_on_signals`, Unix only); the dock arrangement and window place aren't saved then.

### Settings window

- **Settings is a window of its own with a page list**, opened from the menu, its shortcut and any Settings button; it is kept between openings and closes with the main window.
- **Settings are written one section at a time (`patch_settings`)**, so windows never overwrite each other, and every view follows `SettingsChanged`.
- **The Fonts page's notices show in the Settings window** (`ShowToast` on that window), not over the canvas; other pages report a failure in a message box (`App::ShowError`).
- **Saved results (clear cache) belong to the project, in the project Inspector** (`ProjectPanel`).

### View state

- **View state is per project, on this computer, in the database's `view_state` table**: the dock arrangement (`nativeDockLayout`) and the main window's place and size (`mainWindow`).
- **The main window's place is saved shortly after it stops moving or resizing** (`kSaveWindowAfterMs`), never while minimized or full screen; a maximized window keeps its normal size for later.
- **A saved place is used only while a screen can still show it** (`visible_bounds`); otherwise only whether the window was maximized is kept.
- **A project with no saved window opens maximized; the start screen leaves the window as it is.**
- **Nothing is saved while no project is open.**
