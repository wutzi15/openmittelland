//! Branch screen (spec 04 §5.1.2, `formats.md` §11.3): stone-wall background, goods grid, the order
//! panel (0,278,640,120) with editable fields, and the six icons of the bottom row. Positions are
//! checked against the original (docs/FIDELITY.md); the gold price markers over the goods are a
//! placeholder (the original's bead-and-arc sprites are not identified).

use super::{Action, Field, Form, Game, Menu, OrderField, Window, hit, t};
use crate::{assets::Assets, gfx::Frame, input::Cursor, screens::{Ctx, Next}, text};
use mad_sim::{
    Command,
    economy::{CAP, ST_CONFIRMED, ST_IMPOSSIBLE, ST_POSSIBLE},
    state::{CITIES, GOODS, TRANSPORTERS},
};

/// The order panel (§11.3).
const RECT: [i32; 4] = [0, 278, 640, 120];
/// Stone wall behind the screen (§11.3).
const WALL: u16 = 0x31;
/// Goods icons 48x48, normal `32+g` / dark `41+g` (`formats.md` §1.7).
const GOOD_ICON: u16 = 0x32;
const GOOD_ICON_DARK: u16 = 0x41;
/// `ICON50` warehouse (§1.7), the "sell everything" icon left of the goods (spec 04 §5.1.2.1).
const WAREHOUSE: u16 = 0x50;
pub(super) const TRANSPORTER_NAMES: [&str; TRANSPORTERS] = ["Karren", "Schiffe", "Ballons", "Dampfkarren", "Tauchboote", "Flugmaschinen"];
/// Control icons (§1.7 catalogue, identified by eye): residence `51`, storage `5B`, director's seal
/// ring `5C`, wagon `5D`, ship `5E`, balloon `5F`.
const CONTROLS: [u16; 6] = [0x51, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F];
const CONTROL_Y: i32 = 408;
/// Order rows shown (rules-02 §3.1: two rows, the record has room for four).
const ROWS: u8 = 2;

type GoodSlot = (u8, [i32; 4], [i32; 4]);

fn control_rect(i: usize) -> [i32; 4] {
    [38 + 100 * i as i32, CONTROL_Y, 64, 64]
}

/// Abbreviations of the order book (handbook appendix III, H p.37).
pub(super) fn abbrev(name: &str) -> &str {
    const ABBR: [(&str, &str); 11] = [
        ("F\u{fc}rstenbr\u{fc}ck", "F\u{fc}rstenbr."),
        ("Finsterwalde", "Finsterw."),
        ("Lichterheide", "Lichterh."),
        ("Greifenheim", "Greifenh."),
        ("Falkenhorst", "Falkenh."),
        ("Lauenburg", "Lauenb."),
        ("Dampfkarren", "Dampfk."),
        ("Tauchboot", "Tauchb."),
        ("Flugmaschine", "Flugm."),
        ("Tauchboote", "Tauchb."),
        ("Flugmaschinen", "Flugm."),
    ];
    ABBR.iter().find(|(full, _)| *full == name).map_or(name, |&(_, short)| short)
}

/// Stock as the original prints it: thousands separated by a dot, three digits after it ("0.000").
fn stock_text(n: u32) -> String {
    format!("{}.{:03}", n / 1000, n % 1000)
}

/// Ships per `qty` of capacity `CAP[t]`, at least one (rules-02 §4.1: the UI sets it).
fn count_for(qty: u16, tr: u8) -> u8 {
    let cap = CAP[tr as usize % TRANSPORTERS] as u32;
    (qty as u32).div_ceil(cap).clamp(1, 255) as u8
}

/// The next element of `list` after (or before, `d < 0`) `cur`; the first if `cur` is absent.
fn cycle(cur: u8, list: &[u8], d: i32) -> u8 {
    match list.iter().position(|&x| x == cur) {
        Some(i) => list[(i as i32 + d.signum()).rem_euclid(list.len() as i32) as usize],
        None => list.first().copied().unwrap_or(cur),
    }
}


