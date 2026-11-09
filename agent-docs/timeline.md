# Timeline

How the compiled spec becomes bars, how Timeline drags write back into the nodes, keyframes and camera keys, the Timeline's clock and sound, and chapters in exports. Rust paths are relative to `crates/compile/src/`, C++ paths to `crates/qt/cpp/`. Gestures and edits in general are in [document.md](document.md), the Mixer in [audio.md](audio.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                       | Where                                                                                                                                                            |
| ----------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Flattener                     | `timeline/flatten.rs` → `flatten_spec`, `VideoBar::local_frames` (sent as `flat`, `localFrames`)                                                                 |
| Timeline edits                | `edits/timeline.rs` → `TimelineEdits`, `apply_removal`, `close_gaps`; `crates/document/src/lib.rs` → `timeline_removal`                                          |
| Handle round trips            | `crates/compile/tests/unit/timeline_edits.rs`                                                                                                                    |
| Envelopes                     | `envelope.rs`                                                                                                                                                    |
| Keyframe and camera key edits | `edits/keyframes.rs` → `KeyframeEdit`, `apply_keyframe_edit`                                                                                                     |
| Tracks, easings, camera dials | catalogue `timeline` (`catalog.rs` → `TimelineBook`)                                                                                                             |
| Chapters                      | `timeline/chapters.rs` → `film_chapters`, `MIN_LISTED_CHAPTERS`; `crates/render/src/executors.rs` → `chapters_for`; `finish_args` (`crates/media/src/codecs.rs`) |
| Clock                         | `playback_clock.cc` → `PlaybackClock`; `crates/ui/src/ui.rs` → `share_playhead`, `wrap_playhead`, `audio_clock`, `audio_seek`                                    |
| Loops                         | `timeline/view.rs` → `loop_of`, `wrap_in_loop`                                                                                                                   |
| Stem player                   | `crates/audio/src/output.rs` → `Output`                                                                                                                          |
| Active nodes                  | `timeline/view.rs` → `active_at` over the compiled `spans`; `crates/ui/src/graph.rs`; `graph_canvas.cc`                                                          |

## Rules

### The view

- **The Timeline paints itself on a `QAbstractScrollArea` (`crates/qt/cpp/timeline_canvas.cc` → `TimelineCanvas`), not a `QGraphicsView`** like the canvas: it zooms on time only, so a scene transform would stretch its labels; its ruler and track labels stay put; and painting straight from the Rust view keeps no scene items in step.

### Bars

- **`flatten_spec` turns the compiled spec into bars on lanes (greedy packing in drawing order), audio bars, transitions and action marks**, mapping frames through trims, loops and speed remaps, with each bar's local frame at every timeline frame for filmstrips and keys.
- **Edits never touch the spec.** Compile records `handles` on items (`move`, `trimStart`, `trimEnd`, `speed`, `volume` as `ParamRef`s), and container children carry a `slot` (container node + port).

### Edits

- **`TimelineEdits` applies an edit to the owning setting or, when nothing owns it, inserts a named node** ("Delay from timeline", "Trim from timeline", "Speed from timeline", split parts) at the slot edge or right after the clip. Ripples, rolls, splits, volume, camera keys and the removal of selected clips live there too.
- **The editor only snaps (pixels, beats, markers, the playhead) and sends the snapped delta as a `TimelineEdit` naming the bar by its key.**
- **A drag is a gesture: each move applies to the workflow at the gesture's start, against the timeline Rust last sent when the gesture began**, with ids salted by the gesture so an added node keeps its id from move to move.
- **Every handle has a round-trip test in `timeline_edits.rs`; add one when a node gains a handle.**
- **Removing clips closes the gaps and splitting shifts ports up, both through `renumber_family`** ([nodes.md](nodes.md)); `splitBar` splits inside any Sequence, shifting later ports and cuts by one and, when the Sequence has a transition, joining the halves with a plain cut (`split_in_sequence`); a clip outside a Sequence is split into a new one (`split_with_new_sequence`).

### Keyframes

- **Keyframes are an `envelope` setting, in frames local to the item**, shown on the Timeline and in the Inspector (`envelope` widget); they compile onto the item's `envelope` (`ItemBase`) or the sound's (`AudioItem`) and stay pure functions of the frame.
- **Rust applies every keyframe and camera key edit (`apply_keyframe_edit`)**: the Inspector sends it as a `WorkflowEdit`, the Timeline as a `TimelineEdit`, so a drag applies to the keys as they were when it began.
  - Keys are found by their stored position, values kept in the track's range and frames whole, and the keys sorted after every edit.
- **The tracks' labels, ranges and rest values, the easing names and the camera dials come from Rust in the catalogue (`TimelineBook`)**; the camera dials also give `CameraKey` its schema ranges. The editor only draws them and checks a value's shape.

### The Timeline clock

- **The Timeline has its own clock and nothing else: `PlaybackClock`, one for the editor.** It moves the playhead at the shuttle rate, following the sound output's clock while the film plays (`audio_clock`), loops the film or the in and out points (`loop_of`/`wrap_in_loop`), and plays only the film's master stem (`Output`).
  - When wrapping puts the playhead more than a frame from where the sound is, it seeks the sound there (`audio_seek`).
- **Play and Space in the Timeline, the Mixer's meters, the camera field and the node rings all follow that playhead**: the canvas rings the nodes whose span holds it (`active_at`).
- **Selecting a node moves the playhead to its start** (`graph_canvas.cc`).
- **The Timeline plays the latest stem for its film** ([audio.md](audio.md)).

### Chapters in exports

- **`film_chapters` reads the Chapter action marks from `flatten_spec` and follows YouTube's rules**: marks at or after the film's end are dropped; the first mark moves to 0:00 when it is under 10 s in, or else a chapter named by Export video's First chapter name is added when the first mark is 10 s or more in; a chapter starting under 10 s after the one kept before it is dropped, and so are trailing chapters shorter than 10 s.
- **Export video writes them as FFmpeg chapter metadata in the finishing copy when its Chapters in the file setting is on (`chapters_for`, `finish_args`, with the web-ready move)**, so a chapter change never renders the picture again; GIFs get none.
- **The optional `.youtube-chapters.txt` (Export video's YouTube chapter list, apart from Chapters in the file) needs `MIN_LISTED_CHAPTERS`**; with fewer, but some, the run logs a warning and saves no list.
