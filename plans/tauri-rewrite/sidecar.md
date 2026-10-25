# Sidecar

A Bun process whose only job is Remotion: `selectComposition`, `renderMedia` and `renderStill` on the prebuilt bundle with the engine's headless shell. Rust starts it, sends render jobs as JSON lines on stdin, and reads replies on stdout.

## What it adds

- Remotion keeps working unchanged while everything around it moves to Rust.
- The sidecar imports nothing from the other packages: it gets a spec as JSON and passes it to Remotion as `inputProps`.

## Fit with Opsis

Already in place:

- `src/worker/render/render.ts` (162 lines): `getServeUrl` (the prebuilt bundle in a packaged app), `compositionFor` (`selectComposition` with `inputProps`, `binariesDirectory`, `browserExecutable`, `chromiumOptions.gl`), `renderTimeline` (`renderMedia`, `concurrency: 1`), `renderFrame` (`renderStill`), `openRenderBrowser`, cancellation through `makeCancelSignal`, the licence key (`remotionLicenseKey`).
- `bundle.ts`, `webpack-override.ts`, `source-alias.ts`: the build-time bundle; `compositor-package.ts`: the compositor package name per platform.

### Tested: Remotion under Bun (2026-10-25)

The bundle was built once with Node (`bundleRemotionProject`). The same script then rendered with `tsx` on Node 26.12 and with `bun --bun run` on Bun 1.4.3, Remotion 4.0.537, the dev engine's headless shell, `gl: "angle"`:

| Job                                         | Node             | Bun              |
| ------------------------------------------- | ---------------- | ---------------- |
| Still, `core/kinetic-headline`              | 4.3 s            | 4.4 s            |
| Still, `core/text-3d`                       | 2.1 s            | 1.7 s            |
| Still, `core/shapes-3d`                     | 1.8 s            | 1.6 s            |
| Still, `remocn/shader-grain-gradient`       | 1.5 s            | 1.6 s            |
| Still, `remocn/grain-dissolve` (transition) | 1.8 s            | 1.5 s            |
| H.264, `core/kinetic-headline`              | 8.4 s            | 6.9 s            |
| H.264, `remocn/shader-grain-gradient`       | 11.5 s           | 10.0 s           |
| Render cancelled after 1.5 s                | stopped at 1.5 s | stopped at 1.5 s |
| Whole script                                | 39.2 s           | 34.4 s           |

- All five stills and both videos were **byte-identical** between Node and Bun (`cmp`).
- Bun exited by itself without `process.exit`.
- Remotion's Bun page still describes Bun 1.0.24 with Remotion 4.0.88 ("mostly supported"). Its two known issues don't apply: `lazyComponent` (Opsis loads elements through its own `import()` loaders) and render scripts that may not quit (the sidecar is long-lived, and it did quit).
- `bun run file.ts` runs on Bun's runtime; `--bun` only matters for package bins whose shebang names `node`.

### Tested: stdout kept for messages (2026-10-25)

Remotion writes its logs, and the browser's console, to stdout through `console.log` and `process.stdout.write` (`@remotion/renderer` `dist/esm/index.mjs`, its `Log` functions). A test sidecar kept the real stdout writer for messages and pointed `console.log`, `console.info`, `console.debug` and `process.stdout.write` at stderr, read one request from stdin, ran the whole render test above and wrote one reply. **stdout held only the reply line**; the other 15 lines (the test's own log lines plus the browser console lines Remotion forwards) went to stderr.

Missing:

- The message loop, the Rust side, and shipping Bun.

## Work

1. **Package.** `packages/sidecar` with `render.ts`, the bundle build files and the compositor lookup. Nothing from `packages/ui` or `packages/video` is imported at run time; `CODECS` stays in Rust, which sends the resolved `codec`, `crf`, `scale` and `muted`.
2. **Messages.** One JSON object per line.
   - Requests from Rust: `{ id, method, params }` with `render.media` (`spec`, `output`, `codec`, `crf`, `scale`, `muted`, `gl`), `render.still` (`spec`, `frame`, `output`, `format`, `scale`, `jpegQuality`, `gl`), `cancel` (`target`), `ping`.
   - Replies: `{ id, result }` or `{ id, error: { message } }`; progress `{ method: "progress", params: { id, progress } }`.
   - Types are Rust structs exported to TS (`types-and-contracts.md`).
3. **stdout discipline.** At start, keep `process.stdout.write` for messages and redirect `console.*` and `process.stdout.write` to stderr (the tested setup). Exit when stdin closes.
4. **Rust side.** Start the sidecar with `tokio::process` (or `tauri-plugin-shell` 2.5.0's `sidecar()`, which gives stdout events and `child.write`), with `env` for the temp folder and the licence key. A reader task matches replies to waiting requests by id (a map of `oneshot` senders, as `pending-replies.ts` does today); stderr lines go to the Run log; a crash fails the jobs in flight and the next job starts a new sidecar.
5. **Shipping.** Tauri's `bundle.externalBin` names the base path; a file with the target triple suffix must exist for each platform (for example `binaries/opsis-render-x86_64-unknown-linux-gnu`). Measure two options on Linux, then on Windows and macOS in CI:
   - `bun build --compile` of the sidecar into one executable, with the compositor package next to it and passed as `binariesDirectory`;
   - the Bun binary (89.0 MB) plus the sidecar's JS and `node_modules/@remotion/renderer` and its compositor.

   Keep whichever renders the test set above byte-identically and is smaller.

6. **Bundle.** Still built at build time by `@remotion/bundler` (webpack), as `packageAfterCopy` does today; it moves to a `video#build` turbo task. The headless shell loads it for renders and for the streamed preview (`streamed-views.md`).
7. **Assets.** The asset server that serves `project:` files to render tabs (`asset-server.ts`) moves to Rust, which also serves the preview page; the sidecar gets its address in each job.
8. **AGENTS.md.** "Nothing ever runs under Bun's runtime" becomes "only the sidecar runs on Bun" (`agents-md.md`).
9. **Tests.** The render test set above as an integration test in CI (needs the engine): outputs compared by hash against the Node baseline once, then against the previous run.

## Risks

- **Bun releases.** Pin the Bun version used to build the sidecar and rerun the byte-identical test on every bump.
- **Compiled executables.** `bun build --compile` hasn't been tried with `@remotion/renderer`, which finds its compositor and spawns it; Work step 5 measures it, and the plain Bun binary is the fallback.
- **Remotion's licence** still applies to renders (a company licence above three employees); Settings keeps the key.

## Sources

- Render and stdout tests in this session (scripts `render-test.mts`, `stdout-guard.mts`), outputs in `~/.cache/opsis-bun-test` (2026-10-25).
- [Remotion with Bun](https://www.remotion.dev/docs/bun), [Rendering using SSR APIs](https://remotion.dev/docs/ssr-node) (2026-10-25).
- [Tauri: embedding external binaries (sidecar)](https://v2.tauri.app/develop/sidecar/) (2026-10-25).
