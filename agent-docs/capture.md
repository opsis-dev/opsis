# Browser capture

Browser sessions and their actions, the take value every recording shares, recording styles and edits, the screencast, smooth takes, the streamed views and sign-in, traces and screenshots. Rust paths are relative to `crates/capture/src/` unless they name another crate; page paths to `packages/video/src/`. Terminal takes are in [terminal.md](terminal.md), the recording browser in [engine.md](engine.md); see also [film.md](film.md) and [AGENTS.md](../AGENTS.md).

## Code map

| Concern                     | Where                                                                                                                                                                                                |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Take value                  | `crates/types/src/values.rs` → `RecordingValue`, `TakeSource`                                                                                                                                        |
| What the page draws         | `crates/types/src/recording.rs` → `RecordingProps` (`camera`, `shot`, `window`, `pointer`, `screen`/`ScreenProps`)                                                                                   |
| Drawing a take              | `camera/demo-clip.tsx` → `DemoClip`; `core/screen.tsx` → `Screen`                                                                                                                                    |
| Style and edit application  | `crates/compile/src/style.rs` → `restyle_with`, `retime_with`                                                                                                                                        |
| Take clock                  | `take_clock.rs` → `TakeClock` (`tab.clock()`), `frame_time`, `frames_in`, `as_ms`                                                                                                                    |
| Smooth takes                | `smooth.rs` → `Smooth`, `page_ms`; `page_scripts.rs` → `STEP_FRAME`, `STEP_OVERLAYS`, `PLAN_ELEMENT_SCROLL`, `STEP_ELEMENT_SCROLL`, `page_time_for_new_pages`; `clock.rs` → `Clock::hold_for_frames` |
| Encoder                     | `encoder.rs` → `FrameEncoder`, `offer`, `FrameImage`                                                                                                                                                 |
| Drag curves                 | `drag_path.rs`                                                                                                                                                                                       |
| Address log                 | `page.rs` → `Lifecycle` (`ActionLog.addresses`)                                                                                                                                                      |
| Camera events from the log  | `camera/action-log.ts` → `toClickLog`; `camera/lib/cursor.ts` → `cursorAt`                                                                                                                           |
| Streamed views              | `crates/stream` (`pick`, `sign_in`); `apps/frontend/src/app/stream-view.tsx` → `StreamView`                                                                                                          |
| Sign-in stamp               | `executors.rs` → `sign_in_stamp`                                                                                                                                                                     |
| Traces                      | `crates/traces`; `TracePanel`, `read_trace`                                                                                                                                                          |
| Screenshot scale and layout | `still_layout.rs` → `capture_scale`; page `StillElement`, `windowLayout`, `cutoutLayout`, `bleedLayout`, `stillLayout`                                                                               |
| Playwright pieces           | `playwright/*.generated.*`                                                                                                                                                                           |

## Rules

### Sessions and actions

- **Browser nodes are virtual.** Open browser starts a `browser-session` value (`{ browser, steps }`) and every other browser node appends its own step (id, type, label, parsed and interpolated params) in `compile()`.
- **Record take and Screenshot read the session at their `session` input and play `steps` as given, in one take.**
- **The video starts at the first Start recording step**; the steps before it run off camera, and without one it starts with the first step.
- **The session carries the address the last Go to page opened (`address`)**, which the element picker opens and the Run page script warning reads.
- **Capture keeps Playwright's launch switches, injected page script, clock and keyboard but talks CDP itself**; `playwright-core` is only a dev dependency.

### A take is one value

- **Every take is a `recording` (`RecordingValue`)**: what every take has (log, length, node, `edits`) and a `source` (`TakeSource`: `Browser` with its video and viewport, `Terminal` with its cast file, size and theme). The log is in the take's own units (CSS pixels for a browser take).
- **`RecordingValue::file` names the take's own file and `video` only a browser take's.**
- **`DemoClip` draws `Screen` everywhere a picture of a take goes**, so every camera style, frame and mark works around any screen.
- **Save take (`output.take`) copies a take's own file to the exports and passes the saved file on**: a video, or a `.cast` of the `terminal-take` kind, which every file need refuses with its own message (`FileNeed::refuses`, `TERMINAL_TAKE_FILE`), since it has no picture or sound of its own.

