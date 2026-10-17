import { Check, Copy } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { Alert, AlertDescription, AlertTitle } from "@/renderer/components/ui/alert";
import { Badge } from "@/renderer/components/ui/badge";
import { Button } from "@/renderer/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/renderer/components/ui/card";
import { Field, FieldGroup, FieldLabel } from "@/renderer/components/ui/field";
import { Spinner } from "@/renderer/components/ui/spinner";
import { patchAppSettings, useAppSettings } from "../../store/app-settings";
import { isWindows } from "@/renderer/lib/platform";
import { singleQuoted } from "@/shared/format";
import type { ApiStatus } from "@/shared/ipc";
import type { AppSettings } from "@/shared/settings";
import { SwitchField } from "./fields";

const COPIED_MS = 1500;

function useCopied() {
  const [copied, setCopied] = useState(false);
  const timer = useRef<ReturnType<typeof setTimeout>>(undefined);
  useEffect(() => () => clearTimeout(timer.current), []);
  const copy = (text: string) =>
    navigator.clipboard.writeText(text).then(() => {
      clearTimeout(timer.current);
      setCopied(true);
      timer.current = setTimeout(() => setCopied(false), COPIED_MS);
    });
  return { copied, copy };
}

function CopyLine({ text }: { text: string }) {
  const { copied, copy } = useCopied();
  return (
    <div className="bg-muted flex items-start gap-2 rounded-md px-3 py-2">
      <code className="min-w-0 flex-1 font-mono text-xs leading-5 break-all">{text}</code>
      <Button variant="ghost" size="icon-xs" aria-label="Copy" onClick={() => void copy(text)}>
        {copied ? <Check /> : <Copy />}
      </Button>
    </div>
  );
}

function ApiStateBadge({ on, listening }: { on: boolean; listening: boolean }) {
  if (!on) return <Badge variant="outline">Off</Badge>;
  return listening ? (
    <Badge variant="secondary">On</Badge>
  ) : (
    <Badge variant="destructive">Not running</Badge>
  );
}

function ApiStatusCard({ on, status }: { on: boolean; status: ApiStatus }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          Local API
          <ApiStateBadge on={on} listening={status.listening} />
        </CardTitle>
        <CardDescription>
          Scripts and AI agents on this computer can read the open project, change it and run it. It
          only listens on a local {isWindows ? "pipe" : "socket"} that only your user account can
          open, never on the network. Edits show in the editor and can be undone.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <FieldGroup>
          <SwitchField
            id="api-on"
            label="Turn on the API"
            description="Stays on until you turn it off. The top bar shows API while it's on."
            checked={on}
            onChange={(v) => void patchAppSettings({ api: { enabled: v } })}
          />
          {on && status.error ? (
            <Alert variant="destructive">
              <AlertTitle>The API couldn't start</AlertTitle>
              <AlertDescription>{status.error}</AlertDescription>
            </Alert>
          ) : null}
          <Field>
            <FieldLabel>Address</FieldLabel>
            <CopyLine text={status.address} />
          </Field>
        </FieldGroup>
      </CardContent>
    </Card>
  );
}

const curl = (address: string, route: string) =>
  `curl --unix-socket ${singleQuoted(address)} http://localhost${route}`;

function ApiUsageCard({ address }: { address: string }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Using it</CardTitle>
        <CardDescription>
          Plain HTTP with JSON. The description of every endpoint is at /openapi.json; node types
          are at /nodes and /nodes/&#123;type&#125;. Runs started here open the websites in the
          flow, just like pressing Run.
        </CardDescription>
      </CardHeader>
      <CardContent>
        {isWindows ? (
          <p className="text-muted-foreground text-sm">
            On Windows the API is a named pipe. Use an HTTP client that can connect to a named pipe,
            with the address above.
          </p>
        ) : (
          <div className="flex flex-col gap-2">
            <CopyLine text={curl(address, "/openapi.json")} />
            <CopyLine text={curl(address, "/status")} />
            <CopyLine text={curl(address, "/project")} />
          </div>
        )}
      </CardContent>
    </Card>
  );
}

export function ApiPage({ settings }: { settings: AppSettings }) {
  const status = useAppSettings((s) => s.api);
  if (!status) return <Spinner />;
  return (
    <>
      <ApiStatusCard on={settings.api.enabled} status={status} />
      <ApiUsageCard address={status.address} />
    </>
  );
}
