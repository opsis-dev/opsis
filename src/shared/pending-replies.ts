type Waiter<R> = { resolve: (result: R) => void; timer: ReturnType<typeof setTimeout> | null };

type Timeout<R> = { ms: number; result: R };

export function pendingReplies<R>() {
  const waiters = new Map<string, Waiter<R>>();

  const settle = (id: string, result: R): void => {
    const waiter = waiters.get(id);
    if (!waiter) return;
    waiters.delete(id);
    if (waiter.timer) clearTimeout(waiter.timer);
    waiter.resolve(result);
  };

  return {
    wait: (id: string, timeout?: Timeout<R>): Promise<R> =>
      new Promise((resolve) => {
        const timer = timeout ? setTimeout(() => settle(id, timeout.result), timeout.ms) : null;
        waiters.set(id, { resolve, timer });
      }),
    settle,
    settleAll: (result: R): void => {
      for (const id of waiters.keys()) settle(id, result);
    },
  };
}
