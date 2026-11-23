import { type ChildProcess, execFileSync, spawn } from "node:child_process";
import {
  appendFileSync,
  existsSync,
  mkdirSync,
  openSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { type ApiClient, apiClient, type RunEvent } from "./lib/profile/api";
import { type InsideConfig, type Profiler, readConfig } from "./lib/profile/config";
import { cpuByTag, memoryByTag, type Snapshot, snapshot } from "./lib/profile/procfs";
import { serveSite } from "./lib/profile/site";

const DISPLAY = ":99";
const SOCKET_WAIT_MS = 120_000;
const SETTLE_MS = 2000;

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function started(
  command: string,
  args: string[],
  log: string,
  env: NodeJS.ProcessEnv,
): ChildProcess {
  const output = openSync(log, "a");
  return spawn(command, args, { env, stdio: ["ignore", output, output] });
}

function profilerPrefix(profiler: Profiler, out: string): string[] {
  if (profiler === "samply")
    return [
      "samply",
      "record",
      "--save-only",
      "--unstable-presymbolicate",
      "-o",
      path.join(out, "samply.json.gz"),
      "--",
    ];
  if (profiler === "perf")
    return [
      "perf",
      "record",
      "--call-graph",
      "dwarf,16384",
      "-F",
      "199",
      "-o",
      path.join(out, "perf.data"),
      "--",
    ];
  if (profiler === "heaptrack") return ["heaptrack", "--raw", "-o", path.join(out, "heaptrack")];
  return [];
}

async function desktopSession(config: InsideConfig): Promise<NodeJS.ProcessEnv> {
  const runtime = "/tmp/runtime";
  mkdirSync(runtime, { recursive: true, mode: 0o700 });
  const env: NodeJS.ProcessEnv = { ...process.env, DISPLAY, XDG_RUNTIME_DIR: runtime };
  delete env.WAYLAND_DISPLAY;
  started(
    "Xvfb",
    [DISPLAY, "-screen", "0", "1920x1080x24", "-nolisten", "tcp"],
    path.join(config.out, "xvfb.log"),
    env,
  );
  env.DBUS_SESSION_BUS_ADDRESS = execFileSync(
    "dbus-daemon",
    ["--session", "--fork", "--print-address=1"],
    { env, encoding: "utf8" },
  ).trim();
  const keyring = execFileSync("gnome-keyring-daemon", ["--unlock", "--components=secrets"], {
    env,
    input: "\n",
    encoding: "utf8",
  });
  for (const line of keyring.split("\n")) {
    const [name, ...value] = line.split("=");
    if (name && value.length > 0) env[name] = value.join("=");
  }
  await sleep(SETTLE_MS);
  return env;
}

async function waitForSocket(socket: string, app: ChildProcess): Promise<void> {
  const deadline = Date.now() + SOCKET_WAIT_MS;
  while (!existsSync(socket)) {
    if (app.exitCode !== null)
      throw new Error(`The app exited with ${app.exitCode} before its API started.`);
    if (Date.now() > deadline) throw new Error("The app's API didn't start in time.");
    await sleep(250);
  }
}

type RunRecord = {
  ran: Set<string>;
  cached: Set<string>;
  errors: string[];
  done: RunEvent | null;
  notify: (() => void) | null;
};

const STOP_WAIT_MS = 60_000;

async function stopRun(api: ApiClient): Promise<boolean> {
  await api.post("/project/run/stop");
  const deadline = Date.now() + STOP_WAIT_MS;
  while (Date.now() < deadline) {
    const state = (await api.get("/project/run")) as { running: boolean };
    if (!state.running) return true;
    await sleep(500);
  }
  return false;
}

function appProcess(): number | null {
  const found = snapshot().processes.find((process) => process.tag === "app");
  return found ? found.pid : null;
}

type NodeRun = {
  id: string;
  type: string;
  runId: string | null;
  startedAt: number;
  endedAt: number;
  ok: boolean;
  error: string | null;
  ran: string[];
  cached: string[];
};

function sampler(config: InsideConfig) {
  const file = path.join(config.out, "samples.jsonl");
  writeFileSync(file, "");
  let previous: Snapshot = snapshot();
  const state = { stopped: false };
  const programs = new Map<string, Set<string>>();
  async function loop(): Promise<void> {
    while (!state.stopped) {
      await sleep(config.sampleMs);
      const current = snapshot(previous);
      const cpu = Object.fromEntries(cpuByTag(previous, current));
      const memory = Object.fromEntries(memoryByTag(current));
      for (const process of current.processes)
        programs.set(process.tag, (programs.get(process.tag) ?? new Set()).add(process.program));
      appendFileSync(
        file,
        `${JSON.stringify({ from: previous.at, at: current.at, cpu, memory })}\n`,
      );
      previous = current;
    }
  }
  const running = loop();
  return async () => {
    state.stopped = true;
    await running;
    const listed = Object.fromEntries(
      [...programs].map(([tag, names]) => [tag, [...names].sort()]),
    );
    writeFileSync(path.join(config.out, "programs.json"), JSON.stringify(listed, null, 2));
  };
}

function cgroupTotals(): Record<string, string> {
  const totals: Record<string, string> = {};
  for (const name of ["memory.peak", "cpu.stat", "memory.events"]) {
    const file = path.join("/sys/fs/cgroup", name);
    if (existsSync(file)) totals[name] = readFileSync(file, "utf8").trim();
  }
  return totals;
}

async function main(): Promise<void> {
  const config = readConfig(process.argv[2] ?? "");
  mkdirSync(config.out, { recursive: true });
  const socket = path.join(config.userData, "api.sock");
  const env = await desktopSession(config);
  const site = await serveSite(config.sitePort);
  const appEnv = {
    ...env,
    OPSIS_USER_DATA: config.userData,
    QT_QPA_PLATFORM: "xcb",
    RUST_BACKTRACE: "1",
    ...config.appEnv,
  };
  const command = [...profilerPrefix(config.profiler, config.out), config.app];
  const stopSampling = sampler(config);
  const app = started(command[0], command.slice(1), path.join(config.out, "app.log"), appEnv);
  const runs: NodeRun[] = [];
  try {
    await waitForSocket(socket, app);
    const api = apiClient(socket);
    const workflow = JSON.parse(
      readFileSync(path.join(config.project, "project.opsis.json"), "utf8"),
    );
    const checked = (await api.post("/validate", workflow)) as { ok: boolean; problems: unknown[] };
    writeFileSync(
      path.join(config.out, "problems.json"),
      JSON.stringify(checked.problems, null, 2),
    );
    if (!checked.ok)
      throw new Error(`The profile project has problems; see ${config.out}/problems.json.`);
    await api.post("/open", { dir: config.project });
    await sleep(SETTLE_MS);
    const events = path.join(config.out, "events.jsonl");
    writeFileSync(events, "");
    const seen = new Map<string, RunRecord>();
    function record(runId: string): RunRecord {
      const found = seen.get(runId);
      if (found) return found;
      const fresh: RunRecord = {
        ran: new Set(),
        cached: new Set(),
        errors: [],
        done: null,
        notify: null,
      };
      seen.set(runId, fresh);
      return fresh;
    }
    const stopEvents = await api.followEvents((event, at) => {
      appendFileSync(events, `${JSON.stringify({ ...event, receivedAt: at })}\n`);
      if (!event.runId) return;
      const run = record(event.runId);
      if (event.type === "node:done" && event.nodeId)
        (event.cached ? run.cached : run.ran).add(event.nodeId);
      if (event.type === "node:error" && event.nodeId)
        run.errors.push(`${event.nodeId}: ${event.message}`);
      if (event.type === "run:done") {
        run.done = event;
        run.notify?.();
      }
    });
    for (const target of config.targets) {
      const startedAt = Date.now();
      const state = (await api.post("/project/run", { targetNodeIds: [target.id] })) as {
        runId?: string | null;
      };
      const runId = state.runId ?? "";
      const run = record(runId);
      const finished = await new Promise<RunEvent | null>((resolve) => {
        const timer = setTimeout(() => resolve(null), config.nodeTimeoutMs);
        function settle(): void {
          clearTimeout(timer);
          resolve(run.done);
        }
        if (run.done) settle();
        else run.notify = settle;
      });
      const stopped = finished ? true : await stopRun(api);
      const timedOut = stopped ? "timed out" : "timed out, and the run didn't stop when asked";
      runs.push({
        id: target.id,
        type: target.type,
        runId: runId || null,
        startedAt,
        endedAt: Date.now(),
        ok: finished?.ok === true,
        error: finished ? run.errors.join("\n") || null : timedOut,
        ran: [...run.ran],
        cached: [...run.cached],
      });
      writeFileSync(path.join(config.out, "runs.json"), JSON.stringify(runs, null, 2));
      console.log(
        `${runs.length}/${config.targets.length} ${target.type} ${target.id} ${finished?.ok ? "ok" : "FAILED"} ${Date.now() - startedAt} ms`,
      );
      if (!stopped) {
        console.log("The app still reports a run in progress, so the pass ends here.");
        break;
      }
    }
    stopEvents();
  } finally {
    writeFileSync(path.join(config.out, "cgroup.json"), JSON.stringify(cgroupTotals(), null, 2));
    const pid = appProcess();
    if (pid !== null) {
      for (const name of ["status", "smaps_rollup"])
        writeFileSync(
          path.join(config.out, `app-${name}.txt`),
          readFileSync(path.join("/proc", String(pid), name), "utf8"),
        );
      process.kill(pid, "SIGTERM");
    } else app.kill("SIGTERM");
    if (app.exitCode === null) await new Promise((resolve) => app.once("exit", resolve));
    await stopSampling();
    site.close();
  }
}

await main();
process.exit(0);
