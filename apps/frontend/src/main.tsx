import "@xyflow/react/dist/style.css";
import "./index.css";
import { createHashHistory, createRouter, RouterProvider } from "@tanstack/react-router";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Toaster } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { ErrorScreen } from "./app/error-screen";
import { MotionProvider } from "./app/motion";
import { loadFileRoot } from "./editor/file-url";
import { routeTree } from "./routeTree.gen";
import { watchAppSettings } from "./store/app-settings";
import { loadCatalog } from "./store/catalog";
import { watchViewPrefs } from "./store/view-prefs";
import { watchWindowState } from "./store/window-state";

watchAppSettings();
watchViewPrefs();
watchWindowState();

const router = createRouter({
  routeTree,
  history: createHashHistory(),
  defaultPreload: "intent",
  defaultErrorComponent: ErrorScreen,
  defaultOnCatch: (error) => console.error(error),
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

const container = document.getElementById("root");
if (!container) throw new Error("Renderer root element missing.");

await Promise.all([loadCatalog(), loadFileRoot()]);

createRoot(container).render(
  <StrictMode>
    <MotionProvider>
      <TooltipProvider>
        <RouterProvider router={router} />
        <Toaster richColors position="bottom-right" theme="dark" />
      </TooltipProvider>
    </MotionProvider>
  </StrictMode>,
);
