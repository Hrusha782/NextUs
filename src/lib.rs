pub mod style;
pub mod properties;
pub mod element;
pub mod message;
pub mod state;
pub mod ast;
pub mod runtime;
pub mod limits;
pub mod validator;
pub mod error;
pub mod parser;
pub mod renderer;
pub mod primitives {
    pub mod layout;
    pub mod containers;
    pub mod messenger;
    pub mod content;
    pub mod input;
    pub mod scrolling;
}

#[cfg(test)]
mod tests;
pub use element::Element;
pub use style::{Style, Size, Padding, Margins, Colour, Borders, Alignment};
pub use message::Message;
pub use state::{State, Value, StateHistory};
pub use ast::Program;
pub use runtime::{Runtime, RuntimeError};
pub use limits::{RuntimeLimits, ValidationError};
pub use primitives::layout::{Container, Row, Column, Stack, Grid, Spacer, Divider, Direction, Orientation};
pub use primitives::scrolling::{Scroll, List, MessageList, ScrollBar, MessageItem, ScrollDirection, ScrollBarVisibility, ListOrientation};
pub use primitives::containers::{Part, Tab, Panel, Popover, Modal, TabItem, TabBarPosition, PopoverPosition};
pub use primitives::messenger::{MessageBubble, Reaction, Reply, Attachment, TypingIndicator, SearchBar, MessageBubbleType, ReactionItem, AttachmentType, TypingUser, SearchScope, Filters};
pub use primitives::content::{Text, Image, Avatar, Badge, Tooltip, Status, Timestamp};
pub use primitives::input::{Input, Button, Checkbox, Dropdown};
pub const VERSION: &str = env!("CARGO_PKG_VERSION");