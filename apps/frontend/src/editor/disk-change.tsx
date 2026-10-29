import { useEffect, useState } from "react";
import { toast } from "sonner";
import { on } from "@/app/tauri";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { commands, events, type DiskChoice } from "@/generated/bindings";
import { attempt } from "@/lib/toast";

type Asked = { readable: boolean };

function useDiskChangeEvents(ask: (asked: Asked) => void): void {
  useEffect(() => {
    const offAsked = on(events.diskChangeAsked, ask);
    const offReloaded = on(events.projectReloaded, () =>
      toast.info("The project file changed on disk, so the editor loaded it.", {
        description: "Undo brings back the previous version.",
      }),
    );
    return () => {
      offAsked();
      offReloaded();
    };
  }, [ask]);
}

export function DiskChangeDialog() {
  const [asked, setAsked] = useState<Asked | null>(null);
  useDiskChangeEvents(setAsked);
  if (!asked) return null;
  const choose = (choice: DiskChoice) => {
    setAsked(null);
    void attempt(() => commands.resolveDiskChange(choice), "Couldn't save the project");
  };
  return (
    <AlertDialog open>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>The project file changed on disk</AlertDialogTitle>
          <AlertDialogDescription>
            {asked.readable
              ? "Something outside Opsis changed this project while you had unsaved edits. Load the file to use its version (Undo brings yours back), or keep yours to save over it."
              : "Something outside Opsis changed this project, and the file isn't a valid project right now. Keep yours to save over it, or wait and fix the file first."}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          {asked.readable ? (
            <AlertDialogAction variant="outline" onClick={() => choose("load")}>
              Load the file
            </AlertDialogAction>
          ) : (
            <AlertDialogCancel onClick={() => choose("wait")}>Wait</AlertDialogCancel>
          )}
          <AlertDialogAction onClick={() => choose("keepMine")}>Keep mine</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
