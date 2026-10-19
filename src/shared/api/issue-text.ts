import { z } from "zod";

export function firstIssueText(error: z.ZodError): string {
  const issue = error.issues[0];
  return issue.path.length ? `${z.core.toDotPath(issue.path)}: ${issue.message}` : issue.message;
}
