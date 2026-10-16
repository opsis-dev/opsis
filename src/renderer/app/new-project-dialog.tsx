import { FolderOpen } from "lucide-react";
import { useEffect, useState } from "react";
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
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from "@/renderer/components/ui/input-group";
import { ToggleGroup, ToggleGroupItem } from "@/renderer/components/ui/toggle-group";
import { errorMessage } from "@/shared/errors";

const platform = window.demoflow?.platform ?? "linux";

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
  const [choosing, setChoosing] = useState(!fixedTemplate);
  const [location, setLocation] = useState("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    void window.demoflow.projectLocations().then((l) => setLocation(l.suggestedDir));
  }, []);

  const chosen = TEMPLATES.find((t) => t.id === template) ?? TEMPLATES[0];
  const folder = location.trim()
    ? joinFolder(location.trim(), projectFolderName(name), platform)
    : "";

  async function browse() {
    const picked = await window.demoflow.chooseFolder({
      title: "Choose where to create the project",
      defaultPath: location.trim() || undefined,
    });
    if (picked) setLocation(picked);
  }

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
            <InputGroup>
              <InputGroupInput
                id="project-location"
                value={location}
                spellCheck={false}
                onChange={(e) => setLocation(e.target.value)}
              />
              <InputGroupAddon align="inline-end">
                <InputGroupButton size="xs" onClick={() => void browse()}>
                  <FolderOpen data-icon="inline-start" />
                  Browse
                </InputGroupButton>
              </InputGroupAddon>
            </InputGroup>
            <FieldDescription className="break-all">
              {folder ? `Creates ${folder}` : "Choose where the project folder goes."}
            </FieldDescription>
          </Field>
          <Field>
            <div className="flex items-center justify-between">
              <FieldLabel>Template</FieldLabel>
              {choosing ? null : (
                <Button
                  variant="link"
                  size="sm"
                  className="h-auto p-0"
                  onClick={() => setChoosing(true)}
                >
                  Change
                </Button>
              )}
            </div>
            {choosing ? (
              <ToggleGroup
                orientation="vertical"
                variant="outline"
                value={[template]}
                onValueChange={(v) => v[0] && setTemplate(v[0])}
                className="w-full"
              >
                {TEMPLATES.map((t) => (
                  <ToggleGroupItem
                    key={t.id}
                    value={t.id}
                    className="h-auto flex-col items-start gap-0.5 py-2 text-left"
                  >
                    <span className="font-medium">{t.title}</span>
                    <span className="text-muted-foreground text-xs font-normal whitespace-normal">
                      {t.description}
                    </span>
                  </ToggleGroupItem>
                ))}
              </ToggleGroup>
            ) : (
              <div className="bg-muted/40 flex flex-col gap-0.5 rounded-md border px-3 py-2 text-sm">
                <span className="font-medium">{chosen.title}</span>
                <span className="text-muted-foreground text-xs">{chosen.description}</span>
              </div>
            )}
          </Field>
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
