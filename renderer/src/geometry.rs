mod cache;

pub use cache::Cache;

use crate::core::{Point, Rectangle, Size, Vector};
use crate::graphics::geometry::{Fill, Path, Stroke, Text};
use crate::Renderer;

pub enum Frame {
    #[cfg(feature = "tiny_skia")]
    TinySkia(iced_tiny_skia::geometry::Frame),
    Wgpu(iced_wgpu::geometry::Frame),
}

pub enum Geometry {
    #[cfg(feature = "tiny_skia")]
    TinySkia(iced_tiny_skia::Primitive),
    Wgpu(iced_wgpu::Primitive),
}

macro_rules! delegate {
    ($frame:expr, $name:ident, $body:expr) => {
        match $frame {
            #[cfg(feature = "tiny_skia")]
            Self::TinySkia($name) => $body,
            Self::Wgpu($name) => $body,
        }
    };
}

impl Frame {
    pub fn new<Theme>(renderer: &Renderer<Theme>, size: Size) -> Self {
        match renderer {
            #[cfg(feature = "tiny_skia")]
            Renderer::TinySkia(_) => {
                Frame::TinySkia(iced_tiny_skia::geometry::Frame::new(size))
            }
            Renderer::Wgpu(_) => {
                Frame::Wgpu(iced_wgpu::geometry::Frame::new(size))
            }
        }
    }

    /// Returns the width of the [`Frame`].
    #[inline]
    pub fn width(&self) -> f32 {
        delegate!(self, frame, frame.width())
    }

    /// Returns the height of the [`Frame`].
    #[inline]
    pub fn height(&self) -> f32 {
        delegate!(self, frame, frame.height())
    }

    /// Returns the dimensions of the [`Frame`].
    #[inline]
    pub fn size(&self) -> Size {
        delegate!(self, frame, frame.size())
    }

    /// Returns the coordinate of the center of the [`Frame`].
    #[inline]
    pub fn center(&self) -> Point {
        delegate!(self, frame, frame.center())
    }

    /// Draws the given [`Path`] on the [`Frame`] by filling it with the
    /// provided style.
    pub fn fill(&mut self, path: &Path, fill: impl Into<Fill>) {
        delegate!(self, frame, frame.fill(path, fill));
    }

    /// Draws an axis-aligned rectangle given its top-left corner coordinate and
    /// its `Size` on the [`Frame`] by filling it with the provided style.
    pub fn fill_rectangle(
        &mut self,
        top_left: Point,
        size: Size,
        fill: impl Into<Fill>,
    ) {
        delegate!(self, frame, frame.fill_rectangle(top_left, size, fill));
    }

    /// Draws the stroke of the given [`Path`] on the [`Frame`] with the
    /// provided style.
    pub fn stroke<'a>(&mut self, path: &Path, stroke: impl Into<Stroke<'a>>) {
        delegate!(self, frame, frame.stroke(path, stroke));
    }

    /// Draws the characters of the given [`Text`] on the [`Frame`], filling
    /// them with the given color.
    ///
    /// __Warning:__ Text currently does not work well with rotations and scale
    /// transforms! The position will be correctly transformed, but the
    /// resulting glyphs will not be rotated or scaled properly.
    ///
    /// Additionally, all text will be rendered on top of all the layers of
    /// a `Canvas`. Therefore, it is currently only meant to be used for
    /// overlays, which is the most common use case.
    ///
    /// Support for vectorial text is planned, and should address all these
    /// limitations.
    pub fn fill_text(&mut self, text: impl Into<Text>) {
        delegate!(self, frame, frame.fill_text(text));
    }

    /// Stores the current transform of the [`Frame`] and executes the given
    /// drawing operations, restoring the transform afterwards.
    ///
    /// This method is useful to compose transforms and perform drawing
    /// operations in different coordinate systems.
    #[inline]
    pub fn with_save(&mut self, f: impl FnOnce(&mut Frame)) {
        delegate!(self, frame, frame.push_transform());

        f(self);

        delegate!(self, frame, frame.pop_transform());
    }

    /// Executes the given drawing operations within a [`Rectangle`] region,
    /// clipping any geometry that overflows its bounds. Any transformations
    /// performed are local to the provided closure.
    ///
    /// This method is useful to perform drawing operations that need to be
    /// clipped.
    #[inline]
    pub fn with_clip(&mut self, region: Rectangle, f: impl FnOnce(&mut Frame)) {
        let mut frame = match self {
            #[cfg(feature = "tiny_skia")]
            Self::TinySkia(_) => Self::TinySkia(
                iced_tiny_skia::geometry::Frame::new(region.size()),
            ),
            Self::Wgpu(_) => {
                Self::Wgpu(iced_wgpu::geometry::Frame::new(region.size()))
            }
        };

        f(&mut frame);

        let origin = Point::new(region.x, region.y);

        match (self, frame) {
            #[cfg(feature = "tiny_skia")]
            (Self::TinySkia(target), Self::TinySkia(frame)) => {
                target.clip(frame, origin);
            }
            (Self::Wgpu(target), Self::Wgpu(frame)) => {
                target.clip(frame, origin);
            }
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        };
    }

    /// Applies a translation to the current transform of the [`Frame`].
    #[inline]
    pub fn translate(&mut self, translation: Vector) {
        delegate!(self, frame, frame.translate(translation));
    }

    /// Applies a rotation in radians to the current transform of the [`Frame`].
    #[inline]
    pub fn rotate(&mut self, angle: f32) {
        delegate!(self, frame, frame.rotate(angle));
    }

    /// Applies a uniform scaling to the current transform of the [`Frame`].
    #[inline]
    pub fn scale(&mut self, scale: impl Into<f32>) {
        delegate!(self, frame, frame.scale(scale));
    }

    /// Applies a non-uniform scaling to the current transform of the [`Frame`].
    #[inline]
    pub fn scale_nonuniform(&mut self, scale: impl Into<Vector>) {
        delegate!(self, frame, frame.scale_nonuniform(scale));
    }

    pub fn into_geometry(self) -> Geometry {
        match self {
            #[cfg(feature = "tiny_skia")]
            Self::TinySkia(frame) => Geometry::TinySkia(frame.into_primitive()),
            Self::Wgpu(frame) => Geometry::Wgpu(frame.into_primitive()),
        }
    }
}
