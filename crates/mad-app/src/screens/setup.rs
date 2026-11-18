//! Player creation (`docs/re/formats.md` §8.2, `docs/spec/02-game-setup.md` steps 3-7), once per
//! player: full-screen book + banner, left column name/sex/faith/colour, right column hotkey and
//! confirmation. All text is the EXE's own markup. The inherited-branch message needs the sim and
//! is left out.

use super::{Ctx, Next, Screen, Window, backdrop, game::Game, menu, placeholder::Placeholder};
use crate::{
    assets::Assets,
    gfx::Frame,
    input::{Cursor, Event, Input, Key},
    text::{self, Item, Laid},
};

mod addr {
    pub const PLAYER_NO: u32 = 0xBDE40;
    pub const LEFT_NAME: u32 = 0xBDE60;
    pub const ITEM_LINE: u32 = 0xBDE7C;
    pub const SEX: u32 = 0xBDE8C;
    pub const FAITH: u32 = 0xBDEAC;
    pub const COLOUR: u32 = 0xBDEBC;
    pub const COLOUR_ROW: u32 = 0xBDED0;
    pub const RIGHT: u32 = 0xBDED4;
    pub const HOTKEY: u32 = 0xBDEE4;
    pub const HOTKEY_LINE: u32 = 0xBDEF4;
    pub const INHERIT: u32 = 0xBDF00;
    pub const CODE_PROMPT: u32 = 0xBA430;
    pub const CODE_WRONG: u32 = 0xBA4A0;
    pub const CODE_COLOUR: u32 = 0xBA4D8;
    /// 90 u16 secret codes, 3 per mission (rules-10 §2); colour names of the three card colours.
    pub const CODES: u32 = 0x84A34;
    pub const CARD_COLOURS: (u32, usize) = (0xB737C, 3);
    pub const CONTENT: u32 = 0xBDF68;
    pub const YES: u32 = 0xBDBB8;
    pub const NO: u32 = 0xBDBB0;
    pub const MALE: u32 = 0xAF3FB;
    pub const FEMALE: u32 = 0xAF405;
    pub const PROTESTANT: u32 = 0xAF40F;
    pub const CATHOLIC: u32 = 0xAF41C;
}

/// `ICONP6`, the flag bank shown on the six colour buttons.
const FLAGS: u16 = 0x196;
const BUTTON: i32 = 40;
/// Left column: `$10>$56<` on a 640-wide window.
const LEFT_X: i32 = 60;
const LEFT_W: i32 = 640 - 60 - 336;
/// Windows of the code entry: our reading of the original's picture (docs/FIDELITY.md).
const CODE_BOX: [i32; 4] = [160, 120, 320, 240];
const NOTICE_BOX: [i32; 4] = CODE_BOX;
const NAME_MAX: usize = 23; // save block names are 24 bytes, NUL-terminated

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Player {
    pub name: Vec<u8>,
    pub female: bool,
    pub catholic: bool,
    pub colour: u8,
    pub hotkey: Option<u8>,
    pub mission: Option<u8>,
}

/// Steps of one player's creation; each reveals the next part of the page (checked against the
/// original in DOSBox-X, docs/FIDELITY.md): name, sex, faith, colour, hotkey, then the question.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Step {
    Name,
    Sex,
    Faith,
    Colour,
    Hotkey,
    Confirm,
    /// secret code of a mission card (only with missions, rules-10 §2)
    Code,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    Name,
    Male,
    Female,
    Prot,
    Cath,
    Letter(u8),
    Yes,
    No,
}

pub struct PlayerSetup {
    difficulty: u8,
    count: usize,
    done: Vec<Player>,
    cur: Player,
    step: Step,
    /// the world the players are created in (the start city is picked while they confirm)
    world: mad_sim::State,
    city: Option<u8>,
    code: Vec<u8>,
    /// message box over the code entry (wrong code), closed by a click
    notice: Option<Vec<u8>>,
    /// "Neuer Spieler" in a running game: the state to return to (Escape) and the humans already in it
    joining: Option<(mad_sim::State, usize)>,
}

