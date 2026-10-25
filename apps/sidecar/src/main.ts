import { createInterface } from "node:readline";
import type { SidecarMessage, SidecarRequest } from "#sidecar/generated/protocol";
import { renderFrame, renderTimeline } from "#sidecar/render";

const writeMessage = process.stdout.write.bind(process.stdout);
const toStderr = process.stderr.write.bind(process.stderr);
process.stdout.write = toStderr as typeof process.stdout.write;
console.log = console.error;
console.info = console.error;
console.debug = console.error;

const send = (message: SidecarMessage) => writeMessage(`${JSON.stringify(message)}\n`);

const running = new Map<number, { controller: AbortController; done: Promise<void> }>();

const messageOf = (error: unknown) => (error instanceof Error ? error.message : String(error));

function work(request: SidecarRequest, signal: AbortSignal): Promise<void> {
  switch (request.method) {
    case "render.media":
      return renderTimeline(request.params, {
        signal,
        onProgress: (progress) =>
          send({ method: "progress", params: { id: request.id, progress } }),
      });
    case "render.still":
      return renderFrame(request.params, { signal });
    case "cancel":
      running.get(request.params.target)?.controller.abort();
      return Promise.resolve();
    case "ping":
      return Promise.resolve();
  }
}

function start(request: SidecarRequest) {
  const controller = new AbortController();
  const done = work(request, controller.signal)
    .then(
      () => send({ id: request.id, result: null }),
      (error: unknown) => send({ id: request.id, error: { message: messageOf(error) } }),
    )
    .then(() => {
      running.delete(request.id);
    });
  running.set(request.id, { controller, done });
}

const lines = createInterface({ input: process.stdin, crlfDelay: Infinity });

lines.on("line", (line) => {
  if (!line.trim()) return;
  try {
    start(JSON.parse(line) as SidecarRequest);
  } catch (error) {
    console.error(`Unreadable request: ${messageOf(error)}`);
  }
});

lines.on("close", () => {
  const jobs = [...running.values()];
  for (const job of jobs) job.controller.abort();
  void Promise.allSettled(jobs.map((job) => job.done)).then(() => process.exit(0));
});
