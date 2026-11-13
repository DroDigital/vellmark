use crate::actions::RawInput;

/// Anything that produces raw input: SDL, a simulated device, or a replay.
pub trait InputSource {
    /// Appends the input that arrived since the last call.
    fn poll(&mut self, out: &mut Vec<RawInput>);
}

/// A device that plays a script. Tests and bots use it in place of real hardware, to
/// replay hot-plugs, disconnects, and whole controller-only runs.
#[derive(Default)]
pub struct SimulatedDevice {
    script: Vec<(u64, RawInput)>,
    frame: u64,
    next: usize,
}

impl SimulatedDevice {
    pub fn new() -> Self {
        Self::default()
    }

    /// Schedules an input for a frame. Frames count calls to `poll`, from 0.
    pub fn at(mut self, frame: u64, input: RawInput) -> Self {
        self.script.push((frame, input));
        self.script.sort_by_key(|(f, _)| *f);
        self
    }

    pub fn finished(&self) -> bool {
        self.next >= self.script.len()
    }
}

impl InputSource for SimulatedDevice {
    fn poll(&mut self, out: &mut Vec<RawInput>) {
        while self.next < self.script.len() && self.script[self.next].0 <= self.frame {
            out.push(self.script[self.next].1);
            self.next += 1;
        }
        self.frame += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::{Action, ActionState, Bindings, PadAxis, PadButton};

    #[test]
    fn a_scripted_controller_drives_actions() {
        let mut device = SimulatedDevice::new()
            .at(0, RawInput::PadAdded { pad: 7 })
            .at(
                1,
                RawInput::PadAxis {
                    pad: 7,
                    axis: PadAxis::LeftY,
                    value: -1.0,
                },
            )
            .at(
                2,
                RawInput::PadButton {
                    pad: 7,
                    button: PadButton::South,
                    down: true,
                },
            )
            .at(3, RawInput::PadRemoved { pad: 7 })
            .at(5, RawInput::PadAdded { pad: 8 });
        let bindings = Bindings::default();
        let mut state = ActionState::default();
        let mut seen = Vec::new();
        let mut buffer = Vec::new();
        for _ in 0..6 {
            state.begin_frame();
            buffer.clear();
            device.poll(&mut buffer);
            for input in &buffer {
                state.feed(&bindings, *input);
            }
            seen.push((state.movement().1, state.held(Action::Jump), state.pad_lost));
        }
        assert_eq!(
            seen,
            vec![
                (0.0, false, false),
                (1.0, false, false),
                (1.0, true, false),
                (0.0, false, true),
                (0.0, false, true),
                (0.0, false, false),
            ]
        );
        assert!(device.finished());
    }
}
