import { decodeFrames, type TerminalFrames } from "@opsis/video/terminal/frames";
import type { ViewLook } from "@opsis/video/terminal/view";
import { useEffect, useRef, useState, type KeyboardEvent, type PointerEvent } from "react";
import { Button } from "@/components/ui/button";
import { commands, type PointerButton, type TerminalInput } from "@/generated/bindings";
import { packedChannel } from "./tauri";
import { cellAt, TerminalPicture } from "./terminal-picture";

const LOOK: ViewLook = {
  fontSize: 18,
  lineHeight: 1.2,
  letterSpacing: 0,
  padding: 16,
  cursor: "program",
};

type TerminalFrameHeader = { skipped: number };

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

export function TerminalWindow({ id }: { id: string }) {
  const [frames, setFrames] = useState<TerminalFrames | null>(null);
  const [skipped, setSkipped] = useState(0);
  const box = useRef<HTMLDivElement | null>(null);
  useEffect(() => {
    void commands.attachTerminal(
      id,
      packedChannel<TerminalFrameHeader>((header, body) => {
        setFrames(decodeFrames(body));
        setSkipped(header.skipped);
      }),
    );
    box.current?.focus();
  }, [id]);
  function send(input: TerminalInput | null) {
    if (input) void commands.terminalInput(id, input);
  }
  const frame = frames?.frames[0];
  function pointer(e: PointerEvent<SVGSVGElement>) {
    if (!frame) return;
    if (e.type === "pointerdown") {
      e.currentTarget.setPointerCapture(e.pointerId);
      box.current?.focus();
    }
    const action = pointerAction(e.type);
    if (action === "move" && e.buttons === 0) return;
    send({ type: "pointer", action, button: BUTTONS[e.button] ?? null, ...cellAt(e, LOOK, frame) });
  }
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
      >
        {frames && frame ? (
          <TerminalPicture
            frames={frames}
            frame={frame}
            look={LOOK}
            cursorOn
            onPointer={pointer}
            onWheel={(e) =>
              send({
                type: "wheel",
                ...cellAt(e, LOOK, frame),
                notches:
                  Math.sign(e.deltaY) * Math.max(1, Math.round(Math.abs(e.deltaY) / WHEEL_STEP)),
              })
            }
          />
        ) : null}
      </div>
    </div>
  );
}
