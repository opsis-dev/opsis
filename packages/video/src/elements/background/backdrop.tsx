import React from "react";
import { AbsoluteFill, getRemotionEnvironment, Img, staticFile } from "remotion";
import { backdropSource } from "#video/camera/lib/backdrop.ts";

const BackdropImage: React.FC<{ src: string }> = ({ src }) => {
  const style = { width: "100%", height: "100%", objectFit: "cover" } as const;
  return getRemotionEnvironment().isRendering ? (
    <Img src={src} style={style} />
  ) : (
    <img src={src} alt="" decoding="async" style={style} />
  );
};

export const Backdrop: React.FC<{ name?: string }> = ({ name }) => {
  const source = backdropSource(name);
  if (source.kind === "paint") return <AbsoluteFill style={{ background: source.css }} />;
  return (
    <AbsoluteFill>
      <BackdropImage src={source.kind === "image" ? source.url : staticFile(source.path)} />
    </AbsoluteFill>
  );
};
