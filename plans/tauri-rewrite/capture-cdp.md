# Capture over CDP

Record takes, screenshots and sign-in steps by driving the headless shell over the Chrome DevTools Protocol from Rust, with no Playwright.

## What it adds

- `playwright-core` (13.7 MB) and its Node dependency leave the app.
- Capture controls frame timing directly: inputs, screencast frames and the encoder are in one process.

## Fit with Opsis

Already in place:

- **The headless shell already speaks CDP to both of its users.** Playwright launches it with `chromium.launch({ executablePath })` (`src/worker/capture/session.ts:24`) and Remotion drives it for renders. Its version is Remotion's `TESTED_VERSION`, `157.0.8080.0` in `@remotion/renderer` 4.0.537. Remotion downloads it from `remotion.media` for Linux x64, Linux arm64 and Apple Silicon, and from Chrome for Testing (`storage.googleapis.com/chrome-for-testing-public/<version>/<platform>/chrome-headless-shell-<platform>.zip`) elsewhere (`dist/browser/get-chrome-download-url.js`). `src/engine/install.ts` calls `ensureBrowser` into `engine/.download/` and moves the shell into place.
- Capture code (2,039 lines): `driver.ts`, `take-run.ts`, `take-steps.ts`, `selectors.ts`, `session.ts`, `recorder.ts`, `encoder.ts` (`FrameEncoder`), `drag-path.ts`, `cursor-keyword.ts`, `capture-scale.ts`, `driver-helpers.ts`, `flow-types.ts`, and `page-scripts.ts` (203 lines of JS injected into pages).
- Screenshots already use CDP's `Page.captureScreenshot` with a clip scale (`captureScale`).
- What Playwright provides today, counted in `src/worker/capture` and `src/worker/executors/capture.ts`:

| Playwright call                                                                                              | Uses                   | CDP replacement                                                                                                                                                                     |
| ------------------------------------------------------------------------------------------------------------ | ---------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `waitForTimeout`                                                                                             | 12                     | `tokio::time::sleep`                                                                                                                                                                |
| `evaluate`                                                                                                   | 10                     | `Runtime.evaluate` / `Runtime.callFunctionOn`                                                                                                                                       |
| `addInitScript`                                                                                              | 6                      | `Page.addScriptToEvaluateOnNewDocument`                                                                                                                                             |
| `goto`, `waitForURL`, `waitForLoadState`                                                                     | 5, 1, 1                | `Page.navigate` plus `Page.lifecycleEvent` / `Page.frameNavigated`                                                                                                                  |
| `click`, `hover`, `mouse.move/down/up/wheel`, `keyboard.press/type`                                          | 5, 1, 4/3/3/1, 4/1     | `Input.dispatchMouseEvent`, `Input.dispatchKeyEvent`, `Input.insertText`                                                                                                            |
| `getByRole`, `getByLabel`, `getByText`, `getByPlaceholder`, `getByTestId`, `locator`, `isVisible`, `waitFor` | 1, 1, 2, 1, 1, 2, 1, 2 | `Accessibility.queryAXTree` for role and name; injected scripts for label, placeholder, text and test id; `DOM.getBoxModel` and an injected check for visibility; our own wait loop |
| `addCookies`                                                                                                 | 2                      | `Network.setCookies`                                                                                                                                                                |
| `route`                                                                                                      | 1                      | `Fetch.enable` + `Fetch.requestPaused`                                                                                                                                              |
| `newContext`, `newPage`                                                                                      | 1, 1                   | `Target.createBrowserContext`, `Target.createTarget`                                                                                                                                |
| `screencast.start`, `showActions`, `showChapter`, `showOverlay`                                              | 1 each                 | `Page.startScreencast`; the overlays are Playwright features, so draw them with our own injected script                                                                             |
| `tracing.start`, `group`, `groupEnd`                                                                         | 1 each                 | our own trace (Work step 7)                                                                                                                                                         |

### agent-browser (checked 2026-10-25, source cloned into `~/.cache/opsis-research/agent-browser`)

