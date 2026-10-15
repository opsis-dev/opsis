import { createHash, randomUUID } from "node:crypto";
import { existsSync } from "node:fs";
import { mkdir, readdir, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import type { BrowserWindow, MessagePortMain, UtilityProcess } from "electron";
import { engineStatus } from "../engine/paths";
import { IPC, type StillRequest, type StillResult } from "../shared/ipc";
import type { MainToWorker, WorkerToMain } from "../shared/worker-protocol";
import { engineDir, isRunning, spawnWorker, workerEnv } from "./workers";

const STILL_WIDTH = 480;
const IDLE_MS = 60_000;

const stillsDir = (projectDir: string) => path.join(projectDir, ".demoflow", "cache", "stills");
const indexPath = (projectDir: string) => path.join(stillsDir(projectDir), "index.json");
const stillPath = (projectDir: string, hash: string) =>
  path.join(stillsDir(projectDir), `${hash}.jpg`);

type Job = StillRequest & { hash: string };

const queue: Job[] = [];
let busy = false;
let worker: { child: UtilityProcess; control: MessagePortMain } | null = null;
let idleTimer: NodeJS.Timeout | null = null;
const waiting = new Map<string, (error?: string) => void>();
let getWindow: () => BrowserWindow | null = () => null;

export function initStills(window: () => BrowserWindow | null): void {
  getWindow = window;
}

export async function listStills(projectDir: string): Promise<Record<string, string>> {
  try {
    const index = JSON.parse(await readFile(indexPath(projectDir), "utf8")) as Record<
      string,
      string
    >;
    return Object.fromEntries(
      Object.entries(index).filter(([, hash]) => existsSync(stillPath(projectDir, hash))),
    );
  } catch {
    return {};
  }
}

async function remember(projectDir: string, nodeId: string, hash: string): Promise<void> {
  const index = { ...(await listStills(projectDir)), [nodeId]: hash };
  await mkdir(stillsDir(projectDir), { recursive: true });
  await writeFile(indexPath(projectDir), JSON.stringify(index, null, 2));
  const keep = new Set(Object.values(index).map((h) => `${h}.jpg`));
  for (const file of await readdir(stillsDir(projectDir)))
    if (file.endsWith(".jpg") && !keep.has(file))
      await rm(path.join(stillsDir(projectDir), file), { force: true });
  getWindow()?.webContents.send(IPC.stillReady, { projectDir, nodeId, hash });
}

export async function requestStill(request: StillRequest): Promise<StillResult> {
  const hash = createHash("sha256").update(request.key).digest("hex").slice(0, 24);
  if (existsSync(stillPath(request.projectDir, hash))) {
    await remember(request.projectDir, request.nodeId, hash);
    return { hash, ready: true };
  }
  if (!engineStatus(engineDir()).complete) return { hash, ready: false, unavailable: true };
  const existing = queue.findIndex(
    (j) => j.projectDir === request.projectDir && j.nodeId === request.nodeId,
  );
  if (existing !== -1) queue.splice(existing, 1);
  const job = { ...request, hash };
  if (request.priority) queue.unshift(job);
  else queue.push(job);
  void pump();
  return { hash, ready: false };
}

function ensureWorker() {
  if (idleTimer) clearTimeout(idleTimer);
  idleTimer = null;
  if (worker) return worker;
  const spawned = spawnWorker(false);
  spawned.control.on("message", (e) => {
    const msg = e.data as WorkerToMain;
    if (msg.type !== "still:done") return;
    const done = waiting.get(msg.id);
    waiting.delete(msg.id);
    done?.(msg.error);
  });
  spawned.child.on("exit", () => {
    if (worker?.child === spawned.child) worker = null;
    for (const [id, done] of waiting) {
      waiting.delete(id);
      done("The preview worker stopped.");
    }
  });
  worker = { child: spawned.child, control: spawned.control };
  return worker;
}

function scheduleIdle() {
  if (idleTimer) clearTimeout(idleTimer);
  idleTimer = setTimeout(() => {
    worker?.child.kill();
    worker = null;
  }, IDLE_MS);
}

async function pump(): Promise<void> {
  if (busy || queue.length === 0) return;
  if (isRunning()) {
    setTimeout(() => void pump(), 2000);
    return;
  }
  busy = true;
  const job = queue.shift()!;
  try {
    const output = stillPath(job.projectDir, job.hash);
    if (!existsSync(output)) {
      const { control } = ensureWorker();
      const id = randomUUID();
      const error = await new Promise<string | undefined>((resolve) => {
        waiting.set(id, resolve);
        void workerEnv().then((env) =>
          control.postMessage({
            type: "still",
            env,
            id,
            projectDir: job.projectDir,
            spec: job.spec,
            frame: job.frame,
            output,
            width: STILL_WIDTH,
          } satisfies MainToWorker),
        );
      });
      if (error) process.stderr.write(`[stills] ${job.nodeId}: ${error}\n`);
    }
    if (existsSync(output)) await remember(job.projectDir, job.nodeId, job.hash);
  } finally {
    busy = false;
    if (queue.length) void pump();
    else scheduleIdle();
  }
}
