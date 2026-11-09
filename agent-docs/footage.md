# Footage and colour grades

Every video and graded image the film shows comes from Rust as footage; colour grades are lists of steps that either grade footage inside the film or grade an exported file. Rust paths are relative to `crates/` ; page paths to `packages/video/src/`. Terminal grading is in [terminal.md](terminal.md), addresses and prepared files in [assets.md](assets.md); see also [film.md](film.md) and [AGENTS.md](../AGENTS.md).

## Code map

| Concern                   | Where                                                                                                                           |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| Footage uses              | `compile/src/footage.rs` → `footage_uses`, `FootageUse`; `types/src/recording.rs` → `FootageChain`                              |
| Registered for a render   | `render/src/footage.rs` → `film_footage`                                                                                        |
| Decoding and serving      | `footage` (`opsis-footage`)                                                                                                     |
| Showing a frame           | `core/footage.tsx` → `FootageFrame`                                                                                             |
| Grade footage             | `compile/src/nodes/effects.rs` → `GRADED_COMPONENTS`                                                                            |
| Grade nodes               | `compile/src/node.rs` → `grade_modifier`; `compile/src/nodes/grade.rs` → `is_neutral_entry`                                     |
| Grade filter graph        | `media/src/grade.rs` → `grade_filters`; golden graphs `media/tests/golden/grade.rs`                                             |
| Grade nodes' spot picture | `compile/src/inspector.rs` → `graded_file`                                                                                      |
| Mac cursor                | `capture/src/cursor.rs` → `read_shape`; `capture/src/page_scripts.rs` → `cursor_keyword`; `camera/lib/mac-cursors.generated.ts` |

## Rules

### Footage

- **Every video the film shows is a footage use**: recordings in every style, the Video and Video loop elements, file settings whose kind is video (through each element's `kind_prop`), and every Image element with a grade; an image without one is served as it is. A terminal recording is a footage use too ([terminal.md](terminal.md)).
- **A use is the file, its `FootageChain` and, for Video and Video loop, the trim and loop, named by a hash of all of it (`lookup_key`)**; a terminal take's use also carries its theme.
- **`hosted` gives each use its address under `.opsis/footage/<id>/`** (a recording's `screen.footageUrl`, an element's `footageUrls` keyed by the file setting), and the render executor registers the same uses with `opsis-footage` beside the prepared files.
- **`opsis-footage` runs one streaming FFmpeg decoder per source during a render**, terminal takes aside ([terminal.md](terminal.md)), with the source's grade in its filter graph and Redact's areas per frame by their time ranges, encodes JPEG 4:4:4 (`jpeg-encoder`) and serves frames under `.opsis/footage/<id>/<µs>`. Nothing is cached on disk.
- **The page shows each frame with `FootageFrame`**: the time in microseconds after the address, the render held until the picture has decoded, and a frame that fails ends the render with the server's message.
- **Footage has no sound; a clip's sound comes from the mix** ([audio.md](audio.md)).
- **A footage chain is a grade and redactions.**
  - Grade footage (`effect.gradeFootage`, `clip + grade → clip`) appends a grade's steps to the footage of every recording and every Video and Image element in a clip (`GRADED_COMPONENTS`); other elements' videos, Video loop's included, keep their colours.
  - Redact take (`effect.redactTake`, `recording → recording`, Redact's settings with From and Until, areas drawn on any frame of the take) appends a redaction that stays on the page as the camera moves.
- **The window, backdrop and every other layer keep their colours**; Apply grade on an exported file grades everything.

### Grades

- **Grade nodes are `grade? → grade` and append `{ nodeId, type, params }` (`grade_modifier`)**; a skipped one passes its input through. Nothing converts into a `grade`.
- **Each step's card row and neutral check come from one list of changes**, so a card's "No change" and the empty filter always agree (`is_neutral_entry`).
- **Apply grade (`media.applyGrade`, `file + grade → file`) turns the list into one filter graph and encodes once**, so the sound is copied when the container holds it.
- **`grade_filters` is pure and checked against golden graphs**: steps that need no blend are joined with commas, a step below full strength splits and blends back, a look image is the next numbered input to `haldclut`, and paths are escaped for the graph.
- **A grade node's spot picture (Texture's dark edges among them) shows a frame of the file that reaches Apply grade** (`graded_file`).

### The Mac cursor

- **A take with the synthetic cursor reads the page's own cursor keyword** (`read_shape`): once when a glide settles and after each click, stored as `c` on the cursor sample, so the Mac cursor (`pointer.look.shape === "mac"`) can draw it.
  - `cursor_keyword` keeps CSS keywords only: anything else becomes the arrow, and `auto` becomes `text` over an editable element.
- **The film draws each keyword with apple_cursor's macOS set**, an unknown keyword as the arrow and `none` as nothing.

## Deliberate limits

- **A `url()` cursor is never loaded**; the Mac cursor knows CSS keywords only.
