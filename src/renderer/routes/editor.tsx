import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { Spinner } from "@/renderer/components/ui/spinner";
import { Editor } from "../editor/editor";
import { loadViewState } from "../editor/view-state";
import { useLiveStore } from "../store/live";
import { useRunStore } from "../store/run";
import { useSecretsStore } from "../store/secrets";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";

function EditorRoute() {
  const { dir } = Route.useSearch();
  const navigate = useNavigate();
  const loadedDir = useWorkflowStore((s) => (s.workflow ? s.projectDir : null));
  const [error, setError] = useState(false);

  useEffect(() => {
    if (loadedDir === dir) return;
    let cancelled = false;
    window.demoflow
      .openProject(dir)
      .then((p) => {
        if (cancelled || !p) return;
        loadViewState(p.dir);
        useWorkflowStore.getState().load(p.dir, p.workflow);
        useRunStore.getState().seed(p.lastRun);
        useUiStore.getState().setSelected([]);
        useUiStore.getState().setReplay(null);
        useLiveStore.getState().reset();
        void useSecretsStore.getState().refresh(p.dir);
        void window.demoflow.runStatus().then((s) => {
          if (s.runId && s.projectDir === p.dir) useRunStore.getState().start(s.runId);
        });
      })
      .catch((e: unknown) => {
        setError(true);
        toast.error("Could not open the project", {
          description: e instanceof Error ? e.message : String(e),
        });
        void navigate({ to: "/" });
      });
    return () => {
      cancelled = true;
    };
  }, [dir, loadedDir, navigate]);

  if (loadedDir !== dir) {
    return <div className="grid h-screen place-items-center">{error ? null : <Spinner />}</div>;
  }
  return <Editor key={dir} />;
}

export const Route = createFileRoute("/editor")({
  validateSearch: (search: Record<string, unknown>) => ({ dir: String(search.dir ?? "") }),
  component: EditorRoute,
});
