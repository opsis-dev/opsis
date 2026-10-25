import { mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { catalogEntry, nodeCatalog } from "@opsis/core/api/catalog";
import { workflowProblems } from "@opsis/core/api/check";
import { inputTake, isClip, recordingTake, sceneItemCount } from "@opsis/core/compile/clip";
import { compileWorkflow, previewSpec, type CompileResult } from "@opsis/core/compile/compile";
import { ELEMENTS } from "@opsis/core/elements";
import { errorMessage } from "@opsis/core/errors";
import { connectionProblem, runTargets, sessionInto, validateWorkflow } from "@opsis/core/graph";
import { buildMixPlan } from "@opsis/core/mix/plan";
import { namedProjectFiles } from "@opsis/core/named-files";
import type { CardContext, CardRow, InputTake } from "@opsis/core/nodes/card";
import {
  cardParams,
  cardRows,
  type CompileInputs,
  type NodeDef,
  type PortSpec,
} from "@opsis/core/nodes/define";
import { parseParams } from "@opsis/core/nodes/params";
import { defaultParams, getNodeDef, NODE_DEFS } from "@opsis/core/nodes/registry";
import { estimateCardHeight } from "@opsis/core/nodes/size";
import type { Action } from "@opsis/core/schema/actions";
import type { CompileValue, NodeResult, StyleEntry } from "@opsis/core/schema/values";
import {
  DEFAULT_WORKFLOW_SETTINGS,
  workflowSchema,
  type PortType,
  type Workflow,
  type WorkflowNode,
} from "@opsis/core/schema/workflow";
import { TEMPLATES } from "@opsis/core/templates";
import { compileClip, nodeResult, takeRecording, makeLog } from "@opsis/core/test-fixtures";
import { filmChapters } from "@opsis/core/timeline/chapters";
import { flattenSpec } from "@opsis/core/timeline/flatten";
import { workflowVars } from "@opsis/core/vars";
import type { z } from "zod";
import { captureFixtures } from "./lib/capture-fixtures";
import { mediaFixtures } from "./lib/media-fixtures";
import { writeSourceLayoutJson } from "./lib/source-layout-json";
import { ROOT } from "./lib/tool-config";
import { zodTree } from "./lib/zod-tree";

export const FIXTURES_DIR = path.join(ROOT, "fixtures");

const SECRET_NAMES = ["Main account"];

const ACTION_KINDS: Partial<Record<string, Action["kind"]>> = {
  "browser.click": "click",
  "browser.type": "type",
  "browser.hover": "hover",
  "browser.scroll": "scroll",
  "browser.navigate": "navigate",
  "browser.wait": "wait",
  "browser.chapter": "chapter",
  "browser.pressKeys": "key",
  "browser.drag": "drag",
  "browser.select": "click",
  "browser.upload": "click",
};

const STEP_MS = 900;

function sessionLog(steps: ReadonlyArray<{ id: string; type: string; label?: string }>) {
  const actions: Action[] = [];
  steps.forEach((step, i) => {
    const kind = ACTION_KINDS[step.type];
    if (!kind) return;
    const t = 600 + i * STEP_MS;
    const x = 200 + ((i * 173) % 1400);
    const y = 150 + ((i * 97) % 800);
    actions.push({
      id: `${step.id}:${kind}`,
      kind,
      t,
      tEnd: t + 400,
      point: { x, y },
      rect: { x: x - 60, y: y - 20, w: 120, h: 40 },
      label: step.label ?? step.type,
      nodeId: step.id,
    });
  });
  return makeLog(actions, 600 + Math.max(1, steps.length) * STEP_MS + 800);
}

function syntheticValue(type: PortType, id: string): CompileValue {
  switch (type) {
    case "recording":
      return { ...takeRecording(sessionLog([])), nodeId: id };
    case "image":
      return { type: "image", src: `project:shots/${id}.png`, width: 1920, height: 1080, scale: 2 };
    case "media":
      return {
        type: "media",
        src: `project:.opsis/out/${id}.mp4`,
        kind: "video",
        durationMs: 4000,
        width: 1920,
        height: 1080,
        hasAudio: true,
      };
    case "audio":
      return {
        type: "audio",
        items: [{ id: `${id}:sound`, src: "project:sound.mp3", from: 0, durationInFrames: 90 }],
        durationInFrames: 90,
      };
    case "captions":
      return {
        type: "captions",
        captions: [
          { text: "Hello there", startMs: 0, endMs: 1200 },
          { text: "and welcome", startMs: 1200, endMs: 2600 },
        ],
      };
    case "file-info":
      return { type: "file-info", src: `project:${id}.json` };
    case "folder":
      return { type: "folder", src: `project:.opsis/out/${id}` };
    case "clip":
      return compileClip("fx.core.title", { title: `Clip ${id}` });
    case "browser-session":
      return {
        type: "browser-session",
        browser: { nodeId: "session", params: defaultParams("browser.open") },
        steps: [
          { id: "open", type: "browser.navigate", params: { url: "https://example.com" } },
          { id: "press", type: "browser.click", label: "Press", params: { target: "Sign in" } },
        ],
      };
    case "recording-style":
      return { type: "recording-style", entries: [] };
    case "grade":
      return { type: "grade", entries: [] };
  }
}

function syntheticResult(wf: Workflow, compiled: CompileResult, node: WorkflowNode): NodeResult {
  const def = getNodeDef(node.type);
  const outputs: Record<string, CompileValue> = {};
  for (const [port, spec] of Object.entries(def?.outputs ?? {})) {
    if (spec.type === "recording") {
      const session = sessionInto(wf, compiled.values, node.id);
      const log = sessionLog(session?.steps ?? []);
      outputs[port] = {
        type: "recording",
        src: `project:.opsis/cache/${node.id}/recording.mp4`,
        proxySrc: `project:.opsis/cache/${node.id}/recording.preview.mp4`,
        log,
        durationMs: log.durationMs,
        viewport: log.viewport,
        nodeId: node.id,
        style: [],
      };
    } else outputs[port] = syntheticValue(spec.type, node.id);
  }
  return nodeResult({ hash: `hash-${node.id}`, outputs, finishedAt: 1, durationMs: 2 });
}

type CardInputs = Omit<CardContext<unknown>, "params" | "missingFiles">;

function cardInputs(
  wf: Workflow,
  compiled: CompileResult,
  node: WorkflowNode,
  result: NodeResult | undefined,
): CardInputs {
  const incoming = wf.edges.filter((e) => e.target === node.id);
  const clip = compiled.values[node.id]?.clip;
  const frames: Partial<Record<string, number>> = {};
  const items: Partial<Record<string, number>> = {};
  const takes: Partial<Record<string, InputTake>> = {};
  const media: NonNullable<CardContext<unknown>["inputMedia"]> = {};
  const captions: Partial<Record<string, number>> = {};
  const steps: Partial<Record<string, string[]>> = {};
  const style: Partial<Record<string, string[]>> = {};
  const grade: Partial<Record<string, StyleEntry[]>> = {};
  for (const e of incoming) {
    const v = compiled.values[e.source]?.[e.sourceHandle];
    if (v?.type === "media") media[e.targetHandle] = { kind: v.kind, hasAudio: v.hasAudio };
    if (v?.type === "captions") captions[e.targetHandle] = v.captions.at(-1)?.endMs ?? 0;
    if (v?.type === "browser-session") steps[e.targetHandle] = v.steps.map((s) => s.type);
    if (v?.type === "recording-style") style[e.targetHandle] = v.entries.map((s) => s.type);
    if (v?.type === "grade") grade[e.targetHandle] = v.entries;
    if (v?.type === "recording") {
      const take = recordingTake(v);
      if (take !== undefined) takes[e.targetHandle] = take;
    }
    if (!isClip(v)) continue;
    frames[e.targetHandle] = v.durationInFrames;
    items[e.targetHandle] = sceneItemCount(v);
    const take = inputTake(v);
    if (take !== undefined) takes[e.targetHandle] = take;
  }
  return {
    settings: wf.settings,
    result,
    secrets: SECRET_NAMES,
    connected: incoming.map((e) => e.targetHandle),
    clipFrames: isClip(clip) ? clip.durationInFrames : undefined,
    inputFrames: frames,
    inputItems: items,
    inputTakes: takes,
    inputMedia: media,
    inputCaptionsMs: captions,
    inputSteps: steps,
    inputStyle: style,
    inputGrade: grade,
  };
}

type CardOutcome = { rows: CardRow[] } | { error: string };

function cardOf(def: NodeDef, params: unknown, inputs: CardInputs, vars: Record<string, string>) {
  if (!def.card) return { rows: [] };
  try {
    const shown = cardParams(def, params as Record<string, unknown>, vars);
    const missingFiles = namedProjectFiles(shown).filter((_, i) => i % 2 === 1);
    return { rows: cardRows(def, { ...inputs, missingFiles, params: shown }) };
  } catch (error) {
    return { error: errorMessage(error) };
  }
}

function workflowCards(
  wf: Workflow,
  compiled: CompileResult,
  results: Partial<Record<string, NodeResult>>,
): Record<string, CardOutcome> {
  const vars = workflowVars(wf);
  const out: Record<string, CardOutcome> = {};
  for (const node of wf.nodes) {
    const def = getNodeDef(node.type);
    if (!def) continue;
    out[node.id] = cardOf(def, node.params, cardInputs(wf, compiled, node, results[node.id]), vars);
  }
  return out;
}

function timelines(compiled: CompileResult, mix?: Workflow["mix"]) {
  return Object.fromEntries(
    Object.entries(compiled.specs).map(([id, spec]) => {
      const flat = flattenSpec(spec);
      return [id, { flat, chapters: filmChapters(spec), mix: buildMixPlan(spec, mix, flat) }];
    }),
  );
}

function workflowFixture(wf: Workflow) {
  const compiled = compileWorkflow(wf);
  const results: Record<string, NodeResult> = {};
  for (const node of wf.nodes)
    if (getNodeDef(node.type)?.executes && !node.disabled)
      results[node.id] = syntheticResult(wf, compiled, node);
  const withResults = compileWorkflow(wf, { results });
  return {
    workflow: wf,
    compile: compiled,
    problems: validateWorkflow(wf, compiled),
    cards: workflowCards(wf, compiled, {}),
    preview: previewSpec(wf.settings, compiled, null),
    timelines: timelines(compiled, wf.mix),
    run: {
      results,
      compile: withResults,
      problems: validateWorkflow(wf, withResults),
      cards: workflowCards(wf, withResults, results),
      preview: previewSpec(wf.settings, withResults, null),
      timelines: timelines(withResults, wf.mix),
    },
  };
}

function soundWorkflow(base: Workflow): Workflow {
  const node = (id: string, type: string, params: Record<string, unknown>, x: number) => ({
    id,
    type,
    position: { x, y: 900 },
    params,
  });
  const edge = (source: string, sourceHandle: string, target: string, targetHandle: string) => ({
    id: `sound-${source}-${target}-${targetHandle}`,
    source,
    sourceHandle,
    target,
    targetHandle,
  });
  return workflowSchema.parse({
    ...base,
    id: "sound",
    nodes: [
      ...base.nodes,
      node("talk", "media.import", { path: "clips/talk.mp4" }, 0),
      node("talk-clip", "media.toClip", {}, 100),
      node("layers", "timeline.stack", { offsetsMs: "0, 400" }, 200),
      node(
        "song",
        "audio.music",
        { src: "music/song.mp3", startMs: 500, trimMs: 200, fadeInMs: 400 },
        300,
      ),
      node(
        "whoosh",
        "audio.sfx",
        { src: "app:sfx/whoosh.wav", atAction: "complete", offsetMs: -200 },
        400,
      ),
      node("whip", "audio.sfx", { src: "app:sfx/whip.wav", atMs: 1000, keepMusic: true }, 500),
      node(
        "loop",
        "audio.music",
        { src: "music/bed.mp3", loop: true, lengthMs: 3000, volume: 0.3 },
        600,
      ),
      node("mix", "audio.mix", {}, 700),
    ],
    edges: [
      ...base.edges.filter((e) => e.target !== "render"),
      edge("talk", "media", "talk-clip", "media"),
      edge("sequence", "clip", "layers", "in1"),
      edge("talk-clip", "clip", "layers", "in2"),
      edge("layers", "clip", "mix", "clip"),
      edge("song", "audio", "mix", "sounds"),
      edge("whoosh", "audio", "mix", "sounds"),
      edge("whip", "audio", "mix", "sounds"),
      edge("loop", "audio", "mix", "sounds"),
      edge("mix", "clip", "render", "clip"),
    ],
    mix: {
      channels: { song: { fader: -6 }, loop: { mute: true }, whip: { bus: "music" } },
      buses: [
        {
          id: "music",
          name: "Music",
          duck: { on: true, keys: ["effects", "clips"], ignoreShorterMs: 0 },
        },
        { id: "effects", name: "Effects" },
        { id: "clips", name: "Clips", fader: -3 },
      ],
    },
  });
}

function exampleWorkflows(): Array<[string, Workflow]> {
  const dir = path.join(ROOT, "examples");
  return readdirSync(dir).map((name) => [
    `example-${name}`,
    workflowSchema.parse(
      JSON.parse(readFileSync(path.join(dir, name, "project.opsis.json"), "utf8")),
    ),
  ]);
}

const templateWorkflows = (): Array<[string, Workflow]> =>
  TEMPLATES.map((t) => [
    `template-${t.id}`,
    workflowSchema.parse({ ...t.build(t.title), id: `template-${t.id}` }),
  ]);

const ports = (specs: Record<string, PortSpec>) =>
  Object.entries(specs).map(([id, p]) => ({
    id,
    type: p.type,
    label: p.label ?? null,
    optional: !!p.optional,
    many: !!p.many,
  }));

const PLATFORMS: NodeJS.Platform[] = ["linux", "darwin", "win32"];

function nodeFixture(def: NodeDef) {
  return {
    type: def.type,
    title: def.title,
    category: def.category,
    description: def.description ?? null,
    phase: def.phase,
    inputs: ports(def.inputs),
    outputs: ports(def.outputs),
    virtual: !!def.virtual,
    action: !!def.action,
    executes: !!def.executes,
    secretParams: def.secretParams ?? [],
    usesProjectSize: !!def.usesProjectSize,
    element: def.element ?? null,
    hidden: !!def.hidden,
    cardHeight: def.cardHeight ?? null,
    estimatedHeight: estimateCardHeight(def),
    compiles: !!def.compile,
    hasCard: !!def.card,
    defaults: defaultParams(def.type),
    mutates: def.mutates ? (def.mutates(def.params.parse(defaultParams(def.type))) ?? null) : null,
    hostDefaults: def.hostDefaults
      ? Object.fromEntries(PLATFORMS.map((p) => [p, def.hostDefaults?.(p) ?? {}]))
      : null,
    params: zodTree(def.params),
  };
}

const elementFixture = (meta: (typeof ELEMENTS)[number]) => ({
  key: meta.key,
  title: meta.title,
  description: meta.description ?? null,
  category: meta.category,
  kind: meta.kind,
  source: meta.source,
  defaultDurationInFrames: meta.defaultDurationInFrames,
  design: meta.design ?? null,
  optionalClip: !!meta.optionalClip,
  thumbnail: meta.thumbnail ?? null,
  defaultProps: meta.defaultProps,
  fixedProps: meta.fixedProps,
  schema: zodTree(meta.schema),
});

function nodeCase(def: NodeDef, filled: boolean) {
  const inputs: CompileInputs = {};
  if (filled)
    for (const [id, spec] of Object.entries(def.inputs)) inputs[id] = syntheticValue(spec.type, id);
  const wf: Workflow = workflowSchema.parse({
    version: 1,
    id: "case",
    name: "Case",
    nodes: [],
    edges: [],
  });
  const result = def.executes
    ? nodeResult({
        hash: "case",
        outputs: Object.fromEntries(
          Object.entries(def.outputs).map(([id, spec]) => [
            id,
            syntheticValue(spec.type, `out-${id}`),
          ]),
        ),
      })
    : undefined;
  let outputs: unknown = null;
  let error: string | null = null;
  try {
    const params = def.params.parse(defaultParams(def.type));
    outputs =
      def.compile?.({
        nodeId: "n",
        type: def.type,
        params,
        inputs,
        settings: DEFAULT_WORKFLOW_SETTINGS,
        vars: {},
        workflow: wf,
        result,
      }) ??
      result?.outputs ??
      {};
  } catch (e) {
    error = errorMessage(e);
  }
  const connected = filled ? Object.keys(def.inputs) : [];
  const inputFrames = Object.fromEntries(
    Object.entries(inputs).flatMap(([id, v]) => (isClip(v) ? [[id, v.durationInFrames]] : [])),
  );
  const card = cardOf(
    def,
    defaultParams(def.type),
    { settings: DEFAULT_WORKFLOW_SETTINGS, result, secrets: SECRET_NAMES, connected, inputFrames },
    {},
  );
  return {
    type: def.type,
    filled,
    inputs,
    result: result ?? null,
    secrets: SECRET_NAMES,
    connected,
    inputFrames,
    outputs,
    error,
    card,
  };
}

type Case = {
  label: string;
  key: string | null;
  change: { set: unknown } | { unset: true } | null;
};

function unwrapped(schema: z.ZodType): z.ZodType {
  let current = schema;
  for (;;) {
    const def = current.def as unknown as { type: string; innerType?: z.ZodType };
    if (!def.innerType) return current;
    current = def.innerType;
  }
}

type CheckDef = Record<string, unknown>;

function fieldCases(key: string, field: z.ZodType): Case[] {
  const inner = unwrapped(field);
  const def = inner.def as unknown as { type: string; checks?: Array<{ _zod: { def: CheckDef } }> };
  const set = (label: string, value: unknown): Case => ({ label, key, change: { set: value } });
  const cases: Case[] = [{ label: "missing", key, change: { unset: true } }, set("null", null)];
  const bounds = (def.checks ?? []).map((c) => c._zod.def);
  switch (def.type) {
    case "number":
      cases.push(
        set("text", "abc"),
        set("variable", "{{Speed}}"),
        set("fraction", 0.5),
        set("huge", 1e12),
      );
      for (const b of bounds) {
        if (b.check === "greater_than")
          cases.push(set("below", Number(b.value) - 1), set("at low bound", b.value));
        if (b.check === "less_than")
          cases.push(set("above", Number(b.value) + 1), set("at high bound", b.value));
      }
      break;
    case "string":
      cases.push(
        set("number", 12),
        set("empty", ""),
        set("spaces", "   "),
        set("bad colour", "nope"),
        set("variable", "{{Product name}}"),
      );
      break;
    case "boolean":
      cases.push(set("text", "yes"));
      break;
    case "enum":
    case "literal":
    case "union":
      cases.push(set("other", "zz-other"), set("number", 45));
      break;
    case "array":
      cases.push(
        set("not a list", "x"),
        set("empty", []),
        set(
          "many",
          Array.from({ length: 40 }, () => ({})),
        ),
      );
      break;
    case "object":
      cases.push(set("not an object", "x"), set("empty", {}));
      break;
    default:
      break;
  }
  return cases;
}

function applied(base: Record<string, unknown>, c: Case): Record<string, unknown> {
  if (c.key === null || c.change === null) return c.label === "empty" ? {} : base;
  const values = { ...base };
  if ("unset" in c.change) delete values[c.key];
  else values[c.key] = c.change.set;
  return values;
}

const REQUIRED_SAMPLE = "Sign in";

function requiredSamples(shape: Record<string, z.ZodType>): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  for (const [key, field] of Object.entries(shape))
    if (!field.safeParse(undefined).success && unwrapped(field).def.type === "string")
      out[key] = REQUIRED_SAMPLE;
  return out;
}

