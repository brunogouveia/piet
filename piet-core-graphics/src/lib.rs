mod image;
mod text;
mod utils;

use core_graphics::base::*;
use core_graphics::color_space::*;
use core_graphics::context::*;
use core_graphics::geometry::*;
use core_graphics::gradient::{CGGradient, CGGradientDrawingOptions};

use piet::kurbo::{Affine, PathEl, Point, Rect, Shape};
use piet::{
    Color, Error, FixedGradient, GradientStop, ImageFormat, InterpolationMode, IntoBrush, LineCap,
    LineJoin, RenderContext, StrokeStyle,
};

pub use crate::image::Image;

pub use crate::text::{
    CoreGraphicsFont, CoreGraphicsFontBuilder, CoreGraphicsText, CoreGraphicsTextLayout,
    CoreGraphicsTextLayoutBuilder,
};

use crate::utils::{ToCg, ToPiet};
use std::borrow::Cow;
use core_foundation::base::CFRange;

#[derive(Clone)]
pub struct LinearGradientBrush {
    cg_gradient: CGGradient,
    start: Point,
    end: Point,
}

#[derive(Clone)]
pub struct RadialGradientBrush {
    cg_gradient: CGGradient,

    start_center: CGPoint,
    end_center: CGPoint,

    start_radius: CGFloat,
    end_radius: CGFloat,
}

#[derive(Clone)]
pub enum Brush {
    Solid(Color),
    Linear(LinearGradientBrush),
    Radial(RadialGradientBrush),
}

impl<'a> IntoBrush<CoreGraphicsRenderContext<'a>> for Brush {
    fn make_brush<'b>(
        &'b self,
        _piet: &mut CoreGraphicsRenderContext,
        _bbox: impl FnOnce() -> Rect,
    ) -> Cow<'b, Brush> {
        Cow::Borrowed(self)
    }
}

pub struct CoreGraphicsRenderContext<'a> {
    ctx: &'a mut CGContext,
    size: CGSize,
    text: text::CoreGraphicsText<'a>,
}

impl<'a> CoreGraphicsRenderContext<'a> {
    pub fn new(ctx: &'a mut CGContext, size: CGSize, flipped: bool) -> CoreGraphicsRenderContext {
        if !flipped {
            ctx.concat_ctm(CGAffineTransform::make_translation(0.0, size.height));
            ctx.concat_ctm(CGAffineTransform::make_scale(1.0, -1.0));
        }

        CoreGraphicsRenderContext {
            ctx,
            size,
            text: text::CoreGraphicsText::default(),
        }
    }
}

impl<'a> RenderContext for CoreGraphicsRenderContext<'a> {
    type Brush = Brush;

    type Text = text::CoreGraphicsText<'a>;

    type TextLayout = text::CoreGraphicsTextLayout;
    type Image = image::Image;

