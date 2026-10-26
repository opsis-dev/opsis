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
import type { Removal } from "@/generated/bindings";
import { useLastDefined } from "@/lib/use-last-defined";

function RemoveContent({ count, onRemove }: { count: number; onRemove: () => void }) {
  return (
    <AlertDialogContent>
      <AlertDialogHeader>
        <AlertDialogTitle>Remove {count} nodes?</AlertDialogTitle>
        <AlertDialogDescription>
          These nodes only feed the selected clips. Undo brings them back.
        </AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel>Cancel</AlertDialogCancel>
        <AlertDialogAction variant="destructive" onClick={onRemove}>
          Remove
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  );
}

export function RemoveDialog({
  removal,
  onConfirm,
  onClose,
}: {
  removal: Removal | null;
  onConfirm: (removal: Removal) => void;
  onClose: () => void;
}) {
  const shown = useLastDefined(removal);
  return (
    <AlertDialog open={!!removal} onOpenChange={(open) => !open && onClose()}>
      {shown ? (
        <RemoveContent
          count={shown.nodeIds.length}
          onRemove={() => {
            onConfirm(shown);
            onClose();
          }}
        />
      ) : null}
    </AlertDialog>
  );
}
