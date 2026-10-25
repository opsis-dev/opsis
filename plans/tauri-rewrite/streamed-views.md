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

## Decisions for the user

- **Preview resolution.** Stream at the view's size (sharper, more CPU) or at a fixed 960 px in Fast mode; decide after the measurements in step 11.

## Risks

- **Sites that refuse automated browsers.** Google sign-in and some others refuse browsers they detect as automated even with a normal user agent; the sign-in view must say so plainly, and saved sign-ins from a file stay possible.
- **Passkeys and password managers** don't work in a streamed browser.
- **Latency.** Typing and dragging go through two hops; step 11 decides whether it feels acceptable.

## Sources

- `src/renderer/editor/preview-*.tsx`, `src/main/browser-windows.ts`, `src/main/picker-script.ts`, `src/main/login-state.ts`, `src/renderer/editor/live-view.tsx` (read 2026-10-25).
- [tauri `ipc::Channel`](https://docs.rs/tauri/latest/tauri/ipc/struct.Channel.html) (`InvokeResponseBody::Raw` as `ArrayBuffer`; `Channel<Vec<u8>>` as a JSON array) (2026-10-25).
- agent-browser `cli/src/native/stream/websocket.rs` (2026-10-25).
