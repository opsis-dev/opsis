import { projectRelPath } from "@/shared/asset-ref";
import { PROJECT_DIRS } from "@/shared/project-dirs";

export const revealOutputs = (projectDir: string): Promise<void> =>
  window.demoflow.revealInFolder(projectDir, PROJECT_DIRS.out);

export const revealProjectFile = (projectDir: string, ref: string): Promise<void> =>
  window.demoflow.revealInFolder(projectDir, projectRelPath(ref));