/// The page of one step: markup, choice texts and roles, and where the colour row is (if shown).
struct Page {
    src: Vec<u8>,
    items: Vec<Vec<u8>>,
    roles: Vec<Role>,
    args: Vec<Vec<u8>>,
    /// y of the colour buttons: the height of the markup up to the colour title
    row_y: Option<i32>,
}

/// Plain text to CP437 with the markup characters doubled.
fn esc(s: &str) -> Vec<u8> {
    s.chars().filter_map(crate::input::cp437).flat_map(|b| if b == b'$' || b == b'%' { vec![b, b] } else { vec![b] }).collect()
}

fn replace(src: &[u8], pat: &[u8], with: &[u8]) -> Vec<u8> {
    let mut out = vec![];
    let mut i = 0;
    while i < src.len() {
        if src[i..].starts_with(pat) {
            out.extend_from_slice(with);
            i += pat.len();
        } else {
            out.push(src[i]);
            i += 1;
        }
    }
    out
}

impl PlayerSetup {
    pub fn new(difficulty: u8, count: usize) -> Self {
        // Seeded from the clock: a new game should differ each time; the sim itself stays deterministic.
        let seed = std::env::var("MAD_SEED").ok().and_then(|v| v.parse().ok()).unwrap_or_else(|| std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.as_secs() as u32));
        Self { difficulty, count, done: vec![], cur: Player::default(), step: Step::Name, world: mad_sim::State::begin_game(seed, difficulty), city: None, code: vec![], notice: None, joining: None }
    }

    /// One more human in a running game (rules-11 §1); the caller has checked that a city is free.
    pub fn join(st: mad_sim::State) -> Self {
        let n = st.humans().count();
        Self { difficulty: st.globals.b(mad_sim::state::ga::DIFFICULTY), count: 1, done: vec![], cur: Player::default(), step: Step::Name, world: st.clone(), city: None, code: vec![], notice: None, joining: Some((st, n)) }
    }

    fn page(&self, assets: &Assets) -> Page {
        let t = |a| assets.text(a);
        let mut p = Page { src: vec![], items: vec![], roles: vec![], args: vec![], row_y: None };
        let n = self.done.len() + self.joining.as_ref().map_or(0, |j| j.1);
        p.args = vec![vec![b'0' + n as u8 + 1], vec![self.cur.hotkey.unwrap_or(b' ').to_ascii_uppercase()]];
        // a player joining a running game gets the plain heading (formats §8.2 step 3, live capture)
        let head = if self.joining.is_some() { b"$|$H$Z$CEin neuer Spieler$Z".to_vec() } else { t(addr::PLAYER_NO) };
        // while typing, the name is red and left-aligned at x 102 (live capture); afterwards centred
        let line = if self.step == Step::Name { [b"$L$17>".as_slice(), &replace(&t(addr::ITEM_LINE), b"$C", b"")].concat() } else { t(addr::ITEM_LINE) };
        p.src = [head, t(addr::LEFT_NAME), line].concat();
        p.items.push(self.cur.name.clone());
        p.roles.push(Role::Name);
        let (male, female, prot, cath) = (t(addr::MALE), t(addr::FEMALE), t(addr::PROTESTANT), t(addr::CATHOLIC));
        if self.step >= Step::Sex {
            p.src.extend(t(addr::SEX));
            let pair = if self.step == Step::Sex {
                vec![(male.clone(), Role::Male), (female.clone(), Role::Female)]
            } else if self.cur.female {
                vec![(female, Role::Female)]
            } else {
                vec![(male, Role::Male)]
            };
            for (txt, r) in pair {
                p.src.extend(b"%m$N");
                p.items.push(txt);
                p.roles.push(r);
            }
        }
        if self.step >= Step::Faith {
            p.src.extend(t(addr::FAITH));
            let pair = if self.step == Step::Faith {
                vec![(prot.clone(), Role::Prot), (cath.clone(), Role::Cath)]
            } else if self.cur.catholic {
                vec![(cath, Role::Cath)]
            } else {
                vec![(prot, Role::Prot)]
            };
            for (txt, r) in pair {
                p.src.extend(b"%m$N");
                p.items.push(txt);
                p.roles.push(r);
            }
        }
        if self.step >= Step::Colour {
            p.src.extend(t(addr::COLOUR));
            p.row_y = Some(text::layout(&assets.font, &p.src, &p.items.iter().map(Vec::as_slice).collect::<Vec<_>>(), &[&p.args[0]], 640, &[]).height);
            p.src.extend(t(addr::COLOUR_ROW));
        }
        if self.step >= Step::Hotkey {
            p.src.extend(t(addr::RIGHT));
            p.src.extend(t(addr::HOTKEY));
            if self.step == Step::Hotkey {
                for (k, c) in (b'a'..=b'z').enumerate() {
                    p.src.extend(if k == 0 { &b"%m"[..] } else { &b" %m"[..] });
                    p.items.push(vec![c]);
                    p.roles.push(Role::Letter(c));
                }
                p.src.extend(b"$N");
            } else {
                p.src.extend(t(addr::HOTKEY_LINE));
            }
        }
        if self.step >= Step::Confirm {
            let city = assets.name_table((0xB6194, 21)).get(self.city.unwrap_or(0) as usize).cloned().unwrap_or_default();
            p.args.push(esc(&city));
            let name: Vec<u8> = self.cur.name.iter().flat_map(|&b| if b == b'$' || b == b'%' { vec![b, b] } else { vec![b] }).collect();
            p.src.extend(replace(&t(addr::INHERIT), b"%1N0", &name));
            p.src.extend(t(addr::CONTENT));
            p.items.push(t(addr::YES));
            p.roles.push(Role::Yes);
            p.items.push(t(addr::NO));
            p.roles.push(Role::No);
        }
        p
    }

    fn layout(&self, assets: &Assets, hover: Option<usize>) -> (Laid, Page) {
        let p = self.page(assets);
        let items: Vec<&[u8]> = p.items.iter().map(Vec::as_slice).collect();
        let args: Vec<&[u8]> = p.args.iter().map(Vec::as_slice).collect();
        let lit: Vec<usize> = hover.into_iter().chain((self.step == Step::Name).then_some(0)).collect();
        (text::layout(&assets.font, &p.src, &items, &args, 640, &lit), p)
    }

    /// The six colour buttons, centred in the left column (§8.2); after the choice only the chosen one.
    fn buttons(&self, row_y: i32) -> Vec<(u8, Item)> {
        let all = self.step == Step::Colour;
        let n = if all { 6 } else { 1 };
        let x0 = LEFT_X + (LEFT_W - n * BUTTON) / 2;
        (0..n).map(|k| (if all { k as u8 } else { self.cur.colour }, Item { x: x0 + k * BUTTON, y: row_y, w: BUTTON, h: BUTTON })).collect()
    }
}

