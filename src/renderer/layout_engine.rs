//! Layout Engine for NextUs
//! 
//! Computes layout positions and sizes for UI elements.

use std::collections::HashMap;
use crate::Element;
use crate::style::{Size, Padding, Alignment};
use crate::primitives::scrolling::ScrollDirection;
use super::{Rect, Constraints, resolve_size};

/// Represents a node in the layout tree
#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub element_id: String,
    pub rect: Rect,
    pub children: Vec<LayoutNode>,
    pub scroll_offset: (f32, f32),
}

impl LayoutNode {
    pub fn new(element_id: String) -> Self {
        Self {
            element_id,
            rect: Rect::zero(),
            children: Vec::new(),
            scroll_offset: (0.0, 0.0),
        }
    }

    pub fn with_rect(mut self, rect: Rect) -> Self {
        self.rect = rect;
        self
    }

    pub fn add_child(&mut self, child: LayoutNode) {
        self.children.push(child);
    }
}

/// Result of a layout computation
#[derive(Debug, Clone)]
pub struct LayoutResult {
    pub root: LayoutNode,
    pub total_width: f32,
    pub total_height: f32,
}

/// Computed layout information for an element
#[derive(Debug, Clone)]
pub struct ComputedLayout {
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub content_size: (f32, f32),
    pub padding: Padding,
}

impl ComputedLayout {
    pub fn new() -> Self {
        Self {
            position: (0.0, 0.0),
            size: (0.0, 0.0),
            content_size: (0.0, 0.0),
            padding: Padding::zero(),
        }
    }

    pub fn content_rect(&self) -> Rect {
        Rect::new(
            self.position.0 + self.padding.left,
            self.position.1 + self.padding.top,
            self.content_size.0,
            self.content_size.1,
        )
    }
}

impl Default for ComputedLayout {
    fn default() -> Self {
        Self::new()
    }
}

/// The layout engine that computes positions and sizes
pub struct LayoutEngine {
    viewport_width: f32,
    viewport_height: f32,
    computed_layouts: HashMap<String, ComputedLayout>,
    element_counter: usize,
}

