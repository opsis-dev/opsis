# The editor

The native editor's windows, docks and panels, the title bar and menus, the canvas, its look and motion, starting and closing, the Settings window and per-project view state. C++ paths are relative to `crates/qt/cpp/`, Rust paths to `crates/ui/src/` unless they name another crate. How C++ and Rust talk is in [ui.md](ui.md), how windows share the document in [document.md](document.md), the Timeline in [timeline.md](timeline.md), the Inspector in [inspector.md](inspector.md), the About page's Updates card in [updates.md](updates.md), the database in [app-state.md](app-state.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                     | Where                                                                                                                                                                                                                         |
| --------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Main window and its screens | `main_window.cc` → `MainWindow` (start screen, setup, editor), `StartAsChosen`; `start_screen.cc`, `engine_card.cc` → `SetupScreen`                                                                                           |
| Docks and panels            | `main_window.cc` → `BuildDocks`, `MakeDock`; `node_list.cc` (`NodeList`, `CategoryRail`); `dock_style.cc` → `DockTitleBar`, `DockStack` (the pop-out and collapse corner), `DockViewFactory`; KDDockWidgets flags in `app.cc` |
| Quick add                   | `main_window.cc` → `QuickAdd`                                                                                                                                                                                                 |
| Title bar and window frame  | `window_chrome.cc` → `AppBar`, `WindowChrome`; `top_bar.cc` → `TopBar`                                                                                                                                                        |
| Menus                       | `crates/session/src/menu.rs` → `app_menu`; `crates/ui/src/menu.rs` → `menu`; `main_window.cc` → `BuildMenus`, `Dispatch`, `RunRole`                                                                                           |
| Canvas                      | `graph_canvas.cc` → `GraphCanvas`, `CardItem`, `EdgeItem`, `Minimap`; `graph.rs` (cards, edges, lanes, active nodes)                                                                                                          |
| Tidy up                     | `crates/compile/src/layout.rs` → `auto_layout`; `ui.rs` → `tidy_layout`                                                                                                                                                       |
| Edge lanes                  | `crates/compile/src/lanes.rs` → `step_positions`                                                                                                                                                                              |
| Look and motion             | `crates/ui/resources/style.qss`, `look.rs` → `TOKENS`; `look_style.cc` → `LookStyle`, `FocusRing`; `motion.cc` → `ReducedMotion`, `MotionMs`, `EaseOut`, `FadeIn`, `FadeOut`, `GrowWidth`; `widgets.h` → `Combo`              |
| Players and pictures        | `file_player.cc` → `FilePlayer`, `ImagePreview`; `player.rs`                                                                                                                                                                  |
| Closing                     | `main_window.cc` → `ShowCloseDialog`; `crates/session/src/exit.rs` (`close_shown`, `decide_close`, `close_on_signals`)                                                                                                        |
| Settings window             | `settings_window.cc` → `SettingsWindow`; `fonts_page.cc`, `engine_card.cc`, `update_card.cc`                                                                                                                                  |
| Project Inspector           | `project_panel.cc` → `ProjectPanel`                                                                                                                                                                                           |
| Script panel                | `script_panel.cc` → `ScriptPanel`; `script_panel.rs`; [scripting.md](scripting.md)                                                                                                                                            |
| View state                  | `ui.rs` → `load_layout`, `save_layout`, `project_window`, `save_window`; `main_window.cc` → `SaveLayout`, `SaveWindow`; `crates/session/src/windows.rs` → `visible_bounds`                                                    |

## Rules

### Windows and docks

- **The main window holds three screens in one stack**: the start screen, setup (shown while the engine is missing and setup wasn't skipped) and the editor. Settings, Replay, the streamed views, the Live window and the terminal window are windows of their own.
- **The panels are KDDockWidgets docks**: the node list on the left, the Inspector on the right, and Timeline, Mixer, Run and Script as tabs of one bottom dock (`kBottomShare` of the height), Timeline first. The Script panel is in [scripting.md](scripting.md).
- **Every dock group with more than one tab shows two corner buttons (`DockStack`)**: one pops the current panel out into its own window ("Open … in a new window", `setFloating`), where it can dock back, and one hides the bottom panel, whichever group it sits on.
  - As the editor builds the docks only the bottom one has tabs; the node list and the Inspector, lone docks with a zero-height title bar, have no control that pops them out.
- **The arrangement, popped-out panels included, is saved per project (`SaveLayout`, `kdw::LayoutSaver`)** when the project closes (`CloseProject`), when the window goes back to the start screen, and when the window asks Rust whether it may close and Rust asks first (`closeEvent`).
  - On the next start it comes back with popped-out panels still in their own windows (the two-launch `reopen` test, [e2e.md](e2e.md)).
  - It is restored only when the docks are first built in a launch (`BuildDocks`); opening another project later in the same launch keeps the docks as they are and reopens the node list and Inspector, and every bottom panel, if they were closed (`ShowEditor` → `ReopenBottom`).
  - KDDockWidgets floats a dock that a saved arrangement doesn't name (`floatUnknownWidgets`), so `BuildDocks` puts such a bottom panel, and one the arrangement has closed, back as a tab of the bottom group (`LayoutKnows`, `DockBottom`); the `reopen` test saves an arrangement without the Script tab to check it.
- **A bottom panel is never lost: closing one docks it back as a bottom tab** (`isOpenChanged` → `DockBottom` in `BuildDocks`). The only closed panel KDDockWidgets offers is a popped-out one, whose close button would otherwise close it for good; the closes the editor makes itself (`ShowStart`, `ToggleBottom`) set `parking_` so they stay closed. View → Timeline, Mixer, Run and Script dock a closed panel too (`FocusDock`), since a bare `open` puts it back where it last was, which may be a popped-out window.
  - `DockBottom` tabs the panel onto another docked bottom panel; with none open it returns the panel to the place KDDockWidgets remembers for it (`open`, then `setFloating(false)` if that reopened it popped out), which sits between the sidebars. Adding it at the window's bottom (`Location_OnBottom` with no dock to sit beside) is only for a panel never docked, because that row spans the whole window, under the node list and the Inspector.
- **The splitters between panels are 6 px wide, drawing a 2 px line with a 12 by 16 handle holding Lucide's grip at 10 px, turned for a splitter that runs across** (`kSeparatorThickness` in `app.cc`, `DockSeparator` and `SplitterHandle` in `dock_style.cc`): KDDockWidgets only resizes from a press on the splitter itself, so a thinner one can't be grabbed.
  - The handle is wider than the splitter, so it is a widget of its own beside the splitter in the same parent, kept centred on it (`Follow`) and raised above the panels on each new child; it passes its mouse events to the splitter, so a drag can start on any part of it.
  - The hover colour follows the splitter's and the handle's own enter and leave events (`Hot`), and both repaint on each, so a splitter never keeps its hover colour once the mouse has gone.
- **Hovering a node in the list shows a card with its icon beside its title and description, then its ports as they sit on a canvas card**: inputs on the left and outputs on the right, each a dot in its type's colour beside its name, optional ones dimmed (`NodeList::ShowCard`, `PortRows`). The list's own heading already names the group, so the card doesn't.
  - The card's icon plays the node's look again when the card opens, under its own id in the list's `LookPlayer` (`NodeList::CardLook`): the card opens after the row's motion has mostly run, so sharing the row's would show it nearly still. Closing the card lets a morph go back (`CloseCard`).
  - Numbered inputs show as one row named for them together (`PortFamily::name`, "Clips") with stacked dots and how many are needed, since the catalogue lists only the first ones shown; an input taking any number of connections gets a ringed dot (`PortShape`, `crates/ui/src/palette.rs` → `palette_ports`).
- **The node list, Inspector and bottom panel collapse from the View menu, its shortcuts and the top bar**; a closed node list leaves a narrow category rail (`CategoryRail::kWidth`) whose categories open as popovers.
- **Docked panels show no title, and a lone panel no tab** (`dock_style.cc` → `DockTitleBar::setVisible`; `app.cc` sets no `Flag_AlwaysShowTabs`).
  - A lone panel's docked title bar is zero pixels tall rather than hidden, because KDDockWidgets refuses to float a panel through a hidden title bar (`TitleBar::makeWindow`); in the tabbed bottom dock KDDockWidgets hides it itself (`Flag_HideTitleBarWhenTabsVisible`).
  - A floating panel keeps its window's title bar to drag and close it by.
- **A popped-out panel is a normal window of its own** (`Flag_TitleBarHasMinimizeButton`, which brings `Flag_DontUseUtilityFloatingWindows`, and `InternalFlag_DontUseParentForFloatingWindows` in `App::ConfigureDocks`): it can go behind the editor, minimizes on its own and has its own task bar entry. KDDockWidgets' default, a tool window owned by the main window, is always stacked above the editor.
  - Without an owner it outlives the main window, so `App::Run` deletes the floating windows before the main window at exit; the other order crashes on a restored floating window.
- **On Wayland a popped-out panel has one title bar, ours** (`DockViewFactory::createFloatingWindow` gives it `WindowChrome`): KDDockWidgets forces the compositor's title bar there and keeps its own for drag and drop (`Config.cpp`), which shows two bars. `DockTitleBar` moves the window with `startSystemMove`, so dragging a popped-out panel onto the editor to dock it works only on X11; its dock button works everywhere.
- **The title bar's buttons are 24 px with 14 px icons** (`DockTitleButton`): KDDockWidgets makes them 16 px, which the style sheet's tool button padding leaves 6 px of.
  - The style sheet styles `QTabWidget::pane`, or Fusion draws a second sunken frame inside every dock.
- **Files play in `FilePlayer`**: videos and sounds in the Run panel and Replay, and a made sound below its node's settings in the Inspector. Images open full size in `ImagePreview`.

### Title bar and menus

- **The window has no system title bar** (`Qt::FramelessWindowHint`, `WindowChrome`): `AppBar` is the drag handle, starting the system move once the pointer has moved `kDragStartPx`, and a double-click toggles maximize. A border in the `window-border` colour shows while the window is neither maximized nor full screen (`Framed`).
- **Menus come from one list, `app_menu` in `opsis-session`**, which `crates/ui/src/menu.rs` turns into the bar's sections and shortcuts. A command goes to `Dispatch`, which handles the window's own and sends the rest to `menu_command`; standard edits go to `RunRole`.

### Canvas

- **The canvas mode (`settings.editor.canvasMode`, `CanvasMode`) is chosen in the canvas toolbar or Settings → Editor and kept for every project.**
- **The grid and minimap toggles sit in their own toolbar cards** and show their state with the selected overlay; neither is saved, so both start as the canvas builds them each launch.
- **Quick add (`QuickAdd`, Cmd/Ctrl+K) adds a node by name.**
- **The canvas mirrors the session's whole selection**: a selection made elsewhere (the Run panel, Replay, the Local API) selects exactly those cards, so clicking a card always changes what is selected.
- **Tidy up lays the cards out in Rust** (`tidy_layout` → `auto_layout`) from each card's shape, as one undo step.
- **Edges entering one column turn in their own lane where their runs overlap** (`step_positions`).
- **The nodes playing at the playhead are ringed and their edges move as dashes**; selecting exactly one node, with no pinned source, moves the playhead to the start of its span when the last film has one (`selection_start`). The dashes stand still with reduced motion.

### Look and motion

- **A font set in code is set with `UseFont` (`widgets.h`), never `setFont` alone**: the style sheet's `*` rule sets every widget's family and size, which beats `setFont`, so `UseFont` repeats the family, weight and size in the widget's own style sheet. A `setFont` that only changes the weight is kept, since the rule sets no weight.
  - The `*` rule stays: without it the desktop theme's fonts, which the portal sends after start, replace the app's.
- **A window or popover whose text wraps takes its height from its width (`FitHeightToWidth`, `widgets.h`)**, after building and after showing more text, as a fixed-width window's `adjustSize` measures the text at its preferred width and clips it.
- **Lines appended as HTML go through `KeepSpaces` (`widgets.h`)**: HTML drops leading spaces and tabs and merges runs of them, which a log or a script's output must keep.
- **Qt's and KDDockWidgets' own tool buttons have no padding** (`QLineEdit QToolButton`, `QTabBar QToolButton`, `DockTitleButton` in `style.qss`): the style sheet's tool button padding is for the app's own buttons and shrinks the fixed-size ones inside line edits, tab bars and title bars.
- **Colours are tokens in `look.rs` (`TOKENS`)**, resolved into `style.qss` and read in C++ with `App::Token`; panels, the menu bar and the top bar sit on the sidebar colour, popup menus on the popover colour, the canvas on the background colour.
  - C++ writes a colour as a literal only where the theme doesn't change it: black glyphs and text on a category's own colour, black behind pictures (players, Replay, the live view), the colour and gradient pickers' white and black, the Marks field's default mark colours and the glyph effects in `look_glyph.cc`.
- **Motion respects the system's reduced motion setting** (`ReducedMotion`, from `QAccessibilityHints::motionPreference`): fades and slides become instant.
  - Widget animations take their length through `MotionMs` and their curve from `EaseOut` (`motion.cc`, which also holds `FadeIn`, `FadeOut` and `GrowWidth`); the canvas's moving edge dashes and the sweep on a loading card's ring check `ReducedMotion` themselves.
- **The focus ring shows only after Tab or Backtab** (`FocusRing` and `LookStyle` in `look_style.cc`): a `QFocusFrame` draws the `ring` colour outside the widget, which a style sheet can't, and Fusion's dotted focus rectangle is never drawn.
- **Drop-down lists are `Combo` (`widgets.h`), never a bare `QComboBox`**: the style sheet draws a combo without asking the style for its arrow, so `Combo` paints its own chevron.

### Starting and closing

- **Settings → General picks what launch shows**: the start screen, the last project or New project (`StartAsChosen`), once per launch and only when setup isn't showing, and never headless ([assistant.md](assistant.md)).
- **Closing the main window asks first unless the user said not to, or the app is headless**: "Are you sure you want to exit?" with "Don't ask again", and the button "Exit".
  - Unfinished work is always listed, and the button becomes "Close anyway": a run, the labels in `AppState.tasks` (engine setup, a font or update download), and unsaved changes or a value still being typed (`ongoing`, `crates/session/src/exit.rs`).
  - With asking turned off, unfinished work still shows the dialog, titled "Some work hasn't finished yet" and without "Don't ask again".
- **Closing saves the values still being typed first** (`flush_save`, `crates/session/src/exit.rs`): when the editor has reported drafts (`report_drafts`), the session asks for them (`FlushSave`, which the editor receives as `FlushDrafts`), the Inspector ends its typing and answers `save_flushed`, and Rust waits for that up to `FLUSH_WITHIN` before it saves.
- **The dialog tells Rust it is up as soon as it shows (`close_shown`); if nothing does within `SHOWN_WITHIN`, Rust saves and closes without asking**, so a broken window can always close.
- **SIGTERM, SIGINT and SIGHUP save the project and close without asking** (`close_on_signals`, Unix only).

### Settings window

- **Settings is a window of its own with a page list**, opened from the menu, its shortcut, any Settings button, the Assistant badge in the app bar and the session (`SettingsAsked`); it is kept between openings and closes with the main window.
- **Settings are written one section at a time (`patch_settings`)**, so windows never overwrite each other, and every view follows `SettingsChanged`.
- **Settings → General → Advanced holds Threads used for exports**, which also sets the threads of take previews: a number field whose empty value means half of this computer's threads (shown as its placeholder) and which holds a typed number between one and the computer's threads; the rule is `export_threads` ([film.md](film.md), Output).
- **The Fonts page's notices show in the Settings window** (`ShowToast` on that window), not over the canvas; other pages report a failure in a message box (`App::ShowError`).
- **Saved results (clear cache) belong to the project, in the project Inspector** (`ProjectPanel`).

### View state

- **View state is per project, on this computer, in the database's `view_state` table**: the dock arrangement (`nativeDockLayout`) and the main window's place and size (`mainWindow`).
- **The main window's place is saved shortly after it stops moving or resizing** (`kSaveWindowAfterMs`), never while minimized or full screen; a maximized window keeps its normal size for later.
- **A saved place is used only while a screen can still show it** (`visible_bounds`); otherwise only whether the window was maximized is kept.
- **A project with no saved window opens maximized; the start screen leaves the window as it is.**
- **Nothing is saved while no project is open.**

## Deliberate limits

- **A close by signal doesn't save the dock arrangement or the window's place**: only the project is saved (`close_on_signals`), since the window never runs its own close.
