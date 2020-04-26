// //! Text functionality for Core Graphics backend

use crate::utils::ToCg;

use core_foundation::attributed_string::*;
use core_foundation::base::*;
use core_foundation::string::*;

use core_graphics::base::*;
use core_graphics::color::CGColor;

use core_text;
use core_text::font::CTFont;
use core_text::line::CTLine;
use core_text::string_attributes::*;

use piet::kurbo::Point;

use piet::{Error, Font, FontBuilder, HitTestMetrics, HitTestPoint, HitTestTextPosition, Text, TextLayout, TextLayoutBuilder, LineMetric};

use std::marker::PhantomData;
use core_text::frame::CTFrame;
use core_text::framesetter::CTFramesetter;
use core_graphics::geometry::{CGRect, CGPoint, CGSize};

#[derive(Default)]
pub struct CoreGraphicsText<'a>(PhantomData<&'a ()>);

impl<'a> CoreGraphicsText<'a> {
    pub fn new() -> CoreGraphicsText<'a> {
        CoreGraphicsText {
            0: PhantomData::default(),
        }
    }
}

impl<'a> Text for CoreGraphicsText<'a> {
    type FontBuilder = CoreGraphicsFontBuilder;
    type Font = CoreGraphicsFont;

    type TextLayoutBuilder = CoreGraphicsTextLayoutBuilder;
    type TextLayout = CoreGraphicsTextLayout;

    fn new_font_by_name(&mut self, name: &str, size: f64) -> Self::FontBuilder {
        CoreGraphicsFontBuilder {
            name: name.to_owned(),
            pt_size: size,
        }
    }

    fn new_text_layout(&mut self, font: &Self::Font, text: &str,
                       width: impl Into<Option<f64>>, ) -> Self::TextLayoutBuilder {
        CoreGraphicsTextLayoutBuilder {
            text: text.to_owned(),
            font: font.font.clone_with_font_size(font.font.pt_size()),
            width: width.into()
        }
    }
}

pub struct CoreGraphicsFont {
    font: CTFont,
}

impl Font for CoreGraphicsFont {}

pub struct CoreGraphicsFontBuilder {
    name: String,
    pt_size: CGFloat,
}

/// Error returned when the font request is not found
#[derive(Debug)]
struct FontNotFoundError {
    font_name: String,
}

impl std::fmt::Display for FontNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unable to find font {:?}", self.font_name)
    }
}

impl std::error::Error for FontNotFoundError {}

impl From<FontNotFoundError> for piet::Error {
    fn from(error: FontNotFoundError) -> Self {
        piet::new_error(piet::ErrorKind::BackendError(Box::new(error)))
    }
}

impl FontBuilder for CoreGraphicsFontBuilder {
    type Out = CoreGraphicsFont;
    fn build(self) -> Result<Self::Out, Error> {
        match core_text::font::new_from_name(&self.name, self.pt_size) {
            Ok(font) => Ok(CoreGraphicsFont { font }),
            Err(_) => {
                // It's unclear when this can actually happen. From `CTFontCreateWithName` official
                // documentation "If all parameters cannot be matched identically, a best match is
                // found." it seems this function should always return a font instance.
                Err(FontNotFoundError {
                    font_name: self.name,
                }
                    .into())
            }
        }
    }
}

pub(crate) enum CTTextLayout {
    Line(CTLine),
    Framesetter(CTFramesetter),
}

#[derive(Clone)]
pub struct CoreGraphicsTextLayout {
    pub(crate) text: String,
    pub(crate) font: CTFont,
    pub(crate) width: Option<f64>,
    // pub(crate) black_line: CTLine,
    pub(crate) black_line_setter: CTFramesetter,
    pub(crate) black_line: CTFrame,
    pub(crate) suggested_size: CGSize,
}

impl CoreGraphicsTextLayout {
    pub(self) fn new(text: String, font: CTFont, width: Option<f64>) -> CoreGraphicsTextLayout {
        let (black_line_setter, black_line, suggested_size) = CoreGraphicsTextLayout::build_ct_line(&text, &font, width, None);
        CoreGraphicsTextLayout {
            text,
            font,
            width,
            black_line_setter,
            black_line,
            suggested_size,
        }
    }

