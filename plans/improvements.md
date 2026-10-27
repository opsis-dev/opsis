# Improvements

Two improvements found by comparing Opsis with a hand-built feature-video recorder: recording on a simulated clock, and server replies timed on that clock.

Status: plan only. Nothing here is built yet.

## How to read this plan

- Each section is one improvement and can be built on its own, except section 2, which needs section 1.
- Each section has **What it adds**, **In place**, **Work** (numbered) and, where there are any, **Risks**. Rules from `AGENTS.md` hold throughout; when the two disagree, `AGENTS.md` wins.
- Sizes are rough: small (a few files, one crate and its card or widget), medium (several crates or the editor and Rust together), large (a new way of working through a whole crate).

## Conventions

- New settings are `schemars` fields with their label, limits and `x-` keywords; the editor builds the form from them. Technical settings go under `x-advanced`.
- A new node has a `description`, a `card`, a look on its definition (`.look(…)`, then `bun run glyphs:bundle`) and passes `crates/compile/tests/unit/nodes_c.rs`.
- Copy names no library, tool or protocol. Browser nodes are "actions". Durations in seconds.
- A change to how a spec is drawn bumps nothing before release (`PICTURE_VERSION` stays); clear test caches instead.
- A new built-in sound or music file needs a checked free licence, an entry in `THIRD_PARTY_NOTICES.md` and a local file; nothing loads from the internet.
- Checks for every section: `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, the no-comments check, then the app itself (`check-app`).

## Summary

| Improvement                    | What users get                                                                          | Size  |
| ------------------------------ | --------------------------------------------------------------------------------------- | ----- |
| 1. Smooth recording            | A take with exactly one new picture per frame, however busy the computer is             | large |
| 2. Replies on the take's clock | In a smooth take, server replies and streamed answers land on the frames they took live | large |

## 1. Smooth recording

### What it adds

A **Timing** setting on Record take: "Real time" (today) or "Smooth (slower)". A smooth take moves the page forward one frame at a time and takes one picture per frame, so every second of video has exactly as many distinct frames as the frame rate, whatever the computer is doing. Real-time takes skip frames when the page repaints late or the machine is busy. The setting sits on Record take rather than Open browser, because Screenshot doesn't need it and the same session can be recorded both ways.

It runs far slower than real time (a full picture per frame), so it is for final takes, not drafts.

### In place

- Frames come from the screencast with Chrome's timestamps (`crates/capture/src/screencast.rs`) and go to `FrameEncoder::offer` with a time; the encoder writes a constant-rate stream from those times (`crates/capture/src/encoder.rs`). A picture-per-frame source fits the same `offer`.
- The page clock from Freeze clock: Playwright's fake clock, injected by `crates/capture/src/clock.rs` (`install`, `pauseAt`, `setFixedTime`; `runFor` is in the same script).
- Action times come from `recorder.elapsed()` (`crates/capture/src/take.rs:253`, passed to the cursor driver at `:600`).
- About 50 real-time waits: `take.rs` (23), `cursor.rs` (12, including `pause` at `:149` and the 8 ms glide sampler), `page.rs` (9), `selectors.rs` (3 polls), `overlays.rs` (3), `input.rs` (2).

### Work

1. **Setting.** `timing` on `Record` (`crates/compile/src/nodes/capture.rs:47`): `real` and `smooth`, labels "Real time" and "Smooth (slower)", with an `x-help` that says it takes much longer. It is a param, so it is in the cache key.
2. **Take clock.** One clock the take asks for time and waits through: `now()`, `wait(ms)`, and `wait_until(check, timeout)` for polls. The real clock sleeps and polls as today; every wait and time source listed under In place goes through it.
3. **Frame stepper.** At Start recording in a smooth take: install the page clock paused (at the Freeze clock time when there is one, otherwise now), set the page's CSS and web animations to rate 0, stop the screencast. Each tick: run the page clock forward one frame, step every running animation by one frame, take a picture, `offer` it with the tick's time. `wait(ms)` is that many ticks; `wait_until` ticks until the check passes or the timeout in clock time passes.
4. **Cursor and typing.** Glides, drags, typing delays and pauses run on ticks, so the cursor moves one sample per frame and the logged times are clock times. The drawn cursor and Auto zoom read the log as before.
5. **Before Start recording.** Actions before the first Start recording step run on the real clock, as off-camera setup.
6. **Live view and traces.** The ticks' pictures go to the live view (throttled as now) and the trace.
7. **Card.** Record take's card says "Smooth" when it is on.
8. **Self-test page.** A local page with typing, a CSS transition, a CSS animation, a web animation, a `requestAnimationFrame` loop and a timer; the test checks frame count, that each frame differs where the page moves, and the logged times.

### Risks

- With the page's timers paused, anything the take waits for only happens while it ticks; a wait that doesn't tick never ends. Every wait must go through the clock.
- Videos inside the page, canvases driven by real time and workers' own timers don't follow the page clock.
- Server replies arrive in real time while the page clock runs slower, so waits on the server take fewer frames than they took seconds. Section 2 fixes that.
- Long takes: one picture per frame at 2× is slow; measure a 30 s take before choosing the default picture format.

## 2. Replies on the take's clock

### What it adds

In a smooth take, every server reply reaches the page on the frame that matches how long it really took: a reply sent at page time `v0` that took `r1 − r0` live lands at `v0 + (r1 − r0)`. Streamed replies (an assistant typing its answer) arrive piece by piece at their own times instead of all at once. A "Longest wait" setting caps slow replies so a slow server doesn't look like a slow app.

Because smooth recording is slower than real time, a reply has almost always arrived before its frame comes; the take only holds it and hands it over on time. Each take uses the delays it measures, so two smooth takes of the same flow can differ slightly where the server was slower one time; the cap keeps that small.

### In place

- Request interception for Mock response, pausing requests before they're sent (`crates/capture/src/route.rs`, `Fetch.enable` at the request stage).
- The trace records every request and reply with times (`crates/capture/src/trace.rs:186`).
- Section 1's take clock and frame stepper.

### Work

1. **Hold whole replies.** In smooth takes, intercept at the reply stage: note the page time and real time each request was sent, take the body when it arrives, and release it on the first tick at or after its due page time. Covers pages, scripts, styles, images, fonts and plain API calls. Mock response keeps working (a mocked reply is due at once).
2. **Hold streamed replies.** An init script wraps `fetch` bodies, `EventSource` and `WebSocket` messages: it reads the real stream, notes each piece's real arrival, and passes each piece to the page at its own page time. Its timers already run on the paused page clock.
3. **Stall.** When a request is still out and the page clock has caught up with the real time since it was sent, the stepper stops ticking until the reply arrives.
4. **Longest wait.** A setting on Record take, under `x-advanced`, shown only for smooth takes (ms, `x-zero-label` "As live"), capping each reply's delay.
5. **Self-test.** A local server with a delayed reply, a slow image and a stream that sends pieces 200 ms apart; the test checks the frames each one lands on.

### Risks

- Frames from other origins (out-of-process iframes), service workers and cached replies need the same hold per target or are left at real time.
- Uploads and very long streams that never end must not let the stall rule freeze the take; the cap in step 4 and the take's timeout apply.
