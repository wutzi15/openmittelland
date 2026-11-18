//! Window-system-independent input. The game is mouse driven: left click acts, right click closes
//! (`docs/spec/01-controls.md`); the keyboard is for names, hotkeys and Escape.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Escape,
    Enter,
    Backspace,
    /// Printable key as CP437 byte (for name entry and hotkeys).
    Char(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Left(i32, i32),
    Right(i32, i32),
    Key(Key),
}

/// Mouse pointer shapes, icons `00-04` (`docs/re/formats.md` §1.7), and the wheel of `wait_click`: frame
/// 0..15 = icons `0x16D..0x17C` (ICONMD..NC, rules-41 §3). Hotspot is the top-left pixel.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Cursor {
    #[default]
    Arrow,
    Plus,
    Minus,
    Up,
    Down,
    Wheel(u8),
}

#[allow(dead_code)]
impl Cursor {
    pub fn icon(self) -> u16 {
        match self {
            Self::Arrow => 0,
            Self::Plus => 1,
            Self::Minus => 2,
            Self::Up => 3,
            Self::Down => 4,
            Self::Wheel(f) => 0x16D + (f % 16) as u16,
        }
    }

    /// Over an editable digit (`plus_minus`) or text value: upper half increments/next, lower half
    /// decrements/previous.
    pub fn for_half(upper: bool, plus_minus: bool) -> Self {
        match (upper, plus_minus) {
            (true, true) => Self::Plus,
            (false, true) => Self::Minus,
            (true, false) => Self::Up,
            (false, false) => Self::Down,
        }
    }
}

/// Unicode to the CP437 byte the font and EXE strings use; `None` for anything the font cannot draw.
pub fn cp437(c: char) -> Option<u8> {
    Some(match c {
        ' '..='~' => c as u8,
        'ü' => 0x81,
        'ä' => 0x84,
        'Ä' => 0x8E,
        'ö' => 0x94,
        'Ö' => 0x99,
        'Ü' => 0x9A,
        'ß' => 0xE1,
        '\u{a0}' => 0xFF,
        _ => return None,
    })
}

/// Inverse of [`cp437`].
pub fn from_cp437(b: u8) -> Option<char> {
    Some(match b {
        0x20..=0x7E => b as char,
        0x81 => 'ü',
        0x84 => 'ä',
        0x8E => 'Ä',
        0x94 => 'ö',
        0x99 => 'Ö',
        0x9A => 'Ü',
        0xE1 => 'ß',
        0xFF => '\u{a0}',
        _ => return None,
    })
}

#[derive(Default)]
pub struct Input {
    pub mouse: (i32, i32),
    pub events: Vec<Event>,
    /// An auto-repeat of the held left button fired this tick (only the game screen uses it).
    pub repeat: bool,
    /// Held left button: (where it went down or last moved to, ms held, ms of the next repeat, current gap).
    pub held: Option<((i32, i32), u32, u32, u32)>,
}

impl Input {
    /// The EXE's `menu` repeats a held button on the item under the pointer: first after 170 ms, then
    /// 130, 90, 50, 10, 10, … ms; moving restarts the count (rules-26 §7).
    pub fn tick_repeat(&mut self, dt: u32) {
        self.repeat = false;
        let mouse = self.mouse;
        let Some((at, ms, next, gap)) = &mut self.held else { return };
        if *at != mouse {
            (*at, *ms, *next, *gap) = (mouse, 0, 170, 170);
            return;
        }
        *ms += dt;
        if *ms >= *next {
            *gap = gap.saturating_sub(40).max(10);
            *next += *gap;
            self.repeat = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Repeats at 170, 300, 390, 440, 450, 460 ms of holding; moving restarts the count.
    #[test]
    fn auto_repeat() {
        let mut j = Input { held: Some(((5, 5), 0, 170, 170)), mouse: (5, 5), ..Default::default() };
        let at: Vec<u32> = (1..=46).filter(|_| { j.tick_repeat(10); j.repeat }).map(|t| t * 10).collect();
        assert_eq!(at, [170, 300, 390, 440, 450, 460]);
        j.mouse = (6, 5);
        j.tick_repeat(10);
        assert_eq!(j.held, Some(((6, 5), 0, 170, 170)));
    }

    #[test]
    fn mapping() {
        assert_eq!(cp437('ß'), Some(0xE1));
        assert_eq!(cp437('a'), Some(b'a'));
        assert_eq!(cp437('€'), None);
        for c in "aZ~ äöüÄÖÜß".chars() {
            assert_eq!(cp437(c).and_then(from_cp437), Some(c));
        }
        assert_eq!(Cursor::for_half(false, true), Cursor::Minus);
        assert_eq!(Cursor::for_half(true, false).icon(), 3);
        assert_eq!((Cursor::Wheel(0).icon(), Cursor::Wheel(15).icon()), (0x16D, 0x17C));
    }
}
