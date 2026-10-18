import { useEffect, useEffectEvent } from "react";
import { shallow } from "zustand/vanilla/shallow";
import type { DockPanelName } from "@/shared/ipc";
import type { MenuCommand } from "@/shared/menu";
import type { LiveMessage } from "@/shared/live";
import type { RunEvent } from "@/shared/schema/events";
import type { PreviewQuality } from "@/shared/schema/timeline";
import type { Workflow } from "@/shared/schema/workflow";
import { hasStringType } from "@/shared/guards";
import { randomId } from "@/shared/ids";
import { pick } from "@/shared/pick";
import { loadViewState } from "../editor/view-state";
import { useLiveStore } from "../store/live";
import {
  forwardPlaybackTo,
  playbackSnapshot,
  runPlaybackCommand,
  usePlayback,
  type PlaybackCommand,
  type PlaybackSnapshot,
} from "../store/playback";
import { runSnapshot, useRunStore, type RunSnapshot } from "../store/run";
import { selection, useUiStore, type Selection } from "../store/ui";
import { useViewOptions, type PreviewShape } from "../store/view-options";
import { mirrorEditsTo, useWorkflowStore, type RemoteEdit } from "../store/workflow";
import { dispatchMenuCommand } from "./menu-commands";

const CHANNEL = "demoflow-sync";
const LIVE_CHANNEL = "demoflow-live";

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

type MessageOf = { [M in SyncMessage as M["type"]]: M };
type Handlers = { [T in keyof MessageOf]?: (message: MessageOf[T]) => void };

const SYNC_TYPES: Record<SyncMessage["type"], true> = {
  hello: true,
  state: true,
  workflow: true,
  "run-event": true,
  selection: true,
  quality: true,
  shape: true,
  edit: true,
  menu: true,
  replay: true,
  playback: true,
  frame: true,
  inout: true,
  "editor-closed": true,
};

const windowId = randomId();

const isSyncMessage = (data: unknown): data is SyncMessage =>
  hasStringType(data) && Object.hasOwn(SYNC_TYPES, data.type);

const isLiveMessage = (data: unknown): data is LiveMessage =>
  hasStringType(data) && typeof data.captureId === "string";

function whileApplying(applying: Flag, fn: () => void): void {
  applying.current = true;
  try {
    fn();
  } finally {
    applying.current = false;
  }
}

type Subscribable = { subscribe: (listener: () => void) => () => void };

function watchChanges<T extends object>(
  store: Subscribable,
  read: () => T,
  applying: Flag,
  changed: (next: T, last: T) => void,
): () => void {
  let last = read();
  return store.subscribe(() => {
    const next = read();
    if (shallow(next, last)) return;
    const previous = last;
    last = next;
    if (!applying.current) changed(next, previous);
  });
}

const syncPlayback = (post: Post, applying: Flag): (() => void) =>
  watchChanges(usePlayback, playbackSnapshot, applying, (next, last) => {
    if (next.inPoint !== last.inPoint || next.outPoint !== last.outPoint)
      post({ type: "inout", inPoint: next.inPoint, outPoint: next.outPoint, from: windowId });
    if (usePlayback.getState().hasPlayer) post({ type: "frame", playback: next, from: windowId });
  });

const watchSelection = (post: Post, applying: Flag): (() => void) =>
  watchChanges(useUiStore, selection, applying, (next) =>
    post({ type: "selection", selection: next, from: windowId }),
  );

const SYNCED_VIEW_OPTIONS = ["previewQuality", "previewShape"] as const;

const watchViewOptions = (post: Post, applying: Flag): (() => void) =>
  watchChanges(
    useViewOptions,
    () => pick(useViewOptions.getState(), SYNCED_VIEW_OPTIONS),
    applying,
    (next, last) => {
      if (next.previewQuality !== last.previewQuality)
        post({ type: "quality", quality: next.previewQuality });
      if (next.previewShape !== last.previewShape)
        post({ type: "shape", shape: next.previewShape });
    },
  );

const sharedHandlers = (applying: Flag): Handlers => ({
  frame: (m) => {
    if (m.from !== windowId && !usePlayback.getState().hasPlayer)
      whileApplying(applying, () => usePlayback.setState(m.playback));
  },
  inout: (m) => {
    if (m.from !== windowId)
      whileApplying(applying, () =>
        usePlayback.setState({ inPoint: m.inPoint, outPoint: m.outPoint }),
      );
  },
  playback: (m) => runPlaybackCommand(m.command),
  selection: (m) => {
    if (m.from !== windowId) whileApplying(applying, () => useUiStore.setState(m.selection));
  },
  quality: (m) =>
    whileApplying(applying, () => useViewOptions.setState({ previewQuality: m.quality })),
  shape: (m) => whileApplying(applying, () => useViewOptions.setState({ previewShape: m.shape })),
});

