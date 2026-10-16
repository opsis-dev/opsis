import { useNavigate } from "@tanstack/react-router";
import { ReactFlowProvider } from "@xyflow/react";
import {
  ArrowLeft,
  FolderOpen,
  LayoutGrid,
  PanelBottom,
  PanelLeft,
  PanelRight,
  Play,
  Plus,
  Redo2,
  Settings,
  Square,
  Undo2,
  type LucideIcon,
} from "lucide-react";
import { useDefaultLayout } from "react-resizable-panels";
import { useEffect } from "react";
import { useStore } from "zustand";
import { toast } from "sonner";
import { Badge } from "@/renderer/components/ui/badge";
import { Button } from "@/renderer/components/ui/button";
import { Kbd } from "@/renderer/components/ui/kbd";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/renderer/components/ui/resizable";
import { Separator } from "@/renderer/components/ui/separator";
import { Spinner } from "@/renderer/components/ui/spinner";
import { TextShimmer } from "@/renderer/components/ui/text-shimmer";
import { Toggle } from "@/renderer/components/ui/toggle";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";
import { useRunStore } from "../store/run";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { Canvas } from "./canvas";
import { Inspector } from "./inspector";
import { autoLayout } from "./layout";
import { LivePip, LiveToggle } from "./live-pip";
import { AppBar, shortcutLabel } from "../app/app-bar";
import { Palette, QuickAdd } from "./palette";
import {
  focusPanel,
  layoutStorage,
  panelRef,
  togglePanel,
  usePanels,
  type PanelSide,
} from "./panels";
import { ReplayDialog } from "./replay-dialog";
import { cancelRun, startRun } from "./run";
import { CompiledContext, useCompiled } from "./use-compiled";
import { BottomDock } from "./dock";
import { FilmContext, useFilmState } from "./film";
import { useEditorSync } from "../app/sync";
import { useMedia } from "../store/media";
import { save } from "./save";
import { DiskChangeDialog, useProjectFileWatch } from "./disk-change";
import { useApiBridge } from "./api-bridge";
import { useStillsStore } from "../store/stills";
import { errorMessage } from "@/shared/errors";

async function layout(): Promise<void> {
  const wf = useWorkflowStore.getState().workflow;
  if (!wf) return;
  useWorkflowStore.getState().moveNodes(await autoLayout(wf));
}

function useAutosave() {
  const revision = useWorkflowStore((s) => s.revision);
  const saved = useWorkflowStore((s) => s.savedRevision);
  useEffect(() => {
    window.demoflow.reportUnsaved(revision !== saved);
  }, [revision, saved]);
  useEffect(() => {
    const off = window.demoflow.onFlushSave(() => save().then(() => undefined));
    return () => {
      off();
      const state = useWorkflowStore.getState();
      if (state.revision !== state.savedRevision)
        save().catch((e: unknown) => toast.error("Save failed", { description: errorMessage(e) }));
      window.demoflow.reportUnsaved(false);
    };
  }, []);
  useEffect(() => {
    if (revision === saved) return;
    const t = setTimeout(() => {
      save().catch((e: unknown) => toast.error("Save failed", { description: errorMessage(e) }));
    }, 800);
    return () => clearTimeout(t);
  }, [revision, saved]);
}

function useMenuCommands() {
  useEffect(
    () =>
      window.demoflow.onMenuCommand((command) => {
        const { undo, redo } = useWorkflowStore.temporal.getState();
        switch (command) {
          case "edit:undo":
            undo();
            break;
          case "edit:redo":
            redo();
            break;
          case "project:save":
            void save().then((ok) => ok && toast.success("Saved"));
            break;
          case "run:start":
            void startRun();
            break;
          case "run:cancel":
            void cancelRun();
            break;
          case "edit:group":
            useWorkflowStore.getState().groupNodes(useUiStore.getState().selectedIds);
            break;
          case "edit:layout":
            void layout();
            break;
          case "edit:add-node":
            useUiStore.getState().setPaletteOpen(true);
            break;
          case "view:toggle-left":
            togglePanel("left");
            break;
          case "view:toggle-bottom":
            togglePanel("bottom");
            break;
          case "view:toggle-right":
            togglePanel("right");
            break;
          case "view:toggle-minimap":
            useUiStore.getState().setMinimap(!useUiStore.getState().minimap);
            break;
          case "view:focus-preview":
            focusPanel("preview");
            break;
          case "view:focus-timeline":
            focusPanel("timeline");
            break;
          case "view:focus-mixer":
            focusPanel("mixer");
            break;
          case "view:focus-run":
            focusPanel("run");
            break;
        }
      }),
    [],
  );
}

