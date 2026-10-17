import { Box, type LucideIcon } from "lucide-react";
import { createElement } from "react";
import * as NODE_ICONS from "./icons";

const kebabName = (exportName: string): string =>
  exportName.replace(/([a-z])([A-Z0-9])/g, "$1-$2").toLowerCase();

const ICONS = new Map<string, LucideIcon>(
  Object.entries(NODE_ICONS).map(([exportName, icon]) => [kebabName(exportName), icon]),
);

export const nodeIconNames = (): string[] => [...ICONS.keys()];

export const hasIcon = (name: string): boolean => ICONS.has(name);

export const NodeIcon = ({ name, className }: { name?: string; className?: string }) =>
  createElement((name && ICONS.get(name)) || Box, { className });
