# Improvements

One improvement found by comparing Opsis with a hand-built feature-video recorder: server replies timed on the clock of a smooth take.

Status: plan only. Nothing here is built yet.

## How to read this plan

- Each section is one improvement and can be built on its own.
- Each section has **What it adds**, **In place**, **Work** (numbered) and, where there are any, **Risks**. Rules from `AGENTS.md` and `agent-docs/` hold throughout; when they disagree with this plan, they win.
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
| 1. Replies on the take's clock | In a smooth take, server replies and streamed answers land on the frames they took live | large |

## 1. Replies on the take's clock

### What it adds

In a smooth take, every server reply reaches the page on the frame that matches how long it really took: a reply sent at page time `v0` that took `r1 − r0` live lands at `v0 + (r1 − r0)`. Streamed replies (an assistant typing its answer) arrive piece by piece at their own times instead of all at once. A "Longest wait" setting caps slow replies so a slow server doesn't look like a slow app.

Because smooth recording is slower than real time, a reply has almost always arrived before its frame comes; the take only holds it and hands it over on time. Each take uses the delays it measures, so two smooth takes of the same flow can differ slightly where the server was slower one time; the cap keeps that small.

### In place

- Request interception for Fake a response (`MockResponse`), pausing requests before they're sent (`crates/capture/src/route.rs`, `Fetch.enable` at the request stage).
- The trace records every request and reply with times (`crates/capture/src/trace.rs` → `Trace::start`).
- Smooth takes: the take clock (`TakeClock`, `crates/capture/src/take_clock.rs`) and the frame stepper (`Smooth`, `crates/capture/src/smooth.rs`), see Smooth takes in `agent-docs/capture.md`.

### Work

1. **Hold whole replies.** In smooth takes, intercept at the reply stage: note the page time and real time each request was sent, take the body when it arrives, and release it on the first tick at or after its due page time. Covers pages, scripts, styles, images, fonts and plain API calls. Fake a response keeps working (a faked reply is due at once).
2. **Hold streamed replies.** An init script wraps `fetch` bodies, `EventSource` and `WebSocket` messages: it reads the real stream, notes each piece's real arrival, and passes each piece to the page at its own page time. Its timers already run on the paused page clock.
3. **Stall.** When a request is still out and the page clock has caught up with the real time since it was sent, the stepper stops ticking until the reply arrives.
4. **Longest wait.** A setting on Record take, under `x-advanced`, shown only for smooth takes (ms, `x-zero-label` "As live"), capping each reply's delay.
5. **Self-test.** A local server with a delayed reply, a slow image and a stream that sends pieces 200 ms apart; the test checks the frames each one lands on.

### Risks

- Frames from other origins (out-of-process iframes), service workers and cached replies need the same hold per target or are left at real time.
- Uploads and very long streams that never end must not let the stall rule freeze the take; the cap in step 4 and the take's timeout apply.
