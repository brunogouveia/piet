mod text;

use std::borrow::Cow;

use skia_safe::{Color4f, ISize, Paint, Path, Surface};

use piet::kurbo::{Affine, PathEl, Point, Rect, Shape};
use piet::{Color, Error, FixedGradient, HitTestPoint, HitTestTextPosition, ImageFormat, InterpolationMode, IntoBrush, RenderContext, StrokeStyle, Text, TextLayout};

pub use crate::text::{SkiaText, SkiaTextLayout};

fn foo() {
    let surface = Surface::new_raster_n32_premul(ISize::new(1280, 1024));
}

#[derive(Clone)]
pub struct Brush {
    paint: Paint,
}

pub struct Image {
    image: skia_safe::Image,
}


pub struct SkiaRenderContext<'a> {
    surface: &'a mut Surface,
    text: SkiaText,
}

impl<'a> SkiaRenderContext<'a> {
    pub fn new(surface: &mut Surface) -> SkiaRenderContext {
        SkiaRenderContext {
            surface,
            text: SkiaText {},
        }
    }
}

impl<'a> RenderContext for SkiaRenderContext<'a> {
    type Brush = Brush;

    type Image = Image;

    type Text = SkiaText;
    type TextLayout = SkiaTextLayout;

    fn status(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn clear(&mut self, color: Color) {
        let rgba = color.as_rgba_u32();
        self.surface.canvas().clear(skia_safe::Color::from_argb(
            byte_to_byte(rgba),
            byte_to_byte(rgba >> 24),
            byte_to_byte(rgba >> 16),
            byte_to_byte(rgba >> 8),
        ));
    }

    fn solid_brush(&mut self, color: Color) -> Brush {
        let mut paint = Paint::new(color_to_skia_color4f(&color), None);
        paint.set_anti_alias(true);
        Brush { paint }
    }

    fn gradient(&mut self, gradient: impl Into<FixedGradient>) -> Result<Brush, Error> {
        let mut paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        paint.set_anti_alias(true);
        match gradient.into() {
            FixedGradient::Linear(linear) => {
                let points = (
                    point_to_skia_point(linear.start),
                    point_to_skia_point(linear.end),
                );
                let colors: Vec<skia_safe::Color> = linear
                    .stops
                    .iter()
                    .map(|stop| color_to_skia_color(&stop.color))
                    .collect();
                let pos: Vec<skia_safe::scalar> =
                    linear.stops.iter().map(|stop| stop.pos).collect();

                let pos2: &[skia_safe::scalar] = &pos;

                paint.set_shader(skia_safe::gradient_shader::linear(
                    points,
                    skia_safe::gradient_shader::GradientShaderColors::Colors(&colors),
                    Some(pos2),
                    skia_safe::TileMode::Clamp,
                    None,
                    None,
                ));
            }
            FixedGradient::Radial(radial) => {}
        };

        Ok(Brush { paint })
    }

    fn fill(&mut self, shape: impl Shape, brush: &impl IntoBrush<Self>) {
        let brush = brush.make_brush(self, || shape.bounding_box());
        let path = create_path(shape);
        self.surface
            .canvas()
            // .draw_paint(&brush.paint);
            .draw_path(&path, &brush.paint);
    }

    fn fill_even_odd(&mut self, shape: impl Shape, brush: &impl IntoBrush<Self>) {
        let brush = brush.make_brush(self, || shape.bounding_box());
        let mut path = create_path(shape);
        path.set_fill_type(skia_safe::PathFillType::EvenOdd);

        self.surface.canvas().draw_path(&path, &brush.paint);
    }

    fn clip(&mut self, shape: impl Shape) {
        let path = create_path(shape);
        self.surface.canvas().clip_path(&path, None, None);
    }

    fn stroke(&mut self, shape: impl Shape, brush: &impl IntoBrush<Self>, width: f64) {
        let mut brush = brush.make_brush(self, || shape.bounding_box());
        let brush = brush.to_mut();
        let path = create_path(shape);
        brush.paint.set_style(skia_safe::PaintStyle::Stroke);
        brush.paint.set_stroke_width(width as f32);

        self.surface.canvas().draw_path(&path, &brush.paint);
    }

    // TODO: implement this!
    fn stroke_styled(
        &mut self,
        shape: impl Shape,
        brush: &impl IntoBrush<Self>,
        width: f64,
        style: &StrokeStyle,
    ) {
        let mut brush = brush.make_brush(self, || shape.bounding_box());
        let brush = brush.to_mut();
        let path = create_path(shape);
        brush.paint.set_style(skia_safe::PaintStyle::Stroke);
        brush.paint.set_stroke_width(width as f32);

        self.surface.canvas().draw_path(&path, &brush.paint);
    }

    fn save(&mut self) -> Result<(), Error> {
        self.surface.canvas().save();
        self.status()
    }

    fn restore(&mut self) -> Result<(), Error> {
        self.surface.canvas().restore();
        self.status()
    }

    fn finish(&mut self) -> Result<(), Error> {
        self.status()
    }

    fn transform(&mut self, transform: Affine) {
        self.surface
            .canvas()
            .set_matrix(&affine_to_matrix(transform));
    }

    fn current_transform(&mut self) -> Affine {
        matrix_to_affine(self.surface.canvas().total_matrix())
    }

    fn make_image(
        &mut self,
        width: usize,
        height: usize,
        buf: &[u8],
        format: ImageFormat,
    ) -> Result<Self::Image, Error> {
        let image_info = skia_safe::ImageInfo::new_n32(
            ISize::new(width as i32, height as i32),
            skia_safe::AlphaType::Unpremul,
            Some(skia_safe::ColorSpace::new_srgb()),
        );
        let data = skia_safe::Data::new_copy(buf);

        let bytes_per_pixel = match format {
            ImageFormat::Rgb => 3,
            _ => 4,
        };

        if let Some(sk_image) =
            skia_safe::Image::from_raster_data(&image_info, data, width * bytes_per_pixel)
        {
            return Ok(Image { image: sk_image });
        }
        Err(piet::new_error(piet::ErrorKind::MissingFeature))
    }

    fn draw_image(
        &mut self,
        image: &Self::Image,
        dst_rect: impl Into<Rect>,
        interp: InterpolationMode,
    ) {
        let rect = dst_rect.into();
        let left_top = skia_safe::Point::new(rect.x0 as f32, rect.y0 as f32);

        // let paint = skia_safe::Paint::new(skia_safe::Color4f::new(1.0, 1.0, 1.0, 1.0), None);

        self.surface
            .canvas()
            .draw_image(&image.image, left_top, None);
    }

    fn draw_image_area(
        &mut self,
        image: &Self::Image,
        src_rect: impl Into<Rect>,
        dst_rect: impl Into<Rect>,
        interp: InterpolationMode,
    ) {
        self.draw_image(image, dst_rect, interp);
    }

    fn text(&mut self) -> &mut Self::Text {
        &mut self.text
    }

    fn draw_text(
        &mut self,
        layout: &Self::TextLayout,
        pos: impl Into<Point>,
        brush: &impl IntoBrush<Self>,
    ) {
    }
}

impl<'a> IntoBrush<SkiaRenderContext<'a>> for Brush {
    fn make_brush<'b>(
        &'b self,
        _piet: &mut SkiaRenderContext,
        _bbox: impl FnOnce() -> Rect,
    ) -> std::borrow::Cow<'b, Brush> {
        Cow::Borrowed(self)
    }
}

