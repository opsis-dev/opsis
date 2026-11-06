import { spawn, spawnSync, type ChildProcess } from "node:child_process";
import { existsSync, mkdirSync, rmSync } from "node:fs";
import { homedir } from "node:os";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import { ROOT } from "./lib/tool-config";

const SCENARIO = "tour";
const DATA_DIR = path.join(homedir(), ".cache", "opsis-qt-e2e");
const FIRST_DISPLAY = 90;
const LAST_DISPLAY = 140;
const SCREEN = "1600x1000x24";
const START_WAIT_MS = 10_000;
const RUN_TIMEOUT_MS = 300_000;
const CHECK_LINE = /check \S+: |scenario \S+ done|panicked/;

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

function runScenario(display: number): Promise<{ code: number | null; output: string }> {
  const env: Record<string, string | undefined> = {
    ...process.env,
    DISPLAY: `:${display}`,
    QT_QPA_PLATFORM: "xcb",
    OPSIS_USER_DATA: DATA_DIR,
    OPSIS_QT_SCENARIO: SCENARIO,
    OPSIS_QT_SHOTS: "off",
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

async function main(): Promise<number> {
  build();
  rmSync(DATA_DIR, { recursive: true, force: true });
  mkdirSync(DATA_DIR, { recursive: true });
  const display = freeDisplay();
  const server = spawn("Xvfb", [`:${display}`, "-screen", "0", SCREEN, "-nolisten", "tcp"], {
    stdio: "ignore",
  });
  try {
    await waitFor(`/tmp/.X11-unix/X${display}`, START_WAIT_MS);
    const { code, output } = await runScenario(display);
    const lines = output.split("\n").filter((line) => CHECK_LINE.test(line));
    for (const line of lines) console.log(line.replace(/^\[[^\]]*\] /, ""));
    const finished = lines.some((line) => /scenario \S+ done/.test(line));
    const failed = lines.some((line) => / FAILED |panicked/.test(line));
    if (code !== 0 || !finished || failed) {
      console.error(`the ${SCENARIO} scenario failed (exit ${code ?? "killed"})`);
      return 1;
    }
    return 0;
  } finally {
    stop(server);
  }
}

process.exit(await main());
