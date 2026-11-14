import { spawn, spawnSync, type ChildProcess } from "node:child_process";
import {
  copyFileSync,
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { createServer } from "node:http";
import type { AddressInfo } from "node:net";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";
import { setTimeout as sleep } from "node:timers/promises";
import { runApiChecks } from "./lib/e2e-api";
import { stagedFfmpeg } from "./lib/ffmpeg";
import { devCache, ROOT } from "./lib/tool-config";
import { DEV_ENGINE, browserIn, seedUserData } from "./lib/user-data";

const SCENARIOS = ["tour", "runs", "api", "wayland"] as const;
type ScenarioName = (typeof SCENARIOS)[number];
const DATA_DIR = devCache("qt-e2e");
const RUNS_DIR = devCache("qt-e2e-runs");
const API_DIR = devCache("qt-e2e-api");
const WAYLAND_DIR = devCache("qt-e2e-wayland");
const SWAY_CONFIG = [
  "output HEADLESS-1 resolution {size}",
  "default_border none",
  "default_floating_border none",
  "focus_follows_mouse no",
  "",
].join("\n");
const WAYLAND_SOCKET = /^wayland-\d+$/;
const LOG_DIR = path.join(ROOT, "target", "e2e-qt");
const SHOTS_DIR = devCache("qt-shots");
const SHOTS_FLAG = "--shots";
const PAGE = [
  "<!doctype html><html><head><title>Test page</title></head>",
  '<body style="margin:0;background:#fff;font:16px sans-serif">',
  '<button style="position:absolute;left:100px;top:100px;width:200px;height:60px;font-size:20px">Save draft</button>',
  '<div style="position:absolute;left:100px;top:220px;width:480px">',
  '<input class="new-todo" placeholder="What needs to be done?" style="width:100%;font-size:20px">',
  '<label><input type="checkbox" aria-label="Toggle Todo"> First</label>',
  '<a href="#/completed">Completed</a>',
  "</div></body></html>",
].join("");
const FIRST_DISPLAY = 90;
const LAST_DISPLAY = 140;
const SCREEN = "1600x1000x24";
const START_WAIT_MS = 10_000;
const RUN_TIMEOUT_MS = 900_000;
const CHECK_LINE = /check \S+: |scenario \S+ done|panicked/;
const FIXTURES_DIR = path.join(DATA_DIR, "fixtures");
const CLIP_SECONDS = "3";

type Screen = { stop: () => void; env: Record<string, string>; unset: string };

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
  const built = spawnSync(
    "cargo",
    ["build", "-q", "-p", "opsis-ui", "-p", "opsis-e2e-input", "-p", "opsis-effects"],
    {
      cwd: ROOT,
      stdio: "inherit",
    },
  );
  if (built.status !== 0) throw new Error("building the native editor failed");
}

function copySampleFont(): void {
  const found = spawnSync("fc-match", ["-f", "%{file}", "sans"], { encoding: "utf8" });
  const file = found.stdout.trim();
  if (found.status !== 0 || !existsSync(file)) throw new Error("fc-match found no sans font");
  copyFileSync(file, path.join(FIXTURES_DIR, `sample-font${path.extname(file)}`));
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

function devBrowser(): string {
  const browser = existsSync(DEV_ENGINE) ? browserIn(DEV_ENGINE) : null;
  if (browser === null) throw new Error(`the scenarios need the dev engine in ${DEV_ENGINE}`);
  return browser;
}

function seedApi(): string {
  rmSync(API_DIR, { recursive: true, force: true });
  const userData = path.join(API_DIR, "user-data");
  seedUserData(userData, { api: { enabled: true, scripts: true } });
  cpSync(path.join(ROOT, "examples"), path.join(API_DIR, "projects"), { recursive: true });
  return userData;
}

function exitConfirmed(userData: string): boolean | undefined {
  const db = new DatabaseSync(path.join(userData, "opsis.db"));
  const row = db.prepare("SELECT value FROM settings WHERE section = 'app'").get() as
    | { value: string }
    | undefined;
  db.close();
  return row === undefined
    ? undefined
    : (JSON.parse(row.value) as { confirmExit?: boolean }).confirmExit;
}

async function runApi(screen: Screen, ffmpeg: string, page: string): Promise<boolean> {
  const userData = seedApi();
  const handoff = path.join(API_DIR, "api-done");
  const app = runScenario(screen, ffmpeg, "api", userData, {
    OPSIS_QT_PAGE: page,
    OPSIS_QT_HANDOFF: handoff,
  });
  const outside = await runApiChecks(
    { socket: path.join(userData, "api.sock"), projects: path.join(API_DIR, "projects"), page },
    (line) => console.log(line),
  );
  writeFileSync(handoff, "");
  const { code, output } = await app;
  writeLog("api", output);
  const lines = output.split("\n").filter((line) => CHECK_LINE.test(line));
  for (const line of lines) console.log(line.replace(/^\[[^\]]*\] /, ""));
  const inside =
    lines.some((line) => /check dont-ask-again: ok/.test(line)) &&
    !lines.some((line) => / FAILED |panicked/.test(line));
  const remembered = exitConfirmed(userData) === false;
  if (!remembered)
    console.log("check close-remembers-dont-ask-again: FAILED the choice wasn't saved");
  if (code !== 0 || !outside || !inside || !remembered) {
    console.error(`the api scenario failed (exit ${code ?? "killed"})`);
    return false;
  }
  return true;
}

