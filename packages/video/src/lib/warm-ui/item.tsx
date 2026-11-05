import { createContext, type ReactElement, type ReactNode, useContext } from "react";

type ItemHost = (index: number, node: ReactElement) => ReactNode;

const ItemHostContext = createContext<ItemHost | null>(null);

const ItemHostProvider = ItemHostContext.Provider;

export function Item({
  index,
  primary = true,
  children,
}: {
  index: number;
  primary?: boolean;
  children: ReactElement;
}): ReactNode {
  const host = useContext(ItemHostContext);
  return host && primary ? host(index, children) : children;
}
