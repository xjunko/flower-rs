#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeyCode {
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
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Num0,
    Enter,
    Escape,
    Backspace,
    Tab,
    Space,
    Minus,
    Equal,
    LeftBracket,
    RightBracket,
    Backslash,
    Semicolon,
    Apostrophe,
    Grave,
    Comma,
    Dot,
    Slash,
    CapsLock,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    LeftShift,
    RightShift,
    LeftCtrl,
    RightCtrl,
    LeftAlt,
    RightAlt,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    KeypadAsterisk,
    Unknown,
}

pub const SCANCODE_TABLE: [KeyCode; 70] = [
    KeyCode::Unknown,        // 0x00
    KeyCode::Escape,         // 0x01
    KeyCode::Num1,           // 0x02
    KeyCode::Num2,           // 0x03
    KeyCode::Num3,           // 0x04
    KeyCode::Num4,           // 0x05
    KeyCode::Num5,           // 0x06
    KeyCode::Num6,           // 0x07
    KeyCode::Num7,           // 0x08
    KeyCode::Num8,           // 0x09
    KeyCode::Num9,           // 0x0A
    KeyCode::Num0,           // 0x0B
    KeyCode::Minus,          // 0x0C
    KeyCode::Equal,          // 0x0D
    KeyCode::Backspace,      // 0x0E
    KeyCode::Tab,            // 0x0F
    KeyCode::Q,              // 0x10
    KeyCode::W,              // 0x11
    KeyCode::E,              // 0x12
    KeyCode::R,              // 0x13
    KeyCode::T,              // 0x14
    KeyCode::Y,              // 0x15
    KeyCode::U,              // 0x16
    KeyCode::I,              // 0x17
    KeyCode::O,              // 0x18
    KeyCode::P,              // 0x19
    KeyCode::LeftBracket,    // 0x1A
    KeyCode::RightBracket,   // 0x1B
    KeyCode::Enter,          // 0x1C
    KeyCode::LeftCtrl,       // 0x1D
    KeyCode::A,              // 0x1E
    KeyCode::S,              // 0x1F
    KeyCode::D,              // 0x20
    KeyCode::F,              // 0x21
    KeyCode::G,              // 0x22
    KeyCode::H,              // 0x23
    KeyCode::J,              // 0x24
    KeyCode::K,              // 0x25
    KeyCode::L,              // 0x26
    KeyCode::Semicolon,      // 0x27
    KeyCode::Apostrophe,     // 0x28
    KeyCode::Grave,          // 0x29
    KeyCode::LeftShift,      // 0x2A
    KeyCode::Backslash,      // 0x2B
    KeyCode::Z,              // 0x2C
    KeyCode::X,              // 0x2D
    KeyCode::C,              // 0x2E
    KeyCode::V,              // 0x2F
    KeyCode::B,              // 0x30
    KeyCode::N,              // 0x31
    KeyCode::M,              // 0x32
    KeyCode::Comma,          // 0x33
    KeyCode::Dot,            // 0x34
    KeyCode::Slash,          // 0x35
    KeyCode::RightShift,     // 0x36
    KeyCode::KeypadAsterisk, // 0x37 (* on numpad)
    KeyCode::LeftAlt,        // 0x38
    KeyCode::Space,          // 0x39
    KeyCode::CapsLock,       // 0x3A
    // F1–F10
    KeyCode::F1,
    KeyCode::F2,
    KeyCode::F3,
    KeyCode::F4,
    KeyCode::F5,
    KeyCode::F6,
    KeyCode::F7,
    KeyCode::F8,
    KeyCode::F9,
    KeyCode::F10,
    KeyCode::Unknown, // remaining...
];