/// The escort mark of a transport row: inline icon `ICONR6` (crossed swords, 18×18), rules-36 §1.
const ESCORT_ICON: u16 = 0x1B6;
/// Its reserved width as a non-clickable item.
const ESCORT_ITEM: [u8; 2] = [crate::text::PAD_MARK, 18];
/// Its top relative to the item (line) top.
const ESCORT_DY: i32 = 0;

impl Game {
    pub(super) fn city_goods(&self, b: usize) -> Vec<u8> {
        let c = self.st.props[b].loc() as usize;
        (0..GOODS).filter(|&g| c < CITIES && self.st.cities[c].yield_(g) >= 101).map(|g| g as u8).collect()
    }

    /// Valid destinations: other cities for sell/deliver, castles and camps for loot.
    fn dests(&self, b: usize, kind: u8) -> Vec<u8> {
        let c = self.st.props[b].loc();
        if kind >= 6 { (14..21).collect() } else { (0..CITIES as u8).filter(|&d| d != c).collect() }
    }

    /// rules-36 §4.1: step the kind by `d`, skipping kinds the branch cannot use, and fill the new kind's
    /// defaults while keeping the other bytes.
    fn next_kind(&self, b: usize, mut o: [u8; 8], d: i32) -> [u8; 8] {
        let pr = &self.st.props[b];
        let (c, owner) = (pr.loc() as usize, pr.owner() as usize);
        let stock_total: u32 = (0..GOODS).map(|g| pr.stock(g) as u32).sum();
        let trans_total: u32 = (0..TRANSPORTERS).map(|t| pr.trans(t) as u32).sum();
        let branches = self.st.props_of(owner, mad_sim::state::KIND_BRANCH).count();
        let forts = self.st.props_of(owner, mad_sim::state::KIND_CASTLE).count() + self.st.props_of(owner, mad_sim::state::KIND_CAMP).count();
        let ok = |k: u8| match k {
            0 | 1 => true,
            2 | 3 => trans_total > 0 && stock_total > 0,
            4 | 5 => trans_total > 0 && stock_total > 0 && branches >= 2,
            _ => trans_total > 0 && forts > 0,
        };
        let mut k = o[0];
        for _ in 0..8 {
            k = ((k as i32 + 8 + d) % 8) as u8;
            if ok(k) {
                break;
            }
        }
        let ship = |t: u8| mad_sim::economy::needs_water(t as usize);
        match k {
            1 => o[3] = (0..GOODS as u8).find(|&g| self.st.cities[c].yield_(g as usize) > 100).unwrap_or(0),
            2..=7 => {
                if k <= 5 {
                    if o[3] as usize >= GOODS {
                        o[3] = 0;
                    }
                    for _ in 0..GOODS {
                        if pr.stock(o[3] as usize) != 0 {
                            break;
                        }
                        o[3] = (o[3] + 1) % GOODS as u8;
                    }
                }
                if o[5] as usize >= TRANSPORTERS {
                    o[5] = 0;
                }
                for _ in 0..TRANSPORTERS {
                    if pr.trans(o[5] as usize) != 0 {
                        break;
                    }
                    o[5] = (o[5] + 1) % TRANSPORTERS as u8;
                }
                if k <= 5 {
                    if o[6] as usize >= CITIES {
                        o[6] = 0;
                    }
                    for _ in 0..CITIES {
                        let bad = o[6] as usize == c || (ship(o[5]) && !mad_sim::economy::navigable(o[6] as usize)) || (k >= 4 && self.st.branch_at(owner, o[6] as usize).is_none());
                        if !bad {
                            break;
                        }
                        o[6] = (o[6] + 1) % CITIES as u8;
                    }
                } else {
                    if !(14..21).contains(&o[6]) {
                        o[6] = 14;
                    }
                    for _ in 0..7 {
                        let own = (0..mad_sim::state::PROPS).any(|q| self.st.props[q].loc() == o[6] && self.st.props[q].owner() as usize == owner && matches!(self.st.props[q].kind(), mad_sim::state::KIND_CASTLE | mad_sim::state::KIND_CAMP));
                        if o[6] as usize != c && own {
                            break;
                        }
                        o[6] = (o[6] - 13) % 7 + 14;
                    }
                }
            }
            _ => {}
        }
        o[0] = k;
        o
    }

