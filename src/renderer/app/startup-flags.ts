import { readSession, readStored, writeSession, writeStored } from "@/renderer/lib/local-storage";

const SETUP_SKIPPED_KEY = "opsis:setup-skipped";
const STARTED_KEY = "opsis:started";
const ON = "1";

export const isSetupSkipped = (): boolean => readStored(SETUP_SKIPPED_KEY) === ON;

export const skipSetup = (): void => writeStored(SETUP_SKIPPED_KEY, ON);

export function markStarted(): boolean {
  const first = readSession(STARTED_KEY) !== ON;
  writeSession(STARTED_KEY, ON);
  return first;
}
