pub mod lexer;
pub mod syntax;

pub use lexer::{Token, Lexer, LexError};

use crate::ast::{
    Program, StateDecl, ViewDecl, UpdateRule, MessagePattern, StyleRule, StyleSelector,
    Expr, BinaryOperator, UnaryOperator, Type,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ParseError {
    LexError(LexError),
    UnexpectedToken { expected: String, found: Token },
    UnexpectedEof,
    InvalidPattern(String),
    InvalidType(String),
    InvalidExpression(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::LexError(e) => write!(f, "Lex error: {}", e),
            ParseError::UnexpectedToken { expected, found } => {
                write!(f, "Expected {}, found {:?}", expected, found)
            }
            ParseError::UnexpectedEof => write!(f, "Unexpected end of file"),
            ParseError::InvalidPattern(msg) => write!(f, "Invalid pattern: {}", msg),
            ParseError::InvalidType(msg) => write!(f, "Invalid type: {}", msg),
            ParseError::InvalidExpression(msg) => write!(f, "Invalid expression: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<LexError> for ParseError {
    fn from(err: LexError) -> Self {
        ParseError::LexError(err)
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, position: 0 }
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut program = Program::new();

        while !self.is_at_end() {
            match self.current() {
                Token::State => {
                    program.state = self.parse_state_block()?;
                }
                Token::View => {
                    program.view = self.parse_view_block()?;
                }
                Token::Update => {
                    program.update = self.parse_update_block()?;
                }
                Token::Identifier(name) if name == "style" => {
                    let rule = self.parse_single_style_rule()?;
                    program.styles.push(rule);
                }
                Token::Identifier(name) if name == "vars" => {
                    program.vars = self.parse_kv_block()?;
                }
                Token::Identifier(name) if name == "config" => {
                    program.config = self.parse_kv_block()?;
                }
                Token::Eof => break,
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "state, view, update, style, vars, or config".to_string(),
                        found: self.current().clone(),
                    });
                }
            }
        }

        Ok(program)
    }

    // ==================== State Block ====================
    fn parse_state_block(&mut self) -> Result<Vec<StateDecl>, ParseError> {
        self.expect(Token::State)?;
        self.expect(Token::LBrace)?;

        let mut declarations = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let decl = self.parse_state_decl()?;
            declarations.push(decl);

            if self.check(&Token::Comma) || self.check(&Token::Semicolon) {
                self.advance();
            }
        }

        self.expect(Token::RBrace)?;
        Ok(declarations)
    }

    fn parse_state_decl(&mut self) -> Result<StateDecl, ParseError> {
        let name = self.expect_identifier()?;
        self.expect(Token::Colon)?;

        // Optional type annotation: `name: Type = value`
        // Detect by checking if the next two tokens are Identifier then Equal.
        let var_type = if matches!(self.current(), Token::Identifier(_))
            && self.peek_is(&Token::Equal)
        {
            let type_name = self.expect_identifier()?;
            self.expect(Token::Equal)?;
            let t = match type_name.as_str() {
                "String" | "Str" => Type::String,
                "Number" | "Int" | "Float" | "f64" | "i64" | "i32" | "u32" => Type::Number,
                "Boolean" | "Bool" | "bool" => Type::Boolean,
                "Element" => Type::Element,
                _ => Type::Any,
            };
            Some(t)
        } else {
            None
        };

        let initial_value = self.parse_expr()?;

        Ok(StateDecl {
            name,
            initial_value,
            var_type,
        })
    }

    // ==================== vars { } / config { } — shared KV parser ====================
    // Both blocks have the same shape:  keyword { key: value  key: value ... }
    // where values are strings, numbers (kept as-is, no "px" appended), or #hex.
    fn parse_kv_block(&mut self) -> Result<std::collections::HashMap<String, String>, ParseError> {
        self.advance(); // consume the keyword identifier ("vars" / "config")
        self.expect(Token::LBrace)?;

        let mut map = std::collections::HashMap::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let key = self.expect_identifier()?;
            self.expect(Token::Colon)?;
            let val_expr = self.parse_expr()?;
            let val = match val_expr {
                Expr::String(s) => s,
                Expr::Number(n) => {
                    if n == n.floor() { format!("{}", n as i64) } else { format!("{n}") }
                }
                _ => String::new(),
            };
            if !val.is_empty() {
                map.insert(key, val);
            }
            if self.check(&Token::Comma) || self.check(&Token::Semicolon) {
                self.advance();
            }
        }

        self.expect(Token::RBrace)?;
        Ok(map)
    }

    // ==================== View Block ====================
    fn parse_view_block(&mut self) -> Result<ViewDecl, ParseError> {
        self.expect(Token::View)?;

        let params = if self.check(&Token::LParen) {
            self.advance();
            let mut params = Vec::new();

            while !self.check(&Token::RParen) && !self.is_at_end() {
                params.push(self.expect_identifier()?);
                if self.check(&Token::Comma) {
                    self.advance();
                }
            }

            self.expect(Token::RParen)?;
            params
        } else {
            vec!["state".to_string()]
        };

        self.expect(Token::LBrace)?;
        let body = self.parse_expr()?;
        self.expect(Token::RBrace)?;

        Ok(ViewDecl { params, body })
    }

    // ==================== Update Block ====================
    fn parse_update_block(&mut self) -> Result<Vec<UpdateRule>, ParseError> {
        self.expect(Token::Update)?;
        self.expect(Token::LBrace)?;

        let mut rules = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let rule = self.parse_update_rule()?;
            rules.push(rule);
        }

        self.expect(Token::RBrace)?;
        Ok(rules)
    }

    fn parse_update_rule(&mut self) -> Result<UpdateRule, ParseError> {
        self.expect(Token::When)?;

        let pattern = self.parse_message_pattern()?;

        // Accept both `=>` (FatArrow) and `then` keyword
        if self.check(&Token::FatArrow) {
            self.advance();
        } else {
            self.expect(Token::Then)?;
        }
        self.expect(Token::LBrace)?;

        let mut state_update = HashMap::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let var_name = self.expect_identifier()?;
            self.expect(Token::Colon)?;
            let expr = self.parse_expr()?;
            state_update.insert(var_name, expr);

            if self.check(&Token::Comma) {
                self.advance();
            }
        }

        self.expect(Token::RBrace)?;

        Ok(UpdateRule {
            pattern,
            state_update,
        })
    }

    fn parse_message_pattern(&mut self) -> Result<MessagePattern, ParseError> {
        let pattern_name = self.expect_identifier()?;

        match pattern_name.as_str() {
            "Clicked" | "ButtonClicked" => {
                self.expect(Token::LBrace)?;
                self.expect_identifier_eq("id")?;
                self.expect(Token::Colon)?;
                let id = self.expect_string()?;
                self.expect(Token::RBrace)?;
                Ok(MessagePattern::Clicked { id })
            }
            "DoubleClicked" => {
                self.expect(Token::LBrace)?;
                self.expect_identifier_eq("id")?;
                self.expect(Token::Colon)?;
                let id = self.expect_string()?;
                self.expect(Token::RBrace)?;
                Ok(MessagePattern::DoubleClicked { id })
            }
            "RightClicked" => {
                self.expect(Token::LBrace)?;
                self.expect_identifier_eq("id")?;
                self.expect(Token::Colon)?;
                let id = self.expect_string()?;
                self.expect(Token::RBrace)?;
                Ok(MessagePattern::RightClicked { id })
            }
            "InputChanged" => {
                self.expect(Token::LBrace)?;
                let mut id = String::new();
                let mut value_var = None;

                while !self.check(&Token::RBrace) && !self.is_at_end() {
                    let field = self.expect_identifier()?;
                    self.expect(Token::Colon)?;

                    match field.as_str() {
                        "id" => id = self.expect_string()?,
                        "value" => value_var = Some(self.expect_identifier()?),
                        _ => return Err(ParseError::InvalidPattern(format!("Unknown field in InputChanged: {}", field)))
                    }

                    if self.check(&Token::Comma) {
                        self.advance();
                    }
                }

                self.expect(Token::RBrace)?;
                Ok(MessagePattern::InputChanged { id, value_var })
            }
            "FormSubmitted" => {
                self.expect(Token::LBrace)?;
                self.expect_identifier_eq("id")?;
                self.expect(Token::Colon)?;
                let id = self.expect_string()?;
                self.expect(Token::RBrace)?;
                Ok(MessagePattern::FormSubmitted { id })
            }
            // CheckboxToggled { id: "x", checked: c }
            "CheckboxToggled" => {
                self.expect(Token::LBrace)?;
                let mut id = String::new();
                let mut _checked_var = None;
                while !self.check(&Token::RBrace) && !self.is_at_end() {
                    let field = self.expect_identifier()?;
                    self.expect(Token::Colon)?;
                    match field.as_str() {
                        "id" => id = self.expect_string()?,
                        "checked" => _checked_var = Some(self.expect_identifier()?),
                        _ => { self.parse_expr()?; }
                    }
                    if self.check(&Token::Comma) { self.advance(); }
                }
                self.expect(Token::RBrace)?;
                Ok(MessagePattern::Clicked { id }) // reuse Clicked — runtime handles toggle
            }
            // DropdownSelected { id: "x", value: v }
            "DropdownSelected" => {
                self.expect(Token::LBrace)?;
                let mut id = String::new();
                let mut value_var = None;
                while !self.check(&Token::RBrace) && !self.is_at_end() {
                    let field = self.expect_identifier()?;
                    self.expect(Token::Colon)?;
                    match field.as_str() {
                        "id" => id = self.expect_string()?,
                        "value" => value_var = Some(self.expect_identifier()?),
                        _ => { self.parse_expr()?; }
                    }
                    if self.check(&Token::Comma) { self.advance(); }
                }
                self.expect(Token::RBrace)?;
                Ok(MessagePattern::InputChanged { id, value_var })
            }
            // KeyPressed { id: "x", key: k }
            "KeyPressed" => {
                self.expect(Token::LBrace)?;
                let mut id = String::new();
                let mut value_var = None;
                while !self.check(&Token::RBrace) && !self.is_at_end() {
                    let field = self.expect_identifier()?;
                    self.expect(Token::Colon)?;
                    match field.as_str() {
                        "id" => id = self.expect_string()?,
                        "key" => value_var = Some(self.expect_identifier()?),
                        _ => { self.parse_expr()?; }
                    }
                    if self.check(&Token::Comma) { self.advance(); }
                }
                self.expect(Token::RBrace)?;
                Ok(MessagePattern::InputChanged { id, value_var })
            }
            // Scrolled { id: "x", position: p }
            "Scrolled" => {
                self.expect(Token::LBrace)?;
                let mut id = String::new();
                let mut value_var = None;
                while !self.check(&Token::RBrace) && !self.is_at_end() {
                    let field = self.expect_identifier()?;
                    self.expect(Token::Colon)?;
                    match field.as_str() {
                        "id" => id = self.expect_string()?,
                        "position" => value_var = Some(self.expect_identifier()?),
                        _ => { self.parse_expr()?; }
                    }
                    if self.check(&Token::Comma) { self.advance(); }
                }
                self.expect(Token::RBrace)?;
                Ok(MessagePattern::InputChanged { id, value_var })
            }
            "MessageSent" => Ok(MessagePattern::MessageSent),
            "MessageReceived" => {
                self.expect(Token::LBrace)?;
                self.expect_identifier_eq("msg")?;
                self.expect(Token::Colon)?;
                let msg_var = self.expect_identifier()?;
                self.expect(Token::RBrace)?;
                Ok(MessagePattern::MessageReceived { msg_var })
            }
            // ChatSelected { id: chat_id }
            "ChatSelected" => {
                self.expect(Token::LBrace)?;
                let mut _id_field = String::new();
                let mut value_var = None;
                while !self.check(&Token::RBrace) && !self.is_at_end() {
                    let field = self.expect_identifier()?;
                    self.expect(Token::Colon)?;
                    match field.as_str() {
                        "id" => {
                            _id_field = "id".to_string();
                            value_var = Some(self.expect_identifier()?);
                        }
                        _ => { self.parse_expr()?; }
                    }
                    if self.check(&Token::Comma) { self.advance(); }
                }
                self.expect(Token::RBrace)?;
                Ok(MessagePattern::InputChanged { id: "chat-list".to_string(), value_var })
            }
            // No-field chat events
            "MessageDeleted" | "MessageEdited" | "ReactionAdded" | "ReactionRemoved"
            | "TypingStarted" | "TypingStopped" | "SearchQueryChanged" => {
                // Consume the optional { ... } block if present, then treat as Any.
                if self.check(&Token::LBrace) {
                    self.advance();
                    while !self.check(&Token::RBrace) && !self.is_at_end() {
                        let _field = self.expect_identifier()?;
                        self.expect(Token::Colon)?;
                        self.parse_expr()?;
                        if self.check(&Token::Comma) { self.advance(); }
                    }
                    self.expect(Token::RBrace)?;
                }
                Ok(MessagePattern::Any)
            }
            "Any" | "_" => Ok(MessagePattern::Any),
            _ => Err(ParseError::InvalidPattern(format!(
                "Unknown event pattern '{}'. Supported: Clicked, DoubleClicked, InputChanged, \
                 FormSubmitted, CheckboxToggled, DropdownSelected, KeyPressed, Scrolled, \
                 MessageSent, MessageReceived, ChatSelected, MessageDeleted, MessageEdited, \
                 ReactionAdded, ReactionRemoved, TypingStarted, TypingStopped, \
                 SearchQueryChanged, Any, _",
                pattern_name
            ))),
        }
    }

    // ==================== Style Block ====================
    // Kept for backwards compat: `style { rule1 rule2 }` still works.
    fn parse_style_block(&mut self) -> Result<Vec<StyleRule>, ParseError> {
        self.advance(); // skip 'style'
        self.expect(Token::LBrace)?;

        let mut rules = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let rule = self.parse_style_rule()?;
            rules.push(rule);
        }

        self.expect(Token::RBrace)?;
        Ok(rules)
    }

    // `style Selector { ... }` — one rule, `style` keyword already consumed by parse_program.
    fn parse_single_style_rule(&mut self) -> Result<StyleRule, ParseError> {
        self.advance(); // skip 'style' identifier
        let selector = self.parse_style_selector()?;
        self.expect(Token::LBrace)?;

        let mut properties = std::collections::HashMap::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let prop_name = self.expect_identifier()?;
            self.expect(Token::Colon)?;
            let prop_value = self.parse_expr()?;
            properties.insert(prop_name, prop_value);

            if self.check(&Token::Comma) || self.check(&Token::Semicolon) {
                self.advance();
            }
        }

        self.expect(Token::RBrace)?;
        Ok(StyleRule { selector, properties })
    }

    fn parse_style_rule(&mut self) -> Result<StyleRule, ParseError> {
        let selector = self.parse_style_selector()?;

        self.expect(Token::LBrace)?;

        let mut properties = HashMap::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let prop_name = self.expect_identifier()?;
            self.expect(Token::Colon)?;
            let prop_value = self.parse_expr()?;
            properties.insert(prop_name, prop_value);

            if self.check(&Token::Comma) || self.check(&Token::Semicolon) {
                self.advance();
            }
        }

        self.expect(Token::RBrace)?;

        Ok(StyleRule {
            selector,
            properties,
        })
    }

    fn parse_style_selector(&mut self) -> Result<StyleSelector, ParseError> {
        match self.current().clone() {
            Token::Hash => {
                self.advance(); // skip '#'
                // IDs may contain hyphens (e.g. `#close-chat`, `#my-profile`).
                // The lexer stops identifier scanning at '-', so we stitch the
                // parts together: Identifier (Minus Identifier)*.
                let mut id_name = self.expect_identifier()?;
                while self.check(&Token::Minus) {
                    // Peek ahead: only continue if the token after '-' is an identifier.
                    if let Token::Identifier(_) = self.peek_next().clone() {
                        self.advance(); // consume '-'
                        let part = self.expect_identifier()?;
                        id_name.push('-');
                        id_name.push_str(&part);
                    } else {
                        break;
                    }
                }
                Ok(StyleSelector::Id(id_name))
            }
            Token::Dot => {
                self.advance();
                let class_name = self.expect_identifier()?;
                Ok(StyleSelector::Class(class_name))
            }
            Token::Identifier(name) => {
                let element_name = name.clone();
                self.advance();

                if self.check(&Token::Dot) {
                    // `ElementName.Variant`
                    self.advance();
                    let variant = self.expect_identifier()?;
                    Ok(StyleSelector::Variant(element_name, variant))
                } else {
                    Ok(StyleSelector::Element(element_name))
                }
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "style selector (ElementName, ElementName.Variant, #id, or .class)".to_string(),
                found: self.current().clone(),
            }),
        }
    }

    // ==================== Expression Parsing ====================
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_or_expr()
    }

    fn parse_or_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and_expr()?;

        while self.check(&Token::Or) {
            self.advance();
            let right = self.parse_and_expr()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op: BinaryOperator::Or,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_equality_expr()?;

        while self.check(&Token::And) {
            self.advance();
            let right = self.parse_equality_expr()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op: BinaryOperator::And,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_equality_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_comparison_expr()?;

        loop {
            let op = match self.current() {
                Token::EqualEqual => BinaryOperator::Equal,
                Token::BangEqual => BinaryOperator::NotEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison_expr()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_comparison_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_term()?;

        loop {
            let op = match self.current() {
                Token::Less => BinaryOperator::LessThan,
                Token::LessEqual => BinaryOperator::LessOrEqual,
                Token::Greater => BinaryOperator::GreaterThan,
                Token::GreaterEqual => BinaryOperator::GreaterOrEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_term()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_factor()?;

        loop {
            let op = match self.current() {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Subtract,
                _ => break,
            };
            self.advance();
            let right = self.parse_factor()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;

        loop {
            let op = match self.current() {
                Token::Star => BinaryOperator::Multiply,
                Token::Slash => BinaryOperator::Divide,
                Token::Percent => BinaryOperator::Modulo,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        match self.current() {
            Token::Bang => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOperator::Not,
                    operand: Box::new(operand),
                })
            }
            Token::Minus => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOperator::Negate,
                    operand: Box::new(operand),
                })
            }
            _ => self.parse_call_expr(),
        }
    }

    fn parse_call_expr(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.check(&Token::Dot) {
                self.advance();
                let property = self.expect_identifier()?;

                if self.check(&Token::LParen) {
                    self.advance();
                    let args = self.parse_arguments()?;
                    self.expect(Token::RParen)?;

                    expr = Expr::MethodCall {
                        object: Box::new(expr),
                        method: property,
                        args,
                    };
                } else {
                    expr = Expr::PropertyAccess {
                        object: Box::new(expr),
                        property,
                    };
                }
            } else if self.check(&Token::LParen) {
                if let Expr::Variable(name) = expr {
                    self.advance();
                    let args = self.parse_arguments()?;
                    self.expect(Token::RParen)?;

                    expr = Expr::FunctionCall { name, args };
                } else {
                    break;
                }
            } else if self.check(&Token::LBracket) {
                self.advance();
                let index = self.parse_expr()?;
                self.expect(Token::RBracket)?;

                expr = Expr::MethodCall {
                    object: Box::new(expr),
                    method: "get".to_string(),
                    args: vec![index],
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.current().clone() {
            Token::Number(n) => {
                self.advance();
                Ok(Expr::Number(n))
            }
            Token::String(s) => {
                self.advance();
                Ok(Expr::String(s))
            }
            Token::True => {
                self.advance();
                Ok(Expr::Boolean(true))
            }
            Token::False => {
                self.advance();
                Ok(Expr::Boolean(false))
            }
            Token::Null => {
                self.advance();
                Ok(Expr::Null)
            }
            Token::Identifier(name) => {
                self.advance();

                if self.check(&Token::LBrace) && name.chars().next().unwrap_or('a').is_uppercase() {
                    self.parse_element(name)
                } else if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    // Uppercase bare identifier = enum/variant literal (Primary, Online, Vertical…)
                    Ok(Expr::String(name))
                } else {
                    Ok(Expr::Variable(name))
                }
            }
            Token::LBracket => {
                self.advance();
                let items = self.parse_list_items()?;
                self.expect(Token::RBracket)?;
                Ok(Expr::List(items))
            }
            Token::LBrace => {
                self.advance();
                let fields = self.parse_object_fields()?;
                self.expect(Token::RBrace)?;
                Ok(Expr::Object(fields))
            }
            Token::LParen => {
                self.advance();

                if self.is_lambda_start() {
                    let params = self.parse_lambda_params()?;
                    self.expect(Token::RParen)?;
                    self.expect(Token::Arrow)?;
                    let body = self.parse_expr()?;

                    Ok(Expr::Lambda {
                        params,
                        body: Box::new(body),
                    })
                } else {
                    let expr = self.parse_expr()?;
                    self.expect(Token::RParen)?;
                    Ok(expr)
                }
            }
            Token::If => {
                self.advance();
                let condition = self.parse_expr()?;

                self.expect(Token::Then)?;
                let then_branch = self.parse_expr()?;

                let else_branch = if self.check(&Token::Else) {
                    self.advance();
                    Some(Box::new(self.parse_expr()?))
                } else {
                    None
                };

                Ok(Expr::If {
                    condition: Box::new(condition),
                    then_branch: Box::new(then_branch),
                    else_branch,
                })
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: self.current().clone(),
            }),
        }
    }

    fn parse_element(&mut self, element_type: String) -> Result<Expr, ParseError> {
        self.expect(Token::LBrace)?;

        let mut properties = Vec::new();
        let mut children = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            if let Token::Identifier(name) = self.current().clone() {
                if self.peek_is(&Token::Colon) {
                    // Regular property: name: value
                    self.advance(); // consume name
                    self.advance(); // consume colon
                    let value = self.parse_expr()?;
                    properties.push((name, value));
                    if self.check(&Token::Comma) { self.advance(); }
                    continue;
                }
                // Inline block property: lowercase name followed by { }
                // e.g. `style { fontSize: 18  fontWeight: Bold }` — no commas required
                if self.peek_is(&Token::LBrace)
                    && name.chars().next().map(|c| c.is_lowercase()).unwrap_or(false)
                {
                    self.advance(); // consume name
                    self.advance(); // consume LBrace
                    let mut fields: Vec<(String, Expr)> = Vec::new();
                    while !self.check(&Token::RBrace) && !self.is_at_end() {
                        let key = self.expect_identifier()?;
                        self.expect(Token::Colon)?;
                        let val = self.parse_expr()?;
                        fields.push((key, val));
                        if self.check(&Token::Comma) { self.advance(); }
                        // newline-separated fields need no comma — just continue the loop
                    }
                    self.expect(Token::RBrace)?;
                    properties.push((name, Expr::Object(fields)));
                    continue;
                }
            }

            let child = self.parse_expr()?;
            children.push(child);

            if self.check(&Token::Comma) {
                self.advance();
            }
        }

        self.expect(Token::RBrace)?;

        Ok(Expr::Element {
            element_type,
            properties,
            children,
        })
    }

    fn parse_arguments(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();

        while !self.check(&Token::RParen) && !self.is_at_end() {
            args.push(self.parse_expr()?);

            if self.check(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(args)
    }

    fn parse_list_items(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut items = Vec::new();

        while !self.check(&Token::RBracket) && !self.is_at_end() {
            items.push(self.parse_expr()?);

            if self.check(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(items)
    }

    fn parse_object_fields(&mut self) -> Result<Vec<(String, Expr)>, ParseError> {
        let mut fields = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let key = self.expect_identifier()?;
            self.expect(Token::Colon)?;
            let value = self.parse_expr()?;
            fields.push((key, value));

            if self.check(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(fields)
    }

    fn parse_lambda_params(&mut self) -> Result<Vec<String>, ParseError> {
        let mut params = Vec::new();

        while !self.check(&Token::RParen) && !self.is_at_end() {
            params.push(self.expect_identifier()?);

            if self.check(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(params)
    }

    // ==================== Helper Methods ====================
    fn current(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.position += 1;
        }
        self.tokens.get(self.position - 1).unwrap_or(&Token::Eof)
    }

    fn check(&self, token: &Token) -> bool {
        std::mem::discriminant(self.current()) == std::mem::discriminant(token)
    }

    fn peek_is(&self, token: &Token) -> bool {
        self.tokens
            .get(self.position + 1)
            .map(|t| std::mem::discriminant(t) == std::mem::discriminant(token))
            .unwrap_or(false)
    }

    fn peek_next(&self) -> &Token {
        self.tokens.get(self.position + 1).unwrap_or(&Token::Eof)
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current(), Token::Eof)
    }

    fn expect(&mut self, token: Token) -> Result<(), ParseError> {
        if self.check(&token) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("{:?}", token),
                found: self.current().clone(),
            })
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        if let Token::Identifier(name) = self.current().clone() {
            self.advance();
            Ok(name)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: self.current().clone(),
            })
        }
    }

    fn expect_identifier_eq(&mut self, expected: &str) -> Result<(), ParseError> {
        let name = self.expect_identifier()?;
        if name == expected {
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: expected.to_string(),
                found: Token::Identifier(name),
            })
        }
    }

    fn expect_string(&mut self) -> Result<String, ParseError> {
        if let Token::String(s) = self.current().clone() {
            self.advance();
            Ok(s)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: "string".to_string(),
                found: self.current().clone(),
            })
        }
    }

    fn is_lambda_start(&self) -> bool {
        let mut pos = self.position;

        while pos < self.tokens.len() {
            match &self.tokens[pos] {
                Token::Identifier(_) | Token::Comma => pos += 1,
                Token::RParen => {
                    if pos + 1 < self.tokens.len() {
                        return matches!(&self.tokens[pos + 1], Token::Arrow);
                    }
                    return false;
                }
                _ => return false,
            }
        }

        false
    }
}

