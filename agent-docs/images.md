# Image nodes

The Image category: how pictures are loaded and passed between nodes, the pure planning functions the executors and cards share, and text drawn into images. Node definitions are in `crates/compile/src/nodes/image.rs`, executors in `crates/image/src/`. See also [nodes.md](nodes.md), [fonts.md](fonts.md) and [AGENTS.md](../AGENTS.md).

## Code map

| Concern                       | Where                                                                                                                                                                                                                                       |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Definitions and pure planning | `crates/compile/src/nodes/image.rs` → `resized_size`, `crop_rect`, `rotated_size`, `area_rect`, `tilt_corners`, `tilted_size`, `outline_padding`, `grid_plan`, `before_after_plan`, `magnify_plan`, `compare_threshold`, `framed_placement` |
| Pixel operations              | `crates/image/src/ops.rs` → `on_each_depth`, `lit_mask`, `content_rect`, `clear_colour`, `paint_gradient`, `changed_boxes`                                                                                                                  |
| Executors                     | `crates/image/src/executors.rs`                                                                                                                                                                                                             |
| Annotate marks                | `crates/image/src/marks.rs`                                                                                                                                                                                                                 |
| Text in images                | `crates/image/src/text.rs` (`family_for`, `face_for`, `ab_glyph`)                                                                                                                                                                           |

## Rules

- **Every image is loaded upright (its EXIF orientation applied) and in sRGB (an embedded ICC profile converted with `moxcms`), keeping 16 bits and transparency.**
- **Between nodes an image is a PNG**; Image format makes PNG, JPEG (see-through parts filled with a background), lossless WebP or AVIF.
- **Inputs are file ports and outputs are images**, so imported pictures, screenshots, grabbed frames and Sound picture all connect.
- **Areas drawn on a picture become pixels through one function, `area_rect`, kept inside the picture.**
- **Operations that change pixels keep a 16-bit picture 16-bit (`on_each_depth`).**
- **The sizes a node makes are pure functions shared by the executor and the card**, which warns when an image is enlarged more than 2× or a crop leaves under 64 px.
- **The Inspector picture of an image node is its connected input.**
- **Text in images (labels, step numbers, Annotate's text) picks its face like 3D text's** (`family_for`, `face_for`): WOFF and WOFF2 unwrapped with `wuff`, the weight set on a variable font, `ab_glyph` for the outlines. Such nodes set `uses_project_fonts` ([runs.md](runs.md)).
- **Magnify's Size from picks which two of the area, the view and the zoom are set, and `magnify_plan` works out the third**: it grows two boxes of different shapes to the view's proportions, makes a circle cover the whole area drawn but stay inside the box drawn for the view, grows a view drawn smaller than its area so it never shrinks, and keeps the result inside the picture.
- **Annotate draws each mark from its own coverage over its bounding box**, and steps count up in list order.
- **Outline warns when the image has nothing see-through.**
