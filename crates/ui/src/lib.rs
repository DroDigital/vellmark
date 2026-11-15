//! The interface. Everything it shows is typeset in the 95 printable ASCII characters
//! of the two typefaces: no icons, no sprites.
//!
//! The interface draws nothing itself. It fills a [`Canvas`] with glyphs and plain
//! rectangles, in pixels, and the renderer draws the canvas.

pub mod reading_view;
pub mod hud;
pub mod kit;
pub mod item_text;
pub mod inventory_view;
pub mod menu;
pub mod pages;
pub mod dialogue_view;
pub mod title;
pub mod photo;
pub mod captions;
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

    /// Its first `chars` characters: a line as far as it is written.
    pub fn prefix(&self, chars: usize) -> Shown<'static> {
        Shown(Cow::Owned(self.0.chars().take(chars).collect()))
    }

    /// As far as its first `letters` letters, not counting spaces: a Page as far as it
    /// has written itself.
    pub fn prefix_letters(&self, letters: usize) -> Shown<'static> {
        let mut left = letters;
        let mut end = 0;
        for (i, c) in self.0.char_indices() {
            if !c.is_whitespace() {
                if left == 0 {
                    break;
                }
                left -= 1;
            }
            end = i + c.len_utf8();
        }
        Shown(Cow::Owned(self.0[..end].trim_end().to_string()))
    }

    /// Its first `chars` characters, followed by `tail` (itself from the table): for a line
    /// that will not fit its place.
    pub fn clipped(&self, chars: usize, tail: &Shown<'_>) -> Shown<'static> {
        let kept: String = self.0.chars().take(chars).collect();
        Shown(Cow::Owned(format!("{}{}", kept.trim_end(), tail.as_str())))
    }

    /// A string from the table with its `{name}` places filled: each with other shown
    /// text, or with a number. Neither can carry words written in code.
    pub fn filled(strings: &'a StringTable, id: StringId, parts: &[(&str, Part<'_>)]) -> Self {
        let mut text = strings.text(id).into_owned();
        for (name, part) in parts {
            let with = match part {
                Part::Shown(s) => s.as_str().to_owned(),
                Part::Signed(n) => format!("{n:+}"),
                Part::Number(n) => format!("{n}"),
                Part::Repeated(s, n) => s.as_str().repeat(*n),
                Part::Nothing => String::new(),
            };
            text = text.replace(&format!("{{{name}}}"), &with);
        }
        Self(Cow::Owned(text))
    }
}

/// What fills a place in a shown string.
pub enum Part<'a> {
    Shown(Shown<'a>),
    /// A whole number shown with its sign: `+3`.
    Signed(i32),
    Number(u32),
    /// Shown text, so many times over: the strokes of a slider.
    Repeated(Shown<'a>, usize),
    Nothing,
}

/// Text for a development tool. Used only in modules named `dev`; the harness fails
/// the build's tests if it appears anywhere else.
#[cfg(feature = "dev")]
pub fn dev_text(text: &str) -> Shown<'_> {
    Shown(Cow::Borrowed(text))
}

/// Linear RGBA, not premultiplied.
pub type Colour = [f32; 4];

static LETTER_SCALE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0x3f80_0000);

/// The letter-size setting: every piece of interface text is set this much larger, 1 to
/// 1.5. An accessibility setting.
pub fn set_letter_scale(scale: f32) {
    LETTER_SCALE.store(scale.clamp(1.0, 1.5).to_bits(), std::sync::atomic::Ordering::Relaxed);
}

fn letter_scale() -> f32 {
    f32::from_bits(LETTER_SCALE.load(std::sync::atomic::Ordering::Relaxed))
}

impl TextStyle {
    /// As set, at the letter-size setting.
    fn scaled(&self) -> TextStyle {
        TextStyle { size: self.size * letter_scale(), ..*self }
    }
}

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
        let style = &style.scaled();
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

    /// Sets text as a paragraph, wrapped at word breaks to `width`, its first baseline
    /// at `top` plus one line. Returns the height it took.
    pub fn paragraph(&mut self, atlas: &GlyphAtlas, x: f32, top: f32, width: f32, style: &TextStyle, text: &Shown<'_>) -> f32 {
        let style = &style.scaled();
        let line = Self::line_height(atlas, style);
        let space = atlas.glyph(style.face, ' ').advance * style.size;
        let mut pen = 0.0f32;
        let mut baseline = top + line * 0.85;
        for word in text.as_str().split_whitespace() {
            let w: f32 = word.chars().map(|c| atlas.glyph(style.face, c).advance * style.size).sum();
            if pen > 0.0 && pen + w > width {
                pen = 0.0;
                baseline += line;
            }
            let mut at = x + pen;
            for c in word.chars() {
                let glyph = atlas.glyph(style.face, c);
                let face = atlas.face(style.face);
                let cell = style.size * face.cell_ems;
                self.quads.push(Quad {
                    rect: [at + glyph.centre * style.size - cell * 0.5, baseline - cell * face.baseline, cell, cell],
                    colour: style.colour,
                    layer: GlyphAtlas::layer(style.face, c),
                });
                at += glyph.advance * style.size;
            }
            pen += w + space;
        }
        baseline - top + line * 0.3
    }

    /// The width a line of text would take, without drawing it.
    pub fn measure(atlas: &GlyphAtlas, style: &TextStyle, text: &Shown<'_>) -> f32 {
        let style = &style.scaled();
        text.as_str().chars().map(|c| atlas.glyph(style.face, c).advance * style.size).sum()
    }

    /// The distance from one baseline to the next.
    pub fn line_height(atlas: &GlyphAtlas, style: &TextStyle) -> f32 {
        let style = &style.scaled();
        let face = atlas.face(style.face);
        (face.ascent - face.descent) * style.size * 1.15
    }
}
