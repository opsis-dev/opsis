import { publicFile } from "@opsis/runtime/assets.ts";
import { Filtered } from "#oracle/filter-parts.tsx";
import { Preloaded } from "#oracle/parts.tsx";

const PHOTO = publicFile("placeholders/launch-earth.jpg");

export function Case() {
  return (
    <Preloaded urls={[PHOTO]}>
      <Filtered
        filters={[
          [
            "oracle-image-plain",
            <feImage
              key="image"
              href={PHOTO}
              x="0"
              y="0"
              width="136"
              height="148"
              preserveAspectRatio="xMidYMid slice"
            />,
          ],
          [
            "oracle-image-in",
            <>
              <feImage
                key="image"
                href={PHOTO}
                x="0"
                y="0"
                width="136"
                height="148"
                preserveAspectRatio="xMidYMid slice"
                result="photo"
              />
              <feComposite key="in" in="photo" in2="SourceGraphic" operator="in" />
            </>,
          ],
          [
            "oracle-image-displace",
            <>
              <feImage
                key="image"
                href={PHOTO}
                x="0"
                y="0"
                width="136"
                height="148"
                preserveAspectRatio="none"
                result="map"
              />
              <feDisplacementMap
                key="displace"
                in="SourceGraphic"
                in2="map"
                scale="20"
                xChannelSelector="R"
                yChannelSelector="B"
              />
            </>,
          ],
        ]}
      />
    </Preloaded>
  );
}
