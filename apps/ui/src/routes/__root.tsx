import { createRootRoute, Outlet, useRouterState } from "@tanstack/react-router";
import { useApiHost } from "../app/api-host";
import { useDocumentMirror } from "../app/document-mirror";
import { ExitDialog } from "../app/exit-dialog";
import { useMenuShortcuts } from "../app/menu-shortcuts";
import { isSecondaryWindow, useAppEvents } from "../app/use-app-events";
import { useStartup } from "../app/use-startup";
import { WindowBorder } from "../app/window-chrome";

function Root() {
  const path = useRouterState({ select: (s) => s.location.pathname });
  useMenuShortcuts();
  useApiHost();
  useDocumentMirror();
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
