import { RotateCcw } from "lucide-react";
import { useState } from "react";
import {
  KEEP_LAST_TRACES,
  START_OPTION_LABELS,
  START_OPTIONS,
  TRACE_RETENTION,
  TRACE_RETENTION_LABELS,
  type AppSettings,
} from "@/shared/settings";
import { clamp } from "@/shared/math";
import { Button } from "@/renderer/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/renderer/components/ui/card";
import { Field, FieldDescription, FieldGroup, FieldLabel } from "@/renderer/components/ui/field";
import { Input } from "@/renderer/components/ui/input";
import { SingleSlider } from "../single-slider";
import { Spinner } from "@/renderer/components/ui/spinner";
import { Switch } from "@/renderer/components/ui/switch";
import { patchAppSettings } from "../../store/app-settings";
import { useFreshEnvironment } from "../../store/environment";
import { FolderField } from "../folder-field";
import { OptionToggle } from "../option-toggle";
import { useProjectLocations } from "../use-loaded";

const LIVE_FPS = { min: 1, max: 30 };

export function GeneralPage({ settings }: { settings: AppSettings }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Starting and closing</CardTitle>
      </CardHeader>
      <CardContent>
        <FieldGroup>
          <Field>
            <FieldLabel>When Demoflow starts</FieldLabel>
            <OptionToggle
              options={START_OPTIONS}
              labels={START_OPTION_LABELS}
              value={settings.app.onStart}
              onChange={(onStart) => void patchAppSettings({ app: { onStart } })}
            />
          </Field>
          <Field orientation="horizontal" className="justify-between">
            <div className="flex flex-col gap-1">
              <FieldLabel htmlFor="confirm-exit" className="font-normal">
                Ask before closing
              </FieldLabel>
              <FieldDescription>
                When this is off, Demoflow still asks if a run, an engine setup or a save hasn't
                finished.
              </FieldDescription>
            </div>
            <Switch
              id="confirm-exit"
              checked={settings.app.confirmExit}
              onCheckedChange={(v) => void patchAppSettings({ app: { confirmExit: v } })}
            />
          </Field>
        </FieldGroup>
      </CardContent>
    </Card>
  );
}

function useDefaultFolder(custom: string): string {
  const locations = useProjectLocations(custom);
  return custom ? "" : (locations?.defaultDir ?? "");
}

export function ProjectsPage({ settings }: { settings: AppSettings }) {
  const custom = settings.projects.defaultDir;
  const fallback = useDefaultFolder(custom);
  const [draft, setDraft] = useState<string | null>(null);
  const shown = draft ?? (custom || fallback);

  const commit = (dir: string) => {
    setDraft(null);
    void patchAppSettings({ projects: { defaultDir: dir.trim() === fallback ? "" : dir.trim() } });
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Default projects folder</CardTitle>
        <CardDescription>
          New projects are suggested here, and Open project starts here. If you create a project
          somewhere else, that place is suggested next time.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <FieldGroup>
          <Field>
            <FieldLabel htmlFor="projects-dir">Folder</FieldLabel>
            <FolderField
              id="projects-dir"
              value={shown}
              title="Choose the default projects folder"
              onChange={setDraft}
              onPicked={commit}
              onCommit={() => draft !== null && commit(draft)}
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
      </CardContent>
    </Card>
  );
}

function setKeepLast(value: string): void {
  const n = Math.round(Number(value));
  if (n >= KEEP_LAST_TRACES.min)
    void patchAppSettings({
      traces: { keepLast: clamp(n, KEEP_LAST_TRACES.min, KEEP_LAST_TRACES.max) },
    });
}

function TracesCard({ traces }: { traces: AppSettings["traces"] }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Traces</CardTitle>
        <CardDescription>
          A step-by-step record of every take, for finding out what went wrong. Open it from the
          Trace tab in Replay. Stored compressed inside the project.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <FieldGroup>
          <Field>
            <FieldLabel>Keep traces</FieldLabel>
            <OptionToggle
              options={TRACE_RETENTION}
              labels={TRACE_RETENTION_LABELS}
              value={traces.retention}
              onChange={(retention) => void patchAppSettings({ traces: { retention } })}
            />
          </Field>
          {traces.retention === "last-n" ? (
            <Field>
              <FieldLabel htmlFor="keep-last">Runs to keep per node</FieldLabel>
              <Input
                id="keep-last"
                type="number"
                min={KEEP_LAST_TRACES.min}
                max={KEEP_LAST_TRACES.max}
                value={traces.keepLast}
                onChange={(e) => setKeepLast(e.target.value)}
              />
            </Field>
          ) : null}
        </FieldGroup>
      </CardContent>
    </Card>
  );
}

