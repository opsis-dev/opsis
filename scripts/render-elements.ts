import { mkdirSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { emptyClip, itemClip, clipToSpec } from "@opsis/core/compile/clip";
import { ELEMENTS, elementNodeType } from "@opsis/core/elements";
import type { ElementMeta } from "@opsis/core/elements/types";
import { errorMessage } from "@opsis/core/errors";
import type { ClipValue } from "@opsis/core/schema/values";
import { DEFAULT_WORKFLOW_SETTINGS } from "@opsis/core/schema/workflow";
import { appSettingsSchema } from "@opsis/core/settings";
import { compileClip } from "@opsis/core/test-fixtures";
import type { WorkerEnv } from "@opsis/core/worker-protocol";
import {
  getServeUrl,
  openRenderBrowser,
  renderFrame,
  type GlRenderer,
} from "@opsis/sidecar/render";
import { enginePaths, requireHeadlessShell, userEngineDir } from "../src/engine/paths";
import { makeWorkerTemp, removeWorkerTemp, tempEnv } from "../src/main/worker-temp";
import { startAssetServer } from "../src/worker/render/asset-server";
import { devUserData } from "./lib/app-data";
import { ROOT } from "./lib/tool-config";

const OUT = path.join(os.homedir(), ".cache", "opsis-element-stills");
const PROJECT = path.join(OUT, "project");
const STILL_WIDTH = 480;
const GL: GlRenderer = "angle";
const JPEG_QUALITY = 82;
const SOLID_FRAMES = 90;
const only = process.argv[2];
const at = Number(process.argv[3] ?? 0.5);

const solid = (id: string, color: string): ClipValue =>
  itemClip(id, "core/solid", { color }, SOLID_FRAMES);

function inputsFor(meta: ElementMeta): Record<string, ClipValue> {
  if (meta.kind === "presentation") return { a: solid("a", "#334155"), b: solid("b", "#7c3aed") };
  if (meta.kind === "wrapper") return { clip: solid("in", "#334155") };
  if (meta.kind === "sequencer")
    return { in1: emptyClip(SOLID_FRAMES), in2: emptyClip(SOLID_FRAMES) };
  return {};
}

const userData = process.env.OPSIS_USER_DATA ?? devUserData();
const env: WorkerEnv = {
  appRoot: ROOT,
  appVersion: "dev",
  isPackaged: false,
  resourcesPath: path.join(ROOT, "resources"),
  userData,
  engineDir: userEngineDir(userData),
  settings: appSettingsSchema.parse({}),
};

mkdirSync(PROJECT, { recursive: true });
const temp = makeWorkerTemp(path.join(OUT, "tmp"), String(process.pid));
Object.assign(process.env, tempEnv(temp));
const serveUrl = await getServeUrl(env);
const server = await startAssetServer(PROJECT);
const browserExecutable = requireHeadlessShell(enginePaths(env.engineDir));
const browser = await openRenderBrowser(browserExecutable, GL);
const failures: string[] = [];
const elements = ELEMENTS.filter(
  (m) => !m.key.startsWith("core/") && (!only || m.key.includes(only)),
);
for (const meta of elements) {
  const output = path.join(OUT, `${meta.key.replace("/", "--")}.jpg`);
  try {
    const type =
      meta.kind === "presentation"
        ? `transition.${meta.key.replace("/", ".")}`
        : elementNodeType(meta.key);
    const clip = compileClip(type, {}, inputsFor(meta));
    const spec = clipToSpec(clip, DEFAULT_WORKFLOW_SETTINGS);
    await renderFrame({
      env,
      browserExecutable,
      serveUrl,
      spec: { ...spec, assets: { projectBase: server.url } },
      frame: Math.floor((clip.durationInFrames - 1) * at),
      output,
      format: "jpeg",
      gl: GL,
      signal: new AbortController().signal,
      browser,
      scale: Math.min(1, STILL_WIDTH / spec.width),
      jpegQuality: JPEG_QUALITY,
    });
    console.log(`ok   ${meta.key}`);
  } catch (error) {
    failures.push(`${meta.key}: ${errorMessage(error).slice(0, 2000)}`);
    console.log(`FAIL ${meta.key}`);
  }
}
console.log(`\n${elements.length - failures.length} of ${elements.length} rendered into ${OUT}`);
if (failures.length) {
  console.error(failures.join("\n"));
  process.exitCode = 1;
}
await browser.close({ silent: true });
await server.close();
removeWorkerTemp(temp);
process.exit();
