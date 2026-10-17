//! Element and property information for completions/hover

/// Information about a DSL element
pub struct ElementInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub properties: &'static [PropertyInfo],
    pub can_have_children: bool,
}

/// Information about an element property
pub struct PropertyInfo {
    pub name: &'static str,
    pub typ: &'static str,
    pub description: &'static str,
    pub default: Option<&'static str>,
}

/// All available elements in the DSL
pub static ELEMENT_INFO: &[ElementInfo] = &[
    // Layout elements
    ElementInfo {
        name: "Column",
        description: "A vertical layout container that stacks children from top to bottom.",
        properties: &[
            PropertyInfo { name: "spacing", typ: "number", description: "Space between children in pixels", default: Some("0") },
            PropertyInfo { name: "padding", typ: "number", description: "Inner padding", default: Some("0") },
        ],
        can_have_children: true,
    },
    ElementInfo {
        name: "Row",
        description: "A horizontal layout container that arranges children from left to right.",
        properties: &[
            PropertyInfo { name: "spacing", typ: "number", description: "Space between children in pixels", default: Some("0") },
            PropertyInfo { name: "wrap", typ: "boolean", description: "Whether to wrap children to next line", default: Some("false") },
        ],
        can_have_children: true,
    },
    ElementInfo {
        name: "Stack",
        description: "A container that stacks children on top of each other (z-axis).",
        properties: &[
            PropertyInfo { name: "direction", typ: "string", description: "\"Vertical\" or \"Horizontal\"", default: Some("\"Vertical\"") },
            PropertyInfo { name: "spacing", typ: "number", description: "Space between children", default: Some("0") },
        ],
        can_have_children: true,
    },
    ElementInfo {
        name: "Grid",
        description: "A grid layout with rows and columns.",
        properties: &[
            PropertyInfo { name: "columns", typ: "number", description: "Number of columns", default: Some("2") },
            PropertyInfo { name: "column_gap", typ: "number", description: "Gap between columns", default: Some("0") },
            PropertyInfo { name: "row_gap", typ: "number", description: "Gap between rows", default: Some("0") },
        ],
        can_have_children: true,
    },
    ElementInfo {
        name: "Spacer",
        description: "An empty space element for layout purposes.",
        properties: &[
            PropertyInfo { name: "grow", typ: "number", description: "Flex grow factor", default: None },
        ],
        can_have_children: false,
    },
    ElementInfo {
        name: "Divider",
        description: "A visual separator line.",
        properties: &[
            PropertyInfo { name: "orientation", typ: "string", description: "\"Horizontal\" or \"Vertical\"", default: Some("\"Horizontal\"") },
            PropertyInfo { name: "thickness", typ: "number", description: "Line thickness in pixels", default: Some("1") },
        ],
        can_have_children: false,
    },
    ElementInfo {
        name: "Scroll",
        description: "A scrollable container for content that exceeds available space.",
        properties: &[
            PropertyInfo { name: "direction", typ: "string", description: "\"Vertical\", \"Horizontal\", or \"Both\"", default: Some("\"Vertical\"") },
            PropertyInfo { name: "snap_to_bottom", typ: "boolean", description: "Auto-scroll to bottom on new content", default: Some("false") },
            PropertyInfo { name: "smooth_scroll", typ: "boolean", description: "Enable smooth scrolling animation", default: Some("true") },
        ],
        can_have_children: true,
    },
    ElementInfo {
        name: "List",
        description: "A virtualized list for efficiently displaying many items.",
        properties: &[
            PropertyInfo { name: "orientation", typ: "string", description: "\"Vertical\" or \"Horizontal\"", default: Some("\"Vertical\"") },
            PropertyInfo { name: "item_height", typ: "number", description: "Height of each item for virtualization", default: Some("48") },
            PropertyInfo { name: "spacing", typ: "number", description: "Space between items", default: Some("0") },
        ],
        can_have_children: true,
    },

    // Content elements
    ElementInfo {
        name: "Text",
        description: "Displays text content.",
        properties: &[
            PropertyInfo { name: "content", typ: "string", description: "The text to display", default: Some("\"\"") },
            PropertyInfo { name: "id", typ: "string", description: "Unique identifier", default: None },
        ],
        can_have_children: false,
    },
    ElementInfo {
        name: "Image",
        description: "Displays an image.",
        properties: &[
            PropertyInfo { name: "src", typ: "string", description: "Image source URL or path", default: Some("\"\"") },
            PropertyInfo { name: "alt", typ: "string", description: "Alternative text for accessibility", default: Some("\"\"") },
        ],
        can_have_children: false,
    },
    ElementInfo {
        name: "Avatar",
        description: "A circular image for user avatars.",
        properties: &[
            PropertyInfo { name: "src", typ: "string", description: "Image source URL", default: Some("\"\"") },
            PropertyInfo { name: "size", typ: "number", description: "Avatar diameter in pixels", default: Some("40") },
            PropertyInfo { name: "fallback", typ: "string", description: "Text to show if image fails", default: None },
        ],
        can_have_children: false,
    },
    ElementInfo {
        name: "Badge",
        description: "A small status indicator or label.",
        properties: &[
            PropertyInfo { name: "content", typ: "string", description: "Badge text", default: Some("\"\"") },
            PropertyInfo { name: "variant", typ: "string", description: "\"Primary\", \"Success\", \"Warning\", \"Error\", \"Info\"", default: Some("\"Primary\"") },
        ],
        can_have_children: false,
    },
    ElementInfo {
        name: "Status",
        description: "Shows online/offline status indicator.",
        properties: &[
            PropertyInfo { name: "status", typ: "string", description: "\"Online\", \"Offline\", \"Away\", \"Busy\", \"DoNotDisturb\"", default: Some("\"Online\"") },
            PropertyInfo { name: "show_text", typ: "boolean", description: "Show status text label", default: Some("false") },
            PropertyInfo { name: "size", typ: "number", description: "Indicator size in pixels", default: Some("12") },
        ],
        can_have_children: false,
    },
    ElementInfo {
        name: "Timestamp",
        description: "Displays a formatted time/date.",
        properties: &[
            PropertyInfo { name: "time", typ: "number", description: "Unix timestamp", default: Some("0") },
            PropertyInfo { name: "format", typ: "string", description: "\"ShortTime\", \"LongTime\", \"ShortDate\", \"LongDate\", \"DateTime\", \"Relative\"", default: Some("\"ShortTime\"") },
            PropertyInfo { name: "relative", typ: "boolean", description: "Show relative time (e.g., '2 hours ago')", default: Some("false") },
        ],
        can_have_children: false,
    },
    ElementInfo {
        name: "Tooltip",
        description: "A popup hint that appears on hover.",
        properties: &[
            PropertyInfo { name: "content", typ: "string", description: "Tooltip text", default: Some("\"\"") },
            PropertyInfo { name: "target_id", typ: "string", description: "ID of element to attach to", default: Some("\"\"") },
            PropertyInfo { name: "position", typ: "string", description: "\"Top\", \"Bottom\", \"Left\", \"Right\", \"Auto\"", default: Some("\"Top\"") },
            PropertyInfo { name: "delay_ms", typ: "number", description: "Delay before showing in milliseconds", default: Some("500") },
        ],
        can_have_children: false,
    },

    // Input elements
    ElementInfo {
        name: "Button",
        description: "A clickable button.",
        properties: &[
            PropertyInfo { name: "text", typ: "string", description: "Button label", default: Some("\"\"") },
            PropertyInfo { name: "id", typ: "string", description: "Unique identifier for event handling", default: None },
            PropertyInfo { name: "variant", typ: "string", description: "\"Primary\", \"Secondary\", \"Success\", \"Danger\", \"Ghost\", \"Link\"", default: Some("\"Primary\"") },
        ],
        can_have_children: false,
    },
    ElementInfo {
        name: "Input",
        description: "A text input field.",
        properties: &[
            PropertyInfo { name: "placeholder", typ: "string", description: "Placeholder text", default: Some("\"\"") },
            PropertyInfo { name: "value", typ: "string", description: "Current input value", default: Some("\"\"") },
            PropertyInfo { name: "id", typ: "string", description: "Unique identifier for event handling", default: None },
            PropertyInfo { name: "type", typ: "string", description: "\"Text\", \"Password\", \"Email\", \"Number\", \"Search\"", default: Some("\"Text\"") },
            PropertyInfo { name: "max_length", typ: "number", description: "Maximum character limit", default: None },
            PropertyInfo { name: "readonly", typ: "boolean", description: "Prevent editing", default: Some("false") },
            PropertyInfo { name: "required", typ: "boolean", description: "Mark as required field", default: Some("false") },
        ],
        can_have_children: false,
    },
    ElementInfo {
        name: "Checkbox",
        description: "A checkbox toggle.",
        properties: &[
            PropertyInfo { name: "checked", typ: "boolean", description: "Whether checked", default: Some("false") },
            PropertyInfo { name: "label", typ: "string", description: "Label text", default: Some("\"\"") },
            PropertyInfo { name: "id", typ: "string", description: "Unique identifier", default: None },
        ],
        can_have_children: false,
    },
    ElementInfo {
        name: "Dropdown",
        description: "A dropdown select menu.",
        properties: &[
            PropertyInfo { name: "placeholder", typ: "string", description: "Placeholder text", default: Some("\"\"") },
            PropertyInfo { name: "searchable", typ: "boolean", description: "Allow filtering options", default: Some("false") },
            PropertyInfo { name: "multi_select", typ: "boolean", description: "Allow multiple selections", default: Some("false") },
            PropertyInfo { name: "id", typ: "string", description: "Unique identifier", default: None },
        ],
        can_have_children: false,
    },

    // Container elements
    ElementInfo {
        name: "Panel",
        description: "A collapsible panel with header.",
        properties: &[
            PropertyInfo { name: "header", typ: "string", description: "Panel header text", default: Some("\"\"") },
            PropertyInfo { name: "expanded", typ: "boolean", description: "Whether panel is expanded", default: Some("true") },
            PropertyInfo { name: "animate", typ: "boolean", description: "Animate expand/collapse", default: Some("true") },
        ],
        can_have_children: true,
    },
    ElementInfo {
        name: "Modal",
        description: "A modal dialog overlay.",
        properties: &[
            PropertyInfo { name: "closable", typ: "boolean", description: "Show close button", default: Some("true") },
            PropertyInfo { name: "centered", typ: "boolean", description: "Center in viewport", default: Some("true") },
            PropertyInfo { name: "blur_background", typ: "boolean", description: "Blur content behind modal", default: Some("false") },
            PropertyInfo { name: "dismiss_on_click_outside", typ: "boolean", description: "Close when clicking outside", default: Some("true") },
        ],
        can_have_children: true,
    },
    ElementInfo {
        name: "Popover",
        description: "A popup anchored to another element.",
        properties: &[
            PropertyInfo { name: "position", typ: "string", description: "\"Top\", \"Bottom\", \"Left\", \"Right\"", default: Some("\"Bottom\"") },
            PropertyInfo { name: "shadow", typ: "boolean", description: "Show drop shadow", default: Some("true") },
            PropertyInfo { name: "arrow", typ: "boolean", description: "Show arrow pointer", default: Some("true") },
        ],
        can_have_children: true,
    },
    ElementInfo {
        name: "Tab",
        description: "A tabbed container.",
        properties: &[
            PropertyInfo { name: "active_index", typ: "number", description: "Index of active tab", default: Some("0") },
            PropertyInfo { name: "position", typ: "string", description: "Tab bar position: \"Top\", \"Bottom\", \"Left\", \"Right\"", default: Some("\"Top\"") },
        ],
        can_have_children: true,
    },
    ElementInfo {
        name: "Part",
        description: "A resizable panel section.",
        properties: &[
            PropertyInfo { name: "resizable", typ: "boolean", description: "Allow user resizing", default: Some("false") },
        ],
        can_have_children: true,
    },

    // Messenger primitives
    ElementInfo {
        name: "Container",
        description: "A generic box wrapping its children. Note: the runtime currently reads no properties on Container — size, padding and background are fixed defaults.",
        properties: &[],
        can_have_children: true,
    },
    ElementInfo {
        name: "MessageList",
        description: "A scrolling list of chat messages with grouping and auto-scroll.",
        properties: &[
            PropertyInfo { name: "spacing", typ: "number", description: "Space between messages", default: Some("0") },
            PropertyInfo { name: "group_spacing", typ: "number", description: "Extra space between message groups", default: Some("0") },
            PropertyInfo { name: "show_timestamps", typ: "boolean", description: "Show a timestamp per message", default: Some("false") },
            PropertyInfo { name: "auto_scroll", typ: "boolean", description: "Scroll to newest message on new content", default: Some("false") },
            PropertyInfo { name: "reverse_order", typ: "boolean", description: "Render newest message first", default: Some("false") },
            PropertyInfo { name: "scroll_threshold", typ: "number", description: "Distance from bottom that still counts as \"at bottom\"", default: Some("0") },
        ],
        can_have_children: true,
    },
    ElementInfo {
        name: "MessageBubble",
        description: "A single chat message bubble.",
        properties: &[
            PropertyInfo { name: "type", typ: "string", description: "\"Sent\", \"Received\" or \"System\"", default: Some("\"Received\"") },
            PropertyInfo { name: "max_width", typ: "number", description: "Maximum bubble width in pixels", default: Some("0") },
            PropertyInfo { name: "show_tail", typ: "boolean", description: "Draw the pointer tail", default: Some("false") },
            PropertyInfo { name: "tail_size", typ: "number", description: "Tail size in pixels", default: Some("0") },
            PropertyInfo { name: "shadow", typ: "boolean", description: "Drop shadow behind the bubble", default: Some("false") },
        ],
        can_have_children: true,
    },
    ElementInfo {
        name: "Reaction",
        description: "Emoji reactions attached to a message.",
        properties: &[
            PropertyInfo { name: "size", typ: "number", description: "Emoji size in pixels", default: Some("0") },
            PropertyInfo { name: "spacing", typ: "number", description: "Space between reactions", default: Some("0") },
            PropertyInfo { name: "show_count", typ: "boolean", description: "Show the per-emoji count", default: Some("false") },
            PropertyInfo { name: "max_display", typ: "number", description: "Maximum reactions shown before collapsing", default: Some("0") },
        ],
        can_have_children: false,
    },
    ElementInfo {
        name: "Reply",
        description: "A quoted preview of the message being replied to.",
        properties: &[
            PropertyInfo { name: "original_sender", typ: "string", description: "Nickname of the quoted author", default: Some("\"\"") },
            PropertyInfo { name: "original_text", typ: "string", description: "Text of the quoted message", default: Some("\"\"") },
            PropertyInfo { name: "max_preview_length", typ: "number", description: "Truncate the preview past this many characters", default: Some("0") },
            PropertyInfo { name: "show_icon", typ: "boolean", description: "Show the reply icon", default: Some("false") },
        ],
        can_have_children: false,
    },
    ElementInfo {
        name: "Attachment",
        description: "A file attached to a message.",
        properties: &[
            PropertyInfo { name: "file_name", typ: "string", description: "Display name of the file", default: Some("\"\"") },
            PropertyInfo { name: "file_type", typ: "string", description: "\"Image\", \"Video\", \"Audio\", \"Document\" or \"Other\"", default: Some("\"Other\"") },
            PropertyInfo { name: "file_size", typ: "number", description: "Size in bytes", default: Some("0") },
            PropertyInfo { name: "show_preview", typ: "boolean", description: "Render an inline preview", default: Some("false") },
            PropertyInfo { name: "show_download", typ: "boolean", description: "Show the download affordance", default: Some("false") },
        ],
        can_have_children: false,
    },
    ElementInfo {
        name: "TypingIndicator",
        description: "Shows which members are currently typing.",
        properties: &[
            PropertyInfo { name: "text", typ: "string", description: "Label template shown beside the dots", default: Some("\"\"") },
            PropertyInfo { name: "show_avatars", typ: "boolean", description: "Show avatars of typing users", default: Some("false") },
            PropertyInfo { name: "max_display", typ: "number", description: "Maximum users listed by name", default: Some("0") },
        ],
        can_have_children: false,
    },
    ElementInfo {
        name: "SearchBar",
        description: "A search input with optional scope selector and filters.",
        properties: &[
            PropertyInfo { name: "placeholder", typ: "string", description: "Placeholder text", default: Some("\"\"") },
            PropertyInfo { name: "value", typ: "string", description: "Current query", default: Some("\"\"") },
            PropertyInfo { name: "show_scope_selector", typ: "boolean", description: "Show the scope dropdown", default: Some("false") },
            PropertyInfo { name: "show_filters", typ: "boolean", description: "Show the filter controls", default: Some("false") },
            PropertyInfo { name: "show_results_count", typ: "boolean", description: "Show the result count", default: Some("false") },
        ],
        can_have_children: false,
    },
];

/// Get element info by name
pub fn get_element_info(name: &str) -> Option<&'static ElementInfo> {
    ELEMENT_INFO.iter().find(|e| e.name == name)
}

/// Get all element names
#[allow(dead_code)]
pub fn get_element_names() -> Vec<&'static str> {
    ELEMENT_INFO.iter().map(|e| e.name).collect()
}
