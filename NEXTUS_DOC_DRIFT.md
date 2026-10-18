# NextUs: docs vs code drift (checklist)

Found on 2026-10-16 by running test programs through `nextus --check` and `nextus <file>` on the initial commit.
Re-checked on 2026-10-17; ticked items are done on branch `fix/language-blockers-week1-track-b`.
Some of these are doc bugs and some are missing features. Decide per item which side to fix, then tick it off and delete this file when it's empty.

Where docs and code disagreed and neither side was clearly intended, the code won
(per CLAUDE.md) and the docs were corrected.

## Parser / lexer
- [x] `--` comments are used throughout DOCS.md and DESIGN_IDS.txt but the lexer only supports `//` and `/* */` (parse error "found Minus"). → docs now use `//`.
- [x] Ternary `a ? b : c` is documented, but there's no `?` token (lex error). → removed from DOCS.md, which now points at `if … then … else`.
- [x] `and` / `or` / `not` are documented as operators but aren't keywords. Only `&&` `||` `!` work. → DOCS.md documents only the symbol forms and says the word forms do not exist.
- [x] Bare-parameter lambdas `x -> x.v` are documented. Only `(x) -> x.v` parses. → DOCS.md uses the parenthesised form everywhere.
- [x] `//` integer division is documented, but `//` starts a comment. → removed from DOCS.md. The two cannot coexist; `//` stays a comment.
- [ ] `let … in` has tokens but no parser support.
- [ ] `when MessageReceived { message: m }` (docs) vs parser expecting `{ msg: m }`. **Needs a decision**: rename the parser field or the docs.
- [ ] `when MessageSent { text }` (docs) vs parser accepting only bare `MessageSent`.
- [ ] Runtime has `HoverStarted/Ended`, `Focused`, `Blurred`, `Drag*`, `KeyReleased`, `ScrolledToBottom`, `AttachmentSelected` messages, but the parser rejects them as patterns.
- [ ] `vars { }` and `config { }` top-level blocks exist but aren't documented.
- [ ] `parse_style_block` / `parse_style_rule` are a second, unused style parser (dead-code warning). Remove them or wire them in. The documented `style <selector> { }` form works via a different path.

