//! Basic example of rendering on Core Graphics.

use core_graphics::color_space::*;
use core_graphics::context::*;
use core_graphics::geometry::*;

use image;

use piet::kurbo::Affine;
use piet::RenderContext;

use piet_core_graphics::CoreGraphicsRenderContext;

use piet_test::draw_test_picture;

use std::path::Path;

const TEXTURE_WIDTH: usize = 400;
const TEXTURE_HEIGHT: usize = 200;

const HIDPI: f64 = 2.0;

fn main() {
    let test_picture_number = std::env::args()
        .skip(1)
        .next()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);

    let cs = CGColorSpace::create_device_rgb();
    let mut ctx = CGContext::create_bitmap_context(
        None,
        TEXTURE_WIDTH,
        TEXTURE_HEIGHT,
        8,
        TEXTURE_WIDTH * 4,
        &cs,
        core_graphics::base::kCGImageAlphaPremultipliedLast,
    );

    let mut piet_context =
        CoreGraphicsRenderContext::new(&mut ctx, CGSize::new(400.0, 200.0), false);
    piet_context.transform(Affine::scale(HIDPI));

    draw_test_picture(&mut piet_context, test_picture_number).unwrap();
    piet_context.finish().unwrap();

    ctx.flush();
    image::save_buffer(
        Path::new("temp-core-graphics.png"),
        ctx.data(),
        400,
        200,
        image::ColorType::Rgba8,
    )
    .expect("Error writing image file");
}
