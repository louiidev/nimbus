use winit::event::ElementState;

use super::{Axis, Input, InputState};

impl From<winit::event::VirtualKeyCode> for Input {
    fn from(value: winit::event::VirtualKeyCode) -> Self {
        convert_key_code(value)
    }
}

impl From<winit::event::MouseButton> for Input {
    fn from(value: winit::event::MouseButton) -> Self {
        convert_mouse_button(value)
    }
}

impl From<ElementState> for InputState {
    fn from(value: ElementState) -> Self {
        convert_state(value)
    }
}

impl TryFrom<gilrs::Button> for Input {
    type Error = &'static str;
    fn try_from(value: gilrs::Button) -> Result<Self, Self::Error> {
        if let Some(input) = convert_button(value) {
            Ok(input)
        } else {
            Err("Couldnt map")
        }
    }
}

impl TryFrom<gilrs::Axis> for Axis {
    type Error = &'static str;
    fn try_from(value: gilrs::Axis) -> Result<Self, Self::Error> {
        if let Some(axis) = convert_axis(value) {
            Ok(axis)
        } else {
            Err("Couldnt map")
        }
    }
}

pub fn convert_state(state: ElementState) -> InputState {
    match state {
        ElementState::Pressed => InputState::Pressed,
        ElementState::Released => InputState::Released,
    }
}

pub fn convert_mouse_button(mouse_button: winit::event::MouseButton) -> Input {
    match mouse_button {
        winit::event::MouseButton::Left => Input::MouseButtonLeft,
        winit::event::MouseButton::Right => Input::MouseButtonRight,
        winit::event::MouseButton::Middle => Input::MouseButtonMiddle,
        winit::event::MouseButton::Other(v) => Input::MouseButtonOther(v),
    }
}

