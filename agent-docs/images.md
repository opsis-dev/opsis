# Image nodes

The Image category: how pictures are loaded and passed between nodes, the pure planning functions the executors and cards share, and text drawn into images. Node definitions are in `crates/compile/src/nodes/image.rs`, executors in `crates/image/src/`. See also [nodes.md](nodes.md), [fonts.md](fonts.md) and [AGENTS.md](../AGENTS.md).

## Code map

| Concern                       | Where                                                                                                                                                                                                                                             |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Definitions and pure planning | `crates/compile/src/nodes/image.rs` → `resized_size`, `crop_rect`, `rotated_size`, `area_rect`, `tilt_corners`, `tilted_size`, `outline_padding`, `grid_plan`, `before_after_plan`, `magnify_plan`, `compare_threshold`, `framed_placement`       |
| Loading                       | `crates/image/src/load.rs` → `load_image`                                                                                                                                                                                                         |
| Pixel operations              | `crates/image/src/ops.rs` → `is_deep`, `in_depth`, `on_each_depth`, `lit_mask`, `content_rect`, `clear_colour`, `paint_gradient`, `changed_boxes`, `encode`                                                                                       |
| Executors                     | `crates/image/src/executors.rs` → `image_executors`; the shared node builder `crates/compile/src/nodes/image.rs` → `image_node`                                                                                                                   |
| Annotate marks                | `crates/image/src/marks.rs`                                                                                                                                                                                                                       |
| Text in images                | `crates/image/src/text.rs` → `load_font`, `measure`, `draw_text` (`ab_glyph`); the face from `family_for` (`crates/compile/src/fonts/mod.rs`) and `face_for` (`crates/compile/src/elements/prepared.rs`), picked in `executors.rs` → `font_named` |
| Inspector picture             | `crates/compile/src/inspector.rs` → `input_picture`                                                                                                                                                                                               |

## Rules

- **Every image is loaded upright (its EXIF orientation applied) and in sRGB (an embedded ICC profile converted with `moxcms`), keeping 16 bits and transparency** (`load_image`).
- **Between nodes an image is a PNG, except the file Image format writes**: PNG, JPEG (see-through parts filled with a background), lossless WebP or AVIF at its quality and speed settings (`encode`).
  - JPEG, WebP and AVIF are written in 8 bits, so only PNG keeps a 16-bit picture.
- **Inputs are `media` ports and outputs are images** (`image_node`), so imported pictures, screenshots, grabbed frames and Sound picture all connect.
- **Areas drawn on a picture become pixels through one function, `area_rect`, kept inside the picture**: Redact, Spotlight, Crop and Magnify use it; Annotate places its marks itself (`marks.rs`).
- **Operations that change pixels keep a 16-bit picture 16-bit**: they check the depth (`is_deep`) and work at that depth, or run one pixel function at either depth (`on_each_depth`, `in_depth`).
- **The sizes a node makes are pure functions shared by the executor and the card**, which warns when Resize enlarges an image past `SOFT_UPSCALE` (the run logs it too) or a crop leaves less than `SMALL_CROP`.
- **The Inspector picture of an image node is its connected input** (`input_picture`).
- **Text in images (labels, step numbers, Annotate's text) picks its face like 3D text's** (`family_for`, `face_for`, in `font_named`): `load_font` unwraps WOFF and WOFF2 with `wuff` and sets the weight on a variable font, and `ab_glyph` draws the outlines. Such nodes set `uses_project_fonts` ([runs.md](runs.md)).
- **Magnify's Size from picks which two of the area, the view and the zoom are set, and `magnify_plan` works out the third**: it grows two boxes of different shapes to the view's proportions, makes a circle cover the whole area drawn but stay inside the box drawn for the view, grows a view drawn smaller than its area so it never shrinks, and keeps the result inside the picture.
- **Annotate draws each mark from its own coverage over its bounding box**, and steps count up in list order from its First step number setting.
- **Outline logs a warning when it runs on an image with nothing see-through** (`has_see_through`).
