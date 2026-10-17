import { errorMessage } from "../shared/errors";

export function logError(scope: string, error: unknown, subject?: string): void {
  const about = subject ? `${subject}: ` : "";
  process.stderr.write(`[${scope}] ${about}${errorMessage(error)}\n`);
}
