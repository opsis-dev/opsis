import { PROJECT_DIRS } from "@/core/project-dirs";
import { commands } from "@/generated/bindings";

export const revealOutputs = (projectDir: string) =>
  commands.revealInFolder(projectDir, PROJECT_DIRS.out);

export const revealProjectFile = (projectDir: string, ref: string) =>
  commands.revealInFolder(projectDir, ref);
