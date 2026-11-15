# Browser capture

Browser sessions and their actions, the take value every recording shares, recording styles and edits, the screencast, smooth takes, the streamed views and sign-in, traces and screenshots. Rust paths are relative to `crates/capture/src/` unless they name another crate; page paths to `packages/video/src/`. Terminal takes are in [terminal.md](terminal.md), the recording browser in [engine.md](engine.md); see also [film.md](film.md) and [AGENTS.md](../AGENTS.md).

## Code map

| Concern                    | Where                                                                                                                                                                                                |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Take value                 | `crates/types/src/values.rs` → `RecordingValue`, `TakeSource`                                                                                                                                        |
| What the film draws        | `crates/types/src/recording.rs` → `RecordingProps` (`camera`, `shot`, `window`, `pointer`, `screen`/`ScreenProps`)                                                                                   |
| Drawing a take             | `crates/compile/src/elements/camera/recording.rs` → `recording`, `screen_clip`, `narrow_bleed`                                                                                                       |
| Style and edit application | `crates/compile/src/style.rs` → `restyle_with`, `retime_with`                                                                                                                                        |
| Taking a take              | `take.rs` (the driver: `play_chain`, `begin_video`, `start_screencast`, `landed_on_sign_in`, `off_runtime`); `cdp.rs` → `Tab`                                                                        |
| Browser launch             | `browser.rs` → `Session::open`, `check_browser`                                                                                                                                                      |
| Cursor and input           | `cursor.rs` (`CursorDriver`, glides, clicks, drags, `quick_onto`, `quick_along`); `input.rs` (`move_watching_drags`); `keyboard.rs`                                                                  |
| Finding elements           | `selectors.rs`, `locators.rs`                                                                                                                                                                        |
| Faked replies and clock    | `route.rs` → `MockResponse`; `clock.rs` → `Clock::freeze`                                                                                                                                            |
| Take clock                 | `take_clock.rs` → `TakeClock` (`tab.clock()`), `frame_time`, `frames_in`; `clock.rs` → `as_ms`                                                                                                       |
| Smooth takes               | `smooth.rs` → `Smooth`, `page_ms`; `page_scripts.rs` → `STEP_FRAME`, `STEP_OVERLAYS`, `PLAN_ELEMENT_SCROLL`, `STEP_ELEMENT_SCROLL`, `page_time_for_new_pages`; `clock.rs` → `Clock::hold_for_frames` |
| Encoder                    | `encoder.rs` → `FrameEncoder`, `offer`, `FrameImage`                                                                                                                                                 |
| Drag curves                | `drag_path.rs`                                                                                                                                                                                       |
| Address log                | `page.rs` → `Lifecycle` (`ActionLog.addresses`)                                                                                                                                                      |
| Camera events from the log | `camera/action-log.ts` → `toClickLog`; `camera/lib/cursor.ts` → `cursorAt`                                                                                                                           |
| Streamed views             | `crates/stream/src` → `view.rs` (`StreamView`), `picker.rs`, `sign_in.rs`, `host.rs` (`ViewEnv`); `crates/session/src/views.rs` → `view_input`; `crates/qt/cpp/stream_window.cc` → `StreamWindow`    |
| Sign-in stamp              | `executors.rs` → `sign_in_stamp`                                                                                                                                                                     |
| Traces                     | `trace.rs` records; `crates/traces/src/lib.rs` → `read_trace`, `apply_retention`; `crates/qt/cpp/replay_dialog.cc` → `TracePanel`                                                                    |
| Screenshot scale           | `still.rs` (the capture); `still_layout.rs` → `capture_scale`, `tallest_shot`, `LOOKS`                                                                                                               |
| Playwright pieces          | `playwright/*.generated.*` (generated, [AGENTS.md](../AGENTS.md))                                                                                                                                    |
| Tests                      | `crates/capture/tests`; `OPSIS_ENGINE_DIR` names the engine folder they use (default `~/.config/Opsis/engine`, `tests/support/mod.rs`), `OPSIS_TAKE_DIR` keeps `todomvc.rs`'s projects there         |

## Rules

### Sessions and actions

