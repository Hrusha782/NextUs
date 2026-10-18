//! HTML/DOM output for the ChatSick shell.
//!
//! `create_element` turns the AST into typed primitives; this module turns
//! those primitives into the HTML the host app mounts.
//!
//! Two rules come from `DESIGN_IDS.txt` and must not be broken:
//!
//! 1. An element with an `id` carries it as `data-nxt-id`. That attribute is
//!    how the app finds elements (`querySelector('[data-nxt-id="send"]')`).
//! 2. Layout containers mirror the `.nxt` tree exactly: their children are
//!    direct DOM children, with no wrapper in between, because the app walks
//!    `.parentElement` to find, for example, the column holding the message
//!    context menu. A few composite elements do add inner nodes of their own
//!    -- `MessageList`, `Modal`, `Panel`, `Tab`, `Checkbox`, `Reaction`,
//!    `Reply`, `Attachment` and `SearchBar` -- and DOCS.md section 11 lists
//!    them. Nothing fabricates a `data-nxt-id` the program did not write.
//!
//! Output is compact: no whitespace is inserted between nodes, so nothing
//! here can introduce a stray text node. It is also deterministic — the same
//! element tree always produces byte-identical HTML.

use crate::element::Element;
use crate::primitives::containers::{
    Modal, Panel, Part, Popover, PopoverPosition, Tab, TabBarPosition,
};
use crate::primitives::content::{
    Avatar, Badge, BadgeVariant, Image, Status, StatusType, Text, Timestamp, TimestampFormat,
    Tooltip, TooltipPosition,
};
use crate::primitives::input::{Button, ButtonVariant, Checkbox, Dropdown, Input, InputType};
use crate::primitives::layout::{
    Column, Container, Direction, Divider, Grid, Orientation, Row, Spacer, Stack,
};
use crate::primitives::messenger::{
    Attachment, AttachmentType, MessageBubble, MessageBubbleType, Reaction, Reply, SearchBar,
    TypingIndicator,
};
use crate::primitives::scrolling::{List, ListOrientation, MessageList, Scroll, ScrollDirection};
use crate::properties::Properties;
use crate::state::format_number;
use crate::style::{Alignment, Colour, Padding, Size, Style};

/// Render an element tree as HTML.
pub fn render_html(element: &Element) -> String {
    let mut out = String::new();
    write_element(&mut out, element);
    out
}

/// Escape text that becomes character data.
fn escape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

/// Escape a double-quoted attribute value.
///
/// A `.nxt` program is untrusted input, so every string that reaches the host
/// DOM goes through here or [`escape_text`]. Quotes and angle brackets are the
/// characters that would otherwise let a value break out of its attribute.
fn escape_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// A CSS length in pixels, with whole numbers printed without a decimal point.
fn px(value: f32) -> String {
    format!("{}px", format_number(value as f64))
}

fn css_size(size: &Size) -> Option<String> {
    match size {
        Size::Fixed(v) => Some(px(*v)),
        Size::Percentage(v) => Some(format!("{}%", format_number(*v as f64))),
        Size::Auto => None,
    }
}

fn css_colour(colour: &Colour) -> String {
    if colour.a == 255 {
        format!("#{:02x}{:02x}{:02x}", colour.r, colour.g, colour.b)
    } else {
        // Three decimals is enough to round-trip an 8-bit alpha channel.
        let alpha = format!("{:.3}", f32::from(colour.a) / 255.0);
        let alpha = alpha.trim_end_matches('0').trim_end_matches('.');
        format!("rgba({},{},{},{})", colour.r, colour.g, colour.b, alpha)
    }
}

fn css_align(alignment: &Alignment) -> &'static str {
    match alignment {
        Alignment::Start => "flex-start",
        Alignment::Center => "center",
        Alignment::End => "flex-end",
        Alignment::Stretch => "stretch",
    }
}

fn css_padding(padding: &Padding) -> Option<String> {
    if padding.top == 0.0 && padding.right == 0.0 && padding.bottom == 0.0 && padding.left == 0.0 {
        return None;
    }
    Some(format!(
        "{} {} {} {}",
        px(padding.top),
        px(padding.right),
        px(padding.bottom),
        px(padding.left)
    ))
}

/// An HTML element being assembled.
///
/// Attributes are written in the order they are added, so output stays stable
/// across runs.
struct Tag {
    name: &'static str,
    classes: Vec<String>,
    attrs: Vec<(String, String)>,
    /// Valueless attributes such as `disabled` or `checked`.
    flags: Vec<String>,
    css: Vec<(String, String)>,
    void: bool,
}

