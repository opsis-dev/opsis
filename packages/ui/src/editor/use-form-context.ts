import { projectSwatches } from "@opsis/core/color";
import type { CompileResult } from "@opsis/core/compile/compile";
import { hasWidget, targetRoleKey } from "@opsis/core/elements/introspect";
import { downstreamOf, sessionInto, upstreamOf } from "@opsis/core/graph";
import { ACTION_NODE_TYPES, navigateNode, openBrowserNode } from "@opsis/core/nodes/browser";
import { connectedPorts } from "@opsis/core/nodes/card";
import type { NodeDef } from "@opsis/core/nodes/define";
import { applyGradeNode } from "@opsis/core/nodes/media";
import { convertInput, IN_PORTS, SESSION_PORT } from "@opsis/core/nodes/ports";
import { nodeName } from "@opsis/core/nodes/registry";
import { asValue, type BrowserSessionValue, type NodeResult } from "@opsis/core/schema/values";
import type { Workflow } from "@opsis/core/schema/workflow";
import { interpolate, PROJECT_NAME_VAR, SITE_ADDRESS_VAR, workflowVars } from "@opsis/core/vars";
import { useMemo } from "react";
import { toast } from "sonner";
import { attempt } from "@/lib/toast";
import { useRunStore } from "../store/run";
import { useWorkflowStore } from "../store/workflow";
import { fileUrl } from "./file-url";
import type { FormContext, SpotBackground } from "./form/types";
import { useCompiledContext } from "./use-compiled";
import type { VarInfo } from "./var-text";

const NO_PAGE = "Add a Go to page action before this one, so there's a page to pick from.";
const viewportParams = openBrowserNode.params.pick({ viewportWidth: true, viewportHeight: true });
const navigateUrl = navigateNode.params.pick({ url: true });

type PickerPage = { url: string; viewport?: { width: number; height: number } };

function variableList(wf: Workflow): VarInfo[] {
  return [
    ...Object.entries(wf.variables).map(([name, value]) => ({ name, value })),
    { name: PROJECT_NAME_VAR, value: workflowVars(wf)[PROJECT_NAME_VAR] },
  ];
}

function spotBackground(
  wf: Workflow,
  nodeId: string,
  results: Partial<Record<string, NodeResult>>,
): SpotBackground {
  for (const id of upstreamOf(wf, nodeId)) {
    const rec = results[id]?.outputs.recording;
    if (rec?.type === "recording" && rec.src)
      return { src: fileUrl(rec.src), timeMs: 0, durationMs: rec.durationMs };
  }
  return null;
}

function gradedFile(wf: Workflow, nodeId: string, values: CompileResult["values"]): SpotBackground {
  const later = downstreamOf(wf, nodeId);
  for (const apply of wf.nodes.filter((n) => n.type === applyGradeNode.type && later.has(n.id))) {
    const edge = wf.edges.find((e) => e.target === apply.id && e.targetHandle === "media");
    const value = edge && values[edge.source]?.[edge.sourceHandle];
    const media = asValue(convertInput(value, applyGradeNode.inputs.media), "media");
    if (media?.src && media.kind !== "audio")
      return {
        src: fileUrl(media.src),
        timeMs: 0,
        durationMs: media.durationMs ?? 0,
        ...(media.kind === "image" ? { still: true } : {}),
      };
  }
  return null;
}

function sessionAt(
  wf: Workflow,
  values: CompileResult["values"],
  nodeId: string,
): BrowserSessionValue | undefined {
  return (
    asValue(values[nodeId]?.[SESSION_PORT], "browser-session") ?? sessionInto(wf, values, nodeId)
  );
}

function viewportOf(session: BrowserSessionValue | undefined): PickerPage["viewport"] {
  const size = session ? viewportParams.safeParse(session.browser.params) : null;
  return size?.success
    ? { width: size.data.viewportWidth, height: size.data.viewportHeight }
    : undefined;
}

function pageUrl(session: BrowserSessionValue | undefined, wf: Workflow): string | undefined {
  const nav = session?.steps.findLast((s) => s.type === navigateNode.type);
  const navUrl = nav ? navigateUrl.safeParse(nav.params).data?.url : undefined;
  return navUrl ?? wf.variables[SITE_ADDRESS_VAR];
}

type PageResult = { ok: true; page: PickerPage } | { ok: false; problem: string };

function pickerPage(wf: Workflow, values: CompileResult["values"], nodeId: string): PageResult {
  const session = sessionAt(wf, values, nodeId);
  const url = pageUrl(session, wf);
  if (!url) return { ok: false, problem: NO_PAGE };
  const viewport = viewportOf(session);
  const page = { url: interpolate(url, workflowVars(wf)), ...(viewport ? { viewport } : {}) };
  return { ok: true, page };
}

async function pickTarget(
  wf: Workflow,
  values: CompileResult["values"],
  nodeId: string,
  key: string,
) {
  const found = pickerPage(wf, values, nodeId);
  if (!found.ok) {
    toast.error(found.problem);
    return;
  }
  const { page } = found;
  toast.info("A browser window opened. Click the element, or Shift-click a spot.");
  await attempt(async () => {
    const picked = await window.opsis.pickElement(page);
    if (!picked) return;
    useWorkflowStore.getState().updateParams(nodeId, {
      [key]: picked.target,
      [targetRoleKey(key)]: picked.role || undefined,
    });
    toast.success(`Picked “${picked.target}”`);
  }, "Couldn't pick the element");
}

async function chooseAsset(projectDir: string | null, apply: (path: string) => void) {
  if (!projectDir) return;
  await attempt(async () => {
    const [path] = await window.opsis.importAssets(projectDir);
    if (path) apply(path);
  }, "Couldn't add the file");
}

export function useFormContext(workflow: Workflow, nodeId: string, def: NodeDef): FormContext {
  const projectDir = useWorkflowStore((s) => s.projectDir);
  const results = useRunStore((s) => s.results);
  const { result: compiled } = useCompiledContext();
  return useMemo(
    () =>
      formContext({ workflow, nodeId, def, projectDir, results, values: compiled?.values ?? {} }),
    [workflow, nodeId, def, projectDir, results, compiled],
  );
}

function formContext({
  workflow,
  nodeId,
  def,
  projectDir,
  results,
  values,
}: {
  workflow: Workflow;
  nodeId: string;
  def: NodeDef;
  projectDir: string | null;
  results: Partial<Record<string, NodeResult>>;
  values: CompileResult["values"];
}): FormContext {
  const connectedInputs = workflow.edges
    .filter((e) => e.target === nodeId)
    .map((e) => e.targetHandle);
  const layerCount = connectedPorts(IN_PORTS, connectedInputs).length;
  const needsBackground =
    def.category === "Camera" || def.category === "Grade" || hasWidget(def.params, "boxes");
  return {
    nodeId,
    projectDir,
    fps: workflow.settings.fps,
    aspect: workflow.settings.width / workflow.settings.height,
    vars: variableList(workflow),
    layers: Math.max(1, layerCount),
    actions: workflow.nodes
      .filter((n) => ACTION_NODE_TYPES.has(n.type))
      .map((n) => ({ id: n.id, label: nodeName(n) })),
    spotBackground: needsBackground
      ? def.category === "Grade"
        ? gradedFile(workflow, nodeId, values)
        : spotBackground(workflow, nodeId, results)
      : null,
    pickTarget: (key) => void pickTarget(workflow, values, nodeId, key),
    chooseAsset: (apply) => void chooseAsset(projectDir, apply),
    swatches: projectSwatches(workflow.nodes.map((n) => n.params)),
    connectedInputs,
  };
}
