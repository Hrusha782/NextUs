//! Auto-completion provider

use tower_lsp::lsp_types::*;
use crate::capabilities::{ELEMENT_INFO, get_element_info};

/// Get completions at the given position
pub fn get_completions(text: &str, position: Position) -> Vec<CompletionItem> {
    let lines: Vec<&str> = text.lines().collect();
    let line_idx = position.line as usize;
    
    if line_idx >= lines.len() {
        return vec![];
    }
    
    let line = lines[line_idx];
    let col = position.character as usize;
    let prefix = if col <= line.len() { &line[..col] } else { line };
    
    // Determine context
    let context = determine_context(text, line_idx, prefix);
    
    match context {
        CompletionContext::TopLevel => top_level_completions(),
        CompletionContext::InsideState => state_completions(),
        CompletionContext::InsideView => element_completions(prefix),
        CompletionContext::InsideUpdate => update_completions(),
        CompletionContext::InsideElement(element_name) => property_completions(&element_name, prefix),
        CompletionContext::PropertyValue(prop_name) => value_completions(&prop_name),
        CompletionContext::Unknown => element_completions(prefix),
    }
}

#[derive(Debug)]
enum CompletionContext {
    TopLevel,
    InsideState,
    InsideView,
    InsideUpdate,
    InsideElement(String),
    PropertyValue(String),
    Unknown,
}

fn determine_context(text: &str, line_idx: usize, prefix: &str) -> CompletionContext {
    let trimmed = prefix.trim();
    
    // Check if we're after a colon (property value)
    if trimmed.ends_with(':') || (trimmed.contains(':') && !trimmed.ends_with('}')) {
        if let Some(prop_start) = trimmed.rfind(|c: char| c == '{' || c == '\n' || c.is_whitespace()) {
            let after = trimmed[prop_start..].trim();
            if let Some(colon_idx) = after.find(':') {
                let prop_name = after[..colon_idx].trim();
                return CompletionContext::PropertyValue(prop_name.to_string());
            }
        }
    }
    
    // Walk backwards to find context
    let lines: Vec<&str> = text.lines().collect();
    let mut brace_depth = 0;
    let mut in_state = false;
    let mut in_view = false;
    let mut in_update = false;
    let mut current_element: Option<String> = None;
    
    for i in (0..=line_idx).rev() {
        let line = if i < lines.len() { lines[i] } else { continue };
        let line_trimmed = line.trim();
        
        // Count braces on this line
        for ch in line.chars() {
            match ch {
                '{' => brace_depth += 1,
                '}' => brace_depth -= 1,
                _ => {}
            }
        }
        
        // Check for block starts
        if line_trimmed.starts_with("state") && line_trimmed.contains('{') {
            in_state = true;
            break;
        }
        if line_trimmed.starts_with("view") && line_trimmed.contains('{') {
            in_view = true;
            break;
        }
        if line_trimmed.starts_with("update") && line_trimmed.contains('{') {
            in_update = true;
            break;
        }
        
        // Check for element
        if current_element.is_none() && brace_depth > 0 {
            for elem in ELEMENT_INFO {
                if line_trimmed.contains(elem.name) && line_trimmed.contains('{') {
                    current_element = Some(elem.name.to_string());
                    break;
                }
            }
        }
    }
    
    if let Some(elem) = current_element {
        return CompletionContext::InsideElement(elem);
    }
    if in_state {
        return CompletionContext::InsideState;
    }
    if in_view {
        return CompletionContext::InsideView;
    }
    if in_update {
        return CompletionContext::InsideUpdate;
    }
    if prefix.trim().is_empty() && brace_depth == 0 {
        return CompletionContext::TopLevel;
    }
    
    CompletionContext::Unknown
}

fn top_level_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "state".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Define state variables".to_string()),
            insert_text: Some("state {\n    $0\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "view".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Define the UI view".to_string()),
            insert_text: Some("view {\n    $0\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "update".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Define update rules".to_string()),
            insert_text: Some("update {\n    when $1 => {\n        $0\n    }\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ]
}

fn state_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "variable".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Define a state variable".to_string()),
            insert_text: Some("${1:name}: ${2:value}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ]
}