    pub(super) fn edit_order(&mut self, assets: &Assets, b: usize, k: u8, f: OrderField, d: i32) {
        if matches!(f, OrderField::TroopKind | OrderField::TroopTarget | OrderField::Troop(_)) {
            return self.edit_troop_order(assets, b, k, f, d);
        }
        let mut o = self.st.props[b].order(k as usize);
        let qty = u16::from_le_bytes([o[1], o[2]]);
        let set_qty = |o: &mut [u8; 8], q: u16| {
            [o[1], o[2]] = q.to_le_bytes();
            if o[0] < 6 {
                o[4] = count_for(q, o[5]);
            }
        };
        match f {
            OrderField::Kind => o = self.next_kind(b, o, d.signum()),
            OrderField::Good => {
                let list: Vec<u8> = if o[0] == 1 { self.city_goods(b) } else { (0..GOODS as u8).collect() };
                o[3] = cycle(o[3], &list, d);
            }
            // rules-36 §3: production workers 0..40 (house rule: up to 255); a transport count follows from the quantity
            OrderField::Workers if o[0] == 1 || o[0] >= 6 => o[4] = (o[4] as i32 + d).clamp(0, if o[0] == 1 { self.st.house.max_workers as i32 } else { 99 }) as u8,
            OrderField::Workers => {}
            OrderField::Means => o[5] = (o[5] as i32 + d).clamp(0, 99) as u8,
            OrderField::Qty => {
                // rules-36 §3: at most what the transporters carry and what is in stock (both with this
                // slot's reservation when confirmed)
                let pr = &self.st.props[b];
                let r = (o[7] == ST_CONFIRMED) as u32;
                let t = o[5] as usize % TRANSPORTERS;
                let cap = CAP[t] as u32 * (pr.trans(t) as u32 + r * o[4] as u32);
                let have = pr.stock(o[3] as usize % GOODS) as u32 + r * qty as u32;
                set_qty(&mut o, (qty as i32 + d).clamp(0, cap.min(have).min(9999) as i32) as u16)
            }
            OrderField::Trans => {
                o[5] = cycle(o[5], &(0..TRANSPORTERS as u8).collect::<Vec<_>>(), d);
                set_qty(&mut o, qty);
            }
            OrderField::Dest => o[6] = cycle(o[6], &self.dests(b, o[0]), d),
            OrderField::TroopKind | OrderField::TroopTarget | OrderField::Troop(_) => unreachable!(),
        }
        self.sim(assets, Command::SetOrder { branch: b as u8, slot: k, order: o });
    }

