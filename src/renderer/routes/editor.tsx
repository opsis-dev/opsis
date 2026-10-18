import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { z } from "zod";
import { LoadingPanel } from "../app/empty-panel";
import { PROJECT_DIR_PARAM, useOpenProject } from "../app/open-project";
import { Editor } from "../editor/editor";

function EditorRoute() {
  const { dir } = Route.useSearch();
  const navigate = useNavigate();
  const status = useOpenProject(dir, () => void navigate({ to: "/" }));

  if (status === "failed") return null;
  if (status === "opening") return <LoadingPanel className="h-screen" />;
  return <Editor key={dir} projectDir={dir} />;
}

export const Route = createFileRoute("/editor")({
  validateSearch: z.object({ dir: PROJECT_DIR_PARAM }),
  component: EditorRoute,
});
