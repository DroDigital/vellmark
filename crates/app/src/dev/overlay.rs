//! The profiler view and the console, drawn in the interface's own letters.

use std::time::Duration;
use vellmark_content::{Face, GlyphAtlas};
use vellmark_gpu::{PassTiming, PipelineRegistry};
use vellmark_platform::RawInput;
use vellmark_ui::{dev_text, Canvas, TextStyle};

/// What the console can ask of the game.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    /// Fix the hour of the day, or let the clock run again.
    Hour(Option<f32>),
    GlyphLayer(bool),
    /// Light the lamps or put them out.
    Lamps(bool),
    /// Scale the reveal distance.
    Reveal(f32),
    /// Scale the letter contrast.
    Contrast(f32),
    /// Choose the depth look, 0 to 3.
    Look(u32),
    /// Break a thing in the scene into its letters, by its name.
    Break(String, f32),
    /// Heat or cool a thing, by its name: -1 frozen to 1 as hot as it gets.
    Heat(String, f32),
    /// Ask for rain, 0 to 1.
    Rain(f32),
    /// Ask for fog, 0 to 1.
    Fog(f32),
    /// A gust that writes one of the wind's sentences, by number.
    Wind(u32),
    /// Choose a preset: low, medium, or high.
    Preset(vellmark_render::Preset),
    /// Mouse sensitivity, as a multiple of the default.
    Sensitivity(f32),
    InvertY(bool),
    /// Move an action to a button: a key, `mouse_left`, or a pad button such as `south`.
    Bind(String, String),
    Quit,
}

const WHITE: [f32; 4] = [0.92, 0.90, 0.84, 1.0];
const DIM: [f32; 4] = [0.60, 0.58, 0.52, 1.0];
const GOLD: [f32; 4] = [0.95, 0.72, 0.25, 1.0];
const RED: [f32; 4] = [0.95, 0.30, 0.22, 1.0];
const PANEL: [f32; 4] = [0.02, 0.02, 0.025, 0.78];

/// Each pass's share of the frame in milliseconds, from the budget in docs/TECH.md.
/// Passes not built yet have no share and are drawn without a mark.
fn budget_ms(pass: &str) -> Option<f32> {
    Some(match pass {
        "depth" => 1.0,
        "culling" => 0.4,
        "shadows" => 2.0,
        "opaque" => 5.0,
        "foliage" => 2.0,
        "transparent" => 0.9,
        "fog" => 1.0,
        "particles" => 0.7,
        "occlusion" => 0.6,
        "post" => 0.8,
        "interface" => 0.4,
        _ => return None,
    })
}

pub struct Overlay {
    pub profiler: bool,
    pub console: bool,
    /// The tuning panel: the reveal distance and the letter contrast.
    pub tuning: bool,
    tuning_row: usize,
    line: String,
    history: Vec<String>,
    commands: Vec<Command>,
    smoothed_ms: f32,
    worst_ms: f32,
    worst_age: u32,
}

impl Overlay {
    pub fn new() -> Self {
        Self {
            profiler: false,
            console: false,
            tuning: false,
            tuning_row: 0,
            line: String::new(),
            history: vec!["type `help` and press Enter".into()],
            commands: Vec::new(),
            smoothed_ms: 16.6,
            worst_ms: 0.0,
            worst_age: 0,
        }
    }

    /// While the console is open it takes the keyboard: typed characters, Enter, and
    /// Backspace are consumed here and never reach the game.
    pub fn console_input(&mut self, raw: &mut Vec<RawInput>) {
        use vellmark_platform::Key;
        if !self.console {
            return;
        }
        raw.retain(|input| match *input {
            RawInput::Text(c) => {
                if c != '`' {
                    self.line.push(c);
                }
                false
            }
            RawInput::Key { key: Key::Enter, down: true } => {
                let line = std::mem::take(&mut self.line);
                self.run(&line);
                false
            }
            RawInput::Key { key: Key::Backspace, down: true } => {
                self.line.pop();
                false
            }
            // The key that closes the console, and Escape, still get through.
            RawInput::Key { key: Key::Grave | Key::Escape, .. } => true,
            RawInput::Key { .. } => false,
            _ => true,
        });
    }

