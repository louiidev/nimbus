use sdl2::keyboard::Keycode;

use super::{Axis, Input};

impl TryFrom<sdl2::keyboard::Keycode> for Input {
    type Error = &'static str;
    fn try_from(value: sdl2::keyboard::Keycode) -> Result<Self, Self::Error> {
        if let Some(value) = convert_key_code(value) {
            Ok(value)
        } else {
            Err("Couldnt map keycode")
        }
    }
}

impl TryFrom<sdl2::controller::Axis> for Axis {
    type Error = &'static str;
    fn try_from(value: sdl2::controller::Axis) -> Result<Self, Self::Error> {
        if let Some(axis) = convert_axis(value) {
            Ok(axis)
        } else {
            Err("Couldnt map")
        }
    }
}

impl TryFrom<sdl2::mouse::MouseButton> for Input {
    type Error = &'static str;
    fn try_from(value: sdl2::mouse::MouseButton) -> Result<Self, Self::Error> {
        if let Some(btn) = convert_mouse_button(value) {
            Ok(btn)
        } else {
            Err("Couldnt map")
        }
    }
}

impl TryFrom<sdl2::controller::Button> for Input {
    type Error = &'static str;
    fn try_from(value: sdl2::controller::Button) -> Result<Self, Self::Error> {
        if let Some(btn) = convert_gamepad_button(value) {
            Ok(btn)
        } else {
            Err("Couldnt map")
        }
    }
}

fn convert_key_code(keycode: Keycode) -> Option<Input> {
    match keycode {
        Keycode::Num1 => Some(Input::Key1),
        Keycode::Num2 => Some(Input::Key2),
        Keycode::Num3 => Some(Input::Key3),
        Keycode::Num4 => Some(Input::Key4),
        Keycode::Num5 => Some(Input::Key5),
        Keycode::Num6 => Some(Input::Key6),
        Keycode::Num7 => Some(Input::Key7),
        Keycode::Num8 => Some(Input::Key8),
        Keycode::Num9 => Some(Input::Key9),
        Keycode::Num0 => Some(Input::Key0),
        Keycode::A => Some(Input::A),
        Keycode::B => Some(Input::B),
        Keycode::C => Some(Input::C),
        Keycode::D => Some(Input::D),
        Keycode::E => Some(Input::E),
        Keycode::F => Some(Input::F),
        Keycode::G => Some(Input::G),
        Keycode::H => Some(Input::H),
        Keycode::I => Some(Input::I),
        Keycode::J => Some(Input::J),
        Keycode::K => Some(Input::K),
        Keycode::L => Some(Input::L),
        Keycode::M => Some(Input::M),
        Keycode::N => Some(Input::N),
        Keycode::O => Some(Input::O),
        Keycode::P => Some(Input::P),
        Keycode::Q => Some(Input::Q),
        Keycode::R => Some(Input::R),
        Keycode::S => Some(Input::S),
        Keycode::T => Some(Input::T),
        Keycode::U => Some(Input::U),
        Keycode::V => Some(Input::V),
        Keycode::W => Some(Input::W),
        Keycode::X => Some(Input::X),
        Keycode::Y => Some(Input::Y),
        Keycode::Z => Some(Input::Z),
        Keycode::Escape => Some(Input::Escape),
        Keycode::F1 => Some(Input::F1),
        Keycode::F2 => Some(Input::F2),
        Keycode::F3 => Some(Input::F3),
        Keycode::F4 => Some(Input::F4),
        Keycode::F5 => Some(Input::F5),
        Keycode::F6 => Some(Input::F6),
        Keycode::F7 => Some(Input::F7),
        Keycode::F8 => Some(Input::F8),
        Keycode::F9 => Some(Input::F9),
        Keycode::F10 => Some(Input::F10),
        Keycode::F11 => Some(Input::F11),
        Keycode::F12 => Some(Input::F12),
        Keycode::Up => Some(Input::Up),
        Keycode::Down => Some(Input::Down),
        Keycode::Left => Some(Input::Left),
        Keycode::Right => Some(Input::Right),
        Keycode::Return => Some(Input::Return),
        Keycode::Space => Some(Input::Space),
        Keycode::Backspace => Some(Input::Back),
        Keycode::Tab => Some(Input::Tab),
        Keycode::LShift | Keycode::RShift => Some(Input::LShift),
        Keycode::LCtrl | Keycode::RCtrl => Some(Input::LControl),
        Keycode::LAlt | Keycode::RAlt => Some(Input::LAlt),
        Keycode::LGui | Keycode::RGui => Some(Input::LWin),
        Keycode::Delete => Some(Input::Delete),
        Keycode::Insert => Some(Input::Insert),
        Keycode::Home => Some(Input::Home),
        Keycode::End => Some(Input::End),
        Keycode::PageUp => Some(Input::PageUp),
        Keycode::PageDown => Some(Input::PageDown),
        Keycode::Comma => Some(Input::Comma),
        Keycode::Period => Some(Input::Period),
        Keycode::Slash => Some(Input::Slash),
        Keycode::Backslash => Some(Input::Backslash),
        Keycode::Semicolon => Some(Input::Semicolon),
        Keycode::Quote => Some(Input::Apostrophe),
        Keycode::Minus => Some(Input::Minus),
        Keycode::Equals => Some(Input::Equals),
        Keycode::LeftBracket => Some(Input::LBracket),
        Keycode::RightBracket => Some(Input::RBracket),
        Keycode::Plus => Some(Input::Plus),
        Keycode::Asterisk => Some(Input::Asterisk),
        Keycode::Colon => Some(Input::Colon),
        Keycode::At => Some(Input::At),
        Keycode::Caret => Some(Input::Caret),
        Keycode::Underscore => Some(Input::Underline),
        Keycode::Backquote => Some(Input::Grave),
        Keycode::PrintScreen => Some(Input::Snapshot),
        Keycode::ScrollLock => Some(Input::Scroll),
        Keycode::Pause => Some(Input::Pause),
        Keycode::NumLockClear => Some(Input::Numlock),
        Keycode::KpDivide => Some(Input::NumpadDivide),
        Keycode::KpMultiply => Some(Input::NumpadMultiply),
        Keycode::KpMinus => Some(Input::NumpadSubtract),
        Keycode::KpPlus => Some(Input::NumpadAdd),
        Keycode::KpEnter => Some(Input::NumpadEnter),
        Keycode::Kp1 => Some(Input::Numpad1),
        Keycode::Kp2 => Some(Input::Numpad2),
        Keycode::Kp3 => Some(Input::Numpad3),
        Keycode::Kp4 => Some(Input::Numpad4),
        Keycode::Kp5 => Some(Input::Numpad5),
        Keycode::Kp6 => Some(Input::Numpad6),
        Keycode::Kp7 => Some(Input::Numpad7),
        Keycode::Kp8 => Some(Input::Numpad8),
        Keycode::Kp9 => Some(Input::Numpad9),
        Keycode::Kp0 => Some(Input::Numpad0),
        Keycode::KpPeriod => Some(Input::NumpadDecimal),
        Keycode::Application => Some(Input::Apps),
        Keycode::Sleep => Some(Input::Sleep),
        Keycode::CapsLock => Some(Input::Capital),
        Keycode::Power => Some(Input::Power),
        Keycode::KpEquals => Some(Input::NumpadEquals),
        Keycode::F13 => Some(Input::F13),
        Keycode::F14 => Some(Input::F14),
        Keycode::F15 => Some(Input::F15),
        Keycode::F16 => Some(Input::F16),
        Keycode::F17 => Some(Input::F17),
        Keycode::F18 => Some(Input::F18),
        Keycode::F19 => Some(Input::F19),
        Keycode::F20 => Some(Input::F20),
        Keycode::F21 => Some(Input::F21),
        Keycode::F22 => Some(Input::F22),
        Keycode::F23 => Some(Input::F23),
        Keycode::F24 => Some(Input::F24),
        Keycode::Stop => Some(Input::Stop),
        Keycode::Cut => Some(Input::Cut),
        Keycode::Copy => Some(Input::Copy),
        Keycode::Paste => Some(Input::Paste),
        Keycode::Mute => Some(Input::Mute),
        Keycode::VolumeUp => Some(Input::VolumeUp),
        Keycode::VolumeDown => Some(Input::VolumeDown),
        Keycode::KpComma => Some(Input::NumpadComma),
        Keycode::KpEqualsAS400 => Some(Input::NumpadEquals),
        Keycode::Sysreq => Some(Input::Sysrq),
        Keycode::Prior => Some(Input::PageUp),
        Keycode::Return2 => Some(Input::NumpadEnter),
        _ => {
            dbg!(format!("Missing keycode: {}", keycode));
            None
        }
    }
}

