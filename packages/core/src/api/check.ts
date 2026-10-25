import { z } from "zod";
import { compileWorkflow } from "../compile/compile";
import { validateWorkflow } from "../graph";
import { normalizeWorkflow, workflowSchema, type Workflow } from "../schema/workflow";
import type { ApiCheck, ApiProblem } from "./types";

type CheckResult = ApiCheck & { workflow: Workflow | null };

export const problemCheck = (problems: ApiProblem[]): ApiCheck => ({
  ok: !problems.some((p) => p.level === "error"),
  problems,
});

export function workflowProblems(wf: Workflow): ApiProblem[] {
  const compiled = compileWorkflow(wf);
  const problems: ApiProblem[] = validateWorkflow(wf, compiled);
  const flagged = new Set(problems.filter((p) => p.level === "error").map((p) => p.nodeId));
  for (const [nodeId, message] of Object.entries(compiled.errors))
    if (!flagged.has(nodeId)) problems.push({ level: "error", nodeId, message });
  return problems;
}

export function checkWorkflow(input: unknown): CheckResult {
  const parsed = workflowSchema.safeParse(input);
  if (!parsed.success)
    return {
      ok: false,
      workflow: null,
      problems: parsed.error.issues.map((issue) => ({
        level: "error",
        message: issue.message,
        path: z.core.toDotPath(issue.path),
      })),
    };
  const workflow = normalizeWorkflow(parsed.data);
  return { ...problemCheck(workflowProblems(workflow)), workflow };
}
