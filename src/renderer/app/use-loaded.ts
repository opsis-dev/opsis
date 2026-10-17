import { useEffect, useEffectEvent, useState } from "react";
import { toast } from "sonner";
import { errorMessage } from "@/shared/errors";

export function useLoaded<T>(load: () => Promise<T>, failure: string): T | null {
  const [value, setValue] = useState<T | null>(null);
  const start = useEffectEvent(() =>
    load().then(setValue, (e: unknown) => toast.error(failure, { description: errorMessage(e) })),
  );
  useEffect(() => {
    void start();
  }, []);
  return value;
}
