import { useEffect, useLayoutEffect, useRef, useState } from "react";

export function useSettledAfterKeyChange<T>(value: T, key: string, ms: number): T {
  const latest = useRef(value);
  const [settled, setSettled] = useState({ key, value });
  useLayoutEffect(() => {
    latest.current = value;
  });
  useEffect(() => {
    const timer = setTimeout(
      () => setSettled((prev) => (prev.key === key ? prev : { key, value: latest.current })),
      ms,
    );
    return () => clearTimeout(timer);
  }, [key, ms]);
  return settled.value;
}