/// Parse source code into a Program AST
pub fn parse(source: &str) -> Result<Program, ParseError> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

/// Parse and validate source code
pub fn parse_and_validate(source: &str) -> Result<Program, ParseError> {
    let program = parse(source)?;
    
    if let Err(e) = syntax::validate_program(&program) {
        return Err(ParseError::InvalidExpression(e));
    }
    
    Ok(program)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_state() {
        let source = r#"
            state {
                counter: 0
            }
        "#;

        let program = parse(source).unwrap();
        assert_eq!(program.state.len(), 1);
        assert_eq!(program.state[0].name, "counter");
    }

    #[test]
    fn test_parse_view() {
        let source = r#"
            state {
                count: 0
            }
            
            view {
                Text { content: "Hello" }
            }
            
            update {}
        "#;

        let program = parse(source).unwrap();
        // Test that view body exists
        match &program.view.body {
            Expr::Element { .. } => {}
            _ => panic!("Expected Element in view body"),
        }
    }

    #[test]
    fn test_parse_binary_ops() {
        let source = r#"
            state {
                result: 1 + 2 * 3
            }
        "#;

        let program = parse(source).unwrap();
        match &program.state[0].initial_value {
            Expr::BinaryOp { op: BinaryOperator::Add, .. } => {}
            _ => panic!("Expected Add at top level"),
        }
    }
}
