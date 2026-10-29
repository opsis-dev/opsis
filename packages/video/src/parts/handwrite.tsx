import { useAuthoredFrame } from "#video/core/authored-clock.ts";
import { DEFAULT_STEP, hashRange, qf, qstep } from "#video/lib/motion/stop-motion.ts";

export const HAND_FAMILY = '"Caveat"';

export interface HandwriteProps {
  text: string;
  fontSize?: number;
  color?: string;
  delay?: number;
  perStep?: number;
  weight?: number | string;
  align?: "left" | "center";
  fontFamily?: string;
  step?: number;
  wobble?: number;
}

export function Handwrite({
  text,
  fontSize = 54,
  color = "#26242c",
  delay = 0,
  perStep = 1.6,
  weight = 600,
  align = "center",
  fontFamily = HAND_FAMILY,
  step = DEFAULT_STEP,
  wobble = 1,
}: HandwriteProps) {
  const frame = useAuthoredFrame();
  const steps = Math.max(0, qstep(Math.max(0, qf(frame, step) - delay), step));
  const shown = Math.floor(steps * perStep);
  const chars = Array.from(text);
  const freshFrom = shown - Math.ceil(perStep);

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: align === "center" ? "center" : "flex-start",
        background: "transparent",
      }}
    >
      <span
        style={{
          display: "inline-block",
          fontFamily: `${fontFamily}, cursive`,
          fontWeight: weight,
          fontSize,
          lineHeight: 1.15,
          color,
          whiteSpace: "pre-wrap",
          textAlign: align,
        }}
      >
        {chars.map((char, i) => {
          const visible = i < shown;
          const slant = hashRange(`hw:${text}:${i}:r`, -3.2, 3.2) * wobble;
          const dy = hashRange(`hw:${text}:${i}:y`, -1.8, 1.8) * wobble;
          const fresh = visible && i >= freshFrom;
          return (
            <span
              key={i}
              style={{
                display: "inline-block",
                whiteSpace: "pre",
                opacity: visible ? 1 : 0,
                translate: visible ? `0 ${dy}px` : undefined,
                rotate: visible ? `${slant}deg` : undefined,
                scale: visible ? `${fresh ? 1.06 : 1}` : undefined,
              }}
            >
              {char}
            </span>
          );
        })}
      </span>
    </div>
  );
}

export function handwriteDuration(
  text: string,
  options?: { perStep?: number; step?: number },
): number {
  const perStep = options?.perStep ?? 1.6;
  const step = options?.step ?? DEFAULT_STEP;
  return Math.ceil(Array.from(text).length / perStep) * step + step;
}
