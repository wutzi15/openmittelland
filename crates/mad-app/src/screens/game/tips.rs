//! Ours, both default off and not in any game file: tooltips (`prefs.tooltips`, a label of what a click would do
//! after the pointer rested) and keyboard shortcuts (`prefs.shortcuts`: Kontor keys to the places, Space / Enter
//! end the turn, 1-9 pick a menu line, `?` lists the keys).

use super::{Action, BANNER_RECT, GOLD_RECT, Game, Menu, Opt, Popup, SPOTS, Spot, branch, hit, rooms::Room};
use crate::{
    assets::Assets,
    gfx::Frame,
    input::Key,
    prefs::Prefs,
    screens::Ctx,
    text,
};
use mad_sim::Command;

/// Rest time of the pointer before a tooltip shows.
pub(super) const TIP_MS: u32 = 600;

/// Where a Kontor key goes.
#[derive(Clone, Copy)]
enum Go {
    Spot(Spot),
    Banner,
    Kreditbuch,
}

/// Kontor keys (lower case) and what they open. B, P, V, U, H, L, C, E and Tab belong to other Extras.
const KEYS: [(u8, Go, (&str, &str)); 9] = [
    (b'm', Go::Spot(Spot::Handel), ("Handelskarte", "Trade map")),
    (b's', Go::Spot(Spot::Schreibstube), ("Schreibstube", "Scriptorium")),
    (b'r', Go::Spot(Spot::Spielstand), ("Spielstand (Rangliste)", "Standings")),
    (b'g', Go::Spot(Spot::Bezahlung), ("Bezahlung (Löhne)", "Wages")),
    (b'z', Go::Spot(Spot::Hinterzimmer), ("Hinterzimmer", "Back room")),
    (b'k', Go::Spot(Spot::Kirche), ("Kirche", "Church")),
    (b't', Go::Spot(Spot::Truppen), ("Söldner & Räuber", "Mercenaries & robbers")),
    (b'd', Go::Kreditbuch, ("Kreditbuch", "Debt book")),
    (b'o', Go::Banner, ("Spieloptionen", "Game options")),
];

/// The rows of the Tasten window: the Kontor keys, the fixed ones, then the other Extras' keys that are on.
pub(super) fn key_list(p: &Prefs) -> Vec<(String, String)> {
    let tr = |de: &'static str, en: &'static str| p.tr(de, en).to_string();
    let mut v: Vec<(String, String)> = KEYS.iter().map(|&(k, _, (de, en))| ((k as char).to_ascii_uppercase().to_string(), tr(de, en))).collect();
    v.push((tr("Leer, Enter", "Space, Enter"), tr("Runde beenden", "End the turn")));
    v.push(("Esc".into(), tr("Fenster schließen, Raum verlassen", "Close window, leave room")));
    v.push(("1 - 9".into(), tr("Zeile eines Menüs wählen", "Pick a menu line")));
    v.push(("?".into(), tr("Diese Liste", "This list")));
    v.push(("Tab".into(), tr("Extras", "Extras")));
    for (on, k, de, en) in [(p.ledger, "B", "Bilanz", "Ledger"), (p.price_table, "P", "Preistabelle", "Price table"), (p.sell_all, "V", "Alles verkaufen", "Sell all"), (p.undo, "U", "Letzte Runde zurück", "Undo the last turn"), (p.production_help, "H", "Produktionshilfe (Niederlassung)", "Production help (branch)")] {
        if on {
            v.push((k.into(), tr(de, en)));
        }
    }
    v
}

/// Index of the n-th choice that is not a number field.
pub(super) fn nth_bullet(acts: &[Action], n: usize) -> Option<usize> {
    acts.iter().enumerate().filter(|(_, a)| !matches!(a, Action::Step(..))).nth(n).map(|(k, _)| k)
}

fn key_hint(p: &Prefs, s: String, key: Option<u8>) -> String {
    match key.filter(|_| p.shortcuts) {
        Some(k) => format!("{s} ({})", (k as char).to_ascii_uppercase()),
        None => s,
    }
}