### Recording style and edits

- **Clip from take (`camera.take`, `recording + recording-style? → clip`) starts from every style node's defaults, applies the style's entries, then the take's own edits: the most specific wins.**
- **Style nodes are `recording-style? → recording-style` and append `{ nodeId, type, params }` (`style_modifier`).**
- **Zoom here, Pan and Speed ramp need one take's frames and times, so they are `recording → recording` and append to the recording's `edits` (`recording_modifier`).**
- **Each definition gives a `restyle` or `retime`; `restyle_with` applies Camera entries before any other**, so the nodes that adjust the camera (Follow cursor, Pan) always land on it.
- **Apply style (`clip + recording-style → clip`) styles every recording in a clip**, for takes joined in a Sequence or Layers.
- **A style node that only fits one kind of take says so with `only_for`**, and Clip from take's card names the ones that do nothing on its take.

### Actions

- **Click presses the left or right button (`ClickButton`) and, for the left, can double-click**: two presses at one point, the second with click count 2, so the page fires `dblclick`. It logs `button` (right only) and `tSecondDown`; the cursor squashes and rings at each press, a right click's ring hollow (`cursorAt`).
- **Drag (`browser.drag`) drags onto an element (measured again after the press, failing when it isn't on screen), by an offset or along a path.**
  - The smooth method presses, waits `holdMs`, nudges 6 px (pages start drags past a threshold) and walks the points with the same 8 ms sampler as glides; glides keep their exact samples.
  - The intercepted drag's data is sent back with `items` always present, empty when the page put nothing in `dataTransfer` (`move_watching_drags`): chromiumoxide leaves an empty list out, and Chrome refuses drag data without it (-32602).
  - It logs `tDown`, `t` (the release), `path` (at most 64 points) and `drop`. `toClickLog` makes one camera event covering both ends when they fit in half the window, else a pick-up and a drop; `cursorAt` keeps the arrow pressed until `tUpMs`. The baked cursor follows `dragover` too.
- **The picker's Shift-click returns a spot as `x, y`, in a window sized to the take's browser.**
- **The take logs every main-frame address change, loads and in-page route changes alike, with its time** (`ActionLog.addresses`).

### Capture timing

- **Capture timing comes from frames, not markers.** Screencast JPEGs go into `FrameEncoder` through `offer`, which never waits: Chrome sends the next frame only after the last is acknowledged, so waiting on the encoder loses any page change painted during a stall. Encoder errors surface at `finish`.
- **The encoder writes a constant-rate stream from the first frame after the setup steps; action timestamps share that origin.**
- **Live frames for the editor are the same frames**, throttled to the live fps setting, sent as raw bytes to each window that watches ([document.md](document.md)).

### Smooth takes

- **Every wait and time source in a take goes through the tab's take clock (`tab.clock()`: `now`, `wait`, `sleep_until`, `until`), never `tokio::time::sleep` or `Instant`.** The cursor, input, selectors, page lifecycle, overlays, the trace and the take all use it.
- **The clock keeps whole time**: `now` is a `Duration` since the Unix epoch, a stepped clock counts whole frames at its integer rate, timeouts are deadlines compared with `now`, and a time becomes `f64` milliseconds (`as_ms`) only where it leaves Rust (page clock, encoder, trace, action log).
- **Record take's Timing (`real`, `smooth`) picks the clock.** On the real clock waits sleep and poll; at Start recording a smooth take switches to steps (`Smooth`).
  - The screencast stops, the page clock is held paused in the top frame (`Clock::hold_for_frames`) and every CSS and web animation gets rate 0 (`Animation.setPlaybackRate`).
  - Each step moves the page clock, every running animation of the page and its same-origin frames, and the take's own overlays (`STEP_OVERLAYS`, in the injected script's closed shadow root) on by one frame (`STEP_FRAME`, whole milliseconds that add up to the frame times, `page_ms`).
  - It keeps the page time for the next document (`page_time_for_new_pages`, one init script replaced each step), takes the frame's picture and offers it at the step's time, so every frame is its own picture and action times, trace calls and addresses fall on frames.
