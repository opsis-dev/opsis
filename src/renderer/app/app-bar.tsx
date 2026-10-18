import { Menu } from "lucide-react";
import type { ComponentType, ReactNode } from "react";
import { Badge } from "@/renderer/components/ui/badge";
import { Button } from "@/renderer/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from "@/renderer/components/ui/dropdown-menu";
import {
  Menubar,
  MenubarContent,
  MenubarGroup,
  MenubarItem,
  MenubarMenu,
  MenubarSeparator,
  MenubarShortcut,
  MenubarTrigger,
} from "@/renderer/components/ui/menubar";
import { Separator } from "@/renderer/components/ui/separator";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";
import { isMac } from "@/renderer/lib/platform";
import { cn } from "@/renderer/lib/utils";
import type { MenuEntry } from "@/shared/menu";
import logoUrl from "../../../brand/logo.svg";
import { useAppSettings } from "../store/app-settings";
import {
  MENU_SECTIONS,
  menuItemId,
  runMenuItem,
  shortcutLabel,
  type MenuItem,
} from "./menu-shortcuts";
import { WindowControls } from "./window-chrome";

function groups(items: MenuEntry[]): MenuItem[][] {
  const out: MenuItem[][] = [];
  let current: MenuItem[] = [];
  for (const item of items) {
    if (item.kind !== "separator") current.push(item);
    else if (current.length) {
      out.push(current);
      current = [];
    }
  }
  return current.length ? [...out, current] : out;
}

type MenuParts = {
  Group: ComponentType<{ children: ReactNode }>;
  Separator: ComponentType;
  Item: ComponentType<{ onClick: () => void; children: ReactNode }>;
  Shortcut: ComponentType<{ children: ReactNode }>;
};

const MENUBAR_PARTS: MenuParts = {
  Group: MenubarGroup,
  Separator: MenubarSeparator,
  Item: MenubarItem,
  Shortcut: MenubarShortcut,
};

const DROPDOWN_PARTS: MenuParts = {
  Group: DropdownMenuGroup,
  Separator: DropdownMenuSeparator,
  Item: DropdownMenuItem,
  Shortcut: DropdownMenuShortcut,
};

function SectionItems({ items, parts }: { items: MenuEntry[]; parts: MenuParts }) {
  const { Group, Separator: GroupSeparator, Item, Shortcut } = parts;
  return groups(items).map((group, i) => (
    <Group key={i}>
      {i > 0 ? <GroupSeparator /> : null}
      {group.map((entry) => (
        <Item key={menuItemId(entry)} onClick={() => runMenuItem(entry)}>
          {entry.label}
          {entry.accelerator ? <Shortcut>{shortcutLabel(entry.accelerator)}</Shortcut> : null}
        </Item>
      ))}
    </Group>
  ));
}

function WideMenu() {
  return (
    <Menubar className="hidden h-8 border-0 bg-transparent p-0 xl:flex">
      {MENU_SECTIONS.map((section) => (
        <MenubarMenu key={section.label}>
          <MenubarTrigger className="px-2 font-normal">{section.label}</MenubarTrigger>
          <MenubarContent className="min-w-56">
            <SectionItems items={section.items} parts={MENUBAR_PARTS} />
          </MenubarContent>
        </MenubarMenu>
      ))}
    </Menubar>
  );
}

function CompactMenu() {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        render={<Button variant="ghost" size="icon-sm" aria-label="Menu" className="xl:hidden" />}
      >
        <Menu />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="w-48">
        <DropdownMenuGroup>
          {MENU_SECTIONS.map((section) => (
            <DropdownMenuSub key={section.label}>
              <DropdownMenuSubTrigger>{section.label}</DropdownMenuSubTrigger>
              <DropdownMenuSubContent className="min-w-56">
                <SectionItems items={section.items} parts={DROPDOWN_PARTS} />
              </DropdownMenuSubContent>
            </DropdownMenuSub>
          ))}
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

export function AppBar({
  children,
  className,
  menu = true,
}: {
  children?: ReactNode;
  className?: string;
  menu?: boolean;
}) {
  return (
    <header
      className={cn(
        "app-drag bg-sidebar text-sidebar-foreground flex h-12 shrink-0 items-center border-b",
        isMac ? "pr-3 pl-21" : "pl-2",
        className,
      )}
    >
      <div className="flex h-full min-w-0 flex-1 items-center gap-2">
        <img src={logoUrl} alt="Demoflow" className="mx-1 size-5 shrink-0" draggable={false} />
        {isMac || !menu ? null : (
          <>
            <WideMenu />
            <CompactMenu />
            {children ? (
              <Separator orientation="vertical" className="mx-1 h-5 data-vertical:self-center" />
            ) : null}
          </>
        )}
        {children}
      </div>
      <ApiBadge />
      {isMac ? null : (
        <>
          <Separator orientation="vertical" className="mx-3 h-5 data-vertical:self-center" />
          <WindowControls />
        </>
      )}
    </header>
  );
}

function ApiBadge() {
  const on = useAppSettings((s) => s.settings?.api.enabled);
  const status = useAppSettings((s) => s.api);
  if (!on || !status) return null;
  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <button
            type="button"
            className="ml-2 shrink-0"
            onClick={() => void window.demoflow.openSettings("api")}
          />
        }
      >
        <Badge variant={status.listening ? "secondary" : "destructive"}>API</Badge>
      </TooltipTrigger>
      <TooltipContent>
        {status.listening
          ? "Scripts and agents on this computer can use the open project."
          : "The API is switched on but isn't running. Open its settings to see why."}
      </TooltipContent>
    </Tooltip>
  );
}

export function PageBar({ title }: { title: string }) {
  return (
    <AppBar>
      <span className="text-muted-foreground text-sm">{title}</span>
    </AppBar>
  );
}