impl Game {
    /// A shortcut key with no window open; `true` when it was one.
    pub(super) fn shortcut(&mut self, ctx: &mut Ctx, k: Key) -> bool {
        let kontor = self.room.is_none() && self.branch.is_none();
        match k {
            Key::Char(b'?') => self.open(ctx.assets, Menu::Keys),
            // the turn end takes the right click's path, confirmation included
            Key::Enter | Key::Char(b' ') if kontor => match self.end_turn_dialog(ctx.assets) {
                Some(p) => self.popups.push_back(p),
                None => drop(self.act(Action::EndTurn, ctx)),
            },
            Key::Char(c) if kontor => {
                let Some(&(_, go, _)) = KEYS.iter().find(|k| k.0 == c.to_ascii_lowercase()) else { return false };
                match go {
                    Go::Kreditbuch => self.room = Some(Room::Kreditbuch),
                    // the hotspot's own click path (label left under the dialog, confession check of the church)
                    Go::Spot(s) => {
                        let [x, y, w, h] = SPOTS.iter().find(|d| d.0 == s).map(|d| d.3).unwrap_or_default();
                        self.click_kontor(ctx.assets, (x + w / 2, y + h / 2));
                    }
                    Go::Banner => self.click_kontor(ctx.assets, (320, 30)),
                }
            }
            _ => return false,
        }
        true
    }

    /// The tooltip under the rested pointer, if the option is on and there is a click target.
    pub(super) fn tooltip(&self, assets: &Assets, mouse: (i32, i32)) -> Option<String> {
        let p = assets.prefs.borrow();
        if !p.tooltips || self.tip.0 != mouse || self.tip.1 < TIP_MS {
            return None;
        }
        let tr = |de: &'static str, en: &'static str| p.tr(de, en).to_string();
        // a window in front: only its own choices
        if let Some(front) = self.popups.front() {
            let Popup::Choice { acts, .. } = front else { return None };
            let k = front.window(assets, &[])?.item_at(mouse)?;
            return self.action_tip(&p, acts.get(k)?);
        }
        if let Some(b) = self.branch {
            let (w, acts) = self.branch_window(assets, b, &[]);
            if let Some(k) = w.item_at(mouse) {
                return self.action_tip(&p, &acts[k]);
            }
            let (row, house) = self.goods_row(b);
            if hit(house, mouse) {
                return self.action_tip(&p, &Action::Sell(b, 0xFF, 0));
            }
            for (g, icon, num) in row {
                if hit(icon, mouse) {
                    return self.action_tip(&p, &Action::Sell(b, g, 0));
                }
                if hit(num, mouse) {
                    return self.action_tip(&p, &Action::Sell(b, g, 1));
                }
            }
            let names = [("Wohnsitz", "Residence"), ("Lager ausbauen", "Storage"), ("Direktor", "Director"), ("Karren kaufen", "Buy wagons"), ("Schiffe kaufen", "Buy ships"), ("Ballons kaufen", "Buy balloons")];
            return (0..6).find(|&i| hit(branch::control_rect(i), mouse)).map(|i| tr(names[i].0, names[i].1));
        }
        if hit(GOLD_RECT, mouse) && (self.room.is_some() || self.me().is_some()) && !matches!(self.room, Some(Room::Burg(_) | Room::Optionen | Room::Kreditbuch)) {
            return Some(tr("Bestechung", "Bribery"));
        }
        if let Some(r) = self.room {
            return self.action_tip(&p, &self.room_action(assets, r, mouse)?);
        }
        if hit(BANNER_RECT, mouse) {
            return Some(key_hint(&p, tr("Spieloptionen", "Game options"), Some(b'o')));
        }
        let &(spot, ..) = SPOTS.iter().find(|s| hit(s.3, mouse))?;
        let (de, en) = match spot {
            Spot::Truppen => ("Söldner & Räuber: Burgen und Lager", "Mercenaries & robbers: castles and camps"),
            Spot::Schreibstube => ("Schreibstube: Ämter, Kredite, Gesetze, Direktoren", "Scriptorium: offices, loans, laws, directors"),
            Spot::Spielstand => ("Spielstand aller Spieler", "Standings of all players"),
            Spot::Bezahlung => ("Löhne festlegen", "Set the wages"),
            Spot::Hinterzimmer => ("Hinterzimmer: Spione, Sabotage, Erpressung", "Back room: spies, sabotage, blackmail"),
            Spot::Handel => ("Handelskarte: Städte und Niederlassungen", "Trade map: cities and branches"),
            Spot::Kirche => ("Kirche: Heirat, Konfession, Kirchgang", "Church: marriage, confession, service"),
            Spot::GiveUp => ("Gibt das Spiel auf", "Gives up the game"),
        };
        let key = KEYS.iter().find(|k| matches!(k.1, Go::Spot(s) if s == spot)).map(|k| k.0);
        Some(key_hint(&p, tr(de, en), key))
    }

