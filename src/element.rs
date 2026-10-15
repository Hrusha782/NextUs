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