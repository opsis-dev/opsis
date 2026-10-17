import { createRootRoute, Outlet, useRouterState } from "@tanstack/react-router";
import { useMenuShortcuts } from "../app/menu-shortcuts";
import { WindowBorder } from "../app/window-chrome";
import { ExitDialog } from "../app/exit-dialog";
import { useApiHost } from "../app/api-host";
import { isSecondaryWindow, useAppEvents } from "../app/use-app-events";
import { useStartup } from "../app/use-startup";

function Root() {
  const path = useRouterState({ select: (s) => s.location.pathname });
  useMenuShortcuts();
  useApiHost();
  useAppEvents();
  useStartup();

  return (
    <>
      <Outlet />
      {isSecondaryWindow(path) ? null : <ExitDialog />}
      <WindowBorder />
    </>
  );
}

export const Route = createRootRoute({ component: Root });
