# The film page and rendering

The film page and its protocol, the frame driver that renders it, the paint traps of drawing under begin-frame control, canvas transitions, transitions between clips and the `enter` they ride on, other shapes and window frames. Paths are relative to the repository root. Elements are in [elements.md](elements.md), footage in [footage.md](footage.md), asset addresses in [assets.md](assets.md), saved pictures and keys in [runs.md](runs.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                        | Where                                                                                                                                                                                                                                                                         |
| ------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Page protocol (`window.opsis`) | `packages/runtime/src/page.tsx` → `mountFilm`, `ready`; holds in `packages/runtime/src/holds.ts` (`holdRender`, `failRender`, `settled`); the clock `packages/runtime/src/clock.tsx` → `FilmClock`; `packages/video/src/film.tsx` → `FILM`                                    |
| Render environment             | `packages/runtime/src/environment.ts` → `canDrawElementsInCanvas`, `markRendering`, `isRendering`                                                                                                                                                                             |
| Film page app                  | `apps/film` (built into `film-bundle/`)                                                                                                                                                                                                                                       |
| Film served                    | render file server, `/.opsis/film/` (`crates/compile/src/asset_ref.rs` → `FILM_FOLDER`; `crates/engine/src/http_files.rs` → `folder_routes`)                                                                                                                                  |
| Frame driver                   | `crates/capture/src/film.rs` → `FrameDriver`, `Capture`, `film_switches`, `seek`, `ready_within`, `PROBE_ATTEMPTS`, `FRAME_TIMEOUT`, `PAGE_TIMEOUT`                                                                                                                           |
| Launch switches                | `crates/capture/src/browser.rs` → `effective_switches`                                                                                                                                                                                                                        |
| Encoding and stills            | `crates/render/src/frames.rs` → `FRAME_QUALITY`, `render_still`, `still_format`; arguments `crates/media/src/codecs.rs` → `film_encode_args`; test `crates/render/tests/frames.rs`                                                                                            |
| Canvas transitions and filters | `packages/video/src/lib/transitions/` → `makeCanvasPresentation`, `makeCanvasFilter`, `untilPainted` (`canvas-presentation.tsx`)                                                                                                                                              |
| Cut boards                     | `packages/video/src/core/transition-cuts.tsx` → `CutBoardProvider`; `packages/video/src/demo-timeline.tsx` → `TransitionWrap`                                                                                                                                                 |
| Transitions and joins          | `crates/compile/src/nodes/transitions.rs` → `transition_of`, `joins_for`, `join_clips`, `leak_track`; `crates/compile/src/clip.rs` → `sequence_clips`, `sequence_cuts`; `crates/types/src/values.rs` → `ClipValue.enter`; `crates/compile/src/graph.rs` → `transition_issues` |
| Transition warnings            | `crates/compile/src/card_rows.rs` → `cover`, `cover_row`, `squeezed_too_much`, `TRANSITION_WARN_RATIO`; `crates/compile/src/nodes/elements.rs` → `SHORT_ELEMENT_RATIO`; `crates/compile/src/timeline/flatten.rs` → `TransitionMark.covers`                                    |
| Other shapes                   | `packages/video/src/core/frame.ts` (`overlayScale`, `shapeInsets`, `zoomBoost`); `packages/video/src/camera/lib/camera-track.ts` (`narrowPose`, `frameFor`); `packages/video/src/camera/demo-clip.tsx` → `NarrowBleedClip`; `containBox` in `packages/video/src/core/size.ts` |
| Export sizes, shapes           | `crates/compile/src/nodes/output.rs` → `variant_size`, `variant_canvas`, `CANVAS_SHORT_SIDE`; the canvas picker's shapes `crates/ui/src/project.rs` → `shapes`                                                                                                                |
| Keep this layout               | `crates/compile/src/nodes/timeline.rs` → `keep_layout`; the design box is in [elements.md](elements.md)                                                                                                                                                                       |
| Window frames                  | `packages/video/src/camera/window-frame.tsx` → `WindowFrame`; `packages/video/src/camera/frame-address.ts` → `frameAddress`, `terminalTitle`, used by `packages/video/src/core/recording-clip.tsx`                                                                            |
| Paintable scale                | `packages/video/src/elements/data/animated-bar-chart.tsx` → `PAINTABLE_SCALE`                                                                                                                                                                                                 |

## Rules

### The page and the driver

- **Every timeline item has a `kind` (`group`, `recording` with typed `GroupProps`/`RecordingProps`, `element` naming a registry `component`), and the page reads an item by narrowing on `kind`, never by component name.**
- **`window.opsis` is the whole protocol**: `load(spec)` mounts the film at frame 0 and resolves with its size, rate and length once every hold is released and the fonts are ready; `seek(frame)` commits the frame with `flushSync` and resolves, with nothing, once the holds are released and the fonts are ready; `holds()` names what is still open.
  - A React error while drawing fails the render (`failRender` from the root's error handlers in `mountFilm`), so `load` and `seek` reject with it.
- **The driver launches the browser with the film's own switches** (`film_switches`: the GL choice and the element-drawing feature) and, on Linux, under begin-frame control (`Capture::for_this_computer`); on Windows and macOS it takes a screenshot per frame.
  - A film whose scale isn't 1, or a caller that asks for screenshots, takes a screenshot per frame too.
  - Otherwise it probes that a test page draws (`PROBE_ATTEMPTS`), and falls back to a screenshot per frame when it doesn't; the run log says why.
  - Under begin-frame control the page loads while display-less frames tick; each frame is a seek, then one `beginFrame` with its picture. A frame with no damage repeats the last picture.
- **A render or still that fails while drawing with a graphics card setting checks whether the browser can draw WebGL with it (`draws_webgl`, `crates/capture/src/film.rs`)**, and when it can't, says so and names the Draw with choice to use instead (`FilmError::NoWebGl`, `crates/render/src/frames.rs`); the original error goes to the log.
- **A load or a frame that isn't ready within `FRAME_TIMEOUT` fails, naming the holds still open** (`ready_within`); a page that doesn't start within `PAGE_TIMEOUT` fails too.
- **The page only draws when the driver asks, so while a seek waits on holds the driver issues frames it doesn't capture** (`seek`). A hold may therefore wait for an animation frame or a canvas `paint` event (`untilPainted`); the captured frame comes after them.
- **A film's video frames are JPEG at `FRAME_QUALITY`** into FFmpeg (`film_encode_args`); a still is PNG or JPEG at its own quality (`render_still`, `still_format`).
  - The lossless formats (MP4 lossless, `libx264rgb` at `-qp 0` in `bgr24`, with ALAC sound; MKV lossless, FFV1 in `bgr0`, with FLAC sound) take PNG frames instead (`is_lossless`, `FilmEncoding.png_frames`), because a JPEG frame would already have lost detail; `crates/media/tests/lossless.rs` checks every pixel comes back.
- **A film is drawn in one tab, in frame order**, which the footage decoders rely on (each frame decoded once). Measure before adding tabs: shader backgrounds lose their WebGL context when several tabs draw at once.
- **Development film pages keep their source maps** so render errors point at source lines; release staging builds without them.

### Paint traps

- **Canvas effects draw into a canvas that is on the page.** Frames from a canvas given to `transferControlToOffscreen` never reach a begin-frame capture, and reading a WebGL `OffscreenCanvas` back as a bitmap on the page hangs it.
- **An SVG element whose CSS `scale` is exactly 0 stops being repainted when its scale changes**, so a scale that starts at 0 is kept above it (`PAINTABLE_SCALE`).

### Canvas transitions

- **The canvas transitions and filters draw their shaders only where Chromium's Canvas Draw Element is on, and fall back to CSS elsewhere** (`canDrawElementsInCanvas`, `packages/runtime/src/environment.ts`); the film's launch switches turn it on.
- **Feature switches are merged, never replaced (`effective_switches`)**, because Chromium keeps only the last `--enable-features`.
- **On that path each side of a transition hands its captured frame to `onElementImage`; `TransitionWrap` pairs the outgoing and incoming sides by the frame the cut starts on** (`CutBoardProvider`, one board per set of tracks) and draws both through the incoming side.

### Transitions

- **A transition overlaps the clips by its length, capped by what each neighbouring clip can give (`resolve_enters`, `crates/compile/src/clip.rs`), unless Keep every clip whole is on**; the cap is what the squeezed warning reports. With it on (`TransitionRef.hold`), `sequence_clips` holds the outgoing clip's last frame and the incoming clip's first frame for the transition (group `remap`) and delays the incoming sound, so no clip loses screen time.
- **A transition rides on the clip it brings in (`ClipValue.enter`), set by the Transition node (`timeline.transition`)**, so it stays next to its clip and Sequence only joins.
  - The Transition node is a kind over every transition element, the light leak cut and spring settle, with its length, easing and Keep every clip whole (`transition_of`); it outputs the clip with `enter` set and the transition alone on its `transition` port.
  - Every node that makes a clip from a clip keeps `enter`: `wrap_clip` copies it, and a node that builds a new `ClipValue` copies it by hand. Trim drops it when it cuts the start, Layers clears it (its card warns on a marked layer) and Choose by variable passes the picked clip's. `crates/compile/tests/unit/catalogue.rs` → `every_clip_node_keeps_a_marked_clips_transition` feeds a marked clip to every clip-to-clip node.
- **Each join in a Sequence uses the incoming clip's `enter`, then the Sequence's `transition` input, then a plain cut** (`joins_for`); the first clip's `enter` is not a join, so the Sequence's output carries it up to the next Sequence (`join_clips`).
  - Repeat takes the same `transition` input for the seams between repeats.
  - A light leak cut is a plain cut with a leak drawn over it on a track of its own (`leak_track`); spring settle plays the joined clips as scenes that shrink away and land.
  - A marked clip with nothing before it (the first clip of the top Sequence, or a clip that reaches no Sequence) puts "Nothing comes before this clip" on the Transition node (`transition_issues`).
- **Timeline Split inside a Sequence leaves the new half with no `enter`**, so the halves join with a plain cut ([timeline.md](timeline.md)).
- **Cards warn where a clip loses more than its transitions were meant to take:**
  - a transition that covers too much of a clip or is squeezed well below its designed length (`cover`, `cover_row`, `squeezed_too_much`, both at `TRANSITION_WARN_RATIO`);
  - Fit length cutting or speeding a lot;
  - an element much shorter than its usual length (`SHORT_ELEMENT_RATIO`).
- **The Timeline draws every transition the same**: how much of a clip a transition covers is recorded on its flattened mark (`TransitionMark.covers`) but not shown.

### Other shapes

- **Exports size the spec with `variant_size`**, and the project Inspector's canvas picker builds its shapes and sizes from `variant_canvas` at `CANVAS_SHORT_SIDE` (`crates/ui/src/project.rs` → `shapes`), the same source as the catalogue's `shapes`.
- **A recording in a shape narrower than itself keeps its own shape inside the frame (`containBox`) and the camera zooms harder** (`zoomBoost`, `narrowPose`): it never rests fully zoomed out and aims at the cursor between actions.
- **In a narrower shape the camera plans for that frame** (`tuning.narrow`): `frameFor` fits each target to the frame's width, centred, up to the boosted maximum, and moves only as far as it must to keep the click in shot.
- **The full-page styles (Proof, Ledger) show the whole page in any other shape (`NarrowBleedClip`) over a blurred copy of the footage**, with that camera only when narrower. Window styles in wider shapes and the `center`/`fit` choices keep the crop or bars.
- **Overlays size by the long side (`overlayScale`) and keep clear of `shapeInsets`; outside landscape, lower thirds move to the top** so captions own the bottom.
- **A Layers node with Keep this layout in other shapes wraps its layers in a group with `design` (the project's size, `keep_layout`)**, drawn scaled to fit in a `DesignBox` ([elements.md](elements.md)), for marks placed over a picture.
- **`containBox` also sizes screenshot windows**; the capture's copy of the sizing is in [capture.md](capture.md).
- **A take inside a Device frame, or framed with Frame's device kind, draws as the bare screen (`ScreenClip`, `shot.inDevice`)**, with no window frame or reframing; Frame's device kind wraps the clip in the device after the take (`in_device`, `ShotProps.device`).

### Window frames

- **Without a Frame of kind window a take has no frame.** Frame (`camera.frame`, on the take's style) sets the take's `window` (`framed` in `crates/compile/src/nodes/camera.rs`: `window.chrome` on unless its style is None, with its colour, font and title bar), its shot and its background, and `WindowFrame` draws it.
- **Frame for images draws through the same `WindowFrame` and device elements** (the `framed-image` element, `packages/video/src/core/framed-image.tsx`, rendered as a still, [images.md](images.md)), so an image framed with a window or a device matches the same frame in the film.
- **With Title bar off (`window.bar`) the bar shows no text, and any Title bar text is dropped (`Bar::typed`).**
- **With it on, a browser take's title bar shows its first page address unless the frame has Title bar text** (`window.text`, `window.mode`), and a terminal take's shows the window title the take logged (`terminalTitle`): "This address" keeps the typed text; "This host with the page's path" puts the typed host before the path on screen at each frame (`frameAddress`, from the take's address log, [capture.md](capture.md)).
  - The address mode shows only for browser takes (`x-show-if-take`).

## Deliberate limits

- **Nothing checks WebGL before a render**, because films without WebGL render on any drawing choice. On a Linux laptop with both Intel's and NVIDIA's Vulkan drivers, "Graphics card, newer drivers" starts ANGLE on Vulkan, then fails to share the first WebGL image (`initExternalMemory`, error -2) and the page hangs.
