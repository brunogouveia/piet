// allows e.g. raw_data[dst_off + x * 4 + 2] = buf[src_off + x * 4 + 0];
#![allow(clippy::identity_op)]

//! Support for piet Core graphics back-end.

use core_graphics::base::*;
use core_graphics::color_space::*;
use core_graphics::context::*;
use core_graphics::geometry::*;
#[cfg(feature = "png")]
use png::{ColorType, Encoder};
#[cfg(feature = "png")]
use std::fs::File;
#[cfg(feature = "png")]
use std::io::BufWriter;
use std::io::Write;
use std::marker::PhantomData;
use std::path::Path;

use piet::{ErrorKind, ImageFormat};
#[doc(hidden)]
pub use piet_core_graphics::*;

/// The `RenderContext` for the CoreGraphics backend, which is selected.
pub type Piet<'a> = CoreGraphicsRenderContext<'a>;

/// The associated brush type for this backend.
///
/// This type matches `RenderContext::Brush`
pub type Brush = piet_core_graphics::Brush;

/// The associated text factory for this backend.
///
/// This type matches `RenderContext::Text`
pub type PietText<'a> = CoreGraphicsText<'a>;

/// The associated font type for this backend.
///
/// This type matches `RenderContext::Text::Font`
pub type PietFont = CoreGraphicsFont;

/// The associated font builder for this backend.
///
/// This type matches `RenderContext::Text::FontBuilder`
pub type PietFontBuilder<'a> = CoreGraphicsFontBuilder;

/// The associated text layout type for this backend.
///
/// This type matches `RenderContext::Text::TextLayout`
pub type PietTextLayout = CoreGraphicsTextLayout;

/// The associated text layout builder for this backend.
///
/// This type matches `RenderContext::Text::TextLayoutBuilder`
pub type PietTextLayoutBuilder<'a> = CoreGraphicsTextLayoutBuilder;

/// The associated image type for this backend.
///
/// This type matches `RenderContext::Image`
pub type Image = piet_core_graphics::Image;

/// A struct that can be used to create bitmap render contexts.
///
/// In the case of Core graphics, no state is needed.
pub struct Device;

/// A struct provides a `RenderContext` and then can have its bitmap extracted.
pub struct BitmapTarget<'a> {
    ctx: CGContext,
    size: CGSize,
    phantom: PhantomData<&'a ()>,
}

impl Device {
    /// Create a new device.
    pub fn new() -> Result<Device, piet::Error> {
        Ok(Device)
    }

    /// Create a new bitmap target.
    pub fn bitmap_target(
        &mut self,
        width: usize,
        height: usize,
        pix_scale: f64,
    ) -> Result<BitmapTarget, piet::Error> {
        let color_space = CGColorSpace::create_device_rgb();
        let ctx = CGContext::create_bitmap_context(
            None,
            width,
            height,
            8,
            4 * width,
            &color_space,
            core_graphics::base::kCGImageAlphaPremultipliedLast,
        );
        ctx.scale(pix_scale, pix_scale);

        let size = CGSize::new(width as CGFloat, height as CGFloat);
        let phantom = Default::default();
        Ok(BitmapTarget { ctx, size, phantom })
    }
}

impl<'a> BitmapTarget<'a> {
    /// Get a piet `RenderContext` for the bitmap.
    ///
    /// Note: caller is responsible for calling `finish` on the render
    /// context at the end of rendering.
    pub fn render_context(&mut self) -> CoreGraphicsRenderContext {
        CoreGraphicsRenderContext::new(&mut self.ctx, self.size, true)
    }

    /// Get raw RGBA pixels from the bitmap.
    pub fn into_raw_pixels(mut self, fmt: ImageFormat) -> Result<Vec<u8>, piet::Error> {
        // TODO: convert other formats.
        if fmt != ImageFormat::RgbaPremul {
            return Err(piet::new_error(ErrorKind::NotSupported));
        }
        self.ctx.flush();
        let width = self.size.width as usize;
        let height = self.size.height as usize;
        let mut raw_data = vec![0; width * height * 4];
        raw_data
            .write(self.ctx.data())
            .map_err(Into::<Box<dyn std::error::Error>>::into)?;
        Ok(raw_data)
    }

    /// Save bitmap to RGBA PNG file
    #[cfg(feature = "png")]
    pub fn save_to_file<P: AsRef<Path>>(self, path: P) -> Result<(), piet::Error> {
        let height = self.ctx.height();
        let width = self.ctx.width();
        let image = self.into_raw_pixels(ImageFormat::RgbaPremul)?;
        let file = BufWriter::new(File::create(path).map_err(|e| Into::<Box<_>>::into(e))?);
        let mut encoder = Encoder::new(file, width as u32, height as u32);
        encoder.set_color(ColorType::RGBA);
        encoder
            .write_header()
            .map_err(|e| Into::<Box<_>>::into(e))?
            .write_image_data(&image)
            .map_err(|e| Into::<Box<_>>::into(e))?;
        Ok(())
    }

    /// Stub for feature is missing
    #[cfg(not(feature = "png"))]
    pub fn save_to_file<P: AsRef<Path>>(self, _path: P) -> Result<(), piet::Error> {
        Err(piet::new_error(ErrorKind::MissingFeature))
    }
}
