# Opsis

Record a real website and turn it into a polished product demo.

Opsis is a desktop app with a node editor. Browser nodes drive a real website (clicks, typing, scrolling) and record it in one take; camera, cursor, text, motion-graphics, transition, audio and media nodes turn the recording into a finished video, exported in several shapes at once. Everything runs on your computer.

> **Early preview.** Built and tested on Linux (dev and `.deb`). CI is set up to build the rpm, AppImage, Windows and macOS packages, but they haven't been built or run yet, and there are no signed installers or auto-updates. Expect rough edges.

Built with [Tauri](https://tauri.app) and Rust, [xyflow](https://reactflow.dev), React and FFmpeg. See [`plans/`](./plans) for work that's planned but not built.

## What it does

- **Record:** 18 browser actions (go to page, click, type, hover, drag onto an element, by a distance or along a path, scroll, choose option, upload file, press keys, wait, check text, fake a response, freeze time, hide clutter, page scripts, chapters, HTML overlays…), a Sign in step that saves the login once and reuses it in every take, a Start recording step so setup runs off camera, a live view of the recording browser, and a replay with a step-by-step trace of every take.
- **Camera and cursor:** auto zoom on clicks, manual zooms, pans, follow the cursor, speed up quiet parts, speed ramps, crop, laptop and phone frames, macOS, Windows and Linux window frames (light or dark, with the page address), studio backdrops, motion blur; cursor styles (a simple arrow, or a Mac cursor that turns into a hand over links and a text cursor over fields), click ripples, keystroke chips, spotlights and highlight boxes, clean or hand-drawn; blurred areas that hide private details and stay on the page while the camera moves. Cursor, marks, backdrop and frame settings form a style that one take or several share.
- **Graphics:** 138 elements (text effects, overlays, charts, UI scenes, device mockups, backgrounds, effects), plus Lottie animations (`.json` and `.lottie`), a mosaic effect for hiding private details, title cards with a hand-drawn mark on chosen words, and a picture look (colour, dark edges, film grain).
- **3D:** a 3D logo, card and text in any project font, 3D models from your project with their own animations, a clip shown on a 3D laptop, monitor, phone or floating screen that ends facing the camera, and floating shapes as a background; camera keyframes set in the Inspector and dragged on the timeline.
- **Transitions:** 25 plus a light-leak cut. A Sequence can use a different transition, length and hold before any of its clips.
- **Text and fonts:** 16 bundled font families; a font library you fill from font files or ZIPs, fonts installed on your computer, or free fonts downloaded online (only when you ask). Projects set fonts for text, headings and code, and copy the fonts they use so they open anywhere.
- **Audio:** music with fades and ducking under sound effects (3 built-in CC0 songs or your own files), music fitted to the film (stretched a little, or looped with a crossfade), sound effects (7 built-in CC0 sounds or your own files) at a time or on a browser action, click and key sounds at every action of a take, a whoosh at every transition, a riser that ends on an action, a marker or the clip's end, generated sounds (whoosh, riser, pop, ding, tap, swell, click) with their variations, tones and countdown beeps, room tone, any number of sounds under a clip, captions timed from a script.
- **Media:** trim, join, speed (up to 64×), reverse, resize, convert, GIF, extract audio, even out volume, grab a frame, web-ready; for sound: clean speech (noise, rumble, harsh "s" sounds, level), fix clicks and clipping, remove hum, shorten pauses, change pitch, voice effects (telephone, radio, rooms, echo) and a picture of the waveform or spectrum. Media file nodes read their file without a run, and nodes warn before a run when they get the wrong kind of file.
- **Images:** resize, crop (an area you draw or margins), rotate and flip, straighten, tilt in 3D, adjust colour, auto levels, blur and sharpen, place a logo or watermark, redact, spotlight, trim empty edges, make a colour transparent, outline, a grid of images, a framed screenshot on a backdrop, before and after, magnify, compare two screenshots, annotate with arrows, boxes, ovals, numbered steps and text, and save as PNG, JPEG, WebP or AVIF. Phone photos come out the right way up and in the right colours.
- **Export:** 16:9, 9:16, 1:1 and 4:5 in one go. Other shapes keep the whole window and zoom in closer on the action, with captions and lower thirds kept clear of the edges. Video files carry the film's chapters (from Chapter actions) for players that list them, with an optional chapter list to paste into a YouTube description. Also still images in social sizes.
- **Timeline:** the film as bars with filmstrips, waveforms, action marks, chapters, markers and music beats; move, trim, split, ripple, roll, slip, speed and keyframes, each written back to the nodes (or added as a named node) as one undo step; snapping, J/K/L, in and out points.
- **Mixer:** a strip per sound with trim, low cut, 4-band EQ, compressor, pan, fader, mute and solo, grouped under its bus; buses with low cut, EQ, compressor, presets for the bus or every sound on it, and sidechain ducking that can ignore short sounds; a master with EQ, compressor, limiter and loudness targets (−14, −16, −23 LUFS); meters and loudness readouts; type exact values on any knob; Ctrl-click or Shift-click strip names to apply a preset, paste or reset to several sounds at once. Exports play exactly the rendered mix, and a mix change only adds the new sound to the already rendered picture.
- **Editor:** 252 node types, 7 templates, undo/redo, groups, auto layout. Timeline, Mixer and Run are tabs, and each can open in its own window; the Timeline plays the film's sound on its own clock and marks the nodes and clips under its playhead. The canvas has a mouse mode and a touchpad mode (two fingers move, pinch zooms) and a minimap you can hide; the node list keeps its group titles in view.
- **Projects:** new projects go to `~/OpsisProjects` (changeable in Settings), and the New project dialog shows the folder it will create. If the project file changes on disk while it's open (a text editor, a script, version control), the editor loads it, or asks first when you have unsaved edits; it never saves over a change it hasn't seen.
- **Settings:** its own window with pages for General, Projects, Engine, Recording, Fonts, Export licence, API, Licences (every licence Opsis and its fonts come with) and About.
- **Local API (off by default):** scripts and AI agents on your computer can read, change and run the open project over plain HTTP on a local socket (a named pipe on Windows) that only your user account can open. The socket is `api.sock` in the app's data folder; set `OPSIS_API_SOCKET` to another full path (at most 100 bytes) if that path is too long. `GET /openapi.json` describes it; `GET /nodes/{type}` gives each node's ports and settings. Every request is one undo step in the editor.

## Getting started

Requirements: [Bun](https://bun.sh), Node.js 22 or later, a Rust toolchain, and on Linux the WebKitGTK and GTK development packages (on Ubuntu 22.04 or later: `libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev`, plus `gstreamer1.0-libav` to play videos in the editor).

```sh
bun install
bun run film:bundle
bun start
```

`bun run film:bundle` builds the film page that exports draw; run it again after changing `packages/video` or `packages/runtime`. The first time, open **Settings → Engine** and either **Download engine**, which puts the recording browser (Chrome Headless Shell) and FFmpeg in the app's data folder, or **Choose file…** for programs you already have. Opsis checks each one before using it, and again at start if the file changed. Editing works without it; recording and exporting need it. Then **New project → Sample: to-do app → Run**.

Ready-made copies of every template are in [`examples/`](./examples) (**Open project…**).

## Building

| Command                                 | What it does                                              |
| --------------------------------------- | --------------------------------------------------------- |
| `bun run make -- --bundles deb`         | Stage everything the app ships and build the Linux `.deb` |
| `CARGO_TARGET_DIR=~/.cache/opsis-build` | Put build output outside the repo                         |
| `CARGO_BUILD_JOBS=8`                    | Limit how many cores the Rust build uses                  |

The app never includes the recording browser or our FFmpeg; users download them or choose their own from Settings. CI ([`.github/workflows/build.yml`](./.github/workflows/build.yml)) runs the checks and the end-to-end tests, then builds deb, rpm and AppImage on Linux (x64), NSIS and MSI on Windows (x64) and the app and a DMG on macOS (Apple Silicon and Intel), and uploads them as artifacts.

## Scripts

| Script                 | What it does                                                               |
| ---------------------- | -------------------------------------------------------------------------- |
| `bun start`            | Run the app in development                                                 |
| `bun run typecheck`    | `tsc --noEmit` everywhere                                                  |
| `bun run lint:fix`     | oxlint and `cargo clippy`                                                  |
| `bun run format:fix`   | oxfmt and `cargo fmt`                                                      |
| `bun run test`         | vitest and `cargo test`                                                    |
| `bun run test:e2e`     | The app driven by WebdriverIO in a virtual display (needs `xvfb`)          |
| `bun run types:export` | Write the TypeScript types and bindings from the Rust code                 |
| `bun run film:bundle`  | Build the film page that exports draw                                      |
| `bun run fonts:bundle` | Copy the bundled fonts from their Fontsource packages                      |
| `bun run brand:icons`  | Render `brand/logo.svg` to PNGs; `bun run app:icons` makes the app's icons |
| `bun run routes`       | Regenerate the TanStack Router route tree                                  |
| `bun run examples`     | Rewrite `examples/*/project.opsis.json` from the templates                 |

## Architecture

```
Editor (React 19 + shadcn/Base UI + xyflow, in the system webview)
   canvas · node list · inspector · timeline · mixer        ── commands and events (bindings.ts) ──
                                                                                  │
Tauri app (Rust): windows · menus · projects · app database · fonts · Local API
   compile (graph → TimelineSpec) · runner (topo-sort → sha256 cache → phases → events)
   capture over CDP · media (FFmpeg) · prepared 3D files · image nodes · streamed views
        │ CDP
   Chrome Headless Shell: recording, the film page frame by frame, sign-in, element picker
```

- **Nodes never generate film code.** They produce files and compile to a `TimelineSpec`; the film page's one `DemoTimeline` draws any spec, and Rust captures it frame by frame into FFmpeg.
- **Nodes pass resources forward:** a browser session (Open browser and its steps), a recording, a recording style, clips, files, images, sounds and captions. Browser nodes add their steps to the session, and the Record node replays them in one continuous take. Timing comes from the recorded frames' own timestamps, so cursor moves, clicks and camera zooms line up with the picture.
- **Caching:** every node is hashed from its type, settings and upstream hashes; results are cached in `.opsis/cache/<hash>`, so only changed branches run again.

```
crates/      Rust: types, compile, store, project, fonts, engine, traces, runner, media, render,
             capture, stream, prepare, image, api, desktop (the Tauri app)
apps/frontend the editor
apps/film    the film page the renderer draws
e2e/         end-to-end tests (outside the workspace, own install)
packages/video  the film: DemoTimeline, elements, parts, lib, fonts, camera, 3D
packages/runtime the film's page runtime: clock, sequences, holds, maths
scripts/     fonts, icons, engine downloads, packaging stage
```

Contributor rules (no code comments, kebab-case files, pinned versions, what must not regress) are in [AGENTS.md](./AGENTS.md).

## Why it's built this way

| Decision          | Choice                                                                                                      | Why                                                                                                                                 |
| ----------------- | ----------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| App shape         | One Tauri 2 app: Rust for everything that touches the system, the editor in the system webview              | A small install, a typed boundary, and one place (Rust) that owns the project, runs, settings and state that windows share.         |
| Types and logic   | Rust is the source; TypeScript types are generated (specta, tauri-specta)                                   | Commands, events and the timeline spec can't drift apart, and the editor keeps no copy of logic Rust already has.                   |
| Canvas            | [xyflow](https://reactflow.dev) with React Flow UI's shadcn components                                      | A mature node editor whose components install like the rest of the UI.                                                              |
| Workflow format   | A versioned JSON graph, compiled in Rust to one `TimelineSpec`                                              | The graph is for people; the spec is the single input the video renders from.                                                       |
| Video engine      | React with Opsis's own small runtime, drawn frame by frame by Rust                                          | One page, `DemoTimeline`, renders every spec; Rust seeks it, captures each frame and encodes with FFmpeg, with no Node at run time. |
| Browser           | One Chrome Headless Shell for recording, renders, sign-in and the element picker, driven over CDP from Rust | The same engine everywhere; Playwright's page scripts and keyboard are reused without its Node runtime.                             |
| Animations        | Frame-driven only: every element and transition                                                             | Every frame has to be a pure function of the frame number; CSS transitions and tween libraries don't survive a render.              |
| GSAP              | Not used                                                                                                    | The runtime's `interpolate`, `spring` and easing cover it, without GSAP's licence clause on visual builders.                        |
| FFmpeg            | The ffmpeg-static build, downloaded at setup and checked by sha256, never shipped                           | Users get a known build without the app carrying it.                                                                                |
| Camera and cursor | Built on [programatic-demo](https://github.com/ashrafchowdury/programatic-demo) (MIT)                       | It already solved the click log, auto-zoom camera, vector cursor and motion blur; its notice is kept.                               |
| Offline           | Nothing uses the internet unless you ask                                                                    | Engine and font downloads sit behind a button, and a film never loads anything remote.                                              |
| Licensing         | GPLv3 or later                                                                                              | Free software, with no other licence needed to export.                                                                              |

## Not built yet

Each has a plan in [`plans/`](./plans):

- Voiceover and captions from speech ([voice-and-captions](./plans/voice-and-captions.md)).
- AI assist: a flow from a sentence, chapter names, headline variants, picker suggestions ([ai-assist](./plans/ai-assist.md)).
- Ported web motion components, Rive animations and element thumbnails ([motion-library](./plans/motion-library.md)).
- Size work, Windows and macOS builds (set up in CI, never run yet), code signing, updates, the release test pass, profiling and node packs ([release](./plans/release.md)).

## Licences

Opsis is free software under the [GNU General Public License version 3 or later](./LICENSE). Videos you make with Opsis are yours: the licence covers the program, not what it makes.

Bundled and vendored third-party code, fonts, sounds, FFmpeg and the recording browser keep their own licences; see [THIRD_PARTY_NOTICES.md](./THIRD_PARTY_NOTICES.md).
