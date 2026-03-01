# AGENTS.md - Guidelines for AI Agents

This file contains guidelines for AI coding agents working in this repository.

---

## 1. Build, Lint, and Test Commands

### Build Commands
```bash
# Build the project
cargo build

# Build in release mode
cargo build --release

# Run the application
cargo run

# Run in release mode
cargo run --release
```

### Linting Commands
```bash
# Run clippy for all warnings
cargo clippy

# Run clippy with strict warnings
cargo clippy -- -W clippy::all

# Run cargo check (faster than full build)
cargo check

# Format code
cargo fmt

# Check formatting without making changes
cargo fmt -- --check
```

### Testing Commands
```bash
# Run all tests
cargo test

# Run a single test by name
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run doc tests
cargo test --doc
```

### Documentation
```bash
# Generate documentation
cargo doc

# Generate documentation without building dependencies
cargo doc --no-deps
```

---

## 2. Code Style Guidelines

### General Principles
- Prefer clear, idiomatic Rust over clever code
- Keep functions small and focused on single responsibility
- Avoid premature abstraction - prefer concrete types until patterns emerge
- Use meaningful names that convey intent

### Naming Conventions

**Variables and Functions**
- Use `snake_case` for variables and function names
- Prefer descriptive names over abbreviations (except: `buf`, `idx`, `val`, `err`)
- Prefix unused variables with underscore: `_unused_var`

**Types**
- Use `PascalCase` for structs, enums, and type aliases
- Use `CamelCase` for trait names

**Modules**
- Use `snake_case` for module names
- One module per file, or use `mod.rs` for module directories

### Imports

**Organize imports in this order:**
1. Standard library imports (`std::`, `core::`)
2. External crate imports (alphabetical)
3. Local imports (`crate::`, `super::`)

**Example:**
```rust
use std::collections::HashMap;
use std::sync::Arc;

use winit::event::WindowEvent;
use wgpu::{Device, Queue};

use crate::domain::Buffer;
use crate::error::Result;
```

**Avoid:**
- Wildcard imports (`use crate::foo::*`) except for re-exports
- Deep import paths when module-level imports suffice

### Error Handling

**Use the project's custom error types:**
```rust
use crate::error::{EditorError, Result};

// Return Result for functions that can fail
fn some_function() -> Result<String> {
    // Use ? operator for propagation
    let content = std::fs::read_to_string("file.txt")
        .map_err(EditorError::IoError)?;
    Ok(content)
}
```

**Avoid:**
- Using `unwrap()` or `expect()` in production code
- Using `anyhow::Result` for internal functions (use `crate::error::Result`)
- Silently ignoring errors with `let _ = ...`

### Types and Ownership

**Prefer:**
- Value types over reference types when possible
- `&str` over `&String` for function parameters
- `impl Trait` for return types when concrete type doesn't matter
- `Arc<T>` for shared ownership across threads
- `&mut T` for mutable references (avoid `RefCell` unless necessary)

**Avoid:**
- Excessive cloning (prefer references when possible)
- `dyn Trait` when generic bounds work
- Unnecessary `Box<T>` allocation

### Structs and Enums

**Use struct shorthand for simple constructors:**
```rust
// Good
impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            cursor: Cursor::new(),
        }
    }
}

// Avoid unnecessary verbosity
impl Editor {
    pub fn new() -> Self {
        return Self {
            buffer: Buffer::new(),
            cursor: Cursor::new(),
        };
    }
}
```

**Use enum for state variants:**
```rust
#[derive(Clone, Copy, PartialEq)]
pub enum MouseButtonState {
    Released,
    Pressed,
}
```

### Pattern Matching

**Use exhaustive pattern matching:**
```rust
match action {
    MenuAction::Save => { /* ... */ }
    MenuAction::Open => { /* ... */ }
    MenuAction::Quit => { /* ... */ }
}
```

**Use `_` for ignored cases:**
```rust
fn handle_key(event: KeyEvent) {
    match event.state {
        ElementState::Pressed => { /* ... */ }
        ElementState::Released => { /* do nothing */ }
    }
}
```

### Documentation

**Document public APIs:**
```rust
/// Creates a new editor with default configuration.
pub fn new() -> Self

/// Highlights the given buffer for display.
///
/// Returns a HashMap mapping line indices to color vectors.
pub fn highlight(&self, buffer: &Buffer) -> HashMap<usize, Vec<[f32; 4]>>
```

**Avoid:**
- Commenting obvious code
- Leaving commented-out code (delete it, use git if needed)

### Architecture

This project follows Clean Architecture:

```
src/
├── domain/        # Pure business logic (no external deps)
├── ports/        # Trait definitions (interfaces)
├── application/  # Use cases & orchestration
├── infrastructure/ # External implementations
├── interface/    # UI & event handling
├── renderer/     # GPU rendering
├── state/        # GPU state
├── menu/         # Menu handling
├── config/        # Configuration
├── syntax/        # Syntax highlighting
├── themes/        # Theme definitions
└── error.rs      # Error types
```

**Dependency Rules:**
- Domain → No dependencies (except std)
- Ports → Domain
- Application → Domain + Ports
- Infrastructure → Domain + Ports
- Interface → Application + Domain

---

## 3. Project-Specific Notes

### Testing Strategy
- Currently no unit tests exist - write tests for new code
- Focus on testing domain logic (Buffer, Cursor, Operations)
- Use integration tests for application layer

### GPU/Rendering
- Rendering code lives in `renderer/` and `state/` modules
- Uses wgpu for graphics
- Font rendering via fontdue crate

### Editor Core
- Buffer uses `ropey` for efficient text storage
- Cursor/Selection in `domain/cursor.rs`
- Undo/Redo in `domain/operations.rs`

---

## 4. Common Tasks

### Adding a New Domain Type
1. Add to `src/domain/` module
2. Export in `src/domain/mod.rs`
3. Write unit tests

### Adding Infrastructure
1. Define trait in `src/ports/`
2. Implement in `src/infrastructure/`
3. Use in application layer via trait

### Adding UI Features
1. Add to `src/interface/` module
2. Coordinate with application services
3. Update state/renderer as needed
