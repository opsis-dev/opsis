import { createFileRoute } from "@tanstack/react-router";
import { useEffect } from "react";
import { useDocumentMirror } from "@/app/document-mirror";
import { useMenuShortcuts } from "@/app/menu-shortcuts";
import { commands } from "@/generated/bindings";
import { AppBar } from "../app/app-bar";
import { LoadingPanel } from "../app/empty-panel";
import { onMenuCommand, PROJECT_SWITCH_COMMANDS } from "../app/menu-commands";
import { forwardMenuCommand, useWindowSync } from "../app/sync";
import { FilmProvider } from "../editor/film";
import { PreviewPlayer } from "../editor/preview-player";
import { CompiledContext, useCompiled } from "../editor/use-compiled";
import { useFileChanges } from "../editor/use-file-changes";
import { useWorkflowStore } from "../store/workflow";

function PreviewBody() {
  const compiled = useCompiled({ drive: false });
  return (
    <CompiledContext.Provider value={compiled}>
      <FilmProvider>
        <section aria-label="Preview" className="bg-sidebar min-h-0 flex-1">
          <PreviewPlayer />
        </section>
      </FilmProvider>
    </CompiledContext.Provider>
  );
}

function PreviewRoute() {
  const projectName = useWorkflowStore((s) => s.workflow?.name ?? null);
  useWindowSync();
  useDocumentMirror();
  useMenuShortcuts();
  useFileChanges();

  useEffect(() => {
    if (projectName !== null) void commands.setTitle(`Preview · ${projectName} — Opsis`);
  }, [projectName]);

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
        <span className="truncate font-semibold">Preview</span>
        {projectName === null ? null : (
          <span className="text-muted-foreground truncate text-sm">{projectName}</span>
        )}
      </AppBar>
      {projectName === null ? <LoadingPanel className="flex-1" /> : <PreviewBody />}
    </div>
  );
}

export const Route = createFileRoute("/preview")({ component: PreviewRoute });
