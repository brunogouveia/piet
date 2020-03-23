//! Basic example of rendering on Cairo.

use std::fs::File;
use std::io::prelude::*;

use image;

use skia_safe::{EncodedImageFormat, ISize, Surface};

use piet::kurbo::{Circle, Point, Rect, RoundedRect};
use piet::{Color, FixedGradient, FixedLinearGradient, ImageFormat, InterpolationMode, GradientStop, RenderContext};

use piet_skia::SkiaRenderContext;

// use piet_test::draw_test_picture;

const TEXTURE_WIDTH: i32 = 1920;
const TEXTURE_HEIGHT: i32 = 1080;

// const HIDPI: f64 = 2.0;

fn main() {
    // let test_picture_number = std::env::args()
    //     .skip(1)
    //     .next()
    //     .and_then(|s| s.parse::<usize>().ok())
    //     .unwrap_or(0);

    let mut surface =
        Surface::new_raster_n32_premul(ISize::new(TEXTURE_WIDTH, TEXTURE_HEIGHT)).unwrap();

    let mut render_context = SkiaRenderContext::new(&mut surface);
    render_context.clear(Color::rgba8(255, 0, 0, 255));

    let linear_gradient = FixedLinearGradient {
        start: Point::new(0.0, 0.0),
        end: Point::new(0.0, 100.0),
        stops: vec![
            GradientStop {
                pos: 0.0,
                color: Color::BLACK,
            },
            GradientStop {
                pos: 1.0,
                color: Color::WHITE,
            },
        ],
    };

    let brush = render_context.solid_brush(Color::rgb8(0, 255, 220));
    let brush2 = render_context
        .gradient(FixedGradient::Linear(linear_gradient))
        .expect("To be able to create gradient brush");

    render_context.fill(Rect::new(0.0, 0.0, 100.0, 100.0), &brush2);
    render_context.fill(RoundedRect::new(40.0, 40.0, 100.0, 100.0, 25.0), &brush2);
    render_context.fill(Circle::new(Point::new(100.0, 100.0), 40.0), &brush);

    let image =image::open("input-skia.png").expect("Unable to open input image");
    let image_pixels = image.to_rgba();
    let image_dimensions = image_pixels.dimensions();
    
    let mut pixels = vec![255; 200*200*4];

    let image = render_context.make_image(image_dimensions.0 as usize, image_dimensions.1 as usize, &image_pixels.to_vec(), ImageFormat::RgbaSeparate).expect("Unable to create image");
    // let image = render_context.make_image(200, 200, &pixels.as_slice(), ImageFormat::RgbaSeparate).expect("Unable to create image");

    render_context.draw_image(&image, Rect::new(100.0, 100.0, 200.0, 200.0), InterpolationMode::NearestNeighbor);

    if let Some(data) = surface
        .image_snapshot()
        .encode_to_data(EncodedImageFormat::PNG)
    {
        let mut file = File::create("temp-skia.png").expect("Couldn't create 'file.png'");
        file.write(data.as_bytes()).expect("Couldn't write to file");
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
