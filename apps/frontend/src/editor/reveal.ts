import { commands } from "@/generated/bindings";
import { PROJECT_DIRS } from "@/workflow/project-dirs";

export const revealOutputs = (projectDir: string) =>
  commands.revealInFolder(projectDir, PROJECT_DIRS.out);

export const revealProjectFile = (projectDir: string, ref: string) =>
  commands.revealInFolder(projectDir, ref);
