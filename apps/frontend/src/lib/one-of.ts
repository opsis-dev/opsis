const optionFor = <T extends string | number>(
  options: readonly T[],
  value: unknown,
): T | undefined => options.find((o) => String(o) === value);

export const whenOneOf =
  <T extends string | number>(options: readonly T[], apply: (value: T) => void) =>
  (value: unknown): void => {
    const picked = optionFor(options, value);
    if (picked !== undefined) apply(picked);
  };

export const whenFirstOneOf =
  <T extends string | number>(options: readonly T[], apply: (value: T) => void) =>
  (picked: readonly unknown[]): void =>
    whenOneOf(options, apply)(picked[0]);