impl Tag {
    fn new(name: &'static str, class: &str) -> Self {
        Self {
            name,
            classes: vec![class.to_string()],
            attrs: Vec::new(),
            flags: Vec::new(),
            css: Vec::new(),
            void: false,
        }
    }

    fn void(mut self) -> Self {
        self.void = true;
        self
    }

    fn class(mut self, class: impl Into<String>) -> Self {
        self.classes.push(class.into());
        self
    }

    fn attr(mut self, key: &str, value: impl Into<String>) -> Self {
        self.attrs.push((key.to_string(), value.into()));
        self
    }

    /// Add the attribute only when the value is non-empty, so an unset
    /// property does not become `key=""`.
    fn attr_if_set(self, key: &str, value: &str) -> Self {
        if value.is_empty() {
            self
        } else {
            self.attr(key, value.to_string())
        }
    }

    /// A valueless HTML attribute such as `disabled` or `checked`.
    ///
    /// Only for real HTML boolean attributes. A `data-` boolean goes through
    /// [`Tag::data_bool`], because `el.dataset.x === ""` is falsy in JS.
    fn flag(mut self, key: &str, on: bool) -> Self {
        if on {
            self.flags.push(key.to_string());
        }
        self
    }

    /// A `data-` boolean, always written as `"true"` or `"false"` so the host
    /// can read `dataset.x === "true"` instead of having to use
    /// `hasAttribute`.
    fn data_bool(self, key: &str, value: bool) -> Self {
        self.attr(key, if value { "true" } else { "false" })
    }

    fn css_decl(mut self, property: &str, value: impl Into<String>) -> Self {
        self.css.push((property.to_string(), value.into()));
        self
    }

    fn css_opt(self, property: &str, value: Option<String>) -> Self {
        match value {
            Some(v) => self.css_decl(property, v),
            None => self,
        }
    }

    fn size(self, width: &Size, height: &Size) -> Self {
        self.css_opt("width", css_size(width))
            .css_opt("height", css_size(height))
    }

    fn padding(self, padding: &Padding) -> Self {
        self.css_opt("padding", css_padding(padding))
    }

    /// Apply the shared `Properties`: the id the app looks for, user classes,
    /// and the behaviour hooks the shell reads off the DOM.
    fn props(mut self, properties: &Properties) -> Self {
        if let Some(id) = &properties.id {
            self = self.attr("data-nxt-id", id.clone());
        }
        for class in &properties.class {
            self = self.class(class.clone());
        }
        if let Some(value) = &properties.focus_after {
            self = self.attr("data-nxt-focus-after", value.clone());
        }
        if let Some(value) = &properties.navigate {
            self = self.attr("data-nxt-navigate", value.clone());
        }
        if let Some(value) = &properties.action {
            self = self.attr("data-nxt-action", value.clone());
        }
        if let Some(value) = &properties.action_input {
            self = self.attr("data-nxt-action-input", value.clone());
        }
        if let Some(value) = &properties.result_to {
            self = self.attr("data-nxt-result-to", value.clone());
        }
        if let Some(value) = &properties.result_format {
            self = self.attr("data-nxt-result-format", value.clone());
        }
        if properties.draggable {
            self = self.attr("draggable", "true");
        }
        if properties.focusable {
            self = self.attr("tabindex", "0");
        }
        if !properties.enabled {
            self = self.attr("data-nxt-enabled", "false");
        }
        if !properties.visible {
            self = self.css_decl("display", "none");
        }
        self
    }

    /// Apply a `Style` block as inline CSS.
    ///
    /// Only declarations that differ from `Style::default()` are written.
    /// Emitting the defaults would put `color` and `font-size` on every node,
    /// and inline styles beat the host stylesheet, so the app could never
    /// theme anything.
    fn style(self, style: &Style) -> Self {
        let base = Style::default();
        let diff = |changed: bool, value: Option<String>| if changed { value } else { None };

        self.css_opt(
            "width",
            diff(style.width != base.width, css_size(&style.width)),
        )
        .css_opt(
            "height",
            diff(style.height != base.height, css_size(&style.height)),
        )
        .css_opt("min-width", style.min_width.map(px))
        .css_opt("max-width", style.max_width.map(px))
        .css_opt("min-height", style.min_height.map(px))
        .css_opt("max-height", style.max_height.map(px))
        .css_opt(
            "font-size",
            diff(style.font_size != base.font_size, style.font_size.map(px)),
        )
        .css_opt(
            "color",
            diff(
                style.text_colour != base.text_colour,
                style.text_colour.as_ref().map(css_colour),
            ),
        )
        .css_opt(
            "background-color",
            diff(
                style.background_colour != base.background_colour,
                style.background_colour.as_ref().map(css_colour),
            ),
        )
    }

