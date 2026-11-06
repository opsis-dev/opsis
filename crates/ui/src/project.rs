use indexmap::IndexMap;
use opsis_compile::compile::compile_workflow;
use opsis_compile::graph::{GraphIssue, IssueLevel, validate_workflow};
use opsis_compile::nodes::output::{CANVAS_SHORT_SIDE, Variant, variant_canvas};
use opsis_compile::vars::{PAUSE_MARK, Token, tokenize, var_token};
use opsis_qt::{IssueView, ProjectView, ShapeView, TextToken, TokenKind, VariableView};
use opsis_types::values::NodeResult;
use opsis_types::workflow::Workflow;

const MAX_PROBLEMS: usize = 30;

pub fn shapes() -> Vec<ShapeView> {
    Variant::ALL
        .into_iter()
        .map(|shape| {
            let size = variant_canvas(shape, CANVAS_SHORT_SIDE);
            ShapeView {
                label: crate::ui::plain(&shape),
                width: size.width,
                height: size.height,
            }
        })
        .collect()
}

pub fn variables(wf: &Workflow) -> Vec<VariableView> {
    wf.variables
        .iter()
        .map(|(name, value)| VariableView {
            name: name.clone(),
            value: value.clone(),
        })
        .collect()
}

pub fn issues(wf: &Workflow, results: &IndexMap<String, NodeResult>) -> Vec<IssueView> {
    let compiled = compile_workflow(wf, Some(results));
    issue_views(validate_workflow(wf, Some(&compiled)))
}

pub fn issue_views(issues: impl IntoIterator<Item = GraphIssue>) -> Vec<IssueView> {
    issues
        .into_iter()
        .take(MAX_PROBLEMS)
        .map(|issue| IssueView {
            error: issue.level == IssueLevel::Error,
            message: issue.message,
        })
        .collect()
}

pub struct Extras {
    pub fonts: Vec<String>,
    pub issues: Vec<IssueView>,
    pub running: bool,
    pub selected: u32,
}

pub const fn empty() -> ProjectView {
    ProjectView {
        name: String::new(),
        width: 0,
        height: 0,
        fps: 0,
        background: String::new(),
        background_hint: String::new(),
        shapes: Vec::new(),
        main_font: String::new(),
        heading_font: String::new(),
        code_font: String::new(),
        fonts: Vec::new(),
        variables: Vec::new(),
        issues: Vec::new(),
        running: false,
        selected: 0,
    }
}

pub fn view(wf: &Workflow, extras: Extras) -> ProjectView {
    let settings = &wf.settings;
    ProjectView {
        name: wf.name.clone(),
        width: settings.width,
        height: settings.height,
        fps: settings.fps,
        background: settings.background.clone().unwrap_or_default(),
        background_hint: settings.theme.background().to_owned(),
        shapes: shapes(),
        main_font: settings.fonts.main.clone(),
        heading_font: settings.fonts.heading.clone(),
        code_font: settings.fonts.code.clone(),
        fonts: extras.fonts,
        variables: variables(wf),
        issues: extras.issues,
        running: extras.running,
        selected: extras.selected,
    }
}

pub fn text_tokens(value: &str, pauses: bool) -> Vec<TextToken> {
    tokenize(value, pauses)
        .into_iter()
        .map(|token| match token {
            Token::Text(text) => TextToken {
                kind: TokenKind::Text,
                text,
            },
            Token::Var(name) => TextToken {
                kind: TokenKind::Var,
                text: name,
            },
            Token::Pause => TextToken {
                kind: TokenKind::Pause,
                text: String::new(),
            },
        })
        .collect()
}

pub fn join_tokens(tokens: &[TextToken]) -> String {
    tokens
        .iter()
        .map(|token| match token.kind {
            TokenKind::Var => var_token(&token.text),
            TokenKind::Pause => PAUSE_MARK.to_string(),
            _ => token.text.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn token() -> impl Strategy<Value = TextToken> {
        prop_oneof![
            "[a-z .,]{1,8}".prop_map(|text| TextToken {
                kind: TokenKind::Text,
                text
            }),
            "[A-Za-z][A-Za-z ]{0,10}[A-Za-z]".prop_map(|text| TextToken {
                kind: TokenKind::Var,
                text
            }),
            Just(TextToken {
                kind: TokenKind::Pause,
                text: String::new()
            }),
        ]
    }

    proptest! {
        #[test]
        fn joined_tokens_read_back_as_the_same_text(
            tokens in proptest::collection::vec(token(), 0..12)
        ) {
            let text = join_tokens(&tokens);
            prop_assert_eq!(join_tokens(&text_tokens(&text, true)), text);
        }
    }
}
