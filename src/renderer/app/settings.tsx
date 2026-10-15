import { useNavigate } from "@tanstack/react-router";
import { PageBar } from "./app-bar";
import { ArrowLeft, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import type { EnvStatus } from "@/shared/ipc";
import type { AppSettings } from "@/shared/settings";
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
import { Slider } from "@/renderer/components/ui/slider";
import { Switch } from "@/renderer/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "@/renderer/components/ui/toggle-group";
import { useRunStore } from "../store/run";
import { useWorkflowStore } from "../store/workflow";
import { FontsPanel } from "../fonts/fonts-panel";
import { EnginePanel } from "./setup";

const RETENTION: Array<[AppSettings["traces"]["retention"], string]> = [
  ["always", "Always"],
  ["failures", "Failures only"],
  ["last-n", "Last N"],
  ["off", "Off"],
];

export function Settings() {
  const navigate = useNavigate();
  const [env, setEnv] = useState<EnvStatus | null>(null);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const projectDir = useWorkflowStore((s) => s.projectDir);

  useEffect(() => {
    void window.demoflow.getEnvironment().then(setEnv);
    void window.demoflow.getSettings().then(setSettings);
  }, []);

  const update = (next: AppSettings) => {
    setSettings(next);
    void window.demoflow.setSettings(next).then(setSettings);
  };

  const rows: Array<[string, string]> = env
    ? [
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
      ]
    : [];

  return (
    <div className="flex h-screen flex-col">
      <PageBar title="Settings" />
      <div className="min-h-0 flex-1 overflow-y-auto">
        <div className="mx-auto flex min-h-full max-w-3xl flex-col gap-6 px-8 py-10">
          <header className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="icon"
              aria-label="Back"
              onClick={() =>
                projectDir
                  ? void navigate({ to: "/editor", search: { dir: projectDir } })
                  : void navigate({ to: "/" })
              }
            >
              <ArrowLeft />
            </Button>
            <h1 className="text-2xl font-semibold tracking-tight">Settings</h1>
          </header>

          <EnginePanel />

          <FontsPanel />

          {settings ? (
            <Card>
              <CardHeader>
                <CardTitle>Traces</CardTitle>
                <CardDescription>
                  A step-by-step record of every take, for finding out what went wrong. Open it from
                  the Trace tab in Replay. Stored compressed inside the project.
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
                        update({
                          ...settings,
                          traces: {
                            ...settings.traces,
                            retention: v[0] as AppSettings["traces"]["retention"],
                          },
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
                          update({
                            ...settings,
                            traces: { ...settings.traces, keepLast: Number(e.target.value) },
                          })
                        }
                      />
                    </Field>
                  ) : null}
                </FieldGroup>
              </CardContent>
            </Card>
          ) : null}

          {settings ? (
            <Card>
              <CardHeader>
                <CardTitle>Live view</CardTitle>
                <CardDescription>
                  What the recording browser is doing, shown in the editor while a run is in
                  progress.
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
                        update({
                          ...settings,
                          live: { ...settings.live, fps: Array.isArray(v) ? v[0] : v },
                        })
                      }
                    />
                    <FieldDescription>
                      Only affects the preview; recordings always use the Record node's fps.
                    </FieldDescription>
                  </Field>
                  <Field orientation="horizontal" className="justify-between">
                    <FieldLabel htmlFor="live-auto" className="font-normal">
                      Open the live view when a take starts
                    </FieldLabel>
                    <Switch
                      id="live-auto"
                      checked={settings.live.autoOpen}
                      onCheckedChange={(v) =>
                        update({ ...settings, live: { ...settings.live, autoOpen: v } })
                      }
                    />
                  </Field>
                </FieldGroup>
              </CardContent>
            </Card>
          ) : null}

          {settings ? (
            <Card>
              <CardHeader>
                <CardTitle>Video export licence</CardTitle>
                <CardDescription>
                  Demoflow renders videos with Remotion, which has its own licence. It's free for
                  individuals, non-profits and companies with up to 3 employees. Larger companies
                  need a Remotion company licence and must enter its key here.
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
                      onChange={(e) =>
                        update({ ...settings, render: { licenseKey: e.target.value } })
                      }
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
          ) : null}

          <Card>
            <CardHeader>
              <CardTitle>About</CardTitle>
              <CardDescription>Everything runs on this computer.</CardDescription>
            </CardHeader>
            <CardContent>
              <dl className="grid grid-cols-[10rem_1fr] gap-x-4 gap-y-2 text-sm">
                {rows.map(([k, v]) => (
                  <div key={k} className="contents">
                    <dt className="text-muted-foreground">{k}</dt>
                    <dd className="truncate font-mono text-xs leading-5">{v}</dd>
                  </div>
                ))}
              </dl>
            </CardContent>
          </Card>

          {projectDir ? (
            <Card>
              <CardHeader>
                <CardTitle>Saved results</CardTitle>
                <CardDescription>
                  Demoflow keeps the result of every node so unchanged parts don't run again. Clear
                  them to start fresh.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <Button
                  variant="outline"
                  onClick={async () => {
                    await window.demoflow.clearCache(projectDir);
                    useRunStore.getState().seed(null);
                    toast.success("Cleared. The next run records and exports everything again.");
                  }}
                >
                  <Trash2 data-icon="inline-start" />
                  Clear saved results
                </Button>
              </CardContent>
            </Card>
          ) : null}
        </div>
      </div>
    </div>
  );
}