    fn open(&self, out: &mut String) {
        out.push('<');
        out.push_str(self.name);

        let classes: Vec<&str> = self
            .classes
            .iter()
            .map(String::as_str)
            .filter(|c| !c.is_empty())
            .collect();
        if !classes.is_empty() {
            out.push_str(" class=\"");
            out.push_str(&escape_attr(&classes.join(" ")));
            out.push('"');
        }

        for (key, value) in &self.attrs {
            out.push(' ');
            out.push_str(key);
            out.push_str("=\"");
            out.push_str(&escape_attr(value));
            out.push('"');
        }

        for key in &self.flags {
            out.push(' ');
            out.push_str(key);
        }

        if !self.css.is_empty() {
            let decls: Vec<String> = self
                .css
                .iter()
                .map(|(p, v)| format!("{}:{}", p, v))
                .collect();
            out.push_str(" style=\"");
            out.push_str(&escape_attr(&decls.join(";")));
            out.push('"');
        }

        out.push('>');
    }

    fn close(&self, out: &mut String) {
        if !self.void {
            out.push_str("</");
            out.push_str(self.name);
            out.push('>');
        }
    }

    /// Write an element with no children.
    fn empty(self, out: &mut String) {
        self.open(out);
        self.close(out);
    }

    /// Write an element whose content is plain text.
    fn text(self, out: &mut String, content: &str) {
        self.open(out);
        out.push_str(&escape_text(content));
        self.close(out);
    }

    /// Write an element wrapping already-rendered children.
    fn children(self, out: &mut String, children: &[Element]) {
        self.open(out);
        for child in children {
            write_element(out, child);
        }
        self.close(out);
    }

    fn child(self, out: &mut String, child: &Element) {
        self.open(out);
        write_element(out, child);
        self.close(out);
    }
}

fn write_element(out: &mut String, element: &Element) {
    match element {
        // ---- layout ----
        Element::Container(e) => container(out, e),
        Element::Row(e) => row(out, e),
        Element::Column(e) => column(out, e),
        Element::Stack(e) => stack(out, e),
        Element::Grid(e) => grid(out, e),
        Element::Spacer(e) => spacer(out, e),
        Element::Divider(e) => divider(out, e),

        // ---- content ----
        Element::Text(e) => text(out, e),
        Element::Image(e) => image(out, e),
        Element::Avatar(e) => avatar(out, e),
        Element::Badge(e) => badge(out, e),
        Element::Status(e) => status(out, e),
        Element::Timestamp(e) => timestamp(out, e),
        Element::Tooltip(e) => tooltip(out, e),

        // ---- input ----
        Element::Button(e) => button(out, e),
        Element::Input(e) => input(out, e),
        Element::Checkbox(e) => checkbox(out, e),
        Element::Dropdown(e) => dropdown(out, e),

        // ---- scrolling ----
        Element::Scroll(e) => scroll(out, e),
        Element::List(e) => list(out, e),
        Element::MessageList(e) => message_list(out, e),

        // ---- containers ----
        Element::Part(e) => part(out, e),
        Element::Tab(e) => tab(out, e),
        Element::Panel(e) => panel(out, e),
        Element::Popover(e) => popover(out, e),
        Element::Modal(e) => modal(out, e),

        // ---- messenger ----
        Element::MessageBubble(e) => message_bubble(out, e),
        Element::Reaction(e) => reaction(out, e),
        Element::Reply(e) => reply(out, e),
        Element::Attachment(e) => attachment(out, e),
        Element::TypingIndicator(e) => typing_indicator(out, e),
        Element::SearchBar(e) => search_bar(out, e),
    }
}

// ---------------------------------------------------------------- layout

fn container(out: &mut String, e: &Container) {
    Tag::new("div", "nxt-container")
        .size(&e.width, &e.height)
        .padding(&e.padding)
        .css_opt(
            "background-color",
            e.background_colour.as_ref().map(css_colour),
        )
        .css_decl("align-items", css_align(&e.alignment))
        .css_opt("overflow", e.clip.then(|| "hidden".to_string()))
        .children(out, &e.children);
}

