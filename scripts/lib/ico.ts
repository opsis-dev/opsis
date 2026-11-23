const HEADER_BYTES = 6;
const ENTRY_BYTES = 16;
const COLOUR_PLANES = 1;
const BITS_PER_PIXEL = 32;
const ICON_TYPE = 1;
const LARGEST_SIDE = 256;

export const ICO_SIZES = [16, 24, 32, 48, 64, 128, 256];

export type IcoImage = { size: number; png: Uint8Array };

export function ico(images: IcoImage[]): Uint8Array {
  const header = new DataView(new ArrayBuffer(HEADER_BYTES));
  header.setUint16(2, ICON_TYPE, true);
  header.setUint16(4, images.length, true);
  const entries = new DataView(new ArrayBuffer(ENTRY_BYTES * images.length));
  let offset = HEADER_BYTES + ENTRY_BYTES * images.length;
  images.forEach(({ size, png }, index) => {
    const at = index * ENTRY_BYTES;
    const side = size >= LARGEST_SIDE ? 0 : size;
    entries.setUint8(at, side);
    entries.setUint8(at + 1, side);
    entries.setUint16(at + 4, COLOUR_PLANES, true);
    entries.setUint16(at + 6, BITS_PER_PIXEL, true);
    entries.setUint32(at + 8, png.length, true);
    entries.setUint32(at + 12, offset, true);
    offset += png.length;
  });
  const file = new Uint8Array(offset);
  file.set(new Uint8Array(header.buffer), 0);
  file.set(new Uint8Array(entries.buffer), HEADER_BYTES);
  let at = HEADER_BYTES + ENTRY_BYTES * images.length;
  for (const { png } of images) {
    file.set(png, at);
    at += png.length;
  }
  return file;
}
