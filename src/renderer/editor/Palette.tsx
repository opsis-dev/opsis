import { useReactFlow } from "@xyflow/react";
import { ArrowRight, PanelLeftOpen } from "lucide-react";
import { useMemo, useState } from "react";
import {
  CATEGORY_LABELS,
  NODE_CATEGORIES,
  type NodeCategory,
  type NodeDef,
} from "@/shared/nodes/define";
import { NODE_DEFS } from "@/shared/nodes/registry";
import { Button } from "@/renderer/components/ui/button";
import {
  Command,
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/renderer/components/ui/command";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "@/renderer/components/ui/hover-card";
import { Popover, PopoverContent, PopoverTrigger } from "@/renderer/components/ui/popover";
import { Separator } from "@/renderer/components/ui/separator";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { NODE_DRAG_TYPE } from "./Canvas";
import { NodeIcon } from "./icons";
import { openPanel, usePanels } from "./panels";
import { CATEGORY_COLORS, portLabel } from "./ports";

const CATEGORY_ICONS: Record<NodeCategory, string> = {
  Browser: "globe",
  Capture: "video",
  Camera: "scan-search",
  Cursor: "mouse-pointer-2",
  "Text & overlays": "type",
  Transitions: "blend",
  Backgrounds: "wallpaper",
  Effects: "sparkles",
  Elements: "shapes",
  Audio: "music",
  Timeline: "layers",
  Media: "file-play",
  Output: "clapperboard",
  Utility: "sticky-note",
};

function useGroups(only?: NodeCategory) {
  return useMemo(
    () =>
      NODE_CATEGORIES.filter((c) => !only || c === only)
        .map((category) => ({
          category,
          defs: NODE_DEFS.filter((d) => d.category === category && !d.hidden),
        }))
        .filter((g) => g.defs.length),
    [only],
  );
}

function useAddAtCenter() {
  const flow = useReactFlow();
  const addNode = useWorkflowStore((s) => s.addNode);
  const setSelected = useUiStore((s) => s.setSelected);
  return (type: string) => {
    const rect = document.querySelector(".react-flow")?.getBoundingClientRect();
    const center = flow.screenToFlowPosition({
      x: (rect?.left ?? 0) + (rect?.width ?? 800) / 2,
      y: (rect?.top ?? 0) + (rect?.height ?? 600) / 2,
    });
    setSelected([
      addNode(type, { x: Math.round(center.x / 15) * 15, y: Math.round(center.y / 15) * 15 }),
    ]);
  };
}

function CategoryBadge({
  category,
  icon,
  size = "sm",
}: {
  category: string;
  icon?: string;
  size?: "sm" | "md";
}) {
  return (
    <span
      className={
        size === "md"
          ? "grid size-7 shrink-0 place-items-center rounded-md text-black"
          : "grid size-5 shrink-0 place-items-center rounded text-black"
      }
      style={{ background: CATEGORY_COLORS[category] }}
    >
      <NodeIcon name={icon} className={size === "md" ? "size-4" : "size-3"} />
    </span>
  );
}

function NodePreview({ def }: { def: NodeDef }) {
  const inputs = Object.entries(def.inputs).map(([id, p]) => portLabel(id, p, "in"));
  const outputs = Object.entries(def.outputs).map(([id, p]) => portLabel(id, p, "out"));
  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center gap-2">
        <CategoryBadge category={def.category} icon={def.icon} size="md" />
        <div className="flex min-w-0 flex-col">
          <span className="truncate font-medium">{def.title}</span>
          <span className="text-muted-foreground text-xs">{CATEGORY_LABELS[def.category]}</span>
        </div>
      </div>
      {def.description ? <p className="text-muted-foreground text-xs">{def.description}</p> : null}
      {inputs.length || outputs.length ? (
        <div className="text-muted-foreground flex items-center gap-1.5 text-xs">
          <span>{inputs.length ? inputs.join(", ") : "Nothing"}</span>
          <ArrowRight className="size-3 shrink-0" />
          <span>{outputs.length ? outputs.join(", ") : "Nothing"}</span>
        </div>
      ) : null}
    </div>
  );
}

