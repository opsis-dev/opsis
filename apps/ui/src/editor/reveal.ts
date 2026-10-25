import { projectRelPath } from "@opsis/core/asset-ref";
import { PROJECT_DIRS } from "@opsis/core/project-dirs";

export const revealOutputs = (projectDir: string): Promise<void> =>
  window.opsis.revealInFolder(projectDir, PROJECT_DIRS.out);

export const revealProjectFile = (projectDir: string, ref: string): Promise<void> =>
  window.opsis.revealInFolder(projectDir, projectRelPath(ref) ?? undefined);
