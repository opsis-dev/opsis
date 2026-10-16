import { useEffect } from "react";
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
import { useUiStore, type PreviewQuality, type PreviewShape } from "../store/ui";
import { mirrorEditsTo, useWorkflowStore, type RemoteEdit } from "../store/workflow";

const CHANNEL = "demoflow-sync";
const LIVE_CHANNEL = "demoflow-live";

type RunSnapshot = Pick<
  ReturnType<typeof useRunStore.getState>,
  | "runId"
  | "running"
  | "statuses"
  | "progress"
  | "cached"
  | "errors"
  | "traces"
  | "logs"
  | "results"
  | "outputs"
  | "lastError"
>;

type Selection = { selectedIds: string[]; activeIds: string[]; pinnedSource: string | null };

type PlaybackSnapshot = Pick<
  ReturnType<typeof usePlayback.getState>,
  "frame" | "playing" | "rate" | "fps" | "durationInFrames" | "inPoint" | "outPoint"
>;

export type SyncMessage =
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

const windowId = Math.random().toString(36).slice(2);

function channel(): BroadcastChannel | null {
  try {
    return new BroadcastChannel(CHANNEL);
  } catch {
    return null;
  }
}

const runSnapshot = (): RunSnapshot => {
  const s = useRunStore.getState();
  return {
    runId: s.runId,
    running: s.running,
    statuses: s.statuses,
    progress: s.progress,
    cached: s.cached,
    errors: s.errors,
    traces: s.traces,
    logs: s.logs,
    results: s.results,
    outputs: s.outputs,
    lastError: s.lastError,
  };
};

const selection = (): Selection => {
  const s = useUiStore.getState();
  return { selectedIds: s.selectedIds, activeIds: s.activeIds, pinnedSource: s.pinnedSource };
};

const playbackSnapshot = (): PlaybackSnapshot => {
  const s = usePlayback.getState();
  return {
    frame: s.frame,
    playing: s.playing,
    rate: s.rate,
    fps: s.fps,
    durationInFrames: s.durationInFrames,
    inPoint: s.inPoint,
    outPoint: s.outPoint,
  };
};

function samePlayback(a: PlaybackSnapshot, b: PlaybackSnapshot): boolean {
  return (
    a.frame === b.frame &&
    a.playing === b.playing &&
    a.rate === b.rate &&
    a.fps === b.fps &&
    a.durationInFrames === b.durationInFrames &&
    a.inPoint === b.inPoint &&
    a.outPoint === b.outPoint
  );
}

function syncPlayback(bc: BroadcastChannel, applying: { current: boolean }) {
  let last = playbackSnapshot();
  return usePlayback.subscribe((s) => {
    if (applying.current) {
      last = playbackSnapshot();
      return;
    }
    const next = playbackSnapshot();
    if (samePlayback(next, last)) return;
    const inOutChanged = next.inPoint !== last.inPoint || next.outPoint !== last.outPoint;
    last = next;
    if (inOutChanged)
      bc.postMessage({
        type: "inout",
        inPoint: next.inPoint,
        outPoint: next.outPoint,
        from: windowId,
      } satisfies SyncMessage);
    if (s.hasPlayer)
      bc.postMessage({ type: "frame", playback: next, from: windowId } satisfies SyncMessage);
  });
}

function applyRemotePlayback(message: SyncMessage, applying: { current: boolean }): boolean {
  if (message.type === "frame" && message.from !== windowId) {
    if (usePlayback.getState().hasPlayer) return true;
    applying.current = true;
    usePlayback.setState(message.playback);
    applying.current = false;
    return true;
  }
  if (message.type === "inout" && message.from !== windowId) {
    applying.current = true;
    usePlayback.setState({ inPoint: message.inPoint, outPoint: message.outPoint });
    applying.current = false;
    return true;
  }
  if (message.type === "playback") {
    runPlaybackCommand(message.command);
    return true;
  }
  return false;
}

function useLiveRelay(): void {
  useEffect(() => {
    let bc: BroadcastChannel;
    try {
      bc = new BroadcastChannel(LIVE_CHANNEL);
    } catch {
      return;
    }
    let autoOpen = true;
    void window.demoflow.getSettings().then((s) => {
      autoOpen = s.live.autoOpen;
    });
    const offLocal = window.demoflow.onLive((message) => bc.postMessage(message));
    bc.addEventListener("message", (e) =>
      useLiveStore.getState().apply(e.data as LiveMessage, autoOpen),
    );
    return () => {
      offLocal();
      bc.close();
    };
  }, []);
}

