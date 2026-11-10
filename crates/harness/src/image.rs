use std::path::Path;

/// An 8-bit RGBA image, top row first.
#[derive(Clone, PartialEq, Eq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl Image {
    pub fn new(width: u32, height: u32, rgba: Vec<u8>) -> Self {
        assert_eq!(rgba.len(), (width * height * 4) as usize);
        Self { width, height, rgba }
    }

    pub fn filled(width: u32, height: u32, colour: [u8; 4]) -> Self {
        Self { width, height, rgba: colour.repeat((width * height) as usize) }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        let file = std::fs::File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), self.width, self.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer.write_image_data(&self.rgba).map_err(|e| e.to_string())
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let file = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
        decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
        let mut reader = decoder.read_info().map_err(|e| format!("{}: {e}", path.display()))?;
        let size = reader.output_buffer_size().ok_or("image too large")?;
        let mut buffer = vec![0; size];
        let info = reader.next_frame(&mut buffer).map_err(|e| format!("{}: {e}", path.display()))?;
        buffer.truncate(info.buffer_size());
        let rgba = match info.color_type {
            png::ColorType::Rgba => buffer,
            png::ColorType::Rgb => buffer.chunks_exact(3).flat_map(|p| [p[0], p[1], p[2], 255]).collect(),
            png::ColorType::Grayscale => buffer.iter().flat_map(|&g| [g, g, g, 255]).collect(),
            png::ColorType::GrayscaleAlpha => buffer.chunks_exact(2).flat_map(|p| [p[0], p[0], p[0], p[1]]).collect(),
            other => return Err(format!("{}: unsupported colour type {other:?}", path.display())),
        };
        Ok(Self::new(info.width, info.height, rgba))
    }

    /// Luminance of each pixel, 0 to 1, in display space.
    pub fn luminance(&self) -> Vec<f32> {
        self.rgba
            .chunks_exact(4)
            .map(|p| (0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32) / 255.0)
            .collect()
    }

    pub fn blit(&mut self, source: &Image, x: u32, y: u32) {
        for row in 0..source.height.min(self.height.saturating_sub(y)) {
            let w = source.width.min(self.width.saturating_sub(x)) as usize * 4;
            let from = (row * source.width * 4) as usize;
            let to = (((y + row) * self.width + x) * 4) as usize;
            self.rgba[to..to + w].copy_from_slice(&source.rgba[from..from + w]);
        }
    }

    /// Shrinks by a whole factor, averaging each block. For contact sheets and
    /// reference images.
    pub fn shrink(&self, factor: u32) -> Image {
        if factor <= 1 {
            return self.clone();
        }
        let (w, h) = (self.width / factor, self.height / factor);
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                let mut sum = [0u32; 4];
                for dy in 0..factor {
                    for dx in 0..factor {
                        let at = (((y * factor + dy) * self.width + x * factor + dx) * 4) as usize;
                        for c in 0..4 {
                            sum[c] += self.rgba[at + c] as u32;
                        }
                    }
                }
                let n = factor * factor;
                rgba.extend(sum.map(|s| ((s + n / 2) / n) as u8));
            }
        }
        Image::new(w, h, rgba)
    }
}

/// How two images of the same size differ.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Difference {
    /// Mean absolute difference over all colour channels, 0 to 1.
    pub mean: f32,
    /// Largest difference in any channel of any pixel, 0 to 1.
    pub max: f32,
    /// Fraction of pixels in which some channel differs by more than 8 of 255.
    pub changed: f32,
}

pub fn difference(a: &Image, b: &Image) -> Result<Difference, String> {
    difference_masked(a, b, None)
}

/// As [`difference`], counting only pixels where `mask` is true.
pub fn difference_masked(a: &Image, b: &Image, mask: Option<&[bool]>) -> Result<Difference, String> {
    if (a.width, a.height) != (b.width, b.height) {
        return Err(format!("sizes differ: {}x{} and {}x{}", a.width, a.height, b.width, b.height));
    }
    let (mut sum, mut max, mut changed, mut counted) = (0u64, 0u8, 0u64, 0u64);
    for (i, (p, q)) in a.rgba.chunks_exact(4).zip(b.rgba.chunks_exact(4)).enumerate() {
        if mask.is_some_and(|m| !m[i]) {
            continue;
        }
        counted += 1;
        let mut worst = 0u8;
        for c in 0..3 {
            let d = p[c].abs_diff(q[c]);
            sum += d as u64;
            worst = worst.max(d);
        }
        max = max.max(worst);
        if worst > 8 {
            changed += 1;
        }
    }
    if counted == 0 {
        return Ok(Difference { mean: 0.0, max: 0.0, changed: 0.0 });
    }
    Ok(Difference {
        mean: sum as f32 / (counted as f32 * 3.0 * 255.0),
        max: max as f32 / 255.0,
        changed: changed as f32 / counted as f32,
    })
}

/// Lays images out in a grid with a thin gap, all at the size of the first.
pub fn contact_sheet(images: &[Image], columns: u32) -> Image {
    assert!(!images.is_empty() && columns > 0);
    let (w, h) = (images[0].width, images[0].height);
    let rows = (images.len() as u32).div_ceil(columns);
    let gap = 2;
    let mut sheet =
        Image::filled(columns * w + (columns + 1) * gap, rows * h + (rows + 1) * gap, [18, 18, 18, 255]);
    for (i, image) in images.iter().enumerate() {
        let (cx, cy) = (i as u32 % columns, i as u32 / columns);
        sheet.blit(image, gap + cx * (w + gap), gap + cy * (h + gap));
    }
    sheet
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_round_trip() {
        let image = Image::new(3, 2, (0..24).map(|i| i as u8 * 10).collect());
        let path = std::env::temp_dir().join(format!("vellmark-image-{}.png", std::process::id()));
        image.save(&path).unwrap();
        assert!(Image::load(&path).unwrap() == image);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn difference_measures() {
        let a = Image::filled(4, 4, [100, 100, 100, 255]);
        let mut b = a.clone();
        assert_eq!(difference(&a, &b).unwrap(), Difference { mean: 0.0, max: 0.0, changed: 0.0 });
        b.rgba[0] = 151;
        let d = difference(&a, &b).unwrap();
        assert!((d.max - 0.2).abs() < 1e-6);
        assert!((d.changed - 1.0 / 16.0).abs() < 1e-6);
        let mask: Vec<bool> = (0..16).map(|i| i != 0).collect();
        assert_eq!(difference_masked(&a, &b, Some(&mask)).unwrap().max, 0.0);
        assert!(difference(&a, &Image::filled(2, 2, [0; 4])).is_err());
    }

    #[test]
    fn shrink_averages() {
        let mut image = Image::filled(2, 2, [0, 0, 0, 255]);
        image.rgba[0] = 200;
        assert_eq!(image.shrink(2).rgba, vec![50, 0, 0, 255]);
    }

    #[test]
    fn contact_sheet_size() {
        let tile = Image::filled(10, 5, [255; 4]);
        let sheet = contact_sheet(&[tile.clone(), tile.clone(), tile], 2);
        assert_eq!((sheet.width, sheet.height), (26, 16));
    }
}