    fn say(&mut self, text: impl Into<String>) {
        self.history.push(text.into());
        if self.history.len() > 12 {
            self.history.remove(0);
        }
    }

    fn run(&mut self, line: &str) {
        self.say(format!("> {line}"));
        let mut words = line.split_whitespace();
        match (words.next(), words.next()) {
            (None, _) => {}
            (Some("help"), _) => {
                for text in [
                    "hour <0-24>    fix the time of day",
                    "hour run       let the clock run",
                    "glyph on|off   switch the glyph layer",
                    "lamps on|off   light the lamps or put them out",
                    "reveal <n>     scale the reveal distance (1 is the brief's)",
                    "contrast <n>   scale the letter contrast",
                    "look <0-4>     flat, atoms, lattice, source, living (F6)",
                    "break <name> [strength]  break a thing into its letters; 0.1 crumbles it (F7)",
                    "heat <name> <-1 to 1>  heat or cool a thing; up close its letters say so",
                    "quit           leave",
                    "F1 profiler  F2 free camera  F3 glyphs  F4 tuning  F5 lamps",
                ] {
                    self.say(text);
                }
            }
            (Some("hour"), Some("run")) => self.commands.push(Command::Hour(None)),
            (Some("hour"), Some(v)) => match v.parse::<f32>() {
                Ok(h) if (0.0..=24.0).contains(&h) => self.commands.push(Command::Hour(Some(h))),
                _ => self.say("hour takes a number from 0 to 24, or `run`"),
            },
            (Some("glyph"), Some("on")) => self.commands.push(Command::GlyphLayer(true)),
            (Some("glyph"), Some("off")) => self.commands.push(Command::GlyphLayer(false)),
            (Some("lamps"), Some("on")) => self.commands.push(Command::Lamps(true)),
            (Some("lamps"), Some("off")) => self.commands.push(Command::Lamps(false)),
            (Some("reveal"), Some(v)) => match v.parse::<f32>() {
                Ok(n) if (0.1..=8.0).contains(&n) => self.commands.push(Command::Reveal(n)),
                _ => self.say("reveal takes a number from 0.1 to 8"),
            },
            (Some("contrast"), Some(v)) => match v.parse::<f32>() {
                Ok(n) if (0.0..=4.0).contains(&n) => self.commands.push(Command::Contrast(n)),
                _ => self.say("contrast takes a number from 0 to 4"),
            },
            (Some("look"), Some(v)) => match v.parse::<u32>() {
                Ok(n) if n <= 4 => self.commands.push(Command::Look(n)),
                _ => self.say("look takes 0 (flat), 1 (atoms), 2 (lattice), 3 (source), or 4 (living)"),
            },
            (Some("break"), Some(name)) => match words.next().map(str::parse::<f32>) {
                None => self.commands.push(Command::Break(name.to_string(), 1.0)),
                Some(Ok(s)) if (0.0..=4.0).contains(&s) => self.commands.push(Command::Break(name.to_string(), s)),
                _ => self.say("break takes a name, then a strength from 0 to 4 if you like"),
            },
            (Some("heat"), Some(name)) => match words.next().map(str::parse::<f32>) {
                Some(Ok(h)) if (-1.0..=1.0).contains(&h) => self.commands.push(Command::Heat(name.to_string(), h)),
                _ => self.say("heat takes a name, then a number from -1 to 1"),
            },
            (Some("rain"), Some(v)) => match v.parse::<f32>() {
                Ok(r) if (0.0..=1.0).contains(&r) => self.commands.push(Command::Rain(r)),
                _ => self.say("rain takes a number from 0 (dry) to 1 (a downpour)"),
            },
            (Some("fog"), Some(v)) => match v.parse::<f32>() {
                Ok(f) if (0.0..=1.0).contains(&f) => self.commands.push(Command::Fog(f)),
                _ => self.say("fog takes a number from 0 (clear) to 1 (thick)"),
            },
            (Some("wind"), Some(v)) => match v.parse::<u32>() {
                Ok(n) => self.commands.push(Command::Wind(n)),
                _ => self.say("wind takes the number of a sentence, from 0"),
            },
            (Some("preset"), Some(v)) => match vellmark_render::Preset::parse(v) {
                Some(p) => self.commands.push(Command::Preset(p)),
                None => self.say("preset takes low, medium, or high"),
            },
            (Some("sensitivity"), Some(v)) => match v.parse::<f32>() {
                Ok(n) if (0.2..=4.0).contains(&n) => self.commands.push(Command::Sensitivity(n)),
                _ => self.say("sensitivity takes a multiple of the default, from 0.2 to 4"),
            },
            (Some("invert"), Some("on")) => self.commands.push(Command::InvertY(true)),
            (Some("invert"), Some("off")) => self.commands.push(Command::InvertY(false)),
            (Some("bind"), Some(action)) => match words.next() {
                Some(button) => self.commands.push(Command::Bind(action.to_string(), button.to_string())),
                None => self.say("bind takes an action, then a key, `mouse_left`, or a pad button such as `south`"),
            },
            (Some("quit"), _) => self.commands.push(Command::Quit),
            (Some(other), _) => self.say(format!("unknown command `{other}`")),
        }
    }

