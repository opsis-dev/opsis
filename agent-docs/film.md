# The film page and rendering

The film page and its protocol, the frame driver that renders it, the paint traps of drawing under begin-frame control, canvas transitions, transitions between clips, other shapes and window frames. Paths are relative to the repository root. Elements are in [elements.md](elements.md), footage in [footage.md](footage.md), asset addresses in [assets.md](assets.md), saved pictures and keys in [runs.md](runs.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                        | Where                                                                                                                                                                                   |
| ------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Page protocol (`window.opsis`) | `packages/runtime/src/page.tsx` → `mountFilm`                                                                                                                                           |
| Film page app                  | `apps/film` (built into `film-bundle/`)                                                                                                                                                 |
| Film served                    | render file server, `/.opsis/film/` (`FILM_FOLDER`, `folder_routes`)                                                                                                                    |
| Frame driver                   | `crates/capture/src/film.rs` → `FrameDriver`, `film_switches`, `seek`                                                                                                                   |
| Launch switches                | `crates/capture/src/browser.rs` → `effective_switches`                                                                                                                                  |
| Encoding and stills            | `crates/render/src/frames.rs`; arguments `crates/media/src/codecs.rs` → `film_encode_args`; test `crates/render/tests/frames.rs`                                                        |
| Canvas transitions and filters | `packages/video/src/lib/transitions/` → `makeCanvasPresentation`, `makeCanvasFilter`, `untilPainted` (`canvas-presentation.tsx`), `scene-series.tsx` → `SceneSeries`                    |
| Cut boards                     | `packages/video/src/core/transition-cuts.tsx` → `CutBoardProvider`, `TransitionWrap`                                                                                                    |
| Sequence cuts                  | `cuts_for`, `sequence_clips` (`crates/compile/src`)                                                                                                                                     |
| Transition warnings            | `cover_row`; `TransitionMark.covers`                                                                                                                                                    |
| Other shapes                   | `packages/video/src/core/frame.ts` (`overlayScale`, `shapeInsets`, `zoomBoost`, `narrowPose`, `frameFor`, `NarrowBleedClip`); `containBox` in `packages/video/src/core/still-layout.ts` |
| Export sizes, shapes           | `crates/compile/src/nodes/output.rs` → `variant_size`; catalogue `shapes`, `variant_canvas`                                                                                             |
| Window frames                  | `WindowFrame`; `packages/video/src/camera/frame-address.ts` → `frameAddress`                                                                                                            |
| Paintable scale                | `PAINTABLE_SCALE`                                                                                                                                                                       |

## Rules

### The page and the driver

- **`window.opsis` is the whole protocol**: `load(spec)` mounts the film at frame 0 and resolves with its size, rate and length once every hold is released and the fonts are ready; `seek(frame)` commits the frame with `flushSync` and resolves the same way; `holds()` names what is still open.
- **The driver launches the browser with the film's own switches** (`film_switches`: the GL choice and the element-drawing feature) and, on Linux, under begin-frame control.
  - It probes once that a test page draws, tries again, and otherwise takes a screenshot per frame; the run log says which.
- **A render or still that fails while drawing with a graphics card setting checks whether the browser can draw WebGL with it (`draws_webgl`, `crates/capture/src/film.rs`)**, and when it can't, says so and names the Draw with choice to use instead (`FilmError::NoWebGl`, `crates/render/src/frames.rs`); the original error goes to the log.
  - On a Linux laptop with both Intel's and NVIDIA's Vulkan drivers, "Graphics card, newer drivers" starts ANGLE on Vulkan, then fails to share the first WebGL image (`initExternalMemory`, error -2) and the page hangs. Films without WebGL render, so nothing checks before a render.
  - The page loads while display-less frames tick; each frame is a seek, then one `beginFrame` with its picture. A frame with no damage repeats the last picture.
- **A frame that isn't ready within 30 s fails, naming the holds still open.**
- **The page only draws when the driver asks, so while a seek waits on holds the driver issues frames it doesn't capture** (`seek`). A hold may therefore wait for an animation frame or a canvas `paint` event (`untilPainted`); the captured frame comes after them.
- **Frames are JPEG at quality 80** into FFmpeg (`film_encode_args`).
- **A film is drawn in one tab, in frame order**, which the footage decoders rely on (each frame decoded once). Measure before adding tabs: shader backgrounds lose their WebGL context when several tabs draw at once.
- **Development film pages keep their source maps** so render errors point at source lines; release staging builds without them.

### Paint traps

- **Canvas effects draw into a canvas that is on the page.** Frames from a canvas given to `transferControlToOffscreen` never reach a begin-frame capture, and reading a WebGL `OffscreenCanvas` back as a bitmap on the page hangs it.
- **An SVG element whose CSS `scale` is exactly 0 stops being repainted when its scale changes**, so a scale that starts at 0 is kept above it (`PAINTABLE_SCALE`).

### Canvas transitions

- **The canvas transitions and filters draw their shaders only where Chromium's Canvas Draw Element is on, and fall back to CSS elsewhere**; the film's launch switches turn it on.
- **Feature switches are merged, never replaced (`effective_switches`)**, because Chromium keeps only the last `--enable-features`.
- **On that path each side of a transition hands its captured frame to `onElementImage`; `TransitionWrap` pairs the outgoing and incoming sides by the frame the cut starts on** (`CutBoardProvider`, one board per set of tracks) and draws both through the incoming side.
  - `SceneSeries` (the scenes inside one element, such as Data story's) reports to a board of its own the same way. A no-op there leaves the scene blank for the whole transition.

### Transitions

- **A transition overlaps the clips by its length unless Keep every clip whole is on** (`TransitionRef.hold`): then `sequence_clips` holds the outgoing clip's last frame and the incoming clip's first frame for the transition (group `remap`) and delays the incoming sound, so no clip loses screen time.
- **A Sequence's `cuts` give the cut before a port its own transition, length and hold** (`cuts_for`): `before` is a port name from the `cut` widget, so connecting other clips never moves it; the last entry for a port wins, and entries for empty ports are kept and ignored.
- **Cards warn when a transition covers more than 40% of a clip** (`cover_row`, fed `inputFrames`), when it is squeezed below 40% of its designed length, when Fit length cuts or speeds a lot, and when an element is shorter than 60% of its usual length; the Timeline outlines heavy transitions (`TransitionMark.covers`).
- **Transition marks carry the `slot` that owns them**; clicking a mark's label on the Timeline selects that node.

### Other shapes

- **Exports size the spec with `variant_size`**, and the project Inspector's canvas picker takes its shapes and sizes from the catalogue.
- **A recording in a shape narrower than itself keeps its own shape inside the frame (`containBox`) and the camera zooms harder** (`zoomBoost`, `narrowPose`): it never rests fully zoomed out and aims at the cursor between actions.
- **In a narrower shape the camera plans for that frame** (`tuning.narrow`): `frameFor` fits each target to the frame's width, centred, up to the boosted maximum, and moves only as far as it must to keep the click in shot.
- **The full-page styles (Proof, Ledger) show the whole page in any other shape (`NarrowBleedClip`) over a blurred copy of the footage**, with that camera only when narrower. Window styles in wider shapes and the `center`/`fit` choices keep the crop or bars.
- **Overlays size by the long side (`overlayScale`) and keep clear of `shapeInsets`; outside landscape, lower thirds move to the top** so captions own the bottom.
- **A Layers node with Keep this layout in other shapes wraps its layers in a group with `design` (the project's size)**, drawn scaled to fit, for marks placed over a picture.
- **`containBox` also sizes screenshot windows; the capture keeps its own copy** (`crates/capture/src/still_layout.rs`), and the two stay the same ([capture.md](capture.md)).

### Window frames

- **Without a Window frame a take has no frame.** The Window frame style and Device frame's browser `windowStyle` draw the title bar in `WindowFrame`.
- **The title bar shows the take's first page address unless the frame has Title bar text** (`window.text`, `window.mode`): "This address" keeps the typed text; "This host with the page's path" puts the typed host before the path on screen at each frame (`frameAddress`).
  - The address mode shows only for browser takes (`x-show-if-take`); the take logs every main-frame address change with its time ([capture.md](capture.md)).
