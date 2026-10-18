use crate::{
    ast::{BinaryOperator, Expr, MessagePattern, Program, UnaryOperator},
    limits::{RuntimeLimits, ValidationError},
    message::Message,
    primitives::containers::{Modal, Panel, Part, Popover, PopoverPosition, Tab, TabBarPosition},
    primitives::content::{
        Avatar, Badge, BadgeVariant, Image, Status, StatusType, Text, Timestamp, TimestampFormat,
        Tooltip, TooltipPosition,
    },
    primitives::input::{Button, ButtonVariant, Checkbox, Dropdown, Input, InputType},
    primitives::layout::{
        Column, Container, Direction, Divider, Grid, Orientation, Row, Spacer, Stack,
    },
    primitives::messenger::{
        Attachment, AttachmentType, Filters, MessageBubble, MessageBubbleType, Reaction, Reply,
        SearchBar, SearchScope, TypingIndicator,
    },
    primitives::scrolling::{List, ListOrientation, Scroll, ScrollBar, ScrollDirection},
    primitives::scrolling::{MessageItem, MessageList},
    properties::Properties,
    state::{State, StateHistory, Value},
    style::{Alignment, Padding, Size, Style},
    Element,
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeError {
    UndefinedVariable(String),
    UndefinedFunction(String),
    TypeError { expected: String, got: String },
    UnknownElement(String),
    DivisionByZero,
    IndexOutOfBounds { index: usize, length: usize },
    PropertyNotFound { object: String, property: String },
    InvalidOperation(String),
    LimitExceeded(ValidationError),
}

pub struct Runtime {
    program: Program,
    state: State,
    history: StateHistory,
    limits: RuntimeLimits,
    _builtin_functions: HashMap<String, BuiltinFunction>,
    depth: usize,
}

type BuiltinFunction = fn(&[Value]) -> Result<Value, RuntimeError>;

impl Runtime {
    pub fn new(program: Program, limits: RuntimeLimits) -> Result<Self, RuntimeError> {
        let mut initial_state = State::new();

        for state_decl in &program.state {
            let value =
                Self::eval_expr_static(&state_decl.initial_value, &initial_state, &limits, 0)?;
            initial_state = initial_state.set(state_decl.name.clone(), value);
        }

        let history = StateHistory::new(initial_state.clone(), 100);
        let builtin_functions = Self::register_builtins();

        Ok(Self {
            program,
            state: initial_state,
            history,
            limits,
            _builtin_functions: builtin_functions,
            depth: 0,
        })
    }

    pub fn render(&self) -> Result<Element, RuntimeError> {
        let view_expr = &self.program.view.body;
        let value = self.eval_expr(view_expr)?;

        match value {
            Value::Element(el) => Ok(el),
            _ => Err(RuntimeError::TypeError {
                expected: "Element".to_string(),
                got: format!("{:?}", value),
            }),
        }
    }

    pub fn update(&mut self, message: Message, timestamp: u64) -> Result<(), RuntimeError> {
        for rule in &self.program.update {
            if Self::matches_pattern(&rule.pattern, &message) {
                let mut updates = HashMap::new();

                for (var_name, expr) in &rule.state_update {
                    let value = self.eval_expr(expr)?;
                    updates.insert(var_name.clone(), value);
                }

                let new_state = self.state.update_many(updates);

                self.history
                    .push(new_state.clone(), timestamp, Some(message));
                self.state = new_state;

                return Ok(());
            }
        }

        Ok(())
    }

    fn eval_expr(&self, expr: &Expr) -> Result<Value, RuntimeError> {
        Self::eval_expr_static(expr, &self.state, &self.limits, self.depth)
    }

    fn eval_expr_static(
        expr: &Expr,
        state: &State,
        limits: &RuntimeLimits,
        depth: usize,
    ) -> Result<Value, RuntimeError> {
        limits
            .validate_depth(depth)
            .map_err(RuntimeError::LimitExceeded)?;

        match expr {
            Expr::String(s) => {
                limits
                    .validate_text_length(s.len())
                    .map_err(RuntimeError::LimitExceeded)?;
                Ok(Value::String(s.clone()))
            }
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::Boolean(b) => Ok(Value::Boolean(*b)),
            Expr::Null => Ok(Value::Null),

            Expr::Variable(name) => {
                if let Some(v) = state.get(name) {
                    return Ok(v.clone());
                }
                // Uppercase bare identifier slipped through: treat as string literal
                if name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                {
                    return Ok(Value::String(name.clone()));
                }
                Err(RuntimeError::UndefinedVariable(name.clone()))
            }

            Expr::PropertyAccess { object, property } => {
                let obj_value = Self::eval_expr_static(object, state, limits, depth + 1)?;
                Self::get_property(&obj_value, property)
            }

            Expr::List(items) => {
                limits
                    .validate_list_items(items.len())
                    .map_err(RuntimeError::LimitExceeded)?;

                let mut values = Vec::new();
                for item in items {
                    values.push(Self::eval_expr_static(item, state, limits, depth + 1)?);
                }
                Ok(Value::List(values))
            }

            Expr::Object(fields) => {
                let mut map = HashMap::new();
                for (key, value_expr) in fields {
                    let value = Self::eval_expr_static(value_expr, state, limits, depth + 1)?;
                    map.insert(key.clone(), value);
                }
                Ok(Value::Object(map))
            }

            Expr::BinaryOp { left, op, right } => {
                let left_val = Self::eval_expr_static(left, state, limits, depth + 1)?;
                let right_val = Self::eval_expr_static(right, state, limits, depth + 1)?;
                Self::apply_binary_op(&left_val, *op, &right_val, limits)
            }

            Expr::UnaryOp { op, operand } => {
                let operand_val = Self::eval_expr_static(operand, state, limits, depth + 1)?;
                Self::apply_unary_op(*op, &operand_val)
            }

            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_val = Self::eval_expr_static(condition, state, limits, depth + 1)?;
                if cond_val.is_truthy() {
                    Self::eval_expr_static(then_branch, state, limits, depth + 1)
                } else if let Some(else_b) = else_branch {
                    Self::eval_expr_static(else_b, state, limits, depth + 1)
                } else {
                    Ok(Value::Null)
                }
            }

            Expr::FunctionCall { name, args } => {
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(Self::eval_expr_static(arg, state, limits, depth + 1)?);
                }

                Self::call_builtin(name, &arg_values)
            }

            Expr::MethodCall {
                object,
                method,
                args,
            } => {
                let obj_value = Self::eval_expr_static(object, state, limits, depth + 1)?;

                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(Self::eval_expr_static(arg, state, limits, depth + 1)?);
                }

                Self::call_method(&obj_value, method, &arg_values)
            }

            Expr::Element {
                element_type,
                properties,
                children,
            } => Self::create_element(element_type, properties, children, state, limits, depth + 1),

            Expr::Lambda { params, body } => Ok(Value::Lambda {
                params: params.clone(),
                body: body.clone(),
            }),
        }
    }

    fn matches_pattern(pattern: &MessagePattern, message: &Message) -> bool {
        match (pattern, message) {
            (MessagePattern::Clicked { id: p_id }, Message::Clicked { id: m_id }) => p_id == m_id,
            (MessagePattern::RightClicked { id: p_id }, Message::RightClicked { id: m_id }) => {
                p_id == m_id
            }
            (
                MessagePattern::InputChanged { id: p_id, .. },
                Message::InputChanged { id: m_id, .. },
            ) => p_id == m_id,
            (MessagePattern::MessageSent, Message::MessageSent { .. }) => true,
            (MessagePattern::MessageReceived { .. }, Message::MessageReceived { .. }) => true,
            (MessagePattern::Any, _) => true,
            _ => false,
        }
    }

    fn get_property(value: &Value, property: &str) -> Result<Value, RuntimeError> {
        match value {
            Value::Object(map) => {
                map.get(property)
                    .cloned()
                    .ok_or_else(|| RuntimeError::PropertyNotFound {
                        object: "Object".to_string(),
                        property: property.to_string(),
                    })
            }
            _ => Err(RuntimeError::PropertyNotFound {
                object: format!("{:?}", value),
                property: property.to_string(),
            }),
        }
    }

    fn apply_binary_op(
        left: &Value,
        op: BinaryOperator,
        right: &Value,
        limits: &RuntimeLimits,
    ) -> Result<Value, RuntimeError> {
        // `+` on a string concatenates, so `"Count: " + count` works as documented.
        // Numbers are matched first below, so arithmetic is unaffected.
        if matches!(op, BinaryOperator::Add | BinaryOperator::Concat)
            && (matches!(left, Value::String(_)) || matches!(right, Value::String(_)))
        {
            return Self::concat_values(left, right, limits);
        }

        match (left, op, right) {
            (Value::Number(l), BinaryOperator::Add, Value::Number(r)) => Ok(Value::Number(l + r)),
            (Value::Number(l), BinaryOperator::Subtract, Value::Number(r)) => {
                Ok(Value::Number(l - r))
            }
            (Value::Number(l), BinaryOperator::Multiply, Value::Number(r)) => {
                Ok(Value::Number(l * r))
            }
            (Value::Number(l), BinaryOperator::Divide, Value::Number(r)) => {
                if *r == 0.0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Number(l / r))
                }
            }
            (Value::Number(l), BinaryOperator::Modulo, Value::Number(r)) => {
                if *r == 0.0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Number(l % r))
                }
            }
            (Value::Boolean(l), BinaryOperator::And, Value::Boolean(r)) => {
                Ok(Value::Boolean(*l && *r))
            }
            (Value::Boolean(l), BinaryOperator::Or, Value::Boolean(r)) => {
                Ok(Value::Boolean(*l || *r))
            }

            // Equality works on any two values of the same shape.
            (_, BinaryOperator::Equal, _) => Ok(Value::Boolean(left == right)),
            (_, BinaryOperator::NotEqual, _) => Ok(Value::Boolean(left != right)),

            // Ordering is defined for numbers and, lexicographically, for strings.
            // A NaN operand makes every comparison false, as in IEEE 754, rather
            // than raising: that keeps `render` total without hiding a type error.
            (_, BinaryOperator::LessThan, _)
            | (_, BinaryOperator::LessOrEqual, _)
            | (_, BinaryOperator::GreaterThan, _)
            | (_, BinaryOperator::GreaterOrEqual, _) => {
                let ordering = match (left, right) {
                    (Value::Number(l), Value::Number(r)) => l.partial_cmp(r),
                    (Value::String(l), Value::String(r)) => Some(l.cmp(r)),
                    _ => {
                        return Err(RuntimeError::InvalidOperation(format!(
                            "Cannot compare {:?} and {:?}: ordering is defined for two numbers or two strings",
                            left, right
                        )))
                    }
                };

                let Some(ordering) = ordering else {
                    return Ok(Value::Boolean(false));
                };

                use std::cmp::Ordering;
                match op {
                    BinaryOperator::LessThan => Ok(Value::Boolean(ordering == Ordering::Less)),
                    BinaryOperator::LessOrEqual => {
                        Ok(Value::Boolean(ordering != Ordering::Greater))
                    }
                    BinaryOperator::GreaterThan => {
                        Ok(Value::Boolean(ordering == Ordering::Greater))
                    }
                    BinaryOperator::GreaterOrEqual => {
                        Ok(Value::Boolean(ordering != Ordering::Less))
                    }
                    // The outer arm admits only the four above; stay an Err
                    // rather than a panic if that ever changes.
                    _ => Err(RuntimeError::InvalidOperation(format!(
                        "Cannot apply {:?} to {:?} and {:?}",
                        op, left, right
                    ))),
                }
            }

            _ => Err(RuntimeError::InvalidOperation(format!(
                "Cannot apply {:?} to {:?} and {:?}",
                op, left, right
            ))),
        }
    }

    /// Concatenate a string with another scalar. The result is bounded by
    /// `max_text_length` so a program cannot grow text without limit.
    fn concat_values(
        left: &Value,
        right: &Value,
        limits: &RuntimeLimits,
    ) -> Result<Value, RuntimeError> {
        let (l, r) = match (left.to_display_string(), right.to_display_string()) {
            (Some(l), Some(r)) => (l, r),
            _ => {
                return Err(RuntimeError::InvalidOperation(format!(
                    "Cannot concatenate {:?} and {:?}: only strings, numbers and booleans have a text form",
                    left, right
                )))
            }
        };

        limits
            .validate_text_length(l.len() + r.len())
            .map_err(RuntimeError::LimitExceeded)?;

        Ok(Value::String(format!("{}{}", l, r)))
    }

    fn apply_unary_op(op: UnaryOperator, operand: &Value) -> Result<Value, RuntimeError> {
        match (op, operand) {
            (UnaryOperator::Not, Value::Boolean(b)) => Ok(Value::Boolean(!b)),
            (UnaryOperator::Negate, Value::Number(n)) => Ok(Value::Number(-n)),
            _ => Err(RuntimeError::InvalidOperation(format!(
                "Cannot apply {:?} to {:?}",
                op, operand
            ))),
        }
    }

    fn create_element(
        element_type: &str,
        properties: &[(String, Expr)],
        children: &[Expr],
        state: &State,
        limits: &RuntimeLimits,
        depth: usize,
    ) -> Result<Value, RuntimeError> {
        // Evaluate all properties
        let mut props: HashMap<String, Value> = HashMap::new();
        for (key, expr) in properties {
            let value = Self::eval_expr_static(expr, state, limits, depth)?;
            props.insert(key.clone(), value);
        }

        // Evaluate all children
        let mut child_elements: Vec<Element> = Vec::new();
        for child_expr in children {
            let child_value = Self::eval_expr_static(child_expr, state, limits, depth)?;
            if let Value::Element(el) = child_value {
                child_elements.push(el);
            }
        }

        // Helper to get property values
        let get_string = |key: &str, default: &str| -> String {
            props
                .get(key)
                .and_then(|v| v.to_display_string())
                .unwrap_or_else(|| default.to_string())
        };

        let get_f32 = |key: &str, default: f32| -> f32 {
            props
                .get(key)
                .and_then(|v| {
                    if let Value::Number(n) = v {
                        Some(*n as f32)
                    } else {
                        None
                    }
                })
                .unwrap_or(default)
        };

        let get_bool = |key: &str, default: bool| -> bool {
            props
                .get(key)
                .and_then(|v| {
                    if let Value::Boolean(b) = v {
                        Some(*b)
                    } else {
                        None
                    }
                })
                .unwrap_or(default)
        };

        let get_id = || -> Option<String> {
            props.get("id").and_then(|v| {
                if let Value::String(s) = v {
                    Some(s.clone())
                } else {
                    None
                }
            })
        };

        let focus_after: Option<String> = props.get("focusAfter").and_then(|v| {
            if let Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        });

        let navigate: Option<String> = props.get("navigate").and_then(|v| {
            if let Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        });

        let action: Option<String> = props.get("action").and_then(|v| {
            if let Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        });

        let action_input: Option<String> = props.get("actionInput").and_then(|v| {
            if let Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        });

        let result_to: Option<String> = props.get("resultTo").and_then(|v| {
            if let Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        });

        let result_format: Option<String> = props.get("resultFormat").and_then(|v| {
            if let Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        });

        // Match element type and construct
        let element = match element_type {
            "Column" => Element::Column(Column {
                children: child_elements,
                spacing: get_f32("spacing", 0.0),
                alignment: Alignment::Start,
                justify: Alignment::Start,
                padding: Padding::zero(),
                width: Size::Auto,
                height: Size::Auto,
            }),

            "Row" => Element::Row(Row {
                children: child_elements,
                spacing: get_f32("spacing", 0.0),
                alignment: Alignment::Start,
                justify: Alignment::Start,
                wrap: get_bool("wrap", false),
                padding: Padding::zero(),
                width: Size::Auto,
                height: Size::Auto,
            }),

            "Container" => Element::Container(Container {
                children: child_elements,
                width: Size::Auto,
                height: Size::Auto,
                padding: Padding::zero(),
                margins: crate::style::Margins::zero(),
                background_colour: None,
                borders: crate::style::Borders::none(),
                alignment: Alignment::Start,
                clip: false,
            }),

            "Stack" => {
                let direction = match get_string("direction", "Vertical").as_str() {
                    "Horizontal" => Direction::Horizontal,
                    _ => Direction::Vertical,
                };
                Element::Stack(Stack {
                    children: child_elements,
                    direction,
                    spacing: get_f32("spacing", 0.0),
                    alignment: Alignment::Start,
                    justify: Alignment::Start,
                    padding: Padding::zero(),
                    width: Size::Auto,
                    height: Size::Auto,
                })
            }

            "Grid" => Element::Grid(Grid {
                children: child_elements,
                columns: get_f32("columns", 2.0) as usize,
                rows: None,
                column_gap: get_f32("column_gap", 0.0),
                row_gap: get_f32("row_gap", 0.0),
                column_widths: vec![],
                row_heights: vec![],
                alignment: Alignment::Start,
                padding: Padding::zero(),
                width: Size::Auto,
                height: Size::Auto,
            }),

            "Spacer" => Element::Spacer(Spacer {
                width: Size::Auto,
                height: Size::Auto,
                grow: props.get("grow").and_then(|v| {
                    if let Value::Number(n) = v {
                        Some(*n as f32)
                    } else {
                        None
                    }
                }),
            }),

            "Divider" => {
                let orientation = match get_string("orientation", "Horizontal").as_str() {
                    "Vertical" => Orientation::Vertical,
                    _ => Orientation::Horizontal,
                };
                Element::Divider(Divider {
                    orientation,
                    thickness: get_f32("thickness", 1.0),
                    colour: crate::style::Colour::rgb(200, 200, 200),
                    length: Size::Auto,
                    margins: crate::style::Margins::zero(),
                })
            }

            "Text" => Element::Text(Text {
                properties: Properties::default()
                    .with_id(get_id().unwrap_or_default())
                    .with_focus_after(focus_after.clone())
                    .with_navigate(navigate.clone())
                    .with_action(action.clone())
                    .with_action_input(action_input.clone())
                    .with_result_to(result_to.clone()),
                style: Style::default(),
                content: get_string("content", ""),
            }),

            "Image" => Element::Image(Image {
                properties: Properties::default()
                    .with_id(get_id().unwrap_or_default())
                    .with_focus_after(focus_after.clone())
                    .with_navigate(navigate.clone())
                    .with_action(action.clone())
                    .with_action_input(action_input.clone())
                    .with_result_to(result_to.clone()),
                style: Style::default(),
                src: get_string("src", ""),
                alt: get_string("alt", ""),
            }),

            "Avatar" => Element::Avatar(Avatar {
                properties: Properties::default()
                    .with_id(get_id().unwrap_or_default())
                    .with_focus_after(focus_after.clone())
                    .with_navigate(navigate.clone())
                    .with_action(action.clone())
                    .with_action_input(action_input.clone())
                    .with_result_to(result_to.clone()),
                style: Style::default(),
                src: get_string("src", ""),
                size: get_f32("size", 40.0),
                fallback: props.get("fallback").and_then(|v| {
                    if let Value::String(s) = v {
                        Some(s.clone())
                    } else {
                        None
                    }
                }),
            }),

            "Badge" => {
                let variant = match get_string("variant", "Primary").as_str() {
                    "Success" => BadgeVariant::Success,
                    "Warning" => BadgeVariant::Warning,
                    "Error" => BadgeVariant::Error,
                    "Info" => BadgeVariant::Info,
                    _ => BadgeVariant::Primary,
                };
                Element::Badge(Badge {
                    properties: Properties::default()
                        .with_id(get_id().unwrap_or_default())
                        .with_focus_after(focus_after.clone())
                        .with_navigate(navigate.clone())
                        .with_action(action.clone())
                        .with_action_input(action_input.clone())
                        .with_result_to(result_to.clone()),
                    style: Style::default(),
                    content: get_string("content", ""),
                    variant,
                })
            }

            "Status" => {
                let status = match get_string("status", "Online").as_str() {
                    "Offline" => StatusType::Offline,
                    "Away" => StatusType::Away,
                    "Busy" => StatusType::Busy,
                    "DoNotDisturb" => StatusType::DoNotDisturb,
                    _ => StatusType::Online,
                };
                Element::Status(Status {
                    properties: Properties::default()
                        .with_id(get_id().unwrap_or_default())
                        .with_focus_after(focus_after.clone())
                        .with_navigate(navigate.clone())
                        .with_action(action.clone())
                        .with_action_input(action_input.clone())
                        .with_result_to(result_to.clone()),
                    style: Style::default(),
                    status,
                    show_text: get_bool("show_text", false),
                    size: get_f32("size", 12.0),
                })
            }

            "Timestamp" => {
                let format = match get_string("format", "ShortTime").as_str() {
                    "LongTime" => TimestampFormat::LongTime,
                    "ShortDate" => TimestampFormat::ShortDate,
                    "LongDate" => TimestampFormat::LongDate,
                    "DateTime" => TimestampFormat::DateTime,
                    "FullDateTime" => TimestampFormat::FullDateTime,
                    "Relative" => TimestampFormat::Relative,
                    _ => TimestampFormat::ShortTime,
                };
                Element::Timestamp(Timestamp {
                    properties: Properties::default()
                        .with_id(get_id().unwrap_or_default())
                        .with_focus_after(focus_after.clone())
                        .with_navigate(navigate.clone())
                        .with_action(action.clone())
                        .with_action_input(action_input.clone())
                        .with_result_to(result_to.clone()),
                    style: Style::default(),
                    time: get_f32("time", 0.0) as u64,
                    format,
                    relative: get_bool("relative", false),
                })
            }

            "Tooltip" => {
                let position = match get_string("position", "Top").as_str() {
                    "Bottom" => TooltipPosition::Bottom,
                    "Left" => TooltipPosition::Left,
                    "Right" => TooltipPosition::Right,
                    "Auto" => TooltipPosition::Auto,
                    _ => TooltipPosition::Top,
                };
                Element::Tooltip(Tooltip {
                    properties: Properties::default()
                        .with_id(get_id().unwrap_or_default())
                        .with_focus_after(focus_after.clone())
                        .with_navigate(navigate.clone())
                        .with_action(action.clone())
                        .with_action_input(action_input.clone())
                        .with_result_to(result_to.clone()),
                    style: Style::default(),
                    content: get_string("content", ""),
                    target_id: get_string("target_id", ""),
                    position,
                    delay_ms: get_f32("delay_ms", 500.0) as u64,
                })
            }

            "Button" => {
                let variant = match get_string("variant", "Primary").as_str() {
                    "Secondary" => ButtonVariant::Secondary,
                    "Success" => ButtonVariant::Success,
                    "Danger" => ButtonVariant::Danger,
                    "Ghost" => ButtonVariant::Ghost,
                    "Link" => ButtonVariant::Link,
                    _ => ButtonVariant::Primary,
                };
                Element::Button(Button {
                    properties: Properties::default()
                        .with_id(get_id().unwrap_or_default())
                        .with_focus_after(focus_after.clone())
                        .with_navigate(navigate.clone())
                        .with_action(action.clone())
                        .with_action_input(action_input.clone())
                        .with_result_to(result_to.clone())
                        .with_result_format(result_format.clone()),
                    style: Style::default(),
                    text: get_string("text", ""),
                    variant,
                })
            }

            "Input" => {
                let input_type = match get_string("type", "Text").as_str() {
                    "Password" => InputType::Password,
                    "Email" => InputType::Email,
                    "Number" => InputType::Number,
                    "Search" => InputType::Search,
                    "Url" => InputType::Url,
                    _ => InputType::Text,
                };
                Element::Input(Input {
                    properties: Properties::default()
                        .with_id(get_id().unwrap_or_default())
                        .with_focus_after(focus_after.clone())
                        .with_navigate(navigate.clone())
                        .with_action(action.clone())
                        .with_action_input(action_input.clone())
                        .with_result_to(result_to.clone()),
                    style: Style::default(),
                    value: get_string("value", ""),
                    placeholder: get_string("placeholder", ""),
                    input_type,
                    max_length: props.get("max_length").and_then(|v| {
                        if let Value::Number(n) = v {
                            Some(*n as usize)
                        } else {
                            None
                        }
                    }),
                    readonly: get_bool("readonly", false),
                    required: get_bool("required", false),
                })
            }

            "Checkbox" => Element::Checkbox(Checkbox {
                properties: Properties::default()
                    .with_id(get_id().unwrap_or_default())
                    .with_focus_after(focus_after.clone())
                    .with_navigate(navigate.clone())
                    .with_action(action.clone())
                    .with_action_input(action_input.clone())
                    .with_result_to(result_to.clone()),
                style: Style::default(),
                checked: get_bool("checked", false),
                label: get_string("label", ""),
                indeterminate: get_bool("indeterminate", false),
            }),

            "Dropdown" => Element::Dropdown(Dropdown {
                properties: Properties::default()
                    .with_id(get_id().unwrap_or_default())
                    .with_focus_after(focus_after.clone())
                    .with_navigate(navigate.clone())
                    .with_action(action.clone())
                    .with_action_input(action_input.clone())
                    .with_result_to(result_to.clone()),
                style: Style::default(),
                options: vec![],
                selected_index: None,
                placeholder: get_string("placeholder", ""),
                searchable: get_bool("searchable", false),
                multi_select: get_bool("multi_select", false),
            }),

            "Scroll" => {
                let direction = match get_string("direction", "Vertical").as_str() {
                    "Horizontal" => ScrollDirection::Horizontal,
                    "Both" => ScrollDirection::Both,
                    _ => ScrollDirection::Vertical,
                };

                // Scroll can only have one child, wrap multiple in a container
                let child = if child_elements.len() == 1 {
                    Box::new(child_elements.into_iter().next().unwrap())
                } else {
                    Box::new(Element::Column(Column {
                        children: child_elements,
                        spacing: 0.0,
                        alignment: Alignment::Start,
                        justify: Alignment::Start,
                        padding: Padding::zero(),
                        width: Size::Auto,
                        height: Size::Auto,
                    }))
                };

                Element::Scroll(Box::new(Scroll {
                    child,
                    direction,
                    width: Size::Auto,
                    height: Size::Auto,
                    padding: Padding::zero(),
                    scrollbar: ScrollBar::default(),
                    snap_to_bottom: get_bool("snap_to_bottom", false),
                    smooth_scroll: get_bool("smooth_scroll", true),
                }))
            }

            "List" => {
                let orientation = match get_string("orientation", "Vertical").as_str() {
                    "Horizontal" => ListOrientation::Horizontal,
                    _ => ListOrientation::Vertical,
                };
                Element::List(List {
                    items: child_elements,
                    orientation,
                    item_height: get_f32("item_height", 48.0),
                    spacing: get_f32("spacing", 0.0),
                    width: Size::Auto,
                    height: Size::Auto,
                    padding: Padding::zero(),
                    scrollbar: ScrollBar::default(),
                    snap_to_bottom: get_bool("snap_to_bottom", false),
                })
            }

            "Panel" => {
                // Panel header must be an Element, use Text element for header string
                let header = Box::new(Element::Text(Text {
                    properties: Properties::default(),
                    style: Style::default(),
                    content: get_string("header", ""),
                }));

                // Content as single element
                let content = if child_elements.len() == 1 {
                    Box::new(child_elements.into_iter().next().unwrap())
                } else {
                    Box::new(Element::Column(Column {
                        children: child_elements,
                        spacing: 0.0,
                        alignment: Alignment::Start,
                        justify: Alignment::Start,
                        padding: Padding::zero(),
                        width: Size::Auto,
                        height: Size::Auto,
                    }))
                };

                Element::Panel(Box::new(Panel {
                    header,
                    content,
                    expanded: get_bool("expanded", true),
                    width: Size::Auto,
                    height: Size::Auto,
                    padding: Padding::zero(),
                    margins: crate::style::Margins::zero(),
                    header_height: get_f32("header_height", 40.0),
                    header_background: None,
                    content_background: None,
                    borders: crate::style::Borders::none(),
                    icon_expanded: None,
                    icon_collapsed: None,
                    animate: get_bool("animate", true),
                }))
            }

            "Tab" => {
                let position = match get_string("position", "Top").as_str() {
                    "Bottom" => TabBarPosition::Bottom,
                    "Left" => TabBarPosition::Left,
                    "Right" => TabBarPosition::Right,
                    _ => TabBarPosition::Top,
                };
                Element::Tab(Tab {
                    tabs: vec![], // Would need special handling for TabItems
                    active_index: get_f32("active_index", 0.0) as usize,
                    width: Size::Auto,
                    height: Size::Auto,
                    padding: Padding::zero(),
                    tab_bar_height: get_f32("tab_bar_height", 40.0),
                    tab_bar_position: position,
                    tab_bar_background: None,
                    active_tab_colour: None,
                    inactive_tab_colour: None,
                    spacing: get_f32("spacing", 0.0),
                })
            }

            "Part" => {
                // Part requires a single child element
                let child = if child_elements.len() == 1 {
                    Box::new(child_elements.into_iter().next().unwrap())
                } else {
                    Box::new(Element::Column(Column {
                        children: child_elements,
                        spacing: 0.0,
                        alignment: Alignment::Start,
                        justify: Alignment::Start,
                        padding: Padding::zero(),
                        width: Size::Auto,
                        height: Size::Auto,
                    }))
                };

                let direction = match get_string("direction", "horizontal")
                    .to_lowercase()
                    .as_str()
                {
                    "vertical" | "v" => "vertical".to_string(),
                    _ => "horizontal".to_string(),
                };
                let parse_size = |key: &str| -> Option<f32> {
                    props.get(key).and_then(|v| {
                        if let Value::Number(n) = v {
                            Some(*n as f32)
                        } else {
                            None
                        }
                    })
                };
                let width_val = parse_size("width");
                let height_val = parse_size("height");
                Element::Part(Part {
                    child,
                    width: width_val.map(Size::Fixed).unwrap_or(Size::Auto),
                    height: height_val.map(Size::Fixed).unwrap_or(Size::Auto),
                    padding: Padding::zero(),
                    margins: crate::style::Margins::zero(),
                    background_colour: None,
                    borders: crate::style::Borders::none(),
                    resizable: get_bool("resizable", true),
                    direction,
                    min_width: parse_size("minWidth"),
                    max_width: parse_size("maxWidth"),
                    min_height: parse_size("minHeight"),
                    max_height: parse_size("maxHeight"),
                })
            }

            "Modal" => {
                // Modal needs special handling for content
                let content = if child_elements.len() == 1 {
                    Box::new(child_elements.into_iter().next().unwrap())
                } else {
                    Box::new(Element::Column(Column {
                        children: child_elements,
                        spacing: 0.0,
                        alignment: Alignment::Start,
                        justify: Alignment::Start,
                        padding: Padding::zero(),
                        width: Size::Auto,
                        height: Size::Auto,
                    }))
                };

                Element::Modal(Modal {
                    content,
                    width: Size::Auto,
                    height: Size::Auto,
                    padding: Padding::zero(),
                    background_colour: crate::style::Colour::WHITE,
                    borders: crate::style::Borders::none(),
                    overlay_colour: crate::style::Colour::new(0, 0, 0, 128),
                    blur_background: get_bool("blur_background", false),
                    centered: get_bool("centered", true),
                    position: None,
                    closable: get_bool("closable", true),
                    dismiss_on_click_outside: get_bool("dismiss_on_click_outside", true),
                    dismiss_on_escape: get_bool("dismiss_on_escape", true),
                    animate: get_bool("animate", true),
                })
            }

            "Popover" => {
                let position = match get_string("position", "Bottom").as_str() {
                    "Top" => PopoverPosition::Top,
                    "Left" => PopoverPosition::Left,
                    "Right" => PopoverPosition::Right,
                    "TopLeft" => PopoverPosition::TopLeft,
                    "TopRight" => PopoverPosition::TopRight,
                    "BottomLeft" => PopoverPosition::BottomLeft,
                    "BottomRight" => PopoverPosition::BottomRight,
                    _ => PopoverPosition::Bottom,
                };

                let content = if child_elements.len() == 1 {
                    Box::new(child_elements.into_iter().next().unwrap())
                } else {
                    Box::new(Element::Column(Column {
                        children: child_elements,
                        spacing: 0.0,
                        alignment: Alignment::Start,
                        justify: Alignment::Start,
                        padding: Padding::zero(),
                        width: Size::Auto,
                        height: Size::Auto,
                    }))
                };

                Element::Popover(Popover {
                    content,
                    anchor: None,
                    position,
                    offset: (0.0, 0.0),
                    width: Size::Auto,
                    height: Size::Auto,
                    padding: Padding::zero(),
                    background_colour: crate::style::Colour::WHITE,
                    borders: crate::style::Borders::none(),
                    shadow: get_bool("shadow", true),
                    arrow: get_bool("arrow", true),
                    dismiss_on_click_outside: get_bool("dismiss_on_click_outside", true),
                    dismiss_on_escape: get_bool("dismiss_on_escape", true),
                })
            }

            "MessageBubble" => {
                let message_type = match get_string("type", "Received").as_str() {
                    "Sent" => MessageBubbleType::Sent,
                    "System" => MessageBubbleType::System,
                    _ => MessageBubbleType::Received,
                };
                let content = if child_elements.len() == 1 {
                    Box::new(child_elements.into_iter().next().unwrap())
                } else {
                    Box::new(Element::Column(Column {
                        children: child_elements,
                        spacing: 0.0,
                        alignment: Alignment::Start,
                        justify: Alignment::Start,
                        padding: Padding::zero(),
                        width: Size::Auto,
                        height: Size::Auto,
                    }))
                };
                Element::MessageBubble(MessageBubble {
                    content,
                    message_type,
                    width: Size::Auto,
                    max_width: Some(get_f32("max_width", 320.0)),
                    padding: Padding::zero(),
                    margins: crate::style::Margins::zero(),
                    background_colour: crate::style::Colour::rgb(230, 230, 230),
                    text_colour: crate::style::Colour::BLACK,
                    borders: crate::style::Borders::none(),
                    show_tail: get_bool("show_tail", true),
                    tail_size: get_f32("tail_size", 8.0),
                    shadow: get_bool("shadow", false),
                })
            }

            "Reaction" => Element::Reaction(Reaction {
                reactions: vec![],
                max_display: get_f32("max_display", 5.0) as usize,
                size: get_f32("size", 20.0),
                spacing: get_f32("spacing", 4.0),
                padding: Padding::zero(),
                background_colour: crate::style::Colour::TRANSPARENT,
                active_background: crate::style::Colour::rgb(230, 230, 230),
                borders: crate::style::Borders::none(),
                show_count: get_bool("show_count", true),
            }),

            "Reply" => {
                let original = if child_elements.len() == 1 {
                    Box::new(child_elements.into_iter().next().unwrap())
                } else {
                    Box::new(Element::Text(Text {
                        properties: Properties::default(),
                        style: Style::default(),
                        content: get_string("original_text", ""),
                    }))
                };
                Element::Reply(Reply {
                    original_message: original,
                    original_sender: get_string("original_sender", ""),
                    original_text: get_string("original_text", ""),
                    max_preview_length: get_f32("max_preview_length", 100.0) as usize,
                    width: Size::Auto,
                    padding: Padding::zero(),
                    margins: crate::style::Margins::zero(),
                    background_colour: crate::style::Colour::rgb(240, 240, 240),
                    borders: crate::style::Borders::none(),
                    show_icon: get_bool("show_icon", true),
                })
            }

            "Attachment" => {
                let file_type = match get_string("file_type", "Other").as_str() {
                    "Image" => AttachmentType::Image,
                    "Video" => AttachmentType::Video,
                    "Audio" => AttachmentType::Audio,
                    "Document" => AttachmentType::Document,
                    "Geo" => AttachmentType::Geo,
                    _ => AttachmentType::Other,
                };
                Element::Attachment(Attachment {
                    file_name: get_string("file_name", ""),
                    file_size: get_f32("file_size", 0.0) as u64,
                    file_type,
                    thumbnail: None,
                    width: Size::Auto,
                    height: Size::Auto,
                    padding: Padding::zero(),
                    margins: crate::style::Margins::zero(),
                    background_colour: crate::style::Colour::rgb(245, 245, 245),
                    borders: crate::style::Borders::none(),
                    border_radius: 8.0,
                    show_download: get_bool("show_download", true),
                    show_preview: get_bool("show_preview", true),
                    progress: None,
                })
            }

            "TypingIndicator" => Element::TypingIndicator(TypingIndicator {
                users: vec![],
                max_display: get_f32("max_display", 3.0) as usize,
                text: get_string("text", "is typing..."),
                styling: Style::default(),
                show_avatars: get_bool("show_avatars", true),
            }),

            "SearchBar" => Element::SearchBar(SearchBar {
                placeholder: get_string("placeholder", "Search..."),
                value: get_string("value", ""),
                search_scope: SearchScope::All,
                filters: Filters {
                    date_from: None,
                    date_to: None,
                    sender: None,
                    has_attachments: false,
                    has_reactions: false,
                },
                styling: Style::default(),
                show_scope_selector: get_bool("show_scope_selector", false),
                show_filters: get_bool("show_filters", false),
                show_results_count: get_bool("show_results_count", false),
                results_count: 0,
                icon: None,
            }),

            "MessageList" => Element::MessageList(MessageList {
                messages: child_elements
                    .into_iter()
                    .enumerate()
                    .map(|(i, el)| MessageItem {
                        element: el,
                        alignment: Alignment::Start,
                        timestamp: 0,
                        id: i.to_string(),
                        height: None,
                    })
                    .collect(),
                width: Size::Auto,
                height: Size::Auto,
                padding: Padding::zero(),
                spacing: get_f32("spacing", 8.0),
                group_spacing: get_f32("group_spacing", 16.0),
                group_threshold: 300,
                scrollbar: crate::primitives::scrolling::ScrollBar::default(),
                auto_scroll: get_bool("auto_scroll", true),
                scroll_threshold: get_f32("scroll_threshold", 100.0),
                show_timestamps: get_bool("show_timestamps", true),
                reverse_order: get_bool("reverse_order", false),
            }),

            _ => return Err(RuntimeError::UnknownElement(element_type.to_string())),
        };

        Ok(Value::Element(element))
    }

    fn call_builtin(name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        match name {
            "length" => builtin_length(args),
            "append" => builtin_append(args),
            "map" => builtin_map(args),
            "filter" => builtin_filter(args),
            "sort_by" => builtin_sort_by(args),
            _ => Err(RuntimeError::UndefinedFunction(name.to_string())),
        }
    }

    fn call_method(object: &Value, method: &str, _args: &[Value]) -> Result<Value, RuntimeError> {
        match (object, method) {
            (Value::List(list), "length") => Ok(Value::Number(list.len() as f64)),
            (Value::String(s), "length") => Ok(Value::Number(s.len() as f64)),
            _ => Err(RuntimeError::UndefinedFunction(format!(
                "{:?}.{}",
                object, method
            ))),
        }
    }

    fn register_builtins() -> HashMap<String, BuiltinFunction> {
        let mut builtins: HashMap<String, BuiltinFunction> = HashMap::new();
        builtins.insert("length".to_string(), builtin_length);
        builtins.insert("append".to_string(), builtin_append);
        builtins
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn undo(&mut self) -> Option<&State> {
        self.history.undo().map(|state| {
            self.state = state.clone();
            &self.state
        })
    }

    pub fn redo(&mut self) -> Option<&State> {
        self.history.redo().map(|state| {
            self.state = state.clone();
            &self.state
        })
    }
}