    fn action_tip(&self, p: &Prefs, a: &Action) -> Option<String> {
        let tr = |de: &'static str, en: &'static str| p.tr(de, en).to_string();
        Some(match a {
            Action::Close => tr("Schließt das Fenster", "Closes the window"),
            Action::EndTurn => tr("Beendet die Runde", "Ends the turn"),
            Action::Undo => tr("Nimmt die letzte Runde zurück", "Takes the last turn back"),
            Action::NewPlayer => tr("Neuer Spieler", "New player"),
            Action::Quit => tr("Spiel beenden", "Quit the game"),
            Action::GiveUp => tr("Gibt das Spiel auf", "Gives up the game"),
            Action::Open(m) => return self.menu_tip(p, *m),
            Action::Sim(Command::AskLender, _) => tr("Kredit aufnehmen", "Take a loan"),
            Action::Sim(Command::Matchmaker, _) => tr("Kupplerin: Heiratsvermittlung", "Matchmaker"),
            Action::Sim(Command::RepayLoan { .. }, _) => tr("Kredit tilgen", "Repay the loan"),
            Action::Sim(Command::ApplyView, _) => tr("Bewerbung um ein Amt", "Apply for an office"),
            Action::Sim(Command::CastleOffers { .. }, _) => tr("Burg: Angebote und Info", "Castle: offers and info"),
            Action::Step(f, _) if f.is_word() => tr("Obere Hälfte: nächster Eintrag, untere: voriger", "Upper half: next entry, lower: previous"),
            Action::Step(..) => tr("Obere Hälfte erhöht, untere verringert", "Upper half raises, lower half lowers"),
            Action::Opt(Opt::Pref(_), _) => tr("Extra ein- oder ausschalten", "Toggle the extra"),
            Action::Opt(..) => tr("Einstellung ändern", "Change the setting"),
            Action::ToggleOrder(..) => tr("Auftrag bestätigen oder aufheben", "Confirm or cancel the order"),
            Action::Branch(b) => format!("{} {}", tr("Niederlassung in", "Branch in"), self.loc(self.st.props[*b].loc())),
            Action::Save(k) => format!("{} {k} {}", tr("Spielstand", "Game"), tr("speichern", "save")),
            Action::SaveName(k) => format!("{} {k} {}", tr("Spielstand", "Game"), tr("benennen und speichern", "name and save")),
            Action::Load(k) => format!("{} {k} {}", tr("Spielstand", "Game"), tr("laden", "load")),
            Action::Room(Some(r)) => self.room_tip(p, *r),
            Action::Room(None) => tr("Zurück ins Kontor", "Back to the Kontor"),
            Action::Sell(_, 0xFF, _) => tr("Verkauft alles in diesem Lager", "Sells everything in this warehouse"),
            Action::Sell(_, g, 0) => format!("{} {}{}", tr("Verkauft hier allen Vorrat an", "Sells all"), self.good(*g), tr("", " here")),
            Action::Sell(_, g, _) => format!("{} {}", tr("Verkauft ein Stück", "Sells one unit of"), self.good(*g)),
            Action::SellAll => tr("Verkauft die Vorräte aller Niederlassungen", "Sells every branch's stock"),
            Action::Confirm(_, a) | Action::YesNo(_, _, a) | Action::ConfirmText(_, _, a) => return self.action_tip(p, a),
            Action::Select(_) => tr("Wählt diesen Eintrag aus", "Selects this entry"),
            Action::WithSel(..) => tr("Bestätigt die Auswahl", "Confirms the selection"),
            Action::Dismiss(_, _, n) => format!("{} {n} {}", tr("Entlässt", "Dismisses"), tr("Mann", "men")),
            _ => return None,
        })
    }

    fn room_tip(&self, p: &Prefs, r: Room) -> String {
        let tr = |de: &'static str, en: &'static str| p.tr(de, en).to_string();
        match r {
            Room::Handel => tr("Handelskarte", "Trade map"),
            Room::Schreibstube => tr("Schreibstube", "Scriptorium"),
            Room::Bezahlung => tr("Bezahlung", "Wages"),
            Room::Hinterzimmer => tr("Hinterzimmer", "Back room"),
            Room::Kirche => tr("Kirche", "Church"),
            Room::Kirchgang => tr("Kirchgang: Beten, Kerzen, Almosen, Ablass", "Church service: prayer, candles, alms, indulgence"),
            Room::Truppen => tr("Söldner & Räuber", "Mercenaries & robbers"),
            Room::Burg(b) => format!("{} {}", tr("Truppen in", "Troops at"), self.loc(self.st.props[b].loc())),
            Room::Optionen => tr("Spieloptionen", "Game options"),
            Room::Kreditbuch => tr("Kreditbuch", "Debt book"),
        }
    }

