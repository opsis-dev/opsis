import { useEffect } from "react";
import { toast } from "sonner";
import { z } from "zod";
import {
  workflowEdgeSchema,
  workflowNodeSchema,
  type WorkflowEdge,
  type WorkflowNode,
} from "@/shared/schema/workflow";
import { isEditingElement } from "@/renderer/lib/dom";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { tryParseJson } from "@/shared/guards";
import { reportError } from "@/renderer/lib/toast";

const CLIPBOARD_TAG = "demoflow/v1";
const PASTE_OFFSET = { x: 60, y: 60 };

const clipboardTagSchema = z.object({ tag: z.literal(CLIPBOARD_TAG) });

const clipboardSchema = clipboardTagSchema.extend({
  nodes: z.array(workflowNodeSchema),
  edges: z.array(workflowEdgeSchema),
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
  const ids = useWorkflowStore.getState().paste(nodes, edges, PASTE_OFFSET);
  useUiStore.getState().setSelected(ids);
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
  try {
    await navigator.clipboard.writeText(JSON.stringify({ tag: CLIPBOARD_TAG, ...subgraph }));
  } catch (e) {
    reportError("Couldn't copy the nodes", e);
  }
}

async function pasteClipboard(): Promise<void> {
  let text: string;
  try {
    text = await navigator.clipboard.readText();
  } catch (e) {
    reportError("Couldn't read the clipboard", e);
    return;
  }
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
    const onKey = (e: KeyboardEvent) => {
      if (isEditingElement(e.target) || !(e.metaKey || e.ctrlKey)) return;
      if (e.key === "c") void copySelection();
      else if (e.key === "v") void pasteClipboard();
      else if (e.key === "d") {
        e.preventDefault();
        duplicateSelection();
      }
    };
    el.addEventListener("keydown", onKey);
    return () => el.removeEventListener("keydown", onKey);
  }, [wrapper]);
}
