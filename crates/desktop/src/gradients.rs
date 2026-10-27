use opsis_compile::gradient::{GradientEdit, GradientView};

#[tauri::command]
#[specta::specta]
pub fn gradient_view(css: String) -> Option<GradientView> {
    opsis_compile::gradient::gradient_view(&css)
}

#[tauri::command]
#[specta::specta]
pub fn edit_gradient(css: String, edit: GradientEdit) -> Option<GradientView> {
    opsis_compile::gradient::edit_gradient(&css, &edit)
}
