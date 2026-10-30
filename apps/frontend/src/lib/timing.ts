type Debounced<A extends unknown[]> = ((...args: A) => void) & { cancel: () => void };

export function debounce<A extends unknown[]>(fn: (...args: A) => void, ms: number): Debounced<A> {
  let timer: ReturnType<typeof setTimeout> | null = null;
  function cancel() {
    if (timer) clearTimeout(timer);
    timer = null;
  }
  function run(...args: A) {
    cancel();
    timer = setTimeout(() => {
      timer = null;
      fn(...args);
    }, ms);
  }
  return Object.assign(run, { cancel });
}

export function whenReady<T>(
  check: () => T | null,
  subscribe: (onChange: () => void) => () => void,
  ms: number,
): Promise<T | null> {
  const ready = check();
  if (ready !== null) return Promise.resolve(ready);
  return new Promise((resolve) => {
    function finish(value: T | null) {
      clearTimeout(timer);
      stop();
      resolve(value);
    }
    const timer = setTimeout(() => finish(null), ms);
    const stop = subscribe(() => {
      const value = check();
      if (value !== null) finish(value);
    });
  });
}
