import { useEffect, useRef, useState } from "react";
import { on } from "@/app/tauri";
import {
  commands,
  events,
  type PreviewTarget,
  type TimelineSpec,
  type ViewSize,
} from "@/generated/bindings";
import { attempt } from "@/lib/toast";
import { frameChannel, type StreamFrame } from "../app/tauri";
import { singleSelected, useUiStore } from "../store/ui";
import { useViewOptions } from "../store/view-options";
import { usePreviewPlayback } from "./preview-playback";

const FAST_WIDTH = 960;
const RESIZE_SETTLE_MS = 150;

function viewSize(box: DOMRect, spec: TimelineSpec, fast: boolean): ViewSize {
  const aspect = spec.width / spec.height;
  const width = fast ? FAST_WIDTH : Math.max(1, box.width * devicePixelRatio);
  const height = fast ? FAST_WIDTH / aspect : Math.max(1, box.height * devicePixelRatio);
  const fit = Math.min(width / aspect, height);
  return { width: Math.round(fit * aspect), height: Math.round(fit), dpr: 1 };
}

async function draw(canvas: HTMLCanvasElement, frame: StreamFrame): Promise<void> {
  const bitmap = await createImageBitmap(new Blob([frame.jpeg], { type: "image/jpeg" }));
  if (canvas.width !== bitmap.width) canvas.width = bitmap.width;
  if (canvas.height !== bitmap.height) canvas.height = bitmap.height;
  canvas.getContext("2d")?.drawImage(bitmap, 0, 0);
  bitmap.close();
}

function useOpenPreview(
  projectDir: string | null,
  canvas: HTMLCanvasElement | null,
  size: ViewSize | null,
): boolean {
  const [open, setOpen] = useState(false);
  const latest = useRef(size);
  useEffect(() => {
    latest.current = size;
  }, [size]);
  const sized = size !== null;
  useEffect(() => {
    const opening = latest.current;
    if (!projectDir || !canvas || !sized || !opening) return;
    let gone = false;
    void attempt(async () => {
      await commands.previewOpen(
        projectDir,
        opening,
        frameChannel((frame) => {
          if (!gone) void draw(canvas, frame);
        }),
      );
      if (!gone) setOpen(true);
    }, "Couldn't start the preview");
    return () => {
      gone = true;
      setOpen(false);
      void commands.previewClose();
    };
  }, [projectDir, canvas, sized]);
  return open;
}

function useStageSize(stage: HTMLElement | null, spec: TimelineSpec | null, fast: boolean) {
  const [size, setSize] = useState<ViewSize | null>(null);
  useEffect(() => {
    if (!stage || !spec) return;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const measure = () => setSize(viewSize(stage.getBoundingClientRect(), spec, fast));
    measure();
    const observer = new ResizeObserver(() => {
      clearTimeout(timer);
      timer = setTimeout(measure, RESIZE_SETTLE_MS);
    });
    observer.observe(stage);
    return () => {
      clearTimeout(timer);
      observer.disconnect();
    };
  }, [stage, spec, fast]);
  return size;
}

function useReported(): void {
  useEffect(
    () =>
      on(events.previewReported, ({ frame, playing }) =>
        usePreviewPlayback.setState({ frame, playing }),
      ),
    [],
  );
}

function useControls(open: boolean, durationInFrames: number): void {
  useEffect(() => {
    if (!open) return;
    usePreviewPlayback.setState({ open: true });
    const frame = usePreviewPlayback.getState().frame;
    if (frame > 0) void commands.previewSeek(Math.min(frame, durationInFrames - 1));
    return () => usePreviewPlayback.setState({ open: false, playing: false });
  }, [open, durationInFrames]);
}

function useSeekToSelection(open: boolean, preview: PreviewTarget | null): void {
  const selectedId = useUiStore((s) => (s.pinnedSource ? null : singleSelected(s.selectedIds)));
  useEffect(() => {
    if (!open || !selectedId || !preview) return;
    const span = preview.spans[selectedId];
    if (span) usePreviewPlayback.getState().seek(span.from);
  }, [open, selectedId, preview]);
}

export function useStreamedPreview({
  projectDir,
  preview,
  spec,
  stage,
  canvas,
}: {
  projectDir: string | null;
  preview: PreviewTarget | null;
  spec: TimelineSpec | null;
  stage: HTMLElement | null;
  canvas: HTMLCanvasElement | null;
}): void {
  const fast = useViewOptions((s) => s.previewQuality) === "fast";
  const size = useStageSize(stage, spec, fast);
  const open = useOpenPreview(projectDir, canvas, size);
  const durationInFrames = Math.max(1, Math.round(spec?.durationInFrames ?? 1));
  useEffect(() => {
    usePreviewPlayback.setState({ fps: spec?.fps ?? 30, durationInFrames });
  }, [spec?.fps, durationInFrames]);
  useEffect(() => {
    if (open && spec) void commands.previewLoad(spec);
  }, [open, spec]);
  useEffect(() => {
    if (open && size) void commands.previewResize(size);
  }, [open, size]);
  useReported();
  useControls(open, durationInFrames);
  useSeekToSelection(open, preview);
}