pub fn scancode_to_ascii(scancode: u8, shifted: bool) -> Option<u8> {
    let keycode = scancode_to_keycode(scancode);
    let ascii = match (keycode, shifted) {
        (KeyCode::A, false) => b'a',
        (KeyCode::A, true) => b'A',
        (KeyCode::B, false) => b'b',
        (KeyCode::B, true) => b'B',
        (KeyCode::C, false) => b'c',
        (KeyCode::C, true) => b'C',
        (KeyCode::D, false) => b'd',
        (KeyCode::D, true) => b'D',
        (KeyCode::E, false) => b'e',
        (KeyCode::E, true) => b'E',
        (KeyCode::F, false) => b'f',
        (KeyCode::F, true) => b'F',
        (KeyCode::G, false) => b'g',
        (KeyCode::G, true) => b'G',
        (KeyCode::H, false) => b'h',
        (KeyCode::H, true) => b'H',
        (KeyCode::I, false) => b'i',
        (KeyCode::I, true) => b'I',
        (KeyCode::J, false) => b'j',
        (KeyCode::J, true) => b'J',
        (KeyCode::K, false) => b'k',
        (KeyCode::K, true) => b'K',
        (KeyCode::L, false) => b'l',
        (KeyCode::L, true) => b'L',
        (KeyCode::M, false) => b'm',
        (KeyCode::M, true) => b'M',
        (KeyCode::N, false) => b'n',
        (KeyCode::N, true) => b'N',
        (KeyCode::O, false) => b'o',
        (KeyCode::O, true) => b'O',
        (KeyCode::P, false) => b'p',
        (KeyCode::P, true) => b'P',
        (KeyCode::Q, false) => b'q',
        (KeyCode::Q, true) => b'Q',
        (KeyCode::R, false) => b'r',
        (KeyCode::R, true) => b'R',
        (KeyCode::S, false) => b's',
        (KeyCode::S, true) => b'S',
        (KeyCode::T, false) => b't',
        (KeyCode::T, true) => b'T',
        (KeyCode::U, false) => b'u',
        (KeyCode::U, true) => b'U',
        (KeyCode::V, false) => b'v',
        (KeyCode::V, true) => b'V',
        (KeyCode::W, false) => b'w',
        (KeyCode::W, true) => b'W',
        (KeyCode::X, false) => b'x',
        (KeyCode::X, true) => b'X',
        (KeyCode::Y, false) => b'y',
        (KeyCode::Y, true) => b'Y',
        (KeyCode::Z, false) => b'z',
        (KeyCode::Z, true) => b'Z',
        (KeyCode::Num1, false) => b'1',
        (KeyCode::Num1, true) => b'!',
        (KeyCode::Num2, false) => b'2',
        (KeyCode::Num2, true) => b'@',
        (KeyCode::Num3, false) => b'3',
        (KeyCode::Num3, true) => b'#',
        (KeyCode::Num4, false) => b'4',
        (KeyCode::Num4, true) => b'$',
        (KeyCode::Num5, false) => b'5',
        (KeyCode::Num5, true) => b'%',
        (KeyCode::Num6, false) => b'6',
        (KeyCode::Num6, true) => b'^',
        (KeyCode::Num7, false) => b'7',
        (KeyCode::Num7, true) => b'&',
        (KeyCode::Num8, false) => b'8',
        (KeyCode::Num8, true) => b'*',
        (KeyCode::Num9, false) => b'9',
        (KeyCode::Num9, true) => b'(',
        (KeyCode::Num0, false) => b'0',
        (KeyCode::Num0, true) => b')',
        (KeyCode::Space, _) => b' ',
        (KeyCode::Enter, _) => b'\n',
        (KeyCode::Tab, _) => b'\t',
        (KeyCode::Backspace, _) => 0x08,
        (KeyCode::Minus, false) => b'-',
        (KeyCode::Minus, true) => b'_',
        (KeyCode::Equal, false) => b'=',
        (KeyCode::Equal, true) => b'+',
        (KeyCode::LeftBracket, false) => b'[',
        (KeyCode::LeftBracket, true) => b'{',
        (KeyCode::RightBracket, false) => b']',
        (KeyCode::RightBracket, true) => b'}',
        (KeyCode::Backslash, false) => b'\\',
        (KeyCode::Backslash, true) => b'|',
        (KeyCode::Semicolon, false) => b';',
        (KeyCode::Semicolon, true) => b':',
        (KeyCode::Apostrophe, false) => b'\'',
        (KeyCode::Apostrophe, true) => b'"',
        (KeyCode::Grave, false) => b'`',
        (KeyCode::Grave, true) => b'~',
        (KeyCode::Comma, false) => b',',
        (KeyCode::Comma, true) => b'<',
        (KeyCode::Dot, false) => b'.',
        (KeyCode::Dot, true) => b'>',
        (KeyCode::Slash, false) => b'/',
        (KeyCode::Slash, true) => b'?',
        _ => return None,
    };

    Some(ascii)
}

pub fn scancode_to_keycode(scancode: u8) -> KeyCode {
    if scancode < SCANCODE_TABLE.len() as u8 {
        SCANCODE_TABLE[scancode as usize]
    } else {
        KeyCode::Unknown
    }
}