    fn status(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn solid_brush(&mut self, color: Color) -> Self::Brush {
        Brush::Solid(color)
    }

    fn gradient(&mut self, gradient: impl Into<FixedGradient>) -> Result<Self::Brush, Error> {
        let gradient = gradient.into();
        match gradient {
            FixedGradient::Linear(linear) => {
                let color_space = CGColorSpace::create_device_rgb();

                let mut colors = vec![0.0; linear.stops.len() * 4];
                let mut locations = vec![0.0; linear.stops.len()];

                for (idx, stop) in linear.stops.iter().enumerate() {
                    let pos = stop.pos;
                    let color = &stop.color;

                    let color_as_u32 = color.as_rgba_u32();

                    let r: u8 = (color_as_u32 >> 24) as u8;
                    let g: u8 = (color_as_u32 >> 16) as u8;
                    let b: u8 = (color_as_u32 >> 8) as u8;
                    let a: u8 = (color_as_u32) as u8;
                    colors[idx * 4] = (r as CGFloat) / 255.0;
                    colors[idx * 4 + 1] = (g as CGFloat) / 255.0;
                    colors[idx * 4 + 2] = (b as CGFloat) / 255.0;
                    colors[idx * 4 + 3] = (a as CGFloat) / 255.0;

                    locations[idx] = pos as CGFloat;
                }

                let cg_gradient = CGGradient::create_with_color_components(
                    &color_space,
                    &colors,
                    &locations,
                    linear.stops.len(),
                );
                Ok(Brush::Linear(LinearGradientBrush {
                    cg_gradient,
                    start: linear.start,
                    end: linear.end,
                }))
            }
            FixedGradient::Radial(radial) => {
                let cg_gradient = gradient_stops_to_cg_gradient(&radial.stops);
                let start_center = CGPoint::new(
                    radial.center.x + radial.origin_offset.x,
                    radial.center.y + radial.origin_offset.y,
                );
                let end_center = CGPoint::new(radial.center.x, radial.center.y);
                Ok(Brush::Radial(RadialGradientBrush {
                    cg_gradient,
                    start_center,
                    end_center,
                    start_radius: 0.0,
                    end_radius: radial.radius,
                }))
            }
        }
    }

    fn clear(&mut self, color: Color) {
        self.ctx.set_fill_color(&color.to_cg());
        self.ctx
            .fill_rect(CGRect::new(&CGPoint::new(0.0, 0.0), &self.size));
    }

    fn stroke(&mut self, shape: impl Shape, brush: &impl IntoBrush<Self>, width: f64) {
        let brush = brush.make_brush(self, || shape.bounding_box());

        self.ctx.set_line_width(width);
        match brush.as_ref() {
            Brush::Solid(color) => {
                self.set_stroke_solid_color(color);

                if let Some(rect) = shape.as_rect() {
                    self.ctx.stroke_rect(rect.to_cg());
                } else {
                    self.set_path(shape);
                    self.ctx.stroke_path();
                }
            }
            Brush::Linear(gradient) => {
                self.ctx.save();
                self.set_stroked_path_clip(shape);
                self.draw_linear_gradient(gradient);
                self.ctx.restore();
            }
            Brush::Radial(gradient) => {
                self.ctx.save();
                self.set_stroked_path_clip(shape);
                self.draw_radial_gradient(gradient);
                self.ctx.restore();
            }
        }
    }

    fn stroke_styled(
        &mut self,
        shape: impl Shape,
        brush: &impl IntoBrush<Self>,
        width: f64,
        style: &StrokeStyle,
    ) {
        self.ctx.save();
        self.set_stroke_style(style);
        self.stroke(shape, brush, width);
        self.ctx.restore();
    }

    fn fill(&mut self, shape: impl Shape, brush: &impl IntoBrush<Self>) {
        let brush = brush.make_brush(self, || shape.bounding_box());
        match brush.as_ref() {
            Brush::Solid(color) => {
                self.ctx.set_fill_color(&color.to_cg());

                if let Some(rect) = shape.as_rect() {
                    self.ctx.fill_rect(rect.to_cg());
                } else {
                    self.set_path(shape);
                    self.ctx.fill_path();
                }
            }
            Brush::Linear(gradient) => {
                self.ctx.save();

                self.set_path(shape);
                self.ctx.clip();
                self.draw_linear_gradient(gradient);

                self.ctx.restore();
            }
            Brush::Radial(gradient) => {
                self.ctx.save();

                self.set_path(shape);
                self.ctx.clip();
                self.draw_radial_gradient(gradient);

                self.ctx.restore();
            }
        }
    }

    fn fill_even_odd(&mut self, shape: impl Shape, brush: &impl IntoBrush<Self>) {
        let brush = brush.make_brush(self, || shape.bounding_box());
        match brush.as_ref() {
            Brush::Solid(color) => {
                self.ctx.set_fill_color(&color.to_cg());

                self.set_path(shape);
                self.ctx.eo_fill_path();
            }
            Brush::Linear(gradient) => {
                self.ctx.save();

                self.set_path(shape);
                self.ctx.eo_clip();
                self.draw_linear_gradient(gradient);

                self.ctx.restore();
            }
            Brush::Radial(gradient) => {
                self.ctx.save();

                self.set_path(shape);
                self.ctx.eo_clip();
                self.draw_radial_gradient(gradient);

                self.ctx.restore();
            }
        }
    }

    fn clip(&mut self, shape: impl Shape) {
        if let Some(rect) = shape.as_rect() {
            self.ctx.clip_to_rect(rect.to_cg());
        } else {
            self.set_path(shape);
            self.ctx.clip()
        }
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
        let brush = brush.make_brush(self, || Rect::new(0.0, 0.0, 300.0, 300.0));
        let pos = pos.into();

        self.ctx.save();

        self.ctx
            .concat_ctm(CGAffineTransform::make_scale(1.0, -1.0));
        // self.ctx.set_text_position(pos.x, -pos.y);
        self.ctx.concat_ctm(CGAffineTransform::make_translation(pos.x, -pos.y));

        match brush.as_ref() {
            Brush::Solid(color) => {
                let (_, line, _) = layout.build_colored_line(Some(&color.to_cg()));

                let mut origins = Vec::<CGPoint>::new();
                origins.push(CGPoint::new(0.0, 0.0));
                line.get_line_origins(CFRange::init(0, 1), &mut origins);

                self.ctx.concat_ctm(CGAffineTransform::make_translation(0.0, -origins[0].y));

                self.ctx
                    .set_text_drawing_mode(CGTextDrawingMode::CGTextFill);
                line.draw(&self.ctx);
            }
            Brush::Linear(gradient) => {
                self.ctx
                    .set_text_drawing_mode(CGTextDrawingMode::CGTextClip);
                layout.black_line.draw(&self.ctx);

                self.ctx
                    .concat_ctm(CGAffineTransform::make_scale(1.0, -1.0));

                self.draw_linear_gradient(gradient);
            }
            Brush::Radial(gradient) => {
                self.ctx
                    .set_text_drawing_mode(CGTextDrawingMode::CGTextClip);
                layout.black_line.draw(&self.ctx);

                self.ctx
                    .concat_ctm(CGAffineTransform::make_scale(1.0, -1.0));

                self.draw_radial_gradient(gradient);
            }
        }
        self.ctx.restore();
    }

    fn save(&mut self) -> Result<(), Error> {
        self.ctx.save();
        Ok(())
    }

    fn restore(&mut self) -> Result<(), Error> {
        self.ctx.restore();
        Ok(())
    }

    fn finish(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn transform(&mut self, transform: Affine) {
        self.ctx.concat_ctm(transform.to_cg());
    }

    fn make_image(
        &mut self,
        width: usize,
        height: usize,
        buf: &[u8],
        format: ImageFormat,
    ) -> Result<Self::Image, Error> {
        Ok(image::Image::new(width, height, buf, format))
    }

    fn draw_image(
        &mut self,
        image: &Self::Image,
        rect: impl Into<Rect>,
        _interp: InterpolationMode,
    ) {
        let rect = rect.into();
        self.ctx.save();
        self.ctx.concat_ctm(CGAffineTransform::make_translation(
            rect.x0,
            rect.y0 + rect.height(),
        ));
        self.ctx
            .concat_ctm(CGAffineTransform::make_scale(1.0, -1.0));
        self.ctx.draw_image(
            CGRect::new(&CG_ZERO_POINT, &rect.size().to_cg()),
            &image.image,
        );
        self.ctx.restore();
    }

    fn draw_image_area(&mut self, image: &Self::Image, src_rect: impl Into<Rect>, dst_rect: impl Into<Rect>, interp: InterpolationMode) {
        // Not implemented
    }

    fn blurred_rect(&mut self, rect: Rect, blur_radius: f64, brush: &impl IntoBrush<Self>) {
        // Not implemented
    }

    fn current_transform(&self) -> Affine {
        self.ctx.get_ctm().to_piet()
    }
}

impl<'a> CoreGraphicsRenderContext<'a> {
    #[inline]
    fn set_stroke_solid_color(&mut self, color: &Color) {
        let color_as_u32 = color.as_rgba_u32();
        let r: u8 = (color_as_u32 >> 24) as u8;
        let g: u8 = (color_as_u32 >> 16) as u8;
        let b: u8 = (color_as_u32 >> 8) as u8;
        let a: u8 = (color_as_u32) as u8;
        self.ctx.set_rgb_stroke_color(
            (r as CGFloat) / 255.0,
            (g as CGFloat) / 255.0,
            (b as CGFloat) / 255.0,
            (a as CGFloat) / 255.0,
        )
    }

    #[inline]
    fn set_stroke_style(&mut self, style: &StrokeStyle) {
        self.ctx
            .set_line_cap(style.line_cap.unwrap_or(LineCap::Butt).to_cg());
        self.ctx
            .set_line_join(style.line_join.unwrap_or(LineJoin::Miter).to_cg());

        if let Some(dash) = &style.dash {
            let (lengths, phase) = dash;
            self.ctx.set_line_dash(*phase, lengths.as_slice());
        }
    }

    #[inline]
    fn set_stroked_path_clip(&mut self, shape: impl Shape) {
        self.set_path(shape);
        self.ctx.replace_path_with_stroked_path();
        self.ctx.clip();
    }

    fn set_path(&mut self, shape: impl Shape) {
        self.ctx.begin_path();
        for el in shape.to_bez_path(1e-3) {
            match el {
                PathEl::LineTo(point) => {
                    self.ctx.add_line_to_point(point.x, point.y);
                }
                PathEl::MoveTo(point) => {
                    self.ctx.move_to_point(point.x, point.y);
                }
                PathEl::QuadTo(control_point, point) => {
                    self.ctx.add_quad_curve_to_point(
                        control_point.x,
                        control_point.y,
                        point.x,
                        point.y,
                    );
                }
                PathEl::CurveTo(control_point1, control_point2, point) => {
                    self.ctx.add_curve_to_point(
                        control_point1.x,
                        control_point1.y,
                        control_point2.x,
                        control_point2.y,
                        point.x,
                        point.y,
                    );
                }
                PathEl::ClosePath => {
                    self.ctx.close_path();
                }
            }
        }
    }

    #[inline]
    fn draw_linear_gradient(&self, gradient: &LinearGradientBrush) {
        let cg_gradient = &gradient.cg_gradient;
        let start = gradient.start;
        let end = gradient.end;
        self.ctx.draw_linear_gradient(
            cg_gradient,
            start.to_cg(),
            end.to_cg(),
            CGGradientDrawingOptions::all(),
        );
    }

    #[inline]
    fn draw_radial_gradient(&self, gradient: &RadialGradientBrush) {
        let cg_gradient = &gradient.cg_gradient;
        let start_center = gradient.start_center;
        let end_center = gradient.end_center;
        let start_radius = gradient.start_radius;
        let end_radius = gradient.end_radius;
        self.ctx.draw_radial_gradient(
            cg_gradient,
            start_center,
            start_radius,
            end_center,
            end_radius,
            CGGradientDrawingOptions::all(),
        );
    }
}

fn gradient_stops_to_cg_gradient(stops: &Vec<GradientStop>) -> CGGradient {
    let color_space = CGColorSpace::create_device_rgb();

    let mut colors = vec![0.0; stops.len() * 4];
    let mut locations = vec![0.0; stops.len()];

    for (idx, stop) in stops.iter().enumerate() {
        let pos = stop.pos;
        let color = &stop.color;

        let color_as_u32 = color.as_rgba_u32();

        let r: u8 = (color_as_u32 >> 24) as u8;
        let g: u8 = (color_as_u32 >> 16) as u8;
        let b: u8 = (color_as_u32 >> 8) as u8;
        let a: u8 = (color_as_u32) as u8;
        colors[idx * 4] = (r as CGFloat) / 255.0;
        colors[idx * 4 + 1] = (g as CGFloat) / 255.0;
        colors[idx * 4 + 2] = (b as CGFloat) / 255.0;
        colors[idx * 4 + 3] = (a as CGFloat) / 255.0;

        locations[idx] = pos as CGFloat;
    }

    CGGradient::create_with_color_components(&color_space, &colors, &locations, stops.len())
}
