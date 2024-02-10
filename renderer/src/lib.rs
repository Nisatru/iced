#![forbid(rust_2018_idioms)]
#![deny(unsafe_code, unused_results, rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
pub use iced_wgpu as wgpu;

pub mod compositor;

#[cfg(feature = "geometry")]
pub mod geometry;

mod settings;

pub use iced_graphics as graphics;
pub use iced_graphics::core;

pub use compositor::Compositor;
pub use settings::Settings;

#[cfg(feature = "geometry")]
pub use geometry::Geometry;

use crate::core::renderer;
use crate::core::text::{self, Text};
use crate::core::{
    Background, Color, Font, Pixels, Point, Rectangle, Transformation,
};
use crate::graphics::text::Editor;
use crate::graphics::text::Paragraph;
use crate::graphics::Mesh;

use std::borrow::Cow;

/// The default graphics renderer for [`iced`].
///
/// [`iced`]: https://github.com/iced-rs/iced
pub enum Renderer {
    #[cfg(feature = "tiny_skia")]
    TinySkia(iced_tiny_skia::Renderer),
    Wgpu(iced_wgpu::Renderer),
}

macro_rules! delegate {
    ($renderer:expr, $name:ident, $body:expr) => {
        match $renderer {
            #[cfg(feature = "tiny_skia")]
            Self::TinySkia($name) => $body,
            Self::Wgpu($name) => $body,
        }
    };
}

impl Renderer {
    pub fn draw_mesh(&mut self, mesh: Mesh) {
        match self {
            #[cfg(feature = "tiny_skia")]
            Self::TinySkia(_) => {
                log::warn!("Unsupported mesh primitive: {mesh:?}");
            }
            Self::Wgpu(renderer) => {
                renderer.draw_primitive(iced_wgpu::Primitive::Custom(
                    iced_wgpu::primitive::Custom::Mesh(mesh),
                ));
            }
        }
    }
}

impl core::Renderer for Renderer {
    fn with_layer(&mut self, bounds: Rectangle, f: impl FnOnce(&mut Self)) {
        match self {
            #[cfg(feature = "tiny_skia")]
            Self::TinySkia(renderer) => {
                let primitives = renderer.start_layer();

                f(self);

                match self {
                    Self::TinySkia(renderer) => {
                        renderer.end_layer(primitives, bounds);
                    }
                    _ => unreachable!(),
                }
            }
            Self::Wgpu(renderer) => {
                let primitives = renderer.start_layer();

                f(self);

                match self {
                    Self::Wgpu(renderer) => {
                        renderer.end_layer(primitives, bounds);
                    }
                    #[cfg(feature = "tiny_skia")]
                    _ => unreachable!(),
                }
            }
        }
    }

    fn with_transformation(
        &mut self,
        transformation: Transformation,
        f: impl FnOnce(&mut Self),
    ) {
        match self {
            #[cfg(feature = "tiny_skia")]
            Self::TinySkia(renderer) => {
                let primitives = renderer.start_transformation();

                f(self);

                match self {
                    Self::TinySkia(renderer) => {
                        renderer.end_transformation(primitives, transformation);
                    }
                    _ => unreachable!(),
                }
            }
            Self::Wgpu(renderer) => {
                let primitives = renderer.start_transformation();

                f(self);

                match self {
                    Self::Wgpu(renderer) => {
                        renderer.end_transformation(primitives, transformation);
                    }
                    #[cfg(feature = "tiny_skia")]
                    _ => unreachable!(),
                }
            }
        }
    }

    fn fill_quad(
        &mut self,
        quad: renderer::Quad,
        background: impl Into<Background>,
    ) {
        delegate!(self, renderer, renderer.fill_quad(quad, background));
    }

    fn clear(&mut self) {
        delegate!(self, renderer, renderer.clear());
    }
}

impl text::Renderer for Renderer {
    type Font = Font;
    type Paragraph = Paragraph;
    type Editor = Editor;

