//! Popup menus over the Kontor and its §10 rooms (`rooms.rs`). Spielstand and Direktoren follow
//! `formats.md` §10.4 / §10.2; the rest have no RE layout and are our own framed windows with the
//! EXE's strings where located.

use super::{Action, Field, Form, Game, Menu, Opt, OrderField, Plot, dotted, t};
use crate::{assets::Assets, screens::menu::slot_label};
use mad_sim::{
    Command,
    heap::ANY,
    state::{CITIES, G, GOODS, KIND_CASTLE, rd16},
};

mod addr {
    pub const ALLOWED: u32 = 0xA5B6F;
    pub const FORBIDDEN: u32 = 0xA5B78;
    pub const GODLY: u32 = 0xA5B82;
    pub const BLASPHEMOUS: u32 = 0xA5B90;
    /// Strictness words "Sehr liberal" .. "Sehr repressiv" (NUL separated, in this order).
    pub const SAVE_TITLE: u32 = 0xBDC20;
    pub const LOAD_TITLE: u32 = 0xBDC00;
    pub const QUIT: u32 = 0xBDC98;
    pub const NO_OFFICES: u32 = 0xBCFF4;
    pub const CANDLES: u32 = 0xC0130;
    pub const PRAY: u32 = 0xC0184;
    pub const ALMS: u32 = 0xC01A8;
    pub const CONFESSIONLESS: u32 = 0xAB5E4;
    pub const IS_NOW: u32 = 0xC03A4;
    pub const NO_DIRECTORS: u32 = 0xBFF40;
    pub const DIRECTORS_TITLE: u32 = 0xBFF70;
    pub const DIRECTORS_FOOT: u32 = 0xBFFA8;
    pub const JA: u32 = 0xC0268;
    pub const NEIN: u32 = 0xC0260;
}

/// Storage offers per city (`0x85D01`, three u16 per city, value > 10000 = stone; rules-09 §5.8).
const OFFERS: usize = 0x85D01;
/// Privilege names by code (rules-04b §4.1; only codes 1 and 13 are the EXE's exact words, the
/// rest follow the documented table). Empty = dead entry.
const BACK: &str = "Zurück";

const KUPPLERIN_OFFER_RECT: [i32; 3] = [120, 380, 400];

/// Width of a number item with `n` digits (formats §8.6): `((n−1)·adv('.'))/3 + n·adv('0')`, our advances
/// including the 1 px spacing.
pub(super) fn num_slot(assets: &Assets, n: u8) -> u8 {
    let a = |c: u8| assets.font.advance[c as usize] as u32;
    (((n as u32 - 1) * a(b'.')) / 3 + n as u32 * a(b'0')) as u8
}

/// Courtship gift words per gift (rules-56 §1): 0xB57C4 (ids 1371..1377), 0xB57E0 (1378..1384),
/// 0xB57FC (1385..1390).
pub(crate) const GIFT_NAMES: [&[u32]; 3] = [
    &[0x9648A, 0x9649B, 0x964BC, 0x964CD, 0x964E5, 0x964F8, 0x96505],
    &[0x96525, 0x96536, 0x96542, 0x96551, 0x9657E, 0x965A4, 0x965B4],
    &[0x965C3, 0x965D7, 0x965EC, 0x9660E, 0x9661E, 0x96632],
];

/// Office tree geometry (rules-31): entries per level, first index within the level, first global index.
pub(super) fn tree_slots(loc: u8, church: bool) -> (usize, usize, usize) {
    match (loc, church) {
        (0..=9, false) => (7, 0, 0),
        (0..=9, true) => (3, 7, 7),
        (10..=12, false) => (8, 0, 10),
        (10..=12, true) => (5, 8, 18),
        (_, false) => (8, 0, 23),
        (_, true) => (3, 8, 31),
    }
}
/// Column (x 100) and row (x 36 + 48) of each global office in the 600x400 tree window.
pub(super) const COL: [i32; 34] = [0, 2, 4, 0, 2, 4, 2, 1, 3, 2, 0, 2, 4, 0, 2, 4, 2, 2, 0, 2, 3, 1, 2, 0, 2, 4, 0, 2, 4, 2, 2, 1, 3, 2];
pub(super) const ROW: [i32; 34] = [5, 5, 5, 3, 3, 3, 1, 4, 4, 2, 6, 6, 6, 4, 4, 4, 2, 0, 5, 5, 3, 3, 1, 6, 6, 6, 4, 4, 4, 2, 0, 4, 4, 2];
/// Screen rectangle of the tree entry of (loc, idx): the 200 px column, three text rows (rules-31 §5.2).
pub(super) fn tree_entry_rect(loc: u8, idx: u8) -> [i32; 4] {
    let k = mad_sim::career::global_of(loc as usize, idx as usize);
    [20 + COL[k] * 100, 40 + ROW[k] * 36 + 48, 200, 3 * crate::text::LINE_H]
}

impl Game {
    /// Rebuild an open window in place: the new one takes the old one's place in the queue, in front of the
    /// windows waiting behind it (playtest: typing a baby's name sent the cradle behind the year-end windows).
    pub(super) fn reopen(&mut self, assets: &Assets, m: Menu) {
        let at = self.popups.iter().position(|p| matches!(p, super::Popup::Choice { menu: Some(x), .. } if *x == m)).unwrap_or(0);
        self.popups.retain(|p| !matches!(p, super::Popup::Choice { menu: Some(x), .. } if *x == m));
        let behind: Vec<super::Popup> = self.popups.drain(at.min(self.popups.len())..).collect();
        self.open(assets, m);
        self.popups.extend(behind);
    }

