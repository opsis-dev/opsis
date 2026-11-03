const MAGIC = "OPSTERM2";

export const RunFlag = {
  bold: 1,
  italic: 1 << 1,
  strikethrough: 1 << 2,
  overline: 1 << 3,
  blink: 1 << 4,
} as const;

export const Redaction = { none: 0, solid: 1, blur: 2, pixels: 3 } as const;

export const CursorShape = { bar: 0, block: 1, underline: 2, hollow: 3 } as const;

export type Run = {
  col: number;
  cells: number;
  text: string;
  fg: number;
  bg: number;
  underlineColour: number;
  flags: number;
  underline: number;
  redaction: number;
};

export type FrameCursor = {
  x: number;
  y: number;
  visible: boolean;
  shape: number;
  blink: boolean;
  colour: number;
};

export type TerminalFrame = {
  cols: number;
  rows: number;
  rowIndices: number[];
  cursor: FrameCursor;
  bg: number;
};

export type TerminalFrames = {
  cols: number;
  rows: number;
  fps: number;
  rowTable: Run[][];
  frames: TerminalFrame[];
};

class Reader {
  at = 0;
  constructor(private readonly view: DataView) {}

  u8(): number {
    const v = this.view.getUint8(this.at);
    this.at += 1;
    return v;
  }

  u16(): number {
    const v = this.view.getUint16(this.at, true);
    this.at += 2;
    return v;
  }

  u32(): number {
    const v = this.view.getUint32(this.at, true);
    this.at += 4;
    return v;
  }

  bytes(n: number): Uint8Array {
    const out = new Uint8Array(this.view.buffer, this.view.byteOffset + this.at, n);
    this.at += n;
    return out;
  }
}

export function decodeFrames(data: ArrayBuffer | Uint8Array): TerminalFrames {
  const bytes = data instanceof Uint8Array ? data : new Uint8Array(data);
  const r = new Reader(new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength));
  const decoder = new TextDecoder();
  if (decoder.decode(r.bytes(MAGIC.length)) !== MAGIC)
    throw new Error("This isn't a terminal frames file.");
  const cols = r.u16();
  const rows = r.u16();
  const fps = r.u32();
  const frameCount = r.u32();
  const rowCount = r.u32();
  const textBytes = r.bytes(r.u32());
  const rowTable: Run[][] = [];
  for (let i = 0; i < rowCount; i++) {
    const runs: Run[] = [];
    const count = r.u16();
    for (let j = 0; j < count; j++) {
      const col = r.u16();
      const cells = r.u16();
      const offset = r.u32();
      const length = r.u16();
      runs.push({
        col,
        cells,
        text: decoder.decode(textBytes.subarray(offset, offset + length)),
        fg: r.u32(),
        bg: r.u32(),
        underlineColour: r.u32(),
        flags: r.u16(),
        underline: r.u8(),
        redaction: r.u8(),
      });
    }
    rowTable.push(runs);
  }
  const frames: TerminalFrame[] = [];
  for (let i = 0; i < frameCount; i++) {
    const frameCols = r.u16();
    const frameRows = r.u16();
    const rowIndices: number[] = [];
    for (let y = 0; y < frameRows; y++) rowIndices.push(r.u32());
    const cursor: FrameCursor = {
      x: r.u16(),
      y: r.u16(),
      visible: r.u8() !== 0,
      shape: r.u8(),
      blink: r.u8() !== 0,
      colour: r.u32(),
    };
    frames.push({ cols: frameCols, rows: frameRows, rowIndices, cursor, bg: r.u32() });
  }
  if (r.at !== bytes.byteLength) throw new Error("The terminal frames file has bytes left over.");
  return { cols, rows, fps, rowTable, frames };
}

export function hexColour(colour: number): string {
  return `#${colour.toString(16).padStart(6, "0")}`;
}

export function frameAt(frames: TerminalFrames, timeS: number): TerminalFrame | undefined {
  const index = Math.min(frames.frames.length - 1, Math.max(0, Math.round(timeS * frames.fps)));
  return frames.frames[index];
}
