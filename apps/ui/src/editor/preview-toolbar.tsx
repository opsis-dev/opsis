import { formatDuration } from "@opsis/video/common/format";
import { framesToMs } from "@opsis/video/common/time";
import { Badge } from "@/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Spinner } from "@/components/ui/spinner";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import type { TimelineSpec } from "@/generated/bindings";
import { whenFirstOneOf } from "@/lib/one-of";
import { shapes } from "@/store/catalog";
import { PREVIEW_QUALITIES } from "@/timeline/envelopes";
import { isPreviewShape, useViewOptions } from "../store/view-options";
import { WithTooltip } from "./with-tooltip";

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
      onValueChange={(v) => {
        if (isPreviewShape(v)) setOption("previewShape", v);
      }}
    >
      <SelectTrigger size="sm" className="w-32" aria-label="Preview shape">
        <SelectValue>{shapeLabel}</SelectValue>
      </SelectTrigger>
      <SelectContent>
        <SelectGroup>
          <SelectItem value="project">{PROJECT_SHAPE}</SelectItem>
          {shapes().map((choice) => (
            <SelectItem key={choice.shape} value={choice.shape}>
              {choice.shape}
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