function paramCases() {
  const out: Array<Case & { type: string; outcome: unknown }> = [];
  const bases: Record<string, Record<string, unknown>> = {};
  for (const def of NODE_DEFS) {
    const shape = def.params.shape as Record<string, z.ZodType>;
    const base = { ...requiredSamples(shape), ...defaultParams(def.type) };
    bases[def.type] = base;
    const cases: Case[] = [
      { label: "defaults", key: null, change: null },
      { label: "empty", key: null, change: null },
    ];
    for (const [key, field] of Object.entries(shape)) cases.push(...fieldCases(key, field));
    for (const c of cases) {
      const values = applied(base, c);
      const parsed = parseParams(def.params, values, values);
      const outcome = !parsed.ok
        ? parsed
        : c.key === null
          ? parsed
          : {
              ok: true,
              value: (parsed.data as Record<string, unknown>)[c.key] ?? null,
              keys: Object.keys(parsed.data as object).length,
            };
      out.push({ type: def.type, ...c, outcome });
    }
  }
  return { bases, cases: out };
}

type CaseNode = {
  id: string;
  type: string;
  params?: Record<string, unknown>;
  disabled?: boolean;
  label?: string;
};
type CaseEdge = [string, string, string, string];

function caseWorkflow(
  nodes: CaseNode[],
  edges: CaseEdge[] = [],
  variables: Record<string, string> = {},
): Workflow {
  return workflowSchema.parse({
    version: 1,
    id: "problems",
    name: "Problems",
    variables,
    nodes: nodes.map((n, i) => ({ position: { x: i * 100, y: 0 }, params: {}, ...n })),
    edges: edges.map(([source, sourceHandle, target, targetHandle], i) => ({
      id: `e${i}`,
      source,
      sourceHandle,
      target,
      targetHandle,
    })),
  });
}

