
use piet::kurbo::{Point};
use piet::{Error, Font, FontBuilder, HitTestPoint, HitTestTextPosition, Text, TextLayout, TextLayoutBuilder};

pub struct SkiaFont {}

impl Font for SkiaFont {
    
}

pub struct SkiaFontBuilder {}

impl FontBuilder for SkiaFontBuilder {
    type Out =  SkiaFont;

    fn build(self) -> Result<Self::Out, Error> {
        Ok(SkiaFont{})
    }
}


pub struct SkiaText {}

impl Text for SkiaText {

    type FontBuilder = SkiaFontBuilder;
    type Font = SkiaFont;

    type TextLayoutBuilder = SkiaTextLayoutBuilder;
    type TextLayout = SkiaTextLayout;

    fn new_font_by_name(&mut self, name: &str, size: f64) -> Self::FontBuilder{
        SkiaFontBuilder{}
    }

    fn new_text_layout(&mut self, font: &Self::Font, text: &str) -> Self::TextLayoutBuilder{
        SkiaTextLayoutBuilder{}
    }
}



pub struct SkiaTextLayout {}

impl TextLayout for SkiaTextLayout {
    fn width(&self) -> f64 {
        return 0.0;
    }

    fn hit_test_point(&self, point: Point) -> HitTestPoint {
        HitTestPoint::default()
    }

    fn hit_test_text_position(&self, text_position: usize) -> Option<HitTestTextPosition> {
        Some(HitTestTextPosition::default())
    }
}

pub struct SkiaTextLayoutBuilder {}

impl TextLayoutBuilder for SkiaTextLayoutBuilder{
    type Out = SkiaTextLayout;

    fn build(self) -> Result<Self::Out, Error>{
        Ok(SkiaTextLayout{})
    }
}