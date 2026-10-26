import { ReactFlowProvider } from "@xyflow/react";
import { useEffect } from "react";
import { useDefaultLayout } from "react-resizable-panels";
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from "@/components/ui/resizable";
import { useEditorSync } from "../app/sync";
import { useEnvironment } from "../store/environment";
import { useMedia } from "../store/media";
import { useProjectFiles } from "../store/project-files";
import { useWorkflowStore } from "../store/workflow";
import { useApiBridge } from "./api-bridge";
import { Canvas } from "./canvas";
import { DiskChangeDialog } from "./disk-change";
import { BottomDock } from "./dock";
import { FilmProvider } from "./film";
import { Inspector } from "./inspector";
import { LivePip } from "./live-pip";
import { Palette, QuickAdd } from "./palette";
import { layoutStorage, usePanels } from "./panels";
import { ReplayDialog } from "./replay-dialog";
import { SidePanel } from "./side-panel";
import { TopBar } from "./top-bar";
import { useAutosave } from "./use-autosave";
import { CompiledContext, useCompiled } from "./use-compiled";
import { useFileChanges } from "./use-file-changes";
import { useForgetStaleErrors } from "./use-forget-stale-errors";
import { useMenuCommands } from "./use-menu-commands";
import { useProjectFileWatch } from "./use-project-file-watch";
import { useWatchLoadedFiles } from "./use-watch-loaded-files";

function InspectorPanel() {
  const open = usePanels((s) => s.open.right);
  return open ? <Inspector /> : null;
}

function useEditorSetup(projectDir: string): void {
  const name = useWorkflowStore((s) => s.workflow?.name);
  useAutosave();
  useForgetStaleErrors();
  useMenuCommands();
  useProjectFileWatch(projectDir);
  useFileChanges();
  useApiBridge();
  useEffect(() => {
    if (name) window.opsis.setTitle(`${name} — Opsis`);
  }, [name]);
  useEffect(() => {
    void useEnvironment.getState().refresh();
    useMedia.getState().reset(projectDir);
    useProjectFiles.getState().forget();
  }, [projectDir]);
  useEffect(() => {
    const checkAgain = () => void useProjectFiles.getState().checkAgain();
    window.addEventListener("focus", checkAgain);
    return () => window.removeEventListener("focus", checkAgain);
  }, []);
  useEditorSync();
  useEffect(() => () => void window.opsis.closePanelWindows(), []);
}

export function Editor({ projectDir }: { projectDir: string }) {
  const compiled = useCompiled({ drive: true });
  const horizontal = useDefaultLayout({
    id: "opsis:editor-columns",
    storage: layoutStorage,
    panelIds: ["left", "center", "right"],
  });
  const vertical = useDefaultLayout({
    id: "opsis:editor-rows",
    storage: layoutStorage,
    panelIds: ["canvas", "bottom"],
  });
  useEditorSetup(projectDir);
  useWatchLoadedFiles(projectDir, compiled.result);

  return (
    <ReactFlowProvider>
      <CompiledContext.Provider value={compiled}>
        <FilmProvider>
          <div className="flex h-screen flex-col">
            <TopBar projectDir={projectDir} />
            <ResizablePanelGroup
              orientation="horizontal"
              className="min-h-0 flex-1"
              defaultLayout={horizontal.defaultLayout}
              onLayoutChanged={horizontal.onLayoutChanged}
            >
              <SidePanel side="left" defaultSize="17%" minSize="12%" maxSize="30%">
                <Palette />
              </SidePanel>
              <ResizableHandle />
              <ResizablePanel id="center" defaultSize="59%" minSize="30%">
                <ResizablePanelGroup
                  orientation="vertical"
                  defaultLayout={vertical.defaultLayout}
                  onLayoutChanged={vertical.onLayoutChanged}
                >
                  <ResizablePanel id="canvas" defaultSize="60%" minSize="20%">
                    <div className="relative size-full">
                      <Canvas projectDir={projectDir} />
                      <LivePip />
                    </div>
                  </ResizablePanel>
                  <ResizableHandle />
                  <SidePanel side="bottom" defaultSize="40%" minSize="15%">
                    <BottomDock />
                  </SidePanel>
                </ResizablePanelGroup>
              </ResizablePanel>
              <ResizableHandle />
              <SidePanel side="right" defaultSize="24%" minSize="16%" maxSize="40%">
                <InspectorPanel />
              </SidePanel>
            </ResizablePanelGroup>
          </div>
          <QuickAdd />
          <ReplayDialog />
          <DiskChangeDialog />
        </FilmProvider>
      </CompiledContext.Provider>
    </ReactFlowProvider>
  );
}
