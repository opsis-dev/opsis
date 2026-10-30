import { useEffect } from "react";
import { shallow } from "zustand/vanilla/shallow";
import { on, windowLabel } from "@/app/tauri";
import { commands, events, type MenuCommand, type Playhead } from "@/generated/bindings";
import { loadViewState } from "../editor/view-state";
import { useMedia } from "../store/media";
import {
  forwardPlaybackTo,
  playbackSnapshot,
  runPlaybackCommand,
  usePlayback,
} from "../store/playback";
import { useProjectFiles } from "../store/project-files";
import { selection, useUiStore } from "../store/ui";
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

function sharePlayhead(applying: Flag): () => void {
  return watchChanges(usePlayback, playbackSnapshot, applying, (next, last) => {
    const rangeMoved = next.inPoint !== last.inPoint || next.outPoint !== last.outPoint;
    if (!usePlayback.getState().hasPlayer && !rangeMoved) return;
    const { inPoint, outPoint, ...rest } = next;
    void commands.sharePlayhead({
      ...rest,
      ...(inPoint === null ? {} : { inPoint }),
      ...(outPoint === null ? {} : { outPoint }),
    });
  });
}

function showPlayhead({ inPoint, outPoint, ...rest }: Playhead): void {
  const range = { inPoint: inPoint ?? null, outPoint: outPoint ?? null };
  if (usePlayback.getState().hasPlayer) usePlayback.setState(range);
  else usePlayback.setState({ ...rest, ...range });
}

function useSharedState(): void {
  useEffect(() => {
    const applying = { current: false };
    const own = windowLabel();
    const offs = [
      watchChanges(useUiStore, selection, applying, (next) => void commands.shareSelection(next)),
      sharePlayhead(applying),
      on(events.selectionShared, ({ from, selection: shared }) => {
        if (from !== own) whileApplying(applying, () => useUiStore.setState(shared));
      }),
      on(events.playheadShared, ({ from, playhead }) => {
        if (from !== own) whileApplying(applying, () => showPlayhead(playhead));
      }),
    ];
    void commands.sharedState().then((shared) =>
      whileApplying(applying, () => {
        useUiStore.setState(shared.selection);
        if (shared.playhead) showPlayhead(shared.playhead);
      }),
    );
    return () => {
      for (const off of offs) off();
    };
  }, []);
}

export function useEditorSync(): void {
  useSharedState();
  useEffect(() => {
    const offs = [
      on(events.playbackCalled, runPlaybackCommand),
      on(events.replayRequested, ({ nodeId, tab }) => useUiStore.getState().setReplay(nodeId, tab)),
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
    void commands.viewState(projectDir).then((prefs) => {
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
  useSharedState();
  useProjectView();
  useEffect(() => {
    forwardPlaybackTo((call) => void commands.controlPlayback(call));
    return () => forwardPlaybackTo(null);
  }, []);
}

export function forwardMenuCommand(command: MenuCommand): void {
  return void commands.sendMenuToEditor(command);
}
