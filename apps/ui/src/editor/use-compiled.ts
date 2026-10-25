import {
  compileWorkflow,
  previewSpec,
  type CompileResult,
  type PreviewTarget,
} from "@opsis/core/compile/compile";
import { validateWorkflow, type GraphIssue } from "@opsis/core/graph";
import { createContext, useContext, useMemo } from "react";
import { useRunStore } from "../store/run";
import { singleSelected, useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";

type Compiled = {
  result: CompileResult | null;
  issues: GraphIssue[];
  preview: PreviewTarget | null;
};

export function useCompiled(): Compiled {
  const workflow = useWorkflowStore((s) => s.workflow);
  const results = useRunStore((s) => s.results);
  const selectedId = useUiStore((s) => s.pinnedSource ?? singleSelected(s.selectedIds));
  const result = useMemo(
    () => (workflow ? compileWorkflow(workflow, { results }) : null),
    [workflow, results],
  );
  const issues = useMemo(
    () => (workflow && result ? validateWorkflow(workflow, result) : []),
    [workflow, result],
  );
  const preview = useMemo(
    () => (workflow && result ? previewSpec(workflow.settings, result, selectedId) : null),
    [workflow, result, selectedId],
  );
  return { result, issues, preview };
}

export const CompiledContext = createContext<Compiled>({ result: null, issues: [], preview: null });
export const useCompiledContext = () => useContext(CompiledContext);