function handle<T extends keyof MessageOf>(
  handlers: Handlers,
  type: T,
  message: MessageOf[T],
): void {
  handlers[type]?.(message);
}

type SyncChannel = { post: Post; applying: Flag; close: () => void };

function openSyncChannel(handlers: (channel: Omit<SyncChannel, "close">) => Handlers): SyncChannel {
  const bc = new BroadcastChannel(CHANNEL);
  const post: Post = (m) => bc.postMessage(m);
  const applying = { current: false };
  const table = { ...sharedHandlers(applying), ...handlers({ post, applying }) };
  forwardPlaybackTo((command) => post({ type: "playback", command }));
  const offs = [
    watchSelection(post, applying),
    watchViewOptions(post, applying),
    syncPlayback(post, applying),
  ];
  bc.addEventListener("message", (e) => {
    if (isSyncMessage(e.data)) handle(table, e.data.type, e.data);
  });
  return {
    post,
    applying,
    close: () => {
      for (const off of offs) off();
      forwardPlaybackTo(null);
      bc.close();
    },
  };
}

function useLiveRelay(): void {
  useEffect(() => {
    const bc = new BroadcastChannel(LIVE_CHANNEL);
    const offLocal = window.demoflow.onLive((message) => bc.postMessage(message));
    bc.addEventListener("message", (e) => {
      if (isLiveMessage(e.data)) useLiveStore.getState().apply(e.data);
    });
    return () => {
      offLocal();
      bc.close();
    };
  }, []);
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
  if (edit.type === "replace") store.applyPanelEdit(edit.workflow);
  else if (edit.phase === "begin") store.beginGesture();
  else store.endGesture();
}

export function useEditorSync(): void {
  useLiveRelay();
  useEffect(() => {
    let ack = 0;
    const channel = openSyncChannel(({ post }) => ({
      hello: () => sendState(post, ack),
      edit: (m) => {
        ack = Math.max(ack, m.seq);
        applyRemoteEdit(m.edit);
      },
      menu: (m) => dispatchMenuCommand(m.command),
      replay: (m) => useUiStore.getState().setReplay(m.nodeId),
    }));
    const offs = [
      watchWorkflow(channel.post, () => ack),
      window.demoflow.onRunEvent((event) => channel.post({ type: "run-event", event })),
    ];
    sendState(channel.post, ack);
    return () => {
      for (const off of offs) off();
      channel.post({ type: "editor-closed" });
      channel.close();
    };
  }, []);
}

function applyState(m: MessageOf["state"], applying: Flag): void {
  if (useWorkflowStore.getState().projectDir !== m.projectDir) loadViewState(m.projectDir);
  useWorkflowStore.getState().mirrorProject(m.projectDir, m.workflow);
  useRunStore.setState(m.run);
  whileApplying(applying, () => {
    useUiStore.setState(m.selection);
    useViewOptions.setState({ previewQuality: m.quality, previewShape: m.shape });
    if (!usePlayback.getState().hasPlayer) usePlayback.setState(m.playback);
  });
}

export function usePanelSync(panel: DockPanelName, onClosed: () => void): void {
  useLiveRelay();
  const closed = useEffectEvent(onClosed);
  useEffect(() => {
    let seq = 0;
    const channel = openSyncChannel(({ applying }) => ({
      state: (m) => applyState(m, applying),
      workflow: (m) => {
        if (m.ack >= seq) useWorkflowStore.getState().mirrorSaved(m.workflow);
      },
      "run-event": (m) => useRunStore.getState().apply(m.event),
      "editor-closed": () => closed(),
    }));
    mirrorEditsTo((edit) => channel.post({ type: "edit", edit, seq: ++seq }));
    channel.post({ type: "hello", panel });
    return () => {
      mirrorEditsTo(null);
      channel.close();
    };
  }, [panel]);
}

export function forwardMenuCommand(command: MenuCommand): void {
  const bc = new BroadcastChannel(CHANNEL);
  bc.postMessage({ type: "menu", command } satisfies SyncMessage);
  bc.close();
}