    /// One order line as clickable fields, in the handbook's sentence order (H p.12-14). The verb
    /// per good ("Braut", "Fangt", …) is not documented; ours says "Produziert".
    fn order_line(&self, f: &mut Form, b: usize, k: u8) {
        let o = self.st.props[b].order(k as usize);
        let fld = |x| Field::Order(b, k, x);
        let word = |f: &mut Form, s: &str, x| {
            f.item(t(s), Action::Step(fld(x), 1));
        };
        match o[0] {
            // Row layout from the EXE's order-panel format (`0xBE64C`): verb + good right-aligned to x = 172,
            // "mit NN Arbeitern" from x = 180, the preposition centred in 372..414, means from x = 420.
            // An empty slot is the centred verb "Kein Auftrag" (capture of a new game, Kronsbach).
            0 => {
                // rules-36 §2: one wide item, the text padded with 32 spaces on each side
                f.raw(b"$4<$C");
                word(f, &format!("{0}Kein Auftrag{0}", " ".repeat(32)), OrderField::Kind);
                f.raw(b"$E$|$L");
            }
            1 => {
                let g = o[3] as usize;
                let tb = &self.tables;
                f.raw(b"$78<$R");
                word(f, tb.verbs.get(g).map_or("Produziert", String::as_str), OrderField::Kind);
                f.raw(b" ");
                word(f, self.good(o[3]), OrderField::Good);
                f.raw(b"$E$|$30>$L").txt("mit ");
                // a third digit only under the house rule (the row shifts by one digit then)
                f.digits(o[4] as u32, if o[4] > 99 { 3 } else { 2 }, fld(OrderField::Workers));
                f.txt(" Arbeitern").raw(b"$E$|$62>$7=$C").txt(tb.preps.get(g).map_or("an", String::as_str)).raw(b"$E$|$70>$L");
                f.digits(o[5] as u32, 2, fld(OrderField::Means));
                f.raw(b" ").txt(tb.means_pl.get(g).map_or("", String::as_str)).raw(b"$E$|$|$L");
            }
            kind => {
                // Columns of the EXE's transport formats (rules-36 §1: `%m %m$E$30>$14=$C%m$E$|$44>$Lauf %m %m in %m`,
                // `…mit %m %m n. %m`, loot `$78<$R%m%M%M$E$|$30>$Lmit %m %m aus %m`); kind names `0xB64E0`.
                // Live A/B of kinds 2 and 3 (2026-11-17, `mkscene … 2 5`).
                let (verb, prep, with) = match kind {
                    2 | 3 => ("Verkauft", "in", "auf"),
                    4 | 5 => ("Bringt", "n.", "mit"),
                    _ => ("Holt Beute", "aus", "mit"),
                };
                if kind < 6 {
                    // Columns as captured for the sell row (new game, Bergstedt): verb and the dotted
                    // quantity from the left, the good centred over 180..264, then "auf N Karren in Ziel".
                    // Escorted kinds 3 and 5 print `%m %J %m`: the verb, the crossed swords icon 0x1B6 (an inline
                    // icon of its own width, 18 px), the quantity (rules-36 §1).
                    let verb = if kind == 3 { "Verk.".to_string() } else { verb.to_string() };
                    f.raw(b"$L");
                    word(f, &verb, OrderField::Kind);
                    f.raw(b" ");
                    if kind % 2 == 1 {
                        f.item(ESCORT_ITEM.to_vec(), super::Action::Nothing).raw(b" ");
                    }
                    let qty = u16::from_le_bytes([o[1], o[2]]) as u32;
                    f.digits(qty, 4, fld(OrderField::Qty));
                    f.raw(b"$E$|$30>$14=$C");
                    word(f, self.good(o[3]), OrderField::Good);
                    // the count is a 2-digit item without padding: it reserves 32 px (formats §8.6)
                    let n = o[4].to_string();
                    f.raw(b"$E$|$44>$L").txt(&format!("{with} {n}")).raw(format!("${}}}", (32 - 15 * n.len() as i32).max(0)).as_bytes()).txt(" ");
                } else {
                    f.raw(b"$78<$R");
                    word(f, verb, OrderField::Kind);
                    f.raw(b"$E$|$30>$L").txt(&format!("{with} ")).digits(o[4] as u32, 2, fld(OrderField::Workers)).raw(b" ");
                }
                word(f, abbrev(TRANSPORTER_NAMES[o[5] as usize % TRANSPORTERS]), OrderField::Trans);
                f.txt(&format!(" {prep} "));
                word(f, abbrev(self.loc(o[6])), OrderField::Dest);
                f.raw(b"$E$|");
            }
        }
        // Status symbol (H p.12): the font's crossed circle (confirmed), plain circle (possible) and
        // broken circle (impossible), right aligned.
        let glyph = match o[7] {
            ST_CONFIRMED => Some(0x0F),
            ST_POSSIBLE => Some(0x0E),
            ST_IMPOSSIBLE => Some(0x0D),
            _ => None,
        };
        if let (true, Some(gl)) = (o[0] != 0, glyph) {
            f.raw(if o[0] == 1 { b"$R".as_slice() } else { b"$E$R" }).item(vec![gl], Action::ToggleOrder(b, k)).raw(b"$L");
        }
    }