    pub(super) fn open(&mut self, assets: &Assets, m: Menu) {
        let Some(h) = self.me() else { return };
        let mut f = Form::default();
        let (mut x, mut w) = (60, 520);
        match m {
            Menu::Spielstand => {
                self.kontor_left = true; // the Spielstand also restarts the Kontor loop (rules-39 §4)
                // rules-28: humans then each human's rival, score = cached net worth / 100, bar to the scale
                self.sim(assets, Command::ViewScores);
                let humans: Vec<usize> = self.st.humans().take(12).collect();
                let rivals: Vec<usize> = humans.iter().filter_map(|&h| self.st.clone().rival_of(h)).collect();
                let rows: Vec<usize> = humans.iter().chain(&rivals).copied().collect();
                let me = self.me();
                let cached = |p: usize| self.st.persons[p].nw_cache().max(0) as i64;
                let live = |p: usize| {
                    let (tax, tithe) = if Some(p) == me { self.scores_tt } else { (0, 0) };
                    (self.st.net_worth(p) as i64).max(0).saturating_sub(tax as i64 + tithe as i64).max(0)
                };
                let m_ = rows.iter().map(|&p| cached(p).max(live(p))).max().unwrap_or(0);
                let scale = [10_000i64, 25_000, 50_000, 100_000, 250_000, 500_000].into_iter().find(|&e| m_ <= e).unwrap_or(1_000_000);
                self.score_bars = rows.iter().map(|&p| (cached(p) * 200 / scale).min(199) as i32).collect();
                f.raw(b"$C^ Spielstand _$Z");
                for (i, &p) in rows.iter().enumerate() {
                    let msg = if i < humans.len() {
                        let s = super::subst(&super::subst(&assets.text(addr::IS_NOW), b"%1N0", &t(&self.pref(p as u8, 1, 0))), b"%1N4", &t(&self.rank(p)));
                        Action::Boxed(s, [160, 180, 320, 120])
                    } else {
                        let h = humans[i - humans.len()] as u8;
                        let f_ = if self.st.persons[p].gender() == 1 { "Gegenspielerin" } else { "Gegenspieler" };
                        let s = super::subst(&assets.text(0xC03C4), b"%1N1", &t(&self.pref(p as u8, 1, 1)));
                        let s = super::subst(&super::subst(&super::subst(&s, b"%1N4", &t(&self.rank(p))), b"%f", &t(f_)), b"%3N0", &t(&self.pref(h, 3, 0)));
                        Action::Boxed(s, [140, 160, 360, 160])
                    };
                    // only the bullet is the item (rules-28 §4): it at margin 12, the name at 36
                    f.raw(b"$L$2>").item(b"   ".to_vec(), msg).raw(b"$E$6>").txt(&self.pref(p as u8, 1, 0)).raw(b"$E$R").txt(&(cached(p) / 100).to_string()).raw(b"$E$|$N");
                }
                let h = (rows.len() as i32 * 24 + 99) / 40;
                self.popups.push_back(f.popup_at([40, 240 - 20 * h, 560, 40 * h]).with_menu(m));
                return;
            }
            Menu::Quit => {
                f.raw(&assets.text(addr::QUIT));
                f.bind(assets.text(addr::JA), Action::Quit).bind(assets.text(addr::NEIN), Action::Close);
                (x, w) = (160, 320); // live capture: box 160..480 × 160..320
            }
            Menu::Save | Menu::Load => {
                let save = m == Menu::Save;
                f.raw(&assets.text(if save { addr::SAVE_TITLE } else { addr::LOAD_TITLE }));
                for k in 0..5u8 {
                    // Saving lists this app's slots only; loading also falls back to the original's.
                    let own = assets.user_saves.join(format!("GAME{k}.DAT")).is_file();
                    let label = if save && !own { None } else { slot_label(assets, k) };
                    let a = match (save, &label) {
                        // live A/B: a click on a slot first asks for its name (typed into the row, Enter saves)
                        (true, _) => Action::SaveName(k),
                        (false, Some(_)) => Action::Load(k),
                        (false, None) => Action::Nothing,
                    };
                    // live A/B capture: rows left-aligned in a 600-wide window at x 20
                    let label = if save && self.save_edit == Some(k) { self.child_name.clone() } else { label.unwrap_or(b"^ Leer _".to_vec()) };
                    // the row is clickable across the window (live A/B: the hover lights it with the pointer far right)
                    let mut label = label;
                    let pad = (90usize).saturating_sub(label.len());
                    if !(save && self.save_edit == Some(k)) {
                        label.extend(std::iter::repeat_n(b' ', pad));
                    }
                    f.raw(b"$L$7>").txt(&format!("{}.", k + 1)).item(label, a).raw(b"$N");
                }
                (x, w) = (20, 600);
            }
            Menu::City(c) => {
                // Layout after the original's window (rules-11 §11.2); icons are drawn by `draw_city_extras`.
                let city = &self.st.cities[c as usize];
                let owner = self.st.humans().find(|&p| self.st.branch_at(p, c as usize).is_some());
                let wage = ["sehr niedrig", "niedrig", "normal", "hoch", "sehr hoch"][(city.wage_level() / 21).min(4) as usize];
                // EXE strings 0xBE380.. in order, every line centred (live A/B capture); the rings, goods and the
                // Lager icon are drawn by `draw_city_extras` into the space the blanks leave
                let blanks = |px: i32| " ".repeat((px / 6).max(0) as usize);
                f.raw(&super::subst(&assets.text(0xBE380), b"%s", &t(self.loc(c))));
                f.raw(&super::subst(&assets.text(0xBE39C), b"%d", &t(&dotted(city.population()))));
                let rings = city.wealth() as i32 / 10 + 1;
                // 0xBE3AC "Reichtum: $g", one ring glyph 0x0E per wealth step, 0xBE3C4 "$t$Z"
                f.raw(&assets.text(0xBE3AC));
                // each ring with its space: the centred row is one space wider (live A/B 2026-11-15)
                for _ in 0..rings {
                    f.raw(b"\x0E ");
                }
                f.raw(&assets.text(0xBE3C4));
                if city.siege() != 0 {
                    f.txt("Belagerung").raw(b"$Z");
                }
                let tax = super::subst_seq(&assets.text(0xBE3EC), b"%d", &[t(&city.tax().to_string())]);
                f.raw(&super::subst(&tax, b"%s", &t(wage)));
                // rules-48 §2: the city's disaster record names the disaster and its year, the label highlighted
                match self.st.heap.find(2, mad_sim::heap::ANY, c) {
                    Some(i) => {
                        let r = &self.st.heap.recs[i];
                        let k = (r[4] as usize + 1).min(4);
                        let what = assets.name_table((0xB7054, 5)).get(k).cloned().unwrap_or_default();
                        let year = mad_sim::heap::r16(r, 6).to_string();
                        f.raw(b"$h").txt("Katastrophe:").raw(b"$h").txt(" ").raw(&super::subst(&t(&what), b"%l", year.as_bytes()));
                    }
                    None => {
                        f.txt("Katastrophe: keine");
                    }
                }
                f.raw(&assets.text(0xBE424));
                f.txt("Hauptproduktion: ").txt(&blanks(126)).raw(b"$Z");
                f.txt("Begehrte Güter:    ").txt(&blanks(126)).raw(b"$N$N$Z");
                // formats §11.2 (4): the Lager line; `%M` items are empty placeholders and left out
                let line = |a: u32| { let s = assets.text(a); s[s.windows(2).rposition(|w| w == b"%M").map_or(0, |i| i + 2)..].to_vec() };
                match self.st.branch_at(h, c as usize) {
                    // rules-23 §7.1: selling asks 0xBE58C (Ja/Nein) over the info window; the only branch gets 0xBE4D4
                    Some(b) if self.st.branches(h) > 1 => {
                        let ask = super::subst(&assets.text(0xBE58C), b"%s", &t(self.loc(c)));
                        let sell = Action::YesNo(ask, [140, 160, 360, 160], Box::new(Action::Sim(Command::SellBranch { branch: b as u8 }, Some(m))));
                        f.icon_item(&format!("Lager für {} Taler schließen", dotted(self.st.property_value(b).max(0) * 2 / 3)), sell)
                    }
                    Some(_) => f.raw(&super::subst(&line(0xBE4D4), b"%o", &t(self.loc(c)))),
                    None if owner.is_some() => f.raw(&line(0xBE480)),
                    None => {
                        // rules-23 §7.1: without the cash a box over the info window (0xBE500), no purchase
                        let price = self.st.branch_price(c as usize) as u32;
                        let short = (self.st.persons[h].cash() as u32) < price;
                        let act = if short { Action::Boxed(assets.text(0xBE500), [140, 160, 360, 160]) } else { Action::Sim(Command::BuyBranch { city: c }, None) };
                        f.icon_item(&format!("Lager für {} Taler eröffnen", dotted(price)), act)
                    }
                };
                self.popups.push_back(f.popup_at([80, 20, 480, 440]).with_menu(m));
                return;
            }
            Menu::PrivCity(code) => {
                // rules-38 §2.3 / 2.6 / 2.7: bullet, the cycling city, (Keine)
                let (text, next) = match code {
                    9 => (2525, Action::Priv(9, self.priv_city as i32, 0)),
                    28 => (2145, Action::Open(Menu::PrivArmy(28))),
                    _ => (2477, Action::Open(Menu::PrivArmy(29))),
                };
                let src = assets.text_id(text);
                let city = t(self.loc(self.priv_city));
                self.form_icons.retain(|i| i.0 != m);
                let mut parts = src.split(|&b| b == b'%');
                f.raw(parts.next().unwrap_or_default());
                let mut first = true;
                for p in parts {
                    match p.first() {
                        Some(b'a') => {
                            self.form_icons.push((m, f.items.len(), 0x73));
                            f.item(b"   ".to_vec(), if first { next.clone() } else { Action::Close });
                            first = false;
                            f.raw(b" ").raw(&p[1..]);
                        }
                        Some(b'm') => {
                            f.item(city.clone(), Action::Step(Field::PrivCity(code), 1));
                            f.raw(&p[1..]);
                        }
                        _ => _ = f.raw(b"%").raw(p),
                    }
                }
                self.popups.push_back(f.popup_at([120, 120, 400, 240]).with_menu(m));
                return;
            }
            Menu::PrivArmy(code) => {
                // rules-38 §2.6 / 2.7: minimum N0 from the city's population, N spinner N0..50000, Handgeld 10·N (and
                // Sold 5·N for a siege) shown, not clickable
                let pop = self.st.cities[self.priv_city as usize % 10].population();
                let n0 = (pop / if code == 28 { 6 } else { 10 }).min(50000);
                self.priv_n = self.priv_n.clamp(n0, 50000);
                let n = self.priv_n;
                let src = super::subst(&assets.text_id(if code == 28 { 2146 } else { 2478 }), b"%d", &t(&dotted(n0)));
                self.form_icons.retain(|i| i.0 != m);
                let mut parts = src.split(|&b| b == b'%');
                f.raw(parts.next().unwrap_or_default());
                let (mut bullets, mut nums) = (0, 0);
                for p in parts {
                    match p.first() {
                        Some(b'a') => {
                            self.form_icons.push((m, f.items.len(), 0x73));
                            f.item(b"   ".to_vec(), if bullets == 0 { Action::Priv(code, self.priv_city as i32, n as i32) } else { Action::Close });
                            bullets += 1;
                            f.raw(b" ").raw(&p[1..]);
                        }
                        Some(b'm') => {
                            match nums {
                                0 => _ = f.digits(n, 5, Field::PrivN(n0, 50000)),
                                1 => _ = f.raw(&t(&dotted(10 * n))),
                                _ => _ = f.raw(&t(&dotted(5 * n))),
                            }
                            nums += 1;
                            f.raw(&p[1..]);
                        }
                        _ => _ = f.raw(b"%").raw(p),
                    }
                }
                self.popups.push_back(f.popup_at([120, 120, 400, 240]).with_menu(m));
                return;
            }
            Menu::Insurance => {
                // rules-38 §2.4: four toggles (bullet or value), Katastrophen 1, Prozesskosten 2, Überfälle 4, Sabotageakte 8
                f.raw(&super::subst(&assets.text_id(2632), b"%2N0", &t(&self.pref(h as u8, 2, 0))));
                self.form_icons.retain(|i| i.0 != m);
                let ins = self.st.persons[h].insurance();
                for (k, bit) in [1u8, 2, 4, 8].into_iter().enumerate() {
                    let line = super::subst(&assets.text_id(2633), b"%s", &assets.text_id(2634 + k as u32));
                    let (head, rest) = line.split_at(line.windows(2).position(|w| w == b"%a").map_or(0, |i| i + 2));
                    f.raw(&head[..head.len().saturating_sub(2)]).raw(b"$L");
                    self.form_icons.push((m, f.items.len(), 0x73));
                    f.item(b"   ".to_vec(), Action::Priv(25, bit as i32, 0));
                    let cut = rest.windows(2).position(|w| w == b"%m").unwrap_or(rest.len());
                    f.raw(b" ").raw(&rest[..cut]);
                    f.item(assets.text_id(2638 + u32::from(ins & bit != 0)), Action::Priv(25, bit as i32, 0));
                    f.raw(rest.get(cut + 2..).unwrap_or(&[]));
                }
                f.raw(&assets.text(0xC05F0));
                self.popups.push_back(f.popup_at([60, 120, 520, 240]).with_menu(m));
                return;
            }
            Menu::PrivAsk(code) => {
                let holder = self.priv_holder_of();
                let loc = self.st.persons[holder].loc();
                let name_id = u32::from_le_bytes(assets.exe_bytes(0x86778 + 4 * code as u32, 4).try_into().unwrap());
                let (rank, cost) = (t(&self.rank(h)), t(&dotted(self.priv_cost)));
                let tid = |n: u32| assets.text_id(n);
                let with = |n: u32| super::subst(&super::subst(&tid(n), b"%1N4", &rank), b"%d", &cost);
                let pick = |c: u8| Action::Open(Menu::Targets(Plot::Priv(c)));
                // rules-38 §2.8..2.17: the confirmation, its rect and what the first answer does
                let (src, rect, acts): (Vec<u8>, [i32; 4], Vec<Action>) = match code {
                    40 | 69 => (with(name_id + 2), [140, 100, 360, 280], vec![Action::Priv(code, 0, 0), Action::Close]),
                    48 | 74 | 79 if self.st.globals.b(0x85E79) != 0 => (assets.text(0xC0640), [140, 160, 360, 160], vec![]),
                    48 | 74 | 79 => (with(name_id + 2), [140, 120, 360, 240], vec![pick(code), Action::Close]),
                    53 => {
                        let k = (loc as u32).saturating_sub(10);
                        let s = super::subst_seq(&with(3055), b"%s", &[t(self.loc(loc)), tid(2000 + 3 * k), tid(2001 + 3 * k), tid(2002 + 3 * k)]);
                        (s, [120, 100, 400, 280], vec![Action::Priv(53, 0, self.priv_cost), Action::Close])
                    }
                    54 | 84 => (super::subst(&with(name_id + 4), b"%s", &t(self.loc(loc))), [120, 100, 400, 280], vec![Action::Priv(code, 0, self.priv_cost), Action::Priv(code, 1, self.priv_cost), Action::Close]),
                    55 | 86 => (with(name_id + 2), [160, 100, 320, 280], vec![Action::Priv(code, 0, self.priv_cost), Action::Close]),
                    80 => (with(2804), [160, 120, 320, 240], vec![pick(80), Action::Close]),
                    81 => (with(3063), [120, 100, 400, 280], vec![pick(81), Action::Close]),
                    _ => (with(3067), [160, 160, 320, 160], vec![pick(85), Action::Close]),
                };
                if acts.is_empty() {
                    f.raw(&src);
                } else {
                    f.inline(&src, acts);
                }
                self.popups.push_back(f.popup_at(rect).with_menu(m));
                return;
            }
            Menu::Project => {
                // rules-38 §2.14: forward X (0..9999) to a person, or not
                let src = assets.text_id(3059);
                self.form_icons.retain(|i| i.0 != m);
                let mut parts = src.split(|&b| b == b'%');
                f.raw(parts.next().unwrap_or_default());
                let mut first = true;
                for p in parts {
                    match p.first() {
                        Some(b'a') => {
                            self.form_icons.push((m, f.items.len(), 0x73));
                            let go = if self.priv_n > 0 { Action::Open(Menu::Targets(Plot::Priv(58))) } else { Action::Close };
                            f.item(b"   ".to_vec(), if first { go } else { Action::Close });
                            first = false;
                            f.raw(b" ").raw(&p[1..]);
                        }
                        Some(b'm') => {
                            f.digits(self.priv_n, 4, Field::PrivN(0, 9999));
                            f.raw(&p[1..]);
                        }
                        _ => _ = f.raw(b"%").raw(p),
                    }
                }
                self.popups.push_back(f.popup_at([120, 100, 400, 280]).with_menu(m));
                return;
            }
            Menu::Privileges => {
                // rules-38 §1: window (100, 40, 440, 400), "^ X Privilegien _", the holder's entries twelve per page, the
                // switch to the next blackmailed person, "Mehr Privilegien"
                let holder = if self.priv_holder == 0xFF { h } else { self.priv_holder as usize };
                let tid = |n: u32| assets.text_id(n);
                let name_id = |c: u8| u32::from_le_bytes(assets.exe_bytes(0x86778 + 4 * c as u32, 4).try_into().unwrap());
                // the title's `$N` leaves a blank line before the entries in the original (A/B): our `$Z$Z` matches it
                f.raw(&super::subst(&super::subst(&assets.text(0xC0584), b"%2N0", &t(&self.pref(holder as u8, 2, 0))), b"$N", b"$Z$Z"));
                self.form_icons.retain(|i| i.0 != m);
                let codes: Vec<u8> = (0..87).filter(|&c| mad_sim::laws::privilege_for(&self.st, holder, h, c) > 0).collect();
                // the next person after the holder who is blackmailed by the player, or the player (§1.4)
                let next = (1..=mad_sim::state::PERSONS).map(|d| (holder + d) % mad_sim::state::PERSONS).find(|&x| {
                    let pr = &self.st.persons[x];
                    pr.used() && (x == h || pr.favour(h) > 100)
                });
                let switch = next.filter(|&x| !(x == h && holder == h));
                let count = codes.len() + usize::from(switch.is_some());
                let (from, to) = if self.priv_page == 0 { (0, 12) } else { (12, usize::MAX) };
                let pr = &self.st.persons[holder];
                let loc = pr.loc();
                let city = (loc < 10).then(|| &self.st.cities[loc as usize]);
                let g = |a: usize| self.st.globals.b(a) as u32;
                for (k, &c) in codes.iter().enumerate() {
                    if k < from || k >= to {
                        continue;
                    }
                    let src = tid(name_id(c));
                    // the line: bullet, then the text with its arguments; a second `%m` is the setting item
                    let text_of = |n: u32| tid(n);
                    let label: Vec<u8> = src.strip_prefix(b"%m ").unwrap_or(&src).iter().copied().take_while(|&b| b != b'$').collect();
                    self.form_icons.push((m, f.items.len(), 0x73));
                    f.raw(b"$L$2>").item(b"   ".to_vec(), self.priv_action(assets, holder, c)).raw(b"$E$6>");
                    let level = |v: u8, base: u32| text_of(base + v as u32 / 21);
                    let setting: Option<(Vec<u8>, bool)> = match c {
                        14..=17 => city.map(|ct| (level(ct.0[0x7F + (c as usize - 14)], if c < 16 { 2725 } else { 2733 }), false)),
                        61 => Some((level(g(0x85A4D) as u8, 2741), false)),
                        64 => Some((text_of(2981 + g(0x85A4C).min(1)), false)),
                        71 => Some((text_of(2871 + g(0x85A4B).min(2)), false)),
                        22 => city.map(|ct| (vec![ct.tax()], true)),
                        31 => Some((vec![g(0x85A45 + (loc as usize).saturating_sub(10)) as u8], true)),
                        32 => Some((vec![g(0x85A48 + (loc as usize).saturating_sub(10)) as u8], true)),
                        51 => Some((vec![g(0x85A42 + (loc as usize).saturating_sub(10)) as u8], true)),
                        _ => None,
                    };
                    let label = match c {
                        4 => super::subst(&label, b"%s", &tid(2402 + (self.st.persons[h].gender() & 1) as u32)),
                        10 => super::subst(&label, b"%d", &t(&dotted(self.st.office_income(h)))),
                        13 => super::subst(&label, b"%s", &tid(2719 + self.st.persons[h].certificate().min(2) as u32)),
                        _ => label,
                    };
                    match setting {
                        Some((v, number)) => {
                            let cut = label.windows(2).position(|w| w == b"%m").unwrap_or(label.len());
                            f.raw(&label[..cut]);
                            if number {
                                f.digits(v[0] as u32, 2, Field::Priv(c));
                            } else {
                                f.item(v, Action::Step(Field::Priv(c), 1));
                            }
                            f.raw(label.get(cut + 2..).unwrap_or(&[]));
                        }
                        None => _ = f.raw(&label),
                    }
                    f.raw(b"$N$|");
                }
                if let Some(x) = switch {
                    let line = if x == h { assets.text(0xC05C0) } else { super::subst(&assets.text(0xC05A4), b"%2N1", &t(&self.pref(x as u8, 2, 1))) };
                    let label: Vec<u8> = line.iter().copied().skip_while(|&b| b != b'%').skip(3).take_while(|&b| b != b'$').collect();
                    if (from..to).contains(&codes.len()) {
                        self.form_icons.push((m, f.items.len(), 0x73));
                        f.raw(b"$N$L$2>").item(b"   ".to_vec(), Action::PrivHolder(if x == h { 0xFF } else { x as u8 })).raw(b"$E$6>").raw(&label).raw(b"$N$|");
                    }
                }
                // "Mehr Privilegien" (0xC05DC) near the bottom when there is a second page, and on it to lead back
                if count > 12 {
                    f.raw(b"$15H$C").item(assets.text(0xC05DC), Action::PrivPage);
                }
                self.popups.push_back(f.popup_at([100, 40, 440, 400]).with_menu(m));
                return;
            }
            Menu::Options => {
                // EXE 0xBEA5E: one row per flag, label left, value right (rules-00 option flags).
                const ROWS: [(&str, u32); 10] = [
                    ("Geschichtsschreibung:", 0x001),
                    ("Abenteuer:", 0x002),
                    ("Das Schicksal:", 0x004),
                    ("Spieltipps:", 0x008),
                    ("Warnungen:", 0x010),
                    ("Runde bestätigen:", 0x020),
                    ("Zeugen & Plädoyers:", 0x040),
                    ("Gericht bestechen:", 0x200),
                    ("Spieler ankündigen:", 0x100),
                    ("Spielende um 1800:", 0x400),
                ];
                // EXE 0xBE260: `%aLabel:$E$R%m$N` per row; the bullet and the value are the items. The value is a
                // style-0x32 slot of fixed width 1 + 4 · (adv['M'] + 1), its text left-aligned in it (rules-47 §a)
                f.raw(b"$Z$C^ Optionen _$Z");
                let slot = 1 + 4 * assets.font.advance[b'M' as usize]; // our advances already include the 1 px spacing
                for (label, bit) in ROWS {
                    let tog = Action::Opt(Opt::Flag(bit), m);
                    let value = [&[crate::text::PAD_MARK, slot][..], &t(if self.opts & bit != 0 { "Ein" } else { "Aus" })].concat();
                    f.bullet(&t(label), tog.clone()).raw(b"$E$R").item(value, tog).raw(b"$N$|");
                }
                // ours: the modern options, and the undo of the last turn end while one is kept (also key `u` in the Kontor)
                f.raw(b"$C").item(t("Extras... (Tab)"), Action::Open(Menu::Extras(0))).raw(b"$N");
                if self.undo.is_some() {
                    let p = assets.prefs.borrow();
                    f.raw(b"$C").item(t(p.tr("Rückgängig: letzte Runde zurück", "Undo the last turn")), Action::Undo).raw(b"$N");
                }
                self.popups.push_back(f.popup_at([100, 60, 440, 360]).with_menu(m));
                return;
            }
            Menu::Extras(page) => {
                // ours: the modern options on two pages, 0 display and comfort, 1 helpers and house rules
                use super::Pref::*;
                let p = assets.prefs.borrow().clone();
                let tr = |de: &'static str, en: &'static str| p.tr(de, en).to_string();
                let on = |v: bool| if v { tr("Ein", "On") } else { tr("Aus", "Off") };
                let rows: Vec<(String, String, super::Pref)> = if page == 0 {
                    vec![
                        (tr("Vollbild:", "Fullscreen:"), on(p.fullscreen), Fullscreen),
                        (tr("Ganzzahlig skalieren:", "Integer scaling:"), on(p.integer_scale), IntegerScale),
                        (tr("Tempo:", "Speed:"), format!("{}x", p.speed), Speed),
                        (tr("Sprache:", "Language:"), tr("Deutsch", "English"), Lang),
                        (tr("Hinweise:", "Tooltips:"), on(p.tooltips), Tooltips),
                        (tr("Tasten (?):", "Shortcuts (?):"), on(p.shortcuts), Shortcuts),
                        (tr("Autosave alle:", "Autosave every:"), if p.autosave_years == 0 { on(false) } else { format!("{} {}", p.autosave_years, tr("J.", "y.")) }, Autosave),
                        (tr("Rückgängig:", "Undo:"), on(p.undo), Undo),
                        (tr("Spielnummer wählen:", "Choose game number:"), on(p.seed_choice), SeedChoice),
                        (tr("Gericht überspringen:", "Skip trials:"), on(p.skip_trial), SkipTrial),
                        (tr("Posten überspringen:", "Skip auction lot:"), on(p.skip_lot), SkipLot),
                    ]
                } else {
                    vec![
                        (tr("Schuldenwarnung:", "Warnings:"), on(p.warnings), Warnings),
                        (tr("Produktionshilfe:", "Production help:"), on(p.production_help), ProductionHelp),
                        (tr("Bilanz (Taste B):", "Ledger (key B):"), on(p.ledger), Ledger),
                        (tr("Preistabelle (Taste P):", "Price table (key P):"), on(p.price_table), PriceTable),
                        (tr("Alles verkaufen:", "Sell all:"), on(p.sell_all), SellAll),
                        (tr("Chronik (Taste L):", "Log (key L):"), on(p.event_log), EventLog),
                        (tr("Aufträge kopieren (C, E):", "Copy orders (C, E):"), on(p.order_copy), OrderCopy),
                        (tr("Sicherheitsfragen:", "Confirmations:"), on(p.confirmations), Confirmations),
                        (tr("Amtsinfo:", "Office info:"), on(p.office_info), OfficeInfo),
                        (tr("Gnade bei Schulden:", "Debt mercy:"), on(p.debt_mercy), DebtMercy),
                        (tr("Bis 255 Arbeiter:", "Up to 255 workers:"), on(p.more_workers), MoreWorkers),
                    ]
                };
                f.raw(b"$Z$C^ Extras _$Z");
                let slot = 1 + 6 * assets.font.advance[b'M' as usize];
                for (label, value, k) in rows {
                    let tog = Action::Opt(super::Opt::Pref(k), m);
                    let value = [&[crate::text::PAD_MARK, slot][..], &t(&value)].concat();
                    f.bullet(&t(&label), tog.clone()).raw(b"$E$R").item(value, tog).raw(b"$N$|");
                }
                if page == 0
                    && let Some(seed) = self.seed
                {
                    f.raw(b"$C").txt(&format!("{} {seed}", tr("Spielnummer", "Game number"))).raw(b"$N");
                }
                // the two windows' openers, each only with its option on
                if page == 1 && (p.ledger || p.price_table || p.event_log) {
                    f.raw(b"$C");
                    if p.ledger {
                        f.item(t(&tr("Bilanz...", "Ledger...")), Action::Open(Menu::Ledger(0))).raw(b"  ");
                    }
                    if p.price_table {
                        f.item(t(&tr("Preise...", "Prices...")), Action::Open(Menu::Prices)).raw(b"  ");
                    }
                    if p.event_log {
                        f.item(t(&tr("Chronik...", "Log...")), Action::Open(Menu::Log(0)));
                    }
                    f.raw(b"$N");
                }
                let other = if page == 0 { tr("Hilfen und Hausregeln...", "Helpers and house rules...") } else { tr("Anzeige und Komfort...", "Display and comfort...") };
                f.raw(b"$Z$C").item(t(&other), Action::Open(Menu::Extras(1 - page)));
                self.popups.push_back(f.popup_at([80, 40, 480, 400]).with_menu(m));
                return;
            }
            Menu::Ledger(page) => {
                // ours (`prefs.ledger`): page 0 the years' net worth and cash with a bar like the Spielstand's
                // (rules-28 §3), page 1 the branches' sales minus wages, means and transport per year
                let pr = assets.prefs.borrow().clone();
                let tr = |de: &'static str, en: &'static str| pr.tr(de, en).to_string();
                let years: Vec<&super::LedgerYear> = self.ledger.iter().filter(|y| y.person as usize == h).rev().take(10).collect();
                f.raw(b"$Z$C^ ").txt(&tr("Bilanz", "Ledger")).raw(b" _$Z");
                if page == 0 {
                    let top = years.iter().map(|y| y.worth.max(0) as i64).max().unwrap_or(0);
                    let scale = [10_000i64, 25_000, 50_000, 100_000, 250_000, 500_000].into_iter().find(|&e| top <= e).unwrap_or(1_000_000);
                    self.score_bars = years.iter().map(|y| (y.worth.max(0) as i64 * 160 / scale).min(159) as i32).collect();
                    f.raw(b"$L$6>").txt(&tr("Jahr", "Year")).raw(b"$E$|$300]$R").txt(&tr("Vermögen", "Net worth")).raw(b"$E$|$190]$R").txt(&tr("Kasse", "Cash")).raw(b"$E$|$N");
                    for y in &years {
                        f.raw(b"$L$2>").item(b"   ".to_vec(), Action::Nothing).raw(b"$E$6>").txt(&y.year.to_string());
                        f.raw(b"$E$|$300]$R").txt(&dotted(y.worth)).raw(b"$E$|$190]$R").txt(&dotted(y.cash)).raw(b"$E$|$N");
                    }
                } else {
                    self.score_bars.clear();
                    // the cities with a result, most recent first, five at most
                    let mut cities: Vec<u8> = vec![];
                    for c in years.iter().flat_map(|y| y.branches.iter().map(|b| b.0)) {
                        if !cities.contains(&c) {
                            cities.push(c);
                        }
                    }
                    cities.truncate(4);
                    f.raw(b"$L$6>").txt(&tr("Jahr", "Year"));
                    for (k, &c) in cities.iter().enumerate() {
                        let name: String = self.loc(c).chars().take(5).collect();
                        f.raw(format!("$E$|${}]$R", 370 - 110 * k).as_bytes()).txt(&format!("{name}."));
                    }
                    f.raw(b"$E$|$N");
                    for y in &years {
                        f.raw(b"$L$6>").txt(&y.year.to_string());
                        for (k, &c) in cities.iter().enumerate() {
                            let v = y.branches.iter().filter(|b| b.0 == c).map(|b| b.1 - b.2).sum::<i32>();
                            f.raw(format!("$E$|${}]$R", 370 - 110 * k).as_bytes()).txt(&if y.branches.iter().any(|b| b.0 == c) { dotted(v) } else { "-".to_string() });
                        }
                        f.raw(b"$E$|$N");
                    }
                }
                if years.is_empty() {
                    f.raw(b"$C").txt(&tr("Noch kein Jahr abgeschlossen.", "No year finished yet.")).raw(b"$N");
                }
                f.raw(b"$Z$C").item(t(&if page == 0 { tr("Niederlassungen", "Branches") } else { tr("Vermögen", "Net worth") }), Action::Open(Menu::Ledger(1 - page)));
                let hh = ((years.len() as i32 + 4) * 24 + 60 + 39) / 40 * 40;
                self.popups.push_back(f.popup_at([40, 240 - hh / 2, 560, hh]).with_menu(m));
                return;
            }
            Menu::ProdHelp(b) => {
                // ours (`prefs.production_help`): the first production order's best worker:means ratio, the means its
                // workers use, the half output above the 30..39 cap, and the expected yield at the local price
                // (rules-02 §3.3 without the dice: `production_estimate`); then every good of the city
                let pr = assets.prefs.borrow().clone();
                let tr = |de: &'static str, en: &'static str| pr.tr(de, en).to_string();
                let c = self.st.props[b].loc() as usize;
                let ratio = |g: usize| {
                    let (r, k) = (mad_sim::economy::MEANS_DIV[g], 10);
                    let d = (1..=k).rev().find(|d| r % d == 0 && k % d == 0).unwrap_or(1);
                    format!("{}:{}", r / d, k / d)
                };
                f.raw(b"$Z$C^ ").txt(&tr("Produktionshilfe", "Production help")).raw(b" _$Z$L");
                if let Some(o) = (0..4).map(|k| self.st.props[b].order(k)).find(|o| o[0] == mad_sim::economy::ORDER_PRODUCE) {
                    let (g, w, m_) = (o[3] as usize % GOODS, o[4] as i32, o[5] as i32);
                    let (opt, price) = (mad_sim::economy::means_needed(g, w), self.st.cities[c].price(g) as i32);
                    let means = self.tables.means_pl.get(g).cloned().unwrap_or_default();
                    let out = self.st.production_estimate(b, g, w, m_);
                    let cost = self.st.order_forecast(h, c, g, w, m_);
                    f.txt(&format!("{}: {} {} {}", self.good(g as u8), tr("bestes Verhältnis Arbeiter:", "best ratio workers:"), means, ratio(g))).raw(b"$N");
                    f.txt(&format!("{w} {} {opt} {means}", tr("Arbeiter nutzen", "workers use"))).raw(b"$N");
                    f.txt(&tr("Über 30 Arbeiter (Grenze 30..39, zufällig) zählt der Rest nur halb.", "Above 30 workers (limit 30..39, random) the rest counts half.")).raw(b"$N");
                    f.txt(&format!("{}: {} x {}: {} Taler, {}: {}, {}: {}", tr("Erwartet", "Expected"), dotted(out), price, dotted(out * price), tr("Kosten", "cost"), dotted(cost), tr("Gewinn", "profit"), dotted(out * price - cost))).raw(b"$N");
                } else {
                    f.txt(&tr("Kein Produktionsauftrag.", "No production order.")).raw(b"$N");
                }
                f.raw(b"$Z$L").txt(&tr("Ware", "Good")).raw(b"$E$|$270]$R").txt(&tr("Verhältnis", "Ratio")).raw(b"$E$|$140]$R").txt(&tr("Mittelpreis", "Means cost")).raw(b"$E$|$20]$R").txt(&tr("Ortspreis", "Local price")).raw(b"$E$|$N");
                for g in self.city_goods(b) {
                    let g = g as usize;
                    f.raw(b"$L$|").txt(self.good(g as u8)).raw(b"$E$|$270]$R").txt(&ratio(g)).raw(b"$E$|$140]$R").txt(&mad_sim::economy::MEANS_COST[g].to_string()).raw(b"$E$|$20]$R").txt(&self.st.cities[c].price(g).to_string()).raw(b"$E$|$N");
                }
                let hh = ((self.city_goods(b).len() as i32 + 7) * 24 + 60 + 39) / 40 * 40;
                self.popups.push_back(f.popup_at([60, 240 - hh.min(440) / 2, 520, hh.min(440)]).with_menu(m));
                return;
            }
            Menu::Log(page) => {
                // ours (`prefs.event_log`): the session's report lines, newest first, paged by printed height
                let pr = assets.prefs.borrow().clone();
                let tr = |de: &'static str, en: &'static str| pr.tr(de, en).to_string();
                let pages = self.log_pages(assets);
                let page = (page as usize).min(pages.len().saturating_sub(1));
                f.raw(b"$Z$C^ ").txt(&tr("Chronik", "Log")).raw(b" _$Z$L");
                match pages.get(page) {
                    None => drop(f.raw(b"$N$C").txt(&tr("Noch nichts geschehen.", "Nothing happened yet.")).raw(b"$N")),
                    Some(r) => {
                        for (year, line) in self.log.iter().rev().skip(r.start).take(r.len()) {
                            f.txt(&format!("{year}: {line}")).raw(b"$N");
                        }
                    }
                }
                f.raw(b"$Z$C");
                if page > 0 {
                    f.item(t(&tr("Neuere", "Newer")), Action::Open(Menu::Log(page as u16 - 1))).raw(b"    ");
                }
                if page + 1 < pages.len() {
                    f.item(t(&tr("Frühere", "Older")), Action::Open(Menu::Log(page as u16 + 1)));
                }
                self.popups.push_back(f.popup_at([40, 30, 560, 420]).with_menu(m));
                return;
            }
            Menu::Prices => {
                // ours (`prefs.price_table`): every good in every city; the best price of a good in gold
                let pr = assets.prefs.borrow().clone();
                f.raw(b"$Z$C^ ").txt(pr.tr("Preise", "Prices")).raw(b" _$Z$L");
                let col = |k: usize| format!("$E$|${}]$R", 470 - 50 * k);
                for c in 0..CITIES {
                    f.raw(col(c).as_bytes()).txt(&self.loc(c as u8).chars().take(3).collect::<String>());
                }
                f.raw(b"$E$|$N");
                for g in 0..GOODS {
                    let best = (0..CITIES).map(|c| self.st.cities[c].price(g)).max().unwrap_or(0);
                    f.raw(b"$L$|").txt(self.good(g as u8));
                    for (c, city) in self.st.cities.iter().enumerate() {
                        let p = city.price(g);
                        f.raw(col(c).as_bytes());
                        if p == best {
                            f.raw(b"$g").txt(&p.to_string()).raw(b"$t");
                        } else {
                            f.txt(&p.to_string());
                        }
                    }
                    f.raw(b"$E$|$N");
                }
                self.popups.push_back(f.popup_at([10, 10, 620, 460]).with_menu(m));
                return;
            }
            Menu::Keys => {
                // ours (`prefs.shortcuts`): the key list, key ?
                let pr = assets.prefs.borrow().clone();
                f.raw(b"$Z$C^ ").txt(pr.tr("Tasten", "Keys")).raw(b" _$Z");
                // two columns: the Kontor places left, the general keys and the Extras' keys right
                let list = super::tips::key_list(&pr);
                let (places, rest) = list.split_at(super::tips::KEYS.len().min(list.len()));
                for k in 0..places.len().max(rest.len()) {
                    if let Some((key, what)) = places.get(k) {
                        f.raw(b"$|$L").txt(key).raw(b"$E$|$36[$L").txt(what).raw(b"$E");
                    }
                    if let Some((key, what)) = rest.get(k) {
                        f.raw(b"$|$300[$L").txt(key).raw(b"$E$|$378[$L").txt(what).raw(b"$E");
                    }
                    f.raw(b"$|$N");
                }
                let hh = ((places.len().max(rest.len()) as i32 + 2) * 24 + 60 + 39) / 40 * 40;
                self.popups.push_back(f.popup_at([10, 240 - hh.min(460) / 2, 620, hh.min(460)]).with_menu(m));
                return;
            }
            Menu::Music => {
                // EXE 0xBE7E9: three volume rows (labels right, "-" bar "+"), speech toggle, sound-card buttons.
                f.raw(b"$Z$C^ Musik _$N$Z");
                for (i, name) in ["Digital:", "Musik:", "Sprache:"].into_iter().enumerate() {
                    // pixel margins from the A/B capture (6 px margin steps land 2-3 px off)
                    f.raw(b"$|$266]$R").txt(name).raw(b"$E$|$158[$L").item(t("-"), Action::Opt(Opt::Vol(i, -1), m)).raw(b"$E$|$314[$L").item(t("+"), Action::Opt(Opt::Vol(i, 1), m)).raw(b"$E$|$N");
                }
                f.raw(b"$C$Z").txt("Sprachausgabe: ").item(t(if self.opts & 0x80 != 0 { "Ein" } else { "Aus" }), Action::Opt(Opt::Voice, m)).raw(b"$E$|$N$C");
                f.item(t("Soundcheck"), Action::Text(0xBA160)).raw(b"$N").item(t("Soundkarten..."), Action::Nothing).raw(b"$N");
                self.popups.push_back(f.popup_at([120, 120, 400, 240]).with_menu(m));
                return;
            }
            Menu::Graphics => {
                // EXE 0xBE8D0: the two colour depths with a radio circle, then film size and scrolling.
                f.raw(b"$Z$C^ Grafik _$Z$4>$4<$L");
                for (k, name) in ["256 Farben", "32.768 Farben"].into_iter().enumerate() {
                    let on = assets.hicolor == (k == 1);
                    f.item(t(name), Action::Nothing).raw(b"$E$R").raw(if on { b"\x0F" } else { b"\x0E" }).raw(b"$N$L");
                }
                f.raw(b"$Z$C").txt("Filmgröße: ").item(t(if self.black_box { "Blackbox" } else { "Lichtspielhaus" }), Action::Opt(Opt::Film, m)).raw(b"$N");
                // the text speed, labels 0xB6E3C (Zu schnell .. Tödlich, rules-42 §1)
                let speed = assets.name_table((0xB6E3C, 6)).get(self.text_speed as usize).cloned().unwrap_or_default();
                f.txt("Scrolling: ").item(t(&speed), Action::Opt(Opt::Scroll, m)).raw(b"$|");
                self.popups.push_back(f.popup_at([140, 140, 360, 200]).with_menu(m));
                return;
            }
            Menu::Storage(b) => {
                // rules-34 §2: window (60, 100, 520, 320), two columns, sell spinners, offers below
                let pr = &self.st.props[b];
                let c = pr.loc() as usize;
                let city = &self.st.cities[c];
                let (area, price) = ([pr.wood() as u32, pr.stone() as u32], [city.level_a() as u32, city.level_b() as u32]);
                f.raw(&super::subst(&assets.text(0xBEE28), b"%o", &t(self.loc(c as u8))));
                self.form_icons.retain(|i| i.0 != m);
                for k in 0..2usize {
                    let name = assets.text(0x87122 + 6 * k as u32);
                    let head = super::subst_seq(&super::subst(&assets.text(0xBEE48), b"%s", &name), b"%w", &[t(&area[k].to_string()), t(&price[k].to_string())]);
                    let cut = head.windows(2).position(|w| w == b"%j").unwrap_or(head.len());
                    f.raw(format!("$|$2H$Z${k}/").as_bytes()).raw(&head[..cut]);
                    self.form_icons.push((m, f.items.len(), 0x6F));
                    let amt = self.amts[k].min(area[k].min(999));
                    // the `%j` icon reserves 32 px (rules-47; live A/B)
                    f.item(vec![crate::text::PAD_MARK, 32], Action::Sim(Command::SellStorage { branch: b as u8, m2: amt as u16, stone: k == 1 }, Some(m)));
                    f.raw(b" ").digits(amt, 3, Field::Amt(k as u8, area[k].min(999))).raw(b" m$18c verkaufen$E");
                }
                let mut row = [0i32; 2];
                for s in 0..3u8 {
                    let v = rd16(&self.st.globals.0, G(OFFERS) + 6 * c + 2 * s as usize) as u32;
                    if v == 0 {
                        continue;
                    }
                    let (k, v) = if v > 10000 { (1usize, v - 10000) } else { (0, v) };
                    let y = 192 + 36 * row[k];
                    row[k] += 1;
                    let pos = if (y / 12) % 2 == 1 { format!("$|${}H", (y / 12 - 1) / 2) } else { format!("$|${}H$Z", (y / 12 - 2) / 2) };
                    f.raw(format!("{pos}${k}/").as_bytes());
                    self.form_icons.push((m, f.items.len(), 0x78 + k as u16));
                    f.item([t(&format!("      {} m", dotted(v))), vec![18], t(" kaufen")].concat(), Action::Sim(Command::BuyStorage { branch: b as u8, k: s }, Some(m))).raw(b"$E");
                }
                if row == [0, 0] {
                    f.raw(b"$|$7H").raw(&assets.text(0xBEEA8));
                }
                self.popups.push_back(f.popup_at([60, 100, 520, 320]).with_menu(m));
                return;
            }
            Menu::Transport(b, tr) => {
                // rules-34 §4: window (100, 20, 440, 440); `tr` 0..2 is the page-1 kind, 3..5 page 2 (Mehr...)
                let pr = &self.st.props[b];
                let c = pr.loc() as usize;
                let (tk, k) = (tr as usize % 3, tr as usize);
                if tk == 1 && !mad_sim::economy::navigable(c) {
                    self.popups.push_back(Form::new(&super::subst(&assets.text(0xBF0B0), b"%o", &t(self.loc(c as u8)))).popup_at([160, 160, 320, 160]));
                    return;
                }
                let o = pr.owner();
                let count = |ty: u8, kk: usize| -> u32 { self.st.heap.all(ty, o, b as u8).into_iter().filter(|&i| self.st.heap.recs[i][4] as usize == kk).map(|i| self.st.heap.recs[i][5] as u32).sum() };
                let (inbuild, inrepair) = (count(10, k), count(11, k));
                let more = pr.trans(tk + 3) != 0 || count(10, tk + 3) != 0 || count(11, tk + 3) != 0;
                let names = |a: u32| assets.name_table((a, 3));
                let title = if more {
                    super::subst_seq(&assets.text(0xBF100), b"%s", &[t(&names(0xB4954)[tk]), t(&names(0xB4960)[tk])])
                } else {
                    super::subst(&assets.text(0xBF114), b"%s", &t(&names(0xB4954)[tk]))
                };
                f.raw(&title);
                let (stock, cond, p) = (pr.trans(k) as u32, pr.cond(k) as u32, self.st.cities[c].tprice(k) as u32);
                // display-only counts are 2-digit number items without padding: a fixed slot, the digits left in it
                // (rules-47 §7, formats §8.6; live A/B: "Im Lager: 1" is centred on the slot)
                let num = |v: u32| [&[crate::text::PAD_MARK, num_slot(assets, 2)][..], &t(&v.to_string())].concat();
                f.raw(&assets.text(0xBF124)).bind(num(stock), Action::Nothing);
                if stock > 0 {
                    f.raw(&super::subst(&assets.text(0xBF13C), b"%d", &t(&cond.to_string())));
                } else {
                    f.raw(&assets.text(0xBF14C));
                }
                // 0xBF15C: Im Bau / In Reparatur, then the sell row (icon, spinner, price shown as cond·P/300)
                let body = assets.text(0xBF15C);
                let cut = body.windows(2).position(|w| w == b"%j").unwrap_or(body.len());
                f.raw(&super::subst_seq(&body[..cut], b"%d", &[t(&dotted(p)), t(&dotted((100 - cond.min(100)) * p / 100))]));
                f.bind(num(inbuild), Action::Nothing).bind(num(inrepair), Action::Nothing);
                let br = b as u8;
                let amt = |s: usize, max: u32| self.amts[s].min(max);
                let row = |f: &mut Form, icon: u16, s: usize, max: u32, label: &str, act: Action, icons: &mut Vec<(Menu, usize, u16)>| {
                    icons.push((m, f.items.len(), icon));
                    // the `%j` icon reserves its 32 px, then a space (rules-47; was five spaces = 30 px)
                    f.item(vec![crate::text::PAD_MARK, 32], act).raw(b" ").digits(amt(s, max), 2, Field::Amt(s as u8, max)).txt(label).raw(b"$Z");
                };
                let mut icons = vec![];
                let sell = Action::Sim(Command::SellTransporters { branch: br, t: k as u8, n: amt(0, stock) as u8 }, Some(m));
                row(&mut f, 0x6F, 0, stock, &format!(" Verkaufen zu {} Talern", dotted(cond * p / 300)), sell, &mut icons);
                if k < 3 {
                    let build = Action::Sim(Command::BuildTransporters { branch: br, t: k as u8, n: amt(1, 99 - inbuild.min(99)) as u8 }, Some(m));
                    row(&mut f, 0x82, 1, 99 - inbuild.min(99), " Bauauftrag", build, &mut icons);
                } else {
                    f.raw(b"$Z");
                }
                let repair = Action::Sim(Command::RepairTransporters { branch: br, t: k as u8, n: amt(2, stock) as u8 }, Some(m));
                row(&mut f, 0x83, 2, stock, " Reparatur", repair, &mut icons);
                let lot = if !self.st.auction_pending() {
                    Action::Boxed(assets.text(0xBF200), [140, 180, 360, 120])
                } else if self.st.lot_full(self.me().unwrap_or(0), b, k, amt(3, stock) as u8) {
                    Action::Boxed(super::subst(&assets.text(0xBF234), b"%8s", &assets.text_id(7 * 107 + 19 + k as u32)), [160, 160, 320, 160])
                } else {
                    Action::Sim(Command::AuctionTransporters { branch: br, t: k as u8, n: amt(3, stock) as u8 }, Some(m))
                };
                row(&mut f, 0x70, 3, stock, " Versteigern", lot, &mut icons);
                if more {
                    f.raw(b"$|$16H$Z$C").item(t("Mehr..."), Action::Open(Menu::Transport(b, ((k + 3) % 6) as u8)));
                }
                self.form_icons.retain(|i| i.0 != m);
                self.form_icons.extend(icons);
                self.trans_pic = (m, [0x7C, 0x7E, 0x80, 0x7D, 0x7F, 0x81][k]);
                self.popups.push_back(f.popup_at([100, 20, 440, 440]).with_menu(m));
                return;
            }
            Menu::Apply => {
                // rules-64 §2: places 13..0, seats from the top, the vacant promotions (eligibility 2), the highest
                // free seat of a group only, at most six; each row a gem (the applicants) and "Ja"/"Nein" (apply)
                let applied: Vec<(u8, u8)> = self.st.heap.all(5, h as u8, ANY).into_iter().filter(|&i| self.st.heap.recs[i][3] & 0x80 == 0).map(|i| (self.st.heap.recs[i][4], self.st.heap.recs[i][5])).collect();
                let mut rows = vec![];
                for loc in (0..=13u8).rev() {
                    let top = match loc { 0..=9 => 9, 10..=12 => 12, _ => 10 };
                    for idx in (0..=top).rev() {
                        if rows.len() == 6 || self.st.eligibility(h, loc as usize, idx as usize) != 2 {
                            continue;
                        }
                        let group = mad_sim::career::seat_group(loc as usize, idx as usize);
                        if (idx as usize + 1..group.end).any(|i| self.st.eligibility(h, loc as usize, i) == 2) {
                            continue;
                        }
                        rows.push((loc, idx));
                    }
                }
                if rows.is_empty() {
                    self.popups.push_back(Form::new(&assets.text(addr::NO_OFFICES)).popup_at([160, 180, 320, 120]));
                    return;
                }
                f.raw(&assets.text(0xBD048));
                let female = self.st.persons[h].gender() & 1 == 1;
                for (loc, idx) in rows {
                    let o = match loc { 0..=9 => idx as u32, 10..=12 => idx as u32 + 10, _ => idx as u32 + 23 };
                    let name = format!("{} in {}", self.assets_text_id(if female { 1407 + o } else { 72 + o }), self.loc(loc));
                    let on = applied.contains(&(loc, idx));
                    f.bullet(&t(&name), Action::Open(Menu::Applicants(loc, idx)));
                    f.raw(b"$E$R").item(assets.text(if on { super::addr::JA } else { super::addr::NEIN }), Action::Sim(Command::Apply { loc, idx, on: !on }, Some(m))).raw(b"$|");
                }
                self.popups.push_back(f.popup_at([80, 80, 480, 320]).with_menu(m));
                return;
            }
            Menu::Applicants(loc, idx) => {
                // rules-64 §2: the other owners' applications for this seat in heap order, four named, then "...und
                // andere"; "Niemand" when there are none; any click closes. `$a` is the EXE's fixed `$L$2>-$E$B$6>`
                // (0xBB594): a dash, not a bullet item (live A/B 2026-11-22), so the window has no items
                f.raw(&assets.text(0xBD0B0));
                let others: Vec<u8> = self.st.heap.all(5, ANY, ANY).into_iter().filter(|&i| {
                    let r = &self.st.heap.recs[i];
                    r[1] as usize != h && r[3] & 0x80 == 0 && (r[4], r[5]) == (loc, idx)
                }).map(|i| self.st.heap.recs[i][1]).collect();
                let line = |src: Vec<u8>| src.strip_prefix(b"$a").map_or(src.clone(), |s| [b"$L$2>-$E$B$6>".as_slice(), s].concat());
                for (k, &a) in others.iter().enumerate() {
                    let src = if k == 4 {
                        assets.text(0xBD13C)
                    } else if let Some(&(hu, _)) = self.apply_rivals.iter().find(|&&(_, r)| r == a) {
                        let addr = if self.st.persons[a as usize].gender() & 1 == 1 { 0xBD100 } else { 0xBD0DC };
                        self.exe_fmt(assets, &assets.text(addr), &[mad_sim::WArg::P(a), mad_sim::WArg::P(hu)])
                    } else {
                        self.exe_fmt(assets, &assets.text(0xBD128), &[mad_sim::WArg::P(a), mad_sim::WArg::P(a)])
                    };
                    f.raw(&line(src));
                    if k == 4 {
                        break;
                    }
                }
                if others.is_empty() {
                    f.raw(&line(assets.text(0xBD150)));
                }
                self.popups.push_back(f.popup_at([60, 140, 520, 200]).with_menu(m));
                return;
            }
            Menu::Laws(book) | Menu::LawEdit(book) => {
                // edit mode (rules-38 §3, privileges 56/59/82): the values are items that change the law
                let edit = matches!(m, Menu::LawEdit(_));
                // rules-04b §1.1: read-only browser with the three book buttons; the value words are
                // items in the original (editable with the minister's privilege, not built).
                let g = |a: usize| self.st.globals.b(a);
                let (heading, lines, strict): (u32, Vec<(u32, Vec<u8>)>, i32) = match book {
                    0 => {
                        let f = |k: usize| g(0x85A21 + k);
                        let w = |k: usize| -> Vec<u8> { assets.text(if f(k) == 0 { addr::ALLOWED } else { addr::FORBIDDEN }) };
                        let num = |v: String| t(&v);
                        let cash = self.st.globals.d(0x85A28);
                        let l = vec![
                            (0x895E7, w(0)),
                            (0x895F9, num(f(1).to_string())),
                            (0x89616, w(2)),
                            (0x8962E, w(3)),
                            (0x8964C, w(4)),
                            (0x8966A, num(f(5).to_string())),
                            (0x8968B, num(f(6).to_string())),
                            (0x896AB, num(dotted(cash))),
                            (0x896C3, num(g(0x85A2C).to_string())),
                            (0x896DF, assets.text(if g(0x85A2D) == 0 { addr::ALLOWED } else { addr::FORBIDDEN })),
                        ];
                        let st = f(0) as i32 + (f(1) > 9) as i32 + f(2) as i32 + f(3) as i32 + f(4) as i32 + (f(5) > 99) as i32 + (f(6) > 99) as i32 + (cash < 10000) as i32 + (g(0x85A2C) < 4) as i32 + g(0x85A2D) as i32;
                        (0x895CF, l, st / 2)
                    }
                    1 => {
                        let w = |k: usize| -> Vec<u8> { assets.text(if g(0x85A2E + k) == 0 { addr::ALLOWED } else { addr::FORBIDDEN }) };
                        let l: Vec<(u32, Vec<u8>)> = [0xA3DE5u32, 0xA3DF9, 0xA3E0C, 0xA3E1F, 0xA3E37, 0xA3E51, 0xA3E71, 0xA3E91, 0xA3EA7, 0xA3EBE].iter().enumerate().map(|(k, &a)| (a, w(k))).collect();
                        (0xA3DCE, l, (0..10).map(|k| g(0x85A2E + k) as i32).sum::<i32>() / 2)
                    }
                    _ => {
                        let w = |k: usize| -> Vec<u8> { assets.text(if g(0x85A38 + k) == 0 { addr::GODLY } else { addr::BLASPHEMOUS }) };
                        let mut l: Vec<(u32, Vec<u8>)> = [0xA3EF4u32, 0xA3F12, 0xA3F2A, 0xA3F44, 0xA3F57, 0xA3F6B, 0xA3F87, 0xA3F97, 0xA3FAC].iter().enumerate().map(|(k, &a)| (a, w(k))).collect();
                        l.push((0xA3FCA, t(&g(0x85A41).to_string())));
                        let st = (0..9).map(|k| g(0x85A38 + k) as i32).sum::<i32>() + (g(0x85A41) > 9) as i32;
                        (0xA3EDB, l, st / 2)
                    }
                };
                let words = [0xA5B9C, 0xA5BA9, 0xA5BB1, 0xA5BB9, 0xA5BC3];
                f.raw(b"$Z$C^ Gesetze in Mittelland _$Z");
                f.raw(&super::subst(&assets.text(heading), b"%s", &assets.text(words[strict.clamp(0, 4) as usize])));
                // each value is a `%m` item reserving the width of its digit count (formats §8.6): the line centres on it
                let adv = |c: u8| assets.font.advance[c as usize] as i32;
                // digit counts of the number items (formats §8.6 law book styles); the tithe line [UNVERIFIED] 2
                let digits = |a: u32| -> i32 { match a { 0x895F9 | 0x896C3 | 0xA3FCA => 2, 0x8966A | 0x8968B => 3, 0x896AB => 7, _ => 0 } };
                for (i, (a, value)) in lines.into_iter().enumerate() {
                    let n = digits(a);
                    let w = if n == 0 { 0 } else { (n - 1) * adv(b'.') / 3 + n * adv(b'0') };
                    let src = assets.text(a);
                    let cut = src.windows(2).position(|x| x == b"%m").unwrap_or(src.len());
                    f.raw(&src[..cut]);
                    if cut < src.len() {
                        match (edit, n) {
                            (false, _) => _ = f.item([vec![crate::text::PAD_MARK, w as u8], value].concat(), Action::Nothing),
                            // numbers zero-padded, stepping by the clicked digit
                            (true, 1..) => _ = f.digits(self.st.globals.law(book as usize, i), n as u32, Field::Law(book, i as u8)),
                            (true, _) => _ = f.item(value, Action::Sim(Command::EditLaw { book, i: i as u8, delta: 0 }, Some(m))),
                        }
                        f.raw(&src[cut + 2..]);
                    }
                }
                if edit {
                    self.popups.push_back(f.popup_at([80, 40, 480, 400]).with_menu(m));
                    return;
                }
                f.raw(b"$Z$C");
                // icon items ICONB2..B4 (live A/B capture), drawn by the popup code
                for (k, name) in ["Finanzen", "Justiz", "Kirche"].iter().enumerate() {
                    f.icon_item(name, Action::Open(Menu::Laws(k as u8))).raw(if k < 2 { b"   ".as_slice() } else { b"" }); // EXE 0xBD17C
                }
                self.popups.push_back(f.popup_at([80, 40, 480, 400]).with_menu(m));
                return;
            }
            Menu::Targets(plot) => {
                // Opening the office tree rolls its prices (rules-04 §2), as the original does.
                self.sim(assets, Command::OfficeTree);
                self.popups.push_back(super::Popup::Map(plot));
                return;
            }
            Menu::Tree(plot, loc, church) => {
                let shame = super::subst(&assets.text_id(3060), b"%1N1", &t(&self.pref(h as u8, 1, 1)));
                let (priv_n, priv_cost) = (self.priv_n as i32, self.priv_cost);
                // rules-04 §5.4: the drop/keep question of an open trial (kind 0) or inquisition (kind 1)
                let drop: Vec<[Option<Vec<u8>>; 2]> = (0..self.st.persons.len()).map(|p| [0u8, 1].map(|k| self.st.open_trial(h, p, k).map(|_| super::subst(&assets.text([0xBCB04, 0xBCCD4][k as usize]), b"%4N1", &t(&self.pref(p as u8, 4, 1)))))).collect();
                let broke = self.st.persons[h].cash() <= 0;
                // Office tree (captured in the original for a city): three columns, office over name.
                let chosen = |p: usize| match plot {
                    Plot::Spies => Action::Spied(p as u8, self.prices.get(p).copied().unwrap_or(0)),
                    Plot::Saboteur => Action::Sabotage(p as u8, self.prices.get(p).copied().unwrap_or(0)),
                    Plot::Blackmail => Action::Blackmail(p as u8),
                    Plot::Denounce => Action::Open(Menu::Tree(Plot::DenounceTo(p as u8), loc, church)),
                    // rules-04 §5.3: L = V repeats the pick without a message
                    Plot::DenounceTo(v) if v as usize == p => Action::Open(Menu::Tree(plot, loc, church)),
                    Plot::DenounceTo(v) => Action::Sim(Command::Denounce { victim: v, listener: p as u8 }, None),
                    // privilege picks (rules-38 §2.9, 2.14..2.17)
                    Plot::Priv(80) => Action::Open(Menu::Tree(Plot::DiscordTo(p as u8), loc, church)),
                    Plot::DiscordTo(v) => Action::Priv(80, v as i32, p as i32),
                    Plot::Priv(58) if p == h => Action::Boxed(shame.clone(), [160, 160, 320, 120]),
                    Plot::Priv(58) => Action::Priv(58, p as i32, priv_n),
                    // rules-04 §5.4: an open inquisition asks 0xBCCD4 first at (120, 140, 400, 200) (rules-56 §6), item 0 drops it
                    Plot::Priv(81) if drop[p][1].is_some() => Action::Inline(drop[p][1].clone().unwrap(), vec![Action::Priv(81, p as i32, 0), Action::Close], [120, 140, 400, 200]),
                    Plot::Priv(81) => Action::Priv(81, p as i32, 0),
                    Plot::Priv(c) => Action::Priv(c, p as i32, priv_cost),
                    // rules-04 §5.1: without cash the sim answers 0xBC5CC instead of the amount window
                    Plot::Bribe if broke => Action::Sim(Command::Bribe { target: p as u8, amount: 0 }, None),
                    Plot::Bribe => Action::Open(Menu::BribeAmount(p as u8)),
                    // §5.4: an open trial asks 0xBCB04 first, item 0 drops it
                    Plot::Trial if drop[p][0].is_some() => Action::Inline(drop[p][0].clone().unwrap(), vec![Action::Sim(Command::FileTrial { target: p as u8 }, None), Action::Close], [160, 140, 320, 200]),
                    Plot::Trial => Action::Sim(Command::FileTrial { target: p as u8 }, None),
                    Plot::Insult => Action::Sim(Command::Insult { target: p as u8 }, None),
                };
                // rules-31: title, then one entry per office at (COL·100, ROW·36 + 48) in the 600x400 window,
                // centred in 200 px: office / heart name religion / bomb favour-bar spy
                // mode 12 (rules-29 §3): "^ Anschwärzen bei... _" without the place
                let title = if matches!(plot, Plot::DenounceTo(_)) { format!("{} bei...", plot.verb()) } else { format!("{} in {}", plot.verb(), self.loc(loc)) };
                f.raw(b"$H$C^ ").txt(&title).raw(b" _$N");
                let (n, obase, nbase) = tree_slots(loc, church);
                // the level toggle redraws the same window in its place (rules-31 §6), it does not stack a second one
                // (playtest: the church level went behind the bribery map)
                let at = self.popups.iter().position(|p| matches!(p, super::Popup::Choice { menu: Some(Menu::Tree(..)), .. }));
                self.popups.retain(|p| !matches!(p, super::Popup::Choice { menu: Some(Menu::Tree(..)), .. }));
                let mut cl = self.st.clone();
                let rival = cl.rival_of(h);
                // ours (office info): every human's rival, whose score the vote tops up with rand(20)
                self.tree_rivals = if assets.prefs.borrow().office_info { self.st.humans().filter_map(|x| cl.rival_of(x)).collect() } else { vec![] };
                self.tree_bars.clear();
                self.tree_rival = None;
                for i in 0..n {
                    let (k, idx) = (nbase + i, (obase + i) as u8);
                    let (col, row) = (COL[k], ROW[k]);
                    let y = row * 36 + 48;
                    // window y as text rows: odd multiples of 12 via $nH, even ones via $nH$Z
                    let pos = if (y / 12) % 2 == 1 { format!("$|${}H", (y / 12 - 1) / 2) } else { format!("$|${}H$Z", (y / 12 - 2) / 2) };
                    let (l, r) = (col * 100, 400 - col * 100);
                    // rules-31 §2.3: each entry prints with glyph spacing 0 (`$0s` … `$1s`) in its 200 px item
                    f.raw(format!("{pos}${l}[${r}]$0s$C").as_bytes());
                    match self.st.holder(loc as usize, idx as usize) {
                        None => {
                            f.txt(&self.office(loc, idx)).raw(b"$N").txt("(unbesetzt)").raw(b"$N");
                        }
                        Some(p) => {
                            let pr = &self.st.persons[p];
                            let heart = if self.st.heap.find(29, p as u8, mad_sim::heap::ANY).is_some() { "/" } else { " " };
                            let cross = match pr.religion() { 0 => "\"", 1 => "#", _ => " " };
                            let bomb = self.st.heap.all(7, h as u8, mad_sim::heap::ANY).into_iter().any(|r| self.st.heap.recs[r][4] as usize == p);
                            let spy = self.st.heap.all(6, h as u8, mad_sim::heap::ANY).into_iter().find(|&r| self.st.heap.recs[r][4] as usize == p);
                            let spy = match spy { None => " ", Some(r) if mad_sim::heap::r32(&self.st.heap.recs[r], 0x0B) == 0 => "@", Some(_) => "`" };
                            let fav = pr.favour(h) as i32;
                            // the whole entry is one menu item (rules-31 §5.2): all three lines carry the action
                            if Some(p) == rival {
                                self.tree_rival = Some(format!("{:?}", chosen(p)));
                            }
                            f.item(t(&self.office_of(p)), chosen(p)).raw(b"$N");
                            let name = t(&format!("{heart} {} {cross}", self.first_name(p)));
                            f.item(name, chosen(p));
                            f.raw(b"$N");
                            let mid = if fav > 100 || pr.human() { if fav > 100 { "=".to_string() } else { " ".repeat(3) } } else { " ".repeat((fav as usize + 5) / 5) };
                            f.item(t(&format!("{} {mid} {spy}", if bomb { "*" } else { " " })), chosen(p));
                            if fav <= 100 && !pr.human() {
                                self.tree_bars.push((20 + col * 100 + 100 - fav / 2, 40 + y + 54, fav));
                            }
                        }
                    }
                }
                f.raw(b"$1s$|$364{$C").item(t(if church { "Zur weltlichen Ebene..." } else { "Zur kirchlichen Ebene..." }), Action::Open(Menu::Tree(plot, loc, !church)));
                let w = f.popup_at([20, 40, 600, 400]).with_menu(m);
                match at {
                    Some(k) => self.popups.insert(k, w),
                    None => self.popups.push_back(w),
                }
                return;
            }
            Menu::Listener(v) => {
                f.raw(b"$C^ Bei wem? _$Z$L");
                for p in self.targets(h).into_iter().filter(|&p| p != v as usize).take(12) {
                    f.item(t(&format!("{}, {}", self.person_name(p as u8), self.rank(p))), Action::Sim(Command::Denounce { victim: v, listener: p as u8 }, None)).raw(b"$N");
                }
                f.raw(b"$C").item(t(BACK), Action::Close);
            }
            Menu::BribeAmount(tg) => {
                // EXE 0xBC618: the bullet bribes with the amount of the 7-digit spinner (live A/B layout)
                let src = super::subst(&assets.text(0xBC618), b"%3N5", &t(&self.pref(tg, 3, 5)));
                let cut = src.windows(2).position(|w| w == b"%a").unwrap_or(src.len());
                f.raw(&src[..cut]);
                self.form_icons.retain(|i| i.0 != m);
                self.form_icons.push((m, f.items.len(), 0x73));
                f.item(b"    ".to_vec(), Action::Sim(Command::Bribe { target: tg, amount: self.bribe as i32 }, None));
                f.digits(self.bribe, 7, Field::Bribe).txt(" Taler"); // `%a%n Taler`: no space after the bullet
                // rules-04 §5.1: win_open(160, 120, 320, 240) (live A/B: the auto height was 40 px short)
                self.popups.push_back(f.popup_at([160, 120, 320, 240]).with_menu(m));
                return;
            }
            Menu::Kupplerin => return,
            Menu::Naming(slot) => {
                // on the cradle (rules-19 §2.1 steps 7, 8): the announcement at the top, then EXE 0xBAA4C
                // ("$16H$C... Welchen Namen soll das Kind tragen?$Z%m$N") on line 16; the name is typed, Enter confirms
                let said = self.cradles.iter().find(|c| c.0 == slot).map(|c| c.1.clone()).unwrap_or_default();
                let head = assets.text(0xBAA4C);
                let mut label = self.child_name.clone();
                label.push(b'$'); // the caret: glyph 0x24, a solid wedge (rules-42 §3; items print `$` literally)
                f.raw(&said);
                f.raw(&head.iter().copied().take_while(|&b| b != b'%').collect::<Vec<_>>());
                f.item(label, Action::Nothing);
                f.raw(b"$N");
                self.popups.push_back(f.popup_at([0, 0, 640, 480]).with_menu(m));
                return;
            }
            Menu::Question => {
                let Some(q) = self.pending.as_ref().map(|p| p.ask.clone()) else { return };
                if let mad_sim::Ask::Match { fee, .. } = q {
                    // EXE 0x9DB78 on the matchmaker picture, no frame
                    let src = super::subst(&[b"$g".as_slice(), &assets.text(super::addr::KUPPLERIN_OFFER)].concat(), b"%d", &t(&dotted(fee)));
                    f.inline(&src, vec![Action::Answer(0), Action::Answer(1)]);
                    self.popups.push_back(f.popup_at([KUPPLERIN_OFFER_RECT[0], KUPPLERIN_OFFER_RECT[1], KUPPLERIN_OFFER_RECT[2], 100]).with_menu(m));
                    return;
                }
                self.form_icons.retain(|i| i.0 != m);
                self.question(assets, &mut f, &q);
                if matches!(q, mad_sim::Ask::LoanOffer { .. }) {
                    (x, w) = (100, 440); // live capture: box x 100..540
                }
                if let mad_sim::Ask::PickDirector { candidates, .. } = &q {
                    let h = (48 * candidates.len() as i32 + 39) / 40 * 40 + 120;
                    self.form_icons.retain(|i| i.0 != m);
                    if !candidates.is_empty() {
                        self.form_icons.push((m, candidates.len(), 0x7A));
                    }
                    self.popups.push_back(f.popup_at([80, (480 - h) / 2, 480, h]).with_menu(m));
                    return;
                }
                if matches!(q, mad_sim::Ask::JuryBribe { .. }) {
                    self.popups.push_back(f.popup_at([10, 280, 620, 200]).with_menu(m));
                    return;
                }
                // church wedding caption menus (rules-15 §7.1 steps 5, 7, 9): gold text on the altar / banquet picture
                if let Some(rect) = match q {
                    mad_sim::Ask::BlackmailCanon { .. } | mad_sim::Ask::CanonVeto { .. } | mad_sim::Ask::Vows { .. } => Some([80, 180, 480, 280]),
                    mad_sim::Ask::WeddingFeast { .. } => Some([20, 120, 600, 360]),
                    _ => None,
                } {
                    f.src.splice(0..0, *b"$g");
                    self.popups.push_back(f.popup_at(rect).with_menu(m));
                    return;
                }
                if matches!(q, mad_sim::Ask::FencingLesson { .. }) {
                    // rules-38 §2.5
                    self.popups.push_back(f.popup_at([140, 160, 360, 160]).with_menu(m));
                    return;
                }
                if matches!(q, mad_sim::Ask::AttendAuction { .. }) {
                    self.popups.push_back(f.popup(assets, 120, 400, None).with_menu(m));
                    return;
                }
                if matches!(q, mad_sim::Ask::Gift { .. } | mad_sim::Ask::Plot { .. } | mad_sim::Ask::ContinueSiege { .. } | mad_sim::Ask::SiegeWon { .. }) {
                    // rules-63 §2, rules-13, rules-21 §2.2: printed into the turn-end report window, no dialog of their own
                    if let mad_sim::Ask::SiegeWon { .. } = q {
                        // the conquest film plays first, 12/100 s per frame (rules-56 §10), then a fresh window (live A/B)
                        self.popups.extend(Self::film_song(assets, "BELAGER.FLC", 8, 12));
                        self.window.clear();
                    }
                    self.report_menu = Some((f.src.clone(), f.items.clone()));
                    // the window's text so far stands above the question; the gems of menus answered before are inert
                    let pre = super::report_src(&self.window);
                    let n = pre.windows(2).filter(|w| w == b"%m").count();
                    f.src.splice(0..0, pre);
                    f.items.splice(0..0, std::iter::repeat_n(b"   ".to_vec(), n));
                    f.acts.splice(0..0, std::iter::repeat_n(Action::Inert, n));
                    self.popups.push_back(f.popup_at(super::REPORT_BOX).with_menu(m));
                    return;
                }
                if matches!(q, mad_sim::Ask::OccupyCastle { .. }) {
                    self.popups.push_back(f.popup_at(super::battle::REPORT_RECT).with_menu(m));
                    return;
                }
                if matches!(q, mad_sim::Ask::Building { .. } | mad_sim::Ask::PreFight { .. }) {
                    self.popups.push_back(f.popup_at([120, 120, 400, 240]).with_menu(m));
                    return;
                }
                if matches!(q, mad_sim::Ask::AuctionRefused { .. }) {
                    self.popups.push_back(f.popup_at([140, 160, 360, 160]).with_menu(m));
                    return;
                }
                if matches!(q, mad_sim::Ask::AuctionTick { .. } | mad_sim::Ask::AuctionLine { .. }) {
                    // the original prints the hall lines as gold text straight on the benches (live run 2026-11-10),
                    // bids at y = 372, the lot intro at y = 396 (rules-15 §1.4)
                    f.src.splice(0..0, *b"$g");
                    let rect = match q {
                        mad_sim::Ask::AuctionLine { step: 0, .. } => [10, 20, 620, 80],
                        mad_sim::Ask::AuctionLine { step: 1, .. } => [10, 390, 620, 90],
                        _ => [10, 366, 620, 114],
                    };
                    self.popups.push_back(f.popup_at(rect).with_menu(m));
                    return;
                }
            }
            Menu::Candles | Menu::Alms => {
                let (a, price, cmd) = if m == Menu::Candles {
                    (addr::CANDLES, self.st.church_prices()[0], Command::Candles)
                } else {
                    (addr::ALMS, self.st.church_prices()[1], Command::Alms)
                };
                let src = super::subst(&assets.text(a), b"%d", &t(&dotted(price)));
                // live A/B 2026-11-14: both windows are centred on x 320 (Kerzen 1541 px off, the price; Spenden 657)
                (x, w) = if m == Menu::Candles { (160, 320) } else { (140, 360) };
                f.inline(&src, vec![Action::Sim(cmd, None), Action::Close]);
            }
            Menu::Pray => {
                self.sim(assets, Command::Pray);
                (x, w) = (160, 320); // original 163..476; the 12 px inset would wrap the line at 316
                f.raw(&assets.text(addr::PRAY));
            }
            Menu::Indulgence => {
                let price = self.st.indulgence_price(h);
                self.sim(assets, Command::IndulgenceView);
                let city = self.loc(self.ablass_city).to_string();
                let src = super::subst(&super::subst(&assets.text(0xACE40), b"%d", &t(&dotted(price))), b"%s", &t(&city));
                (x, w) = (140, 360);
                f.inline(&src, vec![Action::Sim(Command::Indulgence, None), Action::Close]);
            }
            Menu::Testament => {
                use mad_sim::heap::ANY;
                let kids: Vec<usize> = self.st.heap.all(31, h as u8, ANY).into_iter().filter(|&i| self.st.heap.recs[i][3] < 45).collect();
                let heir = kids.iter().find(|&&i| self.st.heap.recs[i][4] & 0x80 != 0).map_or("Das Erzbistum".to_string(), |&i| self.st.name_str(self.st.heap.recs[i][5] as usize));
                f.raw(&assets.text(0xA9605));
                f.bind(t(&heir), Action::Sim(Command::Testament { d: 1 }, Some(Menu::Testament)));
                match kids.len() {
                    0 | 1 => {}
                    2 => {
                        f.raw(&assets.text(0xA96A9));
                    }
                    _ => {
                        f.raw(&assets.text(0xA96F8));
                    }
                }
                f.raw(b"$Z$C").item(t(BACK), Action::Close);
                (x, w) = (120, 400);
            }
            Menu::Feast(d) => {
                let branches: Vec<usize> = self.st.props_of(h, mad_sim::state::KIND_BRANCH).collect();
                if branches.is_empty() {
                    return;
                }
                let city = |b: usize| self.loc(self.st.props[b].loc()).to_string();
                if let Some(i) = self.st.heap.find(28, h as u8, mad_sim::heap::ANY) {
                    // 0x9DF77: a feast is already planned
                    let (b, year) = (self.st.heap.recs[i][2] as usize, self.st.year + self.st.heap.recs[i][3] as u16 - 1);
                    let src = super::subst(&super::subst(&assets.text(0x9DF77), b"%s", &t(&city(b))), b"%l", &t(&year.to_string()));
                    (x, w) = (140, 360); // centred like the measured Spenden window
                    f.inline(&src, vec![Action::Sim(Command::PlanFeast { branch: b as u8, size: 0, musicians: 0, years: 0 }, None), Action::Close]);
                } else {
                    let word = |a: u32| assets.text(a);
                    let sizes = [0x9DE0C, 0x9DE15, 0x9DE1F, 0x9DE26].map(word);
                    let bands = [0x9DE2F, 0x9DE3F, 0x9DE4A, 0x9DE59, 0x9DE5F].map(word);
                    let k = d[2] as usize % branches.len();
                    let next = |i: usize, n: u8, max: u8| {
                        let mut e = d;
                        e[i] = (n + 1) % max;
                        Action::Open(Menu::Feast(e))
                    };
                    f.raw(b"$Z$L").txt("Was für ein Fest soll es geben?").raw(b"$N$Z$L").txt("Ein ").raw(b"%m");
                    f.bind(sizes[d[0] as usize].clone(), next(0, d[0], 4));
                    f.txt(" Fest mit ").raw(b"%m");
                    f.bind(bands[d[1] as usize].clone(), next(1, d[1], 5));
                    f.txt(" Musikern.").raw(b"$N$Z$L").txt("Wann soll das Fest stattfinden?").raw(b"$N$Z$L").txt("Im Jahr ").raw(b"%m");
                    f.bind(t(&(self.st.year + d[3] as u16).to_string()), next(3, d[3], 11));
                    f.txt("!").raw(b"$N$Z$L").txt("Wo soll das Fest stattfinden?").raw(b"$N$Z$L").txt("In ").raw(b"%m");
                    f.bind(t(&city(branches[k])), next(2, d[2], branches.len() as u8));
                    f.raw(b"$N$Z$L");
                    let plan = Command::PlanFeast { branch: branches[k] as u8, size: d[0], musicians: d[1], years: d[3] };
                    f.item(t("Fest planen"), Action::Sim(plan, None)).raw(b"$N");
                    f.item(t(BACK), Action::Close);
                    (x, w) = (120, 400);
                }
            }
            Menu::Convert | Menu::Leave => {
                let net = self.st.net_worth(h).max(0);
                // centred on x 320 like the measured Kerzen and Spenden windows (live A/B 2026-11-14)
                (x, w) = if m == Menu::Convert { (160, 320) } else { (140, 360) };
                if m == Menu::Convert {
                    let to = if self.st.persons[h].religion() == 0 { "katholisch" } else { "evangelisch" };
                    let src = super::subst(&super::subst(&assets.text(0xA8527), b"%d", &t(&dotted(net / 30))), b"%s", &t(to));
                    f.inline(&src, vec![Action::Sim(Command::Convert, None), Action::Close]);
                } else {
                    let warn = assets.text(if self.st.persons[h].church() { 0xAB6F6 } else { 0xAB6B2 });
                    let src = super::subst(&super::subst(&assets.text(0xAB732), b"%s", &warn), b"%d", &t(&dotted(net / 10)));
                    f.inline(&src, vec![Action::Sim(Command::LeaveChurch, None), Action::Close]);
                }
            }
            Menu::JoinChurch => {
                // `0xAB5E4`: "Den evangelischen, den katholischen oder besser doch keinen" (religion 0, 1).
                let acts = vec![Action::Sim(Command::JoinChurch { religion: 0 }, None), Action::Sim(Command::JoinChurch { religion: 1 }, None), Action::Close];
                f.inline(&assets.text(addr::CONFESSIONLESS), acts);
            }
            Menu::Directors => {
                // §10.2: window (40, y) width 560, one row per director with his budget.
                let rows: Vec<(usize, u8)> = self
                    .st
                    .props_of(h, mad_sim::state::KIND_BRANCH)
                    .filter_map(|b| {
                        let d = self.st.props[b].director();
                        (d != mad_sim::state::NONE).then_some((b, d))
                    })
                    .collect();
                if rows.is_empty() {
                    f.raw(&assets.text(addr::NO_DIRECTORS));
                    // a plain message box when there are none (live A/B capture)
                    self.popups.push_back(f.popup(assets, 160, 320, Some(m)));
                    return;
                } else {
                    f.raw(&assets.text(addr::DIRECTORS_TITLE));
                    for (b, d) in rows {
                        f.raw(b"$L").txt(&format!("{} in {}:", self.person_name(d), self.loc(self.st.props[b].loc()))).raw(b"$E$R");
                        f.digits(self.st.props[b].budget(), 6, Field::Budget(b)).txt(" Taler").raw(b"$N");
                    }
                    f.raw(&assets.text(addr::DIRECTORS_FOOT));
                }
                let h = (f.layout_height(assets, 560) + 12).clamp(80, 440);
                self.popups.push_back(f.popup_at([40, (480 - h) / 2, 560, h]).with_menu(m));
                return;
            }
            Menu::Residence(b) => {
                // rules-34 §1: window (60, 40, 520, 400), picture of the level, six icon items
                let pr = &self.st.props[b];
                let (word, c) = (pr.residence(), pr.loc());
                f.raw(&assets.text(0xBE8E4));
                if pr.has_residence() {
                    if word & 0xE0 == 0 {
                        f.raw(b"$N");
                    }
                    let s = super::subst(&assets.text(0xBE8F8), b"%1W", &t(&self.wform(assets, 1, word)));
                    f.raw(&super::subst(&s, b"%o", &t(self.loc(c))));
                } else {
                    f.raw(&super::subst(&assets.text(0xBE908), b"%o", &t(self.loc(c))));
                }
                f.raw(&assets.text(0xBE934));
                let sell = super::subst(&super::subst(&assets.text(0xBEAD8), b"%4W", &t(&self.wform(assets, 4, word))), b"%d", &t(&dotted(self.st.residence_value_of(b) / 3)));
                let cost = (self.st.res_value(b, (word & 0xFF) | 0x6400) - self.st.residence_value_of(b)).max(100);
                let reno = super::subst(&super::subst(&super::subst(&assets.text(0xBED84), b"%2W", &t(&self.wform(assets, 2, word))), b"%d", &t(&dotted(cost))), b"%l", &t(&(self.st.year as u32 + 1 + ((word >> 8) < 25) as u32).to_string()));
                let lot = Action::Open(Menu::ResAuction(b));
                let acts = [
                    (3, "Kaufen", Action::Open(Menu::ResBuy(b))),
                    (3, "Bauen", Action::Open(Menu::ResBuild(b))),
                    (7, "Verkaufen", Action::ConfirmText(sell, [80, 120, 480, 240], Box::new(Action::Sim(Command::SellResidence { branch: b as u8 }, Some(m))))),
                    (6, "Versteigern", lot),
                    (7, "Erweitern", Action::Open(Menu::ResExtend(b))),
                    (7, "Renovieren", Action::ConfirmText(reno, [80, 100, 480, 280], Box::new(Action::Sim(Command::RenovateResidence { branch: b as u8 }, Some(m))))),
                ];
                for (flags, label, act) in acts {
                    let act = self.residence_check(assets, b, flags).unwrap_or(act);
                    f.icon_item(label, act).raw(b"$Z");
                }
                self.popups.push_back(f.popup_at([60, 40, 520, 400]).with_menu(m));
                return;
            }
            Menu::ResBuy(b) => {
                // rules-35 §1: the three offers with bullets (price V·11/10), credit line, Kaufen icon;
                // a click on an offer buys at once (the original selects, then buys with the icon)
                let pr = &self.st.props[b];
                let c = pr.loc() as usize;
                f.raw(&assets.text(0xBE99C));
                f.gems = true;
                self.form_icons.retain(|i| i.0 != m);
                let mut buys = vec![];
                for k in 0..3u8 {
                    let w = self.st.residence_offer(c, k as usize);
                    let label = format!("{} für {} Taler", self.wform(assets, 5, w), dotted(self.st.res_value(b, w) * 11 / 10));
                    f.bullet(&label.chars().filter_map(super::cp437).collect::<Vec<u8>>(), Action::Select(k as u32)).raw(b"$Z$|");
                    buys.push(Action::Sim(Command::BuyResidence { branch: b as u8, k }, Some(Menu::Residence(b))));
                }
                if pr.has_residence() {
                    f.raw(&super::subst(&assets.text(0xBE9F8), b"%d", &t(&dotted(self.st.residence_value_of(b) * 2 / 3))));
                }
                f.raw(b"$Z$L");
                self.form_icons.push((m, f.items.len(), 0x6A));
                f.icon_item("Kaufen", Action::WithSel(buys, false)).raw(b"$N");
                self.popups.push_back(f.popup_at([60, 40, 520, 400]).with_menu(m));
                return;
            }
            Menu::ResBuild(b) => {
                // rules-35 §2: all ten levels with their listed price, credit line, Bauen icon
                let pr = &self.st.props[b];
                let c = pr.loc() as usize;
                f.raw(&assets.text(0xBEA28));
                f.gems = true;
                self.form_icons.retain(|i| i.0 != m);
                let lvl = self.st.cities[c].building_level() as i64;
                let mut builds = vec![];
                for k in 0..10u16 {
                    builds.push(Action::Sim(Command::BuildResidence { branch: b as u8, level: k as u8 }, Some(Menu::Residence(b))));
                    let price = (lvl + 150) * self.st.res_value(b, 0x5000 | k) / 200;
                    let name: Vec<u8> = Self::u_form(assets, 1, 0x3A + k as u32);
                    f.bullet(&name, Action::Select(k as u32));
                    f.raw(b"$E$R").txt(&format!("{} Taler", dotted(price))).raw(b"$E$|$N");
                }
                if pr.has_residence() {
                    f.raw(&super::subst(&assets.text(0xBEA78), b"%d", &t(&dotted(self.st.residence_value_of(b) * 2 / 3))));
                }
                f.raw(b"$Z$L");
                self.form_icons.push((m, f.items.len(), 0x6E));
                f.icon_item("Bauen", Action::WithSel(builds, true)).raw(b"$N");
                self.popups.push_back(f.popup_at([120, 20, 400, 440]).with_menu(m));
                return;
            }
            Menu::ResAuction(b) => {
                // rules-35 §4: three layouts (no auction / lot exists / register), one click closes
                let pr = &self.st.props[b];
                let word = pr.residence();
                f.raw(&assets.text(0xBEB34));
                self.form_icons.retain(|i| i.0 != m);
                let auction = self.st.heap.find(3, mad_sim::state::NOBODY, mad_sim::heap::ANY).map(|i| self.st.heap.recs[i][3] as u32);
                match auction {
                    None => {
                        f.raw(&assets.text(0xBEB54).iter().copied().take_while(|&c| c != b'%').collect::<Vec<u8>>()).raw(b"$Z");
                    }
                    Some(n) => {
                        let year = t(&(self.st.year as u32 + n).to_string());
                        let lot = self.st.heap.find(14, pr.owner(), b as u8).is_some();
                        let (addr, form, act, label) = if lot {
                            (0xBEB90, 1, Action::Sim(Command::WithdrawLot { prop: b as u8 }, None), "Angebot zurückziehen")
                        } else {
                            (0xBEC04, 4, Action::Sim(Command::AuctionProperty { prop: b as u8 }, None), "Versteigern")
                        };
                        let s = super::subst(&super::subst(&assets.text(addr), b"%l", &year), format!("%{form}W").as_bytes(), &t(&self.wform(assets, form, word)));
                        f.raw(&s[..s.windows(2).position(|w| w == b"%j" || w == b"%M").unwrap_or(s.len())]);
                        self.form_icons.push((m, f.items.len(), 0x70));
                        f.icon_item(label, act).raw(b"$Z");
                        self.form_icons.push((m, f.items.len(), 0x30));
                        f.icon_item("Zurück.", Action::Close).raw(b"$N");
                    }
                }
                self.popups.push_back(f.popup_at([80, 120, 480, 240]).with_menu(m));
                return;
            }
            Menu::ResExtend(b) => {
                // rules-35 §5: the missing extensions with their cost; a click orders that one
                let word = self.st.props[b].residence();
                if word & 0xE0 == 0xE0 {
                    self.popups.push_back(Form::new(&super::subst(&assets.text(0xBEC7C), b"%3W", &t(&self.wform(assets, 3, word)))).popup_at([160, 140, 320, 200]));
                    return;
                }
                f.raw(&super::subst(&assets.text(0xBECBC), b"%3W", &t(&self.wform(assets, 3, word))));
                f.gems = true;
                self.form_icons.retain(|i| i.0 != m);
                let v0 = self.st.residence_value_of(b);
                for (bit, name) in [(0x20u8, "Park"), (0x40, "Nebengebäude"), (0x80, "Ländereien")] {
                    if word as u8 & bit == 0 {
                        let cost = self.st.res_value(b, word | bit as u16) - v0;
                        f.bullet(&format!("{name} für {} Taler", dotted(cost)).chars().filter_map(super::cp437).collect::<Vec<u8>>(), Action::Sim(Command::ExtendResidence { branch: b as u8, bits: bit }, Some(Menu::Residence(b)))).raw(b"$N$|");
                    }
                }
                f.raw(b"$Z$L");
                self.form_icons.push((m, f.items.len(), 0x71));
                f.icon_item("Erweitern lassen", Action::Close).raw(b"$N");
                self.popups.push_back(f.popup_at([80, 120, 480, 240]).with_menu(m));
                return;
            }
            Menu::Director(b) => {
                let d = self.st.props[b].director();
                if d == mad_sim::state::NONE {
                    self.sim(assets, Command::HireDirector { branch: b as u8 });
                    return;
                }
                // rules-34 §3.2: one box (80, 120, 480, 240) with the dismissal icon 0x7B
                let s = super::subst(&assets.text(0xBEFE0), b"%3N5", &t(&self.pref(d, 3, 5)));
                let s = super::subst_seq(&s, b"%d", &[t(&self.st.director_fee_pct(d as usize).to_string()), t(&dotted(self.st.props[b].budget()))]);
                let s = super::subst(&super::subst(&s, b"%1N0", &t(&self.pref(d, 1, 0))), b"%4N1", &t(&self.pref(d, 4, 1)));
                let cut = s.windows(2).position(|w| w == b"%j").unwrap_or(s.len());
                f.raw(&s[..cut]);
                self.form_icons.retain(|i| i.0 != m);
                self.form_icons.push((m, f.items.len(), 0x7B));
                let label = s[cut..].strip_prefix(b"%j").unwrap_or(b"").iter().copied().take_while(|&c| c != b'$').collect::<Vec<u8>>();
                f.item([b"     ".to_vec(), label].concat(), Action::Sim(Command::FireDirector { branch: b as u8 }, None)).raw(b"$N");
                self.popups.push_back(f.popup_at([80, 120, 480, 240]).with_menu(m));
                return;
            }
            Menu::Expand(q) => {
                self.amount = 0;
                let pr = &self.st.props[q];
                let castle = pr.kind() == KIND_CASTLE;
                f.raw(&super::subst(&assets.text(if castle { 0xBF700 } else { 0xBF71C }), b"%o", &t(self.loc(pr.loc()))));
                let (a, b) = if castle { ("Sicherheit", "Kasematten") } else { ("Tarnung", "Strohmatten") };
                // R + 0x12 of the static record 0x8647C (26 bytes, index = location - 14): the year of construction (live: 1210)
                let year = u16::from_le_bytes(assets.exe_bytes(0x8647C + 26 * (pr.loc() as u32 - 14) + 0x12, 2).try_into().unwrap_or([0; 2]));
                // rules-27 §3: the goods in stock (all 15 summed) for camps and stocked castles, else the year built
                let goods: u32 = (0..15).map(|g| pr.stock(g) as u32).sum();
                if goods > 0 || !castle {
                    let dotted = goods.to_string().as_bytes().rchunks(3).rev().map(|c| String::from_utf8_lossy(c).into_owned()).collect::<Vec<_>>().join(".");
                    f.raw(&super::subst(&assets.text(0xBF734), b"%d", dotted.as_bytes()));
                } else {
                    f.raw(&super::subst(&assets.text(0xBF748), b"%l", year.to_string().as_bytes()));
                }
                // condition line and action row as the live capture shows them: the line above the icon row, two columns
                // rules-26 §5.1: the condition item at window y 354, the action row at y 384
                f.raw(b"$354{$C$/").txt(&format!("{a}: {}%", pr.b3())).raw(b"$E$1/").txt(&format!("{b}: {}", pr.b4())).raw(b"$|$384{$C$/");
                f.icon_item(a, Action::Open(Menu::Improve(q, 0))).raw(b"$E$1/");
                f.icon_item("Erweitern", Action::Open(Menu::Improve(q, 1))).raw(b"$|");
                self.popups.push_back(f.popup_at([100, 20, 440, 440]).with_menu(m));
                return;
            }
            Menu::Improve(q, which) => {
                let pr = &self.st.props[q];
                let castle = pr.kind() == KIND_CASTLE;
                let (cur, cap, digits) = if which == 0 { (pr.b3() as u32, 100u32, 2) } else { (pr.b4() as u32, 150, 3) };
                let amount = self.amount.min(cap.saturating_sub(cur));
                let title = match (which, castle) {
                    (0, true) => "Sicherheit verbessern",
                    (0, false) => "Tarnung verbessern",
                    (_, true) => "Zollburg erweitern",
                    _ => "Räuberlager erweitern",
                };
                let what = match (which, castle) {
                    (0, _) => "% verbessern",
                    (_, true) => " Kasematten bauen",
                    _ => " Strohmatten stopfen",
                };
                f.raw(b"$P$C^ ").txt(title).raw(b" _$N$N$C");
                f.item(t("      "), Action::Fortify(q, which, amount as u8)).digits(amount, digits, Field::Amount).txt(what).raw(b"$N$N$C");
                f.txt(&format!("Kosten: {} Taler", dotted(self.st.fortify_cost(q, which, amount as u8) as u32))).raw(b"$N");
                self.popups.push_back(f.popup_at([120, 160, 400, 160]).with_menu(m));
                return;
            }
            Menu::Recruit(q) => {
                let castle = self.st.props[q].kind() == KIND_CASTLE;
                let have: u32 = self.st.heap.all(15, h as u8, q as u8).iter().map(|&i| mad_sim::heap::r16(&self.st.heap.recs[i], 4) as u32).sum();
                f.raw(&assets.text(if castle { 0xBFA2C } else { 0xBFA4C }));
                f.txt(&format!("Werbegelder: {} Taler", dotted(have))).raw(b"$N$15H$C");
                f.item(t("      "), Action::Recruit(q, self.amount as u16)).digits(self.amount.min(9999), 4, Field::Amount).txt(" Taler investieren").raw(b"$N");
                self.popups.push_back(f.popup_at([100, 20, 440, 440]).with_menu(m));
                return;
            }
            Menu::Manoeuvre(q) => {
                self.sim(assets, Command::ManoeuvreView);
                let pr = &self.st.props[q];
                let n: i32 = (0..4).map(|k| pr.troops(k) as i32).sum();
                let avg = if n == 0 { 50 } else { (0..4).map(|k| pr.troops(k) as i32 * pr.training(k) as i32).sum::<i32>() / n };
                let words = assets.name_table((0xB6814, 10));
                f.raw(&assets.text(0xBFAA0));
                f.raw(&super::subst(&assets.text(0xBFAB4), b"%s", &t(&words[(avg as usize / 11).min(9)])));
                let cost = 15 * n + (self.st.year as i32 * 277) % 20;
                f.raw(b"$14H$C").txt(&format!("Ein Manöver kostet {} Taler.", dotted(cost as u32))).raw(b"$Z$L$12>");
                f.icon_item("Durchführen lassen", Action::Manoeuvre(q)).raw(b"$N");
                self.popups.push_back(f.popup_at([100, 20, 440, 440]).with_menu(m));
                return;
            }
            Menu::Toll(q) => {
                let rate = self.st.castles[self.st.props[q].loc() as usize - 14].toll_rate;
                f.raw(b"$Z$C^ Zollsatz Burg ").txt(self.loc(self.st.props[q].loc())).raw(b" _$N$15H$C");
                f.txt("Zollsatz: ").digits(rate as u32, 2, Field::Toll(q)).txt("%").raw(b"$N");
                self.popups.push_back(f.popup_at([100, 20, 440, 440]).with_menu(m));
                return;
            }
            Menu::CastleInfo(k) => {
                // rules-26 §2: picture and icons are drawn by the popup code; text from EXE 0xBF2B8 / 0xBF36C
                let Some(q) = self.place_prop(k) else { return };
                let (castle, owner) = (self.st.props[q].kind() == KIND_CASTLE, self.st.props[q].owner());
                let loc = t(self.loc(14 + k));
                let val = t(&dotted(self.st.property_value(q) as u32));
                let toll = t(&self.st.castles.get(k as usize).map_or(0, |c| c.toll_rate).to_string());
                let who = t(&self.pref(owner, 3, 5));
                let base = assets.text(if castle { 0xBF2B8 } else { 0xBF36C });
                let head = super::subst_seq(&super::subst(&super::subst(&base, b"%o", &loc), b"%3N5", &who), b"%d", &if castle { vec![val, toll] } else { vec![val] });
                f.raw(&head);
                // a single item (camp owner, non-owner) is centred with its icon (live captures); the owner's two castle items sit left and right
                let single = !(castle && owner as usize == h);
                f.raw(if single { b"$C".as_slice() } else { b"$L$14>" });
                // the single item is an icon item: the icon reserves 32 px, then a space and the label (live A/B)
                let put = |f: &mut Form, s: &str, a: Action| {
                    if single {
                        f.icon_item(s, a);
                    } else {
                        f.item(t(s), a);
                    }
                };
                let pb = q as u8;
                let again = Some(Menu::CastleInfo(k));
                if owner as usize == h {
                    let buyers = self.st.heap.all(17, mad_sim::heap::ANY, pb).len();
                    let sell = if buyers == 0 { Action::Bytes(super::subst(&assets.text(0xBF420), b"%s", &loc)) } else { Action::Open(Menu::CastleSell(k)) };
                    put(&mut f, "Verkaufen", sell);
                    if castle {
                        if let Some(i) = self.st.heap.find(16, h as u8, pb) {
                            let year = (self.st.year as u32 + self.st.heap.recs[i][3] as u32).to_string();
                            f.raw(b"$N").raw(&super::subst(&super::subst(&assets.text(0xBF348), b"%s", &loc), b"%l", year.as_bytes()));
                        } else {
                            let lot = if let Some(&i) = self.st.heap.all(3, mad_sim::heap::ANY, mad_sim::heap::ANY).first() {
                                let year = (self.st.year as u32 + self.st.heap.recs[i][3] as u32).to_string();
                                let q = super::subst(&super::subst(&assets.text(0xBF51C), b"%l", year.as_bytes()), b"%s", &loc);
                                Action::ConfirmText(q, [140, 140, 360, 200], Box::new(Action::Sim(Command::AuctionProperty { prop: pb }, again)))
                            } else {
                                Action::Bytes(assets.text(0xBF4E4))
                            };
                            f.raw(b"$E$51>").item(t("Versteigern"), lot);
                        }
                    }
                } else if self.st.heap.find(17, h as u8, pb).is_some() {
                    put(&mut f, "Kaufinteresse zurückziehen", Action::Sim(Command::Interest { prop: pb, on: false }, again));
                } else if self.st.heap.all(17, mad_sim::heap::ANY, pb).len() < 6 {
                    put(&mut f, "Kaufinteresse bekunden", Action::Sim(Command::Interest { prop: pb, on: true }, again));
                }
                f.raw(b"$|");
                self.popups.push_back(f.popup_at([80, 20, 480, 440]).with_menu(m));
                return;
            }
            Menu::CastleSell(k) => {
                let Some(q) = self.place_prop(k) else { return };
                let pb = q as u8;
                let head = super::subst_seq(&assets.text(0xBF454), b"%s", &[t(self.loc(14 + k))]);
                let head = super::subst(&head, b"%d", &t(&dotted(self.st.property_value(q) as u32)));
                let mut src = [head, b"$L".to_vec()].concat();
                let mut acts = vec![];
                for i in self.st.heap.all(17, mad_sim::heap::ANY, pb).into_iter().take(6) {
                    let b = self.st.heap.recs[i][1];
                    src.extend([b"%a".to_vec(), t(&self.person_name(b)), b"$N".to_vec()].concat());
                    acts.push(Action::Sim(Command::SellCastle { prop: pb, buyer: b }, Some(Menu::CastleInfo(k))));
                }
                src.extend(b"%aNiemandem$N");
                acts.push(Action::Close);
                f.inline(&src, acts);
                self.popups.push_back(f.popup_at([140, 140, 360, 200]).with_menu(m));
                return;
            }
            Menu::Orders(p) => {
                // rules-26 §3.3: the order book is a parchment strip over the bottom of the castle or camp screen,
                // one line per slot: "Überfallt Seefels mit 5+0+0+1   Söldner  <status glyph>"
                let shown = self.popups.iter().any(|q| matches!(q, super::Popup::Choice { menu: Some(Menu::Orders(o)), .. } if *o == p));
                if let Some(b) = self.book_shown.get_mut(self.st.props[p].loc().wrapping_sub(14) as usize) {
                    *b = !shown;
                }
                if shown {
                    self.popups.retain(|q| !matches!(q, super::Popup::Choice { menu: Some(Menu::Orders(o)), .. } if *o == p));
                    return;
                }
                let pr = &self.st.props[p];
                let castle = pr.kind() == KIND_CASTLE;
                let base = if castle { 0x10 } else { 0x20 };
                let perm = if castle { [0usize, 1, 2, 3] } else { [0, 3, 2, 1] };
                let verbs = ["Kein Auftrag", "Überfallt", "Schickt", if castle { "Überwacht" } else { "Plündert" }];
                // rules-26 §3.1 step 3: text cursor y 338 on screen
                f.raw(b"$18{");
                for k in 0..2u8 {
                    let o = pr.order(k as usize);
                    let ty = o[0].saturating_sub(base).min(3);
                    let fld = |x| Field::Order(p, k, x);
                    if ty == 0 {
                        f.raw(b"$C").item(t("                                Kein Auftrag                                "), Action::Step(fld(OrderField::TroopKind), 1)).raw(b"$N$|$L");
                        continue;
                    }
                    f.raw(b"$R$25<").item(t(verbs[ty as usize]), Action::Step(fld(OrderField::TroopKind), 1)).raw(b" ");
                    let target = if ty == 3 { self.loc(10 + o[2]) } else { self.loc(14 + o[2]) };
                    f.item(t(target), Action::Step(fld(OrderField::TroopTarget), 1));
                    if ty != 2 {
                        f.txt(" mit ");
                    } else {
                        f.txt(" ");
                    }
                    for (q, &slot) in perm.iter().enumerate() {
                        if q > 0 {
                            f.txt("+");
                        }
                        f.digits(o[3 + slot] as u32, 2, fld(OrderField::Troop(slot as u8)));
                    }
                    let one = perm.iter().map(|&sl| o[3 + sl] as u32).sum::<u32>() == 1;
                    let noun = match (castle, ty == 2 || one) {
                        (true, true) => "Söldner",
                        (true, false) => "Söldnern",
                        (false, true) => "Räuber",
                        (false, false) => "Räubern",
                    };
                    f.raw(b"$E$2<$L$82>").txt(noun).raw(b"$E$R");
                    match o[7] {
                        mad_sim::economy::ST_DONE => {}
                        st => {
                            f.item(vec![0x0D + st.min(2)], Action::ToggleOrder(p, k));
                        }
                    }
                    f.raw(b"$N$|$L");
                }
                self.popups.push_back(f.popup_at([0, 320, 640, 80]).with_menu(m));
                return;
            }
        }
        self.popups.push_back(f.popup(assets, x, w, Some(m)));
    }

