# Profiling findings (2026-11-04)

Everything the profiling work found: bugs, where time and memory go, size, and code paths that cost more than they need to. Each item that changed since starts with its status on 2026-11-05 and the commit that did it. Each item gives what was measured or read, where (file and line, or the command and its output), how far it reaches, and how sure it is. How the measurements are made is in [agent-docs/profiling.md](../agent-docs/profiling.md).

## How these were measured

- **Machine:** Linux x64, 20 cores, 14 GiB of memory, Intel Iris Xe and NVIDIA RTX 3060, Ubuntu 26.04, kernel 7.0.0-42.
- **Build:** the deb from `bun run make -- --bundles deb` with the shipped profile, `CARGO_PROFILE_RELEASE_STRIP=false` and `CARGO_PROFILE_RELEASE_DEBUG=line-tables-only` (full debug info ran the LTO link out of memory: 8 GiB resident plus 7 GiB swapped). `~/.cache/opsis-build/release/bundle/deb/Opsis_0.1.0_amd64.deb`, 156.04 MiB.
- **Runs:** `bun run profile` in Docker: Ubuntu 26.04 with the deb installed, cores 0–3, 6 GiB of memory, no network, Xvfb, no GPU unless stated. A project of 438 nodes holds every one of the 272 node types; each working node runs alone, upstream from cache; `/proc` is read every 250 ms.
- **Full pass:** `~/.cache/opsis-profile/runs/2026-11-04T13-12-42-none/` (208 runs, Sign in left out; 205 passed). Sign in's run is in `2026-11-04T12-08-31-none`.
- **Profiler pass:** samply on a 45-run subset (`2026-11-04T13-55-25-samply`), the app's own CPU only.
- **The editor:** the e2e build (debug Rust, production editor) in the same container, driven through its WebDriver and read through WebKit's remote inspector.

## Bugs

### B1. Every render of an imported video clip fails

- **Status:** Fixed in `6e27fd3f`: a clip made from a file starts from its element's default props, and `crates/render/tests/run.rs` renders an imported clip.
- **What:** a video clip made from a media file has no `playbackRate` prop, so the film page asks for frame `NaN` and the render fails with "A frame's time is whole microseconds."
- **Evidence:** in every full pass, the three renders whose film is an imported video fail within a second: Clip from file (`output_render_1`), Grade footage on the import (`output_render_2`), Fit length on the import (`output_render_150`). Element nodes with their own `src` (Video, Video loop, Text mask video) render, because their settings carry `playbackRate` from the element defaults.
- **Source:**
  - `media_to_clip` writes the video element's props from `FileClipProps` (`src`, `fit`, `muted`, `background`): `crates/compile/src/nodes/media.rs:336-342`, used at `:407`.
  - Rust reads a missing rate as 1 (`#[serde(default = "one")]`, `crates/compile/src/clip.rs:44-45`), so the mix and audio bars are right and nothing in Rust notices.
  - The page doesn't fill defaults (`default-props.generated.json` is imported only by tests) and computes `(frame / fps) * playbackRate`: `packages/video/src/elements/media/video.tsx:19`.
  - `footageFrameUrl` turns `NaN` into the text "NaN" (`Math.max(0, NaN)`): `packages/video/src/core/footage.tsx:9-10`; the footage server refuses it: `crates/footage/src/serve.rs:21-26`.
- **Reach:** any film that uses an imported video through Clip from file, or a node that takes the import's clip output. **Confidence:** reproduced on every pass; cause traced through the code.

### B2. Sign in panics on Linux, and the app then can't run anything until it restarts