- **Browser nodes are virtual.** Open browser starts a `browser-session` value (`{ browser, steps }`) and every other browser node appends its own step (id, type, label, parsed and interpolated params) in `compile()`.
- **Record take and Screenshot read the session at their `session` input and play `steps` as given, in one take.**
- **The video starts at the first Start recording step**; the steps before it run off camera, and without one it starts with the first step that isn't replayed (`video_start`).
- **Record take and Screenshot pass on the session they played** (their `session` output, `BrowserSessionValue::played`), so one chain can make several takes or stills: the same steps, Start recording dropped, each marked `replayed`.
  - The next take plays the replayed steps again off camera in its own fresh browser, so the page reaches the state the earlier take left; that costs their running time again.
  - A replayed step acts on the page but leaves nothing in the take: no action in its log (`push_action`), so no click sound, keystroke chip or camera move from it, and a replayed Chapter shows no card.
- **The session carries the address the last Go to page opened (`address`)**, which the element picker opens and the Run page script warning reads.
- **Capture keeps Playwright's launch switches, injected page script, clock and keyboard but talks CDP itself**; `playwright-core` is only a dev dependency.
- **Actions that change what the page is told about the computer change it mid-take, from Open browser's values.** Switch theme (`browser.switchTheme`) sends the media features again through `Session::set_color_scheme`, because `Emulation.setEmulatedMedia` replaces every feature at once, so the session keeps Open browser's reduced motion (`Session.reduce_motion`) to send it with the new scheme. Pages see the change live, as when the computer's setting changes.
  - Change language (`browser.changeLanguage`, `Session::set_locale`) clears the session's own language override before setting the new one, then sends the user agent again with the new `acceptLanguage`; it reloads the page by default, because most sites read the language only when a page loads. An empty language is the computer's, as in Open browser.
  - Go offline (`browser.goOffline`, `Session::set_offline`) uses `Network.emulateNetworkConditionsByRule`, one rule with an empty pattern for every request, and `Network.overrideNetworkState` for what `navigator.onLine` and the `online`/`offline` events report; the older `Network.emulateNetworkConditions` that did both is deprecated. Both replacements are still experimental in the protocol, so a browser move rechecks them (`page_conditions.rs`).
- **Sign out (`browser.signOut`) undoes Sign in for the rest of the take**: it forgets the sign-in (`signed_in`), removes the scripts Sign in added to restore local storage (`sign_in_scripts`, `Session::remove_script`), clears every cookie and the page's site data (`Session::clear_site_data`) and its session storage, then reloads by default.
  - The restore scripts must go: they restore once per tab by a mark in session storage, so clearing session storage alone would sign the page straight back in on the reload.

### A take is one value

- **Every take is a `recording` (`RecordingValue`)**: what every take has (log, length, node, `edits`) and a `source` (`TakeSource`: `Browser` with its video and viewport, `Terminal` with its cast file, grid size, theme and preview). The log is in the take's own units (CSS pixels for a browser take).
- **`RecordingValue::file` names the take's own file and `video` only a browser take's.**
- **`DemoClip` draws `Screen` everywhere a picture of a take goes**, so every camera style, frame and mark works around any screen.
- **Save take (`output.take`) copies a take's own file to the exports and passes the saved file on**: a video, or a `.cast` of the `terminal-take` kind, which every file need refuses with its own message (`FileNeed::refuses`, `TERMINAL_TAKE_FILE`), since it has no picture or sound of its own.

### Recording style and edits

- **Clip from take (`camera.take`, `recording + recording-style? → clip`) starts from every style node's defaults, applies the style's entries, then the take's own edits: the most specific wins.**
- **Style nodes are `recording-style? → recording-style` and append `{ nodeId, type, params }` (`style_modifier`).**
- **Nodes that need one take's frames and times (Camera move, Speed ramp, Redact take) are `recording → recording` and append to the recording's `edits` (`recording_modifier`).**
- **Each definition gives a `restyle` or `retime`; `restyle_with` applies Camera entries before any other**, so the nodes that adjust the camera (Camera move) always land on it.
- **Camera follows the actions or the cursor (`follows`)**; following the cursor plans the camera along the pointer's path with its own settings, shown only for the cursor.
- **Frame (`camera.frame`) is the one style node for how a take sits in the film**: its kind (window, whole page or device), the window's style and title bar, the shot's shape and the background behind it ([film.md](film.md), Window frames).
- **Apply style (`clip + recording-style → clip`) styles every recording in a clip**, for takes joined in a Sequence or Layers.
- **A style node that only fits one kind of take says so with `only_for`**, and Clip from take's card names the ones that do nothing on its take.

### Actions