    /// Own castle/camp at location `14 + t` (rules-07 §3a).
    /// rules-34 §1 preconditions of a residence action (flags 1 auction lot, 2 construction, 4 a residence).
    fn residence_check(&self, assets: &Assets, b: usize, flags: u8) -> Option<Action> {
        let pr = &self.st.props[b];
        let (o, c, word) = (pr.owner(), pr.loc(), pr.residence());
        if flags & 1 != 0 && !self.st.heap.all(14, o, b as u8).is_empty() {
            return Some(Action::Boxed(super::subst(&assets.text(0xBE804), b"%1W", &t(&self.wform(assets, 1, word))), [160, 140, 320, 200]));
        }
        if flags & 2 != 0 && let Some(&i) = self.st.heap.all(13, o, b as u8).first() {
            let s = super::subst(&assets.text(0xBE864), b"%o", &t(self.loc(c)));
            let year = self.st.year as u32 + self.st.heap.recs[i][3] as u32;
            return Some(Action::Boxed(super::subst(&s, b"%l", &t(&year.to_string())), [160, 160, 320, 160]));
        }
        if flags & 4 != 0 && !pr.has_residence() {
            return Some(Action::Boxed(super::subst(&assets.text(0xBE8B8), b"%o", &t(self.loc(c))), [160, 200, 320, 120]));
        }
        None
    }

