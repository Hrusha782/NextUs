use std::collections::HashMap;
use crate::ast::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    List(Vec<Value>),
    Object(HashMap<String, Value>),
    Element(crate::Element),
    Lambda { params: Vec<String>, body: Box<Expr> },
}

impl Value {
    pub fn as_string(&self) -> Option<&str> {
        if let Value::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        if let Value::Number(n) = self {
            Some(*n)
        } else {
            None
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        if let Value::Boolean(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    pub fn as_list(&self) -> Option<&Vec<Value>> {
        if let Value::List(l) = self {
            Some(l)
        } else {
            None
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, Value>> {
        if let Value::Object(o) = self {
            Some(o)
        } else {
            None
        }
    }

    pub fn as_element(&self) -> Option<&crate::Element> {
        if let Value::Element(e) = self {
            Some(e)
        } else {
            None
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Null => false,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Object(o) => !o.is_empty(),
            Value::Element(_) => true,
            Value::Lambda { .. } => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    variables: HashMap<String, Value>,
}

impl State {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub fn set(&self, name: String, value: Value) -> Self {
        let mut new_state = self.clone();
        new_state.variables.insert(name, value);
        new_state
    }

    pub fn update_many(&self, updates: HashMap<String, Value>) -> Self {
        let mut new_state = self.clone();
        for (name, value) in updates {
            new_state.variables.insert(name, value);
        }
        new_state
    }

    pub fn variables(&self) -> &HashMap<String, Value> {
        &self.variables
    }

    pub fn with_variables(variables: HashMap<String, Value>) -> Self {
        Self { variables }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateSnapshot {
    pub state: State,
    pub timestamp: u64,
    pub message: Option<crate::message::Message>,
}

impl StateSnapshot {
    pub fn new(state: State, timestamp: u64, message: Option<crate::message::Message>) -> Self {
        Self {
            state,
            timestamp,
            message,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StateHistory {
    snapshots: Vec<StateSnapshot>,
    current_index: usize,
    max_history: usize,
}

impl StateHistory {
    pub fn new(initial_state: State, max_history: usize) -> Self {
        Self {
            snapshots: vec![StateSnapshot::new(initial_state, 0, None)],
            current_index: 0,
            max_history,
        }
    }

    pub fn push(&mut self, state: State, timestamp: u64, message: Option<crate::message::Message>) {
        self.snapshots.truncate(self.current_index + 1);

        self.snapshots.push(StateSnapshot::new(state, timestamp, message));

        if self.snapshots.len() > self.max_history {
            self.snapshots.remove(0);
        } else {
            self.current_index += 1;
        }
    }

    pub fn current(&self) -> &State {
        &self.snapshots[self.current_index].state
    }

    pub fn undo(&mut self) -> Option<&State> {
        if self.current_index > 0 {
            self.current_index -= 1;
            Some(&self.snapshots[self.current_index].state)
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<&State> {
        if self.current_index < self.snapshots.len() - 1 {
            self.current_index += 1;
            Some(&self.snapshots[self.current_index].state)
        } else {
            None
        }
    }

    pub fn can_undo(&self) -> bool {
        self.current_index > 0
    }

    pub fn can_redo(&self) -> bool {
        self.current_index < self.snapshots.len() - 1
    }

    pub fn snapshots(&self) -> &[StateSnapshot] {
        &self.snapshots
    }
}