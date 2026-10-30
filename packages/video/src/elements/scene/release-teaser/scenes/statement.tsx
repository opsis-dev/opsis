import { Interactive, interpolate } from "remotion";
import { CLAMPED } from "#video/core/anim.ts";
import type { ReleaseTeaserScene } from "#video/elements/scene/release-teaser/content.ts";
import { characterState, fitText, graphemes } from "#video/elements/scene/release-teaser/motion.ts";

export function Statement({
  scene,
  text,
  frame,
  duration,
}: {
  scene: ReleaseTeaserScene;
  text: string;
  frame: number;
  duration: number;
}) {
  return (
    <Interactive.Div
      name="Statement"
      role="img"
      aria-label={text}
      style={{
        position: "absolute",
        inset: 0,
        display: "flex",
        flexDirection: "column",
        justifyContent: "center",
        alignItems: "center",
        fontSize: fitText(text) * scene.textSize,
        fontWeight: scene.statementWeight,
        lineHeight: 1.13,
        color: scene.theme.foreground,
        scale: scene.reducedMotion ? 1 : interpolate(frame, [0, duration], [0.99, 1.025], CLAMPED),
      }}
    >
      {text.split("\n").map((line, lineIndex) => {
        const chars = graphemes(line);
        return (
          <div
            key={`${lineIndex}-${line}`}
            style={{
              whiteSpace: "pre",
              textAlign: "center",
              minHeight: "1.13em",
            }}
          >
            {scene.reducedMotion
              ? line
              : chars.map((char, index) => {
                  const state = characterState(frame, duration, index, chars.length);
                  return (
                    <span
                      key={`${index}-${char}`}
                      style={{
                        display: "inline-block",
                        opacity: state.opacity,
                        filter: state.blur > 0.08 ? `blur(${state.blur}px)` : undefined,
                        translate: `0 ${state.y}px`,
                      }}
                    >
                      {char}
                    </span>
                  );
                })}
          </div>
        );
      })}
    </Interactive.Div>
  );
}
