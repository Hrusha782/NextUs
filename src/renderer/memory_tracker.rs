//! Memory Tracker for NextUs
//!
//! Tracks memory allocations and provides statistics for debugging and optimization.

use std::collections::HashMap;
use std::time::Instant;

/// Information about a single allocation
#[derive(Debug, Clone)]
pub struct AllocationInfo {
    pub id: String,
    pub size_bytes: usize,
    pub allocation_time: Instant,
    pub source: String,
}

impl AllocationInfo {
    pub fn new(id: String, size_bytes: usize, source: String) -> Self {
        Self {
            id,
            size_bytes,
            allocation_time: Instant::now(),
            source,
        }
    }

    pub fn age_ms(&self) -> u128 {
        self.allocation_time.elapsed().as_millis()
    }
}

/// Memory usage statistics
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    pub total_allocated: usize,
    pub total_freed: usize,
    pub current_usage: usize,
    pub peak_usage: usize,
    pub allocation_count: usize,
    pub free_count: usize,
}

impl MemoryStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fragmentation_ratio(&self) -> f64 {
        if self.total_allocated == 0 {
            0.0
        } else {
            1.0 - (self.current_usage as f64 / self.total_allocated as f64)
        }
    }
}

/// Tracks memory allocations across the application
pub struct MemoryTracker {
    allocations: HashMap<String, AllocationInfo>,
    stats: MemoryStats,
    limit_bytes: Option<usize>,
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self {
            allocations: HashMap::new(),
            stats: MemoryStats::new(),
            limit_bytes: None,
        }
    }

    pub fn with_limit(limit_bytes: usize) -> Self {
        Self {
            allocations: HashMap::new(),
            stats: MemoryStats::new(),
            limit_bytes: Some(limit_bytes),
        }
    }

    /// Record a new allocation
    pub fn allocate(
        &mut self,
        id: String,
        size_bytes: usize,
        source: String,
    ) -> Result<(), MemoryError> {
        // Check if allocation would exceed limit
        if let Some(limit) = self.limit_bytes {
            if self.stats.current_usage + size_bytes > limit {
                return Err(MemoryError::LimitExceeded {
                    requested: size_bytes,
                    available: limit.saturating_sub(self.stats.current_usage),
                    limit,
                });
            }
        }

        let info = AllocationInfo::new(id.clone(), size_bytes, source);
        self.allocations.insert(id, info);

        self.stats.total_allocated += size_bytes;
        self.stats.current_usage += size_bytes;
        self.stats.allocation_count += 1;

        if self.stats.current_usage > self.stats.peak_usage {
            self.stats.peak_usage = self.stats.current_usage;
        }

        Ok(())
    }

    /// Record a deallocation
    pub fn free(&mut self, id: &str) -> Result<usize, MemoryError> {
        let info = self
            .allocations
            .remove(id)
            .ok_or(MemoryError::NotFound(id.to_string()))?;

        self.stats.total_freed += info.size_bytes;
        self.stats.current_usage -= info.size_bytes;
        self.stats.free_count += 1;

        Ok(info.size_bytes)
    }

    /// Get information about a specific allocation
    pub fn get_allocation(&self, id: &str) -> Option<&AllocationInfo> {
        self.allocations.get(id)
    }

    /// Get current memory statistics
    pub fn stats(&self) -> &MemoryStats {
        &self.stats
    }

    /// Get all active allocations
    pub fn allocations(&self) -> &HashMap<String, AllocationInfo> {
        &self.allocations
    }

    /// Find allocations larger than a threshold
    pub fn large_allocations(&self, threshold_bytes: usize) -> Vec<&AllocationInfo> {
        self.allocations
            .values()
            .filter(|info| info.size_bytes >= threshold_bytes)
            .collect()
    }

    /// Find allocations from a specific source
    pub fn allocations_from_source(&self, source: &str) -> Vec<&AllocationInfo> {
        self.allocations
            .values()
            .filter(|info| info.source == source)
            .collect()
    }

    /// Get memory usage by source
    pub fn usage_by_source(&self) -> HashMap<String, usize> {
        let mut usage: HashMap<String, usize> = HashMap::new();
        for info in self.allocations.values() {
            *usage.entry(info.source.clone()).or_insert(0) += info.size_bytes;
        }
        usage
    }

    /// Clear all tracking data
    pub fn reset(&mut self) {
        self.allocations.clear();
        self.stats = MemoryStats::new();
    }

    /// Set or update the memory limit
    pub fn set_limit(&mut self, limit_bytes: Option<usize>) {
        self.limit_bytes = limit_bytes;
    }

    /// Check if we're within memory limits
    pub fn within_limit(&self) -> bool {
        match self.limit_bytes {
            Some(limit) => self.stats.current_usage <= limit,
            None => true,
        }
    }

    /// Get available memory (if limit is set)
    pub fn available(&self) -> Option<usize> {
        self.limit_bytes
            .map(|limit| limit.saturating_sub(self.stats.current_usage))
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during memory tracking
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryError {
    LimitExceeded {
        requested: usize,
        available: usize,
        limit: usize,
    },
    NotFound(String),
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryError::LimitExceeded {
                requested,
                available,
                limit,
            } => {
                write!(
                    f,
                    "Memory limit exceeded: requested {} bytes, {} available (limit: {})",
                    requested, available, limit
                )
            }
            MemoryError::NotFound(id) => {
                write!(f, "Allocation not found: {}", id)
            }
        }
    }
}

impl std::error::Error for MemoryError {}
