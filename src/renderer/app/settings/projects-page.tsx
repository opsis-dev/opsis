import { RotateCcw } from "lucide-react";
import { Button } from "@/renderer/components/ui/button";
import { Field, FieldDescription, FieldGroup, FieldLabel } from "@/renderer/components/ui/field";
import type { AppSettings } from "@/shared/settings";
import { patchAppSettings } from "../../store/app-settings";
import { FolderField } from "../folder-field";
import { useProjectLocations } from "../use-loaded";
import { useDraft } from "./fields";
import { SettingsCard } from "./settings-card";

function useDefaultFolder(custom: string): string {
  const locations = useProjectLocations(custom);
  return custom ? "" : (locations?.defaultDir ?? "");
}

export function ProjectsPage({ projects }: { projects: AppSettings["projects"] }) {
  const custom = projects.defaultDir;
  const fallback = useDefaultFolder(custom);
  const folder = useDraft(custom || fallback, (dir) => {
    void patchAppSettings({ projects: { defaultDir: dir.trim() === fallback ? "" : dir.trim() } });
  });

  return (
    <SettingsCard
      title="Default projects folder"
      description="New projects are suggested here, and Open project starts here. If you create a project somewhere else, that place is suggested next time."
    >
      <FieldGroup>
        <Field>
          <FieldLabel htmlFor="projects-dir">Folder</FieldLabel>
          <FolderField
            id="projects-dir"
            value={folder.text}
            title="Choose the default projects folder"
            onChange={folder.change}
            onPicked={folder.commitText}
            onCommit={folder.commit}
          />
          <FieldDescription>
            {custom
              ? "Your own folder."
              : "The default, in your home folder. It's created with your first project."}
          </FieldDescription>
        </Field>
        {custom ? (
          <Button
            variant="outline"
            size="sm"
            className="self-start"
            onClick={() => void patchAppSettings({ projects: { defaultDir: "" } })}
          >
            <RotateCcw data-icon="inline-start" />
            Use the default folder
          </Button>
        ) : null}
      </FieldGroup>
    </SettingsCard>
  );
}
