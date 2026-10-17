import { useReactFlow } from "@xyflow/react";
import { useCallback } from "react";
import { toast } from "sonner";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { NODE_DRAG_TYPE, snapToGrid } from "./canvas-grid";

const FILE_STAGGER = 30;

export function useCanvasDrop(projectDir: string): (event: React.DragEvent) => Promise<void> {
  const flow = useReactFlow();
  const addAt = useCallback(
    (type: string, clientX: number, clientY: number, params?: Record<string, unknown>) => {
      const pos = flow.screenToFlowPosition({ x: clientX, y: clientY });
      const id = useWorkflowStore.getState().addNode(type, snapToGrid(pos), params);
      useUiStore.getState().setSelected([id]);
    },
    [flow],
  );

  return useCallback(
    async (event: React.DragEvent) => {
      event.preventDefault();
      const type = event.dataTransfer.getData(NODE_DRAG_TYPE);
      if (type) {
        addAt(type, event.clientX, event.clientY);
        return;
      }
      const files = [...event.dataTransfer.files];
      if (!files.length) return;
      const paths = files.map((f) => window.demoflow.pathForFile(f)).filter(Boolean);
      const assets = await window.demoflow.importAssets(projectDir, paths);
      assets.forEach((asset, i) =>
        addAt("media.import", event.clientX + i * FILE_STAGGER, event.clientY + i * FILE_STAGGER, {
          path: asset,
        }),
      );
      if (assets.length)
        toast.success(
          assets.length === 1
            ? "Added 1 file to the project"
            : `Added ${assets.length} files to the project`,
        );
    },
    [addAt, projectDir],
  );
}
