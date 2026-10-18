use crate::properties::Properties;
use crate::style::{Alignment, Colour, Padding, Size};
use crate::Element;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollDirection {
    Vertical,
    Horizontal,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollBarVisibility {
    Always,
    Hover,
    Auto,
    Never,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScrollBar {
    pub width: f32,
    pub colour: Colour,
    pub background_colour: Colour,
    pub border_radius: f32,
    pub visibility: ScrollBarVisibility,
    pub visible_range: Option<(usize, usize)>,
    pub scroll_position: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scroll {
    pub properties: Properties,
    pub child: Box<Element>,
    pub direction: ScrollDirection,
    pub width: Size,
    pub height: Size,
    pub padding: Padding,
    pub scrollbar: ScrollBar,
    pub snap_to_bottom: bool,
    pub smooth_scroll: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ListOrientation {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct List {
    pub properties: Properties,
    pub items: Vec<Element>,
    pub orientation: ListOrientation,
    pub item_height: f32,
    pub spacing: f32,
    pub width: Size,
    pub height: Size,
    pub padding: Padding,
    pub scrollbar: ScrollBar,
    pub snap_to_bottom: bool,
}

impl Default for ScrollBar {
    fn default() -> Self {
        Self {
            width: 8.0,
            colour: Colour {
                r: 128,
                g: 128,
                b: 128,
                a: 200,
            },
            background_colour: Colour {
                r: 240,
                g: 240,
                b: 240,
                a: 100,
            },
            border_radius: 4.0,
            visibility: ScrollBarVisibility::Auto,
            visible_range: None,
            scroll_position: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageItem {
    pub element: Element,
    pub alignment: Alignment,
    pub timestamp: i64,
    pub id: String,
    pub height: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageList {
    pub properties: Properties,
    pub messages: Vec<MessageItem>,
    pub width: Size,
    pub height: Size,
    pub padding: Padding,
    pub spacing: f32,
    pub group_spacing: f32,
    pub group_threshold: i64,
    pub scrollbar: ScrollBar,
    pub auto_scroll: bool,
    pub scroll_threshold: f32,
    pub show_timestamps: bool,
    pub reverse_order: bool,
}
