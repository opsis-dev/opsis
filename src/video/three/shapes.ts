import {
  Box3,
  BufferAttribute,
  Shape,
  ShapeGeometry,
  Vector3,
  type BufferGeometry,
  type Object3D,
} from "three";

export function roundedRect(width: number, height: number, radius: number): Shape {
  const r = Math.min(radius, width / 2, height / 2);
  const x = -width / 2;
  const y = -height / 2;
  const shape = new Shape();
  shape.moveTo(x + r, y);
  shape.lineTo(x + width - r, y);
  shape.quadraticCurveTo(x + width, y, x + width, y + r);
  shape.lineTo(x + width, y + height - r);
  shape.quadraticCurveTo(x + width, y + height, x + width - r, y + height);
  shape.lineTo(x + r, y + height);
  shape.quadraticCurveTo(x, y + height, x, y + height - r);
  shape.lineTo(x, y + r);
  shape.quadraticCurveTo(x, y, x + r, y);
  return shape;
}

export function faceGeometry(shape: Shape, width: number, height: number): BufferGeometry {
  const geometry = new ShapeGeometry(shape, 12);
  const position = geometry.getAttribute("position");
  const uv = new Float32Array(position.count * 2);
  for (let i = 0; i < position.count; i++) {
    uv[i * 2] = position.getX(i) / width + 0.5;
    uv[i * 2 + 1] = position.getY(i) / height + 0.5;
  }
  geometry.setAttribute("uv", new BufferAttribute(uv, 2));
  return geometry;
}

export function fitToSize(object: Object3D, size: number): void {
  object.updateMatrixWorld(true);
  const box = new Box3().setFromObject(object);
  const center = box.getCenter(new Vector3());
  const extent = box.getSize(new Vector3());
  const largest = Math.max(extent.x, extent.y, extent.z) || 1;
  const scale = size / largest;
  object.scale.multiplyScalar(scale);
  object.position.sub(center.multiplyScalar(scale));
}

export const fittedSize = (width: number, height: number, longest: number) =>
  width >= height
    ? { width: longest, height: (longest * height) / width }
    : { width: (longest * width) / height, height: longest };
