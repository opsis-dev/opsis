import { Box } from "lucide-react";
import { createElement } from "react";
import type { NodeIconName } from "@/shared/nodes/icon-name";
import { NODE_ICONS } from "./icons";

export const NodeIcon = ({ name, className }: { name?: NodeIconName; className?: string }) =>
  createElement(name ? NODE_ICONS[name] : Box, { className });
