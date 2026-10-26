import { PROJECT_DIRS } from "@/core/project-dirs";

export const revealOutputs = (projectDir: string) =>
  window.opsis.revealInFolder(projectDir, PROJECT_DIRS.out);

export const revealProjectFile = (projectDir: string, ref: string) =>
  window.opsis.revealInFolder(projectDir, ref);
