# Runs, results and cache keys

How a node's key is made, how results follow keys as the workflow changes, how Media file nodes are checked without a run, how run state reaches the windows, and what each version constant keys. Paths are relative to `crates/` unless they name another folder. Executors are in [nodes.md](nodes.md), the engine's own files in [engine.md](engine.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                       | Where                                                                                            |
| ----------------------------- | ------------------------------------------------------------------------------------------------ |
| Keys                          | `runner/src/keys.rs` → `ENGINE_VERSION`, `Release`; `hash_nodes`                                 |
| Stamps                        | `runner/src/context.rs` → `Executor::stamped`; `project` → `file_stamps`                         |
| File hashes                   | `engine/src/hash.rs` → `versioned_hash`                                                          |
| Picture and prepared versions | `engine/src/versions.rs` → `PICTURE_VERSION`, `PREPARED_VERSION`                                 |
| Mix version                   | `media/src/mix/render.rs` → `MIX_VERSION`                                                        |
| Saved pictures                | `render/src/picture.rs`; `.opsis/cache/pictures/<pictureKey>`                                    |
| Results following keys        | `desktop/src/node_results.rs` → `refresh`, `adopt`, `held`, `check_media_files`, `missing_files` |
| Media file check              | `media/src/executors.rs` → `check_media_file`, `FileCheckError`                                  |
| Run view                      | `desktop/src/run.rs` → `RunHost`, `seeded_view`, `exported_artifacts`, `notify_finished`         |
| Run event reducer             | `runner/src/progress.rs` → `apply_to_view`, `apply_run_event`                                    |
| Editor mirror                 | `apps/frontend/src/store/run.ts` → `useRunMirror`                                                |

## Rules

### Keys

- **A node hash is `sha256(ENGINE_VERSION, type, interpolated params, upstream hashes)`**, plus the size and modified time of every project file its settings name (`file_stamps`, so a file replaced in place runs again).
- **Render-phase nodes add the project settings, the variables (a node such as Choose by variable reads their values directly), the mix, `PICTURE_VERSION` and `PREPARED_VERSION`.**
- **An executing node that reads the project's width, height or frame rate sets `uses_project_size`; one that draws text in the project's fonts sets `uses_project_fonts`**, so those are in its key.
- **An executor whose output depends on something outside its settings and inputs adds it with a stamp (`Executor::stamped`)**, read when the keys are worked out ([capture.md](capture.md), sign-in stamps).
- **Node keys and picture keys hold the app's version, and node keys the recording browser's hash (`Release`)**, so a new release or another browser runs everything again. The bundled FFmpeg changes only with a release.
- **Every key that picks a file on disk is a sha256 (`versioned_hash`)**; the 64-bit `lookup_key` only matches values in memory, such as a mix plan to its result.
- **Export cache keys include the project settings, so changing fonts re-exports.**
- **The working files' encode speed (`settings.media.encodeSpeed`, Settings → General) is in no key**: a file made at another speed is still right, so changing it runs nothing again, and a node uses the new speed the next time it runs for its own reasons. Every media node that makes a video reads it (`encode_speed`, `media/src/executors.rs`); Render keeps its own Speed vs file size.

### Version constants

- **Each constant keys one kind of output**, and once Opsis is released it changes when that output changes; before release they stay as they are ([AGENTS.md](../AGENTS.md) §2):
  - `ENGINE_VERSION`: an executor's output format;
  - `PICTURE_VERSION`: how a spec is drawn (anything in `packages/video` the spec doesn't record); it keys saved pictures and every render-phase node;
  - `PREPARED_VERSION`: a preparer's output; it keys prepared files, saved pictures and every render-phase node;
  - `MIX_VERSION`: stem output.

### Saved pictures

- **The render executor draws the picture once without sound into `.opsis/cache/pictures/<pictureKey>`**; the key leaves out the spec's audio, mix and asset address, and recent pictures are kept. The master stem is added afterwards ([audio.md](audio.md)).

### Results follow the keys

- **Compile hands each node its last result, and after every compile the app works out every executing node's key** (`refresh`, through `hash_nodes`).
  - When a result was made under another key and `.opsis/cache/<key>` holds one for the current key, that result takes its place (undo brings the earlier result back).
  - One that still doesn't match goes in the compile's `outdated`: a Changed badge on the card, Changed since the last run in the Run panel.
- **While a gesture is open, compiles keep the keys worked out last (`held`)** and only the problems of unreadable files are refreshed, with no hashing and no file checks; closing the gesture compiles again with fresh keys.
- **Results and checks land only in the project they were made for (`adopt`)**, and a change in the engine's status compiles again.
- **A Media file whose key has no result is checked without a run (`check_media_files`)** with the same `check_media_file` the run uses: kind, length, picture size, byte size and frame rate, saved under the key so the next run reuses it and the card shows it.
  - A file that isn't a video, image or sound is a warning until the file changes; one that can't be opened (a folder, no permission) is a warning checked again every 10 s (`FileCheckError`); a missing one is "Not in the project folder" (`missing_files`).
- **The Media file node uses its result only while its path names that file.**

### Run state

- **Rust keeps the whole run view (`RunView` and the capped log, `RunHost`) and applies every run event with one reducer** (`apply_to_view` over `apply_run_event`).
- **Windows mirror it (`useRunMirror`)** from `run_state` when they mount and `RunViewChanged`, `RunLogged` and `RunStateReset` after, dropping a view older than the one they show (`seq`).
- **Opening a project seeds the view from its last run (`seeded_view`)**: each node's newest trace, failed or not, and failures newer than the node's last result, unless that project is the one running. Clearing the cache seeds it empty, and an edit forgets the errors of the nodes it touched.
- **A run clears status, progress, errors and traces only for the nodes it includes (`run:start` `nodeIds`)**, so a Run this never hides an earlier failure; `run:done` names `failedNodeIds` and `skippedNodeIds` apart, and the panel shows each failed node as a link.
- **A node whose executor panics fails with "This node stopped unexpectedly." (`run_node`, which logs the panic)**, so the other nodes go on and the run still ends with `run:done`; only `run:done` frees the app for the next run.
- **A run that ends while the main window isn't focused says so in a system notification sent from Rust (`notify_finished`).** The plugin's page script asks whether notifications are allowed in every window that loads, so the default capability allows `notification:allow-is-permission-granted` and nothing else of it.
- **The exports list comes from the results of every `output.*` node, newest first (`exported_artifacts`, sent as `exports`)**, not from the last run's `outputs`.
- **Saved results belong to the project**; clearing the cache is in the project Inspector.