fn point_to_skia_point(point: Point) -> skia_safe::Point {
    skia_safe::Point::new(point.x as f32, point.y as f32)
}

fn color_to_skia_color(color: &Color) -> skia_safe::Color {
    let rgba = color.as_rgba_u32();

    skia_safe::Color::from_argb(
        byte_to_byte(rgba),
        byte_to_byte(rgba >> 24),
        byte_to_byte(rgba >> 16),
        byte_to_byte(rgba >> 8),
    )
}

fn color_to_skia_color4f(color: &Color) -> skia_safe::Color4f {
    let rgba = color.as_rgba_u32();

    skia_safe::Color4f::new(
        byte_to_frac(rgba >> 24),
        byte_to_frac(rgba >> 16),
        byte_to_frac(rgba >> 8),
        byte_to_frac(rgba),
    )
}

fn byte_to_byte(byte: u32) -> u8 {
    ((byte & 255) as u8)
}

fn byte_to_frac(byte: u32) -> f32 {
    ((byte & 255) as f32) * (1.0 / 255.0)
}

fn create_path(shape: impl Shape) -> skia_safe::Path {
    let mut path = skia_safe::Path::new();
    // path.move_to(skia_safe::Point::new(440.0, 400.0));
    // path.cubic_to(skia_safe::Point::new(440.0, 414.29062359632655), skia_safe::Point::new(432.37604307034013, 427.49570435321425), skia_safe::Point::new(420.0, 434.64101615137753));
    // path.close();
    let bez_path = shape.to_bez_path(1e-3);
    for el in bez_path {
        match el {
            PathEl::MoveTo(p) => {
                path.move_to(point_to_skia_point(p));
            }
            PathEl::LineTo(p) => {
                path.line_to(point_to_skia_point(p));
            }
            PathEl::CurveTo(cp1, cp2, ep) => {
                path.cubic_to(
                    point_to_skia_point(cp1),
                    point_to_skia_point(cp2),
                    point_to_skia_point(ep),
                );
            }
            PathEl::QuadTo(p1, p2) => {
                path.quad_to(point_to_skia_point(p1), point_to_skia_point(p2));
            }
            PathEl::ClosePath => {
                path.close();
            }
        }
    }
    path
}

fn affine_to_matrix(affine: Affine) -> skia_safe::Matrix {
    let mut matrix = skia_safe::Matrix::new_identity();

    let affine = affine.as_coeffs();
    matrix.set_affine(&[
        affine[0] as f32,
        affine[1] as f32,
        affine[2] as f32,
        affine[3] as f32,
        affine[4] as f32,
        affine[5] as f32,
    ]);
    matrix
}

fn matrix_to_affine(matrix: skia_safe::Matrix) -> Affine {
    let affine = matrix.to_affine().unwrap();
    Affine::new([
        affine[0] as f64,
        affine[1] as f64,
        affine[2] as f64,
        affine[3] as f64,
        affine[4] as f64,
        affine[5] as f64,
    ])
}
