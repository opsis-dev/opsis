import { hasTypeIn } from "@opsis/video/common/guards";
import { useEffect, useEffectEvent } from "react";
import { shallow } from "zustand/vanilla/shallow";
import { randomId } from "@/core/ids";
import type { MenuCommand, RunEvent } from "@/generated/bindings";
import { loadViewState } from "../editor/view-state";
import { useLiveStore } from "../store/live";
import { useMedia } from "../store/media";
import {
  forwardPlaybackTo,
  playbackSnapshot,
  runPlaybackCommand,
  usePlayback,
  type PlaybackCommand,
  type PlaybackSnapshot,
} from "../store/playback";
import { useProjectFiles } from "../store/project-files";
import { runSnapshot, useRunStore, type RunSnapshot } from "../store/run";
import { selection, useUiStore, type Selection } from "../store/ui";
import { syncedView, useViewOptions, type SyncedView } from "../store/view-options";
import { viewPrefs, type ViewPrefs } from "../store/view-prefs";
import { useWorkflowStore } from "../store/workflow";
import { dispatchMenuCommand } from "./menu-commands";
import { relayChannel } from "./relay-channel";

const CHANNEL = "opsis-sync";

type SyncMessage =
  | { type: "hello" }
  | {
      type: "state";
      projectDir: string;
      run: RunSnapshot;
      selection: Selection;
      view: SyncedView;
      prefs: ViewPrefs;
      playback: PlaybackSnapshot;
    }
  | { type: "run-event"; event: RunEvent }
  | { type: "selection"; selection: Selection; from: string }
  | { type: "view"; view: SyncedView }
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
  "run-event": true,
  selection: true,
  view: true,
  menu: true,
  replay: true,
  playback: true,
  frame: true,
  inout: true,
  "editor-closed": true,
};

const windowId = randomId();

const isSyncMessage = hasTypeIn<SyncMessage>(SYNC_TYPES);

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

const watchViewOptions = (post: Post, applying: Flag): (() => void) =>
  watchChanges(useViewOptions, syncedView, applying, (next) => post({ type: "view", view: next }));

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
  view: (m) => whileApplying(applying, () => useViewOptions.setState(m.view)),
});

function handle<T extends keyof MessageOf>(
  handlers: Handlers,
  type: T,
  message: MessageOf[T],
): void {
  handlers[type]?.(message);
}

type SyncChannel = {
  post: Post;
  applying: Flag;
  on: (handlers: Handlers) => void;
  close: () => void;
};

function openSyncChannel(): SyncChannel {
  const bc = relayChannel(CHANNEL);
  const post: Post = (m) => bc.postMessage(m);
  const applying = { current: false };
  const table = sharedHandlers(applying);
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
    on: (handlers) => Object.assign(table, handlers),
    close: () => {
      for (const off of offs) off();
      forwardPlaybackTo(null);
      bc.close();
    },
  };
}

function useLiveFrames(): void {
  useEffect(() => window.opsis.onLive((message) => useLiveStore.getState().apply(message)), []);
}

function sendState(post: Post): void {
  const { projectDir } = useWorkflowStore.getState();
  if (!projectDir) return;
  post({
    type: "state",
    projectDir,
    run: runSnapshot(),
    selection: selection(),
    view: syncedView(),
    prefs: viewPrefs(),
    playback: playbackSnapshot(),
  });
}

export function useEditorSync(): void {
  useLiveFrames();
  useEffect(() => {
    const channel = openSyncChannel();
    channel.on({
      hello: () => sendState(channel.post),
      menu: (m) => dispatchMenuCommand(m.command),
      replay: (m) => useUiStore.getState().setReplay(m.nodeId),
    });
    const offRun = window.opsis.onRunEvent((event) => channel.post({ type: "run-event", event }));
    sendState(channel.post);
    return () => {
      offRun();
      channel.post({ type: "editor-closed" });
      channel.close();
    };
  }, []);
}

let appliedDir: string | null = null;

function applyState(m: MessageOf["state"], applying: Flag): void {
  if (appliedDir !== m.projectDir) {
    appliedDir = m.projectDir;
    loadViewState(m.projectDir, m.prefs);
    useMedia.getState().reset(m.projectDir);
    useProjectFiles.getState().forget();
  }
  useRunStore.setState(m.run);
  whileApplying(applying, () => {
    useUiStore.setState(m.selection);
    useViewOptions.setState(m.view);
    if (!usePlayback.getState().hasPlayer) usePlayback.setState(m.playback);
  });
}

export function useWindowSync(onClosed: () => void): void {
  useLiveFrames();
  const closed = useEffectEvent(onClosed);
  useEffect(() => {
    const channel = openSyncChannel();
    channel.on({
      state: (m) => applyState(m, channel.applying),
      "run-event": (m) => useRunStore.getState().apply(m.event),
      "editor-closed": () => closed(),
    });
    channel.post({ type: "hello" });
    return () => channel.close();
  }, []);
}

export function forwardMenuCommand(command: MenuCommand): void {
  const message: SyncMessage = { type: "menu", command };
  void window.opsis.relay("editor", { channel: CHANNEL, message });
}
