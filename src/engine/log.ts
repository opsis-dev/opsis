import { errorMessage } from "../shared/errors";

export function logWarning(scope: string, message: string): void {
  process.stderr.write(`[${scope}] ${message}\n`);
}

export function logError(scope: string, error: unknown, subject?: string): void {
  const about = subject ? `${subject}: ` : "";
  logWarning(scope, `${about}${errorMessage(error)}`);
}
