//! The end card, after the Line: the game's name, what the full game will be, and the
//! owner's two links (where to wishlist it and where to say what you thought), each opened
//! in the browser only when the player chooses it; then keep playing, or start again from
//! the beginning. A link the owner has not given is not offered.

use vellmark_content::{sid, Face, GlyphAtlas, StringId, StringTable};

use crate::kit::{self, Focus, Row};
use crate::menu::MenuInput;
use crate::{Canvas, Shown, TextStyle};

/// What a choice on the card does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    KeepPlaying,
    Wishlist,
    Feedback,
    StartAgain,
}

/// What the game should do this frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EndOutcome {
    None,
    /// Close the card and play on.
    Close,
    /// Open this address in the browser.
    Open(String),
    StartAgain,
}

/// The card's state while it is up: its focus, its age, and the last note shown.
#[derive(Clone, Debug, Default)]
pub struct EndCard {
    pub focus: Focus,
    pub since: f32,
    pub note: Option<StringId>,
}

/// An owner's link from the string table, if it is given.
pub fn link(strings: &StringTable, key: StringId) -> Option<&str> {
    strings.get(key).map(str::trim).filter(|l| l.starts_with("https://"))
}

/// The choices the card offers, in order.
pub fn choices(strings: &StringTable) -> Vec<Choice> {
    let mut out = vec![Choice::KeepPlaying];
    if link(strings, sid!("end.link.wishlist")).is_some() {
        out.push(Choice::Wishlist);
    }
    if link(strings, sid!("end.link.feedback")).is_some() {
        out.push(Choice::Feedback);
    }
    out.push(Choice::StartAgain);
    out
}

/// Seconds before the choices answer, so a press meant for the fight does not dismiss it.
pub const SETTLE: f32 = 1.5;

impl EndCard {
    pub fn update(&mut self, input: MenuInput, strings: &StringTable, dt: f32) -> EndOutcome {
        self.since += dt;
        let options = choices(strings);
        self.focus.step(input.step, options.len(), options.len());
        if self.since < SETTLE {
            return EndOutcome::None;
        }
        if input.back {
            return EndOutcome::Close;
        }
        if !input.confirm {
            return EndOutcome::None;
        }
        match options.get(self.focus.at).copied().unwrap_or(Choice::KeepPlaying) {
            Choice::KeepPlaying => EndOutcome::Close,
            Choice::StartAgain => EndOutcome::StartAgain,
            Choice::Wishlist => link(strings, sid!("end.link.wishlist")).map_or(EndOutcome::None, |l| EndOutcome::Open(l.into())),
            Choice::Feedback => link(strings, sid!("end.link.feedback")).map_or(EndOutcome::None, |l| EndOutcome::Open(l.into())),
        }
    }
}

pub struct EndCardView<'a> {
    pub atlas: &'a GlyphAtlas,
    pub strings: &'a StringTable,
    pub size: (u32, u32),
    pub density: f32,
}