- **Status:** Fixed in `3607fc0b`: every keyring call from a run goes through `off_runtime` onto a blocking thread, and a node whose executor panics fails with a message while the run still ends. `crates/capture/tests/keyring_sign_in.rs` saves a sign-in through a real Secret Service.
- **What:** the Sign in executor reads the Secret Service keyring on a tokio worker; the blocking zbus call panics there; the run's task dies, the run never ends, and `/project/run/stop` doesn't clear it.
- **Evidence:** `Cannot start a runtime from within a runtime` on `tokio-rt-worker`, with this stack (from `RUST_BACKTRACE=1`): `opsis_capture` executor `crates/capture/src/executors.rs:458` (`deps.sign_ins.write`) → `crates/desktop/src/settings.rs:206` (`write_secret`) → `crates/store/src/secrets.rs:36` (`keys.seal`) → `crates/store/src/keys.rs:107` (`seal`) → `:90` (`sign_in_key`, `entry.get_password()`) → `zbus-secret-service-keyring-store` → `zbus::blocking` `block_on`. The run waited 900 s, `/project/run/stop` didn't end it (a minute later the app still reported a run in progress), and the next `POST /project/run` answered 409 "A run is already in progress."
- **Source:**
  - `Keyring::sign_in_key` calls the keyring on every `seal` and `unseal`, with no cached key: `crates/store/src/keys.rs:88-124`. `unseal`, which a take that uses the saved sign-in needs, makes the same call.
  - AGENTS.md §7: "Nothing blocks on the async runtime. The keyring is the one thing made in setup itself, because the Linux Secret Service store blocks inside `zbus` and panics on a runtime thread."
  - Only `RunDone` clears the active run (`crates/desktop/src/run.rs:380-385`); the run is a `tauri::async_runtime::spawn` task (`run.rs:475-478`), and a panic in it sends no `RunDone`.
- **Reach:** every Sign in on Linux, and the app stays stuck until it restarts. **Confidence:** reproduced twice; the backtrace names every frame.

### B3. A Drag on a page whose drag carries no data fails the take

- **Status:** Fixed in `43d46eb2`: the drag data always carries `items`; `crates/capture/tests/local_page.rs` drops a card whose page sets no data.
- **What:** when the page's `dragstart` puts nothing in `dataTransfer`, Chrome reports the intercepted drag with `"items": []`; the app re-serializes it through chromiumoxide, which leaves the empty `items` out, and every `Input.dispatchDragEvent` fails with -32602.
- **Evidence:** the app's own calls replayed against the recording browser (HeadlessChrome 157.0.8080.0), card `draggable="true"`, drop zone accepting drops:

  | Case                                           | `dragEnter`, `dragOver`, `drop`                                                      | Page saw                                        | Card ended in |
  | ---------------------------------------------- | ------------------------------------------------------------------------------------ | ----------------------------------------------- | ------------- |
  | no data, `items` left out (what the app sends) | -32602 "Failed to deserialize params.data.items - BINDINGS: mandatory field missing" | `dragstart` only                                | body          |
  | no data, `"items": []` sent                    | ok                                                                                   | `dragstart`, `dragenter`, `dragover` ×2, `drop` | drop zone     |
  | page sets data                                 | ok                                                                                   | same as above                                   | drop zone     |

  In the profile, the take failed at the first "Move mouse" of Drag (its trace) until the test page called `setData`.

- **Source:** `move_watching_drags` sends `serde_json::to_value(&event.data)`: `crates/capture/src/input.rs:295-343` (`:336`); the same data goes to `dragOver` (`:281`) and `drop` (`:407`). chromiumoxide 0.9.1's `DragData` has `#[serde(skip_serializing_if = "Vec::is_empty")]` on `items`: `chromiumoxide_cdp-0.9.1/src/cdp.rs:62221-62224`. `cancel_drag` already writes `"items": []` by hand: `input.rs:353`.
- **Reach:** Drag on any element whose `dragstart` sets no `dataTransfer` data; a drag whose page sets data passes. **Confidence:** reproduced in the take and at the protocol level; sending `"items": []` makes the same drag complete.

### B4. The Render setting "Graphics card, direct" can't work with the bundled browser

- **Status:** Fixed in `32fbb747`: the setting is removed.
- **What:** `Gl::Egl` passes `--use-gl=egl` (`crates/capture/src/film.rs:95`); the bundled headless shell refuses it and its GPU process exits, so WebGL falls back to SwiftShader.
- **Evidence:** with the film's switches, the browser logs "Requested GL implementation (gl=egl-gles2,angle=none) not found in allowed implementations: [(gl=egl-angle,angle=default)]" then "Exiting GPU process due to error", and WebGL reports "SwiftShader driver": in the container with and without a GPU device, and on this desktop.
- **Reach:** anyone who picks that setting gets software rendering without being told. **Confidence:** measured in both places.

## Where the time goes

### P1. Renders are almost all of a pass

