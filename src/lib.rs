pub mod ast;
pub mod element;
pub mod error;
pub mod limits;
pub mod message;
pub mod parser;
pub mod properties;
pub mod renderer;
pub mod runtime;
pub mod state;
pub mod style;
pub mod validator;
pub mod primitives {
    pub mod containers;
    pub mod content;
    pub mod input;
    pub mod layout;
    pub mod messenger;
    pub mod scrolling;
}

#[cfg(test)]
mod tests;
pub use ast::Program;
pub use element::Element;
pub use limits::{RuntimeLimits, ValidationError};
pub use message::Message;
pub use primitives::containers::{
    Modal, Panel, Part, Popover, PopoverPosition, Tab, TabBarPosition, TabItem,
};
pub use primitives::content::{Avatar, Badge, Image, Status, Text, Timestamp, Tooltip};
pub use primitives::input::{Button, Checkbox, Dropdown, Input};
pub use primitives::layout::{
    Column, Container, Direction, Divider, Grid, Orientation, Row, Spacer, Stack,
};
pub use primitives::messenger::{
    Attachment, AttachmentType, Filters, MessageBubble, MessageBubbleType, Reaction, ReactionItem,
    Reply, SearchBar, SearchScope, TypingIndicator, TypingUser,
};
pub use primitives::scrolling::{
    List, ListOrientation, MessageItem, MessageList, Scroll, ScrollBar, ScrollBarVisibility,
    ScrollDirection,
};
pub use runtime::{Runtime, RuntimeError};
pub use state::{State, StateHistory, Value};
pub use style::{Alignment, Borders, Colour, Margins, Padding, Size, Style};
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
