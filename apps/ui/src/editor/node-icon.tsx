import { Box } from "lucide-react";
import type { Icon as NodeIconName } from "@/generated/bindings";
import { NODE_ICONS } from "./icons";

export function NodeIcon({ name, className }: { name?: NodeIconName; className?: string }) {
  const Icon = name ? NODE_ICONS[name] : Box;
  return <Icon className={className} />;
}
