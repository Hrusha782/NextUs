//! Renderer module for NextUs
//!
//! Provides layout computation and memory tracking for UI elements.

pub mod html;
mod layout_engine;
mod memory_tracker;

pub use html::render_html;
pub use layout_engine::{ComputedLayout, LayoutEngine, LayoutNode, LayoutResult};
pub use memory_tracker::{AllocationInfo, MemoryStats, MemoryTracker};

use crate::style::{Margins, Padding, Size};

/// Represents a computed position and size for an element
#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    pub fn contains(&self, point_x: f32, point_y: f32) -> bool {
        point_x >= self.x
            && point_x <= self.x + self.width
            && point_y >= self.y
            && point_y <= self.y + self.height
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}

/// Constraint for layout computation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Constraints {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

impl Constraints {
    pub fn new(min_width: f32, max_width: f32, min_height: f32, max_height: f32) -> Self {
        Self {
            min_width,
            max_width,
            min_height,
            max_height,
        }
    }

    pub fn unbounded() -> Self {
        Self {
            min_width: 0.0,
            max_width: f32::INFINITY,
            min_height: 0.0,
            max_height: f32::INFINITY,
        }
    }

    pub fn tight(width: f32, height: f32) -> Self {
        Self {
            min_width: width,
            max_width: width,
            min_height: height,
            max_height: height,
        }
    }

    pub fn constrain(&self, width: f32, height: f32) -> (f32, f32) {
        let w = width.max(self.min_width).min(self.max_width);
        let h = height.max(self.min_height).min(self.max_height);
        (w, h)
    }
}

impl Default for Constraints {
    fn default() -> Self {
        Self::unbounded()
    }
}

/// Resolve a Size enum to a concrete pixel value
pub fn resolve_size(size: &Size, parent_size: f32, _constraints: &Constraints) -> f32 {
    match size {
        Size::Fixed(px) => *px,
        Size::Percentage(pct) => parent_size * pct / 100.0,
        Size::Auto => parent_size, // Default to parent size for auto
    }
}

/// Calculate total horizontal space from padding and margins
pub fn horizontal_space(padding: &Padding, margins: &Margins) -> f32 {
    padding.left + padding.right + margins.left + margins.right
}

/// Calculate total vertical space from padding and margins
pub fn vertical_space(padding: &Padding, margins: &Margins) -> f32 {
    padding.top + padding.bottom + margins.top + margins.bottom
}
