//! Ours, both default off and not in any game file: tooltips (`prefs.tooltips`, a label of what a click would do
//! after the pointer rested) and keyboard shortcuts (`prefs.shortcuts`: Kontor keys to the places, Space / Enter
//! end the turn, 1-9 pick a menu line, `?` lists the keys).

use super::{Action, BANNER_RECT, GOLD_RECT, Game, Menu, Opt, Popup, Pref, SPOTS, Spot, branch, hit, rooms::Room};
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
pub(super) enum Go {
    Spot(Spot),
    Banner,
    Kreditbuch,
}

/// Kontor keys (lower case) and what they open. B, P, V, U, H, L, C, E and Tab belong to other Extras.
pub(super) const KEYS: [(u8, Go, (&str, &str)); 9] = [
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
    v.push((tr("Leer", "Space"), tr("Runde beenden", "End the turn")));
    v.push(("Enter".into(), tr("Runde beenden", "End the turn")));
    v.push(("Esc".into(), tr("Schließen, verlassen", "Close, leave")));
    v.push(("1 - 9".into(), tr("Menüzeile wählen", "Pick a menu line")));
    v.push(("?".into(), tr("Diese Liste", "This list")));
    v.push(("Tab".into(), tr("Extras", "Extras")));
    for (on, k, de, en) in [(p.ledger, "B", "Bilanz", "Ledger"), (p.price_table, "P", "Preistabelle", "Price table"), (p.sell_all, "V", "Alles verkaufen", "Sell all"), (p.undo, "U", "Runde zurück", "Undo turn"), (p.production_help, "H", "Produktionshilfe", "Production help"), (p.event_log, "L", "Chronik", "Event log"), (p.order_copy, "C", "Aufträge kopieren", "Copy orders"), (p.order_copy, "E", "Aufträge einfügen", "Paste orders")] {
        if on {
            v.push((k.into(), tr(de, en)));
        }
    }
    v
}

