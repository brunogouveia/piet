//! Basic example of rendering on Cairo.

use std::fs::File;
use std::io::prelude::*;

use skia_safe::{Bitmap, EncodedImageFormat, ISize, Surface};

use piet::{Color, RenderContext};
use piet_skia::SkiaRenderContext;

use piet_test::draw_test_picture;

const TEXTURE_WIDTH: i32 = 400;
const TEXTURE_HEIGHT: i32 = 200;

const HIDPI: f64 = 2.0;

fn main() {
    let test_picture_number = std::env::args()
        .skip(1)
        .next()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);

    let mut surface =
        Surface::new_raster_n32_premul(ISize::new(TEXTURE_WIDTH, TEXTURE_HEIGHT)).unwrap();

    let mut renderContext = SkiaRenderContext::new(&mut surface);
    renderContext.clear(Color::rgba8(255, 0, 0, 255));

    if let Some(data) = surface
        .image_snapshot()
        .encode_to_data(EncodedImageFormat::PNG)
    {
        let mut file = File::create("temp-cairo.png").expect("Couldn't create 'file.png'");
        file.write(data.as_bytes());
    }
    // surface.read_pixels_to_bitmap(bitmap, src)

    // let bitmap = Bitmap::new();
    // bitmap.

    // draw_test_picture(&mut piet_context, test_picture_number).unwrap();
    // piet_context.finish().unwrap();
    // surface.flush();
    // let mut file = File::create("temp-cairo.png").expect("Couldn't create 'file.png'");
    // surface
    //     .write_to_png(&mut file)
    //     .expect("Error writing image file");
}