function TopBar() {
  const navigate = useNavigate();
  const workflow = useWorkflowStore((s) => s.workflow)!;
  const projectDir = useWorkflowStore((s) => s.projectDir)!;
  const dirty = useWorkflowStore((s) => s.revision !== s.savedRevision);
  const running = useRunStore((s) => s.running);
  const canUndo = useStore(useWorkflowStore.temporal, (s) => s.pastStates.length > 0);
  const canRedo = useStore(useWorkflowStore.temporal, (s) => s.futureStates.length > 0);
  const { undo, redo } = useWorkflowStore.temporal.getState();

  return (
    <AppBar>
      <Button
        variant="ghost"
        size="icon-sm"
        aria-label="All projects"
        onClick={() => void navigate({ to: "/" })}
      >
        <ArrowLeft />
      </Button>
      <span className="truncate font-semibold">{workflow.name}</span>
      <Badge variant="outline" className="font-normal">
        {dirty ? "Unsaved" : "Saved"}
      </Badge>
      <Separator orientation="vertical" className="mx-1 h-5 data-vertical:self-center" />
      <Tooltip>
        <TooltipTrigger
          render={
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label="Undo"
              disabled={!canUndo}
              onClick={() => undo()}
            />
          }
        >
          <Undo2 />
        </TooltipTrigger>
        <TooltipContent>
          Undo <Kbd>{shortcutLabel("CmdOrCtrl+Z")}</Kbd>
        </TooltipContent>
      </Tooltip>
      <Tooltip>
        <TooltipTrigger
          render={
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label="Redo"
              disabled={!canRedo}
              onClick={() => redo()}
            />
          }
        >
          <Redo2 />
        </TooltipTrigger>
        <TooltipContent>Redo</TooltipContent>
      </Tooltip>
      <Tooltip>
        <TooltipTrigger
          render={
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label="Tidy up layout"
              onClick={() => void layout()}
            />
          }
        >
          <LayoutGrid />
        </TooltipTrigger>
        <TooltipContent>Tidy up layout</TooltipContent>
      </Tooltip>
      <Button variant="ghost" size="sm" onClick={() => useUiStore.getState().setPaletteOpen(true)}>
        <Plus data-icon="inline-start" />
        Add node <Kbd>{shortcutLabel("CmdOrCtrl+K")}</Kbd>
      </Button>
      <div className="flex-1" />
      <PanelToggles />
      <Separator orientation="vertical" className="mx-1 h-5 data-vertical:self-center" />
      <LiveToggle />
      <Button
        variant="ghost"
        size="sm"
        onClick={() => void window.demoflow.revealInFolder(projectDir, ".demoflow/out")}
      >
        <FolderOpen data-icon="inline-start" />
        Outputs
      </Button>
      <Button
        variant="ghost"
        size="icon-sm"
        aria-label="Settings"
        onClick={() => void window.demoflow.openSettings()}
      >
        <Settings />
      </Button>
      {running ? (
        <>
          <span className="flex items-center gap-1.5 text-sm">
            <Spinner className="text-muted-foreground" />
            <TextShimmer as="span" duration={1.6}>
              Running
            </TextShimmer>
          </span>
          <Button size="sm" variant="outline" onClick={() => void cancelRun()}>
            <Square data-icon="inline-start" />
            Stop
          </Button>
        </>
      ) : (
        <Button size="sm" onClick={() => void startRun()}>
          <Play data-icon="inline-start" />
          Run
        </Button>
      )}
    </AppBar>
  );
}

