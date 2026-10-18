use crate::style::{Borders, Colour, Margins, Padding, Size};
use crate::Element;

#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    pub child: Box<Element>,
    pub width: Size,
    pub height: Size,
    pub padding: Padding,
    pub margins: Margins,
    pub background_colour: Option<Colour>,
    pub borders: Borders,
    pub resizable: bool,
    pub direction: String, // "horizontal" | "vertical" — axis along which this Part is resizable
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TabItem {
    pub label: String,
    pub icon: Option<Element>,
    pub content: Element,
    pub closable: bool,
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tab {
    pub tabs: Vec<TabItem>,
    pub active_index: usize,
    pub width: Size,
    pub height: Size,
    pub padding: Padding,
    pub tab_bar_height: f32,
    pub tab_bar_position: TabBarPosition,
    pub tab_bar_background: Option<Colour>,
    pub active_tab_colour: Option<Colour>,
    pub inactive_tab_colour: Option<Colour>,
    pub spacing: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TabBarPosition {
    Top,
    Bottom,
    Left,
    Right,
    Cursor,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Panel {
    pub header: Box<Element>,
    pub content: Box<Element>,
    pub expanded: bool,
    pub width: Size,
    pub height: Size,
    pub padding: Padding,
    pub margins: Margins,
    pub header_height: f32,
    pub header_background: Option<Colour>,
    pub content_background: Option<Colour>,
    pub borders: Borders,
    pub icon_expanded: Option<String>,
    pub icon_collapsed: Option<String>,
    pub animate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PopoverPosition {
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Cursor,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Popover {
    pub content: Box<Element>,
    pub anchor: Option<(f32, f32)>,
    pub position: PopoverPosition,
    pub offset: (f32, f32),
    pub width: Size,
    pub height: Size,
    pub padding: Padding,
    pub background_colour: Colour,
    pub borders: Borders,
    pub shadow: bool,
    pub arrow: bool,
    pub dismiss_on_click_outside: bool,
    pub dismiss_on_escape: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Modal {
    pub content: Box<Element>,
    pub width: Size,
    pub height: Size,
    pub padding: Padding,
    pub background_colour: Colour,
    pub borders: Borders,
    pub overlay_colour: Colour,
    pub blur_background: bool,
    pub centered: bool,
    pub position: Option<(f32, f32)>,
    pub closable: bool,
    pub dismiss_on_click_outside: bool,
    pub dismiss_on_escape: bool,
    pub animate: bool,
}

impl Default for TabBarPosition {
    fn default() -> Self {
        Self::Top
    }
}

impl Default for PopoverPosition {
    fn default() -> Self {
        Self::Bottom
    }
}