    const ICON_FONT: Font = iced_wgpu::Renderer::ICON_FONT;
    const CHECKMARK_ICON: char = iced_wgpu::Renderer::CHECKMARK_ICON;
    const ARROW_DOWN_ICON: char = iced_wgpu::Renderer::ARROW_DOWN_ICON;

    fn default_font(&self) -> Self::Font {
        delegate!(self, renderer, renderer.default_font())
    }

    fn default_size(&self) -> Pixels {
        delegate!(self, renderer, renderer.default_size())
    }

    fn load_font(&mut self, bytes: Cow<'static, [u8]>) {
        delegate!(self, renderer, renderer.load_font(bytes));
    }

    fn fill_paragraph(
        &mut self,
        paragraph: &Self::Paragraph,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        delegate!(
            self,
            renderer,
            renderer.fill_paragraph(paragraph, position, color, clip_bounds)
        );
    }

    fn fill_editor(
        &mut self,
        editor: &Self::Editor,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        delegate!(
            self,
            renderer,
            renderer.fill_editor(editor, position, color, clip_bounds)
        );
    }

    fn fill_text(
        &mut self,
        text: Text<'_, Self::Font>,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        delegate!(
            self,
            renderer,
            renderer.fill_text(text, position, color, clip_bounds)
        );
    }
}

#[cfg(feature = "image")]
impl crate::core::image::Renderer for Renderer {
    type Handle = crate::core::image::Handle;

    fn dimensions(
        &self,
        handle: &crate::core::image::Handle,
    ) -> core::Size<u32> {
        delegate!(self, renderer, renderer.dimensions(handle))
    }

    fn draw(
        &mut self,
        handle: crate::core::image::Handle,
        filter_method: crate::core::image::FilterMethod,
        bounds: Rectangle,
    ) {
        delegate!(self, renderer, renderer.draw(handle, filter_method, bounds));
    }
}

#[cfg(feature = "svg")]
impl crate::core::svg::Renderer for Renderer {
    fn dimensions(&self, handle: &crate::core::svg::Handle) -> core::Size<u32> {
        delegate!(self, renderer, renderer.dimensions(handle))
    }

    fn draw(
        &mut self,
        handle: crate::core::svg::Handle,
        color: Option<crate::core::Color>,
        bounds: Rectangle,
    ) {
        delegate!(self, renderer, renderer.draw(handle, color, bounds));
    }
}

#[cfg(feature = "geometry")]
impl crate::graphics::geometry::Renderer for Renderer {
    type Geometry = crate::Geometry;

    fn draw(&mut self, layers: Vec<Self::Geometry>) {
        match self {
            #[cfg(feature = "tiny_skia")]
            Self::TinySkia(renderer) => {
                for layer in layers {
                    match layer {
                        crate::Geometry::TinySkia(primitive) => {
                            renderer.draw_primitive(primitive);
                        }
                        crate::Geometry::Wgpu(_) => unreachable!(),
                    }
                }
            }
            Self::Wgpu(renderer) => {
                for layer in layers {
                    match layer {
                        crate::Geometry::Wgpu(primitive) => {
                            renderer.draw_primitive(primitive);
                        }
                        #[cfg(feature = "tiny_skia")]
                        crate::Geometry::TinySkia(_) => unreachable!(),
                    }
                }
            }
        }
    }
}

impl iced_wgpu::primitive::pipeline::Renderer for Renderer {
    fn draw_pipeline_primitive(
        &mut self,
        bounds: Rectangle,
        primitive: impl wgpu::primitive::pipeline::Primitive,
    ) {
        match self {
            #[cfg(feature = "tiny_skia")]
            Self::TinySkia(_renderer) => {
                log::warn!(
                    "Custom shader primitive is unavailable with tiny-skia."
                );
            }
            Self::Wgpu(renderer) => {
                renderer.draw_pipeline_primitive(bounds, primitive);
            }
        }
    }
}