fn setup_of(p: &Player, city: u8) -> mad_sim::world::HumanSetup {
    mad_sim::world::HumanSetup {
        name: p.name.iter().filter_map(|&b| crate::input::from_cp437(b)).collect(),
        gender: p.female as u8,
        religion: p.catholic as u8, // 0 evangelisch, 1 katholisch (rules-00 §4)
        colour: p.colour,
        hotkey: p.hotkey.unwrap_or(0),
        city,
        mission: p.mission,
    }
}

impl PlayerSetup {
    /// Looks the typed code up in the EXE's table (rules-10 §2); a valid card of the chosen
    /// difficulty sets the mission and goes back to the confirmation, which then finishes.
    fn check_code(&mut self, assets: &Assets) {
        let typed: u16 = String::from_utf8_lossy(&self.code).parse().unwrap_or(u16::MAX);
        let table = assets.exe_bytes(addr::CODES, 180);
        let found = table.chunks(2).position(|c| u16::from_le_bytes([c[0], c[1]]) == typed);
        self.code.clear();
        match found {
            None => self.notice = Some(assets.text(addr::CODE_WRONG)),
            Some(i) if i / 3 / 10 != self.difficulty as usize => {
                let colour = assets.name_table(addr::CARD_COLOURS).get(i / 3 / 10).cloned().unwrap_or_default();
                let src = replace(&assets.text(addr::CODE_COLOUR), b"%s", &esc(&colour));
                self.notice = Some(src);
            }
            Some(i) => {
                self.cur.mission = Some((i / 3) as u8);
                self.step = Step::Confirm;
            }
        }
    }

