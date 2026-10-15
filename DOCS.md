# NextUs Language Documentation

NextUs (`.nxt`) is a declarative, pure, domain-specific language for building messenger UI components. It is sandboxed — no file system, network, or global state access — and is designed to be deterministic and safe.

---

## Table of Contents

1. [Program Structure](#1-program-structure)
2. [State](#2-state)
3. [View](#3-view)
4. [Update](#4-update)
5. [Style](#5-style)
6. [Expressions](#6-expressions)
7. [Built-in Functions](#7-built-in-functions)
8. [UI Primitives](#8-ui-primitives)
   - [Layout](#layout)
   - [Content](#content)
   - [Input](#input)
   - [Scrolling & Lists](#scrolling--lists)
   - [Containers & Overlays](#containers--overlays)
   - [Messenger-Specific](#messenger-specific)
   - [Backend Actions](#backend-actions)
   - [Result Formatting](#result-formatting)
9. [Events (Messages)](#9-events-messages)
10. [Resource Limits](#10-resource-limits)
11. [Full Examples](#11-full-examples)

---

## 1. Program Structure

A `.nxt` file has four top-level sections. Only `view` is strictly required; the others are optional.

```
state { ... }    -- initial data
view  { ... }    -- UI tree (pure function of state)
update { ... }   -- how messages change state
style  { ... }   -- visual rules
```

Sections can appear in any order. Comments use `//`.

---

## 2. State

Declares named variables and their initial values. All state is immutable per render — only `update` rules produce a new state.

```nextus
state {
    count: 0,
    username: "guest",
    messages: [],
    logged_in: false
}
```

### Types

Values are inferred from their literals:

| Literal | Type |
|---|---|
| `0`, `3.14` | Number |
| `"hello"` | String |
| `true`, `false` | Boolean |
| `null` | Null |
| `[]` | List |
| `{}` | Object |

Type annotations are optional:

```nextus
state {
    count: 0        -- inferred as Number
}
```

---

## 3. View

Describes the entire UI as a single expression. It is a pure function — given the same state, it always returns the same UI tree.

```nextus
view {
    Column {
        spacing: 10
        Text { content: "Hello!" }
    }
}
```

State fields are accessed directly by name inside `view`:

```nextus
state {
    name: "Alice"
}

view {
    Text { content: name }
}
```

You can also declare view with an explicit parameter name:

```nextus
view(state) {
    Text { content: state.name }
}
```

### Nesting Elements

Elements nest by placing child elements inside the parent's block:

```nextus
Column {
    spacing: 8
    Row {
        Button { id: "ok", text: "OK" }
        Button { id: "cancel", text: "Cancel" }
    }
    Text { content: "Footer" }
}
```

Alternatively, use the explicit `children` property:

```nextus
Column {
    children: [
        Text { content: "A" },
        Text { content: "B" }
    ]
}
```

---

## 4. Update

Defines how the state changes in response to user events. Each rule matches an event pattern and returns a partial state object — only the listed fields are updated; the rest are unchanged.

```nextus
update {
    when Clicked { id: "increment" } => {
        count: count + 1
    }

    when InputChanged { id: "search", value: query } => {
        search_text: query
    }
}
```

- Field references inside the `=> { }` block refer to the **current** state fields by name.
- Only fields listed in the result are updated; omitted fields keep their current value.
- Rules are matched top-to-bottom; the first matching rule wins.

### Wildcard Rule

```nextus
update {
    when _ => {
        -- matches any unhandled event; state unchanged
    }
}
```

---

## 5. Style

Applies visual properties to elements by type, variant, id, or class. Style blocks are separate from the view and apply globally.

```nextus
style Button {
    padding: 10,
    backgroundColor: #0084ff,
    textColor: white,
    borderRadius: 5
}

style Button.Primary {
    backgroundColor: #0084ff
}

style Button.Danger {
    backgroundColor: #e53935
}

style #send_button {
    fontWeight: bold
}

style .highlighted {
    backgroundColor: #fff3cd
}

style MessageBubble.Sent {
    alignment: right,
    backgroundColor: #0084ff,
    textColor: white
}

style MessageBubble.Received {
    alignment: left,
    backgroundColor: #f0f0f0
}
```

### Style Selector Forms

| Selector | Matches |
|---|---|
| `ElementName` | All elements of that type |
| `ElementName.Variant` | Elements with a specific variant |
| `#id` | Element with that exact `id` |
| `.classname` | Elements with that class |

### Style Properties

| Property | Type | Description |
|---|---|---|
| `padding` | number | All-sides padding (px) |
| `margin` | number | All-sides margin (px) |
| `backgroundColor` | color | Background fill |
| `textColor` | color | Text color |
| `borderRadius` | number | Corner radius (px) |
| `borderWidth` | number | Border thickness (px) |
| `borderColor` | color | Border color |
| `fontSize` | number | Font size (px) |
| `width` | number / `"auto"` / `"50%"` | Element width |
| `height` | number / `"auto"` / `"50%"` | Element height |
| `alignment` | `left` / `center` / `right` / `stretch` | Content alignment |

### Color Values

```nextus
backgroundColor: #ff5500      -- hex
backgroundColor: white        -- named: white, black, transparent, red, green, blue
```

---

## 6. Expressions

Expressions can appear as property values in `view`, as right-hand sides of state updates, and inside `if` expressions.

### Literals

```nextus
42
3.14
"hello"
true
false
null
```

### Arithmetic

```nextus
count + 1
price * 1.2
total - discount
index % 10
width // 2         -- integer division
```

### Comparison

```nextus
count == 0
name != "guest"
score >= 100
length < 50
```

### Logic

```nextus
logged_in && has_chats
not locked
is_admin || is_moderator
```

Or written as symbols:

```nextus
logged_in && has_chats
!locked
```

### String Concatenation

```nextus
"Hello, " + username + "!"
```

### Property Access

```nextus
user.name
message.timestamp
state.messages
```

### Conditionals

**Inline if/else:**

```nextus
if count > 0 then "positive" else "zero or negative"
```

**Ternary:**

```nextus
count > 0 ? "positive" : "zero or negative"
```

The `else` branch is optional when used as a statement inside `update`, but required in expressions used as a value.

### Lists

```nextus
[1, 2, 3]
[Text { content: "A" }, Text { content: "B" }]
[]
```

### Objects

```nextus
{ name: "Alice", age: 30 }
```

### Lambda (Anonymous Function)

Used with `map`, `filter`, etc.:

```nextus
map(messages, msg -> msg.text)
filter(items, item -> item.visible)
```

---

## 7. Built-in Functions

### List Operations

| Function | Description |
|---|---|
| `length(list)` | Number of items |
| `append(list, item)` | New list with item added at the end |
| `map(list, fn)` | New list with `fn` applied to each item |
| `filter(list, fn)` | New list with only items where `fn` returns true |
| `sort_by(list, fn)` | New list sorted ascending by the key returned by `fn` |

```nextus
length(messages)
append(messages, new_message)
map(items, item -> item.text)
filter(messages, m -> m.visible)
sort_by(messages, m -> m.timestamp)
```

### String Operations

| Function | Description |
|---|---|
| `length(string)` | Character count |
| `concat(a, b)` | Concatenate two strings (same as `a + b`) |
| `substring(str, start, end)` | Extract a slice (0-indexed) |
| `contains(str, search)` | Returns `true` if `search` is found in `str` |

```nextus
length(input_text)
contains(message, "urgent")
substring(text, 0, 100)
```

### Utility

| Function | Description |
|---|---|
| `now()` | Current Unix timestamp (ms), supplied by the runtime |

---

## 8. UI Primitives

All elements follow this syntax:

```nextus
ElementName {
    property: value
    property: value
    ChildElement { ... }
}
```

Every element accepts these optional universal properties:

| Property | Type | Default | Description |
|---|---|---|---|
| `id` | string | — | Element identifier, used for event matching and action wiring |
| `class` | string | — | CSS class name for style selectors |
| `focusAfter` | string | — | ID of the element to focus after this element is clicked |
| `navigate` | string | — | Page file to navigate to on click (filename without `.nxt`, or `"back"`) |
| `action` | string | — | Backend action to run on click — see [Backend Actions](#backend-actions) |
| `actionInput` | string | — | ID of an `Input` whose current value is sent as the action argument |
| `resultTo` | string | — | ID of any element whose text is replaced with the action result |
| `resultFormat` | string | — | `{field}` template applied to each item in a list result — see [Result Formatting](#result-formatting) |

```nextus
Button {
    id: "send"
    text: "Send"
    variant: "Primary"
    focusAfter: "msg-input"   -- after clicking Send, cursor returns to the input
}
```

Only one element should be focused per render. If multiple elements carry `focusAfter`, the one whose interaction triggered the render wins.

---

### Backend Actions

Buttons can call backend API actions directly using the `action` property. Results are written back into the element identified by `resultTo`.

```nextus
Button {
    id: "btn-save-nick"
    text: "Save"
    variant: "Primary"
    action: "change_nick"
    actionInput: "nick-input"   -- reads the value from Input with id="nick-input"
    resultTo: "nick-result"     -- writes "✓ Nickname updated." into that element
}
Input { id: "nick-input" placeholder: "New nickname…" }
Button { id: "nick-result" text: "" variant: "Ghost" }
```

#### Available Actions

| Action | `actionInput` | Returns | Description |
|---|---|---|---|
| `change_nick` | Input ID (new nickname) | status string | Change your display nickname |
| `verify_password` | Input ID (password) | status string | Check if a password is correct |
| `add_friend` | Input ID (user ID) | status string | Send a friend request |
| `remove_friend` | Input ID (user ID) | status string | Remove a friend |
| `list_friends` | — | list `{nickname, id}` | List all friends |
| `create_chat` | Input ID (comma-separated user IDs) | status string | Create a new chat |
| `leave_chat` | — | status string | Leave the currently open chat |
| `block_user` | Input ID (user ID) | status string | Block a user |
| `unblock_user` | Input ID (user ID) | status string | Unblock a user |
| `list_blocked` | — | list `{nickname, id}` | List all blocked users |
| `list_chats` | — | list `{name, id, unread}` | List all chats |
| `logout` | — | — | Log out and return to login screen |

---

### Result Formatting

When an action returns a **list** of records, `resultFormat` is a `{field}` template applied to each item. Items are joined with newlines and written into `resultTo`.

```nextus
-- Show only the nickname per friend
Button {
    action: "list_friends"
    resultTo: "friend-result"
    resultFormat: "{nickname}"
}

-- Show nickname + id
Button {
    action: "list_friends"
    resultTo: "friend-result"
    resultFormat: "{nickname} ({id})"
}

-- Show chat name + unread count
Button {
    action: "list_chats"
    resultTo: "chats-result"
    resultFormat: "{name} ({unread} unread)"
}
```

If `resultFormat` is omitted, **all available fields** are shown automatically separated by ` | `.

#### Available fields per action

| Action | Fields |
|---|---|
| `list_friends` | `{nickname}` `{id}` |
| `list_blocked` | `{nickname}` `{id}` |
| `list_chats` | `{name}` `{id}` `{unread}` |

---

### Layout

#### `Container`

Generic box. Wraps one or more children.

| Property | Type | Default | Description |
|---|---|---|---|
| `width` | size | `auto` | Width |
| `height` | size | `auto` | Height |
| `padding` | number | 0 | Inner spacing |
| `margin` | number | 0 | Outer spacing |
| `backgroundColor` | color | — | Background color |
| `alignment` | alignment | `start` | Child alignment |
| `clip` | bool | `false` | Clip overflowing content |

---

#### `Row`

Horizontal layout. Children are placed left-to-right.

| Property | Type | Default | Description |
|---|---|---|---|
| `spacing` | number | 0 | Gap between children |
| `alignment` | alignment | `start` | Cross-axis alignment |
| `justify` | alignment | `start` | Main-axis distribution |
| `wrap` | bool | `false` | Wrap to next line when full |
| `padding` | number | 0 | Inner spacing |

```nextus
Row {
    spacing: 8
    Button { id: "ok", text: "OK" }
    Button { id: "cancel", text: "Cancel" }
}
```

---

#### `Column`

Vertical layout. Children are placed top-to-bottom.

| Property | Type | Default | Description |
|---|---|---|---|
| `spacing` | number | 0 | Gap between children |
| `alignment` | alignment | `start` | Cross-axis alignment |
| `justify` | alignment | `start` | Main-axis distribution |
| `padding` | number | 0 | Inner spacing |

```nextus
Column {
    spacing: 12
    Text { content: "Title" }
    Input { placeholder: "Type here..." }
}
```

---

#### `Stack`

Layered layout. Children overlap each other (z-order).

| Property | Type | Default | Description |
|---|---|---|---|
| `direction` | `"Vertical"` / `"Horizontal"` | `"Vertical"` | Stack axis |
| `spacing` | number | 0 | Gap between layers |
| `alignment` | alignment | `start` | Alignment |

---

#### `Grid`

Two-dimensional grid layout.

| Property | Type | Default | Description |
|---|---|---|---|
| `columns` | number | — | Number of columns (**required**) |
| `rows` | number | — | Number of rows (optional) |
| `columnGap` | number | 0 | Gap between columns |
| `rowGap` | number | 0 | Gap between rows |
| `alignment` | alignment | `start` | Cell alignment |

```nextus
Grid {
    columns: 3
    columnGap: 8
    rowGap: 8
    Text { content: "A" }
    Text { content: "B" }
    Text { content: "C" }
}
```

---

#### `Spacer`

Fills available space between elements.

| Property | Type | Description |
|---|---|---|
| `width` | size | Fixed width |
| `height` | size | Fixed height |
| `grow` | number | Flex-grow factor |

```nextus
Row {
    Text { content: "Left" }
    Spacer { grow: 1 }
    Text { content: "Right" }
}
```

---

#### `Divider`

Visual separator line.

| Property | Type | Default | Description |
|---|---|---|---|
| `orientation` | `"Horizontal"` / `"Vertical"` | `"Horizontal"` | Direction |
| `thickness` | number | 1 | Line thickness (px) |
| `color` | color | — | Line color |

---

### Content

#### `Text`

Displays text.

| Property | Type | Description |
|---|---|---|
| `content` | string / expression | Text to display (**required**) |

```nextus
Text { content: "Hello, " + username + "!" }
Text { content: message_count }
```

---

#### `Image`

Displays an image.

| Property | Type | Description |
|---|---|---|
| `src` | string | Image URL or path (**required**) |
| `alt` | string | Accessibility description |
| `width` | size | Display width |
| `height` | size | Display height |

---

#### `Avatar`

Circular user avatar with optional fallback initials.

| Property | Type | Description |
|---|---|---|
| `src` | string | Image URL |
| `size` | number | Diameter (px) |
| `fallback` | string | Text to show when image fails to load |

```nextus
Avatar {
    src: user.avatar_url
    size: 40
    fallback: "AB"
}
```

---

#### `Badge`

Label badge for counts, status, or labels.

| Property | Type | Values | Description |
|---|---|---|---|
| `content` | string | | Badge text |
| `variant` | enum | `Primary` `Success` `Warning` `Error` `Info` | Visual style |

```nextus
Badge { content: "3", variant: "Error" }
Badge { content: "New", variant: "Success" }
```

---

#### `Tooltip`

Shows a tooltip on hover near a target element.

| Property | Type | Description |
|---|---|---|
| `content` | string | Tooltip text |
| `targetId` | string | `id` of the element to attach to |
| `position` | `Top` `Bottom` `Left` `Right` `Auto` | Placement |
| `delayMs` | number | Delay before showing (ms) |

---

#### `Status`

Presence/status indicator dot.

| Property | Type | Values | Description |
|---|---|---|---|
| `status` | enum | `Online` `Offline` `Away` `Busy` `DoNotDisturb` | Status |
| `showText` | bool | | Show text label next to dot |
| `size` | number | | Dot diameter (px) |

```nextus
Status { status: "Online", showText: false }
```

---

#### `Timestamp`

Formatted date/time display.

| Property | Type | Values | Description |
|---|---|---|---|
| `time` | number | | Unix timestamp (ms) |
| `format` | enum | `ShortTime` `LongTime` `ShortDate` `LongDate` `DateTime` `FullDateTime` `Relative` | Format |
| `relative` | bool | | Force relative format ("2 hours ago") |

```nextus
Timestamp { time: message.sent_at, format: "Relative" }
```

---

### Input

#### `Button`

Clickable button. Triggers `Clicked` when pressed.

| Property | Type | Values | Description |
|---|---|---|---|
| `id` | string | | Required for event matching |
| `text` | string | | Button label |
| `variant` | enum | `Primary` `Secondary` `Success` `Danger` `Ghost` `Link` | Visual style |

```nextus
Button { id: "send", text: "Send", variant: "Primary" }
Button { id: "cancel", text: "Cancel", variant: "Ghost" }
```

---

#### `Input`

Single-line text input. Triggers `InputChanged` when typed.

| Property | Type | Values | Description |
|---|---|---|---|
| `id` | string | | Required for event matching |
| `value` | string | | Current value (bind to state) |
| `placeholder` | string | | Hint text when empty |
| `type` | enum | `Text` `Password` `Email` `Number` `Search` `Url` | Input mode |
| `maxLength` | number | | Character limit |
| `readonly` | bool | | Prevent editing |
| `required` | bool | | Mark as required |
| `focusAfter` | string | | After pressing Enter inside this input, focus the element with this id |

```nextus
Input {
    id: "message_input"
    value: input_text
    placeholder: "Type a message..."
    type: "Text"
    focusAfter: "message_input"   -- pressing Enter refocuses this same input
}
```

---

#### `Checkbox`

Boolean toggle. Triggers `CheckboxToggled` when changed.

| Property | Type | Description |
|---|---|---|
| `id` | string | Required for event matching |
| `checked` | bool | Current state |
| `label` | string | Label text |
| `indeterminate` | bool | Show indeterminate (mixed) state |

```nextus
Checkbox {
    id: "notify"
    checked: notifications_enabled
    label: "Enable notifications"
}
```

---

#### `Dropdown`

Selection menu. Triggers `DropdownSelected` when an option is chosen.

| Property | Type | Description |
|---|---|---|
| `id` | string | Required for event matching |
| `placeholder` | string | Hint when nothing selected |
| `searchable` | bool | Show a search box inside |
| `multiSelect` | bool | Allow multiple selections |

Options are passed as a list of objects:

```nextus
Dropdown {
    id: "theme"
    placeholder: "Choose theme"
    options: [
        { label: "Light", value: "light" },
        { label: "Dark", value: "dark" }
    ]
}
```

---

### Scrolling & Lists

#### `Scroll`

Scrollable container around a single child.

| Property | Type | Values | Description |
|---|---|---|---|
| `direction` | enum | `Vertical` `Horizontal` `Both` | Scroll axis |
| `width` | size | | Container width |
| `height` | size | | Container height |
| `snapToBottom` | bool | | Auto-scroll to bottom (for chats) |
| `smoothScroll` | bool | | Smooth scroll animation |

```nextus
Scroll {
    direction: "Vertical"
    height: 400
    snapToBottom: true
    Column {
        spacing: 4
        -- message elements...
    }
}
```

---

#### `List`

Virtualized list for large datasets.

| Property | Type | Description |
|---|---|---|
| `items` | list | Array of elements |
| `orientation` | `Vertical` / `Horizontal` | Layout direction |
| `itemHeight` | number | Fixed height per item (enables virtualization) |
| `spacing` | number | Gap between items |
| `snapToBottom` | bool | Auto-scroll to newest item |

---

#### `MessageList`

Chat-optimized virtualized list with grouping and auto-scroll.

| Property | Type | Description |
|---|---|---|
| `messages` | list | Array of message items |
| `spacing` | number | Gap between messages |
| `groupSpacing` | number | Extra space between different senders |
| `autoScroll` | bool | Always scroll to latest message |
| `showTimestamps` | bool | Show timestamp separators between groups |
| `reverseOrder` | bool | Newest messages at top |

```nextus
MessageList {
    messages: messages
    spacing: 4
    groupSpacing: 12
    autoScroll: true
    showTimestamps: true
}
```

---

### Containers & Overlays

#### `Part`

Named screen region. Used to divide the layout into logical areas (sidebar, main, footer).

| Property | Type | Description |
|---|---|---|
| `width` | size | Width |
| `height` | size | Height |
| `resizable` | bool | Allow user to resize |
| `minWidth` | number | Minimum width (px) |
| `maxWidth` | number | Maximum width (px) |

```nextus
Row {
    Part {
        width: 240
        resizable: true
        -- sidebar content
        Column { ... }
    }
    Part {
        width: "auto"
        -- main content
        Column { ... }
    }
}
```

---

#### `Tab`

Tab bar with multiple panels.

| Property | Type | Description |
|---|---|---|
| `activeIndex` | number | Currently shown tab (0-indexed) |
| `tabBarPosition` | `Top` `Bottom` `Left` `Right` | Where the tab bar appears |
| `tabBarHeight` | number | Height of tab bar (px) |
| `spacing` | number | Gap between tab buttons |

Tabs are defined as children with `label` and `content`:

```nextus
Tab {
    activeIndex: active_tab
    tabBarPosition: "Top"
    TabItem {
        label: "Chats"
        content: Column { ... }
    }
    TabItem {
        label: "Settings"
        content: Column { ... }
    }
}
```

---

#### `Panel`

Collapsible section with a header.

| Property | Type | Description |
|---|---|---|
| `expanded` | bool | Whether the panel is open |
| `animate` | bool | Animate open/close |
| `header` | element | Header element |
| `content` | element | Body element |

```nextus
Panel {
    expanded: show_details
    header: Text { content: "Details" }
    content: Column { ... }
}
```

---

#### `Popover`

Floating overlay anchored to a position. Dismisses on click outside.

| Property | Type | Values | Description |
|---|---|---|---|
| `position` | enum | `Top` `Bottom` `Left` `Right` `TopLeft` `TopRight` `BottomLeft` `BottomRight` `Cursor` | Where to appear |
| `shadow` | bool | | Drop shadow |
| `arrow` | bool | | Show pointer arrow |
| `dismissOnClickOutside` | bool | | Close when clicking outside |
| `dismissOnEscape` | bool | | Close on Escape key |

---

#### `Modal`

Blocking overlay dialog. Covers the screen with a backdrop.

| Property | Type | Description |
|---|---|---|
| `width` | size | Dialog width |
| `height` | size | Dialog height |
| `centered` | bool | Center on screen |
| `closable` | bool | Show a close button |
| `blurBackground` | bool | Blur the content behind |
| `overlayColor` | color | Backdrop color (supports alpha) |
| `dismissOnClickOutside` | bool | Close when clicking backdrop |
| `dismissOnEscape` | bool | Close on Escape key |
| `animate` | bool | Fade-in animation |

```nextus
Modal {
    centered: true
    width: 400
    closable: true
    blurBackground: true
    Column {
        Text { content: "Are you sure?" }
        Row {
            Button { id: "confirm", text: "Yes", variant: "Danger" }
            Button { id: "dismiss", text: "No" }
        }
    }
}
```

---

### Messenger-Specific

#### `MessageBubble`

A styled chat bubble. The `type` controls side and color.

| Property | Type | Values | Description |
|---|---|---|---|
| `type` | enum | `Sent` `Received` `System` | Message direction |
| `showTail` | bool | | Show the speech bubble pointer |
| `maxWidth` | number | | Max bubble width |
| `shadow` | bool | | Drop shadow |

```nextus
MessageBubble {
    type: "Sent"
    showTail: true
    Text { content: message.text }
}

MessageBubble {
    type: "Received"
    Text { content: message.text }
}

MessageBubble {
    type: "System"
    Text { content: "Alice joined the chat" }
}
```

---

#### `Reaction`

Emoji reaction row displayed below a message.

| Property | Type | Description |
|---|---|---|
| `reactions` | list | List of `{ emoji, count, userReacted }` objects |
| `maxDisplay` | number | Max reactions before collapse |
| `showCount` | bool | Show count numbers |
| `size` | number | Emoji size (px) |

```nextus
Reaction {
    reactions: message.reactions
    showCount: true
    maxDisplay: 5
}
```

---

#### `Reply`

Quoted reply preview shown above a message bubble.

| Property | Type | Description |
|---|---|---|
| `originalSender` | string | Name of the quoted user |
| `originalText` | string | Preview of the quoted message |
| `maxPreviewLength` | number | Truncate preview after N characters |
| `showIcon` | bool | Show reply icon |

---

#### `Attachment`

File or media attachment preview.

| Property | Type | Values | Description |
|---|---|---|---|
| `fileName` | string | | File name |
| `fileSize` | number | | File size in bytes |
| `fileType` | enum | `Image` `Video` `Audio` `Document` `Geo` `Other` | Type |
| `showDownload` | bool | | Show download button |
| `showPreview` | bool | | Show thumbnail/preview |
| `progress` | number | | Upload progress 0.0–1.0 |

```nextus
Attachment {
    fileName: "photo.jpg"
    fileSize: 204800
    fileType: "Image"
    showPreview: true
    showDownload: true
}
```

---

#### `TypingIndicator`

Animated "X is typing..." indicator.

| Property | Type | Description |
|---|---|---|
| `users` | list | List of `{ name, avatar? }` objects |
| `maxDisplay` | number | Max users to name before "..." |
| `text` | string | Override default label text |
| `showAvatars` | bool | Show user avatars |

```nextus
TypingIndicator {
    users: typing_users
    showAvatars: true
}
```

---

#### `SearchBar`

Search input with optional filters.

| Property | Type | Values | Description |
|---|---|---|---|
| `placeholder` | string | | Hint text |
| `value` | string | | Current search text |
| `searchScope` | enum | `All` `CurrentChat` `User` `Select` | Scope |
| `showScopeSelector` | bool | | Show scope dropdown |
| `showFilters` | bool | | Show filter panel |
| `showResultsCount` | bool | | Show result count |
| `resultsCount` | number | | Current result count |

```nextus
SearchBar {
    id: "main_search"
    placeholder: "Search messages..."
    value: search_query
    searchScope: "CurrentChat"
    showResultsCount: true
    resultsCount: result_count
}
```

---

## 9. Events (Messages)

Events are produced by user interaction and matched in `update` rules. The `id` always refers to the element's `id` property.

### Interaction Events

| Event | Trigger | Captured Variables |
|---|---|---|
| `Clicked { id: "x" }` | Button pressed | — |
| `DoubleClicked { id: "x" }` | Element double-clicked | — |
| `RightClicked { id: "x" }` | Element right-clicked | — |
| `InputChanged { id: "x", value: v }` | Text input changed | `v` = new text |
| `FormSubmitted { id: "x" }` | Form submitted | — |
| `CheckboxToggled { id: "x", checked: c }` | Checkbox clicked | `c` = new boolean |
| `DropdownSelected { id: "x", value: v }` | Option chosen | `v` = selected value |
| `KeyPressed { id: "x", key: k }` | Key pressed in element | `k` = key name |
| `Scrolled { id: "x", position: p }` | Scroll position changed | `p` = position |

### Chat Events

| Event | Trigger | Captured Variables |
|---|---|---|
| `MessageSent { text }` | Message send action | — |
| `MessageReceived { message: m }` | New message arrives from server | `m` = message object |
| `MessageDeleted { id: i }` | Message deleted | `i` = message id |
| `MessageEdited { id: i, text: t }` | Message edited | `i` = id, `t` = new text |
| `ReactionAdded { messageId: mid, emoji: e }` | Reaction added | `mid` = message id, `e` = emoji |
| `ReactionRemoved { messageId: mid, emoji: e }` | Reaction removed | `mid`, `e` |
| `TypingStarted { user: u }` | User starts typing | `u` = user info |
| `TypingStopped { user: u }` | User stops typing | `u` = user info |
| `SearchQueryChanged { query: q }` | Search text changed | `q` = query string |
| `ChatSelected { id: i }` | Chat item clicked | `i` = chat id |

### Usage in Update

```nextus
update {
    when Clicked { id: "send" } => {
        messages: append(messages, { text: input_text, sent: true }),
        input_text: ""
    }

    when MessageReceived { message: msg } => {
        messages: append(messages, msg)
    }

    when InputChanged { id: "message_input", value: text } => {
        input_text: text
    }

    when CheckboxToggled { id: "mute", checked: on } => {
        muted: on
    }

    when ChatSelected { id: chat_id } => {
        active_chat: chat_id,
        messages: []
    }
}
```

---

## 10. Resource Limits

The runtime enforces these hard limits to ensure safety and predictability:

| Limit | Default | Description |
|---|---|---|
| Max UI depth | 50 | Maximum nesting levels in the element tree |
| Max children per container | 100 | Children per single element |
| Max text length | 10,000 chars | Length of any single string value |
| Max list items | 1,000 | Items in any list |
| Max grid cells | 10,000 | Total cells in a Grid |
| Max tabs | 50 | Tabs in a single Tab element |
| Max memory per session | 100 MB | RAM used per running program |
| Max render time | 100 ms | Time allowed per render call |
| Max events per second | 100 | Rate-limit on incoming events |

Exceeding any limit produces a runtime error and halts the program safely without crashing the host application.

---

## 11. Full Examples

### Counter

```nextus
state {
    count: 0
}

view {
    Column {
        spacing: 10

        Text { content: "Counter: " + count }

        Row {
            spacing: 8
            Button { id: "decrement", text: "-", variant: "Secondary" }
            Button { id: "reset", text: "Reset", variant: "Ghost" }
            Button { id: "increment", text: "+", variant: "Primary" }
        }
    }
}

update {
    when Clicked { id: "increment" } => {
        count: count + 1
    }
    when Clicked { id: "decrement" } => {
        count: count - 1
    }
    when Clicked { id: "reset" } => {
        count: 0
    }
}
```

---

### Messenger UI

```nextus
state {
    messages: [],
    input_text: "",
    typing_users: []
}

view {
    Column {
        -- Header
        Row {
            Avatar { src: "", size: 36, fallback: "CH" }
            Column {
                Text { content: "Chat Room" }
                Status { status: "Online", showText: true }
            }
            Spacer { grow: 1 }
            Button { id: "search_btn", text: "Search", variant: "Ghost" }
        }

        Divider {}

        -- Message area
        MessageList {
            messages: messages
            autoScroll: true
            showTimestamps: true
            spacing: 4
            groupSpacing: 12
        }

        -- Typing indicator
        if length(typing_users) > 0
        then TypingIndicator { users: typing_users, showAvatars: true }
        else Spacer { height: 0 }

        Divider {}

        -- Input bar
        Row {
            spacing: 8
            Input {
                id: "message_input"
                value: input_text
                placeholder: "Type a message..."
                type: "Text"
            }
            Button {
                id: "send"
                text: "Send"
                variant: "Primary"
            }
        }
    }
}

update {
    when InputChanged { id: "message_input", value: text } => {
        input_text: text
    }

    when Clicked { id: "send" } => {
        messages: append(messages, { text: input_text, type: "Sent" }),
        input_text: ""
    }

    when MessageReceived { message: msg } => {
        messages: append(messages, msg)
    }

    when TypingStarted { user: u } => {
        typing_users: append(typing_users, u)
    }

    when TypingStopped { user: u } => {
        typing_users: filter(typing_users, t -> t.name != u.name)
    }
}

style MessageBubble.Sent {
    backgroundColor: #0084ff,
    textColor: white,
    alignment: right
}

style MessageBubble.Received {
    backgroundColor: #f0f0f0,
    textColor: black,
    alignment: left
}

style Button.Primary {
    backgroundColor: #0084ff,
    textColor: white,
    borderRadius: 8
}
```

---

### Chat List Sidebar

```nextus
state {
    chats: [],
    active_chat: null,
    search_query: ""
}

view {
    Column {
        SearchBar {
            id: "chat_search"
            value: search_query
            placeholder: "Search chats..."
            searchScope: "All"
        }

        Scroll {
            direction: "Vertical"
            List {
                items: filter(chats, c -> contains(c.name, search_query))
                orientation: "Vertical"
                spacing: 2
            }
        }
    }
}

update {
    when InputChanged { id: "chat_search", value: q } => {
        search_query: q
    }

    when ChatSelected { id: chat_id } => {
        active_chat: chat_id
    }
}
```
