use std::fmt::Write;

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::look;

const MARKER_WIDTH: u32 = 22;
const MONO: &str = "font-family: 'JetBrains Mono'; font-size: 12px;";

fn escape(text: &str, out: &mut String) {
    for character in text.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(character),
        }
    }
}

const fn heading_size(level: HeadingLevel) -> u32 {
    match level {
        HeadingLevel::H1 => 16,
        HeadingLevel::H2 => 15,
        _ => 13,
    }
}

fn open(tag: &Tag<'_>, lists: &mut Vec<Option<u64>>, out: &mut String) {
    match tag {
        Tag::Paragraph => out.push_str("<p>"),
        Tag::Heading { level, .. } => {
            let _ = write!(
                out,
                "<p style=\"font-size: {}px; font-weight: 600;\">",
                heading_size(*level)
            );
        }
        Tag::BlockQuote(_) => {
            let _ = write!(
                out,
                "<blockquote style=\"color: {};\">",
                look::token("muted-foreground")
            );
        }
        Tag::CodeBlock(_) => {
            let _ = write!(
                out,
                "<table width=\"100%\" cellpadding=\"8\" cellspacing=\"0\" style=\"background-color: {}; margin-top: 10px; margin-bottom: 10px;\"><tr><td><pre style=\"{MONO} white-space: pre-wrap; margin: 0px;\">",
                look::token("background")
            );
        }
        Tag::List(start) => {
            lists.push(*start);
            out.push_str(
                "<table cellspacing=\"0\" cellpadding=\"0\" style=\"margin-top: 10px; margin-bottom: 6px;\">",
            );
        }
        Tag::Item => {
            let marker = match lists.last_mut() {
                Some(Some(number)) => {
                    let shown = format!("{number}.");
                    *number += 1;
                    shown
                }
                _ => "•".to_owned(),
            };
            let _ = write!(
                out,
                "<tr><td width=\"{MARKER_WIDTH}\" style=\"color: {}; padding-left: 4px;\">{marker}</td><td style=\"padding-bottom: 4px;\">",
                look::token("muted-foreground")
            );
        }
        Tag::Emphasis => out.push_str("<i>"),
        Tag::Strong => out.push_str("<b>"),
        Tag::Strikethrough => out.push_str("<s>"),
        Tag::Link { dest_url, .. } => {
            out.push_str("<a href=\"");
            escape(dest_url, out);
            out.push_str("\">");
        }
        Tag::Table(_) => {
            let _ = write!(
                out,
                "<table cellpadding=\"4\" cellspacing=\"0\" border=\"1\" style=\"border-color: {}; margin-bottom: 8px;\">",
                look::token("border")
            );
        }
        Tag::TableHead | Tag::TableRow => out.push_str("<tr>"),
        Tag::TableCell => out.push_str("<td>"),
        _ => {}
    }
}

fn close(tag: TagEnd, lists: &mut Vec<Option<u64>>, out: &mut String) {
    match tag {
        TagEnd::Paragraph | TagEnd::Heading(_) => out.push_str("</p>"),
        TagEnd::BlockQuote(_) => out.push_str("</blockquote>"),
        TagEnd::CodeBlock => out.push_str("</pre></td></tr></table>"),
        TagEnd::List(_) => {
            lists.pop();
            out.push_str("</table>");
        }
        TagEnd::Item => out.push_str("</td></tr>"),
        TagEnd::Emphasis => out.push_str("</i>"),
        TagEnd::Strong => out.push_str("</b>"),
        TagEnd::Strikethrough => out.push_str("</s>"),
        TagEnd::Link => out.push_str("</a>"),
        TagEnd::Table => out.push_str("</table>"),
        TagEnd::TableHead | TagEnd::TableRow => out.push_str("</tr>"),
        TagEnd::TableCell => out.push_str("</td>"),
        _ => {}
    }
}

pub fn html(text: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    let mut out = String::new();
    let mut lists = Vec::new();
    for event in Parser::new_ext(text, options) {
        match event {
            Event::Start(tag) => open(&tag, &mut lists, &mut out),
            Event::End(tag) => close(tag, &mut lists, &mut out),
            Event::Text(text) | Event::Html(text) | Event::InlineHtml(text) => {
                escape(&text, &mut out);
            }
            Event::Code(code) => {
                let _ = write!(
                    out,
                    "<span style=\"{MONO} background-color: {};\">",
                    look::token("muted")
                );
                escape(&code, &mut out);
                out.push_str("</span>");
            }
            Event::SoftBreak => out.push(' '),
            Event::HardBreak => out.push_str("<br>"),
            Event::Rule => out.push_str("<hr>"),
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    const TAGS: [&str; 30] = [
        "p",
        "/p",
        "blockquote",
        "/blockquote",
        "table",
        "/table",
        "tr",
        "/tr",
        "td",
        "/td",
        "pre",
        "/pre",
        "ol",
        "/ol",
        "ul",
        "/ul",
        "li",
        "/li",
        "i",
        "/i",
        "b",
        "/b",
        "s",
        "/s",
        "span",
        "/span",
        "a",
        "/a",
        "br",
        "hr",
    ];

    proptest! {
        #[test]
        fn only_its_own_tags_reach_the_page(text in "[ -~\n]{0,200}") {
            let made = html(&text);
            for piece in made.split('<').skip(1) {
                let name: String = piece
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '/')
                    .collect();
                prop_assert!(TAGS.contains(&name.as_str()), "{name} in {made}");
            }
        }
    }
}
