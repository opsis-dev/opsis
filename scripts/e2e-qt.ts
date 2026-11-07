import { spawn, spawnSync, type ChildProcess } from "node:child_process";
import { cpSync, existsSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import type { AddressInfo } from "node:net";
import { homedir } from "node:os";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import { ensureFfmpeg } from "./lib/ffmpeg";
import { ROOT } from "./lib/tool-config";
import { DEV_ENGINE, seedUserData } from "./lib/user-data";

const SCENARIOS = ["tour", "runs"] as const;
type ScenarioName = (typeof SCENARIOS)[number];
const DATA_DIR = path.join(homedir(), ".cache", "opsis-qt-e2e");
const RUNS_DIR = path.join(homedir(), ".cache", "opsis-qt-e2e-runs");
const BUNDLES = [
  ["film-bundle", "film:bundle"],
  ["terminal-bundle", "terminal:bundle"],
] as const;
const PAGE = `<!doctype html><html><body style="margin:0;background:#fff"><button style="position:absolute;left:100px;top:100px;width:200px;height:60px;font-size:20px">Save draft</button></body></html>`;
const FIRST_DISPLAY = 90;
const LAST_DISPLAY = 140;
const SCREEN = "1600x1000x24";
const START_WAIT_MS = 10_000;
const RUN_TIMEOUT_MS = 300_000;
const CHECK_LINE = /check \S+: |scenario \S+ done|panicked/;
const FIXTURES_DIR = path.join(DATA_DIR, "fixtures");
const CLIP_SECONDS = "3";

function freeDisplay(): number {
  for (let n = FIRST_DISPLAY; n <= LAST_DISPLAY; n += 1) {
    if (!existsSync(`/tmp/.X11-unix/X${n}`) && !existsSync(`/tmp/.X${n}-lock`)) return n;
  }
  throw new Error(`no free X display between :${FIRST_DISPLAY} and :${LAST_DISPLAY}`);
}

async function waitFor(file: string, ms: number): Promise<void> {
  const until = Date.now() + ms;
  while (!existsSync(file)) {
    if (Date.now() > until) throw new Error(`${file} didn't appear`);
    await sleep(100);
  }
}

function build(): void {
  const built = spawnSync("cargo", ["build", "-q", "-p", "opsis-ui"], {
    cwd: ROOT,
    stdio: "inherit",
  });
  if (built.status !== 0) throw new Error("building the native editor failed");
}

function makeClip(ffmpeg: string): void {
  mkdirSync(FIXTURES_DIR, { recursive: true });
  const made = spawnSync(
    ffmpeg,
    [
      "-v",
      "error",
      "-f",
      "lavfi",
      "-i",
      "testsrc2=size=320x180:rate=30",
      "-f",
      "lavfi",
      "-i",
      "sine=frequency=440:sample_rate=48000",
      "-t",
      CLIP_SECONDS,
      "-c:v",
      "libx264",
      "-pix_fmt",
      "yuv420p",
      "-c:a",
      "aac",
      "-shortest",
      path.join(FIXTURES_DIR, "clip.mp4"),
    ],
    { stdio: "inherit" },
  );
  if (made.status !== 0) throw new Error("making the test clip failed");
}

function seedRuns(): string {
  if (!existsSync(DEV_ENGINE))
    throw new Error(`the runs scenario needs the dev engine in ${DEV_ENGINE}`);
  rmSync(RUNS_DIR, { recursive: true, force: true });
  const userData = path.join(RUNS_DIR, "user-data");
  seedUserData(userData);
  cpSync(path.join(ROOT, "examples"), path.join(RUNS_DIR, "projects"), { recursive: true });
  return userData;
}

async function servePage(): Promise<{ url: string; close: () => void }> {
  const server = createServer((_, res) => {
    res.setHeader("content-type", "text/html");
    res.end(PAGE);
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const { port } = server.address() as AddressInfo;
  return { url: `http://127.0.0.1:${port}/`, close: () => server.close() };
}

function ensureBundles(): void {
  for (const [folder, script] of BUNDLES) {
    if (existsSync(path.join(ROOT, folder, "index.html"))) continue;
    const made = spawnSync("bun", ["run", script], { cwd: ROOT, stdio: "inherit" });
    if (made.status !== 0) throw new Error(`bun run ${script} failed`);
  }
}

function runScenario(
  display: number,
  ffmpeg: string,
  scenario: ScenarioName,
  userData: string,
  extra: Record<string, string>,
): Promise<{ code: number | null; output: string }> {
  const env: Record<string, string | undefined> = {
    ...process.env,
    OPSIS_FFMPEG: ffmpeg,
    DISPLAY: `:${display}`,
    QT_QPA_PLATFORM: "xcb",
    OPSIS_USER_DATA: userData,
    OPSIS_QT_SCENARIO: scenario,
    OPSIS_QT_SHOTS: "off",
    ...extra,
  };
  delete env.WAYLAND_DISPLAY;
  return new Promise((resolve) => {
    const app = spawn(path.join(ROOT, "target", "debug", "opsis"), [], { cwd: ROOT, env });
    let output = "";
    function collect(chunk: Buffer): void {
      output += chunk.toString();
    }
    app.stdout.on("data", collect);
    app.stderr.on("data", collect);
    const timer = setTimeout(() => app.kill("SIGTERM"), RUN_TIMEOUT_MS);
    app.on("exit", (code) => {
      clearTimeout(timer);
      resolve({ code, output });
    });
  });
}

function stop(server: ChildProcess): void {
  if (server.exitCode === null) server.kill("SIGTERM");
}

function chosen(): ScenarioName[] {
  const named = process.argv.slice(2);
  if (named.length === 0) return [...SCENARIOS];
  for (const name of named) {
    if (!SCENARIOS.includes(name as ScenarioName)) throw new Error(`no scenario ${name}`);
  }
  return named as ScenarioName[];
}

async function runOne(display: number, ffmpeg: string, scenario: ScenarioName): Promise<boolean> {
  let userData = DATA_DIR;
  const extra: Record<string, string> = {};
  const page = scenario === "runs" ? await servePage() : null;
  if (scenario === "tour") {
    rmSync(DATA_DIR, { recursive: true, force: true });
    mkdirSync(DATA_DIR, { recursive: true });
    makeClip(ffmpeg);
    extra.OPSIS_QT_FIXTURES = FIXTURES_DIR;
  } else {
    ensureBundles();
    userData = seedRuns();
    extra.OPSIS_QT_PROJECTS = path.join(RUNS_DIR, "projects");
    extra.OPSIS_QT_PAGE = page?.url ?? "";
  }
  try {
    const { code, output } = await runScenario(display, ffmpeg, scenario, userData, extra);
    writeFileSync(path.join(path.dirname(DATA_DIR), `opsis-qt-e2e-${scenario}.log`), output);
    const lines = output.split("\n").filter((line) => CHECK_LINE.test(line));
    for (const line of lines) console.log(line.replace(/^\[[^\]]*\] /, ""));
    const finished = lines.some((line) => /scenario \S+ done/.test(line));
    const failed = lines.some((line) => / FAILED |panicked/.test(line));
    if (code !== 0 || !finished || failed) {
      console.error(`the ${scenario} scenario failed (exit ${code ?? "killed"})`);
      return false;
    }
    return true;
  } finally {
    page?.close();
  }
}

async function main(): Promise<number> {
  const scenarios = chosen();
  build();
  const display = freeDisplay();
  const server = spawn("Xvfb", [`:${display}`, "-screen", "0", SCREEN, "-nolisten", "tcp"], {
    stdio: "ignore",
  });
  try {
    await waitFor(`/tmp/.X11-unix/X${display}`, START_WAIT_MS);
    const ffmpeg = await ensureFfmpeg();
    let passed = true;
    for (const scenario of scenarios) passed = (await runOne(display, ffmpeg, scenario)) && passed;
    return passed ? 0 : 1;
  } finally {
    stop(server);
  }
}

process.exit(await main());
