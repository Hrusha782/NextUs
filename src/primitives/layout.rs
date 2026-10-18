use crate::style::{Alignment, Borders, Colour, Margins, Padding, Size};
use crate::Element;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Container {
    pub children: Vec<Element>,
    pub width: Size,
    pub height: Size,
    pub padding: Padding,
    pub margins: Margins,
    pub background_colour: Option<Colour>,
    pub borders: Borders,
    pub alignment: Alignment,
    pub clip: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub children: Vec<Element>,
    pub spacing: f32,
    pub alignment: Alignment,
    pub justify: Alignment,
    pub wrap: bool,
    pub padding: Padding,
    pub width: Size,
    pub height: Size,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    pub children: Vec<Element>,
    pub spacing: f32,
    pub alignment: Alignment,
    pub justify: Alignment,
    pub padding: Padding,
    pub width: Size,
    pub height: Size,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stack {
    pub children: Vec<Element>,
    pub direction: Direction,
    pub spacing: f32,
    pub alignment: Alignment,
    pub justify: Alignment,
    pub padding: Padding,
    pub width: Size,
    pub height: Size,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Grid {
    pub children: Vec<Element>,
    pub columns: usize,
    pub rows: Option<usize>,
    pub column_gap: f32,
    pub row_gap: f32,
    pub column_widths: Vec<Size>,
    pub row_heights: Vec<Size>,
    pub alignment: Alignment,
    pub padding: Padding,
    pub width: Size,
    pub height: Size,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spacer {
    pub width: Size,
    pub height: Size,
    pub grow: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Divider {
    pub orientation: Orientation,
    pub thickness: f32,
    pub colour: Colour,
    pub length: Size,
    pub margins: Margins,
}
