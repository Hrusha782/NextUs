# NextUs - Project Structure Overview

## ✅ Completed Structure

### Core Language Components

**AST (Abstract Syntax Tree)** - [`src/ast.rs`](src/ast.rs)
- Program structure (state, view, update, styles)
- Expression types (literals, operators, elements)
- Message patterns for update rules
- Type system

**Parser** - [`src/parser/`](src/parser/)
- [`mod.rs`](src/parser/mod.rs) - Main parser logic
- [`lexer.rs`](src/parser/lexer.rs) - Tokenizer
- [`syntax.rs`](src/parser/syntax.rs) - Syntax utilities

**Runtime** - [`src/runtime.rs`](src/runtime.rs)
- Pure expression evaluator
- State management
- Message handling
- Built-in functions

**State Management** - [`src/state.rs`](src/state.rs)
- Immutable state
- Value types
- State history (undo/redo)
- Time-travel debugging

### UI Primitives

**Layout** - [`src/primitives/layout.rs`](src/primitives/layout.rs)
- Container, Row, Column, Stack, Grid, Spacer, Divider

**Content** - [`src/primitives/content.rs`](src/primitives/content.rs)
- Text, Image, Avatar, Badge, Tooltip, Status, Timestamp

**Input** - [`src/primitives/input.rs`](src/primitives/input.rs)
- Input, Button, Checkbox, Dropdown

**Messenger** - [`src/primitives/messenger.rs`](src/primitives/messenger.rs)
- MessageBubble, Reaction, Reply, Attachment, TypingIndicator, SearchBar

**Containers** - [`src/primitives/containers.rs`](src/primitives/containers.rs)
- Part, Tab, Panel, Popover, Modal

**Scrolling** - [`src/primitives/scrolling.rs`](src/primitives/scrolling.rs)
- Scroll, List, MessageList

### Supporting Systems

**Messages** - [`src/message.rs`](src/message.rs)
- Pure message types for user interactions
- No callbacks, just data

**Properties** - [`src/properties.rs`](src/properties.rs)
- Pure element properties (id, class, visible, enabled)

**Style** - [`src/style.rs`](src/style.rs)
- Colour, Padding, Margins, Borders, Alignment
- Size, Position types

**Limits** - [`src/limits.rs`](src/limits.rs)
- Resource limits (depth, memory, time)
- Validation

**Element** - [`src/element.rs`](src/element.rs)
- Unified Element enum for all primitives

**Library** - [`src/lib.rs`](src/lib.rs)
- Public API exports

## 📁 Project Structure

```
NextUs/
├── src/
│   ├── lib.rs                    # Main library exports
│   ├── ast.rs                    # AST definitions
│   ├── runtime.rs                # Runtime engine
│   ├── state.rs                  # State management
│   ├── message.rs                # Message types
│   ├── properties.rs             # Element properties
│   ├── style.rs                  # Styling types
│   ├── limits.rs                 # Resource limits
│   ├── element.rs                # Unified Element enum
│   ├── error.rs                  # Error types
│   ├── validator.rs              # Validation logic
│   ├── context.rs                # Context types
│   ├── interactions.rs           # Interaction handling
│   ├── parser/
│   │   ├── mod.rs               # Parser
│   │   ├── lexer.rs             # Tokenizer
│   │   └── syntax.rs            # Syntax utilities
│   ├── primitives/
│   │   ├── mod.rs
│   │   ├── layout.rs            # Layout primitives
│   │   ├── content.rs           # Content primitives
│   │   ├── input.rs             # Input primitives
│   │   ├── messenger.rs         # Messenger primitives
│   │   ├── containers.rs        # Container primitives
│   │   └── scrolling.rs         # Scrolling primitives
│   ├── renderer/
│   │   ├── mod.rs
│   │   ├── layout_engine.rs     # Layout calculation
│   │   └── memory_tracker.rs    # Memory tracking
│   └── tests/
│
├── examples/
│   ├── counter.nxt              # Simple counter example
│   └── messenger.nxt            # Full messenger example
│
├── DSL_README.md                # Language documentation
└── Cargo.toml
```

## 🎯 Architecture Flow

```
1. User writes .nxt file
   ↓
2. Lexer tokenizes source code
   ↓
3. Parser builds AST (Program)
   ↓
4. Validator checks safety constraints
   ↓
5. Runtime initializes with state
   ↓
6. Runtime.render() → Element tree
   ↓
7. Renderer displays UI
   ↓
8. User interacts → Message generated
   ↓
9. Runtime.update(message) → new State
   ↓
10. Loop back to step 6
```

## ✅ Key Features Implemented

### Pure & Deterministic
- ✅ No side effects
- ✅ Pure functions only
- ✅ Same input = same output
- ✅ Immutable data structures

### Safety
- ✅ Resource limits (depth, memory, time)
- ✅ No file system access
- ✅ No network access
- ✅ No global state
- ✅ No random numbers

### Functional
- ✅ Declarative UI
- ✅ Message-based updates
- ✅ State history (undo/redo)
- ✅ Time-travel debugging

### Messenger-Optimized
- ✅ Message bubbles with reactions
- ✅ Virtual scrolling for performance
- ✅ Typing indicators
- ✅ Search functionality
- ✅ Attachments
- ✅ Threaded replies

## 🚧 Next Steps to Complete

### 1. Element Creation in Runtime
Implement `create_element()` in [`runtime.rs`](src/runtime.rs) to convert AST elements to actual UI primitives.

### 2. Renderer Implementation
Build the actual renderer in [`src/renderer/`](src/renderer/) to display UI:
- Layout engine (calculate positions/sizes)
- Graphics backend (wgpu, skia, or similar)
- Event handling

### 3. Lambda Support
Add closure support for `map()`, `filter()`, etc.

### 4. Style Application
Implement style rule matching and application.

### 5. More Built-in Functions
Add remaining functions: `map`, `filter`, `reduce`, etc.

### 6. Testing
Write comprehensive tests for:
- Parser
- Runtime
- State management
- Validators

### 7. Documentation
- API documentation
- Tutorial
- Cookbook with examples

### 8. CLI Tool
Create a command-line tool to:
- Parse .nxt files
- Run applications
- Debug (time-travel)

## 🎓 Language Principles

Your DSL follows these principles:

1. **Declarative over Imperative** - Describe WHAT, not HOW
2. **Pure over Impure** - No side effects
3. **Immutable over Mutable** - All data is immutable
4. **Explicit over Implicit** - Everything is explicit
5. **Safe over Fast** - Safety first, performance second
6. **Simple over Complex** - Easy to learn and use

## 📚 Resources

- **Language Spec**: See [DSL_README.md](DSL_README.md)
- **Examples**: See [examples/](examples/)
- **Source Code**: See [src/](src/)

## 🚀 Getting Started

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run example (once renderer is complete)
cargo run --example messenger
```

## 💡 Example Usage

```nextus
// Simple app
state {
    message: "Hello, NextUs!"
}

view(state) {
    Text { content: state.message }
}

update {
    when Clicked { id: "button" } => {
        message: "Clicked!"
    }
}
```

This creates a **pure, deterministic, open-source messenger framework** that users can customize without writing Rust! 🎉
