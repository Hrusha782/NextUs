use crate::ast::*;

pub fn is_lvalue(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Variable(_) | Expr::PropertyAccess { .. }
    )
}
pub fn is_pure_expr(expr: &Expr) -> bool {
    match expr {
        Expr::String(_) | Expr::Number(_) | Expr::Boolean(_) | Expr::Null => true,
        Expr::Variable(_) => true,
        Expr::PropertyAccess { object, .. } => is_pure_expr(object),
        Expr::List(items) => items.iter().all(is_pure_expr),
        Expr::Object(fields) => fields.iter().all(|(_, expr)| is_pure_expr(expr)),
        Expr::BinaryOp { left, right, .. } => is_pure_expr(left) && is_pure_expr(right),
        Expr::UnaryOp { operand, .. } => is_pure_expr(operand),
        Expr::If { condition, then_branch, else_branch } => {
            is_pure_expr(condition)
                && is_pure_expr(then_branch)
                && else_branch.as_ref().map_or(true, |e| is_pure_expr(e))
        }
        Expr::FunctionCall { .. } => false,  // Functions may have side effects
        Expr::MethodCall { object, args, .. } => {
            is_pure_expr(object) && args.iter().all(is_pure_expr)
        }
        Expr::Lambda { body, .. } => is_pure_expr(body),
        Expr::Element { properties, children, .. } => {
            properties.iter().all(|(_, expr)| is_pure_expr(expr))
                && children.iter().all(is_pure_expr)
        }
    }
}

pub fn free_variables(expr: &Expr) -> Vec<String> {
    fn collect(expr: &Expr, bound: &[String], free: &mut Vec<String>) {
        match expr {
            Expr::Variable(name) => {
                if !bound.contains(name) && !free.contains(name) {
                    free.push(name.clone());
                }
            }
            Expr::PropertyAccess { object, .. } => collect(object, bound, free),
            Expr::List(items) => {
                for item in items {
                    collect(item, bound, free);
                }
            }
            Expr::Object(fields) => {
                for (_, value) in fields {
                    collect(value, bound, free);
                }
            }
            Expr::BinaryOp { left, right, .. } => {
                collect(left, bound, free);
                collect(right, bound, free);
            }
            Expr::UnaryOp { operand, .. } => collect(operand, bound, free),
            Expr::If { condition, then_branch, else_branch } => {
                collect(condition, bound, free);
                collect(then_branch, bound, free);
                if let Some(else_b) = else_branch {
                    collect(else_b, bound, free);
                }
            }
            Expr::FunctionCall { args, .. } => {
                for arg in args {
                    collect(arg, bound, free);
                }
            }
            Expr::MethodCall { object, args, .. } => {
                collect(object, bound, free);
                for arg in args {
                    collect(arg, bound, free);
                }
            }
            Expr::Lambda { params, body } => {
                let mut new_bound = bound.to_vec();
                new_bound.extend(params.clone());
                collect(body, &new_bound, free);
            }
            Expr::Element { properties, children, .. } => {
                for (_, value) in properties {
                    collect(value, bound, free);
                }
                for child in children {
                    collect(child, bound, free);
                }
            }
            _ => {}
        }
    }

    let mut free = Vec::new();
    collect(expr, &[], &mut free);
    free
}