    /// Cost of the confirmed or possible orders as the panel prints it: wages and means of a
    /// production order, trips of a transport order (rules-02 §3, §4).
    pub(super) fn order_cost(&self, b: usize) -> i32 {
        let pr = &self.st.props[b];
        let (h, c) = (pr.owner() as usize, pr.loc() as usize);
        (0..ROWS)
            .map(|k| {
                let o = pr.order(k as usize);
                // only confirmed lines cost (A/B: unconfirming the line shows "Kosten: 0 Taler")
                if o[7] != mad_sim::economy::ST_CONFIRMED {
                    return 0;
                }
                match o[0] {
                    1 => self.st.order_forecast(h, c, o[3] as usize % GOODS, o[4] as i32, o[5] as i32),
                    // rules-36 §7: escorted kinds 3 and 5 cost half again, loot 7 likewise
                    2..=5 => (mad_sim::economy::TRIP_COST[o[5] as usize % TRANSPORTERS] * o[4] as i32 * (2 + (o[0] & 1) as i32)) >> 1,
                    6 | 7 => mad_sim::economy::TRIP_COST[o[5] as usize % TRANSPORTERS] * o[4] as i32 * (2 + (o[0] == 7) as i32) / 2,
                    _ => 0,
                }
            })
            .sum()
    }

    /// Auslastung in percent.
    fn utilisation(&self, b: usize) -> i32 {
        // rules-36 §7: stock over the storage capacity (17 units per m² of wood and stone, +1)
        let pr = &self.st.props[b];
        let cap = (pr.wood() as u32 + pr.stone() as u32) * 17 + 1;
        ((0..GOODS).map(|g| pr.stock(g) as u32).sum::<u32>() * 100 / cap) as i32
    }

    /// `help`: ours, the production help's opener between the two bottom numbers (`prefs.production_help`).
    fn branch_form(&self, b: usize, help: bool) -> Form {
        let mut f = Form::new(b"$C");
        for k in 0..ROWS {
            if !matches!(self.st.props[b].order(k as usize)[0], 0 | 1) {
                f.raw(b"$C");
            }
            self.order_line(&mut f, b, k);
            f.raw(b"$N");
        }
        f.raw(b"$Z$L").txt(&format!("Auslastung: {}%", self.utilisation(b)));
        if help {
            f.raw(b"$E$C").item(t("Hilfe (H)"), Action::Open(Menu::ProdHelp(b)));
        }
        f.raw(b"$E$R").txt(&format!("Kosten: {} Taler", self.order_cost(b))).raw(b"$N");
        f
    }

    pub(super) fn branch_window(&self, assets: &Assets, b: usize, lit: &[usize]) -> (Window, Vec<super::Action>) {
        let f = self.branch_form(b, assets.prefs.borrow().production_help);
        let refs: Vec<&[u8]> = f.items.iter().map(Vec::as_slice).collect();
        (Window::new(assets, RECT, &f.src, &refs, &[], lit).with_text_dy(6), f.acts)
    }

    /// Centres of the goods slots (§11.3): the warehouse first, then the goods of the city.
    fn slots(&self, n: usize) -> Vec<(i32, i32)> {
        if n <= 7 {
            return (0..n as i32).map(|k| (80 + 80 * k, 176)).collect();
        }
        let rest = n as i32 - 7;
        (0..n as i32).map(|k| if k < 7 { (80 + 80 * k, 125) } else { ((640 - rest * 64) / 2 + 32 + 64 * (k - 7), 227) }).collect()
    }