- **Click presses the left or right button (`ClickButton`) and, for the left, can double-click**: two presses at one point, the second with click count 2, so the page fires `dblclick`. It logs `button` (right only) and `tSecondDown`; the cursor squashes and rings at each press, a right click's ring hollow (`cursorAt`).
- **Drag (`browser.drag`) drags onto an element (measured again after the press, failing when it isn't on screen), by an offset or along a path.**
  - The quick method moves in `QUICK_DRAG_STEPS` steps with no hold or nudge (`quick_onto`, `quick_along`).
  - The smooth method presses, waits `holdMs`, nudges by `DRAG_NUDGE_PX` (pages start drags past a threshold) and walks the points with the same sampler as glides (`SAMPLE_MS`, `cursor.rs`); glides keep their exact samples.
  - The intercepted drag's data is sent back with `items` always present, empty when the page put nothing in `dataTransfer` (`move_watching_drags`): chromiumoxide leaves an empty list out, and Chrome refuses drag data without it (-32602).
  - It logs `tDown`, `t` (the release), `path` (at most `MAX_PATH_POINTS`) and `drop`. `toClickLog` makes one camera event covering both ends when they fit in half the window, else a pick-up and a drop; `cursorAt` keeps the arrow pressed until `tUpMs`. The baked cursor follows `dragover` too.
- **The picker's Shift-click returns a spot as `x, y`, in a window sized to the take's browser.**
- **The take logs every main-frame address change, loads and in-page route changes alike, with its time** (`ActionLog.addresses`).

### Capture timing

- **Capture timing comes from frames, not markers.** Screencast JPEGs go into `FrameEncoder` through `offer`, which never waits: Chrome sends the next frame only after the last is acknowledged, so waiting on the encoder loses any page change painted during a stall. Encoder errors surface at `finish`.
- **The screencast starts with the take (`start_screencast`) and runs to its end on the real clock; a smooth take stops it at Start recording and draws its own frames from there (`step_frames`).**
- **The video starts at `begin_video`**, after the steps before Start recording: the encoder writes a constant-rate stream from there with a keyframe every second (`encoder_args`, so a render can start reading anywhere cheaply, [footage.md](footage.md)), action timestamps share that origin, and the take then waits Record take's pause before the first action (`establish_ms`) on the take clock.
- **Live frames for the editor are the same frames**, throttled to the live fps setting, sent as raw bytes to each window that watches ([document.md](document.md)).

### Smooth takes

- **Every wait and time source in a take goes through the tab's take clock (`tab.clock()`: `now`, `wait`, `sleep_until`, `until`), never `tokio::time::sleep` or `Instant`.** The cursor, input, selectors, overlays, the trace and the take all use it.
  - Inside a take, `tokio` deadlines are left only where Rust waits for the browser to answer rather than for page time: the reply to `Page.navigate` (`page.rs`), an intercepted drag (`input.rs`), the list of visible elements for an error (`selectors.rs`), and, on the real clock only, the page lifecycle's waits (`Lifecycle::until`), which wake on the page's own change notices.
- **The clock keeps whole time**: `now` is a `Duration` since the Unix epoch, a stepped clock counts whole frames at its integer rate, timeouts are deadlines compared with `now`, and a time becomes `f64` milliseconds (`as_ms`) only where it leaves Rust (page clock, encoder, trace, action log).
- **Record take's Timing (`real`, `smooth`) picks the clock.** On the real clock waits sleep and poll; at Start recording a smooth take switches to steps (`Smooth`).
  - The screencast stops, the page clock is held paused in the top frame (`Clock::hold_for_frames`) and every CSS and web animation gets rate 0 (`Animation.setPlaybackRate`).
  - Each step moves the page clock, every running animation of the page and its same-origin frames, and the take's own overlays (`STEP_OVERLAYS`, in the injected script's closed shadow root) on by one frame (`STEP_FRAME`, whole milliseconds that add up to the frame times, `page_ms`).
  - It keeps the page time for the next document (`page_time_for_new_pages`, one init script replaced each step), takes the frame's picture and offers it at the step's time, so every frame is its own picture and action times, trace calls and addresses fall on frames.
