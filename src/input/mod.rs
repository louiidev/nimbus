#[cfg(feature = "winit")]
mod convert_winit_inputs;

#[cfg(feature = "sdl")]
mod convert_sdl_inputs;

use crate::{window::Gamepads, Engine};
use glam::{UVec2, Vec2};
use std::{
    collections::{hash_set::Iter, HashMap, HashSet},
    hash::Hash,
};

#[derive(Default)]
pub struct InputManager {
    pub mouse_position: Vec2,
    pub mouse_motion: Vec2,
    // pub key_mappings: HashMap<String, KeyMapping>,
    pub controllers: Gamepads,
    pub axis: HashMap<Axis, f32>,

    /// A collection of every button that is currently being pressed.
    pub(crate) pressed: HashSet<Input>,
    /// A collection of every button that has just been pressed.
    pub(crate) just_pressed: HashSet<Input>,
    /// A collection of every button that has just been released.
    pub(crate) just_released: HashSet<Input>,
    pub scroll_value: f32,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Axis {
    LeftX,
    LeftY,
    RightX,
    RightY,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Input {
    MouseButtonLeft,
    MouseButtonRight,
    MouseButtonMiddle,
    MouseButtonOther(u16),
    GamepadA,
    GamepadB,
    GamepadX,
    GamepadY,
    GamepadSelect,
    GamepadMode,
    GamepadStart,
    GamepadLeftStick,
    GamepadRightStick,
    GamepadLeftShoulder,
    GamepadLeftTrigger,
    GamepadRightShoulder,
    GamepadRightTrigger,
    GamepadDPadUp,
    GamepadDPadDown,
    GamepadDPadLeft,
    GamepadDPadRight,
    GamepadMisc1,
    GamepadPaddle1,
    GamepadPaddle2,
    GamepadPaddle3,
    GamepadPaddle4,
    GamepadTouchpad,

    /// The `1` key over the letters.
    Key1,
    /// The `2` key over the letters.
    Key2,
    /// The `3` key over the letters.
    Key3,
    /// The `4` key over the letters.
    Key4,
    /// The `5` key over the letters.
    Key5,
    /// The `6` key over the letters.
    Key6,
    /// The `7` key over the letters.
    Key7,
    /// The `8` key over the letters.
    Key8,
    /// The `9` key over the letters.
    Key9,
    /// The `0` key over the letters.
    Key0,

    /// The `A` key.
    A,
    /// The `B` key.
    B,
    /// The `C` key.
    C,
    /// The `D` key.
    D,
    /// The `E` key.
    E,
    /// The `F` key.
    F,
    /// The `G` key.
    G,
    /// The `H` key.
    H,
    /// The `I` key.
    I,
    /// The `J` key.
    J,
    /// The `K` key.
    K,
    /// The `L` key.
    L,
    /// The `M` key.
    M,
    /// The `N` key.
    N,
    /// The `O` key.
    O,
    /// The `P` key.
    P,
    /// The `Q` key.
    Q,
    /// The `R` key.
    R,
    /// The `S` key.
    S,
    /// The `T` key.
    T,
    /// The `U` key.
    U,
    /// The `V` key.
    V,
    /// The `W` key.
    W,
    /// The `X` key.
    X,
    /// The `Y` key.
    Y,
    /// The `Z` key.
    Z,

    /// The `Escape` / `ESC` key, next to the `F1` key.
    Escape,

    /// The `F1` key.
    F1,
    /// The `F2` key.
    F2,
    /// The `F3` key.
    F3,
    /// The `F4` key.
    F4,
    /// The `F5` key.
    F5,
    /// The `F6` key.
    F6,
    /// The `F7` key.
    F7,
    /// The `F8` key.
    F8,
    /// The `F9` key.
    F9,
    /// The `F10` key.
    F10,
    /// The `F11` key.
    F11,
    /// The `F12` key.
    F12,
    /// The `F13` key.
    F13,
    /// The `F14` key.
    F14,
    /// The `F15` key.
    F15,
    /// The `F16` key.
    F16,
    /// The `F17` key.
    F17,
    /// The `F18` key.
    F18,
    /// The `F19` key.
    F19,
    /// The `F20` key.
    F20,
    /// The `F21` key.
    F21,
    /// The `F22` key.
    F22,
    /// The `F23` key.
    F23,
    /// The `F24` key.
    F24,

    /// The `Snapshot` / `Print Screen` key.
    Snapshot,
    /// The `Scroll` / `Scroll Lock` key.
    Scroll,
    /// The `Pause` / `Break` key, next to the `Scroll` key.
    Pause,

    /// The `Insert` key, next to the `Backspace` key.
    Insert,
    /// The `Home` key.
    Home,
    /// The `Delete` key.
    Delete,
    /// The `End` key.
    End,
    /// The `PageDown` key.
    PageDown,
    /// The `PageUp` key.
    PageUp,

    /// The `Left` / `Left Arrow` key.
    Left,
    /// The `Up` / `Up Arrow` key.
    Up,
    /// The `Right` / `Right Arrow` key.
    Right,
    /// The `Down` / `Down Arrow` key.
    Down,

    /// The `Back` / `Backspace` key.
    Back,
    /// The `Return` / `Enter` key.
    Return,
    /// The `Space` / `Spacebar` / ` ` key.
    Space,

    /// The `Compose` key on Linux.
    Compose,
    /// The `Caret` / `^` key.
    Caret,

    /// The `Numlock` key.
    Numlock,
    /// The `Numpad0` / `0` key.
    Numpad0,
    /// The `Numpad1` / `1` key.
    Numpad1,
    /// The `Numpad2` / `2` key.
    Numpad2,
    /// The `Numpad3` / `3` key.
    Numpad3,
    /// The `Numpad4` / `4` key.
    Numpad4,
    /// The `Numpad5` / `5` key.
    Numpad5,
    /// The `Numpad6` / `6` key.
    Numpad6,
    /// The `Numpad7` / `7` key.
    Numpad7,
    /// The `Numpad8` / `8` key.
    Numpad8,
    /// The `Numpad9` / `9` key.
    Numpad9,

    /// The `AbntC1` key.
    AbntC1,
    /// The `AbntC2` key.
    AbntC2,

    /// The `NumpadAdd` / `+` key.
    NumpadAdd,
    /// The `Apostrophe` / `'` key.
    Apostrophe,
    /// The `Apps` key.
    Apps,
    /// The `Asterisk` / `*` key.
    Asterisk,
    /// The `Plus` / `+` key.
    Plus,
    /// The `At` / `@` key.
    At,
    /// The `Ax` key.
    Ax,
    /// The `Backslash` / `\` key.
    Backslash,
    /// The `Calculator` key.
    Calculator,
    /// The `Capital` key.
    Capital,
    /// The `Colon` / `:` key.
    Colon,
    /// The `Comma` / `,` key.
    Comma,
    /// The `Convert` key.
    Convert,
    /// The `NumpadDecimal` / `.` key.
    NumpadDecimal,
    /// The `NumpadDivide` / `/` key.
    NumpadDivide,
    /// The `Equals` / `=` key.
    Equals,
    /// The `Grave` / `Backtick` / `` ` `` key.
    Grave,
    /// The `Kana` key.
    Kana,
    /// The `Kanji` key.
    Kanji,

    /// The `LAlt` / `Left Alt` key. Maps to `Left Option` on Mac.
    LAlt,
    /// The `LBracket` / `Left Bracket` key.
    LBracket,
    /// The `LControl` / `Left Control` key.
    LControl,
    /// The `LShift` / `Left Shift` key.
    LShift,
    /// The `LWin` / `Left Windows` key. Maps to `Left Command` on Mac.
    LWin,

    /// The `Mail` key.
    Mail,
    /// The `MediaSelect` key.
    MediaSelect,
    /// The `MediaStop` key.
    MediaStop,
    /// The `Minus` / `-` key.
    Minus,
    /// The `NumpadMultiply` / `*` key.
    NumpadMultiply,
    /// The `Mute` key.
    Mute,
    /// The `MyComputer` key.
    MyComputer,
    /// The `NavigateForward` / `Prior` key.
    NavigateForward,
    /// The `NavigateBackward` / `Next` key.
    NavigateBackward,
    /// The `NextTrack` key.
    NextTrack,
    /// The `NoConvert` key.
    NoConvert,
    /// The `NumpadComma` / `,` key.
    NumpadComma,
    /// The `NumpadEnter` key.
    NumpadEnter,
    /// The `NumpadEquals` / `=` key.
    NumpadEquals,
    /// The `Oem102` key.
    Oem102,
    /// The `Period` / `.` key.
    Period,
    /// The `PlayPause` key.
    PlayPause,
    /// The `Power` key.
    Power,
    /// The `PrevTrack` key.
    PrevTrack,

    /// The `RAlt` / `Right Alt` key. Maps to `Right Option` on Mac.
    RAlt,
    /// The `RBracket` / `Right Bracket` key.
    RBracket,
    /// The `RControl` / `Right Control` key.
    RControl,
    /// The `RShift` / `Right Shift` key.
    RShift,
    /// The `RWin` / `Right Windows` key. Maps to `Right Command` on Mac.
    RWin,

    /// The `Semicolon` / `;` key.
    Semicolon,
    /// The `Slash` / `/` key.
    Slash,
    /// The `Sleep` key.
    Sleep,
    /// The `Stop` key.
    Stop,
    /// The `NumpadSubtract` / `-` key.
    NumpadSubtract,
    /// The `Sysrq` key.
    Sysrq,
    /// The `Tab` / `   ` key.
    Tab,
    /// The `Underline` / `_` key.
    Underline,
    /// The `Unlabeled` key.
    Unlabeled,

    /// The `VolumeDown` key.
    VolumeDown,
    /// The `VolumeUp` key.
    VolumeUp,

    /// The `Wake` key.
    Wake,

    /// The `WebBack` key.
    WebBack,
    /// The `WebFavorites` key.
    WebFavorites,
    /// The `WebForward` key.
    WebForward,
    /// The `WebHome` key.
    WebHome,
    /// The `WebRefresh` key.
    WebRefresh,
    /// The `WebSearch` key.
    WebSearch,
    /// The `WebStop` key.
    WebStop,

    /// The `Yen` key.
    Yen,

    /// The `Copy` key.
    Copy,
    /// The `Paste` key.
    Paste,
    /// The `Cut` key.
    Cut,
}

pub enum InputState {
    Pressed,
    Released,
}

pub struct InputEvent<T> {
    pub state: InputState,
    pub value: T,
}

impl InputManager {
    // pub fn create_mapping(&mut self, key: impl ToString, mapping: KeyMapping) {
    //     self.key_mappings.insert(key.to_string(), mapping);
    // }

    pub(crate) fn update_input_state(&mut self, input: InputEvent<impl Into<Input>>) {
        let InputEvent { value, state } = input.into();

        match state {
            InputState::Pressed => self.press(value.into()),
            InputState::Released => self.release(value.into()),
        }
    }

    pub(crate) fn update_axis(&mut self, axis: Axis, value: f32) {
        self.axis.insert(axis, value);
    }

    pub(crate) fn update_cursor_position(&mut self, mut mouse_position: Vec2, window_size: UVec2) {
        mouse_position.y = window_size.y as f32 - mouse_position.y;

        self.mouse_position = mouse_position;
    }

    pub fn press(&mut self, input: Input) {
        // Returns `true` if the `input` wasn't pressed.
        if self.pressed.insert(input) {
            self.just_pressed.insert(input);
        }
    }

    /// Registers a release for the given `input`.
    pub fn release(&mut self, input: Input) {
        // Returns `true` if the `input` was pressed.
        if self.pressed.remove(&input) {
            self.just_released.insert(input);
        }
    }

    // pub fn pressed(&self, mapping_key: impl ToString) -> bool {
    //     let mapping = self
    //         .key_mappings
    //         .get(&mapping_key.to_string())
    //         .expect(&format!("Missing mapping for {}", mapping_key.to_string()));

    //     let keypress = mapping
    //         .keys
    //         .iter()
    //         .any(|key| self.keyboards_inputs.pressed(*key));
    //     let button = mapping
    //         .gamepad_button
    //         .iter()
    //         .any(|button| self.gamepad_button_inputs.pressed(*button));

    //     keypress || button
    // }

    // /// Returns `true` if the `input` has just been pressed.
    // pub fn just_pressed(&self, mapping_key: impl ToString) -> bool {
    //     let mapping = self
    //         .key_mappings
    //         .get(&mapping_key.to_string())
    //         .expect(&format!("Missing mapping for {}", mapping_key.to_string()));

    //     let keypress = mapping
    //         .keys
    //         .iter()
    //         .any(|key| self.keyboards_inputs.just_pressed(*key));
    //     let button = mapping
    //         .gamepad_button
    //         .iter()
    //         .any(|button| self.gamepad_button_inputs.just_pressed(*button));

    //     keypress || button
    // }

    // /// Returns `true` if the `input` has just been released.
    // pub fn just_released(&self, mapping_key: impl ToString) -> bool {
    //     let mapping = self
    //         .key_mappings
    //         .get(&mapping_key.to_string())
    //         .expect(&format!("Missing mapping for {}", mapping_key.to_string()));

    //     let keypress = mapping
    //         .keys
    //         .iter()
    //         .any(|key| self.keyboards_inputs.just_released(*key));
    //     let button = mapping
    //         .gamepad_button
    //         .iter()
    //         .any(|button| self.gamepad_button_inputs.just_released(*button));

    //     keypress || button
    // }
}

impl Engine {
    /// Registers a press for the given `input`.

    /// Returns `true` if the `input` has been pressed.
    pub fn pressed(&self, input: Input) -> bool {
        self.input.pressed.contains(&input)
    }

    /// Returns `true` if the `input` has just been pressed.
    pub fn just_pressed(&self, input: Input) -> bool {
        self.input.just_pressed.contains(&input)
    }

    /// Returns `true` if the `input` has just been released.
    pub fn just_released(&self, input: Input) -> bool {
        self.input.just_released.contains(&input)
    }

    /// Clears the `pressed`, `just_pressed` and `just_released` data of the `input`.
    pub fn reset(&mut self, input: Input) {
        self.input.pressed.remove(&input);
        self.input.just_pressed.remove(&input);
        self.input.just_released.remove(&input);
    }

    /// An iterator visiting every pressed input in arbitrary order.
    pub fn get_pressed(&self) -> Iter<Input> {
        self.input.pressed.iter()
    }

    /// An iterator visiting every just pressed input in arbitrary order.
    pub fn get_just_pressed(&self) -> Iter<Input> {
        self.input.just_pressed.iter()
    }

    /// An iterator visiting every just released input in arbitrary order.
    pub fn get_just_released(&self) -> Iter<Input> {
        self.input.just_released.iter()
    }

    /// Clears the `just pressed` and `just released` data for every input.
    ///
    /// See also [`Input::reset_all`] for a full reset.
    pub fn clear_inputs(&mut self) {
        self.input.just_pressed.clear();
        self.input.just_released.clear();
        self.input.scroll_value = 0.;
    }

    pub fn get_axis(&mut self, axis: Axis) -> f32 {
        self.input.axis.get(&axis).copied().unwrap_or(0.)
    }
}
