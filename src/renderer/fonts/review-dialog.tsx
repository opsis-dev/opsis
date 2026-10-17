import { useState } from "react";
import { toast } from "sonner";
import type { InspectedFonts } from "@/shared/ipc";
import { errorMessage } from "@/shared/errors";
import { plural } from "@/shared/format";
import { Button } from "@/renderer/components/ui/button";
import { Checkbox } from "@/renderer/components/ui/checkbox";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/renderer/components/ui/dialog";
import { ScrollArea } from "@/renderer/components/ui/scroll-area";
import { Spinner } from "@/renderer/components/ui/spinner";
import { describeFaces } from "./font-labels";

export type FontReview = { fonts: InspectedFonts; installed: boolean };

function licenceNote({ fonts, installed }: FontReview): string {
  if (fonts.license) return `Licence file found: ${fonts.license}. It's kept with the fonts.`;
  return installed
    ? "Fonts installed on a computer often have licences that limit use in videos. Check yours before you publish."
    : "No licence file found. Check that you're allowed to use these fonts in videos.";
}

export function ReviewDialog({
  review,
  onClose,
  onAdded,
}: {
  review: FontReview;
  onClose: () => void;
  onAdded: () => void;
}) {
  const { fonts } = review;
  const [chosen, setChosen] = useState(() => fonts.families.map((f) => f.family));
  const [busy, setBusy] = useState(false);

  const add = async () => {
    setBusy(true);
    try {
      const added = await window.demoflow.addInspectedFonts(fonts.token, chosen);
      toast.success(
        added.length === 1 ? `Added ${added[0].family}` : `Added ${plural(added.length, "font")}`,
      );
      onAdded();
      onClose();
    } catch (error) {
      toast.error(errorMessage(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog open onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Add fonts</DialogTitle>
          <DialogDescription>
            Found {plural(fonts.families.length, "font family", "font families")}.{" "}
            {fonts.skipped ? `${fonts.skipped} duplicate files in other formats were skipped.` : ""}
          </DialogDescription>
        </DialogHeader>
        <ScrollArea className="max-h-72">
          <div className="flex flex-col gap-2 pr-3">
            {fonts.families.map((f) => (
              <label key={f.family} className="flex items-start gap-3 text-sm">
                <Checkbox
                  checked={chosen.includes(f.family)}
                  onCheckedChange={(on) =>
                    setChosen((c) => (on ? [...c, f.family] : c.filter((x) => x !== f.family)))
                  }
                />
                <span className="flex min-w-0 flex-col">
                  <span className="font-medium">{f.family}</span>
                  <span className="text-muted-foreground text-xs">{describeFaces(f.faces)}</span>
                </span>
              </label>
            ))}
          </div>
        </ScrollArea>
        <p className="text-muted-foreground text-xs">{licenceNote(review)}</p>
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button disabled={!chosen.length || busy} onClick={() => void add()}>
            {busy ? <Spinner data-icon="inline-start" /> : null}
            Add {plural(chosen.length, "font")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
