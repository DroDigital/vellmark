//! Authored text set onto surfaces. Text runs in reading order across a surface's
//! cells, breaks only between words, and flows around whatever interrupts the surface:
//! a doorway, the end of a wall.

/// A rectangle of surface in cells: columns `x0..x1`, rows `y0..y1` counted upwards.
#[derive(Clone, Copy, Debug)]
pub struct CellRect {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

/// One page being laid out: the block the text may fill, and which of its cells are
/// really on the surface.
pub struct Layout {
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
    /// Row by row from the top. True where the cell lies on the surface.
    covered: Vec<bool>,
    /// Row by row from the top. 0 where no text has been set.
    pub chars: Vec<u8>,
    /// Rows from one line of text to the next. 1 sets every row; 2 leaves a blank row
    /// between lines, which is what makes a block of text easy to read.
    pub leading: u32,
}

impl Layout {
    /// `block` is where text may go. `surface` is the faces it lies on, in cells but
    /// not rounded to them: left, bottom, right, top. A cell takes text only if all of
    /// it is on the surface. Its corners may lie on different faces, so a cell across
    /// the seam of two wall pieces counts, and one hanging over a doorway does not.
    pub fn on_surface(block: CellRect, surface: &[(f32, f32, f32, f32)]) -> Self {
        let mut layout = Self::new(block, &[]);
        let inside = |x: f32, y: f32| surface.iter().any(|r| x >= r.0 && x <= r.2 && y >= r.1 && y <= r.3);
        for row in 0..layout.height as i32 {
            let y = (block.y1 - 1 - row) as f32;
            for col in 0..layout.width as i32 {
                let x = (block.x0 + col) as f32;
                let (a, b) = (0.04, 0.96);
                layout.covered[(row * layout.width as i32 + col) as usize] =
                    inside(x + a, y + a) && inside(x + b, y + a) && inside(x + a, y + b) && inside(x + b, y + b);
            }
        }
        layout
    }

    /// As [`Layout::on_surface`], for faces that are whole numbers of cells.
    pub fn new(block: CellRect, surface: &[CellRect]) -> Self {
        let (width, height) = ((block.x1 - block.x0).max(0) as u32, (block.y1 - block.y0).max(0) as u32);
        let mut covered = vec![false; (width * height) as usize];
        for row in 0..height as i32 {
            let y = block.y1 - 1 - row;
            for col in 0..width as i32 {
                let x = block.x0 + col;
                covered[(row * width as i32 + col) as usize] =
                    surface.iter().any(|r| x >= r.x0 && x < r.x1 && y >= r.y0 && y < r.y1);
            }
        }
        Self { left: block.x0, top: block.y1 - 1, width, height, covered, chars: vec![0; (width * height) as usize], leading: 1 }
    }

    /// Sets words from `words` into the page, starting where the last call stopped.
    /// Returns how many words were placed.
    pub fn flow(&mut self, words: &[&str], cursor: &mut (u32, u32)) -> usize {
        let mut placed = 0;
        'words: for word in words {
            let len = word.len() as u32;
            loop {
                let (col, row) = *cursor;
                if row >= self.height {
                    break 'words;
                }
                if col + len > self.width {
                    *cursor = (0, row + self.leading);
                    continue;
                }
                let at = (row * self.width + col) as usize;
                // The word needs an unbroken run of surface, and a clear cell before it
                // unless it starts a run.
                let run_ok = (0..len as usize).all(|i| self.covered[at + i]);
                if !run_ok {
                    // Skip past the break in the surface and try again on the same row.
                    let next = (col + 1..self.width).find(|&c| {
                        self.covered[(row * self.width + c) as usize] && !self.covered[(row * self.width + c - 1) as usize]
                    });
                    *cursor = match next {
                        Some(c) if c > col => (c, row),
                        _ => (0, row + self.leading),
                    };
                    continue;
                }
                self.chars[at..at + len as usize].copy_from_slice(word.as_bytes());
                // One blank cell after each word. A blank is a space, not grain.
                let after = col + len;
                if after < self.width && self.covered[(row * self.width + after) as usize] {
                    self.chars[(row * self.width + after) as usize] = b' ';
                }
                *cursor = (after + 1, row);
                placed += 1;
                break;
            }
        }
        placed
    }