fn builtin_length(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidOperation(
            "length expects 1 argument".to_string(),
        ));
    }

    match &args[0] {
        Value::List(list) => Ok(Value::Number(list.len() as f64)),
        Value::String(s) => Ok(Value::Number(s.len() as f64)),
        _ => Err(RuntimeError::TypeError {
            expected: "List or String".to_string(),
            got: format!("{:?}", args[0]),
        }),
    }
}

fn builtin_append(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidOperation(
            "append expects 2 arguments".to_string(),
        ));
    }

    match &args[0] {
        Value::List(list) => {
            let mut new_list = list.clone();
            new_list.push(args[1].clone());
            Ok(Value::List(new_list))
        }
        _ => Err(RuntimeError::TypeError {
            expected: "List".to_string(),
            got: format!("{:?}", args[0]),
        }),
    }
}

fn builtin_map(_args: &[Value]) -> Result<Value, RuntimeError> {
    Err(RuntimeError::InvalidOperation(
        "map not yet implemented".to_string(),
    ))
}

fn builtin_filter(_args: &[Value]) -> Result<Value, RuntimeError> {
    Err(RuntimeError::InvalidOperation(
        "filter not yet implemented".to_string(),
    ))
}

fn builtin_sort_by(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidOperation(
            "sort_by expects 2 arguments".to_string(),
        ));
    }

    let list = match &args[0] {
        Value::List(l) => l.clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "List".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    let key_prop = match &args[1] {
        Value::String(prop) => prop.clone(),
        Value::Lambda { params, body } => {
            if params.len() != 1 {
                return Err(RuntimeError::InvalidOperation(
                    "sort_by: lambda must have exactly one parameter".to_string(),
                ));
            }
            match body.as_ref() {
                Expr::PropertyAccess { object, property } => {
                    match object.as_ref() {
                        Expr::Variable(param_name) if param_name == &params[0] => property.clone(),
                        _ => return Err(RuntimeError::InvalidOperation(
                            "sort_by: lambda body must access a property on the parameter (e.g. m -> m.timestamp)".to_string(),
                        )),
                    }
                }
                _ => return Err(RuntimeError::InvalidOperation(
                    "sort_by: lambda body must be a property access (e.g. m -> m.timestamp)".to_string(),
                )),
            }
        }
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "String or Lambda".to_string(),
                got: format!("{:?}", args[1]),
            })
        }
    };

    let mut sorted = list;
    sorted.sort_by(|a, b| {
        let ka = numeric_property(a, &key_prop);
        let kb = numeric_property(b, &key_prop);
        ka.partial_cmp(&kb).unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(Value::List(sorted))
}

fn numeric_property(value: &Value, key: &str) -> f64 {
    if let Value::Object(map) = value {
        if let Some(Value::Number(n)) = map.get(key) {
            return *n;
        }
    }
    0.0
}
