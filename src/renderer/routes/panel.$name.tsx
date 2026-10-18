import { createFileRoute } from "@tanstack/react-router";
import { PanelBottomClose } from "lucide-react";
import { useEffect } from "react";
import { z } from "zod";
import { Button } from "@/renderer/components/ui/button";
import { DOCK_TITLES, type DockPanelName } from "@/shared/ipc";
import { AppBar } from "../app/app-bar";
import { LoadingPanel } from "../app/empty-panel";
import { onMenuCommand, PROJECT_SWITCH_COMMANDS } from "../app/menu-commands";
import { PROJECT_DIR_PARAM } from "../app/open-project";
import { forwardMenuCommand, usePanelSync } from "../app/sync";
import { DockContent } from "../editor/dock";
import { FilmProvider } from "../editor/film";
import { isDockPanel } from "../editor/panels";
import { CompiledContext, useCompiled } from "../editor/use-compiled";
import { useMedia } from "../store/media";
import { useWorkflowStore } from "../store/workflow";

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
  const { name: panel } = Route.useParams();
  const { project } = Route.useSearch();
  const projectName = useWorkflowStore((s) => s.workflow?.name ?? null);
  usePanelSync(panel, close);

  useEffect(() => {
    useMedia.getState().reset(project || null);
  }, [project]);

  useEffect(() => {
    if (projectName !== null)
      window.demoflow.setTitle(`${DOCK_TITLES[panel]} · ${projectName} — Demoflow`);
  }, [panel, projectName]);

  useEffect(
    () =>
      onMenuCommand((command) => {
        if (!PROJECT_SWITCH_COMMANDS.has(command)) forwardMenuCommand(command);
      }),
    [],
  );

  return (
    <div className="flex h-screen flex-col">
      <AppBar>
        <span className="truncate font-semibold">{DOCK_TITLES[panel]}</span>
        {projectName === null ? null : (
          <span className="text-muted-foreground truncate text-sm">{projectName}</span>
        )}
        <div className="flex-1" />
        <Button variant="ghost" size="sm" onClick={close}>
          <PanelBottomClose data-icon="inline-start" />
          Back to the editor
        </Button>
      </AppBar>
      {projectName === null ? <LoadingPanel className="flex-1" /> : <PanelBody panel={panel} />}
    </div>
  );
}

export const Route = createFileRoute("/panel/$name")({
  params: {
    parse: ({ name }) => {
      if (!isDockPanel(name)) throw new Error(`There's no panel called “${name}”.`);
      return { name };
    },
  },
  validateSearch: z.object({ project: PROJECT_DIR_PARAM }),
  component: PanelRoute,
});
