export type Extrapolate = "clamp" | "extend" | "identity" | "wrap";

export type InterpolateOptions = {
  easing?: (t: number) => number;
  extrapolateLeft?: Extrapolate;
  extrapolateRight?: Extrapolate;
  output?: "linear" | "perceptual-scale";
};

function signedArea(size: number): number {
  return Math.sign(size) * size * size;
}

function sizeOfArea(area: number): number {
  return Math.sign(area) * Math.sqrt(Math.abs(area));
}

function checkRanges(inputRange: readonly number[], outputRange: readonly number[]) {
  if (inputRange.length < 2 || inputRange.length !== outputRange.length)
    throw new Error("interpolate needs two ranges of the same length, at least 2 long.");
  for (let i = 1; i < inputRange.length; i++) {
    if (!((inputRange[i] ?? 0) > (inputRange[i - 1] ?? 0)))
      throw new Error("interpolate's input range has to rise strictly.");
  }
}

function wrapped(input: number, first: number, last: number): number {
  const span = last - first;
  return first + ((((input - first) % span) + span) % span);
}

export function interpolate(
  input: number,
  inputRange: readonly number[],
  outputRange: readonly number[],
  {
    easing,
    extrapolateLeft = "extend",
    extrapolateRight = "extend",
    output = "linear",
  }: InterpolateOptions = {},
): number {
  checkRanges(inputRange, outputRange);
  const last = inputRange.length - 1;
  const first = inputRange[0] ?? 0;
  const end = inputRange[last] ?? 0;
  let x = input;
  if (x < first) {
    if (extrapolateLeft === "clamp") return outputRange[0] ?? 0;
    if (extrapolateLeft === "identity") return input;
    if (extrapolateLeft === "wrap") x = wrapped(x, first, end);
  }
  if (x > end) {
    if (extrapolateRight === "clamp") return outputRange[last] ?? 0;
    if (extrapolateRight === "identity") return input;
    if (extrapolateRight === "wrap") x = wrapped(x, first, end);
  }
  let segment = 1;
  while (segment < last && x > (inputRange[segment] ?? 0)) segment++;
  const a = inputRange[segment - 1] ?? 0;
  const b = inputRange[segment] ?? 0;
  const from = outputRange[segment - 1] ?? 0;
  const to = outputRange[segment] ?? 0;
  const t = (x - a) / (b - a);
  const eased = easing ? easing(t) : t;
  if (output === "perceptual-scale") {
    const [a2, b2] = [signedArea(from), signedArea(to)];
    return sizeOfArea(a2 + (b2 - a2) * eased);
  }
  return from + (to - from) * eased;
}