fn row(out: &mut String, e: &Row) {
    Tag::new("div", "nxt-row")
        .css_decl("display", "flex")
        .css_decl("flex-direction", "row")
        .css_decl("gap", px(e.spacing))
        .css_decl("align-items", css_align(&e.alignment))
        .css_decl("justify-content", css_align(&e.justify))
        .css_opt("flex-wrap", e.wrap.then(|| "wrap".to_string()))
        .padding(&e.padding)
        .size(&e.width, &e.height)
        .children(out, &e.children);
}

fn column(out: &mut String, e: &Column) {
    Tag::new("div", "nxt-column")
        .css_decl("display", "flex")
        .css_decl("flex-direction", "column")
        .css_decl("gap", px(e.spacing))
        .css_decl("align-items", css_align(&e.alignment))
        .css_decl("justify-content", css_align(&e.justify))
        .padding(&e.padding)
        .size(&e.width, &e.height)
        .children(out, &e.children);
}

fn stack(out: &mut String, e: &Stack) {
    let direction = match e.direction {
        Direction::Vertical => "column",
        Direction::Horizontal => "row",
    };
    Tag::new("div", "nxt-stack")
        .css_decl("display", "flex")
        .css_decl("flex-direction", direction)
        .css_decl("gap", px(e.spacing))
        .css_decl("align-items", css_align(&e.alignment))
        .css_decl("justify-content", css_align(&e.justify))
        .padding(&e.padding)
        .size(&e.width, &e.height)
        .children(out, &e.children);
}

fn grid(out: &mut String, e: &Grid) {
    Tag::new("div", "nxt-grid")
        .css_decl("display", "grid")
        .css_decl(
            "grid-template-columns",
            format!("repeat({},1fr)", e.columns.max(1)),
        )
        .css_decl("column-gap", px(e.column_gap))
        .css_decl("row-gap", px(e.row_gap))
        .css_decl("align-items", css_align(&e.alignment))
        .padding(&e.padding)
        .size(&e.width, &e.height)
        .children(out, &e.children);
}

fn spacer(out: &mut String, e: &Spacer) {
    Tag::new("div", "nxt-spacer")
        .size(&e.width, &e.height)
        .css_opt("flex-grow", e.grow.map(|g| format_number(g as f64)))
        .empty(out);
}

fn divider(out: &mut String, e: &Divider) {
    let tag = Tag::new("hr", "nxt-divider").void();
    let tag = match e.orientation {
        Orientation::Horizontal => tag
            .css_decl("border-width", format!("{} 0 0 0", px(e.thickness)))
            .css_opt("width", css_size(&e.length)),
        Orientation::Vertical => tag
            .css_decl("border-width", format!("0 0 0 {}", px(e.thickness)))
            .css_opt("height", css_size(&e.length)),
    };
    tag.css_decl("border-style", "solid")
        .css_decl("border-color", css_colour(&e.colour))
        .empty(out);
}

// --------------------------------------------------------------- content

fn text(out: &mut String, e: &Text) {
    Tag::new("span", "nxt-text")
        .props(&e.properties)
        .style(&e.style)
        .text(out, &e.content);
}

/// Schemes a `.nxt` program may put in a `src`.
///
/// `javascript:` and `vbscript:` are script URLs, and a non-image `data:` URL
/// can carry markup, so they are dropped rather than passed to the host. A
/// relative path or an `https:` URL is allowed: note that an absolute URL is
/// still an outbound request made by the host on the program's behalf, so the
/// host should apply a CSP as well.
fn safe_src(src: &str) -> &str {
    let lowered = src.trim_start().to_ascii_lowercase();
    let blocked = lowered.starts_with("javascript:")
        || lowered.starts_with("vbscript:")
        || (lowered.starts_with("data:") && !lowered.starts_with("data:image/"));
    if blocked {
        ""
    } else {
        src
    }
}

fn image(out: &mut String, e: &Image) {
    Tag::new("img", "nxt-image")
        .void()
        .props(&e.properties)
        .attr("src", safe_src(&e.src).to_string())
        .attr("alt", e.alt.clone())
        .style(&e.style)
        .empty(out);
}

fn avatar(out: &mut String, e: &Avatar) {
    if e.src.is_empty() {
        // No image: show the fallback initials instead of a broken <img>.
        Tag::new("div", "nxt-avatar")
            .class("nxt-avatar-fallback")
            .props(&e.properties)
            .css_decl("width", px(e.size))
            .css_decl("height", px(e.size))
            .style(&e.style)
            .text(out, e.fallback.as_deref().unwrap_or(""));
    } else {
        Tag::new("img", "nxt-avatar")
            .void()
            .props(&e.properties)
            .attr("src", safe_src(&e.src).to_string())
            .attr("alt", e.fallback.clone().unwrap_or_default())
            .css_decl("width", px(e.size))
            .css_decl("height", px(e.size))
            .style(&e.style)
            .empty(out);
    }
}

