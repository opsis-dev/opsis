# Native renderer: the transform table

How each construct the film's TSX writes becomes Rust that returns scene nodes ([native-renderer.md](native-renderer.md), Elements). One row per CSS property, value, tag and hook in that plan's Appendix: constructs, with its target. Paths are relative to the repository root; `Style` is `opsis_scene::style::Style`, whose builder methods are the CSS property names in snake case.

## Properties

| CSS (TSX key)                                                                      | Scene target                                                                                                                                         |
| ---------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `width`, `height`, `minWidth`, `minHeight`, `maxWidth`, `maxHeight`, `aspectRatio` | `.width()`, `.height()`, `.min_width()`, `.min_height()`, `.max_width()`, `.max_height()`, `.aspect_ratio()`                                         |
| `position`, `left`, `top`, `right`, `bottom`, `inset`, `zIndex`                    | `.position(Position)`, `.left()`…, `.inset()` (all four), `.z_index()`                                                                               |
| `boxSizing`                                                                        | `.box_sizing(BoxSizing)`; the scene's default is `border-box`, as `video.css` sets for every element                                                |
| `display`                                                                          | `.display(Display)`; `-webkit-box` with `WebkitLineClamp` is `Display::WebkitBox` and `.line_clamp()`                                                |
| `alignItems`, `alignSelf`, `alignContent`, `justifyContent`, `justifyItems`        | `.align_items(Align)` and the rest by name                                                                                                           |
| `placeItems`, `placeContent`                                                       | `.align_items()` and `.justify_items()`; `.align_content()` and `.justify_content()`                                                                 |
| `gap`, `rowGap`, `columnGap`                                                       | `.gap()` (both), `.row_gap()`, `.column_gap()`                                                                                                       |
| `flex`, `flexGrow`, `flexShrink`, `flexBasis`, `flexDirection`, `flexWrap`         | `.flex_grow()`, `.flex_shrink()`, `.flex_basis()` (the shorthand written out), `.flex_direction()`, `.flex_wrap()`                                   |
| `gridTemplateColumns`, `gridTemplateRows`, `gridColumn`, `gridRow`                 | `.grid_template_columns(Vec<Track>)`, `.grid_template_rows()`, `.grid_column((GridLine, GridLine))`, `.grid_row()`                                   |
| `background`, `backgroundImage`, `backgroundSize`, `backgroundPosition`, `backgroundRepeat`, `backgroundBlendMode` | `.background(Vec<ImageLayer>)`, one `ImageLayer` per layer, top first, with `size`, `position`, `repeat`, `clip`, `origin`, `blend` |
| `backgroundColor`                                                                  | `.background_color(Color)`                                                                                                                           |
| `backgroundClip: "text"`                                                           | `.background_clip_text(true)`, set after the background like the CSS (the shorthand resets it)                                                     |
| `borderRadius` and its corners                                                     | `.border_radius(Corners<(Length, Length)>)`, `.border_radius_all()`                                                                                  |
| `border`, its sides, widths, styles and colours                                    | `.border(Edges<BorderSide>)`; a side with no colour takes `currentColor`                                                                             |
| `outline`, `outlineOffset`                                                         | `.outline(Outline)`                                                                                                                                  |
| `padding`, `margin`                                                                | `.padding(Edges<Length>)`, `.margin(Edges<Length>)`                                                                                                  |
| `overflow`, `overflowX`, `overflowY`                                               | `.overflow()` (both), `.overflow_x()`, `.overflow_y()`                                                                                               |
| `boxShadow`                                                                        | `.box_shadow(Vec<Shadow>)`                                                                                                                           |
| `objectFit`, `objectPosition`, `imageRendering`                                    | `.object_fit()`, `.object_position(Position2)`, `.image_rendering()`                                                                                 |
| `opacity`                                                                          | `.opacity()`                                                                                                                                         |
| `transform`                                                                        | `.transform(Vec<Transform>)`, the functions in order                                                                                                 |
| `translate`, `rotate`, `scale`                                                     | `.translate(x, y)`, `.rotate(Angle)`, `.scale(x, y)`; `.translate3d()`, `.rotate3d()`, `.scale3d()` for three values                                |
| `transformOrigin`, `perspective`, `perspectiveOrigin`, `transformStyle`, `backfaceVisibility` | `.transform_origin((x, y, z))`, `.perspective(Px)`, `.perspective_origin()`, `.transform_style()`, `.backface_visibility()`               |
| `filter`                                                                           | `.filter(Vec<Filter>)`; `url(#id)` is `Filter::Url` naming an SVG filter anywhere in the scene                                                        |
| `backdropFilter`                                                                   | `.backdrop_filter(Vec<Filter>)`                                                                                                                      |
| `clipPath`                                                                         | `.clip_path(ClipPath)`: `inset()`, `circle()`, `ellipse()`, `polygon()`, `path()`, `url(#id)`                                                        |
| `maskImage`, `maskSize`, `maskPosition`, `maskRepeat`, `maskMode`                  | `.mask(Vec<MaskLayer>)`, the layer's origin and clip the border box                                                                                  |
| `mixBlendMode`, `isolation`                                                        | `.mix_blend_mode(BlendMode)`, `.isolation(true)`                                                                                                     |
| `visibility`                                                                       | `.visibility(Visibility)`                                                                                                                            |
| `color`                                                                            | `.color(Color)`; `currentColor` inherits                                                                                                             |
| `fontFamily`, `fontSize`, `fontWeight`, `fontStyle`, `lineHeight`                  | `.font_family(Vec<String>)` (role names such as `df-main` resolve as today), `.font_size()`, `.font_weight()`, `.font_style()`, `.line_height(LineHeight)` |
| `letterSpacing`, `wordSpacing`, `textAlign`, `textTransform`                       | `.letter_spacing()`, `.word_spacing()`, `.text_align()`, `.text_transform()`                                                                         |
| `whiteSpace`, `wordBreak`, `overflowWrap`, `textWrap`, `tabSize`                   | `.white_space()`, `.word_break()`, `.overflow_wrap()`, `.text_wrap()`, `.tab_size()`                                                                 |
| `textOverflow`                                                                     | `.text_overflow()`                                                                                                                                   |
| `fontVariantNumeric`, `fontFeatureSettings`, `fontVariationSettings`               | `.font_variant_numeric(NumericVariant)`, `.font_feature_settings(Vec<(Tag, i32)>)`, `.font_variation_settings(Vec<(Tag, Num)>)`                       |
| `textShadow`, `WebkitTextStroke`, `paintOrder`, `textDecoration`                   | `.text_shadow(Vec<Shadow>)`, `.text_stroke(TextStroke)`, `.paint_order()`, `.text_decoration(TextDecoration)`                                        |
| `verticalAlign`                                                                    | `.vertical_align(VerticalAlign)`                                                                                                                     |
| `containerType`                                                                    | `.container_type(ContainerType)`                                                                                                                     |
| `pointerEvents`, `userSelect`, `cursor`, `willChange`                              | dropped: nothing in a film reads them                                                                                                                |