function writeLog(scenario: ScenarioName, output: string): void {
  mkdirSync(LOG_DIR, { recursive: true });
  writeFileSync(path.join(LOG_DIR, `${scenario}.log`), output);
}

function seedRuns(): string {
  rmSync(RUNS_DIR, { recursive: true, force: true });
  const userData = path.join(RUNS_DIR, "user-data");
  seedUserData(userData, {}, { engine: false });
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

function runScenario(
  screen: Screen,
  ffmpeg: string,
  scenario: ScenarioName,
  userData: string,
  extra: Record<string, string>,
): Promise<{ code: number | null; output: string }> {
  const env: Record<string, string | undefined> = {
    ...process.env,
    OPSIS_FFMPEG: ffmpeg,
    OPSIS_AUDIO_DEVICE: "alsa:null",
    ...screen.env,
    OPSIS_USER_DATA: userData,
    OPSIS_QT_SCENARIO: scenario,
    OPSIS_QT_SHOTS: shotsFor(scenario),
    ...extra,
  };
  delete env[screen.unset];
  const binary = path.join(ROOT, "target", "debug", "opsis");
  const hang = Number(process.env.OPSIS_QT_E2E_HANG_MS ?? 0);
  const [program, args] =
    hang > 0
      ? ["gdb", ["-q", "-batch", "-ex", "run", "-ex", "thread apply all bt", "--args", binary]]
      : [binary, []];
  return new Promise((resolve) => {
    const app = spawn(program, args, { cwd: ROOT, env });
    const interrupt = hang > 0 ? setTimeout(() => interruptChild(app), hang) : undefined;
    let output = "";
    function collect(chunk: Buffer): void {
      output += chunk.toString();
    }
    app.stdout.on("data", collect);
    app.stderr.on("data", collect);
    const timer = setTimeout(() => app.kill("SIGTERM"), RUN_TIMEOUT_MS);
    app.on("exit", (code) => {
      clearTimeout(timer);
      clearTimeout(interrupt);
      resolve({ code, output });
    });
  });
}

function interruptChild(debugger_: ChildProcess): void {
  const pid = debugger_.pid;
  if (pid === undefined) return;
  const children = readFileSync(`/proc/${pid}/task/${pid}/children`, "utf8").trim().split(" ");
  for (const child of children) if (child !== "") process.kill(Number(child), "SIGINT");
}

function stop(server: ChildProcess): void {
  if (server.exitCode === null) server.kill("SIGTERM");
}

function shotsFor(scenario: ScenarioName): string {
  if (!process.argv.includes(SHOTS_FLAG)) return "off";
  const dir = path.join(SHOTS_DIR, scenario);
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  return dir;
}

function chosen(): ScenarioName[] {
  const named = process.argv.slice(2).filter((arg) => arg !== SHOTS_FLAG);
  if (named.length === 0) return [...SCENARIOS];
  for (const name of named) {
    if (!SCENARIOS.includes(name as ScenarioName)) throw new Error(`no scenario ${name}`);
  }
  return named as ScenarioName[];
}

async function runOne(screen: Screen, ffmpeg: string, scenario: ScenarioName): Promise<boolean> {
  let userData = DATA_DIR;
  const extra: Record<string, string> = {};
  const page = await servePage();
  if (scenario === "api") {
    try {
      return await runApi(screen, ffmpeg, page.url);
    } finally {
      page.close();
    }
  }
  if (scenario === "wayland") {
    rmSync(WAYLAND_DIR, { recursive: true, force: true });
    userData = path.join(WAYLAND_DIR, "user-data");
    seedUserData(userData, { api: { enabled: false } }, { engine: false });
    mkdirSync(path.join(WAYLAND_DIR, "projects"), { recursive: true });
    extra.OPSIS_QT_PROJECTS = path.join(WAYLAND_DIR, "projects");
  } else if (scenario === "tour") {
    rmSync(DATA_DIR, { recursive: true, force: true });
    seedUserData(DATA_DIR, { api: { enabled: false } });
    makeClip(ffmpeg);
    copySampleFont();
    extra.OPSIS_QT_FIXTURES = FIXTURES_DIR;
    extra.OPSIS_QT_PAGE = page.url;
  } else {
    userData = seedRuns();
    extra.OPSIS_QT_PROJECTS = path.join(RUNS_DIR, "projects");
    extra.OPSIS_QT_PAGE = page.url;
    extra.OPSIS_QT_BROWSER = devBrowser();
  }
  try {
    const { code, output } = await runScenario(screen, ffmpeg, scenario, userData, extra);
    writeLog(scenario, output);
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
    page.close();
  }
}

async function startXvfb(): Promise<Screen> {
  const display = freeDisplay();
  const server = spawn("Xvfb", [`:${display}`, "-screen", "0", SCREEN, "-nolisten", "tcp"], {
    stdio: "ignore",
  });
  await waitFor(`/tmp/.X11-unix/X${display}`, START_WAIT_MS);
  return {
    stop: () => stop(server),
    env: { DISPLAY: `:${display}`, QT_QPA_PLATFORM: "xcb" },
    unset: "WAYLAND_DISPLAY",
  };
}

async function waylandSocket(runtime: string): Promise<string> {
  const until = Date.now() + START_WAIT_MS;
  while (Date.now() < until) {
    const found = readdirSync(runtime).find((name) => WAYLAND_SOCKET.test(name));
    if (found !== undefined) return found;
    await sleep(100);
  }
  throw new Error("headless sway didn't open its socket");
}

function untilReady(helper: ChildProcess): Promise<void> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(
      () => reject(new Error("the input helper didn't start")),
      START_WAIT_MS,
    );
    let said = "";
    helper.stdout?.on("data", (chunk: Buffer) => {
      said += chunk.toString();
      if (said.split("\n").includes("ready")) {
        clearTimeout(timer);
        resolve();
      }
    });
    helper.on("exit", (code) => {
      clearTimeout(timer);
      reject(new Error(`the input helper stopped (exit ${code})`));
    });
  });
}

