import { Check, Copy } from "lucide-react";
import { useState } from "react";
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
import { Field, FieldDescription, FieldGroup, FieldLabel } from "@/renderer/components/ui/field";
import { Spinner } from "@/renderer/components/ui/spinner";
import { Switch } from "@/renderer/components/ui/switch";
import { patchAppSettings, useAppSettings } from "../../store/app-settings";

const windows = window.demoflow?.platform === "win32";

function CopyLine({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <div className="bg-muted flex items-start gap-2 rounded-md px-3 py-2">
      <code className="min-w-0 flex-1 font-mono text-xs leading-5 break-all">{text}</code>
      <Button
        variant="ghost"
        size="icon-xs"
        aria-label="Copy"
        onClick={() => {
          void navigator.clipboard.writeText(text).then(() => {
            setCopied(true);
            setTimeout(() => setCopied(false), 1500);
          });
        }}
      >
        {copied ? <Check /> : <Copy />}
      </Button>
    </div>
  );
}

export function ApiPage() {
  const settings = useAppSettings((s) => s.settings);
  const status = useAppSettings((s) => s.api);
  if (!settings || !status) return <Spinner />;
  const on = settings.api.enabled;
  const address = status.address;
  const curl = (route: string) => `curl --unix-socket '${address}' http://localhost${route}`;

  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            Local API
            {on ? (
              status.listening ? (
                <Badge variant="secondary">On</Badge>
              ) : (
                <Badge variant="destructive">Not running</Badge>
              )
            ) : (
              <Badge variant="outline">Off</Badge>
            )}
          </CardTitle>
          <CardDescription>
            Scripts and AI agents on this computer can read the open project, change it and run it.
            It only listens on a local {windows ? "pipe" : "socket"} that only your user account can
            open, never on the network. Edits show in the editor and can be undone.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <FieldGroup>
            <Field orientation="horizontal" className="justify-between">
              <div className="flex flex-col gap-1">
                <FieldLabel htmlFor="api-on" className="font-normal">
                  Turn on the API
                </FieldLabel>
                <FieldDescription>
                  Stays on until you turn it off. The top bar shows API while it's on.
                </FieldDescription>
              </div>
              <Switch
                id="api-on"
                checked={on}
                onCheckedChange={(v) => void patchAppSettings({ api: { enabled: v } })}
              />
            </Field>
            {on && status.error ? (
              <Alert variant="destructive">
                <AlertTitle>The API couldn't start</AlertTitle>
                <AlertDescription>{status.error}</AlertDescription>
              </Alert>
            ) : null}
            <Field>
              <FieldLabel>Address</FieldLabel>
              <CopyLine text={address} />
            </Field>
          </FieldGroup>
        </CardContent>
      </Card>
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
          {windows ? (
            <p className="text-muted-foreground text-sm">
              On Windows the API is a named pipe. Use an HTTP client that can connect to a named
              pipe, with the address above.
            </p>
          ) : (
            <div className="flex flex-col gap-2">
              <CopyLine text={curl("/openapi.json")} />
              <CopyLine text={curl("/status")} />
              <CopyLine text={curl("/project")} />
            </div>
          )}
        </CardContent>
      </Card>
    </>
  );
}