fn element_completions(prefix: &str) -> Vec<CompletionItem> {
    let prefix_lower = prefix.trim().to_lowercase();
    
    ELEMENT_INFO
        .iter()
        .filter(|e| prefix_lower.is_empty() || e.name.to_lowercase().starts_with(&prefix_lower))
        .map(|elem| {
            let snippet = if elem.can_have_children {
                format!("{} {{\n    $0\n}}", elem.name)
            } else {
                format!("{} {{ $0 }}", elem.name)
            };
            
            CompletionItem {
                label: elem.name.to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some(elem.description.to_string()),
                insert_text: Some(snippet),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            }
        })
        .collect()
}

fn update_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "when Clicked".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Handle click event".to_string()),
            insert_text: Some("when Clicked { id: \"$1\" } => {\n    $0\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "when InputChanged".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Handle input change event".to_string()),
            insert_text: Some("when InputChanged { id: \"$1\", value: $2 } => {\n    $0\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "when MessageSent".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Handle message sent event".to_string()),
            insert_text: Some("when MessageSent => {\n    $0\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ]
}

fn property_completions(element_name: &str, prefix: &str) -> Vec<CompletionItem> {
    let prefix_lower = prefix.trim().to_lowercase();
    
    // Get element-specific properties
    let mut completions: Vec<CompletionItem> = if let Some(elem) = get_element_info(element_name) {
        elem.properties
            .iter()
            .filter(|p| prefix_lower.is_empty() || p.name.to_lowercase().starts_with(&prefix_lower))
            .map(|prop| {
                let snippet = match prop.typ {
                    "string" => format!("{}: \"$1\"", prop.name),
                    "boolean" => format!("{}: ${{1|true,false|}}", prop.name),
                    "number" => format!("{}: $1", prop.name),
                    _ => format!("{}: $1", prop.name),
                };
                
                CompletionItem {
                    label: prop.name.to_string(),
                    kind: Some(CompletionItemKind::PROPERTY),
                    detail: Some(format!("{} ({})", prop.description, prop.typ)),
                    insert_text: Some(snippet),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..Default::default()
                }
            })
            .collect()
    } else {
        vec![]
    };
    
    // Add common properties
    completions.push(CompletionItem {
        label: "id".to_string(),
        kind: Some(CompletionItemKind::PROPERTY),
        detail: Some("Unique identifier for event handling".to_string()),
        insert_text: Some("id: \"$1\"".to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    });
    
    // Also suggest child elements for containers
    if let Some(elem) = get_element_info(element_name) {
        if elem.can_have_children {
            completions.extend(element_completions(prefix));
        }
    }
    
    completions
}

fn value_completions(prop_name: &str) -> Vec<CompletionItem> {
    match prop_name {
        "direction" => vec![
            completion_value("Vertical"),
            completion_value("Horizontal"),
            completion_value("Both"),
        ],
        "orientation" => vec![
            completion_value("Vertical"),
            completion_value("Horizontal"),
        ],
        "variant" => vec![
            completion_value("Primary"),
            completion_value("Secondary"),
            completion_value("Success"),
            completion_value("Danger"),
            completion_value("Warning"),
            completion_value("Ghost"),
        ],
        "position" => vec![
            completion_value("Top"),
            completion_value("Bottom"),
            completion_value("Left"),
            completion_value("Right"),
        ],
        "status" => vec![
            completion_value("Online"),
            completion_value("Offline"),
            completion_value("Away"),
            completion_value("Busy"),
        ],
        "format" => vec![
            completion_value("ShortTime"),
            completion_value("LongTime"),
            completion_value("ShortDate"),
            completion_value("DateTime"),
            completion_value("Relative"),
        ],
        "type" => vec![
            completion_value("Text"),
            completion_value("Password"),
            completion_value("Email"),
            completion_value("Number"),
            completion_value("Search"),
        ],
        _ => vec![],
    }
}

fn completion_value(value: &str) -> CompletionItem {
    CompletionItem {
        label: value.to_string(),
        kind: Some(CompletionItemKind::ENUM_MEMBER),
        insert_text: Some(format!("\"{}\"", value)),
        ..Default::default()
    }
}
