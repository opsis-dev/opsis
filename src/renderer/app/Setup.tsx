import { useNavigate } from "@tanstack/react-router";
import { CheckCircle2, Download } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import type { EngineInstallEvent, EnvStatus } from "@/shared/ipc";
import { Badge } from "@/renderer/components/ui/badge";
import { Button } from "@/renderer/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/renderer/components/ui/card";
import { Progress } from "@/renderer/components/ui/progress";
import { Spinner } from "@/renderer/components/ui/spinner";

export const SETUP_SKIPPED_KEY = "demoflow:setup-skipped";

export function EnginePanel({ onDone }: { onDone?: () => void }) {
  const [env, setEnv] = useState<EnvStatus | null>(null);
  const [event, setEvent] = useState<EngineInstallEvent | null>(null);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(() => void window.demoflow.getEnvironment().then(setEnv), []);
  useEffect(() => {
    refresh();
    return window.demoflow.onEngineInstall(setEvent);
  }, [refresh]);

  const ready = !!env?.engine.complete;
  const bundled = env?.edition === "full";

  async function install() {
    setBusy(true);
    try {
      await window.demoflow.installEngine();
    } catch {
      setBusy(false);
      refresh();
      return;
    }
    setBusy(false);
    refresh();
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          Engine
          {ready ? (
            <Badge variant="secondary">
              <CheckCircle2 data-icon="inline-start" />
              {bundled ? "Included" : "Set up"}
            </Badge>
          ) : (
            <Badge variant="outline">Not set up</Badge>
          )}
          {env ? <Badge variant="outline">{env.edition} edition</Badge> : null}
        </CardTitle>
        <CardDescription>
          The recording browser and video tools Demoflow uses to record takes and export videos.
          Editing and preview work without them.
        </CardDescription>
      </CardHeader>
      {busy || event?.error ? (
        <CardContent className="flex flex-col gap-2">
          {event?.progress != null ? <Progress value={Math.round(event.progress * 100)} /> : null}
          <p
            className={event?.error ? "text-destructive text-xs" : "text-muted-foreground text-xs"}
          >
            {event ? (event.error ?? event.message) : "Starting…"}
          </p>
        </CardContent>
      ) : null}
      <CardFooter className="gap-2">
        {bundled ? null : (
          <Button
            size="sm"
            variant={ready ? "outline" : "default"}
            disabled={busy}
            onClick={() => void install()}
          >
            {busy ? <Spinner data-icon="inline-start" /> : <Download data-icon="inline-start" />}
            {ready ? "Set up again" : "Set up engine"}
          </Button>
        )}
        {onDone ? (
          <Button size="sm" variant="ghost" onClick={onDone}>
            {ready ? "Continue" : "Skip for now"}
          </Button>
        ) : null}
      </CardFooter>
    </Card>
  );
}

export function Setup() {
  const navigate = useNavigate();
  return (
    <div className="h-screen overflow-y-auto">
      <div className="mx-auto flex min-h-full max-w-xl flex-col justify-center gap-6 px-6 py-10">
        <div className="flex flex-col gap-1">
          <h1 className="text-2xl font-semibold tracking-tight">Welcome to Demoflow</h1>
          <p className="text-muted-foreground text-sm">
            This version doesn't include the recording and export engine. Set it up once to record
            and export.
          </p>
        </div>
        <EnginePanel
          onDone={() => {
            try {
              localStorage.setItem(SETUP_SKIPPED_KEY, "1");
            } catch {
              void 0;
            }
            void navigate({ to: "/" });
          }}
        />
      </div>
    </div>
  );
}