- **A wait is whole frames and, unless it is zero, at least one; `sleep_until` takes none for a time already passed, and overlapping waits share their frames.**
- **Every frame picture is the window's region of the page (`clip`, placed by the scroll position the step reports) at an explicit scale**, because a plain screenshot comes back at the page's pixel ratio. Standard is a JPEG at the window's size, Lossless a PNG at that size (`FrameImage::Png`), Full detail a JPEG at the page's pixel ratio.
- **A smooth Scroll to element plans where each scrolling box ends (`PLAN_ELEMENT_SCROLL`, an instant `scrollIntoView` undone at once) and moves them a frame at a time with the camera easing (`STEP_ELEMENT_SCROLL`)**, because the browser runs a smooth scroll on its own frame clock, which neither the page clock nor the animation rate reaches. Wheel scrolls are instant in the headless shell, so a smooth Scroll by needs no plan: it is wheel ticks spread over its length on the take clock (`SCROLL_TICK_MS`, `take.rs`).

### Streamed views and sign-in

- **The element picker and Sign in are pages in the headless shell streamed to a window of their own** (`StreamView`, shown in a `StreamWindow`): JPEG frames over a `Channel`, mouse, wheel, keys and text back as `view_input`, mapped through the letterboxing; the page keeps its own size, and dialogs, file choosers and popups are answered from the window.
- **Sign in (`browser.signIn`) is an ordinary executing node in the action chain.** Its executor opens the view, at the session browser's size, only when the sign-in named in Save as isn't saved yet or the node runs on its own (`forced`).
  - Its step in the take adds the saved cookies and, per origin, the page storage (an init script, plus the current page when it is that origin).
- **Its executor is stamped (`sign_in_stamp`)**: the saved sign-in's file stamp goes into its cache key, so a new sign-in runs every take after it again. The runner works keys out again as soon as a stamped node has run (`rekey`), so a take recorded right after a new sign-in is saved under its new key.
- **A Go to page that lands on the Sign in step's address, without asking for it, fails the take with "Signed out during the take. Run “<name>” again."** (`landed_on_sign_in`); nothing retries across nodes. A sign-in page reached by a click or an in-page route isn't caught.
- **The sign-in finishes on the one condition its Done when setting picks (`DoneWhen`)**: the address moving off the first page it landed on (in single-page apps too), an element appearing, or the window closing.
  - Whatever it picks, closing the window or reaching the time limit also finishes it; each of these saves the sign-in, and only cancelling the run saves nothing (`crates/stream/src/sign_in.rs` → `watch`).
  - Sign-in state is storage-state JSON, sealed ([app-state.md](app-state.md)).

### Traces

- **Traces are recorded per capture unless retention is `off`, in our own format** (`crates/traces`): per take a zstd JSON manifest (header, viewport, actions with node ids, console, page errors, requests) and its frames as content-addressed JPEG blobs in `.opsis/traces`, shared across runs and pruned by the retention setting.
- **Blob files keep their extension (`<sha>.jpeg`)**, so a frame's file name says its format.
- **A trace's frames come from the encoded video when the take records one, else from the screencast.**
  - With a recording, `finalize_trace` takes a frame every `FRAME_GAP_MS` from the finished video (`recording_frames`, `executors.rs`).
  - Without one, the screencast hands its frames to the trace (`FrameSinks`, `Trace::frame`).
- **The Trace tab in Replay is our own view over `read_trace` (`crates/qt/cpp/replay_dialog.cc` → `TracePanel`); never embed Playwright's Trace Viewer.**

### Screenshots

- **Screenshot (`capture.still`) outputs only its image**; Frame for images and Use as clip frame and place it ([images.md](images.md)), so a screenshot is framed the same way as any other image.
- **Screenshot captures a PNG through `Page.captureScreenshot` with a clip `scale` from `capture_scale`** (divided by the page's pixel ratio for the clip): enough pixels to draw the shot at the largest export (`MAX_EXPORT_SCALE` times the project size), never below the page's pixel ratio, capped by a surface budget for very large regions.
- **The capture can't know how the image will be framed, so it takes enough pixels for the largest of the three ways a shot fills a frame** (`LOOKS`, `still.rs`): a window in its own shape, an element cut out and fitted to a share of the frame, and a page covering the frame. A page taller than the frame's shape by more than `TALL_SHOT` is cut at the bottom (`tallest_shot`).
- **Its image carries the region's size, that scale and, for one element, the element's box inside the padded region** (`padded_region`).

## Deliberate limits

- **On a held clock nothing the page itself waits for happens between steps**: a page script that awaits a timer never ends, and server replies, videos, canvases driven by real time, workers and other-origin frames keep real time.
- **Frame pictures cost more the more they keep**: Lossless and Full detail take longer than Standard, by how much the Frame pictures setting's help says (`Record`, `crates/compile/src/nodes/capture.rs`).