/// What an Extras option does (title line, then at most two lines of about 45 characters).
fn pref_tip(k: Pref) -> (&'static str, &'static str) {
    match k {
        Pref::Fullscreen => ("Vollbild\nAuch mit F11 oder Alt+Enter.", "Fullscreen\nAlso with F11 or Alt+Enter."),
        Pref::IntegerScale => ("Ganzzahlig skalieren\nScharfe Pixel, dafür schwarze Ränder.\nAus: füllt das Fenster im Format 4:3.", "Integer scaling\nSharp pixels, black borders.\nOff: fills the window at 4:3."),
        Pref::Speed => ("Tempo 1x bis 4x\nSchnellere Texte, Wartezeiten, Filme\nund Schlachten. Die Musik bleibt gleich.", "Speed 1x to 4x\nFaster texts, waits, films and battles.\nThe music stays the same."),
        Pref::Lang => ("Sprache\nNur für die Texte der Extras.\nDas Spiel selbst bleibt deutsch.", "Language\nFor the Extras' own texts only.\nThe game itself stays German."),
        Pref::Tooltips => ("Hinweise\nZeigt nach kurzem Verweilen,\nwas ein Klick bewirkt.", "Tooltips\nAfter a short rest the pointer\nshows what a click does."),
        Pref::Shortcuts => ("Tasten\nKontor-Tasten zu allen Räumen, Leertaste\nbeendet die Runde. ? zeigt alle Tasten.", "Shortcuts\nKontor keys to every room, Space ends\nthe turn. ? lists all keys."),
        Pref::Autosave => ("Autosave\nSpeichert alle 1, 5 oder 10 Jahre in den\nVerlauf (Spiel laden, Verlauf...).", "Autosave\nSaves every 1, 5 or 10 years into the\nhistory (Load game, History...)."),
        Pref::Undo => ("Rückgängig\nTaste U nimmt die letzte Runde einmal\nzurück. Danach wird neu gewürfelt.", "Undo\nKey U takes the last turn back once.\nThe dice are rolled anew."),
        Pref::SeedChoice => ("Spielnummer wählen\nNeues Spiel: Zufall, Tagesspiel oder eine\nNummer. Gleiche Nummer, gleiche Welt.", "Choose game number\nNew game: random, daily game or a number.\nSame number, same world."),
        Pref::SkipTrial => ("Szenen überspringen\nLeertaste überspringt Gericht, Begräbnis,\nWahlen und Entlassungen. Gleiches Ergebnis.", "Skip scenes\nSpace skips court, funeral, elections and\ndismissals. The outcome is the same."),
        Pref::SkipLot => ("Posten überspringen\nTab lässt einen Auktionsposten ohne dich\nlaufen, ohne Wartezeit.", "Skip auction lot\nTab runs an auction lot without you,\nwith no waits."),
        Pref::Warnings => ("Schuldenwarnung\nWarnt zu Rundenbeginn vor Pfändung,\nfälligen Krediten und Ähnlichem.", "Warnings\nWarns at the turn start of seizure,\nloans due and the like."),
        Pref::ProductionHelp => ("Produktionshilfe\nTaste H: bestes Verhältnis, nötige Mittel\nund erwarteter Ertrag je Ware.", "Production help\nKey H: best ratio, means needed and\nexpected yield per good."),
        Pref::Ledger => ("Bilanz\nTaste B: Vermögen, Bargeld und Ergebnis\nder Niederlassungen über die Jahre.", "Ledger\nKey B: net worth, cash and branch\nresults over the years."),
        Pref::PriceTable => ("Preistabelle\nTaste P: alle Waren in allen Städten.", "Price table\nKey P: every good in every city."),
        Pref::SellAll => ("Alles verkaufen\nTaste V oder Verkaufen auf der Karte:\nverkauft den Vorrat aller Niederlassungen.", "Sell all\nKey V or Sell all on the map: sells\nevery branch's stock."),
        Pref::EventLog => ("Chronik\nTaste L: alle Meldungen dieser Sitzung,\ndie neueste zuerst.", "Event log\nKey L: every report of this session,\nnewest first."),
        Pref::OrderCopy => ("Aufträge kopieren\nTaste C kopiert die Aufträge einer\nNiederlassung, Taste E fügt sie ein.", "Copy orders\nKey C copies a branch's orders,\nkey E pastes them."),
        Pref::Confirmations => ("Sicherheitsfragen\nFragt vor Bestechung, Erpressung, Spionen\nund anderem, das nicht zurückgeht.", "Confirmations\nAsks before bribes, blackmail, spies\nand other things that can't be undone."),
        Pref::OfficeInfo => ("Amtsinfo\nIm Ämterbaum: wer ein Amt vergibt, die\nBewerber und wie die Wahl steht.", "Office info\nIn the office tree: who appoints, the\napplicants and how the vote stands."),
        Pref::DebtMercy => ("Gnade bei Schulden (Hausregel)\nSchulden nehmen nie die letzte\nNiederlassung, es urteilen die Gläubiger.", "Debt mercy (house rule)\nDebt never takes your last branch,\nthe creditors judge instead."),
        Pref::ContestNames => ("Wettbewerbsnamen\nZeitungswettbewerb: Namen für Salz, Tuch,\nSilber, Pfeffer und Waffen (im Original leer).", "Contest names\nNewspaper contest: names for Salz, Tuch,\nSilber, Pfeffer and Waffen (blank originally)."),
        Pref::MoreWorkers => ("Bis 255 Arbeiter (Hausregel)\nProduktionsaufträge mit bis zu 255\nArbeitern statt 40.", "Up to 255 workers (house rule)\nProduction orders with up to 255\nworkers instead of 40."),
    }
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
            Action::Opt(Opt::Pref(k), _) => {
                let (de, en) = pref_tip(*k);
                tr(de, en)
            }
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

impl Game {
    /// Ours (`prefs.office_info`): the office of the open tree window under the rested pointer, and its info lines.
    pub(super) fn office_tip(&self, assets: &Assets, mouse: (i32, i32)) -> Option<Vec<String>> {
        let p = assets.prefs.borrow();
        if !p.office_info || self.tip.0 != mouse || self.tip.1 < TIP_MS {
            return None;
        }
        let Some(Popup::Choice { menu: Some(Menu::Tree(_, loc, church)), .. }) = self.popups.front() else { return None };
        let (n, obase, _) = super::menus::tree_slots(*loc, *church);
        let idx = (obase..obase + n).map(|i| i as u8).find(|&i| hit(super::menus::tree_entry_rect(*loc, i), mouse))?;
        Some(self.office_info(&p, *loc as usize, idx as usize))
    }

    /// Who fills (loc, idx) and how (rules-03 §3, §4.6; rules-65): appointers with the player's favour, holder or
    /// applicants, the player's own standing, and the RNG-free forecast of the vote.
    fn office_info(&self, p: &Prefs, loc: usize, idx: usize) -> Vec<String> {
        use mad_sim::career::global_of;
        let tr = |de: &'static str, en: &'static str| p.tr(de, en).to_string();
        let st = &self.st;
        let me = self.me();
        let o = global_of(loc, idx);
        let who = |x: usize| format!("{} ({})", self.first_name(x), self.office_of(x));
        let mut v = vec![format!("{} {} {}", self.office(loc as u8, idx as u8), tr("in", "in"), self.loc(loc as u8))];
        let holder = st.holder(loc, idx);
        match holder {
            Some(x) => {
                v.push(format!("{} {}", tr("Inhaber:", "Holder:"), self.person_name(x as u8)));
                v.push(tr("Frei nur durch Entlassung, Tod oder Aufstieg.", "Freed only by dismissal, death or promotion."));
            }
            None if st.year < 1601 => v.push(tr("Unbesetzt, Wahl am Jahresende ab 1601.", "Vacant, elected at the year end from 1601.")),
            None => v.push(tr("Unbesetzt, Wahl am Jahresende.", "Vacant, elected at the year end.")),
        }
        // the appointer seats, each with its holder or the blackmailers who vote in his place
        let seats = mad_sim::State::appointer_seats(loc, idx);
        let electors = st.electors(loc, idx);
        if seats.is_empty() {
            v.push(tr("Ernennung: das Los.", "Appointment: by lot."));
        } else {
            v.push(tr("Ernennung durch:", "Appointed by:"));
            for (l, i) in seats {
                let seat = if l == loc { self.office(l as u8, i as u8) } else { format!("{} ({})", self.office(l as u8, i as u8), self.loc(l as u8)) };
                let Some(a) = st.holder(l, i) else {
                    v.push(format!("  {seat}: {}", tr("unbesetzt", "vacant")));
                    continue;
                };
                let fav = |e: usize| me.map(|h| format!(", {} {}", tr("Gunst zu dir", "favour to you"), st.persons[e].favour(h))).unwrap_or_default();
                let bm: Vec<usize> = electors.iter().filter(|&&(_, f)| f == a as u8).map(|&(e, _)| e).collect();
                if bm.is_empty() {
                    v.push(format!("  {seat}: {}{}", self.first_name(a), fav(a)));
                } else {
                    v.push(format!("  {seat}: {} {}:", self.first_name(a), tr("erpresst, es wählen", "blackmailed, voting instead")));
                    for e in bm {
                        v.push(format!("    {}{}", who(e), fav(e)));
                    }
                }
            }
        }
        if holder.is_some() {
            return v;
        }
        // a vacant seat: privileged candidate, applicants, the player, the forecast
        let f = st.forecast(loc, idx);
        if let Some(x) = match o { 33 => st.holder(13, 8), 22 => st.holder(loc, 10), 21 => st.holder(loc, 9), _ => None } {
            v.push(format!("{} {}", tr("Privilegierter Kandidat:", "Privileged candidate:"), who(x)));
            v.push(tr("  Bewirbt er sich, ist er der einzige Kandidat.", "  If he applies he is the only candidate."));
        }
        if f.candidates.is_empty() {
            v.push(tr("Bewerber: keine. Zufällige wählbare Personen und die", "Applicants: none. Random eligible persons and the"));
            v.push(tr("Rivalen treten an; ist ein Mensch wählbar, bleibt der", "rivals stand; if a human is eligible the seat stays"));
            v.push(tr("Sitz ein Jahr länger frei.", "empty one more year."));
        } else {
            v.push(format!("{} {}", tr("Bewerber:", "Applicants:"), f.candidates.iter().map(|&c| self.first_name(c)).collect::<Vec<_>>().join(", ")));
        }
        if let Some(h) = me {
            let applied = st.applicants(loc, &mad_sim::career::seat_group(loc, idx)).contains(&h);
            let s = match (st.eligibility(h, loc, idx), applied) {
                (0, _) => tr("nicht wählbar", "not eligible"),
                (_, true) => tr("wählbar, beworben", "eligible, applied"),
                _ => tr("wählbar, nicht beworben", "eligible, not applied"),
            };
            v.push(format!("{} {s}", tr("Du:", "You:")));
        }
        if f.candidates.is_empty() {
            return v;
        }
        v.push(tr("Voraussichtliche Stimmen:", "Likely votes:"));
        if f.electors.is_empty() {
            v.push(tr("  keine Wähler: das Los entscheidet.", "  no electors: the lot decides."));
        }
        for (k, &(e, _)) in f.electors.iter().enumerate() {
            let line = match f.picks[k] {
                None => tr("wählt selbst", "votes on screen"),
                Some(c) => {
                    let (s, office) = f.scores[k][c];
                    let rnd = if office && self.tree_rivals.contains(&f.candidates[c]) { tr(" +0..19 Zufall", " +0..19 random") } else { String::new() };
                    format!("{} ({s}{rnd})", self.first_name(f.candidates[c]))
                }
            };
            v.push(format!("  {}: {line}", self.first_name(e)));
        }
        let names = |ks: &[usize]| ks.iter().map(|&k| self.first_name(f.candidates[k])).collect::<Vec<_>>().join(", ");
        if f.leaders.len() == 1 {
            let k = f.leaders[0];
            v.push(format!("{} {} ({} {})", tr("Voraussichtlich:", "Likely winner:"), names(&[k]), f.votes[k], tr("Stimmen", "votes")));
        } else {
            v.push(format!("{} {} ({})", tr("Patt:", "Tie:"), names(&f.leaders), tr("das Los entscheidet", "the lot decides")));
        }
        v
    }
}

/// The label box below right of the pointer, kept on screen; `\n` starts a new line.
pub(super) fn draw_tip(frame: &mut Frame, assets: &Assets, tip: &str, mouse: (i32, i32)) {
    let lines: Vec<String> = tip.split('\n').map(str::to_string).collect();
    draw_lines(frame, assets, &lines, (mouse.0 + 14, mouse.1 + 22));
}

/// Line pitch of the box: tighter than the game's 24 px rows.
const TIP_LINE_H: i32 = 20;
/// Inner margin left/right and top/bottom.
const TIP_PAD: (i32, i32) = (10, 7);
/// Gap under a title line, with the rule in it.
const TIP_RULE: i32 = 6;

/// The parchment box: a soft shadow, a dark frame with a gold inner line; with several lines the first is a title
/// set off by a rule. Clamped to the screen.
pub(super) fn draw_lines(frame: &mut Frame, assets: &Assets, lines: &[String], at: (i32, i32)) {
    // a title in the game's red highlight
    let head = |k: usize| if k == 0 && lines.len() > 1 { b"$t$L$h".as_slice() } else { b"$t$L" };
    let laid: Vec<text::Laid> = lines.iter().enumerate().map(|(k, l)| text::layout(&assets.font, &[head(k), &super::t(l)].concat(), &[], &[], 640, &[])).collect();
    let tw: i32 = laid.iter().map(|l| l.glyphs.iter().map(|g| assets.font.advance[g.2 as usize] as i32).sum::<i32>()).max().unwrap_or(0) + 4;
    let rule = if lines.len() > 1 { TIP_RULE } else { 0 };
    let (w, h) = (tw + 2 * TIP_PAD.0, TIP_LINE_H * lines.len() as i32 + rule + 2 * TIP_PAD.1);
    let x = at.0.min(640 - w - 4).max(0);
    let y = at.1.min(480 - h - 4).max(0);
    frame.shade([x + 4, y + 4, w, h]);
    frame.fill_rgb([x, y, w, h], [52, 30, 10]);
    frame.fill_rgb([x + 1, y + 1, w - 2, h - 2], [232, 210, 160]);
    frame.outline_rgb([x + 3, y + 3, w - 6, h - 6], [176, 132, 52]);
    // the layout starts at the window inset
    let ty = |k: i32| y + TIP_PAD.1 + 3 - text::INSET + k * TIP_LINE_H + if k > 0 { rule } else { 0 };
    for (k, l) in laid.iter().enumerate() {
        text::draw(frame, &assets.font, l, x + TIP_PAD.0 - text::INSET, ty(k as i32));
    }
    if rule > 0 {
        frame.fill_rgb([x + TIP_PAD.0, y + TIP_PAD.1 + TIP_LINE_H + rule / 2 + 1, w - 2 * TIP_PAD.0, 1], [176, 132, 52]);
    }
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
        assert_eq!(key_list(&p).len(), KEYS.len() + 7);
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
        assert_eq!(g.tooltip(&assets, at).as_deref(), Some(pref_tip(Pref::Fullscreen).0));
        shot_at(&g, &assets, "tip_menu", at);
        let r = w.item_rect(2 * 9);
        let at = (r.x + 2, r.y + 2);
        rest(&mut g, &mut ctx, at, TIP_MS);
        shot_at(&g, &assets, "tip_menu_long", at);
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

    /// Office info: inert while off; on, resting on a vacant Ratsherr names the appointers, the applicants and the
    /// forecast, an occupied office its holder; every tree entry's rectangle holds its menu items.
    #[test]
    fn real_office_info() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, mad_sim::State::new_game(5, 3, &[mad_sim::world::HumanSetup::test(3)]).unwrap());
        g.popups.clear();
        g.st.year = 1601;
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        let c = 3u8;
        let v = g.st.holder(c as usize, 1).unwrap();
        g.st.office_vacate(v);
        let cands: Vec<usize> = g.st.computers().filter(|&x| g.st.eligibility(x, c as usize, 1) == 2).take(2).collect();
        for &x in &cands {
            g.st.apply_for(x, c as usize, 1, true);
        }
        let rest = |g: &mut Game, ctx: &mut Ctx, at: (i32, i32)| {
            g.update(ctx, &Input { mouse: (-1, -1), ..Default::default() }, 0);
            g.update(ctx, &Input { mouse: at, ..Default::default() }, 0);
            g.update(ctx, &Input { mouse: at, ..Default::default() }, TIP_MS);
        };
        let centre = |r: [i32; 4]| (r[0] + r[2] / 2, r[1] + r[3] / 2);
        let vacant = centre(super::super::menus::tree_entry_rect(c, 1));
        g.open(&assets, Menu::Tree(super::super::Plot::Bribe, c, false));
        rest(&mut g, &mut ctx, vacant);
        assert_eq!(g.office_tip(&assets, vacant), None, "off");
        assets.prefs.borrow_mut().office_info = true;
        g.popups.clear();
        g.open(&assets, Menu::Tree(super::super::Plot::Bribe, c, false));
        // every occupied entry's items lie inside its computed rectangle
        let w = g.popups.front().unwrap().window(&assets, &[]).unwrap();
        let Some(Popup::Choice { acts, .. }) = g.popups.front() else { panic!() };
        for (k, a) in acts.iter().enumerate() {
            let Action::Open(Menu::BribeAmount(p)) = a else { continue };
            let pr = &g.st.persons[*p as usize];
            let r = super::super::menus::tree_entry_rect(pr.loc(), pr.idx());
            let it = w.item_rect(k);
            assert!(it.x >= r[0] && it.x + it.w <= r[0] + r[2] && it.y >= r[1] && it.y + it.h <= r[1] + r[3], "{r:?} {it:?}");
        }
        rest(&mut g, &mut ctx, vacant);
        let lines = g.office_tip(&assets, vacant).expect("the vacant Ratsherr");
        let all = lines.join("\n");
        assert!(all.contains("Unbesetzt") && all.contains("Baumeister") && all.contains("Bewerber:") && all.contains("Voraussichtlich"), "{all}");
        assert!(lines.iter().all(|l| !l.contains('/')), "{all}");
        shot_at(&g, &assets, "office_info_vacant", vacant);
        let bm = centre(super::super::menus::tree_entry_rect(c, 6));
        rest(&mut g, &mut ctx, bm);
        let lines = g.office_tip(&assets, bm).expect("the Bürgermeister");
        assert!(lines.iter().any(|l| l.starts_with("Inhaber:")) && lines.iter().any(|l| l.contains("Ratsherr")), "{lines:?}");
        shot_at(&g, &assets, "office_info_holder", bm);
        // the county tree: the Vogt is drawn by lot; a vacant Bischof names the Diakon as privileged candidate
        g.popups.clear();
        g.open(&assets, Menu::Tree(super::super::Plot::Bribe, 10, false));
        let vogt = centre(super::super::menus::tree_entry_rect(10, 7));
        rest(&mut g, &mut ctx, vogt);
        assert!(g.office_tip(&assets, vogt).unwrap().iter().any(|l| l.contains("Los")));
        if let Some(b) = g.st.holder(10, 12) {
            g.st.office_vacate(b);
        }
        g.popups.clear();
        g.open(&assets, Menu::Tree(super::super::Plot::Bribe, 10, true));
        let bishop = centre(super::super::menus::tree_entry_rect(10, 12));
        rest(&mut g, &mut ctx, bishop);
        let lines = g.office_tip(&assets, bishop).unwrap();
        assert!(lines.iter().any(|l| l.starts_with("Privilegierter Kandidat:")) && lines.iter().any(|l| l.contains("Bewerber: keine")), "{lines:?}");
        shot_at(&g, &assets, "office_info_bishop", bishop);
        // the Extras page with the new row
        g.popups.clear();
        g.open(&assets, Menu::Extras(1));
        shot_at(&g, &assets, "extras_office_info", (630, 470));
        assets.prefs.borrow_mut().office_info = false;
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
        // every Extra with a key on: still fits
        let saved = assets.prefs.borrow().clone();
        *assets.prefs.borrow_mut() = Prefs { ledger: true, price_table: true, sell_all: true, undo: true, production_help: true, event_log: true, order_copy: true, ..saved.clone() };
        g.popups.clear();
        g.open(&assets, Menu::Keys);
        shot_at(&g, &assets, "keys_all", (630, 470));
        *assets.prefs.borrow_mut() = saved;
        g.popups.clear();
        g.open(&assets, Menu::Keys);
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
