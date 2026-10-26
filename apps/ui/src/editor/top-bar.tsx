import {
  ArrowLeft,
  FolderOpen,
  LayoutGrid,
  MonitorPlay,
  Play,
  Plus,
  Redo2,
  Settings,
  Undo2,
} from "lucide-react";
import { useStore } from "zustand";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Kbd } from "@/components/ui/kbd";
import { Separator } from "@/components/ui/separator";
import { Spinner } from "@/components/ui/spinner";
import { Toggle } from "@/components/ui/toggle";
import { AppBar } from "../app/app-bar";
import { dispatchMenuCommand } from "../app/menu-commands";
import { commandLabel, commandShortcut } from "../app/menu-shortcuts";
import { useRunStore } from "../store/run";
import { useWorkflowStore } from "../store/workflow";
import { LiveToggle } from "./live-pip";
import { PANEL_SIDES, PANEL_TOGGLES, usePanels } from "./panels";
import { revealOutputs } from "./reveal";
import { RunningLabel, StopRunButton } from "./run-controls";
import { TooltipButton, WithTooltip } from "./with-tooltip";

const PANEL_BUTTONS = PANEL_SIDES.map((side) => ({
  side,
  icon: PANEL_TOGGLES[side].icon,
  label: commandLabel(PANEL_TOGGLES[side].command).toLowerCase(),
  keys: commandShortcut(PANEL_TOGGLES[side].command),
}));

function PanelToggles() {
  const open = usePanels((s) => s.open);
  return (
    <div className="flex items-center gap-0.5">
      {PANEL_BUTTONS.map(({ side, icon: Icon, label, keys }) => (
        <WithTooltip
          key={side}
          trigger={
            <Toggle
              size="sm"
              aria-label={`${open[side] ? "Hide" : "Show"} ${label}`}
              pressed={open[side]}
              onPressedChange={() => dispatchMenuCommand(PANEL_TOGGLES[side].command)}
            />
          }
          tip={
            <>
              {open[side] ? "Hide" : "Show"} {label} <Kbd>{keys}</Kbd>
            </>
          }
        >
          <Icon />
        </WithTooltip>
      ))}
    </div>
  );
}

function HistoryButtons() {
  const canUndo = useStore(useWorkflowStore.temporal, (s) => s.pastStates.length > 0);
  const canRedo = useStore(useWorkflowStore.temporal, (s) => s.futureStates.length > 0);
  return (
    <>
      <TooltipButton
        label="Undo"
        icon={Undo2}
        shortcut={commandShortcut("edit:undo")}
        disabled={!canUndo}
        onClick={() => dispatchMenuCommand("edit:undo")}
      />
      <TooltipButton
        label="Redo"
        icon={Redo2}
        shortcut={commandShortcut("edit:redo")}
        disabled={!canRedo}
        onClick={() => dispatchMenuCommand("edit:redo")}
      />
    </>
  );
}

function PreviewButton() {
  return (
    <WithTooltip
      trigger={
        <Button variant="ghost" size="sm" onClick={() => dispatchMenuCommand("view:preview")} />
      }
      tip={
        <>
          Open the preview in its own window <Kbd>{commandShortcut("view:preview")}</Kbd>
        </>
      }
    >
      <MonitorPlay data-icon="inline-start" />
      Preview
    </WithTooltip>
  );
}

function RunControls() {
  const running = useRunStore((s) => s.running);
  if (!running)
    return (
      <Button size="sm" onClick={() => dispatchMenuCommand("run:start")}>
        <Play data-icon="inline-start" />
        Run
      </Button>
    );
  return (
    <>
      <span className="flex items-center gap-1.5 text-sm">
        <Spinner className="text-muted-foreground" />
        <RunningLabel />
      </span>
      <StopRunButton size="sm" />
    </>
  );
}

export function TopBar({ projectDir }: { projectDir: string }) {
  const name = useWorkflowStore((s) => s.workflow?.name);
  const dirty = useWorkflowStore((s) => s.revision !== s.savedRevision);

  return (
    <AppBar>
      <Button
        variant="ghost"
        size="icon-sm"
        aria-label="All projects"
        onClick={() => dispatchMenuCommand("view:projects")}
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
        onClick={() => dispatchMenuCommand("edit:layout")}
      />
      <Button variant="ghost" size="sm" onClick={() => dispatchMenuCommand("edit:add-node")}>
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
        onClick={() => dispatchMenuCommand("view:settings")}
      >
        <Settings />
      </Button>
      <PreviewButton />
      <RunControls />
    </AppBar>
  );
}
