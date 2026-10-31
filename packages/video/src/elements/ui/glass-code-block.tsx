import { Sequence } from "@opsis/runtime/clock.tsx";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { fontOr } from "#video/common/fonts.ts";
import { CLAMPED } from "#video/core/anim.ts";
import { useAuthoredFrame, useRealFrames } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { type CodeToken, type CodeTokenKind, tokenizeCode } from "#video/lib/code/code-syntax.tsx";

const MIN_FONT_SIZE = 9;

type TokenColors = Record<"code" | "comment" | "string" | "keyword" | "number", string>;

function colorOf(colors: TokenColors, kind: CodeTokenKind): string {
  switch (kind) {
    case "comment":
    case "string":
    case "keyword":
    case "number":
      return colors[kind];
    default:
      return colors.code;
  }
}

export function GlassCodeBlock({
  code,
  title,
  width,
  height,
  fontSize,
  glassColor,
  staggerFrames,
  showTrafficLights,
  aura,
  speed,
  frost,
  cornerRadius,
  lineHeight,
  font,
  codeColor,
  commentColor,
  stringColor,
  keywordColor,
  numberColor,
  titleColor,
  lineNumberColor,
  auraColors,
}: ElementSettings["glass-code-block"]) {
  const colors: TokenColors = {
    code: codeColor,
    comment: commentColor,
    string: stringColor,
    keyword: keywordColor,
    number: numberColor,
  };
  const realFrames = useRealFrames();
  const lines = code.split("\n");
  const tokenized = tokenizeCode(lines);
  const digits = String(lines.length).length;
  const longest = Math.max(1, ...lines.map((line) => line.length));
  const bodyHeight = height - 2 - 40 - 40;
  const fitted = Math.max(
    MIN_FONT_SIZE,
    Math.min(
      fontSize,
      (width - 2 - 48) / ((longest + digits + 1) * 0.62),
      (bodyHeight / lines.length - 4) / lineHeight,
    ),
  );
  const frame = useAuthoredFrame();
  const shown =
    staggerFrames > 0
      ? Math.min(lines.length, Math.floor((frame * speed) / staggerFrames) + 1)
      : lines.length;
  const scroll = Math.max(0, shown * (fitted * lineHeight + 4) - bodyHeight);

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      {aura && <BackdropAura colors={auraColors} />}

      <div
        style={{
          position: "relative",
          padding: 1,
          borderRadius: cornerRadius,
          background:
            "linear-gradient(180deg, rgba(255,255,255,0.18) 0%, rgba(255,255,255,0) 100%)",
          width,
          height,
          boxShadow: "0 50px 120px rgba(0,0,0,0.55)",
        }}
      >
        <div
          style={{
            width: "100%",
            height: "100%",
            borderRadius: Math.max(0, cornerRadius - 1),
            background: glassColor,
            backdropFilter: `blur(${frost}px)`,
            WebkitBackdropFilter: `blur(${frost}px)`,
            boxShadow: "inset 0 1px 0 rgba(255,255,255,0.06)",
            display: "flex",
            flexDirection: "column",
            overflow: "hidden",
            fontFamily: fontOr(font, "code"),
          }}
        >
          <div
            style={{
              height: 40,
              display: "flex",
              alignItems: "center",
              gap: 8,
              padding: "0 16px",
              borderBottom: "1px solid rgba(255,255,255,0.06)",
            }}
          >
            {showTrafficLights && (
              <>
                <Light color="#ff5f57" />
                <Light color="#febc2e" />
                <Light color="#28c840" />
              </>
            )}
            <div
              style={{
                flex: 1,
                textAlign: "center",
                color: titleColor,
                fontSize: 12,
                letterSpacing: "0.02em",
              }}
            >
              {title}
            </div>
          </div>

          <div
            style={{
              flex: 1,
              padding: "20px 24px",
              overflow: "hidden",
              clipPath: "inset(20px 0)",
              display: "flex",
              flexDirection: "column",
              gap: 4,
              fontSize: fitted,
              lineHeight,
            }}
          >
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                gap: 4,
                translate: `0 ${-scroll}px`,
              }}
            >
              {tokenized.map((tokens, i) => (
                <Sequence
                  key={i}
                  from={Math.round(realFrames((i * staggerFrames) / speed))}
                  layout="none"
                >
                  <CodeLine
                    tokens={tokens}
                    index={i}
                    digits={digits}
                    fontSize={fitted}
                    colors={colors}
                    gutter={lineNumberColor}
                  />
                </Sequence>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function Light({ color }: { color: string }) {
  return (
    <div
      style={{
        width: 12,
        height: 12,
        borderRadius: "50%",
        background: color,
        opacity: 0.6,
      }}
    />
  );
}

function CodeLine({
  tokens,
  index,
  digits,
  fontSize,
  colors,
  gutter,
}: {
  tokens: CodeToken[];
  index: number;
  digits: number;
  fontSize: number;
  colors: TokenColors;
  gutter: string;
}) {
  const frame = useAuthoredFrame();
  const opacity = interpolate(frame, [0, 8], [0, 1], CLAMPED);
  const ty = interpolate(frame, [0, 8], [4, 0], CLAMPED);

  if (tokens.length === 0) {
    return <div style={{ height: fontSize * 0.8, opacity }} />;
  }
  return (
    <div
      style={{
        opacity,
        translate: `0 ${ty}px`,
        whiteSpace: "pre",
        display: "flex",
        gap: 0,
      }}
    >
      <span style={{ width: `${digits + 1}ch`, flexShrink: 0, color: gutter, userSelect: "none" }}>
        {String(index + 1).padStart(digits, " ")}
      </span>
      <span>
        {tokens.map((t, i) => (
          <span key={i} style={{ color: colorOf(colors, t.kind) }}>
            {t.text}
          </span>
        ))}
      </span>
    </div>
  );
}

function BackdropAura({ colors }: { colors: string[] }) {
  const frame = useAuthoredFrame();
  const t = frame / 60;
  const x = 50 + Math.sin(t) * 20;
  const y = 50 + Math.cos(t * 0.7) * 15;
  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        pointerEvents: "none",
        background: `radial-gradient(circle at ${x}% ${y}%, ${colors[0]}, transparent 50%), radial-gradient(circle at ${100 - x}% ${100 - y}%, ${colors[1]}, transparent 55%)`,
      }}
    />
  );
}
