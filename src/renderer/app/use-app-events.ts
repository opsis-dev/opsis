import { useRouter } from "@tanstack/react-router";
import { useEffect } from "react";
import { exportedClipNodes } from "@/shared/graph";
import { useLiveStore } from "../store/live";
import { useRunStore } from "../store/run";
import { useSecretsStore } from "../store/secrets";
import { useStillsStore } from "../store/stills";
import { useWorkflowStore } from "../store/workflow";
import { onMenuCommand, projectActionOf, usePendingProjectAction } from "./menu-commands";

export const isSecondaryWindow = (path: string): boolean =>
  path.startsWith("/panel/") || path.startsWith("/settings");

function useMenuNavigation(): void {
  const router = useRouter();
  useEffect(
    () =>
      onMenuCommand((command) => {
        const path = router.state.location.pathname;
        if (isSecondaryWindow(path)) return;
        const action = projectActionOf(command);
        if (command === "view:settings") void window.demoflow.openSettings();
        else if (command === "view:projects") void router.navigate({ to: "/" });
        else if (action) {
          usePendingProjectAction.setState({ pending: action });
          if (path !== "/") void router.navigate({ to: "/" });
        }
      }),
    [router],
  );
}

function useRunEvents(): void {
  useEffect(
    () =>
      window.demoflow.onRunEvent((e) => {
        useRunStore.getState().apply(e);
        if (e.type !== "run:done") return;
        void useSecretsStore.getState().refresh(useWorkflowStore.getState().projectDir);
        const wf = useWorkflowStore.getState().workflow;
        if (e.ok && wf) useStillsStore.getState().want(exportedClipNodes(wf));
      }),
    [],
  );
}

function useStillEvents(): void {
  useEffect(
    () =>
      window.demoflow.onStillReady((still) =>
        useStillsStore.getState().ready(still.projectDir, still.nodeId, still.hash),
      ),
    [],
  );
}

function useLiveEvents(): void {
  useEffect(() => window.demoflow.onLive((message) => useLiveStore.getState().apply(message)), []);
}

export function useAppEvents(): void {
  useMenuNavigation();
  useRunEvents();
  useStillEvents();
  useLiveEvents();
}
