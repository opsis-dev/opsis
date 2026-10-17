import { Panel } from "@xyflow/react";
import { Map as MapIcon, Mouse, Touchpad } from "lucide-react";
import { CANVAS_MODES, type CanvasMode } from "@/shared/settings";
import { Toggle } from "@/renderer/components/ui/toggle";
import { ToggleGroup, ToggleGroupItem } from "@/renderer/components/ui/toggle-group";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";
import { isOneOf } from "@/renderer/lib/one-of";
import { patchAppSettings } from "../store/app-settings";
import { useViewOptions } from "../store/view-options";

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
        onValueChange={(v) =>
          isOneOf(CANVAS_MODES, v[0]) && void patchAppSettings({ editor: { canvasMode: v[0] } })
        }
      >
        <Tooltip>
          <TooltipTrigger render={<ToggleGroupItem value="mouse" aria-label="Mouse" />}>
            <Mouse />
          </TooltipTrigger>
          <TooltipContent>
            Mouse: the wheel zooms, drag the background to move around
          </TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger render={<ToggleGroupItem value="touchpad" aria-label="Touchpad" />}>
            <Touchpad />
          </TooltipTrigger>
          <TooltipContent>Touchpad: two fingers move around, pinch to zoom</TooltipContent>
        </Tooltip>
      </ToggleGroup>
      <Tooltip>
        <TooltipTrigger
          render={
            <Toggle
              size="sm"
              aria-label="Minimap"
              pressed={minimap}
              onPressedChange={(v) => useViewOptions.getState().setMinimap(v)}
            />
          }
        >
          <MapIcon />
        </TooltipTrigger>
        <TooltipContent>{minimap ? "Hide the minimap" : "Show the minimap"}</TooltipContent>
      </Tooltip>
    </Panel>
  );
}
