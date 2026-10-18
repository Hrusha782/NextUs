#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Clicked {
        id: String,
    },
    DoubleClicked {
        id: String,
    },
    RightClicked {
        id: String,
    },
    HoverStarted {
        id: String,
    },
    HoverEnded {
        id: String,
    },

    InputChanged {
        id: String,
        value: String,
    },
    FormSubmitted {
        id: String,
    },
    CheckboxToggled {
        id: String,
        checked: bool,
    },
    DropdownSelected {
        id: String,
        value: String,
    },

    DragStarted {
        id: String,
        x: f32,
        y: f32,
    },
    Dragged {
        id: String,
        x: f32,
        y: f32,
    },
    DragEnded {
        id: String,
        x: f32,
        y: f32,
    },

    Focused {
        id: String,
    },
    Blurred {
        id: String,
    },

    KeyPressed {
        id: String,
        key: String,
    },
    KeyReleased {
        id: String,
        key: String,
    },

    Scrolled {
        id: String,
        position: f32,
    },
    ScrolledToBottom {
        id: String,
    },

    MessageSent {
        text: String,
    },
    MessageReceived {
        message_id: String,
        sender: String,
        text: String,
        timestamp: i64,
    },
    MessageDeleted {
        message_id: String,
    },
    MessageEdited {
        message_id: String,
        new_text: String,
    },
    ReactionAdded {
        message_id: String,
        emoji: String,
    },
    ReactionRemoved {
        message_id: String,
        emoji: String,
    },
    TypingStarted {
        user_id: String,
    },
    TypingStopped {
        user_id: String,
    },
    AttachmentSelected {
        file_name: String,
        file_size: u64,
    },
    SearchQueryChanged {
        query: String,
    },
    ChatSelected {
        chat_id: String,
    },

    None,
}
