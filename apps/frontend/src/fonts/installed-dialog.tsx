import { plural } from "@opsis/video/common/format";
import { useMemo, useState, type ReactNode } from "react";
import { Spinner } from "@/components/ui/spinner";
import { commands, type InspectedFonts, type InstalledFamily } from "@/generated/bindings";
import { useBusy } from "@/lib/use-busy";
import { ListRow } from "../editor/list-row";
import { FontSample } from "./font-sample";
import { matchesQuery, SearchListDialog } from "./search-list-dialog";

function BusyCount({ busy, children }: { busy: boolean; children: ReactNode }) {
  return (
    <span className="text-muted-foreground flex items-center gap-2 text-xs">
      {busy ? <Spinner /> : null}
      {children}
    </span>
  );
}

export function InstalledDialog({
  fonts,
  onClose,
  onPicked,
}: {
  fonts: InstalledFamily[];
  onClose: () => void;
  onPicked: (fonts: InspectedFonts) => void;
}) {
  const [query, setQuery] = useState("");
  const { busy, isBusy, run } = useBusy();
  const families = useMemo(
    () => fonts.toSorted((a, b) => a.family.localeCompare(b.family)),
    [fonts],
  );
  const shown = families.filter(({ family }) => matchesQuery(query, family));

  function pick(family: string) {
    return run(family, async () => onPicked(await commands.inspectInstalledFonts([family])));
  }

  return (
    <SearchListDialog
      onClose={onClose}
      title="Add an installed font"
      description={`${families.length} font families are installed on this computer. Pick one to copy into your library.`}
      placeholder="Search installed fonts…"
      query={query}
      onQuery={setQuery}
      items={shown}
    >
      {({ family, faces, stack }) => (
        <ListRow
          key={family}
          className="justify-between"
          disabled={isBusy}
          onClick={() => void pick(family)}
        >
          <FontSample stack={stack}>{family}</FontSample>
          <BusyCount busy={busy === family}>{plural(faces, "style")}</BusyCount>
        </ListRow>
      )}
    </SearchListDialog>
  );
}
