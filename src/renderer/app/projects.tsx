import { useNavigate } from "@tanstack/react-router";
import { useUiStore } from "../store/ui";
import { PageBar } from "./app-bar";
import { Stagger, StaggerItem } from "./motion";
import {
  Columns2,
  Download,
  File,
  FolderOpen,
  ListChecks,
  Map as MapIcon,
  Plus,
  Rocket,
  ScrollText,
  Settings,
  Smartphone,
} from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import type { RecentProject } from "@/shared/ipc";
import { acceleratorLabel } from "@/shared/menu";
import { TEMPLATES } from "@/shared/templates";
import { Button } from "@/renderer/components/ui/button";
import { Alert, AlertDescription, AlertTitle } from "@/renderer/components/ui/alert";
import { Kbd } from "@/renderer/components/ui/kbd";
import { NewProjectDialog } from "./new-project-dialog";
import { errorMessage } from "@/shared/errors";

export function Projects() {
  const navigate = useNavigate();
  const [recent, setRecent] = useState<RecentProject[]>([]);
  const [creating, setCreating] = useState(false);
  const [template, setTemplate] = useState("todomvc");
  const [fixedTemplate, setFixedTemplate] = useState(false);
  const [session, setSession] = useState(0);
  const [engineMissing, setEngineMissing] = useState(false);

  useEffect(() => {
    void window.demoflow.listRecentProjects().then(setRecent);
    void window.demoflow.getEnvironment().then((env) => setEngineMissing(!env.engine.complete));
    window.demoflow.setTitle("Demoflow");
    const pending = useUiStore.getState().pendingProjectAction;
    useUiStore.getState().setPendingProjectAction(null);
    if (pending === "new") newProject();
    if (pending === "open") void open();
    return window.demoflow.onMenuCommand((c) => {
      if (c === "project:new") newProject();
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
        description: errorMessage(e),
      });
    }
  }

  function newProject() {
    setFixedTemplate(false);
    setSession((n) => n + 1);
    setCreating(true);
  }

  const startTemplate = (id: string) => {
    setTemplate(id);
    setFixedTemplate(true);
    setSession((n) => n + 1);
    setCreating(true);
  };
  const platform = window.demoflow.platform;
  const actions: Array<{
    icon: typeof Plus;
    label: string;
    keys: string;
    run: () => void;
  }> = [
    { icon: Plus, label: "New project…", keys: "CmdOrCtrl+N", run: newProject },
    { icon: FolderOpen, label: "Open project…", keys: "CmdOrCtrl+O", run: () => void open() },
    {
      icon: Settings,
      label: "Settings",
      keys: "CmdOrCtrl+,",
      run: () => void window.demoflow.openSettings(),
    },
  ];

  return (
    <div className="flex h-screen flex-col">
      <PageBar title="" />
      <div className="min-h-0 flex-1 overflow-y-auto">
        <div className="flex min-h-full items-center justify-center px-10 py-14">
          <div className="grid w-full max-w-5xl grid-cols-1 gap-12 md:grid-cols-[minmax(0,1fr)_minmax(0,1.15fr)]">
            <div className="flex min-w-0 flex-col gap-10">
              <section className="flex flex-col gap-1">
                <h2 className="text-muted-foreground mb-2 text-xs font-medium tracking-wide uppercase">
                  Start
                </h2>
                {actions.map((a) => (
                  <button
                    key={a.label}
                    type="button"
                    onClick={a.run}
                    className="group hover:bg-accent -mx-2 flex items-center gap-3 rounded-md px-2 py-1.5 text-left text-sm"
                  >
                    <a.icon className="text-muted-foreground group-hover:text-foreground size-4" />
                    <span className="flex-1">{a.label}</span>
                    <Kbd>{acceleratorLabel(a.keys, platform)}</Kbd>
                  </button>
                ))}
              </section>

              <section className="flex min-w-0 flex-col gap-1">
                <h2 className="text-muted-foreground mb-2 text-xs font-medium tracking-wide uppercase">
                  Recent
                </h2>
                {recent.length === 0 ? (
                  <p className="text-muted-foreground text-sm">
                    No projects yet. Start from a template to see a recording turn into a video.
                  </p>
                ) : (
                  <Stagger className="flex flex-col">
                    {recent.slice(0, 10).map((p) => (
                      <StaggerItem key={p.dir}>
                        <button
                          type="button"
                          onClick={() => void open(p.dir)}
                          title={p.dir}
                          className="hover:bg-accent -mx-2 flex w-full min-w-0 items-baseline gap-3 rounded-md px-2 py-1.5 text-left text-sm"
                        >
                          <span className="shrink-0 font-medium">{p.name}</span>
                          <span className="text-muted-foreground min-w-0 flex-1 truncate text-xs [direction:rtl]">
                            <span dir="ltr">{p.dir}</span>
                          </span>
                          <span className="text-muted-foreground shrink-0 text-xs">
                            {timeAgo(p.openedAt)}
                          </span>
                        </button>
                      </StaggerItem>
                    ))}
                  </Stagger>
                )}
              </section>
            </div>

            <div className="flex min-w-0 flex-col gap-10">
              {engineMissing ? (
                <Alert>
                  <Download />
                  <AlertTitle>Set up the recording engine</AlertTitle>
                  <AlertDescription className="flex flex-col items-start gap-2">
                    <span>
                      Editing and preview work now. Recording and exporting need the engine.
                    </span>
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => void window.demoflow.openSettings("engine")}
                    >
                      Open Settings
                    </Button>
                  </AlertDescription>
                </Alert>
              ) : null}
              <section className="flex flex-col gap-1">
                <h2 className="text-muted-foreground mb-2 text-xs font-medium tracking-wide uppercase">
                  Start from a template
                </h2>
                <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                  {TEMPLATES.map((t) => {
                    const Icon = TEMPLATE_ICONS[t.id] ?? File;
                    return (
                      <button
                        key={t.id}
                        type="button"
                        onClick={() => startTemplate(t.id)}
                        className="group bg-card hover:bg-accent flex items-start gap-3 rounded-lg border p-3 text-left transition-colors"
                      >
                        <span className="bg-muted text-muted-foreground group-hover:text-foreground grid size-8 shrink-0 place-items-center rounded-md">
                          <Icon className="size-4" />
                        </span>
                        <span className="flex min-w-0 flex-col gap-0.5">
                          <span className="text-sm font-medium">{t.title}</span>
                          <span className="text-muted-foreground line-clamp-2 text-xs">
                            {t.description}
                          </span>
                        </span>
                      </button>
                    );
                  })}
                </div>
              </section>
            </div>
          </div>
          <NewProjectDialog
            key={session}
            open={creating}
            onOpenChange={setCreating}
            template={template}
            fixedTemplate={fixedTemplate}
            onCreated={go}
          />
        </div>
      </div>
    </div>
  );
}

const TEMPLATE_ICONS: Record<string, typeof File> = {
  blank: File,
  todomvc: ListChecks,
  "feature-launch": Rocket,
  changelog: ScrollText,
  onboarding: MapIcon,
  "social-teaser": Smartphone,
  "before-after": Columns2,
};

const RELATIVE = new Intl.RelativeTimeFormat(undefined, { numeric: "auto" });

function timeAgo(at: number | string): string {
  const seconds = (new Date(at).getTime() - Date.now()) / 1000;
  const steps: Array<[Intl.RelativeTimeFormatUnit, number]> = [
    ["year", 31_536_000],
    ["month", 2_592_000],
    ["week", 604_800],
    ["day", 86_400],
    ["hour", 3_600],
    ["minute", 60],
  ];
  for (const [unit, size] of steps)
    if (Math.abs(seconds) >= size) return RELATIVE.format(Math.round(seconds / size), unit);
  return "just now";
}
