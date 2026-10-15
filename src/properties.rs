#[derive(Debug, Clone, PartialEq)]
pub struct Properties {
    pub id: Option<String>,     
    pub class: Vec<String>,        
    pub visible: bool,
    pub enabled: bool,            
    pub focusable: bool,
    pub draggable: bool,
    /// After the user interacts with this element, the App moves keyboard focus
    /// to the element whose `id` matches this string.
    /// E.g. `focusAfter: "msg-input"` on a Button means: after clicking this
    /// button, focus the element with id "msg-input".
    pub focus_after: Option<String>,
    /// Page file to navigate to when this element is clicked.
    /// The value is the filename (without `.nxt`) of another file in the
    /// same design folder. `"back"` is a built-in value that returns to the
    /// previous page.
    pub navigate: Option<String>,
    /// Backend action to execute when the element is activated.
    /// E.g. `action: "change_nick"` fires the change_nick command.
    pub action: Option<String>,
    /// Id of an Input element whose current value is forwarded as the action's
    /// single user-supplied argument. Omit for zero-argument actions.
    pub action_input: Option<String>,
    /// Id of any element whose text content will be replaced with the action
    /// result ("✓ Done" on success, "✗ <error>" on failure).
    pub result_to: Option<String>,
    /// Template string used to format each item in an array result before it
    /// is written into the `result_to` element.  Use `{field}` placeholders
    /// matching the keys returned by the backend action.
    ///
    /// Examples:
    ///   `resultFormat: "{nickname}"`            — show only the nickname
    ///   `resultFormat: "{nickname} ({id})"`     — nickname and id
    ///   `resultFormat: "{nickname} — {id}"`     — with em-dash
    ///
    /// If omitted the default is `{nickname}` for list actions.
    pub result_format: Option<String>,
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            id: None,
            class: Vec::new(),
            visible: true,
            enabled: true,
            focusable: false,
            draggable: false,
            focus_after: None,
            navigate: None,
            action: None,
            action_input: None,
            result_to: None,
            result_format: None,
        }
    }
}

impl Properties {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id(self, id: impl Into<String>) -> Self {
        Self { id: Some(id.into()), ..self }
    }

    pub fn with_class(mut self, class: impl Into<String>) -> Self {
        self.class.push(class.into());
        self
    }

    pub fn visible(self, visible: bool) -> Self {
        Self { visible, ..self }
    }

    pub fn enabled(self, enabled: bool) -> Self {
        Self { enabled, ..self }
    }

    pub fn focusable(self, focusable: bool) -> Self {
        Self { focusable, ..self }
    }

    pub fn draggable(self, draggable: bool) -> Self {
        Self { draggable, ..self }
    }

    pub fn with_focus_after(self, focus_after: Option<String>) -> Self {
        Self { focus_after, ..self }
    }

    pub fn with_navigate(self, navigate: Option<String>) -> Self {
        Self { navigate, ..self }
    }

    pub fn with_action(self, action: Option<String>) -> Self {
        Self { action, ..self }
    }

    pub fn with_action_input(self, action_input: Option<String>) -> Self {
        Self { action_input, ..self }
    }

    pub fn with_result_to(self, result_to: Option<String>) -> Self {
        Self { result_to, ..self }
    }

    pub fn with_result_format(self, result_format: Option<String>) -> Self {
        Self { result_format, ..self }
    }
}