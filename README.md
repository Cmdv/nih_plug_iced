# NIH-plug: Iced 0.14 + resize_window + process_stopped

This is a standalone modernized port of [nih_plug_iced](https://github.com/robbert-vdh/nih-plug/tree/master/nih_plug_iced)
from NIH-plug, updated to work with [Iced 0.14](https://github.com/iced-rs/iced) and featuring
full window resize support for audio plugin GUIs.

## Key Changes from Original

This port modernizes the original crate with several improvements:

- **Updated to Iced 0.14**: Uses the modern wgpu-based rendering pipeline (OpenGL support removed)
- **Inlined iced_baseview**: [BillyDM's iced_baseview](https://github.com/BillyDM/iced_baseview) has been integrated directly into this crate to allow rapid iteration on core resize logic without managing multiple crate dependencies. Credit to BillyDM for the original iced_baseview implementation. Once stabilized, these changes will be contributed back via PRs.
- **Window Resize Support**: Full support for resizable plugin windows with custom resize handles
- **Enhanced widget support**: Added new feature flags for modern Iced widgets including a custom resize handle widget
- **Process Stopped Callback**: Integration with forked nih_plug to handle DAW bypass/disable events gracefully

## Features

By default this uses wgpu rendering (the modern Iced 0.14 approach):

```toml
[features]
default = ["wgpu"]

# Core rendering
wgpu = ["iced_renderer/wgpu", "iced_widget/wgpu"]  # Modern GPU rendering (default)

# Widgets and capabilities
debug = ["toggle_debug"]               # Debug view (F12 in native platforms)
image = ["iced_graphics/image", "iced_widget/image", "iced_renderer/image"]
svg = ["iced_graphics/svg", "iced_widget/svg", "iced_renderer/svg"]
canvas = ["iced_widget/canvas"]        # Canvas widget for custom drawing
geometry = ["iced_graphics/geometry", "iced_renderer/geometry"]
web-colors = ["iced_graphics/web-colors", "iced_renderer/web-colors"]
system = ["dep:sysinfo"]               # System information support
```

## Forked Dependencies

This crate relies on several forked dependencies with custom features:

### nih_plug ([PR #231](https://github.com/robbert-vdh/nih-plug/pull/231))
- **Branch**: `expose-process-stopped`
- **Purpose**: Adds a callback for when the DAW bypasses or turns off the plugin
- **Benefits**: Enables graceful shutdown animations (e.g., spectrum analyzer decay) and UI state updates when processing stops

### baseview ([PR #211](https://github.com/RustAudio/baseview/pull/211))
- **Branch**: `resize-window`
- **Purpose**: Adds screen-absolute mouse position support for window resize operations
- **Benefits**: Provides reliable mouse coordinate tracking during window geometry changes, essential for custom resize handles

## Window Resize Support

This crate includes a custom `resize_handle` widget that enables draggable window resizing in audio plugin GUIs:

```rust
use nih_plug_iced::widgets::resize_handle;

// Add a resize handle to your UI
resize_handle::ResizeHandle::new(state)
    .on_resize(Message::WindowResize)
```

The resize system works by:
1. Capturing screen-absolute mouse coordinates during drag operations
2. Calculating delta movements that remain stable even as window geometry changes
3. Propagating resize events through the iced rendering pipeline
4. Updating the baseview window size in real-time

This approach solves the common problem where window-relative coordinates become unreliable during resize operations.

### Removed Features

The following features from the original are no longer available due to the modernization:
- OpenGL/glow rendering (wgpu-only for modern Iced 0.14)
- Async runtime features (palette, tokio, async-std)
- QR code support

## Usage

Include in your `Cargo.toml`:

```toml
nih_plug_iced = { git = "https://github.com/Cmdv/nih_plug_iced" }
```

Or with specific features:

```toml
nih_plug_iced = { git = "https://github.com/Cmdv/nih_plug_iced", features = ["canvas", "image"] }
```