    /// Clears the grain from every cell of the block that the text has passed: the
    /// rows it used, the blank rows between its lines, and a row of margin above and
    /// below. A passage then sits on a quiet plate of its own, not in the wall's noise.
    pub fn clear_plate(&mut self) {
        let width = self.width as usize;
        let used: Vec<usize> = (0..self.height as usize)
            .filter(|r| self.chars[r * width..(r + 1) * width].iter().any(|&c| c != 0))
            .collect();
        let (Some(&first), Some(&last)) = (used.first(), used.last()) else { return };
        let (from, to) = (first.saturating_sub(1), (last + 1).min(self.height as usize - 1));
        for row in from..=to {
            for col in 0..width {
                let i = row * width + col;
                if self.chars[i] == 0 && self.covered[i] {
                    self.chars[i] = b' ';
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(layout: &Layout) -> Vec<String> {
        layout
            .chars
            .chunks(layout.width as usize)
            .map(|r| r.iter().map(|&c| if c == 0 { '.' } else { c as char }).collect())
            .collect()
    }

    #[test]
    fn words_wrap_at_the_edge_and_are_never_split() {
        let block = CellRect { x0: 0, y0: 0, x1: 10, y1: 3 };
        let mut layout = Layout::new(block, &[block]);
        let words: Vec<&str> = "THIS WALL STANDS HERE NOW".split(' ').collect();
        let placed = layout.flow(&words, &mut (0, 0));
        assert_eq!(placed, 5);
        assert_eq!(rows(&layout), vec!["THIS WALL ", "STANDS ...", "HERE NOW ."]);
    }

    #[test]
    fn text_flows_around_a_doorway() {
        // A wall ten cells wide and three high, with a door in columns 4..6 of the
        // lower two rows.
        let block = CellRect { x0: 0, y0: 0, x1: 10, y1: 3 };
        let surface = [
            CellRect { x0: 0, y0: 0, x1: 4, y1: 3 },
            CellRect { x0: 6, y0: 0, x1: 10, y1: 3 },
            CellRect { x0: 4, y0: 2, x1: 6, y1: 3 },
        ];
        let mut layout = Layout::new(block, &surface);
        let words: Vec<&str> = "A DOOR IS HUNG IN IT NOW OK".split(' ').collect();
        layout.flow(&words, &mut (0, 0));
        let out = rows(&layout);
        assert_eq!(out[0], "A DOOR IS ");
        // On the rows the door cuts, no word crosses columns 4 and 5.
        assert_eq!(&out[1][4..6], "..");
        assert_eq!(&out[2][4..6], "..");
        assert!(out[1].starts_with("HUNG") && out[1][6..].starts_with("IN"));
    }

    #[test]
    fn a_cell_across_the_seam_of_two_faces_takes_text() {
        let block = CellRect { x0: 0, y0: 0, x1: 8, y1: 1 };
        // Two faces that meet in the middle of column 3, and a gap after 6.5.
        let layout = Layout::on_surface(block, &[(0.0, 0.0, 3.5, 1.0), (3.5, 0.0, 6.5, 1.0)]);
        assert_eq!(layout.covered, vec![true, true, true, true, true, true, false, false]);
    }

    #[test]
    fn leading_leaves_blank_rows_and_the_plate_is_cleared() {
        let block = CellRect { x0: 0, y0: 0, x1: 6, y1: 5 };
        let mut layout = Layout::new(block, &[block]);
        layout.leading = 2;
        layout.flow(&["ONE", "TWO"], &mut (0, 1));
        layout.clear_plate();
        assert_eq!(rows(&layout), vec!["      ", "ONE   ", "      ", "TWO   ", "      "]);
    }

    #[test]
    fn text_that_does_not_fit_stops() {
        let block = CellRect { x0: 0, y0: 0, x1: 4, y1: 1 };
        let mut layout = Layout::new(block, &[block]);
        assert_eq!(layout.flow(&["ONE", "TWO"], &mut (0, 0)), 1);
    }
}
