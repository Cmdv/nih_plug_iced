# NIH-plug: iced support (Modernized Port)

This is a modernized port of the [nih_plug_iced](https://github.com/robbert-vdh/nih-plug/tree/master/nih_plug_iced)
internal crate from NIH-plug, updated to work with [Iced 0.13](https://github.com/iced-rs/iced) through
[BillyDM's iced_baseview](https://github.com/BillyDM/iced_baseview).

## Key Changes from Original

This port modernizes the original crate with several improvements:

- **Updated to Iced 0.13**: Uses the modern wgpu-based rendering pipeline (OpenGL support removed)
- **Switched to BillyDM's iced_baseview**: The original used robbert-vdh's fork, this uses BillyDM's actively maintained version
- **Enhanced widget support**: Added new feature flags for modern Iced widgets

## Features

By default this uses wgpu rendering (the modern Iced 0.13 approach):

```toml
[features]
default = ["wgpu"]

# Core rendering
wgpu = ["iced_baseview/wgpu"]          # Modern GPU rendering (default)

# Widgets and capabilities
debug = ["iced_baseview/debug"]        # Debug view (F12 in native platforms)
image = ["iced_baseview/image"]        # Image widget support
svg = ["iced_baseview/svg"]            # SVG widget support
canvas = ["iced_baseview/canvas"]      # Canvas widget for custom drawing
geometry = ["iced_baseview/geometry"]  # Geometry rendering support
web-colors = ["iced_baseview/web-colors"] # Web color names support
```

### Removed Features

The following features from the original are no longer available due to the modernization:
- OpenGL/glow rendering (BillyDM's iced_baseview is wgpu-only)
- Async runtime features (palette, tokio, async-std, smol)
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
