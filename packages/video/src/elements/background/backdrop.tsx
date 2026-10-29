import { AbsoluteFill, getRemotionEnvironment, Img, staticFile } from "remotion";
import { backdropSource } from "#video/camera/lib/backdrop.ts";

function BackdropImage({ src }: { src: string }) {
  const style = { width: "100%", height: "100%", objectFit: "cover" } as const;
  return getRemotionEnvironment().isRendering ? (
    <Img src={src} style={style} />
  ) : (
    <img src={src} alt="" decoding="async" style={style} />
  );
}

export function Backdrop({ name }: { name?: string }) {
  const source = backdropSource(name);
  if (source.kind === "paint") return <AbsoluteFill style={{ background: source.css }} />;
  return (
    <AbsoluteFill>
      <BackdropImage src={source.kind === "image" ? source.url : staticFile(source.path)} />
    </AbsoluteFill>
  );
}
