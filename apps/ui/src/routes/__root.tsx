import { createRootRoute, Outlet } from "@tanstack/react-router";
import { WindowBorder } from "../app/window-chrome";

function Root() {
  return (
    <>
      <Outlet />
      <WindowBorder />
    </>
  );
}

export const Route = createRootRoute({ component: Root });
