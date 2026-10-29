import { fontOr } from "#video/common/fonts.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { useTypewriter } from "#video/lib/neutral-ui/index.ts";
import { Caret } from "#video/parts/caret.tsx";

export function Typewriter({
  fontFamily,
  text,
  cursor,
  charsPerSecond,
  speed,
  fontSize,
  color,
  cursorColor,
  fontWeight,
  background,
  cursorWidth,
  blinkRate,
  align,
  letterSpacing,
}: ElementSettings["typewriter"]) {
  const tw = useTypewriter(text, { cps: charsPerSecond, speed });

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        background,
      }}
    >
      <span
        style={{
          fontSize,
          fontWeight,
          color,
          letterSpacing: `${letterSpacing}em`,
          fontFamily: fontOr(fontFamily, "main"),
          whiteSpace: "pre-wrap",
          maxWidth: "90%",
          textAlign: align,
        }}
      >
        {tw.text}
        {cursor && (
          <Caret
            color={cursorColor}
            blink={!tw.typing}
            speed={speed}
            blinkPerSecond={blinkRate}
            radius={0}
            style={{
              width: `${cursorWidth}em`,
              height: "1em",
              marginLeft: "0.04em",
              verticalAlign: "text-bottom",
            }}
          />
        )}
      </span>
    </div>
  );
}
