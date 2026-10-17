import { CircleCheck, CircleX, Radio, type LucideIcon } from "lucide-react";
import { useState } from "react";
import { boundsFromRect } from "@/shared/schema/actions";
import { Badge } from "@/renderer/components/ui/badge";
import { Spinner } from "@/renderer/components/ui/spinner";
import { cn } from "@/renderer/lib/utils";
import { designScaleFor } from "@/video/camera/lib/design-width";
import { Ripple } from "../app/motion";
import type { LiveCapture } from "../store/live";
import { FrameOverlay, OVERLAY_WIDTHS } from "./frame-overlay";

const RIPPLE_MS = 600;
const RIPPLE_START_PX = 12;
const RIPPLE_GROWTH_PX = 36;

export const CAPTURE_STATUS: Record<
  LiveCapture["status"],
  {
    icon: LucideIcon;
    badge: string;
    word: string;
    variant: "default" | "secondary" | "destructive";
  }
> = {
  live: { icon: Radio, badge: "Live", word: "Recording", variant: "default" },
  done: { icon: CircleCheck, badge: "Done", word: "Done", variant: "secondary" },
  error: { icon: CircleX, badge: "Failed", word: "Failed", variant: "destructive" },
};

function StatusBadge({ status }: { status: LiveCapture["status"] }) {
  const { icon: Icon, badge, variant } = CAPTURE_STATUS[status];
  return (
    <Badge variant={variant}>
      <Icon data-icon="inline-start" />
      {badge}
    </Badge>
  );
}

function ClickRipple({ click, width }: { click: LiveCapture["clicks"][number]; width: number }) {
  const [age] = useState(() => Date.now() - click.at);
  if (age >= RIPPLE_MS) return null;
  const scale = designScaleFor(width);
  const start = age / RIPPLE_MS;
  return (
    <Ripple
      cx={click.x}
      cy={click.y}
      from={(RIPPLE_START_PX + start * RIPPLE_GROWTH_PX) * scale}
      to={(RIPPLE_START_PX + RIPPLE_GROWTH_PX) * scale}
      opacity={1 - start}
      strokeWidth={width * OVERLAY_WIDTHS.rippleStroke}
      seconds={(RIPPLE_MS - age) / 1000}
      className="fill-none stroke-primary"
    />
  );
}

export function LiveView({ capture, className }: { capture: LiveCapture; className?: string }) {
  const { width, height } = capture.viewport;
  const current = capture.steps.find((s) => s.status === "running");
  const target = capture.target;
  return (
    <div
      className={cn("relative overflow-hidden rounded-md bg-black", className)}
      style={{ aspectRatio: `${width} / ${height}` }}
    >
      {capture.frameUrl ? (
        <img
          src={capture.frameUrl}
          alt={capture.title}
          className="absolute inset-0 size-full object-contain"
        />
      ) : (
        <div className="text-muted-foreground absolute inset-0 grid place-items-center text-xs">
          <Spinner />
        </div>
      )}
      <FrameOverlay
        width={width}
        height={height}
        box={target ? boundsFromRect(target.rect) : null}
        point={capture.cursor}
        pointSize={OVERLAY_WIDTHS.livePoint}
      >
        {capture.clicks.map((c) => (
          <ClickRipple key={c.at} click={c} width={width} />
        ))}
      </FrameOverlay>
      <div className="absolute inset-x-2 bottom-2 flex items-center gap-1.5">
        <StatusBadge status={capture.status} />
        {current ? (
          <Badge variant="outline" className="bg-background/80 truncate">
            {current.label}
          </Badge>
        ) : null}
        {target?.label && target.label !== current?.label ? (
          <Badge variant="outline" className="bg-background/80 truncate">
            {target.label}
          </Badge>
        ) : null}
      </div>
    </div>
  );
}
