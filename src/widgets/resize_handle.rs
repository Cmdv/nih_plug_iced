//! A resize handle for uniformly scaling a plugin GUI.

use crate::core::event::Event;
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget::{tree, Tree};
use crate::core::{
    Border, Clipboard, Color, Element, Length, Point, Rectangle, Shadow, Shell, Size, Vector, Widget,
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
    /// The last cursor position (used to calculate delta between frames)
    last_cursor: Point,
    /// The accumulated size from the start
    accumulated_size: Size,
    /// The last size we emitted to prevent duplicate messages
    last_emitted_size: Size,
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
            on_resize: Box::new(on_resize),
        }
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
                        state.last_cursor = cursor_position;
                        state.accumulated_size = self.current_size;
                        state.last_emitted_size = self.current_size;
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
                    if let Some(cursor_position) = cursor.position() {
                        // Calculate delta from LAST cursor position (incremental)
                        // This avoids coordinate space issues when window resizes
                        let delta = Vector::new(
                            cursor_position.x - state.last_cursor.x,
                            cursor_position.y - state.last_cursor.y,
                        );

                        // Update last cursor position for next frame
                        state.last_cursor = cursor_position;

                        // Accumulate the delta into our size
                        state.accumulated_size.width = (state.accumulated_size.width + delta.x).max(self.min_width);
                        state.accumulated_size.height = (state.accumulated_size.height + delta.y).max(self.min_height);

                        // Only emit if the size actually changed to reduce message spam
                        if state.accumulated_size != state.last_emitted_size {
                            nih_plug::nih_log!(
                                "ResizeHandle: cursor: ({}, {}), delta: ({}, {}), bounds: ({}, {}), accumulated size: {}x{}",
                                cursor_position.x, cursor_position.y,
                                delta.x, delta.y,
                                bounds.x, bounds.y,
                                state.accumulated_size.width, state.accumulated_size.height
                            );

                            state.last_emitted_size = state.accumulated_size;
                            // Emit the resize message
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

        // Draw a simple triangle in the bottom-right corner
        // Points: bottom-left, bottom-right, top-right (forming a right-angled triangle)
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                shadow: Shadow::default(),
                ..Default::default()
            },
            self.color,
        );
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if let Some(cursor_position) = cursor.position() {
            // TODO: Use triangle intersection when we draw actual triangle
            if layout.bounds().contains(cursor_position) {
                return mouse::Interaction::Grabbing;
            }
        }

        mouse::Interaction::default()
    }
}

impl<'a, Message, Theme, Renderer> From<ResizeHandle<Message>> for Element<'a, Message, Theme, Renderer>
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

/// Test whether a point intersects with the triangle of this resize handle.
///
/// The triangle is formed by three points:
/// - Bottom-left corner of the bounds
/// - Bottom-right corner of the bounds
/// - Top-right corner of the bounds
///
/// This creates a right-angled triangle in the bottom-right corner.
fn intersects_triangle(bounds: Rectangle, point: Point) -> bool {
    // We use the determinant method (cross product) to check if the point is on the correct side
    // of each edge of the triangle. For a point to be inside, it must be on the right side of all
    // edges when traversed clockwise.

    // Triangle vertices (clockwise from bottom-left)
    let p1 = Point::new(bounds.x, bounds.y + bounds.height); // Bottom-left
    let p2 = Point::new(bounds.x + bounds.width, bounds.y + bounds.height); // Bottom-right
    let p3 = Point::new(bounds.x + bounds.width, bounds.y); // Top-right

    // Edge from p1 to p2 (bottom edge)
    let v1 = Vector::new(p2.x - p1.x, p2.y - p1.y);
    let to_point1 = Vector::new(point.x - p1.x, point.y - p1.y);
    let cross1 = v1.x * to_point1.y - v1.y * to_point1.x;

    // Edge from p2 to p3 (right edge)
    let v2 = Vector::new(p3.x - p2.x, p3.y - p2.y);
    let to_point2 = Vector::new(point.x - p2.x, point.y - p2.y);
    let cross2 = v2.x * to_point2.y - v2.y * to_point2.x;

    // Edge from p3 to p1 (diagonal edge)
    let v3 = Vector::new(p1.x - p3.x, p1.y - p3.y);
    let to_point3 = Vector::new(point.x - p3.x, point.y - p3.y);
    let cross3 = v3.x * to_point3.y - v3.y * to_point3.x;

    // Point is inside if all cross products have the same sign (all >= 0 for clockwise winding)
    cross1 >= 0.0 && cross2 >= 0.0 && cross3 >= 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_intersection() {
        let bounds = Rectangle {
            x: 10.0,
            y: 10.0,
            width: 10.0,
            height: 10.0,
        };

        // Corners
        assert!(!intersects_triangle(bounds, Point::new(10.0, 10.0))); // Top-left (outside)
        assert!(intersects_triangle(bounds, Point::new(20.0, 10.0))); // Top-right (vertex)
        assert!(intersects_triangle(bounds, Point::new(10.0, 20.0))); // Bottom-left (vertex)
        assert!(intersects_triangle(bounds, Point::new(20.0, 20.0))); // Bottom-right (vertex)

        // Inside the triangle
        assert!(intersects_triangle(bounds, Point::new(15.0, 15.0)));

        // Outside the triangle (top-left region)
        assert!(!intersects_triangle(bounds, Point::new(14.9, 15.0)));
        assert!(!intersects_triangle(bounds, Point::new(15.0, 14.9)));
    }
}
