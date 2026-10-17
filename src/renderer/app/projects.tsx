import { useNavigate, useSearch } from "@tanstack/react-router";
import { Download, FolderOpen, Plus, Settings, type LucideIcon } from "lucide-react";
import { useEffect, useEffectEvent, useState, type ReactNode } from "react";
import { toast } from "sonner";
import type { RecentProject } from "@/shared/ipc";
import { acceleratorLabel } from "@/shared/menu";
import { TEMPLATES } from "@/shared/templates";
import { errorMessage } from "@/shared/errors";
import { formatTimeAgo } from "@/shared/format";
import { Button } from "@/renderer/components/ui/button";
import { Alert, AlertDescription, AlertTitle } from "@/renderer/components/ui/alert";
import { Kbd } from "@/renderer/components/ui/kbd";
import { platform } from "@/renderer/lib/platform";
import { NodeIcon } from "../editor/icons";
import { useFreshEnvironment } from "../store/environment";
import { PageBar } from "./app-bar";
import { Stagger, StaggerItem } from "./motion";
import { NewProjectDialog } from "./new-project-dialog";
import { useLoaded } from "./use-loaded";

const RECENT_LIMIT = 10;

type Action = { icon: LucideIcon; label: string; keys: string; run: () => void };

function SectionTitle({ children }: { children: ReactNode }) {
  return (
    <h2 className="text-muted-foreground mb-2 text-xs font-medium tracking-wide uppercase">
      {children}
    </h2>
  );
}

function ActionList({ actions }: { actions: Action[] }) {
  return (
    <section className="flex flex-col gap-1">
      <SectionTitle>Start</SectionTitle>
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
  );
}

function RecentList({
  recent,
  onOpen,
}: {
  recent: RecentProject[];
  onOpen: (dir: string) => void;
}) {
  return (
    <section className="flex min-w-0 flex-col gap-1">
      <SectionTitle>Recent</SectionTitle>
      {recent.length === 0 ? (
        <p className="text-muted-foreground text-sm">
          No projects yet. Start from a template to see a recording turn into a video.
        </p>
      ) : (
        <Stagger className="flex flex-col">
          {recent.slice(0, RECENT_LIMIT).map((p) => (
            <StaggerItem key={p.dir}>
              <button
                type="button"
                onClick={() => onOpen(p.dir)}
                title={p.dir}
                className="hover:bg-accent -mx-2 flex w-full min-w-0 items-baseline gap-3 rounded-md px-2 py-1.5 text-left text-sm"
              >
                <span className="shrink-0 font-medium">{p.name}</span>
                <span className="text-muted-foreground min-w-0 flex-1 truncate text-xs [direction:rtl]">
                  <span dir="ltr">{p.dir}</span>
                </span>
                <span className="text-muted-foreground shrink-0 text-xs">
                  {formatTimeAgo(p.openedAt)}
                </span>
              </button>
            </StaggerItem>
          ))}
        </Stagger>
      )}
    </section>
  );
}

function EngineAlert() {
  return (
    <Alert>
      <Download />
      <AlertTitle>Set up the recording engine</AlertTitle>
      <AlertDescription className="flex flex-col items-start gap-2">
        <span>Editing and preview work now. Recording and exporting need the engine.</span>
        <Button
          size="sm"
          variant="outline"
          onClick={() => void window.demoflow.openSettings("engine")}
        >
          Open Settings
        </Button>
      </AlertDescription>
    </Alert>
  );
}

function TemplateGrid({ onPick }: { onPick: (id: string) => void }) {
  return (
    <section className="flex flex-col gap-1">
      <SectionTitle>Start from a template</SectionTitle>
      <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
        {TEMPLATES.map((t) => (
          <button
            key={t.id}
            type="button"
            onClick={() => onPick(t.id)}
            className="group bg-card hover:bg-accent flex items-start gap-3 rounded-lg border p-3 text-left transition-colors"
          >
            <span className="bg-muted text-muted-foreground group-hover:text-foreground grid size-8 shrink-0 place-items-center rounded-md">
              <NodeIcon name={t.icon} className="size-4" />
            </span>
            <span className="flex min-w-0 flex-col gap-0.5">
              <span className="text-sm font-medium">{t.title}</span>
              <span className="text-muted-foreground line-clamp-2 text-xs">{t.description}</span>
            </span>
          </button>
        ))}
      </div>
    </section>
  );
}

type NewDialog = { template: string; fixed: boolean; session: number };

function useProjectActions() {
  const navigate = useNavigate();
  const [dialog, setDialog] = useState<NewDialog>({
    template: "todomvc",
    fixed: false,
    session: 0,
  });
  const [creating, setCreating] = useState(false);

  const go = (dir: string) => void navigate({ to: "/editor", search: { dir } });

  async function open(dir?: string) {
    try {
      const p = await window.demoflow.openProject(dir);
      if (p) go(p.dir);
    } catch (e) {
      toast.error("Could not open the project", { description: errorMessage(e) });
    }
  }

  const startNew = (template?: string) => {
    setDialog((d) => ({
      template: template ?? d.template,
      fixed: template !== undefined,
      session: d.session + 1,
    }));
    setCreating(true);
  };

  return { dialog, creating, setCreating, go, open, startNew };
}

function useProjectCommands(startNew: () => void, open: () => Promise<void>): void {
  const navigate = useNavigate();
  const { action } = useSearch({ from: "/" });
  const onNew = useEffectEvent(startNew);
  const onOpen = useEffectEvent(() => void open());

  useEffect(() => {
    if (!action) return;
    void navigate({ to: "/", search: {}, replace: true });
    if (action === "new") onNew();
    else onOpen();
  }, [action, navigate]);

  useEffect(
    () =>
      window.demoflow.onMenuCommand((c) => {
        if (c === "project:new") onNew();
        if (c === "project:open") onOpen();
      }),
    [],
  );
}

export function Projects() {
  const recent = useLoaded(
    () => window.demoflow.listRecentProjects(),
    "Could not read the recent projects",
  );
  const env = useFreshEnvironment();
  const { dialog, creating, setCreating, go, open, startNew } = useProjectActions();
  useProjectCommands(startNew, open);

  useEffect(() => window.demoflow.setTitle("Demoflow"), []);

  const actions: Action[] = [
    { icon: Plus, label: "New project…", keys: "CmdOrCtrl+N", run: () => startNew() },
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
              <ActionList actions={actions} />
              <RecentList recent={recent ?? []} onOpen={(dir) => void open(dir)} />
            </div>
            <div className="flex min-w-0 flex-col gap-10">
              {env && !env.engine.complete ? <EngineAlert /> : null}
              <TemplateGrid onPick={startNew} />
            </div>
          </div>
          <NewProjectDialog
            key={dialog.session}
            open={creating}
            onOpenChange={setCreating}
            template={dialog.template}
            fixedTemplate={dialog.fixed}
            onCreated={go}
          />
        </div>
      </div>
    </div>
  );
}
