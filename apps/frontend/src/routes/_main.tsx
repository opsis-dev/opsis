import { createFileRoute, Outlet } from "@tanstack/react-router";
import { useApiHost } from "@/app/api-host";
import { ExitDialog } from "@/app/exit-dialog";
import { useMenuNavigation } from "@/app/menu-navigation";
import { useMenuShortcuts } from "@/app/menu-shortcuts";
import { useStartup } from "@/app/use-startup";

function MainWindow() {
  useMenuShortcuts();
  useMenuNavigation();
  useApiHost();
  useStartup();
  return (
    <>
      <Outlet />
      <ExitDialog />
    </>
  );
}

export const Route = createFileRoute("/_main")({ component: MainWindow });