    fn owns_at(&self, h: usize, t: u8) -> bool {
        (0..mad_sim::state::PROPS).any(|q| {
            let p = &self.st.props[q];
            p.loc() == 14 + t && matches!(p.kind(), mad_sim::state::KIND_CASTLE | mad_sim::state::KIND_CAMP) && p.owner() as usize == h
        })
    }

    /// New order after a click on the type word or the target word, or a troop count step.
    pub(super) fn edit_troop_order(&mut self, assets: &Assets, p: usize, k: u8, f: OrderField, d: i32) {
        let Some(h) = self.me() else { return };
        let base = if self.st.props[p].kind() == mad_sim::state::KIND_CASTLE { 0x10 } else { 0x20 };
        let mut o = self.st.props[p].order(k as usize);
        let first = |s: &Self, ty: u8| match ty {
            1 => (0..7).find(|&t| !s.owns_at(h, t)).unwrap_or(0),
            2 => (0..7).find(|&t| s.owns_at(h, t) && 14 + t != s.st.props[p].loc()).unwrap_or(0),
            _ => 0,
        };
        match f {
            OrderField::TroopKind => {
                let ty = (o[0].saturating_sub(base) + 1) % 4;
                o = [base + ty, 0, first(self, ty), 0, 0, 0, 0, 0];
            }
            OrderField::TroopTarget => {
                let ty = o[0].saturating_sub(base);
                let n = if ty == 3 { 3 } else { 7 };
                let ok = |s: &Self, t: u8| match ty {
                    1 => !s.owns_at(h, t),
                    2 => s.owns_at(h, t) && 14 + t != s.st.props[p].loc(),
                    _ => true,
                };
                if let Some(t) = (1..=n).map(|i| (o[2] + i) % n).find(|&t| ok(self, t)) {
                    o[2] = t;
                }
            }
            OrderField::Troop(j) => o[3 + j as usize] = (o[3 + j as usize] as i32 + d).clamp(0, 255) as u8,
            _ => return,
        }
        self.sim(assets, Command::SetOrder { branch: p as u8, slot: k, order: o });
    }

