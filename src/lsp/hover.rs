//! Hover information provider

use crate::capabilities::{get_element_info, ELEMENT_INFO};
use tower_lsp::lsp_types::*;

/// Get hover information at the given position
pub fn get_hover(text: &str, position: Position) -> Option<Hover> {
    let lines: Vec<&str> = text.lines().collect();
    let line_idx = position.line as usize;

    if line_idx >= lines.len() {
        return None;
    }

    let line = lines[line_idx];
    let col = position.character as usize;

    // Get the word at cursor
    let word = get_word_at_position(line, col)?;

    // Check if it's an element name
    if let Some(elem) = get_element_info(&word) {
        let mut markdown = format!("## {}\n\n{}\n\n", elem.name, elem.description);

        if !elem.properties.is_empty() {
            markdown.push_str("### Properties\n\n");
            markdown.push_str("| Property | Type | Description |\n");
            markdown.push_str("|----------|------|-------------|\n");

            for prop in elem.properties {
                let default = prop
                    .default
                    .map(|d| format!(" (default: {})", d))
                    .unwrap_or_default();
                markdown.push_str(&format!(
                    "| `{}` | {} | {}{} |\n",
                    prop.name, prop.typ, prop.description, default
                ));
            }
        }

        if elem.can_have_children {
            markdown.push_str("\n*This element can contain child elements.*\n");
        }

        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: markdown,
            }),
            range: None,
        });
    }

    // Check if it's a keyword
    match word.as_str() {
        "state" => {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "## state\n\nDefines the application state variables.\n\n```\nstate {\n    count: 0\n    name: \"User\"\n    items: []\n}\n```\n\nState is immutable - updates create new state.".to_string(),
                }),
                range: None,
            });
        }
        "view" => {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "## view\n\nDefines the UI structure as a pure function of state.\n\n```\nview {\n    Column {\n        Text { content: \"Hello\" }\n    }\n}\n```\n\nThe view is automatically re-rendered when state changes.".to_string(),
                }),
                range: None,
            });
        }
        "update" => {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "## update\n\nDefines rules for how events update state.\n\n```\nupdate {\n    when Clicked { id: \"btn\" } => {\n        count: count + 1\n    }\n}\n```\n\nEach rule matches an event pattern and produces new state.".to_string(),
                }),
                range: None,
            });
        }
        "when" => {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "## when\n\nPattern-matches an event to trigger a state update.\n\n**Available patterns:**\n- `Clicked { id: \"...\" }` - Button/element click\n- `InputChanged { id: \"...\", value: v }` - Text input change\n- `MessageSent` - Message sent event\n- `MessageReceived { ... }` - Message received".to_string(),
                }),
                range: None,
            });
        }
        "Clicked" => {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "## Clicked\n\nEvent fired when an element is clicked.\n\n```\nwhen Clicked { id: \"my_button\" } => {\n    // update state\n}\n```\n\n**Fields:**\n- `id` (string) - The ID of the clicked element".to_string(),
                }),
                range: None,
            });
        }
        "InputChanged" => {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "## InputChanged\n\nEvent fired when an input's value changes.\n\n```\nwhen InputChanged { id: \"my_input\", value: text } => {\n    input_text: text\n}\n```\n\n**Fields:**\n- `id` (string) - The ID of the input\n- `value` (binding) - Variable to capture the new value".to_string(),
                }),
                range: None,
            });
        }
        _ => {}
    }

    // Check if it's a property name - look for element context
    let element_context = find_element_context(text, line_idx);
    if let Some(elem_name) = element_context {
        if let Some(elem) = get_element_info(&elem_name) {
            for prop in elem.properties {
                if prop.name == word {
                    let default_str = prop
                        .default
                        .map(|d| format!("\n\n**Default:** `{}`", d))
                        .unwrap_or_default();
                    return Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: format!(
                                "## {}.{}\n\n**Type:** `{}`\n\n{}{}",
                                elem_name, prop.name, prop.typ, prop.description, default_str
                            ),
                        }),
                        range: None,
                    });
                }
            }
        }
    }

    None
}

/// Extract the word at the given column position
fn get_word_at_position(line: &str, col: usize) -> Option<String> {
    if col > line.len() {
        return None;
    }

    let chars: Vec<char> = line.chars().collect();

    // Find word boundaries
    let mut start = col;
    while start > 0 && is_word_char(chars.get(start - 1).copied().unwrap_or(' ')) {
        start -= 1;
    }

    let mut end = col;
    while end < chars.len() && is_word_char(chars[end]) {
        end += 1;
    }

    if start == end {
        return None;
    }

    Some(chars[start..end].iter().collect())
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Find the element name containing the current line
fn find_element_context(text: &str, line_idx: usize) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();

    // Look backwards for element start
    for i in (0..=line_idx).rev() {
        let line = lines.get(i)?;
        let trimmed = line.trim();

        // Check for element pattern: ElementName {
        for elem in ELEMENT_INFO {
            if trimmed.starts_with(elem.name) && trimmed.contains('{') {
                return Some(elem.name.to_string());
            }
        }
    }

    None
}
