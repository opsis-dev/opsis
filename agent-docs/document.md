# The document and shared state

The open project lives in Rust: every view edits it through the session and follows its changes. This doc covers edits and gestures, compiles, the state the editor's windows share, live frames, autosave and changes made to the project file outside the app. Rust paths are relative to `crates/session/src/` unless they name another crate. Window layout is in [editor.md](editor.md), the bridge the views use in [ui.md](ui.md), runs in [runs.md](runs.md), the files a film loads in [assets.md](assets.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                        | Where                                                                                                 |
| ------------------------------ | ----------------------------------------------------------------------------------------------------- |
| Session                        | `crates/session` → `Session`, `Part`, `events::SessionEvent`, `platform::Platform`                    |
| The editor host of the session | `crates/ui/src/ui.rs` → `Ui` (`opsis_qt::Host`), `crates/ui/src/platform.rs` → `QtPlatform`           |
| Document                       | `crates/document` → `Document`                                                                        |
| Workflow and timeline edits    | `WorkflowEdit`, `TimelineEdit` (`crates/compile/src/edits/`)                                          |
| Views of the document          | `crates/ui/src/graph.rs`, `inspector.rs`, `timeline.rs`, built from `document_state` when a view asks |
| Autosave and disk changes      | `document.rs` → `schedule_autosave`, `save_due`; `crates/project` → `ProjectFiles::save`              |
| Disk change dialog             | `crates/qt/cpp/main_window.cc` → `AskAboutDisk`; `resolve_disk_change`                                |
| Compile requests               | `compile::request`                                                                                    |
| Shared selection and playhead  | `shared.rs`; `crates/ui/src/ui.rs` → `select`, `share_playhead`                                       |
| Calls and events               | the `Host` trait (`crates/qt/src/host.rs`) and its bridge (`crates/qt/src/bridge.rs`)                 |
| Frames to the views            | `crates/ui/src/streams.rs` → `FrameSlot` (the `FrameSender`), `FrameRef`                              |
| Live PiP and window            | `crates/qt/cpp/live_view.cc` → `LivePip`, `LiveWindow`; `crates/ui/src/live.rs` → `Live`              |

## Rules

### The session

- **`opsis-session` owns the open project, the document, compiles, runs, shared state, settings, fonts, the engine, streamed views and the exit decision, with no window code.** Its host, `opsis-ui`, starts it with `Session::start`, subscribes to its events, then calls `Session::begin`, and calls `Session::end` when the app exits.
- **The session never opens a window or a dialog itself: it asks the host through `Platform`** (folder and file pickers, opening paths, notifications, raising the editor, exiting, the streamed view and terminal windows), which `QtPlatform` turns into calls to the C++ side.
- **Events leave the session on one `tokio` broadcast channel of `Addressed` events**, each for every window or for one window by label (`MAIN_WINDOW`, `LIVE_WINDOW`). The host's pump turns them into the editor's event kinds ([ui.md](ui.md)); a host that lags re-reads current state.

- **Frames reach the views through a `FrameSender`**: the host's `FrameSlot` keeps the newest frame and wakes the view that shows it, which takes it as a `FrameRef` without a copy.

### Edits

- **Every view edits the one document through the session**: `edit_workflow` (`WorkflowEdit`: nodes, connections, settings, fonts, variables, paste, groups, mix, markers, replace), `edit_timeline` (`TimelineEdit`), `undo`, `redo`, `begin_gesture`, `end_gesture`.
- **Views follow `DocumentChanged`** (the document and the nodes the change edited) and build their view from `document_state` when asked; the editor keeps no copy of the document.
- **A gesture (a drag, a text field being typed in, an open colour picker) is one undo step**; nested gestures count as one, and undo or redo closes an open gesture.
- **Opening another project clears the shared selection.**

### Compiles

- **After each change Rust compiles again** (`compile::request`, at most one compile running and one waiting) and sends `Compiled` to every window.
- **Selecting says which node is inspected (`set_compile_focus`)**, and the views read the latest compile from the host, which keeps only the newest (`seq`).

### Shared state

- **The selection and the Timeline's playhead live in Rust (`shared.rs`)**, written with `share_selection` and `share_playhead` and followed through `SelectionShared` and `PlayheadShared`.
- **The playhead runs in the main window, which shares every frame.**
- **Replay is asked for through `show_replay` (`ReplayRequested`)**, so a popped-out Run panel opens it in the main window.

### Live frames

- **Live recording frames and live events go only to the windows that watch them** (`watch_live`, each with its own `FrameSender`), stopping with `unwatch_live`, which removes only its own.
- **Rust sends `LiveNoted` only to watching windows, and `watch_live` returns the current run's start, step and end notes**, so a view that starts watching mid-run still knows each capture's title and steps.
- **The editor watches once, for the main window, and both places that show the stream draw from that one feed** (`Live`): the picture-in-picture (`LivePip`) and the Live window, which is only the live picture with a LIVE badge and keeps no history.
  - The PiP's enlarge button and its picture open the Live window, and the PiP hides while that window is open.
- **Only the recording browser's frames (Record take, Screenshot) and terminal live pictures are live** ([terminal.md](terminal.md)).

### Saving

- **Rust saves the open project itself**: 800 ms after the last change (`schedule_autosave`), or at once on `save_project` and before a run, sending the file's `version` (a hash of its text) as the base.
- **`ProjectFiles::save` refuses to write when the file on disk has another version.**
- **The document state says whether a save is coming** (`saving`: set when an autosave is scheduled, cleared when a save ends and no newer one is due, `save_due`). The top bar reads Saving while changes wait, Unsaved only when changes have no save coming (a failed or refused save), and Saved otherwise.

### Changes on disk

- **Rust watches the open project's file from the open**, watching the folder because saves replace the file, and ignores its own writes.
  - Its own writes are recognised by the version each save records (`ProjectFiles::remember`). Recording a version and writing the file take one turn, and so does the watcher's read and compare (`ProjectFiles::outside_version`); otherwise a debounced read landing in a save's write could see the previous save's file under the new record, take it for an outside change and load the older project over the newer one.
- **With nothing unsaved it loads an outside change as one undo step and tells the editor (`ProjectReloaded`).**
- **With unsaved edits, an unreadable file or a refused save it asks (`DiskChangeAsked`)**; `DiskChangeDialog` answers with `resolve_disk_change`: Load the file (undoable), Keep mine (saves over it) or, for an unreadable file, Wait. Nothing saves while that question is open.