## Values

| Value                                         | Scene target                                                                                          |
| --------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| numbers in px, `em`, `rem`, `%`, `vw`, `vh`, `lh`, `ch`, `ex` | `Length::px()`, `Length::em()`, `Length::Rem`, `Length::percent()`, `Length::Vw`… |
| `calc()`, `min()`, `max()`, `clamp()`         | `Length::sum()`, `.scaled()`, `Length::min()`, `Length::max()`, `Length::clamp()`                     |
| container query units                         | `Length::Container(ContainerUnit, Num)`                                                               |
| hex, `rgb()`, `hsl()`, `hwb()`                | `Color::rgb8()`, `Color::srgb_alpha()`, `Color::Components` in `Hsl` or `Hwb` with CSS percentages   |
| `oklch()`, `oklab()`, `lab()`, `lch()`, `color(display-p3 …)` | `Color::oklch()`, `Color::Components` in its space                                         |
| `color-mix()`                                 | `Color::mix(Interpolation, (a, share), (b, share))`                                                   |
| `linear-gradient()` and the rest              | `Gradient::{Linear, Radial, Conic}` with `repeating` and `interpolation`; conic stops in degrees as percentages of a turn |
| `url(…)` for pictures                         | `Image::Url(AssetRef)` with the same `app:` or project ref the film uses                              |

## Tags

| Tag                                                              | Scene target                                                                                      |
| ---------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| `div`, `span`                                                    | `div(Style)`, `span(Style)`; `p`, `strong`, `em` and the rest as these with their `video.css` styles |
| `img`                                                            | `img(AssetRef, Style)`                                                                            |
| `svg` and every SVG tag and filter primitive                     | `svg(SvgElement, Style)`: the tree with its attributes as written, drawn through `usvg`            |
| `canvas` with a 2D context                                       | `drawing(Drawing, Style)`: the context's calls as `Command`s                                      |
| `foreignObject`                                                  | not drawn: the one use (the terminal under the lens) becomes an HTML element with `filter: url(#…)` |
| `video`, footage frames                                          | `Kind::Footage(FootageFrame)`                                                                     |
| Lottie                                                           | `Kind::Lottie(LottieFrame)`, fitted with `.object_fit()`                                          |
| shader passes, 3D scenes                                         | `Kind::Effect(Effect)` with its input nodes ([native-renderer.md](native-renderer.md), Effects and 3D) |

## Hooks and page calls

| Construct                                                          | Target                                                                                                          |
| ------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------- |
| `useFrame()`, `useAuthoredFrame()`                                 | `At::clock` (the authored clock for elements registered with it)                                                |
| `useMemo`                                                          | plain computation in the draw function; costly pure work through `#[comemo::memoize]` helpers                  |
| `useState`, `useRef`, `useEffect`, `useLayoutEffect`               | derived from the clock: the value a frame needs is computed from the frame, as the film's rule already requires |
| `holdRender`, `useHeldLoad`, `untilPainted`, `document.fonts.ready` | the resolve pass: fonts, pictures, SVG, Lottie and footage are loaded before the first frame                    |
| `measureText`                                                      | `Measure::text` on the measurer the library passes to every element                                             |
| `getBoundingClientRect`, `offsetWidth`, `ResizeObserver`           | `Measure::size`, which lays out a node for an available width                                                   |
| `getBBox`, `getTotalLength`                                        | `usvg`'s bounding boxes and `kurbo` arc lengths over the same path data                                         |
| `random(seed)`, `spring`, `interpolate`, `interpolateColors`        | `opsis_scene::{random, spring, interpolate}`, colours through `opsis-paint`'s `color` (OKLCH)                   |
| rough.js                                                           | `roughr` with rough.js's Park–Miller generator                                                                  |
| `simplex-noise`                                                    | a port with the same output for the same seed                                                                   |
