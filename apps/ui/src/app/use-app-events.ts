import { useRouter } from "@tanstack/react-router";
import { useEffect } from "react";
import { on } from "@/app/tauri";
import { watchLive } from "@/app/tauri";
import { commands, events } from "@/generated/bindings";
import { useLiveStore } from "../store/live";
import { useSecretsStore } from "../store/secrets";
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
        if (command === "view:settings") void commands.openSettings(null);
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
      on(events.runEventFired, (e) => {
        if (e.type === "run:done")
          void useSecretsStore.getState().refresh(useWorkflowStore.getState().projectDir);
      }),
    [],
  );
}

function useLiveEvents(): void {
  useEffect(() => watchLive((message) => useLiveStore.getState().apply(message)), []);
}

export function useAppEvents(): void {
  useMenuNavigation();
  useRunEvents();
  useLiveEvents();
}
