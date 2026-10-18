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
import { whenFirstOneOf, whenOneOf } from "@/renderer/lib/one-of";
import { formatDuration } from "@/shared/format";
import { VARIANTS } from "@/shared/nodes/output";
import { PREVIEW_QUALITIES, type TimelineSpec } from "@/shared/schema/timeline";
import { framesToMs } from "@/shared/timeline/time";
import { PREVIEW_SHAPES, useViewOptions } from "../store/view-options";
import { WithTooltip } from "./tooltip-button";

const PROJECT_SHAPE = "Project shape";
const shapeLabel = (shape: string) => (shape === "project" ? PROJECT_SHAPE : shape);

function UpdatingSound() {
  return (
    <WithTooltip
      trigger={<span className="flex shrink-0 items-center gap-1" />}
      tip="The sound is being mixed again. You hear the previous mix until it's ready."
    >
      <Spinner className="size-3" />
      Updating sound
    </WithTooltip>
  );
}

function ShapeSelect() {
  const shape = useViewOptions((s) => s.previewShape);
  const setOption = useViewOptions((s) => s.setViewOption);
  return (
    <Select
      value={shape}
      onValueChange={whenOneOf(PREVIEW_SHAPES, (v) => setOption("previewShape", v))}
    >
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
  const quality = useViewOptions((s) => s.previewQuality);
  const setOption = useViewOptions((s) => s.setViewOption);
  return (
    <ToggleGroup
      variant="outline"
      size="sm"
      value={[quality]}
      onValueChange={whenFirstOneOf(PREVIEW_QUALITIES, (v) => setOption("previewQuality", v))}
    >
      <WithTooltip
        trigger={<ToggleGroupItem value="fast" />}
        tip="Smooth playback: smaller recordings and no motion blur. The export is unchanged."
      >
        Fast
      </WithTooltip>
      <WithTooltip
        trigger={<ToggleGroupItem value="exact" />}
        tip="Exactly what the export looks like. May stutter."
      >
        Exact
      </WithTooltip>
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
        {formatDuration(framesToMs(spec.durationInFrames, spec.fps))}
      </span>
      {stale ? <UpdatingSound /> : null}
      <div className="flex-1" />
      <ShapeSelect />
      <QualityToggle />
    </div>
  );
}
