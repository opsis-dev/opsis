import { useState } from "react";
import { Button } from "@/renderer/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/renderer/components/ui/dialog";
import { Field, FieldDescription, FieldGroup, FieldLabel } from "@/renderer/components/ui/field";
import { Input } from "@/renderer/components/ui/input";
import { ToggleGroup, ToggleGroupItem } from "@/renderer/components/ui/toggle-group";
import { platform } from "@/renderer/lib/platform";
import { useBusy } from "@/renderer/lib/use-busy";
import { joinFolder, projectFolderName } from "@/shared/project-folder";
import { getTemplate, TEMPLATES } from "@/shared/templates";
import { FolderField } from "./folder-field";
import { useProjectLocations } from "./use-loaded";

type Template = (typeof TEMPLATES)[number];

const DEFAULT_PROJECT_NAME = "My launch video";

function TemplateSummary({ template }: { template: Template }) {
  return (
    <>
      <span className="font-medium">{template.title}</span>
      <span className="text-muted-foreground text-xs font-normal whitespace-normal">
        {template.description}
      </span>
    </>
  );
}

function TemplateField({
  value,
  onChange,
  fixed,
}: {
  value: string;
  onChange: (id: string) => void;
  fixed: boolean;
}) {
  const [choosing, setChoosing] = useState(!fixed);
  const chosen = getTemplate(value);
  return (
    <Field>
      <div className="flex items-center justify-between">
        <FieldLabel>Template</FieldLabel>
        {choosing ? null : (
          <Button variant="link" size="sm" className="h-auto p-0" onClick={() => setChoosing(true)}>
            Change
          </Button>
        )}
      </div>
      {choosing ? (
        <ToggleGroup
          orientation="vertical"
          variant="outline"
          value={[value]}
          onValueChange={(v) => v[0] && onChange(v[0])}
          className="w-full"
        >
          {TEMPLATES.map((t) => (
            <ToggleGroupItem
              key={t.id}
              value={t.id}
              className="h-auto flex-col items-start gap-0.5 py-2 text-left"
            >
              <TemplateSummary template={t} />
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      ) : (
        <div className="bg-muted/40 flex flex-col gap-0.5 rounded-md border px-3 py-2 text-sm">
          <TemplateSummary template={chosen} />
        </div>
      )}
    </Field>
  );
}

function useNewProject(initialTemplate: string, onCreated: (dir: string) => void) {
  const [name, setName] = useState(DEFAULT_PROJECT_NAME);
  const [template, setTemplate] = useState(initialTemplate);
  const [typedLocation, setTypedLocation] = useState<string | null>(null);
  const { isBusy, run } = useBusy();
  const locations = useProjectLocations();
  const location = typedLocation ?? locations?.suggestedDir ?? "";
  const folder = location.trim()
    ? joinFolder(location.trim(), projectFolderName(name), platform)
    : "";

  const create = () =>
    run(
      "create",
      async () => {
        const p = await window.demoflow.createProject({ name, template, dir: location.trim() });
        if (p) onCreated(p.dir);
      },
      "Could not create the project",
    );

  return {
    name,
    setName,
    template,
    setTemplate,
    location,
    setTypedLocation,
    folder,
    busy: isBusy,
    create,
  };
}

export function NewProjectDialog({
  open,
  onOpenChange,
  template: initialTemplate,
  fixedTemplate,
  onCreated,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  template: string;
  fixedTemplate: boolean;
  onCreated: (dir: string) => void;
}) {
  const { name, setName, template, setTemplate, location, setTypedLocation, folder, busy, create } =
    useNewProject(initialTemplate, onCreated);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>New project</DialogTitle>
          <DialogDescription>
            Each project is a folder with your flow, files and finished videos.
          </DialogDescription>
        </DialogHeader>
        <FieldGroup>
          <Field>
            <FieldLabel htmlFor="project-name">Name</FieldLabel>
            <Input
              id="project-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              autoFocus
            />
          </Field>
          <Field>
            <FieldLabel htmlFor="project-location">Location</FieldLabel>
            <FolderField
              id="project-location"
              value={location}
              title="Choose where to create the project"
              onChange={setTypedLocation}
              onPicked={setTypedLocation}
            />
            <FieldDescription className="break-all">
              {folder ? `Creates ${folder}` : "Choose where the project folder goes."}
            </FieldDescription>
          </Field>
          <TemplateField value={template} onChange={setTemplate} fixed={fixedTemplate} />
        </FieldGroup>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={() => void create()} disabled={!name.trim() || !location.trim() || busy}>
            Create
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
