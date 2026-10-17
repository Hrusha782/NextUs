use crate::primitives::content::*;
use crate::primitives::layout::*;
use crate::primitives::input::*;
use crate::primitives::scrolling::*;
use crate::primitives::containers::*;
use crate::primitives::messenger::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Element {
    Text(Text),
    Image(Image),
    Avatar(Avatar),
    Badge(Badge),
    Status(Status),
    Timestamp(Timestamp),
    Tooltip(Tooltip),

    Container(Container),
    Row(Row),
    Column(Column),
    Stack(Stack),
    Grid(Grid),
    Spacer(Spacer),
    Divider(Divider),

    Input(Input),
    Checkbox(Checkbox),
    Dropdown(Dropdown),
    Button(Button),

    Scroll(Box<Scroll>),
    List(List),
    MessageList(MessageList),

    Part(Part),
    Tab(Tab),
    Panel(Box<Panel>),
    Popover(Popover),
    Modal(Modal),

    MessageBubble(MessageBubble),
    Reaction(Reaction),
    Reply(Reply),
    Attachment(Attachment),
    TypingIndicator(TypingIndicator),
    SearchBar(SearchBar),
}

/// Every element name the runtime can build, in one place.
///
/// This is the contract shared by `create_element` (runtime), `validate_program`
/// (`--check`) and the LSP completion list. The names match the `Element`
/// variants above one for one; `element_names_match_runtime` in `src/tests`
/// fails if a name is added here without a matching runtime arm, or the reverse.
///
/// Deliberately absent: `TopBar` and the left rail. Those are app chrome and
/// must never be programmable from a design or a build.
pub const ELEMENT_NAMES: &[&str] = &[
    "Attachment", "Avatar", "Badge", "Button",
    "Checkbox", "Column", "Container", "Divider",
    "Dropdown", "Grid", "Image", "Input",
    "List", "MessageBubble", "MessageList", "Modal",
    "Panel", "Part", "Popover", "Reaction",
    "Reply", "Row", "Scroll", "SearchBar",
    "Spacer", "Stack", "Status", "Tab",
    "Text", "Timestamp", "Tooltip", "TypingIndicator",
];

/// Whether `name` is an element the runtime can build.
pub fn is_known_element(name: &str) -> bool {
    ELEMENT_NAMES.contains(&name)
}