async function startSway(): Promise<Screen> {
  const parent = process.env.XDG_RUNTIME_DIR;
  if (parent === undefined) throw new Error("the Wayland scenario needs XDG_RUNTIME_DIR");
  const runtime = path.join(parent, `opsis-e2e-${process.pid}`);
  rmSync(runtime, { recursive: true, force: true });
  mkdirSync(runtime, { mode: 0o700 });
  const [width = "", height = ""] = SCREEN.split("x");
  const config = path.join(runtime, "sway.conf");
  writeFileSync(config, SWAY_CONFIG.replace("{size}", `${width}x${height}`));
  const env: Record<string, string | undefined> = {
    ...process.env,
    XDG_RUNTIME_DIR: runtime,
    WLR_BACKENDS: "headless",
    WLR_LIBINPUT_NO_DEVICES: "1",
    WLR_RENDERER: "pixman",
  };
  delete env.WAYLAND_DISPLAY;
  delete env.DISPLAY;
  delete env.SWAYSOCK;
  const sway = spawn("sway", ["-c", config], { stdio: "ignore", env });
  const socket = await waylandSocket(runtime);
  const requests = path.join(runtime, "input.txt");
  writeFileSync(requests, "");
  const helper = spawn(
    path.join(ROOT, "target", "debug", "opsis-e2e-input"),
    [requests, width, height],
    {
      env: { ...process.env, XDG_RUNTIME_DIR: runtime, WAYLAND_DISPLAY: socket },
      stdio: ["ignore", "pipe", "inherit"],
    },
  );
  await untilReady(helper);
  return {
    stop: () => {
      stop(helper);
      stop(sway);
      rmSync(runtime, { recursive: true, force: true });
    },
    env: {
      XDG_RUNTIME_DIR: runtime,
      WAYLAND_DISPLAY: socket,
      QT_QPA_PLATFORM: "wayland",
      OPSIS_QT_INPUT: requests,
    },
    unset: "DISPLAY",
  };
}

async function main(): Promise<number> {
  const scenarios = chosen();
  build();
  const xvfb = scenarios.some((scenario) => scenario !== "wayland") ? await startXvfb() : null;
  try {
    const ffmpeg = stagedFfmpeg();
    let passed = true;
    for (const scenario of scenarios) {
      if (scenario === "wayland") {
        const sway = await startSway();
        try {
          passed = (await runOne(sway, ffmpeg, scenario)) && passed;
        } finally {
          sway.stop();
        }
      } else if (xvfb !== null) {
        passed = (await runOne(xvfb, ffmpeg, scenario)) && passed;
      }
    }
    return passed ? 0 : 1;
  } finally {
    xvfb?.stop();
  }
}

process.exit(await main());
