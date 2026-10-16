import { useStoreApi } from "@xyflow/react";
import { ArrowRight, PanelLeftOpen } from "lucide-react";
import { useMemo, useState } from "react";
import {
  CATEGORY_LABELS,
  NODE_CATEGORIES,
  type NodeCategory,
  type NodeDef,
} from "@/shared/nodes/define";
import { VISIBLE_NODE_DEFS } from "@/shared/nodes/registry";
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
import { NODE_DRAG_TYPE, snapToGrid } from "./canvas";
import { CategoryBadge } from "./category-badge";
import { openPanel, usePanels } from "./panels";
import { CATEGORY_STYLES, portLabel } from "./ports";

function useGroups(only?: NodeCategory) {
  return useMemo(
    () =>
      NODE_CATEGORIES.filter((c) => !only || c === only)
        .map((category) => ({
          category,
          defs: VISIBLE_NODE_DEFS.filter((d) => d.category === category),
        }))
        .filter((g) => g.defs.length),
    [only],
  );
}

function useAddAtCenter() {
  const flowStore = useStoreApi();
  const addNode = useWorkflowStore((s) => s.addNode);
  const setSelected = useUiStore((s) => s.setSelected);
  return (type: string) => {
    const {
      width,
      height,
      transform: [x, y, zoom],
    } = flowStore.getState();
    const center = { x: (width / 2 - x) / zoom, y: (height / 2 - y) / zoom };
    setSelected([addNode(type, snapToGrid(center))]);
  };
}

function NodePreview({ def }: { def: NodeDef }) {
  const inputs = Object.values(def.inputs).map((p) => portLabel(p, "in"));
  const outputs = Object.values(def.outputs).map((p) => portLabel(p, "out"));
  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center gap-2">
        <CategoryBadge category={def.category} icon={def.icon} size="lg" />
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

const STICKY_HEADINGS =
  "overflow-visible **:[[cmdk-group-heading]]:sticky **:[[cmdk-group-heading]]:top-0 **:[[cmdk-group-heading]]:z-10 **:[[cmdk-group-heading]]:bg-sidebar";

function PaletteList({
  onPick,
  draggable,
  only,
  onDragEnd,
  stickyHeadings,
}: {
  onPick: (type: string) => void;
  draggable?: boolean;
  only?: NodeCategory;
  onDragEnd?: () => void;
  stickyHeadings?: boolean;
}) {
  const groups = useGroups(only);
  return (
    <CommandList className="max-h-none">
      <CommandEmpty>No nodes found.</CommandEmpty>
      {groups.map(({ category, defs }) => (
        <CommandGroup
          key={category}
          heading={only ? undefined : CATEGORY_LABELS[category]}
          className={stickyHeadings ? STICKY_HEADINGS : undefined}
        >
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

function RailCategory({
  category,
  open,
  onOpenChange,
  onPick,
}: {
  category: NodeCategory;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onPick: (type: string) => void;
}) {
  const label = CATEGORY_LABELS[category];
  const trigger = (
    <PopoverTrigger
      render={
        <Button variant="ghost" size="icon-sm" aria-label={label}>
          <CategoryBadge category={category} icon={CATEGORY_STYLES[category].icon} />
        </Button>
      }
    />
  );
  return (
    <Popover open={open} onOpenChange={onOpenChange}>
      <Tooltip>
        <TooltipTrigger render={trigger} />
        <TooltipContent side="right">{label}</TooltipContent>
      </Tooltip>
      <PopoverContent side="right" align="start" className="w-64 p-0">
        <Command>
          <CommandInput placeholder={`Search ${label.toLowerCase()}…`} />
          <div className="max-h-96 overflow-y-auto">
            <PaletteList
              only={category}
              draggable
              onPick={(type) => {
                onPick(type);
                onOpenChange(false);
              }}
              onDragEnd={() => onOpenChange(false)}
            />
          </div>
        </Command>
      </PopoverContent>
    </Popover>
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
        <RailCategory
          key={category}
          category={category}
          open={openCategory === category}
          onOpenChange={(open) => setOpenCategory(open ? category : null)}
          onPick={add}
        />
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
      <CommandInput placeholder={`Search ${VISIBLE_NODE_DEFS.length} nodes…`} />
      <PaletteList onPick={add} draggable stickyHeadings />
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
