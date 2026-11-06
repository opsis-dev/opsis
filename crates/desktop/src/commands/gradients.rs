use opsis_compile::gradient::{GradientEdit, GradientView};

#[tauri::command]
#[specta::specta]
pub fn gradient_view(css: String) -> Option<GradientView> {
    opsis_session::gradients::gradient_view(&css)
}

#[tauri::command]
#[specta::specta]
pub fn edit_gradient(css: String, edit: GradientEdit) -> Option<GradientView> {
    opsis_session::gradients::edit_gradient(&css, &edit)
}
