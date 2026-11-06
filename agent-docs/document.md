# The document and shared state

The open project lives in Rust: every window edits it through typed commands and follows its changes. This doc covers edits and gestures, compiles, the state windows share, live frames, autosave and changes made to the project file outside the app. Rust paths are relative to `crates/desktop/src/`, editor paths to `apps/frontend/src/`. Window layout is in [editor.md](editor.md), runs in [runs.md](runs.md), the files a film loads in [assets.md](assets.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                       | Where                                                                                    |
| ----------------------------- | ---------------------------------------------------------------------------------------- |
| Session                       | `crates/session` → `Session`, `Part`, `events::SessionEvent`, `platform::Platform`       |
| Tauri host of the session     | `crates/desktop/src/commands/`, `forward.rs`, `platform.rs` → `TauriPlatform`            |
| Document                      | `crates/document` → `Document`                                                           |
| Workflow and timeline edits   | `WorkflowEdit`, `TimelineEdit` (`crates/compile/src/edits/`)                             |
| Editor mirror                 | `store/workflow.ts` → `useWorkflowStore`, `workflowEdits`; `useDocumentMirror`           |
| Autosave and disk changes     | `document.rs` → `schedule_autosave`, `save_due`; `crates/project` → `ProjectFiles::save` |
| Disk change dialog            | `DiskChangeDialog`, `resolve_disk_change`                                                |
| Compile requests              | `compile::request`                                                                       |
| Shared selection and playhead | `shared.rs`; editor `sync.ts`                                                            |
| Commands and events           | `bindings.rs` (tauri-specta); `@/generated/bindings`                                     |
| Event and channel helpers     | `app/tauri.ts` → `on`, `frameChannel`, `readFrame`, `watchLive`, `pathForFile`           |
| Capabilities                  | `crates/desktop/capabilities`                                                            |
| Live PiP and window           | `LivePip`; `open_live_window` (route `#/live`, label `live`)                             |

## Rules

### The session

- **`opsis-session` owns the open project, the document, compiles, runs, shared state, settings, fonts, the engine, streamed views and the exit decision, with no window code.** A host (the Tauri app today, `opsis-ui` next) starts it with `Session::start`, subscribes to its events, then calls `Session::begin`, and calls `Session::end` when the app exits.
- **The session never opens a window or a dialog itself: it asks the host through `Platform`** (folder and file pickers, opening paths, notifications, raising the editor, exiting, the streamed view and terminal windows). Window-specific work in a command (a project's window bounds) stays in the host's wrapper.
- **Events leave the session on one `tokio` broadcast channel of `Addressed` events**, each for every window or for one window by label (`MAIN_WINDOW`, `LIVE_WINDOW`). A host that lags re-reads current state, as a window does on mount.
- **The Tauri app forwards each session event as the `tauri-specta` event of the same name** (`forward.rs`): its structs in `crates/desktop/src/events.rs` mirror the session's, so `bindings.ts` stays as it was. A command that returns a session event type returns the desktop mirror, since two exported types can't share a name.
- **Frames reach a window through a `FrameSender`** (`platform.rs`), Tauri's IPC channel in the Tauri app, packed by `frames::packed`.

### Edits

- **Every window edits the one document with typed commands**: `edit_workflow` (`WorkflowEdit`: nodes, connections, settings, fonts, variables, paste, groups, mix, markers, replace), `edit_timeline` (`TimelineEdit`), `undo`, `redo`, `begin_gesture`, `end_gesture`.
- **Windows follow `DocumentChanged`** (the document and the nodes the change edited, whose old run errors the editor forgets) and take `document_state` when they mount (`useDocumentMirror`). `useWorkflowStore` is only that mirror.
- **A gesture (a drag, a text field being typed in, an open colour picker) is one undo step**; nested gestures count as one, and undo or redo closes an open gesture.
- **Opening another project clears the shared selection.**

### Compiles

- **After each change Rust compiles again** (`compile::request`, at most one compile running and one waiting) and sends `Compiled` to every window.
- **Windows say which node is selected and inspected with `set_compile_focus`**, take `latest_compile` when they mount (an event sent before their listener is registered is lost) and drop a result older than the newest they have seen (`seq`).

### Shared state

- **The selection and the Timeline's playhead live in Rust (`shared.rs`)**, written with `share_selection` and `share_playhead`, followed through `SelectionShared` and `PlayheadShared` (each names the window it came from, which ignores its own) and read with `shared_state` when a window mounts.
- **The playhead runs in the editor window, which shares every frame**; other windows share only their in and out points and send play, pause, seek and rate to it with `control_playback` (`PlaybackCalled`).
- **Menu commands go to the editor with `send_menu_to_editor` and Replay with `show_replay` (`ReplayRequested`)**, so a popped-out Run panel opens it there.
- **Commands and events are declared once in Rust and the editor calls the generated `commands` and `events` directly**; commands return their value and throw their error (`ErrorHandlingMode::Throw`).
- **Capabilities let only the editor, its panel windows, Settings and the streamed views call commands.** Engine set-up progress, settings changes and API status go to every app window.

### Live frames

- **Live recording frames and live events go only to the windows that watch them**, each with `watchLive` (`watch_live`, its own `Channel`), stopping with `unwatch_live`, which removes only its own channel (a destroyed window's goes too).
- **Rust sends `LiveNoted` only to watching windows, and `watch_live` returns the current run's start, step and end notes**, so a window that starts watching mid-run still knows each capture's title and steps.
- **Two places show the stream and only one watches at a time**: the editor's picture-in-picture (`LivePip`) and the Live window, which is only the live picture with a LIVE badge and keeps no history.
  - The PiP's enlarge button and its picture open the Live window; once it watches, Rust tells the editor (`LiveWindowWatching`) and the PiP stops watching and hides. When the window closes, the PiP watches again. Two streams exist only while switching.
- **Only the recording browser's frames (Record take, Screenshot) and terminal live pictures are live** ([terminal.md](terminal.md)).

### Saving

- **Rust saves the open project itself**: 800 ms after the last change (`schedule_autosave`), or at once on `save_project` and before a run, sending the file's `version` (a hash of its text) as the base.
- **`ProjectFiles::save` refuses to write when the file on disk has another version.**
- **The document state says whether a save is coming** (`saving`: set when an autosave is scheduled, cleared when a save ends and no newer one is due, `save_due`). The top bar reads Saving while changes wait, Unsaved only when changes have no save coming (a failed or refused save), and Saved otherwise.

### Changes on disk

- **Rust watches the open project's file from the open**, watching the folder because saves replace the file, and ignores its own writes.
- **With nothing unsaved it loads an outside change as one undo step and tells the editor (`ProjectReloaded`).**
- **With unsaved edits, an unreadable file or a refused save it asks (`DiskChangeAsked`)**; `DiskChangeDialog` answers with `resolve_disk_change`: Load the file (undoable), Keep mine (saves over it) or, for an unreadable file, Wait. Nothing saves while that question is open.
