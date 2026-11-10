//! The glyph atlas: both typefaces, all 95 printable ASCII characters of each, as
//! signed distance fields. One atlas serves every surface, the interface, flying words,
//! and damage letters.
//!
//! Every glyph sits centred in a square cell. A texel holds a multi-channel distance
//! in red, green, and blue, for sharp corners at any size, and the true distance in
//! alpha, for small sizes and for weight. 0.5 is the edge; inside is greater.
//!
//! ```text
//! magic "VLMKGLY1"
//! cell u32            side of a cell in texels at the largest level
//! mips u32            number of levels stored, each half the side of the last
//! range f32           distance covered by 0..1, in texels of the largest level
//! faces u32           2
//! for each face:      ascent f32, descent f32 (em), cell_ems f32 (ems across a cell),
//!                     baseline f32 (down the cell, 0 to 1)
//!   for each of 95:   advance f32, centre f32 (em), min_cover f32, max_cover f32,
//!                     TONE_STEPS thresholds f32
//! texels              RGBA8, for each level, for each face, for each glyph
//! ```

pub const GLYPHS_PER_FACE: usize = 95;
pub const FACES: usize = 2;
/// Entries in each glyph's lookup from target coverage to distance threshold.
pub const TONE_STEPS: usize = 16;
const MAGIC: &[u8; 8] = b"VLMKGLY1";

/// The two typefaces, in the order they are stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Face {
    /// Tiny5, the pixel face, for cells drawn small.
    Small = 0,
    /// IM Fell English, the printed face, for cells drawn large and for books.
    Large = 1,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GlyphInfo {
    /// How far the pen moves after this glyph, in ems.
    pub advance: f32,
    /// Where the middle of the cell sits, in ems to the right of the pen.
    pub centre: f32,
    /// The least and most of the cell this glyph can cover by shifting its threshold.
    pub min_cover: f32,
    pub max_cover: f32,
    /// The threshold that makes the glyph cover `i / (TONE_STEPS - 1)` of its cell,
    /// clamped to what it can reach. 0.5 draws the glyph as designed.
    pub tone: [f32; TONE_STEPS],
}

#[derive(Clone, Debug, PartialEq)]
pub struct FaceInfo {
    pub ascent: f32,
    pub descent: f32,
    /// Ems across one cell. A glyph drawn with an em of `s` pixels fills a cell of
    /// `s * cell_ems` pixels.
    pub cell_ems: f32,
    /// How far down the cell the baseline lies, 0 at the top to 1 at the bottom.
    pub baseline: f32,
    pub glyphs: Vec<GlyphInfo>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GlyphAtlas {
    pub cell: u32,
    pub mips: u32,
    pub range: f32,
    pub faces: Vec<FaceInfo>,
    /// RGBA8 texels: for each level, for each face, for each glyph.
    pub texels: Vec<u8>,
}

/// The index of a character within a face, or `None` if it is not printable ASCII.
pub fn glyph_index(c: char) -> Option<usize> {
    (' '..='~').contains(&c).then(|| c as usize - 32)
}

impl GlyphAtlas {
    pub fn layers(&self) -> u32 {
        (FACES * GLYPHS_PER_FACE) as u32
    }

    /// The array layer of a character in a face.
    pub fn layer(face: Face, c: char) -> u32 {
        (face as usize * GLYPHS_PER_FACE + glyph_index(c).unwrap_or(('?' as usize) - 32)) as u32
    }

    pub fn glyph(&self, face: Face, c: char) -> &GlyphInfo {
        &self.faces[face as usize].glyphs[glyph_index(c).unwrap_or(('?' as usize) - 32)]
    }

    pub fn face(&self, face: Face) -> &FaceInfo {
        &self.faces[face as usize]
    }

    /// The byte range of one level, holding every layer back to back.
    pub fn level_range(&self, level: u32) -> std::ops::Range<usize> {
        let mut start = 0usize;
        for l in 0..level {
            let side = (self.cell >> l) as usize;
            start += side * side * 4 * self.layers() as usize;
        }
        let side = (self.cell >> level) as usize;
        start..start + side * side * 4 * self.layers() as usize
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.texels.len() + 16 * 1024);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&self.cell.to_le_bytes());
        out.extend_from_slice(&self.mips.to_le_bytes());
        out.extend_from_slice(&self.range.to_le_bytes());
        out.extend_from_slice(&(self.faces.len() as u32).to_le_bytes());
        for face in &self.faces {
            for v in [face.ascent, face.descent, face.cell_ems, face.baseline] {
                out.extend_from_slice(&v.to_le_bytes());
            }
            for g in &face.glyphs {
                for v in [g.advance, g.centre, g.min_cover, g.max_cover] {
                    out.extend_from_slice(&v.to_le_bytes());
                }
                for v in g.tone {
                    out.extend_from_slice(&v.to_le_bytes());
                }
            }
        }
        out.extend_from_slice(&self.texels);
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        struct Reader<'a>(&'a [u8], usize);
        impl Reader<'_> {
            fn take(&mut self, n: usize) -> Result<&[u8], &'static str> {
                let end = self.1.checked_add(n).filter(|e| *e <= self.0.len()).ok_or("glyph atlas is cut short")?;
                let s = &self.0[self.1..end];
                self.1 = end;
                Ok(s)
            }
            fn u32(&mut self) -> Result<u32, &'static str> {
                Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
            }
            fn f32(&mut self) -> Result<f32, &'static str> {
                Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
            }
        }
        let mut r = Reader(bytes, 0);
        if r.take(8)? != MAGIC {
            return Err("not a glyph atlas");
        }
        let cell = r.u32()?;
        let mips = r.u32()?;
        let range = r.f32()?;
        let face_count = r.u32()? as usize;
        if face_count != FACES || cell == 0 || !cell.is_power_of_two() || mips == 0 || (cell >> (mips - 1)) == 0 {
            return Err("glyph atlas header is not valid");
        }
        let mut faces = Vec::with_capacity(face_count);
        for _ in 0..face_count {
            let (ascent, descent, cell_ems, baseline) = (r.f32()?, r.f32()?, r.f32()?, r.f32()?);
            let mut glyphs = Vec::with_capacity(GLYPHS_PER_FACE);
            for _ in 0..GLYPHS_PER_FACE {
                let (advance, centre, min_cover, max_cover) = (r.f32()?, r.f32()?, r.f32()?, r.f32()?);
                let mut tone = [0.0; TONE_STEPS];
                for t in &mut tone {
                    *t = r.f32()?;
                }
                glyphs.push(GlyphInfo { advance, centre, min_cover, max_cover, tone });
            }
            faces.push(FaceInfo { ascent, descent, cell_ems, baseline, glyphs });
        }
        let texels = bytes[r.1..].to_vec();
        let atlas = Self { cell, mips, range, faces, texels };
        if atlas.level_range(mips - 1).end != atlas.texels.len() {
            return Err("glyph atlas texels have the wrong length");
        }
        Ok(atlas)
    }
}
