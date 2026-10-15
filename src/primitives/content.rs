use crate::style::Style;
use crate::properties::Properties;

#[derive(Debug, Clone, PartialEq)]
pub struct Text { 
    pub properties: Properties,
    pub style: Style,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub properties: Properties,
    pub style: Style,
    pub src: String,
    pub alt: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Avatar {
    pub properties: Properties,
    pub style: Style,
    pub src: String,
    pub size: f32,
    pub fallback: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Badge {
    pub properties: Properties,
    pub style: Style,
    pub content: String,
    pub variant: BadgeVariant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BadgeVariant {
    Primary,
    Success,
    Warning,
    Error,
    Info,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tooltip {
    pub properties: Properties,
    pub style: Style,
    pub content: String,
    pub target_id: String,
    pub position: TooltipPosition,
    pub delay_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TooltipPosition {
    Top,
    Bottom,
    Left,
    Right,
    Auto,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Status {
    pub properties: Properties,
    pub style: Style,
    pub status: StatusType,
    pub show_text: bool,
    pub size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatusType {
    Online,
    Offline,
    Away,
    Busy,
    DoNotDisturb,
    Custom { color: crate::style::Colour },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Timestamp {
    pub properties: Properties,
    pub style: Style,
    pub time: u64,
    pub format: TimestampFormat,
    pub relative: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimestampFormat {
    ShortTime,        // "14:30"
    LongTime,         // "14:30:45"
    ShortDate,        // "Jan 11"
    LongDate,         // "January 11, 2026"
    DateTime,         // "Jan 11, 14:30"
    FullDateTime,     // "January 11, 2026 14:30:45"
    Relative,         // "2 hours ago", "just now"
}