impl EndCardView<'_> {
    /// `hint` is the line along the foot that says how to choose.
    pub fn draw(&self, canvas: &mut Canvas, card: &EndCard, hint: Option<Shown<'_>>) {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let d = self.density.max(1.0);
        // The world fades back behind the card over its first second.
        let fade = (card.since / 1.0).clamp(0.0, 1.0);
        canvas.rect([0.0, 0.0, w, h], [0.0, 0.0, 0.0, 0.72 * fade]);
        let centred = |canvas: &mut Canvas, y: f32, style: &TextStyle, text: &Shown<'_>| {
            let tw = Canvas::measure(self.atlas, style, text);
            canvas.text(self.atlas, (w - tw) * 0.5, y, style, text);
        };
        let a = |c: [f32; 4]| [c[0], c[1], c[2], c[3] * fade];
        let name = TextStyle { face: Face::Large, size: (64.0 * d).round(), colour: a(kit::ink()) };
        let line = TextStyle { face: Face::Large, size: (22.0 * d).round(), colour: a(kit::gold()) };
        let body = TextStyle { face: Face::Large, size: (18.0 * d).round(), colour: a(kit::ink()) };
        let small = TextStyle { face: Face::Small, size: (15.0 * d).round(), colour: a(kit::faint()) };
        let mut y = h * 0.24;
        centred(canvas, y, &name, &Shown::from_table(self.strings, sid!("title.name")));
        y += 54.0 * d;
        centred(canvas, y, &line, &Shown::from_table(self.strings, sid!("end.line")));
        y += 44.0 * d;
        let width = (w * 0.6).min(640.0 * d);
        let more = Shown::from_table(self.strings, sid!("end.more"));
        let used = canvas.paragraph(self.atlas, (w - width) * 0.5, y, width, &body, &more);
        y += used + 30.0 * d;
        centred(canvas, y, &TextStyle { colour: a(kit::faint()), ..body }, &Shown::from_table(self.strings, sid!("end.thanks")));
        y += 40.0 * d;
        if card.since < SETTLE * 0.6 {
            return;
        }
        let options = choices(self.strings);
        let rows: Vec<Row<'_>> = options
            .iter()
            .map(|c| {
                let key = match c {
                    Choice::KeepPlaying => sid!("end.keep"),
                    Choice::Wishlist => sid!("end.wishlist"),
                    Choice::Feedback => sid!("end.feedback"),
                    Choice::StartAgain => sid!("end.again"),
                };
                Row { text: Shown::from_table(self.strings, key), marked: false, colour: kit::ink() }
            })
            .collect();
        let lw = (360.0 * d).min(w * 0.8);
        let row = 19.0;
        let lh = Canvas::line_height(self.atlas, &TextStyle { size: (row * d).round(), ..body }) * 1.15;
        kit::list(canvas, self.atlas, self.strings, [(w - lw) * 0.5, y, lw, lh * rows.len() as f32 + 4.0 * d], d, &rows, card.focus, row);
        y += lh * rows.len() as f32 + 24.0 * d;
        if let Some(note) = card.note {
            centred(canvas, y, &TextStyle { colour: a(kit::gold()), ..small }, &Shown::from_table(self.strings, note));
            y += 24.0 * d;
        }
        if let Some(hint) = hint {
            centred(canvas, y.max(h - 40.0 * d), &small, &hint);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vellmark_content::StringTableBuilder;

    fn table(wishlist: &str, feedback: &str) -> StringTable {
        let mut b = StringTableBuilder::new();
        b.add("end.link.wishlist", wishlist).unwrap();
        b.add("end.link.feedback", feedback).unwrap();
        StringTable::from_bytes(&b.finish()).unwrap()
    }

    const OK: MenuInput = MenuInput { step: 0, change: 0, confirm: true, back: false, pick: None };
    const DOWN: MenuInput = MenuInput { step: 1, change: 0, confirm: false, back: false, pick: None };

    #[test]
    fn only_the_links_given_are_offered_and_opened_when_chosen() {
        assert_eq!(choices(&table("", "")), vec![Choice::KeepPlaying, Choice::StartAgain]);
        assert_eq!(choices(&table("not a link", "")), vec![Choice::KeepPlaying, Choice::StartAgain], "only https addresses");
        let strings = table("https://example.com/wishlist", "");
        assert_eq!(choices(&strings), vec![Choice::KeepPlaying, Choice::Wishlist, Choice::StartAgain]);
        let mut card = EndCard::default();
        assert_eq!(card.update(OK, &strings, 0.5), EndOutcome::None, "a press as it appears does not dismiss it");
        card.update(DOWN, &strings, SETTLE);
        assert_eq!(card.update(OK, &strings, 0.1), EndOutcome::Open("https://example.com/wishlist".into()));
        card.update(DOWN, &strings, 0.1);
        assert_eq!(card.update(OK, &strings, 0.1), EndOutcome::StartAgain);
        let mut card = EndCard { since: SETTLE, ..Default::default() };
        assert_eq!(card.update(OK, &strings, 0.1), EndOutcome::Close, "the first choice plays on");
    }
}
