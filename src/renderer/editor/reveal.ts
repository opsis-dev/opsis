import { projectRelPath } from "@/shared/asset-ref";

const OUTPUT_DIR = ".demoflow/out";

export const revealOutputs = (projectDir: string): Promise<void> =>
  window.demoflow.revealInFolder(projectDir, OUTPUT_DIR);

export const revealProjectFile = (projectDir: string, ref: string): Promise<void> =>
  window.demoflow.revealInFolder(projectDir, projectRelPath(ref));
