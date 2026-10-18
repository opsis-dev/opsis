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
import { TracedError, type ExecutorContext } from "../engine/context";
import { stillExecutor } from "./capture";

const take = vi.hoisted((): { failure: Error | null } => ({ failure: null }));

vi.mock("../capture/driver", () => ({
  recordTake: vi.fn(),
  stillTake: async (opts: { tracePath?: string }) => {
    if (opts.tracePath) await writeFile(opts.tracePath, "not a trace");
    if (take.failure) throw take.failure;
    return { fromPage: { width: 320, height: 200, scale: 2 } };
  },
}));

const project = tempDirs("demoflow-capture-");

async function stillContext(logs: string[]): Promise<ExecutorContext> {
  const dir = await project();
  const g = new GraphBuilder();
  g.node("session", "browser.session", 0, 0);
  g.node("shot", "capture.still", 1, 0);
  g.chain(["session", "shot"]);
  const workflow = g.build({ id: "wf", name: "Capture test" });
  const engine = enginePaths(path.join(dir, "engine"));
  const shell = engine.headlessShells[0];
  await mkdir(path.dirname(shell), { recursive: true });
  await writeFile(shell, "");
  const nodeDir = path.join(dir, ".demoflow", "cache", "shot");
  await mkdir(nodeDir, { recursive: true });
  const unused = () => {
    throw new Error("Not used by a screenshot.");
  };
  return {
    runId: "run",
    node: must(findById(workflow.nodes, "shot"), "screenshot node"),
    workflow,
    env: {
      appRoot: dir,
      isPackaged: false,
      resourcesPath: dir,
      userData: dir,
      engineDir: dir,
      settings: DEFAULT_SETTINGS,
    },
    projectDir: dir,
    dir: nodeDir,
    relDir: ".demoflow/cache/shot",
    outDir: path.join(dir, "out"),
    hash: "shot",
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
  };
}

describe("taking a screenshot", () => {
  it("keeps the screenshot when its trace can't be saved", async () => {
    take.failure = null;
    const logs: string[] = [];
    const out = await stillExecutor.run(await stillContext(logs), {});
    expect(out.outputs.image).toMatchObject({ type: "image", width: 320, height: 200 });
    expect(out.traceId).toBeUndefined();
    expect(logs.some((l) => l.startsWith("warn: Couldn't save the trace"))).toBe(true);
  });

  it("reports the page's error, not the trace's, when both fail", async () => {
    take.failure = new Error("The page closed.");
    const logs: string[] = [];
    const error = await stillExecutor.run(await stillContext(logs), {}).then(
      () => null,
      (e: unknown) => e,
    );
    expect(error).toBeInstanceOf(TracedError);
    expect(error).toHaveProperty("message", "The page closed.");
    expect(logs.some((l) => l.startsWith("warn: Couldn't save the trace"))).toBe(true);
  });
});