- **A wait is whole frames and at least one; `sleep_until` takes none for a time already passed, and overlapping waits share their frames.**
- **Every frame picture is the window's region of the page (`clip`, placed by the scroll position the step reports) at an explicit scale**, because a plain screenshot comes back at the page's pixel ratio. Standard is a JPEG at the window's size, Lossless a PNG at that size (`FrameImage::Png`), Full detail a JPEG at the page's pixel ratio.
- **A smooth Scroll to element plans where each scrolling box ends (`PLAN_ELEMENT_SCROLL`, an instant `scrollIntoView` undone at once) and moves them a frame at a time with the camera easing (`STEP_ELEMENT_SCROLL`)**, because the browser runs a smooth scroll on its own frame clock, which neither the page clock nor the animation rate reaches. Wheel scrolls are instant in the headless shell, so Scroll by already steps per frame.

### Streamed views and sign-in

- **The element picker and Sign in are pages in the headless shell streamed to a window of their own** (`#/view/<id>`, `StreamView`): JPEG frames over a `Channel`, mouse, wheel, keys and text back as `view_input`, mapped through the letterboxing; the page keeps its own size, and dialogs, file choosers and popups are answered from the window.
- **Sign in (`browser.signIn`) is an ordinary executing node in the action chain.** Its executor opens the view, at the session browser's size, only when the sign-in named in Save as isn't saved yet or the node runs on its own (`forced`).
  - Its step in the take adds the saved cookies and, per origin, the page storage (an init script, plus the current page when it is that origin).
- **Its executor is stamped (`sign_in_stamp`)**: the saved sign-in's file stamp goes into its cache key, so a new sign-in runs every take after it again. The runner works keys out again as soon as a stamped node has run (`rekey`), so a take recorded right after a new sign-in is saved under its new key.
- **A take that lands on the Sign in step's address fails with "Signed out during the take. Run “<name>” again."**; nothing retries across nodes.
- **The sign-in finishes on an address change or a selector**, in single-page apps too. Sign-in state is storage-state JSON, sealed ([app-state.md](app-state.md)).

### Traces

- **Traces are recorded per capture unless retention is `off`, in our own format** (`crates/traces`): per take a zstd JSON manifest (header, viewport, actions with node ids, console, page errors, requests) and its frames as content-addressed JPEG blobs in `.opsis/traces`, shared across runs and pruned by the retention setting.
- **Blob files keep their extension (`<sha>.jpeg`)** so screenshots load straight through the file server.
- **The Trace tab in Replay is our own shadcn UI over `read_trace`; never embed Playwright's Trace Viewer.**

### Screenshots

- **Screenshot captures through `Page.captureScreenshot` with a clip `scale` from `capture_scale`**: enough pixels to draw the shot at the largest export (twice the project size), never below the page's pixel ratio, capped by a surface budget for very large regions.
- **Its image carries the region's size, that scale and, for one element, the element's box inside the padded region**; `StillElement` lays it out through `stillLayout`:
  - `windowLayout`: the shot's own shape; a page more than 1.5× taller than the frame's shape takes the frame's shape and is cut only at the bottom;
  - `cutoutLayout`: the element fitted to the share of the frame in Fit, at most 1.5× the captured pixels;
  - `bleedLayout`: covers the frame, keeping the top.
- **The capture sizes its shot with the same Fit.**

## Deliberate limits

- **On a held clock nothing the page itself waits for happens between steps**: a page script that awaits a timer never ends, and server replies, videos, canvases driven by real time, workers and other-origin frames keep real time.
- **Frame pictures cost more the more they keep**: a 30 s take of a busy 1920×1080 page at 2× takes about 36 s with Standard, 62 s with Lossless and 149 s with Full detail.
