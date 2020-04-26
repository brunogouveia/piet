use core_graphics::color::CGColor;
use core_graphics::context::{CGLineCap, CGLineJoin};
use core_graphics::geometry::{CGAffineTransform, CGPoint, CGRect, CGSize};

use piet::kurbo::{Affine, Point, Rect, Size};
use piet::{Color, LineCap, LineJoin};

/// Used to easily convert from piet types to  core graphics equivalents
pub(crate) trait ToCg<T> {
    fn to_cg(&self) -> T;
}

/// Used to easily convert from core graphics types to piet equivalents
pub(crate) trait ToPiet<T> {
    fn to_piet(&self) -> T;
}

impl ToCg<CGColor> for Color {
    fn to_cg(&self) -> CGColor {
        let color_as_u32 = self.as_rgba_u32();
        let r: u8 = (color_as_u32 >> 24) as u8;
        let g: u8 = (color_as_u32 >> 16) as u8;
        let b: u8 = (color_as_u32 >> 8) as u8;
        let a: u8 = (color_as_u32) as u8;
        CGColor::rgb(
            (r as f64) / 255.0,
            (g as f64) / 255.0,
            (b as f64) / 255.0,
            (a as f64) / 255.0,
        )
    }
}

impl ToCg<CGLineCap> for LineCap {
    fn to_cg(&self) -> CGLineCap {
        match self {
            LineCap::Butt => CGLineCap::CGLineCapButt,
            LineCap::Round => CGLineCap::CGLineCapRound,
            LineCap::Square => CGLineCap::CGLineCapSquare,
        }
    }
}

impl ToCg<CGLineJoin> for LineJoin {
    fn to_cg(&self) -> CGLineJoin {
        match self {
            LineJoin::Bevel => CGLineJoin::CGLineJoinBevel,
            LineJoin::Miter => CGLineJoin::CGLineJoinMiter,
            LineJoin::Round => CGLineJoin::CGLineJoinRound,
        }
    }
}

impl ToCg<CGPoint> for Point {
    fn to_cg(&self) -> CGPoint {
        CGPoint::new(self.x, self.y)
    }
}

impl ToCg<CGSize> for Size {
    fn to_cg(&self) -> CGSize {
        CGSize::new(self.width, self.height)
    }
}

impl ToCg<CGRect> for Rect {
    fn to_cg(&self) -> CGRect {
        CGRect::new(&self.origin().to_cg(), &self.size().to_cg())
    }
}

impl ToCg<CGAffineTransform> for Affine {
    fn to_cg(&self) -> CGAffineTransform {
        let coeffs = self.as_coeffs();
        CGAffineTransform::new(
            coeffs[0], coeffs[1], coeffs[2], coeffs[3], coeffs[4], coeffs[5],
        )
    }
}

impl ToPiet<Affine> for CGAffineTransform {
    fn to_piet(&self) -> Affine {
        Affine::new([self.a, self.b, self.c, self.d, self.tx, self.ty])
    }
}
