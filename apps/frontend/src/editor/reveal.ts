import { commands } from "@/generated/bindings";
import { PROJECT_DIRS } from "@/workflow/project-dirs";

export function revealOutputs(projectDir: string) {
  return commands.revealInFolder(projectDir, PROJECT_DIRS.out);
}

export function revealProjectFile(projectDir: string, ref: string) {
  return commands.revealInFolder(projectDir, ref);
}
