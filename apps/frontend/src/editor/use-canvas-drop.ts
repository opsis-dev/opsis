import { useReactFlow } from "@xyflow/react";
import { useCallback } from "react";
import { toast } from "sonner";
import { pathForFile } from "@/app/tauri";
import { commands } from "@/generated/bindings";
import { attempt } from "@/lib/toast";
import { useUiStore } from "../store/ui";
import { workflowEdits } from "../store/workflow";
import { NODE_DRAG_TYPE, snapToGrid } from "./canvas-grid";

const FILE_STAGGER = 30;

export function useCanvasDrop(projectDir: string): (event: React.DragEvent) => Promise<void> {
  const flow = useReactFlow();
  return useCallback(
    async (event: React.DragEvent) => {
      const addAt = (type: string, clientX: number, clientY: number, params?: { path: string }) => {
        const pos = flow.screenToFlowPosition({ x: clientX, y: clientY });
        void workflowEdits
          .addNode(type, snapToGrid(pos), params)
          .then((id) => id && useUiStore.getState().setSelected([id]));
      };
      event.preventDefault();
      const type = event.dataTransfer.getData(NODE_DRAG_TYPE);
      if (type) {
        addAt(type, event.clientX, event.clientY);
        return;
      }
      const files = [...event.dataTransfer.files];
      if (!files.length) return;
      const paths = files.map(pathForFile).filter(Boolean);
      await attempt(async () => {
        const assets = await commands.importAssets(projectDir, paths);
        assets.forEach((asset, i) =>
          addAt(
            "media.import",
            event.clientX + i * FILE_STAGGER,
            event.clientY + i * FILE_STAGGER,
            { path: asset },
          ),
        );
        if (assets.length)
          toast.success(
            assets.length === 1
              ? "Added 1 file to the project"
              : `Added ${assets.length} files to the project`,
          );
      }, "Couldn't add the files");
    },
    [flow, projectDir],
  );
}
