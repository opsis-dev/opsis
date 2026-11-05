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
