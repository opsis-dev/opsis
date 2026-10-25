import { uniqueKeys } from "@opsis/core/lists";
import { boundsFromRect } from "@opsis/core/schema/actions";
import { designScaleFor } from "@opsis/video/spec";
import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Spinner } from "@/components/ui/spinner";
import { cn } from "@/lib/utils";
import { Ripple } from "../app/motion";
import { CLICK_RIPPLE_MS, type LiveCapture } from "../store/live";
import { FrameOverlay, OVERLAY_WIDTHS } from "./frame-overlay";
import { STATUS_LOOKS } from "./status-icon";

const RIPPLE_START_PX = 12;
const RIPPLE_GROWTH_PX = 36;

function StatusBadge({ status }: { status: LiveCapture["status"] }) {
  const { icon: Icon, word, short, badge } = STATUS_LOOKS[status];
  return (
    <Badge variant={badge}>
      <Icon data-icon="inline-start" />
      {short ?? word}
    </Badge>
  );
}

function ClickRipple({ click, width }: { click: LiveCapture["clicks"][number]; width: number }) {
  const [age] = useState(() => Date.now() - click.at);
  if (age >= CLICK_RIPPLE_MS) return null;
  const scale = designScaleFor(width);
  const start = age / CLICK_RIPPLE_MS;
  return (
    <Ripple
      cx={click.x}
      cy={click.y}
      from={(RIPPLE_START_PX + start * RIPPLE_GROWTH_PX) * scale}
      to={(RIPPLE_START_PX + RIPPLE_GROWTH_PX) * scale}
      opacity={1 - start}
      strokeWidth={width * OVERLAY_WIDTHS.rippleStroke}
      seconds={(CLICK_RIPPLE_MS - age) / 1000}
      className="fill-none stroke-primary"
    />
  );
}

export function LiveView({ capture, className }: { capture: LiveCapture; className?: string }) {
  const { width, height } = capture.viewport;
  const current = capture.steps.find((s) => s.status === "running");
  const target = capture.target;
  const clickKeys = uniqueKeys(capture.clicks.map((c) => String(c.at)));
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
        {capture.clicks.map((c, i) => (
          <ClickRipple key={clickKeys[i]} click={c} width={width} />
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