fn badge(out: &mut String, e: &Badge) {
    let variant = match e.variant {
        BadgeVariant::Primary => "primary",
        BadgeVariant::Success => "success",
        BadgeVariant::Warning => "warning",
        BadgeVariant::Error => "error",
        BadgeVariant::Info => "info",
    };
    Tag::new("span", "nxt-badge")
        .class(format!("nxt-badge-{}", variant))
        .props(&e.properties)
        .style(&e.style)
        .text(out, &e.content);
}

fn status(out: &mut String, e: &Status) {
    let (name, colour) = match &e.status {
        StatusType::Online => ("online", None),
        StatusType::Offline => ("offline", None),
        StatusType::Away => ("away", None),
        StatusType::Busy => ("busy", None),
        StatusType::DoNotDisturb => ("dnd", None),
        StatusType::Custom { color } => ("custom", Some(color)),
    };
    let tag = Tag::new("span", "nxt-status")
        .class(format!("nxt-status-{}", name))
        .props(&e.properties)
        .attr("data-nxt-status", name)
        .css_decl("width", px(e.size))
        .css_decl("height", px(e.size))
        .css_opt("background-color", colour.map(css_colour))
        .style(&e.style);

    if e.show_text {
        tag.text(out, name);
    } else {
        tag.empty(out);
    }
}

/// Explicit names for the app contract. Never `{:?}`: a `derive(Debug)`
/// variant rename would silently change an attribute the host parses.
fn timestamp_format(format: &TimestampFormat) -> &'static str {
    match format {
        TimestampFormat::ShortTime => "short-time",
        TimestampFormat::LongTime => "long-time",
        TimestampFormat::ShortDate => "short-date",
        TimestampFormat::LongDate => "long-date",
        TimestampFormat::DateTime => "date-time",
        TimestampFormat::FullDateTime => "full-date-time",
        TimestampFormat::Relative => "relative",
    }
}

fn tab_bar_position(position: &TabBarPosition) -> &'static str {
    match position {
        TabBarPosition::Top => "top",
        TabBarPosition::Bottom => "bottom",
        TabBarPosition::Left => "left",
        TabBarPosition::Right => "right",
        TabBarPosition::Cursor => "cursor",
    }
}

fn timestamp(out: &mut String, e: &Timestamp) {
    // The runtime has no clock, so the raw value is emitted and the host
    // formats it. `data-nxt-format` tells the host which form was asked for.
    Tag::new("span", "nxt-timestamp")
        .props(&e.properties)
        .attr("data-nxt-time", e.time.to_string())
        .attr("data-nxt-format", timestamp_format(&e.format))
        .data_bool("data-nxt-relative", e.relative)
        .style(&e.style)
        .text(out, &e.time.to_string());
}

fn tooltip(out: &mut String, e: &Tooltip) {
    let position = match e.position {
        TooltipPosition::Top => "top",
        TooltipPosition::Bottom => "bottom",
        TooltipPosition::Left => "left",
        TooltipPosition::Right => "right",
        TooltipPosition::Auto => "auto",
    };
    Tag::new("span", "nxt-tooltip")
        .props(&e.properties)
        .attr_if_set("data-nxt-target-id", &e.target_id)
        .attr("data-nxt-position", position)
        .attr("data-nxt-delay-ms", e.delay_ms.to_string())
        .style(&e.style)
        .text(out, &e.content);
}

// ----------------------------------------------------------------- input

fn button(out: &mut String, e: &Button) {
    let variant = match e.variant {
        ButtonVariant::Primary => "primary",
        ButtonVariant::Secondary => "secondary",
        ButtonVariant::Success => "success",
        ButtonVariant::Danger => "danger",
        ButtonVariant::Ghost => "ghost",
        ButtonVariant::Link => "link",
    };
    Tag::new("button", "nxt-button")
        .class(format!("nxt-button-{}", variant))
        .attr("type", "button")
        .props(&e.properties)
        .flag("disabled", !e.properties.enabled)
        .style(&e.style)
        .text(out, &e.text);
}

