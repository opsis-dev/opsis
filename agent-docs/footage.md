# Footage and colour grades

Every video and graded image the film shows comes from Rust as footage; colour grades are lists of steps that either grade footage inside the film or grade an exported file. Rust paths are relative to `crates/`; page paths to `packages/video/src/`. Terminal grading is in [terminal.md](terminal.md), addresses and prepared files in [assets.md](assets.md); see also [film.md](film.md) and [AGENTS.md](../AGENTS.md).

## Code map

| Concern                   | Where                                                                                                                           |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| Footage uses              | `compile/src/footage.rs` → `footage_uses`, `FootageUse`; `types/src/recording.rs` → `FootageChain`                              |
| Registered for a render   | `render/src/footage.rs` → `film_footage`, mounted in `render/src/executors.rs` → `asset_server`                                 |
| Decoding and serving      | `footage` (`opsis-footage`): `footage/src/footage.rs` (sources, restarts), `footage/src/chain.rs` → `decoder_args`, `footage/src/serve.rs` |
| The editor's file player  | `ui/src/player.rs` (the same `Decoder` with a scale-only chain)                                                                 |
| Use names                 | `compile/src/mix/plan.rs` → `lookup_key`                                                                                        |
| Showing a frame           | `core/footage.tsx` → `FootageFrame`                                                                                             |
| Grade footage             | `compile/src/nodes/effects.rs` → `GRADED_COMPONENTS`                                                                            |
| Grade nodes               | `compile/src/node.rs` → `grade_modifier`; `compile/src/nodes/grade.rs` → `is_neutral_entry`                                     |
| Grade filter graph        | `media/src/grade.rs` → `grade_filters`; golden graphs `media/tests/golden/grade.rs`                                             |
| Apply grade               | `media/src/executors.rs` → `apply_grade`; `media/src/codecs.rs` → `grade_sound_args`; `compile/src/nodes/grade.rs` → `grade_needs_video` |
| Grade nodes' spot picture | `compile/src/inspector.rs` → `graded_file`                                                                                      |
| Mac cursor                | `capture/src/cursor.rs` → `read_shape`; `capture/src/page_scripts.rs` → `cursor_keyword`; `camera/lib/cursor.ts` → `shapeAt`; `camera/cursor.tsx` → `MacCursor`; `camera/lib/mac-cursors.generated.ts` (`scripts/mac-cursors.ts`) |

## Rules

### Footage

- **Every video the film shows is a footage use**: recordings in every style, the Video and Video loop elements, file settings whose kind is video (through each element's `kind_prop`), and every Image element with a grade; an image without one is served as it is. A terminal recording is a footage use too ([terminal.md](terminal.md)).
- **A use is the file, its `FootageChain` and, for Video and Video loop, the trim and loop, named by a hash of all of it (`lookup_key`, `compile/src/mix/plan.rs`)**; a terminal take's use also carries its theme.
- **`hosted` gives each use its address under `.opsis/footage/<id>/`** (a recording's `screen.footageUrl`, an element's `footageUrls` keyed by the file setting), and the render executor registers the same uses with `opsis-footage` beside the prepared files.
- **`opsis-footage` runs one FFmpeg decoder per source during a render**, terminal takes aside ([terminal.md](terminal.md)), with the source's grade in its filter graph (`decoder_args`, sound dropped) and Redact's areas per frame by their time ranges, encodes JPEG 4:4:4 (`jpeg-encoder`) and serves frames under `.opsis/footage/<id>/<µs>`.
  - A request before the kept frames or more than `JUMP_US` ahead restarts the decoder a little early (`SEEK_MARGIN_US`); the frame served is the last one at or before the time (`MATCH_TOLERANCE_US`).
  - Only a few recent frames and pictures are kept, in memory (`KEPT_FRAMES`, `KEPT_PICTURES`); nothing is cached on disk, and a decoder idle for `IDLE` or failing stops.
  - The answers are never cached by the page (`no-store`): 404 for an unknown use, 422 when decoding fails, 400 for a bad time.
- **The editor's file player decodes through the same `Decoder`** with a chain that only scales (`ui/src/player.rs`).
- **The page shows each frame with `FootageFrame`**: the time in microseconds after the address, the render held until the picture has decoded, and a frame that fails ends the render with the server's message.
- **Footage has no sound; a clip's sound comes from the mix** ([audio.md](audio.md)).
- **A footage chain is a grade and redactions.**
  - Grade footage (`effect.gradeFootage`, `clip + grade → clip`) appends a grade's steps to the footage of every recording and every Video and Image element in a clip (`GRADED_COMPONENTS`); other elements' videos, Video loop's included, keep their colours.
  - Redact take (`effect.redactTake`, `recording → recording`, Redact's settings with From and Until, areas drawn on any frame of the take) appends a redaction that stays on the page as the camera moves; its Text to hide applies to terminal takes only ([terminal.md](terminal.md)).
- **The window, backdrop and every other layer keep their colours**; Apply grade on an exported file grades everything.

### Grades

- **Grade nodes are `grade? → grade`** ([nodes.md](nodes.md) has how their entries are appended and passed through).
- **Each step's card row and neutral check come from one list of changes**, so a card's "No change" and the empty filter always agree (`is_neutral_entry`).
- **Apply grade (`media.applyGrade`, `file + grade → file`) turns the list into one filter graph and encodes once**: the sound is copied when the container holds it and encoded again otherwise (`grade_sound_args`).
  - An image comes out as one PNG frame (`graded.png`); Texture's grain and Lens trails need a video and are left out of a still, with a warning (`grade_needs_video`).
- **`grade_filters` is pure and checked against golden graphs**: steps that need no blend are joined with commas, a step below full strength splits and blends back, a look image is the next numbered input to `haldclut`, and paths are escaped for the graph.
- **A grade node's spot picture (Texture's dark edges among them) shows a frame of the file that reaches Apply grade** (`graded_file`).

### The Mac cursor

- **A take with the synthetic cursor reads the page's own cursor keyword** (`read_shape`): the arrow at a glide's start, then once when the glide settles and after each click, stored as `c` on the cursor sample only when it changes, so the Mac cursor (`pointer.look.shape === "mac"`) can draw it.
  - The page carries the last `c` forward to each later sample, and the arrow before any (`shapeAt`).
  - `cursor_keyword` keeps CSS keywords only: anything else becomes the arrow, and `auto` becomes `text` over an editable element.
- **The film draws each keyword with apple_cursor's macOS set**, an unknown keyword as the arrow and `none` as nothing.

## Deliberate limits

- **A `url()` cursor is never loaded**; the Mac cursor knows CSS keywords only.