- 147 of 150 renders passed (the 3 failures are B1). Renders took 1,641 s of the 1,747 s of passed runs, and 4,960 of 5,180 CPU seconds.

  | Part                                                               | Runs | Wall      | CPU       |
  | ------------------------------------------------------------------ | ---- | --------- | --------- |
  | renders of element nodes                                           | 112  | 1,092.5 s | 3,138.8 s |
  | renders of transitions                                             | 26   | 472.0 s   | 1,631.2 s |
  | renders of the mixed browser film, the camera take, timeline nodes | 9    | 76.7 s    | 190.0 s   |
  | media nodes                                                        | 23   | 51.8 s    | 168.1 s   |
  | the two takes and the page still                                   | 3    | 31.9 s    | 24.5 s    |
  | terminal (Play tape, Cast file)                                    | 2    | 13.5 s    | 10.0 s    |
  | other outputs (take ×3, tape, still)                               | 5    | 4.2 s     | 9.7 s     |
  | image nodes                                                        | 20   | 3.9 s     | 7.4 s     |
  | sounds                                                             | 5    | 0.5 s     | 0.8 s     |

- Render length: minimum 1.9 s, median 5.7 s, 90th percentile 20.0 s, maximum 128.0 s. Starting and finishing are small: median 0.5 s before the first frame and 0.6 s after the last, 203 s of the 1,641 s in all; the rest is drawing frames.
- **By process (whole pass):** Chrome's GPU process 3,636 CPU s (70%), FFmpeg 866 s, the Chrome browser process 202 s, the harness 198 s (the sampler itself, kept apart), the app 104 s, processes that exited between samples 86 s, Chrome renderers 84 s.

### P2. Shader, 3D and particle elements: software WebGL in the container, several times faster on a GPU

