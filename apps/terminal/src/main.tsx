import { registerFaces } from "@opsis/video/common/font-faces";
import type { FontFamilyDef, TerminalLook } from "@opsis/video/generated/spec";
import { decodeFrames } from "@opsis/video/terminal/frames";
import {
  frameText,
  measurer,
  screenSize,
  TERMINAL_FONT_STACK,
  TerminalView,
} from "@opsis/video/terminal/view";
import { flushSync } from "react-dom";
import { createRoot } from "react-dom/client";

const APP_SCHEME = "app:";
const WEIGHTS = ["400", "700"];

type Painted = { width: number; height: number };

type TerminalPage = {
  fonts: (defs: FontFamilyDef[]) => Promise<boolean>;
  paint: (frames: string, look: TerminalLook) => Promise<Painted>;
};

const container = document.getElementById("terminal");
if (!container) throw new Error("The terminal page has no place for the terminal.");
const root = createRoot(container);

function resolve(ref: string): string {
  if (!ref.startsWith(APP_SCHEME)) return ref;
  const path = ref.slice(APP_SCHEME.length).replace(/^\/+/, "");
  return new URL(`/${path.split("/").map(encodeURIComponent).join("/")}`, location.origin).href;
}

function bytesOf(base64: string): Uint8Array<ArrayBuffer> {
  const text = atob(base64);
  const bytes = new Uint8Array(new ArrayBuffer(text.length));
  for (let i = 0; i < text.length; i++) bytes[i] = text.charCodeAt(i);
  return bytes;
}

function painted(): Promise<void> {
  return new Promise((done) => requestAnimationFrame(() => requestAnimationFrame(() => done())));
}

const page: TerminalPage = {
  async fonts(defs) {
    await Promise.all(defs.map((def) => registerFaces(def.family, def, resolve)));
    return true;
  },
  async paint(base64, look) {
    const frames = decodeFrames(bytesOf(base64));
    const frame = frames.frames[0];
    if (!frame) throw new Error("The terminal frame is empty.");
    const text = frameText(frames, frame);
    await Promise.allSettled(
      WEIGHTS.map((weight) =>
        document.fonts.load(`${weight} ${look.fontSize}px ${TERMINAL_FONT_STACK}`, text),
      ),
    );
    flushSync(() =>
      root.render(
        <TerminalView
          frames={frames}
          frame={frame}
          look={look}
          cursorOn
          measure={measurer(look.fontSize)}
        />,
      ),
    );
    await painted();
    return screenSize(look, frame.cols, frame.rows);
  },
};

Object.assign(window, { opsisTerminal: page });
