import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import { Spinner } from "@/renderer/components/ui/spinner";
import { Editor } from "../editor/editor";
import { whenSaved } from "../editor/save";
import { loadViewState } from "../editor/view-state";
import { useLiveStore } from "../store/live";
import { useRunStore } from "../store/run";
import { useSecretsStore } from "../store/secrets";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { errorMessage } from "@/shared/errors";

function EditorRoute() {
  const { dir } = Route.useSearch();
  const navigate = useNavigate();
  const loadedDir = useWorkflowStore((s) => (s.workflow ? s.projectDir : null));
  const [error, setError] = useState(false);
  const opened = useRef(false);

  useEffect(() => {
    if (loadedDir === dir && opened.current) return;
    opened.current = true;
    let cancelled = false;
    let done = false;
    whenSaved()
      .then(() => window.demoflow.openProject(dir))
      .then((p) => {
        if (cancelled || !p) return;
        done = true;
        loadViewState(p.dir);
        useWorkflowStore.getState().load(p.dir, p.workflow, p.version);
        useRunStore.getState().seed(p.lastRun);
        useUiStore.getState().setSelected([]);
        useUiStore.getState().setReplay(null);
        useLiveStore.getState().reset();
        void useSecretsStore.getState().refresh(p.dir);
        void window.demoflow.runStatus().then((s) => {
          if (s.runId && s.projectDir === p.dir) useRunStore.getState().resume(s.progress);
        });
      })
      .catch((e: unknown) => {
        setError(true);
        toast.error("Could not open the project", {
          description: errorMessage(e),
        });
        void navigate({ to: "/" });
      });
    return () => {
      cancelled = true;
      if (!done) opened.current = false;
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