    pub fn build_colored_line(&self, color: Option<&CGColor>) -> (CTFramesetter, CTFrame, CGSize) {
        CoreGraphicsTextLayout::build_ct_line(&self.text, &self.font, self.width, color)
    }

    fn build_ct_line(text: &str, font: &CTFont, width: Option<f64>, color: Option<&CGColor>) -> (CTFramesetter, CTFrame, CGSize) {
        let mut attributed_string = CFMutableAttributedString::new();
        attributed_string.replace_str(&CFString::new(text), CFRange::init(0, 0));

        let range = CFRange::init(0, attributed_string.char_len());

        if let Some(color) = color {
            attributed_string.set_attribute(
                range,
                unsafe { kCTForegroundColorAttributeName },
                color,
            );
        }
        attributed_string.set_attribute(range, unsafe { kCTFontAttributeName }, font);

        let max_width = if let Some(width) = width {width} else {std::f64::MAX};

        let framesetter = CTFramesetter::new_with_attributed_string(attributed_string.as_concrete_TypeRef());
        let suggested_size = framesetter.suggest_frame_size_with_constraints(range, None, CGSize::new(max_width, 1000.0));
        // println!("suggested_size {} {} {}", text, suggested_size.width, suggested_size.height);
        let path = core_graphics::path::CGPath::from_rect(CGRect::new(&CGPoint::new(0.0, 0.0), &suggested_size), None);
        let frame = framesetter.create_frame(CFRange::init(0, attributed_string.char_len()), &path);
        (framesetter, frame, suggested_size)
    }
}

impl TextLayout for CoreGraphicsTextLayout {
    fn width(&self) -> f64 {
        self.suggested_size.width
    }

    fn update_width(&mut self, new_width: impl Into<Option<f64>>) -> Result<(), Error> {
        let (black_line_setter, black_line, suggested_size) = CoreGraphicsTextLayout::build_ct_line(&self.text, &self.font, new_width.into(), None);
        self.black_line_setter = black_line_setter;
        self.black_line = black_line;
        self.suggested_size = suggested_size;
        Ok(())
    }

    fn line_text(&self, line_number: usize) -> Option<&str> {
        let lines = self.black_line.get_lines();
        let line = lines.get(line_number as CFIndex);
        if let Some(line) = line {
            let range = line.get_string_range();

            let start = range.location as usize;
            let end = (range.location + range.length) as usize;
            return self.text.get(start..end);
        }
        None
    }

    fn line_metric(&self, line_number: usize) -> Option<LineMetric> {
        None
    }

    fn line_count(&self) -> usize {
        1
    }

    fn hit_test_point(&self, point: Point) -> HitTestPoint {
        // let index = self.black_line.get_string_index_for_position(point.to_cg());

        let mut hit_test = HitTestPoint::default();
        // let is_inside = index != kCFNotFound;
        // hit_test.is_inside = is_inside;
        //
        // if is_inside {
        //     hit_test.metrics.text_position = index as usize;
        // }

        hit_test
    }

    fn hit_test_text_position(&self, text_position: usize) -> Option<HitTestTextPosition> {
        // let offset = self
        //     .black_line
        //     .get_string_offset_for_string_index(text_position as CFIndex);
        //
        // Some(HitTestTextPosition {
        //     point: Point::new(offset, 0.0),
        //     metrics: HitTestMetrics { text_position },
        // })
        None
    }
}

pub struct CoreGraphicsTextLayoutBuilder {
    text: String,
    font: CTFont,
    width: Option<f64>,
}

impl TextLayoutBuilder for CoreGraphicsTextLayoutBuilder {
    type Out = CoreGraphicsTextLayout;
    fn build(self) -> Result<Self::Out, Error> {
        Ok(CoreGraphicsTextLayout::new(self.text, self.font, self.width))
    }
}
