import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { CODE_THEMES } from "#video/lib/code/code-syntax.tsx";
import { getSafeAreaPadding } from "#video/lib/layout/layout.ts";
import { EASING } from "#video/lib/motion/motion-tokens.ts";

const fontFamily = '"Inter"';

export type ToastTone = "info" | "success" | "warn" | "error";

export type Toast = {
  title: string;
  body?: string;
  meta?: string;
  tone?: ToastTone;
  atSeconds?: number;
  holdFor?: number;
};

export type NotificationStackProps = {
  toasts?: Toast[];
  align?: "top-right" | "top-left" | "bottom-right" | "bottom-left";
  startAtSeconds?: number;
  staggerSeconds?: number;
  lifeSeconds?: number;
  showProgress?: boolean;
  accentColor?: string;
  theme?: "dark" | "light";
  speed?: number;
};

const DEFAULT_TOASTS: Toast[] = [
  { title: "Render finished", body: "launch-teaser.mp4 · 1080p", tone: "success", meta: "now" },
  { title: "3 new comments", body: "Mina left notes on scene 4", tone: "info", meta: "1m" },
  { title: "Storage at 86%", body: "Archive old renders to free space", tone: "warn", meta: "2m" },
  { title: "Upload failed", body: "b-roll-rooftop.mov · retrying", tone: "error", meta: "2m" },
];

const T = {
  enterFor: 0.34,
  leaveFor: 0.3,
} as const;

const clamp = {
  extrapolateLeft: "clamp",
  extrapolateRight: "clamp",
} as const;

const toneColor = (tone: ToastTone, accentColor: string): string =>
  tone === "success"
    ? "#7FD1A0"
    : tone === "error"
      ? "#F97362"
      : tone === "warn"
        ? accentColor
        : "#7DD3E8";

const ToneGlyph: React.FC<{ tone: ToastTone; size: number; color: string }> = ({
  tone,
  size,
  color,
}) => {
  if (tone === "success") {
    return (
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path
          d="M5.5 12.5l4.3 4.3L18.5 7.6"
          stroke={color}
          strokeWidth={2.4}
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
    );
  }
  if (tone === "error") {
    return (
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path
          d="M7.4 7.4l9.2 9.2M16.6 7.4l-9.2 9.2"
          stroke={color}
          strokeWidth={2.4}
          strokeLinecap="round"
        />
      </svg>
    );
  }
  if (tone === "warn") {
    return (
      <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
        <path d="M12 4.6L21 19.4H3L12 4.6Z" stroke={color} strokeWidth={2} strokeLinejoin="round" />
        <path d="M12 10.2v4.1" stroke={color} strokeWidth={2} strokeLinecap="round" />
        <circle cx="12" cy="16.7" r="1.05" fill={color} />
      </svg>
    );
  }
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <circle cx="12" cy="12" r="8.4" stroke={color} strokeWidth={2} />
      <path d="M12 11v5.4" stroke={color} strokeWidth={2} strokeLinecap="round" />
      <circle cx="12" cy="7.9" r="1.15" fill={color} />
    </svg>
  );
};

