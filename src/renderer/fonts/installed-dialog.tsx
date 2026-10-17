import { useMemo, useState } from "react";
import type { FontInstalledFace, InspectedFonts } from "@/shared/ipc";
import { plural } from "@/shared/format";
import type { LocalFontData } from "./local-fonts";
import { useBusy } from "@/renderer/lib/use-busy";
import { FontSample } from "./font-sample";
import { BusyCount, matchesQuery, SearchListDialog } from "./search-list-dialog";

async function readFaces(faces: LocalFontData[]): Promise<FontInstalledFace[]> {
  const data: FontInstalledFace[] = [];
  for (const face of faces)
    data.push({
      postscriptName: face.postscriptName,
      data: await (await face.blob()).arrayBuffer(),
    });
  return data;
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
      open
      onClose={onClose}
      title="Add an installed font"
      description={`${families.length} font families are installed on this computer. Pick one to copy into your library.`}
      placeholder="Search installed fonts…"
      query={query}
      onQuery={setQuery}
      items={shown}
    >
      {([name, faces]) => (
        <button
          key={name}
          type="button"
          className="hover:bg-muted flex items-center justify-between rounded-md px-2 py-1.5 text-left text-sm"
          disabled={isBusy}
          onClick={() => void pick(name, faces)}
        >
          <FontSample family={name}>{name}</FontSample>
          <BusyCount busy={busy === name}>{plural(faces.length, "style")}</BusyCount>
        </button>
      )}
    </SearchListDialog>
  );
}
