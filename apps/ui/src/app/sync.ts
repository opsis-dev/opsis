import { useEffect } from "react";
import { shallow } from "zustand/vanilla/shallow";
import type { MenuCommand, Playhead, SharedView } from "@/generated/bindings";
import { loadViewState } from "../editor/view-state";
import { useLiveStore } from "../store/live";
import { useMedia } from "../store/media";
import {
  forwardPlaybackTo,
  playbackSnapshot,
  runPlaybackCommand,
  usePlayback,
} from "../store/playback";
import { useProjectFiles } from "../store/project-files";
import { selection, useUiStore } from "../store/ui";
import { isPreviewShape, syncedView, useViewOptions } from "../store/view-options";
import { useWorkflowStore } from "../store/workflow";

type Flag = { current: boolean };

type Subscribable = { subscribe: (listener: () => void) => () => void };

function whileApplying(applying: Flag, fn: () => void): void {
  applying.current = true;
  try {
    fn();
  } finally {
    applying.current = false;
  }
}

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

const sharePlayhead = (applying: Flag): (() => void) =>
  watchChanges(usePlayback, playbackSnapshot, applying, (next, last) => {
    const rangeMoved = next.inPoint !== last.inPoint || next.outPoint !== last.outPoint;
    if (!usePlayback.getState().hasPlayer && !rangeMoved) return;
    const { inPoint, outPoint, ...rest } = next;
    window.opsis.sharePlayhead({
      ...rest,
      ...(inPoint === null ? {} : { inPoint }),
      ...(outPoint === null ? {} : { outPoint }),
    });
  });

function showPlayhead({ inPoint, outPoint, ...rest }: Playhead): void {
  const range = { inPoint: inPoint ?? null, outPoint: outPoint ?? null };
  if (usePlayback.getState().hasPlayer) usePlayback.setState(range);
  else usePlayback.setState({ ...rest, ...range });
}

const showView = ({ previewQuality, previewShape }: SharedView): void =>
  useViewOptions.setState({
    previewQuality,
    previewShape: isPreviewShape(previewShape) ? previewShape : "project",
  });

function useSharedState(): void {
  useEffect(() => {
    const applying = { current: false };
    const own = window.opsis.windowLabel;
    const offs = [
      watchChanges(useUiStore, selection, applying, (next) => window.opsis.shareSelection(next)),
      watchChanges(useViewOptions, syncedView, applying, (next) => window.opsis.shareView(next)),
      sharePlayhead(applying),
      window.opsis.onSelectionShared(({ from, selection: shared }) => {
        if (from !== own) whileApplying(applying, () => useUiStore.setState(shared));
      }),
      window.opsis.onViewShared(({ from, view }) => {
        if (from !== own) whileApplying(applying, () => showView(view));
      }),
      window.opsis.onPlayheadShared(({ from, playhead }) => {
        if (from !== own) whileApplying(applying, () => showPlayhead(playhead));
      }),
    ];
    void window.opsis.sharedState().then((shared) =>
      whileApplying(applying, () => {
        useUiStore.setState(shared.selection);
        if (shared.view) showView(shared.view);
        if (shared.playhead) showPlayhead(shared.playhead);
      }),
    );
    return () => {
      for (const off of offs) off();
    };
  }, []);
}

function useLiveFrames(): void {
  useEffect(() => window.opsis.onLive((message) => useLiveStore.getState().apply(message)), []);
}

export function useEditorSync(): void {
  useLiveFrames();
  useSharedState();
  useEffect(() => {
    const offs = [
      window.opsis.onPlaybackCalled(runPlaybackCommand),
      window.opsis.onReplayRequested(({ nodeId, tab }) =>
        useUiStore.getState().setReplay(nodeId, tab),
      ),
    ];
    return () => {
      for (const off of offs) off();
    };
  }, []);
}

function useProjectView(): void {
  const projectDir = useWorkflowStore((s) => s.projectDir);
  useEffect(() => {
    if (!projectDir) return;
    let current = true;
    void window.opsis.viewState(projectDir).then((prefs) => {
      if (!current) return;
      loadViewState(projectDir, prefs);
      useMedia.getState().reset(projectDir);
      useProjectFiles.getState().forget();
    });
    return () => {
      current = false;
    };
  }, [projectDir]);
}

export function useWindowSync(): void {
  useLiveFrames();
  useSharedState();
  useProjectView();
  useEffect(() => {
    forwardPlaybackTo((call) => window.opsis.controlPlayback(call));
    return () => forwardPlaybackTo(null);
  }, []);
}

export const forwardMenuCommand = (command: MenuCommand): void =>
  window.opsis.sendMenuToEditor(command);
