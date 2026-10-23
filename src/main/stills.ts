import { existsSync } from "node:fs";
import { mkdir } from "node:fs/promises";
import path from "node:path";
import { z } from "zod";
import { pruneDir } from "../engine/dir";
import { refStamps } from "../engine/file-stamps";
import { sha256Hex } from "../engine/hash";
import { readJson, writeJson } from "../engine/json";
import { logError } from "../engine/log";
import { engineStatus } from "../engine/paths";
import { PICTURE_VERSION } from "../engine/picture-version";
import { projectPaths } from "../engine/project-paths";
import { IPC, type StillRequest, type StillResult } from "../shared/ipc";
import { projectRefsIn } from "../shared/named-files";
import { oneAtATime } from "../shared/timing";
import { createQueuedWorker } from "./lazy-worker";
import { isRunning } from "./run-host";
import { getMainWindow } from "./window-registry";
import { engineDir } from "./workers";

const STILL_WIDTH = 480;

const indexSchema = z.record(z.string(), z.string());

const stillsDir = (projectDir: string) => projectPaths(projectDir).stills;
const indexPath = (projectDir: string) => path.join(stillsDir(projectDir), "index.json");
const stillPath = (projectDir: string, hash: string) =>
  path.join(stillsDir(projectDir), `${hash}.jpg`);

type Job = StillRequest & { hash: string };

const IDLE_MS = 60_000;

const reportFailure = (job: Job, error: unknown) => logError("stills", error, job.nodeId);

const stills = createQueuedWorker<string | undefined, Job>({
  kind: "stills",
  idleMs: IDLE_MS,
  keyOf: (job) => JSON.stringify([job.projectDir, job.nodeId]),
  reply: (msg) => (msg.type === "still:done" ? { id: msg.id, result: msg.error } : null),
  stopped: "The preview worker stopped.",
  holdWhile: isRunning,
  run: async (job, request) => {
    const output = stillPath(job.projectDir, job.hash);
    const error = existsSync(output)
      ? undefined
      : await request((env, id) => ({
          type: "still",
          env,
          id,
          projectDir: job.projectDir,
          spec: job.spec,
          frame: job.frame,
          output,
          width: STILL_WIDTH,
        }));
    if (error) reportFailure(job, error);
    else await recordStill(job.projectDir, job.nodeId, job.hash);
  },
  failed: reportFailure,
});

const oneIndexWriteAtATime = oneAtATime();

export async function listStills(projectDir: string): Promise<Record<string, string>> {
  const index = (await readJson(indexPath(projectDir), indexSchema)) ?? {};
  return Object.fromEntries(
    Object.entries(index).filter(([, hash]) => existsSync(stillPath(projectDir, hash))),
  );
}

async function recordStill(projectDir: string, nodeId: string, hash: string): Promise<void> {
  const changed = await oneIndexWriteAtATime(async () => {
    const stored = await listStills(projectDir);
    if (stored[nodeId] === hash) return false;
    const index = { ...stored, [nodeId]: hash };
    await mkdir(stillsDir(projectDir), { recursive: true });
    await writeJson(indexPath(projectDir), index);
    const keep = new Set(Object.values(index).map((h) => `${h}.jpg`));
    await pruneDir(stillsDir(projectDir), {
      keep: (name) => !name.endsWith(".jpg") || keep.has(name),
    });
    return true;
  });
  if (changed) getMainWindow()?.webContents.send(IPC.stillReady, { projectDir, nodeId, hash });
}

export async function requestStill(request: StillRequest): Promise<StillResult> {
  const stamps = JSON.stringify(refStamps(request.projectDir, projectRefsIn(request.spec)));
  const hash = sha256Hex(`${PICTURE_VERSION}:${request.key}:${stamps}`, 24);
  if (existsSync(stillPath(request.projectDir, hash))) {
    await recordStill(request.projectDir, request.nodeId, hash);
    return { hash, ready: true };
  }
  if (!engineStatus(engineDir()).complete) return { hash, ready: false, unavailable: true };
  stills.enqueue({ ...request, hash }, { first: request.priority });
  return { hash, ready: false };
}

export const stopStills = (): void => stills.stop();
