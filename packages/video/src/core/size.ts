export type Size = { width: number; height: number };
export type Box = { x: number; y: number; width: number; height: number };

export function containBox(outer: Size, aspect: number): Size {
  return outer.width / outer.height > aspect
    ? { width: outer.height * aspect, height: outer.height }
    : { width: outer.width, height: outer.width / aspect };
}