- **Status:** Corrected on 2026-11-05. On this laptop now (NVIDIA 595.99.02, loaded since the 12:37 boot on 2026-11-04), "Graphics card, newer drivers" draws no WebGL: with both Intel's and NVIDIA's Vulkan drivers installed, ANGLE starts on Vulkan and then fails to share the first WebGL image (`initExternalMemory`, error -2). With only NVIDIA's driver visible (`VK_ICD_FILENAMES`) it reaches the RTX 3060, and with only Intel's it falls back to SwiftShader. The "Newer drivers" column below is not what this laptop shows now; the "Default" desktop rows still hold. A film without WebGL renders with it (a Title in 3.9 s). Since `fed07e38` a render that fails this way says the setting can't draw shader and 3D effects and which Draw with to choose.
- **What:** the default "Graphics card" setting reaches the GPU only through the display the browser inherits (X or Wayland). With no display, or with a display that has no GPU behind it like Xvfb, it draws WebGL in software: SwiftShader in the full pass, Mesa's llvmpipe once Mesa is in the image (samply: `libgallium` 41.8% of all samples). The "newer drivers" setting (Vulkan) reaches the GPU with no display at all.
- **Evidence:** the WebGL renderer the recording browser reports with the film's switches:

  | Where                                             | Default (`--use-gl=angle`)                                            | Newer drivers (`--use-angle=vulkan`)                     |
  | ------------------------------------------------- | --------------------------------------------------------------------- | -------------------------------------------------------- |
  | this desktop, `DISPLAY` and `WAYLAND_DISPLAY` set | Intel Iris Xe, OpenGL ES 3.2                                          | NVIDIA RTX 3060                                          |
  | this desktop, only `DISPLAY`                      | Intel Iris Xe, OpenGL 4.6                                             | NVIDIA RTX 3060                                          |
  | this desktop, only `WAYLAND_DISPLAY`              | Intel Iris Xe, OpenGL ES 3.2                                          | NVIDIA RTX 3060                                          |
  | this desktop, neither                             | SwiftShader ("Could not open the default X display")                  | NVIDIA RTX 3060                                          |
  | container, no display, with or without `/dev/dri` | SwiftShader                                                           | llvmpipe without `/dev/dri`, Intel Iris Xe with it       |
  | container under Xvfb (the app's runs)             | software: the app's log shows "DRI3 error: Could not get DRI3 device" | Intel Iris Xe with `--gpu` (the right-hand column below) |
  - The same renders on the same image (with Mesa), with the default setting and then with the Intel GPU through Vulkan (`--gpu --render-gl vulkan`); passing `/dev/dri` with the default setting changed nothing (every render within 7%):

    | Render                      | Default setting, no usable GPU | Intel GPU (Vulkan) |
    | --------------------------- | ------------------------------ | ------------------ |
    | Lens zoom (transition)      | 137.6 s                        | 8.8 s              |
    | God rays (shader)           | 119.0 s                        | 10.1 s             |
    | Launch anything             | 109.1 s                        | 30.5 s             |
    | Smoke dissolve (transition) | 90.4 s                         | 10.4 s             |
    | Grain gradient (shader)     | 69.2 s                         | 8.3 s              |
    | Grain dissolve (transition) | 47.9 s                         | 6.6 s              |
    | Ripple zoom (transition)    | 42.1 s                         | 6.5 s              |
    | Warp (shader)               | 38.9 s                         | 8.6 s              |
    | Perlin noise (shader)       | 38.9 s                         | 11.1 s             |
    | Wave wipe (transition)      | 35.7 s                         | 6.6 s              |
    | Shapes 3D                   | 21.0 s                         | 7.8 s              |
    | Model 3D                    | 12.6 s                         | 6.6 s              |
    | Solid (baseline)            | 2.8 s                          | 3.0 s              |

  - In the full pass (SwiftShader), Lens zoom drew 91 frames in 126.8 s (1.4 s a frame) and God rays 150 frames in 107.7 s; a Solid drew 90 frames in 1.5 s (frames counted by the render's progress events).

- **Reach:** every render of these elements with the default setting where the app has no display backed by a GPU: a headless machine, a VM, a CI runner, a virtual display. On a desktop session the default setting reaches the GPU. **Confidence:** measured; the desktop rows are the renderer the browser reports, the render times are the container's.

### P3. Media nodes re-encode at x264 CRF 18, even when nothing changes

- **Status:** Trim with no cut hands the file on as it is (`0a994062`). The encode speed of every working file is one app-wide setting, Settings → General → Working files, Balanced (`medium`) by default, in place of Apply grade's and Convert's own (`afb49db9`). Render keeps its own.
- Every node that writes video re-encodes the 12 s 1080p test clip: Apply grade 12.9 s (45.4 CPU s), Reverse 6.9 s (23.0), Scale and pad 6.3 s (23.4), Convert 6.1 s (22.7), Trim 5.6 s (20.4), Speed 3.4 s (12.0). The encoder settings are `edit_video` (x264, CRF 18, FFmpeg's default preset): `crates/media/src/codecs.rs:13`, `:58-63`.
- **Trim with its defaults re-encodes the whole file:** the defaults are `startMs: 0`, `endMs: 0` (no cut), and `trim` always goes through `transcode` with `edited_file`'s codecs: `crates/media/src/executors.rs:337-374`, `crates/media/src/codecs.rs:417-432`.
- Sound-only nodes take 0.1–1.5 s; Hum 1.5 s, Pitch 0.9 s, Clean speech 0.5 s.

### P4. Takes

- Browser take with 19 actions on the local site: 17.1 s, 15.2 CPU s (FFmpeg 9.6). Terminal take with 10 actions: 13.7 s. Play tape (8.1 s tape): 10.8 s. Cast file: 2.7 s. Page still: 1.0 s.

### P5. The standalone programs

| Program                                   | Wall    | CPU (user + system) | Peak RSS                    |
| ----------------------------------------- | ------- | ------------------- | --------------------------- |
| `editor-data catalog`                     | 0.11 s  | 0.07 s              | 70.3 MiB                    |
| `editor-data film`, the 438-node project  | 0.07 s  | 0.05 s              | 56.9 MiB                    |
| `render-elements`, all 138 element stills | 110.4 s | 38.7 s              | 246.8 MiB (its own process) |

## The app's own CPU (samply)

The 45-run subset: image and capture nodes, the terminal, take, tape and still outputs, and ten renders (footage, mix, 3D, Lottie). The app used 42.3 s of CPU, 99.2% of it on tokio workers. Shares are of the app's samples.

### E1. Encoding footage frames as JPEG is 42% of the app's CPU

- `opsis_footage::footage::jpeg` is 42.3% inclusive (`jpeg_encoder`'s AVX2 encode, `fill_buffers_avx2`, `write_ac_block`, `fdct`); Redact take's frames (`footage::redacted`) another 8.2%.
- By design: during a render the app runs one FFmpeg decoder per source, encodes each frame as JPEG 4:4:4 at quality 92 and serves it, and the page decodes it again (`agent-docs/footage.md`; `crates/footage/src/footage.rs:15`, `:56-67`).
- **Reach:** every render with footage: takes, imported video, Video elements. **Confidence:** measured.

### E2. Each run event rebuilds and sends the whole run view, with a copy of the workflow

- **Status:** The workflow is no longer cloned for each event (`8a5d38cd`); the view is still rebuilt and sent per event.
- `emit_view` runs on every event except log lines (`crates/desktop/src/run.rs:366-367`), including each `node:progress`; a render sends one per frame (815 for Launch anything). Each call clones the view and, in `with_exports` (`run.rs:88-97`), takes a clone of the whole document workflow (`document_workflow`, `crates/desktop/src/document.rs:99-103`) to read each node's id and type, then serializes it to the webview.
- `emit_view` is 7.2% of the app's CPU, `with_exports` 5.7% (both inclusive; the second is part of the first).

### E3. Compiling after node results

- `compile::request` is 16.7% inclusive: `refreshed_compile` 14.7%, `validate_workflow` 7.9%, `compile_workflow` 6.8%. Requests are coalesced (`crates/desktop/src/compile.rs:159-168`), so this is the cost of compiling 438 nodes as results arrive, not of a repeat bug.

### E4. Settings and props as JSON maps

- `IndexMap<String, serde_json::Value>` work shows across the app's own time: `get_index_of` 1.5%, `insert_full` 1.1%, `clone` and `clone_from` 0.8%, dropping `Value` 0.4%; std's SipHash 2.4% and `hash_one` 1.6%; `opsis_compile::graph::nodes_by_id` 0.6%.
- At startup the editor's catalogue request clones such maps 429,361 times (heaptrack, startup only, see T1): `opsis_compile::catalog::editor_bundle`, from the invoke handler of `bindings::builder`.

## Memory

### M1. The app keeps about 390 MiB of freed heap in glibc's malloc arenas

- **Status:** Left as it is.
- **Evidence:** the app's PSS rises from 128 MiB at start to 572 MiB after 3 minutes and 762 MiB by the end of the full pass, and doesn't come down between runs. After the same 62-run subset, with no run in progress (`/proc/<pid>/status`):

  |                                   | Default   | `MALLOC_ARENA_MAX=2` |
  | --------------------------------- | --------- | -------------------- |
  | resident (`VmRSS`)                | 730.9 MiB | 349.8 MiB            |
  | anonymous (`RssAnon`)             | 551.2 MiB | 164.2 MiB            |
  | peak (`VmHWM`)                    | 914.6 MiB | 485.8 MiB            |
  | sum of the passed runs' wall time | 157 s     | 166 s                |

  The app has 48–50 threads.

- **What grows it:** the image nodes, which run in the app on full-size pictures. The app's PSS just before and just after each run of the full pass:

  | Run          | Before  | After   | Growth   |
  | ------------ | ------- | ------- | -------- |
  | Format       | 311 MiB | 541 MiB | +230 MiB |
  | Spotlight    | 371 MiB | 572 MiB | +200 MiB |
  | Trim         | 399 MiB | 572 MiB | +173 MiB |
  | Clear colour | 399 MiB | 572 MiB | +173 MiB |
  | Redact       | 371 MiB | 541 MiB | +170 MiB |
  | Adjust       | 288 MiB | 439 MiB | +151 MiB |
  | Tilt         | 541 MiB | 684 MiB | +143 MiB |
  | Blur         | 311 MiB | 439 MiB | +127 MiB |

  Crop, Rotate, Outline, Frame, Resize and Levels add 60–111 MiB each. All 112 element renders together add 160 MiB, the 23 media nodes about 160 MiB, the sound nodes almost nothing. With `MALLOC_ARENA_MAX=2` the same image nodes add 30–103 MiB each.

- **Source:** `mallopt(3)`, `M_ARENA_MAX`: "The more arenas you have, the lower the per-thread contention, but the higher the memory usage."
- **Reach:** every session that runs image nodes; the memory stays until the app quits. **Confidence:** measured: the growth comes with the image runs, and limiting the arenas removes 387 MiB of the 551 MiB retained heap.

### M2. Peaks

- The container's peak during the full pass: 2.22 GiB (`memory.peak` 2,379,448,320). The heaviest renders are footage: FFmpeg 813–818 MiB PSS, the app 647–760 MiB, Chrome's GPU process 340–610 MiB. The editor's WebKit web process holds about 300 MiB with the project open and idle.

## The editor with 438 nodes

Measured on the e2e build (debug Rust, the production editor), in the container under Xvfb, with input through the WebDriver plugin, which dispatches its events from script like the e2e specs. Each step ran in its own ScriptProfiler session; a 1,000 ms busy loop took 925 samples, so one sample is 1.08 ms.

| Step                                                         | Wall   | Script   | Frames                                          |
| ------------------------------------------------------------ | ------ | -------- | ----------------------------------------------- |
| open the project (all 439 canvas nodes, including the group) | 2.49 s | 1,816 ms | 13 frames, one of 1,785 ms                      |
| zoom in ×3, out ×3, fit (button clicks 400 ms apart)         | 3.19 s | 554 ms   | 19 frames, median 137 ms, every one over 100 ms |
| pan: a 40-step pointer drag on the pane                      | 2.70 s | 483 ms   | 1 frame of 1,316 ms                             |
| select a node until the Inspector shows it                   | 3.71 s | 1,144 ms | 1 frame of 2,854 ms                             |

With WebKit's Timeline instrument also recording, the same steps took 2.4–2.5 s, 3.1–3.2 s, 7.7–8.0 s and 7.4–7.5 s (two sessions), so the Timeline instrument itself doubles pan and select.

### U1. Every node and edge is in the DOM, all the time

- **Status:** Fixed in `93999ea0`: only the cards and edges on screen are drawn, and each card's measured size and handles are saved per project and given back on open. Reopening the 438-node project, the most cards in the DOM went from 439 to 285 and the settled page to 14,363 elements.
- 21,852 DOM elements for 439 nodes and 360 edges. `FLOW_OPTIONS` doesn't set `onlyRenderVisibleElements` (`apps/frontend/src/editor/canvas.tsx:39-52`), so React Flow renders every node and edge, off-screen too; the minimap draws them again (`canvas.tsx:144`). React Flow 12.12.0 documents the option as an optimisation "when you have a large number of nodes and edges", off by default (`@xyflow/react` `dist/esm/types/component-props.d.ts:325-331`).
- Script is a fifth of zoom and pan (554 ms of 3.19 s, 483 ms of 2.70 s) and under a third of select; the rest of each long frame is style, layout and paint of those 21,852 elements.
- The script, by step: opening is mostly the Timeline's `scrollerView`/`update` (`src/editor/timeline/timeline-view.tsx:41-50`), `KnownNode` (`src/editor/flow-node.tsx:88`), `Canvas` and `useFlowEdges` (`src/editor/use-flow-elements.ts:107`); selecting is React Flow's node update (`@xyflow/react` `dist/esm/index.js:2944`), `selectFromChanges` (`src/editor/use-flow-elements.ts:44`), `KnownNode`, `NodeGlyph` (`src/editor/node-glyph.tsx:50`), `CardRows` and `useNodeProblems`; zoom and pan are React's store checks (`zustand` `shallow`, `checkIfSnapshotChanged`) and `react-resizable-panels`.
- **Forced layout in Base UI's composites:** 42 of select's 1,058 samples are in `scrollIntoViewIfNeeded` (`@base-ui/react` 1.9.0 `internals/composite/composite.mjs:26`), always from a `CompositeList` layout effect through `CompositeRoot` (`internals/composite/root/useCompositeRoot.mjs:43`, `CompositeRoot.mjs:74`, `list/CompositeList.mjs:114`, `:144`), with no editor frame on the stack: a composite in the Inspector registering its items on mount. The function reads `clientWidth`, `scrollWidth`, offsets and computed styles, which in a layout effect makes the page lay out all its elements then and there.
- **Confidence:** measured; the counts and the missing option are read from the page and the code.

### U2. The layout engine loads with the editor and runs on its main thread

- **Status:** Fixed in `7dd34f5d`: ELK loads when Tidy up runs and lays out in its Web Worker; the editor's main chunk went from 2,408 kB (752 kB gzipped) to 934 kB (297 kB gzipped). Tidy up on the 433 top-level nodes took 0.2 to 0.45 s on the page before.
- `apps/frontend/src/editor/layout.ts:1` imports `elkjs/lib/elk.bundled.js` statically and makes `new ELK()` without a `workerUrl` (`:10`). elkjs 0.12.1's README: "since we don't want to freeze your UI, Web Workers are supported out of the box", with the worker form `new ELK({ workerUrl: … })`.
- It is 1.47 MB of the editor's 3.67 MB of JavaScript (40%), in the editor chunk (`editor-z3ZyNgZ9.js`), loaded at start though only Tidy up uses it.

## Size

### S1. The binary

- Stripped `opsis-desktop`: 79,961,920 bytes (76.3 MiB), `.text` 58.1 MiB, `.rodata` 9.1 MiB, `.eh_frame` 3.3 MiB, `.gcc_except_table` 2.5 MiB. The bundled FFmpeg beside it: 79,826,272 bytes (76.1 MiB). The film page folder: 37,781,635 bytes (36.0 MiB): fonts 20 MiB, music 7.6 MiB, backdrops 4.6 MiB, the page's JavaScript 4.2 MiB.
- Code by crate (`cargo bloat --crates`, 57.5 MiB of functions; on 2026-10-26 it was 45.8 MiB, and `opsis_compile` 2.9 MiB):

  | Crate                             | Code         |
  | --------------------------------- | ------------ |
  | `opsis_compile`                   | 8.3 MiB      |
  | not attributed                    | 5.1 MiB      |
  | `std`                             | 4.8 MiB      |
  | `serde_core`                      | 3.2 MiB      |
  | `opsis_types`                     | 2.1 MiB      |
  | `tokio`, `jsonschema`             | 1.9 MiB each |
  | `rav1e`, `tauri`, `opsis_desktop` | 1.5 MiB each |
  | `serde_json`                      | 1.4 MiB      |
  | `aws_lc_sys`                      | 1.3 MiB      |
  | `chromiumoxide`                   | 1.2 MiB      |
  | `zbus`                            | 1.1 MiB      |
  | `opsis_capture`                   | 1.0 MiB      |
  | `schemars`                        | 1013 KiB     |

### S2. Generated code is most of `opsis-compile`

- In the binary: schemars `JsonSchema` builders 3.61 MiB (173 functions), serde `Deserialize` 6.04 MiB (4,837), specta `Type::definition` 1.64 MiB (900), chromiumoxide's protocol types 2.48 MiB (3,744). The largest single function is the `ELEMENTS` registry's initializer, 768.7 KiB; the largest schema builder Launch anything's, 180.2 KiB.
- `cargo llvm-lines -p opsis-compile` (dev): 2,121,859 lines in 56,763 copies; schemars builders 24.6%, serde deserializers 17.3%, specta definitions 6.9%, serde serializers 4.7%.

### S3. The release app carries and builds every TypeScript type definition

- **Status:** The editor's extra types are built only for the bindings export (`c9d19db1`); the size saved isn't measured.
- `run()` calls `bindings::builder()` (`crates/desktop/src/lib.rs:117-118`), which registers every command, event and frontend type (`crates/desktop/src/bindings.rs`, `.types(&frontend_types())` at `:141`). tauri-specta's `commands()` and `events()` build each definition into a `TypeCollection` there and then (`tauri-specta-2.0.0-rc.26/src/builder.rs:217-253`); only the bindings export reads it.
- The cost is size, not time: the 1.64 MiB of specta code ships in the binary, and building the collection took 4 of the 1,063 app samples of a whole startup (samply at 1 ms: start, validate and open the 438-node project, quit), about 4 ms. The startup's time went to compiling the project (`compile::request` 53.0%, `validate_workflow` 20.3%, `compile_workflow` 18.5%), the API's `/validate` (11.5%) and making the webview (`WebContext::new` 7.1%).

### S4. Crates built in more than one version

- 65 crates in 109 versions in the binary's graph (`cargo tree -d --workspace -e normal --depth 0`). Through whom, for the ones that put a second copy of the same kind of code in the binary: schemars 0.8 and indexmap 1 (tauri-utils), skrifa 0.44 and read-fonts 0.41 (usvg 0.49, harfrust), sha2 0.10 and its digest stack (sqlx), rand 0.9 (nanoid), base64 0.23 (hyper-util), thiserror 1 (the GTK 0.18 crates), hashbrown 0.14 (fundsp), moxcms 0.8 (image, fast_image_resize), brotli-decompressor 5 (wuff), roxmltree 0.20 (fontdb's fontconfig parser). Nearly all come through third-party crates.

### S5. Unused Rust dependencies

- **Status:** Removed in `bfdf0264`.
- `cargo machete`: `opsis-footage` declares `opsis-engine`, `serde` and `serde_json` and uses none (no reference in `crates/footage/src`); `opsis-render` declares `specta` and doesn't use it (its `serde_json` feature comes through `opsis-types` anyway). Its fifth hit, `smart-default` in `opsis-compile`, is used through the `#[settings]` expansion (`crates/macros/src/settings.rs:69`).

### S6. The film page

- **Status:** Launch anything's photos are `app:` placeholders (`0278ae9b`); the page's script went from 4,438.90 kB to 2,722.89 kB. The full Lottie player stays: the light one drops expressions and every layer effect.
- One chunk: 4,438.90 kB, 2,061.05 kB gzipped (Vite: "Some chunks are larger than 500 kB after minification"). By package: `packages/video` 2.50 MB (56%), three 0.72 MB, lottie-web 0.25 MB, react-dom 0.22 MB, @react-three/fiber 0.18 MB, zod 0.10 MB.
- **Launch anything inlines four photos:** `packages/video/src/elements/scene/launch-anything/assets.ts` holds four 1672×941 JPEGs as base64 (desk 234 KiB, horizon 282 KiB, rocket 217 KiB, earth 524 KiB), 1.72 MB of the bundle, used as defaults (`content.ts:58-61`). AGENTS.md §7: "Element defaults use neutral copy and `app:` placeholders (`public/placeholders`, `public/backdrops`)." They are parsed on every film page load, whatever the film shows.
- lottie-web's full SVG player runs Lottie expressions through direct `eval` (`lottie-web/build/player/lottie_svg.js:11559`, Rolldown's EVAL warning), so a Lottie file with expressions runs its own script in the film page.

### S7. The editor

- 3.67 MB of JavaScript in 55 chunks: elkjs 1.47 MB (U2), the editor's own code 0.46 MB, @base-ui/react 0.22 MB, react-dom 0.21 MB, konva 0.19 MB, react-reconciler 0.13 MB, motion-dom 0.12 MB.

### S8. Unused TypeScript (knip)

- **Status:** Done in `c1d893d4` (the test graph helper, React in the film app, knip kept off generated files) and `b0af1e42` to `68af6376` (unused exports and the code only they used, each traced to why it was left); knip reports nothing.
- `apps/frontend/src/test/graph.ts` is imported by nothing.
- `apps/film` declares `react` and `react-dom` and imports neither.
- 305 exports and 145 exported types are used only in their own file, mostly element files: `elements/ui/agent-run.tsx` (27 exports, 33 types), `elements/device/laptop-frame.tsx` (20), `elements/overlay/keystroke.tsx` (16, 15 types), `elements/scene/logo-assemble.tsx` (14), `elements/effect/speed-lines.tsx` (13, 10 types). Spot-checked: `defaultAgentRun`, `agentRunDefaults`, `formatAgentRunSeconds`, `tokenizeAgentRun` appear only in `agent-run.tsx`.

## Tooling notes

- **Status:** The profile runs Sign in in its own place (`10a2e90a`) and its drag card carries no data (`af0cf39f`).

Not app issues; traps for the next profiling run, all handled or recorded in `agent-docs/profiling.md`.

- **T1. heaptrack can't interpret a whole pass of this binary on this machine.** Recording works with `--raw`, but `heaptrack_interpret` on the 871 MB recording read 4.46 MB in 50 minutes at 100% CPU against the line-tables build, and was killed at 10 GiB against a symbols-only copy; it read the startup (E4). M1 is measured instead from the app's memory around each run and with `MALLOC_ARENA_MAX`. Without `--raw`, the app blocks on heaptrack's pipe at startup for minutes.
- **T2. WebKit's remote inspector sends every Timeline and ScriptProfiler timestamp as 0**, even after `Inspector.enable` and `Timeline.enable`. The editor's steps are split by giving each its own ScriptProfiler session, and the sample interval is measured with a busy loop of known length (1.08 ms). The Timeline instrument also slows the page, doubling pan and select.
- **T3. The sampler's own CPU** was 198 s over the full pass, about 11% of one core.
- **T4.** Play tape records a new tape live when it runs on its own with Record it live on; the profile sets `recordLive: false`.

## Local leftovers

- **Status:** `apps/sidecar/` is gone.
- `apps/sidecar/` holds only ignored `.turbo` and `node_modules` folders from the sidecar app removed in `fbd31487`; `apps/*` still matches it as a workspace folder.
