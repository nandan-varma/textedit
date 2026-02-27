# textedit - A High-Performance Text Editor Built with Rust, wgpu, and Winit

A minimalist, high-performance text editor similar to Notepad++ with a foundation ready for future expansion. Built entirely with hand-rolled UI using **wgpu** for absolute performance and cross-platform support.

## Architecture Overview

### Core Stack
- **wgpu 0.20**: GPU rendering backend with multi-platform support (Vulkan, Metal, DX12, WebGPU)
- **winit 0.30**: Cross-platform windowing and event handling (Windows, macOS, Linux)
- **ropey 1.6**: Efficient rope data structure for text buffer operations
- **fontdue 0.7**: Fast font rasterization for glyph rendering
- **arboard 3.3**: Cross-platform clipboard support

### Project Structure

```
src/
├── main.rs              # Application entry point
├── app.rs               # ApplicationHandler (winit event loop integration)
├── state.rs             # wgpu State (device, queue, surface)
├── config.rs            # Configuration (theme, fonts)
├── file.rs              # File I/O operations
├── editor/
│   ├── mod.rs           # Editor orchestrator
│   ├── buffer.rs        # Text buffer (ropey wrapper)
│   ├── cursor.rs        # Cursor & selection management
│   └── operations.rs    # Undo/redo operation history
├── renderer/
│   ├── mod.rs
│   ├── text.rs          # Text layout & rendering
│   └── glyph_cache.rs   # Glyph caching (foundation)
└── ui/
    ├── mod.rs
    ├── components.rs    # Status bar, line numbers
    └── layout.rs        # UI positioning
```

## MVP Features

### Text Editing
- ✅ Character insertion with full Unicode support
- ✅ Backspace and Delete keys
- ✅ Tab support (4-space indentation)
- ✅ Enter for newlines
- ✅ Arrow key navigation (Left, Right, Up, Down)
- ✅ Home and End key navigation
- ✅ Selection management

### File Operations
- ✅ Open and save text files
- ✅ UTF-8 encoding support
- ✅ File path tracking

### Editing Features
- ✅ **Undo/Redo** (Ctrl+Z / Ctrl+Y)
- ✅ **Cut/Copy/Paste** (Ctrl+X / Ctrl+C / Ctrl+V)
- ✅ **Save** (Ctrl+S)
- ✅ Operation history with efficient memory management

### UI Components
- ✅ Dark theme by default
- ✅ Status bar foundation
- ✅ Line numbers foundation
- ✅ Window management and resizing

## Building & Running

### Prerequisites
- Rust 1.70+ (with 2021 edition support)
- Cargo

### Build
```bash
cargo build --release
```

### Run
```bash
./target/release/textedit
```

### Development
```bash
cargo run
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `Ctrl+C` | Copy selection |
| `Ctrl+X` | Cut selection |
| `Ctrl+V` | Paste |
| `Ctrl+S` | Save file |
| `Arrow Keys` | Move cursor |
| `Home` / `End` | Move to line start/end |
| `Backspace` | Delete previous character |
| `Delete` | Delete current character |
| `Tab` | Insert 4 spaces |
| `Enter` | New line |

## Future Roadmap

### Phase 2: Rendering & UI Polish
- [ ] Proper text rendering with glyph atlas and batching
- [ ] Syntax highlighting framework
- [ ] Theme customization (dark/light mode toggle)
- [ ] Custom font support with system font fallback
- [ ] Cursor visibility and blinking animation
- [ ] Selection highlighting with proper colors

### Phase 3: Advanced Features
- [ ] Multi-file tabs/buffers
- [ ] Find & Replace (Ctrl+H)
- [ ] Search highlighting (Ctrl+F)
- [ ] Word wrap toggle
- [ ] Line/column display in status bar
- [ ] File modification indicator

### Phase 4: Developer Experience
- [ ] Settings file (TOML config)
- [ ] Persistent window state
- [ ] Recent files list
- [ ] Configuration documentation
- [ ] Performance profiling & optimization

### Phase 5: Extended Features
- [ ] Minimap
- [ ] Code folding
- [ ] Split view editing
- [ ] Multi-cursor support
- [ ] Macro recording
- [ ] Plugin system foundation

## Architecture Decisions

### Why wgpu?
- **Cross-platform**: Single codebase for Windows, macOS, Linux, and WebGPU (future)
- **Performance**: Direct GPU access without runtime overhead
- **Modern**: Uses latest graphics APIs (Vulkan, Metal, DX12)
- **Battle-tested**: Used in production Rust projects

### Why Rope for Text Buffer?
- **Efficient insertions/deletions**: O(log n) for typical operations
- **Memory efficient**: Doesn't require reallocating entire buffer on edits
- **UTF-8 safe**: ropey handles Unicode grapheme boundaries correctly

### Battle-Tested Dependencies
All dependencies are carefully selected for production use:
- `ropey`: Standard choice for text editors in Rust
- `winit`: Industry standard for Rust GUI windowing
- `wgpu`: WebGPU standard implementation
- `arboard`: Only pure-Rust clipboard solution

## Performance Notes

The foundation is designed for performance:
- Immediate-mode rendering ready (no retained state overhead)
- Lazy glyph caching to avoid font rendering bottlenecks
- Direct GPU rendering pipeline
- No unnecessary allocations in hot paths
- Rope data structure scales to multi-megabyte files

## Contributing

This is the MVP foundation. Areas ready for enhancement:
1. **Text Rendering**: Implement actual glyph atlas rendering
2. **UI Components**: Full implementation of line numbers and status bar
3. **Features**: Syntax highlighting, search, etc.
4. **Optimization**: Profile and optimize hot paths

## License

Check LICENSE files in repository root for licensing information.

---

**Status**: MVP Complete. Ready for phase 2 rendering implementation.
