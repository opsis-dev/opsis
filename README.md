# Demoflow

Record a real website and turn it into a polished product demo.

Demoflow is a desktop app with a node editor. Browser nodes drive a real website (clicks, typing, scrolling) and record it in one take; camera, cursor, text, motion-graphics, transition, audio and media nodes turn the recording into a finished video, exported in several shapes at once. Everything runs on your computer.

> **Early preview.** Built and tested on Linux (dev, `.deb`, AppImage). Windows and macOS builds haven't been made or tested yet, and there are no signed installers or auto-updates. Expect rough edges.

Built with [Electron](https://www.electronjs.org), [xyflow](https://reactflow.dev), [Playwright](https://playwright.dev), [Remotion](https://www.remotion.dev) and FFmpeg. See [`plans/`](./plans) for work that's planned but not built.

## What it does

- **Record:** 18 browser actions (go to page, click, type, hover, scroll, choose option, upload file, press keys, wait, check text, fake a response, freeze time, hide clutter, page scripts, chapters, HTML overlays…), sign in once and reuse the login, a live view of the recording browser, and a replay with a step-by-step trace of every take.
- **Camera and cursor:** auto zoom on clicks, manual zooms, pans, follow the cursor, speed up quiet parts, speed ramps, crop, device frames, macOS, Windows and Linux window frames (light or dark, with the page address), studio backdrops, motion blur; cursor styles, click ripples, keystroke chips, spotlights and highlight boxes.
- **Graphics:** 132 elements — 23 built in, 109 curated from the [Remocn](https://remocn.dev), [snapcn](https://snapcn.dev) and [RemotionUI](https://remotionui.com) registries (text effects, overlays, charts, UI scenes, device mockups, backgrounds, effects), plus Lottie animations (`.json` and `.lottie`) and a mosaic effect for hiding private details.
- **Transitions:** 25 (6 standard, 14 Remocn, 5 RemotionUI) plus a light-leak cut.
- **Text and fonts:** 16 bundled font families; a font library you fill from font files or ZIPs, fonts installed on your computer, or free fonts downloaded online (only when you ask). Projects set fonts for text, headings and code, and copy the fonts they use so they open anywhere.
- **Audio:** music with fades and ducking under sound effects, sound effects (7 built-in CC0 sounds or your own files) at a time or on a browser action, any number of sounds under a clip, captions timed from a script.
- **Media:** trim, join, speed, reverse, resize, convert, GIF, extract audio, even out volume, grab a frame, web-ready, file info.
- **Export:** 16:9, 9:16, 1:1 and 4:5 in one go. Other shapes keep the whole window and zoom in closer on the action, with captions and lower thirds kept clear of the edges; the Preview can show any shape. Also still images in social sizes, or a standalone Remotion project.
- **Timeline:** the film as bars with filmstrips, waveforms, action marks, chapters, markers and music beats; move, trim, split, ripple, roll, slip, speed and keyframes, each written back to the nodes (or added as a named node) as one undo step; snapping, J/K/L, in and out points.
- **Mixer:** a strip per sound with trim, low cut, 4-band EQ, compressor, pan, fader, mute and solo, grouped under its bus; buses with low cut, EQ, compressor, presets for the bus or every sound on it, and sidechain ducking that can ignore short sounds; a master with EQ, compressor, limiter and loudness targets (−14, −16, −23 LUFS); meters and loudness readouts; type exact values on any knob; Ctrl-click or Shift-click strip names to apply a preset, paste or reset to several sounds at once. Exports play exactly the rendered mix, and a mix change only adds the new sound to the already rendered picture.
- **Editor:** 202 node types, 7 templates, undo/redo, groups, auto layout, card previews saved as stills, a preview that matches the export frame for frame. Preview, Timeline, Mixer and Run are tabs, and each can open in its own window. The canvas has a mouse mode and a touchpad mode (two fingers move, pinch zooms) and a minimap you can hide; the node list keeps its group titles in view.
- **Projects:** new projects go to `~/DemoflowProjects` (changeable in Settings), and the New project dialog shows the folder it will create. If the project file changes on disk while it's open (a text editor, a script, version control), the editor loads it, or asks first when you have unsaved edits; it never saves over a change it hasn't seen.
- **Settings:** its own window with pages for General, Projects, Engine, Recording, Fonts, Export licence, API, Licences (every licence Demoflow and its fonts come with) and About.
- **Local API (off by default):** scripts and AI agents on your computer can read, change and run the open project over plain HTTP on a local socket (a named pipe on Windows) that only your user account can open. `GET /openapi.json` describes it; `GET /nodes/{type}` gives each node's ports and settings. Every request is one undo step in the editor.

## Getting started

Requirements: [Bun](https://bun.sh) and Node.js 22 or later.

```sh
bun install
bun start
```

The first time, open **Settings → Engine → Set up engine**. It downloads the recording browser (Chrome Headless Shell) and FFmpeg into the app's data folder. Editing and preview work without it; recording and exporting need it. Then **New project → Sample: to-do app → Run**.

Ready-made copies of every template are in [`examples/`](./examples) (**Open project…**).

## Building

| Command                                    | What it does                                                    |
| ------------------------------------------ | --------------------------------------------------------------- |
| `bun run make -- --targets deb,AppImage`   | Linux `.deb` and AppImage (the RPM maker also needs `rpmbuild`) |
| `DEMOFLOW_EDITION=lite bun run make …`     | Leave the engine out; users set it up from Settings             |
| `DEMOFLOW_OUT_DIR=~/.cache/demoflow-build` | Put build output outside the repo                               |

The full edition bundles the engine and saves the recording browser's licence notices next to it. CI ([`.github/workflows/build.yml`](./.github/workflows/build.yml)) runs the checks, then builds both editions on Linux (x64), Windows (x64) and macOS (Apple Silicon and Intel) and uploads the packages as artifacts. On systems that block unprivileged user namespaces (Ubuntu 24.04 and later), the AppImage starts without Chromium's sandbox; the `.deb` keeps it.

## Scripts

| Script                    | What it does                                                            |
| ------------------------- | ----------------------------------------------------------------------- |
| `bun start`               | Run the app in development                                              |
| `bun run typecheck`       | `tsc --noEmit` (TypeScript 6.0.3)                                       |
| `bun run lint:fix`        | oxlint, including the frame-driven animation guardrails                 |
| `bun run format:fix`      | oxfmt                                                                   |
| `bun run test`            | vitest                                                                  |
| `bun run registry:ingest` | Re-install the curated registry elements and regenerate their manifests |
| `bun run fonts:bundle`    | Copy the bundled fonts from their Fontsource packages                   |
| `bun run brand:icons`     | Render `brand/logo.svg` to the app icon PNGs                            |
| `bun run routes`          | Regenerate the TanStack Router route tree                               |
| `bun run studio`          | Remotion Studio on the `DemoTimeline` composition                       |
| `bun run examples`        | Rewrite `examples/*/project.demoflow.json` from the templates           |

## Architecture

```
Renderer (React 19 + shadcn/Base UI + xyflow)  ── preload (window.demoflow) ──  Main (Electron)
   canvas · node list · inspector · <Player>                                     window · menus · protocols
   live compile (graph → TimelineSpec)                                           projects · logins · fonts · stills
                                                                                      │ MessagePort
                                                          Worker (utilityProcess)
                                       engine: validate → topo-sort → sha256 cache → phase lanes → events
                                       capture (Playwright) · media (FFmpeg) · render (@remotion/renderer)
```

- **Nodes never generate Remotion code.** They produce files and compile to a `TimelineSpec`; one composition, `DemoTimeline`, renders any spec. The editor's `<Player>` and the export use the same composition, so the preview matches the export.
- **Browser nodes form a chain** that the Record node replays in one continuous take. Timing comes from the recorded frames' own timestamps, so cursor moves, clicks and camera zooms line up with the picture.
- **Caching:** every node is hashed from its type, settings and upstream hashes; results are cached in `.demoflow/cache/<hash>`, so only changed branches run again.

```
src/
├─ main/       Electron main: window, menus, IPC, protocols, projects, secrets, fonts, card stills, workers
├─ preload/    typed window.demoflow API
├─ engine/     engine paths, installer and licence notices (headless shell + FFmpeg)
├─ traces/     compressed, deduplicated Playwright trace store
├─ shared/     runtime-neutral: schema, nodes, elements, fonts, compile, templates
├─ worker/     utilityProcess: engine, executors, capture (Playwright), media (FFmpeg), render (Remotion)
├─ video/      Remotion root: DemoTimeline, registry, core elements, fonts, camera, registry elements
└─ renderer/   React UI: routes, editor, fonts, store, components (shadcn + React Flow UI)
```

Contributor rules (no code comments, kebab-case files, pinned Remotion and TypeScript versions, what must not regress) are in [AGENTS.md](./AGENTS.md).

## Not built yet

Each has a plan in [`plans/`](./plans):

- Voiceover and captions from speech ([voice-and-captions](./plans/voice-and-captions.md)).
- AI assist: a flow from a sentence, chapter names, headline variants, picker suggestions ([ai-assist](./plans/ai-assist.md)).
- A Drag action, a different transition at each cut, chapters in exported files, and two Run panel fixes ([app-gaps](./plans/app-gaps.md)).
- An app state database in place of the JSON files in the app's data folder ([app-state](./plans/app-state.md)).
- Ported web motion components, Rive animations and element thumbnails ([motion-library](./plans/motion-library.md)) and more Remotion libraries ([remotion-extras](./plans/remotion-extras.md)).
- Windows and macOS builds (never built or run yet), their icons, code signing, updates and node packs ([shipping](./plans/shipping.md)).

## Licences

Demoflow is licensed under the [Apache License 2.0](./LICENSE).

**Remotion is not open source.** Demoflow renders videos with [Remotion](https://www.remotion.dev), which is free for individuals, non-profits and companies with up to 3 employees. Larger companies need a [Remotion company licence](https://www.remotion.dev/license) and must enter its key in **Settings → Video export licence**.

Bundled and vendored third-party code, fonts, sounds, FFmpeg and the recording browser keep their own licences; see [THIRD_PARTY_NOTICES.md](./THIRD_PARTY_NOTICES.md).
