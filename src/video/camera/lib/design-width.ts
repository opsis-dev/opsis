const DESIGN_WIDTH = 1920;

export const designScaleFor = (width: number): number => (width > 0 ? width / DESIGN_WIDTH : 1);
