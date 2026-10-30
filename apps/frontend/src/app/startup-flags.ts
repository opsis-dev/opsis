import { readSession, writeSession } from "@/lib/session-storage";
import { reportError } from "@/lib/toast";
import { patchAppSettings } from "../store/app-settings";

const STARTED_KEY = "opsis:started";
const ON = "1";

export function skipSetup(): void {
  return void patchAppSettings({ app: { setupSkipped: true } }).catch((e: unknown) =>
    reportError("Couldn't save that choice", e),
  );
}

export function markStarted(): boolean {
  const first = readSession(STARTED_KEY) !== ON;
  writeSession(STARTED_KEY, ON);
  return first;
}
