import { ReactFlowProvider } from "@xyflow/react";
import { useDefaultLayout } from "react-resizable-panels";
import { useEffect } from "react";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/renderer/components/ui/resizable";
import { useEditorSync } from "../app/sync";
import { useMedia } from "../store/media";
import { useStillsStore } from "../store/stills";
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
import { useMenuCommands } from "./use-menu-commands";
import { useProjectFileWatch } from "./use-project-file-watch";

function InspectorPanel() {
  const open = usePanels((s) => s.open.right);
  return open ? <Inspector /> : null;
}

function useEditorSetup(projectDir: string): void {
  const name = useWorkflowStore((s) => s.workflow?.name);
  useAutosave();
  useMenuCommands();
  useProjectFileWatch(projectDir);
  useApiBridge();
  useEffect(() => {
    if (name) window.demoflow.setTitle(`${name} — Demoflow`);
  }, [name]);
  useEffect(() => {
    void useStillsStore.getState().load(projectDir);
    void useStillsStore.getState().checkEngine();
    useMedia.getState().reset(projectDir);
  }, [projectDir]);
  useEditorSync();
  useEffect(() => () => void window.demoflow.closePanelWindows(), []);
}

export function Editor({ projectDir }: { projectDir: string }) {
  const compiled = useCompiled();
  const horizontal = useDefaultLayout({
    id: `demoflow-editor-columns:${projectDir}`,
    storage: layoutStorage,
    panelIds: ["left", "center", "right"],
  });
  const vertical = useDefaultLayout({
    id: `demoflow-editor-rows:${projectDir}`,
    storage: layoutStorage,
    panelIds: ["canvas", "bottom"],
  });
  useEditorSetup(projectDir);

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
