import { tryParseJson } from "@opsis/video/common/guards";
import { useEffect } from "react";
import { toast } from "sonner";
import { z } from "zod";
import { matchesAccelerator } from "@/app/accelerators";
import type { WorkflowEdge, WorkflowNode } from "@/generated/bindings";
import { isEditingElement } from "@/lib/dom";
import { platform } from "@/lib/platform";
import { attempt, loadOrReport } from "@/lib/toast";
import { useUiStore } from "../store/ui";
import { useWorkflowStore, workflowEdits } from "../store/workflow";

const CLIPBOARD_TAG = "opsis/v1";
const PASTE_OFFSET = { x: 60, y: 60 };

const clipboardTagSchema = z.object({ tag: z.literal(CLIPBOARD_TAG) });

const nodeSchema = z.object({
  id: z.string().min(1),
  type: z.string().min(1),
  position: z.object({ x: z.number(), y: z.number() }),
  params: z.record(z.string(), z.unknown()).default({}),
  label: z.string().optional(),
  disabled: z.boolean().optional(),
  parentId: z.string().optional(),
  size: z.object({ width: z.number(), height: z.number() }).optional(),
});

const edgeSchema = z.object({
  id: z.string().min(1),
  source: z.string(),
  sourceHandle: z.string(),
  target: z.string(),
  targetHandle: z.string(),
});

const clipboardSchema = clipboardTagSchema.extend({
  nodes: z.array(nodeSchema),
  edges: z.array(edgeSchema),
});

type Subgraph = { nodes: WorkflowNode[]; edges: WorkflowEdge[] };

function selectedSubgraph(): Subgraph | null {
  const ids = new Set(useUiStore.getState().selectedIds);
  const wf = useWorkflowStore.getState().workflow;
  if (!wf || !ids.size) return null;
  return {
    nodes: wf.nodes.filter((n) => ids.has(n.id)),
    edges: wf.edges.filter((e) => ids.has(e.source) && ids.has(e.target)),
  };
}

function pasteSubgraph({ nodes, edges }: Subgraph): void {
  void workflowEdits
    .paste(nodes, edges, PASTE_OFFSET)
    .then((ids) => useUiStore.getState().setSelected(ids));
}

function readClipboard(text: string): Subgraph | "foreign" | "unreadable" {
  const json = tryParseJson(text);
  if (!json.ok || !clipboardTagSchema.safeParse(json.value).success) return "foreign";
  const parsed = clipboardSchema.safeParse(json.value);
  return parsed.success ? parsed.data : "unreadable";
}

async function copySelection(): Promise<void> {
  const subgraph = selectedSubgraph();
  if (!subgraph) return;
  await attempt(
    () => navigator.clipboard.writeText(JSON.stringify({ tag: CLIPBOARD_TAG, ...subgraph })),
    "Couldn't copy the nodes",
  );
}

async function pasteClipboard(): Promise<void> {
  const text = await loadOrReport(
    () => navigator.clipboard.readText(),
    "Couldn't read the clipboard",
  );
  if (text === null) return;
  const pasted = readClipboard(text);
  if (pasted === "foreign") return;
  if (pasted === "unreadable") {
    toast.error("Couldn't paste the copied nodes", {
      description: "The clipboard holds nodes this project can't read.",
    });
    return;
  }
  pasteSubgraph(pasted);
}

function duplicateSelection(): void {
  const subgraph = selectedSubgraph();
  if (subgraph) pasteSubgraph(subgraph);
}

export function useClipboardShortcuts(wrapper: React.RefObject<HTMLDivElement | null>): void {
  useEffect(() => {
    const el = wrapper.current;
    if (!el) return;
    function onKey(e: KeyboardEvent) {
      if (isEditingElement(e.target)) return;
      if (matchesAccelerator("CmdOrCtrl+C", e, platform)) void copySelection();
      else if (matchesAccelerator("CmdOrCtrl+V", e, platform)) void pasteClipboard();
      else if (matchesAccelerator("CmdOrCtrl+D", e, platform)) {
        e.preventDefault();
        duplicateSelection();
      }
    }
    el.addEventListener("keydown", onKey);
    return () => el.removeEventListener("keydown", onKey);
  }, [wrapper]);
}