pub fn convert_key_code(key_code: winit::event::VirtualKeyCode) -> Input {
    match key_code {
        winit::event::VirtualKeyCode::Key1 => Input::Key1,
        winit::event::VirtualKeyCode::Key2 => Input::Key2,
        winit::event::VirtualKeyCode::Key3 => Input::Key3,
        winit::event::VirtualKeyCode::Key4 => Input::Key4,
        winit::event::VirtualKeyCode::Key5 => Input::Key5,
        winit::event::VirtualKeyCode::Key6 => Input::Key6,
        winit::event::VirtualKeyCode::Key7 => Input::Key7,
        winit::event::VirtualKeyCode::Key8 => Input::Key8,
        winit::event::VirtualKeyCode::Key9 => Input::Key9,
        winit::event::VirtualKeyCode::Key0 => Input::Key0,
        winit::event::VirtualKeyCode::A => Input::A,
        winit::event::VirtualKeyCode::B => Input::B,
        winit::event::VirtualKeyCode::C => Input::C,
        winit::event::VirtualKeyCode::D => Input::D,
        winit::event::VirtualKeyCode::E => Input::E,
        winit::event::VirtualKeyCode::F => Input::F,
        winit::event::VirtualKeyCode::G => Input::G,
        winit::event::VirtualKeyCode::H => Input::H,
        winit::event::VirtualKeyCode::I => Input::I,
        winit::event::VirtualKeyCode::J => Input::J,
        winit::event::VirtualKeyCode::K => Input::K,
        winit::event::VirtualKeyCode::L => Input::L,
        winit::event::VirtualKeyCode::M => Input::M,
        winit::event::VirtualKeyCode::N => Input::N,
        winit::event::VirtualKeyCode::O => Input::O,
        winit::event::VirtualKeyCode::P => Input::P,
        winit::event::VirtualKeyCode::Q => Input::Q,
        winit::event::VirtualKeyCode::R => Input::R,
        winit::event::VirtualKeyCode::S => Input::S,
        winit::event::VirtualKeyCode::T => Input::T,
        winit::event::VirtualKeyCode::U => Input::U,
        winit::event::VirtualKeyCode::V => Input::V,
        winit::event::VirtualKeyCode::W => Input::W,
        winit::event::VirtualKeyCode::X => Input::X,
        winit::event::VirtualKeyCode::Y => Input::Y,
        winit::event::VirtualKeyCode::Z => Input::Z,
        winit::event::VirtualKeyCode::Escape => Input::Escape,
        winit::event::VirtualKeyCode::F1 => Input::F1,
        winit::event::VirtualKeyCode::F2 => Input::F2,
        winit::event::VirtualKeyCode::F3 => Input::F3,
        winit::event::VirtualKeyCode::F4 => Input::F4,
        winit::event::VirtualKeyCode::F5 => Input::F5,
        winit::event::VirtualKeyCode::F6 => Input::F6,
        winit::event::VirtualKeyCode::F7 => Input::F7,
        winit::event::VirtualKeyCode::F8 => Input::F8,
        winit::event::VirtualKeyCode::F9 => Input::F9,
        winit::event::VirtualKeyCode::F10 => Input::F10,
        winit::event::VirtualKeyCode::F11 => Input::F11,
        winit::event::VirtualKeyCode::F12 => Input::F12,
        winit::event::VirtualKeyCode::F13 => Input::F13,
        winit::event::VirtualKeyCode::F14 => Input::F14,
        winit::event::VirtualKeyCode::F15 => Input::F15,
        winit::event::VirtualKeyCode::F16 => Input::F16,
        winit::event::VirtualKeyCode::F17 => Input::F17,
        winit::event::VirtualKeyCode::F18 => Input::F18,
        winit::event::VirtualKeyCode::F19 => Input::F19,
        winit::event::VirtualKeyCode::F20 => Input::F20,
        winit::event::VirtualKeyCode::F21 => Input::F21,
        winit::event::VirtualKeyCode::F22 => Input::F22,
        winit::event::VirtualKeyCode::F23 => Input::F23,
        winit::event::VirtualKeyCode::F24 => Input::F24,
        winit::event::VirtualKeyCode::Snapshot => Input::Snapshot,
        winit::event::VirtualKeyCode::Scroll => Input::Scroll,
        winit::event::VirtualKeyCode::Pause => Input::Pause,
        winit::event::VirtualKeyCode::Insert => Input::Insert,
        winit::event::VirtualKeyCode::Home => Input::Home,
        winit::event::VirtualKeyCode::Delete => Input::Delete,
        winit::event::VirtualKeyCode::End => Input::End,
        winit::event::VirtualKeyCode::PageDown => Input::PageDown,
        winit::event::VirtualKeyCode::PageUp => Input::PageUp,
        winit::event::VirtualKeyCode::Left => Input::Left,
        winit::event::VirtualKeyCode::Up => Input::Up,
        winit::event::VirtualKeyCode::Right => Input::Right,
        winit::event::VirtualKeyCode::Down => Input::Down,
        winit::event::VirtualKeyCode::Back => Input::Back,
        winit::event::VirtualKeyCode::Return => Input::Return,
        winit::event::VirtualKeyCode::Space => Input::Space,
        winit::event::VirtualKeyCode::Compose => Input::Compose,
        winit::event::VirtualKeyCode::Caret => Input::Caret,
        winit::event::VirtualKeyCode::Numlock => Input::Numlock,
        winit::event::VirtualKeyCode::Numpad0 => Input::Numpad0,
        winit::event::VirtualKeyCode::Numpad1 => Input::Numpad1,
        winit::event::VirtualKeyCode::Numpad2 => Input::Numpad2,
        winit::event::VirtualKeyCode::Numpad3 => Input::Numpad3,
        winit::event::VirtualKeyCode::Numpad4 => Input::Numpad4,
        winit::event::VirtualKeyCode::Numpad5 => Input::Numpad5,
        winit::event::VirtualKeyCode::Numpad6 => Input::Numpad6,
        winit::event::VirtualKeyCode::Numpad7 => Input::Numpad7,
        winit::event::VirtualKeyCode::Numpad8 => Input::Numpad8,
        winit::event::VirtualKeyCode::Numpad9 => Input::Numpad9,
        winit::event::VirtualKeyCode::AbntC1 => Input::AbntC1,
        winit::event::VirtualKeyCode::AbntC2 => Input::AbntC2,
        winit::event::VirtualKeyCode::NumpadAdd => Input::NumpadAdd,
        winit::event::VirtualKeyCode::Apostrophe => Input::Apostrophe,
        winit::event::VirtualKeyCode::Apps => Input::Apps,
        winit::event::VirtualKeyCode::Asterisk => Input::Asterisk,
        winit::event::VirtualKeyCode::Plus => Input::Plus,
        winit::event::VirtualKeyCode::At => Input::At,
        winit::event::VirtualKeyCode::Ax => Input::Ax,
        winit::event::VirtualKeyCode::Backslash => Input::Backslash,
        winit::event::VirtualKeyCode::Calculator => Input::Calculator,
        winit::event::VirtualKeyCode::Capital => Input::Capital,
        winit::event::VirtualKeyCode::Colon => Input::Colon,
        winit::event::VirtualKeyCode::Comma => Input::Comma,
        winit::event::VirtualKeyCode::Convert => Input::Convert,
        winit::event::VirtualKeyCode::NumpadDecimal => Input::NumpadDecimal,
        winit::event::VirtualKeyCode::NumpadDivide => Input::NumpadDivide,
        winit::event::VirtualKeyCode::Equals => Input::Equals,
        winit::event::VirtualKeyCode::Grave => Input::Grave,
        winit::event::VirtualKeyCode::Kana => Input::Kana,
        winit::event::VirtualKeyCode::Kanji => Input::Kanji,
        winit::event::VirtualKeyCode::LAlt => Input::LAlt,
        winit::event::VirtualKeyCode::LBracket => Input::LBracket,
        winit::event::VirtualKeyCode::LControl => Input::LControl,
        winit::event::VirtualKeyCode::LShift => Input::LShift,
        winit::event::VirtualKeyCode::LWin => Input::LWin,
        winit::event::VirtualKeyCode::Mail => Input::Mail,
        winit::event::VirtualKeyCode::MediaSelect => Input::MediaSelect,
        winit::event::VirtualKeyCode::MediaStop => Input::MediaStop,
        winit::event::VirtualKeyCode::Minus => Input::Minus,
        winit::event::VirtualKeyCode::NumpadMultiply => Input::NumpadMultiply,
        winit::event::VirtualKeyCode::Mute => Input::Mute,
        winit::event::VirtualKeyCode::MyComputer => Input::MyComputer,
        winit::event::VirtualKeyCode::NavigateForward => Input::NavigateForward,
        winit::event::VirtualKeyCode::NavigateBackward => Input::NavigateBackward,
        winit::event::VirtualKeyCode::NextTrack => Input::NextTrack,
        winit::event::VirtualKeyCode::NoConvert => Input::NoConvert,
        winit::event::VirtualKeyCode::NumpadComma => Input::NumpadComma,
        winit::event::VirtualKeyCode::NumpadEnter => Input::NumpadEnter,
        winit::event::VirtualKeyCode::NumpadEquals => Input::NumpadEquals,
        winit::event::VirtualKeyCode::OEM102 => Input::Oem102,
        winit::event::VirtualKeyCode::Period => Input::Period,
        winit::event::VirtualKeyCode::PlayPause => Input::PlayPause,
        winit::event::VirtualKeyCode::Power => Input::Power,
        winit::event::VirtualKeyCode::PrevTrack => Input::PrevTrack,
        winit::event::VirtualKeyCode::RAlt => Input::RAlt,
        winit::event::VirtualKeyCode::RBracket => Input::RBracket,
        winit::event::VirtualKeyCode::RControl => Input::RControl,
        winit::event::VirtualKeyCode::RShift => Input::RShift,
        winit::event::VirtualKeyCode::RWin => Input::RWin,
        winit::event::VirtualKeyCode::Semicolon => Input::Semicolon,
        winit::event::VirtualKeyCode::Slash => Input::Slash,
        winit::event::VirtualKeyCode::Sleep => Input::Sleep,
        winit::event::VirtualKeyCode::Stop => Input::Stop,
        winit::event::VirtualKeyCode::NumpadSubtract => Input::NumpadSubtract,
        winit::event::VirtualKeyCode::Sysrq => Input::Sysrq,
        winit::event::VirtualKeyCode::Tab => Input::Tab,
        winit::event::VirtualKeyCode::Underline => Input::Underline,
        winit::event::VirtualKeyCode::Unlabeled => Input::Unlabeled,
        winit::event::VirtualKeyCode::VolumeDown => Input::VolumeDown,
        winit::event::VirtualKeyCode::VolumeUp => Input::VolumeUp,
        winit::event::VirtualKeyCode::Wake => Input::Wake,
        winit::event::VirtualKeyCode::WebBack => Input::WebBack,
        winit::event::VirtualKeyCode::WebFavorites => Input::WebFavorites,
        winit::event::VirtualKeyCode::WebForward => Input::WebForward,
        winit::event::VirtualKeyCode::WebHome => Input::WebHome,
        winit::event::VirtualKeyCode::WebRefresh => Input::WebRefresh,
        winit::event::VirtualKeyCode::WebSearch => Input::WebSearch,
        winit::event::VirtualKeyCode::WebStop => Input::WebStop,
        winit::event::VirtualKeyCode::Yen => Input::Yen,
        winit::event::VirtualKeyCode::Copy => Input::Copy,
        winit::event::VirtualKeyCode::Paste => Input::Paste,
        winit::event::VirtualKeyCode::Cut => Input::Cut,
    }
}

