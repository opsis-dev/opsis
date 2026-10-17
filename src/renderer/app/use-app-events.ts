import { useRouter } from "@tanstack/react-router";
import { useEffect } from "react";
import { exportedClipNodes } from "@/shared/graph";
import { liveAutoOpen } from "../store/app-settings";
import { useLiveStore } from "../store/live";
import { useRunStore } from "../store/run";
import { useSecretsStore } from "../store/secrets";
import { useStillsStore } from "../store/stills";
import { useWorkflowStore } from "../store/workflow";
import { onMenuCommand } from "@/renderer/app/menu-commands";

export const isSecondaryWindow = (path: string): boolean =>
  path.startsWith("/panel/") || path.startsWith("/settings");

function useMenuNavigation(): void {
  const router = useRouter();
  useEffect(
    () =>
      onMenuCommand((command) => {
        const path = router.state.location.pathname;
        if (isSecondaryWindow(path)) return;
        if (command === "view:settings") void window.demoflow.openSettings();
        else if (command === "view:projects") void router.navigate({ to: "/" });
        else if ((command === "project:new" || command === "project:open") && path !== "/")
          void router.navigate({
            to: "/",
            search: { action: command === "project:new" ? "new" : "open" },
          });
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
  useEffect(
    () =>
      window.demoflow.onLive((message) => useLiveStore.getState().apply(message, liveAutoOpen())),
    [],
  );
}

export function useAppEvents(): void {
  useMenuNavigation();
  useRunEvents();
  useStillEvents();
  useLiveEvents();
}
