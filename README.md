# textedit — A GPU-Accelerated Text Editor in Rust

A minimalist, high-performance text editor built with **wgpu** for GPU-accelerated rendering, **winit** for cross-platform windowing, and Clean Architecture principles.

## Architecture

```
src/
├── main.rs              # Entry point
├── config.rs            # Editor configuration
├── error.rs             # EditorError enum (thiserror)
├── syntax.rs            # Syntax highlighting (syntect)
├── domain/              # Pure business logic (no external deps)
│   ├── buffer.rs        # Rope-backed text buffer
│   ├── cursor.rs        # Cursor & selection
│   ├── position.rs      # Line/column position
│   └── operations.rs    # Undo/redo history
├── ports/               # Trait definitions
│   ├── clipboard_port.rs
│   ├── file_port.rs
│   ├── render_port.rs
│   └── window_port.rs
├── application/         # Use cases
│   ├── editor_service.rs
│   └── file_service.rs
├── infrastructure/      # Adapter implementations
│   ├── clipboard.rs
│   └── file_system.rs
├── interface/           # Event handling (winit)
│   ├── app.rs
│   └── keyboard.rs
├── ui/                  # UI primitives, widgets, modals
│   ├── primitives.rs    # Point, Rect, Color, Primitive enum
│   ├── widget.rs        # Widget trait
│   ├── layers.rs        # Z-index layer management
│   ├── components.rs    # Status bar, line numbers
│   ├── layout.rs        # UI layout
│   ├── event_router.rs  # Mouse/keyboard routing
│   ├── widgets/         # Button, Input, Label
│   └── modal/           # FindModal, InputField
├── renderer/            # GPU rendering (wgpu, fontdue)
│   ├── layout.rs        # EditorLayout, Colors
│   ├── glyph_cache.rs   # Font atlas
│   ├── text.rs          # Text layout
│   ├── text_geometry.rs # Text quads
│   ├── cursor.rs        # Cursor geometry
│   ├── scrollbar.rs     # Scrollbar geometry
│   ├── line_numbers.rs  # Line number geometry
│   ├── status_bar.rs    # Status bar geometry
│   ├── ui_background.rs # Background geometry
│   ├── primitive_builders.rs
│   ├── primitive_renderer.rs  # Primitive-to-GPU conversion
│   └── modal/           # Modal geometry builders
├── state/               # GPU state management
│   ├── init.rs          # State struct, wgpu init
│   ├── font.rs          # System font loading
│   ├── render.rs        # Render pass
│   ├── scroll.rs        # Scrolling state
│   └── geometry/        # Buffer updates
├── menu/                # Native menu bar (muda)
│   ├── actions.rs       # MenuAction enum
│   ├── handler.rs       # Menu building
│   ├── helpers.rs       # Menu item helpers
│   └── platform.rs      # Platform menu setup
└── themes/              # Editor themes
    ├── mod.rs           # EditorTheme enum
    ├── dark.rs          # Dark theme
    └── light.rs         # Light theme
```

## Features

### Editing
- Insert, delete, backspace with full Unicode support
- Tab (4-space indentation), Enter for newlines
- Arrow key navigation, Home/End
- Selection management (Shift+Arrow)
- **Undo/Redo** (Ctrl+Z / Ctrl+Y)
- **Cut/Copy/Paste** (Ctrl+X / Ctrl+C / Ctrl+V)

### File Operations
- **Open/Save** via native file dialogs
- UTF-8 encoding
- Multi-file support (new/close without quitting)
- File modification detection

### Find & Replace
- **Find** (Ctrl+F) with match highlighting
- **Find Next/Previous** (Enter/Shift+Enter, Ctrl+G/Ctrl+Shift+G)
- **Replace** (Ctrl+H): replace one or all matches
- Match count display

### Syntax Highlighting
- Language detection via file extension
- Multiple themes (syntect)
- GPU-optimized highlight caching

### UI
- Dark & Light editor themes
- Toggle line numbers / status bar
- Native menu bar (macOS/Windows/Linux)
- GPU-accelerated rendering via wgpu
- Resizable window

## Quick Start

```bash
# Build & run (debug)
cargo run

# Build & run (release)
cargo run --release

# Run tests
cargo test

# Check & lint
cargo check
cargo clippy
cargo fmt
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Z` | Undo |
| `Ctrl+Y` / `Ctrl+Shift+Z` | Redo |
| `Ctrl+X` | Cut |
| `Ctrl+C` | Copy |
| `Ctrl+V` | Paste |
| `Ctrl+S` | Save |
| `Ctrl+O` | Open |
| `Ctrl+N` | New |
| `Ctrl+W` | Close |
| `Ctrl+F` | Find |
| `Ctrl+G` | Find Next |
| `Ctrl+Shift+G` | Find Previous |
| `Ctrl+H` | Replace |
| `Ctrl+A` | Select All |
| `Delete` | Delete |
| `Home` | Line start |
| `End` | Line end |
| `←` `→` `↑` `↓` | Move cursor |
| `Shift+Arrow` | Extend selection |

## Stack

- **wgpu 0.20** — GPU rendering (Vulkan, Metal, DX12, WebGPU)
- **winit 0.30** — Windowing & input
- **ropey 1.6** — Rope data structure for text buffer
- **fontdue 0.7** — Font rasterization
- **syntect** — Syntax highlighting
- **muda** — Native menu bar
- **arboard** — Clipboard
- **rfd** — Native file dialogs
