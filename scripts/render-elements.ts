import { mkdirSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { userEngineDir } from "../src/engine/paths";
import { makeWorkerTemp, removeWorkerTemp, tempEnv } from "../src/main/worker-temp";
import { emptyClip, itemClip, clipToSpec } from "../src/shared/compile/clip";
import { ELEMENTS, elementNodeType } from "../src/shared/elements";
import type { ElementMeta } from "../src/shared/elements/types";
import { errorMessage } from "../src/shared/errors";
import type { ClipValue } from "../src/shared/schema/values";
import { DEFAULT_WORKFLOW_SETTINGS } from "../src/shared/schema/workflow";
import { appSettingsSchema } from "../src/shared/settings";
import { compileClip } from "../src/shared/test-fixtures";
import type { WorkerEnv } from "../src/shared/worker-protocol";
import { renderCardStill } from "../src/worker/render/card-still";
import { devUserData } from "./lib/app-data";
import { ROOT } from "./lib/tool-config";

const OUT = path.join(os.homedir(), ".cache", "opsis-element-stills");
const PROJECT = path.join(OUT, "project");
const STILL_WIDTH = 480;
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
    await renderCardStill(
      {
        type: "still",
        env,
        id: meta.key,
        projectDir: PROJECT,
        spec,
        frame: Math.floor((clip.durationInFrames - 1) * at),
        output,
        width: STILL_WIDTH,
      },
      new AbortController().signal,
    );
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
removeWorkerTemp(temp);
process.exit();
