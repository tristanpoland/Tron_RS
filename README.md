# 🔷 Tron

> A powerful, composable template system for Rust

[![Crates.io](https://img.shields.io/crates/v/tron.svg)](https://crates.io/crates/tron)
[![Documentation](https://docs.rs/tron/badge.svg)](https://docs.rs/tron)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Build Status](https://github.com/yourusername/tron/workflows/CI/badge.svg)](https://github.com/tristanpoland/tron/actions)

Tron is a modern template engine that brings composability and type safety to code generation. Build and compose templates with confidence, execute them with rust-script, and generate Rust code dynamically.

## ✨ Features

- 🧩 **Composable Templates** - Nest and combine templates seamlessly
- 🎯 **Type-Safe** - Catch template errors at compile time
- 🔌 **rust-script Integration** - Execute generated code directly
- 📦 **Dependency Management** - Handle external crate dependencies gracefully
- 🛠 **Rich Tooling** - Comprehensive error handling and debugging

## 🚀 Quick Start

Add Tron to your project:

```toml
[dependencies]
tron = "0.1.0"

[features]
default = []
execute = ["tempfile", "which"]  # Optional: For rust-script support
```

Create your first template:

```rust
use tron::{TronTemplate, TronRef};

// Create a template
let mut template = TronTemplate::new(r#"
    fn @[name]@() {
        println!("@[message]@");
    }
"#)?;

let mut template_ref = TronRef::new(template);

// Set values
template_ref.set("name", "greet")?;
template_ref.set("message", "Hello from Tron!")?;

// Render
let code = template_ref.render()?;
```

## 🎭 Template Composition

Tron shines when composing templates:

```rust
// Create a module template
let mut module = TronTemplate::new(r#"
mod @[name]@ {
    @[content]@
}
"#)?;

// Create a function template
let mut function = TronTemplate::new(r#"
    fn @[func_name]@() {
        @[body]@
    }
"#)?;

// Compose them
let mut module_ref = TronRef::new(module);
let mut function_ref = TronRef::new(function);

function_ref.set("func_name", "example")?;
function_ref.set("body", "println!(\"Composed!\");")?;

module_ref.set("name", "generated")?;
module_ref.set_ref("content", function_ref)?;

// Renders:
// mod generated {
//     fn example() {
//         println!("Composed!");
//     }
// }
```

## 🎯 Key Concepts

### Templates

Templates are the building blocks of Tron. They use a simple `@[placeholder]@` syntax:

```rust
let template = TronTemplate::new("fn @[name]@() -> @[return_type]@ { @[body]@ }")?;
```

### Template References

`TronRef` wraps templates with additional capabilities:

```rust
let template_ref = TronRef::new(template)
    .with_dependency("serde = \"1.0\"");
```

### Template Assembly

Combine multiple templates with `TronAssembler`:

```rust
let mut assembler = TronAssembler::new();
assembler.add_template(header_ref);
assembler.add_template(body_ref);
assembler.add_template(footer_ref);

let combined = assembler.render_all()?;
```

## 📚 Documentation

Visit our [documentation](https://docs.rs/tron) for:
- Detailed API reference
- Advanced usage examples
- Best practices
- Tutorial guides

## 🤝 Contributing

We welcome contributions! Check out our [Contributing Guide](CONTRIBUTING.md) to get started.

1. Fork the repository
2. Create a feature branch
3. Submit a pull request

## 📝 License

Tron is MIT licensed. See [LICENSE](LICENSE) for details.

---

<div align="center">
Made with ❤️ by Tristan J. Poland for the Rust community
</div>