import * as lucide from "lucide-react";
import { iconNames } from "lucide-react/dynamic";
import { describe, expect, it } from "vitest";
import { NODE_ICONS } from "./icons";

const exportName = (name: string): string =>
  `${name
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join("")}Icon`;

const lucideExports: Record<string, unknown> = lucide;

describe("node icons", () => {
  it("names every icon after the icon it draws", () => {
    const known = new Set<string>(iconNames);
    const wrong = Object.entries(NODE_ICONS).filter(
      ([name, icon]) => !known.has(name) || lucideExports[exportName(name)] !== icon,
    );
    expect(wrong.map(([name]) => name)).toEqual([]);
  });
});