    /// The pre-fight window's question after its page text (rules-52): 2881 ransom (5-digit spinner), 3069 attack,
    /// 2888..2890 helper bribes (a row per helper but the briber's own), 3142 side.
    fn prefight_question(&self, assets: &Assets, f: &mut Form, q: &mad_sim::fight::Parley) {
        use mad_sim::{WArg::{P, Word}, fight::Parley};
        let cut = |src: &[u8]| src.windows(2).position(|w| w == b"%a").unwrap_or(src.len());
        // the page so far stays above the question unless it opens a new page (`$P`)
        let start = |f: &mut Form, src: Vec<u8>| match src.strip_prefix(b"$P") {
            Some(r) => r.to_vec(),
            None => {
                f.raw(&self.page);
                src
            }
        };
        match q {
            Parley::None => {}
            Parley::Ransom { defender, attacker } => {
                let src = start(f, self.exe_fmt(assets, &assets.text_id(2881), &[P(*defender), P(*attacker)]));
                // `…$Z%a%m Taler.%aNiemals, ich gewinne!$|`: the bullet pays the spinner's amount
                let (head, tail) = src.split_at(cut(&src));
                let tail = tail.strip_prefix(b"%a%m").unwrap_or(tail);
                let (unit, rest) = tail.split_at(cut(tail));
                f.raw(head).item_bullet_raw(b"", Action::Answer(self.amts[0]));
                f.digits(self.amts[0], 5, Field::Amt(0, 99_999)).raw(unit).raw(b"$N");
                f.inline(rest, vec![Action::Answer(0)]);
            }
            Parley::Attack { attacker } => {
                let src = start(f, self.exe_fmt(assets, &assets.text_id(3069), &[P(*attacker)]));
                f.inline(&src, vec![Action::Answer(0), Action::Answer(1)]);
            }
            Parley::Bribe { briber, helpers } => {
                let src = start(f, self.exe_fmt(assets, &assets.text_id(2888), &[P(*briber)]));
                f.raw(&src);
                for (h, &(owner, word)) in helpers.iter().enumerate() {
                    // the briber's own troops: two invisible items (0xBBA90 `%M%M$E`)
                    if owner == *briber {
                        continue;
                    }
                    let row = self.exe_fmt(assets, &assets.text_id(2889), &[P(owner), Word(0xB601C, word)]);
                    let row = row.strip_prefix(b"%a%m").unwrap_or(&row);
                    let row = row.strip_suffix(b"$|").unwrap_or(row);
                    let amt = self.amts[h & 3];
                    f.item_bullet_raw(b"", Action::Answer(1 + ((h as u32) << 20 | amt)));
                    f.digits(amt, 5, Field::Amt(h as u8 & 3, 99_999)).raw(row).raw(b"$N$|");
                }
                f.inline(&assets.text_id(2890), vec![Action::Answer(0)]);
            }
            Parley::Side { helper, attacker, defender } => {
                let src = start(f, self.exe_fmt(assets, &assets.text_id(3142), &[P(*helper), P(*attacker), P(*defender)]));
                f.inline(&src, (0..3).map(Action::Answer).collect());
            }
        }
    }

