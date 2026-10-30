import { useRunStore } from "../store/run";
import { useCompiledContext } from "./use-compiled";

function unique(messages: Array<string | undefined>): string[] {
  return [...new Set(messages.filter((m): m is string => !!m))];
}

export function useNodeProblems(nodeId: string): { errors: string[]; warnings: string[] } {
  const runError = useRunStore((s) => s.errors[nodeId]);
  const { result, issues } = useCompiledContext();
  const own = issues.filter((i) => i.nodeId === nodeId);
  const errors = unique([
    runError,
    result?.errors[nodeId],
    ...own.filter((i) => i.level === "error").map((i) => i.message),
  ]);
  const warnings = unique(own.filter((i) => i.level === "warning").map((i) => i.message));
  return { errors, warnings };
}
