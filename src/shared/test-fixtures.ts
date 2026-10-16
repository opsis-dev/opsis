import type { ActionLog } from "./schema/actions";
import type { NodeResult } from "./schema/values";

export const recordResult = (log: ActionLog, hash = "h"): NodeResult => ({
  hash,
  outputs: {
    recording: {
      type: "recording",
      src: `project:.demoflow/cache/${hash}/recording.mp4`,
      log,
      durationMs: log.durationMs,
      viewport: log.viewport,
      nodeId: "record",
    },
  },
  artifacts: [],
  finishedAt: 0,
  durationMs: 0,
});
