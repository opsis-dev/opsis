import { interpolate, Sequence } from "remotion";
import { fontOr } from "#video/common/fonts.ts";
import { CLAMPED } from "#video/core/anim.ts";
import { useAuthoredConfig, useAuthoredFrame, useRealFrames } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { parseColor } from "#video/lib/ui-kit/color.ts";

export type TerminalLineType = "command" | "log" | "success" | "error";

export interface TerminalLine {
  text: string;
  type: TerminalLineType;
  delay?: number;
  pause?: number;
}

const TYPE_COLORS: Record<TerminalLineType, string> = {
  command: "#fafafa",
  log: "#a1a1aa",
  success: "#22c55e",
  error: "#ef4444",
};

const LIGHT_TYPE_COLORS: Record<TerminalLineType, string> = {
  command: "#18181b",
  log: "#52525b",
  success: "#15803d",
  error: "#b91c1c",
};

const CHROME_HEIGHT = 40;
const BODY_PADDING = 20;

function isLight(color: string): boolean {
  const { r, g, b, alpha = 1 } = parseColor(color);
  return alpha > 0.5 && 0.2126 * r + 0.7152 * g + 0.0722 * b > 0.5;
}

function autoPause(line: TerminalLine, ellipsis: number): number {
  if (line.pause !== undefined) return line.pause;
  if (line.text.trimEnd().endsWith("...")) return ellipsis;
  return 0;
}

export function TerminalSimulator({
  lines,
  prompt,
  title,
  background,
  chromeColor,
  fontSize,
  charsPerFrame,
  chunkSize,
  speed,
  width: windowWidth,
  height: windowHeight,
  ellipsisPause,
  startAt,
  windowDots,
  cornerRadius,
  lineHeight: lineSpacing,
  font,
  commandColor,
  logColor,
  successColor,
  errorColor,
  promptColor,
  cursorColor,
  titleColor,
}: ElementSettings["terminal-simulator"]) {
  const frame = useAuthoredFrame() * speed;
  const { fps } = useAuthoredConfig();
  const realFrames = useRealFrames();

  const lineHeight = Math.round(fontSize * lineSpacing);
  const visibleLines = Math.max(
    1,
    Math.floor((windowHeight - CHROME_HEIGHT - BODY_PADDING * 2) / lineHeight),
  );
  const auto = isLight(background) ? LIGHT_TYPE_COLORS : TYPE_COLORS;
  const typeColors: Record<TerminalLineType, string> = {
    command: commandColor || auto.command,
    log: logColor || auto.log,
    success: successColor || auto.success,
    error: errorColor || auto.error,
  };

  const starts: number[] = [];
  let acc = startAt;
  for (let i = 0; i < lines.length; i++) {
    const delay = lines[i].delay ?? 8;
    acc += delay;
    starts.push(acc);
    const typingFrames = Math.ceil(lines[i].text.length / (chunkSize * charsPerFrame));
    acc += typingFrames + autoPause(lines[i], ellipsisPause);
  }

  let translateY = 0;
  for (let i = visibleLines; i < lines.length; i++) {
    if (frame >= starts[i]) {
      translateY -= lineHeight;
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
      }}
    >
      <div
        style={{
          width: windowWidth,
          height: windowHeight,
          background,
          borderRadius: cornerRadius,
          overflow: "hidden",
          boxShadow: "0 30px 80px rgba(0,0,0,0.6), 0 0 0 1px rgba(255,255,255,0.06)",
          display: "flex",
          flexDirection: "column",
          fontFamily: fontOr(font, "code"),
        }}
      >
        <div
          style={{
            height: 40,
            background: chromeColor,
            display: "flex",
            alignItems: "center",
            padding: "0 16px",
            gap: 8,
            borderBottom: "1px solid rgba(255,255,255,0.06)",
          }}
        >
          {windowDots ? (
            <>
              <Light color="#ff5f57" />
              <Light color="#febc2e" />
              <Light color="#28c840" />
            </>
          ) : null}
          <div
            style={{
              flex: 1,
              textAlign: "center",
              color: titleColor,
              fontSize: 13,
            }}
          >
            {title}
          </div>
        </div>

        <div
          style={{
            flex: 1,
            padding: BODY_PADDING,
            overflow: "hidden",
            clipPath: `inset(${BODY_PADDING}px 0)`,
            position: "relative",
          }}
        >
          <div
            style={{
              position: "absolute",
              left: 20,
              right: 20,
              top: 20,
              translate: `0 ${translateY}px`,
            }}
          >
            {lines.map((line, index) => (
              <Sequence
                key={index}
                from={Math.round(realFrames(starts[index] / speed))}
                layout="none"
              >
                <TerminalLineRow
                  line={line}
                  colors={typeColors}
                  prompt={prompt}
                  promptColor={promptColor || typeColors.success}
                  cursorColor={cursorColor}
                  fontSize={fontSize}
                  lineHeight={lineHeight}
                  charsPerFrame={charsPerFrame}
                  chunkSize={chunkSize}
                  fps={fps}
                  speed={speed}
                />
              </Sequence>
            ))}
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
        opacity: 0.85,
      }}
    />
  );
}

function TerminalLineRow({
  line,
  colors,
  prompt,
  promptColor,
  cursorColor,
  fontSize,
  lineHeight,
  charsPerFrame,
  chunkSize,
  fps,
  speed,
}: {
  line: TerminalLine;
  colors: Record<TerminalLineType, string>;
  prompt: string;
  promptColor: string;
  cursorColor: string;
  fontSize: number;
  lineHeight: number;
  charsPerFrame: number;
  chunkSize: number;
  fps: number;
  speed: number;
}) {
  const localFrame = useAuthoredFrame() * speed;
  const totalChars = line.text.length;

  const linearRevealed = Math.floor(
    interpolate(localFrame, [0, totalChars / charsPerFrame], [0, totalChars], CLAMPED),
  );
  const revealed = Math.min(totalChars, Math.ceil(linearRevealed / chunkSize) * chunkSize);
  const visible = line.text.substring(0, revealed);
  const typingDone = revealed >= totalChars;
  const cursorVisible = Math.floor((localFrame / fps) * 2) % 2 === 0;

  return (
    <div
      style={{
        height: lineHeight,
        fontSize,
        color: colors[line.type],
        display: "flex",
        alignItems: "center",
        whiteSpace: "pre",
      }}
    >
      {line.type === "command" && (
        <span style={{ color: promptColor, marginRight: 8 }}>{prompt}</span>
      )}
      <span>{visible}</span>
      {!typingDone && cursorVisible && (
        <span
          style={{
            display: "inline-block",
            width: fontSize * 0.55,
            height: fontSize,
            background: cursorColor || colors[line.type],
            marginLeft: 2,
          }}
        />
      )}
    </div>
  );
}
