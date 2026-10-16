import { useEffect, useState } from "react";
import type { CloseRequest } from "@/shared/ipc";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/renderer/components/ui/alert-dialog";
import { Checkbox } from "@/renderer/components/ui/checkbox";
import { Label } from "@/renderer/components/ui/label";

export function ExitDialog() {
  const [request, setRequest] = useState<CloseRequest | null>(null);
  const [dontAsk, setDontAsk] = useState(false);

  useEffect(
    () =>
      window.demoflow.onConfirmClose((next) => {
        setDontAsk(false);
        setRequest(next);
      }),
    [],
  );

  const answer = (close: boolean) => {
    window.demoflow.decideClose({ close, dontAsk: close && dontAsk });
    setRequest(null);
  };

  const busy = request?.busy ?? [];
  return (
    <AlertDialog open={!!request} onOpenChange={(open) => !open && request && answer(false)}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>
            {request?.ask ? "Are you sure you want to exit?" : "Some work hasn't finished yet"}
          </AlertDialogTitle>
          <AlertDialogDescription>
            {busy.length
              ? "Closing now stops it:"
              : "Your project is saved. You can pick up where you left off."}
          </AlertDialogDescription>
          {busy.length ? (
            <ul className="text-muted-foreground list-disc pl-5 text-sm">
              {busy.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          ) : null}
        </AlertDialogHeader>
        {request?.ask ? (
          <Label className="flex items-center gap-2 text-sm font-normal">
            <Checkbox checked={dontAsk} onCheckedChange={(v) => setDontAsk(v === true)} />
            Don't ask again
          </Label>
        ) : null}
        <AlertDialogFooter>
          <AlertDialogCancel onClick={() => answer(false)}>Cancel</AlertDialogCancel>
          <AlertDialogAction
            variant={busy.length ? "destructive" : "default"}
            onClick={() => answer(true)}
          >
            {busy.length ? "Close anyway" : "Exit"}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