export function useEditorSync(): void {
  useLiveRelay();
  useEffect(() => {
    const bc = channel();
    if (!bc) return;
    const applying = { current: false };
    let ack = 0;
    let lastWorkflow = useWorkflowStore.getState().workflow;
    const post = (m: SyncMessage) => bc.postMessage(m);
    const sendState = () => {
      const { projectDir, workflow } = useWorkflowStore.getState();
      if (!projectDir || !workflow) return;
      post({
        type: "state",
        projectDir,
        workflow,
        ack,
        run: runSnapshot(),
        selection: selection(),
        quality: useUiStore.getState().previewQuality,
        shape: useUiStore.getState().previewShape,
        playback: playbackSnapshot(),
        host: usePlayback.getState().hasPlayer,
      });
    };
    forwardPlaybackTo((command) => post({ type: "playback", command }));
    const offWorkflow = useWorkflowStore.subscribe((s) => {
      if (s.workflow === lastWorkflow || !s.workflow) return;
      lastWorkflow = s.workflow;
      post({ type: "workflow", workflow: s.workflow, ack });
    });
    let lastSelection = selection();
    let lastQuality = useUiStore.getState().previewQuality;
    let lastShape = useUiStore.getState().previewShape;
    const offUi = useUiStore.subscribe((s) => {
      if (s.previewQuality !== lastQuality) {
        lastQuality = s.previewQuality;
        post({ type: "quality", quality: s.previewQuality });
      }
      if (s.previewShape !== lastShape) {
        lastShape = s.previewShape;
        post({ type: "shape", shape: s.previewShape });
      }
      const next = selection();
      if (
        next.selectedIds === lastSelection.selectedIds &&
        next.activeIds === lastSelection.activeIds &&
        next.pinnedSource === lastSelection.pinnedSource
      )
        return;
      lastSelection = next;
      if (!applying.current) post({ type: "selection", selection: next, from: windowId });
    });
    const offPlayback = syncPlayback(bc, applying);
    const offRun = window.demoflow.onRunEvent((event) => post({ type: "run-event", event }));
    bc.addEventListener("message", (e) => {
      const m = e.data as SyncMessage;
      if (applyRemotePlayback(m, applying)) return;
      switch (m.type) {
        case "hello":
          sendState();
          break;
        case "edit": {
          ack = Math.max(ack, m.seq);
          const store = useWorkflowStore.getState();
          if (m.edit.type === "replace") store.replace(m.edit.workflow);
          else if (m.edit.phase === "begin") store.beginGesture();
          else store.endGesture();
          break;
        }
        case "selection":
          if (m.from === windowId) break;
          applying.current = true;
          useUiStore.setState(m.selection);
          applying.current = false;
          break;
        case "quality":
          useUiStore.getState().setPreviewQuality(m.quality);
          break;
        case "shape":
          useUiStore.getState().setPreviewShape(m.shape);
          break;
        case "menu":
          window.demoflow.dispatchMenuCommand(m.command);
          break;
        case "replay":
          useUiStore.getState().setReplay(m.nodeId);
          break;
      }
    });
    sendState();
    return () => {
      offWorkflow();
      offUi();
      offPlayback();
      offRun();
      forwardPlaybackTo(null);
      post({ type: "editor-closed" });
      bc.close();
    };
  }, []);
}

export function usePanelSync(panel: DockPanelName, onClosed: () => void): void {
  useLiveRelay();
  useEffect(() => {
    const bc = channel();
    if (!bc) return;
    const applying = { current: false };
    let seq = 0;
    const post = (m: SyncMessage) => bc.postMessage(m);
    mirrorEditsTo((edit) => post({ type: "edit", edit, seq: ++seq }));
    forwardPlaybackTo((command) => post({ type: "playback", command }));
    const setWorkflow = (workflow: Workflow, ack: number) => {
      if (ack < seq) return;
      const store = useWorkflowStore.getState();
      if (store.workflow === workflow) return;
      useWorkflowStore.setState({
        workflow,
        revision: store.revision + 1,
        savedRevision: store.revision + 1,
      });
    };
    let lastSelection = selection();
    let lastQuality = useUiStore.getState().previewQuality;
    let lastShape = useUiStore.getState().previewShape;
    const offUi = useUiStore.subscribe((s) => {
      if (s.previewQuality !== lastQuality) {
        lastQuality = s.previewQuality;
        if (!applying.current) post({ type: "quality", quality: s.previewQuality });
      }
      if (s.previewShape !== lastShape) {
        lastShape = s.previewShape;
        if (!applying.current) post({ type: "shape", shape: s.previewShape });
      }
      const next = selection();
      if (
        next.selectedIds === lastSelection.selectedIds &&
        next.pinnedSource === lastSelection.pinnedSource
      ) {
        lastSelection = next;
        return;
      }
      lastSelection = next;
      if (!applying.current) post({ type: "selection", selection: next, from: windowId });
    });
    const offPlayback = syncPlayback(bc, applying);
    bc.addEventListener("message", (e) => {
      const m = e.data as SyncMessage;
      if (applyRemotePlayback(m, applying)) return;
      switch (m.type) {
        case "state":
          if (useWorkflowStore.getState().projectDir !== m.projectDir) loadViewState(m.projectDir);
          useWorkflowStore.setState({
            projectDir: m.projectDir,
            workflow: m.workflow,
            revision: 0,
            savedRevision: 0,
          });
          useRunStore.setState(m.run);
          applying.current = true;
          useUiStore.setState({
            ...m.selection,
            previewQuality: m.quality,
            previewShape: m.shape,
          });
          if (!usePlayback.getState().hasPlayer) usePlayback.setState(m.playback);
          applying.current = false;
          break;
        case "workflow":
          setWorkflow(m.workflow, m.ack);
          break;
        case "run-event":
          useRunStore.getState().apply(m.event);
          break;
        case "selection":
          if (m.from === windowId) break;
          applying.current = true;
          useUiStore.setState(m.selection);
          applying.current = false;
          break;
        case "quality":
          applying.current = true;
          useUiStore.setState({ previewQuality: m.quality });
          applying.current = false;
          break;
        case "shape":
          applying.current = true;
          useUiStore.setState({ previewShape: m.shape });
          applying.current = false;
          break;
        case "editor-closed":
          onClosed();
          break;
      }
    });
    post({ type: "hello", panel });
    return () => {
      offUi();
      offPlayback();
      mirrorEditsTo(null);
      forwardPlaybackTo(null);
      bc.close();
    };
  }, [panel, onClosed]);
}

export function forwardMenuCommand(command: MenuCommand): void {
  const bc = channel();
  if (!bc) return;
  bc.postMessage({ type: "menu", command } satisfies SyncMessage);
  bc.close();
}
