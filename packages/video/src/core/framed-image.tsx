import { AbsoluteFill, Img } from "@opsis/runtime/parts.tsx";
import type React from "react";
import { WindowFrame } from "#video/camera/window-frame.tsx";
import { LaptopFrame } from "#video/elements/device/laptop-frame.tsx";
import { PhoneFrame } from "#video/elements/device/phone-frame.tsx";
import type { ElementSettings, ImageTilt } from "#video/generated/spec.ts";
import DEFAULT_PROPS from "#video/registry/default-props.generated.json";

type FramedImageProps = ElementSettings["framed-image"];

function Ground({ props }: { props: FramedImageProps }) {
  if (props.backgroundImage) {
    return (
      <AbsoluteFill>
        <Img
          src={props.backgroundImage}
          style={{ width: "100%", height: "100%", objectFit: "cover" }}
        />
      </AbsoluteFill>
    );
  }
  return props.background ? <AbsoluteFill style={{ background: props.background }} /> : null;
}

function tiltTransform(tilt: ImageTilt | undefined, longSide: number): string | undefined {
  if (!tilt) return undefined;
  return `perspective(${tilt.perspective * longSide}px) rotateY(${-tilt.turn}deg) rotateX(${tilt.tilt}deg) rotateZ(${tilt.lean}deg)`;
}

function shadowOf(props: FramedImageProps): string | undefined {
  if (props.shadowOpacity <= 0) return undefined;
  const alpha = Math.round(Math.min(1, props.shadowOpacity) * 100);
  return `0 ${props.shadowDrop}px ${props.shadowSoftness}px color-mix(in srgb, ${props.shadowColor} ${alpha}%, transparent)`;
}

function Picture({ src }: { src: string }) {
  return (
    <Img
      src={src}
      style={{ width: "100%", height: "100%", objectFit: "cover", objectPosition: "top" }}
    />
  );
}

function OnDevice({ props }: { props: FramedImageProps }) {
  const screen = <Picture src={props.src} />;
  if (props.device === "phone") {
    return (
      <PhoneFrame
        {...(DEFAULT_PROPS["phone-frame"] as unknown as ElementSettings["phone-frame"])}
        floatLoop={false}
      >
        {screen}
      </PhoneFrame>
    );
  }
  return (
    <LaptopFrame
      {...(DEFAULT_PROPS["laptop-frame"] as unknown as ElementSettings["laptop-frame"])}
      entrance="none"
      finale="none"
      floatLoop={false}
      showNotch={false}
    >
      {screen}
    </LaptopFrame>
  );
}

function Placed({ props, children }: { props: FramedImageProps; children: React.ReactNode }) {
  return (
    <div
      style={{
        position: "absolute",
        left: props.x,
        top: props.y,
        width: props.width,
        height: props.height,
        transform: tiltTransform(props.tilt, Math.max(props.width, props.height)),
        transformOrigin: "50% 50%",
        borderRadius: props.radius,
        boxShadow: shadowOf(props),
      }}
    >
      {children}
    </div>
  );
}

export function FramedImage(props: FramedImageProps) {
  if (!props.src) return null;
  return (
    <AbsoluteFill>
      <Ground props={props} />
      {props.device ? (
        <OnDevice props={props} />
      ) : (
        <Placed props={props}>
          {props.window ? (
            <WindowFrame chrome windowStyle={props.window} theme={props.theme} addressBar={false}>
              <Picture src={props.src} />
            </WindowFrame>
          ) : (
            <div
              style={{
                position: "absolute",
                inset: 0,
                borderRadius: props.radius,
                overflow: "hidden",
              }}
            >
              <Picture src={props.src} />
            </div>
          )}
        </Placed>
      )}
    </AbsoluteFill>
  );
}
