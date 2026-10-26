import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Field, FieldDescription, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { commands, type TemplateInfo } from "@/generated/bindings";
import { whenFirstOneOf } from "@/lib/one-of";
import { useBusy } from "@/lib/use-busy";
import { getTemplate, templates } from "@/store/catalog";
import { FolderField } from "./folder-field";
import { useProjectLocations } from "./use-loaded";

type Template = TemplateInfo;

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
          onValueChange={whenFirstOneOf(
            templates().map((t) => t.id),
            onChange,
          )}
          className="w-full"
        >
          {templates().map((t) => (
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
          {chosen ? <TemplateSummary template={chosen} /> : null}
        </div>
      )}
    </Field>
  );
}

function useProjectFolder(parent: string, name: string): string {
  const [folder, setFolder] = useState({ for: "", path: "" });
  const wanted = `${parent}\n${name}`;
  useEffect(() => {
    if (!parent) return;
    let current = true;
    void commands
      .projectFolder(parent, name)
      .then((path) => {
        if (current) setFolder({ for: `${parent}\n${name}`, path });
      })
      .catch(() => undefined);
    return () => {
      current = false;
    };
  }, [parent, name]);
  return parent && folder.for === wanted ? folder.path : "";
}

function useNewProject(initialTemplate: string, onCreated: (dir: string) => void) {
  const [name, setName] = useState(DEFAULT_PROJECT_NAME);
  const [template, setTemplate] = useState(initialTemplate);
  const [typedLocation, setTypedLocation] = useState<string | null>(null);
  const { isBusy, run } = useBusy();
  const locations = useProjectLocations();
  const location = typedLocation ?? locations?.suggestedDir ?? "";
  const dir = location.trim();
  const folder = useProjectFolder(dir, name);

  const create = () =>
    run(
      "create",
      async () => {
        const p = await commands.createProject(name, template, dir);
        if (p) onCreated(p.dir);
      },
      "Couldn't create the project",
    );

  return {
    name,
    setName,
    template,
    setTemplate,
    location,
    setTypedLocation,
    folder,
    ready: name.trim() !== "" && dir !== "" && !isBusy,
    create,
  };
}

type FormProps = {
  template: string;
  fixedTemplate: boolean;
  onCreated: (dir: string) => void;
  onCancel: () => void;
};

function NewProjectForm({
  template: initialTemplate,
  fixedTemplate,
  onCreated,
  onCancel,
}: FormProps) {
  const {
    name,
    setName,
    template,
    setTemplate,
    location,
    setTypedLocation,
    folder,
    ready,
    create,
  } = useNewProject(initialTemplate, onCreated);

  return (
    <>
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
        <Button variant="outline" onClick={onCancel}>
          Cancel
        </Button>
        <Button onClick={() => void create()} disabled={!ready}>
          Create
        </Button>
      </DialogFooter>
    </>
  );
}

export function NewProjectDialog({
  open,
  onOpenChange,
  ...form
}: Omit<FormProps, "onCancel"> & { open: boolean; onOpenChange: (open: boolean) => void }) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>New project</DialogTitle>
          <DialogDescription>
            Each project is a folder with your flow, files and finished videos.
          </DialogDescription>
        </DialogHeader>
        <NewProjectForm {...form} onCancel={() => onOpenChange(false)} />
      </DialogContent>
    </Dialog>
  );
}
