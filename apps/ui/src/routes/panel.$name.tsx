import { createFileRoute } from "@tanstack/react-router";
import { PanelBottomClose } from "lucide-react";
import { useEffect } from "react";
import { DOCK_TITLES } from "@/app/shell";
import { Button } from "@/components/ui/button";
import { commands, type DockPanelName } from "@/generated/bindings";
import { AppBar } from "../app/app-bar";
import { LoadingPanel } from "../app/empty-panel";
import { onMenuCommand, PROJECT_SWITCH_COMMANDS } from "../app/menu-commands";
import { forwardMenuCommand, useWindowSync } from "../app/sync";
import { DockContent } from "../editor/dock";
import { FilmProvider } from "../editor/film";
import { isDockPanel } from "../editor/panels";
import { CompiledContext, useCompiled } from "../editor/use-compiled";
import { useFileChanges } from "../editor/use-file-changes";
import { useWorkflowStore } from "../store/workflow";

const close = () => void commands.windowRole("close");

function PanelBody({ panel }: { panel: DockPanelName }) {
  const compiled = useCompiled({ drive: false });
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
  const projectName = useWorkflowStore((s) => s.workflow?.name ?? null);
  useWindowSync();
  useFileChanges();

  useEffect(() => {
    if (projectName !== null)
      void commands.setTitle(`${DOCK_TITLES[panel]} · ${projectName} — Opsis`);
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
  component: PanelRoute,
});
