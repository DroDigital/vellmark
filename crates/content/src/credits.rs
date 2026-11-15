//! The credits, as baked from `assets/data/credits.ron` and `THIRD_PARTY_NOTICES.txt`:
//! who made the game, the typefaces, and every crate it is built with and its license.
//! These are names, not words to translate. One table, `credits.table`.

pub const CREDITS_KEY: &str = "credits.table";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Credits {
    pub made_by: Vec<String>,
    /// Each typeface's name, author, and license.
    pub typefaces: Vec<(String, String, String)>,
    /// Each crate, as `name version`, and its license.
    pub crates: Vec<(String, String)>,
}

impl Credits {
    const MAGIC: &'static [u8; 8] = b"VLMKCRD1";

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Self::MAGIC.to_vec();
        let s = |out: &mut Vec<u8>, t: &str| {
            out.extend_from_slice(&(t.len() as u32).to_le_bytes());
            out.extend_from_slice(t.as_bytes());
        };
        out.extend_from_slice(&(self.made_by.len() as u32).to_le_bytes());
        for m in &self.made_by {
            s(&mut out, m);
        }
        out.extend_from_slice(&(self.typefaces.len() as u32).to_le_bytes());
        for (a, b, c) in &self.typefaces {
            s(&mut out, a);
            s(&mut out, b);
            s(&mut out, c);
        }
        out.extend_from_slice(&(self.crates.len() as u32).to_le_bytes());
        for (a, b) in &self.crates {
            s(&mut out, a);
            s(&mut out, b);
        }
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 8 || &bytes[..8] != Self::MAGIC {
            return Err("not the credits".into());
        }
        let mut r = R(bytes, 8);
        let mut c = Credits::default();
        for _ in 0..r.count()? {
            c.made_by.push(r.text()?);
        }
        for _ in 0..r.count()? {
            c.typefaces.push((r.text()?, r.text()?, r.text()?));
        }
        for _ in 0..r.count()? {
            c.crates.push((r.text()?, r.text()?));
        }
        Ok(c)
    }
}

struct R<'a>(&'a [u8], usize);

impl R<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], String> {
        let b = self.0.get(self.1..self.1 + n).ok_or("the credits are cut short")?;
        self.1 += n;
        Ok(b)
    }
    fn count(&mut self) -> Result<usize, String> {
        let n = u32::from_le_bytes(self.take(4)?.try_into().unwrap()) as usize;
        if n > self.0.len() {
            return Err("the credits are cut short".into());
        }
        Ok(n)
    }
    fn text(&mut self) -> Result<String, String> {
        let n = self.count()?;
        String::from_utf8(self.take(n)?.to_vec()).map_err(|_| "a credit is not text".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_credits_round_trip() {
        let c = Credits { made_by: vec!["Someone".into()], typefaces: vec![("Tiny5".into(), "Authors".into(), "OFL".into())], crates: vec![("glam 0.30".into(), "MIT OR Apache-2.0".into())] };
        assert_eq!(Credits::from_bytes(&c.to_bytes()).unwrap(), c);
    }
}