    /// Enters the last step: the game picks the start city (rules-11 §1), the page names it.
    fn confirm(&mut self) {
        self.city = self.world.pick_start_city();
        // debug aid for comparisons with the original: MAD_CITY=<0..9> forces the start city
        if let Some(c) = std::env::var("MAD_CITY").ok().and_then(|v| v.parse().ok()) {
            self.city = Some(c);
        }
        self.step = Step::Confirm;
    }
}

impl Screen for PlayerSetup {
    fn update(&mut self, ctx: &mut Ctx, input: &Input, _: u32) -> Next {
        for e in &input.events {
            match *e {
                Event::Left(..) | Event::Right(..) | Event::Key(_) if self.notice.is_some() => self.notice = None,
                Event::Key(Key::Char(c)) if self.step == Step::Code && c.is_ascii_digit() && self.code.len() < 4 => {
                    self.code.push(c);
                    if self.code.len() == 4 {
                        self.check_code(ctx.assets);
                    }
                }
                Event::Key(Key::Backspace) if self.step == Step::Code => _ = self.code.pop(),
                Event::Left(..) | Event::Right(..) | Event::Key(Key::Escape) if self.step == Step::Code => {}
                Event::Key(Key::Char(c)) if self.step == Step::Hotkey => {
                    if c.is_ascii_alphabetic() {
                        self.cur.hotkey = Some(c.to_ascii_lowercase());
                        self.confirm();
                    }
                }
                Event::Key(Key::Char(c)) if self.step == Step::Name && self.cur.name.len() < NAME_MAX && mad_assets::font::glyph_for_cp437(c).is_some() => {
                    self.cur.name.push(c)
                }
                Event::Key(Key::Backspace) if self.step == Step::Name => _ = self.cur.name.pop(),
                Event::Key(Key::Enter) if self.step == Step::Name && !self.cur.name.is_empty() => self.step = Step::Sex,
                Event::Right(..) | Event::Key(Key::Escape) if self.joining.is_some() => {
                    return Next::Go(Box::new(Game::start(ctx.assets, self.joining.take().unwrap().0)));
                }
                Event::Right(..) | Event::Key(Key::Escape) => return Next::Go(menu::player_count(ctx.assets, self.difficulty)),
                Event::Left(x, y) => {
                    let (laid, page) = self.layout(ctx.assets, None);
                    if let Some(row_y) = page.row_y
                        && self.step == Step::Colour
                        && let Some(&(k, _)) = self.buttons(row_y).iter().find(|(_, b)| b.contains(x, y))
                    {
                        self.cur.colour = k;
                        self.step = Step::Hotkey;
                        continue;
                    }
                    let Some(k) = laid.items.iter().position(|it| it.contains(x, y)) else { continue };
                    match (page.roles[k], self.step) {
                        (Role::Male | Role::Female, Step::Sex) => {
                            self.cur.female = page.roles[k] == Role::Female;
                            self.step = Step::Faith;
                        }
                        (Role::Prot | Role::Cath, Step::Faith) => {
                            self.cur.catholic = page.roles[k] == Role::Cath;
                            self.step = Step::Colour;
                        }
                        (Role::Letter(c), Step::Hotkey) => {
                            self.cur.hotkey = Some(c);
                            self.confirm();
                        }
                        (Role::Yes, Step::Confirm) if self.difficulty < 3 && self.cur.mission.is_none() => {
                            self.code.clear();
                            self.step = Step::Code;
                        }
                        (Role::Yes, Step::Confirm) => {
                            let p = std::mem::take(&mut self.cur);
                            if self.world.setup_human(&setup_of(&p, self.city.unwrap_or(0))).is_none() {
                                return Next::Go(Box::new(Placeholder::new("Die Simulation hat das neue Spiel abgelehnt.")));
                            }
                            self.done.push(p);
                            self.step = Step::Name;
                            if self.done.len() == self.count && self.joining.is_some() {
                                return Next::Go(Box::new(Game::start(ctx.assets, self.world.clone())));
                            }
                            if self.done.len() == self.count {
                                self.world.finish_game_setup(self.count);
                                return Next::Go(Box::new(Game::start(ctx.assets, self.world.clone())));
                            }
                        }
                        (Role::No, Step::Confirm) => {
                            // "Nein": start this player over
                            self.cur = Player::default();
                            self.step = Step::Name;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Next::Stay
    }

    fn draw(&self, frame: &mut Frame, assets: &Assets, mouse: (i32, i32)) -> Option<Cursor> {
        backdrop(frame, assets, b"", &[]);
        let hover = self.layout(assets, None).0.items.iter().position(|it| it.contains(mouse.0, mouse.1));
        let (laid, page) = self.layout(assets, hover);
        text::draw(frame, &assets.font, &laid, 0, 0);
        if let Some(row_y) = page.row_y {
            let flags = assets.icon8(FLAGS);
            for (k, b) in self.buttons(row_y) {
                let f = assets.remap(&flags[0], 5 * k);
                frame.blit(&f, b.x + (BUTTON - f.width as i32) / 2, b.y);
            }
        }
        if self.step == Step::Code {
            let name: Vec<u8> = self.cur.name.iter().flat_map(|&b| if b == b'$' || b == b'%' { vec![b, b] } else { vec![b] }).collect();
            let colour = assets.name_table(addr::CARD_COLOURS).get(self.difficulty as usize).cloned().unwrap_or_default();
            let mut entry = self.code.clone();
            entry.resize(4, b'-');
            let src = replace(&replace(&assets.text(addr::CODE_PROMPT), b"%1N0", &name), b"%s", &esc(&colour));
            Window::new(assets, CODE_BOX, &src, &[&entry], &[], &[]).draw(frame, assets);
            if let Some(n) = &self.notice {
                Window::new(assets, NOTICE_BOX, n, &[], &[], &[]).draw(frame, assets);
            }
        }
        if self.step == Step::Name {
            // text cursor: glyph 0x24 right after the name (rules-42 §3)
            let end = laid.items.first().map_or(LEFT_X, |it| it.x + it.w);
            text::draw_caret(frame, &assets.font, end, laid.items.first().map_or(108, |it| it.y));
        }
        Some(Cursor::Arrow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_helpers() {
        assert_eq!(replace(b"%1N0, Ihr", b"%1N0", b"Anna"), b"Anna, Ihr");
        assert_eq!(esc("a$b%"), b"a$$b%%");
        assert!(Step::Colour < Step::Hotkey);
    }

    /// A player joining a running game sees "Ein neuer Spieler" (live capture), not "Spieler Nr.".
    #[test]
    fn join_heading() {
        let Some(assets) = crate::screens::game::tests::test_assets() else { return };
        let st = mad_sim::State::new_game(4, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap();
        let p = PlayerSetup::join(st).page(&assets);
        assert!(p.src.windows(17).any(|w| w == b"Ein neuer Spieler"));
        if let Ok(dir) = std::env::var("MAD_SHOT_DIR") {
            let mut frame = crate::gfx::Frame::new(assets.palette);
            let mut j = PlayerSetup::join(mad_sim::State::new_game(4, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
            j.cur.name = b"anna".to_vec();
            j.draw(&mut frame, &assets, (630, 470));
            crate::gfx::write_screenshot(&format!("{dir}/join.png"), &frame).unwrap();
        }
    }
}
