import { plural } from "@opsis/video/common/format";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Spinner } from "@/components/ui/spinner";
import { commands, type InspectedFonts } from "@/generated/bindings";
import { useBusy } from "@/lib/use-busy";
import { FontDialog } from "./font-dialog";
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
  onAdded: () => Promise<void>;
}) {
  const { fonts } = review;
  const [chosen, setChosen] = useState(() => fonts.families.map((f) => f.family));
  const { isBusy, run } = useBusy();

  useEffect(() => {
    function release() {
      return void commands.releaseInspectedFonts(fonts.token);
    }
    window.addEventListener("pagehide", release);
    return () => window.removeEventListener("pagehide", release);
  }, [fonts.token]);

  function close() {
    void commands.releaseInspectedFonts(fonts.token);
    onClose();
  }

  function add() {
    return run("add", async () => {
      const added = await commands.addInspectedFonts(fonts.token, chosen);
      toast.success(
        added.length === 1 ? `Added ${added[0].family}` : `Added ${plural(added.length, "font")}`,
      );
      await onAdded();
      close();
    });
  }

  return (
    <FontDialog
      onClose={close}
      title="Add fonts"
      description={
        <>
          Found {plural(fonts.families.length, "font family", "font families")}.{" "}
          {fonts.skipped ? `${fonts.skipped} duplicate files in other formats were skipped. ` : ""}
          {fonts.unreadable
            ? `${plural(fonts.unreadable, "file")} couldn't be read as a font.`
            : ""}
        </>
      }
      footer={
        <>
          <Button variant="outline" onClick={close}>
            Cancel
          </Button>
          <Button disabled={!chosen.length || isBusy} onClick={() => void add()}>
            {isBusy ? <Spinner data-icon="inline-start" /> : null}
            Add {plural(chosen.length, "font")}
          </Button>
        </>
      }
    >
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
    </FontDialog>
  );
}
