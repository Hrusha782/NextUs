#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Click { x: f32, y: f32, button: MouseButton },
    DoubleClick { x: f32, y: f32, button: MouseButton },
    Hover { x: f32, y: f32 },
    HoverEnd { x: f32, y: f32 },
    DragStart { x: f32, y: f32 },
    Drag { start_x: f32, start_y: f32, current_x: f32, current_y: f32 },
    DragEnd { start_x: f32, start_y: f32, end_x: f32, end_y: f32 },

    Input { text: String },
    Submit { text: String },
    Change { text: String },

    Focus,
    Blur,
    
    KeyPress { key: Key },
    KeyRelease { key: Key },

    Scroll { x: f32, y: f32 },

    ResizeStart { width: f32, height: f32 },
    Resize { current_width: f32, current_height: f32 },
    ResizeEnd { final_width: f32, final_height: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Key {
    Char(char),
    Enter,
    Escape,
    Tab,
    Space,
    Backspace,
    Delete,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Shift,
    Control,
    Alt,
}