    /// Dialogs for the sim questions with a real UI (rules-07 §5.5). Wording is ours except the
    /// occupy question, which is the EXE's (`0xBBE08`, castle; `0xBBF4C`, camp: the sim does not
    /// say which, so the castle text is used, with the location left out).
    fn question(&self, assets: &Assets, f: &mut Form, q: &mad_sim::Ask) {
        use mad_sim::Ask;
        match *q {
            Ask::PreFight { ref q, .. } => self.prefight_question(assets, f, q),
            Ask::Gift { target, prices, names } => {
                let src = self.gift_text(assets, target, prices, names);
                f.inline(&src, (0..4).map(Action::Answer).collect());
            }
            Ask::GiveUpCourtship { target, city } => {
                let src = super::subst(&super::subst(&assets.text(0x9D96C), b"%4N5", &t(&self.person_name(target))), b"%s", &t(self.loc(city)));
                f.inline(&src, vec![Action::Answer(0), Action::Answer(1)]);
            }
            Ask::Vows { suitor, target, .. } => {
                let word = assets.text(if self.st.persons[target as usize].gender() == 0 { 0x9D4A8 } else { 0x9D4AE });
                let src = super::subst(&super::subst(&super::subst(&assets.text(0x9D427), b"%1N0", &t(&self.person_name(suitor))), b"%4N0", &t(&self.person_name(target))), b"%s", &word);
                f.inline(&src, (0..3).map(Action::Answer).collect());
            }
            Ask::WeddingFeast { spouse, city, prices, .. } => {
                let src = super::subst(&assets.text(0x9D605), b"%2N0", &t(&super::genitive(&self.person_name(spouse))));
                let src = super::subst(&src, b"%g", &t(&super::genitive(self.loc(city))));
                let src = super::subst_seq(&src, b"%d", &prices.map(|p| t(&dotted(p))));
                f.inline(&src, (0..5).map(|k| Action::Answer(if k == 4 { 4 } else { k })).collect());
            }
            Ask::BlackmailCanon { by, canon, .. } => {
                let src = super::subst(&super::subst(&assets.text(0xBA9AC), b"%1N0", &t(&self.person_name(by))), b"%4N5", &t(&self.person_name(canon)));
                f.inline(&src, vec![Action::Answer(0), Action::Answer(1)]);
            }
            Ask::CanonVeto { canon, .. } => {
                let src = super::subst(&super::subst(&assets.text(0x9D215), b"%1N1", &t(&self.person_name(canon))), b"%1N4", &t(&self.rank(canon as usize)));
                f.inline(&src, vec![Action::Answer(0), Action::Answer(1)]);
            }
            Ask::ContinueSiege { city, cost } => {
                // EXE 0x9F5B5: Ja! continues (0), Nein! (1)
                let src = super::subst(&super::subst(&assets.text(0x9F5B5), b"%o", &t(self.loc(city))), b"%d", &t(&dotted(cost)));
                f.inline(&src, vec![Action::Answer(0), Action::Answer(1)]);
            }
            Ask::SiegeWon { city } => {
                // EXE 0x9F6C6: plunder (0), tribute (1), both (2)
                let src = super::subst(&assets.text(0x9F6C6), b"%o", &t(self.loc(city)));
                f.inline(&src, (0..3).map(Action::Answer).collect());
            }
            Ask::Vote { elector, ref candidates, for_, .. } => {
                // EXE 0xC18A0, or 0xC18C4 when the human votes for the appointer he blackmails (rules-12 §3.7 step 4);
                // the EXE strings end with `$N$H` (cursor home, for their own list); the list follows here
                let head = if for_ == 0xFF {
                    super::subst(&assets.text(0xC18A0), b"%1N3", &t(&self.pref(elector, 1, 3)))
                } else {
                    super::subst(&super::subst(&assets.text(0xC18C4), b"%1N3", &t(&self.pref(elector, 1, 3))), b"%4N3", &t(&self.pref(for_, 4, 3)))
                };
                f.raw(head.strip_suffix(b"$H").unwrap_or(&head)).raw(b"$Z$L");
                for (k, &c) in candidates.iter().enumerate() {
                    f.item(t(&format!("{}, {}", self.person_name(c), self.rank(c as usize))), Action::Answer(k as u32)).raw(b"$N");
                }
            }
            Ask::JuryBribe { person, county } => {
                // rules-15 §6.1: EXE 0xC121C in gold over the county castle picture, `%a%m Taler`: the bullet ends the
                // input with the amount, the 4-digit spinner sets it
                let src = super::subst(&super::subst(&assets.text(0xC121C), b"%1N1", &t(&self.pref(person, 1, 1))), b"%o", &t(self.loc(county)));
                let cut = src.windows(2).position(|w| w == b"%a").unwrap_or(src.len());
                f.raw(&src[..cut]);
                f.item_bullet_raw(b"", Action::Answer(self.amount));
                f.digits(self.amount, 4, Field::Amount).txt(" Taler").raw(b"$N");
            }
            Ask::TrialWitnesses { person } => {
                // EXE 0xAFD31: witnesses and court in one window; the sim asks twice, both answers
                // go out together (the following `TrialBribe` is then answered from the list)
                let src = assets.text(0xAFD31);
                let cut = src.windows(2).position(|w| w == b"%m").unwrap_or(src.len());
                let head = super::subst(&super::subst(&src[..cut], b"%1N0", &t(&self.person_name(person))), b"$g", b"");
                f.raw(&head);
                f.digits(self.amount, 5, Field::Amount).txt(" Taler in Zeugen und").raw(b"$N$L");
                f.digits(self.bribe, 5, Field::Bribe).txt(" Taler in das hohe Gericht.").raw(b"$Z$C");
                f.item(t("Hier sind die Taler!"), Action::AnswerPair(self.amount, self.bribe));
            }
            // answered together with the witnesses; never shown on its own
            Ask::TrialBribe { .. } => {
                f.txt("");
            }
            Ask::FencingLesson { cost } => {
                // EXE 0xAEB44: En garde! (0) / Keine Gewalt! (1)
                let src = super::subst(&assets.text(0xAEB44), b"%d", &t(&dotted(cost)));
                f.inline(&src, vec![Action::Answer(0), Action::Answer(1)]);
            }
            Ask::JudgeVote { judge } => {
                // EXE 0xA5F5A: Schuldig! (0) / Nicht schuldig. (1); gold `$g` is for the original's picture
                let src = super::subst(&super::subst(&assets.text(0xA5F5A), b"%1N3", &t(&self.pref(judge, 1, 3))), b"$g", b"");
                f.inline(&src, vec![Action::Answer(0), Action::Answer(1)]);
            }
            Ask::Sentence { defendant } => {
                // EXE 0xA600D and the sentence names 0xA8201..0xA8263 (rules-18 §10): one cycling item, entry 3
                // (loss of title) skipped when the defendant has none
                let me = self.me().unwrap_or(0) as u8;
                let head = assets.text(0xA600D);
                let cut = head.windows(2).position(|w| w == b"%a").unwrap_or(head.len());
                f.raw(&super::subst(&super::subst(&head[..cut], b"%1N3", &t(&self.pref(me, 1, 3))), b"$g", b"")).raw(b"$Z$L");
                let names = [0xA8201u32, 0xA8209, 0xA8214, 0xA822B, 0xA823D, 0xA8252, 0xA8263];
                let skip3 = self.st.persons[defendant as usize].title() == 0;
                f.item(assets.text(names[self.amount as usize % 7]), Action::Step(Field::Choice(7, skip3), 1)).raw(b"$Z$C").item(t("Das ist mein Urteil!"), Action::Answer(self.amount));
            }
            Ask::Plot { kind: 32, step, arg, .. } => {
                // inventor (rules-13 §2.1): EXE 0xA44DB, 0xA4767, 0xA49C7, 0xA5005; handler 0xA9EE0, 0xAA26D
                let text = |a: u32| assets.text(a);
                let (src, n) = match step {
                    0 => {
                        let tav = assets.name_table((0xB5AE4, 30)).get(arg as usize).cloned().unwrap_or_default();
                        (super::subst(&text(0xA44DB), b"%s", &t(&tav)), 3)
                    }
                    1 => (text(0xA4767), 2),
                    2 => {
                        let female = self.me().is_some_and(|h| self.st.persons[h].gender() == 1);
                        let price = self.st.cities[5].price(0) as i32 + 3;
                        (super::subst(&super::subst(&text(0xA49C7), b"%s", if female { b" Dame" } else { b"r Herr" }), b"%b", price.to_string().as_bytes()), 2)
                    }
                    3 => (text(0xA5005), 2),
                    10 => (text(0xA9EE0), 2),
                    _ => (super::subst(&text(0xAA26D), b"%d", &t(&dotted(arg))), 2),
                };
                f.inline(&src, (0..n).map(Action::Answer).collect());
            }
            Ask::Plot { kind: 41, step, arg, arg2 } => {
                // excavation (rules-13 §2.9): EXE 0xAC74F, 0x9E83F, 0x9E931 / 0x9E7BD, answers 0xAC7C6.
                // step 0: arg = cost, arg2 = branch; 10: arg = find, arg2 = branch; 11: arg = cost, arg2 = find (-1 none)
                let text = |a: u32| assets.text(a);
                // step 11 with a find: arg2 = find | answer << 4 | branch << 8, its reaction 0x9E8DF / 0x9E90E leads in
                let prop = if step == 11 { arg2 >> 8 } else { arg2 };
                let (word, city) = self.st.props.get(prop.max(0) as usize).map_or((0, 0), |p| (p.residence(), p.loc()));
                let src = match step {
                    0 => [super::subst(&super::subst(&super::subst(&text(0xAC74F), b"%2W", &t(&self.wform(assets, 2, word))), b"%d", &t(&dotted(arg))), b"%s", &t(self.loc(city))), text(0xAC7C6)].concat(),
                    10 => {
                        let find = assets.name_table((0xB62F0, 15)).get(arg as usize).cloned().unwrap_or_default();
                        let s = super::subst_seq(&text(0x9E83F), b"%s", &[t(&find), t(self.loc(city))]);
                        super::subst(&s, b"%4W", &t(&self.wform(assets, 4, word)))
                    }
                    _ => {
                        let lead = match arg2 {
                            ..0 => vec![],
                            _ if arg2 >> 4 & 1 == 0 => text(0x9E8DF),
                            _ => super::subst(&text(0x9E90E), b"%s", &t(self.loc(city))),
                        };
                        let head = if arg2 >= 0 { text(0x9E931) } else { text(0x9E7BD) };
                        [lead, super::subst(&head, b"%d", &t(&dotted(arg))), text(0xAC7C6)].concat()
                    }
                };
                f.inline(&src, vec![Action::Answer(0), Action::Answer(1)]);
            }
            Ask::Plot { kind: 42, arg, arg2, .. } => {
                // church seminar offer (rules-13 §2.10): EXE 0x892D7; arg2 = title | city << 4 | season << 8
                let title = assets.name_table((0xB4E98, 4)).get((arg2 & 15) as usize % 4).cloned().unwrap_or_default();
                let season = assets.name_table((0xB715C, 4)).get((arg2 >> 8 & 3) as usize).cloned().unwrap_or_default();
                let city = self.loc(((arg2 >> 4) & 15) as u8 % 10).to_string();
                let src = super::subst(&super::subst_seq(&assets.text(0x892D7), b"%s", &[t(&title), t(&season), t(&city)]), b"%d", &t(&dotted(arg)));
                f.inline(&src, vec![Action::Answer(0), Action::Answer(1)]);
            }
            Ask::Plot { kind: 48, arg, arg2, .. } => {
                // cart lease offer (rules-13 §2.16): EXE 0xA8E7E; arg = yearly payment, arg2 = n | place << 8 | branch << 16
                let place = assets.name_table((0xB6EA0, 3)).get(((arg2 >> 8) & 255) as usize % 3).cloned().unwrap_or_default();
                let city = self.st.props.get((arg2 >> 16) as usize).map_or(String::new(), |p| self.loc(p.loc()).to_string());
                let me = self.me().unwrap_or(0) as u8;
                let src = super::subst_seq(&assets.text(0xA8E7E), b"%s", &[t(&place), t(&city)]);
                let src = super::subst(&super::subst(&super::subst(&src, b"%1N1", &t(&self.pref(me, 1, 1))), b"%l", &t(&(self.st.year as i32 + 5).to_string())), b"%w", &t(&(arg2 & 255).to_string()));
                f.inline(&super::subst(&src, b"%d", &t(&dotted(arg))), vec![Action::Answer(0), Action::Answer(1)]);
            }
            Ask::Plot { kind: 49, step: 10, arg, arg2 } => {
                // regatta bribe (rules-13 §2.17): EXE 0xA575D with the official (arg) and his favour word 0xB6B28[arg2];
                // the amount (0..9999) is the answer
                let x = arg as u8;
                let word = assets.name_table((0xB6B28, 5)).get(arg2.clamp(0, 4) as usize).cloned().unwrap_or_default();
                let s = super::subst(&super::subst(&assets.text(0xA575D), b"%3N5", &t(&self.pref(x, 3, 5))), b"%1N0", &t(&self.pref(x, 1, 0)));
                let s = super::subst(&s, b"%s", &t(&word));
                let cut = s.windows(2).position(|w| w == b"%a").unwrap_or(s.len());
                f.raw(&s[..cut]);
                f.item(b"   ".to_vec(), Action::Answer(self.amount)).raw(b" ");
                f.digits(self.amount, 4, Field::Amount).txt(" Taler aufwenden!");
            }
            Ask::Plot { kind: 49, step, arg, .. } => {
                // regatta (rules-13 §2.17): EXE 0xA788B, 0xA79BE, 0xA7B39
                let src = match step {
                    0 => {
                        let tav = assets.name_table((0xB5AE4, 30)).get(arg as usize).cloned().unwrap_or_default();
                        super::subst(&assets.text(0xA788B), b"%s", &t(&tav))
                    }
                    1 => assets.text(0xA79BE),
                    _ => assets.text(0xA7B39),
                };
                f.inline(&src, vec![Action::Answer(0), Action::Answer(1)]);
            }
            Ask::Plot { kind: 43, step, arg, arg2 } => {
                // card game 17+4 (rules-13 §2.11): arg2 = x | counter << 8
                let x = (arg2 & 255) as u8;
                let tav = assets.name_table((0xB5AE4, 30)).get(arg as usize).cloned().unwrap_or_default();
                match step {
                    10 => {
                        // EXE 0xA278A (first game) / 0xAF8D4 (rematch); the stake is the answer, 0 declines
                        let first = (arg2 >> 8) & 255 == 1;
                        let head = assets.text(if first { 0xA278A } else { 0xAF8D4 });
                        let cut = head.windows(2).position(|w| w == b"%a").unwrap_or(head.len());
                        let head = super::subst(&super::subst(&head[..cut], b"%s", &t(&tav)), b"%3N5", &t(&self.pref(x, 3, 5)));
                        f.raw(&head).raw(b"$Z$L");
                        f.item(t("Ja sagen und"), Action::Answer(self.amount)).raw(b" ");
                        f.digits(self.amount, 4, Field::Amount).txt(" Taler setzen").raw(b"$N");
                        f.item(t("oder lieber ablehnen?"), Action::Answer(0));
                    }
                    11 => {
                        let src = super::subst(&assets.text(0xA28F1), b"%d", &t(&dotted(arg)));
                        f.inline(&src, vec![Action::Answer(0), Action::Answer(1)]);
                    }
                    _ => {
                        let src = if arg == 1 { super::subst(&assets.text(0xA29F3), b"%3N1", &t(&self.pref(x, 3, 1))) } else { super::subst(&assets.text(0xA2A49), b"%1N1", &t(&self.pref(x, 1, 1))) };
                        f.inline(&src, vec![Action::Answer(0), Action::Answer(1)]);
                    }
                }
            }
            Ask::Plot { kind: 44, arg, arg2, .. } => {
                // topple contract (rules-13 §2.12): EXE 0xA6B65; arg = reward, arg2 = target | figure << 8 | counter << 12
                let (x, figure, c) = ((arg2 & 255) as u8, ((arg2 >> 8) & 15) as usize, (arg2 >> 12) & 15);
                let me = self.me().unwrap_or(0) as u8;
                let fig = assets.name_table((0xB6C84, 3)).get(figure % 3).cloned().unwrap_or_default();
                let poss = assets.name_table((0xB6C90, 2)).get((self.st.persons[x as usize].gender() & 1) as usize).cloned().unwrap_or_default();
                let s = super::subst(&super::subst(&assets.text(0xA6B65), b"%1N0", &t(&self.pref(me, 1, 0))), b"%1N5", &t(&self.pref(x, 1, 5)));
                let s = super::subst_seq(&s, b"%s", &[t(&fig), t(&poss)]);
                let s = super::subst(&super::subst(&s, b"%l", &t(&(self.st.year as i32 + 3 + c).to_string())), b"%d", &t(&dotted(arg)));
                f.inline(&s, vec![Action::Answer(0), Action::Answer(1)]);
            }
            Ask::Plot { kind: 47, arg, arg2, .. } => {
                // sales contract (rules-13 §2.15): EXE 0xA7495; arg = penalty, arg2 = c | city << 3 | good << 7 | name << 11 | adj << 18 | Q << 20
                let (c, city, g, name, adj, q) = (arg2 & 7, (arg2 >> 3) & 15, ((arg2 >> 7) & 15) as u8, (arg2 >> 11) & 127, (arg2 >> 18) & 3, (arg2 >> 20) & 1023);
                let price = if (g as usize) < 15 { self.st.cities[(city as usize).min(9)].price(g as usize) as i32 } else { 0 };
                let y = q * price / 2;
                let adjective = assets.name_table((0xB6D44, 4)).get(adj as usize).cloned().unwrap_or_default();
                let first = assets.name_table((0xB4FB8, 117)).get(name as usize).cloned().unwrap_or_default();
                let cname = t(self.loc(city as u8));
                let s = super::subst_seq(&assets.text(0xA7495), b"%s", &[t(&adjective), t(&first), cname.clone(), cname, t(self.good(g))]);
                let s = super::subst(&s, b"%w", &t(&c.to_string()));
                let s = super::subst_seq(&s, b"%d", &[t(&q.to_string()), t(&y.to_string()), t(&arg.to_string())]);
                f.inline(&s, vec![Action::Answer(0), Action::Answer(1)]);
            }
            Ask::Plot { kind: 45, step, arg, arg2 } => {
                // wedding speech (rules-13 §2.13). Step 0: arg = x | county << 8. Step 10: arg = 3 fees (9 bits each),
                // arg2 = x | (rank | grade << 2) of the three monks (5 bits each) from bit 8
                let x = (if step == 0 { arg } else { arg2 } & 255) as u8;
                if step == 0 {
                    let me = self.me().unwrap_or(0) as u8;
                    let gword: &[u8] = if self.st.persons[me as usize].gender() == 1 { b" " } else { b"r " };
                    let euer = assets.text(if self.st.persons[x as usize].gender() == 0 { 0xB4270 } else { 0xB4274 });
                    let county = t(self.loc(((arg >> 8) & 255) as u8));
                    let s = super::subst(&super::subst(&assets.text(0xAADEC), b"%3N1", &t(&self.pref(x, 3, 1))), b"%1N0", &t(&self.pref(x, 1, 0)));
                    let name = t(&self.first_name(me as usize));
                    let s = super::subst_seq(&s, b"%s", &[county, gword.to_vec(), name, euer]);
                    f.inline(&super::subst(&s, b"%l", &t(&(self.st.year as i32 + 2).to_string())), vec![Action::Answer(0), Action::Answer(1)]);
                } else {
                    let grades = assets.name_table((0xB6AFC, 5));
                    let ranks = assets.name_table((0xB6B10, 3));
                    let mut s = super::subst(&assets.text(0xA544B), b"%4N1", &t(&self.pref(x, 4, 1)));
                    for k in 0..3 {
                        let look = (arg2 >> (8 + 5 * k)) & 31;
                        let (rank, grade) = ((look & 3) as usize, ((look >> 2) & 7) as usize);
                        let fee = (arg >> (9 * k)) & 511;
                        let piece = [t(grades.get(grade).map_or("", String::as_str)), t(ranks.get(rank).map_or("", String::as_str))];
                        s = super::subst_seq_first(&s, &piece, &fee.to_string());
                    }
                    f.inline(&s, (0..3).map(Action::Answer).collect());
                }
            }
            Ask::Plot { kind: 46, step, arg, arg2 } => {
                // peasant revolt (rules-13 §2.14): step 0 arg = x | county << 8; step 10 arg = mercenaries, arg2 = x | county << 8
                let packed = if step == 0 { arg } else { arg2 };
                let (x, county) = ((packed & 255) as u8, ((packed >> 8) & 3) as u8);
                if step == 0 {
                    let src = super::subst(&super::subst(&assets.text(0xAE05D), b"%3N5", &t(&self.pref(x, 3, 5))), b"%s", &t(self.loc(10 + county)));
                    f.inline(&src, vec![Action::Answer(0), Action::Answer(1)]);
                } else {
                    // EXE 0xAB114: "Wie viele wollt Ihr stellen?" with a 3-digit field (0 .. S)
                    let head = assets.text(0xAB114);
                    let cut = head.windows(2).position(|w| w == b"%a").unwrap_or(head.len());
                    f.raw(&super::subst(&head[..cut], b"%d", &t(&dotted(arg)))).raw(b"$Z$L");
                    f.digits(self.amount.min(arg as u32), 3, Field::Amount).txt(" Söldner!").raw(b"$N");
                    f.item(t("Entsenden"), Action::Answer(self.amount.min(arg as u32)));
                }
            }
            Ask::Plot { kind: 34, step: 10, arg, arg2 } => {
                // trade expedition (rules-13 §2.3): one window for carts and escort, both answers at once;
                // arg = free carts, arg2 = destination | W << 4 | M << 14
                let dest = assets.name_table((0xB6C44, 7)).get((arg2 & 15) as usize % 7).cloned().unwrap_or_default();
                let (w, m) = ((arg2 >> 4) & 1023, (arg2 >> 14) & 1023);
                let mut head = super::subst(&super::subst_seq(&assets.text(0xAA65D), b"%d", &[t(&w.to_string()), t(&m.to_string())]), b"%s", &t(&dest));
                let body = assets.text(0xAA710);
                let cut = body.windows(2).position(|w| w == b"%a").unwrap_or(body.len());
                head.extend(super::subst(&body[..cut], b"%d", &t(&dotted(arg))));
                f.raw(&head).raw(b"$Z$L");
                let (n, soldiers) = (self.amount.min(arg as u32), self.bribe);
                f.digits(n, 3, Field::Amount).txt(" Fuhrwerke mit ");
                f.digits(soldiers, 4, Field::Bribe).txt(&format!(" Söldnern als Eskorte. Eure Kosten: {} Taler.", 100 * n + 200 * soldiers)).raw(b"$Z$C");
                f.item(t("Losschicken"), Action::AnswerPair(n, soldiers));
            }
            Ask::Plot { kind: 34, .. } => {
                f.txt("");
            }
            Ask::Plot { kind: 39, step, arg, .. } => {
                // cartwright cartel (rules-13 §2.7): step 0 arg = city | figure << 4 | time << 6; step 1 arg = yearly price
                if step == 0 {
                    let city = t(self.loc((arg & 15) as u8 % 10));
                    let figure = assets.name_table((0xB4D58, 3)).get(((arg >> 4) & 3) as usize % 3).cloned().unwrap_or_default();
                    let time = assets.name_table((0xB4D4C, 3)).get(((arg >> 6) & 3) as usize % 3).cloned().unwrap_or_default();
                    let me = self.me().unwrap_or(0) as u8;
                    let s = super::subst_seq(&assets.text(0x88BB5), b"%s", &[city, t(&time), t(&figure)]);
                    f.inline(&super::subst(&s, b"%1N1", &t(&self.pref(me, 1, 1))), vec![Action::Answer(0), Action::Answer(1)]);
                } else {
                    // EXE 0x88E20: rent for 0..10 years (digit field) or shout "Betrüger!"
                    let head = assets.text(0x88E20);
                    let cut = head.windows(2).position(|w| w == b"%a").unwrap_or(head.len());
                    f.raw(&super::subst(&head[..cut], b"%d", &t(&dotted(arg)))).raw(b"$Z$L");
                    let y = self.amount.min(10);
                    f.item(t("Den Service"), Action::Answer(y)).raw(b" ");
                    f.digits(y, 2, Field::Amount).txt(if y == 1 { " Jahr" } else { " Jahre" }).txt(" mieten oder").raw(b"$N");
                    f.item(t("laut >Betrüger!< rufen"), Action::Answer(0));
                }
            }
            Ask::Plot { kind: 40, step, arg, arg2 } => {
                // scholar's lecture (rules-13 §2.8). Step 0 arg = profession | title << 4 | surname << 5 | city << 9 | reputation << 13 | name << 16
                let text = |a: u32| assets.text(a);
                match step {
                    0 => {
                        let nt = |addr: u32, n: usize, i: i32| t(assets.name_table((addr, n)).get(i as usize % n).map_or("", String::as_str));
                        let (prof, title, sur, city, rep, name) = (arg & 15, (arg >> 4) & 1, (arg >> 5) & 15, (arg >> 9) & 15, (arg >> 13) & 7, (arg >> 16) & 127);
                        let parts = [nt(0xB59F0, 7, rep), nt(0xB5A0C, 9, prof), nt(0xB5A30, 2, title), nt(0xB4FB8, 117, name), nt(0xB5A38, 10, sur), t(self.loc(city as u8 % 10))];
                        // the text starts "der %s %s %s %s %s in %s": reputation, profession, title, first name, surname, city
                        f.inline(&super::subst_seq(&text(0x97204), b"%s", &parts), vec![Action::Answer(0), Action::Answer(1)]);
                    }
                    10 => {
                        // led in by 0xA63D8 (city, title, surname, profession, hours, title) or, on a repeat, 0xA66AD
                        // (hours); arg = hours, arg2 = profession | title << 4 | surname << 5 | city << 9 | repeat << 13
                        let nt = |addr: u32, n: usize, i: i32| t(assets.name_table((addr, n)).get(i as usize % n).map_or("", String::as_str));
                        let lead = if arg2 >> 13 & 1 == 1 {
                            text(0xA66AD)
                        } else {
                            let title = nt(0xB5A30, 2, (arg2 >> 4) & 1);
                            super::subst_seq(&text(0xA63D8), b"%s", &[t(self.loc(((arg2 >> 9) & 15) as u8 % 10)), title.clone(), nt(0xB5A38, 10, (arg2 >> 5) & 15), nt(0xB5A0C, 9, arg2 & 15), title])
                        };
                        let src = [super::subst(&lead, b"%d", &t(&dotted(arg))), text(0xA6631)].concat();
                        f.inline(&src, (0..3).map(Action::Answer).collect());
                    }
                    11 => {
                        let g = t(self.good(arg as u8));
                        f.inline(&super::subst(&text(0xA6786), b"%s", &g), vec![Action::Answer(0), Action::Answer(1)]);
                    }
                    _ => {
                        // EXE 0xA6903: accept the offer A, or counter with the value of a 4-digit field
                        let head = text(0xA6903);
                        let cut = head.windows(2).position(|w| w == b"%a").unwrap_or(head.len());
                        f.raw(&super::subst(&super::subst(&head[..cut], b"%s", &t(self.good(arg2 as u8))), b"%d", &t(&dotted(arg)))).raw(b"$Z$L");
                        f.item(t(">Abgemacht!<"), Action::Answer(0)).raw(b"$N");
                        f.item(t(">Hm, wie wäre es mit"), Action::AnswerPair(1, self.amount)).raw(b" ");
                        f.digits(self.amount, 4, Field::Amount).txt(" Talern?<");
                    }
                }
            }
            Ask::Plot { kind: 37, arg, arg2, .. } => {
                // wager (rules-13 §2.5): intro 0x947A8 and the claim 0x9499C in one window.
                // arg2 = tavern | drink << 5 | hair << 8 | shoes << 11 | greeting << 14 | kind << 16 | byte5 << 18 | byte6 << 22
                let nt = |addr: u32, n: usize, i: i32| assets.name_table((addr, n)).get(i as usize % n).cloned().unwrap_or_default();
                let (tav, drink, hair, shoes, greet, kind) = (arg2 & 31, (arg2 >> 5) & 7, (arg2 >> 8) & 7, (arg2 >> 11) & 7, (arg2 >> 14) & 3, (arg2 >> 16) & 3);
                let (b5, b6) = ((arg2 >> 18) & 15, (arg2 >> 22) & 31);
                let intro = super::subst_seq(&assets.text(0x947A8), b"%s", &[t(&nt(0xB5AE4, 30, tav)), t(&nt(0xB55F8, 5, drink)), t(&nt(0xB560C, 7, hair)), t(&nt(0xB5628, 8, shoes))]);
                // the claim sub-format consumes (good, city, discipline): `%s` prints the next one, `%v` swallows it
                let args = [self.good(b6 as u8).to_string(), self.loc(b5 as u8 % 10).to_string(), nt(0xB5668, 8, b6)];
                let (mut claim, mut k) = (String::new(), 0);
                let fmt = nt(0xB565C, 3, kind);
                let mut it = fmt.chars().peekable();
                while let Some(c) = it.next() {
                    if c == '%' && matches!(it.peek(), Some('s' | 'v')) {
                        if it.next() == Some('s') {
                            claim.push_str(&args[k.min(2)]);
                        }
                        k += 1;
                    } else {
                        claim.push(c);
                    }
                }
                let line = super::subst_seq(&assets.text(0x9499C), b"%s", &[t(&nt(0xB564C, 4, greet)), t(&claim)]);
                let line = super::subst(&line, b"%d", &t(&dotted(arg)));
                f.inline(&[intro, line].concat(), vec![Action::Answer(0), Action::Answer(1)]);
            }
            Ask::Plot { kind: 33, step, arg, arg2 } => {
                // book publishing (rules-13 §2.2)
                let nt = |addr: u32, n: usize, i: i32| assets.name_table((addr, n)).get(i as usize % n).cloned().unwrap_or_default();
                let me = self.me().unwrap_or(0);
                match step {
                    0 => {
                        // EXE 0x9CA0E: arg = name | profession << 7 | dress << 12 | season << 13
                        let (name, prof, dress, season) = (arg & 127, (arg >> 7) & 31, (arg >> 12) & 1, (arg >> 13) & 3);
                        let s = super::subst_seq(&assets.text(0x9CA0E), b"%s", &[t(&nt(0xB6028, 4, season)), t(&nt(0xB6038, 2, dress)), t(&nt(0xB4FB8, 117, name)), t(&nt(0xB596C, 32, prof))]);
                        f.inline(&s, vec![Action::Answer(0), Action::Answer(1)]);
                    }
                    1 => {
                        // EXE 0x9CBE7: arg = price, arg2 = book | grade << 6 | place << 10 | name << 11
                        let (book, grade, place, name) = (arg2 & 63, (arg2 >> 6) & 15, (arg2 >> 10) & 1, (arg2 >> 11) & 127);
                        let parts = [t(&self.st.name_str(self.st.persons[me].name() as usize)), t(&nt(0xB6048, 2, place)), t(&nt(0xB5530, 49, book)), t(&nt(0xB6050, 11, grade)), t(&nt(0xB4FB8, 117, name))];
                        let s = super::subst(&super::subst_seq(&assets.text(0x9CBE7), b"%s", &parts), b"%d", &t(&dotted(arg)));
                        f.inline(&s, vec![Action::Answer(0), Action::Answer(1)]);
                    }
                    10 => {
                        // EXE 0xA308B and the offer format 0xA3124; arg = three qualities (4 bits each), arg2 = book | grade << 6
                        let (book, grade) = (arg2 & 63, (arg2 >> 6) & 15);
                        let mut s = super::subst_seq(&assets.text(0xA308B), b"%s", &[t(&nt(0xB5530, 49, book)), t(&nt(0xB6050, 11, grade))]);
                        for k in 0..3 {
                            let q = (arg >> (4 * k)) & 15;
                            let line = super::subst_seq(&assets.text(0xA3124), b"%s", &[
                                t(&nt(0xB68BC, 2, q >> 3 & 1)), t(&nt(0xB68A4, 2, q >> 2 & 1)), t(&nt(0xB68AC, 2, q >> 1 & 1)), t(&nt(0xB68B4, 2, q & 1)),
                            ]);
                            let line = super::subst(&line, b"%d", &t(&dotted(q + 5)));
                            s = super::subst_seq_first_one(&s, &line);
                        }
                        f.inline(&s, (0..3).map(Action::Answer).collect());
                    }
                    _ => {
                        // EXE 0xA31DA: number of copies, 0..999
                        let head = assets.text(0xA31DA);
                        let cut = head.windows(2).position(|w| w == b"%a").unwrap_or(head.len());
                        f.raw(&head[..cut]).raw(b"$Z$L");
                        f.digits(self.amount.min(999), 3, Field::Amount).txt(" Stück ").raw(b"$N");
                        f.item(t("Drucken lassen"), Action::Answer(self.amount.min(999)));
                    }
                }
            }
            Ask::Plot { kind: 38, step, arg, arg2 } => {
                // robbers' raid on a count's silver (rules-13 §2.6)
                let me = self.me().unwrap_or(0) as u8;
                let text = |a: u32| assets.text(a);
                match step {
                    0 => {
                        // EXE 0xA0DDB: arg = x | look << 8, arg2 = camp property
                        let x = (arg & 255) as u8;
                        let look = assets.name_table((0xB6658, 3)).get(((arg >> 8) & 3) as usize % 3).cloned().unwrap_or_default();
                        let gword: &[u8] = if self.st.persons[me as usize].gender() == 1 { b" " } else { b"r " };
                        let camp = self.st.props.get(arg2.max(0) as usize).map_or(String::new(), |p| self.loc(p.loc()).to_string());
                        let s = super::subst(&super::subst(&text(0xA0DDB), b"%1N1", &t(&self.pref(x, 1, 1))), b"%1N0", &t(&self.pref(me, 1, 0)));
                        let s = super::subst_seq(&s, b"%s", &[t(&look), gword.to_vec(), t(&camp)]);
                        f.inline(&s, vec![Action::Answer(0), Action::Answer(1)]);
                    }
                    1 => {
                        // EXE 0xA1106: bribe amount (digit row) or throw him out; both answers at once
                        let head = text(0xA1106);
                        let cut = head.windows(2).position(|w| w == b"%a").unwrap_or(head.len());
                        f.raw(&head[..cut]).raw(b"$Z$L");
                        f.item(t("den Mann mit"), Action::AnswerPair(self.amount, 0)).raw(b" ");
                        f.digits(self.amount, 5, Field::Amount).txt(" Talern bestechen oder").raw(b"$N");
                        f.item(t("ihn an die frische Luft setzen?"), Action::AnswerPair(0, 1));
                    }
                    11 => {
                        // EXE 0x885C6: the city guard at the door; submit or flee
                        let s = super::subst(&text(0x885C6), b"%1N0", &t(&self.pref(me, 1, 0)));
                        f.inline(&s, vec![Action::Answer(0), Action::Answer(1)]);
                    }
                    10 => {
                        // EXE 0x889DF: how many robbers, 0..min(T, 99); arg2 = the count's first-name index
                        let first = assets.name_table((0xB4FB8, 117)).get(arg2 as usize % 117).cloned().unwrap_or_default();
                        let head = text(0x889DF);
                        let cut = head.windows(2).position(|w| w == b"%a").unwrap_or(head.len());
                        f.raw(&super::subst(&head[..cut], b"%s", &t(&first))).raw(b"$Z$L");
                        f.item(t("Mit"), Action::Answer(self.amount.min(arg as u32))).raw(b" ");
                        f.digits(self.amount.min(arg as u32), 2, Field::Amount).txt(" Räubern!");
                    }
                    _ => {
                        f.txt("");
                    }
                }
            }
            Ask::Plot { kind: 36, step, arg, arg2 } => {
                // concert (rules-13 §2.4)
                let nt = |addr: u32, n: usize, i: i32| assets.name_table((addr, n)).get(i.max(0) as usize % n).cloned().unwrap_or_default();
                let me = self.me().unwrap_or(0) as u8;
                let female = self.st.persons[me as usize].gender() == 1;
                if step == 0 {
                    // EXE 0x9FBE9: arg = host | R << 8 | (quality + 16) << 16 | place << 22
                    let (x, r, q, place) = ((arg & 255) as u8, (arg >> 8) & 255, ((arg >> 16) & 63) - 16, (arg >> 22) & 15);
                    let gword: &[u8] = if female { b" " } else { b"r " };
                    let job = t(&nt(0xB6498, 2, female as i32));
                    let s = super::subst(&super::subst(&super::subst(&assets.text(0x9FBE9), b"%3N5", &t(&self.pref(x, 3, 5))), b"%1N1", &t(&self.pref(me, 1, 1))), b"%1N2", &t(&self.pref(x, 1, 2)));
                    let s = super::subst_seq(&s, b"%s", &[gword.to_vec(), job, t(self.loc(place as u8 % 10)), t(&nt(0xB646C, 11, q.clamp(0, 10)))]);
                    let s = super::subst(&super::subst(&s, b"%l", &t(&(self.st.year as i32 + 2).to_string())), b"%d", &t(&dotted(r)));
                    f.inline(&s, vec![Action::Answer(0), Action::Answer(1)]);
                } else {
                    // EXE 0xAF6CF: arg = n0 | n1 << 7 | n2 << 14 | host << 21, arg2 = (s_i + 8) 5 bits each | tavern groups from bit 15
                    let x = ((arg >> 21) & 255) as u8;
                    let tav = [nt(0xB5AE4, 30, (arg2 >> 15) & 15), nt(0xB5AE4, 30, 10 + ((arg2 >> 19) & 15)), nt(0xB5AE4, 30, 20 + ((arg2 >> 23) & 15))];
                    let mut s = super::subst(&assets.text(0xAF6CF), b"%3N1", &t(&self.pref(x, 3, 1)));
                    s = super::subst_seq(&s, b"%s", &[t(&tav[0]), t(&tav[1]), t(&tav[2])]);
                    for k in 0..3 {
                        let n = (arg >> (7 * k)) & 127;
                        let sk = ((arg2 >> (5 * k)) & 31) - 8;
                        let price = (30 - sk) * n;
                        s = super::subst_offer(&s, n, &t(&nt(0xB646C, 11, sk.clamp(0, 10))), price);
                    }
                    f.inline(&s, (0..3).map(Action::Answer).collect());
                }
            }
            Ask::ContestEntry { person, fee, goods, paper } => {
                // newspaper contest entry (rules-13 §3.1): EXE 0x9A0C8, 0x9A2B4; a toggle line per discipline, then "Fertig"
                let paper = assets.name_table((0xB5D34, 6)).get(paper as usize % 6).cloned().unwrap_or_default();
                let head = super::subst(&super::subst(&assets.text(0x9A0C8), b"%s", &t(&paper)), b"%d", &t(&dotted(fee)));
                f.raw(&head).raw(&super::subst(&assets.text(0x9A2B4), b"%1N1", &t(&self.pref(person, 1, 1))));
                // the lines are 0x9A2EA "%a>%s %s<" (superlative, good; empty past the 10-entry table) and 0x9A2F6
                // "%aFertig": gem bullets, a chosen discipline's gem lit (multi-select, rules-13 §3.1; playtest report)
                let superl = assets.name_table((0xB5D58, 10));
                let label = |src: Vec<u8>| src.windows(2).position(|w| w == b"%a").map_or(src.clone(), |i| src[i + 2..].to_vec()).strip_suffix(b"$|").map(<[u8]>::to_vec).unwrap_or_default();
                for (d, &g) in goods.iter().enumerate() {
                    let line = super::subst_seq(&assets.text(0x9A2EA), b"%s", &[t(superl.get(g as usize).map_or("", String::as_str)), t(self.good(g))]);
                    f.item_bullet_raw(&label(line), Action::Step(Field::Mask, 1 << d)).raw(b"$N");
                }
                f.item_bullet_raw(&label(assets.text(0x9A2F6)), Action::Answer(self.amount & 31));
            }
            Ask::AttendAuction { person } => {
                // EXE 0xBB0A8 "Euer Bargeld" with a cash line per human (0xBB0C0 / 0xBB0D8), then 0xBB100 Ja/Nein
                f.raw(&assets.text(0xBB0A8));
                for h in self.st.humans() {
                    let cash = self.st.persons[h].cash();
                    let line = assets.text(if cash < 0 { 0xBB0D8 } else { 0xBB0C0 });
                    f.raw(&super::subst(&super::subst(&line, b"%1N0", &t(&self.pref(h as u8, 1, 0))), b"%d", &t(&dotted(cash.abs()))));
                }
                let _ = person;
                f.raw(&assets.text(0xBB100));
                f.bind(assets.text(super::addr::JA), Action::Answer(0)).bind(assets.text(super::addr::NEIN), Action::Answer(1));
            }
            Ask::AuctionTick { price, high, .. } => {
                // one tick of the auction (rules-12 §3.5): the price line, then who presses his hotkey; 0 = nobody
                let line = match high {
                    Some(b) => super::subst(&super::subst(&assets.text(0xBADAC), b"%1N3", &t(&self.pref(b, 1, 3))), b"%d", &t(&dotted(price))),
                    None => super::subst(&assets.text_id(3370), b"%d", &t(&dotted(price))),
                };
                // idle ticks speak one of the auctioneer's calls (0228900..0228915, rules-14 §3), the closing hammer lines 916..918
                if high.is_none() {
                    f.raw(&super::speech(228900 + (price as u32 / 5) % 16));
                }
                // no items: the hall runs by itself and a human bids with his hotkey (rules-08 §1.4, `Game::auction_input`)
                f.raw(b"$C").raw(&line).raw(b"$Z");
                // ours (`prefs.skip_lot`): the lot is resolved without the human (`Game::auction_input`)
                if assets.prefs.borrow().skip_lot {
                    let p = assets.prefs.borrow();
                    f.raw(b"$C").item(t(p.tr("Tab: Posten überspringen", "Tab: skip this lot")), Action::SkipLot).raw(b"$N");
                }
            }
            Ask::Building { city, prices } => {
                // rules-38 §2.3: "%aEin%s für %d Taler.$|" per building, then "%aNichts!"; answer = the building
                let mut acts = vec![];
                let mut src = vec![];
                for (k, p) in prices.iter().enumerate() {
                    src.extend(super::subst(&super::subst(&assets.text_id(2526), b"%s", &assets.text_id(2520 + k as u32)), b"%d", &t(&dotted(*p))));
                    acts.push(Action::Answer(k as u32));
                }
                src.extend(assets.text_id(2527));
                acts.push(Action::Answer(5));
                let _ = city;
                f.inline(&src, acts);
            }
            Ask::AuctionRefused { person, text, city } => {
                // a refused hotkey bid (rules-15 §1.4 step 5): message box (140, 160, 360, 160), closed by a click
                let src = super::subst(&super::subst(&assets.text(text), b"%1N0", &t(&self.pref(person, 1, 0))), b"%o", &t(self.loc(city)));
                f.raw(&src);
            }
            Ask::ResidenceSwap { branch, new_word } => {
                // EXE 0xBB14C: which of the two residences to sell; 0 sells the old one (keeps the won one)
                let me = self.me().unwrap_or(0) as u8;
                let b = branch as usize;
                let city = self.st.props[b].loc();
                let (old, oldv) = (self.st.props[b].residence(), self.st.residence_value_of(b));
                let newv = self.st.residence_value_word(city as usize, new_word);
                let s = super::subst(&super::subst(&assets.text(0xBB14C), b"%1N0", &t(&self.pref(me, 1, 0))), b"%o", &t(self.loc(city)));
                let s = super::subst_seq(&s, b"%3W", &[t(&self.wform(assets, 3, old)), t(&self.wform(assets, 3, new_word))]);
                let s = super::subst_seq(&s, b"%d", &[t(&oldv.to_string()), t(&newv.to_string())]);
                f.inline(&s, vec![Action::Answer(0), Action::Answer(1)]);
            }
            Ask::BlackmailVerdict { by, judge } => {
                // EXE 0xBB334: the blackmailer may sway the vote; Schuldig! (0) / Nicht schuldig. (1)
                let src = super::subst(&super::subst(&assets.text(0xBB334), b"%1N0", &t(&self.pref(by, 1, 0))), b"%4N5", &t(&self.pref(judge, 4, 5)));
                f.inline(&super::subst(&src, b"$g", b""), vec![Action::Answer(0), Action::Answer(1)]);
            }
            Ask::BuyCastle { prop, price } => {
                // EXE 0xC1D4C (rules-12 §3.5): a computer owner offers his castle; 0 buys, 1 declines
                let me = self.me().unwrap_or(0) as u8;
                let p = &self.st.props[prop as usize];
                let s = super::subst(&super::subst(&assets.text(0xC1D4C), b"%1N0", &t(&self.pref(me, 1, 0))), b"%1N1", &t(&self.pref(p.owner(), 1, 1)));
                f.inline(&super::subst(&super::subst(&s, b"%o", &t(self.loc(p.loc()))), b"%d", &t(&dotted(price))), vec![Action::Answer(0), Action::Answer(1)]);
            }
            Ask::Apologise { target } => {
                let src = super::subst(&assets.text(0xBCC10), b"%4N1", &t(&self.pref(target, 4, 1)));
                f.inline(&src, vec![Action::Answer(0), Action::Answer(1)]);
            }
            Ask::AcceptDuel { insulter, insulted, rolls: [taunt, voice, reason, place] } => {
                // the insult, spoken in the insulter's voice (rules-24 §1, rules-39 §2): a computer insulter's box 0x9B7D2
                // (occasion, reason), a human insulter's 0x9B99E (place)
                let tab = |a: u32, n: usize, k: u8| assets.text_table((a, n)).get(k as usize).cloned().unwrap_or_default();
                let said = super::voice([1 + voice as u32, self.st.persons[insulter as usize].gender() as u32 & 1, 2, 1], 11 + taunt as u32);
                let tn = tab(0xB5434, 12, taunt);
                let s = if self.st.persons[insulter as usize].human() {
                    let s = super::subst(&super::subst(&assets.text(0x9B99E), b"%4N0", &t(&self.pref(insulted, 4, 0))), b"%1N0", &t(&self.pref(insulted, 1, 0)));
                    super::subst_seq(&s, b"%s", &[tab(0xB5EF8, 6, place), tn])
                } else {
                    let s = super::subst(&assets.text(0x9B7D2), b"%1N5", &t(&self.pref(insulter, 1, 5)));
                    let s = super::subst(&super::subst(&s, b"%4N0", &t(&self.pref(insulted, 4, 0))), b"%1N0", &t(&self.pref(insulted, 1, 0)));
                    super::subst_seq(&s, b"%s", &[tab(0xB5EF8, 6, place), tab(0xB5F10, 8, reason), tn])
                };
                f.inline(&[said, s].concat(), vec![Action::Answer(0), Action::Answer(1)]);
            }
            Ask::ConfirmCastleSale { buyer } => {
                // EXE 0xBF4C0 (rules-23 §4.2)
                let s = super::subst(&assets.text(0xBF4C0), b"%1N0", &t(&self.pref(buyer, 1, 0)));
                f.inline(&s, vec![Action::Answer(0), Action::Answer(1)]);
            }
            Ask::AuctionCatalog { .. } => {
                f.txt("");
            }
            Ask::AuctionLine { step, kind, seller, winner, a, b, c, .. } => {
                // hall lines (rules-15 §1.4): the opening caption on top, the other lines in the text area at the bottom
                let text = match step {
                    0 => super::subst(&super::subst(&assets.text(0xBAE00), b"%l", &t(&a.to_string())), b"%o", &t(self.loc(b as u8 % 10))),
                    1 => match kind {
                        12 => super::subst(&super::subst(&assets.text_id(3100), b"%4u", &Game::u_form(assets, a, 19 + b as u32)), b"%3N5", &t(&self.pref(seller, 3, 5))),
                        14 => super::subst(&super::subst(&super::subst(&assets.text_id(3101), b"%5W", &t(&self.wform(assets, 5, a as u16))), b"%o", &t(self.loc(c as u8))), b"%3N5", &t(&self.pref(seller, 3, 5))),
                        _ => super::subst(&assets.text_id(3102), b"%s", &t(self.loc(c as u8))),
                    },
                    2 => t(&self.sold_line(assets, winner, kind, c as u8, a as u8, b as u8, a as u16)),
                    3 => assets.text(0xBADC4),
                    _ => [super::speech(228919), assets.text(if a == 1 { 0xBB1D0 } else { 0xBB228 })].concat(),
                };
                f.raw(b"$C").raw(&super::subst(&super::subst(&text, b"$g", b""), b"$15H", b""));
            }
            Ask::Plea => {
                // EXE 0x99E8D "was wollt Ihr sagen?" with one cycling item over the nine pleas of table 0xB5C9C
                // (rules-18 §4 step 5); a click on the upper half goes on, on the lower half back
                let me = self.me().unwrap_or(0) as u8;
                let head = assets.text(0x99E8D);
                let cut = head.windows(2).position(|w| w == b"%a").unwrap_or(head.len());
                f.raw(&super::subst(&super::subst(&head[..cut], b"$g", b""), b"%1N0", &t(&self.pref(me, 1, 0)))).raw(b"$Z$L");
                let plea = assets.text_table((0xB5C9C, 9)).get(self.amount as usize).cloned().unwrap_or_default();
                f.item(plea, Action::Step(Field::Choice(9, false), 1)).raw(b"$Z$C").item(t("Das sage ich!"), Action::Answer(self.amount));
            }
            Ask::LeaderLost { .. } => {
                f.raw(&assets.text(0xBB66C)).raw(b"$Z$C").item(t("Weiter"), Action::Answer(0));
            }
            Ask::ContinueAfterMission { .. } => {
                // EXE 0xBA858 "Wollt Ihr weiterspielen?": Ja continues (0), Nein ends the game (1)
                f.raw(&assets.text(0xBA858));
                f.bind(assets.text(super::addr::JA), Action::Answer(0)).bind(assets.text(super::addr::NEIN), Action::Answer(1));
            }
            Ask::OccupyCastle { code, attacker, loc, .. } => {
                // rules-59 §2: 0xBBE08 (castle) / 0xBBF4C (camp) with (A, P+1), on the page after the verdict
                let src = self.exe_fmt(assets, &assets.text(if code == 4 { 0xBBE08 } else { 0xBBF4C }), &[mad_sim::WArg::P(attacker), mad_sim::WArg::Loc(loc)]);
                let src = src.strip_prefix(b"$W$P").map(<[u8]>::to_vec).unwrap_or(src);
                f.inline(&src, vec![Action::Answer(0), Action::Answer(1)]);
            }
            Ask::LoanOffer { lender, amount, rate, years } => {
                // EXE 0xC04F0: lender, lender's place, amount, rate, year due (live capture)
                let s = super::subst(&assets.text(0xC04F0), b"%1N3", &t(&self.pref(lender, 1, 3)));
                let s = super::subst(&s, b"%o", &t(self.loc(self.st.persons[lender as usize].loc())));
                let s = super::subst_seq(&s, b"%d", &[t(&dotted(amount.max(0) as u32)), t(&rate.to_string())]);
                let s = super::subst(&s, b"%l", &t(&(self.st.year as u32 + years as u32).to_string()));
                f.inline(&s, vec![Action::Answer(0), Action::Answer(1)]);
            }
            Ask::PickDirector { ref candidates, ref fees } => {
                // rules-34 §3.1 (EXE 0xBEED8..): bullets per candidate, then the hire icon; a click on a
                // candidate hires at once (the original selects first, then hires with the icon)
                f.raw(&assets.text(0xBEED8));
                if candidates.is_empty() {
                    f.raw(&assets.text(0xBEEF8).iter().copied().skip_while(|&b| b != b'$').collect::<Vec<_>>());
                } else {
                    f.raw(&assets.text(0xBEF3C));
                    f.gems = true;
                    for (k, (&c, &fee)) in candidates.iter().zip(fees).enumerate() {
                        // `%a` = bullet at margin 12, text from margin 36 (rules-34 §3.1); item labels are raw, so no `%%`
                        let label: Vec<u8> = format!("{} für {fee}% des Einsatzes", self.pref(c, 1, 3)).chars().filter_map(super::cp437).collect();
                        f.bullet(&label, Action::Select(k as u32)).raw(b"$N$|");
                    }
                    let hire = (0..candidates.len() as u32).map(Action::Answer).collect();
                    f.raw(b"$Z$L").icon_item("Direktor einstellen", Action::WithSel(hire, false)).raw(b"$N");
                }
            }
            _ => {
                f.txt(&format!("{q:?}")).raw(b"$Z$C").item(t("Weiter"), Action::Answer(0));
            }
        }
    }