pub fn convert_mouse_button(mouse_button: sdl2::mouse::MouseButton) -> Option<Input> {
    match mouse_button {
        sdl2::mouse::MouseButton::Left => Some(Input::MouseButtonLeft),
        sdl2::mouse::MouseButton::Right => Some(Input::MouseButtonRight),
        sdl2::mouse::MouseButton::Middle => Some(Input::MouseButtonMiddle),
        sdl2::mouse::MouseButton::Unknown => Some(Input::MouseButtonOther(0)),
        _ => None,
    }
}

pub fn convert_axis(axis: sdl2::controller::Axis) -> Option<Axis> {
    match axis {
        sdl2::controller::Axis::LeftX => Some(Axis::LeftX),
        sdl2::controller::Axis::LeftY => Some(Axis::LeftY),
        sdl2::controller::Axis::RightX => Some(Axis::RightX),
        sdl2::controller::Axis::RightY => Some(Axis::RightY),
        _ => None,
    }
}

pub fn convert_gamepad_button(button: sdl2::controller::Button) -> Option<Input> {
    match button {
        sdl2::controller::Button::A => Some(Input::GamepadA),
        sdl2::controller::Button::B => Some(Input::GamepadB),
        sdl2::controller::Button::X => Some(Input::GamepadX),
        sdl2::controller::Button::Y => Some(Input::GamepadY),
        sdl2::controller::Button::Start => Some(Input::GamepadStart),
        sdl2::controller::Button::LeftStick => Some(Input::GamepadLeftStick),
        sdl2::controller::Button::RightStick => Some(Input::GamepadRightStick),
        sdl2::controller::Button::LeftShoulder => Some(Input::GamepadLeftShoulder),
        sdl2::controller::Button::RightShoulder => Some(Input::GamepadRightShoulder),
        sdl2::controller::Button::DPadUp => Some(Input::GamepadDPadUp),
        sdl2::controller::Button::DPadDown => Some(Input::GamepadDPadDown),
        sdl2::controller::Button::DPadLeft => Some(Input::GamepadDPadLeft),
        sdl2::controller::Button::DPadRight => Some(Input::GamepadDPadRight),
        sdl2::controller::Button::Touchpad => Some(Input::GamepadTouchpad),
        _ => None,
    }
}
