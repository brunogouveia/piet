use core_graphics::base::*;
use core_graphics::color_space::*;
use core_graphics::data_provider::*;
use core_graphics::image::*;

use piet::ImageFormat;

pub struct Image {
    pub image: CGImage,
}

impl Image {
    pub fn new(width: usize, height: usize, buf: &[u8], format: ImageFormat) -> Image {
        let bytes_per_pixel = match format {
            ImageFormat::Rgb => 3,
            ImageFormat::RgbaPremul | ImageFormat::RgbaSeparate => 4,
            _ => panic!("Unsupported format"),
        };

        let bits_per_component = 8;
        let bits_per_pixel = bytes_per_pixel * 8;
        let bytes_per_row = width * bytes_per_pixel;

        let colorspace = CGColorSpace::create_device_rgb();
        let bitmap_info = kCGBitmapByteOrderDefault
            | match format {
                ImageFormat::Rgb => kCGImageAlphaNone,
                ImageFormat::RgbaPremul => kCGImageAlphaPremultipliedLast,
                ImageFormat::RgbaSeparate => kCGImageAlphaLast,
                _ => panic!("Unsupported format"),
            };
        let data_provider = unsafe { CGDataProvider::from_slice(buf) };
        let image = CGImage::new(
            width,
            height,
            bits_per_component,
            bits_per_pixel,
            bytes_per_row,
            &colorspace,
            bitmap_info,
            &data_provider,
            true,
            kCGRenderingIntentDefault,
        );
        Image { image }
    }
}
