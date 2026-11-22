//! Full-screen Kontor sub-screens with labelled hotspots (`docs/re/formats.md` §10): Handel map,
//! Schreibstube, Löhne, Hinterzimmer, Kirche, Söldner & Räuber map, Spieloptionen, Kreditbuch.
//! Each hotspot pairs a gold label (shown on hover, spec 01) with a click rectangle.

use super::{Action, Game, Menu, Plot, hit, t};
use crate::{assets::Assets, gfx::Frame, screens::draw_window, text};
use mad_sim::{
    Command,
    state::{CITIES, KIND_CAMP, KIND_CASTLE, PROPS},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Room {
    Handel,
    Schreibstube,
    Bezahlung,
    Hinterzimmer,
    Kirche,
    Kirchgang,
    Truppen,
    /// Castle or camp screen of property `p` (rules-26 §3, §4).
    Burg(usize),
    Optionen,
    Kreditbuch,
}

/// Where a hotspot's label goes: centred on `pos` (Kontor-style rooms, as in §9.2) or with its top-left
/// at `pos` (map names, which then sit on the names printed on ICON23/ICON24). [UNVERIFIED] both.
#[derive(Clone, Copy)]
enum Anchor {
    Centre,
    TopLeft,
}

pub(super) struct Hot {
    label: Vec<u8>,
    pos: (i32, i32),
    pub(super) rect: [i32; 4],
    pub(super) act: Action,
}

pub(super) struct RoomDef {
    /// Icons drawn first: (id, x, y).
    bg: Vec<(u16, i32, i32)>,
    /// Framed window (Spieloptionen) drawn before the icons.
    window: Option<[i32; 4]>,
    banner: Option<Vec<u8>>,
    /// Draw ICON22 under the banner text ("drawn without redrawing ICON22" on the maps).
    banner_icon: bool,
    /// Text printed into the screen/window (titles, book pages).
    text: Vec<u8>,
    /// `%m` items in `text`.
    items: Vec<Vec<u8>>,
    /// Owner flags (colour, position), drawn over the background.
    flags: Vec<(u8, (i32, i32))>,
    anchor: Anchor,
    pub(super) hots: Vec<Hot>,
    /// Hover outlines (rules-63 §1): the hovered item's rect lights its icon at the rect's top-left; items listed
    /// with `None` (owner flags) win the hover and light nothing.
    outlines: Vec<([i32; 4], Option<u16>)>,
}

mod addr {
    pub const HANDEL: u32 = 0xBDB38;
    pub const SCHREIBSTUBE: u32 = 0xBFD6C;
    pub const LOEHNE: u32 = 0xBFCB0;
    pub const HINTERZIMMER: u32 = 0xBFD28;
    pub const KIRCHE: u32 = 0xBFFF0;
    pub const TRUPPEN: u32 = 0xC01FC;
    pub const KREDITBUCH: u32 = 0xBFDA8;
    pub const NO_LOANS: u32 = 0xBFF08;
    pub const OPTIONS_TITLE: u32 = 0xBDB48;
    pub const NO_CITIZEN: u32 = 0xBFDC0;
    pub const MARRIED_M: u32 = 0xC0034;
    pub const MARRIED_F: u32 = 0xC0074;
    pub const NEW_PLAYER: u32 = 0xBDBBC;
}

/// §10.1 city points P1 (name label), P2 (50×40 click item), in city order.
pub(super) const CITY_POINTS: [((i32, i32), (i32, i32)); CITIES] = [
    ((323, 180), (329, 201)),
    ((81, 197), (58, 207)),
    ((119, 163), (144, 145)),
    ((115, 281), (106, 296)),
    ((352, 122), (410, 91)),
    ((497, 91), (549, 113)),
    ((503, 199), (553, 220)),
    ((471, 279), (500, 309)),
    ((302, 334), (346, 359)),
    ((410, 239), (449, 224)),
];

/// §10.7 castle/camp points P1, P2 (records in location order 14..20 assumed: the record ids
/// `2014+k` suggest it). Which point is clickable is not stated; we use a 50×40 box centred on P1
/// and the name at P2 ([UNVERIFIED]).
pub(super) const CAMP_POINTS: [((i32, i32), (i32, i32)); 7] =
    [((396, 172), (494, 155)), ((347, 260), (311, 266)), ((301, 148), (279, 147)), ((223, 109), (251, 125)), ((144, 234), (103, 250)), ((376, 288), (403, 306)), ((447, 261), (516, 246))];

/// (label string, label pos, rect) per hotspot, from the §10 tables.
type Spots = &'static [(u32, (i32, i32), [i32; 4])];
const SCHREIBSTUBE: Spots = &[
    (0xBFD7C, (277, 324), [200, 357, 150, 120]),
    (0xBFD88, (95, 290), [1, 300, 170, 140]),
    (0xBFD94, (580, 230), [500, 250, 138, 130]),
    (0xBFD9C, (438, 332), [360, 350, 200, 128]),
    (0xBFDA8, (90, 150), [0, 80, 200, 200]),
    (0xBFDB4, (550, 150), [440, 80, 200, 200]),
];
const HINTERZIMMER: Spots = &[
    (0xBFD38, (425, 188), [401, 225, 46, 128]),
    (0xBFD40, (52, 181), [21, 216, 60, 35]),
    (0xBFD4C, (147, 402), [1, 416, 368, 63]),
    (0xBFD58, (207, 289), [153, 317, 92, 68]),
    (0xBFD64, (530, 218), [483, 187, 90, 215]),
];
/// Kirchgang room (captured; the first three rects and the booth are measured from hover probes).
const KIRCHGANG_BG: u16 = 0xBC;
const KIRCHGANG: Spots = &[
    (0xC010C, (95, 174), [70, 200, 60, 120]),
    (0xC00FC, (504, 273), [490, 300, 40, 60]),
    (0xC0114, (262, 276), [245, 300, 45, 90]),
    (0xC00E8, (363, 251), [280, 205, 205, 225]),
];
const KIRCHE: Spots = &[
    (0xBFFF8, (318, 108), [250, 130, 145, 111]),
    (0xC000C, (540, 380), [460, 325, 180, 155]),
    (0xC001C, (100, 380), [0, 325, 180, 155]),
    (0xC0028, (318, 331), [242, 315, 152, 141]),
];
/// §10.8, window-relative to (40,20): checked against ICON2D, the Laden/Speichern signs and the town
/// gate (spec 03 "Spiel beenden: town gate") line up only with the window offset added.
const OPTIONEN: Spots = &[
    (0xBDB60, (443, 149), [420, 161, 42, 42]),
    (0xBDB68, (97, 152), [64, 157, 56, 59]),
    (0xBDB74, (169, 240), [140, 174, 56, 61]),
    (0xBDB84, (358, 378), [316, 264, 98, 98]),
    (0xBDB94, (180, 371), [158, 250, 74, 98]),
    (0xBDB9C, (348, 182), [326, 180, 64, 69]),
    (0xBDBA4, (255, 159), [212, 179, 88, 72]),
];
const OPTIONS_WINDOW: [i32; 4] = [40, 20, 560, 440];

fn spots(assets: &Assets, s: Spots, off: (i32, i32), acts: Vec<Action>) -> Vec<Hot> {
    s.iter()
        .zip(acts)
        .map(|(&(l, (x, y), [rx, ry, w, h]), act)| Hot { label: assets.text(l), pos: (x + off.0, y + off.1), rect: [rx + off.0, ry + off.1, w, h], act })
        .collect()
}

/// The Kreditbuch's "Kredit tilgen" icon (`%j`, rules-46 §1).
const LOAN_ICON: u16 = 0x6A;

/// Top left of the Alles-verkaufen button on the Handel map (ours).
pub(super) const SELL_ALL_AT: (i32, i32) = (48, 398);

impl Game {
    pub(super) fn room_def(&self, assets: &Assets, r: Room) -> RoomDef {
        let h = self.me().unwrap_or(0);
        let mut d = RoomDef { bg: vec![], window: None, banner: None, banner_icon: true, text: vec![], items: vec![], flags: vec![], anchor: Anchor::Centre, hots: vec![], outlines: vec![] };
        match r {
            Room::Handel => {
                d.bg.push((0x23, 0, 0));
                (d.banner, d.banner_icon, d.anchor) = (Some(assets.text(addr::HANDEL)), false, Anchor::TopLeft);
                for (c, &(p1, p2)) in CITY_POINTS.iter().enumerate() {
                    let act = match self.st.branch_at(h, c) {
                        Some(b) => Action::Branch(b),
                        None => Action::Open(Menu::City(c as u8)),
                    };
                    // rules-30 §1: the town drawing (ICONI0+c, Falkenhorst a fixed 40x50) and the name label
                    // (ICONJ5+c) are the click items, each as large as its icon; no hover text
                    let size = |id: u16| { let i = &assets.icon(id)[0]; (i.width as i32, i.height as i32) };
                    let (w2, h2) = if c == 6 { (40, 50) } else { size(0x120 + c as u16) };
                    let (w1, h1) = size(0x135 + c as u16);
                    // formats §11.1: the drawing leads to an own branch, the label always to the city info
                    d.hots.push(Hot { label: vec![], pos: p2, rect: [p2.0, p2.1, w2, h2], act });
                    d.hots.push(Hot { label: vec![], pos: p1, rect: [p1.0, p1.1, w1, h1], act: Action::Open(Menu::City(c as u8)) });
                }
                // ours (pref `sell_all`): a button on the open sea below the ship (drawn in
                // `Game::draw`), hotkey V
                if assets.prefs.borrow().sell_all {
                    d.hots.push(Hot { label: vec![], pos: SELL_ALL_AT, rect: [SELL_ALL_AT.0, SELL_ALL_AT.1, 105, 22], act: Game::sell_all_ask(assets) });
                }
                // owner flags (rules-30 §1): ICONP6 in the owner's colour at the city's P3; a 40x20 item created before
                // the towns, so it wins the hover where they overlap and lights nothing (rules-63 §1)
                for c in 0..CITIES {
                    if let Some(o) = self.st.rights_holder(c) {
                        let r = &self.st.cities[c].0;
                        let at = (u16::from_le_bytes([r[0xA], r[0xB]]) as i32, u16::from_le_bytes([r[0xC], r[0xD]]) as i32);
                        d.flags.push((self.st.persons[o].colour(), at));
                        d.outlines.push(([at.0, at.1, 40, 20], None));
                    }
                }
                // the hovered town lights its yellow outline ICONI0+c at P2, the ribbon its outline ICONJ5+c at P1
                for (c, &(p1, p2)) in CITY_POINTS.iter().enumerate() {
                    let size = |id: u16| { let i = &assets.icon(id)[0]; (i.width as i32, i.height as i32) };
                    let (w2, h2) = if c == 6 { (40, 50) } else { size(0x120 + c as u16) };
                    let (w1, h1) = size(0x135 + c as u16);
                    d.outlines.push(([p2.0, p2.1, w2, h2], Some(0x120 + c as u16)));
                    d.outlines.push(([p1.0, p1.1, w1, h1], Some(0x135 + c as u16)));
                }
            }
            Room::Truppen => {
                d.bg.push((0x24, 0, 0));
                (d.banner, d.banner_icon, d.anchor) = (Some(assets.text(addr::TRUPPEN)), false, Anchor::TopLeft);
                for (k, &(p1, p2)) in CAMP_POINTS.iter().enumerate() {
                    let loc = 14 + k as u8;
                    let Some(p) = (0..PROPS).find(|&q| self.st.props[q].loc() == loc && matches!(self.st.props[q].kind(), KIND_CASTLE | KIND_CAMP)) else {
                        continue;
                    };
                    let owner = self.st.props[p].owner();
                    let info = if owner as usize == h { Action::Sim(Command::CastleOffers { prop: p as u8 }, Some(Menu::CastleInfo(k as u8))) } else { Action::Open(Menu::CastleInfo(k as u8)) };
                    // rules-26 §1: the name label sits at the first point (127x38), the drawing (icon 0x12E + k, its own
                    // size) at the second; the label always opens the info window, the drawing the owner's troop screen
                    let (dw, dh) = assets.icon(0x12E + k as u16).first().map_or((55, 59), |i| (i.width as i32, i.height as i32));
                    d.hots.push(Hot { label: vec![], pos: p2, rect: [p2.0, p2.1, dw, dh], act: if owner as usize == h { Action::Room(Some(Room::Burg(p))) } else { info.clone() } });
                    d.hots.push(Hot { label: t(self.loc(loc)), pos: p1, rect: [p1.0, p1.1, 127, 38], act: info });
                }
            }
            Room::Burg(p) => {
                // rules-26 §3.1/§4: backdrop, banner with the place name, troop names, counts, five icon buttons,
                // four "Info" hotspots. The icon buttons all open the combined window until the sub-windows exist.
                let pr = &self.st.props[p];
                let castle = pr.kind() == KIND_CASTLE;
                d.bg.push((if castle { 0x101 } else { 0x100 }, 0, 0));
                d.banner = Some(t(self.loc(pr.loc())));
                let (names, perm) = if castle { (assets.name_table((0xB6BE0, 4)), [0usize, 1, 2, 3]) } else { (assets.name_table((0xB6BF0, 4)), [0, 3, 2, 1]) };
                type Layout = ([i32; 4], [i32; 4], [i32; 4], [(u16, i32); 5]);
                let (nx, cx, ix, icons): Layout = if castle {
                    ([60, 184, 320, 464], [87, 230, 353, 499], [103, 246, 369, 515], [(0x87, 32), (0x88, 160), (0x89, 288), (0x8A, 416), (0x5C, 544)])
                } else {
                    ([35, 154, 320, 484], [62, 205, 363, 519], [78, 221, 379, 535], [(0xA5, 48), (0x88, 208), (0xA6, 368), (0x5C, 528), (0, 0)])
                };
                // rules-26 §3.1 step 4: names at text cursor y 76
                let mut src = b"$g$L$76{".to_vec();
                for (i, &k) in perm.iter().enumerate() {
                    src.extend([format!("${}[", nx[i]).into_bytes(), t(&names[k]), b"$E".to_vec()].concat());
                }
                src.extend(b"$N$N");
                let book = self.popups.iter().any(|q| matches!(q, super::Popup::Choice { menu: Some(Menu::Orders(o)), .. } if *o == p));
                // counts: 3-digit items at y 290 (book shown) or 370 (rules-26 §3.2), digits at the item top (A/B)
                src.extend(if book { b"$290{".as_slice() } else { b"$370{".as_slice() });
                for (i, &k) in perm.iter().enumerate() {
                    src.extend(format!("${}[{:03}$E", cx[i], pr.troops(k)).bytes());
                }
                d.text = src;
                let words = assets.name_table((0xB6814, 10));
                for (i, &k) in perm.iter().enumerate() {
                    let msg = format!("$N$CDie Moral Eurer {} ist {}.$N", names[k], words[(pr.training(k) as usize / 11).min(9)]);
                    d.hots.push(Hot { label: t("Info"), pos: (ix[i], 180), rect: [[43, 186, 309, 455][i], 110, 120, 170], act: Action::Boxed(msg.chars().filter_map(crate::input::cp437).collect(), if castle { [140, 180, 360, 120] } else { [160, 180, 320, 120] }) });
                }
                // lower half of a troop count dismisses 1, 10 or 100 men by the digit clicked (rules-26 §3.2)
                let y0 = if book { 290 } else { 370 };
                // digit pitch of the gold numbers, from the font (was a guessed 16 px)
                let g = text::layout(&assets.font, b"$g$L00", &[], &[], 640, &[]).glyphs;
                let dw = g.get(1).map_or(16, |b| b.0 - g[0].0);
                for (i, &k) in perm.iter().enumerate() {
                    for (n, dv) in [100u8, 10, 1].into_iter().enumerate() {
                        d.hots.push(Hot { label: vec![], pos: (0, 0), rect: [cx[i] + dw * n as i32, y0 + 9, dw, 9], act: Action::Dismiss(p, k as u8, dv) });
                    }
                }
                // castle: expansion, recruiting, manoeuvre, toll, order book; camp: no toll
                let acts: Vec<Menu> = if castle { vec![Menu::Expand(p), Menu::Recruit(p), Menu::Manoeuvre(p), Menu::Toll(p), Menu::Orders(p)] } else { vec![Menu::Expand(p), Menu::Recruit(p), Menu::Manoeuvre(p), Menu::Orders(p)] };
                for (&(id, x), m) in icons.iter().filter(|i| i.0 != 0).zip(acts) {
                    d.bg.push((id, x, 408));
                    d.hots.push(Hot { label: vec![], pos: (0, 0), rect: [x, 408, 64, 64], act: Action::Open(m) });
                }
            }
            Room::Schreibstube => {
                d.bg.push((0xC8, 0, 28));
                d.banner = Some(assets.text(addr::SCHREIBSTUBE));
                // Bewerbung needs citizenship; title 0 means not yet a citizen (rules-03 §3.2 [UNVERIFIED] link).
                // rules-64 §2: without citizenship a box (160, 160, 320, 160); opening draws the rival table once (ApplyView)
                let apply = if self.st.persons[h].title() == 0 { Action::Boxed(assets.text(addr::NO_CITIZEN), [160, 160, 320, 160]) } else { Action::Sim(Command::ApplyView, Some(Menu::Apply)) };
                let acts = vec![apply, Action::Sim(Command::AskLender, None), Action::Open(Menu::Laws(0)), Action::Open(Menu::Privileges), Action::Room(Some(Room::Kreditbuch)), Action::Open(Menu::Directors)];
                d.hots = spots(assets, SCHREIBSTUBE, (0, 0), acts);
            }
            Room::Bezahlung => {
                d.bg.push((0xA9, 0, 0));
                d.banner = Some(assets.text(addr::LOEHNE));
            }
            Room::Hinterzimmer => {
                d.bg.push((0xAC, 0, 0));
                d.banner = Some(assets.text(addr::HINTERZIMMER));
                let acts = [Plot::Spies, Plot::Saboteur, Plot::Blackmail, Plot::Denounce].map(|p| Action::Open(Menu::Targets(p))).into_iter().chain([Action::Room(None)]).collect();
                d.hots = spots(assets, HINTERZIMMER, (0, 0), acts);
            }
            Room::Kirchgang => {
                d.bg.push((KIRCHGANG_BG, 0, 60));
                d.banner = Some(assets.text(0xC0028));
                let acts = vec![Action::Open(Menu::Pray), Action::Open(Menu::Candles), Action::Open(Menu::Alms), Action::Open(Menu::Indulgence)];
                d.hots = spots(assets, KIRCHGANG, (0, 0), acts);
            }
            Room::Kirche => {
                let p = &self.st.persons[h];
                d.bg.push((if p.religion() == 0 { 0xC2 } else { 0xC3 }, 0, 52));
                d.banner = Some(assets.text(addr::KIRCHE));
                let married = p.spouse() != mad_sim::state::NONE && p.spouse() != 0xFF;
                let wedding = if married { Action::Boxed(assets.text(if p.gender() == 0 { addr::MARRIED_M } else { addr::MARRIED_F }), [120, 180, 400, 120]) } else { Action::Sim(Command::Matchmaker, None) };
                let acts = vec![wedding, Action::Open(Menu::Convert), Action::Open(Menu::Leave), Action::Room(Some(Room::Kirchgang))];
                d.hots = spots(assets, KIRCHE, (0, 0), acts);
            }
            Room::Optionen => {
                d.window = Some(OPTIONS_WINDOW);
                d.bg.push((0x2D, 80, 70));
                d.text = assets.text(addr::OPTIONS_TITLE);
                let new_player = Action::Confirm(addr::NEW_PLAYER, Box::new(Action::NewPlayer));
                let acts = vec![Action::Open(Menu::Load), Action::Open(Menu::Save), Action::Open(Menu::Quit), new_player, Action::Open(Menu::Music), Action::Open(Menu::Graphics), Action::Open(Menu::Options)];
                d.hots = spots(assets, OPTIONEN, (OPTIONS_WINDOW[0], OPTIONS_WINDOW[1]), acts);
            }
            Room::Kreditbuch => {
                d.bg.push((0xD4, 0, 40));
                d.banner = Some(assets.text(addr::KREDITBUCH));
                // Left page: one entry per loan with its "Kredit tilgen" item (formats §10.2). Right page: no debts.
                let loans = self.st.heap.all(56, h as u8, mad_sim::heap::ANY);
                // the page starts half a line lower, with or without loans (live A/B: no-debts line and a loan line)
                let mut text = b"$t$|$4H$10>$56<$Z".to_vec();
                for &i in &loans {
                    let r = &self.st.heap.recs[i];
                    // formats §10.2: EXE 0xBFEA0, or 0xBFE44 with the year highlighted when at most 2 years are left
                    // (rules-11); `%qe` = sein/ihr + e of the lender; `%j` is the clickable icon 0x6A, top-left at (pen x,
                    // line y - 6), the pen moves on by its width; "Kredit tilgen" is plain text (rules-46 §1)
                    let src = assets.text(if r[3] <= 2 { 0xBFE44 } else { 0xBFEA0 });
                    let q: &[u8] = if self.st.persons[r[4] as usize].gender() & 1 == 1 { b"ihre" } else { b"seine" };
                    let s = super::subst(&super::subst(&src, b"%1N3", &t(&self.pref(r[4], 1, 3))), b"%qe", q);
                    let s = super::subst_seq(&s, b"%d", &[t(&super::dotted(mad_sim::heap::r32(r, 5))), t(&r[9].to_string())]);
                    let s = super::subst(&s, b"%l", &t(&(self.st.year as u32 + r[3] as u32).to_string()));
                    let cut = s.windows(2).position(|w| w == b"%j").unwrap_or(s.len());
                    text.extend_from_slice(&s[..cut]);
                    text.extend_from_slice(b"%m");
                    text.extend_from_slice(s.get(cut + 2..).unwrap_or(&[]));
                    d.items.push(vec![text::PAD_MARK, assets.icon(LOAN_ICON)[0].width as u8]);
                }
                // live A/B capture: the no-debts line stands on the left page (formats §10.2 said right)
                if loans.is_empty() {
                    text.extend(assets.text(addr::NO_LOANS));
                }
                let items: Vec<&[u8]> = d.items.iter().map(Vec::as_slice).collect();
                let laid = text::layout(&assets.font, &text, &items, &[], 640, &[]);
                for (k, &i) in loans.iter().enumerate() {
                    let it = laid.items[k];
                    let img = &assets.icon(LOAN_ICON)[0];
                    d.bg.push((LOAN_ICON, it.x, it.y - 6));
                    d.hots.push(Hot { label: vec![], pos: (0, 0), rect: [it.x, it.y - 6, img.width as i32, img.height as i32], act: Action::Sim(Command::RepayLoan { loan: i as u16 }, None) });
                }
                d.text = text;
            }
        }
        d
    }

    pub(super) fn draw_room(&self, frame: &mut Frame, assets: &Assets, r: Room, mouse: (i32, i32), hover: bool) {
        let d = self.room_def(assets, r);
        if let Some(w) = d.window {
            draw_window(frame, assets, w);
        }
        for &(id, x, y) in &d.bg {
            crate::screens::blit_wrapped(frame, &assets.icon(id)[0], x, y);
        }
        if !d.flags.is_empty() {
            let flags = assets.icon8(0x196);
            for &(k, (x, y)) in &d.flags {
                // the flags wave: frames 0..5 in turn, 110 ms each (live capture 2026-11-13; rules-38 §10 says frame 0)
                let f = (self.clock_ms / 110) as usize % flags.len().max(1);
                frame.blit(&assets.remap(&flags[f], 5 * k), x, y);
            }
        }
        if let Some(title) = &d.banner {
            self.draw_banner(frame, assets, title, d.banner_icon);
        }
        let (ox, oy, w) = d.window.map_or((0, 0, 640), |w| (w[0], w[1], w[2]));
        let items: Vec<&[u8]> = d.items.iter().map(Vec::as_slice).collect();
        text::draw(frame, &assets.font, &text::layout(&assets.font, &d.text, &items, &[], w, &[]), ox, oy);
        if r == Room::Bezahlung {
            self.draw_wages(frame, assets);
        }
        // a dialog opened from a hotspot leaves its label on screen (live captures), like the Kontor's
        let at = if hover { Some(mouse) } else if self.popups.is_empty() || self.branch.is_some() { None } else { self.spot_click };
        if let Some(&(r, Some(id))) = at.and_then(|m| d.outlines.iter().find(|o| hit(o.0, m))) {
            frame.blit(&assets.icon(id)[0], r[0], r[1]);
        }
        if let Some(hs) = at.and_then(|m| d.hots.iter().find(|hs| hit(hs.rect, m))) {
            let (lx, ly) = hs.pos;
            let (align, x, w) = match d.anchor {
                Anchor::Centre => (b"$g$C", lx - lx.min(640 - lx), 2 * lx.min(640 - lx)),
                Anchor::TopLeft => (b"$g$L", lx, 640 - lx),
            };
            let src = [align.as_slice(), &hs.label].concat();
            let laid = text::layout(&assets.font, &src, &[], &[], w + 24, &[]);
            // the EXE centres at centre − width/2 rounded down; our `$C` rounds the other way for odd widths (A/B)
            let tw: i32 = laid.glyphs.iter().map(|g| assets.font.advance[g.2 as usize] as i32).sum();
            let odd = (matches!(d.anchor, Anchor::Centre) && tw % 2 == 1) as i32;
            text::draw(frame, &assets.font, &laid, x - 12 + odd, ly - 12);
        }
    }

    /// Column x centres, stack bottom and height of 100 % (measured on the original, docs/FIDELITY.md).
    const WAGE_X: [i32; 4] = [170, 270, 370, 470];
    /// Heading centres (the headings are not centred over the stacks in the original).
    const HEAD_X: [i32; 4] = [150, 270, 370, 470];
    const STACK_BOTTOM: i32 = 398;

    /// Löhne (§10.3): four gold headings, a stack of coins per wage level and the percentage.
    fn draw_wages(&self, frame: &mut Frame, assets: &Assets) {
        let Some(h) = self.me() else { return };
        let p = &self.st.persons[h];
        let pays = [p.pay_directors(), p.pay_workers(), p.pay_mercs(), p.pay_robbers()];
        let heads = ["Direktoren", "Arbeiter", "Söldner", "Räuber"];
        let coin = &assets.icon(0xAA)[0];
        for (k, &cx) in Self::WAGE_X.iter().enumerate() {
            let head = text::layout(&assets.font, &[b"$g$C".as_slice(), &t(heads[k])].concat(), &[], &[], 200 + 24, &[]);
            text::draw(frame, &assets.font, &head, Self::HEAD_X[k] - 100 - 12, 78);
            // rules-54: clamp((level − 50)·50/100, 1, 50) coins at a 5 px pitch, the top coin drawn last
            let n = ((pays[k] as i32 - 50) * 50 / 100).clamp(1, 50);
            for i in 0..n {
                frame.blit(coin, cx - coin.width as i32 / 2, Self::STACK_BOTTOM - 12 - 5 * i);
            }
            text::draw(frame, &assets.font, &Self::wage_pct(assets, pays[k]), cx - 50 - 12, 396);
        }
    }

    /// The percentage: a 3-digit `%m` item reserving its width (formats §8.6), then "%".
    pub(super) fn wage_pct(assets: &Assets, level: u8) -> text::Laid {
        let adv = |c: u8| assets.font.advance[c as usize] as i32;
        let w = 2 * adv(b'.') / 3 + 3 * adv(b'0');
        // three digits with leading zeros (live: "094 %")
        let num = [vec![text::PAD_MARK, w as u8], t(&format!("{level:03}"))].concat();
        text::layout(&assets.font, b"$g$C%m%%", &[&num], &[], 100 + 24, &[])
    }

    /// rules-54: the coin stacks are pictures; the percentage item is a 3-digit spinner (50..150). The digit under
    /// the mouse picks the step 100/10/1, the upper half adds, the lower subtracts; an odd result moves one more.
    pub(super) fn wage_click(&mut self, assets: &Assets, (x, y): (i32, i32)) {
        let Some(h) = self.me() else { return };
        let p = &self.st.persons[h];
        let pays = [p.pay_directors(), p.pay_workers(), p.pay_mercs(), p.pay_robbers()];
        for (k, &cx) in Self::WAGE_X.iter().enumerate() {
            let (ox, oy) = (cx - 50 - 12, 396);
            let Some(it) = Self::wage_pct(assets, pays[k]).items.first().map(|i| text::Item { x: i.x + ox, y: i.y + oy, ..*i }) else { continue };
            if !it.contains(x, y) {
                continue;
            }
            // ponytail: digit by thirds of the item rect, the per-digit glyph boxes if a capture disagrees
            let step = [100, 10, 1][((x - it.x) * 3 / it.w).clamp(0, 2) as usize];
            let dir = if y < it.y + it.h / 2 - 2 { 1 } else { -1 };
            let mut v = (pays[k] as i32 + dir * step).clamp(50, 150);
            if v % 2 != 0 {
                v = (v + dir).clamp(50, 150);
            }
            self.sim(assets, Command::SetPay { which: k as u8, pct: v as u8 });
        }
    }

    /// Hotspot action at `pos`, if any.
    pub(super) fn room_action(&self, assets: &Assets, r: Room, pos: (i32, i32)) -> Option<Action> {
        self.room_def(assets, r).hots.into_iter().find(|hs| hit(hs.rect, pos)).map(|hs| hs.act)
    }
}

/// Test probe: the Kreditbuch text has no codes left and one hotspot per loan line.
#[cfg(test)]
pub(super) fn kreditbuch_ok(d: &RoomDef) -> bool {
    !d.text.windows(2).any(|w| w == b"%d" || w == b"%q") && d.hots.len() == d.items.len() && !d.items.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every room builds with its documented hotspot count on a fresh game.
    #[test]
    fn real_rooms() {
        let Some(assets) = super::super::tests::test_assets() else { return };
        let g = Game::start(&assets, mad_sim::State::new_game(5, 3, &[mad_sim::world::HumanSetup::test(3)]).unwrap());
        for (r, n) in [(Room::Handel, 20), (Room::Truppen, 14), (Room::Schreibstube, 6), (Room::Hinterzimmer, 5), (Room::Kirche, 4), (Room::Kirchgang, 4), (Room::Optionen, 7), (Room::Bezahlung, 0), (Room::Kreditbuch, 0)] {
            assert_eq!(g.room_def(&assets, r).hots.len(), n, "{r:?}");
        }
        // own city on the map opens the branch
        assert!(matches!(g.room_action(&assets, Room::Handel, (110, 300)), Some(Action::Branch(_))));
        assert!(matches!(g.room_action(&assets, Room::Handel, (335, 210)), Some(Action::Open(Menu::City(0)))));
    }
}
