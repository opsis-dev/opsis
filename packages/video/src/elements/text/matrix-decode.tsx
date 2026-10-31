import { random } from "@opsis/runtime/random.ts";
import { fontOr } from "#video/common/fonts.ts";
import { useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";

export function MatrixDecode({
  fontFamily,
  text,
  charset,
  fontSize,
  color,
  fontWeight,
  revealDuration,
  speed,
  background,
  scrambleEvery,
  seed,
  letterSpacing,
}: ElementSettings["matrix-decode"]) {
  const frame = useAuthoredFrame() * speed;

  let output = "";
  for (let i = 0; i < text.length; i++) {
    const revealFrame = (i / Math.max(text.length, 1)) * revealDuration;
    if (text[i] === " ") {
      output += " ";
    } else if (frame >= revealFrame) {
      output += text[i];
    } else {
      const tick = Math.floor(frame / Math.max(scrambleEvery, 1));
      const r = random(seed === 0 ? `${i}-${tick}` : `${seed}-${i}-${tick}`);
      const ch = charset[Math.floor(r * charset.length)];
      output += ch;
    }
  }

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
          whiteSpace: "pre",
          fontFamily: fontOr(fontFamily, "code"),
        }}
      >
        {output}
      </span>
    </div>
  );
}
