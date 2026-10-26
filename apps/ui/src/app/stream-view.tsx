import { ArrowLeft, ArrowRight, RotateCw } from "lucide-react";
import { useEffect, useRef, useState, type KeyboardEvent, type PointerEvent } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type { MouseButton, TabInfo, ViewEvent, ViewInput } from "@/generated/bindings";
import { cn } from "@/lib/utils";
import type { StreamFrame } from "./opsis";

const BUTTONS: MouseButton[] = ["left", "middle", "right", "back", "forward"];

type Modifiable = { altKey: boolean; ctrlKey: boolean; metaKey: boolean; shiftKey: boolean };

const modifiers = (e: Modifiable): number =>
  (e.altKey ? 1 : 0) | (e.ctrlKey ? 2 : 0) | (e.metaKey ? 4 : 0) | (e.shiftKey ? 8 : 0);

const buttonsOf = (buttons: number): number =>
  (buttons & 1) | (buttons & 2) | (buttons & 4) | (buttons & 8) | (buttons & 16);

type Dialog = Extract<ViewEvent, { kind: "dialog" }>;

type PageSize = { width: number; height: number };

function toPage(e: { clientX: number; clientY: number }, element: HTMLElement, page: PageSize) {
  const box = element.getBoundingClientRect();
  const scale = Math.min(box.width / page.width, box.height / page.height) || 1;
  const left = box.left + (box.width - page.width * scale) / 2;
  const top = box.top + (box.height - page.height * scale) / 2;
  return { x: (e.clientX - left) / scale, y: (e.clientY - top) / scale };
}

async function draw(canvas: HTMLCanvasElement, frame: StreamFrame): Promise<void> {
  const bitmap = await createImageBitmap(new Blob([frame.jpeg], { type: "image/jpeg" }));
  if (canvas.width !== bitmap.width) canvas.width = bitmap.width;
  if (canvas.height !== bitmap.height) canvas.height = bitmap.height;
  canvas.getContext("2d")?.drawImage(bitmap, 0, 0);
  bitmap.close();
}

function useViewEvents(id: string) {
  const [tabs, setTabs] = useState<{ tabs: TabInfo[]; active: number }>({ tabs: [], active: 0 });
  const [dialog, setDialog] = useState<Dialog | null>(null);
  useEffect(
    () =>
      window.opsis.onViewEvent(id, (event) => {
        if (event.kind === "tabs") setTabs({ tabs: event.tabs, active: event.active });
        else if (event.kind === "dialog") setDialog(event);
        else void window.opsis.viewChooseFiles(id);
      }),
    [id],
  );
  return { tabs, dialog, closeDialog: () => setDialog(null) };
}

function AddressBar({ id, url }: { id: string; url: string }) {
  const [typed, setTyped] = useState<string | null>(null);
  return (
    <form
      className="flex min-w-0 flex-1 items-center gap-1"
      onSubmit={(e) => {
        e.preventDefault();
        if (typed) void window.opsis.viewNavigate(id, typed);
        setTyped(null);
      }}
    >
      <Button
        variant="ghost"
        size="icon-sm"
        aria-label="Back"
        onClick={() => void window.opsis.viewGo(id, -1)}
      >
        <ArrowLeft />
      </Button>
      <Button
        variant="ghost"
        size="icon-sm"
        aria-label="Forward"
        onClick={() => void window.opsis.viewGo(id, 1)}
      >
        <ArrowRight />
      </Button>
      <Button
        variant="ghost"
        size="icon-sm"
        aria-label="Reload"
        onClick={() => void window.opsis.viewReload(id)}
      >
        <RotateCw />
      </Button>
      <Input
        aria-label="Address"
        className="h-8 min-w-0 flex-1"
        value={typed ?? url}
        onChange={(e) => setTyped(e.target.value)}
      />
    </form>
  );
}

