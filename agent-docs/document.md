# The document and shared state

The open project lives in Rust: every view edits it through the session and follows its changes. This doc covers edits and gestures, compiles, the state the editor's windows share, live frames, autosave and changes made to the project file outside the app. Rust paths are relative to `crates/session/src/` unless they name another crate. Window layout is in [editor.md](editor.md), the bridge the views use in [ui.md](ui.md), runs in [runs.md](runs.md), the files a film loads in [assets.md](assets.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                        | Where                                                                                                                     |
| ------------------------------ | ------------------------------------------------------------------------------------------------------------------------- |
| Session                        | `crates/session` → `Session`, `Part`, `events::SessionEvent`, `platform::Platform`                                        |
| The editor host of the session | `crates/ui/src/ui.rs` → `Ui` (`opsis_qt::Host`), `crates/ui/src/platform.rs` → `QtPlatform`                               |
| Document                       | `crates/document/src/lib.rs` → `Document`, `MAX_UNDO_STEPS`                                                               |
| Workflow and timeline edits    | `crates/types/src/edits.rs` → `WorkflowEdit`, `TimelineEdit`; applied in `crates/compile/src/edits/`                      |
| Views of the document          | `crates/ui/src/ui.rs` reads `document_state`; `crates/ui/src/graph.rs`, `inspector.rs`, `timeline.rs` build the views     |
| Autosave and disk changes      | `document.rs` → `schedule_autosave`, `AUTOSAVE_DELAY`, `save`, `outside_change`; `crates/project` → `ProjectFiles::save`  |
| Disk change dialog             | `crates/qt/cpp/main_window.cc` → `AskAboutDisk`; `resolve_disk_change`                                                    |
| Compile requests               | `compile::request`                                                                                                        |
| A script's batch               | `crates/document/src/lib.rs` → `begin_batch`, `end_batch`; `document.rs` → `begin_batch`, `end_batch`, `edit_timeline_on` |
| Shared selection and playhead  | `shared.rs`; `crates/ui/src/ui.rs` → `select`, `share_playhead`                                                           |
| Calls and events               | the `Host` trait (`crates/qt/src/host.rs`) and its bridge (`crates/qt/src/bridge.rs`)                                     |
| Frames to the views            | `crates/ui/src/streams.rs` → `FrameSlot` (the `FrameSender`), `Shown`; `crates/qt/src/frame.rs` → `FrameRef`              |
| Live PiP and window            | `crates/qt/cpp/live_view.cc` → `LivePip`, `LiveWindow`; `crates/ui/src/live.rs` → `Live`                                  |

## Rules

### The session

- **The host starts the session, subscribes, and only then begins it**: `opsis-ui` starts it with `Session::start`, subscribes to its events, then calls `Session::begin`, and calls `Session::end` when the app exits. What the session owns is in [AGENTS.md](../AGENTS.md) §6.
- **The session never opens a window or a dialog itself: it asks the host through `Platform`** (folder and file pickers, opening paths, notifications, raising the editor, exiting, the streamed view and terminal windows), which `QtPlatform` turns into calls to the C++ side.
- **Events leave the session on one `tokio` broadcast channel of `Addressed` events**, each for every window or for one window by label (`MAIN_WINDOW`).
  - The host's pump turns them into the editor's event kinds ([ui.md](ui.md)).
  - A host that lags behind the channel only tells the editor the document changed (`EventKind::Document`), so other missed events wait for the next one of their kind.
- **Frames reach the views through a `FrameSender`**: the host's `FrameSlot` unpacks each frame and hands its header and JPEG bytes to its view's callback, which copies the JPEG into the newest frame it keeps (`Shown::next`) and wakes the view that shows it.
  - The view takes that frame as a `FrameRef` sharing those bytes (`Shown::frame`), with no further copy.

### Edits

- **Every view edits the one document through the session**: `edit_workflow` (`WorkflowEdit`), `edit_timeline` (`TimelineEdit`), `undo`, `redo`, `begin_gesture`, `end_gesture`.
- **A new connection is refused before it is applied** when its value can't reach the input (`document::edit` → `refused_edge`, through `connection_problem_now`), with the problem as the error (`DocumentError::Refused`).
- **Timeline edits run against the film of the last compile (`editor.film`)**, or the gesture's film during a gesture; with no film yet they fail with "There's no film on the timeline yet."
- **An edit that leaves the workflow as it was records nothing**: no undo step and no new revision.
- **Undo keeps the last `MAX_UNDO_STEPS` steps, dropping the oldest, and a new step clears redo.**
- **Views follow `DocumentChanged`** (the document and the nodes the change edited), sent after every change, every save and the end of every gesture, and build their view from `document_state` when asked; the editor keeps no copy of the document.
- **A gesture (a drag, a text field being typed in, an open colour picker) is one undo step**; nested gestures count as one, and undo or redo closes an open gesture.
  - Timeline edits inside a gesture start from the workflow the gesture began with (`gesture_start`) against the film it began with (`gesture_film`), and new ids are salted by the gesture, so each drag move replaces the last and a node it adds keeps its id.
  - How compiles treat an open gesture is in [runs.md](runs.md), Results follow the keys (`held`).