    /// Goods row geometry: (good, icon rect, stock-number rect), plus the warehouse icon.
    fn goods_row(&self, b: usize) -> (Vec<GoodSlot>, [i32; 4]) {
        // formats §11.3: the city's own goods, then every other good the branch holds (a stolen order book's
        // product, deliveries; playtest: leather in Lichterheide could not be sold)
        let mut goods = self.city_goods(b);
        let pr = &self.st.props[b];
        goods.extend((0..GOODS as u8).filter(|&g| !goods.contains(&g) && pr.stock(g as usize) > 0).collect::<Vec<_>>());
        let pos = self.slots(goods.len() + 1);
        let row = goods.iter().zip(&pos[1..]).map(|(&g, &(cx, cy))| (g, [cx - 24, cy - 24, 48, 48], [cx - 24, cy + 22, 48, 24]));
        (row.collect(), [pos[0].0 - 24, pos[0].1 - 24, 48, 48])
    }

    pub(super) fn click_branch(&mut self, ctx: &mut Ctx, b: usize, pos: (i32, i32)) -> Next {
        let (w, acts) = self.branch_window(ctx.assets, b, &[]);
        if w.item_at(pos).is_some() {
            return self.click_form(ctx, &w, &acts, pos);
        }
        // Local sales (spec 04 §5.1.2.1): stock number = one unit, goods icon = all of that good,
        // warehouse icon = everything.
        let (row, house) = self.goods_row(b);
        let sell = if hit(house, pos) {
            Some(Action::Sell(b, 0xFF, 0))
        } else {
            row.iter().find_map(|&(g, icon, num)| if hit(icon, pos) { Some(Action::Sell(b, g, 0)) } else if hit(num, pos) { Some(Action::Sell(b, g, 1)) } else { None })
        };
        if let Some(a) = sell {
            return self.act_keep(a, ctx);
        }
        match (0..6).find(|&i| hit(control_rect(i), pos)) {
            Some(0) => self.open(ctx.assets, Menu::Residence(b)),
            Some(1 | 2) | Some(3..) if { self.amts = [0; 4]; false } => {}
            Some(1) => self.open(ctx.assets, Menu::Storage(b)),
            Some(2) => self.open(ctx.assets, Menu::Director(b)),
            Some(i) => self.open(ctx.assets, Menu::Transport(b, i as u8 - 3)),
            None => {}
        }
        Next::Stay
    }

    pub(super) fn draw_branch(&self, frame: &mut Frame, assets: &Assets, b: usize, mouse: (i32, i32)) -> Cursor {
        let pr = &self.st.props[b];
        let c = pr.loc() as usize;
        frame.blit(&assets.icon(WALL)[0], 0, 0);
        self.draw_banner(frame, assets, &t(self.loc(pr.loc())), true);
        let (w, acts) = self.branch_window(assets, b, &[]);
        // a number spinner lights as a whole (A/B): every digit item of the hovered field
        let hover: Vec<usize> = match w.item_at(mouse).and_then(|k| acts.get(k)) {
            Some(Action::Step(f, _)) => acts.iter().enumerate().filter(|(_, a)| matches!(a, Action::Step(g, _) if g == f)).map(|(k, _)| k).collect(),
            _ => w.item_at(mouse).into_iter().collect(),
        };
        let w = self.branch_window(assets, b, &hover).0;
        w.draw(frame, assets);
        // the escort icons (`%J` 0x1B6) of kinds 3 and 5, at their reserved items
        for (k, _) in self.branch_form(b, assets.prefs.borrow().production_help).items.iter().enumerate().filter(|(_, it)| it.as_slice() == ESCORT_ITEM) {
            let r = w.item_rect(k);
            frame.blit(&assets.icon(ESCORT_ICON)[0], r.x, r.y + ESCORT_DY);
        }
        let (row, house) = self.goods_row(b);
        frame.blit(&assets.icon(WAREHOUSE)[0], house[0], house[1]);
        let mut cursor = None;
        for &(g, icon_r, num) in &row {
            let (cx, cy) = (icon_r[0] + 24, icon_r[1] + 24);
            // rules-33: dark when g / 5 exceeds the higher of owner's and director's certificate level
            let level = [Some(pr.owner()), Some(pr.director()).filter(|&d| d != mad_sim::state::NONE)].into_iter().flatten().map(|p| self.st.persons[p as usize].certificate()).max().unwrap_or(0);
            let dark = g / 5 > level;
            let icon = if dark { GOOD_ICON_DARK } else { GOOD_ICON } + g as u16;
            frame.blit(&assets.icon(icon)[0], icon_r[0], icon_r[1]);
            small_digits(frame, assets, &self.st.cities[c].price(g as usize).to_string(), cx + 1, cy - 41);
            small_digits(frame, assets, &stock_text(pr.stock(g as usize) as u32), cx, cy + 27);
            // rules-33: the skill gauge, font glyph 0x80 + k (arc and beads baked in), after the price
            let k = ((self.st.effective_skill(b, g as usize) - 50) / 10).clamp(0, 9) as u8;
            crate::text::blit_glyph(frame, &assets.font, 0x80 + k, cx - 18, cy - 48);
            if hit(num, mouse) {
                cursor = Some(Cursor::Minus);
            }
        }
        let numbers = [None, Some(pr.wood() as u32 + pr.stone() as u32), None, Some(pr.trans(0) as u32), Some(pr.trans(1) as u32), Some(pr.trans(2) as u32)];
        for (i, n) in numbers.iter().enumerate() {
            let [x, y, ..] = control_rect(i);
            let id = if i == 0 { CONTROLS[0] + if pr.has_residence() { (pr.residence() & 0x1F).min(9) } else { 0 } } else { CONTROLS[i] };
            frame.blit(&assets.icon(id)[0], x, y);
            if let Some(n) = n {
                // a two-digit field: one digit sits a digit's width further right (live A/B, Bergstedt branch)
                let pad = if *n < 10 { 6 } else { 0 };
                small_digits(frame, assets, &n.to_string(), x + 21 + pad, y + 4);
            }
        }
        cursor.unwrap_or_else(|| Self::form_cursor(&w, &acts, mouse))
    }
}


