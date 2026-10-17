import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { z } from "zod";
import { Spinner } from "@/renderer/components/ui/spinner";
import { Editor } from "../editor/editor";
import { useOpenProject } from "../app/open-project";

function EditorRoute() {
  const { dir } = Route.useSearch();
  const navigate = useNavigate();
  const status = useOpenProject(dir, () => void navigate({ to: "/" }));

  if (status !== "open") {
    return (
      <div className="grid h-screen place-items-center">
        {status === "failed" ? null : <Spinner />}
      </div>
    );
  }
  return <Editor key={dir} projectDir={dir} />;
}

export const Route = createFileRoute("/editor")({
  validateSearch: z.object({ dir: z.coerce.string().default("") }),
  component: EditorRoute,
});
