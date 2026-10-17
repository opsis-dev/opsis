import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";
import { PanelBottomClose } from "lucide-react";
import { useEffect } from "react";
import type { DockPanelName } from "@/shared/ipc";
import { Button } from "@/renderer/components/ui/button";
import { Spinner } from "@/renderer/components/ui/spinner";
import { AppBar } from "../app/app-bar";
import { forwardMenuCommand, usePanelSync } from "../app/sync";
import { DockContent } from "../editor/dock";
import { FilmProvider } from "../editor/film";
import { DOCK_TITLES, isDockPanel } from "../editor/panels";
import { CompiledContext, useCompiled } from "../editor/use-compiled";
import { useMedia } from "../store/media";
import { useWorkflowStore } from "../store/workflow";
import { onMenuCommand } from "@/renderer/app/menu-commands";

const close = () => void window.demoflow.windowRole("close");

function PanelBody({ panel }: { panel: DockPanelName }) {
  const compiled = useCompiled();
  return (
    <CompiledContext.Provider value={compiled}>
      <FilmProvider>
        <section
          id={`panel-${panel}`}
          tabIndex={-1}
          aria-label={DOCK_TITLES[panel]}
          className="bg-sidebar min-h-0 flex-1 outline-none"
        >
          <DockContent panel={panel} />
        </section>
      </FilmProvider>
    </CompiledContext.Provider>
  );
}

function PanelRoute() {
  const { name } = Route.useParams();
  const panel = isDockPanel(name) ? name : "preview";
  const { project } = Route.useSearch();
  const workflow = useWorkflowStore((s) => s.workflow);
  usePanelSync(panel, close);

  useEffect(() => {
    useMedia.getState().reset(project || null);
  }, [project]);

  useEffect(() => {
    if (workflow) window.demoflow.setTitle(`${DOCK_TITLES[panel]} · ${workflow.name} — Demoflow`);
  }, [panel, workflow]);

  useEffect(
    () =>
      onMenuCommand((command) => {
        if (command === "view:projects" || command === "project:new" || command === "project:open")
          return;
        forwardMenuCommand(command);
      }),
    [],
  );

  return (
    <div className="flex h-screen flex-col">
      <AppBar>
        <span className="truncate font-semibold">{DOCK_TITLES[panel]}</span>
        {workflow ? (
          <span className="text-muted-foreground truncate text-sm">{workflow.name}</span>
        ) : null}
        <div className="flex-1" />
        <Button variant="ghost" size="sm" onClick={close}>
          <PanelBottomClose data-icon="inline-start" />
          Back to the editor
        </Button>
      </AppBar>
      {workflow ? (
        <PanelBody panel={panel} />
      ) : (
        <div className="grid flex-1 place-items-center">
          <Spinner />
        </div>
      )}
    </div>
  );
}

export const Route = createFileRoute("/panel/$name")({
  validateSearch: z.object({ project: z.coerce.string().default("") }),
  component: PanelRoute,
});
