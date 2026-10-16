import { createRootRoute, Outlet, useNavigate, useRouterState } from "@tanstack/react-router";
import { useEffect } from "react";
import { useLiveStore } from "../store/live";
import { useRunStore } from "../store/run";
import { useUiStore } from "../store/ui";
import { useSecretsStore } from "../store/secrets";
import { useWorkflowStore } from "../store/workflow";
import { SETUP_SKIPPED_KEY } from "../app/setup";
import { useMenuShortcuts, WindowBorder } from "../app/app-bar";
import { ExitDialog } from "../app/exit-dialog";
import { useApiHost } from "../app/api-host";
import { useStillsStore } from "../store/stills";
import { exportedClipNodes } from "../editor/stills";
import { liveAutoOpen } from "../store/app-settings";

const STARTED_KEY = "demoflow.started";

function Root() {
  const navigate = useNavigate();
  const path = useRouterState({ select: (s) => s.location.pathname });
  useMenuShortcuts();
  useApiHost();

  useEffect(
    () =>
      window.demoflow.onMenuCommand((command) => {
        const home = window.location.hash.replace(/^#/, "").split("?")[0] || "/";
        if (home.startsWith("/panel/") || home.startsWith("/settings")) return;
        if (command === "view:settings") void window.demoflow.openSettings();
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
        if (e.type === "run:done" && e.ok) {
          const wf = useWorkflowStore.getState().workflow;
          if (wf) useStillsStore.getState().want(exportedClipNodes(wf));
        }
      }),
    [],
  );

  useEffect(
    () =>
      window.demoflow.onStillReady((still) =>
        useStillsStore.getState().ready(still.projectDir, still.nodeId, still.hash),
      ),
    [],
  );

  useEffect(
    () =>
      window.demoflow.onLive((message) => useLiveStore.getState().apply(message, liveAutoOpen())),
    [],
  );

  useEffect(() => {
    if (path !== "/") return;
    let skipped = false;
    let first = true;
    try {
      skipped = localStorage.getItem(SETUP_SKIPPED_KEY) === "1";
      first = sessionStorage.getItem(STARTED_KEY) !== "1";
      sessionStorage.setItem(STARTED_KEY, "1");
    } catch {
      skipped = false;
    }
    void Promise.all([window.demoflow.getEnvironment(), window.demoflow.getSettings()]).then(
      async ([env, settings]) => {
        if (!env.engine.complete && !skipped) {
          void navigate({ to: "/setup" });
          return;
        }
        if (!first) return;
        if (settings.app.onStart === "last-project") {
          const [last] = await window.demoflow.listRecentProjects();
          if (last) void navigate({ to: "/editor", search: { dir: last.dir } });
        } else if (settings.app.onStart === "new-project") {
          window.demoflow.dispatchMenuCommand("project:new");
        }
      },
    );
    // oxlint-disable-next-line react/exhaustive-deps
  }, []);

  return (
    <>
      <Outlet />
      {path.startsWith("/panel/") || path.startsWith("/settings") ? null : <ExitDialog />}
      <WindowBorder />
    </>
  );
}

export const Route = createRootRoute({ component: Root });
