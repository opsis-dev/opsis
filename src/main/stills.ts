import { existsSync } from "node:fs";
import { mkdir, readdir, rm } from "node:fs/promises";
import path from "node:path";
import type { BrowserWindow } from "electron";
import { z } from "zod";
import { sha256Hex } from "../engine/hash";
import { readJson, writeJson } from "../engine/json";
import { engineStatus } from "../engine/paths";
import { errorMessage } from "../shared/errors";
import { IPC, type StillRequest, type StillResult } from "../shared/ipc";
import { createLazyWorker } from "./lazy-worker";
import { engineDir, isRunning } from "./workers";
import { projectPaths } from "../engine/project-paths";

const STILL_WIDTH = 480;
const RUN_POLL_MS = 2000;

const indexSchema = z.record(z.string(), z.string());

const stillsDir = (projectDir: string) => projectPaths(projectDir).stills;
const indexPath = (projectDir: string) => path.join(stillsDir(projectDir), "index.json");
const stillPath = (projectDir: string, hash: string) =>
  path.join(stillsDir(projectDir), `${hash}.jpg`);

type Job = StillRequest & { hash: string };

const worker = createLazyWorker<string | undefined>({
  kind: "stills",
  idleMs: 60_000,
  reply: (msg) => (msg.type === "still:done" ? { id: msg.id, result: msg.error } : null),
  stopped: "The preview worker stopped.",
});

const queue: Job[] = [];
let busy = false;
let waitingForRun: NodeJS.Timeout | null = null;
let getWindow: () => BrowserWindow | null = () => null;

export function initStills(window: () => BrowserWindow | null): void {
  getWindow = window;
}

export async function listStills(projectDir: string): Promise<Record<string, string>> {
  const index = indexSchema.safeParse(await readJson(indexPath(projectDir))).data ?? {};
  return Object.fromEntries(
    Object.entries(index).filter(([, hash]) => existsSync(stillPath(projectDir, hash))),
  );
}

async function remember(projectDir: string, nodeId: string, hash: string): Promise<void> {
  const index = { ...(await listStills(projectDir)), [nodeId]: hash };
  await mkdir(stillsDir(projectDir), { recursive: true });
  await writeJson(indexPath(projectDir), index);
  const keep = new Set(Object.values(index).map((h) => `${h}.jpg`));
  for (const file of await readdir(stillsDir(projectDir)))
    if (file.endsWith(".jpg") && !keep.has(file))
      await rm(path.join(stillsDir(projectDir), file), { force: true });
  getWindow()?.webContents.send(IPC.stillReady, { projectDir, nodeId, hash });
}

export async function requestStill(request: StillRequest): Promise<StillResult> {
  const hash = sha256Hex(request.key, 24);
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

async function pump(): Promise<void> {
  if (busy || waitingForRun || queue.length === 0) return;
  if (isRunning()) {
    waitingForRun = setTimeout(() => {
      waitingForRun = null;
      void pump();
    }, RUN_POLL_MS);
    return;
  }
  const job = queue.shift();
  if (!job) return;
  busy = true;
  try {
    const output = stillPath(job.projectDir, job.hash);
    const error = existsSync(output)
      ? undefined
      : await worker.request((env, id) => ({
          type: "still",
          env,
          id,
          projectDir: job.projectDir,
          spec: job.spec,
          frame: job.frame,
          output,
          width: STILL_WIDTH,
        }));
    if (error) process.stderr.write(`[stills] ${job.nodeId}: ${error}\n`);
    else await remember(job.projectDir, job.nodeId, job.hash);
  } catch (error) {
    process.stderr.write(`[stills] ${job.nodeId}: ${errorMessage(error)}\n`);
  } finally {
    busy = false;
    if (queue.length) void pump();
    else worker.scheduleIdle();
  }
}

export const stopStills = (): void => worker.stop();