const TOGGLES: Array<{ side: PanelSide; icon: LucideIcon; label: string; keys: string }> = [
  { side: "left", icon: PanelLeft, label: "Node list", keys: shortcutLabel("CmdOrCtrl+B") },
  {
    side: "bottom",
    icon: PanelBottom,
    label: "Preview, timeline, mixer and run",
    keys: shortcutLabel("CmdOrCtrl+J"),
  },
  {
    side: "right",
    icon: PanelRight,
    label: "Inspector",
    keys: shortcutLabel("CmdOrCtrl+Alt+B"),
  },
];

function PanelToggles() {
  const open = usePanels((s) => s.open);
  return (
    <div className="flex items-center gap-0.5">
      {TOGGLES.map(({ side, icon: Icon, label, keys }) => (
        <Tooltip key={side}>
          <TooltipTrigger
            render={
              <Toggle
                size="sm"
                aria-label={`${open[side] ? "Hide" : "Show"} ${label.toLowerCase()}`}
                pressed={open[side]}
                onPressedChange={() => togglePanel(side)}
              />
            }
          >
            <Icon />
          </TooltipTrigger>
          <TooltipContent>
            {open[side] ? "Hide" : "Show"} {label.toLowerCase()} <Kbd>{keys}</Kbd>
          </TooltipContent>
        </Tooltip>
      ))}
    </div>
  );
}

function InspectorPanel() {
  const open = usePanels((s) => s.open.right);
  return open ? <Inspector /> : null;
}

const sizeWatcher = (side: PanelSide, minOpenPx: number) => (size: { inPixels: number }) =>
  usePanels.getState().setOpen(side, size.inPixels > minOpenPx);

function FilmProvider({ children }: { children: React.ReactNode }) {
  const film = useFilmState();
  return <FilmContext.Provider value={film}>{children}</FilmContext.Provider>;
}

export function Editor() {
  const compiled = useCompiled();
  const projectDir = useWorkflowStore((s) => s.projectDir);
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
  const name = useWorkflowStore((s) => s.workflow?.name);
  useAutosave();
  useMenuCommands();
  useProjectFileWatch(projectDir!);
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

  return (
    <ReactFlowProvider>
      <CompiledContext.Provider value={compiled}>
        <FilmProvider>
          <div className="flex h-screen flex-col">
            <TopBar />
            <ResizablePanelGroup
              orientation="horizontal"
              className="min-h-0 flex-1"
              defaultLayout={horizontal.defaultLayout}
              onLayoutChanged={horizontal.onLayoutChanged}
            >
              <ResizablePanel
                id="left"
                className="bg-sidebar"
                style={{ overflow: "hidden" }}
                panelRef={panelRef("left")}
                defaultSize="17%"
                minSize="12%"
                maxSize="30%"
                collapsible
                collapsedSize="44px"
                onResize={sizeWatcher("left", 60)}
              >
                <Palette />
              </ResizablePanel>
              <ResizableHandle />
              <ResizablePanel id="center" defaultSize="59%" minSize="30%">
                <ResizablePanelGroup
                  orientation="vertical"
                  defaultLayout={vertical.defaultLayout}
                  onLayoutChanged={vertical.onLayoutChanged}
                >
                  <ResizablePanel id="canvas" defaultSize="60%" minSize="20%">
                    <div className="relative size-full">
                      <Canvas />
                      <LivePip />
                    </div>
                  </ResizablePanel>
                  <ResizableHandle />
                  <ResizablePanel
                    id="bottom"
                    className="bg-sidebar"
                    style={{ overflow: "hidden" }}
                    panelRef={panelRef("bottom")}
                    defaultSize="40%"
                    minSize="15%"
                    collapsible
                    collapsedSize="37px"
                    onResize={sizeWatcher("bottom", 48)}
                  >
                    <BottomDock />
                  </ResizablePanel>
                </ResizablePanelGroup>
              </ResizablePanel>
              <ResizableHandle />
              <ResizablePanel
                id="right"
                className="bg-sidebar"
                panelRef={panelRef("right")}
                defaultSize="24%"
                minSize="16%"
                maxSize="40%"
                collapsible
                collapsedSize="0px"
                onResize={sizeWatcher("right", 8)}
                style={{ overflow: "hidden" }}
              >
                <InspectorPanel />
              </ResizablePanel>
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
