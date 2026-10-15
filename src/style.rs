#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Colour {
    pub r: u8,
    pub g: u8, 
    pub b: u8,
    pub a: u8,
}

impl Colour {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const WHITE: Colour = Colour { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Colour = Colour { r: 0, g: 0, b: 0, a: 255 };
    pub const TRANSPARENT: Colour = Colour { r: 0, g: 0, b: 0, a: 0 };
    pub const RED: Colour = Colour { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Colour = Colour { r: 0, g: 255, b: 0, a: 255 };
    pub const BLUE: Colour = Colour { r: 0, g: 0, b: 255, a: 255 };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Padding {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl Padding {
    pub fn new(top: f32, bottom: f32, left: f32, right: f32) -> Self {
        Self { top, bottom, left, right }
    }

    pub fn all(num: f32) -> Self {
        Self { top: num, bottom: num, left: num, right: num }
    }

    pub fn ver(num: f32) -> Self {
        Self { top: num, bottom: num, left: 0.0, right: 0.0 }
    }

    pub fn hor(num: f32) -> Self {
        Self { top: 0.0, bottom: 0.0, left: num, right: num }
    }

    pub fn zero() -> Self {
        Self { top: 0.0, bottom: 0.0, left: 0.0, right: 0.0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Margins {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl Margins {
    pub fn new(top: f32, bottom: f32, left: f32, right: f32) -> Self {
        Self { top, bottom, left, right }
    }

    pub fn all(num: f32) -> Self {
        Self { top: num, bottom: num, left: num, right: num }
    }

    pub fn ver(num: f32) -> Self {
        Self { top: num, bottom: num, left: 0.0, right: 0.0 }
    }

    pub fn hor(num: f32) -> Self {
        Self { top: 0.0, bottom: 0.0, left: num, right: num }
    }

    pub fn zero() -> Self {
        Self { top: 0.0, bottom: 0.0, left: 0.0, right: 0.0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spacing {
    pub margins: Margins,
    pub padding: Padding,
}

impl Spacing {
    pub fn new(margins: Margins, padding: Padding) -> Self {
        Self { margins, padding }
    }

    pub fn all(num: f32) -> Self {
        Self {
            margins: Margins::all(num),
            padding: Padding::all(num),
        }
    }

    pub fn zero() -> Self {
        Self {
            margins: Margins::zero(),
            padding: Padding::zero(),
        }
    }

}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Borders {
    pub width: f32,
    pub colour: Colour,
    pub radius: f32,
}

impl Borders {
    pub fn new(width: f32, colour: Colour, radius: f32) -> Self {
        Self { width, colour, radius }
    }

    pub fn none() -> Self {
        Self { width: 0.0, colour: Colour::TRANSPARENT, radius: 0.0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position {
    Absolute {x: f32, y: f32},
    Relative {x: f32, y: f32},
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    Fixed(f32),
    Percentage(f32),
    Auto,
}

impl Size {
    pub fn validate(&self) -> Result<(), String> {
        if let Size::Percentage(p) = self {
            if *p < 0.0 || *p > 100.0 {
                return Err(format!("Percentage must be 0-100, got {}", p));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Alignment {
    Start,
    Center, 
    End,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,

    pub position: Option<Position>,
    pub width: Size,
    pub height: Size,
    pub spacing: Spacing,
    pub text_colour: Option<Colour>,
    pub background_colour: Option<Colour>,
    pub borders: Borders,
    pub font_size: Option<f32>,
    pub alignment: Alignment,
    pub text_alignment: Option<TextAlignment>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            position: None,
            width: Size::Auto,
            height: Size::Auto,
            spacing: Spacing::zero(),
            text_colour: Some(Colour::BLACK),
            background_colour: None,
            borders: Borders::none(),
            font_size: Some(16.0),
            alignment: Alignment::Start,
            text_alignment: Some(TextAlignment::Left),
        }
    }
}