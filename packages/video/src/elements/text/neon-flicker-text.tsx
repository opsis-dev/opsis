import { AbsoluteFill, random, useCurrentFrame, useVideoConfig } from "remotion";
import type { ElementSettings } from "#video/generated/spec.ts";
import type { SplitUnitState } from "#video/lib/text/text-split.ts";
import { SplitTextChars } from "#video/parts/split-text-chars.tsx";

export function NeonFlickerText({
  text,
  glowColor,
  glowSize,
  offColor: offSetting,
  offLevel,
  hum,
  buzz,
  flickerOnExit,
  order,
  staggerInFrames,
  seed,
  color,
  fontSize,
  fontWeight,
  durationInFrames,
  humRate,
  wordSpacing,
}: ElementSettings["neon-flicker-text"]) {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const offColor = offSetting || `color-mix(in srgb, ${glowColor} 16%, transparent)`;
  const humStep = (humRate * Math.PI * 2) / fps;

  function renderUnit(unit: SplitUnitState) {
    const striking = unit.enter > 0 && unit.enter < 1;
    const leaving = flickerOnExit && unit.exit > 0;

    let lit: number;
    if (unit.enter <= 0) {
      lit = 0;
    } else if (striking) {
      const beat = Math.floor(frame / 2);
      lit = random(`${seed}:${unit.index}:${beat}`) < 0.22 + 0.78 * unit.enter ? 1 : offLevel;
    } else {
      const wave = 0.5 + 0.5 * Math.sin(frame * humStep + unit.index * 1.3);
      const stutter =
        buzz > 0 && random(`${seed + 7}:${unit.index}:${Math.floor(frame / 3)}`) < buzz;
      lit = stutter ? offLevel + 0.2 : 1 - hum * wave;
    }

    if (leaving) {
      const dying = random(`${seed + 3}:${unit.index}:${Math.floor(frame / 2)}`);
      if (dying < unit.exit) lit = offLevel;
    }

    const on = lit > offLevel + 0.001;
    const glow = Math.max(0, lit);

    return (
      <span
        style={{
          display: "inline-block",
          whiteSpace: "pre",
          opacity: unit.enter > 0 ? 1 - unit.exit : 0,
          color: on ? color : offColor,
          textShadow: on
            ? `0 0 ${(glowSize * 0.16 * glow).toFixed(1)}px ${glowColor}, 0 0 ${(glowSize * 0.5 * glow).toFixed(1)}px ${glowColor}, 0 0 ${(glowSize * glow).toFixed(1)}px ${glowColor}, 0 0 ${(glowSize * 2.2 * glow).toFixed(1)}px ${glowColor}`
            : "none",
        }}
      >
        {unit.text}
      </span>
    );
  }

  return (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", textAlign: "center" }}>
      <SplitTextChars
        text={text}
        order={order}
        staggerInFrames={staggerInFrames}
        durationInFrames={durationInFrames}
        fontSize={fontSize}
        fontWeight={fontWeight}
        wordSpacing={wordSpacing}
        exit
        color={color}
        seed={seed}
        effect="none"
        renderUnit={renderUnit}
      />
    </AbsoluteFill>
  );
}
