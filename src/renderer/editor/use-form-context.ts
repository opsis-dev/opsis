import { useMemo } from "react";
import { toast } from "sonner";
import { attempt } from "@/renderer/lib/toast";
import { projectSwatches } from "@/shared/color";
import { hasWidget, targetRoleKey } from "@/shared/elements/introspect";
import { errorMessage } from "@/shared/errors";
import { downstreamOf, sessionChain, upstreamOf } from "@/shared/graph";
import { ACTION_NODE_TYPES, navigateNode, sessionNode } from "@/shared/nodes/browser";
import { recordNode, stillNode } from "@/shared/nodes/capture";
import { connectedPorts } from "@/shared/nodes/card";
import type { NodeDef } from "@/shared/nodes/define";
import { IN_PORTS } from "@/shared/nodes/ports";
import { nodeName } from "@/shared/nodes/registry";
import type { NodeResult } from "@/shared/schema/values";
import type { Workflow, WorkflowNode } from "@/shared/schema/workflow";
import { interpolate, PROJECT_NAME_VAR, SITE_ADDRESS_VAR, workflowVars } from "@/shared/vars";
import { useRunStore } from "../store/run";
import { useSecretsStore } from "../store/secrets";
import { useWorkflowStore } from "../store/workflow";
import { fileUrl } from "./file-url";
import type { FormContext, SpotBackground } from "./form/types";
import type { VarInfo } from "./var-text";

const NO_PAGE = "Add a Go to page action before this one, so there's a page to pick from.";
const CAPTURE_TYPES = new Set([recordNode.type, stillNode.type]);
const viewportParams = sessionNode.params.pick({ viewportWidth: true, viewportHeight: true });
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
    if (rec?.type === "recording" && rec.src) return { src: fileUrl(rec.src), timeMs: 0 };
  }
  return null;
}

function captureChain(wf: Workflow, nodeId: string): { chain: WorkflowNode[]; problem?: string } {
  const captures = new Set(wf.nodes.filter((n) => CAPTURE_TYPES.has(n.type)).map((n) => n.id));
  const capture = [nodeId, ...downstreamOf(wf, nodeId)].find((id) => captures.has(id));
  if (!capture) return { chain: [] };
  try {
    return { chain: sessionChain(wf, capture) };
  } catch (error) {
    return { chain: [], problem: errorMessage(error) };
  }
}

function viewportOf(chain: WorkflowNode[]): PickerPage["viewport"] {
  const session = chain.find((n) => n.type === sessionNode.type);
  const size = session ? viewportParams.safeParse(session.params) : null;
  return size?.success
    ? { width: size.data.viewportWidth, height: size.data.viewportHeight }
    : undefined;
}

function pageUrlBefore(chain: WorkflowNode[], nodeId: string, wf: Workflow): string | undefined {
  const idx = chain.findIndex((n) => n.id === nodeId);
  const before = idx >= 0 ? chain.slice(0, idx + 1) : chain;
  const nav = before.findLast((n) => n.type === navigateNode.type);
  const navUrl = nav ? navigateUrl.safeParse(nav.params).data?.url : undefined;
  return navUrl ?? wf.variables[SITE_ADDRESS_VAR];
}

type PageResult = { ok: true; page: PickerPage } | { ok: false; problem: string };

function pickerPage(wf: Workflow, nodeId: string): PageResult {
  const { chain, problem } = captureChain(wf, nodeId);
  const url = pageUrlBefore(chain, nodeId, wf);
  if (!url) return { ok: false, problem: problem ?? NO_PAGE };
  const viewport = viewportOf(chain);
  const page = { url: interpolate(url, workflowVars(wf)), ...(viewport ? { viewport } : {}) };
  return { ok: true, page };
}

async function pickTarget(wf: Workflow, nodeId: string, key: string) {
  const found = pickerPage(wf, nodeId);
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
  const secrets = useSecretsStore((s) => s.names);
  const results = useRunStore((s) => s.results);
  return useMemo(
    () => formContext({ workflow, nodeId, def, projectDir, secrets, results }),
    [workflow, nodeId, def, projectDir, secrets, results],
  );
}

function formContext({
  workflow,
  nodeId,
  def,
  projectDir,
  secrets,
  results,
}: {
  workflow: Workflow;
  nodeId: string;
  def: NodeDef;
  projectDir: string | null;
  secrets: string[] | null;
  results: Partial<Record<string, NodeResult>>;
}): FormContext {
  const layerCount = connectedPorts(
    IN_PORTS,
    workflow.edges.filter((e) => e.target === nodeId).map((e) => e.targetHandle),
  ).length;
  const needsBackground = def.category === "Camera" || hasWidget(def.params, "boxes");
  return {
    projectDir,
    fps: workflow.settings.fps,
    aspect: workflow.settings.width / workflow.settings.height,
    vars: variableList(workflow),
    secrets,
    layers: Math.max(1, layerCount),
    actions: workflow.nodes
      .filter((n) => ACTION_NODE_TYPES.has(n.type))
      .map((n) => ({ id: n.id, label: nodeName(n) })),
    spotBackground: needsBackground ? spotBackground(workflow, nodeId, results) : null,
    pickTarget: (key) => void pickTarget(workflow, nodeId, key),
    chooseAsset: (apply) => void chooseAsset(projectDir, apply),
    swatches: projectSwatches(workflow.nodes.map((n) => n.params)),
  };
}
