import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { describe, expect, it, vi } from "vitest";
import { secretFile } from "../../engine/secret-paths";
import { tempDirs } from "../../engine/test-temp-dirs";
import type { RunEvent } from "../../shared/schema/events";
import type { Workflow } from "../../shared/schema/workflow";
import { DEFAULT_SETTINGS } from "../../shared/settings";
import { GraphBuilder } from "../../shared/templates/build";
import type { Executor } from "./context";
import { runWorkflow } from "./runner";

const ran = vi.hoisted((): string[] => []);

async function saveSignIn(projectDir: string, name: string) {
  const file = secretFile(projectDir, name);
  await mkdir(path.dirname(file), { recursive: true });
  await writeFile(file, "sealed sign-in");
}

vi.mock("../executors", () => ({
  getExecutor: (type: string): Executor => ({
    type,
    run: async (ctx) => {
      ran.push(ctx.node.id);
      if (type === "browser.signIn") await saveSignIn(ctx.projectDir, "default");
      return { outputs: {} };
    },
  }),
}));

const project = tempDirs("opsis-runner-");

function signedInTake(): Workflow {
  const g = new GraphBuilder();
  g.node("open", "browser.open", 0, 0);
  g.node("login", "browser.signIn", 1, 0, { url: "https://x.test/login" });
  g.node("go", "browser.navigate", 2, 0, { url: "https://x.test/app" });
  g.node("rec", "capture.record", 3, 0);
  g.chain(["open", "login", "go", "rec"]);
  return g.build({ id: "x", name: "x" });
}

async function run(projectDir: string, workflow: Workflow): Promise<RunEvent[]> {
  const events: RunEvent[] = [];
  ran.length = 0;
  await runWorkflow(
    "run",
    { projectDir, workflow },
    {
      env: {
        appRoot: projectDir,
        appVersion: "test",
        isPackaged: false,
        resourcesPath: projectDir,
        userData: projectDir,
        engineDir: projectDir,
        settings: DEFAULT_SETTINGS,
      },
      emit: (event) => events.push(event),
      signal: new AbortController().signal,
      secrets: { read: async () => null, write: async () => undefined },
      login: async () => "{}",
      live: null,
    },
  );
  return events;
}

describe("a take after a Sign in", () => {
  it("runs the Sign in first, as the node before it", async () => {
    const events = await run(await project(), signedInTake());
    expect(ran).toEqual(["login", "rec"]);
    expect(events.at(-1)).toMatchObject({ type: "run:done", ok: true });
  });

  it("keeps what it recorded with a new sign-in, so the next run is all saved results", async () => {
    const dir = await project();
    const workflow = signedInTake();
    await run(dir, workflow);
    const again = await run(dir, workflow);
    expect(ran).toEqual([]);
    const done = again.filter((e) => e.type === "node:done" && ["login", "rec"].includes(e.nodeId));
    expect(done).toEqual([
      expect.objectContaining({ nodeId: "login", cached: true }),
      expect.objectContaining({ nodeId: "rec", cached: true }),
    ]);
  });
});
