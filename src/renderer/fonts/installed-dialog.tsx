import { useMemo, useState, type ReactNode } from "react";
import { Spinner } from "@/renderer/components/ui/spinner";
import { useBusy } from "@/renderer/lib/use-busy";
import { plural } from "@/shared/format";
import type { FontInstalledFace, InspectedFonts } from "@/shared/ipc";
import { ListRow } from "../editor/list-row";
import { FontSample } from "./font-sample";
import type { LocalFontData } from "./local-fonts";
import { matchesQuery, SearchListDialog } from "./search-list-dialog";

async function readFaces(faces: LocalFontData[]): Promise<FontInstalledFace[]> {
  const data: FontInstalledFace[] = [];
  for (const face of faces)
    data.push({
      postscriptName: face.postscriptName,
      data: await (await face.blob()).arrayBuffer(),
    });
  return data;
}

function BusyCount({ busy, children }: { busy: boolean; children: ReactNode }) {
  return (
    <span className="text-muted-foreground flex items-center gap-2 text-xs">
      {busy ? <Spinner /> : null}
      {children}
    </span>
  );
}

const groupByFamily = (fonts: LocalFontData[]): Array<[string, LocalFontData[]]> =>
  [...Map.groupBy(fonts, (f) => f.family)].toSorted(([a], [b]) => a.localeCompare(b));

export function InstalledDialog({
  fonts,
  onClose,
  onPicked,
}: {
  fonts: LocalFontData[];
  onClose: () => void;
  onPicked: (fonts: InspectedFonts) => void;
}) {
  const [query, setQuery] = useState("");
  const { busy, isBusy, run } = useBusy();
  const families = useMemo(() => groupByFamily(fonts), [fonts]);
  const shown = families.filter(([name]) => matchesQuery(query, name));

  const pick = (family: string, faces: LocalFontData[]) =>
    run(family, async () =>
      onPicked(await window.demoflow.inspectInstalledFonts(await readFaces(faces))),
    );

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
      {([name, faces]) => (
        <ListRow
          key={name}
          className="justify-between"
          disabled={isBusy}
          onClick={() => void pick(name, faces)}
        >
          <FontSample family={name}>{name}</FontSample>
          <BusyCount busy={busy === name}>{plural(faces.length, "style")}</BusyCount>
        </ListRow>
      )}
    </SearchListDialog>
  );
}
