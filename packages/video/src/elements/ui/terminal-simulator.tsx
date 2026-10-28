import { interpolate, Sequence } from "remotion";
import { useAuthoredConfig, useAuthoredFrame, useRealFrames } from "#video/core/authored-clock.ts";
import { parseColor } from "#video/lib/neutral-ui/color.ts";

export type TerminalLineType = "command" | "log" | "success" | "error";

export interface TerminalLine {
  text: string;
  type: TerminalLineType;
  delay?: number;
  pause?: number;
}

export interface TerminalSimulatorProps {
  lines?: TerminalLine[];
  prompt?: string;
  title?: string;
  background?: string;
  chromeColor?: string;
  fontSize?: number;
  charsPerFrame?: number;
  chunkSize?: number;
  speed?: number;
  className?: string;
}

const DEFAULT_LINES: TerminalLine[] = [
  { text: "npm run build", type: "command", delay: 0 },
  { text: "Resolving dependencies...", type: "log", delay: 6 },
  { text: "> acme@1.0.0 build", type: "log", delay: 4 },
  { text: "> next build", type: "log", delay: 4 },
  { text: "Compiling...", type: "log", delay: 12 },
  { text: "Compiled successfully in 4.2s", type: "success", delay: 14 },
  { text: "Generating static pages (24/24)", type: "log", delay: 10 },
  { text: "Build completed without errors", type: "success", delay: 12 },
];

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

function autoPause(line: TerminalLine): number {
  if (line.pause !== undefined) return line.pause;
  if (line.text.trimEnd().endsWith("...")) return 18;
  return 0;
}

export function TerminalSimulator({
  lines = DEFAULT_LINES,
  prompt = "$",
  title = "~/projects/acme",
  background = "#0a0a0a",
  chromeColor = "#1a1a1a",
  fontSize = 18,
  charsPerFrame = 1,
  chunkSize = 1,
  speed = 1,
  className,
}: TerminalSimulatorProps) {
  const frame = useAuthoredFrame() * speed;
  const { fps } = useAuthoredConfig();
  const realFrames = useRealFrames();

  const lineHeight = Math.round(fontSize * 1.6);
  const windowWidth = 900;
  const windowHeight = 480;
  const visibleLines = Math.max(
    1,
    Math.floor((windowHeight - CHROME_HEIGHT - BODY_PADDING * 2) / lineHeight),
  );
  const typeColors = isLight(background) ? LIGHT_TYPE_COLORS : TYPE_COLORS;

  const starts: number[] = [];
  let acc = 10;
  for (let i = 0; i < lines.length; i++) {
    const delay = lines[i].delay ?? 8;
    acc += delay;
    starts.push(acc);
    const typingFrames = Math.ceil(lines[i].text.length / (chunkSize * charsPerFrame));
    acc += typingFrames + autoPause(lines[i]);
  }

  let translateY = 0;
  for (let i = visibleLines; i < lines.length; i++) {
    if (frame >= starts[i]) {
      translateY -= lineHeight;
    }
  }

  return (
    <div
      className={className}
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
          borderRadius: 12,
          overflow: "hidden",
          boxShadow: "0 30px 80px rgba(0,0,0,0.6), 0 0 0 1px rgba(255,255,255,0.06)",
          display: "flex",
          flexDirection: "column",
          fontFamily: "var(--font-geist-mono), ui-monospace, SFMono-Regular, monospace",
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
          <Light color="#ff5f57" />
          <Light color="#febc2e" />
          <Light color="#28c840" />
          <div
            style={{
              flex: 1,
              textAlign: "center",
              color: "#71717a",
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
    interpolate(localFrame, [0, totalChars / charsPerFrame], [0, totalChars], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
    }),
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
        <span style={{ color: colors.success, marginRight: 8 }}>{prompt}</span>
      )}
      <span>{visible}</span>
      {!typingDone && cursorVisible && (
        <span
          style={{
            display: "inline-block",
            width: fontSize * 0.55,
            height: fontSize,
            background: TYPE_COLORS[line.type],
            marginLeft: 2,
          }}
        />
      )}
    </div>
  );
}
