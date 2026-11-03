import type { TerminalFrame, TerminalFrames } from "@opsis/video/terminal/frames";
import {
  frameText,
  type Measure,
  measurer,
  screenSize,
  TERMINAL_FONT_STACK,
  TerminalView,
  type ViewLook,
} from "@opsis/video/terminal/view";
import { useEffect, useState, type PointerEvent, type WheelEvent } from "react";
import { useTerminalFonts } from "@/fonts/terminal";

export type CellAt = { col: number; row: number };

function useMeasure(text: string, fontSize: number): Measure | undefined {
  const fonts = useTerminalFonts();
  const [measure, setMeasure] = useState<{ size: number; measure: Measure }>();
  useEffect(() => {
    if (!fonts || !text) return;
    let alive = true;
    const loads = ["400", "700"].map((weight) =>
      document.fonts.load(`${weight} ${fontSize}px ${TERMINAL_FONT_STACK}`, text),
    );
    void Promise.allSettled(loads).then(() => {
      if (alive) setMeasure({ size: fontSize, measure: measurer(fontSize) });
    });
    return () => {
      alive = false;
    };
  }, [fonts, text, fontSize]);
  return measure?.size === fontSize ? measure.measure : undefined;
}

export function cellAt(
  e: { clientX: number; clientY: number; currentTarget: Element },
  look: ViewLook,
  frame: TerminalFrame,
): CellAt {
  const size = screenSize(look, frame.cols, frame.rows);
  const box = e.currentTarget.getBoundingClientRect();
  const scale = Math.min(box.width / size.width, box.height / size.height) || 1;
  const left = box.left + (box.width - size.width * scale) / 2;
  const top = box.top + (box.height - size.height * scale) / 2;
  const x = (e.clientX - left) / scale - look.padding;
  const y = (e.clientY - top) / scale - look.padding;
  const cellWidth = (size.width - 2 * look.padding) / frame.cols;
  const cellHeight = (size.height - 2 * look.padding) / frame.rows;
  return {
    col: Math.min(frame.cols - 1, Math.max(0, Math.floor(x / cellWidth))),
    row: Math.min(frame.rows - 1, Math.max(0, Math.floor(y / cellHeight))),
  };
}

export function TerminalPicture({
  frames,
  frame,
  look,
  cursorOn,
  onPointer,
  onWheel,
}: {
  frames: TerminalFrames;
  frame: TerminalFrame;
  look: ViewLook;
  cursorOn: boolean;
  onPointer?: (e: PointerEvent<SVGSVGElement>) => void;
  onWheel?: (e: WheelEvent<SVGSVGElement>) => void;
}) {
  const measure = useMeasure(frameText(frames, frame), look.fontSize);
  const size = screenSize(look, frame.cols, frame.rows);
  return (
    <svg
      className="absolute inset-0 size-full"
      viewBox={`0 0 ${size.width} ${size.height}`}
      preserveAspectRatio="xMidYMid meet"
      role="img"
      aria-label="Terminal"
      onPointerDown={onPointer}
      onPointerUp={onPointer}
      onPointerMove={onPointer}
      onWheel={onWheel}
      onContextMenu={(e) => e.preventDefault()}
    >
      <foreignObject x={0} y={0} width={size.width} height={size.height}>
        <TerminalView
          frames={frames}
          frame={frame}
          look={look}
          cursorOn={cursorOn}
          measure={measure}
        />
      </foreignObject>
    </svg>
  );
}