export const NotificationStack: React.FC<NotificationStackProps> = ({
  toasts = DEFAULT_TOASTS,
  align = "top-right",
  startAtSeconds = 0.3,
  staggerSeconds = 0.62,
  lifeSeconds = 2.3,
  showProgress = true,
  accentColor = "#E8B86D",
  theme = "dark",
  speed = 1,
}) => {
  const frame = useCurrentFrame();
  const { fps, width, height } = useVideoConfig();
  const palette = CODE_THEMES[theme];
  const safe = getSafeAreaPadding({ width, height });

  const at = (seconds: number) => (seconds * fps) / speed;
  const ease = (from: number, to: number, easing = EASING.enter) =>
    interpolate(frame, [at(from), at(to)], [0, 1], { easing, ...clamp });

  const portrait = height > width;
  const u = portrait ? Math.min(width / 496, height / 896) : Math.min(width / 1024, height / 576);

  const isRight = align.endsWith("right");
  const isBottom = align.startsWith("bottom");

  const gap = 11 * u;
  const toastW = Math.min(width - safe.paddingLeft - safe.paddingRight, 370 * u);

  const plan = toasts.map((toast, index) => {
    const spawn = toast.atSeconds ?? startAtSeconds + staggerSeconds * index;
    const life = toast.holdFor ?? lifeSeconds;
    const enter = ease(spawn, spawn + T.enterFor);
    const leave = ease(spawn + life, spawn + life + T.leaveFor, EASING.exit);
    return {
      spawn,
      life,
      enter,
      leave,
      occupancy: enter * (1 - leave),
      boxH: toast.body ? 82 * u : 60 * u,
    };
  });

  return (
    <AbsoluteFill style={{ pointerEvents: "none", fontFamily }}>
      <div
        style={{
          position: "absolute",
          left: isRight ? undefined : safe.paddingLeft,
          right: isRight ? safe.paddingRight : undefined,
          top: isBottom ? undefined : safe.paddingTop,
          bottom: isBottom ? safe.paddingBottom : undefined,
          width: toastW,
        }}
      >
        {toasts.map((toast, index) => {
          const beat = plan[index];
          const tone = toast.tone ?? "info";
          const color = toneColor(tone, accentColor);

          const offset = plan
            .slice(0, index)
            .reduce((sum, other) => sum + other.occupancy * (other.boxH + gap), 0);

          const life = beat.life;
          const remaining = showProgress
            ? 1 - interpolate(frame, [at(beat.spawn), at(beat.spawn + life)], [0, 1], clamp)
            : 1;

          const slide = spring({
            frame: frame - at(beat.spawn),
            fps,
            config: { damping: 19, stiffness: 150, mass: 0.7 },
          });
          const away = isRight ? 1 : -1;

          return (
            <div
              key={`${toast.title}-${index}`}
              style={{
                position: "absolute",
                left: 0,
                right: 0,
                top: isBottom ? undefined : 0,
                bottom: isBottom ? 0 : undefined,
                transform: `translateY(${isBottom ? -offset : offset}px) translateX(${
                  ((1 - slide) * 44 * u + beat.leave * 64 * u) * away
                }px)`,
                opacity: beat.enter * (1 - beat.leave),
              }}
            >
              <div
                style={{
                  height: beat.boxH,
                  display: "flex",
                  alignItems: "center",
                  gap: 12 * u,
                  padding: `0 ${15 * u}px`,
                  borderRadius: 14 * u,
                  background: `${palette.window}F2`,
                  border: `1px solid ${palette.border}`,
                  boxShadow: `inset 0 1px 0 ${palette.highlight}, 0 ${
                    16 * u
                  }px ${40 * u}px ${palette.shadow}`,
                  overflow: "hidden",
                  position: "relative",
                }}
              >
                <div
                  style={{
                    flexShrink: 0,
                    width: 32 * u,
                    height: 32 * u,
                    borderRadius: 10 * u,
                    background: `color-mix(in srgb, ${color} 12%, transparent)`,
                    border: `1px solid color-mix(in srgb, ${color} 35%, transparent)`,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                  }}
                >
                  <ToneGlyph tone={tone} size={19 * u} color={color} />
                </div>

                <div style={{ minWidth: 0, flex: 1 }}>
                  <div
                    style={{
                      display: "flex",
                      alignItems: "baseline",
                      gap: 10 * u,
                    }}
                  >
                    <span
                      style={{
                        color: palette.fg,
                        fontSize: 18 * u,
                        fontWeight: 600,
                        whiteSpace: "nowrap",
                        textOverflow: "ellipsis",
                        overflow: "hidden",
                      }}
                    >
                      {toast.title}
                    </span>
                    {toast.meta ? (
                      <span
                        style={{
                          marginLeft: "auto",
                          flexShrink: 0,
                          color: palette.faint,
                          fontSize: 14.5 * u,
                          fontWeight: 500,
                        }}
                      >
                        {toast.meta}
                      </span>
                    ) : null}
                  </div>
                  {toast.body ? (
                    <div
                      style={{
                        marginTop: 3 * u,
                        color: palette.dim,
                        fontSize: 15 * u,
                        fontWeight: 500,
                        whiteSpace: "nowrap",
                        textOverflow: "ellipsis",
                        overflow: "hidden",
                      }}
                    >
                      {toast.body}
                    </div>
                  ) : null}
                </div>

                {showProgress ? (
                  <div
                    style={{
                      position: "absolute",
                      left: 0,
                      bottom: 0,
                      height: 2.5 * u,
                      width: `${remaining * 100}%`,
                      background: color,
                      opacity: 0.75,
                    }}
                  />
                ) : null}
              </div>
            </div>
          );
        })}
      </div>
    </AbsoluteFill>
  );
};