fn input(out: &mut String, e: &Input) {
    let input_type = match e.input_type {
        InputType::Text => "text",
        InputType::Password => "password",
        InputType::Email => "email",
        InputType::Number => "number",
        InputType::Search => "search",
        InputType::Url => "url",
    };
    Tag::new("input", "nxt-input")
        .void()
        .attr("type", input_type)
        .props(&e.properties)
        .attr("value", e.value.clone())
        .attr("placeholder", e.placeholder.clone())
        .flag("readonly", e.readonly)
        .flag("required", e.required)
        .flag("disabled", !e.properties.enabled)
        .style(&e.style)
        .empty_with_maxlength(out, e.max_length);
}

fn checkbox(out: &mut String, e: &Checkbox) {
    // <label> ties the caption to the box natively, so clicking the text
    // toggles it without any script.
    let outer = Tag::new("label", "nxt-checkbox")
        .props(&e.properties)
        .style(&e.style);
    outer.open(out);

    Tag::new("input", "nxt-checkbox-box")
        .void()
        .attr("type", "checkbox")
        .flag("checked", e.checked)
        .flag("disabled", !e.properties.enabled)
        .data_bool("data-nxt-indeterminate", e.indeterminate)
        .empty(out);

    Tag::new("span", "nxt-checkbox-label").text(out, &e.label);
    outer.close(out);
}

fn dropdown(out: &mut String, e: &Dropdown) {
    let outer = Tag::new("select", "nxt-dropdown")
        .props(&e.properties)
        .flag("multiple", e.multi_select)
        .flag("disabled", !e.properties.enabled)
        .data_bool("data-nxt-searchable", e.searchable)
        .style(&e.style);
    outer.open(out);

    if !e.placeholder.is_empty() {
        Tag::new("option", "nxt-dropdown-placeholder")
            .attr("value", "")
            .flag("disabled", true)
            .flag("selected", e.selected_index.is_none())
            .text(out, &e.placeholder);
    }

    for (index, option) in e.options.iter().enumerate() {
        Tag::new("option", "nxt-dropdown-option")
            .attr("value", option.value.clone())
            .flag("selected", e.selected_index == Some(index))
            .flag("disabled", option.disabled)
            .text(out, &option.label);
    }

    outer.close(out);
}

// ------------------------------------------------------------- scrolling

fn scroll(out: &mut String, e: &Scroll) {
    let (overflow_x, overflow_y) = match e.direction {
        ScrollDirection::Vertical => ("hidden", "auto"),
        ScrollDirection::Horizontal => ("auto", "hidden"),
        ScrollDirection::Both => ("auto", "auto"),
    };
    Tag::new("div", "nxt-scroll")
        .css_decl("overflow-x", overflow_x)
        .css_decl("overflow-y", overflow_y)
        .css_opt(
            "scroll-behavior",
            e.smooth_scroll.then(|| "smooth".to_string()),
        )
        .data_bool("data-nxt-snap-to-bottom", e.snap_to_bottom)
        .padding(&e.padding)
        .size(&e.width, &e.height)
        .child(out, &e.child);
}

fn list(out: &mut String, e: &List) {
    let direction = match e.orientation {
        ListOrientation::Vertical => "column",
        ListOrientation::Horizontal => "row",
    };
    Tag::new("div", "nxt-list")
        .css_decl("display", "flex")
        .css_decl("flex-direction", direction)
        .css_decl("gap", px(e.spacing))
        .data_bool("data-nxt-snap-to-bottom", e.snap_to_bottom)
        .padding(&e.padding)
        .size(&e.width, &e.height)
        .children(out, &e.items);
}

fn message_list(out: &mut String, e: &MessageList) {
    let outer = Tag::new("div", "nxt-message-list")
        .css_decl("display", "flex")
        .css_decl(
            "flex-direction",
            if e.reverse_order {
                "column-reverse"
            } else {
                "column"
            },
        )
        .css_decl("gap", px(e.spacing))
        .data_bool("data-nxt-auto-scroll", e.auto_scroll)
        .data_bool("data-nxt-show-timestamps", e.show_timestamps)
        .padding(&e.padding)
        .size(&e.width, &e.height);
    outer.open(out);

    for item in &e.messages {
        // `MessageItem.id` is synthesised by the runtime (currently the loop
        // index), so it must not land in `data-nxt-id` -- that namespace
        // belongs to ids the program wrote and the app queries.
        let wrapper = Tag::new("div", "nxt-message-item")
            .attr("data-nxt-index", item.id.clone())
            .attr("data-nxt-timestamp", item.timestamp.to_string())
            .css_decl("align-self", css_align(&item.alignment))
            .css_opt("height", item.height.map(px));
        wrapper.child(out, &item.element);
    }

    outer.close(out);
}

// ------------------------------------------------------------ containers