    /// The tuning panel's keys: down picks a row, left and right change its value.
    pub fn tune(&mut self, tuning: &mut vellmark_render::GlyphTuning, next: bool, less: bool, more: bool) {
        if !self.tuning {
            return;
        }
        if next {
            self.tuning_row = (self.tuning_row + 1) % 2;
        }
        let step = if more { 1.0 } else if less { -1.0 } else { 0.0 };
        if step != 0.0 {
            // Steps of a twentieth, so that a value can be read back and written down.
            let value = if self.tuning_row == 0 { &mut tuning.reveal } else { &mut tuning.contrast };
            *value = ((*value + step * 0.05) * 20.0).round() / 20.0;
            *value = value.clamp(0.1, 8.0);
        }
    }

    pub fn take_commands(&mut self) -> Vec<Command> {
        std::mem::take(&mut self.commands)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        canvas: &mut Canvas,
        atlas: &GlyphAtlas,
        target: (u32, u32),
        density: f32,
        frame_time: Duration,
        passes: &[PassTiming],
        registry: &PipelineRegistry,
        last_reload_ms: Option<f32>,
        tuning: &vellmark_render::GlyphTuning,
    ) {
        let ms = frame_time.as_secs_f32() * 1000.0;
        self.smoothed_ms += (ms - self.smoothed_ms) * 0.05;
        self.worst_age += 1;
        if ms > self.worst_ms || self.worst_age > 120 {
            self.worst_ms = ms;
            self.worst_age = 0;
        }
        let style = TextStyle { face: Face::Small, size: (14.0 * density.max(1.0)).round(), colour: WHITE };
        let line = Canvas::line_height(atlas, &style).round();
        let pad = (8.0 * density.max(1.0)).round();

        if self.profiler {
            let rows = passes.len() as f32 + 4.0;
            let width = style.size * 30.0;
            canvas.rect([pad, pad, width, rows * line + pad * 2.0], PANEL);
            let mut y = pad * 2.0 + line * 0.75;
            let mut put = |canvas: &mut Canvas, text: &str, colour: [f32; 4]| {
                canvas.text(atlas, pad * 2.0, y, &TextStyle { colour, ..style }, &dev_text(text));
                y += line;
            };
            let over = self.smoothed_ms > 16.7;
            put(
                canvas,
                &format!("FRAME {:5.2} ms  {:5.1} fps  worst {:5.2}", self.smoothed_ms, 1000.0 / self.smoothed_ms.max(0.01), self.worst_ms),
                if over { RED } else { GOLD },
            );
            for pass in passes {
                // One character of bar for each quarter of a millisecond, to twenty.
                let cells = ((pass.milliseconds * 4.0).round() as usize).min(20);
                let mark = budget_ms(pass.name).map(|b| ((b * 4.0).round() as usize).min(20));
                let bar: String = (0..20)
                    .map(|i| if Some(i + 1) == mark { '|' } else if i < cells { '#' } else { '.' })
                    .collect();
                let over = budget_ms(pass.name).is_some_and(|b| pass.milliseconds > b);
                put(canvas, &format!("{:<10} {bar} {:5.2}", pass.name, pass.milliseconds), if over { RED } else { WHITE });
            }
            if passes.is_empty() {
                put(canvas, "this graphics card cannot time passes", DIM);
            }
            put(canvas, &format!("pipelines {}  made in play {}", registry.created(), registry.created_during_play()), DIM);
            if let Some(ms) = last_reload_ms {
                put(canvas, &format!("last reload {ms:.1} ms"), DIM);
            }
        }

        if self.tuning {
            let width = style.size * 26.0;
            let x = target.0 as f32 - width - pad;
            canvas.rect([x, pad, width, line * 4.0 + pad * 2.0], PANEL);
            let rows = [
                ("TUNING   down: next   left, right: change".to_owned(), GOLD),
                // 6 m is where the first trace of letters shows at the brief's setting.
                (format!("{} reveal    {:4.2}  first trace {:4.1} m", if self.tuning_row == 0 { '>' } else { ' ' }, tuning.reveal, 6.0 * tuning.reveal), WHITE),
                (format!("{} contrast  {:4.2}", if self.tuning_row == 1 { '>' } else { ' ' }, tuning.contrast), WHITE),
                ("write the values down for the Look Gate".to_owned(), DIM),
            ];
            let mut y = pad * 2.0 + line * 0.75;
            for (text, colour) in rows {
                canvas.text(atlas, x + pad, y, &TextStyle { colour, ..style }, &dev_text(&text));
                y += line;
            }
        }

        if self.console {
            let rows = self.history.len() as f32 + 1.0;
            let height = rows * line + pad * 2.0;
            let top = target.1 as f32 - height - pad;
            canvas.rect([pad, top, target.0 as f32 - pad * 2.0, height], PANEL);
            let mut y = top + pad + line * 0.75;
            for text in &self.history {
                canvas.text(atlas, pad * 2.0, y, &TextStyle { colour: DIM, ..style }, &dev_text(text));
                y += line;
            }
            canvas.text(atlas, pad * 2.0, y, &TextStyle { colour: GOLD, ..style }, &dev_text(&format!("> {}_", self.line)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vellmark_platform::Key;

    fn typed(overlay: &mut Overlay, text: &str) {
        let mut raw: Vec<RawInput> = text.chars().map(RawInput::Text).collect();
        raw.push(RawInput::Key { key: Key::Enter, down: true });
        overlay.console_input(&mut raw);
        assert!(raw.is_empty(), "the console consumes what is typed into it");
    }

    #[test]
    fn commands() {
        let mut overlay = Overlay::new();
        overlay.console = true;
        typed(&mut overlay, "hour 18.5");
        typed(&mut overlay, "glyph off");
        typed(&mut overlay, "hour run");
        typed(&mut overlay, "hour 99");
        typed(&mut overlay, "nonsense");
        assert_eq!(
            overlay.take_commands(),
            vec![Command::Hour(Some(18.5)), Command::GlyphLayer(false), Command::Hour(None)]
        );
        typed(&mut overlay, "reveal 1.5");
        typed(&mut overlay, "contrast 9");
        assert_eq!(overlay.take_commands(), vec![Command::Reveal(1.5)]);
        let mut tuning = vellmark_render::GlyphTuning::default();
        overlay.tune(&mut tuning, false, false, true);
        assert_eq!(tuning.reveal, 1.0, "the panel does nothing while it is closed");
        overlay.tuning = true;
        overlay.tune(&mut tuning, false, false, true);
        overlay.tune(&mut tuning, true, false, false);
        overlay.tune(&mut tuning, false, true, false);
        assert_eq!((tuning.reveal, tuning.contrast), (1.05, 0.95)
        );
        assert!(overlay.history.iter().any(|l| l.contains("unknown command")));
    }

    #[test]
    fn a_closed_console_leaves_input_alone() {
        let mut overlay = Overlay::new();
        let mut raw = vec![RawInput::Text('x'), RawInput::Key { key: Key::W, down: true }];
        overlay.console_input(&mut raw);
        assert_eq!(raw.len(), 2);
        overlay.console = true;
        overlay.console_input(&mut raw);
        assert!(raw.is_empty(), "movement keys do not reach the game while typing");
    }
}
