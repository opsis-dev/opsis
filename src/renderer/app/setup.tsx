import { useNavigate } from "@tanstack/react-router";
import { CheckCircle2, Download } from "lucide-react";
import { useEffect, useState } from "react";
import type { EngineInstallEvent } from "@/shared/engine";
import { errorMessage } from "@/shared/errors";
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
import { PageBar } from "./app-bar";
import { AnimatedPercent } from "./motion";
import { useEnvironment, useFreshEnvironment } from "../store/environment";
import { skipSetup } from "./startup-flags";

function useEngineInstall() {
  const [event, setEvent] = useState<EngineInstallEvent | null>(null);
  const [failure, setFailure] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => window.demoflow.onEngineInstall(setEvent), []);

  async function install() {
    setBusy(true);
    setFailure(null);
    try {
      await window.demoflow.installEngine();
    } catch (e) {
      setFailure(errorMessage(e));
    } finally {
      setBusy(false);
      void useEnvironment.getState().refresh();
    }
  }

  return { event, error: event?.error ?? failure, busy, install };
}

function EngineProgress({
  event,
  error,
}: {
  event: EngineInstallEvent | null;
  error: string | null;
}) {
  return (
    <CardContent className="flex flex-col gap-2">
      {!error && event?.progress != null ? (
        <div key={`${event.part}|${event.step}`} className="flex items-center gap-2">
          <Progress className="flex-1" value={Math.round(event.progress * 100)} />
          <AnimatedPercent
            className="text-muted-foreground w-10 text-right text-xs"
            value={event.progress * 100}
          />
        </div>
      ) : null}
      <p className={error ? "text-destructive text-xs" : "text-muted-foreground text-xs"}>
        {error ?? event?.message ?? "Starting…"}
      </p>
    </CardContent>
  );
}

function EngineBadges({
  ready,
  bundled,
  edition,
}: {
  ready: boolean;
  bundled: boolean;
  edition?: string;
}) {
  return (
    <>
      {ready ? (
        <Badge variant="secondary">
          <CheckCircle2 data-icon="inline-start" />
          {bundled ? "Included" : "Set up"}
        </Badge>
      ) : (
        <Badge variant="outline">Not set up</Badge>
      )}
      {edition ? <Badge variant="outline">{edition} edition</Badge> : null}
    </>
  );
}

export function EnginePanel({ onDone, title = "Engine" }: { onDone?: () => void; title?: string }) {
  const env = useFreshEnvironment();
  const { event, error, busy, install } = useEngineInstall();
  const ready = !!env?.engine.complete;
  const bundled = env?.edition === "full";

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          {title}
          <EngineBadges ready={ready} bundled={bundled} edition={env?.edition} />
        </CardTitle>
        <CardDescription>
          The recording browser and video tools Demoflow uses to record takes and export videos.
          Editing and preview work without them.
        </CardDescription>
      </CardHeader>
      {busy || error ? <EngineProgress event={event} error={error} /> : null}
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
    <div className="flex h-screen flex-col">
      <PageBar title="Welcome" />
      <div className="min-h-0 flex-1 overflow-y-auto">
        <div className="mx-auto flex min-h-full max-w-xl flex-col justify-center gap-6 px-6 py-10">
          <div className="flex flex-col gap-1">
            <h1 className="text-2xl font-semibold tracking-tight">Welcome to Demoflow</h1>
            <p className="text-muted-foreground text-sm">
              This version doesn't include the recording and export engine. Set it up once to record
              and export.
            </p>
          </div>
          <p className="text-muted-foreground text-xs">
            Video export uses Remotion, which is free for individuals, non-profits and companies
            with up to 3 employees. Larger companies need a Remotion company licence; enter its key
            in Settings.
          </p>
          <EnginePanel
            onDone={() => {
              skipSetup();
              void navigate({ to: "/" });
            }}
          />
        </div>
      </div>
    </div>
  );
}
