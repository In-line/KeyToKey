use alloc::{format, string::String};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum KeyCode {
    No = 0x00,
    ErrorRollOver,
    PostFail,
    ErrorUndefined,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M, // 0x10
    N,
    O,
    P,
    Q, //20
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,   //29
    Kb1, // Keyboard 1
    Kb2,
    Kb3, // 0x20
    Kb4,
    Kb5,
    Kb6,
    Kb7,
    Kb8,
    Kb9,
    Kb0,
    Enter,
    Escape,
    BSpace,
    Tab,
    Space,
    Minus,
    Equal,
    LBracket,
    RBracket,  // 0x30
    Bslash,    // \ (and |)
    NonUsHash, // Non-US # and ~ (Typically near the Enter key)
    SColon,    // ; (and :)
    Quote,     // ' and "
    Grave,     // Grave accent and tilde
    Comma,     // , and <
    Dot,       // . and >
    Slash,     // / and ?
    CapsLock,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7, // 0x40
    F8,
    F9,
    F10,
    F11,
    F12,
    PScreen,
    ScrollLock,
    Pause,
    Insert,
    Home,
    PgUp,
    Delete,
    End,
    PgDown,
    Right,
    Left, // 0x50
    Down,
    Up,
    NumLock,
    KpSlash,
    KpAsterisk,
    KpMinus,
    KpPlus,
    KpEnter,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8, // 0x60
    Kp9,
    Kp0,
    KpDot,
    NonUsBslash, // Non-US \ and | (Typically near the Left-Shift key)
    Application, // 0x65

    // Modifiers
    LCtrl = 0xE0,
    LShift,
    LAlt,
    LGui,
    RCtrl,
    RShift,
    RAlt,
    RGui, // 0xE7
}
impl KeyCode {
    pub fn is_modifier(self) -> bool {
        KeyCode::LCtrl <= self && self <= KeyCode::RGui
    }
    pub fn as_modifier_bit(self) -> u8 {
        if self.is_modifier() {
            1 << (self as u8 - KeyCode::LCtrl as u8)
        } else {
            0
        }
    }
}

impl Into<u32> for KeyCode {
    fn into(self) -> u32 {
        let r: u8 = self.into();
        return r as u32;
    }
}
