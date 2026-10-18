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

    /// Renders `view` and returns the text of the single Text element in it.
    fn render_text(source: &str) -> Result<String, crate::runtime::RuntimeError> {
        let program = parser::parse(source).unwrap();
        let runtime = Runtime::new(program, RuntimeLimits::default())?;
        match runtime.render()? {
            crate::Element::Text(t) => Ok(t.content),
            other => panic!("Expected a Text element, got {:?}", other),
        }
    }

    #[test]
    fn concat_string_and_number_with_plus() {
        let source = r#"
            state { count: 7 }
            view { Text { content: "Count: " + count } }
            update {}
        "#;
        assert_eq!(render_text(source).unwrap(), "Count: 7");
    }

    #[test]
    fn concat_two_strings_with_plus() {
        let source = r#"
            state { name: "world" }
            view { Text { content: "hello " + name } }
            update {}
        "#;
        assert_eq!(render_text(source).unwrap(), "hello world");
    }

    #[test]
    fn concat_number_on_the_left_of_a_string() {
        let source = r#"
            state { n: 3 }
            view { Text { content: n + " unread" } }
            update {}
        "#;
        assert_eq!(render_text(source).unwrap(), "3 unread");
    }

    #[test]
    fn whole_numbers_concat_without_a_decimal_point() {
        let source = r#"
            state { count: 0 }
            view { Text { content: "" + count } }
            update {}
        "#;
        assert_eq!(render_text(source).unwrap(), "0");
    }

    #[test]
    fn plus_on_two_numbers_still_adds() {
        let source = r#"
            state { count: 2 }
            view { Text { content: count + 1 } }
            update {}
        "#;
        // Text coerces the number for display; the addition itself is numeric.
        assert_eq!(render_text(source).unwrap(), "3");
    }

    #[test]
    fn concat_is_bounded_by_max_text_length() {
        use crate::limits::ValidationError;

        let source = r#"
            state { a: "xxxxxx" }
            view { Text { content: a + a } }
            update {}
        "#;
        let program = parser::parse(source).unwrap();
        let limits = RuntimeLimits {
            max_text_length: 8,
            ..RuntimeLimits::default()
        };
        let runtime = Runtime::new(program, limits).unwrap();

        match runtime.render() {
            Err(crate::runtime::RuntimeError::LimitExceeded(ValidationError::TextTooLong {
                current,
                max,
            })) => {
                assert_eq!((current, max), (12, 8));
            }
            other => panic!("Expected TextTooLong, got {:?}", other),
        }
    }

    #[test]
    fn concat_rejects_values_with_no_text_form() {
        let source = r#"
            state { items: [1, 2] }
            view { Text { content: "items: " + items } }
            update {}
        "#;
        assert!(matches!(
            render_text(source),
            Err(crate::runtime::RuntimeError::InvalidOperation(_))
        ));
    }

    #[test]
    fn comparison_operators_evaluate() {
        // if/then/else forces each comparison to actually be evaluated.
        let cases = [
            ("1 < 2", "yes"),
            ("2 < 1", "no"),
            ("2 > 1", "yes"),
            ("1 >= 1", "yes"),
            ("1 <= 0", "no"),
            ("2 == 2", "yes"),
            ("2 != 2", "no"),
            ("\"a\" == \"a\"", "yes"),
            ("\"a\" != \"b\"", "yes"),
            ("\"a\" < \"b\"", "yes"),
            ("true == true", "yes"),
        ];

        for (expr, want) in cases {
            let source = format!(
                "state {{}}
view {{ Text {{ content: if {} then \"yes\" else \"no\" }} }}
update {{}}",
                expr
            );
            let got = render_text(&source)
                .unwrap_or_else(|e| panic!("`{}` failed to evaluate: {:?}", expr, e));
            assert_eq!(got, want, "`{}`", expr);
        }
    }

    #[test]
    fn modulo_divides_and_rejects_zero() {
        let source = r#"
            state { x: 7 }
            view { Text { content: x % 3 } }
            update {}
        "#;
        assert_eq!(render_text(source).unwrap(), "1");

        let by_zero = r#"
            state { x: 7 }
            view { Text { content: x % 0 } }
            update {}
        "#;
        assert!(matches!(
            render_text(by_zero),
            Err(crate::runtime::RuntimeError::DivisionByZero)
        ));
    }

    #[test]
    fn ordering_rejects_types_with_no_order() {
        let source = r#"
            state { items: [1] }
            view { Text { content: if items > 2 then "a" else "b" } }
            update {}
        "#;
        assert!(matches!(
            render_text(source),
            Err(crate::runtime::RuntimeError::InvalidOperation(_))
        ));
    }

    /// `ELEMENT_NAMES` is the contract between `--check`, the runtime and the
    /// LSP. If a name here has no runtime arm, `--check` would accept a program
    /// the runtime cannot render.
    #[test]
    fn element_names_match_runtime() {
        use crate::element::ELEMENT_NAMES;
        use crate::runtime::RuntimeError;

        for name in ELEMENT_NAMES {
            let source = format!(
                "state {{}}
view {{ {} {{ }} }}
update {{}}",
                name
            );
            let program = parser::parse(&source)
                .unwrap_or_else(|e| panic!("{} failed to parse: {:?}", name, e));
            let runtime = Runtime::new(program, RuntimeLimits::default()).unwrap();

            if let Err(RuntimeError::UnknownElement(unknown)) = runtime.render() {
                panic!(
                    "'{}' is in ELEMENT_NAMES but the runtime cannot build it",
                    unknown
                );
            }
        }
    }
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
    use crate::renderer::{Constraints, MemoryTracker, Rect};

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

        tracker
            .allocate("element_1".to_string(), 1024, "test".to_string())
            .unwrap();
        tracker
            .allocate("element_2".to_string(), 2048, "test".to_string())
            .unwrap();

        assert_eq!(tracker.stats().current_usage, 3072);
        assert_eq!(tracker.stats().allocation_count, 2);

        tracker.free("element_1").unwrap();
        assert_eq!(tracker.stats().current_usage, 2048);
    }
}

mod validation_tests {
    use crate::parser;

    fn validate(source: &str) -> Result<(), String> {
        let program = parser::parse(source).unwrap();
        crate::parser::syntax::validate_program(&program)
    }

    #[test]
    fn accepts_known_elements() {
        let source = r#"
            state {}
            view { Column { Container { Text { content: "hi" } } } }
            update {}
        "#;
        assert!(validate(source).is_ok());
    }

    #[test]
    fn rejects_unknown_element_in_view() {
        let source = r#"
            state {}
            view { Column { Foo { } } }
            update {}
        "#;
        let err = validate(source).expect_err("Foo is not a primitive");
        assert!(
            err.contains("Foo"),
            "error should name the element: {}",
            err
        );
    }

    /// The app's top bar and left rail are never programmable, so `TopBar`
    /// must not be reachable from a design or a build.
    #[test]
    fn rejects_top_bar() {
        let source = r#"
            state {}
            view { TopBar { } }
            update {}
        "#;
        let err = validate(source).expect_err("TopBar must not be programmable");
        assert!(
            err.contains("TopBar"),
            "error should name the element: {}",
            err
        );
    }

    #[test]
    fn rejects_unknown_element_nested_in_a_property() {
        let source = r#"
            state {}
            view { Tooltip { content: "x" Column { Bogus { } } } }
            update {}
        "#;
        assert!(validate(source).is_err());
    }

    #[test]
    fn rejects_unknown_element_in_state_and_update() {
        let state_source = r#"
            state { broken: Nope { } }
            view { Text { content: "hi" } }
            update {}
        "#;
        assert!(validate(state_source).is_err());

        let update_source = r#"
            state { x: 0 }
            view { Text { content: "hi" } }
            update {
                when Clicked { id: "b" } => { x: Nope { } }
            }
        "#;
        assert!(validate(update_source).is_err());
    }
}

mod html_tests {
    use crate::{parser, render_html, Runtime, RuntimeLimits};

    /// Parse a program, render it once and return the HTML the host would mount.
    fn html(source: &str) -> String {
        let program = parser::parse(source).unwrap_or_else(|e| panic!("parse failed: {:?}", e));
        let runtime = Runtime::new(program, RuntimeLimits::default()).unwrap();
        let element = runtime
            .render()
            .unwrap_or_else(|e| panic!("render failed: {:?}", e));
        render_html(&element)
    }

    const SIMPLE: &str = r#"
        state { count: 0 }
        view {
            Column {
                spacing: 10
                Text { content: "Counter: " + count }
                Button { id: "increment"  text: "+" }
            }
        }
        update {
            when Clicked { id: "increment" } => { count: count + 1 }
        }
    "#;

    /// The whole point of week 1 track B: a simple .nxt becomes DOM.
    #[test]
    fn simple_program_renders_to_html() {
        assert_eq!(
            html(SIMPLE),
            concat!(
                r#"<div class="nxt-column" style="display:flex;flex-direction:column;gap:10px;"#,
                r#"align-items:flex-start;justify-content:flex-start">"#,
                r#"<span class="nxt-text">Counter: 0</span>"#,
                r#"<button class="nxt-button nxt-button-primary" type="button" "#,
                r#"data-nxt-id="increment">+</button>"#,
                r#"</div>"#
            )
        );
    }

    /// `data-nxt-id` is how the app finds elements, so it must appear exactly
    /// when the program set an id -- never as an empty attribute.
    #[test]
    fn id_becomes_data_nxt_id_only_when_set() {
        let out = html(SIMPLE);
        assert!(out.contains(r#"data-nxt-id="increment""#));
        assert!(
            !out.contains(r#"data-nxt-id="""#) && !out.contains("data-nxt-id>"),
            "an element without an id must not carry data-nxt-id: {}",
            out
        );
    }

    /// The app walks `.parentElement`, so children must be direct children
    /// with no wrapper node in between.
    #[test]
    fn dom_nesting_mirrors_the_nxt_tree() {
        let out = html(
            r#"
            state {}
            view {
                Column {
                    Button { id: "msg-ctx-delete-me"  text: "Delete" }
                }
            }
            update {}
        "#,
        );
        let column_open = out.find("<div class=\"nxt-column\"").expect("column");
        let button_open = out.find("<button").expect("button");
        let between = &out[column_open..button_open];
        assert_eq!(
            between.matches('<').count(),
            1,
            "no node may sit between a Column and its child: {}",
            out
        );
    }

    /// A .nxt program is untrusted input that reaches the host DOM.
    #[test]
    fn text_content_is_escaped() {
        let out = html(
            r#"
            state {}
            view { Text { content: "<script>alert('x')</script> & more" } }
            update {}
        "#,
        );
        assert!(!out.contains("<script>"), "script tag survived: {}", out);
        assert!(out.contains("&lt;script&gt;"));
        assert!(out.contains("&amp; more"));
    }

    #[test]
    fn attribute_values_are_escaped() {
        let out = html(
            r#"
            state {}
            view { Input { placeholder: "say \"hi\" <b>" } }
            update {}
        "#,
        );
        assert!(out.contains("&quot;hi&quot;"), "quotes unescaped: {}", out);
        assert!(out.contains("&lt;b&gt;"));
        // The attribute must not be terminated early by the payload.
        assert!(!out.contains(r#"placeholder="say ""#));
    }

    #[test]
    fn id_is_escaped_in_the_data_attribute() {
        let out = html(
            r#"
            state {}
            view { Button { id: "a\" onclick=\"evil()"  text: "x" } }
            update {}
        "#,
        );
        assert!(!out.contains("onclick=\"evil()\""), "escaped out: {}", out);
        assert!(out.contains("&quot;"));
    }

    /// Same tree in, same bytes out -- the language is deterministic and the
    /// DOM output must not leak hash ordering.
    #[test]
    fn rendering_is_deterministic() {
        let first = html(SIMPLE);
        for _ in 0..5 {
            assert_eq!(html(SIMPLE), first);
        }
    }

    #[test]
    fn void_elements_have_no_closing_tag() {
        let out = html(
            r#"
            state {}
            view { Column { Input { placeholder: "p" }  Divider { } } }
            update {}
        "#,
        );
        assert!(!out.contains("</input>"), "{}", out);
        assert!(!out.contains("</hr>"), "{}", out);
    }

    /// Style defaults must not be inlined, or the host stylesheet can never
    /// win against them.
    #[test]
    fn default_style_is_not_inlined() {
        let out = html(
            r#"
            state {}
            view { Text { content: "plain" } }
            update {}
        "#,
        );
        assert_eq!(out, r#"<span class="nxt-text">plain</span>"#);
    }

    /// Every element the runtime can build must also render to HTML without
    /// panicking and must produce a tag.
    #[test]
    fn every_element_renders_to_html() {
        use crate::element::ELEMENT_NAMES;

        for name in ELEMENT_NAMES {
            let source = format!("state {{}}\nview {{ {} {{ }} }}\nupdate {{}}", name);
            let program = parser::parse(&source).unwrap();
            let runtime = Runtime::new(program, RuntimeLimits::default()).unwrap();
            let element = runtime
                .render()
                .unwrap_or_else(|e| panic!("{} failed to render: {:?}", name, e));
            let out = render_html(&element);
            assert!(out.starts_with('<'), "{} produced no tag: {:?}", name, out);
        }
    }

    /// Negative: a program cannot smuggle a script URL into the DOM.
    #[test]
    fn script_urls_are_dropped_from_src() {
        let out = html(
            r#"
            state {}
            view {
                Column {
                    Image { src: "javascript:alert(1)"  alt: "a" }
                    Image { src: "data:text/html;base64,PHNjcmlwdD4="  alt: "b" }
                    Image { src: "https://example.com/ok.png"  alt: "c" }
                }
            }
            update {}
        "#,
        );
        assert!(!out.contains("javascript:"), "script URL survived: {}", out);
        assert!(
            !out.contains("data:text/html"),
            "html data URL survived: {}",
            out
        );
        assert!(
            out.contains(r#"src="https://example.com/ok.png""#),
            "{}",
            out
        );
    }

    #[test]
    fn src_and_alt_are_escaped() {
        let out = html(
            r#"
            state {}
            view { Image { src: "a.png\" onerror=\"evil()"  alt: "<b>" } }
            update {}
        "#,
        );
        assert!(!out.contains("onerror=\"evil()\""), "broke out: {}", out);
        assert!(out.contains("&lt;b&gt;"), "{}", out);
    }

    /// Negative: ids the runtime synthesises must not enter the namespace the
    /// app queries with `[data-nxt-id="..."]`.
    #[test]
    fn message_list_does_not_fabricate_ids() {
        let out = html(
            r#"
            state {}
            view {
                MessageList {
                    Text { content: "one" }
                    Text { content: "two" }
                }
            }
            update {}
        "#,
        );
        assert!(
            !out.contains("data-nxt-id"),
            "synthetic ids leaked into data-nxt-id: {}",
            out
        );
        assert!(out.contains(r#"data-nxt-index="0""#), "{}", out);
    }

    /// Booleans must be readable as `dataset.x === "true"`; a valueless
    /// attribute reads as the empty string, which is falsy.
    #[test]
    fn data_booleans_carry_an_explicit_value() {
        let out = html(
            r#"
            state {}
            view { MessageList { auto_scroll: true } }
            update {}
        "#,
        );
        assert!(out.contains(r#"data-nxt-auto-scroll="true""#), "{}", out);
        assert!(!out.contains("data-nxt-auto-scroll>"), "{}", out);
        assert!(!out.contains("data-nxt-auto-scroll "), "{}", out);
    }

    #[test]
    fn disabled_and_hidden_reach_the_dom() {
        let out = html(
            r#"
            state {}
            view {
                Column {
                    Button { id: "b"  text: "x"  enabled: false }
                    Text { id: "t"  content: "gone"  visible: false }
                }
            }
            update {}
        "#,
        );
        assert!(out.contains("disabled"), "{}", out);
        assert!(out.contains(r#"data-nxt-enabled="false""#), "{}", out);
        assert!(out.contains("display:none"), "{}", out);
    }

    /// A search field must not answer to `.nxt-input`, which the send action
    /// uses to find the message box.
    #[test]
    fn search_bar_input_is_not_the_message_input() {
        let out = html(
            r#"
            state {}
            view { SearchBar { placeholder: "find" } }
            update {}
        "#,
        );
        assert!(out.contains("nxt-searchbar-input"), "{}", out);
        assert!(
            !out.contains(r#"class="nxt-input"#),
            "search box claimed .nxt-input: {}",
            out
        );
    }

    /// Enum names in the app contract come from explicit matches, not from
    /// `derive(Debug)`, which a rename would silently change.
    #[test]
    fn contract_enums_are_lowercase_names() {
        let out = html(
            r#"
            state {}
            view { Timestamp { time: 5  format: "Relative" } }
            update {}
        "#,
        );
        assert!(out.contains(r#"data-nxt-format="relative""#), "{}", out);
    }

    #[test]
    fn message_bubble_carries_the_class_the_app_listens_on() {
        let out = html(
            r#"
            state {}
            view { MessageBubble { type: "Sent"  Text { content: "hi" } } }
            update {}
        "#,
        );
        assert!(out.contains("nxt-bubble"), "{}", out);
        assert!(out.contains("nxt-bubble-sent"), "{}", out);
    }

    #[test]
    fn behaviour_properties_become_data_attributes() {
        let out = html(
            r#"
            state {}
            view {
                Button {
                    id: "nick-save"
                    text: "Save"
                    action: "add_friend"
                    actionInput: "nick-input"
                    resultTo: "nick-result"
                    focusAfter: "nick-input"
                }
            }
            update {}
        "#,
        );
        assert!(out.contains(r#"data-nxt-action="add_friend""#), "{}", out);
        assert!(
            out.contains(r#"data-nxt-action-input="nick-input""#),
            "{}",
            out
        );
        assert!(
            out.contains(r#"data-nxt-result-to="nick-result""#),
            "{}",
            out
        );
        assert!(
            out.contains(r#"data-nxt-focus-after="nick-input""#),
            "{}",
            out
        );
    }
}

mod properties_tests {
    use crate::properties::Properties;

    /// An absent `id:` arrives here as an empty string. It must not become
    /// `Some("")`, which would emit a meaningless `data-nxt-id=""`.
    #[test]
    fn empty_id_is_no_id() {
        assert_eq!(Properties::default().with_id("").id, None);
        assert_eq!(
            Properties::default().with_id("send").id,
            Some("send".to_string())
        );
    }
}
