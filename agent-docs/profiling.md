# Profiling

How Opsis is measured for size, CPU and memory with development tools only: the profiling build, the Docker run that profiles every node, and the size tools. Paths are relative to the repository root. The release build is in [packaging.md](packaging.md), the recording browser and FFmpeg in [engine.md](engine.md), runs and cache keys in [runs.md](runs.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                         | Where                                                                            |
| ------------------------------- | -------------------------------------------------------------------------------- |
| Host side of a run              | `scripts/profile.ts` (`bun run profile`)                                         |
| Container side of a run         | `scripts/profile-inside.ts`                                                      |
| Profile project and run order   | `scripts/lib/profile/project.ts` → `profileWorkflow`, `missingTypes`, `runOrder` |
| Generated inputs                | `scripts/lib/profile/inputs.ts` → `writeInputs`                                  |
| Local test site                 | `scripts/lib/profile/site.ts` → `serveSite`                                      |
| `/proc` sampling and accounting | `scripts/lib/profile/procfs.ts` → `snapshot`, `cpuByTag`, `memoryByTag`, `tagOf` |
| Local API client and run events | `scripts/lib/profile/api.ts` → `apiClient`, `eventsIn`                           |
| Report                          | `scripts/lib/profile/report.ts` → `nodeCosts`, `overlapShare`, `writeReport`     |
| Image                           | `scripts/lib/profile/Dockerfile`                                                 |
| Data folder seed                | `scripts/lib/user-data.ts` → `seedUserData` (shared with the `check-app` skill)  |
| Unused files and exports        | `knip.config.ts` (`bun run knip`)                                                |

## Rules

- **Profiling is development tooling: build settings, scripts and containers, never code in the app.**
- **The profiling build is the deb with its symbols kept**, from the shipped profile:

  ```sh
  CARGO_BUILD_JOBS=8 CARGO_TARGET_DIR=$HOME/.cache/opsis-build CARGO_PROFILE_RELEASE_STRIP=false \
  CARGO_PROFILE_RELEASE_DEBUG=line-tables-only bun run make -- --bundles deb
  ```

  - `line-tables-only` keeps function names and lines for samply, perf and heaptrack. Full debug info runs the full-LTO link out of memory on a 14 GiB machine.
  - `bun run profile` takes the one deb in `~/.cache/opsis-build/release/bundle/deb`, or `--deb <file>`.

- **`bun run profile` runs the app in Docker, never on the host**: Ubuntu 26.04 with the deb installed (`apt` resolves its libraries), fixed cores and memory (`--cpus`, default `0-3`; `--memory`, default `6g`, swap equal to it), `--network none`, and the user's uid.
  - samply and perf carry `cap_perfmon,cap_ipc_lock` as file capabilities and the container gets `--cap-add PERFMON --cap-add IPC_LOCK`, so they record whatever the host's `kernel.perf_event_paranoid` is. Without the added capabilities the kernel refuses to start them.
  - perf ships as `linux-perf` in Ubuntu 26.04; samply comes from its release, pinned by version and sha256 in `scripts/profile.ts`.
  - The repository, the dev engine (`~/.config/Opsis/engine`, read-only) and the run folder are mounted at their host paths, and the host's `bun` runs the container side.
- **`--profiler samply|perf|heaptrack` starts the app under that tool**, writing `samply.json.gz` (with `.syms.json` beside it), `perf.data` (DWARF stacks; Rust keeps no frame pointers) or `heaptrack.raw.gz` to the run's `out/`. heaptrack follows only the app's own process.
  - heaptrack records with `--raw`: interpreting as it records makes the app wait on heaptrack's pipe for minutes at startup while it reads the line tables.
  - Interpreting a whole pass afterwards (`heaptrack_interpret`) doesn't finish on a 14 GiB machine: against the line-tables build it reads about 90 KB a minute, against a symbols-only copy (`objcopy --strip-debug`, mounted over `/usr/bin/opsis-desktop`) it runs out of memory. Record short passes, or use the app's `/proc` files below.
- **`--gpu` passes `/dev/dri` into the container with its device groups, and `--render-gl <choice>` sets every Render node's `gl`**; only `vulkan` reaches the GPU from the container (see Deliberate limits).
- **`--app-env NAME=VALUE` (repeatable) sets a variable for the app**, such as `MALLOC_ARENA_MAX`.
- **When the pass ends, the app's `/proc/<pid>/status` and `smaps_rollup` are saved as `app-status.txt` and `app-smaps_rollup.txt`**: resident, anonymous and file-backed memory and the peak.
- **The image carries GNU `time`, for `/usr/bin/time -v` on the standalone programs** (`render-elements`, `editor-data`), run in the container with the repository mounted at its own path.
- **The profile project holds every node type in the catalogue, and `project.test.ts` fails when one is missing**, when an edge joins ports that don't connect, or when a setting that may not be blank is blank.
  - It is built from `editor-data catalog` with each node's defaults; a table in `project.ts` wires each type and fills the settings it needs.
  - Inputs are generated (a test clip and pictures from the staged FFmpeg, a look file, a Lottie file) or copied from fixtures (a cast, a VHS tape, a Draco model), and the browser nodes record the site `serveSite` serves on 127.0.0.1 inside the container.
  - Every output node saves under its own node id; outputs with the same file name are an error.
- **Each working node is measured alone**: one `POST /project/run` with `targetNodeIds: [id]` per working node in dependency order. The runner forces the target and takes everything upstream from the cache, so each run does one node's work (`needed_nodes`, `forced_nodes` in `crates/runner/src/runner.rs`). Compile-only nodes are measured by the Render of their own film.
- **`--only <regex>` picks runs by `"<type> <id>"`**; the upstream work of a picked run runs inside it, because the cache starts empty.
- **Every 250 ms (`--sample-ms`) the container side reads `/proc` for every process in the container**: CPU from `stat` (own and reaped children), RSS and PSS from `smaps_rollup`, a tag from the command line (`tagOf`).
  - CPU of a process that exits is counted up to its last sample; what the reapers' counters gain beyond that is `exited between samples`, so the tags add up to the container's CPU (the cgroup's `cpu.stat`, written to `cgroup.json`).
  - A run's CPU is each sample's CPU times the share of the sample inside the run (`overlapShare`); peak memory is the largest sample inside it.
  - `programs.json` lists the programs behind each tag.
- **Each run writes `runs.json`, `events.jsonl` (every run event), `samples.jsonl`, `nodes.csv`, `samples.csv`, `report.html`, `problems.json`, `app.log` and `cgroup.json` to `~/.cache/opsis-profile/runs/<time>-<profiler>/out/`.**
- **Size has its own tools, none of them in the app**:
  - the binary by crate: `cargo bloat --release -p opsis-desktop --features tauri/custom-protocol --crates -n 40`, then `--filter <crate>`, with the same `CARGO_TARGET_DIR` and profile variables as the deb (it links `opsis-desktop` once more, because the Tauri CLI passes its config through `TAURI_CONFIG`);
  - generic code: `cargo llvm-lines -p <crate>` on the dev profile;
  - duplicates and unused crates: `cargo tree -d --workspace -e normal --depth 0` and `cargo machete` (it misses crates used only through a macro's expansion, like `smart-default` behind `#[settings]`);
  - the pages by package: `vite build --sourcemap --outDir <folder in ~/.cache>` in `apps/film` or `apps/frontend`, then `bunx source-map-explorer '<folder>/assets/*.js' --json`. Building into a cache folder keeps source maps out of `film-bundle/` and `apps/frontend/dist`, which the deb packs;
  - unused TS files, exports and dependencies: `bun run knip`.

## How it works

- **A run on the host builds the catalogue, the inputs, the project and a seeded data folder, builds the image, runs the container and writes the report from what the container left.**
- **The container side starts Xvfb, a D-Bus session and an unlocked keyring, serves the site, starts the app (under the profiler if one is asked for) and waits for the API socket**, then validates the project with `/validate`, opens it, follows `/project/run/events` and runs the targets.
- **A run that doesn't finish within `--timeout-s` (default 900) is stopped; if the app still reports a run in progress a minute later, the pass ends there and the report covers what ran.**
- **Chrome writes its process title over its command line as one string**, and its helpers start as `/proc/self/exe --type=…`; `argsOf` and `tagOf` split and tag both.

## Deliberate limits

- **In the container a Render's default "Graphics card" WebGL draws in software, with or without `--gpu`**: ANGLE's default backend goes through the X display, and Xvfb has no DRI3. It is SwiftShader without Mesa in the image and Mesa's llvmpipe with it. Shader, 3D and particle elements then take several times longer than on a desktop, where the same setting reaches the GPU; `--gpu --render-gl vulkan` measures them on the GPU.
- **Play tape runs from the tape file (`recordLive: false`)**: with Record it live on and the node run on its own, it waits for a person to type a new tape.
- **Sign in runs last (`RUN_LAST`), on a session nothing else reads.**
- **The editor's own work (canvas, Inspector, Timeline) isn't sampled per node**; it is read on the e2e build (`bun run e2e:app`, debug Rust with the production editor embedded) in the same image, with `WEBKIT_INSPECTOR_HTTP_SERVER=127.0.0.1:9222` and the build's WebDriver on 127.0.0.1:4445 (`TAURI_WEBDRIVER_PORT`).
  - The inspector's index page lists the target; its socket is `ws://127.0.0.1:9222/socket/1/1/WebPage`, speaking the WebKit Inspector Protocol through `Target.sendMessageToTarget`. `Runtime.evaluate` doesn't wait for promises; `Runtime.awaitPromise` does.
  - The inspector sends every Timeline and ScriptProfiler timestamp as 0, so each step gets a ScriptProfiler session of its own (`startTracking`/`stopTracking`, samples in `trackingComplete`), and a busy loop of known length gives the sample interval (1.08 ms).
  - The Timeline instrument slows the page (it doubles pan and select), so time steps without it.
  - Input goes through the WebDriver (clicks, pointer drags; it has no wheel), which dispatches its events from script like the e2e specs do.
