import { useMemo } from "react";
import { toast } from "sonner";
import { targetRoleKey } from "@/core/schema";
import type { InspectorContext, PickerTarget, Workflow } from "@/generated/bindings";
import { attempt } from "@/lib/toast";
import { useWorkflowStore, workflowEdits } from "../store/workflow";
import { fileUrl } from "./file-url";
import type { FormContext } from "./form/types";
import { useCompiledContext } from "./use-compiled";

async function pickTarget(nodeId: string, picker: PickerTarget | undefined, key: string) {
  if (!picker) return;
  if (picker.kind === "problem") {
    toast.error(picker.message);
    return;
  }
  toast.info("A browser window opened. Click the element, or Shift-click a spot.");
  await attempt(async () => {
    const picked = await window.opsis.pickElement({ url: picker.url, viewport: picker.viewport });
    if (!picked) return;
    workflowEdits.updateParams(nodeId, {
      [key]: picked.target,
      [targetRoleKey(key)]: picked.role || undefined,
    });
    toast.success(`Picked “${picked.target}”`);
  }, "Couldn't pick the element");
}

async function chooseAsset(projectDir: string | null, apply: (path: string) => void) {
  if (!projectDir) return;
  await attempt(async () => {
    const [path] = await window.opsis.importAssets(projectDir);
    if (path) apply(path);
  }, "Couldn't add the file");
}

const ofNode = (inspector: InspectorContext | null, nodeId: string) =>
  inspector?.nodeId === nodeId ? inspector : null;

export function useFormContext(workflow: Workflow, nodeId: string): FormContext {
  const projectDir = useWorkflowStore((s) => s.projectDir);
  const inspector = ofNode(useCompiledContext().inspector, nodeId);
  const { fps, width, height } = workflow.settings;
  return useMemo(() => {
    const spot = inspector?.spotBackground;
    return {
      nodeId,
      projectDir,
      fps,
      aspect: width / height,
      vars: inspector?.vars ?? [],
      layers: inspector?.layers ?? 1,
      actions: inspector?.actions ?? [],
      spotBackground: spot ? { ...spot, src: fileUrl(spot.src) } : null,
      pickTarget: (key) => void pickTarget(nodeId, inspector?.picker, key),
      chooseAsset: (apply) => void chooseAsset(projectDir, apply),
      swatches: inspector?.swatches ?? [],
      connectedInputs: inspector?.connectedInputs ?? [],
    };
  }, [inspector, nodeId, projectDir, fps, width, height]);
}