- **A script's run is one batch** (`begin_batch`, `end_batch`; [scripting.md](scripting.md)): while it is open no change records an undo step, ended gestures included; kept, the batch records its start as one undo step when the workflow changed; dropped, it puts its start back and records nothing, and the redo list is as it was because undo and redo are refused meanwhile (`DocumentError::ScriptRunning`, 409 on the Local API).
  - A batch is not a gesture: a gesture makes every timeline edit start from the gesture's start and holds compiles, so a script's second timeline edit would replace its first; inside a batch timeline edits build on each other and compiles run as usual.
- **The nodes each change edited collect until the next `DocumentChanged` takes them** (`take_edited`), and a run forgets the errors of those nodes.
- **Opening or adopting another project stops a running script and waits for its rollback, saves the open one, drops a pending disk question and clears the shared selection.**
- **A project is known by its real folder** (`opsis_folders::real_folder`, in `document::open`, `document::adopt` and `projects::create_project`): the path `std::fs::canonicalize` gives, without Windows' `\\?\` prefix, so the recent list, the view state and the saved panel layout find one key however the folder was spelled, since on Windows a Qt path (`C:/x/projects`) joined in Rust (`\Reopen`) differs from the same folder opened again (`C:/x/projects/Reopen`), as do its case, its 8.3 short names and `.` or `..`.

### Compiles

- **After each change Rust compiles again** (`compile::request`, at most one compile running and one waiting) and sends `Compiled` to every window.
  - Each compile then follows the project files the film loads, and a change to one of them compiles again ([assets.md](assets.md), Followed project files).
- **Selecting says which film the compile picks and which node is inspected (`set_compile_focus(selected, inspected)`)**.
  - A plain selection (`select`, `crates/ui/src/ui.rs`) makes its first node both.
  - A selection with a pinned source (`select_pinned`) picks the film by the pinned source, or by the one selected node when none is pinned, and inspects only a single selected node.
- **The views read the latest compile from the host, which replaces its copy with each `Compiled` it receives**; compiles run one at a time, so the last one received is the newest.

### Shared state

- **The selection and the Timeline's playhead live in Rust (`shared.rs`)**, written with `share_selection` and `share_playhead` and followed through `SelectionShared` and `PlayheadShared`.
- **The playhead runs in the main window, which shares every frame.**
- **Replay is asked for through `show_replay` (`ReplayRequested`)**, so a popped-out Run panel opens it in the main window.

### Live frames

- **Live recording frames and live events go only to the windows that watch them** (`watch_live`, each with its own `FrameSender`).
- **Rust sends `LiveNoted` only to watching windows, and `watch_live` returns the current run's start, step and end notes**, so a view that starts watching mid-run still knows each capture's title and steps.
- **The editor watches once, for the main window, and both places that show the stream draw from that one feed** (`Live`): the picture-in-picture (`LivePip`) and the Live window, which is only the live picture with a LIVE badge and keeps no history.
  - The PiP's enlarge button and its picture open the Live window, and the PiP hides while that window is open.
- **Only the recording browser's frames (Record take, Screenshot) and terminal live pictures are live** ([terminal.md](terminal.md)).

### Saving

- **Rust saves the open project itself, sending the file's `version` (a hash of its text) as the base**: `AUTOSAVE_DELAY` after the last change (`schedule_autosave`), or at once on `save_project`, before a run, on the Local API's save, before opening or adopting another project, before closing it, and at exit (`flush_save`, `crates/session/src/exit.rs`; [editor.md](editor.md) has how the editor's drafts are saved first).
  - Saves take turns, and a timer that a newer change replaced returns without saving.
  - A save records the revision it wrote (`mark_saved`), so changes made while it was writing stay unsaved.
- **`ProjectFiles::save` refuses to write when the file on disk has another version.**
- **The top bar reads Unsaved while the document's revision differs from the saved one, and Saved otherwise** (`dirty` in `crates/ui/src/ui.rs`, shown by `crates/qt/cpp/top_bar.cc`).
  - The document state also says whether a save is coming (`saving`, from `save_due`); no view reads it.

### Changes on disk

- **Rust watches the open project's file from the open**, watching the folder because saves replace the file, and ignores its own writes.
  - Its own writes are recognised by the version each save records (`ProjectFiles::remember`). Recording a version and writing the file take one turn, and so does the watcher's read and compare (`ProjectFiles::outside_version`); otherwise a debounced read landing in a save's write could see the previous save's file under the new record, take it for an outside change and load the older project over the newer one.
- **With nothing unsaved it loads an outside change as one undo step and tells the editor (`ProjectReloaded`).**
- **With unsaved edits, an unreadable file or a refused save it asks (`DiskChangeAsked`)**; `DiskChangeDialog` answers with `resolve_disk_change`: Load the file (undoable), Keep mine (saves over it) or, for an unreadable file, Wait.
  - Nothing saves while that question is open, so a run refuses to start then (`StartError::VersionUnchosen`, `run.rs`).
  - Wait closes the question without choosing: the next save that has changes to write finds the file still changed and asks again.