pub fn convert_button(button: gilrs::Button) -> Option<Input> {
    match button {
        gilrs::Button::South => Some(Input::GamepadA),
        gilrs::Button::East => Some(Input::GamepadB),
        gilrs::Button::North => Some(Input::GamepadY),
        gilrs::Button::West => Some(Input::GamepadX),
        gilrs::Button::C => None,
        gilrs::Button::Z => None,
        gilrs::Button::LeftTrigger => Some(Input::GamepadLeftShoulder),
        gilrs::Button::LeftTrigger2 => Some(Input::GamepadLeftTrigger),
        gilrs::Button::RightTrigger => Some(Input::GamepadRightShoulder),
        gilrs::Button::RightTrigger2 => Some(Input::GamepadRightTrigger),
        gilrs::Button::Select => Some(Input::GamepadSelect),
        gilrs::Button::Start => Some(Input::GamepadStart),
        gilrs::Button::Mode => Some(Input::GamepadMode),
        gilrs::Button::LeftThumb => Some(Input::GamepadLeftStick),
        gilrs::Button::RightThumb => Some(Input::GamepadRightStick),
        gilrs::Button::DPadUp => Some(Input::GamepadDPadUp),
        gilrs::Button::DPadDown => Some(Input::GamepadDPadDown),
        gilrs::Button::DPadLeft => Some(Input::GamepadDPadLeft),
        gilrs::Button::DPadRight => Some(Input::GamepadDPadRight),
        gilrs::Button::Unknown => None,
    }
}

pub fn convert_axis(axis: gilrs::Axis) -> Option<Axis> {
    match axis {
        gilrs::Axis::LeftStickX => Some(Axis::LeftX),
        gilrs::Axis::LeftStickY => Some(Axis::LeftY),
        gilrs::Axis::RightStickX => Some(Axis::RightX),
        gilrs::Axis::RightStickY => Some(Axis::RightY),
        _ => None,
    }
}
