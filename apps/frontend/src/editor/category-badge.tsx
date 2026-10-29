import type {
  Category as NodeCategory,
  EditorNode,
  Icon as NodeIconName,
} from "@/generated/bindings";
import { cn } from "@/lib/utils";
import { NodeGlyph } from "./node-glyph";
import { NodeIcon } from "./node-icon";
import { CATEGORY_STYLES } from "./ports";

const SIZES = {
  sm: { box: "size-5 rounded", icon: "size-3" },
  md: { box: "size-6 rounded-md", icon: "size-3.5" },
  lg: { box: "size-7 rounded-md", icon: "size-4" },
  xl: { box: "size-8 rounded-md", icon: "size-4" },
};

export function CategoryBadge({
  category,
  icon,
  size = "sm",
}: {
  category: NodeCategory;
  icon: NodeIconName | undefined;
  size?: keyof typeof SIZES;
}) {
  return (
    <span
      className={cn("grid shrink-0 place-items-center text-black", SIZES[size].box)}
      style={{ background: CATEGORY_STYLES[category].color }}
    >
      <NodeIcon name={icon} className={SIZES[size].icon} />
    </span>
  );
}

export function NodeBadge({
  def,
  size = "sm",
}: {
  def: Pick<EditorNode, "type" | "category">;
  size?: keyof typeof SIZES;
}) {
  return (
    <span
      className={cn(
        "relative grid shrink-0 place-items-center overflow-hidden text-black",
        SIZES[size].box,
      )}
      style={{ background: CATEGORY_STYLES[def.category].color }}
    >
      <NodeGlyph type={def.type} className={SIZES[size].icon} />
    </span>
  );
}
