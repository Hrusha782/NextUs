use crate::ast::{Expr, Program};
use crate::limits::{RuntimeLimits, ValidationError};

/// Validates a parsed program against runtime limits before execution.
pub fn validate(program: &Program, limits: &RuntimeLimits) -> Result<(), ValidationError> {
    validate_view_expr(&program.view.body, limits, 0)
}

fn validate_view_expr(
    expr: &Expr,
    limits: &RuntimeLimits,
    depth: usize,
) -> Result<(), ValidationError> {
    limits.validate_depth(depth)?;

    match expr {
        Expr::String(s) => {
            limits.validate_text_length(s.len())?;
        }

        Expr::Element { children, .. } => {
            limits.validate_children_count(children.len())?;
            for child in children {
                validate_view_expr(child, limits, depth + 1)?;
            }
        }

        Expr::List(items) => {
            limits.validate_list_items(items.len())?;
            for item in items {
                validate_view_expr(item, limits, depth + 1)?;
            }
        }

        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            validate_view_expr(condition, limits, depth + 1)?;
            validate_view_expr(then_branch, limits, depth + 1)?;
            if let Some(else_b) = else_branch {
                validate_view_expr(else_b, limits, depth + 1)?;
            }
        }

        Expr::BinaryOp { left, right, .. } => {
            validate_view_expr(left, limits, depth + 1)?;
            validate_view_expr(right, limits, depth + 1)?;
        }

        Expr::UnaryOp { operand, .. } => {
            validate_view_expr(operand, limits, depth + 1)?;
        }

        Expr::PropertyAccess { object, .. } => {
            validate_view_expr(object, limits, depth + 1)?;
        }

        Expr::FunctionCall { args, .. } => {
            for arg in args {
                validate_view_expr(arg, limits, depth + 1)?;
            }
        }

        Expr::MethodCall { object, args, .. } => {
            validate_view_expr(object, limits, depth + 1)?;
            for arg in args {
                validate_view_expr(arg, limits, depth + 1)?;
            }
        }

        Expr::Lambda { body, .. } => {
            validate_view_expr(body, limits, depth + 1)?;
        }

        Expr::Object(fields) => {
            for (_, value) in fields {
                validate_view_expr(value, limits, depth + 1)?;
            }
        }

        // Leaf nodes — no further traversal needed
        Expr::Number(_) | Expr::Boolean(_) | Expr::Null | Expr::Variable(_) => {}
    }

    Ok(())
}
