# Improvements

Five improvements found by comparing Opsis with a hand-built feature-video recorder: a sound for every click and keystroke, Sequences and Layers with any number of clips, built-in music, recording on a simulated clock, and server replies timed on that clock.

Status: plan only. Nothing here is built yet.

## How to read this plan

- Each section is one improvement and can be built on its own, except section 5, which needs section 4.
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

| Improvement                    | What users get                                                                          | Size   |
| ------------------------------ | --------------------------------------------------------------------------------------- | ------ |
| 1. Action sounds               | One node that puts a click sound on every click and a key sound on every keystroke      | medium |
| 2. Sequences and Layers grow   | Any number of clips in one Sequence or Layers node; splitting never runs out of inputs  | medium |
| 3. Built-in music              | A few free music beds to pick, next to the built-in sound effects                       | small  |
| 4. Smooth recording            | A take with exactly one new picture per frame, however busy the computer is             | large  |
| 5. Replies on the take's clock | In a smooth take, server replies and streamed answers land on the frames they took live | large  |

## 1. Action sounds

### What it adds

One node, **Action sounds**, that adds a click sound at every click and a key sound at every keystroke of the takes in a clip, as one Mixer channel with its own volume. Today each sound is a Sound effect placed on one action, so a take with 30 clicks needs 30 nodes and 30 channels, and typing can't get a sound per key at all. A single action stays quiet through its own "No sound" switch, so the film keeps one node and one channel however many actions are silenced.

### In place

- Sound effect (`audio.sfx`) outputs a `sound` with an optional action anchor; Add sound (`audio.mix`) places anchored sounds at the action's mark through `anchor_sounds` and `flatten_spec` (`crates/compile/src/nodes/audio.rs:408`).
- Action marks are one per action (`ActionMark`, `crates/compile/src/timeline/flatten.rs:235`). Keystroke times exist only in the take's log (`KeyEvent`, and typed text on the `TakeClick` of a Type).
- Built-in sounds include "Mouse click" (`crates/compile/src/data/sounds.json`); there is no key sound.

### Work

1. **Key sound.** Find a CC0 key click, first in the set Remotion lists in `@remotion/sfx` that the built-in sounds came from, check its licence on its source page, and add it to `public/sfx`, `sounds.json` and `THIRD_PARTY_NOTICES.md`.
2. **Keystroke times.** Make the take's keystroke times available to compile as a list on the flattened bar, mapped through trims, loops and speed like the action marks. A mark per keystroke would crowd the Timeline.
3. **Node.** `audio.actionSounds`: input `clip`, output `sound`; settings: click sound (`sound` widget, default "Mouse click"), key sound (default the new one), volume, "Clicks" and "Keys" switches, and a small seeded change in pitch and level per hit so typing doesn't sound mechanical and preview still equals export. Compile turns every click and keystroke inside the clip into an `AudioItem` with `role: "effect"`.
4. **Mixer.** One channel keyed by the node id, on the Effects bus, like a Sound effect. These sounds never key the music ducking (`keepMusic` behaviour).
5. **No sound on an action.** Click, Type and Press keys get a "No sound" switch (`silent`, under `x-advanced`). It travels with the action into the take's log, and Action sounds skips that action's click and keystrokes.
6. **Card.** Rows for the count of clicks and keys it places and the sounds chosen; an action's own card says "No sound" when its switch is on.
7. **Look.** A look on the definition; no motion.
8. **Tests.** Compile tests on a clip with a trim and a speed ramp (the sounds follow the mapped times), a silent Click and a silent Type in the same take, the Mixer channel, the round trip into Add sound.

## 2. Sequences and Layers grow

### What it adds

A Sequence or Layers node takes as many clips as are connected: a new empty input appears each time the last one is connected. Today both stop at six (`in1`…`in6`), so a long film needs nested Sequences, and splitting a clip on the Timeline does nothing when its Sequence is full. Layers grows the same way as Sequence because they share the same inputs.

### In place

- `IN_PORTS` and `CUT_PORTS` are fixed lists of six and five (`crates/compile/src/ports.rs:64`).
- A cut's own transition names its port with the `CutSlot` enum (`in2`…`in6`, `crates/compile/src/nodes/timeline.rs:82`).
- Timeline slots are container node + port; `splitBar` shifts later ports and cuts by one and returns nothing when the last input is used (`crates/compile/src/edits/timeline.rs:1094`).
- Nesting works today: a Sequence outputs a `clip` and the spec is recursive.

### Work

