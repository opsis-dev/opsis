import { useEffect, useEffectEvent } from "react";
import { shallow } from "zustand/vanilla/shallow";
import type { DockPanelName, MenuCommand } from "@/shared/ipc";
import type { LiveMessage } from "@/shared/live";
import type { RunEvent } from "@/shared/schema/events";
import type { Workflow } from "@/shared/schema/workflow";
import { useLiveStore } from "../store/live";
import { loadViewState } from "../editor/view-state";
import {
  forwardPlaybackTo,
  runPlaybackCommand,
  usePlayback,
  type PlaybackCommand,
} from "../store/playback";
import { useRunStore } from "../store/run";
import { useUiStore } from "../store/ui";
import { useViewOptions, type PreviewQuality, type PreviewShape } from "../store/view-options";
import { mirrorEditsTo, useWorkflowStore, type RemoteEdit } from "../store/workflow";
import { liveAutoOpen } from "../store/app-settings";
import { randomId } from "@/shared/ids";

const CHANNEL = "demoflow-sync";
const LIVE_CHANNEL = "demoflow-live";

const RUN_KEYS = [
  "runId",
  "running",
  "statuses",
  "progress",
  "cached",
  "errors",
  "traces",
  "logs",
  "results",
  "outputs",
  "lastError",
  "failedNodeIds",
  "skippedNodeIds",
] as const;

const SELECTION_KEYS = ["selectedIds", "activeIds", "pinnedSource"] as const;

const PLAYBACK_KEYS = [
  "frame",
  "playing",
  "rate",
  "fps",
  "durationInFrames",
  "inPoint",
  "outPoint",
] as const;

type RunSnapshot = Pick<ReturnType<typeof useRunStore.getState>, (typeof RUN_KEYS)[number]>;
type Selection = Pick<ReturnType<typeof useUiStore.getState>, (typeof SELECTION_KEYS)[number]>;
type PlaybackSnapshot = Pick<
  ReturnType<typeof usePlayback.getState>,
  (typeof PLAYBACK_KEYS)[number]
>;

type SyncMessage =
  | { type: "hello"; panel: DockPanelName }
  | {
      type: "state";
      projectDir: string;
      workflow: Workflow;
      ack: number;
      run: RunSnapshot;
      selection: Selection;
      quality: PreviewQuality;
      shape: PreviewShape;
      playback: PlaybackSnapshot;
      host: boolean;
    }
  | { type: "workflow"; workflow: Workflow; ack: number }
  | { type: "run-event"; event: RunEvent }
  | { type: "selection"; selection: Selection; from: string }
  | { type: "quality"; quality: PreviewQuality }
  | { type: "shape"; shape: PreviewShape }
  | { type: "edit"; edit: RemoteEdit; seq: number }
  | { type: "menu"; command: MenuCommand }
  | { type: "replay"; nodeId: string | null }
  | { type: "playback"; command: PlaybackCommand }
  | { type: "frame"; playback: PlaybackSnapshot; from: string }
  | { type: "inout"; inPoint: number | null; outPoint: number | null; from: string }
  | { type: "editor-closed" };

type Post = (message: SyncMessage) => void;
type Flag = { current: boolean };

const windowId = randomId();

const pick = <T, K extends keyof T>(source: T, keys: readonly K[]): Pick<T, K> =>
  Object.fromEntries(keys.map((k) => [k, source[k]])) as Pick<T, K>;

const runSnapshot = (): RunSnapshot => pick(useRunStore.getState(), RUN_KEYS);
const selection = (): Selection => pick(useUiStore.getState(), SELECTION_KEYS);
const playbackSnapshot = (): PlaybackSnapshot => pick(usePlayback.getState(), PLAYBACK_KEYS);

const isMessage = (data: unknown): data is { type: string } =>
  typeof data === "object" && data !== null && "type" in data && typeof data.type === "string";

const isSyncMessage = (data: unknown): data is SyncMessage => isMessage(data);