/// Small gold digits (font codes 0x10..0x19; the dot is 0x1F), centred on x.
fn small_digits(frame: &mut Frame, assets: &Assets, s: &str, cx: i32, y: i32) {
    let g: Vec<u8> = s.bytes().map(|c| if c == b'.' { 0x1F } else { c - b'0' + 0x10 }).collect();
    let w: i32 = g.iter().map(|&c| assets.font.advance[c as usize] as i32).sum();
    let mut x = cx - w / 2;
    for c in g {
        text::blit_glyph(frame, &assets.font, c, x, y);
        x += assets.font.advance[c as usize] as i32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helpers() {
        assert_eq!((count_for(100, 0), count_for(101, 0), count_for(0, 0), count_for(1500, 1)), (1, 2, 1, 2));
        assert_eq!((cycle(3, &[1, 3, 5], 1), cycle(5, &[1, 3, 5], 1), cycle(1, &[1, 3, 5], -1), cycle(9, &[1, 3], 1)), (5, 1, 5, 1));
    }

    /// Editing the uncle's production order through the fields reaches the sim.
    #[test]
    fn real_order_editing() {
        let Some(assets) = super::super::tests::test_assets() else { return };
        let mut g = Game::start(&assets, mad_sim::State::new_game(3, 3, &[mad_sim::world::HumanSetup::test(3)]).unwrap());
        let h = g.me().unwrap();
        let b = g.st.branch_at(h, 3).unwrap();
        let before = g.st.props[b].order(0);
        g.edit_order(&assets, b, 0, OrderField::Workers, 1);
        assert_eq!(g.st.props[b].order(0)[4], before[4] + 1);
        g.edit_order(&assets, b, 1, OrderField::Kind, 1);
        assert_eq!(g.st.props[b].order(1)[0], 1);
        // without stock a sell order is skipped (rules-36 §4.1): back to "Kein Auftrag"
        g.edit_order(&assets, b, 1, OrderField::Kind, 1);
        assert_eq!(g.st.props[b].order(1)[0], 0);
        g.st.props[b].set_stock(3, 300);
        g.edit_order(&assets, b, 1, OrderField::Kind, 1);
        g.edit_order(&assets, b, 1, OrderField::Kind, 1);
        let o = g.st.props[b].order(1);
        assert_eq!((o[0], o[6] != 3), (2, true), "sell order with another city as destination");
        g.edit_order(&assets, b, 1, OrderField::Qty, 100);
        assert_eq!((u16::from_le_bytes([g.st.props[b].order(1)[1], g.st.props[b].order(1)[2]]), g.st.props[b].order(1)[4]), (100, 1));
    }

    /// formats §11.3: a good the city does not produce shows after the city's goods once the branch holds some, so it
    /// can be sold (playtest: a stolen order book's leather had no icon).
    #[test]
    fn real_goods_row_shows_foreign_stock() {
        let Some(assets) = super::super::tests::test_assets() else { return };
        let mut g = Game::start(&assets, mad_sim::State::new_game(3, 3, &[mad_sim::world::HumanSetup::test(3)]).unwrap());
        let b = g.st.branch_at(g.me().unwrap(), 3).unwrap();
        let own = g.city_goods(b);
        let foreign = (0..GOODS as u8).find(|x| !own.contains(x)).unwrap();
        assert!(!g.goods_row(b).0.iter().any(|s| s.0 == foreign));
        g.st.props[b].set_stock(foreign as usize, 1039);
        let row = g.goods_row(b).0;
        assert_eq!(row.iter().map(|s| s.0).collect::<Vec<_>>(), [own, vec![foreign]].concat());
    }

    /// With `MAD_SHOT_DIR`, writes our Kronsbach branch screen (diff against a capture with refdiff).
    #[test]
    fn real_branch_shot() {
        use crate::screens::Screen;
        let Some(assets) = super::super::tests::test_assets() else { return };
        let Ok(dir) = std::env::var("MAD_SHOT_DIR") else { return };
        let mut g = Game::start(&assets, mad_sim::State::new_game(3, 3, &[mad_sim::world::HumanSetup::test(3)]).unwrap());
        g.popups.clear();
        let h = g.me().unwrap();
        let b = g.st.branch_at(h, 3).unwrap();
        let good = g.city_goods(b)[0];
        g.st.props[b].set_order(0, mad_sim::economy::production_order(good, 10, 2, mad_sim::economy::ST_CONFIRMED));
        g.branch = Some(b);
        let mut frame = crate::gfx::Frame::new(assets.palette);
        g.draw(&mut frame, &assets, (630, 470));
        crate::gfx::write_screenshot(&format!("{dir}/branch.png"), &frame).unwrap();
    }

    /// With `MAD_SHOT_DIR`, writes our sell rows (kinds 2 and 3, empty quantity) for comparison with captures.
    #[test]
    fn real_sell_rows_shot() {
        use crate::screens::Screen;
        let Some(assets) = super::super::tests::test_assets() else { return };
        let Ok(dir) = std::env::var("MAD_SHOT_DIR") else { return };
        let mut g = Game::start(&assets, mad_sim::State::new_game(3, 3, &[mad_sim::world::HumanSetup::test(3)]).unwrap());
        g.popups.clear();
        let h = g.me().unwrap();
        let b = g.st.branch_at(h, 3).unwrap();
        let wool = g.tables.goods.iter().position(|n| n == "Wolle").unwrap() as u8;
        let dest = g.tables.locations.iter().position(|n| n.starts_with("F\u{fc}rstenbr")).unwrap() as u8;
        g.st.props[b].set_order(0, mad_sim::economy::production_order(wool, 4, 8, mad_sim::economy::ST_CONFIRMED));
        g.branch = Some(b);
        for k in [2u8, 3] {
            g.st.props[b].set_order(1, [k, 0, 0, wool, 0, 0, dest, 0]);
            let mut frame = crate::gfx::Frame::new(assets.palette);
            g.draw(&mut frame, &assets, (320, 215));
            crate::gfx::write_screenshot(&format!("{dir}/sell{k}.png"), &frame).unwrap();
        }
    }
}