    fn menu_tip(&self, p: &Prefs, m: Menu) -> Option<String> {
        let tr = |de: &'static str, en: &'static str| p.tr(de, en).to_string();
        Some(match m {
            Menu::Spielstand => tr("Spielstand aller Spieler", "Standings of all players"),
            Menu::Save => tr("Spiel speichern", "Save the game"),
            Menu::Load => tr("Spiel laden", "Load a game"),
            Menu::Quit => tr("Spiel beenden", "Quit the game"),
            Menu::City(c) => format!("{} {}", tr("Stadtinfo", "City info"), self.loc(c)),
            Menu::Storage(_) => tr("Lager ausbauen", "Extend the storage"),
            Menu::Transport(_, k) => format!("{} {}", branch::TRANSPORTER_NAMES[k as usize], tr("kaufen", "buy")),
            Menu::Apply => tr("Bewerbung um ein Amt", "Apply for an office"),
            Menu::Targets(plot) | Menu::Tree(plot, _, _) => plot.verb().to_string(),
            Menu::Candles => tr("Kerzen stiften", "Donate candles"),
            Menu::Alms => tr("Almosen geben", "Give alms"),
            Menu::Pray => tr("Beten", "Pray"),
            Menu::Orders(_) => tr("Befehlsbuch", "Order book"),
            Menu::CastleInfo(_) => tr("Info zur Burg oder zum Lager", "Castle or camp info"),
            Menu::Recruit(_) => tr("Anwerben", "Recruit"),
            Menu::Manoeuvre(_) => tr("Manöver", "Manoeuvre"),
            Menu::Toll(_) => tr("Wegzoll", "Toll"),
            Menu::Expand(_) => tr("Zustand und Ausbau", "Condition and expansion"),
            Menu::CastleSell(_) => tr("Burg verkaufen", "Sell the castle"),
            Menu::Directors => tr("Direktoren", "Directors"),
            Menu::Residence(_) => tr("Wohnsitz", "Residence"),
            Menu::Director(_) => tr("Direktor dieser Niederlassung", "Director of this branch"),
            Menu::JoinChurch => tr("Kircheneintritt", "Join a church"),
            Menu::Kupplerin => tr("Kupplerin", "Matchmaker"),
            Menu::Convert => tr("Konvertieren", "Convert"),
            Menu::Leave => tr("Aus der Kirche austreten", "Leave the church"),
            Menu::Indulgence => tr("Ablass kaufen", "Buy an indulgence"),
            Menu::Feast(_) => tr("Fest geben", "Give a feast"),
            Menu::Testament => tr("Mein Testament", "My will"),
            Menu::Laws(_) => tr("Gesetzbuch", "Law book"),
            Menu::Options => tr("Spieloptionen", "Game options"),
            Menu::Extras(_) => tr("Extras (unsere Optionen)", "Extras (our options)"),
            Menu::Ledger(_) => tr("Bilanz", "Ledger"),
            Menu::Prices => tr("Preistabelle", "Price table"),
            Menu::ProdHelp(_) => tr("Produktionshilfe", "Production help"),
            Menu::Privileges => tr("Privilegien", "Privileges"),
            Menu::Music => tr("Musik", "Music"),
            Menu::Graphics => tr("Grafik", "Graphics"),
            Menu::Keys => tr("Tasten", "Keys"),
            _ => return None,
        })
    }
}

/// The label box: one line of dark text on parchment with a 1 px border, below right of the pointer, kept on screen.
pub(super) fn draw_tip(frame: &mut Frame, assets: &Assets, tip: &str, mouse: (i32, i32)) {
    let laid = text::layout(&assets.font, &[b"$t$L".as_slice(), &super::t(tip)].concat(), &[], &[], 640, &[]);
    let tw: i32 = laid.glyphs.iter().map(|g| assets.font.advance[g.2 as usize] as i32).sum();
    let (w, h) = (tw + 12, text::LINE_H);
    let x = (mouse.0 + 14).min(640 - w).max(0);
    let y = (mouse.1 + 22).min(480 - h).max(0);
    frame.fill_rgb([x, y, w, h], [64, 40, 16]);
    frame.fill_rgb([x + 1, y + 1, w - 2, h - 2], [226, 202, 150]);
    // the layout starts at the window inset
    text::draw(frame, &assets.font, &laid, x + 6 - text::INSET, y + 3 - text::INSET);
}