const open = (id: string, params: Record<string, unknown> = {}): CaseNode => ({
  id,
  type: "browser.open",
  params,
});
const record = (id: string): CaseNode => ({ id, type: "capture.record" });
const zoom = (id: string): CaseNode => ({ id, type: "camera.autoZoom" });
const render = (id: string, fileName: string): CaseNode => ({
  id,
  type: "output.render",
  params: { fileName },
});

const PROBLEM_CASES: Record<string, Workflow> = {
  unknownType: caseWorkflow([{ id: "x", type: "nope.nothing" }]),
  sharedId: caseWorkflow([open("a"), open("a")]),
  badSetting: caseWorkflow([open("a", { viewportWidth: 100 })]),
  numberVariable: caseWorkflow([open("a", { dpr: "{{Sharpness}}" })], [], { Sharpness: "2" }),
  unknownVariable: caseWorkflow([open("a", { userAgent: "{{Agent}}" })]),
  missingInput: caseWorkflow([record("r")]),
  doubleInput: caseWorkflow(
    [open("a"), open("b"), record("r")],
    [
      ["a", "session", "r", "session"],
      ["b", "session", "r", "session"],
    ],
  ),
  deletedNode: caseWorkflow([record("r")], [["gone", "session", "r", "session"]]),
  missingPort: caseWorkflow([open("a"), record("r")], [["a", "nope", "r", "session"]]),
  wrongType: caseWorkflow([open("a"), zoom("z")], [["a", "session", "z", "recording"]]),
  loop: caseWorkflow(
    [
      { id: "d1", type: "timeline.delay" },
      { id: "d2", type: "timeline.delay" },
    ],
    [
      ["d1", "clip", "d2", "clip"],
      ["d2", "clip", "d1", "clip"],
    ],
  ),
  reloadFirst: caseWorkflow(
    [open("a"), { id: "s", type: "browser.setState", params: { reload: true } }, record("r")],
    [
      ["a", "session", "s", "session"],
      ["s", "session", "r", "session"],
    ],
  ),
  goneAction: caseWorkflow([
    { id: "fx", type: "audio.sfx", params: { atAction: "missing", src: "app:sfx/whoosh.wav" } },
  ]),
  exportClash: caseWorkflow(
    [open("a"), record("r"), zoom("z"), render("v1", "film"), render("v2", "film")],
    [
      ["a", "session", "r", "session"],
      ["r", "recording", "z", "recording"],
      ["z", "clip", "v1", "clip"],
      ["z", "clip", "v2", "clip"],
    ],
  ),
  signInClash: caseWorkflow(
    [
      open("a"),
      { id: "s1", type: "browser.signIn" },
      { id: "s2", type: "browser.signIn", label: "Second" },
    ],
    [
      ["a", "session", "s1", "session"],
      ["s1", "session", "s2", "session"],
    ],
  ),
  skippedSource: caseWorkflow(
    [{ ...open("a"), disabled: true }, record("r")],
    [["a", "session", "r", "session"]],
  ),
};

