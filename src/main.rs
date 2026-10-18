use nextus::{parser, Element, Runtime, RuntimeLimits};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "--help" | "-h" => {
            print_usage();
            return;
        }
        "--version" | "-v" => {
            println!("NextUs {}", nextus::VERSION);
            return;
        }
        "--check" => {
            if args.len() < 3 {
                eprintln!("Error: --check requires a file path");
                std::process::exit(1);
            }
            run_check(&args[2]);
        }
        "--html" => {
            if args.len() < 3 {
                eprintln!("Error: --html requires a file path");
                std::process::exit(1);
            }
            run_html(&args[2]);
        }
        "--ast" => {
            if args.len() < 3 {
                eprintln!("Error: --ast requires a file path");
                std::process::exit(1);
            }
            run_ast(&args[2]);
        }
        path if path.ends_with(".nxt") => {
            run_program(path);
        }
        _ => {
            eprintln!("Unknown command or file: {}", command);
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("NextUs - A pure, deterministic DSL for messenger UIs");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    nextus <file.nxt>              Run a NextUs program");
    eprintln!("    nextus --check <file.nxt>      Validate without running");
    eprintln!("    nextus --html <file.nxt>       Render once and print the HTML");
    eprintln!("    nextus --ast <file.nxt>        Show the AST");
    eprintln!("    nextus --help                  Show this help");
    eprintln!("    nextus --version               Show version");
}

fn load_and_parse(file_path: &str) -> Result<nextus::Program, String> {
    let source = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read '{}': {}", file_path, e))?;

    let program = parser::parse(&source).map_err(|e| format!("Parse error: {}", e))?;

    parser::syntax::validate_program(&program).map_err(|e| format!("Validation error: {}", e))?;

    Ok(program)
}

fn run_check(file_path: &str) {
    match load_and_parse(file_path) {
        Ok(program) => {
            println!("✓ {} is valid", file_path);
            println!("  State variables: {}", program.state.len());
            println!("  Update rules: {}", program.update.len());
        }
        Err(e) => {
            eprintln!("✗ {}", e);
            std::process::exit(1);
        }
    }
}

fn run_ast(file_path: &str) {
    match load_and_parse(file_path) {
        Ok(program) => {
            println!("=== AST for {} ===\n", file_path);

            println!("STATE:");
            for var in &program.state {
                println!(
                    "  {} = {}",
                    var.name,
                    parser::syntax::format_expr(&var.initial_value)
                );
            }

            println!("\nVIEW:");
            println!("  params: {:?}", program.view.params);
            println!(
                "  body: {}",
                parser::syntax::format_expr(&program.view.body)
            );

            println!("\nUPDATE RULES:");
            for (i, rule) in program.update.iter().enumerate() {
                println!("  {}. when {:?}", i + 1, rule.pattern);
                for (var, expr) in &rule.state_update {
                    println!("       {} = {}", var, parser::syntax::format_expr(expr));
                }
            }
        }
        Err(e) => {
            eprintln!("✗ {}", e);
            std::process::exit(1);
        }
    }
}

/// Render once and print the HTML the host app would mount.
///
/// Only the markup goes to stdout, so the output can be piped straight into a
/// file or a formatter; progress and errors go to stderr.
fn run_html(file_path: &str) {
    let program = match load_and_parse(file_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("✗ {}", e);
            std::process::exit(1);
        }
    };

    let runtime = match Runtime::new(program, RuntimeLimits::default()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("✗ Runtime error: {:?}", e);
            std::process::exit(1);
        }
    };

    match runtime.render() {
        Ok(element) => println!("{}", nextus::render_html(&element)),
        Err(e) => {
            eprintln!("✗ Render error: {:?}", e);
            std::process::exit(1);
        }
    }
}

fn run_program(file_path: &str) {
    let program = match load_and_parse(file_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("✗ {}", e);
            std::process::exit(1);
        }
    };

    println!("✓ Loaded {}", file_path);

    let limits = RuntimeLimits::default();
    let runtime = match Runtime::new(program, limits) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("✗ Runtime error: {:?}", e);
            std::process::exit(1);
        }
    };

    println!("✓ Runtime initialized");
    println!("  State: {} variables", runtime.state().variables().len());
    for (name, value) in runtime.state().variables() {
        println!("    {} = {:?}", name, value);
    }

    println!("\n--- Rendering View ---\n");

    match runtime.render() {
        Ok(element) => {
            print_element(&element, 0);
        }
        Err(e) => {
            eprintln!("✗ Render error: {:?}", e);
            std::process::exit(1);
        }
    }
}

fn print_element(element: &Element, indent: usize) {
    let pad = "  ".repeat(indent);

    match element {
        Element::Text(t) => {
            println!("{}Text \"{}\"", pad, t.content);
        }
        Element::Button(b) => {
            println!("{}Button \"{}\"", pad, b.text);
        }
        Element::Input(i) => {
            println!("{}Input placeholder=\"{}\"", pad, i.placeholder);
        }
        Element::Image(i) => {
            println!("{}Image src=\"{}\"", pad, i.src);
        }
        Element::Avatar(a) => {
            println!("{}Avatar src=\"{}\"", pad, a.src);
        }
        Element::Row(r) => {
            println!("{}Row (spacing: {}):", pad, r.spacing);
            for child in &r.children {
                print_element(child, indent + 1);
            }
        }
        Element::Column(c) => {
            println!("{}Column (spacing: {}):", pad, c.spacing);
            for child in &c.children {
                print_element(child, indent + 1);
            }
        }
        Element::Stack(s) => {
            println!("{}Stack:", pad);
            for child in &s.children {
                print_element(child, indent + 1);
            }
        }
        Element::Container(c) => {
            println!("{}Container:", pad);
            for child in &c.children {
                print_element(child, indent + 1);
            }
        }
        Element::Scroll(s) => {
            println!("{}Scroll ({:?}):", pad, s.direction);
            print_element(&s.child, indent + 1);
        }
        Element::List(l) => {
            println!("{}List ({} items):", pad, l.items.len());
            for item in &l.items {
                print_element(item, indent + 1);
            }
        }
        Element::MessageList(ml) => {
            println!("{}MessageList ({} messages):", pad, ml.messages.len());
        }
        Element::MessageBubble(mb) => {
            println!("{}MessageBubble ({:?}):", pad, mb.message_type);
            print_element(&mb.content, indent + 1);
        }
        Element::Panel(p) => {
            println!("{}Panel:", pad);
            println!("{}  header:", pad);
            print_element(&p.header, indent + 2);
            println!("{}  content:", pad);
            print_element(&p.content, indent + 2);
        }
        Element::Modal(m) => {
            println!("{}Modal:", pad);
            print_element(&m.content, indent + 1);
        }
        Element::Popover(p) => {
            println!("{}Popover:", pad);
            print_element(&p.content, indent + 1);
        }
        _ => {
            println!("{}[{:?}]", pad, std::mem::discriminant(element));
        }
    }
}