#[cfg(test)]
mod tests {
    use super::super::tests::{ctx_for, shot_at, test_assets};
    use super::*;
    use crate::input::{Event, Input};
    use crate::screens::Screen;

    /// No Kontor key doubles another Extra's key, the Tab of the Extras window or each other.
    #[test]
    fn key_table() {
        let reserved = b"bpvuhlce\t";
        for (i, k) in KEYS.iter().enumerate() {
            assert!(!reserved.contains(&k.0), "{}", k.0 as char);
            assert!(KEYS[i + 1..].iter().all(|o| o.0 != k.0));
        }
        let p = Prefs { shortcuts: true, ledger: true, ..Prefs::default() };
        assert_eq!(key_list(&p).len(), KEYS.len() + 6);
        assert_eq!(nth_bullet(&[Action::Step(super::super::Field::Bribe, 1), Action::Close, Action::EndTurn], 1), Some(2));
        assert_eq!(nth_bullet(&[Action::Close], 1), None);
    }

    /// Tooltips: nothing with the option off or before the rest time; the Kontor spot, a menu line (and not the
    /// Kontor under it), the branch warehouse; a move restarts the wait.
    #[test]
    fn real_tooltips() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, mad_sim::State::new_game(5, 3, &[mad_sim::world::HumanSetup::test(3)]).unwrap());
        g.popups.clear();
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        let handel = (60, 400);
        let rest = |g: &mut Game, ctx: &mut Ctx, at: (i32, i32), ms: u32| {
            g.update(ctx, &Input { mouse: (-1, -1), ..Default::default() }, 0);
            g.update(ctx, &Input { mouse: at, ..Default::default() }, 0);
            g.update(ctx, &Input { mouse: at, ..Default::default() }, ms);
        };
        rest(&mut g, &mut ctx, handel, 1000);
        assert_eq!(g.tooltip(&assets, handel), None, "off");
        assets.prefs.borrow_mut().tooltips = true;
        rest(&mut g, &mut ctx, handel, TIP_MS - 1);
        assert_eq!(g.tooltip(&assets, handel), None, "too early");
        g.update(&mut ctx, &Input { mouse: handel, ..Default::default() }, 1);
        assert_eq!(g.tooltip(&assets, handel).as_deref(), Some("Handelskarte: Städte und Niederlassungen"));
        shot_at(&g, &assets, "tip_kontor", handel);
        assert_eq!(g.tooltip(&assets, (61, 400)), None, "moved");
        assets.prefs.borrow_mut().shortcuts = true;
        rest(&mut g, &mut ctx, handel, TIP_MS);
        assert_eq!(g.tooltip(&assets, handel).as_deref(), Some("Handelskarte: Städte und Niederlassungen (M)"));
        // a menu in front: its lines, not the Kontor behind it
        g.open(&assets, Menu::Extras(0));
        rest(&mut g, &mut ctx, handel, TIP_MS);
        assert_eq!(g.tooltip(&assets, handel), None, "under the window");
        let w = g.popups.front().unwrap().window(&assets, &[]).unwrap();
        let r = w.item_rect(0);
        let at = (r.x + 2, r.y + 2);
        rest(&mut g, &mut ctx, at, TIP_MS);
        assert_eq!(g.tooltip(&assets, at).as_deref(), Some("Extra ein- oder ausschalten"));
        shot_at(&g, &assets, "tip_menu", at);
        g.popups.clear();
        // the branch screen: warehouse, a good, a bottom icon
        let h = g.me().unwrap();
        let b = g.st.branch_at(h, 3).unwrap();
        g.branch = Some(b);
        let (row, house) = g.goods_row(b);
        let at = (house[0] + 5, house[1] + 5);
        rest(&mut g, &mut ctx, at, TIP_MS);
        assert_eq!(g.tooltip(&assets, at).as_deref(), Some("Verkauft alles in diesem Lager"));
        shot_at(&g, &assets, "tip_branch", at);
        let at = (row[0].1[0] + 5, row[0].1[1] + 5);
        rest(&mut g, &mut ctx, at, TIP_MS);
        assert!(g.tooltip(&assets, at).unwrap().starts_with("Verkauft hier allen Vorrat an "));
        let at = (40, 410);
        rest(&mut g, &mut ctx, at, TIP_MS);
        assert_eq!(g.tooltip(&assets, at).as_deref(), Some("Wohnsitz"));
        g.branch = None;
        // the Handel map: an own city and a foreign one
        g.room = Some(Room::Handel);
        rest(&mut g, &mut ctx, (110, 300), TIP_MS);
        assert!(g.tooltip(&assets, (110, 300)).unwrap().starts_with("Niederlassung in "));
        rest(&mut g, &mut ctx, (335, 210), TIP_MS);
        assert!(g.tooltip(&assets, (335, 210)).unwrap().starts_with("Stadtinfo "));
        shot_at(&g, &assets, "tip_map", (335, 210));
        assets.prefs.borrow_mut().tooltips = false;
        rest(&mut g, &mut ctx, (335, 210), TIP_MS);
        assert_eq!(g.tooltip(&assets, (335, 210)), None);
    }

    /// Shortcuts: inert while off; on, the Kontor keys open the places, Esc leaves them, a digit picks a menu line,
    /// Enter ends the turn through the confirmation, `?` lists the keys.
    #[test]
    fn real_shortcuts() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, mad_sim::State::new_game(5, 3, &[mad_sim::world::HumanSetup::test(3)]).unwrap());
        g.popups.clear();
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        let key = |g: &mut Game, ctx: &mut Ctx, k: Key| g.update(ctx, &Input { events: vec![Event::Key(k)], ..Default::default() }, 0);
        key(&mut g, &mut ctx, Key::Char(b'm'));
        key(&mut g, &mut ctx, Key::Char(b'?'));
        key(&mut g, &mut ctx, Key::Enter);
        assert!(g.room.is_none() && g.popups.is_empty(), "off");
        assets.prefs.borrow_mut().shortcuts = true;
        for (k, r) in [(b'm', Room::Handel), (b'S', Room::Schreibstube), (b'g', Room::Bezahlung), (b'z', Room::Hinterzimmer), (b'k', Room::Kirche), (b't', Room::Truppen), (b'd', Room::Kreditbuch), (b'o', Room::Optionen)] {
            key(&mut g, &mut ctx, Key::Char(k));
            assert_eq!(g.room, Some(r), "{}", k as char);
            key(&mut g, &mut ctx, Key::Char(b'm'));
            assert_eq!(g.room, Some(r), "a room key only works in the Kontor");
            key(&mut g, &mut ctx, Key::Escape);
            if r == Room::Kreditbuch {
                assert_eq!(g.room, Some(Room::Schreibstube));
                key(&mut g, &mut ctx, Key::Escape);
            }
            assert_eq!(g.room, None);
        }
        key(&mut g, &mut ctx, Key::Char(b'r'));
        assert!(matches!(g.popups.front(), Some(Popup::Choice { menu: Some(Menu::Spielstand), .. })));
        // digit 1 clicks the first line: its message box comes up; 9 is out of range and does nothing
        key(&mut g, &mut ctx, Key::Char(b'9'));
        assert_eq!(g.popups.len(), 1);
        key(&mut g, &mut ctx, Key::Char(b'1'));
        assert!(matches!(g.popups.front(), Some(Popup::Choice { menu: None, .. })), "{:?}", g.popups.len());
        g.popups.clear();
        key(&mut g, &mut ctx, Key::Char(b'?'));
        assert!(matches!(g.popups.front(), Some(Popup::Choice { menu: Some(Menu::Keys), .. })));
        shot_at(&g, &assets, "keys", (630, 470));
        key(&mut g, &mut ctx, Key::Escape);
        // Enter: the end-turn confirmation when it is configured, with the turn end behind its yes
        g.cfg_end |= 0x20;
        let year = g.st.year;
        key(&mut g, &mut ctx, Key::Enter);
        assert!(matches!(g.popups.front(), Some(Popup::Choice { acts, .. }) if acts.iter().any(|a| matches!(a, Action::EndTurn))));
        assert_eq!(g.st.year, year);
        key(&mut g, &mut ctx, Key::Escape);
        assert!(g.popups.is_empty());
    }
}
