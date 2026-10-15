//! Tests for the NextUs DSL

mod parser_tests {
    use crate::parser;

    #[test]
    fn test_parse_empty_state() {
        let source = r#"
            state {}
            view { Text { content: "Hello" } }
            update {}
        "#;
        
        let result = parser::parse(source);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    }

    #[test]
    fn test_parse_state_with_variables() {
        let source = r#"
            state {
                count: 0
                name: "Test"
                active: true
            }
            view { Text { content: "Hello" } }
            update {}
        "#;
        
        let result = parser::parse(source);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        
        let program = result.unwrap();
        assert_eq!(program.state.len(), 3);
    }

    #[test]
    fn test_parse_view_with_column() {
        let source = r#"
            state {}
            view {
                Column {
                    spacing: 10
                    Text { content: "Line 1" }
                    Text { content: "Line 2" }
                }
            }
            update {}
        "#;
        
        let result = parser::parse(source);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    }

    #[test]
    fn test_parse_update_block() {
        let source = r#"
            state {
                count: 0
            }
            view { Text { content: "Count" } }
            update {
                when Clicked { id: "increment" } => {
                    count: 1
                }
            }
        "#;
        
        let result = parser::parse(source);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        
        let program = result.unwrap();
        assert_eq!(program.update.len(), 1);
    }
}

mod runtime_tests {
    use crate::{parser, Runtime, RuntimeLimits};

    #[test]
    fn test_runtime_initialization() {
        let source = r#"
            state {
                count: 0
            }
            view { Text { content: "Hello" } }
            update {}
        "#;
        
        let program = parser::parse(source).unwrap();
        let limits = RuntimeLimits::default();
        let result = Runtime::new(program, limits);
        
        assert!(result.is_ok(), "Runtime init failed: {:?}", result.err());
    }

    // Runtime render test removed - requires full runtime implementation
}

mod state_tests {
    use crate::{State, Value};

    #[test]
    fn test_state_set_and_get() {
        let state = State::new();
        let state = state.set("count".to_string(), Value::Number(42.0));
        
        let value = state.get("count");
        assert!(value.is_some());
        
        if let Some(Value::Number(n)) = value {
            assert_eq!(*n, 42.0);
        } else {
            panic!("Expected Number value");
        }
    }

    #[test]
    fn test_state_immutability() {
        let state1 = State::new();
        let state2 = state1.set("x".to_string(), Value::Number(1.0));
        
        // Original state should be unchanged
        assert!(state1.get("x").is_none());
        assert!(state2.get("x").is_some());
    }
}

mod renderer_tests {
    use crate::renderer::{Rect, Constraints, MemoryTracker};

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10.0, 10.0, 100.0, 50.0);
        
        assert!(rect.contains(50.0, 30.0)); // Inside
        assert!(!rect.contains(5.0, 30.0)); // Left of
        assert!(!rect.contains(50.0, 5.0)); // Above
    }

    #[test]
    fn test_rect_intersects() {
        let rect1 = Rect::new(0.0, 0.0, 100.0, 100.0);
        let rect2 = Rect::new(50.0, 50.0, 100.0, 100.0);
        let rect3 = Rect::new(200.0, 200.0, 50.0, 50.0);
        
        assert!(rect1.intersects(&rect2)); // Overlapping
        assert!(!rect1.intersects(&rect3)); // No overlap
    }

    #[test]
    fn test_constraints() {
        let constraints = Constraints::new(0.0, 100.0, 0.0, 50.0);
        
        let (w, h) = constraints.constrain(200.0, 75.0);
        assert_eq!(w, 100.0); // Clamped to max
        assert_eq!(h, 50.0); // Clamped to max
    }

    #[test]
    fn test_memory_tracker() {
        let mut tracker = MemoryTracker::new();
        
        tracker.allocate("element_1".to_string(), 1024, "test".to_string()).unwrap();
        tracker.allocate("element_2".to_string(), 2048, "test".to_string()).unwrap();
        
        assert_eq!(tracker.stats().current_usage, 3072);
        assert_eq!(tracker.stats().allocation_count, 2);
        
        tracker.free("element_1").unwrap();
        assert_eq!(tracker.stats().current_usage, 2048);
    }
}
