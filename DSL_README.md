# NextUs DSL - Pure, Deterministic UI Language for Messengers

NextUs is a domain-specific language (DSL) for building custom messenger applications with a pure, functional approach.

## Features

✅ **Pure & Deterministic** - No side effects, same input = same output  
✅ **Declarative** - Describe what the UI should be, not how to build it  
✅ **Type-Safe** - Static analysis before execution  
✅ **Resource-Limited** - Built-in safety limits  
✅ **Immutable** - All data is immutable  
✅ **No External Access** - No file system, network, or global state  

## Language Structure

### 1. State Declaration
Define your application state with initial values:

```nextus
state {
    count: 0,
    message: "Hello!",
    items: []
}
```

### 2. View Declaration
Pure function that describes the UI:

```nextus
view(state) {
    Column {
        children: [
            Text { content: state.message },
            Button { id: "click_me", label: "Click" }
        ]
    }
}
```

### 3. Update Rules
Pattern match on messages and return new state:

```nextus
update {
    when Clicked { id: "click_me" } => {
        count: state.count + 1
    }
    
    when InputChanged { id: "input", value } => {
        message: value
    }
}
```

### 4. Styling (Optional)
Style your UI elements:

```nextus
style Button {
    padding: 10,
    backgroundColor: #0084ff,
    textColor: white,
    borderRadius: 5
}

style MessageBubble.Sent {
    alignment: right,
    backgroundColor: #0084ff
}
```

## Available UI Primitives

### Universal Element Properties

Every element accepts these optional properties:

| Property | Type | Description |
|---|---|---|
| `id` | string | Identifier for event matching and action wiring |
| `class` | string | CSS class for style selectors |
| `focusAfter` | string | ID of element to focus after this element is activated |
| `navigate` | string | Page to navigate to on click (filename without `.nxt`, or `"back"`) |
| `action` | string | Backend action to run on click |
| `actionInput` | string | ID of `Input` whose value is sent as the action argument |
| `resultTo` | string | ID of element whose text receives the action result |
| `resultFormat` | string | `{field}` template applied to each item in a list result |

### Backend Actions

Buttons can call backend actions using `action:`. Results write back via `resultTo:` + `resultFormat:`.

```nextus
Button {
    id: "btn-list-friends"
    text: "Show Friends"
    variant: "Ghost"
    action: "list_friends"
    resultTo: "friends-output"
    resultFormat: "{nickname}"   -- one nickname per line
}
Button { id: "friends-output" text: "" variant: "Ghost" }
```

| Action | `actionInput` | Result fields | Description |
|---|---|---|---|
| `change_nick` | Input ID | status | Change display nickname |
| `verify_password` | Input ID | status | Check password |
| `add_friend` | Input ID (user ID) | status | Send friend request |
| `remove_friend` | Input ID (user ID) | status | Remove friend |
| `list_friends` | — | `{nickname}` `{id}` | List friends |
| `create_chat` | Input ID (comma-sep IDs) | status | Create new chat |
| `leave_chat` | — | status | Leave current chat |
| `block_user` | Input ID (user ID) | status | Block a user |
| `unblock_user` | Input ID (user ID) | status | Unblock a user |
| `list_blocked` | — | `{nickname}` `{id}` | List blocked users |
| `list_chats` | — | `{name}` `{id}` `{unread}` | List all chats |
| `logout` | — | — | Log out |

### Result Formatting

When an action returns a list, `resultFormat` is a `{field}` template applied to each item (newline-separated):

```nextus
resultFormat: "{nickname}"              -- nicknames only
resultFormat: "{nickname} ({id})"       -- nickname + id
resultFormat: "{name} ({unread} unread)"-- chat name + unread count
```

If omitted, all available fields are shown separated by ` | `.

### Layout
- `Container` - Generic box
- `Row` - Horizontal layout
- `Column` - Vertical layout
- `Stack` - Layered elements
- `Grid` - 2D grid layout
- `Spacer` - Empty space
- `Divider` - Visual separator

### Content
- `Text` - Text display
- `Image` - Image display
- `Avatar` - User avatar
- `Badge` - Label badge
- `Tooltip` - Hover tooltip
- `Status` - Status indicator
- `Timestamp` - Time display

