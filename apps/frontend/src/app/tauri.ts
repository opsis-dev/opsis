import { Channel } from "@tauri-apps/api/core";
import type { EventCallback, UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWebviewWindow, type WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { commands, events, type LiveEvent } from "@/generated/bindings";

type WindowEvent<T> = (target: WebviewWindow) => {
  listen: (callback: EventCallback<T>) => Promise<UnlistenFn>;
};

export function on<T>(event: WindowEvent<T>, listener: (payload: T) => void): () => void {
  let off: UnlistenFn | null = null;
  let gone = false;
  void event(getCurrentWebviewWindow())
    .listen((fired) => listener(fired.payload))
    .then((unlisten) => {
      if (gone) unlisten();
      else off = unlisten;
    });
  return () => {
    gone = true;
    off?.();
  };
}

export function windowLabel(): string {
  return getCurrentWebviewWindow().label;
}

type FrameHeader = {
  width: number;
  height: number;
  t: number;
  captureId?: string;
  nodeId?: string;
};

export type StreamFrame = { header: FrameHeader; jpeg: Uint8Array<ArrayBuffer> };

const decoder = new TextDecoder();

function readPacked<H>(buffer: ArrayBuffer): { header: H; body: Uint8Array<ArrayBuffer> } {
  const length = new DataView(buffer).getUint32(0, true);
  const header = JSON.parse(decoder.decode(new Uint8Array(buffer, 4, length))) as H;
  return { header, body: new Uint8Array(buffer.slice(4 + length)) };
}

function readFrame(buffer: ArrayBuffer): StreamFrame {
  const { header, body } = readPacked<FrameHeader>(buffer);
  return { header, jpeg: body };
}

export function packedChannel<H>(
  listener: (header: H, body: Uint8Array<ArrayBuffer>) => void,
): Channel<ArrayBuffer> {
  return new Channel<ArrayBuffer>((buffer) => {
    const { header, body } = readPacked<H>(buffer);
    listener(header, body);
  });
}

export function frameChannel(listener: (frame: StreamFrame) => void): Channel<ArrayBuffer> {
  return new Channel<ArrayBuffer>((buffer) => listener(readFrame(buffer)));
}

export type LiveMessage =
  | {
      type: "frame";
      captureId: string;
      nodeId: string;
      jpeg: Uint8Array<ArrayBuffer>;
      width: number;
      height: number;
      t: number;
    }
  | { type: "event"; captureId: string; nodeId: string; t: number; event: LiveEvent };

export function watchLive(listener: (message: LiveMessage) => void): () => void {
  const frames = frameChannel(({ header, jpeg }) => {
    const ids = { captureId: header.captureId ?? "", nodeId: header.nodeId ?? "", t: header.t };
    listener({ type: "frame", ...ids, jpeg, width: header.width, height: header.height });
  });
  const off = on(events.liveNoted, (note) => listener({ type: "event", ...note }));
  void commands.watchLive(frames).then((notes) => {
    for (const note of notes) listener({ type: "event", ...note });
  });
  return () => {
    off();
    void commands.unwatchLive(frames.id);
  };
}

const droppedPaths = new Map<string, string>();

export function rememberDrops(): void {
  void getCurrentWebview().onDragDropEvent((event) => {
    if (event.payload.type !== "drop") return;
    for (const path of event.payload.paths)
      droppedPaths.set(path.split(/[\\/]/).at(-1) ?? path, path);
  });
}

export function pathForFile(file: File): string {
  return droppedPaths.get(file.name) ?? "";
}
