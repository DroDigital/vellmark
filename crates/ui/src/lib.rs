//! The interface. Everything it shows is typeset in the 95 printable ASCII characters
//! of the two typefaces: no icons, no sprites.
//!
//! The interface draws nothing itself. It fills a [`Canvas`] with glyphs and plain
//! rectangles, in pixels, and the renderer draws the canvas.

use std::borrow::Cow;
use vellmark_content::{Face, GlyphAtlas, StringId, StringTable};

/// Text that may be shown to a player. It can only be made from the string table, so
/// no text written in code can reach the screen. Development tools have their own
/// door, [`dev_text`], which does not exist in a shipping build.
pub struct Shown<'a>(Cow<'a, str>);

impl<'a> Shown<'a> {
    pub fn from_table(strings: &'a StringTable, id: StringId) -> Self {
        Self(strings.text(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Text for a development tool. Used only in modules named `dev`; the harness fails
/// the build's tests if it appears anywhere else.
#[cfg(feature = "dev")]
pub fn dev_text(text: &str) -> Shown<'_> {
    Shown(Cow::Borrowed(text))
}

/// Linear RGBA, not premultiplied.
pub type Colour = [f32; 4];

#[derive(Clone, Copy, Debug)]
pub struct TextStyle {
    pub face: Face,
    /// Height of an em in pixels.
    pub size: f32,
    pub colour: Colour,
}

/// One thing to draw: a glyph's cell or a plain rectangle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quad {
    /// Left, top, width, height, in pixels.
    pub rect: [f32; 4],
    pub colour: Colour,
    /// The atlas layer of the glyph, or [`Quad::SOLID`] for a plain rectangle.
    pub layer: u32,
}

impl Quad {
    pub const SOLID: u32 = u32::MAX;
}

/// What the interface wants drawn this frame.
#[derive(Default, Debug)]
pub struct Canvas {
    pub quads: Vec<Quad>,
}

impl Canvas {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.quads.clear();
    }

    pub fn rect(&mut self, rect: [f32; 4], colour: Colour) {
        self.quads.push(Quad { rect, colour, layer: Quad::SOLID });
    }

    /// Sets a line of text with its left edge at `x` and its baseline at `baseline`.
    /// Returns the width it took.
    pub fn text(&mut self, atlas: &GlyphAtlas, x: f32, baseline: f32, style: &TextStyle, text: &Shown<'_>) -> f32 {
        let face = atlas.face(style.face);
        let cell = style.size * face.cell_ems;
        let mut pen = x;
        for c in text.as_str().chars() {
            let glyph = atlas.glyph(style.face, c);
            if c != ' ' {
                self.quads.push(Quad {
                    rect: [
                        pen + glyph.centre * style.size - cell * 0.5,
                        baseline - cell * face.baseline,
                        cell,
                        cell,
                    ],
                    colour: style.colour,
                    layer: GlyphAtlas::layer(style.face, c),
                });
            }
            pen += glyph.advance * style.size;
        }
        pen - x
    }

    /// The width a line of text would take, without drawing it.
    pub fn measure(atlas: &GlyphAtlas, style: &TextStyle, text: &Shown<'_>) -> f32 {
        text.as_str().chars().map(|c| atlas.glyph(style.face, c).advance * style.size).sum()
    }

    /// The distance from one baseline to the next.
    pub fn line_height(atlas: &GlyphAtlas, style: &TextStyle) -> f32 {
        let face = atlas.face(style.face);
        (face.ascent - face.descent) * style.size * 1.15
    }
}
