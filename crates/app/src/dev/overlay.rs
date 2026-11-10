//! The profiler view and the console, drawn in the interface's own letters.

use std::time::Duration;
use vellmark_content::{Face, GlyphAtlas};
use vellmark_gpu::{PassTiming, PipelineRegistry};
use vellmark_platform::RawInput;
use vellmark_ui::{dev_text, Canvas, TextStyle};

/// What the console can ask of the game.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    /// Fix the hour of the day, or let the clock run again.
    Hour(Option<f32>),
    GlyphLayer(bool),
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
                    "quit           leave",
                    "F1 profiler   F2 free camera   F3 glyph layer",
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
            (Some("quit"), _) => self.commands.push(Command::Quit),
            (Some(other), _) => self.say(format!("unknown command `{other}`")),
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
