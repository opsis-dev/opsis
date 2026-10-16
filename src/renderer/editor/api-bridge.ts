import { useEffect } from "react";
import { workflowProblems } from "@/shared/api/check";
import type { ApiEditorReply, ApiEditorRequest } from "@/shared/api/types";
import { defaultParams, getNodeDef } from "@/shared/nodes/registry";
import { canConnect } from "@/shared/nodes/ports";
import { workflowSettingsSchema, type WorkflowNode } from "@/shared/schema/workflow";
import { setEditorApi } from "../app/api-host";
import { useRunStore } from "../store/run";
import { newId, useWorkflowStore } from "../store/workflow";
import { beginRun, cancelRun } from "./run";
import { save } from "./save";

const fail = (status: number, error: string): ApiEditorReply => ({ ok: false, status, error });
const done = (result: unknown): ApiEditorReply => ({ ok: true, result });

function edited(id?: string): ApiEditorReply {
  const { workflow, revision } = useWorkflowStore.getState();
  const problems = workflow ? workflowProblems(workflow) : [];
  return done({ ...(id ? { id } : {}), revision, problems });
}

function read(): ApiEditorReply {
  const { projectDir, workflow, revision, savedRevision } = useWorkflowStore.getState();
  const { running, runId } = useRunStore.getState();
  return done({
    dir: projectDir,
    workflow,
    revision,
    saved: revision === savedRevision,
    running,
    runId,
  });
}

function nextPosition(nodes: WorkflowNode[]): { x: number; y: number } {
  if (!nodes.length) return { x: 0, y: 0 };
  return { x: Math.max(...nodes.map((n) => n.position.x)) + 300, y: 0 };
}

function portList(ports: Record<string, unknown>): string {
  const ids = Object.keys(ports);
  return ids.length ? ids.join(", ") : "none";
}

async function handle(request: ApiEditorRequest): Promise<ApiEditorReply> {
  const store = useWorkflowStore.getState();
  const wf = store.workflow;
  if (!wf) return fail(409, "No project is open.");
  const node = (id: string) => wf.nodes.find((n) => n.id === id);

  switch (request.op) {
    case "open":
    case "read":
      return read();
    case "replace":
      store.update(() => request.workflow);
      return edited();
    case "addNode": {
      const def = getNodeDef(request.type);
      if (!def) return fail(400, `There's no node type “${request.type}”. See GET /nodes.`);
      const id = request.id ?? newId(request.type.split(".")[0]);
      if (node(id)) return fail(400, `A node with the id “${id}” already exists.`);
      store.update((w) => ({
        ...w,
        nodes: [
          ...w.nodes,
          {
            id,
            type: request.type,
            position: request.position ?? nextPosition(w.nodes),
            params: { ...defaultParams(request.type), ...request.params },
            ...(request.label ? { label: request.label } : {}),
          },
        ],
      }));
      return edited(id);
    }
    case "updateNode": {
      if (!node(request.id)) return fail(404, `There's no node “${request.id}”.`);
      store.update((w) => ({
        ...w,
        nodes: w.nodes.map((n) => {
          if (n.id !== request.id) return n;
          const next: WorkflowNode = {
            ...n,
            ...(request.position ? { position: request.position } : {}),
            ...(request.disabled !== undefined ? { disabled: request.disabled || undefined } : {}),
            ...(request.params ? { params: { ...n.params, ...request.params } } : {}),
          };
          if (request.label === null || request.label === "") delete next.label;
          else if (request.label !== undefined) next.label = request.label;
          return next;
        }),
      }));
      return edited();
    }
    case "removeNodes": {
      const missing = request.ids.find((id) => !node(id));
      if (missing) return fail(404, `There's no node “${missing}”.`);
      store.removeNodes(request.ids);
      return edited();
    }
    case "addEdge": {
      const source = node(request.source);
      const target = node(request.target);
      if (!source) return fail(404, `There's no node “${request.source}”.`);
      if (!target) return fail(404, `There's no node “${request.target}”.`);
      const out = getNodeDef(source.type)?.outputs ?? {};
      const inp = getNodeDef(target.type)?.inputs ?? {};
      const from = out[request.sourceHandle];
      const to = inp[request.targetHandle];
      if (!from)
        return fail(
          400,
          `“${request.source}” has no output “${request.sourceHandle}”. Its outputs: ${portList(out)}.`,
        );
      if (!to)
        return fail(
          400,
          `“${request.target}” has no input “${request.targetHandle}”. Its inputs: ${portList(inp)}.`,
        );
      if (!canConnect(from.type, to.type))
        return fail(400, `A ${from.type} output can't connect to a ${to.type} input.`);
      store.addEdge({
        source: request.source,
        sourceHandle: request.sourceHandle,
        target: request.target,
        targetHandle: request.targetHandle,
      });
      return edited(
        `${request.source}.${request.sourceHandle}->${request.target}.${request.targetHandle}`,
      );
    }
    case "removeEdges": {
      const missing = request.ids.find((id) => !wf.edges.some((e) => e.id === id));
      if (missing) return fail(404, `There's no connection “${missing}”.`);
      store.removeEdges(request.ids);
      return edited();
    }
    case "setSettings": {
      const parsed = workflowSettingsSchema.partial().strict().safeParse(request.settings);
      if (!parsed.success) {
        const issue = parsed.error.issues[0];
        const where = issue?.path.length ? `${issue.path.join(".")}: ` : "";
        return fail(400, `${where}${issue?.message ?? "Invalid settings."}`);
      }
      store.setSettings(parsed.data);
      return edited();
    }
    case "setVariables":
      store.setVariables(request.variables);
      return edited();
    case "problems": {
      const problems = workflowProblems(wf);
      return done({ ok: !problems.some((p) => p.level === "error"), problems });
    }
    case "undo":
      useWorkflowStore.temporal.getState().undo();
      return edited();
    case "redo":
      useWorkflowStore.temporal.getState().redo();
      return edited();
    case "save":
      return (await save())
        ? edited()
        : fail(409, "The project file changed on disk. Choose a version in the editor first.");
    case "run": {
      if (useRunStore.getState().running) return fail(409, "A run is already going.");
      try {
        const runId = await beginRun({
          targetNodeIds: request.targetNodeIds,
          fromNodeId: request.fromNodeId,
        });
        return done({ running: true, runId });
      } catch (e) {
        return fail(409, e instanceof Error ? e.message : String(e));
      }
    }
    case "stop":
      await cancelRun();
      return done({ running: useRunStore.getState().running, runId: useRunStore.getState().runId });
  }
}

export function useApiBridge(): void {
  useEffect(() => setEditorApi(handle), []);
}