    /// The courtship gift menu text (EXE 0x963C3; gift k names from GIFT_NAMES[k], rules-56 §1).
    pub(super) fn gift_text(&self, assets: &Assets, target: u8, prices: [i32; 3], names: [u8; 3]) -> Vec<u8> {
        let gifts: Vec<Vec<u8>> = (0..3).map(|k| assets.text(GIFT_NAMES[k][names[k] as usize % GIFT_NAMES[k].len()])).collect();
        let src = super::subst(&assets.text(0x963C3), b"%3N5", &t(&self.pref(target, 3, 5)));
        let src = super::subst_seq(&src, b"%s", &gifts);
        super::subst_seq(&src, b"%d", &prices.map(|p| p.to_string().into_bytes()))
    }

    /// Codes and names of the privileges the current player holds (rules-04b §4.1).
    /// The person whose privileges the menu shows.
    pub(super) fn priv_holder_of(&self) -> usize {
        if self.priv_holder == 0xFF { self.me().unwrap_or(0) } else { self.priv_holder as usize }
    }

    /// Cities the privilege dialog `code` may pick: all (9), unbesieged (28), besieged by someone else (29).
    pub(super) fn priv_cities(&self, code: u8) -> Vec<u8> {
        let me = self.me().unwrap_or(0);
        let sieges = self.st.heap.all(25, mad_sim::heap::ANY, mad_sim::heap::ANY);
        let besieged = |c: u8, others: bool| sieges.iter().any(|&i| self.st.heap.recs[i][4] == c && (!others || self.st.heap.recs[i][1] as usize != me));
        (0..10).filter(|&c| match code { 28 => !besieged(c, false), 29 => besieged(c, true), _ => true }).collect()
    }

