use opsis_scene::css::font_family;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontRole {
    Main,
    Heading,
    Code,
}

const fn role_stack(role: FontRole) -> &'static str {
    match role {
        FontRole::Main => "\"df-main\"",
        FontRole::Heading => "\"df-heading\"",
        FontRole::Code => "\"df-code\"",
    }
}

pub fn font_or(family: &str, role: FontRole) -> Vec<String> {
    if family.is_empty() {
        font_family(role_stack(role))
    } else {
        font_family(family)
    }
}