const isLiveMessage = (data: unknown): data is LiveMessage =>
  isMessage(data) && "captureId" in data && typeof data.captureId === "string";

function whileApplying(applying: Flag, fn: () => void): void {
  applying.current = true;
  try {
    fn();
  } finally {
    applying.current = false;
  }
}

function syncPlayback(post: Post, applying: Flag): () => void {
  let last = playbackSnapshot();
  return usePlayback.subscribe((s) => {
    const next = playbackSnapshot();
    if (applying.current || shallow(next, last)) {
      last = next;
      return;
    }
    const inOutChanged = next.inPoint !== last.inPoint || next.outPoint !== last.outPoint;
    last = next;
    if (inOutChanged)
      post({ type: "inout", inPoint: next.inPoint, outPoint: next.outPoint, from: windowId });
    if (s.hasPlayer) post({ type: "frame", playback: next, from: windowId });
  });
}

function watchSelection(post: Post, applying: Flag): () => void {
  let last = selection();
  return useUiStore.subscribe(() => {
    const next = selection();
    const changed = !shallow(next, last);
    last = next;
    if (changed && !applying.current) post({ type: "selection", selection: next, from: windowId });
  });
}

function watchViewOptions(post: Post, applying: Flag): () => void {
  return useViewOptions.subscribe((s, prev) => {
    if (applying.current) return;
    if (s.previewQuality !== prev.previewQuality)
      post({ type: "quality", quality: s.previewQuality });
    if (s.previewShape !== prev.previewShape) post({ type: "shape", shape: s.previewShape });
  });
}

function watchUi(post: Post, applying: Flag): () => void {
  const offs = [watchSelection(post, applying), watchViewOptions(post, applying)];
  return () => {
    for (const off of offs) off();
  };
}

function applyShared(m: SyncMessage, applying: Flag): boolean {
  switch (m.type) {
    case "frame":
      if (m.from !== windowId && !usePlayback.getState().hasPlayer)
        whileApplying(applying, () => usePlayback.setState(m.playback));
      return true;
    case "inout":
      if (m.from !== windowId)
        whileApplying(applying, () =>
          usePlayback.setState({ inPoint: m.inPoint, outPoint: m.outPoint }),
        );
      return true;
    case "playback":
      runPlaybackCommand(m.command);
      return true;
    case "selection":
      if (m.from !== windowId) whileApplying(applying, () => useUiStore.setState(m.selection));
      return true;
    case "quality":
      whileApplying(applying, () => useViewOptions.setState({ previewQuality: m.quality }));
      return true;
    case "shape":
      whileApplying(applying, () => useViewOptions.setState({ previewShape: m.shape }));
      return true;
    default:
      return false;
  }
}

function useLiveRelay(): void {
  useEffect(() => {
    const bc = new BroadcastChannel(LIVE_CHANNEL);
    const offLocal = window.demoflow.onLive((message) => bc.postMessage(message));
    bc.addEventListener("message", (e) => {
      if (isLiveMessage(e.data)) useLiveStore.getState().apply(e.data, liveAutoOpen());
    });
    return () => {
      offLocal();
      bc.close();
    };
  }, []);
}

function listen(bc: BroadcastChannel, handle: (message: SyncMessage) => void): void {
  bc.addEventListener("message", (e) => {
    if (isSyncMessage(e.data)) handle(e.data);
  });
}

function sendState(post: Post, ack: number): void {
  const { projectDir, workflow } = useWorkflowStore.getState();
  if (!projectDir || !workflow) return;
  const { previewQuality, previewShape } = useViewOptions.getState();
  post({
    type: "state",
    projectDir,
    workflow,
    ack,
    run: runSnapshot(),
    selection: selection(),
    quality: previewQuality,
    shape: previewShape,
    playback: playbackSnapshot(),
    host: usePlayback.getState().hasPlayer,
  });
}

