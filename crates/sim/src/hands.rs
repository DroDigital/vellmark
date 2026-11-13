//! The two hands. Each has its own button: a tap acts at once, a hold charges until it is
//! let go, and both pressed together act as one. A hand holds a weapon, a shield, or a
//! word, and what a tap or hold does is decided by what it holds; this only says what
//! the hands did, step by step, so that every rule reads the same thing.

use crate::clock::STEP_SECONDS;

/// A press held longer than this is a hold, not a tap.
pub const HOLD_SECONDS: f32 = 0.22;
/// Two presses this close together are both hands at once.
pub const TOGETHER_SECONDS: f32 = 0.12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hand {
    Main,
    Off,
}

/// What the hands did this step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HandEvent {
    /// Pressed and let go quickly.
    Tap(Hand),
    /// Held long enough to be a hold: the charge begins.
    HoldBegin(Hand),
    /// A hold let go, after charging this long in seconds.
    HoldEnd(Hand, f32),
    /// Both hands pressed together: one act with both.
    BothBegin,
    /// Both hands let go: charged this long.
    BothEnd(f32),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct One {
    down: bool,
    /// Seconds since pressed, while down.
    held: f32,
    holding: bool,
    /// Interrupted: letting go does nothing.
    spent: bool,
}

/// The hands' state between steps.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Hands {
    main: One,
    off: One,
    /// Both hands are acting as one, since this long.
    both: Option<f32>,
}

impl Hands {
    /// How long a hand has been charging, if it is.
    pub fn charging(&self, hand: Hand) -> Option<f32> {
        if self.both.is_some() {
            return None;
        }
        let h = match hand {
            Hand::Main => self.main,
            Hand::Off => self.off,
        };
        h.holding.then_some(h.held - HOLD_SECONDS)
    }

    /// A blow lands: whatever the hands were charging is lost, and letting go of the
    /// buttons does nothing until they are pressed again.
    pub fn interrupt(&mut self) {
        for h in [&mut self.main, &mut self.off] {
            if h.down {
                *h = One {
                    down: true,
                    held: 0.0,
                    holding: false,
                    spent: true,
                };
            }
        }
        self.both = None;
    }

    /// How long both hands have been acting together, if they are.
    pub fn both(&self) -> Option<f32> {
        self.both
    }

    /// One step, from whether each hand's button is down. Events go to `out`, which is
    /// cleared first; at most four happen in a step.
    pub fn step(&mut self, main_down: bool, off_down: bool, out: &mut Vec<HandEvent>) {
        out.clear();
        let dt = STEP_SECONDS;
        // Both hands at once: the second pressed soon after the first, before either is
        // a hold.
        if self.both.is_none() {
            let pressing = |h: &One, down: bool| down && !h.down;
            let early =
                |h: &One, down: bool| down && h.down && !h.holding && h.held < TOGETHER_SECONDS;
            let together = (pressing(&self.main, main_down)
                && (early(&self.off, off_down) || pressing(&self.off, off_down)))
                || (pressing(&self.off, off_down) && early(&self.main, main_down));
            if together {
                self.both = Some(0.0);
                self.main = One {
                    down: true,
                    held: 0.0,
                    holding: true,
                    spent: false,
                };
                self.off = One {
                    down: true,
                    held: 0.0,
                    holding: true,
                    spent: false,
                };
                out.push(HandEvent::BothBegin);
                return;
            }
        }
        if let Some(t) = self.both {
            if main_down && off_down {
                self.both = Some(t + dt);
                return;
            }
            // Letting go of either ends it; the other must be pressed again to act.
            out.push(HandEvent::BothEnd(t));
            self.both = None;
            self.main = One {
                down: main_down,
                held: 0.0,
                holding: false,
                spent: main_down,
            };
            self.off = One {
                down: off_down,
                held: 0.0,
                holding: false,
                spent: off_down,
            };
            return;
        }
        for (hand, h, down) in [
            (Hand::Main, &mut self.main, main_down),
            (Hand::Off, &mut self.off, off_down),
        ] {
            match (h.down, down) {
                (false, true) => {
                    *h = One {
                        down: true,
                        held: 0.0,
                        holding: false,
                        spent: false,
                    }
                }
                (true, true) => {
                    h.held += dt;
                    if !h.holding && !h.spent && h.held >= HOLD_SECONDS {
                        h.holding = true;
                        out.push(HandEvent::HoldBegin(hand));
                    }
                }
                (true, false) => {
                    if h.spent {
                        // Interrupted, or the end of both hands' act: nothing.
                    } else if h.holding {
                        out.push(HandEvent::HoldEnd(hand, (h.held - HOLD_SECONDS).max(0.0)));
                    } else {
                        out.push(HandEvent::Tap(hand));
                    }
                    *h = One::default();
                }
                (false, false) => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(script: &[(bool, bool, u32)]) -> Vec<HandEvent> {
        let mut hands = Hands::default();
        let mut all = Vec::new();
        let mut out = Vec::new();
        for &(m, o, steps) in script {
            for _ in 0..steps {
                hands.step(m, o, &mut out);
                all.extend_from_slice(&out);
            }
        }
        all
    }

    const STEPS_A_SECOND: u32 = (1.0 / STEP_SECONDS) as u32;

    #[test]
    fn a_quick_press_is_a_tap() {
        assert_eq!(
            run(&[(true, false, 3), (false, false, 2)]),
            vec![HandEvent::Tap(Hand::Main)]
        );
        assert_eq!(
            run(&[(false, true, 3), (false, false, 2)]),
            vec![HandEvent::Tap(Hand::Off)]
        );
    }

    #[test]
    fn a_long_press_is_a_hold_that_charges() {
        let events = run(&[(false, true, STEPS_A_SECOND), (false, false, 1)]);
        assert_eq!(events[0], HandEvent::HoldBegin(Hand::Off));
        let HandEvent::HoldEnd(Hand::Off, charged) = events[1] else {
            panic!("{events:?}")
        };
        assert!((charged - (1.0 - HOLD_SECONDS)).abs() < 0.05, "{charged}");
    }

    #[test]
    fn each_hand_is_its_own() {
        // The main hand taps while the off hand holds.
        let events = run(&[
            (false, true, STEPS_A_SECOND / 2),
            (true, true, 3),
            (false, true, 3),
            (false, false, 1),
        ]);
        assert!(events.contains(&HandEvent::HoldBegin(Hand::Off)));
        assert!(events.contains(&HandEvent::Tap(Hand::Main)));
        assert!(matches!(
            events.last(),
            Some(HandEvent::HoldEnd(Hand::Off, _))
        ));
    }

    #[test]
    fn both_together_act_as_one() {
        let events = run(&[
            (true, false, 2),
            (true, true, STEPS_A_SECOND),
            (false, false, 1),
        ]);
        assert_eq!(events[0], HandEvent::BothBegin);
        assert!(
            matches!(events[1], HandEvent::BothEnd(t) if t > 0.9),
            "{events:?}"
        );
        assert_eq!(events.len(), 2, "no tap or hold of either hand: {events:?}");
    }

    #[test]
    fn a_second_hand_pressed_late_is_not_together() {
        let events = run(&[
            (true, false, STEPS_A_SECOND / 2),
            (true, true, 3),
            (false, false, 1),
        ]);
        assert!(!events.contains(&HandEvent::BothBegin), "{events:?}");
    }
}
