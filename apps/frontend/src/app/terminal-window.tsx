import { clamp } from "@opsis/video/common/math";
import { useEffect, useRef, useState, type KeyboardEvent, type PointerEvent } from "react";
import { Button } from "@/components/ui/button";
import { commands, type PointerButton, type TerminalInput } from "@/generated/bindings";
import { packedChannel } from "./tauri";

type TerminalFrameHeader = {
  skipped: number;
  width: number;
  height: number;
  cols: number;
  rows: number;
  cellWidth: number;
  cellHeight: number;
  padding: number;
};

type Shown = { header: TerminalFrameHeader; url: string };

type CellAt = { col: number; row: number };

const MODIFIER_KEYS = new Set(["Shift", "Control", "Alt", "Meta", "AltGraph", "CapsLock"]);
const BUTTONS: (PointerButton | null)[] = ["left", "middle", "right"];
const WHEEL_STEP = 100;

function keyInput(e: KeyboardEvent): TerminalInput | null {
  if (MODIFIER_KEYS.has(e.key) || e.metaKey) return null;
  if (e.key.length === 1 && !e.ctrlKey && !e.altKey) return { type: "text", text: e.key };
  return { type: "key", key: e.key, ctrl: e.ctrlKey, alt: e.altKey, shift: e.shiftKey };
}

function pointerAction(type: string): "press" | "release" | "move" {
  if (type === "pointerdown") return "press";
  if (type === "pointerup") return "release";
  return "move";
}

function cellAt(
  e: { clientX: number; clientY: number; currentTarget: Element },
  header: TerminalFrameHeader,
): CellAt {
  const box = e.currentTarget.getBoundingClientRect();
  const scale = Math.min(box.width / header.width, box.height / header.height) || 1;
  const left = box.left + (box.width - header.width * scale) / 2;
  const top = box.top + (box.height - header.height * scale) / 2;
  const x = (e.clientX - left) / scale - header.padding;
  const y = (e.clientY - top) / scale - header.padding;
  return {
    col: clamp(Math.floor(x / header.cellWidth), 0, header.cols - 1),
    row: clamp(Math.floor(y / header.cellHeight), 0, header.rows - 1),
  };
}

function useShownFrame(id: string): Shown | null {
  const [shown, setShown] = useState<Shown | null>(null);
  useEffect(() => {
    let last: string | null = null;
    void commands.attachTerminal(
      id,
      packedChannel<TerminalFrameHeader>((header, body) => {
        const url = URL.createObjectURL(new Blob([body], { type: "image/jpeg" }));
        if (last) URL.revokeObjectURL(last);
        last = url;
        setShown({ header, url });
      }),
    );
    return () => {
      if (last) URL.revokeObjectURL(last);
    };
  }, [id]);
  return shown;
}

export function TerminalWindow({ id }: { id: string }) {
  const shown = useShownFrame(id);
  const box = useRef<HTMLDivElement | null>(null);
  useEffect(() => box.current?.focus(), []);
  function send(input: TerminalInput | null) {
    if (input) void commands.terminalInput(id, input);
  }
  const header = shown?.header;
  function pointer(e: PointerEvent<HTMLDivElement>) {
    if (!header) return;
    if (e.type === "pointerdown") {
      e.currentTarget.setPointerCapture(e.pointerId);
      box.current?.focus();
    }
    const action = pointerAction(e.type);
    if (action === "move" && e.buttons === 0) return;
    send({ type: "pointer", action, button: BUTTONS[e.button] ?? null, ...cellAt(e, header) });
  }
  const skipped = header?.skipped ?? 0;
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex items-center gap-2 border-b px-3 py-1.5 text-sm">
        <span className="text-muted-foreground min-w-0 flex-1 truncate">
          {skipped > 0
            ? `Recording what you type. ${skipped} ${skipped === 1 ? "key" : "keys"} can't be saved in a tape file.`
            : "Recording what you type. Pauses become waits in the tape."}
        </span>
        <Button variant="ghost" size="sm" onClick={() => void commands.finishTerminal(id, false)}>
          Cancel
        </Button>
        <Button size="sm" onClick={() => void commands.finishTerminal(id, true)}>
          Save tape
        </Button>
      </div>
      <div
        ref={box}
        tabIndex={0}
        className="bg-background relative min-h-0 flex-1 overflow-hidden outline-none"
        onKeyDown={(e) => {
          if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "v") return;
          e.preventDefault();
          send(keyInput(e));
        }}
        onPaste={(e) => {
          e.preventDefault();
          send({ type: "paste", text: e.clipboardData.getData("text") });
        }}
        onFocus={() => send({ type: "focus", gained: true })}
        onBlur={() => send({ type: "focus", gained: false })}
        onPointerDown={pointer}
        onPointerMove={pointer}
        onPointerUp={pointer}
        onWheel={(e) => {
          if (!header) return;
          send({
            type: "wheel",
            ...cellAt(e, header),
            notches: Math.sign(e.deltaY) * Math.max(1, Math.round(Math.abs(e.deltaY) / WHEEL_STEP)),
          });
        }}
      >
        {shown ? (
          <img
            src={shown.url}
            alt="Terminal"
            draggable={false}
            className="pointer-events-none absolute inset-0 size-full object-contain"
          />
        ) : null}
      </div>
    </div>
  );
}
