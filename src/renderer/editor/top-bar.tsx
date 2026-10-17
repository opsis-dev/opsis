import { useNavigate } from "@tanstack/react-router";
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
import { useStore } from "zustand";
import { Badge } from "@/renderer/components/ui/badge";
import { Button } from "@/renderer/components/ui/button";
import { Kbd } from "@/renderer/components/ui/kbd";
import { Separator } from "@/renderer/components/ui/separator";
import { Spinner } from "@/renderer/components/ui/spinner";
import { TextShimmer } from "@/renderer/components/ui/text-shimmer";
import { Toggle } from "@/renderer/components/ui/toggle";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";
import { AppBar, commandLabel, commandShortcut } from "../app/app-bar";
import { useRunStore } from "../store/run";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { tidyLayout } from "./layout";
import { LiveToggle } from "./live-pip";
import { PANEL_SIDES, togglePanel, usePanels, type PanelSide } from "./panels";
import { revealOutputs } from "./reveal";
import { cancelRun, startRun } from "./run";
import { TooltipButton } from "./tooltip-button";

const PANEL_ICONS: Record<PanelSide, LucideIcon> = {
  left: PanelLeft,
  bottom: PanelBottom,
  right: PanelRight,
};

const PANEL_TOGGLES = PANEL_SIDES.map((side) => ({
  side,
  icon: PANEL_ICONS[side],
  label: commandLabel(`view:toggle-${side}`).toLowerCase(),
  keys: commandShortcut(`view:toggle-${side}`),
}));

function PanelToggles() {
  const open = usePanels((s) => s.open);
  return (
    <div className="flex items-center gap-0.5">
      {PANEL_TOGGLES.map(({ side, icon: Icon, label, keys }) => (
        <Tooltip key={side}>
          <TooltipTrigger
            render={
              <Toggle
                size="sm"
                aria-label={`${open[side] ? "Hide" : "Show"} ${label}`}
                pressed={open[side]}
                onPressedChange={() => togglePanel(side)}
              />
            }
          >
            <Icon />
          </TooltipTrigger>
          <TooltipContent>
            {open[side] ? "Hide" : "Show"} {label} <Kbd>{keys}</Kbd>
          </TooltipContent>
        </Tooltip>
      ))}
    </div>
  );
}

function HistoryButtons() {
  const canUndo = useStore(useWorkflowStore.temporal, (s) => s.pastStates.length > 0);
  const canRedo = useStore(useWorkflowStore.temporal, (s) => s.futureStates.length > 0);
  const { undo, redo } = useWorkflowStore.temporal.getState();
  return (
    <>
      <TooltipButton
        label="Undo"
        icon={Undo2}
        shortcut={commandShortcut("edit:undo")}
        disabled={!canUndo}
        onClick={() => undo()}
      />
      <TooltipButton
        label="Redo"
        icon={Redo2}
        shortcut={commandShortcut("edit:redo")}
        disabled={!canRedo}
        onClick={() => redo()}
      />
    </>
  );
}

function RunControls() {
  const running = useRunStore((s) => s.running);
  if (!running)
    return (
      <Button size="sm" onClick={() => void startRun()}>
        <Play data-icon="inline-start" />
        Run
      </Button>
    );
  return (
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
  );
}

export function TopBar({ projectDir }: { projectDir: string }) {
  const navigate = useNavigate();
  const name = useWorkflowStore((s) => s.workflow?.name);
  const dirty = useWorkflowStore((s) => s.revision !== s.savedRevision);

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
      <span className="truncate font-semibold">{name}</span>
      <Badge variant="outline" className="font-normal">
        {dirty ? "Unsaved" : "Saved"}
      </Badge>
      <Separator orientation="vertical" className="mx-1 h-5 data-vertical:self-center" />
      <HistoryButtons />
      <TooltipButton
        label="Tidy up layout"
        icon={LayoutGrid}
        shortcut={commandShortcut("edit:layout")}
        onClick={() => void tidyLayout()}
      />
      <Button variant="ghost" size="sm" onClick={() => useUiStore.getState().setPaletteOpen(true)}>
        <Plus data-icon="inline-start" />
        Add node <Kbd>{commandShortcut("edit:add-node")}</Kbd>
      </Button>
      <div className="flex-1" />
      <PanelToggles />
      <Separator orientation="vertical" className="mx-1 h-5 data-vertical:self-center" />
      <LiveToggle />
      <Button variant="ghost" size="sm" onClick={() => void revealOutputs(projectDir)}>
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
      <RunControls />
    </AppBar>
  );
}
