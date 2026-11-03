import { boundsFromRect } from "@opsis/video/common/geometry";
import { uniqueKeys } from "@opsis/video/common/lists";
import { designScaleFor } from "@opsis/video/spec";
import type { TerminalFrames } from "@opsis/video/terminal/frames";
import { screenSize } from "@opsis/video/terminal/view";
import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Spinner } from "@/components/ui/spinner";
import { cn } from "@/lib/utils";
import { Ripple } from "../app/motion";
import { TerminalPicture } from "../app/terminal-picture";
import { CLICK_RIPPLE_MS, type LiveCapture } from "../store/live";
import { FrameOverlay, OVERLAY_WIDTHS } from "./frame-overlay";
import { STATUS_LOOKS } from "./status-icon";
import { PLAYER_LOOK } from "./terminal-player";

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

export function LiveBadge({ className }: { className?: string }) {
  return (
    <span
      className={cn(
        "bg-live text-live-foreground inline-flex items-center gap-1.5 rounded-sm px-1.5 py-0.5 text-[11px] leading-none font-semibold tracking-wide",
        className,
      )}
    >
      <span className="bg-live-foreground size-1.5 rounded-full" />
      LIVE
    </span>
  );
}

function LiveTerminal({
  capture,
  frames,
  className,
}: {
  capture: LiveCapture;
  frames: TerminalFrames;
  className?: string;
}) {
  const frame = frames.frames[0];
  if (!frame) return null;
  const size = screenSize(PLAYER_LOOK, frame.cols, frame.rows);
  const current = capture.steps.find((s) => s.status === "running");
  return (
    <div
      className={cn("bg-muted relative overflow-hidden rounded-md", className)}
      style={{ aspectRatio: `${size.width} / ${size.height}` }}
    >
      <TerminalPicture frames={frames} frame={frame} look={PLAYER_LOOK} cursorOn />
      {capture.status === "live" ? <LiveBadge className="absolute top-2 left-2 z-10" /> : null}
      <div className="absolute inset-x-2 bottom-2 flex items-center gap-1.5">
        {capture.status === "live" ? null : <StatusBadge status={capture.status} />}
        {current ? (
          <Badge variant="outline" className="bg-background/80 truncate">
            {current.label}
          </Badge>
        ) : null}
      </div>
    </div>
  );
}

export function LiveView({ capture, className }: { capture: LiveCapture; className?: string }) {
  if (capture.terminal)
    return <LiveTerminal capture={capture} frames={capture.terminal} className={className} />;
  if (!capture.viewport)
    return (
      <div
        className={cn(
          "text-muted-foreground grid aspect-video place-items-center rounded-md bg-black",
          className,
        )}
      >
        <Spinner />
      </div>
    );
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
          alt={capture.title || "Live recording"}
          className="absolute inset-0 size-full object-contain"
        />
      ) : (
        <div className="text-muted-foreground absolute inset-0 grid place-items-center text-xs">
          <Spinner />
        </div>
      )}
      {capture.frameUrl && capture.status === "live" ? (
        <LiveBadge className="absolute top-2 left-2 z-10" />
      ) : null}
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
        {capture.status === "live" ? null : <StatusBadge status={capture.status} />}
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
