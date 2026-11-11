use crate::actions::{Key, MouseButton, PadAxis, PadButton, RawInput};
use crate::device::InputSource;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use sdl3::event::{Event, WindowEvent};
use sdl3::gamepad::{Axis, Button, Gamepad};
use sdl3::keyboard::Scancode;

/// What the OS told us that is not input.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlatformEvent {
    Quit,
    /// The drawable size changed, in pixels.
    Resized(u32, u32),
    FocusLost,
    FocusGained,
}

pub struct WindowOptions<'a> {
    pub title: &'a str,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub hidden: bool,
}

pub struct Window {
    inner: sdl3::video::Window,
}

impl Window {
    /// Size of the drawable area in pixels. On a high-DPI display this is larger than
    /// the size in points.
    pub fn size_in_pixels(&self) -> (u32, u32) {
        self.inner.size_in_pixels()
    }

    /// Pixels to a point. 2 on a Retina display.
    pub fn pixel_density(&self) -> f32 {
        self.inner.pixel_density()
    }

    /// Resizes the window so that its drawable area is this many pixels, whatever the
    /// display's density. The performance run uses it to measure at exactly 1080p.
    pub fn set_size_in_pixels(&mut self, width: u32, height: u32) {
        let density = self.pixel_density().max(0.25);
        let _ = self.inner.set_size((width as f32 / density).round() as u32, (height as f32 / density).round() as u32);
    }

    pub fn set_fullscreen(&mut self, on: bool) {
        let _ = self.inner.set_fullscreen(on);
    }

    pub fn handles(&self) -> Result<(RawWindowHandle, RawDisplayHandle), String> {
        let window = self.inner.window_handle().map_err(|e| e.to_string())?.as_raw();
        let display = self.inner.display_handle().map_err(|e| e.to_string())?.as_raw();
        Ok((window, display))
    }
}

/// The SDL context: video, events, and controllers.
pub struct Platform {
    _sdl: sdl3::Sdl,
    video: sdl3::VideoSubsystem,
    gamepads: sdl3::GamepadSubsystem,
    pump: sdl3::EventPump,
    mouse: sdl3::mouse::MouseUtil,
    open_pads: Vec<Gamepad>,
    events: Vec<PlatformEvent>,
}

impl Platform {
    pub fn new() -> Result<Self, String> {
        let sdl = sdl3::init().map_err(|e| e.to_string())?;
        let video = sdl.video().map_err(|e| e.to_string())?;
        let gamepads = sdl.gamepad().map_err(|e| e.to_string())?;
        let pump = sdl.event_pump().map_err(|e| e.to_string())?;
        let mouse = sdl.mouse();
        Ok(Self { _sdl: sdl, video, gamepads, pump, mouse, open_pads: Vec::new(), events: Vec::new() })
    }

    pub fn create_window(&self, options: &WindowOptions<'_>) -> Result<Window, String> {
        let mut builder = self.video.window(options.title, options.width, options.height);
        builder.resizable().high_pixel_density();
        if options.fullscreen {
            builder.fullscreen();
        }
        if options.hidden {
            builder.hidden();
        }
        #[cfg(target_os = "macos")]
        builder.metal_view();
        #[cfg(all(unix, not(target_os = "macos")))]
        builder.vulkan();
        let inner = builder.build().map_err(|e| e.to_string())?;
        Ok(Window { inner })
    }

    /// Hides the cursor and reports mouse movement without limit, for looking around.
    pub fn capture_mouse(&self, window: &Window, on: bool) {
        self.mouse.set_relative_mouse_mode(&window.inner, on);
    }

    /// Turns typed-character events on or off. On for the developer console and, later,
    /// for naming things; off the rest of the time so that keys are only keys.
    pub fn text_entry(&self, window: &Window, on: bool) {
        let input = self.video.text_input();
        if on { input.start(&window.inner) } else { input.stop(&window.inner) }
    }

    /// Window and quit events gathered by the last `poll`.
    pub fn drain_events(&mut self) -> Vec<PlatformEvent> {
        std::mem::take(&mut self.events)
    }
}

