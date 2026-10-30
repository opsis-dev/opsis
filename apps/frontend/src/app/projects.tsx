import { formatTimeAgo } from "@opsis/video/common/format";
import { useNavigate } from "@tanstack/react-router";
import { Download, FolderOpen, Plus, Settings, type LucideIcon } from "lucide-react";
import { useEffect, useEffectEvent, useState, type ReactNode } from "react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Kbd } from "@/components/ui/kbd";
import { commands, type RecentProject, type MenuCommand } from "@/generated/bindings";
import { templates } from "@/store/catalog";
import { NodeIcon } from "../editor/node-icon";
import { needsEngine, useFreshEnvironment } from "../store/environment";
import { AppBar } from "./app-bar";
import { dispatchMenuCommand, usePendingProjectAction, type ProjectAction } from "./menu-commands";
import { commandLabel, commandShortcut } from "./menu-shortcuts";
import { Stagger, StaggerItem } from "./motion";
import { NewProjectDialog } from "./new-project-dialog";
import { openEditor } from "./open-project";
import { useLoaded } from "./use-loaded";

const RECENT_LIMIT = 10;

const DEFAULT_TEMPLATE = "todomvc";

const START_ACTIONS: Array<{ icon: LucideIcon; command: MenuCommand }> = [
  { icon: Plus, command: "project:new" },
  { icon: FolderOpen, command: "project:open" },
  { icon: Settings, command: "view:settings" },
];

function SectionTitle({ children }: { children: ReactNode }) {
  return (
    <h2 className="text-muted-foreground mb-2 text-xs font-medium tracking-wide uppercase">
      {children}
    </h2>
  );
}

function ActionList() {
  return (
    <section className="flex flex-col gap-1">
      <SectionTitle>Start</SectionTitle>
      {START_ACTIONS.map(({ icon: Icon, command }) => {
        const keys = commandShortcut(command);
        return (
          <button
            key={command}
            type="button"
            onClick={() => dispatchMenuCommand(command)}
            className="group hover:bg-accent -mx-2 flex items-center gap-3 rounded-md px-2 py-1.5 text-left text-sm"
          >
            <Icon className="text-muted-foreground group-hover:text-foreground size-4" />
            <span className="flex-1">{commandLabel(command)}</span>
            {keys ? <Kbd>{keys}</Kbd> : null}
          </button>
        );
      })}
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
        <span>Editing works now. Recording and exporting need the engine.</span>
        <Button size="sm" variant="outline" onClick={() => void commands.openSettings("engine")}>
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
        {templates().map((t) => (
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

type NewDialog = { template: string; fixed: boolean };

function useProjectActions() {
  const navigate = useNavigate();
  const [dialog, setDialog] = useState<NewDialog>({ template: DEFAULT_TEMPLATE, fixed: false });
  const [creating, setCreating] = useState(false);

  const go = (dir: string) => openEditor(navigate, dir);

  async function open(dir?: string) {
    const target = dir ?? (await commands.pickProjectDir());
    if (target) go(target);
  }

  const startNew = (template?: string) => {
    setDialog((d) => ({ template: template ?? d.template, fixed: template !== undefined }));
    setCreating(true);
  };

  return { dialog, creating, setCreating, go, open, startNew };
}

function useProjectCommands(startNew: () => void, open: () => Promise<void>): void {
  const pending = usePendingProjectAction((s) => s.pending);
  const run = useEffectEvent((next: ProjectAction) => {
    if (next === "new") startNew();
    else void open();
  });

  useEffect(() => {
    if (!pending) return;
    usePendingProjectAction.setState({ pending: null });
    run(pending);
  }, [pending]);
}

export function Projects() {
  const recent = useLoaded(
    () => commands.listRecentProjects(),
    "Couldn't read the recent projects",
  );
  const env = useFreshEnvironment();
  const { dialog, creating, setCreating, go, open, startNew } = useProjectActions();
  useProjectCommands(startNew, open);

  useEffect(() => void commands.setTitle("Opsis"), []);

  return (
    <div className="flex h-screen flex-col">
      <AppBar />
      <div className="min-h-0 flex-1 overflow-y-auto">
        <div className="flex min-h-full items-center justify-center px-10 py-14">
          <div className="grid w-full max-w-5xl grid-cols-1 gap-12 md:grid-cols-[minmax(0,1fr)_minmax(0,1.15fr)]">
            <div className="flex min-w-0 flex-col gap-10">
              <ActionList />
              <RecentList recent={recent ?? []} onOpen={(dir) => void open(dir)} />
            </div>
            <div className="flex min-w-0 flex-col gap-10">
              {env && needsEngine(env.engine) ? <EngineAlert /> : null}
              <TemplateGrid onPick={startNew} />
            </div>
          </div>
          <NewProjectDialog
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
