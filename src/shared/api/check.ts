import { compileWorkflow } from "../compile/compile";
import { validateWorkflow } from "../graph";
import { normalizeWorkflow, workflowSchema, type Workflow } from "../schema/workflow";
import type { ApiProblem } from "./types";

export type CheckResult = { ok: boolean; workflow: Workflow | null; problems: ApiProblem[] };

const pathText = (path: PropertyKey[]) =>
  path
    .map((p) => (typeof p === "number" ? `[${p}]` : `.${String(p)}`))
    .join("")
    .replace(/^\./, "");

export function workflowProblems(wf: Workflow): ApiProblem[] {
  const problems: ApiProblem[] = validateWorkflow(wf).map((i) => ({
    level: i.level,
    message: i.message,
    ...(i.nodeId ? { nodeId: i.nodeId } : {}),
    ...(i.edgeId ? { edgeId: i.edgeId } : {}),
  }));
  const ids = new Set<string>();
  for (const n of wf.nodes) {
    if (ids.has(n.id))
      problems.push({ level: "error", nodeId: n.id, message: "Two nodes share this id" });
    ids.add(n.id);
  }
  const flagged = new Set(problems.filter((p) => p.level === "error").map((p) => p.nodeId));
  for (const [nodeId, message] of Object.entries(compileWorkflow(wf).errors))
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
        path: pathText(issue.path),
      })),
    };
  const workflow = normalizeWorkflow(parsed.data);
  const problems = workflowProblems(workflow);
  return { ok: !problems.some((p) => p.level === "error"), workflow, problems };
}
