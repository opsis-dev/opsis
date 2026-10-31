import { useFrame } from "@opsis/runtime/clock.tsx";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { useId } from "react";
import { CLAMPED } from "#video/core/anim.ts";
import { EASING } from "#video/lib/motion/motion-tokens.ts";
import {
  clampProgress,
  fitViewBox,
  getPathDrawStyles,
  samplePath,
} from "#video/lib/paths/path-utils.ts";

export type PathDrawProps = {
  d: string | string[];
  durationInFrames: number;
  delayInFrames: number;
  staggerInFrames: number;
  stroke: string;
  strokeWidth: number;
  width: number;
  height: number;
  viewBox?: string;
  fill?: string;
  head: boolean;
  glow: boolean;
};

const AUTO_FIT_PADDING = 8;

export function PathDraw({
  d,
  durationInFrames,
  delayInFrames,
  staggerInFrames,
  stroke,
  strokeWidth,
  width,
  height,
  viewBox,
  fill,
  head,
  glow,
}: PathDrawProps) {
  const frame = useFrame();
  const id = useId();
  const paths = (Array.isArray(d) ? d : [d]).filter(Boolean);
  const filterId = `path-draw-glow-${id}`;
  const resolvedViewBox = viewBox ?? fitViewBox(paths, AUTO_FIT_PADDING);
  const [boxX, boxY, boxWidth, boxHeight] = resolvedViewBox.split(/[\s,]+/).map(Number);

  return (
    <svg width={width} height={height} viewBox={resolvedViewBox}>
      {glow ? (
        <defs>
          <filter
            id={filterId}
            filterUnits="userSpaceOnUse"
            x={boxX}
            y={boxY}
            width={boxWidth}
            height={boxHeight}
          >
            <feGaussianBlur stdDeviation="1.5" result="blur" />
            <feMerge>
              <feMergeNode in="blur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
        </defs>
      ) : null}

      {paths.map((path, index) => {
        const start = delayInFrames + index * staggerInFrames;
        const progress = clampProgress(
          interpolate(frame, [start, start + durationInFrames], [0, 1], {
            ...CLAMPED,
            easing: EASING.enter,
          }),
        );
        const { strokeDasharray, strokeDashoffset } = getPathDrawStyles(progress, path);
        const fillOpacity = fill
          ? interpolate(
              frame,
              [start + durationInFrames * 0.75, start + durationInFrames * 1.3],
              [0, 1],
              { ...CLAMPED, easing: EASING.enter },
            )
          : 0;
        const tip = head && progress > 0 && progress < 1 ? samplePath(path, progress) : null;

        return (
          <g key={`${path.slice(0, 16)}-${index}`}>
            {fill ? <path d={path} fill={fill} opacity={fillOpacity} stroke="none" /> : null}
            <path
              d={path}
              fill="none"
              stroke={stroke}
              strokeWidth={strokeWidth}
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeDasharray={strokeDasharray}
              strokeDashoffset={strokeDashoffset}
              filter={glow ? `url(#${filterId})` : undefined}
              opacity={0.95}
            />
            {tip ? (
              <circle
                cx={tip.x}
                cy={tip.y}
                r={strokeWidth * 0.9}
                fill={stroke}
                filter={glow ? `url(#${filterId})` : undefined}
              />
            ) : null}
          </g>
        );
      })}
    </svg>
  );
}
