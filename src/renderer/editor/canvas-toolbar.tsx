import { Panel } from "@xyflow/react";
import { Map as MapIcon, Mouse, Touchpad } from "lucide-react";
import { CANVAS_MODES, type CanvasMode } from "@/shared/settings";
import { Toggle } from "@/renderer/components/ui/toggle";
import { ToggleGroup, ToggleGroupItem } from "@/renderer/components/ui/toggle-group";
import { whenFirstOneOf } from "@/renderer/lib/one-of";
import { patchAppSettings } from "../store/app-settings";
import { useViewOptions } from "../store/view-options";
import { WithTooltip } from "./tooltip-button";

export function CanvasToolbar({ mode }: { mode: CanvasMode }) {
  const minimap = useViewOptions((s) => s.minimap);
  return (
    <Panel
      position="top-right"
      className="bg-primary-foreground flex items-center gap-1 rounded-md p-1"
    >
      <ToggleGroup
        size="sm"
        value={[mode]}
        onValueChange={whenFirstOneOf(
          CANVAS_MODES,
          (canvasMode) => void patchAppSettings({ editor: { canvasMode } }),
        )}
      >
        <WithTooltip
          trigger={<ToggleGroupItem value="mouse" aria-label="Mouse" />}
          tip="Mouse: the wheel zooms, drag the background to move around"
        >
          <Mouse />
        </WithTooltip>
        <WithTooltip
          trigger={<ToggleGroupItem value="touchpad" aria-label="Touchpad" />}
          tip="Touchpad: two fingers move around, pinch to zoom"
        >
          <Touchpad />
        </WithTooltip>
      </ToggleGroup>
      <WithTooltip
        trigger={
          <Toggle
            size="sm"
            aria-label="Minimap"
            pressed={minimap}
            onPressedChange={(v) => useViewOptions.getState().setViewOption("minimap", v)}
          />
        }
        tip={minimap ? "Hide the minimap" : "Show the minimap"}
      >
        <MapIcon />
      </WithTooltip>
    </Panel>
  );
}