function LiveViewCard({ live }: { live: AppSettings["live"] }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Live view</CardTitle>
        <CardDescription>
          What the recording browser is doing, shown in the editor while a run is going.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <FieldGroup>
          <Field>
            <div className="flex items-center justify-between">
              <FieldLabel>Preview frame rate</FieldLabel>
              <span className="text-muted-foreground text-sm tabular-nums">{live.fps} fps</span>
            </div>
            <SingleSlider
              min={LIVE_FPS.min}
              max={LIVE_FPS.max}
              step={1}
              value={live.fps}
              onChange={(fps) => void patchAppSettings({ live: { fps } })}
            />
            <FieldDescription>
              Only affects the live view; recordings always use the Record node's frame rate.
            </FieldDescription>
          </Field>
          <Field orientation="horizontal" className="justify-between">
            <FieldLabel htmlFor="live-auto" className="font-normal">
              Open the live view when a take starts
            </FieldLabel>
            <Switch
              id="live-auto"
              checked={live.autoOpen}
              onCheckedChange={(v) => void patchAppSettings({ live: { autoOpen: v } })}
            />
          </Field>
        </FieldGroup>
      </CardContent>
    </Card>
  );
}

export function RecordingPage({ settings }: { settings: AppSettings }) {
  return (
    <>
      <TracesCard traces={settings.traces} />
      <LiveViewCard live={settings.live} />
    </>
  );
}

export function ExportPage({ settings }: { settings: AppSettings }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Video export licence</CardTitle>
        <CardDescription>
          Demoflow renders videos with Remotion, which has its own licence. It's free for
          individuals, non-profits and companies with up to 3 employees. Larger companies need a
          Remotion company licence and must enter its key here.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <FieldGroup>
          <Field>
            <FieldLabel htmlFor="remotion-key">Company licence key</FieldLabel>
            <Input
              id="remotion-key"
              type="password"
              autoComplete="off"
              placeholder="Leave empty to use the free licence"
              value={settings.render.licenseKey}
              onChange={(e) => void patchAppSettings({ render: { licenseKey: e.target.value } })}
            />
            <FieldDescription>
              Read the terms at{" "}
              <a
                href="https://www.remotion.dev/license"
                target="_blank"
                rel="noreferrer"
                className="underline underline-offset-2"
              >
                remotion.dev/license
              </a>
              ; company licences are at{" "}
              <a
                href="https://www.remotion.pro"
                target="_blank"
                rel="noreferrer"
                className="underline underline-offset-2"
              >
                remotion.pro
              </a>
              .
            </FieldDescription>
          </Field>
        </FieldGroup>
      </CardContent>
    </Card>
  );
}

export function AboutPage() {
  const env = useFreshEnvironment();
  if (!env) return <Spinner />;
  const rows: Array<[string, string]> = [
    ["App", `${env.versions.app} (${env.edition})`],
    ["Electron", env.versions.electron],
    ["Chromium", env.versions.chrome],
    ["Node", env.versions.node],
    ["Platform", `${env.platform} / ${env.arch}`],
    ["Engine folder", env.engine.dir],
    ["Recording browser", env.engine.headlessShell ?? "Not set up"],
    ["Video tools", env.engine.ffmpeg ?? "Not set up"],
    ["Video inspector", env.engine.ffprobe ?? "Not set up"],
    ["App data", env.userData],
  ];
  return (
    <Card>
      <CardHeader>
        <CardTitle>Demoflow {env.versions.app}</CardTitle>
        <CardDescription>Everything runs on this computer.</CardDescription>
      </CardHeader>
      <CardContent>
        <dl className="grid grid-cols-[10rem_1fr] gap-x-4 gap-y-2 text-sm">
          {rows.map(([k, v]) => (
            <div key={k} className="contents">
              <dt className="text-muted-foreground">{k}</dt>
              <dd className="font-mono text-xs leading-5 break-all">{v}</dd>
            </div>
          ))}
        </dl>
      </CardContent>
    </Card>
  );
}
