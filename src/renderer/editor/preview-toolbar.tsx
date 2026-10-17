import { VARIANTS } from "@/shared/nodes/output";
import { formatDuration } from "@/shared/format";
import type { TimelineSpec } from "@/shared/schema/timeline";
import { Badge } from "@/renderer/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/renderer/components/ui/select";
import { Spinner } from "@/renderer/components/ui/spinner";
import { ToggleGroup, ToggleGroupItem } from "@/renderer/components/ui/toggle-group";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";
import { isPreviewQuality, isPreviewShape, useUiStore } from "../store/ui";

const PROJECT_SHAPE = "Project shape";
const shapeLabel = (shape: string) => (shape === "project" ? PROJECT_SHAPE : shape);

function UpdatingSound() {
  return (
    <Tooltip>
      <TooltipTrigger render={<span className="flex shrink-0 items-center gap-1" />}>
        <Spinner className="size-3" />
        Updating sound
      </TooltipTrigger>
      <TooltipContent>
        The sound is being mixed again. You hear the previous mix until it's ready.
      </TooltipContent>
    </Tooltip>
  );
}

function ShapeSelect() {
  const shape = useUiStore((s) => s.previewShape);
  const setShape = useUiStore((s) => s.setPreviewShape);
  return (
    <Select value={shape} onValueChange={(v) => isPreviewShape(v) && setShape(v)}>
      <SelectTrigger size="sm" className="w-32" aria-label="Preview shape">
        <SelectValue>{shapeLabel}</SelectValue>
      </SelectTrigger>
      <SelectContent>
        <SelectGroup>
          <SelectItem value="project">{PROJECT_SHAPE}</SelectItem>
          {VARIANTS.map((v) => (
            <SelectItem key={v} value={v}>
              {v}
            </SelectItem>
          ))}
        </SelectGroup>
      </SelectContent>
    </Select>
  );
}

function QualityToggle() {
  const quality = useUiStore((s) => s.previewQuality);
  const setQuality = useUiStore((s) => s.setPreviewQuality);
  return (
    <ToggleGroup
      variant="outline"
      size="sm"
      value={[quality]}
      onValueChange={(v) => isPreviewQuality(v[0]) && setQuality(v[0])}
    >
      <Tooltip>
        <TooltipTrigger render={<ToggleGroupItem value="fast" />}>Fast</TooltipTrigger>
        <TooltipContent>
          Smooth playback: smaller recordings and no motion blur. The export is unchanged.
        </TooltipContent>
      </Tooltip>
      <Tooltip>
        <TooltipTrigger render={<ToggleGroupItem value="exact" />}>Exact</TooltipTrigger>
        <TooltipContent>Exactly what the export looks like. May stutter.</TooltipContent>
      </Tooltip>
    </ToggleGroup>
  );
}

export function PreviewToolbar({
  spec,
  sourceTitle,
  stale,
}: {
  spec: TimelineSpec;
  sourceTitle: string | null;
  stale: boolean;
}) {
  return (
    <div className="text-muted-foreground flex items-center gap-2 text-xs">
      {sourceTitle ? <Badge variant="secondary">{sourceTitle}</Badge> : null}
      <span className="truncate tabular-nums">
        {spec.width}×{spec.height} · {spec.fps} fps ·{" "}
        {formatDuration((spec.durationInFrames / spec.fps) * 1000)}
      </span>
      {stale ? <UpdatingSound /> : null}
      <div className="flex-1" />
      <ShapeSelect />
      <QualityToggle />
    </div>
  );
}
