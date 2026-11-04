# Timeline

How the compiled spec becomes bars, how Timeline drags write back into the nodes, keyframes and camera keys, the Timeline's clock and sound, and chapters in exports. Rust paths are relative to `crates/compile/src/`, editor paths to `apps/frontend/src/`. Gestures and edits in general are in [document.md](document.md), the Mixer in [audio.md](audio.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                       | Where                                                                                            |
| ----------------------------- | ------------------------------------------------------------------------------------------------ |
| Flattener                     | `timeline/flatten.rs` → `flatten_spec`, `VideoBar::local_frames` (sent as `flat`, `localFrames`) |
| Timeline edits                | `edits/timeline.rs` → `TimelineEdits`, `timeline_removal`, `apply_removal`, `close_gaps`         |
| Handle round trips            | `crates/compile/tests/unit/timeline_edits.rs`                                                    |
| Envelopes                     | `envelope.rs`                                                                                    |
| Keyframe and camera key edits | `edits/keyframes.rs` → `KeyframeEdit`, `apply_keyframe_edit`                                     |
| Tracks, easings, camera dials | catalogue `timeline` (`TimelineBook`)                                                            |
| Chapters                      | `timeline/chapters.rs` → `film_chapters`; `finish_args` (`crates/media/src/codecs.rs`)           |
| Clock                         | `editor/timeline/timeline-clock.ts` → `TimelineClock`, `usePlayback`                             |
| Loops                         | `timeline/playhead.ts` → `loopOf`, `wrapInLoop`                                                  |
| Stem player                   | `editor/stem-player.ts` → `StemPlayer`                                                           |
| Active nodes and bars         | `activeAt` over the compiled `spans`, `setActive`; `BarFrame`                                    |

## Rules

### Bars

- **`flatten_spec` turns the compiled spec into bars on lanes (greedy packing in drawing order), audio bars, transitions and action marks**, mapping frames through trims, loops and speed remaps, with each bar's local frame at every timeline frame for filmstrips and keys.
- **Edits never touch the spec.** Compile records `handles` on items (`move`, `trimStart`, `trimEnd`, `speed`, `volume` as `ParamRef`s), and container children carry a `slot` (container node + port).

### Edits

- **`TimelineEdits` applies an edit to the owning setting or, when nothing owns it, inserts a named node** ("Delay from timeline", "Trim from timeline", "Speed from timeline", split parts) at the slot edge or right after the clip. Ripples, rolls, splits, volume, camera keys and the removal of selected clips live there too.
- **The editor only snaps (pixels, beats, markers, the playhead) and sends the snapped delta as a `TimelineEdit` naming the bar by its key.**
- **A drag is a gesture: each move applies to the workflow at the gesture's start, against the timeline Rust last sent when the gesture began**, with ids salted by the gesture so an added node keeps its id from move to move.
- **Every handle has a round-trip test in `timeline_edits.rs`; add one when a node gains a handle.**
- **Removing clips closes the gaps and splitting shifts ports up, both through `renumber_family`** ([nodes.md](nodes.md)); `splitBar` splits inside any Sequence, shifting later ports and cuts by one and joining the halves with a plain cut.

### Keyframes

- **Keyframes are an `envelope` setting, in frames local to the item**, shown on the Timeline and in the Inspector (`envelope` widget); they compile onto `TimelineItem.envelope`/`AudioItem.envelope` and stay pure functions of the frame.
- **Rust applies every keyframe and camera key edit (`apply_keyframe_edit`)**: the Inspector sends it as a `WorkflowEdit`, the Timeline as a `TimelineEdit`, so a drag applies to the keys as they were when it began.
  - Keys are found by their stored position, values kept in the track's range and frames whole, and the keys sorted after every edit.
- **The tracks' labels, ranges and rest values, the easing names and the camera dials come from Rust in the catalogue (`TimelineBook`)**; the camera dials also give `CameraKey` its schema ranges. The editor only draws them and checks a value's shape.

### The Timeline clock

- **The Timeline has its own clock and nothing else: `TimelineClock`, mounted in the editor window, owns `usePlayback`.** It moves the playhead on animation frames at the shuttle rate, loops the film or the in and out points (`loopOf`/`wrapInLoop`), and plays only the film's master stem (`StemPlayer`: an `<audio>` through a Web Audio chain, kept within two frames of the playhead).
- **Play and Space in the Timeline, the Mixer's meters, the camera field and the node rings all follow that playhead**: the canvas rings the nodes whose span holds it (`activeAt`, `setActive`) and the Timeline brightens the bars under it (`BarFrame`).
- **Selecting a node moves the playhead to its start.**
- **The Timeline plays the latest stem for its film** ([audio.md](audio.md)).

### Chapters in exports

- **`film_chapters` reads the Chapter action marks from `flatten_spec`**, starts the first at 0:00 (or adds an "Intro" chapter when the first starts 10 s or more in) and merges chapters under 10 s into the one before, YouTube's rules.
- **Export video writes them as FFmpeg chapter metadata in the finishing copy (`finish_args`, with the web-ready move)**, so a chapter change never renders the picture again; GIFs get none.
- **The optional `.youtube-chapters.txt` needs at least three chapters.**
