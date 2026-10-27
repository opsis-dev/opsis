import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { z } from "zod";
import { useDocumentMirror } from "@/app/document-mirror";
import { LoadingPanel } from "@/app/empty-panel";
import { PROJECT_DIR_PARAM, useOpenProject } from "@/app/open-project";
import { useRunMirror } from "@/app/run-mirror";
import { Editor } from "@/editor/editor";

function EditorRoute() {
  const { dir } = Route.useSearch();
  const navigate = useNavigate();
  useDocumentMirror();
  useRunMirror();
  const status = useOpenProject(dir, () => void navigate({ to: "/" }));

  if (status === "failed") return null;
  if (status === "opening") return <LoadingPanel className="h-screen" />;
  return <Editor key={dir} projectDir={dir} />;
}

export const Route = createFileRoute("/_main/editor")({
  validateSearch: z.object({ dir: PROJECT_DIR_PARAM }),
  component: EditorRoute,
});
