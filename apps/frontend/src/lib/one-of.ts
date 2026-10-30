function optionFor<T extends string | number>(
  options: readonly T[],
  value: unknown,
): T | undefined {
  return options.find((o) => String(o) === value);
}

export function whenOneOf<T extends string | number>(
  options: readonly T[],
  apply: (value: T) => void,
) {
  return (value: unknown): void => {
    const picked = optionFor(options, value);
    if (picked !== undefined) apply(picked);
  };
}

export function whenFirstOneOf<T extends string | number>(
  options: readonly T[],
  apply: (value: T) => void,
) {
  return (picked: readonly unknown[]): void => whenOneOf(options, apply)(picked[0]);
}
