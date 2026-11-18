//! PLACEHOLDER: stands in for every screen that needs `mad-sim` (game start, loading, continuing).
//! Shows what was requested; any click returns to the start menu. Listed in `docs/IMPL-GAPS.md`.

use super::{Ctx, Next, Screen, Window, backdrop, menu};
use crate::{assets::Assets, gfx::Frame, input::{Cursor, Event, Input, cp437}};

pub struct Placeholder {
    text: Vec<u8>,
}

impl Placeholder {
    pub fn new(what: &str) -> Self {
        let body: Vec<u8> = what.chars().filter_map(cp437).collect();
        Self { text: [b"$C^ Platzhalter _$Z$C".as_slice(), &body].concat() }
    }
}

impl Screen for Placeholder {
    fn update(&mut self, ctx: &mut Ctx, input: &Input, _: u32) -> Next {
        if input.events.iter().any(|e| matches!(e, Event::Left(..) | Event::Right(..))) {
            return Next::Go(menu::main_menu(ctx));
        }
        Next::Stay
    }

    fn draw(&self, frame: &mut Frame, assets: &Assets, _: (i32, i32)) -> Option<Cursor> {
        backdrop(frame, assets, b"", &[]);
        Window::new(assets, [100, 180, 440, 120], &self.text, &[], &[], &[]).draw(frame, assets);
        Some(Cursor::Arrow)
    }
}