function PaletteList({
  onPick,
  draggable,
  only,
  onDragEnd,
}: {
  onPick: (type: string) => void;
  draggable?: boolean;
  only?: NodeCategory;
  onDragEnd?: () => void;
}) {
  const groups = useGroups(only);
  return (
    <CommandList className="max-h-none">
      <CommandEmpty>No nodes found.</CommandEmpty>
      {groups.map(({ category, defs }) => (
        <CommandGroup key={category} heading={only ? undefined : CATEGORY_LABELS[category]}>
          {defs.map((def) => (
            <CommandItem
              key={def.type}
              value={`${def.title} ${CATEGORY_LABELS[category]} ${def.description ?? ""}`}
              onSelect={() => onPick(def.type)}
              draggable={draggable}
              onDragStart={(e: React.DragEvent) => {
                e.dataTransfer.setData(NODE_DRAG_TYPE, def.type);
                e.dataTransfer.effectAllowed = "copy";
              }}
              onDragEnd={onDragEnd}
            >
              <HoverCard>
                <HoverCardTrigger
                  delay={500}
                  render={<span className="flex min-w-0 flex-1 items-center gap-2" />}
                >
                  <CategoryBadge category={category} icon={def.icon} />
                  <span className="truncate">{def.title}</span>
                </HoverCardTrigger>
                <HoverCardContent side="right" align="start" sideOffset={12}>
                  <NodePreview def={def} />
                </HoverCardContent>
              </HoverCard>
            </CommandItem>
          ))}
        </CommandGroup>
      ))}
    </CommandList>
  );
}

function PaletteRail() {
  const add = useAddAtCenter();
  const [openCategory, setOpenCategory] = useState<NodeCategory | null>(null);
  const groups = useGroups();
  return (
    <div className="flex h-full flex-col items-center gap-1 overflow-y-auto py-2">
      <Tooltip>
        <TooltipTrigger
          render={
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label="Show node list"
              onClick={() => openPanel("left")}
            >
              <PanelLeftOpen />
            </Button>
          }
        />
        <TooltipContent side="right">Show node list</TooltipContent>
      </Tooltip>
      <Separator className="my-1 w-6" />
      {groups.map(({ category }) => (
        <Popover
          key={category}
          open={openCategory === category}
          onOpenChange={(open) => setOpenCategory(open ? category : null)}
        >
          <Tooltip>
            <TooltipTrigger
              render={
                <PopoverTrigger
                  render={
                    <Button variant="ghost" size="icon-sm" aria-label={CATEGORY_LABELS[category]}>
                      <CategoryBadge category={category} icon={CATEGORY_ICONS[category]} />
                    </Button>
                  }
                />
              }
            />
            <TooltipContent side="right">{CATEGORY_LABELS[category]}</TooltipContent>
          </Tooltip>
          <PopoverContent side="right" align="start" className="w-64 p-0">
            <Command>
              <CommandInput placeholder={`Search ${CATEGORY_LABELS[category].toLowerCase()}…`} />
              <div className="max-h-96 overflow-y-auto">
                <PaletteList
                  only={category}
                  draggable
                  onPick={(type) => {
                    add(type);
                    setOpenCategory(null);
                  }}
                  onDragEnd={() => setOpenCategory(null)}
                />
              </div>
            </Command>
          </PopoverContent>
        </Popover>
      ))}
    </div>
  );
}

export function Palette() {
  const add = useAddAtCenter();
  const open = usePanels((s) => s.open.left);
  if (!open) return <PaletteRail />;
  return (
    <Command className="rounded-none! bg-transparent">
      <CommandInput placeholder={`Search ${NODE_DEFS.filter((d) => !d.hidden).length} nodes…`} />
      <PaletteList onPick={add} draggable />
    </Command>
  );
}

export function QuickAdd() {
  const open = useUiStore((s) => s.paletteOpen);
  const setOpen = useUiStore((s) => s.setPaletteOpen);
  const add = useAddAtCenter();
  return (
    <CommandDialog
      open={open}
      onOpenChange={setOpen}
      title="Add node"
      description="Search every node"
    >
      <Command>
        <CommandInput placeholder="Add a node…" />
        <PaletteList
          onPick={(type) => {
            add(type);
            setOpen(false);
          }}
        />
      </Command>
    </CommandDialog>
  );
}
