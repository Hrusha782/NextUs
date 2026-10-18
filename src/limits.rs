#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuntimeLimits {
    pub max_ui_depth: usize,
    pub max_children_per_container: usize,
    pub max_text_length: usize,
    pub max_list_items: usize,
    pub max_grid_cells: usize,
    pub max_tabs: usize,
    pub max_memory_per_session_mb: usize,
    pub max_render_time_ms: usize,
    pub max_events_per_second: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    DepthExceeded { current: usize, max: usize },
    TooManyChildren { current: usize, max: usize },
    TextTooLong { current: usize, max: usize },
    TooManyListItems { current: usize, max: usize },
    TooManyGridCells { current: usize, max: usize },
    TooManyTabs { current: usize, max: usize },
    MemoryExceeded { current: usize, max: usize },
    RenderTimeout { elapsed: u64, max: u64 },
    EventRateLimitExceeded { current: usize, max: usize },
}

impl Default for RuntimeLimits {
    fn default() -> Self {
        Self {
            max_ui_depth: 50,
            max_children_per_container: 100,
            max_text_length: 10_000,
            max_list_items: 1_000,
            max_grid_cells: 10_000,
            max_tabs: 50,
            max_memory_per_session_mb: 100,
            max_render_time_ms: 100,
            max_events_per_second: 100,
        }
    }
}

impl RuntimeLimits {
    pub fn validate_depth(&self, depth: usize) -> Result<(), ValidationError> {
        if depth > self.max_ui_depth {
            return Err(ValidationError::DepthExceeded {
                current: depth,
                max: self.max_ui_depth,
            });
        }
        Ok(())
    }

    pub fn validate_children_count(&self, count: usize) -> Result<(), ValidationError> {
        if count > self.max_children_per_container {
            return Err(ValidationError::TooManyChildren {
                current: count,
                max: self.max_children_per_container,
            });
        }
        Ok(())
    }

    pub fn validate_text_length(&self, length: usize) -> Result<(), ValidationError> {
        if length > self.max_text_length {
            return Err(ValidationError::TextTooLong {
                current: length,
                max: self.max_text_length,
            });
        }
        Ok(())
    }

    pub fn validate_list_items(&self, count: usize) -> Result<(), ValidationError> {
        if count > self.max_list_items {
            return Err(ValidationError::TooManyListItems {
                current: count,
                max: self.max_list_items,
            });
        }
        Ok(())
    }

    pub fn validate_grid_cells(&self, count: usize) -> Result<(), ValidationError> {
        if count > self.max_grid_cells {
            return Err(ValidationError::TooManyGridCells {
                current: count,
                max: self.max_grid_cells,
            });
        }
        Ok(())
    }

    pub fn validate_tabs(&self, count: usize) -> Result<(), ValidationError> {
        if count > self.max_tabs {
            return Err(ValidationError::TooManyTabs {
                current: count,
                max: self.max_tabs,
            });
        }
        Ok(())
    }
}

use std::fmt;

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidationError::DepthExceeded { current, max } => {
                write!(f, "UI depth exceeded: {} levels (max {})", current, max)
            }
            ValidationError::TooManyChildren { current, max } => {
                write!(f, "Too many children: {} items (max {})", current, max)
            }
            ValidationError::TextTooLong { current, max } => {
                write!(f, "Text too long: {} characters (max {})", current, max)
            }
            ValidationError::TooManyListItems { current, max } => {
                write!(f, "Too many list items: {} items (max {})", current, max)
            }
            ValidationError::TooManyGridCells { current, max } => {
                write!(f, "Too many grid cells: {} cells (max {})", current, max)
            }
            ValidationError::TooManyTabs { current, max } => {
                write!(f, "Too many tabs: {} tabs (max {})", current, max)
            }
            ValidationError::MemoryExceeded { current, max } => {
                write!(f, "Memory exceeded: {} MB (max {} MB)", current, max)
            }
            ValidationError::RenderTimeout { elapsed, max } => {
                write!(f, "Render timeout: {} ms (max {} ms)", elapsed, max)
            }
            ValidationError::EventRateLimitExceeded { current, max } => {
                write!(
                    f,
                    "Event rate limit exceeded: {} events/sec (max {})",
                    current, max
                )
            }
        }
    }
}