function DialogBar({ id, dialog, onDone }: { id: string; dialog: Dialog; onDone: () => void }) {
  const [prompt, setPrompt] = useState(dialog.defaultPrompt);
  const answer = (accept: boolean) => {
    void window.opsis.viewAnswerDialog(id, accept, prompt);
    onDone();
  };
  return (
    <div className="bg-muted flex items-center gap-2 border-b px-3 py-2 text-sm">
      <span className="min-w-0 flex-1 truncate">{dialog.message}</span>
      {dialog.dialog === "prompt" ? (
        <Input className="h-8 w-48" value={prompt} onChange={(e) => setPrompt(e.target.value)} />
      ) : null}
      {dialog.dialog === "alert" ? null : (
        <Button variant="ghost" size="sm" onClick={() => answer(false)}>
          Cancel
        </Button>
      )}
      <Button size="sm" onClick={() => answer(true)}>
        OK
      </Button>
    </div>
  );
}

function pointerInput(
  kind: "mouseDown" | "mouseUp" | "mouseMove",
  e: PointerEvent<HTMLCanvasElement>,
  page: PageSize,
): ViewInput {
  const { x, y } = toPage(e, e.currentTarget, page);
  const shared = { x, y, buttons: buttonsOf(e.buttons), modifiers: modifiers(e) };
  if (kind === "mouseMove") return { kind, ...shared };
  return { kind, ...shared, button: BUTTONS[e.button] ?? "none", clickCount: e.detail || 1 };
}

function keyInput(kind: "keyDown" | "keyUp", e: KeyboardEvent): ViewInput {
  const shared = {
    key: e.key,
    code: e.code,
    keyCode: e.keyCode,
    location: e.location,
    modifiers: modifiers(e),
  };
  if (kind === "keyUp") return { kind, ...shared };
  const text =
    e.key.length === 1 && !e.ctrlKey && !e.metaKey ? e.key : e.key === "Enter" ? "\r" : "";
  return { kind, ...shared, text, repeat: e.repeat };
}

export function StreamView({ id }: { id: string }) {
  const [box, setBox] = useState<HTMLDivElement | null>(null);
  const canvas = useRef<HTMLCanvasElement | null>(null);
  const page = useRef<PageSize>({ width: 1, height: 1 });
  const { tabs, dialog, closeDialog } = useViewEvents(id);
  useEffect(() => {
    void window.opsis.attachView(id, (frame) => {
      page.current = { width: frame.header.width, height: frame.header.height };
      if (canvas.current) void draw(canvas.current, frame);
    });
  }, [id]);
  const send = (input: ViewInput) => void window.opsis.viewInput(id, input);
  const active = tabs.tabs[tabs.active];
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex items-center gap-2 border-b px-2 py-1.5">
        <AddressBar key={active?.url} id={id} url={active?.url ?? ""} />
      </div>
      {tabs.tabs.length > 1 ? (
        <div className="flex gap-1 border-b px-2 py-1">
          {tabs.tabs.map((tab, i) => (
            <Button
              key={tab.id}
              size="sm"
              variant={i === tabs.active ? "secondary" : "ghost"}
              className="max-w-48 truncate"
              onClick={() => void window.opsis.viewSwitchTab(id, tab.id)}
            >
              {tab.url}
            </Button>
          ))}
        </div>
      ) : null}
      {dialog ? <DialogBar id={id} dialog={dialog} onDone={closeDialog} /> : null}
      <div
        ref={setBox}
        tabIndex={0}
        className={cn("bg-muted relative min-h-0 flex-1 overflow-hidden outline-none")}
        onKeyDown={(e) => {
          e.preventDefault();
          send(keyInput("keyDown", e));
        }}
        onKeyUp={(e) => {
          e.preventDefault();
          send(keyInput("keyUp", e));
        }}
        onPaste={(e) => {
          e.preventDefault();
          send({ kind: "text", text: e.clipboardData.getData("text") });
        }}
      >
        <canvas
          ref={canvas}
          aria-label="Web page"
          className="absolute inset-0 size-full object-contain"
          onContextMenu={(e) => e.preventDefault()}
          onPointerDown={(e) => {
            e.currentTarget.setPointerCapture(e.pointerId);
            box?.focus();
            send(pointerInput("mouseDown", e, page.current));
          }}
          onPointerUp={(e) => send(pointerInput("mouseUp", e, page.current))}
          onPointerMove={(e) => send(pointerInput("mouseMove", e, page.current))}
          onWheel={(e) =>
            send({
              kind: "wheel",
              ...toPage(e, e.currentTarget, page.current),
              deltaX: e.deltaX,
              deltaY: e.deltaY,
              modifiers: modifiers(e),
            })
          }
        />
      </div>
    </div>
  );
}
