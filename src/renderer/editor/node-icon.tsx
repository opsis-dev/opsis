import { Box } from "lucide-react";
import type { NodeIconName } from "@/shared/nodes/icon-name";
import { NODE_ICONS } from "./icons";

export function NodeIcon({ name, className }: { name?: NodeIconName; className?: string }) {
  const Icon = name ? NODE_ICONS[name] : Box;
  return <Icon className={className} />;
}
