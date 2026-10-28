import type React from "react";
import { AbsoluteFill, random } from "remotion";
import { type ElementProps } from "#video/common/spec.ts";
import type { SplitUnitState } from "#video/lib/text/text-split.ts";
import { SplitTextChars, type SplitTextCharsProps } from "#video/parts/split-text-chars.tsx";

export type HandwritingTextProps = Omit<
  SplitTextCharsProps,
  "effect" | "renderUnit" | "order" | "mode"
> & {
  penSize?: number;
  penColor?: string;
  inkSoftness?: number;
  wobble?: number;
  seed?: string | number;
};

function jitter(seed: string | number, index: number, salt: number): number {
  return random(`${seed}-${index}-${salt}`) * 2 - 1;
}

const HandwritingTextView: React.FC<HandwritingTextProps> = ({
  penSize = 0.14,
  penColor,
  inkSoftness = 0.18,
  wobble = 1.6,
  seed = "handwriting",
  color = "#f4f4f5",
  fontFamily = '"Segoe Script", "Bradley Hand", "Snell Roundhand", cursive',
  staggerInFrames = 3,
  durationInFrames = 10,
  ...splitProps
}) => {
  const nib = penColor ?? color;
  const softness = Math.max(0, inkSoftness) * 100;

  const renderUnit = (unit: SplitUnitState) => {
    const written = Math.max(0, Math.min(1, unit.enter));
    const stop = written * 100;
    const ink = `linear-gradient(to right, #000 ${stop.toFixed(2)}%, rgba(0,0,0,0) ${Math.min(100, stop + softness).toFixed(2)}%)`;
    const tilt = jitter(seed, unit.index, 1) * wobble;
    const drift = jitter(seed, unit.index, 2) * wobble * 0.012;
    const writing = written > 0.02 && written < 0.98;

    return (
      <span
        style={{
          position: "relative",
          display: "inline-block",
          whiteSpace: "pre",
          opacity: unit.opacity,
          rotate: `${tilt.toFixed(3)}deg`,
          translate: `0 ${drift.toFixed(4)}em`,
        }}
      >
        <span
          style={{
            display: "inline-block",
            maskImage: ink,
            WebkitMaskImage: ink,
          }}
        >
          {unit.text}
        </span>
        {penSize > 0 ? (
          <span
            aria-hidden
            style={{
              position: "absolute",
              left: `${stop.toFixed(2)}%`,
              bottom: "0.08em",
              width: `${penSize}em`,
              height: `${penSize}em`,
              marginLeft: `${(-penSize / 2).toFixed(4)}em`,
              borderRadius: "50%",
              background: nib,
              opacity: writing ? unit.opacity : 0,
            }}
          />
        ) : null}
      </span>
    );
  };

  return (
    <SplitTextChars
      {...splitProps}
      color={color}
      fontFamily={fontFamily}
      mode="chars"
      order="start"
      staggerInFrames={staggerInFrames}
      durationInFrames={durationInFrames}
      effect="none"
      renderUnit={renderUnit}
    />
  );
};

export function HandwritingText(props: ElementProps & { children?: React.ReactNode }) {
  return (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", textAlign: "center" }}>
      <HandwritingTextView
        {...(props as unknown as React.ComponentProps<typeof HandwritingTextView>)}
      />
    </AbsoluteFill>
  );
}
