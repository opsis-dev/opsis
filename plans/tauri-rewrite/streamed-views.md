# Streamed views

The preview, sign-in and the element picker run in the headless shell and are shown in the editor as streamed pictures, with the user's mouse and keyboard sent back. Nothing about the video or the user's site runs in the system webview.

Crate: `opsis-stream` (phase 5); the editor's view component lands with the switch in phase 6.

## What it adds

- **Preview equals render, more strictly than today.** Today the preview runs in Electron's Chromium and exports in the headless shell, two different Chromium versions. Streamed, both are the same shell, with WebGL, canvas transitions and video decoding all Chromium's, on every OS.
- **Sign-in in the browser that records.** The cookies and storage come from the same browser, so there is no conversion from an Electron session.
- **WebKit limits don't apply** to the user's site or the video: no WebKitGTK WebGL fallbacks, no GStreamer for the preview's video.

## Fit with Opsis

Already in place:

- **Preview:** the Remotion Player in the editor (`preview-composition.tsx`, `preview-controls.tsx`, `preview-dock.tsx`, `preview-toolbar.tsx`, `use-player-bridge.ts`, 600 lines), driven through `usePlayback`; our own controls (play, seek, time in the ruler's mode, volume, full screen); Fast quality plays `recording.preview.mp4` (960 px) and skips camera motion blur; the shape switcher (`previewShape`); the latest master stem plays with a Web Audio insert for live mixer drags (`AudioInsert`); the preview tab stays mounted while hidden.
- **Sign-in and picker:** sandboxed Electron windows with their own session partitions (`src/main/browser-windows.ts`, 169 lines); `PICKER_SCRIPT` (`src/main/picker-script.ts`, 219 lines) computes `nameOf`, `cssOf`, `roleOf`; Shift-click returns a spot as `x, y`; sign-in finishes on a URL change or a selector, including single-page apps (`did-navigate-in-page`, selector polling); the storage state is converted (`login-state.ts`) and encrypted.
- **Live view:** screencast frames of a running take reach the editor PiP (`live-view.tsx`) over a MessagePort.
- agent-browser's stream server is a working reference for the same idea: screencast JPEG frames out over a WebSocket, `input_mouse`/`input_keyboard`/`input_touch` messages in, turned into `Input.dispatch*` (`cli/src/native/stream/websocket.rs`; `capture-cdp.md`).

Missing:

- A view component that shows frames and forwards input; a preview page for the shell; popup, file chooser and keyboard handling.

## Work

1. **Frame transport.** Rust receives `Page.screencastFrame` (base64 JPEG), decodes it once and sends the bytes to the window over a `tauri::ipc::Channel` as `InvokeResponseBody::Raw`, which reaches JS as an `ArrayBuffer` without JSON encoding (a `Channel<Vec<u8>>` would send a JSON array of numbers). The UI draws each frame with `createImageBitmap` on a canvas sized to the view.
2. **Input.** The view sends pointer, wheel and key events in page coordinates; Rust turns them into `Input.dispatchMouseEvent`, `Input.dispatchKeyEvent` and, for text and paste, `Input.insertText`; IME composition through `Input.imeSetComposition`. The view resizes the page with `Emulation.setDeviceMetricsOverride`.
3. **One `StreamView` component** used by the preview, sign-in, the picker and the live view, with the tab's address bar, back and reload for sign-in and the picker.
4. **Preview page.** A small page built with `packages/video` (the Remotion Player and `DemoTimeline`) served by Rust next to the bundle. Rust loads it in a shell tab and controls it through `Runtime.evaluate` (load spec, play, pause, seek, rate, shape, Fast or Exact); the page reports the current frame through a binding (`Runtime.addBinding`). While paused, Rust takes a `Page.captureScreenshot` of the frame for a sharp still instead of the last JPEG.
5. **Preview sound.** The page stays muted; the webview plays the master stem with an `<audio>` element (the master stem is 16-bit PCM WAV, `pcm_s16le` in `masterStem`, `src/worker/mix/render.ts`; bus stems are 32-bit float). The page's reported frame is the clock: the webview corrects the audio's `currentTime` when it drifts past one frame. Mixer drags apply at once through a Web Audio gain node on that element, as `AudioInsert` does today, until the new stem is ready.
6. **Preview controls.** `preview-controls.tsx` keeps its UI and sends commands to Rust instead of `player.play(e)`; the Timeline's playhead follows the reported frame.
7. **Sign-in.** A shell tab at the Sign in node's address; the user signs in through the streamed view; it finishes on a URL change (`Page.frameNavigated`, `Page.navigatedWithinDocument`) or a selector; Rust reads cookies (`Network.getAllCookies`) and each origin's local storage, builds today's storage-state JSON and stores it encrypted (`desktop-shell.md`, Work step 8).
8. **Picker.** A shell tab sized to the take's browser with `PICKER_SCRIPT` injected as it is; clicks pick, Shift-click returns a spot; the result comes back through a binding.
9. **Popups, file choosers, dialogs.** New targets (OAuth popups) open in the same view with a tab strip (`Target.targetCreated`); `Page.setInterceptFileChooserDialog` plus a native file dialog from Tauri for uploads; `Page.javascriptDialogOpening` shown as an in-app dialog.
10. **User agent.** The shell reports `HeadlessChrome`; set a normal Chrome user agent and client hints with `Emulation.setUserAgentOverride` for sign-in and the picker.
11. **Measure first (a spike before the rest).** On Linux, Windows and macOS: frames per second at 960 px and 1080p with JPEG quality 70 and 85, input-to-frame latency, seek-to-frame time on a recording and on a shader element, CPU use, and audio drift over a 2-minute film. Record the numbers here; choose the default size and quality from them.

## Phase 5 as built

- **`opsis-stream`.** `StreamView` is a headless-shell tab streamed to a `ViewSink` (frames and events) that a host attaches; a frame sent before that is kept for the host. It takes `ViewInput` (mouse, wheel, keys with their codes, text, composition) as `Input.dispatch*`, resizes with `Emulation.setDeviceMetricsOverride`, navigates, goes back and forward, reloads, shows popups as tabs (`ViewEvent::Tabs`), reports dialogs and file choosers and answers them, and gives the page a normal Chrome user agent with Chromium brands. `ViewHost` is what phase 6 implements with a window and a `Channel`.
- **Sign-in and picker.** `sign_in` follows `loginInWindow`: it finishes on an address change, a selector or the view closing, and saves the cookies (`Network.getAllCookies`) and each origin's local storage as today's storage state; `StreamLogin` is the `Login` the capture executor calls. `pick` injects the picker script (`crates/stream/src/picker.js`, `src/main/picker-script.ts` as plain JavaScript) and gets the result through the `opsisPick` binding instead of the console.
- **Preview page.** `apps/preview` is the Remotion Player with `DemoTimeline`, muted, controlled through `window.opsisPreview` (`load`, `play`, `pause`, `seek`, `rate`) and reporting through the `opsisFrame` and `opsisState` bindings. `Preview` serves its build and `public/` from one local server and the project from another, and takes a sharp still with `Page.captureScreenshot`. Sound (step 5) and the editor's controls (step 6) come with phase 6.
- **Fixed on the way.** `apps/ui/src/index.css` pointed Tailwind at `apps/video/src`, which doesn't exist, since phase 1.5; it now points at `packages/video/src`, so the classes video elements use are generated in the editor again.
- **Checks.** On the engine: a click and a Shift-click in the view pick an element and a spot; sign-in finishes on an address change and on a selector with the cookie and local storage; a popup opens as a second tab; the TodoMVC film loads, seeks, plays and gives a 960 × 540 still (`crates/stream/tests`, ignored by default).

## Measurements (step 11)

Linux only (Intel Core i7-12700H, 20 threads, software WebGL as in renders); Windows and macOS are still to measure. The TodoMVC film in Fast quality, `crates/stream/tests/measure.rs`, 2026-10-26:

| Stream                  | Frames a second while playing | Size a frame | Browser CPU     |
| ----------------------- | ----------------------------- | ------------ | --------------- |
| 960 × 540, quality 70   | 25.2                          | 15 KB        | 16% of one core |
| 960 × 540, quality 85   | 23.2                          | 20 KB        | 18%             |
| 1920 × 1080, quality 70 | 13.4                          | 38 KB        | 34%             |
| 1920 × 1080, quality 85 | 15.8                          | 49 KB        | 36%             |

- The film plays at its own 30 frames a second in every case; the stream delivers fewer, since the screencast sends a frame only after the last one is acknowledged.
- Input to the next frame: 14 ms (p90 20 ms) at 960 × 540, 35 ms (p90 43 ms) at 1920 × 1080.
- Seek to a recording frame: reported in 1–2 ms, on screen in 77 ms at either size. Seek to a shader frame: 191 ms to report and 213 ms on screen at 960 × 540, 437 and 484 ms at 1920 × 1080.
- Clock over 2 minutes (8 plays): the Player starts at once, falls 0.87 s behind wall time during the 2.5 s shader intro (it waits for slow frames instead of skipping them) and keeps time after it. The paper shader draws a 3840 × 2160 canvas whatever the view's size. The webview's sound (step 5) would have to jump to follow the reported frame through such parts.
- Not measured here: Windows and macOS, and sound drift with the webview's `<audio>`, which needs phase 6.

## Decisions for the user

- **Preview resolution (decided 2026-10-26).** A fixed 960 px in Fast mode: 25 frames a second against 13–16 at the view's 1080p, and shader seeks under half the time (step 11). Exact mode and paused stills stay sharp at the view's size.

## Risks

- **Sites that refuse automated browsers.** Google sign-in and some others refuse browsers they detect as automated even with a normal user agent; the sign-in view must say so plainly, and saved sign-ins from a file stay possible.
- **Passkeys and password managers** don't work in a streamed browser.
- **Latency.** Typing and dragging go through two hops; step 11 decides whether it feels acceptable.

## Sources

- `src/renderer/editor/preview-*.tsx`, `src/main/browser-windows.ts`, `src/main/picker-script.ts`, `src/main/login-state.ts`, `src/renderer/editor/live-view.tsx` (read 2026-10-25).
- [tauri `ipc::Channel`](https://docs.rs/tauri/latest/tauri/ipc/struct.Channel.html) (`InvokeResponseBody::Raw` as `ArrayBuffer`; `Channel<Vec<u8>>` as a JSON array) (2026-10-25).
- agent-browser `cli/src/native/stream/websocket.rs` (2026-10-25).
