use skia_safe::{ISize, Surface};

use piet::{Color, RenderContext};

fn foo() {
    let surface = Surface::new_raster_n32_premul(ISize::new(1280, 1024));
}

pub struct SkiaRenderContext<'a> {
    surface: &'a mut Surface,
}

impl<'a> SkiaRenderContext<'a> {
    pub fn new(surface: &mut Surface) -> SkiaRenderContext {
        SkiaRenderContext { surface }
    }

    pub fn clear(&mut self, color: Color) {
        let rgba = color.as_rgba_u32();
        self.surface.canvas().clear(skia_safe::Color::from_argb(
            byte_to_byte(rgba),
            byte_to_byte(rgba >> 24),
            byte_to_byte(rgba >> 16),
            byte_to_byte(rgba >> 8),
        ));
    }
}

// impl<'a> RenderContext for SkiaRenderContext<'a>{

//     fn solid_brush(&mut self, color: Color) -> Self::Brush {

//     }

// }

fn byte_to_byte(byte: u32) -> u8 {
    ((byte & 255) as u8)
}