fn part(out: &mut String, e: &Part) {
    Tag::new("div", "nxt-part")
        .data_bool("data-nxt-resizable", e.resizable)
        .attr("data-nxt-direction", e.direction.clone())
        .size(&e.width, &e.height)
        .padding(&e.padding)
        .css_opt(
            "background-color",
            e.background_colour.as_ref().map(css_colour),
        )
        .css_opt("min-width", e.min_width.map(px))
        .css_opt("max-width", e.max_width.map(px))
        .css_opt("min-height", e.min_height.map(px))
        .css_opt("max-height", e.max_height.map(px))
        .child(out, &e.child);
}

fn tab(out: &mut String, e: &Tab) {
    let outer = Tag::new("div", "nxt-tab")
        .attr("data-nxt-active-index", e.active_index.to_string())
        .attr(
            "data-nxt-bar-position",
            tab_bar_position(&e.tab_bar_position),
        )
        .size(&e.width, &e.height)
        .padding(&e.padding);
    outer.open(out);

    let bar = Tag::new("div", "nxt-tab-bar")
        .css_decl("display", "flex")
        .css_decl("gap", px(e.spacing))
        .css_decl("height", px(e.tab_bar_height));
    bar.open(out);
    for (index, item) in e.tabs.iter().enumerate() {
        Tag::new("button", "nxt-tab-button")
            .attr("type", "button")
            .attr("data-nxt-tab-index", index.to_string())
            .data_bool("data-nxt-active", index == e.active_index)
            .flag("disabled", item.disabled)
            .text(out, &item.label);
    }
    bar.close(out);

    for (index, item) in e.tabs.iter().enumerate() {
        let panel = Tag::new("div", "nxt-tab-panel")
            .attr("data-nxt-tab-index", index.to_string())
            .css_opt(
                "display",
                (index != e.active_index).then(|| "none".to_string()),
            );
        panel.child(out, &item.content);
    }

    outer.close(out);
}

fn panel(out: &mut String, e: &Panel) {
    let outer = Tag::new("div", "nxt-panel")
        .data_bool("data-nxt-expanded", e.expanded)
        .size(&e.width, &e.height)
        .padding(&e.padding);
    outer.open(out);

    Tag::new("div", "nxt-panel-header")
        .css_decl("height", px(e.header_height))
        .css_opt(
            "background-color",
            e.header_background.as_ref().map(css_colour),
        )
        .child(out, &e.header);

    Tag::new("div", "nxt-panel-content")
        .css_opt(
            "background-color",
            e.content_background.as_ref().map(css_colour),
        )
        .css_opt("display", (!e.expanded).then(|| "none".to_string()))
        .child(out, &e.content);

    outer.close(out);
}

fn popover(out: &mut String, e: &Popover) {
    let position = match e.position {
        PopoverPosition::Top => "top",
        PopoverPosition::Bottom => "bottom",
        PopoverPosition::Left => "left",
        PopoverPosition::Right => "right",
        PopoverPosition::TopLeft => "top-left",
        PopoverPosition::TopRight => "top-right",
        PopoverPosition::BottomLeft => "bottom-left",
        PopoverPosition::BottomRight => "bottom-right",
        PopoverPosition::Cursor => "cursor",
    };
    Tag::new("div", "nxt-popover")
        .attr("data-nxt-position", position)
        .data_bool("data-nxt-arrow", e.arrow)
        .data_bool("data-nxt-dismiss-outside", e.dismiss_on_click_outside)
        .data_bool("data-nxt-dismiss-escape", e.dismiss_on_escape)
        .css_decl("background-color", css_colour(&e.background_colour))
        .css_opt(
            "box-shadow",
            e.shadow.then(|| "0 2px 8px #0003".to_string()),
        )
        .size(&e.width, &e.height)
        .padding(&e.padding)
        .child(out, &e.content);
}

fn modal(out: &mut String, e: &Modal) {
    let overlay = Tag::new("div", "nxt-modal-overlay")
        .css_decl("background-color", css_colour(&e.overlay_colour))
        .css_opt(
            "backdrop-filter",
            e.blur_background.then(|| "blur(4px)".to_string()),
        )
        .data_bool("data-nxt-dismiss-outside", e.dismiss_on_click_outside)
        .data_bool("data-nxt-dismiss-escape", e.dismiss_on_escape);
    overlay.open(out);

    Tag::new("div", "nxt-modal")
        .data_bool("data-nxt-closable", e.closable)
        .data_bool("data-nxt-centered", e.centered)
        .css_decl("background-color", css_colour(&e.background_colour))
        .size(&e.width, &e.height)
        .padding(&e.padding)
        .child(out, &e.content);

    overlay.close(out);
}

