import { useNavigate } from "@tanstack/react-router";
import { useUiStore } from "../store/ui";
import { PageBar } from "./AppBar";
import { Clapperboard, FolderOpen, Plus, Settings } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import type { RecentProject } from "@/shared/ipc";
import { TEMPLATES } from "@/shared/templates";
import { Button } from "@/renderer/components/ui/button";
import {
  Card,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/renderer/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/renderer/components/ui/dialog";
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/renderer/components/ui/empty";
import { Field, FieldGroup, FieldLabel } from "@/renderer/components/ui/field";
import { Input } from "@/renderer/components/ui/input";
import { ToggleGroup, ToggleGroupItem } from "@/renderer/components/ui/toggle-group";

export function Projects() {
  const navigate = useNavigate();
  const [recent, setRecent] = useState<RecentProject[]>([]);
  const [creating, setCreating] = useState(false);
  const [name, setName] = useState("My launch video");
  const [template, setTemplate] = useState("todomvc");

  useEffect(() => {
    void window.demoflow.listRecentProjects().then(setRecent);
    window.demoflow.setTitle("Demoflow");
    const pending = useUiStore.getState().pendingProjectAction;
    useUiStore.getState().setPendingProjectAction(null);
    if (pending === "new") setCreating(true);
    if (pending === "open") void open();
    return window.demoflow.onMenuCommand((c) => {
      if (c === "project:new") setCreating(true);
      if (c === "project:open") void open();
    });
    // oxlint-disable-next-line react/exhaustive-deps
  }, []);

  const go = (dir: string) => void navigate({ to: "/editor", search: { dir } });

  async function open(dir?: string) {
    try {
      const p = await window.demoflow.openProject(dir);
      if (p) go(p.dir);
    } catch (e) {
      toast.error("Could not open the project", {
        description: e instanceof Error ? e.message : String(e),
      });
    }
  }

  async function create() {
    try {
      const p = await window.demoflow.createProject({ name, template });
      if (p) go(p.dir);
    } catch (e) {
      toast.error("Could not create the project", {
        description: e instanceof Error ? e.message : String(e),
      });
    }
  }

  return (
    <div className="flex h-screen flex-col">
      <PageBar title="" />
      <div className="min-h-0 flex-1 overflow-y-auto">
        <div className="mx-auto flex min-h-full max-w-5xl flex-col gap-8 px-8 py-10">
          <header className="flex items-center gap-3">
            <span className="bg-primary text-primary-foreground grid size-10 place-items-center rounded-xl">
              <Clapperboard />
            </span>
            <div className="flex-1">
              <h1 className="text-2xl font-semibold tracking-tight">Demoflow</h1>
              <p className="text-muted-foreground text-sm">
                Record a real flow. Turn it into a cinematic product demo.
              </p>
            </div>
            <Button
              variant="ghost"
              size="icon"
              aria-label="Settings"
              onClick={() => void navigate({ to: "/settings" })}
            >
              <Settings />
            </Button>
            <Button variant="outline" onClick={() => void open()}>
              <FolderOpen data-icon="inline-start" />
              Open…
            </Button>
            <Button onClick={() => setCreating(true)}>
              <Plus data-icon="inline-start" />
              New project
            </Button>
          </header>

          {recent.length === 0 ? (
            <Empty className="border">
              <EmptyHeader>
                <EmptyMedia variant="icon">
                  <Clapperboard />
                </EmptyMedia>
                <EmptyTitle>No projects yet</EmptyTitle>
                <EmptyDescription>
                  Start from the TodoMVC sample to see a recording turn into a finished video.
                </EmptyDescription>
              </EmptyHeader>
              <EmptyContent>
                <Button onClick={() => setCreating(true)}>
                  <Plus data-icon="inline-start" />
                  New project
                </Button>
              </EmptyContent>
            </Empty>
          ) : (
            <section className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
              {recent.map((p) => (
                <Card
                  key={p.dir}
                  className="hover:border-primary/60 cursor-pointer transition-colors"
                  onClick={() => void open(p.dir)}
                >
                  <CardHeader>
                    <CardTitle className="truncate">{p.name}</CardTitle>
                    <CardDescription className="truncate font-mono text-xs">
                      {p.dir}
                    </CardDescription>
                  </CardHeader>
                  <CardFooter className="text-muted-foreground text-xs">
                    Opened {new Date(p.openedAt).toLocaleString()}
                  </CardFooter>
                </Card>
              ))}
            </section>
          )}

          <Dialog open={creating} onOpenChange={setCreating}>
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
                  <FieldLabel>Template</FieldLabel>
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
                </Field>
              </FieldGroup>
              <DialogFooter>
                <Button variant="outline" onClick={() => setCreating(false)}>
                  Cancel
                </Button>
                <Button onClick={() => void create()} disabled={!name.trim()}>
                  Choose folder & create
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        </div>
      </div>
    </div>
  );
}