- Vercel's agent-browser is Apache-2.0. Its current release is 0.39.2, with 16–18 MB binaries per platform; the copy installed here through cargo is 0.19.0 and 7.6 MB.
- On crates.io it is binary-only; docs.rs says "agent-browser-0.19.0 is not a library". There is no Rust API to depend on.
- Its daemon takes one JSON command per line and answers one JSON line, over a Unix socket (`cli/src/connection.rs`) or a 127.0.0.1 TCP port on Windows (`cli/src/native/daemon.rs`). Commands are `{"action": …}` objects with 167 handlers.
- It has its own CDP client (`cli/src/native/cdp/client.rs`, on `tokio-tungstenite`, with the protocol JSON vendored in `cli/cdp-protocol/`).
- It has a stream server: a WebSocket that sends screencast JPEG frames and turns `input_mouse`, `input_keyboard` and `input_touch` messages into `Input.dispatch*` (`cli/src/native/stream/websocket.rs`).
- It doesn't fit capture: one JSON command per action is too coarse for 8 ms glide samples and screencast frame timing. Its CDP client, accessibility snapshot (`native/snapshot.rs`) and stream server are useful references, and Apache-2.0 allows copying them with the notice.

Missing:

- A Rust CDP layer and everything in the table.

## Work

1. **Engine installer in Rust.** Download the shell for the platform from the same URLs Remotion uses for `TESTED_VERSION`, check its hash, unpack into `engine/chrome-headless-shell/`. Keep `TESTED_VERSION` read from the installed `@remotion/renderer` at build time, so the shell always matches the sidecar's Remotion.
2. **CDP client.** `chromiumoxide` 0.9.1 (2026-02-25, MIT OR Apache-2.0): launches a custom executable, connects to a running browser, and `Page::execute` sends any CDP command. If it lacks something capture needs, copy agent-browser's client instead (with its notice in `THIRD_PARTY_NOTICES.md`).
3. **Sessions.** Launch with the Session node's viewport, device scale and user agent; one browser context per take; saved sign-ins applied with `Network.setCookies` and a per-origin storage init script, as the Sign in step does today.
4. **Selectors.** Port the ladder (`autoCandidates`, `candidatesFor`, `softFirst`, `dumpVisible` in `selectors.ts`): role and name through `Accessibility.queryAXTree`; label, placeholder, visible text and test id through an injected script that returns candidates with boxes; visibility from the box and computed style; an auto-wait loop with the same timeouts and messages.
5. **Actions and recording.** Port `driver.ts`, `take-run.ts` and `take-steps.ts`: glides with the 8 ms sampler and the exact samples (`walk`, `drag-path.ts`), drags with `holdMs` and the 6 px nudge, cursor sampling and the cursor keyword after glides and clicks, chapters, typing. Overlays that `page.screencast.show*` drew are drawn by an injected script.
6. **Screencast and encoder.** `Page.startScreencast` frames go straight into the FFmpeg encoder's stdin without waiting (`FrameEncoder.offer` semantics: never block the frame callback; acknowledge each frame with `Page.screencastFrameAck` immediately). Timing comes from frame timestamps from the first frame after setup, as today. Live PiP frames are the same frames, throttled (`runner-and-executors.md`, Work step 9).
7. **Traces.** Our own format: per take, a manifest with groups (node ids in order, as `groups` maps today), each action with its time, the screenshots, console messages (`Runtime.consoleAPICalled`) and network requests (`Network.*` events), stored in `.opsis/traces` with the same content-addressed blobs. The Trace tab (`TracePanel`) reads the new manifest. DOM snapshots, which Playwright's traces include, are left out.
8. **Executors.** `capture.record`, `capture.still` and `browser.signIn` on the above; "Signed out during the take" and the other messages unchanged.
9. **Tests.** Port the capture tests (677 lines); the pure parts (`drag-path`, `capture-scale`, `cursor-keyword`, `take-steps`, `encoder`) keep their expected values. Run the TodoMVC sample and compare the action log and frame count with a take recorded before the move.

## Risks

- **Playwright's waiting rules.** Auto-waiting for actionability (attached, visible, stable, enabled, receiving events) is what makes takes reliable; reproduce it step by step and test on the templates' sites.
- **Accessible names.** `getByRole` uses Playwright's accessible-name computation; Chromium's accessibility tree computes its own. Test the ladder against the picked names in every template.
- **Frame timing.** The screencast acknowledges frames one at a time; any wait in the frame path loses page changes, as `AGENTS.md` **Capture timing** warns.

## Sources

- `src/worker/capture/*`, `src/worker/executors/capture.ts`, `src/engine/install.ts`, `node_modules/@remotion/renderer/dist/browser/get-chrome-download-url.js` (read 2026-10-25).
- agent-browser source at its default branch and its releases (`gh api repos/vercel-labs/agent-browser/releases/latest`), [agent-browser on docs.rs](https://docs.rs/crate/agent-browser/latest), [agent-browser native mode](https://agent-browser.dev/native-mode) (2026-10-25).
- [chromiumoxide on docs.rs](https://docs.rs/crate/chromiumoxide/latest) (2026-10-25).
