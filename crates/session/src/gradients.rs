use opsis_compile::gradient::{GradientEdit, GradientView};
pub fn gradient_view(css: &str) -> Option<GradientView> {
    opsis_compile::gradient::gradient_view(css)
}
pub fn edit_gradient(css: &str, edit: &GradientEdit) -> Option<GradientView> {
    opsis_compile::gradient::edit_gradient(css, edit)
}
