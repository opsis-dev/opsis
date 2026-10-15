import { createContext, useContext, useMemo } from "react";
import {
  compileWorkflow,
  previewSpec,
  type CompileResult,
  type PreviewTarget,
} from "@/shared/compile/compile";
import { validateWorkflow, type GraphIssue } from "@/shared/graph";
import { useRunStore } from "../store/run";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";

export function useCompiled(): {
  result: CompileResult | null;
  issues: GraphIssue[];
  preview: PreviewTarget | null;
} {
  const workflow = useWorkflowStore((s) => s.workflow);
  const results = useRunStore((s) => s.results);
  const selected = useUiStore((s) => s.selectedIds);
  const selectedId = selected.length === 1 ? selected[0] : null;
  const result = useMemo(
    () => (workflow ? compileWorkflow(workflow, { results }) : null),
    [workflow, results],
  );
  const issues = useMemo(() => (workflow ? validateWorkflow(workflow) : []), [workflow]);
  const preview = useMemo(
    () => (workflow && result ? previewSpec(workflow, result, selectedId) : null),
    [workflow, result, selectedId],
  );
  return { result, issues, preview };
}

type Compiled = ReturnType<typeof useCompiled>;
export const CompiledContext = createContext<Compiled>({ result: null, issues: [], preview: null });
export const useCompiledContext = () => useContext(CompiledContext);