### Input
- `Input` - Text input field
- `Button` - Clickable button
- `Checkbox` - Boolean toggle
- `Dropdown` - Selection menu

### Messenger-Specific
- `MessageBubble` - Styled message
- `MessageList` - Virtualized message list
- `Reaction` - Emoji reactions
- `Reply` - Threaded reply
- `Attachment` - File attachment
- `TypingIndicator` - Typing animation
- `SearchBar` - Search input

### Containers
- `Part` - Screen region
- `Tab` - Tab system
- `Panel` - Collapsible section
- `Popover` - Dismissible overlay
- `Modal` - Blocking overlay

### Scrolling
- `Scroll` - Scrollable container
- `List` - Virtual list
- `MessageList` - Chat-optimized list

## Message Types

Your update rules can match these messages:

```nextus
Clicked { id: "button_id" }
DoubleClicked { id: "element_id" }
InputChanged { id: "input_id", value }
CheckboxToggled { id: "checkbox_id", checked }
MessageSent { text }
MessageReceived { message }
ReactionAdded { messageId, emoji }
TypingStarted { userId }
ChatSelected { chatId }
SearchQueryChanged { query }
```

## Built-in Functions

### List Operations
- `length(list)` - Get list length
- `append(list, item)` - Add item to list
- `map(list, fn)` - Transform each item
- `filter(list, fn)` - Filter items

### String Operations
- `length(string)` - String length
- `concat(str1, str2)` - Concatenate strings
- `substring(str, start, end)` - Extract substring
- `contains(str, search)` - Check if contains

### Utility
- `now()` - Current timestamp (provided by runtime)

## Operators

### Arithmetic
`+`, `-`, `*`, `/`, `%`, `//`

### Comparison
`==`, `!=`, `<`, `<=`, `>`, `>=`

### Logical
`and`, `or`, `not`

### String
`+` (concatenation)

## Control Flow

### Conditionals
```nextus
if condition then value1 else value2
```

### Ternary
```nextus
count > 0 ? "positive" : "zero or negative"
```

## Resource Limits

NextUs enforces these limits for safety:

- Max UI depth: 50 levels
- Max children per container: 100
- Max text length: 10,000 characters
- Max list items: 1,000
- Max grid cells: 10,000
- Max tabs: 50
- Max memory per session: 100 MB
- Max render time: 100 ms
- Max events per second: 100

## Examples

### Simple Counter

```nextus
state {
    count: 0
}

view(state) {
    Column {
        children: [
            Text { content: "Count: " + state.count },
            Button { id: "inc", label: "+" },
            Button { id: "dec", label: "-" }
        ]
    }
}

update {
    when Clicked { id: "inc" } => {
        count: state.count + 1
    }
    
    when Clicked { id: "dec" } => {
        count: state.count - 1
    }
}
```

### Messenger

See [`examples/messenger.nxt`](examples/messenger.nxt) for a full messenger implementation.

## Architecture

```
User's .nxt file
    ↓
Lexer (tokenize)
    ↓
Parser (AST)
    ↓
Validator (check safety)
    ↓
Runtime (pure execution)
    ↓
Renderer (UI display)
```

## Safety Guarantees

1. **No side effects** - All functions are pure
2. **Deterministic** - Same state + same message = same new state
3. **No crashes** - Resource limits prevent abuse
4. **No secrets** - No file/network access
5. **Time-travel debugging** - Complete state history
6. **Undo/Redo** - Built-in state history

## Usage

```rust
use nextus::{Parser, Runtime, RuntimeLimits};

fn main() {
    // Parse your .nxt file
    let source = std::fs::read_to_string("app.nxt").unwrap();
    let program = Parser::parse(&source).unwrap();
    
    // Create runtime with limits
    let limits = RuntimeLimits::default();
    let mut runtime = Runtime::new(program, limits).unwrap();
    
    // Render initial UI
    let ui = runtime.render().unwrap();
    display(ui);
    
    // Handle user interaction
    let message = get_user_action();
    runtime.update(message, timestamp).unwrap();
    
    // Re-render
    let ui = runtime.render().unwrap();
    display(ui);
}
```

## License

Open source - users can create their own custom messengers!

## Contributing

This is a pure, deterministic language. All contributions must maintain:
- Pure functions only
- No side effects
- No external access
- Deterministic behavior
- Resource limits