function watchWorkflow(post: Post, ack: () => number): () => void {
  let lastWorkflow = useWorkflowStore.getState().workflow;
  return useWorkflowStore.subscribe((s) => {
    if (s.workflow === lastWorkflow || !s.workflow) return;
    lastWorkflow = s.workflow;
    post({ type: "workflow", workflow: s.workflow, ack: ack() });
  });
}

function applyRemoteEdit(edit: RemoteEdit): void {
  const store = useWorkflowStore.getState();
  if (edit.type === "replace") store.replace(edit.workflow);
  else if (edit.phase === "begin") store.beginGesture();
  else store.endGesture();
}

export function useEditorSync(): void {
  useLiveRelay();
  useEffect(() => {
    const bc = new BroadcastChannel(CHANNEL);
    const post: Post = (m) => bc.postMessage(m);
    const applying = { current: false };
    let ack = 0;
    forwardPlaybackTo((command) => post({ type: "playback", command }));
    const offs = [
      watchWorkflow(post, () => ack),
      watchUi(post, applying),
      syncPlayback(post, applying),
      window.demoflow.onRunEvent((event) => post({ type: "run-event", event })),
    ];
    listen(bc, (m) => {
      if (applyShared(m, applying)) return;
      if (m.type === "hello") sendState(post, ack);
      else if (m.type === "edit") {
        ack = Math.max(ack, m.seq);
        applyRemoteEdit(m.edit);
      } else if (m.type === "menu") window.demoflow.dispatchMenuCommand(m.command);
      else if (m.type === "replay") useUiStore.getState().setReplay(m.nodeId);
    });
    sendState(post, ack);
    return () => {
      for (const off of offs) off();
      forwardPlaybackTo(null);
      post({ type: "editor-closed" });
      bc.close();
    };
  }, []);
}

function applyState(m: Extract<SyncMessage, { type: "state" }>, applying: Flag): void {
  if (useWorkflowStore.getState().projectDir !== m.projectDir) loadViewState(m.projectDir);
  useWorkflowStore.setState({
    projectDir: m.projectDir,
    workflow: m.workflow,
    revision: 0,
    savedRevision: 0,
  });
  useRunStore.setState(m.run);
  whileApplying(applying, () => {
    useUiStore.setState(m.selection);
    useViewOptions.setState({ previewQuality: m.quality, previewShape: m.shape });
    if (!usePlayback.getState().hasPlayer) usePlayback.setState(m.playback);
  });
}

function applyEditorWorkflow(workflow: Workflow): void {
  const store = useWorkflowStore.getState();
  if (store.workflow === workflow) return;
  useWorkflowStore.setState({
    workflow,
    revision: store.revision + 1,
    savedRevision: store.revision + 1,
  });
}

export function usePanelSync(panel: DockPanelName, onClosed: () => void): void {
  useLiveRelay();
  const closed = useEffectEvent(onClosed);
  useEffect(() => {
    const bc = new BroadcastChannel(CHANNEL);
    const post: Post = (m) => bc.postMessage(m);
    const applying = { current: false };
    let seq = 0;
    mirrorEditsTo((edit) => post({ type: "edit", edit, seq: ++seq }));
    forwardPlaybackTo((command) => post({ type: "playback", command }));
    const offs = [watchUi(post, applying), syncPlayback(post, applying)];
    listen(bc, (m) => {
      if (applyShared(m, applying)) return;
      if (m.type === "state") applyState(m, applying);
      else if (m.type === "workflow") {
        if (m.ack >= seq) applyEditorWorkflow(m.workflow);
      } else if (m.type === "run-event") useRunStore.getState().apply(m.event);
      else if (m.type === "editor-closed") closed();
    });
    post({ type: "hello", panel });
    return () => {
      for (const off of offs) off();
      mirrorEditsTo(null);
      forwardPlaybackTo(null);
      bc.close();
    };
  }, [panel]);
}

export function forwardMenuCommand(command: MenuCommand): void {
  const bc = new BroadcastChannel(CHANNEL);
  bc.postMessage({ type: "menu", command } satisfies SyncMessage);
  bc.close();
}