pub fn format_expr(expr: &Expr) -> String {
    match expr {
        Expr::String(s) => format!(r#""{}""#, s),
        Expr::Number(n) => n.to_string(),
        Expr::Boolean(b) => b.to_string(),
        Expr::Null => "null".to_string(),
        Expr::Variable(name) => name.clone(),
        Expr::PropertyAccess { object, property } => {
            format!("{}.{}", format_expr(object), property)
        }
        Expr::List(items) => {
            let items_str: Vec<String> = items.iter().map(format_expr).collect();
            format!("[{}]", items_str.join(", "))
        }
        Expr::Object(fields) => {
            let fields_str: Vec<String> = fields
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_expr(v)))
                .collect();
            format!("{{ {} }}", fields_str.join(", "))
        }
        Expr::BinaryOp { left, op, right } => {
            format!(
                "({} {:?} {})",
                format_expr(left),
                op,
                format_expr(right)
            )
        }
        Expr::UnaryOp { op, operand } => {
            format!("({:?} {})", op, format_expr(operand))
        }
        Expr::If { condition, then_branch, else_branch } => {
            if let Some(else_b) = else_branch {
                format!(
                    "if {} then {} else {}",
                    format_expr(condition),
                    format_expr(then_branch),
                    format_expr(else_b)
                )
            } else {
                format!(
                    "if {} then {}",
                    format_expr(condition),
                    format_expr(then_branch)
                )
            }
        }
        Expr::FunctionCall { name, args } => {
            let args_str: Vec<String> = args.iter().map(format_expr).collect();
            format!("{}({})", name, args_str.join(", "))
        }
        Expr::MethodCall { object, method, args } => {
            let args_str: Vec<String> = args.iter().map(format_expr).collect();
            format!("{}.{}({})", format_expr(object), method, args_str.join(", "))
        }
        Expr::Lambda { params, body } => {
            format!("({}) => {}", params.join(", "), format_expr(body))
        }
        Expr::Element { element_type, properties, children } => {
            let props_str: Vec<String> = properties
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_expr(v)))
                .collect();
            let children_str: Vec<String> = children
                .iter()
                .map(format_expr)
                .collect();
            format!("{} {{ {} }}\n{}", 
                element_type, 
                props_str.join(", "),
                children_str.join("\n")
            )
        }
    }
}

/// Walk an expression tree and reject any element the runtime cannot build.
///
/// Without this, a typo or a non-programmable name like `TopBar` passes
/// `--check` and only fails later at render time.
fn validate_elements(expr: &Expr) -> Result<(), String> {
    match expr {
        Expr::Element { element_type, properties, children } => {
            if !crate::element::is_known_element(element_type) {
                return Err(format!(
                    "Unknown element '{}'. It is not a NextUs primitive.",
                    element_type
                ));
            }
            for (_, value) in properties {
                validate_elements(value)?;
            }
            for child in children {
                validate_elements(child)?;
            }
            Ok(())
        }
        Expr::List(items) => items.iter().try_for_each(validate_elements),
        Expr::Object(fields) => fields.iter().try_for_each(|(_, v)| validate_elements(v)),
        Expr::BinaryOp { left, right, .. } => {
            validate_elements(left)?;
            validate_elements(right)
        }
        Expr::UnaryOp { operand, .. } => validate_elements(operand),
        Expr::If { condition, then_branch, else_branch } => {
            validate_elements(condition)?;
            validate_elements(then_branch)?;
            match else_branch {
                Some(e) => validate_elements(e),
                None => Ok(()),
            }
        }
        Expr::FunctionCall { args, .. } => args.iter().try_for_each(validate_elements),
        Expr::MethodCall { object, args, .. } => {
            validate_elements(object)?;
            args.iter().try_for_each(validate_elements)
        }
        Expr::Lambda { body, .. } => validate_elements(body),
        Expr::PropertyAccess { object, .. } => validate_elements(object),
        Expr::String(_) | Expr::Number(_) | Expr::Boolean(_) | Expr::Null | Expr::Variable(_) => {
            Ok(())
        }
    }
}

pub fn validate_program(program: &Program) -> Result<(), String> {
    if matches!(program.view.body, Expr::Null) {
        return Err("View block is required".to_string());
    }

    validate_elements(&program.view.body)?;

    for state_var in &program.state {
        validate_elements(&state_var.initial_value)?;
        if !is_pure_expr(&state_var.initial_value) {
            return Err(format!(
                "State variable '{}' must have a pure initial value",
                state_var.name
            ));
        }
    }


    let state_names: Vec<String> = program.state.iter().map(|s| s.name.clone()).collect();
    
    for rule in &program.update {
        for (var_name, update_expr) in &rule.state_update {
            validate_elements(update_expr)?;
            if !state_names.contains(var_name) {
                return Err(format!(
                    "Update rule references undefined state variable: {}",
                    var_name
                ));
            }
        }
    }

    Ok(())
}
