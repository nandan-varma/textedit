# AGENTS.md - TextEdit Development Guide

This document provides guidelines for agents working on the TextEdit codebase.

## Project Overview

- **Name**: TextEdit
- **Identifier**: com.nandanvarma.textedit
- **Type**: Cross-platform text editor (Rust/egui)
- **Edition**: Rust 2024
- **Minimum Rust Version**: 1.88

## Build, Test, and Lint Commands

### Building
```bash
# Development build
cargo build

# Release build
cargo build --release

# Build for web (WASM)
cargo build --target wasm32-unknown-unknown
trunk serve  # Development server at http://127.0.0.1:8080
trunk build --release  # Production build to dist/
```

### Testing
```bash
# Run all tests
cargo test

# Run a single test file
cargo test --lib editor::buffer
cargo test --lib features::find

# Run a specific test
cargo test test_insert
cargo test -- --test-threads=1  # Run tests serially
```

### Linting
```bash
# Run clippy lints
cargo clippy

# Fix auto-fixable issues
cargo clippy --fix --allow-dirty

# Check formatting
cargo fmt --check

# Format code
cargo fmt
```

## Code Style Guidelines

### General Principles
- Follow Rust idioms and best practices
- Use the 2024 edition features where appropriate
- Keep functions small and focused
- Prefer explicit over implicit
- No comments unless explaining complex business logic (per user request)

### Naming Conventions
- **Types/Structs**: `PascalCase` (e.g., `TextBuffer`, `Document`)
- **Functions/Methods**: `snake_case` (e.g., `get_text()`, `save_to_file()`)
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case` (e.g., `editor`, `features`)
- **Fields**: `snake_case`
- **Traits**: `PascalCase` with -able suffix when appropriate (e.g., `Serialize`)

### Imports
```rust
// Order: standard library → external crates → local modules
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::editor::{Document, TextBuffer};
use crate::features::FindReplace;
use crate::ui::MenuBarState;
```

### Structs and Enums
```rust
// Use derive macros for common traits
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MyStruct {
    pub public_field: Type,
    private_field: Type,
}

// Skip fields that shouldn't be serialized
#[derive(Serialize, Deserialize)]
pub struct Document {
    pub buffer: TextBuffer,
    #[serde(skip)]
    internal_state: InternalType,
}

// Use Copy where appropriate for small types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LineEnding {
    #[default]
    Crlf,
    Lf,
    Cr,
}
```

### Error Handling
```rust
// Use Result for fallible operations
pub fn load_from_file(&mut self, path: &PathBuf) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    // ... processing
    Ok(())
}

// Use ? operator when appropriate
fn process() -> Result<Value, Error> {
    let data = read_file()?;  // early return on error
    parse(data)?
}
```

### Visibility and Privacy
```rust
// Default to private, expose only what's needed
pub struct TabBar {
    tabs: Vec<Tab>,  // private - use getters/setters
    pub active_tab: Option<usize>,  // public when needed
}

// Provide accessor methods
impl TabBar {
    pub fn get_active_document(&self) -> Option<Arc<RwLock<Document>>> {
        self.active_tab.and_then(|i| self.tabs.get(i).map(|t| t.document.clone()))
    }

    pub fn get_tab(&self, index: usize) -> Option<&Tab> {
        self.tabs.get(index)
    }

    pub fn get_tab_mut(&mut self, index: usize) -> Option<&mut Tab> {
        self.tabs.get_mut(index)
    }
}
```

### Async and Concurrency
```rust
// Use parking_lot for mutexes (faster than std::sync)
use parking_lot::RwLock;

pub struct EditorState {
    document: Arc<RwLock<Document>>,
}

// Reading - multiple allowed
let doc = self.document.read();
let text = doc.buffer.get_text();

// Writing - exclusive
let mut doc = self.document.write();
doc.save_to_file(path)?;
```

### Platform-Specific Code
```rust
// Use cfg attributes for platform-specific code
#[cfg(not(target_arch = "wasm32"))]
pub fn show_open_dialog() -> Option<PathBuf> {
    rfd::FileDialog::new().pick_file()
}

#[cfg(target_arch = "wasm32")]
pub fn show_open_dialog() -> Option<PathBuf> {
    None  // Web doesn't support native dialogs
}
```

### Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Test implementation
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_with_setup() {
        let mut obj = MyStruct::new();
        // ... test
    }
}
```

### Egui/eframe Specific
```rust
// Widget trait implementation
impl Widget for StatusBarWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        TopBottomPanel::bottom("status_bar")
            .exact_height(24.0)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("text");
                });
            }).response
    }
}

// App trait implementation
impl eframe::App for TextEditApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // UI code
        });
    }
}
```

### Dependencies

#### Adding New Dependencies
```bash
# Add a regular dependency
cargo add <crate>

# Add a dev dependency
cargo add <crate> --dev

# Add with specific features
cargo add serde --features derive
```

#### Version Guidelines
- Use specific versions for main dependencies (e.g., `egui = "0.33.0"`)
- Use `*` or ranges cautiously
- Pin critical dependencies

### Module Structure
```
src/
├── main.rs          # Entry point (platform-specific setup)
├── lib.rs           # Public API exports
├── app.rs           # Main application state and eframe App impl
├── editor/          # Core editor functionality
│   ├── mod.rs       # Module exports
│   ├── buffer.rs    # Text buffer implementation
│   ├── cursor.rs    # Cursor/selection handling
│   ├── document.rs  # Document management and file I/O
│   └── panel.rs     # Editor panel widget
├── ui/              # UI components
│   ├── mod.rs
│   ├── menu.rs      # Menu bar
│   ├── tabs.rs      # Tab bar for multiple documents
│   └── statusbar.rs # Status bar
├── features/        # Feature modules
│   ├── mod.rs
│   ├── find.rs      # Find and replace
│   ├── theme.rs     # Theme management
│   ├── undo.rs      # Undo/redo system
│   └── settings.rs  # Application settings
└── platform/        # Platform-specific code
    └── mod.rs       # Native/web abstractions
```

## Key Dependencies

- **egui/eframe**: UI framework
- **ropey**: Text buffer (optional, currently using String)
- **encoding_rs**: Character encoding detection/conversion
- **parking_lot**: Fast mutex primitives
- **serde**: Serialization
- **uuid**: Unique identifiers
- **rfd**: Native file dialogs (desktop only)
- **regex**: Pattern matching for find/replace

## Git Conventions

- Commit messages should be descriptive
- Run tests before committing
- Use `cargo fmt` before committing
- Reference issues in commit messages when applicable
