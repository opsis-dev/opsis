import { Box, type LucideIcon } from "lucide-react";
import { createElement } from "react";
import { NODE_ICONS } from "./icons";

const ICONS = new Map<string, LucideIcon>(Object.entries(NODE_ICONS));

export const NodeIcon = ({ name, className }: { name?: string; className?: string }) =>
  createElement((name && ICONS.get(name)) || Box, { className });
