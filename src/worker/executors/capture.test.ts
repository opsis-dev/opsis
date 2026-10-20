import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { describe, expect, it, vi } from "vitest";
import { enginePaths } from "../../engine/paths";
import { tempDirs } from "../../engine/test-temp-dirs";
import { compileWorkflow } from "../../shared/compile/compile";
import { findById } from "../../shared/lists";
import { DEFAULT_SETTINGS } from "../../shared/settings";
import { GraphBuilder } from "../../shared/templates/build";
import { must } from "../../shared/test-fixtures";
import type { LoginRequest } from "../../shared/worker-protocol";
import { TracedError, type ExecutorContext } from "../engine/context";
import { signInExecutor, stillExecutor } from "./capture";

const take = vi.hoisted((): { failure: Error | null } => ({ failure: null }));

vi.mock("../capture/driver", () => ({
  recordTake: vi.fn(),
  stillTake: async (opts: { tracePath?: string }) => {
    if (opts.tracePath) await writeFile(opts.tracePath, "not a trace");
    if (take.failure) throw take.failure;
    return { fromPage: { width: 320, height: 200, scale: 2 } };
  },
}));

const project = tempDirs("opsis-capture-");

const sessionInputs = (ctx: ExecutorContext) => ({
  session: compileWorkflow(ctx.workflow).values.session?.session,
});

async function stillContext(
  logs: string[],
  nodeId = "shot",
  more: Partial<ExecutorContext> = {},
): Promise<ExecutorContext> {
  const dir = await project();
  const g = new GraphBuilder();
  g.node("session", "browser.open", 0, 0, { viewportWidth: 1280, viewportHeight: 800 });
  g.node("login", "browser.signIn", 1, 0, { url: "https://x.test/login", secretName: "shop" });
  g.node("shot", "capture.still", 2, 0);
  g.chain(["session", "shot"]);
  g.chain(["session", "login"]);
  const workflow = g.build({ id: "wf", name: "Capture test" });
  const engine = enginePaths(path.join(dir, "engine"));
  const shell = engine.headlessShells[0];
  await mkdir(path.dirname(shell), { recursive: true });
  await writeFile(shell, "");
  const nodeDir = path.join(dir, ".opsis", "cache", "shot");
  await mkdir(nodeDir, { recursive: true });
  const unused = () => {
    throw new Error("Not used by a screenshot.");
  };
  return {
    runId: "run",
    node: must(findById(workflow.nodes, nodeId), nodeId),
    workflow,
    env: {
      appRoot: dir,
      appVersion: "test",
      isPackaged: false,
      resourcesPath: dir,
      userData: dir,
      engineDir: dir,
      settings: DEFAULT_SETTINGS,
    },
    projectDir: dir,
    dir: nodeDir,
    relDir: ".opsis/cache/shot",
    outDir: path.join(dir, "out"),
    hash: "shot",
    forced: false,
    vars: {},
    signal: new AbortController().signal,
    ffmpeg: { run: unused, pipe: unused, probe: unused },
    engine,
    live: () => null,
    login: unused,
    log: (level, message) => logs.push(`${level}: ${message}`),
    progress: () => undefined,
    step: () => undefined,
    compile: () => compileWorkflow(workflow),
    results: {},
    secrets: { read: async () => null, write: async () => undefined },
    ...more,
  };
}

describe("taking a screenshot", () => {
  it("keeps the screenshot when its trace can't be saved", async () => {
    take.failure = null;
    const logs: string[] = [];
    const ctx = await stillContext(logs);
    const out = await stillExecutor.run(ctx, sessionInputs(ctx));
    expect(out.outputs.image).toMatchObject({ type: "image", width: 320, height: 200 });
    expect(out.traceId).toBeUndefined();
    expect(logs.some((l) => l.startsWith("warn: Couldn't save the trace"))).toBe(true);
  });

  it("reports the page's error, not the trace's, when both fail", async () => {
    take.failure = new Error("The page closed.");
    const logs: string[] = [];
    const ctx = await stillContext(logs);
    const error = await stillExecutor.run(ctx, sessionInputs(ctx)).then(
      () => null,
      (e: unknown) => e,
    );
    expect(error).toBeInstanceOf(TracedError);
    expect(error).toHaveProperty("message", "The page closed.");
    expect(logs.some((l) => l.startsWith("warn: Couldn't save the trace"))).toBe(true);
  });
});

describe("signing in", () => {
  const signIn = async (saved: string | null, forced = false) => {
    const opened: LoginRequest[] = [];
    const written: string[] = [];
    const ctx = await stillContext([], "login", {
      forced,
      login: async (request) => {
        opened.push(request);
        return "new sign-in";
      },
      secrets: {
        read: async () => saved,
        write: async (name, value) => {
          written.push(`${name}=${value}`);
        },
      },
    });
    await signInExecutor.run(ctx, sessionInputs(ctx));
    return { opened, written };
  };

  it("opens no window for a sign-in that is already saved", async () => {
    expect(await signIn("old sign-in")).toEqual({ opened: [], written: [] });
  });

  it("opens a window the size of the browser and saves what comes back", async () => {
    const { opened, written } = await signIn(null);
    expect(opened).toEqual([
      {
        url: "https://x.test/login",
        doneWhen: "window-closed",
        selector: "",
        timeoutSec: 300,
        viewport: { width: 1280, height: 800 },
      },
    ]);
    expect(written).toEqual(["shop=new sign-in"]);
  });

  it("signs in again when asked to run, even with a saved sign-in", async () => {
    expect((await signIn("old sign-in", true)).written).toEqual(["shop=new sign-in"]);
  });
});