    /// What a privilege entry's bullet does (rules-38 §2): an information box, a dialog, or the action.
    pub(super) fn priv_action(&self, assets: &Assets, holder: usize, c: u8) -> Action {
        let Some(h) = self.me() else { return Action::Nothing };
        let tid = |n: u32| assets.text_id(n);
        let name_id = u32::from_le_bytes(assets.exe_bytes(0x86778 + 4 * c as u32, 4).try_into().unwrap());
        let loc = self.st.persons[holder].loc();
        let rank = t(&self.rank(h));
        let place = t(self.loc(loc));
        let castle = tid(2003 + loc as u32);
        let g = |a: usize| self.st.globals.b(a);
        // the player's rank, then the place (`%v` swallows it)
        let info = |n: u32, rect: [i32; 4]| {
            let s = super::subst(&tid(n), b"%1N4", &rank);
            let s = super::subst(&super::subst(&s, b"%v", b""), b"%s", &place);
            Action::Boxed(s, rect)
        };
        match c {
            1 => Action::Medicus,
            // rules-12 §5: one already → box 2404 with (" " / "n ", Mätresse / Geliebten), nothing else
            4 if self.st.heap.has(29, h as u8) => {
                let g = (self.st.persons[h].gender() & 1) as u32;
                Action::Boxed(super::subst_seq(&tid(2404), b"%s", &[t([" ", "n "][g as usize]), tid(2402 + g)]), [160, 140, 320, 200])
            }
            4 => Action::Sim(Command::TakeMistress, None),
            6 => Action::Open(Menu::Testament),
            8 => Action::Open(Menu::Feast([1, 2, 0, 1])),
            12 => Action::Open(Menu::Targets(Plot::Trial)),
            26 => Action::Open(Menu::Targets(Plot::Insult)),
            10 => Action::Boxed(super::subst(&super::subst(&tid(3179), b"%1N4", &rank), b"%d", &t(&dotted(self.st.office_income(h)))), [160, 160, 320, 160]),
            11 => Action::Boxed(super::subst(&tid(3181), b"%1N4", &rank), [160, 160, 320, 160]),
            13 => Action::Boxed(tid(2716 + self.st.persons[h].certificate().min(2) as u32), [140, 140, 360, 200]),
            14..=17 => info(name_id + 2, [140, 140, 360, 200]),
            22 => Action::Boxed(super::subst(&super::subst(&super::subst(&tid(3114), b"%1N4", &rank), b"%s", &place), b"%w", &t(&g(0x85A22).to_string())), [140, 120, 360, 240]),
            31 => info(3120, [160, 140, 320, 200]),
            32 => Action::Boxed(super::subst(&super::subst(&tid(3123), b"%1N4", &rank), b"%s", &castle), [160, 140, 320, 200]),
            41 => Action::Boxed(super::subst(&super::subst(&tid(name_id + 1), b"%1N4", &rank), b"%s", &castle), [160, 160, 320, 160]),
            51 => Action::Boxed(super::subst(&super::subst(&super::subst(&tid(3129), b"%1N4", &rank), b"%s", &place), b"%w", &t(&g(0x85A41).to_string())), [140, 120, 360, 240]),
            61 => info(2740, [160, 140, 320, 200]),
            64 => info(2980, [160, 140, 320, 200]),
            71 => info(2874, [160, 160, 320, 160]),
            18..=21 | 23 | 24 | 30 | 33..=39 | 42..=47 | 49 | 50 | 52 | 57 | 60 | 62 | 63 | 65..=68 | 70 | 72 | 73 | 75..=78 | 83 => info(name_id + 1, [140, 120, 360, 240]),
            9 => Action::Open(Menu::PrivCity(9)),
            25 => Action::Open(Menu::Insurance),
            27 => Action::Priv(27, 0, 0),
            28 | 29 => {
                // rules-38 §2.6, 2.7: refusals before the city pick
                let sieges = self.st.heap.all(25, mad_sim::heap::ANY, mad_sim::heap::ANY);
                let own = sieges.iter().filter(|&&i| self.st.heap.recs[i][1] as usize == h).count();
                let others = sieges.iter().any(|&i| self.st.heap.recs[i][1] as usize != h);
                match c {
                    28 if sieges.len() >= 10 => Action::Boxed(tid(2156), [160, 140, 320, 200]),
                    28 if own >= 3 => Action::Boxed(tid(2144), [140, 180, 360, 120]),
                    29 if !others => Action::Boxed(tid(2483), [160, 160, 320, 160]),
                    _ => Action::Open(Menu::PrivCity(c)),
                }
            }
            40 | 69 => Action::Open(Menu::PrivAsk(c)),
            48 | 74 | 79 | 53 | 54 | 84 | 55 | 86 | 80 | 81 | 85 => Action::PrivView(c),
            56 | 59 | 82 => Action::Open(Menu::LawEdit([56, 59, 82].iter().position(|&x| x == c).unwrap_or(0) as u8)),
            58 => Action::Open(Menu::Project),
            _ => Action::Nothing,
        }
    }

    fn targets(&self, h: usize) -> Vec<usize> {
        let mut v: Vec<usize> = self
            .st
            .props_of(h, mad_sim::state::KIND_BRANCH)
            .flat_map(|b| {
                let c = self.st.props[b].loc() as usize;
                (0..10).filter_map(move |i| self.st.holder(c, i))
            })
            .collect();
        v.extend(self.st.humans().filter(|&p| p != h));
        v.dedup();
        v.retain(|&p| p != h);
        v
    }

    pub(super) fn step(&mut self, assets: &Assets, f: Field, d: i32) {
        if self.me().is_none() { return }
        match f {
            Field::Bribe => self.bribe = (self.bribe as i32 + d).clamp(0, 9_999_999) as u32,
            Field::PrivCity(code) => {
                let cs = self.priv_cities(code);
                if let Some(k) = cs.iter().position(|&c| c == self.priv_city).or(Some(0)) {
                    self.priv_city = cs.get((k as i32 + d.signum()).rem_euclid(cs.len().max(1) as i32) as usize).copied().unwrap_or(0);
                }
            }
            Field::PrivN(lo, hi) => self.priv_n = (self.priv_n as i64 + d as i64).clamp(lo as i64, hi as i64) as u32,
            Field::Law(book, i) => self.sim(assets, Command::EditLaw { book, i, delta: d }),
            Field::Priv(code) => {
                let me = self.me().unwrap_or(0);
                let holder = if self.priv_holder == 0xFF { me } else { self.priv_holder as usize };
                self.sim(assets, Command::Privilege { holder: holder as u8, code, a: d, b: 0 });
            }
            Field::Budget(b) => {
                let pr = &self.st.props[b];
                let budget = (pr.budget() as i64 + d as i64).clamp(0, 999_999) as u32;
                self.sim(assets, Command::SetBudget { branch: b as u8, budget });
            }
            Field::Amount => self.amount = (self.amount as i32 + d).clamp(0, 999_999) as u32,
            Field::Choice(n, skip3) => {
                let n = n as i32;
                let mut v = (self.amount as i32 + n + d.signum()) % n;
                if skip3 && v == 3 {
                    v = (v + n + d.signum()) % n;
                }
                self.amount = v as u32;
            }
            Field::Mask => self.amount ^= d.unsigned_abs() & 31,
            Field::Toll(p) => {
                let rate = self.st.castles[self.st.props[p].loc() as usize - 14].toll_rate as i32;
                self.sim(assets, Command::TollRate { prop: p as u8, rate: (rate + d).clamp(0, 14) as u8 });
            }
            Field::Order(b, k, of) => self.edit_order(assets, b, k, of, d),
            Field::Amt(k, max) => self.amts[k as usize] = (self.amts[k as usize] as i32 + d).clamp(0, max as i32) as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Popup, tests::{ctx_for, test_assets}};
    use super::*;

    /// Every menu opens on a fresh game, and the pay digits step through the sim.
    #[test]
    fn real_menus_open_and_pay() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, mad_sim::State::new_game(5, 3, &[mad_sim::world::HumanSetup::test(3)]).unwrap());
        g.popups.clear();
        let h = g.me().unwrap();
        let b = g.st.branch_at(h, 3).unwrap();
        for m in [
            Menu::Spielstand, Menu::Save, Menu::Load, Menu::Quit, Menu::City(0), Menu::Storage(b), Menu::Transport(b, 0),
            Menu::Applicants(3, 6), Menu::Tree(Plot::Trial, 3, false), Menu::Listener(0), Menu::BribeAmount(0),
            Menu::Candles, Menu::Alms, Menu::Pray, Menu::JoinChurch, Menu::Convert, Menu::Leave, Menu::Indulgence, Menu::Feast([1, 2, 0, 1]), Menu::Testament, Menu::Laws(0), Menu::Laws(1), Menu::Laws(2), Menu::Directors, Menu::Residence(b), Menu::ResBuy(b), Menu::ResBuild(b),
        ] {
            g.open(&assets, m);
            assert!(matches!(g.popups.back(), Some(Popup::Choice { menu: Some(x), .. }) if *x == m), "{m:?}");
        }
        // rules-64 §2: a fresh player has no vacant promotion: the "keine Ämter" box, no menu
        g.open(&assets, Menu::Apply);
        assert!(matches!(g.popups.back(), Some(Popup::Choice { src, menu: None, .. }) if src.windows(9).any(|w| w == b"Derzeit s")));
        // without a director the window is the hiring question (rules-34 §3.1)
        g.open(&assets, Menu::Director(b));
        assert!(g.popups.iter().any(|p| matches!(p, Popup::Choice { menu: Some(Menu::Question), .. })));
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        g.popups.clear();
        g.pending = None;
        g.act(Action::Sim(Command::BuildTransporters { branch: b as u8, t: 0, n: 1 }, None), &mut ctx);
        assert!(g.st.persons[h].cash() < 750);
    }
}