function problemCases() {
  return Object.fromEntries(
    Object.entries(PROBLEM_CASES).map(([name, wf]) => [
      name,
      {
        workflow: wf,
        problems: validateWorkflow(wf),
        checked: workflowProblems(wf),
        runTargets: runTargets(wf),
      },
    ]),
  );
}

function connectionCases() {
  const wf = PROBLEM_CASES.exportClash;
  const ends: Array<[string, string, string, string]> = [
    ["a", "session", "r", "session"],
    ["gone", "session", "r", "session"],
    ["a", "session", "a", "session"],
    ["a", "nope", "r", "session"],
    ["a", "session", "r", "nope"],
    ["a", "session", "z", "recording"],
  ];
  return ends.map(([source, sourceHandle, target, targetHandle]) => ({
    ends: { source, sourceHandle, target, targetHandle },
    problem: connectionProblem(wf, { source, sourceHandle, target, targetHandle }),
  }));
}

export function parityFixtures(): Record<string, unknown> {
  const files: Record<string, unknown> = {
    "compile/nodes.json": NODE_DEFS.map(nodeFixture),
    "compile/elements.json": ELEMENTS.map(elementFixture),
    "compile/catalog.json": {
      list: nodeCatalog(),
      entries: NODE_DEFS.map((d) => catalogEntry(d.type)),
    },
    "compile/node-cases.json": NODE_DEFS.flatMap((d) => [nodeCase(d, false), nodeCase(d, true)]),
    "compile/param-cases.json": paramCases(),
    "compile/problem-cases.json": { workflows: problemCases(), connections: connectionCases() },
  };
  const todomvc = exampleWorkflows().find(([name]) => name === "example-todomvc");
  const extra: Array<[string, Workflow]> = todomvc ? [["sound", soundWorkflow(todomvc[1])]] : [];
  const workflows = [...exampleWorkflows(), ...templateWorkflows(), ...extra];
  for (const [name, wf] of workflows) files[`compile/workflows/${name}.json`] = workflowFixture(wf);
  return { ...files, ...mediaFixtures(workflows), ...captureFixtures() };
}

export const fixtureText = (value: unknown): string => `${JSON.stringify(value, null, 1)}\n`;

if (process.argv[1] && import.meta.filename === path.resolve(process.argv[1])) {
  rmSync(FIXTURES_DIR, { recursive: true, force: true });
  for (const [name, value] of Object.entries(parityFixtures())) {
    const file = path.join(FIXTURES_DIR, name);
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(file, fixtureText(value));
  }
  console.log(`wrote ${FIXTURES_DIR}`);
  writeSourceLayoutJson();
}