impl LayoutEngine {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            viewport_width,
            viewport_height,
            computed_layouts: HashMap::new(),
            element_counter: 0,
        }
    }

    /// Compute layout for an element tree
    pub fn compute(&mut self, element: &Element) -> LayoutResult {
        self.element_counter = 0;
        self.computed_layouts.clear();

        let constraints = Constraints::tight(self.viewport_width, self.viewport_height);
        let root = self.layout_element(element, &constraints, 0.0, 0.0);

        LayoutResult {
            total_width: root.rect.width,
            total_height: root.rect.height,
            root,
        }
    }

    fn generate_id(&mut self) -> String {
        self.element_counter += 1;
        format!("element_{}", self.element_counter)
    }

    fn layout_element(&mut self, element: &Element, constraints: &Constraints, x: f32, y: f32) -> LayoutNode {
        let id = self.generate_id();
        let mut node = LayoutNode::new(id.clone());

        match element {
            Element::Text(text) => {
                // Simple text measurement
                let char_width = 8.0;
                let line_height = 16.0;
                let text_width = text.content.len() as f32 * char_width;
                let text_height = line_height;
                
                let (width, height) = constraints.constrain(text_width, text_height);
                node.rect = Rect::new(x, y, width, height);
                self.store_layout(&id, x, y, width, height, &Padding::zero());
            }

            Element::Row(row) => {
                let (width, height) = self.layout_flex_row(
                    &mut node,
                    &row.children,
                    &row.width,
                    &row.height,
                    &row.padding,
                    row.spacing,
                    row.alignment,
                    constraints,
                    x, y,
                );
                node.rect = Rect::new(x, y, width, height);
                self.store_layout(&id, x, y, width, height, &row.padding);
            }

            Element::Column(col) => {
                let (width, height) = self.layout_flex_column(
                    &mut node,
                    &col.children,
                    &col.width,
                    &col.height,
                    &col.padding,
                    col.spacing,
                    col.alignment,
                    constraints,
                    x, y,
                );
                node.rect = Rect::new(x, y, width, height);
                self.store_layout(&id, x, y, width, height, &col.padding);
            }

            Element::Stack(stack) => {
                let padding = &stack.padding;
                
                let available_width = resolve_size(&stack.width, constraints.max_width, constraints)
                    - padding.left - padding.right;
                let available_height = resolve_size(&stack.height, constraints.max_height, constraints)
                    - padding.top - padding.bottom;

                let child_constraints = Constraints::new(0.0, available_width, 0.0, available_height);
                
                let content_x = x + padding.left;
                let content_y = y + padding.top;

                let mut max_width: f32 = 0.0;
                let mut max_height: f32 = 0.0;

                for child in &stack.children {
                    let child_node = self.layout_element(child, &child_constraints, content_x, content_y);
                    max_width = max_width.max(child_node.rect.width);
                    max_height = max_height.max(child_node.rect.height);
                    node.add_child(child_node);
                }

                let width = max_width + padding.left + padding.right;
                let height = max_height + padding.top + padding.bottom;
                let (width, height) = constraints.constrain(width, height);

                node.rect = Rect::new(x, y, width, height);
                self.store_layout(&id, x, y, width, height, padding);
            }

            Element::Button(button) => {
                // Measure button text
                let char_width = 8.0;
                let line_height = 16.0;
                let text_width = button.text.len() as f32 * char_width;
                
                let width = text_width + 20.0; // Add some default padding
                let height = line_height + 10.0;

                let (width, height) = constraints.constrain(width, height);
                node.rect = Rect::new(x, y, width, height);
                self.store_layout(&id, x, y, width, height, &Padding::zero());
            }

            Element::Scroll(scroll) => {
                let scroll = scroll.as_ref();
                let padding = &scroll.padding;

                let width = resolve_size(&scroll.width, constraints.max_width, constraints);
                let height = resolve_size(&scroll.height, constraints.max_height, constraints);
                let (width, height) = constraints.constrain(width, height);

                // Layout content with unbounded constraints in scroll direction
                let content_constraints = match scroll.direction {
                    ScrollDirection::Vertical => Constraints::new(
                        0.0,
                        width - padding.left - padding.right,
                        0.0,
                        f32::INFINITY,
                    ),
                    ScrollDirection::Horizontal => Constraints::new(
                        0.0,
                        f32::INFINITY,
                        0.0,
                        height - padding.top - padding.bottom,
                    ),
                    ScrollDirection::Both => Constraints::new(
                        0.0,
                        f32::INFINITY,
                        0.0,
                        f32::INFINITY,
                    ),
                };

                let content_x = x + padding.left;
                let content_y = y + padding.top;
                let child_node = self.layout_element(&scroll.child, &content_constraints, content_x, content_y);
                node.add_child(child_node);

                node.rect = Rect::new(x, y, width, height);
                self.store_layout(&id, x, y, width, height, padding);
            }

            // Default case for other element types
            _ => {
                let default_padding = Padding::zero();
                let (width, height) = constraints.constrain(100.0, 50.0);
                node.rect = Rect::new(x, y, width, height);
                self.store_layout(&id, x, y, width, height, &default_padding);
            }
        }

        node
    }

    fn layout_flex_row(
        &mut self,
        node: &mut LayoutNode,
        children: &[Element],
        width_size: &Size,
        height_size: &Size,
        padding: &Padding,
        spacing: f32,
        alignment: Alignment,
        constraints: &Constraints,
        x: f32,
        y: f32,
    ) -> (f32, f32) {
        let width = resolve_size(width_size, constraints.max_width, constraints);
        let height = resolve_size(height_size, constraints.max_height, constraints);

        let content_x = x + padding.left;
        let content_y = y + padding.top;
        let available_width = width - padding.left - padding.right;
        let available_height = height - padding.top - padding.bottom;

        if children.is_empty() {
            return (width, height);
        }

        // Measure and position children horizontally
        let child_constraints = Constraints::new(0.0, available_width, 0.0, available_height);
        let mut current_x = content_x;

        for child in children {
            let child_node = self.layout_element(child, &child_constraints, current_x, content_y);
            let child_height = child_node.rect.height;
            
            // Apply cross-axis alignment
            let aligned_y = match alignment {
                Alignment::Start => content_y,
                Alignment::Center => content_y + (available_height - child_height) / 2.0,
                Alignment::End => content_y + available_height - child_height,
                Alignment::Stretch => content_y,
            };
            
            let mut aligned_node = child_node;
            aligned_node.rect.y = aligned_y;
            
            current_x += aligned_node.rect.width + spacing;
            node.add_child(aligned_node);
        }

        (width, height)
    }

    fn layout_flex_column(
        &mut self,
        node: &mut LayoutNode,
        children: &[Element],
        width_size: &Size,
        height_size: &Size,
        padding: &Padding,
        spacing: f32,
        alignment: Alignment,
        constraints: &Constraints,
        x: f32,
        y: f32,
    ) -> (f32, f32) {
        let width = resolve_size(width_size, constraints.max_width, constraints);
        let height = resolve_size(height_size, constraints.max_height, constraints);

        let content_x = x + padding.left;
        let content_y = y + padding.top;
        let available_width = width - padding.left - padding.right;
        let available_height = height - padding.top - padding.bottom;

        if children.is_empty() {
            return (width, height);
        }

        // Measure and position children vertically
        let child_constraints = Constraints::new(0.0, available_width, 0.0, available_height);
        let mut current_y = content_y;

        for child in children {
            let child_node = self.layout_element(child, &child_constraints, content_x, current_y);
            let child_width = child_node.rect.width;
            
            // Apply cross-axis alignment
            let aligned_x = match alignment {
                Alignment::Start => content_x,
                Alignment::Center => content_x + (available_width - child_width) / 2.0,
                Alignment::End => content_x + available_width - child_width,
                Alignment::Stretch => content_x,
            };
            
            let mut aligned_node = child_node;
            aligned_node.rect.x = aligned_x;
            
            current_y += aligned_node.rect.height + spacing;
            node.add_child(aligned_node);
        }

        (width, height)
    }

    fn store_layout(&mut self, id: &str, x: f32, y: f32, width: f32, height: f32, padding: &Padding) {
        let layout = ComputedLayout {
            position: (x, y),
            size: (width, height),
            content_size: (
                width - padding.left - padding.right,
                height - padding.top - padding.bottom,
            ),
            padding: padding.clone(),
        };
        self.computed_layouts.insert(id.to_string(), layout);
    }

    pub fn get_layout(&self, element_id: &str) -> Option<&ComputedLayout> {
        self.computed_layouts.get(element_id)
    }

    pub fn viewport_size(&self) -> (f32, f32) {
        (self.viewport_width, self.viewport_height)
    }

    pub fn resize_viewport(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }
}
