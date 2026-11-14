# Footage and colour grades

Every video and graded image the film shows comes from Rust as footage; colour grades are lists of steps that either grade footage inside the film or grade an exported file. Paths are relative to `crates/`. Terminal grading is in [terminal.md](terminal.md), addresses and prepared files in [assets.md](assets.md); see also [film.md](film.md) and [AGENTS.md](../AGENTS.md).

## Code map

| Concern                   | Where                                                                                                                                                                                                                                             |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Footage uses              | `compile/src/footage.rs` → `footage_uses`, `FootageUse`; `types/src/recording.rs` → `FootageChain`                                                                                                                                                |
| Registered for a render   | `render/src/footage.rs` → `film_footage`, handed to `render/src/native.rs` → `NativeFiles`                                                                                                                                                        |
| Decoding                  | `footage` (`opsis-footage`): `footage/src/footage.rs` (sources, restarts, `Footage::shown`), `footage/src/chain.rs` → `decoder_args`                                                                                                              |
| The editor's file player  | `ui/src/player.rs` (the same `Decoder` with a scale-only chain)                                                                                                                                                                                   |
| Use names                 | `compile/src/mix/plan.rs` → `lookup_key`                                                                                                                                                                                                          |
| Showing a frame           | `render/src/native.rs` → `NativeFiles::frame`; the footage leaf `scene/src/node.rs` → `FootageFrame`; recordings `compile/src/elements/camera/recording.rs`                                                                                       |
| Grade footage             | `compile/src/nodes/effects.rs` → `GRADED_COMPONENTS`                                                                                                                                                                                              |
| Grade nodes               | `compile/src/node.rs` → `grade_modifier`; `compile/src/nodes/grade.rs` → `is_neutral_entry`                                                                                                                                                       |
| Grade filter graph        | `media/src/grade.rs` → `grade_filters`; golden graphs `media/tests/golden/grade.rs`                                                                                                                                                               |
| Apply grade               | `media/src/executors.rs` → `apply_grade`; `media/src/codecs.rs` → `edit_video`, `graded_sound_args`; `compile/src/nodes/grade.rs` → `grade_needs_video`                                                                                           |
| Grade nodes' spot picture | `compile/src/inspector.rs` → `graded_file`                                                                                                                                                                                                        |
| Mac cursor                | `capture/src/cursor.rs` → `read_shape`; `capture/src/page_scripts.rs` → `cursor_keyword`; `compile/src/elements/camera/cursor.rs` → `shape_at`, `mac_cursor`; `compile/src/elements/camera/mac-cursors.generated.json` (`scripts/mac-cursors.ts`) |

## Rules

### Footage

- **Every video the film shows is a footage use**: recordings in every style, the Video and Video loop elements, file settings whose kind is video (through each element's `kind_prop`), and every Image element with a grade; an image without one is read as it is. A terminal recording is a footage use too ([terminal.md](terminal.md)).
- **A use is the file, its `FootageChain` and, for Video and Video loop, the trim and loop, named by a hash of all of it (`lookup_key`, `compile/src/mix/plan.rs`)**; a terminal take's use also carries its theme.
- **`hosted` gives each use its address under `.opsis/footage/<id>/`** (a recording's `screen.footageUrl`, an element's `footageUrls` keyed by the file setting), and the render executor registers the same uses with `opsis-footage`, which `NativeFiles` reads them from.
- **`opsis-footage` runs one FFmpeg decoder per source during a render**, terminal takes aside ([terminal.md](terminal.md)), with the source's grade in its filter graph (`decoder_args`, sound dropped) and Redact's areas per frame by their time ranges, and hands each frame over decoded as RGB (`Footage::shown`) for `.opsis/footage/<id>/<µs>`.
  - A request before the kept frames or more than `JUMP_US` ahead restarts the decoder a little early (`SEEK_MARGIN_US`); the frame shown is the last one at or before the time (`MATCH_TOLERANCE_US`).
  - Only a few recent frames are kept, in memory (`KEPT_FRAMES`); nothing is cached on disk, and a decoder idle for `IDLE` or failing stops.
  - A decoder that sends nothing for 30 s (`STALL_LIMIT`, `footage/src/decoder.rs`) fails the render (`FootageError::Stalled`).
- **The editor's file player decodes through the same `Decoder`** with a chain that only scales (`ui/src/player.rs`).
- **The film shows each frame as a footage leaf** (`FootageFrame`): its address names the use and the time in microseconds, and a frame that can't be decoded leaves the leaf empty and logs why.
- **Footage has no sound; a clip's sound comes from the mix** ([audio.md](audio.md)).
- **A footage chain is a grade and redactions.**
  - Grade footage (`effect.gradeFootage`, `clip + grade → clip`) appends a grade's steps to the footage of every recording and every Video and Image element in a clip (`GRADED_COMPONENTS`); other elements' videos, Video loop's included, keep their colours.
  - Redact take (`effect.redactTake`, `recording → recording`, Redact's settings with From and Until, areas drawn on any frame of the take) appends a redaction that stays on the recorded page as the camera moves; its Text to hide applies to terminal takes only ([terminal.md](terminal.md)).
- **The window, background and every other layer keep their colours**; Apply grade on an exported file grades everything.

### Grades

- **Grade nodes are `grade? → grade`** ([nodes.md](nodes.md) has how their entries are appended and passed through).
- **Each step's card row and neutral check come from one list of changes (`changes` in `compile/src/nodes/grade.rs`)**, so a card's "No change" and the "The grade changes nothing" warning agree (`is_neutral_entry`, `grade_changes_nothing`).
  - The filter graph is built apart from that list (`grade_filters`, `media/src/grade.rs`), and no test checks that a step the list calls neutral adds no filter.
- **Apply grade (`media.applyGrade`, `file + grade → file`) has no settings: it turns the list into one filter graph and encodes once**, a video as an MP4 at the editing quality the file nodes share (`edit_video`) and an image as a PNG; the sound is copied when MP4 holds it and made AAC otherwise (`graded_sound_args`). Convert chooses another format.
- **Apply grade takes image files too, so the grade nodes (Colour and light with its Invert, Auto fix, Texture, Black and white and tint) are also the image colour fixes** ([images.md](images.md)).
  - An image comes out as one PNG frame (`graded.png`); Texture's grain and Lens trails need a video and are left out of a still, with a warning (`grade_needs_video`).
- **`grade_filters` is pure and checked against golden graphs**: steps that need no blend are joined with commas, a step below full strength splits and blends back, a look image is the next numbered input to `haldclut`, and paths are escaped for the graph.
- **A grade node's spot picture (Texture's dark edges among them) shows a frame of the file that reaches Apply grade** (`graded_file`).

### The Mac cursor

- **A take with the synthetic cursor reads the page's own cursor keyword** (`read_shape`): the arrow at a glide's start, then once when the glide settles and after each click, stored as `c` on the cursor sample only when it changes, so the Mac cursor (the pointer look's Mac shape) can draw it.
  - The film carries the last `c` forward to each later sample, and the arrow before any (`shape_at`).
  - `cursor_keyword` keeps CSS keywords only: anything else becomes the arrow, and `auto` becomes `text` over an editable element.
- **The film draws each keyword with apple_cursor's macOS set**, an unknown keyword as the arrow and `none` as nothing.

## Deliberate limits

- **A `url()` cursor is never loaded**; the Mac cursor knows CSS keywords only.
