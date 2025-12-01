//! A resize handle for uniformly scaling a plugin GUI.

use crate::core::event::Event;
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget::{tree, Tree};
use crate::core::{
    Border, Clipboard, Color, Element, Length, Point, Rectangle, Shadow, Shell, Size, Vector,
    Widget,
};

/// A resize handle placed at the bottom right of the window that lets you resize the window.
///
/// This widget should be rendered on top of other UI elements (last in the layout tree) to ensure
/// it receives mouse events properly.
pub struct ResizeHandle<Message> {
    /// The size of the handle in logical pixels
    size: f32,
    /// The color of the triangle
    color: Color,
    /// Minimum window width
    min_width: f32,
    /// Minimum window height
    min_height: f32,
    /// Current window size (needed for drag calculations)
    current_size: Size,
    /// Current screen cursor position (from State)
    screen_cursor: Option<Point>,
    /// Callback to emit the new window size when dragging
    on_resize: Box<dyn Fn(Size) -> Message>,
}

/// Internal state for tracking drag operations
#[derive(Debug, Default)]
struct State {
    /// Whether we're currently dragging
    drag_active: bool,
    /// The window size when we started dragging
    start_size: Size,
    /// The last screen cursor position (used to calculate delta between frames)
    /// Using screen coordinates prevents issues when window resizes change the coordinate space
    last_screen_cursor: Point,
    /// The accumulated size from the start
    accumulated_size: Size,
    /// The last size we emitted to prevent duplicate messages
    last_emitted_size: Size,
    /// Whether we've initialized the last_screen_cursor (to avoid using Point::ORIGIN as sentinel)
    screen_cursor_initialized: bool,
}

impl<Message> ResizeHandle<Message> {
    /// The default size of the resize handle in logical pixels
    const DEFAULT_SIZE: f32 = 20.0;

    /// The default color of the resize handle (semi-transparent gray)
    const DEFAULT_COLOR: Color = Color {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        a: 0.5,
    };

    /// Create a new resize handle.
    ///
    /// # Parameters
    /// - `on_resize`: Callback that receives the new window `Size` when the user drags the handle
    ///
    /// # Example
    /// ```ignore
    /// ResizeHandle::new(|size| Message::ResizeWindow(size))
    /// ```
    pub fn new(current_size: Size, on_resize: impl Fn(Size) -> Message + 'static) -> Self {
        Self {
            size: Self::DEFAULT_SIZE,
            color: Self::DEFAULT_COLOR,
            min_width: 400.0,
            min_height: 300.0,
            current_size,
            screen_cursor: None,
            on_resize: Box::new(on_resize),
        }
    }

    /// Set the current screen cursor position from State
    pub fn screen_cursor(mut self, screen_cursor: Option<Point>) -> Self {
        self.screen_cursor = screen_cursor;
        self
    }

    /// Set the size of the handle in logical pixels (default: 20.0)
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Set the color of the triangle
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set the minimum window size (default: 400x300)
    pub fn min_size(mut self, width: f32, height: f32) -> Self {
        self.min_width = width;
        self.min_height = height;
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for ResizeHandle<Message>
where
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fixed(self.size),
            height: Length::Fixed(self.size),
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(self.size, self.size))
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(cursor_position) = cursor.position() {
                    // Check if cursor is within the bounds (for now, we draw a rectangle)
                    // TODO: Draw actual triangle and use triangle intersection test
                    if bounds.contains(cursor_position) {
                        state.drag_active = true;
                        state.start_size = self.current_size;
                        state.accumulated_size = self.current_size;
                        state.last_emitted_size = self.current_size;
                        state.screen_cursor_initialized = false;
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.drag_active {
                    state.drag_active = false;
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.drag_active {
                    if let Some(screen_position) = self.screen_cursor {
                        // Use screen coordinates for delta calculation
                        // This prevents coordinate space issues when the window resizes during drag

                        // On first move, initialize last_screen_cursor
                        if !state.screen_cursor_initialized {
                            state.last_screen_cursor = screen_position;
                            state.screen_cursor_initialized = true;
                            return; // Skip first frame to avoid false delta
                        }

                        // Calculate delta from LAST screen cursor position (incremental)
                        let delta = Vector::new(
                            screen_position.x - state.last_screen_cursor.x,
                            screen_position.y - state.last_screen_cursor.y,
                        );

                        // Update last screen cursor position for next frame
                        state.last_screen_cursor = screen_position;

                        // Accumulate the delta into our size
                        state.accumulated_size.width =
                            (state.accumulated_size.width + delta.x).max(self.min_width);
                        state.accumulated_size.height =
                            (state.accumulated_size.height + delta.y).max(self.min_height);

                        // Only emit if the size actually changed to reduce message spam
                        if state.accumulated_size != state.last_emitted_size {
                            state.last_emitted_size = state.accumulated_size;
                            shell.publish((self.on_resize)(state.accumulated_size));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        // Draw 6 circles in a diagonal grid pattern
        // Pattern: 1 circle (top), 2 circles (middle), 3 circles (bottom)
        let circle_radius = 2.0;
        let spacing = bounds.width / 3.5; // Space between circles

        // Define circle positions (row, col) in the grid
        let positions = [
            (0, 2), // Top row: 1 circle (right)
            (1, 1), (1, 2), // Middle row: 2 circles
            (2, 0), (2, 1), (2, 2), // Bottom row: 3 circles
        ];

        for (row, col) in positions {
            let x = bounds.x + col as f32 * spacing + spacing * 0.5;
            let y = bounds.y + row as f32 * spacing + spacing * 0.5;

            let circle_bounds = Rectangle {
                x: x - circle_radius,
                y: y - circle_radius,
                width: circle_radius * 2.0,
                height: circle_radius * 2.0,
            };

            renderer.fill_quad(
                renderer::Quad {
                    bounds: circle_bounds,
                    border: Border::default(),
                    shadow: Shadow::default(),
                    ..Default::default()
                },
                self.color,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();

        // If we're actively dragging, always show the resize cursor
        // even if the mouse moves outside the handle bounds
        if state.drag_active {
            return mouse::Interaction::ResizingDiagonallyDown;
        }

        // Otherwise, only show resize cursor when hovering over the handle
        if let Some(cursor_position) = cursor.position() {
            if layout.bounds().contains(cursor_position) {
                return mouse::Interaction::ResizingDiagonallyDown;
            }
        }

        mouse::Interaction::default()
    }
}

impl<'a, Message, Theme, Renderer> From<ResizeHandle<Message>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(handle: ResizeHandle<Message>) -> Self {
        Element::new(handle)
    }
}

/// Helper function to create a resize handle
pub fn resize_handle<Message>(
    current_size: Size,
    on_resize: impl Fn(Size) -> Message + 'static,
) -> ResizeHandle<Message> {
    ResizeHandle::new(current_size, on_resize)
}
