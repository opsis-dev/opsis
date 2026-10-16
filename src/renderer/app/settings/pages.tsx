import { FolderOpen, RotateCcw } from "lucide-react";
import { useEffect, useState } from "react";
import type { EnvStatus } from "@/shared/ipc";
import type { AppSettings, StartOption } from "@/shared/settings";
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
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from "@/renderer/components/ui/input-group";
import { Slider } from "@/renderer/components/ui/slider";
import { Spinner } from "@/renderer/components/ui/spinner";
import { Switch } from "@/renderer/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "@/renderer/components/ui/toggle-group";
import { patchAppSettings, useAppSettings } from "../../store/app-settings";

const RETENTION: Array<[AppSettings["traces"]["retention"], string]> = [
  ["always", "Always"],
  ["failures", "Failures only"],
  ["last-n", "Last N"],
  ["off", "Off"],
];

export function useSettings(): AppSettings | null {
  return useAppSettings((s) => s.settings);
}

export function GeneralPage() {
  const settings = useSettings();
  if (!settings) return <Spinner />;
  return (
    <Card>
      <CardHeader>
        <CardTitle>Starting and closing</CardTitle>
      </CardHeader>
      <CardContent>
        <FieldGroup>
          <Field>
            <FieldLabel>When Demoflow starts</FieldLabel>
            <ToggleGroup
              variant="outline"
              size="sm"
              value={[settings.app.onStart]}
              onValueChange={(v) =>
                v[0] && void patchAppSettings({ app: { onStart: v[0] as StartOption } })
              }
            >
              <ToggleGroupItem value="start-screen">Show the start screen</ToggleGroupItem>
              <ToggleGroupItem value="last-project">Open the last project</ToggleGroupItem>
              <ToggleGroupItem value="new-project">Start a new project</ToggleGroupItem>
            </ToggleGroup>
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

export function ProjectsPage() {
  const settings = useSettings();
  const [fallback, setFallback] = useState("");
  const [draft, setDraft] = useState<string | null>(null);

  useEffect(() => {
    void window.demoflow.projectLocations().then((l) => {
      if (!useAppSettings.getState().settings?.projects.defaultDir) setFallback(l.defaultDir);
    });
  }, []);

  if (!settings) return <Spinner />;
  const custom = settings.projects.defaultDir;
  const shown = draft ?? (custom || fallback);
  const commit = (dir: string) => {
    setDraft(null);
    void patchAppSettings({ projects: { defaultDir: dir.trim() === fallback ? "" : dir.trim() } });
  };

  async function browse() {
    const picked = await window.demoflow.chooseFolder({
      title: "Choose the default projects folder",
      defaultPath: shown || undefined,
    });
    if (picked) commit(picked);
  }

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
            <InputGroup>
              <InputGroupInput
                id="projects-dir"
                value={shown}
                spellCheck={false}
                onChange={(e) => setDraft(e.target.value)}
                onBlur={() => draft !== null && commit(draft)}
                onKeyDown={(e) => e.key === "Enter" && draft !== null && commit(draft)}
              />
              <InputGroupAddon align="inline-end">
                <InputGroupButton size="xs" onClick={() => void browse()}>
                  <FolderOpen data-icon="inline-start" />
                  Browse
                </InputGroupButton>
              </InputGroupAddon>
            </InputGroup>
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

export function RecordingPage() {
  const settings = useSettings();
  if (!settings) return <Spinner />;
  return (
    <>
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
              <ToggleGroup
                variant="outline"
                size="sm"
                value={[settings.traces.retention]}
                onValueChange={(v) =>
                  v[0] &&
                  void patchAppSettings({
                    traces: { retention: v[0] as AppSettings["traces"]["retention"] },
                  })
                }
              >
                {RETENTION.map(([value, label]) => (
                  <ToggleGroupItem key={value} value={value}>
                    {label}
                  </ToggleGroupItem>
                ))}
              </ToggleGroup>
            </Field>
            {settings.traces.retention === "last-n" ? (
              <Field>
                <FieldLabel htmlFor="keep-last">Runs to keep per node</FieldLabel>
                <Input
                  id="keep-last"
                  type="number"
                  min={1}
                  max={500}
                  value={settings.traces.keepLast}
                  onChange={(e) =>
                    Number(e.target.value) > 0 &&
                    void patchAppSettings({ traces: { keepLast: Number(e.target.value) } })
                  }
                />
              </Field>
            ) : null}
          </FieldGroup>
        </CardContent>
      </Card>
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
                <span className="text-muted-foreground text-sm tabular-nums">
                  {settings.live.fps} fps
                </span>
              </div>
              <Slider
                min={1}
                max={30}
                step={1}
                value={[settings.live.fps]}
                onValueChange={(v) =>
                  void patchAppSettings({ live: { fps: Array.isArray(v) ? v[0] : v } })
                }
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
                checked={settings.live.autoOpen}
                onCheckedChange={(v) => void patchAppSettings({ live: { autoOpen: v } })}
              />
            </Field>
          </FieldGroup>
        </CardContent>
      </Card>
    </>
  );
}

export function ExportPage() {
  const settings = useSettings();
  if (!settings) return <Spinner />;
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
  const [env, setEnv] = useState<EnvStatus | null>(null);
  useEffect(() => void window.demoflow.getEnvironment().then(setEnv), []);
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
