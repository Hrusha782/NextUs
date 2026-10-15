//! Diagnostics - syntax error detection and reporting

use tower_lsp::lsp_types::*;
use nextus::parser::{Lexer, Parser, LexError};

/// Analyze document and return diagnostics (errors/warnings)
pub fn get_diagnostics(text: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Try to lex the document
    let mut lexer = Lexer::new(text);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(lex_error) => {
            // Extract line/column from LexError
            let (line, col) = get_lex_error_position(&lex_error);
            
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position { line, character: col },
                    end: Position { line, character: col + 1 },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("nextus".to_string()),
                message: lex_error.to_string(),
                ..Default::default()
            });
            return diagnostics;
        }
    };

    // Try to parse
    let mut parser = Parser::new(tokens);
    if let Err(parse_error) = parser.parse_program() {
        let (line, col) = parse_error_position(&parse_error.to_string());
        
        diagnostics.push(Diagnostic {
            range: Range {
                start: Position { line, character: col },
                end: Position { line, character: col + 10 },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("nextus".to_string()),
            message: parse_error.to_string(),
            ..Default::default()
        });
    }

    // Additional semantic checks
    check_structure(text, &mut diagnostics);

    diagnostics
}

/// Get line/column from LexError
fn get_lex_error_position(error: &LexError) -> (u32, u32) {
    match error {
        LexError::UnexpectedCharacter(_, line, col) => ((*line as u32).saturating_sub(1), *col as u32),
        LexError::InvalidNumber(_, line, col) => ((*line as u32).saturating_sub(1), *col as u32),
        LexError::UnterminatedString(line, col) => ((*line as u32).saturating_sub(1), *col as u32),
    }
}

/// Try to extract line/column from error message string
fn parse_error_position(error: &str) -> (u32, u32) {
    // Look for patterns like "at line 5" or "line 5, column 10"
    let error_lower = error.to_lowercase();
    
    let mut line = 0u32;
    let mut col = 0u32;
    
    if let Some(idx) = error_lower.find("line ") {
        let after = &error[idx + 5..];
        if let Some(num_end) = after.find(|c: char| !c.is_ascii_digit()) {
            if let Ok(n) = after[..num_end].parse::<u32>() {
                line = n.saturating_sub(1); // LSP is 0-indexed
            }
        } else if let Ok(n) = after.trim().parse::<u32>() {
            line = n.saturating_sub(1);
        }
    }
    
    if let Some(idx) = error_lower.find("column ") {
        let after = &error[idx + 7..];
        if let Some(num_end) = after.find(|c: char| !c.is_ascii_digit()) {
            if let Ok(n) = after[..num_end].parse::<u32>() {
                col = n.saturating_sub(1);
            }
        }
    }
    
    (line, col)
}

/// Check document structure for common issues
fn check_structure(text: &str, diagnostics: &mut Vec<Diagnostic>) {
    let lines: Vec<&str> = text.lines().collect();
    
    let mut has_state = false;
    let mut has_view = false;
    let mut _has_update = false;
    
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // Check for required blocks
        if trimmed.starts_with("state ") || trimmed == "state {" || trimmed.starts_with("state{") {
            has_state = true;
        }
        if trimmed.starts_with("view ") || trimmed == "view {" || trimmed.starts_with("view{") {
            has_view = true;
        }
        if trimmed.starts_with("update ") || trimmed == "update {" || trimmed.starts_with("update{") {
            _has_update = true;
        }
        
        // Warn about common mistakes
        if trimmed.contains("#") && !trimmed.starts_with("//") {
            // Hex colors not supported
            if let Some(hash_idx) = line.find('#') {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: i as u32, character: hash_idx as u32 },
                        end: Position { line: i as u32, character: (hash_idx + 7).min(line.len()) as u32 },
                    },
                    severity: Some(DiagnosticSeverity::WARNING),
                    source: Some("nextus".to_string()),
                    message: "Hex colors (#RRGGBB) are not supported. Use named colors or rgb().".to_string(),
                    ..Default::default()
                });
            }
        }
    }
    
    // Missing required blocks (only warn if file has some content)
    if text.trim().len() > 10 {
        if !has_state {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: 1 },
                },
                severity: Some(DiagnosticSeverity::HINT),
                source: Some("nextus".to_string()),
                message: "Consider adding a 'state { }' block to define state variables.".to_string(),
                ..Default::default()
            });
        }
        
        if !has_view {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: 1 },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("nextus".to_string()),
                message: "Missing required 'view { }' block.".to_string(),
                ..Default::default()
            });
        }
    }
}
