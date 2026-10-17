import { useMemo } from "react";
import { toast } from "sonner";
import { PROJECT_NAME_VAR, workflowVars } from "@/shared/compile/compile";
import { hasWidget, targetRoleKey } from "@/shared/elements/introspect";
import { errorMessage } from "@/shared/errors";
import { downstreamOf, sessionChain, upstreamOf } from "@/shared/graph";
import { ACTION_NODE_TYPES, navigateNode, sessionNode } from "@/shared/nodes/browser";
import { recordNode, stillNode } from "@/shared/nodes/capture";
import { countConnected } from "@/shared/nodes/card";
import type { NodeDef } from "@/shared/nodes/define";
import { nodeName } from "@/shared/nodes/registry";
import { IN_PORTS } from "@/shared/nodes/timeline";
import type { NodeResult } from "@/shared/schema/values";
import type { Workflow, WorkflowNode } from "@/shared/schema/workflow";
import { interpolate } from "@/shared/vars";
import { useRunStore } from "../store/run";
import { useSecretsStore } from "../store/secrets";
import { useWorkflowStore } from "../store/workflow";
import { fileUrl } from "./file-url";
import type { FormContext, SpotBackground } from "./form/types";
import type { VarInfo } from "./var-text";
import { reportError } from "@/renderer/lib/toast";

const SITE_VAR = "Site address";
const NO_PAGE = "Add a Go to page action before this one, so there's a page to pick from.";
const CAPTURE_TYPES = new Set([recordNode.type, stillNode.type]);
const viewportParams = sessionNode.params.pick({ viewportWidth: true, viewportHeight: true });

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
  const types = new Map(wf.nodes.map((n) => [n.id, n.type]));
  const capture = [nodeId, ...downstreamOf(wf, nodeId)].find((id) =>
    CAPTURE_TYPES.has(types.get(id) ?? ""),
  );
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

function pickerPage(wf: Workflow, nodeId: string): PickerPage | { problem: string } {
  const { chain, problem } = captureChain(wf, nodeId);
  const idx = chain.findIndex((n) => n.id === nodeId);
  const before = idx >= 0 ? chain.slice(0, idx + 1) : chain;
  const navUrl = before.toReversed().find((n) => n.type === navigateNode.type)?.params.url;
  const url =
    (typeof navUrl === "string" ? navUrl : undefined) ??
    wf.variables[SITE_VAR] ??
    wf.variables.BASE_URL;
  if (!url) return { problem: problem ?? NO_PAGE };
  const viewport = viewportOf(chain);
  return { url: interpolate(url, workflowVars(wf)), ...(viewport ? { viewport } : {}) };
}

async function pickTarget(wf: Workflow, nodeId: string, key: string) {
  const page = pickerPage(wf, nodeId);
  if ("problem" in page) {
    toast.error(page.problem);
    return;
  }
  toast.info("A browser window opened. Click the element, or Shift-click a spot.");
  try {
    const picked = await window.demoflow.pickElement(page);
    if (!picked) return;
    useWorkflowStore.getState().updateParams(nodeId, {
      [key]: picked.target,
      [targetRoleKey(key)]: picked.role || undefined,
    });
    toast.success(`Picked “${picked.target}”`);
  } catch (e) {
    reportError("Couldn't pick the element", e);
  }
}

async function chooseAsset(projectDir: string | null, nodeId: string, key: string) {
  if (!projectDir) return;
  const [path] = await window.demoflow.importAssets(projectDir);
  if (path) useWorkflowStore.getState().updateParams(nodeId, { [key]: path });
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
  const layers = countConnected(
    IN_PORTS,
    workflow.edges.filter((e) => e.target === nodeId).map((e) => e.targetHandle),
  );
  const needsBackground = def.category === "Camera" || hasWidget(def.params, "boxes");
  return {
    projectDir,
    fps: workflow.settings.fps,
    aspect: workflow.settings.width / workflow.settings.height,
    vars: variableList(workflow),
    secrets,
    layers: Math.max(1, layers),
    actions: workflow.nodes
      .filter((n) => ACTION_NODE_TYPES.has(n.type))
      .map((n) => ({ id: n.id, label: nodeName(n) })),
    spotBackground: needsBackground ? spotBackground(workflow, nodeId, results) : null,
    pickTarget: (key) => void pickTarget(workflow, nodeId, key),
    chooseAsset: (key) => void chooseAsset(projectDir, nodeId, key),
  };
}
