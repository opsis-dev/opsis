const REFERENCE_WIDTH = 1080;
const REFERENCE_HEIGHT = 1080;
const HORIZONTAL_SAFE = 80;
const VERTICAL_SAFE = 100;

export type SafeAreaPadding = {
  paddingLeft: number;
  paddingRight: number;
  paddingTop: number;
  paddingBottom: number;
};

export function getSafeAreaPadding({
  width,
  height,
}: {
  width: number;
  height: number;
}): SafeAreaPadding {
  const horizontal = Math.round((HORIZONTAL_SAFE / REFERENCE_WIDTH) * width);
  const vertical = Math.round((VERTICAL_SAFE / REFERENCE_HEIGHT) * height);

  return {
    paddingLeft: horizontal,
    paddingRight: horizontal,
    paddingTop: vertical,
    paddingBottom: vertical,
  };
}

export function scaleFont(
  sizeAt1080: number,
  width: number,
  referenceWidth = REFERENCE_WIDTH,
): number {
  return Math.round(sizeAt1080 * (width / referenceWidth));
}

