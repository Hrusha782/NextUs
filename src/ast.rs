use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub state: Vec<StateDecl>,
    pub view: ViewDecl,
    pub update: Vec<UpdateRule>,
    pub styles: Vec<StyleRule>,
    /// CSS custom-property overrides (vars { } block)
    pub vars: HashMap<String, String>,
    /// Behavioural/runtime settings (config { } block)
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateDecl {
    pub name: String,
    pub initial_value: Expr,
    pub var_type: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewDecl {
    pub params: Vec<String>,  
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateRule {
    pub pattern: MessagePattern,
    pub state_update: HashMap<String, Expr>,
}


#[derive(Debug, Clone, PartialEq)]
pub enum MessagePattern {
    Clicked { id: String },
    DoubleClicked { id: String },
    RightClicked { id: String },
    InputChanged { id: String, value_var: Option<String> },
    FormSubmitted { id: String },
    CheckboxToggled { id: String, checked_var: Option<String> },
    DropdownSelected { id: String, value_var: Option<String> },
    MessageSent,
    MessageReceived { msg_var: String },
    MessageDeleted { id_var: String },
    MessageEdited { id_var: String, text_var: String },
    ReactionAdded { msg_id_var: String, emoji_var: String },
    ReactionRemoved { msg_id_var: String, emoji_var: String },
    TypingStarted { user_var: String },
    TypingStopped { user_var: String },
    SearchQueryChanged { query_var: String },
    ChatSelected { id_var: String },
    Scrolled { id: String, position_var: String },
    KeyPressed { id: String, key_var: String },
    Any,  
}


#[derive(Debug, Clone, PartialEq)]
pub struct StyleRule {
    pub selector: StyleSelector,
    pub properties: HashMap<String, Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StyleSelector {
    Element(String),          
    Variant(String, String),  
    Id(String),               
    Class(String),            
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Variable(String),
    PropertyAccess {
        object: Box<Expr>,
        property: String,
    },
    
    Element {
        element_type: String,
        properties: Vec<(String, Expr)>,
        children: Vec<Expr>,
    },
    
    List(Vec<Expr>),
    Object(Vec<(String, Expr)>),
    
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOperator,
        right: Box<Expr>,
    },
    
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expr>,
    },
    
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },
    
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
    
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    
    Lambda {
        params: Vec<String>,
        body: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
    
    And,
    Or,
    
    Concat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Not,
    Negate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    String,
    Number,
    Boolean,
    List(Box<Type>),
    Object(Vec<(String, Type)>),
    Element,
    Any,
}

impl Program {
    pub fn new() -> Self {
        Self {
            state: Vec::new(),
            view: ViewDecl {
                params: vec!["state".to_string()],
                body: Expr::Null,
            },
            update: Vec::new(),
            styles: Vec::new(),
            vars: HashMap::new(),
            config: HashMap::new(),
        }
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}
