import { useState } from "react";
import { toast } from "sonner";
import { joinFolder, projectFolderName } from "@/shared/project-folder";
import { TEMPLATES } from "@/shared/templates";
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
import { errorMessage } from "@/shared/errors";
import { platform } from "@/renderer/lib/platform";
import { FolderField } from "./folder-field";
import { useLoaded } from "./use-loaded";

type Template = (typeof TEMPLATES)[number];

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
  const chosen = TEMPLATES.find((t) => t.id === value) ?? TEMPLATES[0];
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
  const [name, setName] = useState("My launch video");
  const [template, setTemplate] = useState(initialTemplate);
  const [typed, setLocation] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const locations = useLoaded(
    () => window.demoflow.projectLocations(),
    "Could not find the projects folder",
  );
  const location = typed ?? locations?.suggestedDir ?? "";

  const folder = location.trim()
    ? joinFolder(location.trim(), projectFolderName(name), platform)
    : "";

  async function create() {
    setBusy(true);
    try {
      const p = await window.demoflow.createProject({ name, template, dir: location.trim() });
      if (p) onCreated(p.dir);
    } catch (e) {
      toast.error("Could not create the project", {
        description: errorMessage(e),
      });
    } finally {
      setBusy(false);
    }
  }

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
              onChange={setLocation}
              onPicked={setLocation}
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
