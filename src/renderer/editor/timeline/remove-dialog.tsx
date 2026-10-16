import type { Removal } from "@/shared/timeline/edits";
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

export function RemoveDialog({
  removal,
  onConfirm,
  onClose,
}: {
  removal: Removal | null;
  onConfirm: (removal: Removal) => void;
  onClose: () => void;
}) {
  return (
    <AlertDialog open={!!removal} onOpenChange={(open) => !open && onClose()}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Remove {removal?.nodeIds.length} nodes?</AlertDialogTitle>
          <AlertDialogDescription>
            These nodes only feed the selected clips. Undo brings them back.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction
            variant="destructive"
            onClick={() => {
              if (removal) onConfirm(removal);
              onClose();
            }}
          >
            Remove
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