## Runtime
- [x] String concatenation fails: `"a" + "b"` and `"Count: " + count` give "Cannot apply Add to String…" at render (the documented counter example breaks). Only Number + Number works. `BinaryOperator::Concat` exists but `+` doesn't map to it. → `+` now concatenates when either side is a string, via one shared `Value::to_display_string`. Bounded by `max_text_length`.
- [ ] Built-ins `concat`, `substring`, `contains`, `now` are documented but undefined ("UndefinedFunction"). DOCS.md now marks them as not defined, so the drift is gone, but the functions are still missing. `now` must stay host-supplied to keep the runtime deterministic.
- [ ] **Correction to the original entry:** `map` and `filter` are *not* implemented — they are stubs returning `InvalidOperation("map not yet implemented")`. Only `length`, `append`, `sort_by` and `.length` work, and `sort_by` only accepts a lambda whose body is a property access, which it pattern-matches into a key name rather than calling. Implementing `map`/`filter` means threading state, limits and depth into the builtin signature so a lambda can actually be invoked. DOCS.md now states the real status.
- [x] **Found on 2026-10-17, not in the original list:** only 7 of the 15 `BinaryOperator` variants were implemented. `Modulo`, `Equal`, `NotEqual`, `LessThan`, `LessOrEqual`, `GreaterThan` and `GreaterOrEqual` all fell through to "Cannot apply …", so every documented comparison and `%` failed at render and no `if` with a condition could work. → implemented. Equality compares any two values; ordering covers two numbers or two strings, with `NaN` making comparisons false as in IEEE 754.
- [ ] `register_builtins()` only registers `length` and `append`; `call_builtin` has its own list. One source of truth would be better. (`_builtin_functions` is unused — the leading underscore hides it.)
- [ ] Resource limits: only depth, text length and list items are enforced during eval. `validator::validate` (children per container, etc.) is never called. Grid cells, tabs, memory, render time and events/s aren't enforced (ChatSick #52). DOCS.md now has an "Enforced" column so the table stops overstating.
- [ ] `Container` reads none of its documented properties — the runtime builds it with fixed defaults, so `padding`, `backgroundColor` etc. are silently dropped. DOCS.md flags this; the runtime still needs to read them.
- [ ] Unknown *properties* are still silently ignored (e.g. a typo'd `txet:` on a Button renders an empty label). Only unknown *elements* are caught. Needs per-element property lists to be complete first.

- [ ] **Found on 2026-10-17:** `MessageDeleted`, `MessageEdited`, `ReactionAdded`, `ReactionRemoved`, `TypingStarted`, `TypingStopped` and `SearchQueryChanged` parse their field blocks and then throw them away, returning `MessagePattern::Any`. A rule written for one of them silently becomes a wildcard that matches every event, with its captured variable unbound — a trap rather than an error. `ChatSelected` is likewise rewritten into `InputChanged { id: "chat-list" }`. The AST has correct variants for all of them; only the parser is missing. DOCS.md now warns about this.
- [ ] Seven `child_elements.into_iter().next().unwrap()` calls in `create_element` (Scroll, Panel, Popover, Modal, …). Each is guarded by `len() == 1` so none can panic today, but they sit on the runtime path that CLAUDE.md principle 5 covers; a `match` would make the guard impossible to break by accident.

## Validation / tooling
- [x] `--check` passes unknown elements (`Foo {}`, `TopBar {}`) and type errors. They fail only at render. → `validate_program` now rejects any element not in `element::ELEMENT_NAMES`, across `view`, `state` initial values and `update` right-hand sides. **Correction to the original entry:** the LSP's `ELEMENT_INFO` was *not* a usable source for this list — it was missing 8 of the 32 names, including `Container`. The list now lives in `src/element.rs` and a test keeps it in step with the runtime. Type errors are still render-time only.
- [x] `Button { label: … }` (DSL_README) parses but renders empty. The runtime reads `text`. → DSL_README fixed to `text`. (`Checkbox` genuinely does read `label`, so DOCS.md is right there.)
- [x] LSP completion was missing `Container`, `MessageList`, `MessageBubble`, `Reaction`, `Reply`, `Attachment`, `TypingIndicator` and `SearchBar` — 8 of 32 elements, so autocomplete silently omitted them. → added to `ELEMENT_INFO` with the property names the runtime actually reads.
- [x] `cargo clippy --all-targets` failed outright on `approx_constant` in a lexer test (`3.14`), so the documented lint command and CI (#6) could never pass. → allowed on that test, with a note that it is a decimal literal under test.
- [ ] `ParseError` carries no line/column at all. The lexer tracks position but drops it when emitting `Vec<Token>`, so `lsp/diagnostics.rs` recovers a position by string-scraping the error's `Display` text — which only works for `LexError`. Every parse error lands at line 0. This breaks CLAUDE.md principle 4 and blocks #15 (inline parse errors in the editor). Fixing it means positioned tokens: lexer → parser → LSP.
- [ ] `validate_program` returns `Result<(), String>` with no position either, so the new unknown-element error cannot point at the offending line. Same root cause as above.

## Docs
- [x] STRUCTURE.md lists `context.rs` and `interactions.rs`, which don't exist, and describes a wgpu/skia graphics backend. The product direction is HTML/DOM with `data-nxt-id`. → module tree corrected (and `lsp/` added, which was missing); the graphics backend section now says HTML/DOM explicitly. Its "Next Steps" also claimed `create_element()` was unimplemented when all 32 elements are done — corrected to name DOM emission as the real gap.
- [x] STRUCTURE.md: `cargo run --example messenger` (there's no `[[example]]`; the examples are `.nxt` files run via the CLI). → replaced with the real CLI invocations.
- [x] DESIGN_IDS.txt used `Label { }` in its syntax example and prose, but `Label` is not a NextUs primitive, so `--check` now rejects it. → changed to `Text { content: … }`. **Worth confirming app-side**: if the ChatSick app expects a `Label` element, the primitive is missing rather than the doc being wrong. The ids themselves were not touched.
- [ ] DOCS.md vs DSL_README.md overlap and disagree. Pick one reference and generate or trim the other. **Needs a decision.**
