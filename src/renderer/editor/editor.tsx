import { useNavigate } from "@tanstack/react-router";
import { ReactFlowProvider } from "@xyflow/react";
import {
  ArrowLeft,
  ChevronDown,
  ChevronUp,
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
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/renderer/components/ui/tabs";
import { Toggle } from "@/renderer/components/ui/toggle";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";
import { useRunStore } from "../store/run";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { Canvas } from "./canvas";
import { Inspector } from "./inspector";
import { autoLayout } from "./layout";
import { LivePip, LiveToggle } from "./live-pip";
import { AppBar } from "../app/app-bar";
import { Palette, QuickAdd } from "./palette";
import {
  layoutStorage,
  openPanel,
  panelRef,
  togglePanel,
  usePanels,
  type PanelSide,
} from "./panels";
import { ReplayDialog } from "./replay-dialog";
import { PreviewDock } from "./preview-dock";
import { cancelRun, startRun } from "./run";
import { RunPanel } from "./run-panel";
import { CompiledContext, useCompiled } from "./use-compiled";

const mod = window.demoflow?.platform === "darwin" ? "⌘" : "Ctrl";

async function save(): Promise<void> {
  const { projectDir, workflow, revision, markSaved } = useWorkflowStore.getState();
  if (!projectDir || !workflow) return;
  await window.demoflow.saveWorkflow(projectDir, workflow);
  markSaved(revision);
}

async function layout(): Promise<void> {
  const wf = useWorkflowStore.getState().workflow;
  if (!wf) return;
  useWorkflowStore.getState().moveNodes(await autoLayout(wf));
}

function useAutosave() {
  const revision = useWorkflowStore((s) => s.revision);
  const saved = useWorkflowStore((s) => s.savedRevision);
  useEffect(() => {
    if (revision === saved) return;
    const t = setTimeout(() => {
      save().catch((e: unknown) => toast.error("Save failed", { description: String(e) }));
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
            void save().then(() => toast.success("Saved"));
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
      <Separator orientation="vertical" className="mx-1 h-5" />
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
          Undo <Kbd>{mod}Z</Kbd>
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
        Add node <Kbd>{mod}K</Kbd>
      </Button>
      <div className="flex-1" />
      <PanelToggles />
      <Separator orientation="vertical" className="mx-1 h-5" />
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
        onClick={() => void navigate({ to: "/settings" })}
      >
        <Settings />
      </Button>
      {running ? (
        <>
          <span className="text-muted-foreground flex items-center gap-1.5 text-sm">
            <Spinner />
            Running
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
  { side: "left", icon: PanelLeft, label: "Node list", keys: `${mod}B` },
  { side: "bottom", icon: PanelBottom, label: "Preview and run", keys: `${mod}J` },
  {
    side: "right",
    icon: PanelRight,
    label: "Inspector",
    keys: `${mod}${mod === "⌘" ? "⌥" : "Alt+"}B`,
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

function BottomDock() {
  const tab = useUiStore((s) => s.bottomTab);
  const setTab = useUiStore((s) => s.setBottomTab);
  const open = usePanels((s) => s.open.bottom);
  return (
    <Tabs
      value={tab}
      onValueChange={(v) => {
        setTab(v as "preview" | "run");
        openPanel("bottom");
      }}
      className="flex h-full flex-col gap-0"
    >
      <div className="flex h-9 shrink-0 items-center border-b pr-1">
        <TabsList variant="line" className="h-9 flex-1 justify-start px-2">
          <TabsTrigger value="preview" className="flex-none" onClick={() => openPanel("bottom")}>
            Preview
          </TabsTrigger>
          <TabsTrigger value="run" className="flex-none" onClick={() => openPanel("bottom")}>
            Run
          </TabsTrigger>
        </TabsList>
        <Button
          variant="ghost"
          size="icon-xs"
          aria-label={open ? "Hide preview and run" : "Show preview and run"}
          onClick={() => togglePanel("bottom")}
        >
          {open ? <ChevronDown /> : <ChevronUp />}
        </Button>
      </div>
      <TabsContent value="preview" className="min-h-0 flex-1">
        <PreviewDock />
      </TabsContent>
      <TabsContent value="run" className="min-h-0 flex-1">
        <RunPanel />
      </TabsContent>
    </Tabs>
  );
}

function InspectorPanel() {
  const open = usePanels((s) => s.open.right);
  return open ? <Inspector /> : null;
}

const sizeWatcher = (side: PanelSide, minOpenPx: number) => (size: { inPixels: number }) =>
  usePanels.getState().setOpen(side, size.inPixels > minOpenPx);

export function Editor() {
  const compiled = useCompiled();
  const horizontal = useDefaultLayout({
    id: "demoflow-editor-columns",
    storage: layoutStorage,
    panelIds: ["left", "center", "right"],
  });
  const vertical = useDefaultLayout({
    id: "demoflow-editor-rows",
    storage: layoutStorage,
    panelIds: ["canvas", "bottom"],
  });
  const name = useWorkflowStore((s) => s.workflow?.name);
  useAutosave();
  useMenuCommands();
  useEffect(() => {
    if (name) window.demoflow.setTitle(`${name} — Demoflow`);
  }, [name]);

  return (
    <ReactFlowProvider>
      <CompiledContext.Provider value={compiled}>
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
      </CompiledContext.Provider>
    </ReactFlowProvider>
  );
}