1. **Ports per node.** Let a node definition declare a growing input family (`in1`, `in2`, … with one empty input after the last connected one). The catalogue keeps the family's port type and label pattern; each compile sends every node's current inputs so the editor draws them per node.
2. **Cuts.** `CutSlot` becomes a port name checked against the family (`in<n>`, n ≥ 2), with the "Before" field listing the node's connected ports from `InspectorContext`.
3. **Edits.** `splitBar`, removal and ripple edits shift ports without a ceiling; removing a clip closes the gap so the inputs stay contiguous.
4. **Editor.** Draw the node's inputs from the compile, with the empty one last; dropping a connection on the empty one adds the next.
5. **Local API.** `/nodes/{type}` describes the family; edge checks accept any `in<n>`.
6. **Tests.** Round trips in `crates/compile/tests/unit/timeline_edits.rs` with ten and more clips, a split in a node with seven, cut transitions past `in6`, every example opening as before.

### Risks

- Every place that loops over `IN_PORTS` assumes six; find them all (`rg IN_PORTS`, `CUT_PORTS`, `in6`).

## 3. Built-in music

### What it adds

A few free music beds (calm, upbeat, ambient; 1–3 minutes each, loopable) offered in Music's song field next to the project's files, the way built-in sounds are offered in Sound effect. Opsis makes no sound itself: Music plays a file and the Mixer only processes, so a project without a song file has no music at all. A short list of good beds covers demo films; generating music is out of scope.

### In place

- Seven built-in sound effects in `public/sfx` (mouse click, whoosh, whip, switch, page turn, two camera shutters), CC0 files from the set Remotion lists in `@remotion/sfx`, credited in `THIRD_PARTY_NOTICES.md` ("Built-in sound effects"). No music ships with the app.
- Music (`audio.music`) with `src` as an `asset` field, loop, trim, fades and keyframes.
- The `sound` widget lists built-in sounds from `crates/compile/src/data/sounds.json` with a play button.

### Work

1. **Pick the tracks.** CC0 or similarly free for commercial use, checked one by one on their source pages; credits in `THIRD_PARTY_NOTICES.md`; files under `public/music`, kept small (a compressed format the file server and FFmpeg already handle).
2. **List.** `crates/compile/src/data/music.json` (label, mood, `app:` ref, length, loop points); the editor reads the same file.
3. **Field.** Music's song field offers the built-in list with a play button, like the `sound` widget.
4. **Size.** Note the added size in `plans/release.md` section 1.

## 4. Smooth recording

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
- Server replies arrive in real time while the page clock runs slower, so waits on the server take fewer frames than they took seconds. Section 5 fixes that.
- Long takes: one picture per frame at 2× is slow; measure a 30 s take before choosing the default picture format.

## 5. Replies on the take's clock

### What it adds

In a smooth take, every server reply reaches the page on the frame that matches how long it really took: a reply sent at page time `v0` that took `r1 − r0` live lands at `v0 + (r1 − r0)`. Streamed replies (an assistant typing its answer) arrive piece by piece at their own times instead of all at once. A "Longest wait" setting caps slow replies so a slow server doesn't look like a slow app.

Because smooth recording is slower than real time, a reply has almost always arrived before its frame comes; the take only holds it and hands it over on time. Each take uses the delays it measures, so two smooth takes of the same flow can differ slightly where the server was slower one time; the cap keeps that small.

### In place

- Request interception for Mock response, pausing requests before they're sent (`crates/capture/src/route.rs`, `Fetch.enable` at the request stage).
- The trace records every request and reply with times (`crates/capture/src/trace.rs:186`).
- Section 4's take clock and frame stepper.

### Work

1. **Hold whole replies.** In smooth takes, intercept at the reply stage: note the page time and real time each request was sent, take the body when it arrives, and release it on the first tick at or after its due page time. Covers pages, scripts, styles, images, fonts and plain API calls. Mock response keeps working (a mocked reply is due at once).
2. **Hold streamed replies.** An init script wraps `fetch` bodies, `EventSource` and `WebSocket` messages: it reads the real stream, notes each piece's real arrival, and passes each piece to the page at its own page time. Its timers already run on the paused page clock.
3. **Stall.** When a request is still out and the page clock has caught up with the real time since it was sent, the stepper stops ticking until the reply arrives.
4. **Longest wait.** A setting on Record take, under `x-advanced`, shown only for smooth takes (ms, `x-zero-label` "As live"), capping each reply's delay.
5. **Self-test.** A local server with a delayed reply, a slow image and a stream that sends pieces 200 ms apart; the test checks the frames each one lands on.

### Risks

- Frames from other origins (out-of-process iframes), service workers and cached replies need the same hold per target or are left at real time.
- Uploads and very long streams that never end must not let the stall rule freeze the take; the cap in step 4 and the take's timeout apply.
