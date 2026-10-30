import { useAuthoredFrame } from "#video/core/authored-clock.ts";
import { useNeutralTheme } from "#video/lib/neutral-ui/index.ts";
import { mixOklch } from "#video/lib/ui-kit/color.ts";

export interface SkeletonBlockProps {
  width?: number | string;
  height?: number;
  radius?: number;
  speed?: number;
  baseColor?: string;
  highlightColor?: string;
  flexShrink?: number;
  className?: string;
}

const SWEEP_FRAMES = 60;

export function SkeletonBlock({
  width = 120,
  height = 16,
  radius = 6,
  speed = 1,
  baseColor,
  highlightColor,
  flexShrink,
  className,
}: SkeletonBlockProps) {
  const theme = useNeutralTheme();
  const base = baseColor ?? theme.muted;
  const highlight = highlightColor ?? mixOklch(base, theme.foreground, 0.13);

  const frame = useAuthoredFrame() * speed;
  const progress = (frame % SWEEP_FRAMES) / SWEEP_FRAMES;
  const positionX = 100 - progress * 200;

  return (
    <div
      className={className}
      style={{
        width,
        height,
        flexShrink,
        borderRadius: radius,
        background: `linear-gradient(90deg, ${base} 20%, ${highlight} 50%, ${base} 80%)`,
        backgroundSize: "200% 100%",
        backgroundPosition: `${positionX}% 0`,
      }}
    />
  );
}
