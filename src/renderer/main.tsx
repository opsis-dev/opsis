import "@xyflow/react/dist/style.css";
import "./index.css";
import { createHashHistory, createRouter, RouterProvider } from "@tanstack/react-router";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Toaster } from "@/renderer/components/ui/sonner";
import { TooltipProvider } from "@/renderer/components/ui/tooltip";
import { routeTree } from "./routeTree.gen";

const router = createRouter({ routeTree, history: createHashHistory(), defaultPreload: "intent" });

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

const container = document.getElementById("root");
if (!container) throw new Error("Renderer root element missing.");

createRoot(container).render(
  <StrictMode>
    <TooltipProvider>
      <RouterProvider router={router} />
      <Toaster richColors position="bottom-right" theme="dark" />
    </TooltipProvider>
  </StrictMode>,
);
