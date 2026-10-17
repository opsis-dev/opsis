import { useRunStore } from "../store/run";
import { useCompiledContext } from "./use-compiled";

export function useNodeProblems(nodeId: string): { errors: string[]; warnings: string[] } {
  const runError = useRunStore((s) => s.errors[nodeId]);
  const { result, issues } = useCompiledContext();
  const own = issues.filter((i) => i.nodeId === nodeId);
  const errors = [
    runError,
    result?.errors[nodeId],
    ...own.filter((i) => i.level === "error").map((i) => i.message),
  ].filter((p): p is string => !!p);
  const warnings = own.filter((i) => i.level === "warning").map((i) => i.message);
  return { errors, warnings };
}
