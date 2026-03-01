# AGENTS.md - Guidelines for AI Agents

Guidelines for AI coding agents working in this Rust text editor repository.

## 1. Build, Lint, and Test Commands

### Build
```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo run                # Run debug
cargo run --release      # Run release
```

### Lint
```bash
cargo check              # Fast type checking (use frequently)
cargo clippy             # Lint warnings
cargo fmt                # Format code
cargo fmt -- --check     # Check formatting only
```

### Test
```bash
cargo test                                    # Run all tests (142 tests)
cargo test test_buffer_insert                 # Run single test by name
cargo test buffer::tests::                    # Run tests in a module
cargo test -- --nocapture                     # Show println! output
cargo test --doc                              # Run doc tests only
```

## 2. Code Style Guidelines

### Import Order
1. Standard library (`std::`, `core::`)
2. External crates (alphabetical)
3. Local imports (`crate::`, `super::`)

```rust
use std::collections::HashMap;
use std::sync::Arc;

use wgpu::{Device, Queue};
use winit::event::WindowEvent;

use crate::domain::Buffer;
use crate::error::Result;
```

### Naming Conventions
- **Functions/variables**: `snake_case`
- **Types/traits**: `PascalCase`
- **Modules**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Unused variables**: prefix with `_`

### Error Handling
Use `thiserror` for error types. Prefer `crate::error::Result<T>` over `anyhow::Result`:

```rust
use crate::error::{EditorError, Result};

fn load_file(path: &str) -> Result<String> {
    std::fs::read_to_string(path).map_err(EditorError::IoError)
}
```

**Rules:**
- Use `?` for error propagation
- Avoid `unwrap()`/`expect()` in production code
- Don't silently ignore errors with `let _ = ...`

### Types and Ownership
- Prefer `&str` over `&String` for parameters
- Use `Arc<T>` for shared ownership across threads
- Prefer `&mut T` over `RefCell<T>`
- Avoid excessive `.clone()` - use references where possible

### Common Derives
```rust
#[derive(Debug, Clone, Copy, PartialEq)]  // For simple value types
#[derive(Debug, Clone)]                    // For complex types
```

### Documentation
Document public APIs with `///` comments. Skip obvious getters/setters.

## 3. Architecture

Clean Architecture with dependency rules:

```
src/
├── domain/         # Pure business logic (Buffer, Cursor, Operations)
│                   # NO external dependencies except std
├── ports/          # Trait definitions (interfaces)
├── application/    # Use cases (EditorService, FileService)
├── infrastructure/ # External implementations
├── interface/      # Event handling (App, KeyboardController)
├── ui/             # UI primitives, widgets, modals
│   ├── primitives.rs  # Point, Rect, Color, Primitive enum
│   ├── widget.rs      # Widget trait
│   ├── widgets/       # Button, Input, Label
│   ├── modal/         # FindModal, InputField
│   └── layers.rs      # Z-index layer management
├── renderer/       # GPU rendering (wgpu)
│   ├── layout.rs      # EditorLayout, Rect, Colors
│   ├── glyph_cache.rs # Font atlas (fontdue)
│   └── modal/         # Modal geometry builders
├── state/          # GPU state management
├── menu/           # Native menu bar (muda)
├── syntax.rs       # Syntax highlighting (syntect)
└── error.rs        # EditorError enum
```

**Dependency Rules:**
- Domain → std only
- Application → Domain + Ports
- Interface/UI → Application + Domain
- Renderer/State → All layers (GPU boundary)

## 4. Key Patterns

### Buffer (ropey)
```rust
let buffer = Buffer::from_str("hello");
buffer.insert(5, " world");
let (line, col) = buffer.char_to_line_col(char_idx);
```

### UI Primitives
Widgets produce primitives; renderer converts to GPU geometry:
```rust
pub enum Primitive {
    Rect { rect: Rect, color: Color },
    RoundedRect { rect: Rect, color: Color, radius: f32 },
    Text { position: Point, text: String, color: Color, size: f32 },
    // ...
}
```

### Modal Hit Testing
Button/input regions stored in State for mouse click handling:
```rust
// In State struct
pub modal_button_regions: Vec<(FindButton, Rect)>,
pub modal_input_regions: Vec<(FindField, Rect)>,
```

### Event Flow
1. `winit` events → `App::window_event()`
2. Modal checks first (hit test buttons/inputs)
3. Keyboard → `KeyboardController` → `EditorService`
4. Updates → `State::update_geometry()` → GPU buffers

## 5. Testing

Tests live alongside code in `#[cfg(test)]` modules:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_insert() {
        let mut buffer = Buffer::new();
        buffer.insert(0, "hello");
        assert_eq!(buffer.len_chars(), 5);
    }
}
```

**Focus areas:**
- Domain logic (Buffer, Cursor, Operations)
- Widget behavior (Input, Button)
- Modal state transitions

## 6. Common Tasks

### Adding a Widget
1. Create `src/ui/widgets/my_widget.rs`
2. Implement `Widget` trait
3. Export in `src/ui/widgets/mod.rs`
4. Add tests

### Adding Modal Functionality
1. Update `FindModal` in `src/ui/modal/find_modal.rs`
2. Add button to `FindButton` enum if needed
3. Update geometry in `src/renderer/modal/find_geometry.rs`
4. Handle in `App::handle_modal_action()`

### Debugging Rendering
- Check `update_geometry()` is called after state changes
- Verify buffer counts: `modal_bg_index_count`, etc.
- Use `cargo run` and Ctrl+F to test modal
