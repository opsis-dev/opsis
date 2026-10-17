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
import { useDiskChange } from "./disk-change-store";
import { keepMine, loadFromDisk } from "./save";

export function DiskChangeDialog() {
  const change = useDiskChange((s) => s.change);
  const readable = !!change?.workflow;
  return (
    <AlertDialog open={!!change}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>The project file changed on disk</AlertDialogTitle>
          <AlertDialogDescription>
            {readable
              ? "Something outside Demoflow changed this project while you had unsaved edits. Load the file to use its version (Undo brings yours back), or keep yours to save over it."
              : "Something outside Demoflow changed this project, and the file isn't a valid project right now. Keep yours to save over it, or wait and fix the file first."}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          {readable ? (
            <AlertDialogAction variant="outline" onClick={() => change && loadFromDisk(change)}>
              Load the file
            </AlertDialogAction>
          ) : (
            <AlertDialogCancel onClick={() => useDiskChange.getState().show(null)}>
              Wait
            </AlertDialogCancel>
          )}
          <AlertDialogAction onClick={() => change && void keepMine(change)}>
            Keep mine
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
