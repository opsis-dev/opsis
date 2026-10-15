import { createRootRoute, Outlet, useNavigate, useRouterState } from "@tanstack/react-router";
import { useEffect } from "react";
import { useLiveStore } from "../store/live";
import { useRunStore } from "../store/run";
import { useUiStore } from "../store/ui";
import { useSecretsStore } from "../store/secrets";
import { useWorkflowStore } from "../store/workflow";
import { SETUP_SKIPPED_KEY } from "../app/setup";
import { useMenuShortcuts, WindowBorder } from "../app/app-bar";

function Root() {
  const navigate = useNavigate();
  const path = useRouterState({ select: (s) => s.location.pathname });
  useMenuShortcuts();

  useEffect(
    () =>
      window.demoflow.onMenuCommand((command) => {
        const home = window.location.hash.replace(/^#/, "").split("?")[0] || "/";
        if (command === "view:settings") void navigate({ to: "/settings" });
        else if (command === "view:projects") void navigate({ to: "/" });
        else if ((command === "project:new" || command === "project:open") && home !== "/") {
          useUiStore.getState().setPendingProjectAction(command === "project:new" ? "new" : "open");
          void navigate({ to: "/" });
        }
      }),
    [navigate],
  );

  useEffect(
    () =>
      window.demoflow.onRunEvent((e) => {
        useRunStore.getState().apply(e);
        if (e.type === "run:done")
          void useSecretsStore.getState().refresh(useWorkflowStore.getState().projectDir);
      }),
    [],
  );

  useEffect(() => {
    let autoOpen = true;
    void window.demoflow.getSettings().then((s) => {
      autoOpen = s.live.autoOpen;
    });
    return window.demoflow.onLive((message) => useLiveStore.getState().apply(message, autoOpen));
  }, []);

  useEffect(() => {
    let skipped = false;
    try {
      skipped = localStorage.getItem(SETUP_SKIPPED_KEY) === "1";
    } catch {
      skipped = false;
    }
    if (skipped || path !== "/") return;
    void window.demoflow.getEnvironment().then((env) => {
      if (!env.engine.complete) void navigate({ to: "/setup" });
    });
    // oxlint-disable-next-line react/exhaustive-deps
  }, []);

  return (
    <>
      <Outlet />
      <WindowBorder />
    </>
  );
}

export const Route = createRootRoute({ component: Root });
