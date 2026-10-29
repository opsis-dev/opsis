import { AbsoluteFill, random } from "remotion";
import type { ElementSettings } from "#video/generated/spec.ts";
import type { SplitUnitState } from "#video/lib/text/text-split.ts";
import { HAND_FAMILY } from "#video/parts/handwrite.tsx";
import { SplitTextChars } from "#video/parts/split-text-chars.tsx";

function jitter(seed: string | number, index: number, salt: number): number {
  return random(`${seed}-${index}-${salt}`) * 2 - 1;
}

export function HandwritingText({
  text,
  penSize,
  penColor,
  inkSoftness,
  wobble,
  seed,
  color,
  fontFamily,
  fontSize,
  fontWeight,
  staggerInFrames,
  durationInFrames,
  wordSpacing,
}: ElementSettings["handwriting-text"]) {
  const nib = penColor;
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
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", textAlign: "center" }}>
      <SplitTextChars
        text={text}
        color={color}
        fontFamily={fontFamily || HAND_FAMILY}
        fontSize={fontSize}
        fontWeight={fontWeight}
        wordSpacing={wordSpacing}
        mode="chars"
        order="start"
        staggerInFrames={staggerInFrames}
        durationInFrames={durationInFrames}
        effect="none"
        renderUnit={renderUnit}
      />
    </AbsoluteFill>
  );
}