// ------------------------------------------------------------- messenger

fn message_bubble(out: &mut String, e: &MessageBubble) {
    // `.nxt-bubble` is the right-click target the app listens on.
    let kind = match e.message_type {
        MessageBubbleType::Sent => "sent",
        MessageBubbleType::Received => "received",
        MessageBubbleType::System => "system",
    };
    Tag::new("div", "nxt-bubble")
        .class(format!("nxt-bubble-{}", kind))
        .attr("data-nxt-bubble-type", kind)
        .css_decl("background-color", css_colour(&e.background_colour))
        .css_decl("color", css_colour(&e.text_colour))
        .css_opt("max-width", e.max_width.map(px))
        .css_opt("width", css_size(&e.width))
        .padding(&e.padding)
        .data_bool("data-nxt-tail", e.show_tail)
        .css_opt(
            "box-shadow",
            e.shadow.then(|| "0 1px 2px #0002".to_string()),
        )
        .child(out, &e.content);
}

fn reaction(out: &mut String, e: &Reaction) {
    let outer = Tag::new("div", "nxt-reactions")
        .css_decl("display", "flex")
        .css_decl("gap", px(e.spacing))
        .padding(&e.padding);
    outer.open(out);

    for item in e.reactions.iter().take(e.max_display) {
        let chip = Tag::new("span", "nxt-reaction")
            .data_bool("data-nxt-reacted", item.user_reacted)
            .attr("data-nxt-emoji", item.emoji.clone())
            .attr("data-nxt-count", item.count.to_string())
            .css_decl("font-size", px(e.size));
        chip.open(out);
        out.push_str(&escape_text(&item.emoji));
        Tag::new("span", "nxt-reaction-count").text(out, &item.count.to_string());
        chip.close(out);
    }

    outer.close(out);
}

fn reply(out: &mut String, e: &Reply) {
    let outer = Tag::new("div", "nxt-reply").data_bool("data-nxt-icon", e.show_icon);
    outer.open(out);

    Tag::new("span", "nxt-reply-sender").text(out, &e.original_sender);

    let preview: String = if e.max_preview_length > 0 {
        e.original_text.chars().take(e.max_preview_length).collect()
    } else {
        e.original_text.clone()
    };
    Tag::new("span", "nxt-reply-text").text(out, &preview);

    outer.close(out);
}

fn attachment(out: &mut String, e: &Attachment) {
    let kind = match e.file_type {
        AttachmentType::Image => "image",
        AttachmentType::Video => "video",
        AttachmentType::Audio => "audio",
        AttachmentType::Document => "document",
        AttachmentType::Geo => "geo",
        AttachmentType::Other => "other",
    };
    let outer = Tag::new("div", "nxt-attachment")
        .class(format!("nxt-attachment-{}", kind))
        .attr("data-nxt-file-type", kind)
        .attr("data-nxt-file-size", e.file_size.to_string())
        .data_bool("data-nxt-preview", e.show_preview)
        .data_bool("data-nxt-download", e.show_download);
    outer.open(out);
    Tag::new("span", "nxt-attachment-name").text(out, &e.file_name);
    outer.close(out);
}

fn typing_indicator(out: &mut String, e: &TypingIndicator) {
    Tag::new("div", "nxt-typing")
        .data_bool("data-nxt-avatars", e.show_avatars)
        .attr("data-nxt-max-display", e.max_display.to_string())
        .text(out, &e.text);
}

fn search_bar(out: &mut String, e: &SearchBar) {
    let outer = Tag::new("div", "nxt-searchbar")
        .data_bool("data-nxt-scope-selector", e.show_scope_selector)
        .data_bool("data-nxt-filters", e.show_filters)
        .data_bool("data-nxt-results-count", e.show_results_count);
    outer.open(out);

    // Deliberately not `.nxt-input`: the send action targets the nearest
    // `.nxt-input`, and a search box must never become the message source.
    Tag::new("input", "nxt-searchbar-input")
        .void()
        .attr("type", "search")
        .attr("value", e.value.clone())
        .attr("placeholder", e.placeholder.clone())
        .empty(out);

    outer.close(out);
}

impl Tag {
    /// `<input>` needs `maxlength` only when the program set one.
    fn empty_with_maxlength(self, out: &mut String, max_length: Option<usize>) {
        match max_length {
            Some(max) => self.attr("maxlength", max.to_string()).empty(out),
            None => self.empty(out),
        }
    }
}