impl InputSource for Platform {
    fn poll(&mut self, out: &mut Vec<RawInput>) {
        while let Some(event) = self.pump.poll_event() {
            match event {
                Event::Quit { .. } => self.events.push(PlatformEvent::Quit),
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::CloseRequested => self.events.push(PlatformEvent::Quit),
                    WindowEvent::PixelSizeChanged(w, h) => {
                        self.events.push(PlatformEvent::Resized(w.max(1) as u32, h.max(1) as u32))
                    }
                    WindowEvent::FocusLost => self.events.push(PlatformEvent::FocusLost),
                    WindowEvent::FocusGained => self.events.push(PlatformEvent::FocusGained),
                    _ => {}
                },
                Event::KeyDown { scancode: Some(code), repeat: false, .. } => {
                    if let Some(key) = key_of(code) {
                        out.push(RawInput::Key { key, down: true });
                    }
                }
                Event::KeyUp { scancode: Some(code), .. } => {
                    if let Some(key) = key_of(code) {
                        out.push(RawInput::Key { key, down: false });
                    }
                }
                Event::TextInput { text, .. } => {
                    out.extend(text.chars().filter(|c| (' '..='~').contains(c)).map(RawInput::Text));
                }
                Event::MouseMotion { xrel, yrel, .. } => out.push(RawInput::MouseMove { dx: xrel, dy: yrel }),
                Event::MouseButtonDown { mouse_btn, .. } => {
                    if let Some(button) = mouse_button_of(mouse_btn) {
                        out.push(RawInput::MouseButton { button, down: true });
                    }
                }
                Event::MouseButtonUp { mouse_btn, .. } => {
                    if let Some(button) = mouse_button_of(mouse_btn) {
                        out.push(RawInput::MouseButton { button, down: false });
                    }
                }
                Event::MouseWheel { y, .. } => out.push(RawInput::MouseWheel { dy: y }),
                Event::GamepadAdded { which, .. } => {
                    if let Ok(pad) = self.gamepads.open(which) {
                        self.open_pads.push(pad);
                        out.push(RawInput::PadAdded { pad: which.raw() });
                    }
                }
                Event::GamepadRemoved { which, .. } => {
                    self.open_pads.retain(|p| p.id().map(|id| id != which).unwrap_or(false));
                    out.push(RawInput::PadRemoved { pad: which.raw() });
                }
                Event::GamepadButtonDown { which, button, .. } => {
                    if let Some(button) = pad_button_of(button) {
                        out.push(RawInput::PadButton { pad: which.raw(), button, down: true });
                    }
                }
                Event::GamepadButtonUp { which, button, .. } => {
                    if let Some(button) = pad_button_of(button) {
                        out.push(RawInput::PadButton { pad: which.raw(), button, down: false });
                    }
                }
                Event::GamepadAxisMotion { which, axis, value, .. } => {
                    let axis = match axis {
                        Axis::LeftX => PadAxis::LeftX,
                        Axis::LeftY => PadAxis::LeftY,
                        Axis::RightX => PadAxis::RightX,
                        Axis::RightY => PadAxis::RightY,
                        Axis::TriggerLeft => PadAxis::LeftTrigger,
                        Axis::TriggerRight => PadAxis::RightTrigger,
                    };
                    let value = (value as f32 / 32767.0).clamp(-1.0, 1.0);
                    out.push(RawInput::PadAxis { pad: which.raw(), axis, value });
                }
                _ => {}
            }
        }
    }
}

fn mouse_button_of(button: sdl3::mouse::MouseButton) -> Option<MouseButton> {
    Some(match button {
        sdl3::mouse::MouseButton::Left => MouseButton::Left,
        sdl3::mouse::MouseButton::Right => MouseButton::Right,
        sdl3::mouse::MouseButton::Middle => MouseButton::Middle,
        _ => return None,
    })
}

fn pad_button_of(button: Button) -> Option<PadButton> {
    Some(match button {
        Button::South => PadButton::South,
        Button::East => PadButton::East,
        Button::West => PadButton::West,
        Button::North => PadButton::North,
        Button::LeftShoulder => PadButton::LeftShoulder,
        Button::RightShoulder => PadButton::RightShoulder,
        Button::LeftStick => PadButton::LeftStick,
        Button::RightStick => PadButton::RightStick,
        Button::Start => PadButton::Start,
        Button::Back => PadButton::Back,
        Button::DPadUp => PadButton::DPadUp,
        Button::DPadDown => PadButton::DPadDown,
        Button::DPadLeft => PadButton::DPadLeft,
        Button::DPadRight => PadButton::DPadRight,
        _ => return None,
    })
}

fn key_of(code: Scancode) -> Option<Key> {
    use Scancode as S;
    Some(match code {
        S::W => Key::W, S::A => Key::A, S::S => Key::S, S::D => Key::D,
        S::Q => Key::Q, S::E => Key::E, S::R => Key::R, S::F => Key::F,
        S::C => Key::C, S::X => Key::X, S::Z => Key::Z, S::V => Key::V,
        S::T => Key::T, S::G => Key::G, S::B => Key::B, S::I => Key::I,
        S::M => Key::M, S::P => Key::P,
        S::_1 => Key::Num1, S::_2 => Key::Num2, S::_3 => Key::Num3, S::_4 => Key::Num4,
        S::_5 => Key::Num5, S::_6 => Key::Num6, S::_7 => Key::Num7,
        S::Space => Key::Space, S::LShift => Key::LeftShift, S::LCtrl => Key::LeftCtrl,
        S::LAlt => Key::LeftAlt, S::Tab => Key::Tab, S::Escape => Key::Escape,
        S::Return => Key::Enter, S::Backspace => Key::Backspace, S::Grave => Key::Grave,
        S::Up => Key::Up, S::Down => Key::Down, S::Left => Key::Left, S::Right => Key::Right,
        S::F1 => Key::F1, S::F2 => Key::F2, S::F3 => Key::F3, S::F4 => Key::F4,
        S::F5 => Key::F5, S::F6 => Key::F6, S::F7 => Key::F7, S::F8 => Key::F8,
        S::F9 => Key::F9, S::F10 => Key::F10, S::F11 => Key::F11, S::F12 => Key::F12,
        _ => return None,
    })
}

/// Shows a message in a system dialog. Used when the game cannot start, before or
/// instead of a window. The text comes from the string table, never from code.
pub fn message_box(title: &str, message: &str) {
    use sdl3::messagebox::{show_simple_message_box, MessageBoxFlag};
    if show_simple_message_box(MessageBoxFlag::ERROR, title, message, None).is_err() {
        eprintln!("{title}: {message}");
    }
}
