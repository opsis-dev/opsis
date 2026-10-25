import { stat } from "node:fs/promises";
import path from "node:path";
import { artifactName, type Artifact } from "../../shared/schema/values";
import { warnFailed } from "./context";

export async function withSizes(
  projectDir: string,
  artifacts: Artifact[],
  log: (level: "warn", message: string) => void,
): Promise<Artifact[]> {
  return Promise.all(
    artifacts.map((artifact) =>
      stat(path.join(projectDir, artifact.path)).then(
        (s) => ({ ...artifact, bytes: s.size }),
        (error: unknown) => {
          warnFailed(log, `Couldn't read ${artifactName(artifact)}`, error);
          return artifact;
        },
      ),
    ),
  );
}
