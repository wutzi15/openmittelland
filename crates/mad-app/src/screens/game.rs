//! In-game screens driven by `mad-sim`, per `docs/re/formats.md` §9: "Nächster Spieler", the
//! Erträge report, the Kontor with its banner. The Kontor menus (`menus.rs`) and the branch screen
//! (`branch.rs`) have no RE layout yet and are our own framed windows. Gaps: `docs/IMPL-GAPS.md`.

mod battle;
mod branch;
mod menus;
mod rooms;
mod tips;

use rooms::Room;

use super::{Ctx, Next, Screen, Window, menu};
use crate::{
    assets::Assets,
    gfx::Frame,
    input::{Cursor, Event, Input, Key, cp437},
    text,
};
use mad_sim::{Ask, Choices, Command, Event as SimEvent, State, state::NOBODY};
use std::collections::VecDeque;

/// EXE name tables (`docs/re/rules-00-map.md`): u32 pointers stored relative to linear 0x80000.
mod names {
    pub const LOCATIONS: (u32, usize) = (0xB6194, 21);
    pub const OFFICES: (u32, usize) = (0xB4378, 34);
    pub const GOODS: (u32, usize) = (0xB6618, 15);
    pub const TITLES_M: (u32, usize) = (0xB6A58, 10);
    pub const TITLES_F: (u32, usize) = (0xB6A80, 10);
    pub const FIRST_M: (u32, usize) = (0xB4FB8, 117);
    pub const FIRST_F: (u32, usize) = (0xB531C, 70);
}

/// EXE strings used here (`docs/re/strings.md` addresses).
mod addr {
    pub const KONTOR: u32 = 0xBFD64;
    pub const ERTRAEGE: u32 = 0xBD9D8;
    pub const NEXT_PLAYER: u32 = 0xBD9AC;
    pub const KUPPLERIN: u32 = 0x9D9F9;
    pub const KUPPLERIN_OFFER: u32 = 0x9DB78;
    pub const KUPPLERIN_THIS_IS: u32 = 0x9DAD4;
    pub const KUPPLERIN_ADVICE: u32 = 0x9DBD0;
    pub const KUPPLERIN_LEAVE: u32 = 0x9DAB2;
    pub const NONE_PRODUCED: u32 = 0xBDA04;
    pub const EXPORTS: u32 = 0xBDA84;
    pub const NO_EXPORTS: u32 = 0xBDA98;
    pub const EXPORTS_BROUGHT: u32 = 0xBDAC4;
    pub const GIVEN_AWAY: u32 = 0xBDAF0;
    pub const GIVE_UP: u32 = 0xC026C;
    pub const END_TURN: u32 = 0xC02EC;
    pub const NOT_VISITED: u32 = 0xC0324;
    pub const NEIN: u32 = 0xC0260;
    pub const JA: u32 = 0xC0268;
}

const BANNER: u16 = 0x22;
const KONTOR_BG: u16 = 0xAB;
/// A 32 px `%j` icon's top is this far above its line's top (rules-46 §1: line y - 6; live A/B of the law book's
/// tab icons 2026-11-14, which an earlier capture had put at 5).
const ICON_DY: i32 = 6;
/// Plot map hit rects (map-local x1, y1, x2, y2, place), rules-29 §2 (EXE 0x86E88..0x86F50).
const MAP_RECTS: [(i32, i32, i32, i32, u8); 25] = [
    (248, 152, 283, 182, 0), (44, 154, 76, 184, 1), (107, 106, 140, 131, 2), (79, 223, 111, 248, 3), (306, 68, 341, 101, 4),
    (410, 86, 441, 113, 5), (409, 155, 436, 189, 6), (373, 230, 415, 260, 7), (257, 270, 286, 295, 8), (335, 167, 368, 193, 9),
    (364, 114, 419, 168, 14), (231, 198, 288, 242, 15), (199, 114, 259, 159, 16), (188, 94, 212, 110, 17), (78, 188, 112, 209, 18),
    (303, 229, 342, 269, 19), (389, 184, 415, 217, 20), (32, 67, 171, 249, 10), (171, 67, 225, 217, 10), (239, 65, 449, 121, 11),
    (369, 121, 449, 223, 11), (267, 180, 369, 225, 12), (201, 225, 449, 345, 12), (239, 147, 289, 181, 13), (289, 128, 365, 185, 13),
];
/// Highlight sprite origins per place 0..20, map-local (rules-29 §4, EXE 0x86E34/0x86E5E).
const MAP_HL: [(i32, i32); 21] = [
    (248, 152), (44, 154), (107, 106), (79, 223), (306, 68), (410, 86), (415, 161), (373, 230), (257, 270), (335, 167),
    (31, 65), (221, 63), (191, 167), (31, 62), (364, 114), (231, 198), (199, 114), (188, 94), (78, 188), (303, 229), (389, 184),
];
const NEXT_PLAYER_BG: u16 = 0xBB;
const KUPPLERIN_BG: u16 = 0xBD;
const GEM_DY: i32 = 0;
const BOOK: u16 = 0xD4;
/// Small goods sprites `0x14A + good` (`ICONKA`…), drawn at (text x, line y − 9) (§9.3).
const GOOD_SPRITE: u16 = 0x14A;

/// Banner items (§9.1): gold display, then the whole banner.
const GOLD_RECT: [i32; 4] = [18, 13, 220, 24];
const BANNER_RECT: [i32; 4] = [1, 0, 640, 72];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Spot {
    Truppen,
    Schreibstube,
    Spielstand,
    Bezahlung,
    Hinterzimmer,
    Handel,
    Kirche,
    GiveUp,
}

type SpotDef = (Spot, u32, (i32, i32), [i32; 4]);

/// Kontor label/hotspot pairs (§9.2): label string, label position, hotspot rect. A hotspot click
/// acts as its label (RE's recommended reading). Label x is taken as the text centre ([UNVERIFIED]:
/// "Geld zum Fenster hinauswerfen" at x = 464 only fits on screen that way).
const SPOTS: [SpotDef; 8] = [
    (Spot::Truppen, 0xC01FC, (364, 178), [337, 204, 54, 74]),
    (Spot::Schreibstube, 0xBFD6C, (502, 442), [311, 377, 300, 100]),
    (Spot::Spielstand, 0xC0210, (200, 176), [161, 200, 78, 50]),
    (Spot::Bezahlung, 0xC021C, (201, 283), [143, 307, 112, 78]),
    (Spot::Hinterzimmer, 0xBFD28, (86, 254), [39, 254, 84, 70]),
    (Spot::Handel, 0xC0228, (64, 366), [0, 350, 280, 130]),
    (Spot::Kirche, 0xBFFF0, (488, 169), [449, 192, 77, 108]),
    (Spot::GiveUp, 0xC0240, (464, 128), [576, 180, 62, 154]),
];
const KIRCHENEINTRITT: u32 = 0xC0230;

fn hit([x, y, w, h]: [i32; 4], (mx, my): (i32, i32)) -> bool {
    mx >= x && mx < x + w && my >= y && my < y + h
}

/// 500000 -> "500.000": the EXE's `%d` puts thousands dots into every number.
fn dotted(n: impl std::fmt::Display) -> String {
    let s = n.to_string();
    let (sign, s) = s.split_at(usize::from(s.starts_with('-')));
    let mut out = sign.to_string();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(c);
    }
    out
}

/// Plain text to CP437 with markup characters escaped.
/// Marks a `$h` highlight toggle inside a plain line (see [`plain`]).
const HL: char = '\u{1}';

fn t(s: &str) -> Vec<u8> {
    let mut out = vec![];
    for c in s.chars().filter_map(|c| if c == HL { Some(0x01) } else { cp437(c) }) {
        if c == 0x01 {
            out.extend(b"$h");
            continue;
        }
        if c == b'$' || c == b'%' {
            out.push(c);
        }
        out.push(c);
    }
    out
}

/// EXE markup to one plain line: `$` codes dropped except `$h` (kept as [`HL`]), `$$` and `%%` unescaped.
fn plain(src: &[u8]) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < src.len() {
        match src[i] {
            b'$' if src.get(i + 1) == Some(&b'$') => {
                out.push('$');
                i += 2;
            }
            b'$' => {
                i += 1;
                let start = i;
                while src.get(i).is_some_and(u8::is_ascii_digit) {
                    i += 1;
                }
                // vertical gaps and line ends separate words
                if matches!(src.get(i), Some(b'Z' | b'N')) && start == i && !out.ends_with(' ') && !out.is_empty() {
                    out.push(' ');
                }
                if start == i && src.get(i) == Some(&b'h') {
                    out.push(HL);
                }
                // `$18c` is the superscript two (m²)
                if &src[start..i] == b"18" && src.get(i) == Some(&b'c') {
                    out.push('²');
                }
                i += 1;
            }
            b'%' if src.get(i + 1) == Some(&b'%') => {
                out.push('%');
                i += 2;
            }
            b => {
                out.extend(crate::input::from_cp437(b));
                i += 1;
            }
        }
    }
    out.trim().to_string()
}

/// Replaces the successive occurrences of `pat` by the successive `vals` (already escaped).
fn subst_seq(src: &[u8], pat: &[u8], vals: &[Vec<u8>]) -> Vec<u8> {
    let (mut out, mut i, mut k) = (vec![], 0, 0);
    while i < src.len() {
        if src[i..].starts_with(pat) && k < vals.len() {
            out.extend_from_slice(&vals[k]);
            k += 1;
            i += pat.len();
        } else {
            out.push(src[i]);
            i += 1;
        }
    }
    out
}

/// Fills the first `%s %s ... %d` triple still open in `src` (a monk line): two strings, then a number.
fn subst_seq_first(src: &[u8], strs: &[Vec<u8>; 2], num: &str) -> Vec<u8> {
    let mut out = src.to_vec();
    for v in strs {
        if let Some(i) = out.windows(2).position(|w| w == b"%s") {
            out.splice(i..i + 2, v.iter().copied());
        }
    }
    if let Some(i) = out.windows(2).position(|w| w == b"%d") {
        out.splice(i..i + 2, num.bytes());
    }
    out
}

/// Replaces the first remaining `%s` of `src` (an offer line of the book dialog).
fn subst_seq_first_one(src: &[u8], with: &[u8]) -> Vec<u8> {
    let mut out = src.to_vec();
    if let Some(i) = out.windows(2).position(|w| w == b"%s") {
        out.splice(i..i + 2, with.iter().copied());
    }
    out
}

/// Fills the first open `%w %s ... %d` ensemble line of the concert dialog.
fn subst_offer(src: &[u8], n: i32, quality: &[u8], price: i32) -> Vec<u8> {
    let mut out = src.to_vec();
    for (pat, val) in [(&b"%w"[..], n.to_string().into_bytes()), (b"%s", quality.to_vec()), (b"%d", price.to_string().into_bytes())] {
        if let Some(i) = out.windows(2).position(|w| w == pat) {
            out.splice(i..i + 2, val);
        }
    }
    out
}

/// Marks a caption with the speech line spoken when it appears (`$<code>S`, ignored by the layout). The code
/// packs the four name characters (base 36) and the number: `speech_load(c0, c1, c2, c3, n)` names
/// `<c0><c1><c2><c3><nnn>.WAV` (rules-16 §1).
fn voice(c: [u32; 4], n: u32) -> Vec<u8> {
    let v = c.iter().fold(0u64, |a, &x| a * 36 + u64::from(x)) * 1000 + u64::from(n);
    format!("${v}S").into_bytes()
}

/// Same for the narrator families written as one number, e.g. 229906 = (0, 2, 2, 9, 906).
fn speech(n: u32) -> Vec<u8> {
    voice([0, n / 100_000, n / 10_000 % 10, n / 1000 % 10], n % 1000)
}

/// Speech volume in percent: the EXE's default (`0x8457C`, rules-25 §2).
const SPEECH_GAIN: u32 = 160;

/// The EXE's speech gain on 8-bit unsigned PCM: `128 + (s - 128) * v / 100`, clipped to a byte (rules-25 §2).
fn speech_gain(mut wav: Vec<u8>, v: u32) -> Vec<u8> {
    let at = wav.windows(4).position(|w| w == b"data").map(|i| i + 8);
    let eight_bit = wav.get(34..36) == Some(&[8, 0]);
    if let (Some(at), true) = (at, eight_bit) {
        for s in &mut wav[at..] {
            *s = (128 + (*s as i32 - 128) * v as i32 / 100).clamp(0, 255) as u8;
        }
    }
    wav
}

/// The WAV name (without extension) a caption asks for, if any.
fn speech_of(src: &[u8]) -> Option<String> {
    const DIGITS: &[u8; 36] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut names = vec![];
    for (i, _) in src.iter().enumerate().filter(|&(_, &b)| b == b'$') {
        let rest = &src[i + 1..];
        let digits = rest.iter().take_while(|b| b.is_ascii_digit()).count();
        if digits == 0 || rest.get(digits) != Some(&b'S') {
            continue;
        }
        let Some(v) = std::str::from_utf8(&rest[..digits]).ok().and_then(|d| d.parse::<u64>().ok()) else { continue };
        let (mut c, n) = (v / 1000, v % 1000);
        let mut name = [0u8; 4];
        for k in (0..4).rev() {
            name[k] = DIGITS[(c % 36) as usize];
            c /= 36;
        }
        names.push(format!("{}{n:03}", String::from_utf8_lossy(&name)));
    }
    (!names.is_empty()).then(|| names.join("+"))
}

/// A caption box that closes by itself (ours, `$<n>T`): `n` ms after its speech ends, or 2 s + `n` when no speech plays
/// (the callers' "idle → delay(2000), else until idle", rules-42 §2); clicks do nothing.
fn hold_of(src: &[u8]) -> Option<u32> {
    src.iter().enumerate().filter(|&(_, &b)| b == b'$').find_map(|(i, _)| {
        let d = src[i + 1..].iter().take_while(|b| b.is_ascii_digit()).count();
        (d > 0 && src.get(i + 1 + d) == Some(&b'T')).then(|| std::str::from_utf8(&src[i + 1..i + 1 + d]).ok()?.parse().ok())?
    })
}

/// Play length of a WAV in ms, from its header.
fn wav_ms(w: &[u8]) -> u32 {
    let rate = w.get(28..32).map_or(0, |b| u32::from_le_bytes(b.try_into().unwrap()));
    let data = w.windows(4).position(|x| x == b"data").map_or(0, |i| w.len().saturating_sub(i + 8) as u64);
    if rate == 0 { 0 } else { (data * 1000 / u64::from(rate)) as u32 }
}

/// The lines of a `+`-joined speech name as one WAV: the first header with the data of all lines appended
/// (the original plays them back to back, rules-27 §2).
fn join_wavs(mut wavs: Vec<Vec<u8>>) -> Option<Vec<u8>> {
    let data = |w: &[u8]| w.windows(4).position(|x| x == b"data").map(|i| (i + 8, i + 8 + u32::from_le_bytes(w[i + 4..i + 8].try_into().unwrap()) as usize));
    let mut out = wavs.first()?.clone();
    let (at, end) = data(&out)?;
    out.truncate(end.min(out.len()));
    for w in wavs.drain(1..) {
        let (a, e) = data(&w)?;
        out.extend_from_slice(&w[a..e.min(w.len())]);
    }
    let n = (out.len() - at) as u32;
    out[at - 4..at].copy_from_slice(&n.to_le_bytes());
    let riff = (out.len() - 8) as u32;
    out[4..8].copy_from_slice(&riff.to_le_bytes());
    Some(out)
}

/// German genitive of a name or place: `+s`, but `'` after s, x, z [UNVERIFIED against `%2N0`/`%g`].
fn genitive(n: &str) -> String {
    if n.ends_with(['s', 'x', 'z', 'ß']) { format!("{n}'") } else { format!("{n}s") }
}

/// The pre-fight message window `win_open(120, 120, 400, 240, 0x8810)` (rules-52 §0.3).
const PREFIGHT_WIN: [i32; 4] = [120, 120, 400, 240];

/// Replaces each `pat` in EXE markup by already-escaped `with`.
/// A scene caption box (rules-15 §0.1): gold text that starts 2 px into the box (live A/B, plague).
fn caption(assets: &Assets, r: [i32; 4], src: &[u8]) -> Window {
    Window::new(assets, r, &[b"$g", src].concat(), &[], &[], &[]).with_text_dy(-10)
}

/// Splits a `print` stream into its `$P` pages (empty ones dropped).
fn split_pages(s: &[u8]) -> Vec<Vec<u8>> {
    let mut out = vec![];
    let mut start = 0;
    let mut i = 0;
    while i + 1 < s.len() {
        if s[i] == b'$' && s[i + 1] == b'$' {
            i += 2;
            continue;
        }
        if s[i] == b'$' && s[i + 1] == b'P' {
            out.push(s[start..i].to_vec());
            start = i + 2;
            i += 2;
            continue;
        }
        i += 1;
    }
    out.push(s[start..].to_vec());
    out.retain(|p| p.iter().any(|b| b.is_ascii_alphanumeric()));
    out
}

fn subst(src: &[u8], pat: &[u8], with: &[u8]) -> Vec<u8> {
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

/// Answers sim questions from a script. The sim asks synchronously in the middle of a command;
/// the UI cannot block there, so when the script runs out on a question we have a dialog for, the
/// call is abandoned by unwinding (`NeedAnswer`), the question is shown, and the command is re-run
/// from the untouched state with the answer appended. This is exact because the sim is
/// deterministic (seeded RNG inside `State`, no clock). Questions without a dialog yet are
/// answered 0 and logged.
struct Scripted<'a> {
    answers: &'a [u32],
    pos: usize,
    log: Vec<String>,
    /// tactic questions past the script are answered "no change" without a dialog (rules-44 §2: the menu opens only on
    /// the hotkey) and kept here with their answer index, except the one at `stop`, where the hotkey asked for it
    auto: Vec<(usize, Ask)>,
    stop: Option<usize>,
    /// the events the sim offered to show before its next question (`Choices::show`)
    early: Vec<SimEvent>,
}

struct NeedAnswer(Ask);

fn has_dialog(q: &Ask) -> bool {
    matches!(q, Ask::BattleIntro { .. } | Ask::PreFight { .. } | Ask::OccupyCastle { .. } | Ask::Tactics { .. } | Ask::Match { .. } | Ask::ContinueAfterMission { .. } | Ask::Gift { .. } | Ask::GiveUpCourtship { .. } | Ask::Vows { .. } | Ask::WeddingFeast { .. } | Ask::BlackmailCanon { .. } | Ask::CanonVeto { .. } | Ask::ContinueSiege { .. } | Ask::SiegeWon { .. } | Ask::Vote { .. } | Ask::LeaderLost { .. } | Ask::JuryBribe { .. } | Ask::TrialWitnesses { .. } | Ask::TrialBribe { .. } | Ask::FencingLesson { .. } | Ask::JudgeVote { .. } | Ask::Sentence { .. } | Ask::ContestEntry { .. } | Ask::AuctionCatalog { .. } | Ask::AuctionLine { .. } | Ask::AttendAuction { .. } | Ask::ResidenceSwap { .. } | Ask::AuctionTick { .. } | Ask::AuctionRefused { .. } | Ask::Building { .. } | Ask::Plea | Ask::LoanOffer { .. } | Ask::BlackmailVerdict { .. } | Ask::BuyCastle { .. } | Ask::ConfirmCastleSale { .. } | Ask::AcceptDuel { .. } | Ask::Apologise { .. } | Ask::PickDirector { .. } | Ask::Plot { kind: 32 | 33 | 34 | 36 | 37 | 38 | 39 | 40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 | 48 | 49, .. })
}

impl Choices for Scripted<'_> {
    fn ask(&mut self, q: &Ask) -> u32 {
        if !has_dialog(q) {
            self.log.push(format!("(Frage automatisch beantwortet: {q:?})"));
            // no dialog yet: decline what would spend money or start a duel, else the first option
            return 0;
        }
        if matches!(q, Ask::Tactics { .. }) && self.pos >= self.answers.len() && self.stop != Some(self.pos) {
            self.auto.push((self.pos, q.clone()));
            self.pos += 1;
            return 0;
        }
        let Some(&a) = self.answers.get(self.pos) else {
            // resume_unwind skips the panic hook: this is control flow, not an error.
            std::panic::resume_unwind(Box::new(NeedAnswer(q.clone())));
        };
        self.pos += 1;
        a
    }

    fn show(&mut self, ev: &[SimEvent]) {
        self.early = ev.to_vec();
    }
}

/// A command waiting for the player's answers (see [`Scripted`]); `None` = `State::run`.
struct Pending {
    cmd: Option<Command>,
    answers: Vec<u32>,
    ask: Ask,
}

/// Menus reachable from the Kontor (`menus.rs`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Menu {
    /// Privilege dialogs (rules-38 §2): city pick for 9 / 28 / 29, army size for 28 / 29, insurance, the cost
    /// confirmation of a code, law book edit mode of a book, Projekte unterstützen.
    PrivCity(u8),
    PrivArmy(u8),
    Insurance,
    PrivAsk(u8),
    LawEdit(u8),
    Project,
    Spielstand,
    Save,
    Load,
    City(u8),
    Storage(usize),
    Transport(usize, u8),
    Apply,
    /// The applicants for one office (rules-64 §2): (place, idx).
    Applicants(u8, u8),
    Targets(Plot),
    Listener(u8),
    BribeAmount(u8),
    Candles,
    Alms,
    Pray,
    Orders(usize),
    /// Information window of a castle or camp by place index 0..7 (rules-26 §2).
    CastleInfo(u8),
    /// Recruiting, manoeuvre and toll windows of a castle or camp (rules-26 §5.2..§5.4).
    Recruit(usize),
    Manoeuvre(usize),
    Toll(usize),
    /// Condition window of a castle or camp (rules-26 §5.1) and its improvement dialog (`which` 0 security, 1 casemates).
    Expand(usize),
    Improve(usize, u8),
    CastleSell(u8),
    Quit,
    /// The pending sim question (`Game::pending`).
    Question,
    Directors,
    Residence(usize),
    /// Residence sub-lists: buy offers, build levels, extensions.
    ResBuy(usize),
    ResBuild(usize),
    ResExtend(usize),
    ResAuction(usize),
    Director(usize),
    JoinChurch,
    /// Matchmaker intro on `ICONBD` (EXE 0x9D9F9); the offer is the `Ask::Match` question.
    Kupplerin,
    /// Name of a newborn child (EXE 0xBAA4C): the child's name slot.
    Naming(u8),
    /// Konvertieren / Austreten confirmations (EXE 0xA8527, 0xAB732).
    Convert,
    Leave,
    Indulgence,
    /// Fest geben (EXE 0x9DDC4..0x9DF77): size, musicians, branch number among own, years ahead.
    Feast([u8; 4]),
    /// Mein Testament (EXE 0xA9605, rules-05 §7.1).
    Testament,
    /// Law book browser (rules-04b §1.1 mode 3): 0 finance, 1 criminal, 2 church.
    Laws(u8),
    /// Spieloptionen sub-pages (rules-00 option flags, EXE 0xBE845..).
    Options,
    /// Modern options (ours, `prefs.rs`) by page, opened from the bottom of the Optionen window.
    Extras(u8),
    /// Ours: the ledger (page 0 net worth and cash by year, 1 branch results) and the price table (`prefs.ledger`,
    /// `prefs.price_table`).
    Ledger(u8),
    Prices,
    /// Ours (`prefs.shortcuts`): the key list.
    Keys,
    /// Ours (`prefs.event_log`): the session's report lines, newest first, by page (Kontor key L).
    Log(u16),
    /// Ours (`prefs.production_help`): the production help of branch `b` (key H on the branch screen).
    ProdHelp(usize),
    /// The player's privileges (rules-04b §4.1).
    Privileges,
    Music,
    Graphics,
    /// Office tree of one place (rules-04 §2): plot, location, church level.
    Tree(Plot, u8, bool),
}

/// Back-room actions that need a target person (spec 04, rules-04).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Plot {
    Spies,
    Saboteur,
    Blackmail,
    Denounce,
    Bribe,
    /// Privilege 12 / 26 (rules-04b: office tree modes 5 and 6).
    Trial,
    Insult,
    /// Anschwärzen, second step (rules-29 §3 mode 12): the listener for this victim.
    DenounceTo(u8),
    /// A privilege's person pick (rules-29 modes 0, 7, 8, 9): the code; Zwietracht's second victim after the first.
    Priv(u8),
    DiscordTo(u8),
}

impl Plot {
    /// Title verb on the office tree (EXE 0xA2D..: Bestechung, Erpressung, Spionage, Sabotage).
    fn verb(self) -> &'static str {
        match self {
            Plot::Spies => "Spionage",
            Plot::Saboteur => "Sabotage",
            Plot::Blackmail => "Erpressung",
            Plot::Denounce => "Das Opfer", // map mode 11 (rules-29 §3)
            Plot::Bribe => "Bestechung",
            Plot::Trial => "Prozess",
            Plot::Insult => "Beleidigung",
            Plot::DenounceTo(_) => "Anschwärzen",
            Plot::Priv(81) => "Inquisition",
            Plot::Priv(80) => "Das 1. Opfer",
            Plot::Priv(_) => "Wählt",
            Plot::DiscordTo(_) => "Das 2. Opfer",
        }
    }
}

/// Editable numbers and words. Numbers step by the clicked digit's place value, words cycle; the
/// upper half of the item goes up, the lower half down (spec 01).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Field {
    Bribe,
    /// Director budget of a branch (Direktoren window, §10.2).
    Budget(usize),
    /// Amount typed into a sim question (ransom, helper bribe).
    Amount,
    /// Discipline mask of the contest entry (toggle of one bit per click).
    Mask,
    /// Cycling choice of `n` entries in `self.amount`; the flag skips entry 3 (plea, sentence, rules-18 §4, §10).
    Choice(u8, bool),
    Toll(usize),
    Order(usize, u8, OrderField),
    /// A window's own spinner `amts[slot]`, limited to 0..=max.
    Amt(u8, u32),
    /// A privilege setting (rules-38 §2.2): the step goes to the sim as the delta of `code`.
    Priv(u8),
    /// The city of a privilege dialog (9 all, 28 unbesieged, 29 besieged by others), cycling.
    PrivCity(u8),
    /// The number of a privilege dialog (`priv_n`), within lo..=hi.
    PrivN(u32, u32),
    /// Law book edit mode: line i of a book moves by the step (rules-38 §3).
    Law(u8, u8),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum OrderField {
    Kind,
    Good,
    Workers,
    Means,
    Qty,
    Trans,
    Dest,
    /// Castle/camp order book (rules-07 §3a): type word, target word, troop count of slot j.
    TroopKind,
    TroopTarget,
    Troop(u8),
}

impl Field {
    fn is_word(self) -> bool {
        matches!(self, Field::Choice(..) | Field::Order(_, _, OrderField::Kind | OrderField::Good | OrderField::Trans | OrderField::Dest | OrderField::TroopKind | OrderField::TroopTarget))
    }
}

/// One Spieloptionen setting.
#[derive(Clone, Copy, Debug)]
enum Opt {
    /// Option flag bit (CONFIG.DAT dword 0x17D).
    Flag(u32),
    /// Volume bar (digital, music, speech) by a step.
    Vol(usize, i8),
    Voice,
    Film,
    Scroll,
    /// A modern option (`prefs.rs`), by its key.
    Pref(Pref),
}

/// Keys of the Extras window, in display order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pref {
    Fullscreen,
    IntegerScale,
    Speed,
    Autosave,
    Undo,
    Tooltips,
    Shortcuts,
    Warnings,
    Lang,
    ProductionHelp,
    DebtMercy,
    MoreWorkers,
    SeedChoice,
    SellAll,
    Ledger,
    PriceTable,
    EventLog,
    OrderCopy,
    Confirmations,
}

#[derive(Clone, Debug)]
enum Action {
    Close,
    EndTurn,
    /// Ours ("Rückgängig"): back to the state before the last turn end.
    Undo,
    /// "Neuer Spieler" (Spieloptionen): the player creation on the running game.
    NewPlayer,
    Quit,
    GiveUp,
    Open(Menu),
    /// Apply a sim command, then show the menu again (or close).
    Sim(Command, Option<Menu>),
    Step(Field, i32),
    /// Change a Spieloptionen setting, then show its page again.
    Opt(Opt, Menu),
    /// Hire spies on a person: run the command, then the EXE's confirmation (0xBC904).
    Spied(u8, u32),
    /// Evidence overview of a spy contract (EXE 0xBC8BC).
    Evidence(u8),
    /// Blackmail a person (all outcomes are sim message boxes, rules-04 §5.2).
    Blackmail(u8),
    /// Medicus: the doctor's verdict (EXE 0xA1E09, health lines from 0xA1E50).
    Medicus,
    /// The credits without a game save (the mission's quit box, rules-39 §8).
    Credits,
    /// Privileges menu: the other page, or the next holder whose privileges the player may use (rules-38 §1.3, §1.4).
    PrivPage,
    PrivHolder(u8),
    /// Draw the dialog's cost in the sim, then open its confirmation.
    PrivView(u8),
    /// Run privilege `code` with (a, b) for the current holder.
    Priv(u8, i32, i32),
    /// Saboteur offer or, when they already run, the recall dialog (EXE 0xBCA0C / 0xBC994).
    Sabotage(u8, u32),
    ToggleOrder(usize, u8),
    Branch(usize),
    Save(u8),
    Load(u8),
    /// Invest `amount` Taler in recruiting at property `p`, then reopen the window.
    Recruit(usize, u16),
    /// Improve security or casemates of property `p` by `amount`, then reopen the dialog.
    Fortify(usize, u8, u8),
    /// Dismiss men of a slot from the castle screen.
    Dismiss(usize, u8, u8),
    /// Hold a manoeuvre at property `p` and tell which troop type improved.
    Manoeuvre(usize),
    /// Answer to the pending sim question.
    Answer(u32),
    /// Start typing the name of save slot k.
    SaveName(u8),
    /// Select bullet k of the open list (icon 0x74), or clear it when it is selected (rules-34 §3.1, rules-35 §1).
    Select(u32),
    /// An item that does nothing: the gems of menus answered earlier in the same turn-end window.
    Inert,
    /// Confirm with the selected bullet: run the k-th of these actions, nothing without a selection.
    /// (`true`: close when nothing is selected, Wohnsitz bauen; `false`: stay, director and Wohnsitz kaufen)
    WithSel(Vec<Action>, bool),
    /// Two answers at once (the trial investment dialog asks witnesses and court in one window).
    AnswerPair(u32, u32),
    /// Enter a §10 room, or go back to the Kontor (`None`).
    Room(Option<Room>),
    /// Show an EXE string in a window.
    Text(u32),
    /// Show ready EXE text bytes in a window.
    Bytes(Vec<u8>),
    /// A message in a box at the given rect.
    Boxed(Vec<u8>, [i32; 4]),
    /// Ours (pref `sell_all`): sell every branch's stock locally, one `SellStock` per branch as by hand, then a summary.
    SellAll,
    /// An EXE question with `%a` items at the given rect, item k runs action k.
    Inline(Vec<u8>, Vec<Action>, [i32; 4]),
    /// Ja/Nein dialog from an EXE string ending in `%m %m`; Ja runs the action.
    Confirm(u32, Box<Action>),
    /// Same with ready text (substituted) and the box rectangle.
    YesNo(Vec<u8>, [i32; 4], Box<Action>),
    /// Same with ready text (substituted) and the box rectangle.
    ConfirmText(Vec<u8>, [i32; 4], Box<Action>),
    /// Ours (pref `confirmations`): the action after its safety question was answered with Ja.
    Sure(Box<Action>),
    /// Sell stock locally (spec 04 §5.1.2.1): good (0xFF = everything), quantity (0 = all of it).
    Sell(usize, u8, u16),
    Nothing,
}

/// One step of the election screen: a strip line at y 421, a coin for candidate `coin`, and whether it ends after
/// 180 ticks (else only on a click).
struct ElectStep {
    strip: Vec<u8>,
    coin: Option<usize>,
    timed: bool,
    /// a human elector picks a candidate line here (the sim's `Ask::Vote`)
    ask: bool,
}

/// rules-15 §3.4: everything is printed straight onto the place picture; the votes come one by one.
struct Elect {
    /// (place, seat): the vote question and the finished election share one screen
    key: (u8, u8),
    pic: u16,
    head: Vec<u8>,
    names: Vec<Vec<u8>>,
    steps: Vec<ElectStep>,
    pos: usize,
    ms: u32,
}

impl Elect {
    /// The wait of a timed step: 180 ticks of `menu`'s timeout (rules-15 §3.4) read as 1/100 s [UNVERIFIED unit].
    const TIMEOUT_MS: u32 = 1800;

    /// Next step; false when the screen is done.
    fn advance(&mut self) -> bool {
        self.pos += 1;
        self.ms = 0;
        self.pos < self.steps.len()
    }

    /// The candidate line under the pointer (bullet at x 120, name from 150, 24 px per line).
    fn line_at(&self, (x, y): (i32, i32)) -> Option<usize> {
        let k = (y - 150 + 4).div_euclid(24);
        ((100..560).contains(&x) && (0..self.names.len() as i32).contains(&k)).then_some(k as usize)
    }

    /// Takes the steps of the same election once more (the finished one after a human vote): the screen goes on
    /// where it stands, past the vote just made.
    fn continue_with(&mut self, next: Elect) {
        let past_ask = self.steps.get(self.pos).is_some_and(|st| st.ask);
        (self.steps, self.names, self.head) = (next.steps, next.names, next.head);
        if past_ask {
            self.pos += 1;
            self.ms = 0;
        }
        self.pos = self.pos.min(self.steps.len().saturating_sub(1));
    }

    fn coins(&self) -> Vec<usize> {
        let mut n = vec![0; self.names.len()];
        for st in &self.steps[..=self.pos.min(self.steps.len() - 1)] {
            if let Some(c) = st.coin.filter(|&c| c < n.len()) {
                n[c] += 1;
            }
        }
        n
    }

    fn draw(&self, frame: &mut Frame, assets: &Assets, ticks: u32, mouse: (i32, i32)) {
        frame.blit(&assets.icon(self.pic)[0], 0, 0);
        caption(assets, [0, 0, 640, 480], &self.head).draw_text(frame, assets);
        let coin = assets.icon(0x19C);
        for (i, (name, n)) in self.names.iter().zip(self.coins()).enumerate() {
            let y = 150 + 24 * i as i32;
            // bullet item (style 0x90, glyph 0x73) at x 120, the name at x 150 (rules-15 §3.4 step 2)
            // the gems show only while a human elector picks (live A/B 2026-11-22: not before, not after his vote, not
            // when he only applies); the line under the pointer lights its gem (0x74), as every gem does
            if self.steps.get(self.pos).is_some_and(|st| st.ask) {
                let lit = self.line_at(mouse) == Some(i);
                frame.blit(&assets.icon(if lit { 0x74 } else { 0x73 })[0], 120, y + 3);
            }
            // the caption box prints 12 px in: the name starts at x 150 (live A/B 2026-11-22: ours was at 162)
            caption(assets, [138, y - 2, 502, 24], name).draw_text(frame, assets);
            // coins stack right to left in front of the bullet, at most six (step 4)
            for k in 0..n.min(6) as i32 {
                if let Some(img) = coin.get((ticks as usize / 7 + k as usize) % coin.len().max(1)) {
                    frame.blit(img, 96 - 18 * k, 153 + 24 * i as i32);
                }
            }
        }
        if let Some(st) = self.steps.get(self.pos).filter(|st| !st.strip.is_empty()) {
            caption(assets, [0, 419, 640, 60], &st.strip).draw_text(frame, assets);
        }
    }
}

/// Markup with clickable items, built up piece by piece.
#[derive(Default)]
struct Form {
    /// Choices of an EXE dialog get the blue gem bullet (`ICON73`).
    gems: bool,
    src: Vec<u8>,
    items: Vec<Vec<u8>>,
    acts: Vec<Action>,
}

impl Form {
    fn new(src: &[u8]) -> Self {
        Self { src: src.to_vec(), ..Self::default() }
    }
    fn raw(&mut self, s: &[u8]) -> &mut Self {
        self.src.extend_from_slice(s);
        self
    }
    fn txt(&mut self, s: &str) -> &mut Self {
        self.src.extend(t(s));
        self
    }
    fn item(&mut self, label: Vec<u8>, a: Action) -> &mut Self {
        self.src.extend_from_slice(b"%m");
        self.items.push(label);
        self.acts.push(a);
        self
    }
    /// An item whose `%m` is already in the EXE text.
    fn bind(&mut self, label: Vec<u8>, a: Action) -> &mut Self {
        self.items.push(label);
        self.acts.push(a);
        self
    }
    /// `n` digit items for `v`, the most significant first.
    fn digits(&mut self, v: u32, n: u32, f: Field) -> &mut Self {
        for k in (0..n).rev() {
            let d = v / 10u32.pow(k) % 10;
            // thousands dot, as in the original's number items (live: recruiting shows "0.000"); a click on a
            // dot steps the digit after it (A/B: the order quantity), so the dot belongs to that digit's item
            let dot = k + 1 < n && (k + 1) % 3 == 0;
            let mut text = if dot { vec![b'.'] } else { vec![] };
            text.push(b'0' + d as u8);
            self.item(text, Action::Step(f, 10i32.pow(k)));
        }
        // the item reserves ((n−1)·adv['.'])/3 + n·adv['0'] (formats §8.6); digits are 15 px, the dot 6:
        // the part not drawn as dots stays empty after the number
        let pad = (2 * (n as i32 - 1) - 6 * ((n as i32 - 1) / 3)).max(0);
        if pad > 0 {
            self.src.extend(format!("${pad}}}").bytes());
        }
        self
    }
    /// `%a Label` with label bytes already in EXE text form.
    fn item_bullet_raw(&mut self, label: &[u8], a: Action) -> &mut Self {
        self.gems = true;
        self.src.extend_from_slice(b"$L$2>");
        self.item(b"   ".to_vec(), a);
        // `$B`: a wrapped label is justified (rules-47 §1; live A/B, the ship raid's "Gnade vor Recht")
        self.src.extend_from_slice(b"$E$B$6>");
        self.src.extend_from_slice(label);
        self
    }
    /// `%a Label`: a bullet item and the label as plain text.
    fn bullet(&mut self, label: &[u8], a: Action) -> &mut Self {
        let esc: Vec<u8> = label.iter().flat_map(|&b| if b == b'$' || b == b'%' { vec![b, b] } else { vec![b] }).collect();
        self.item_bullet_raw(&esc, a)
    }
    /// `%j Label`: the icon is the item (drawn by the popup code at the item's start), the label plain text.
    fn icon_item(&mut self, label: &str, a: Action) -> &mut Self {
        // the icon item reserves its 32 px (formats §8.6 class 0x90); `%j Label` has a space before the label
        self.item(vec![text::PAD_MARK, 32], a).txt(&format!(" {label}"))
    }
    /// EXE dialog text whose `%a` choices carry their labels inline (`%aJa.%aNein.$|`): each
    /// becomes an item with the given action.
    fn inline(&mut self, src: &[u8], acts: Vec<Action>) -> &mut Self {
        let mut parts = src.split(|&b| b == b'%');
        self.src.extend(parts.next().unwrap_or_default());
        let mut acts = acts.into_iter();
        self.gems = true;
        for p in parts {
            match p.first() {
                Some(b'a') => {
                    // `%a` = `$L$2>%m$E$B$6>` (rules-28 §4): only the bullet is the item, the text is plain
                    // (live A/B: hovering or clicking the text does nothing in the original)
                    let start = if p[1..].starts_with(b"$L") { 3 } else { 1 };
                    let end = p[start..].iter().position(|&b| b == b'$').map_or(p.len(), |k| k + start);
                    self.item_bullet_raw(&p[start..end], acts.next().unwrap_or(Action::Close));
                    if !p[end..].starts_with(b"$N") {
                        self.src.extend_from_slice(b"$N");
                    }
                    self.src.extend_from_slice(&p[end..]);
                }
                _ => {
                    self.src.push(b'%');
                    self.src.extend_from_slice(p);
                }
            }
        }
        self
    }
    fn popup(self, assets: &Assets, x: i32, w: i32, menu: Option<Menu>) -> Popup {
        let refs: Vec<&[u8]> = self.items.iter().map(Vec::as_slice).collect();
        let h = (text::layout(&assets.font, &self.src, &refs, &[], w, &[]).height + if self.gems { 11 } else { 12 }).clamp(80, 400);
        // the EXE sizes text windows in 40 px steps (Spielstand formula, rules-28 §5; save and lender boxes measured)
        let h = (h + 39) / 40 * 40;
        Popup::Choice { rect: [x, 240 - h / 2, w, h], src: self.src, items: self.items, acts: self.acts, menu, gems: self.gems }
    }
    fn layout_height(&self, assets: &Assets, w: i32) -> i32 {
        let refs: Vec<&[u8]> = self.items.iter().map(Vec::as_slice).collect();
        text::layout(&assets.font, &self.src, &refs, &[], w, &[]).height
    }
    /// A popup at a fixed rectangle (documented dialogs).
    fn popup_at(self, rect: [i32; 4]) -> Popup {
        Popup::Choice { rect, src: self.src, items: self.items, acts: self.acts, menu: None, gems: self.gems }
    }
}

enum Popup {
    /// Region map for choosing where to run a plot (Hinterzimmer, original: map with outlines).
    Map(Plot),
    /// Playback of a fight, then the report behind it.
    Battle(Box<battle::BattleView>),
    /// Full-screen `ICONBB` with the announce text (§9.3).
    NextPlayer(Vec<u8>),
    /// Book spread: produced (good, amount), export revenue (None = nothing sold), given away.
    Ertraege(Vec<(u8, u32)>, Option<i64>, u32),
    /// Our event list for everything §9 has no screen for; paged by measured height.
    Report(Vec<String>),
    /// The year-end report (ICONB8, window (100, 126, 440, 300), banner "Ereignisse in Mittelland").
    YearReport(Vec<String>),
    /// Chronicle of the year (rules-06 §7) on the year-end scroll; scrolls like a report (rules-42 §1).
    Chronicle(Vec<u8>),
    /// The charter of a new title (rules-19 §3): desk picture, parchment window with the text, seal at the bottom.
    Award(Vec<u8>),
    /// The testament read at the funeral (rules-17 §5): parchment window (40, 100, 560, 280) over the notary picture.
    Will(Vec<u8>),
    /// The bill page that opens a turn-end report (EXE 0xC06B0..0xC0850), markup ready for the window.
    /// The second field is the cash the banner shows on it (right after paying).
    Bill(Vec<u8>, i32),
    /// The election screen (rules-15 §3.4): place picture, candidates, votes with coins, result.
    Election(Box<Elect>),
    /// A full-screen picture with caption boxes shown one after another, each closed by a click (rules-15 §0.1).
    Scene { pic: u16, boxes: Vec<([i32; 4], Vec<u8>)> },
    /// The court room (ICONC6) with its figures (icon, x, y) and caption boxes (rules-15 §6.3).
    /// A film (FLCFILME) playing over the screen until it ends or is skipped.
    Film(Box<super::film::Film>),
    /// `anim`: frame range all figures step through, one frame per 100 ms (rules-18 §2)
    Court { pic: u16, figures: Vec<(u16, i32, i32)>, boxes: Vec<([i32; 4], Vec<u8>)>, anim: Option<(usize, usize)> },
    Choice { rect: [i32; 4], src: Vec<u8>, items: Vec<Vec<u8>>, acts: Vec<Action>, menu: Option<Menu>, gems: bool },
}

impl Popup {
    fn with_menu(mut self, m: Menu) -> Self {
        if let Popup::Choice { menu, .. } = &mut self {
            *menu = Some(m);
        }
        self
    }

    fn window(&self, assets: &Assets, lit: &[usize]) -> Option<Window> {
        Some(match self {
            Popup::Report(lines) => report_window(assets, lines),
            Popup::Bill(src, _) => Window::new(assets, REPORT_BOX, src, &[], &[], &[]).with_scroll(),
            Popup::Scene { boxes, .. } | Popup::Court { boxes, .. } => caption(assets, boxes[0].0, &boxes[0].1),
            Popup::Election(_) => return None,
            Popup::Award(src) => Window::new(assets, [80, 20, 400, 440], src, &[], &[], &[]),
            Popup::Will(src) => Window::new(assets, [40, 100, 560, 280], src, &[], &[], &[]),
            Popup::Chronicle(src) => Window::new(assets, YEAR_BOX, src, &[], &[], &[]).with_scroll(),
            Popup::YearReport(lines) => Window::new(assets, YEAR_BOX, &report_src(lines), &[], &[], &[]).with_scroll(),
            Popup::Choice { rect, src, items, gems, .. } => {
                let refs: Vec<&[u8]> = items.iter().map(Vec::as_slice).collect();
                let w = Window::new(assets, *rect, src, &refs, &[], lit).with_text_dy(if *gems { GEM_DY } else { 0 });
                // a menu printed into the turn-end report window keeps its scroll frame and is scrolled up until its
                // last line ends at the window bottom, to the pixel (rules-63 §2.3; live A/B 2026-11-22: 94 px)
                if *rect == REPORT_BOX {
                    let over = (w.laid.height - rect[3]).max(0);
                    w.with_scroll().with_text_dy(GEM_DY - over)
                } else {
                    w
                }
            }
            _ => return None,
        })
    }
}

pub struct Game {
    st: State,
    /// Ours: the state before the human's last turn end, kept with the Rückgängig option for one undo.
    undo: Option<Box<State>>,
    /// Ours: the game number this game was started with (not in the save, so unknown after a load).
    pub(super) seed: Option<u32>,
    tables: Tables,
    branch: Option<usize>,
    room: Option<Room>,
    popups: VecDeque<Popup>,
    /// Kontor entered this turn (for the entry music roll).
    entered: bool,
    /// sample effects waiting to be played (docs/re/rules-14), drained in `update`
    sounds: Vec<u8>,
    /// Duel effect samples still to play, one list per taunt box (rules-25 §2): played when that box is clicked away.
    duel_fx: std::collections::VecDeque<Vec<(u16, u8)>>,
    /// Timed samples running: (ms left, sample).
    fx_due: Vec<(u32, u8)>,
    /// the speech caption last played, so a caption speaks once when it comes to the front
    spoken: Option<(usize, String)>,
    /// Picture of the open manoeuvre window (drawn once per opening).
    mv_pic: u16,
    /// Order book shown on the castle/camp screen, per place (static byte R+0x19, not saved: castles 1, camps 0, rules-00 §10).
    book_shown: [bool; 7],
    /// City named by the open indulgence offer.
    ablass_city: u8,
    /// Where the last Kontor hotspot was clicked: its label stays under the dialog it opened.
    spot_click: Option<(i32, i32)>,
    /// Spielstand: the viewer's tax and tithe (rules-28 §2) and the bar width per row.
    scores_tt: (i32, i32),
    /// The year that just ended, for the banners of its reports (see `shown_year`).
    report_year: Option<u16>,
    /// Office list favour bars (x, y, width), rules-31 §5.
    tree_bars: Vec<(i32, i32, i32)>,
    /// formats §8.7: the last hovered item index (0x84D4C, kept across windows) and whether the highlight is off
    /// since a click re-entered the menu unlit; it lights again only when the hovered index changes
    hover: (Option<usize>, bool),
    /// The banner's gold display is a menu item (formats §9.1): lit under the pointer, and it stays lit under the
    /// bribery map it opens, which does not repaint the banner (live A/B)
    gold_hot: bool,
    /// Ours (`prefs.tooltips`): where the pointer rests and for how many ms.
    tip: ((i32, i32), u32),
    /// rules-55 §15: 0x85CC0 "no branch visited this turn", set at the turn start, cleared when a branch screen opens
    /// (kept app-side: the replay rebuilds the sim state from commands, and opening a branch is no command)
    unvisited: bool,
    /// The rival's entry action in the office list (drawn highlighted at rest).
    tree_rival: Option<String>,
    /// Spinner values of the branch sub-windows (rules-34), reset when a window opens from the branch screen.
    amts: [u32; 4],
    /// The pre-fight window's page text before its question (rules-52).
    page: Vec<u8>,
    /// The open pre-fight question is the bribe menu (a re-ask after a commit keeps the other rows' amounts).
    bribing: bool,
    /// Selected bullet of a select-then-confirm list and the menu it belongs to.
    sel: Option<u32>,
    sel_menu: Option<Menu>,
    /// Icons of `%j` items for the open window: (menu, item index, icon id).
    form_icons: Vec<(Menu, usize, u16)>,
    /// The transporter window's picture (rules-34 §4).
    trans_pic: (Menu, u16),
    score_bars: Vec<i32>,
    /// Ours (`prefs.ledger`): one entry per finished year of a human, in order; empty after a load.
    ledger: Vec<LedgerYear>,
    /// Ours (`prefs.event_log`): (year, line) of every report line shown while the option was on; empty after a load.
    log: Vec<(u16, String)>,
    /// Ours (`prefs.order_copy`): the four order slots copied from a branch screen (key C).
    clip: Option<[[u8; 8]; 4]>,
    /// `BranchCosts` of the running turn end, folded into the ledger at the next turn start.
    branch_costs: Vec<(u8, i32)>,
    /// Fanfare drawn for the last title charter (rules-27 §1).
    award_song: u8,
    /// The herald's fanfare (rules-12 §3.6), playing while the year-end pages are shown.
    herald_song: Option<u8>,
    /// Privileges menu: whose privileges are shown (0xFF the player) and the page (rules-38 §1.3, §1.4).
    priv_holder: u8,
    priv_page: u8,
    /// Privilege dialogs: the cost drawn by `PrivilegeView`, the city picked, the number entered.
    priv_cost: i32,
    priv_city: u8,
    priv_n: u32,
    /// The song the concert drew for the lute picture.
    lute_song: Option<u8>,
    /// The answer index whose tactic question opens the menu on the next run (a hotkey rewound the fight).
    tactic_stop: Option<usize>,
    /// Erträge reveal (rules-50 §3): goods rows shown so far, ms the current harvest line has been up.
    ert_reveal: (usize, u32),
    /// The jury verdict's song (0x0B tower, the drawn fanfare on acquittal).
    jury_song: u8,
    /// The Kontor was left for a room that makes its loop start over (music roll, rules-39 §4).
    kontor_left: bool,
    /// The song and mode a screen's `Command::Music` / `KontorView` rolled, played by `update`.
    room_song: Option<(u8, u8)>,
    /// scanline reveal running over the front scene's picture (rules-16b §8), and the picture it belongs to
    reveal: Option<(u16, super::reveal::Reveal)>,
    revealed: Option<u16>,
    /// kind of the disaster scene being told (its branch and insurance captions use its picture)
    disaster: usize,
    /// time the front court popup has been shown (figure animation), and how many popups were queued then
    court_ms: u32,
    /// Running time of the screen, for idle animations (map flags).
    clock_ms: u32,
    /// Time on the current scene box and its key (popups, boxes left), for `$<n>T` boxes.
    scene_ms: (u32, (usize, usize)),
    /// Length of the speech started last.
    speech_ms: u32,
    /// Time since the current question appeared (drives the auctioneer's hammer frames).
    ask_ms: u32,
    court_len: usize,
    /// Config bytes the turn flow reads (§9.2, §9.3): end-turn flags `0x17D`, announce `0x17E`.
    cfg_end: u8,
    /// Text speed 0..5 (CONFIG.DAT 0x0F3, rules-43 §2): the report scroll step `SCROLL_STEPS[speed]`.
    text_speed: u8,
    cfg_announce: bool,
    /// Option flag dword (CONFIG.DAT 0x17D), volume bars 0..=16, speech, film size, scrolling.
    opts: u32,
    /// Digital, music and speech volume as CONFIG.DAT stores them: 0x194, 0x195 (0..127), 0x278 (0..255) (rules-44 §1.2).
    vol: [u8; 3],
    black_box: bool,
    /// Office-tree prices of the last `Command::OfficeTree` (rules-04 §2), per person slot.
    prices: Vec<u32>,
    bribe: u32,
    /// typed name of the child being named (CP437)
    child_name: Vec<u8>,
    /// Births waiting for a name: slot, announcement (with its speech), cradle sound rolls (rules-19 §2.1).
    cradles: Vec<(u8, Vec<u8>, [u8; 2])>,
    /// Church wedding progress inside one sim command (rules-15 §7.1): 1 = portal and priest shown, 2 = the
    /// vows were asked, 3 = a human canon said he has no objection. Back to 0 when the command commits.
    church: u8,
    /// the player's cash at the point a pending turn-end question was reached (the bill paid): the banner shows it
    pending_cash: Option<i32>,
    /// The turn-end report window so far while its questions are asked (rules-63 §2, rules-13): the lines shown and
    /// the answered menus (`RAW` lines). The original prints the whole turn end into one window; each window here
    /// starts with this and continues at its end.
    window: Vec<String>,
    /// (report window identity, lines at its top that an earlier window of the same turn end already showed)
    continued: Option<(usize, usize)>,
    /// the question open in the turn-end report window (markup, item labels), as last built
    report_menu: Option<(Vec<u8>, Vec<Vec<u8>>)>,
    /// `report` runs for the turn-end lines shown before a question (`Choices::show`), the turn end goes on
    reporting_early: bool,
    /// (human, rival) drawn when the Bewerbung window opened (rules-64 §2)
    apply_rivals: Vec<(u8, u8)>,
    /// (report window identity, cash): the turn-end report's banner cash (`TurnEndCash`)
    turn_cash: Option<(usize, i32)>,
    /// events of the pending command already shown before its question (`Choices::show`)
    early_shown: usize,
    /// The cradle stays until the second baby sound has ended (rules-19 §2.1 step 10), ms left.
    cry_ms: u32,
    scroll: ReportScroll,
    scroll_ms: u32,
    /// Time in the current click wait and the wait it belongs to (see `wait_key`): the wheel cursor's clock.
    wait_ms: (u32, Option<(usize, usize)>),
    /// Save slot whose name is being typed (the text goes to `child_name`).
    save_edit: Option<u8>,
    pending: Option<Pending>,
    amount: u32,
}

/// Ours: a human's year in the ledger. `branches`: (city, sales revenue, wages + means + transport) per branch.
#[derive(Clone, Debug, PartialEq, Eq)]
struct LedgerYear {
    person: u8,
    year: u16,
    worth: i32,
    cash: i32,
    branches: Vec<(u8, i32, i32)>,
}

struct Tables {
    locations: Vec<String>,
    offices: Vec<String>,
    goods: Vec<String>,
    /// order-line words (EXE): verb per good, preposition per good, plural dative of the means
    verbs: Vec<String>,
    preps: Vec<String>,
    means_pl: Vec<String>,
    titles: [Vec<String>; 2],
    /// NPC first names (male, female), indexed by the person's name byte
    first: [Vec<String>; 2],
    /// chronicle text addresses for 1600..1699 (pointer table at 0xB518C; the 1600 entry was live-checked)
    chronicle: Vec<u32>,
    /// office nouns by text id (72 + o male, 1407 + o female), see `Assets::text_id`
    text_ids: std::collections::HashMap<u32, String>,
}

/// Fate kinds whose text needs at most a city and an amount: (kind, text id), rules-12 §3.9.
const FATE_IDS: &[(u8, u32)] = &[
    (0, 3270), (1, 3271), (2, 3272), (3, 3273), (6, 3275), (7, 3276), (8, 3277), (9, 3278), (10, 3279), (20, 3289), (22, 3291), (23, 3292),
    (26, 3293), (27, 3294), (28, 3295), (30, 3297), (31, 3298), (33, 3300), (34, 3301), (36, 3302), (38, 3304), (39, 3305), (40, 3306),
    (42, 3308), (43, 3309), (44, 3310), (49, 3315), (52, 3316), (55, 3319), (58, 3321), (70, 3333), (71, 3334),
    (74, 3337), (76, 3339), (37, 3303), (48, 3314), (69, 3332), (59, 3322), (60, 3323), (79, 3341), (80, 3342), (81, 3343), (83, 3345), (84, 3346), (88, 3350), (89, 3351),
    (15, 3284), (16, 3285), (90, 3352), (91, 3353), (92, 3354), (93, 3355), (96, 3358), (97, 3358),
];

const REPORT_BOX: [i32; 4] = [30, 110, 520, 290];
const YEAR_BOX: [i32; 4] = [100, 126, 440, 300];

/// Leads a report line that is markup kept from a menu answered in the window (the gifts, rules-63 §2), one char
/// per byte; its `%m` items are the gem bullets. The char after it is the chosen bullet ('0'..) or '-'.
const RAW: char = '\u{1}';

/// An answered menu's markup with its values filled in: number and word items become their text, bullets stay
/// items (`%m`) so they keep their gems.
fn answered(src: &[u8], items: &[Vec<u8>]) -> Vec<u8> {
    let (mut out, mut k, mut i) = (vec![], 0, 0);
    while i < src.len() {
        match (src[i], src.get(i + 1)) {
            (b'%', Some(b'%')) => {
                out.extend_from_slice(b"%%");
                i += 2;
            }
            (b'%', Some(b'm')) => {
                match items.get(k) {
                    Some(l) if l.as_slice() != b"   " => out.extend_from_slice(l),
                    _ => out.extend_from_slice(b"%m"),
                }
                k += 1;
                i += 2;
            }
            (b, _) => {
                out.push(b);
                i += 1;
            }
        }
    }
    out
}

fn raw_line(src: &[u8], lit: Option<usize>) -> String {
    let lit = lit.map_or('-', |k| (b'0' + k.min(9) as u8) as char);
    [RAW, lit].into_iter().chain(src.iter().map(|&b| b as char)).collect()
}

/// The gem items of `report_window(lines)` that stay lit: the chosen bullet of each answered menu (live A/B 2026-11-22).
fn report_lit(lines: &[String]) -> Vec<usize> {
    let (mut lit, mut n) = (vec![], 0);
    for l in lines.iter().filter_map(|l| l.strip_prefix(RAW)) {
        let k = l.matches("%m").count();
        if let Some(c) = l.bytes().next().filter(u8::is_ascii_digit).map(|c| (c - b'0') as usize).filter(|&c| c < k) {
            lit.push(n + c);
        }
        n += k;
    }
    lit
}

/// Ours: the Chronik window's text width and lines per page (rect [40, 30, 560, 420], 24 px lines, the title and
/// the page items off).
const LOG_WIDTH: usize = 530;
const LOG_LINES: usize = 13;

/// Greedy pages over entries of `costs` lines each: a page takes entries while they fit in `cap` lines, at least one.
fn pack_pages(costs: impl Iterator<Item = usize>, cap: usize) -> Vec<std::ops::Range<usize>> {
    let mut pages: Vec<std::ops::Range<usize>> = vec![];
    let mut used = 0;
    for (i, c) in costs.enumerate() {
        match pages.last_mut() {
            Some(p) if used + c <= cap => {
                p.end = i + 1;
                used += c;
            }
            _ => {
                pages.push(i..i + 1);
                used = c;
            }
        }
    }
    pages
}

/// The turn-end report window for `lines` (answered menus keep their gem items).
fn report_window(assets: &Assets, lines: &[String]) -> Window {
    let n = lines.iter().filter(|l| l.starts_with(RAW)).map(|l| l.matches("%m").count()).sum();
    let items = vec![b"   ".as_slice(); n];
    Window::new(assets, REPORT_BOX, &report_src(lines), &items, &[], &[]).with_scroll()
}

fn report_src(lines: &[String]) -> Vec<u8> {
    let line = |l: &String| match l.strip_prefix(RAW) {
        Some(l) => l.chars().skip(1).map(|c| c as u8).collect(),
        None => [t(l), b"$N".to_vec()].concat(),
    };
    [b"$L".to_vec(), lines.iter().flat_map(line).collect()].concat()
}

/// Scrolling report windows (rules-42 §1): the first view shows what fits; each click lets one more window
/// height of lines in, scrolling up `scroll_step` px every `SCROLL_MS` until the newest line sits at the bottom.
/// The text is kept, never cleared. The step is the text-speed option, CONFIG.DAT 0x0F3 (rules-43 §2); the original
/// steps once per overflow repaint with no timer (rules-56 §8: its speed is the machine's repaint rate); `SCROLL_MS` is our pacing.
const SCROLL_STEPS: [i32; 6] = [24, 12, 8, 6, 3, 1];
const SCROLL_MS: u32 = 17;

/// Report scroll state: offset, target offset, content bottom shown so far, the popup it belongs to.
#[derive(Default, Clone, Copy, PartialEq)]
struct ReportScroll {
    s: i32,
    target: i32,
    bottom: i32,
    key: usize,
}

impl Game {
    /// Takes a fresh or loaded state and advances it to the first human's Kontor.
    pub fn start(assets: &Assets, st: State) -> Self {
        let tables = Tables {
            locations: assets.name_table(names::LOCATIONS),
            verbs: assets.name_table((0xB65A0, 15)),
            preps: assets.name_table((0xB65A0 + 60, 15)),
            means_pl: assets.name_table((0xB4D0C, 15)),
            offices: assets.name_table(names::OFFICES),
            goods: assets.name_table(names::GOODS),
            titles: [assets.name_table(names::TITLES_M), assets.name_table(names::TITLES_F)],
            first: [assets.name_table(names::FIRST_M), assets.name_table(names::FIRST_F)],
            text_ids: (72..106).chain(1407..1441).map(|n| (n, assets.text_id(n).iter().filter_map(|&b| crate::input::from_cp437(b)).collect())).collect(),
            chronicle: assets.exe_bytes(0xB518C, 4 * 100).chunks(4).map(|c| 0x80000 + u32::from_le_bytes(c.try_into().unwrap())).collect(),
        };
        let cfg = |o: usize, default| assets.config.borrow().as_ref().map_or(default, |c| c.raw[o]);
        let mut st = st;
        st.house = assets.prefs.borrow().house();
        let mut g = Self {
            st,
            undo: None,
            seed: None,
            tables,
            branch: None,
            room: None,
            popups: VecDeque::new(),
            entered: false,
            sounds: vec![],
            duel_fx: Default::default(),
            fx_due: vec![],
            spoken: None,
            mv_pic: 0x199,
            book_shown: [true, true, true, false, false, false, false],
            ablass_city: 0,
            spot_click: None,
            scores_tt: (0, 0),
            report_year: None,
            tree_bars: vec![],
            hover: (None, false),
            gold_hot: false,
            tip: ((-1, -1), 0),
            unvisited: true, // a loaded game starts in a turn with no branch visited yet
            tree_rival: None,
            amts: [0; 4],
            page: vec![],
            bribing: false,
            sel: None,
            sel_menu: None,
            form_icons: vec![],
            trans_pic: (Menu::Quit, 0),
            score_bars: vec![],
            ledger: vec![],
            log: vec![],
            clip: None,
            branch_costs: vec![],
            award_song: 7,
            herald_song: None,
            priv_holder: 0xFF,
            priv_page: 0,
            priv_cost: 0,
            priv_city: 0,
            priv_n: 0,
            lute_song: None,
            jury_song: 0x0B,
            ert_reveal: (0, 0),
            tactic_stop: None,
            kontor_left: false,
            room_song: None,
            reveal: None,
            revealed: None,
            disaster: 0,
            court_ms: 0,
            clock_ms: 0,
            scene_ms: (0, (0, 0)),
            speech_ms: 0,
            ask_ms: 0,
            court_len: 0,
            cfg_end: cfg(0x17D, 0xFF),
            text_speed: cfg(0xF3, 2).min(5),
            cfg_announce: cfg(0x17E, 0xFF) & 1 != 0,
            opts: u32::from_le_bytes([cfg(0x17D, 0xFF), cfg(0x17E, 0xFF), cfg(0x17F, 0xFF), cfg(0x180, 0xFF)]),
            vol: [cfg(0x194, 127), cfg(0x195, 127), cfg(0x278, 175)],
            black_box: cfg(0x273, 0) != 0,
            prices: vec![],
            bribe: 0,
            child_name: vec![],
            cradles: vec![],
            church: 0,
            pending_cash: None,
            window: vec![],
            continued: None,
            report_menu: None,
            reporting_early: false,
            apply_rivals: vec![],
            turn_cash: None,
            early_shown: 0,
            cry_ms: 0,
            scroll: ReportScroll::default(),
            scroll_ms: 0,
            wait_ms: (0, None),
            save_edit: None,
            pending: None,
            amount: 0,
        };
        // Battles need the battlefield layouts; the sim reports `BattlePending` without them.
        if g.st.szenario.is_empty() {
            g.st.szenario = assets.read("SZENARIO.DAT").unwrap_or_else(|e| {
                eprintln!("SZENARIO.DAT: {e}");
                vec![]
            });
        }
        g.run_cmd(assets, None, vec![]);
        g
    }

    fn sim(&mut self, assets: &Assets, cmd: Command) {
        self.run_cmd(assets, Some(cmd), vec![]);
    }

    /// Runs `cmd` (or `State::run`) on a copy of the state with `answers`; commits on success,
    /// or parks it in `pending` and asks the next question.
    fn run_cmd(&mut self, assets: &Assets, cmd: Option<Command>, answers: Vec<u32>) {
        if self.popups.is_empty() {
            self.report_year = None;
        }
        let mut st = self.st.clone();
        let y0 = st.year;
        let mut ch = Scripted { answers: &answers, pos: 0, log: vec![], auto: vec![], stop: self.tactic_stop.take(), early: vec![] };
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match &cmd {
            Some(c) => st.apply(c.clone(), &mut ch),
            None => st.run(&mut ch),
        }));
        if let Some((last, ask)) = ch.auto.last().cloned() {
            if let Err(e) = r
                && !e.is::<NeedAnswer>()
            {
                std::panic::resume_unwind(e);
            }
            // a fight went on without the player's say: show it and hold at its last tactic question, so a hotkey
            // can still go back to any activation (rules-44 §2); nothing of this run is kept
            for (at, q) in &ch.auto {
                match self.popups.iter_mut().find_map(|p| if let Popup::Battle(v) = p { v.live.then_some(v) } else { None }) {
                    Some(v) => v.extend(q, *at),
                    None => self.popups.push_front(Popup::Battle(Box::new(self.battle_view(assets, q, *at)))),
                }
            }
            let mut answers = answers;
            answers.resize(last, 0);
            self.pending = Some(Pending { cmd, answers, ask });
            self.ask_ms = 0;
            return;
        }
        match r {
            Ok(mut ev) => {
                self.st = st;
                if self.st.year != y0 {
                    self.report_year = Some(y0);
                }
                // the turn-end windows already shown before a question (`Choices::show`) are not shown again
                self.pending_cash = None;
                let n = std::mem::take(&mut self.early_shown).min(ev.len());
                ev.drain(..n);
                self.report(assets, ev, ch.log);
                self.church = 0;
            }
            Err(e) => match e.downcast::<NeedAnswer>() {
                Ok(q) => {
                    // turn-end windows the original shows before this question: up to the first battle, whose
                    // replay is still in the uncommitted state
                    let upto = ch.early.iter().position(|e| matches!(e, SimEvent::Battle { .. })).unwrap_or(ch.early.len());
                    let mut early_popups = 0;
                    if upto > self.early_shown {
                        let new: Vec<SimEvent> = ch.early[self.early_shown..upto].to_vec();
                        if let Some(c) = new.iter().rev().find_map(|e| match *e {
                            SimEvent::Bill { cash, .. } | SimEvent::TurnEndCash { cash, .. } => Some(cash),
                            _ => None,
                        }) {
                            self.pending_cash = Some(c);
                        }
                        self.early_shown = upto;
                        // they go before whatever was already waiting, in their own order
                        let waiting: Vec<Popup> = self.popups.drain(..).collect();
                        self.reporting_early = true;
                        self.report(assets, new, vec![]);
                        self.reporting_early = false;
                        early_popups = self.popups.len();
                        self.popups.extend(waiting);
                    }
                    if let Ask::BattleIntro { .. } = &q.0 {
                        // the field opens with the intro captions; the last click answers (rules-50 §1.2)
                        let v = self.battle_view(assets, &q.0, answers.len());
                        // after the turn-end windows shown just before it
                        self.popups.insert(early_popups, Popup::Battle(Box::new(v)));
                        self.pending = Some(Pending { cmd, answers, ask: q.0 });
                        self.ask_ms = 0;
                        return;
                    }
                    if let Ask::Vote { elector, ref candidates, for_, loc, seat, religion, ref rivals, privileged, ref votes } = q.0 {
                        // rules-15 §3.4: the vote is asked over the election screen, after the votes so far
                        let o = match loc { 0..=9 => seat as usize, 10..=12 => seat as usize + 10, _ => seat as usize + 23 };
                        let pic = Self::place_picture(loc, mad_sim::career::is_church_office(o), religion);
                        let view = self.election_screen(assets, (loc, seat), pic, candidates, rivals, privileged, votes, None, Some((elector, for_)));
                        match self.popups.iter_mut().find_map(|p| if let Popup::Election(el) = p { (el.key == view.key).then_some(el) } else { None }) {
                            Some(el) => el.continue_with(view),
                            None => self.popups.push_back(Popup::Election(Box::new(view))),
                        }
                        self.pending = Some(Pending { cmd, answers, ask: q.0 });
                        self.ask_ms = 0;
                        return;
                    }
                    if let Ask::OccupyCastle { attacker, defender, verdict, .. } = q.0 {
                        // rules-59 §2: the report's first page (title, verdict) comes before the question in its window
                        let head = self.battle_report_head(assets, attacker, defender, verdict);
                        if let Some(v) = self.popups.iter_mut().find_map(|p| if let Popup::Battle(v) = p { v.live.then_some(v) } else { None }) {
                            v.add_captions(vec![(battle::REPORT_RECT, head, true)]);
                        }
                    }
                    if let Ask::Tactics { person, .. } = &q.0 {
                        // the tactic menu lives on the battlefield, not in a form
                        // only the hotkey's question gets here: its frames are already shown
                        let name = self.person_name(*person);
                        let at = answers.len();
                        match self.popups.iter_mut().find_map(|p| if let Popup::Battle(v) = p { v.live.then_some(v) } else { None }) {
                            Some(v) => v.open_menu(&q.0, name),
                            None => {
                                let mut v = self.battle_view(assets, &q.0, at);
                                v.open_menu(&q.0, name);
                                self.popups.push_front(Popup::Battle(Box::new(v)));
                            }
                        }
                        self.pending = Some(Pending { cmd, answers, ask: q.0 });
                        self.ask_ms = 0;
                        return;
                    }
                    if let Ask::PreFight { lines, q: pq, frames, .. } = &q.0 {
                        let (caps, page) = self.prefight_pages(assets, lines);
                        let at = answers.len();
                        let view = match self.popups.iter_mut().find_map(|p| if let Popup::Battle(v) = p { v.live.then_some(v) } else { None }) {
                            Some(v) => {
                                v.extend(&q.0, at);
                                Some(v)
                            }
                            None if !frames.is_empty() => {
                                self.popups.push_front(Popup::Battle(Box::new(self.battle_view(assets, &q.0, at))));
                                let Some(Popup::Battle(v)) = self.popups.front_mut() else { unreachable!() };
                                Some(v)
                            }
                            None => None,
                        };
                        let waits = match view {
                            Some(v) => {
                                v.add_captions(caps);
                                v.caption_open()
                            }
                            // no field (a human only owns a helper): plain windows that close on a click
                            None => {
                                for (r, src, _) in caps {
                                    self.popups.push_back(Form::new(&src).popup_at(r));
                                }
                                false
                            }
                        };
                        if *pq == mad_sim::fight::Parley::None {
                            if !waits {
                                let mut answers = answers;
                                answers.push(0);
                                return self.run_cmd(assets, cmd, answers);
                            }
                            self.pending = Some(Pending { cmd, answers, ask: q.0 });
                            self.ask_ms = 0;
                            return;
                        }
                        self.page = page;
                        // a commit clears its row's amount, the others keep theirs (rules-52 §3)
                        match (pq, answers.last()) {
                            (mad_sim::fight::Parley::Bribe { .. }, Some(&a)) if a > 0 && self.bribing => self.amts[((a - 1) >> 20) as usize & 3] = 0,
                            _ => self.amts = [0; 4],
                        }
                        self.bribing = matches!(pq, mad_sim::fight::Parley::Bribe { .. });
                    }
                    if let Ask::ContinueAfterMission { person, song } = q.0 {
                        if song != 0 {
                            self.lute_song = Some(song);
                        }
                        for scene in self.mission_success_scenes(assets, person as usize) {
                            self.popups.push_back(scene);
                        }
                    }
                    self.church_before(assets, &q.0);
                    if let Ask::Match { target, .. } = q.0 {
                        let intro = self.kupplerin_intro(assets, target as usize);
                        self.popups.push_back(Popup::Choice { rect: [10, 360, 620, 90], src: intro, items: vec![], acts: vec![], menu: Some(Menu::Kupplerin), gems: false });
                    }
                    if matches!(q.0, Ask::AuctionTick { blow: true, .. }) {
                        self.sounds.push(0x4B); // the hammer blow (rules-25 §1.3)
                    }
                    self.pending = Some(Pending { cmd, answers, ask: q.0 });
                    self.ask_ms = 0;
                    // rules-18 §4, §10: the plea starts at entry 7, the sentence at entry 2
                    self.amount = match self.pending.as_ref().map(|p| &p.ask) {
                        Some(Ask::Plea) => 7,
                        Some(Ask::Sentence { .. }) => 2,
                        _ => 0,
                    };
                    self.bribe = 0;
                    self.open(assets, Menu::Question);
                    // asked during a fight (leader lost, occupy): over the battlefield, which stays for the rest
                    // (the pre-fight window's question waits behind its pages, rules-52)
                    if matches!(self.popups.front(), Some(Popup::Battle(v)) if v.live && !v.caption_open())
                        && let Some(p) = self.popups.pop_back()
                    {
                        self.popups.push_front(p);
                    }
                }
                Err(e) => std::panic::resume_unwind(e),
            },
        }
    }

    /// Splits sim output into screens: everything up to a turn start goes to the event list; a
    /// turn start brings "Nächster Spieler" (if enabled) and, after the first year, the Erträge
    /// spread built from the turn-start production/sales events. A year end autosaves slot 5.
    fn report(&mut self, assets: &Assets, ev: Vec<SimEvent>, asked: Vec<String>) {
        let before: Vec<usize> = self.popups.iter().filter(|p| matches!(p, Popup::Report(_) | Popup::YearReport(_))).map(Self::report_key).collect();
        self.report_screens(assets, ev, asked);
        if !assets.prefs.borrow().event_log {
            return;
        }
        // ours: every new report window's lines go to the Chronik (a continued window only with its new lines;
        // the answered menus' `RAW` lines are not events)
        let year = self.st.year;
        let mut new = vec![];
        for p in &self.popups {
            if let Popup::Report(l) | Popup::YearReport(l) = p
                && !before.contains(&Self::report_key(p))
            {
                let skip = self.continued.filter(|c| c.0 == Self::report_key(p)).map_or(0, |c| c.1);
                new.extend(l.iter().skip(skip).filter(|l| !l.starts_with(RAW)).map(|l| (year, l.clone())));
            }
        }
        self.log.extend(new);
    }

    /// Ours (`prefs.event_log`): the Chronik's pages, newest entry first; a page holds `LOG_LINES` printed lines, an
    /// entry as many as its width needs in the window.
    fn log_pages(&self, assets: &Assets) -> Vec<std::ops::Range<usize>> {
        let width = |s: &str| t(s).iter().map(|&b| assets.font.advance[b as usize] as usize).sum::<usize>();
        pack_pages(self.log.iter().rev().map(|(y, l)| width(&format!("{y}: {l}")).div_ceil(LOG_WIDTH).max(1)), LOG_LINES)
    }

    fn report_screens(&mut self, assets: &Assets, ev: Vec<SimEvent>, asked: Vec<String>) {
        let n0 = self.popups.len();
        if ev.iter().any(|e| matches!(e, SimEvent::TurnStart { .. })) {
            self.unvisited = true;
        }
        self.herald_song = None;
        // "Weiterspielen? Nein" (rules-39 §8): 0x4D880(0) asks "Spiel beenden – Seid Ihr Euch sicher?"; Ja rolls the
        // credits without a game save, Nein resumes play (the mission is gone either way)
        let mission_quit = ev.iter().any(|e| matches!(e, SimEvent::GameOver)) && ev.iter().any(|e| matches!(e, SimEvent::MissionDone { .. }));
        if ev.iter().any(|e| matches!(e, SimEvent::YearEnd { .. })) {
            // formats.md §3.2: slot 5 is the autosave ("letztes Spiel").
            if let Err(e) = assets.write_save(5, &self.st.to_game_dat()) {
                eprintln!("autosave: {e}");
            }
            self.store_config(assets);
            // ours: the rolling history every n years (Extras), under HISTORY/<year>.DAT
            let (every, keep) = {
                let p = assets.prefs.borrow();
                (p.autosave_years as u16, p.history as usize)
            };
            if every > 0
                && self.st.year.is_multiple_of(every)
                && let Err(e) = assets.write_history(self.st.year, &self.st.to_game_dat(), keep)
            {
                eprintln!("history: {e}");
            }
        }
        let mut chronicle = None;
        // Chronicle (rules-06 §7): with the option on, the year's entry for 1600..1699 (100 EXE
        // texts, pointer table found by searching for the first entry; "?" entries are skipped)
        if let Some(year) = ev.iter().find_map(|e| if let SimEvent::YearEnd { year } = e { Some(*year) } else { None })
            && self.st.opt(mad_sim::state::OPT_HISTORY)
            && (1600..1700).contains(&year)
            && let Some(addr) = self.tables.chronicle.get(year as usize - 1600).copied()
            && assets.text(addr).first() != Some(&b'?')
        {
            let head = assets.text(0xBA880).strip_prefix(b"$V$P").map(<[u8]>::to_vec).unwrap_or_default();
            let mut body = assets.text(addr);
            body.truncate(body.len().saturating_sub(4)); // the closing `$N$L`
            chronicle = Some(Popup::Chronicle([head, b"$C".to_vec(), body].concat()));
        }
        if let Some(p) = ev.iter().find_map(|e| if let SimEvent::TreePrices(p) = e { Some(p.clone()) } else { None }) {
            self.prices = p;
        }
        // Battle results first, one window each (§5.6 outcome texts from the EXE).
        for e in &ev {
            if let SimEvent::Battle { code, attacker, defender, ref report, .. } = *e {
                // the report pages follow the fight over its last frame (rules-59 §0)
                let pages = self.battle_report(assets, code, attacker, defender, report);
                if [attacker, defender].iter().any(|&p| self.st.persons[p as usize].human()) && !self.st.replays.is_empty() {
                    let r = self.st.replays.remove(0);
                    match self.popups.iter_mut().find_map(|p| if let Popup::Battle(v) = p { v.live.then_some(v) } else { None }) {
                        Some(v) => {
                            v.finish(r);
                            v.add_report(pages);
                        }
                        None => {
                            let mut v = battle::BattleView::from_replay(assets, r, 0);
                            v.add_report(pages);
                            self.popups.push_back(Popup::Battle(Box::new(v)));
                        }
                    }
                } else {
                    self.popups.extend(pages.into_iter().map(|(src, _)| Form::new(&subst(&src, b"%m", b"")).popup_at(battle::REPORT_RECT)));
                }
            }
        }
        // a field that never got to fight (called off, ransom taken) closes with the command (rules-50 §2)
        self.popups.retain(|p| !matches!(p, Popup::Battle(v) if v.live));
        // EXE message boxes of the sim (rules-04 §5), over the window that started the action
        for e in &ev {
            if let SimEvent::MsgBox { text, args, rect } = e {
                let src = self.exe_fmt(assets, &assets.text(*text), args);
                self.popups.push_front(Form::new(&src).popup_at(*rect));
            }
        }
        let start = ev.iter().rposition(|e| matches!(e, SimEvent::TurnStart { .. }));
        let (before, after) = ev.split_at(start.unwrap_or(ev.len()));
        if let Some(SimEvent::Bill { wages, troops, robbers, means, transport, toll, directors, tax, tithe, spies, saboteurs, interest, insurance, cash }) = ev.iter().find(|e| matches!(e, SimEvent::Bill { .. })) {
            let rows = [(0xC06D0, wages), (0xC06EC, means), (0xC0710, transport), (0xC072C, toll), (0xC0744, directors), (0xC0760, tax), (0xC0780, tithe), (0xC07A0, troops), (0xC07BC, robbers), (0xC07D4, spies), (0xC07F4, saboteurs), (0xC0810, interest), (0xC0830, insurance)];
            let total: i32 = rows.iter().map(|r| *r.1).sum();
            if total > 0 {
                let mut src = assets.text(0xC06B0);
                for (a, v) in rows.iter().filter(|r| *r.1 > 0) {
                    src.extend(subst(&assets.text(*a), b"%d", &t(&dotted(v))));
                }
                src.extend(subst(&assets.text(0xC0850), b"%d", &t(&dotted(total))));
                self.sounds.push(0x84);
                self.popups.push_back(Popup::Bill(src, *cash));
            }
        }
        // rules-19 §4: the report window is interleaved with the scenes in event order; a scene flushes the lines before it
        let mut flushed = 0;
        let mut cert = None;
        for (i, e) in ev.iter().enumerate() {
            let n = self.popups.len();
            match e {
                // jury verdict (rules-15 §6.1): the tower or the night landscape with a caption
                SimEvent::Jury { person, guilty, tower, last_branch, song } => {
                    self.jury_song = *song;
                    // the jury votes on the county castle picture, a seal for every guilty vote (rules-15 §6.1)
                    let county = State::jury_county(self.st.persons[*person as usize].loc()) as u16;
                    // rules-18 §12: a juror who has not voted shows the wheel (0x179), a guilty vote the bottle (0x48), an
                    // acquittal nothing; first all twelve wheels, then the result
                    let caption = subst(&subst(&assets.text(0xC12F4), b"$15H", b""), b"$g", b"");
                    let wheels: Vec<(u16, i32, i32)> = (0..12).map(|i| (0x179, 62 + 44 * i, 420)).collect();
                    let result: Vec<(u16, i32, i32)> = (0..*guilty as i32).map(|i| (0x48, 62 + 44 * i, 420)).collect();
                    self.popups.push_back(Popup::Court { pic: 0x193 + county, figures: wheels, boxes: vec![([10, 330, 620, 70], caption.clone())], anim: None });
                    self.popups.push_back(Popup::Court { pic: 0x193 + county, figures: result, boxes: vec![([10, 330, 620, 70], caption)], anim: None });
                    let who = t(&self.pref(*person, 1, 1));
                    let (pic, rect, addr) = if !*tower { (0xB8, [10, 220, 620, 100], 0xC1470) } else if *last_branch { (0xB9, [10, 20, 620, 130], 0xC13A8) } else { (0xB9, [10, 20, 620, 130], 0xC1320) };
                    let said = match addr { 0xC1470 => 229_950, 0xC13A8 => 229_949, _ => 0 };
                    let src = [if said > 0 { speech(said) } else { vec![] }, b"$C".to_vec(), subst(&assets.text(addr), b"%1N1", &who)].concat();
                    self.popups.push_back(Popup::Scene { pic, boxes: vec![(rect, src)] });
                }
                // bankruptcy (rules-15 §6.2): the dungeon corridor and four captions
                SimEvent::Bankrupt { name, .. } => {
                    let n = t(name);
                    let boxes = vec![
                        ([40, 376, 560, 104], [speech(229951), subst(&assets.text(0xC1544), b"%1N0", &n)].concat()),
                        ([40, 380, 560, 80], [speech(229952), assets.text(0xC15A4)].concat()),
                        ([40, 380, 560, 100], [speech(229953), assets.text(0xC1620)].concat()),
                        ([40, 380, 560, 100], [speech(229954), subst(&assets.text(0xC167C), b"%1N1", &n)].concat()),
                    ];
                    self.popups.push_back(Popup::Scene { pic: 0xBA, boxes });
                }
                // sabotage film (rules-15 §5.1), once per year end with 10 %
                SimEvent::Sabotage { saboteur, target, film: true, .. } if [saboteur, target].iter().any(|&&p| self.st.persons[p as usize].human()) => {
                    self.sounds.push(0x0D);
                    self.popups.extend(Self::film_song(assets, "SABOTAGE.FLC", 0x0D, 10));
                }
                // city disaster films (rules-15 §5.3): fire and avalanche
                // city disaster (rules-19 §1): the picture of the kind, the city and the sentence, the film for fire and
                // avalanche, then the damage captions (human branches and the insurance refund follow as further events)
                SimEvent::Disaster { city, kind, variant, lost, adj } => {
                    let d = *kind as usize;
                    self.disaster = d;
                    let (pic, y) = ([0x86, 0xC9, 0xCA, 0x86][d], [216, 12, 12, 216][d]);
                    let x = 2 * d + *variant as usize;
                    let sentence = assets.text_table((0xB5E10, 8)).get(x).cloned().unwrap_or_default();
                    let named = subst(&assets.text(0xBA898), b"%o", &t(self.loc(*city)));
                    self.popups.push_back(Popup::Scene {
                        pic,
                        boxes: vec![
                            ([10, y, 620, 160], [speech(220_900 + u32::from(*city)), named].concat()),
                            ([10, y, 620, 160], [speech(220_910 + x as u32), b"$C".to_vec(), sentence].concat()),
                        ],
                    });
                    if d == 0 || d == 3 {
                        let mut film = Self::film(assets, if d == 0 { "FEUER.FLC" } else { "LAWINE.FLC" }, 12);
                        if let Some(Popup::Film(f)) = &mut film {
                            f.cue = Some(0x45); // rules-19 §1 step 5: sample 0x45 runs with the film
                        }
                        self.popups.extend(film);
                    }
                    let word = assets.text_table((0xB5E48, 4)).get(*adj as usize).cloned().unwrap_or_default();
                    let market = assets.text_table((0xB5E58, 2)).get((d != 2) as usize).cloned().unwrap_or_default();
                    let tail = subst_seq(&assets.text(0x9B0D7), b"%s", &[word, market]);
                    let head = assets.text_table((0xB5E30, 4)).get(d).cloned().unwrap_or_default();
                    let text = subst_seq(&subst(&head, b"%d", &t(&dotted(lost))), b"%s", &[tail]);
                    self.popups.push_back(Popup::Scene { pic, boxes: vec![([10, y, 620, 280], text)] });
                }
                SimEvent::DisasterBranches { person, goods } => {
                    let d = self.disaster;
                    let (pic, y) = ([0x86, 0xC9, 0xCA, 0x86][d], [216, 12, 12, 216][d]);
                    let extra: &[u8] = if *goods { b" und einige Waren " } else { b" " };
                    let text = subst_seq(&subst(&assets.text(0x9B151), b"%2N0", &t(&self.pref(*person, 2, 0))), b"%s", &[extra.to_vec()]);
                    self.popups.push_back(Popup::Scene { pic, boxes: vec![([10, y, 620, 280], text)] });
                }
                SimEvent::DisasterInsured { person, loss } => {
                    let d = self.disaster;
                    let (pic, y) = ([0x86, 0xC9, 0xCA, 0x86][d], [216, 12, 12, 216][d]);
                    let text = subst(&subst(&assets.text(0xA9DBC), b"%2N1", &t(&self.pref(*person, 2, 1))), b"%d", &t(&dotted(loss)));
                    self.popups.push_back(Popup::Scene { pic, boxes: vec![([10, y, 620, 280], text)] });
                }
                // the heir takes over (rules-15 §7.4): the notary's last words
                SimEvent::Succession { person, kids } => {
                    let src = subst(&[b"$C".to_vec(), assets.text(0x9B4AF)].concat(), b"%1N0", &t(&self.pref(*person, 1, 0)));
                    // the testament as it was read (rules-17 §5): the heir's name where the cycling item sat
                    let will = subst(&assets.text(0xA9605), b"%m", &t(&self.person_name(*person)));
                    let fifth = match kids { 2 => assets.text(0xA96A9), 3.. => assets.text(0xA96F8), _ => vec![] };
                    // the signature is the testator's name; the person already carries the heir's
                    let testator = ev.iter().find_map(|e| match e {
                        SimEvent::Died { person: p, name, female, .. } if p == person => Some(format!("{} {name}", self.tables.titles[usize::from(*female)][(self.st.persons[*p as usize].title() as usize).min(9)])),
                        _ => None,
                    });
                    let sign = subst(&assets.text(0xA975D), b"%1N0", &t(&testator.unwrap_or_else(|| self.person_name(*person))));
                    let will = [will, fifth, sign].concat();
                    let last = ev.iter().find_map(|e| match e {
                        SimEvent::Died { person: p, heckles, .. } if p == person => Some(Self::heckles(assets, heckles, 5)),
                        _ => None,
                    });
                    self.popups.push_back(Popup::Will(will));
                    let boxes = [vec![([0, 0, 640, 60], [speech(223_907), b"$0T".to_vec(), src].concat())], last.unwrap_or_default()].concat();
                    self.popups.push_back(Popup::Scene { pic: 0xC8, boxes });
                }
                // the conquest plays the film (rules-21 §2.3), before the conquest text, 10/100 s per frame (rules-56 §10);
                // a human besieger's plays before his plunder question (`Ask::SiegeWon`)
                SimEvent::SiegeWon { besieger, .. } if !self.st.persons[*besieger as usize].human() => self.popups.extend(Self::film_song(assets, "BELAGER.FLC", 8, 10)),
                // the Informant film: one of two, by the same config byte as the intro (rules-21 §4.2)
                SimEvent::SpyNotified { film: true, .. } => {
                    let name = if assets.config.borrow().as_ref().is_some_and(|c| c.raw[0x273] != 0) { "INFORMNB.FLC" } else { "INFORMNT.FLC" };
                    self.popups.extend(Self::film_song(assets, name, 0x0D, 10));
                }
                // a successful relief plays the film twice (rules-15 §5.2)
                SimEvent::Relief { ok: true, song, besieger, .. } => {
                    let cs = if self.st.persons[*besieger as usize].human() { 12 } else { 10 };
                    self.popups.extend(Self::film_song(assets, "ENTLAGER.FLC", *song, cs));
                    self.popups.extend(Self::film_song(assets, "ENTLAGER.FLC", *song, cs));
                }
                // duel (rules-15 §7.7, only outlined there): the duelling ground and the outcome lines
                SimEvent::Duel { winner, loser, office_lost, scene } if [winner, loser].iter().any(|&&p| self.st.persons[p as usize].human()) => {
                    // rules-22: the fight bank with the opening captions, the fight, the fog, taunts over the fog picture,
                    // the second fog line and the result bank with the outcome lines
                    let cap = |addr: u32| [b"$C".to_vec(), assets.text(addr)].concat();
                    let tab = |a: u32, n: usize, i: usize| assets.text_table((a, n)).get(i).cloned().unwrap_or_default();
                    let a = scene.insulter;
                    let b = if *winner == a { *loser } else { *winner };
                    let (ga, gb) = (self.st.persons[a as usize].gender() as usize, self.st.persons[b as usize].gender() as usize);
                    let hum = |p: u8| self.st.persons[p as usize].human();
                    let code = |p: u8| if hum(p) { self.st.persons[p as usize].gender() as u32 } else { 2 };
                    let rect = [40, 40, 560, 120];
                    let low = [240, 380, 320, 100];
                    let (x, y) = if hum(a) { (a, b) } else { (b, a) };
                    let fight = [0xD9u16, 0xDA, 0xDB, 0xDC][gb + 2 * ga];
                    let voice_of = |g: usize| [if g == 0 { 2 } else { 1 }, g as u32, 2, 1];
                    let place = scene.place as usize;
                    let open = vec![
                        (rect, subst(&subst(&subst(&[speech(221_921 + place as u32), cap(0xA2B04)].concat(), b"%1N5", &t(&self.pref(x, 1, 5))), b"%s", &tab(0xB67AC, 11, place)), b"%4N5", &t(&self.pref(y, 4, 5)))),
                        (low, subst_seq(&[speech(221_932), cap(0xA3FEA)].concat(), b"%1N0", &[t(&self.pref(a, 1, 0)), t(&self.pref(b, 1, 0))])),
                        ([100, 180, 240, 120], [voice(voice_of(ga), scene.answers[0] as u32 + 1), tab(0xB6988, 8, scene.answers[0] as usize)].concat()),
                        ([250, 150, 240, 120], [voice(voice_of(gb), scene.answers[1] as u32 + 1), tab(0xB6988, 8, scene.answers[1] as usize)].concat()),
                        (low, [voice(voice_of(ga), 9), cap(0xA40D5)].concat()),
                        (low, [voice(voice_of(ga), 10), cap(0xA40FE)].concat()),
                    ];
                    self.popups.push_back(Popup::Court { pic: fight, figures: vec![(fight, 0, 0)], boxes: open, anim: None });
                    let fog = (rect, [speech(221_933 + scene.fog as u32), tab(0xB6CA0, 8, scene.fog as usize)].concat());
                    self.popups.push_back(Popup::Court { pic: fight, figures: vec![(fight, 0, 0)], boxes: vec![fog], anim: Some((0, 104)) });
                    let mut taunts: Vec<([i32; 4], Vec<u8>)> = scene
                        .taunts
                        .iter()
                        .map(|&(side, k, x, y)| {
                            let who = if side == 0 { a } else { b };
                            let c0 = if hum(who) { 1 } else { 2 };
                            ([x as i32, y as i32, 240, 120], [voice([c0, self.st.persons[who as usize].gender() as u32, 2, 1], 11 + k as u32), tab(0xB5434, 63, k as usize)].concat())
                        })
                        .collect();
                    taunts.push((rect, [speech(221_941 + scene.after as u32), tab(0xB7090, 5, scene.after as usize)].concat()));
                    self.duel_fx = scene.fx.iter().cloned().collect();
                    self.popups.push_back(Popup::Scene { pic: 0xD1, boxes: taunts });
                    let q = [0xDDu16, 0xDE, 0xE0, 0xE1, 0xDF, 0xE3, 0xE2, 0xE4][4 * usize::from(*winner != a) + gb + 2 * ga];
                    let g = self.st.persons[*loser as usize].gender();
                    let mut boxes = vec![
                        (rect, subst(&[voice([0, 2, code(*winner), 1], 946), cap(0xA8277)].concat(), b"%1N1", &t(&self.pref(*winner, 1, 1)))),
                        (rect, subst(&subst(&[voice([0, 2, code(*loser), 1], 947), cap(0xA8296)].concat(), b"%1N1", &t(&self.pref(*loser, 1, 1))), b"%s", if g == 0 { b"Seine" } else { b"Ihre" })),
                    ];
                    if *office_lost {
                        let (who, pron) = if g == 0 { (&b"er"[..], &b"sein"[..]) } else { (&b"sie"[..], &b"ihr"[..]) };
                        let src = subst(&subst_seq(&[voice([0, 2, code(*loser), 1], 948), cap(0xA82D7)].concat(), b"%s", &[who.to_vec()]), b"%2N0", &t(&genitive(&self.first_name(*loser as usize))));
                        boxes[1] = (rect, subst(&src, b"%q", pron));
                    }
                    self.popups.push_back(Popup::Court { pic: q, figures: vec![(q, 0, 0)], boxes, anim: Some((0, 18)) });
                }
                // everybody is out (rules-16 §2): the dungeon corridor and the closing words
                SimEvent::AllPlayersOut => {
                    self.popups.push_back(Popup::Scene { pic: 0xBA, boxes: vec![([10, 360, 620, 120], [speech(229_948), b"$C".to_vec(), assets.text(0xBD5B0)].concat())] });
                }
                // death (rules-15 §7.4): the candle film, gravestone with the epitaph, then the notary
                SimEvent::Died { name, female, nw, epitaph, heckles, .. } => {
                    self.popups.extend(Self::film_song(assets, "KERZE.FLC", 9, 10));
                    let n = t(name);
                    let epi = assets.text_table((0xB6330, 18)).get(2 * *epitaph as usize + *female as usize).cloned().unwrap_or_default();
                    let stone = [subst(&assets.text(0x9EB83), b"%1N0", &n), b"$Z".to_vec(), epi].concat();
                    self.popups.push_back(Popup::Scene { pic: 0xC7, boxes: vec![([160, 150, 320, 200], [b"$C".to_vec(), stone].concat())] });
                    let boxes: Vec<([i32; 4], Vec<u8>)> = assets
                        .text_table((0xB5E7C, 5))
                        .iter()
                        .enumerate()
                        .map(|(i, l)| {
                            // rules-16 §2: narrator steps 900, 901, 902+g, 904+g, 906 (step 0 uses the voice for one male heir or several)
                            let file = [213_900, 223_901, 223_902 + *female as u32, 223_904 + *female as u32, 223_906][i.min(4)];
                            // no click: the speech (or 2 s), then 4 s after the greeting and 2 s before the mourners (rules-17 §4.3)
                            let hold = format!("${}T", if i == 0 { 4000 } else { 2000 }).into_bytes();
                            ([0, 0, 640, 60], [speech(file), hold, subst(&subst(l, b"%1N0", &n), b"%d", &t(&dotted(nw)))].concat())
                        })
                        .collect();
                    // mourners' heckles after steps 1..4 (rules-17 §4.3), as the sim drew them
                    let mut with_heckles = vec![];
                    for (i, b) in boxes.into_iter().enumerate() {
                        with_heckles.push(b);
                        with_heckles.extend(Self::heckles(assets, heckles, i as u8));
                    }
                    let boxes = with_heckles;
                    self.popups.push_back(Popup::Scene { pic: 0xC8, boxes });
                }
                // feast (rules-15 §7.5): the lute picture and the captions; the quality words are read as consecutive
                // strings from 0x9EE23 and 0x9F02C, which index by quality is [UNVERIFIED]
                SimEvent::Feast { host, guests, quality, loc, cost, food_short, drink_short } if self.st.persons[*host as usize].human() => {
                    let words = |first: u32, n: usize| -> Vec<Vec<u8>> {
                        let (mut a, mut out) = (first, vec![]);
                        for _ in 0..n {
                            let w = assets.text(a);
                            a += w.len() as u32 + 1;
                            out.push(w);
                        }
                        out
                    };
                    let q = *quality as usize;
                    let how = words(0x9EE23, 5)[q.saturating_sub(1).min(4)].clone();
                    let verdict = words(0x9F02C, 4)[q.saturating_sub(1).min(3)].clone();
                    let cap = |addr: u32| [b"$C".to_vec(), assets.text(addr)].concat();
                    let supply = match (*food_short, *drink_short) { (false, false) => 0x9EE3F, (true, false) => 0x9EEAB, (false, true) => 0x9EF30, _ => 0x9EF98 };
                    let boxes = vec![
                        ([20, 340, 600, 140], subst(&subst(&cap(0x9ED91), b"%o", &t(self.loc(*loc))), b"%d", &t(&dotted(cost)))),
                        ([20, 340, 600, 140], cap(0x9EDD6)),
                        ([20, 340, 600, 140], subst(&subst(&cap(0x9EDFE), b"%w", guests.to_string().as_bytes()), b"%s", &how)),
                        ([20, 340, 600, 140], cap(supply)),
                        ([20, 340, 600, 140], subst(&cap(0x9F015), b"%s", &verdict)),
                    ];
                    self.popups.push_back(Popup::Scene { pic: 0xBB, boxes });
                }
                // trial (rules-15 §6.3): the charge on the hall stairs, then the court room with its figures and the votes
                SimEvent::Trial { plaintiff, defendant, judges, kind, loc, votes } => {
                    let human = |p: &u8| self.st.persons[*p as usize].human();
                    if human(plaintiff) || human(defendant) || judges.iter().any(human) {
                        let cap = |addr: u32| [b"$C".to_vec(), assets.text(addr)].concat();
                        let word = assets.text(if *kind == 1 { 0xA837C } else { 0xA8372 });
                        let charge = subst_seq(&subst(&subst(&cap(0xA8332), b"%1N5", &t(&self.pref(*defendant, 1, 5))), b"%3N5", &t(&self.pref(*plaintiff, 3, 5))), b"%s", &[t(self.loc(*loc)), word]);
                        let bench = subst_seq(&cap(0xA83A3), b"%1N3", &judges.map(|j| t(&self.pref(j, 1, 3))));
                        self.popups.push_back(Popup::Scene { pic: 0xC1, boxes: vec![([20, 20, 600, 120], charge), ([20, 20, 600, 160], bench)] });
                        let male = |p: u8| self.st.persons[p as usize].gender() == 0;
                        let figures = vec![
                            (if male(judges[0]) { 0xE9 } else { 0xEA }, 126, 162),
                            (0xED, 60, 162),
                            (0xEE, 189, 162),
                            (if male(*defendant) { 0xE5 } else { 0xE6 }, 387, 223),
                            (if male(*plaintiff) { 0xE7 } else { 0xE8 }, 235, 306),
                        ];
                        let mut boxes = vec![];
                        let money = ev.iter().find_map(|e| if let SimEvent::TrialMoney { invested, bribes } = e { Some((invested, bribes)) } else { None });
                        if let Some((invested, _)) = money {
                            // investments of the computer sides (rules-18 §3); humans answered the dialog
                            for &p in invested {
                                boxes.push(([20, 20, 600, 120], subst(&[b"$C".to_vec(), assets.text(0xAFCFF)].concat(), b"%1N1", &t(&self.pref(p, 1, 1)))));
                            }
                        }
                        if let Some(SimEvent::TrialCharge { evidence, groups, plea }) = ev.iter().find(|e| matches!(e, SimEvent::TrialCharge { .. })) {
                            // reading of the charge (rules-18 §4)
                            let (dg, pg) = (self.st.persons[*defendant as usize].gender() as u32, self.st.persons[*plaintiff as usize].gender() as u32);
                            let pv = if human(plaintiff) { 1 } else { 2 };
                            let b = [20, 20, 600, 120];
                            let tab = |a: u32, n: usize| assets.text_table((a, n));
                            boxes.push((b, [speech(222_900), assets.text(0x99C64)].concat()));
                            let reader = if human(plaintiff) { (901, subst(&assets.text(0x99C82), b"%v", b"")) } else { (902, subst(&assets.text(0x99CB9), b"%1N3", &t(&self.pref(*plaintiff, 1, 3)))) };
                            boxes.push((b, [speech(222_000 + reader.0), reader.1].concat()));
                            for &(g, tx, c) in groups {
                                let bits: Vec<u32> = (0..10).filter(|b| evidence >> (10 * g as u32 + b) & 1 != 0).collect();
                                let more = usize::from(bits.len() > 1);
                                let conn = if g == groups[0].0 { 0 } else { 1 + c as usize };
                                let kind = tab(0xB5C84, 3).get(g as usize).cloned().unwrap_or_default();
                                let law = subst(&tab(0xB5C7C, 2).get(more).cloned().unwrap_or_default(), b"%s", &kind);
                                let word = tab(0xB5C64, 6).get(conn).cloned().unwrap_or_default();
                                let head = subst(&tab(0xB5C58, 3).get(tx as usize).cloned().unwrap_or_default(), b"%1N0", &t(&self.pref(*defendant, 1, 0)));
                                let head = subst_seq(&head, b"%s", &[word, law]);
                                boxes.push((b, [voice([0, 2, dg, 2], 907 - 2 * g as u32 + more as u32), head].concat()));
                                for bit in bits {
                                    let n = 10 * g as u32 + bit;
                                    boxes.push((b, [voice([pv, pg, 2, 2], n), tab(0xB5B5C, 30).get(n as usize).cloned().unwrap_or_default()].concat()));
                                }
                            }
                            // the plea of a computer defendant is spoken and shown (a human chose it in the dialog)
                            if !human(defendant) {
                                let plea_text = subst(&subst(&assets.text(0x99E7A), b"%1N1", &t(&self.pref(*defendant, 1, 1))), b"%s", &tab(0xB5C9C, 9).get(*plea as usize).cloned().unwrap_or_default());
                                boxes.push((b, [voice([2, dg, 2, 2], 30 + *plea as u32), plea_text].concat()));
                            }
                        }
                        if !ev.iter().any(|e| matches!(e, SimEvent::TrialTalk { .. })) {
                            boxes.push(([20, 20, 600, 120], assets.text(0xBB28C)));
                        }
                        if let Some(SimEvent::TrialTalk { witnesses, plea }) = ev.iter().find(|e| matches!(e, SimEvent::TrialTalk { .. })) {
                            // witnesses and pleas (rules-18 §6, §7), shown as captions without the columns and animated heads
                            let d = self.st.persons[*defendant as usize].gender() as usize;
                            let gold = |addr: u32| [b"$g".to_vec(), assets.text(addr)].concat();
                            let line = |tab: (u32, usize), i: usize| [b"$C$g".to_vec(), assets.text_table(tab).get(i).cloned().unwrap_or_default()].concat();
                            let b = [20, 20, 600, 120];
                            boxes.push((b, [speech(222_915), gold(0x968BA)].concat()));
                            // one popup per witness: the talking head (bank 0x18E + roll, 23 frames) on the outer side of its column
                            for &(side, v, k, head) in witnesses {
                                let (side, k) = (side as usize, k as usize);
                                if !boxes.is_empty() {
                                    self.popups.push_back(Popup::Court { pic: 0xC6, figures: figures.clone(), boxes: std::mem::take(&mut boxes), anim: None });
                                }
                                let mut with_head = figures.clone();
                                with_head.push((0x18E + u16::from(head), 10 + 556 * side as i32, 10));
                                let column = if side == 0 { [78, 10, 322, 160] } else { [240, 10, 322, 160] };
                                // no click: the head's 2.76 s, then the speech or 2.5 s (rules-18 §6)
                                let text = [voice([v as u32, 0, d as u32, 2], 39 + 8 * side as u32 + k as u32), b"$760T".to_vec(), line((0xB58E4, 34), d + 2 * k + 16 * side)].concat();
                                self.popups.push_back(Popup::Court { pic: 0xC6, figures: with_head, boxes: vec![(column, text)], anim: Some((0, 22)) });
                            }
                            let pv = if human(plaintiff) { 1 } else { 2 };
                            let (pg, dg) = (self.st.persons[*plaintiff as usize].gender() as u32, d as u32);
                            let r = |i: usize| plea[i] as usize;
                            boxes.push((b, [speech(222_916), gold(0x955FB)].concat()));
                            // each plea line plays the figures' speech frames (rules-18 §7: 52..67, 68..83, 84..99; defence 0..15, 16..31, 32..48)
                            let pro = [(56 + r(0), line((0xB5730, 14), 7 * d + r(0)), Some((52, 67))), (63 + r(1), line((0xB5768, 12), 6 * d + r(1)), Some((68, 83))), (69 + r(2), line((0xB5798, 4), r(2)), Some((84, 99))), (73, line((0xB57B8, 2), d), None)];
                            for (n, text, frames) in pro {
                                // no click: the frames' 2.4 s, then the speech or 2.5 s (rules-18 §7.1)
                                let hold = if frames.is_some() { b"$400T".to_vec() } else { b"$500T".to_vec() };
                                let spoken = (b, [voice([pv, pg, dg, 2], n as u32), hold, text].concat());
                                match frames {
                                    Some(anim) => {
                                        if !boxes.is_empty() {
                                            self.popups.push_back(Popup::Court { pic: 0xC6, figures: figures.clone(), boxes: std::mem::take(&mut boxes), anim: None });
                                        }
                                        self.popups.push_back(Popup::Court { pic: 0xC6, figures: figures.clone(), boxes: vec![spoken], anim: Some(anim) });
                                    }
                                    None => boxes.push(spoken),
                                }
                            }
                            boxes.push((b, [speech(222_917), gold(0xA15AC)].concat()));
                            let defence = [(74 + r(3), line((0xB6694, 3), r(3)), (0, 15)), (77 + r(4), line((0xB66AC, 4), d + 2 * r(4)), (16, 31)), (79 + r(5), line((0xB66BC, 4), d + 2 * r(5)), (32, 48))];
                            for (n, text, anim) in defence {
                                if !boxes.is_empty() {
                                    self.popups.push_back(Popup::Court { pic: 0xC6, figures: figures.clone(), boxes: std::mem::take(&mut boxes), anim: None });
                                }
                                self.popups.push_back(Popup::Court { pic: 0xC6, figures: figures.clone(), boxes: vec![(b, [voice([11, dg, dg, 2], n as u32), b"$400T".to_vec(), text].concat())], anim: Some(anim) });
                            }
                        }
                        if let Some((_, bribes)) = money {
                            // the bribes (rules-18 §8): 0xACD18 / 0xACD3E for a computer side, 0xACD81 / 0xACDBD for a human
                            for (s, &(p, x)) in bribes.iter().enumerate() {
                                let none = u32::from(x == 0);
                                let (n, addr) = if human(&p) { (922 + none, [0xACD81, 0xACDBD][none as usize]) } else { (918 + 2 * s as u32 + none, [0xACD18, 0xACD3E][none as usize]) };
                                boxes.push(([20, 20, 600, 120], subst(&[speech(222_000 + n), assets.text(addr)].concat(), b"%1N1", &t(&self.pref(p, 1, 1)))));
                            }
                        }
                        boxes.push(([20, 20, 600, 120], [speech(222_924), cap(0xA5EE2)].concat()));
                        for (j, &judge) in judges.iter().enumerate() {
                            if j > 0 && judges[..j].contains(&judge) {
                                continue;
                            }
                            // "stimmt für..." stays 2 s, no click (rules-18 §9 step 3)
                            boxes.push(([20, 20, 600, 120], subst(&[b"$0T".to_vec(), cap(0xA5F1D)].concat(), b"%1N3", &t(&self.pref(judge, 1, 3)))));
                            // verdict in the judge's voice (rules-16: 1 human, 12..14 computer judges, gender slot)
                            let who = &self.st.persons[judge as usize];
                            let v = if who.human() { 1 } else { 12 + j as u32 % 3 };
                            let said = voice([v, who.gender() as u32, 2, 2], if votes[j] { 80 } else { 81 });
                            // the verdict words sit at the seat's figure (rules-18 §9: 190x30 strips at (56,151), (0,184), (108,127))
                            let seat = [[56, 151, 190, 30], [0, 184, 190, 30], [108, 127, 190, 30]][j.min(2)];
                            boxes.push((seat, [said, cap(if votes[j] { 0xA5F44 } else { 0xA5F34 })].concat()));
                        }
                        self.popups.push_back(Popup::Court { pic: 0xC6, figures: figures.clone(), boxes, anim: None });
                        // the summary with the figures' verdict animation (rules-18 §9: frames 100..133, speech 84/85)
                        let convicted = votes.iter().filter(|&&v| v).count() < 2;
                        let (head, said) = if convicted { (0xA5F9E, 84) } else { (0xA5FC0, 85) };
                        let gj = self.st.persons[judges[0] as usize].gender() as u32;
                        let gd = self.st.persons[*defendant as usize].gender() as u32;
                        // no click: the frames' 3.4 s, then the speech or 2.5 s
                        let text = [voice([1, gj, gd, 2], said), b"$1400T".to_vec(), subst(&cap(head), b"%1N1", &t(&self.pref(*defendant, 1, 1)))].concat();
                        self.popups.push_back(Popup::Court { pic: 0xC6, figures, boxes: vec![([20, 20, 600, 120], text)], anim: Some((100, 133)) });
                    }
                }
                // trial costs (rules-18 §11), over the court picture: who pays, then the insurance
                SimEvent::TrialCosts { plaintiff, defendant, judges, payer, amount, insured }
                    if [*plaintiff, *defendant].iter().chain(judges.iter()).any(|&p| self.st.persons[p as usize].human()) =>
                {
                    let cap = |addr: u32| [b"$C".to_vec(), assets.text(addr)].concat();
                    let who = t(&self.pref(*payer, 1, 1));
                    let g = self.st.persons[*payer as usize].gender() as u32;
                    let paid = subst(&subst(&cap(0xAFDCE), b"%d", &t(&dotted(amount))), b"%1N1", &who);
                    let mut boxes = vec![([20, 20, 600, 160], [voice([0, 2, g, 2], 925 + u32::from(payer == defendant)), paid].concat())];
                    if *insured {
                        boxes.push(([20, 20, 600, 120], [speech(222_927), subst(&cap(0xAFE07), b"%1N1", &who)].concat()));
                    }
                    self.popups.push_back(Popup::Court { pic: 0xC6, figures: vec![], boxes, anim: None });
                }
                // a new branch (rules-16: speech 0227900 + city, message box (140, 160, 360, 160))
                SimEvent::BranchBought { prop, .. } if self.st.persons[self.st.props[*prop as usize].owner() as usize].human() => {
                    let loc = self.st.props[*prop as usize].loc();
                    let text = subst(&assets.text(0xBE548), b"%s", &t(self.loc(loc)));
                    self.popups.push_back(Form::new(&[speech(227_900 + u32::from(loc)), text].concat()).popup(assets, 140, 360, None));
                }
                // title award (rules-19 §3, rules-03 §2.1): the charter comes before the bill
                SimEvent::Title { person, title, city, song, awarder } if self.st.persons[*person as usize].human() => {
                    let pi = *person as usize;
                    let pr = &self.st.persons[pi];
                    let city = *city;
                    self.award_song = *song;
                    // the authority is picked by the sim (rules-41 §1)
                    let awarder = if *awarder == 0xFF { *person } else { *awarder };
                    let a = match *title { 1 => 0, 2 => 1, 3 | 4 => 2, _ => 3 };
                    let name = t(&self.tables.titles[pr.gender() as usize % 2].get(*title as usize).cloned().unwrap_or_default());
                    let emperor = self.st.heap.find(1, 0xC0, mad_sim::heap::ANY).map_or(0, |i| self.st.heap.recs[i][4] as usize);
                    let emperor = assets.text_table((0xB707C, 5)).get(emperor).cloned().unwrap_or_default();
                    let (me, who) = (*person, awarder);
                    let pf = |p: u8, k: u8, n: u8| t(&self.pref(p, k, n));
                    let base = assets.text([0xA914C, 0xA921C, 0xA9298, 0xA9315][a]);
                    let src = match a {
                        0 => {
                            let s = subst_seq(&base, b"%s", &[t(self.loc(city)), name]);
                            subst(&subst(&subst(&s, b"%1N1", &pf(me, 1, 1)), b"%1N0", &pf(who, 1, 0)), b"%1N4", &pf(who, 1, 4))
                        }
                        1 | 2 => {
                            let s = subst(&base, b"%s", &name);
                            let s = subst(&subst(&subst(&s, b"%1N5", &pf(who, 1, 5)), b"%1N0", &pf(me, 1, 0)), b"%4N1", &pf(me, 4, 1));
                            subst(&subst(&s, b"%1N2", &pf(who, 1, 2)), b"%1N4", &pf(who, 1, 4))
                        }
                        _ => subst(&subst_seq(&base, b"%s", &[emperor.clone(), name, emperor]), b"%1N2", &pf(me, 1, 2)),
                    };
                    // the county and principality office pages follow in the same window (rules-27 §1)
                    let extra = match *title {
                        3 => Some(subst(&subst(&assets.text(0xBD1A8), b"%1N1", &pf(me, 1, 1)), b"%o", &t(self.loc(mad_sim::economy::COUNTY[city as usize % 10])))),
                        6 => Some(subst(&assets.text(0xBD298), b"%1N1", &pf(me, 1, 1))),
                        _ => None,
                    };
                    self.popups.insert(n0.min(self.popups.len()), Popup::Award(src));
                    if let Some(x) = extra {
                        self.popups.insert((n0 + 1).min(self.popups.len()), Popup::Award(x));
                    }
                }
                // trade certificate II / III (rules-19 §3, 0x494F0): the charter of the city council or the county government
                SimEvent::Certificate { person, level } if matches!(*level, 1 | 2) && self.st.persons[*person as usize].human() => {
                    let pi = *person as usize;
                    let city = self.st.props_of(pi, mad_sim::state::KIND_BRANCH).next().map_or(self.st.persons[pi].loc(), |b| self.st.props[b].loc()) as usize % 10;
                    // `level` is the new +0x05: 1 = certificate II, 2 = III (rules-27 §2)
                    let (base, place, who) = if *level == 1 {
                        (assets.text(0xA93EF), self.loc(city as u8), t(&self.pref(*person, 3, 1)))
                    } else {
                        (assets.text(0xA94B9), self.loc(mad_sim::economy::COUNTY[city]), t(&self.pref(*person, 1, 3)))
                    };
                    let src = subst(&subst(&subst(&base, b"%3N1", &who), b"%1N3", &who), b"%s", &t(place));
                    // three lines back to back (rules-27 §2): the third names the office place
                    let (l, place) = (12 * (u32::from(*level) - 1), if *level == 1 { city as u32 } else { u32::from(mad_sim::economy::COUNTY[city]) });
                    let said = [speech(224_900 + l), speech(224_901 + l), speech(224_902 + place + 2 * u32::from(place >= 10))].concat();
                    let pop = Popup::Award([said, src].concat());
                    // at a turn start it follows the Erträge window (rules-55 §10)
                    if start.is_some_and(|s| i > s) {
                        cert = Some(pop);
                    } else {
                        self.popups.insert(n0.min(self.popups.len()), pop);
                    }
                }
                // the herald (rules-12 §3.6): centred heading and the call, at the top of the year-end window
                SimEvent::ManoeuvrePic { pic } => self.mv_pic = *pic,
                SimEvent::PrivCost { cost, .. } => self.priv_cost = *cost,
                SimEvent::Song { song, mode } => self.room_song = Some((*song, *mode)),
                SimEvent::IndulgenceCity { city } => self.ablass_city = *city,
                SimEvent::ApplyRivals(t) => self.apply_rivals = t.clone(),
                SimEvent::Notice { text } => self.popups.push_back(Form::new(&assets.text(*text)).popup(assets, 100, 440, None)),
                SimEvent::Scores { tax, tithe } => self.scores_tt = (*tax, *tithe),
                SimEvent::Herald { song } => {
                    self.herald_song = Some(*song);
                    self.popups.push_back(Popup::Chronicle(assets.text(0xC19D0)));
                }
                // the cradle (rules-19 §2.1): announcement text of the parent's gender, spoken, then the name dialog
                SimEvent::ChildBorn { parent, slot, female, cry } if self.st.persons[*parent as usize].human() => {
                    let g = self.st.persons[*parent as usize].gender() as u32;
                    let sex = assets.text_table((0xB7124, 2)).get(usize::from(*female)).cloned().unwrap_or_default();
                    let text = subst_seq(&subst(&assets.text_table((0xB711C, 2))[g as usize], b"%1N1", &t(&self.pref(*parent, 1, 1))), b"%s", &[sex]);
                    self.cradles.push((*slot, [speech(226_900 + 2 * g + u32::from(*female)), text].concat(), *cry));
                    self.open(assets, Menu::Naming(*slot));
                }
                // church wedding (rules-15 §7.1): portal, priest, vows and the feast verdict came with the questions

                // a computer canon forbids it (step 5); a human canon's veto was told with its answer
                SimEvent::Vetoed { person, target, canon } if self.st.persons[*person as usize].human() => {
                    self.church_intro(assets, *person, *target);
                    if !self.st.persons[*canon as usize].human() {
                        let c = |addr: u32| [b"$C".to_vec(), assets.text(addr)].concat();
                        self.popups.push_back(Popup::Scene { pic: 0xBE, boxes: vec![([20, 180, 600, 120], subst(&c(0x9D2C8), b"%1N5", &t(&self.pref(*canon, 1, 5)))), ([20, 180, 600, 120], c(0x9D32A))] });
                    }
                }
                _ => {}
            }
            self.scene_extra(assets, e);
            if self.popups.len() > n && i < before.len() {
                let mut lines: Vec<String> = ev[flushed..=i].iter().filter(|e| !matches!(e, SimEvent::Election { .. } | SimEvent::Dismissed { .. } | SimEvent::Divine { .. } | SimEvent::Tornado { .. } | SimEvent::Insulted { .. })).filter_map(|e| self.describe(assets, e)).collect();
                // the window of an answered menu goes on here, before the scene (as `turn_lines` below)
                if flushed == 0 && !self.window.is_empty() && !lines.is_empty() {
                    let prefix = self.window.len();
                    lines.splice(0..0, self.window.drain(..));
                    self.continued = Some((lines.as_ptr() as usize, prefix));
                }
                if !lines.is_empty() {
                    self.popups.insert(n, Popup::Report(lines));
                }
                flushed = i + 1;
            }
        }
        if ev.iter().any(|e| matches!(e, SimEvent::Seized { .. })) {
            self.sounds.push(0x4E);
        }
        // the turn-end report (desk) holds the events before `YearEnd`, the year-end window the rest (live A/B)
        let rest = &before[flushed.min(before.len())..];
        let cut = rest.iter().position(|e| matches!(e, SimEvent::YearEnd { .. })).unwrap_or(rest.len());
        let desc = |es: &[SimEvent]| -> Vec<String> { es.iter().filter(|e| !matches!(e, SimEvent::Election { .. } | SimEvent::Dismissed { .. } | SimEvent::Divine { .. } | SimEvent::Tornado { .. } | SimEvent::Insulted { .. })).filter_map(|e| self.describe(assets, e)).collect() };
        let (mut turn_lines, mut world_lines) = (desc(&rest[..cut]), desc(&rest[cut..]));
        let year_end = ev.iter().any(|e| matches!(e, SimEvent::YearEnd { .. }));
        if mission_quit {
            let mut f = Form::new(&assets.text(0xBDC98));
            f.bind(assets.text(addr::JA), Action::Credits).bind(assets.text(addr::NEIN), Action::Close);
            self.popups.push_back(f.popup_at([160, 160, 320, 160]));
        }
        if year_end { world_lines.extend(asked) } else { turn_lines.extend(asked) }
        // the turn-end window goes on with what it showed before a question, menus included (rules-63 §2: the
        // reaction under the menu; rules-13); it ends with the turn end
        let prefix = self.window.len();
        turn_lines.splice(0..0, self.window.iter().cloned());
        if self.reporting_early {
            self.window = turn_lines.clone();
        } else {
            self.window.clear();
        }
        if self.reporting_early && turn_lines.len() == prefix {
            turn_lines.clear();
        }
        if prefix > 0 && !turn_lines.is_empty() {
            self.continued = Some((turn_lines.as_ptr() as usize, prefix));
        }
        if !turn_lines.is_empty() {
            // its banner shows the cash at the end of the turn-end settlement, not after the year end
            if let Some(&SimEvent::TurnEndCash { cash, .. }) = rest[..cut].iter().rev().find(|e| matches!(e, SimEvent::TurnEndCash { .. })) {
                self.turn_cash = Some((turn_lines.as_ptr() as usize, cash));
            }
            self.popups.push_back(Popup::Report(turn_lines));
        } else if let Some(&SimEvent::Bill { cash, .. }) = ev.iter().find(|e| matches!(e, SimEvent::Bill { .. }))
            && !self.reporting_early
            && self.popups.iter().skip(n0).all(|p| matches!(p, Popup::Bill(..)))
        {
            // rules-19 §4 step 6: nothing was printed since the bill, line 4, centred (live capture ye1)
            self.popups.push_back(Popup::Bill(assets.text(0xC169C), cash));
        }
        if year_end {
            // rules-32 §2: the chronicle comes after everything else of the year-end window, on its own page
            // "Im nächsten Jahr werden neu gewählt" (rules-44 §3): its own page after the year's lines, before the chronicle
            let next = ev.iter().find_map(|e| if let SimEvent::NextElections(l) = e { Some(self.next_elections(assets, l)) } else { None });
            let empty = world_lines.is_empty() && chronicle.is_none() && next.is_none();
            if !world_lines.is_empty() {
                self.popups.push_back(Popup::YearReport(world_lines));
            }
            self.popups.extend(next.map(Popup::Chronicle));
            self.popups.extend(chronicle);
            if empty && !self.popups.iter().skip(n0).any(|p| matches!(p, Popup::Chronicle(_))) {
                // rules-19 §4: nothing was printed into the year-end window
                self.popups.push_back(Popup::Chronicle(assets.text(0xC2024)));
            }
        }
        for e in &ev {
            if let SimEvent::BranchCosts { branch, wages, means, transport } = *e {
                self.branch_costs.push((branch, wages + means + transport));
            }
        }
        let Some(SimEvent::TurnStart { person, year }) = after.first() else { return };
        self.entered = false;
        if *year > 1600 && self.st.persons[*person as usize].human() {
            self.ledger_record(*person, *year - 1, after);
        }
        if self.cfg_announce {
            let src = assets.text(addr::NEXT_PLAYER);
            let src = subst(&src, b"%1N1", &t(&self.person_name(*person)));
            self.popups.push_back(Popup::NextPlayer(subst(&src, b"%1N4", &t(&self.rank(*person as usize)))));
        }
        let (mut prod, mut exports, mut given) = (vec![], None, 0);
        let mut rest = vec![];
        for e in &after[1..] {
            match *e {
                SimEvent::Produced { good, amount, .. } => match prod.iter_mut().find(|p: &&mut (u8, u32)| p.0 == good) {
                    Some(p) => p.1 += amount,
                    None => prod.push((good, amount)),
                },
                SimEvent::Sold { revenue, .. } => *exports.get_or_insert(0) += revenue as i64,
                SimEvent::GivenAway { amount, .. } => given += amount,
                ref e => rest.extend(self.describe(assets, e)),
            }
        }
        if *year > 1600 {
            self.ert_reveal = (0, 0);
            self.popups.push_back(Popup::Ertraege(prod, exports, given));
        }
        if !rest.is_empty() {
            self.popups.push_back(Popup::Report(rest));
        }
        self.popups.extend(cert);
        if assets.prefs.borrow().warnings && self.st.persons[*person as usize].human() {
            self.popups.extend(self.warnings(assets, *person as usize));
        }
    }

    /// Ours: the finished year `year` of `person` from the turn-start sales and the turn-end costs collected.
    fn ledger_record(&mut self, person: u8, year: u16, after: &[SimEvent]) {
        let mut branches: Vec<(u8, i32, i32)> = vec![];
        let mut add = |b: u8, rev: i32, cost: i32| match branches.iter_mut().find(|r| r.0 == b) {
            Some(r) => (r.1, r.2) = (r.1 + rev, r.2 + cost),
            None => branches.push((b, rev, cost)),
        };
        for &(b, cost) in &self.branch_costs {
            add(b, 0, cost);
        }
        for e in after {
            if let SimEvent::Sold { branch, revenue, .. } = *e {
                add(branch, revenue, 0);
            }
        }
        self.branch_costs.clear();
        let branches = branches.into_iter().map(|(b, r, c)| (self.st.props[b as usize].loc(), r, c)).collect();
        let p = person as usize;
        self.ledger.push(LedgerYear { person, year, worth: self.st.net_worth(p), cash: self.st.persons[p].cash(), branches });
    }

    /// Ours (`prefs.warnings`): the turn-start warnings of human `h` as one window, none when nothing is due.
    /// No RNG: the debt outlook runs on a copy, the order costs are the order book's forecast.
    fn warnings(&self, assets: &Assets, h: usize) -> Option<Popup> {
        let pr = assets.prefs.borrow();
        let mut lines: Vec<String> = vec![];
        if let Some((debt, last)) = self.st.debt_outlook(h) {
            lines.push(pr.tr(&format!("Am Jahresende droht eine Pfändung: die Kasse liegt {} Taler unter dem erlaubten Minus.", dotted(debt)), &format!("A seizure threatens at the year's end: the cash is {} Taler below the allowed overdraft.", dotted(debt))).to_string());
            if last {
                lines.push(pr.tr("Dabei kann Eure letzte Niederlassung verloren gehen!", "Your last branch may be taken!").to_string());
            }
        }
        for i in self.st.heap.all(56, h as u8, mad_sim::heap::ANY) {
            let r = &self.st.heap.recs[i];
            if r[3] <= 1 {
                let (who, amount, year) = (self.pref(r[4], 1, 3), dotted(mad_sim::heap::r32(r, 5)), self.st.year + r[3] as u16);
                lines.push(pr.tr(&format!("Der Kredit über {amount} Taler bei {who} wird {year} fällig."), &format!("The loan of {amount} Taler from {who} falls due in {year}.")).to_string());
            }
        }
        let cost: i32 = self.st.props_of(h, mad_sim::state::KIND_BRANCH).map(|b| self.order_cost(b)).sum();
        let cash = self.st.persons[h].cash();
        if cost > cash {
            lines.push(pr.tr(&format!("Die bestätigten Aufträge kosten {} Taler, in der Kasse sind {}.", dotted(cost), dotted(cash)), &format!("The confirmed orders cost {} Taler, the cash is {}.", dotted(cost), dotted(cash))).to_string());
        }
        if lines.is_empty() {
            return None;
        }
        let mut f = Form::new(b"$C^ ");
        f.txt(pr.tr("Warnung", "Warning")).raw(b" _$Z$L");
        for l in lines {
            f.txt(&l).raw(b"$N$Z");
        }
        Some(f.popup(assets, 100, 440, None))
    }

    /// The election screen's content (rules-15 §3.4, texts rules-12 §3.7 steps 1..5) from the finished election.
    fn election_view(&self, assets: &Assets, e: &SimEvent, pic: u16) -> Elect {
        let SimEvent::Election { loc, seat, candidates, votes, winner, kind, rivals, privileged, .. } = e else { unreachable!() };
        self.election_screen(assets, (*loc, *seat), pic, candidates, rivals, *privileged, votes, Some((*winner, *kind)), None)
    }

    /// Builds the screen: header, candidate lines, the privileged line, every vote (a computer's "X stimmt für..."
    /// and "Y!", a human's prompt and "Y!"), then the result, or the prompt of the human who votes now.
    #[allow(clippy::too_many_arguments)]
    fn election_screen(&self, assets: &Assets, (loc, seat): (u8, u8), pic: u16, candidates: &[u8], rivals: &[u8], privileged: bool, votes: &[(u8, u8)], result: Option<(u8, u8)>, now: Option<(u8, u8)>) -> Elect {
        let o = match loc { 0..=9 => seat as u32, 10..=12 => seat as u32 + 10, _ => seat as u32 + 23 };
        let office_name = t(assets.name_table((0xB4378, 34)).get(o as usize).map_or("", String::as_str));
        let head = subst(&subst_seq(&assets.text(0xC17C8), b"%s", &[office_name]), b"%o", &t(self.loc(loc)));
        let names = candidates
            .iter()
            .enumerate()
            .map(|(k, &c)| {
                let s = match rivals.get(k).copied().filter(|&r| r != 0xFF) {
                    Some(r) => format!("{}, {} {}", self.pref(c, 1, 0), self.pref(r, 2, 0), self.gegenspieler(c)),
                    None => format!("{}, {}", self.pref(c, 1, 0), self.rank(c as usize)),
                };
                [b"$L".to_vec(), t(&s)].concat()
            })
            .collect();
        let prompt = |el: u8, for_: u8| {
            // 0xC18A0, or 0xC18C4 when the human votes in place of the appointer he blackmails (step 4)
            let p = if for_ == 0xFF {
                subst(&assets.text(0xC18A0), b"%1N3", &t(&self.pref(el, 1, 3)))
            } else {
                subst(&subst(&assets.text(0xC18C4), b"%1N3", &t(&self.pref(el, 1, 3))), b"%4N3", &t(&self.pref(for_, 4, 3)))
            };
            p.strip_suffix(b"$H").map(<[u8]>::to_vec).unwrap_or(p)
        };
        // the candidate lines alone first, until a click (live A/B 2026-11-22: 23 s without a click stay)
        let mut steps = vec![ElectStep { strip: vec![], coin: None, timed: false, ask: false }];
        if privileged && let Some((x, _)) = result {
            let q = if self.st.persons[x as usize].gender() & 1 == 1 { "ihr" } else { "sein" };
            let s = subst(&subst(&assets.text(0xC183C), b"%1N4", &t(&self.rank(x as usize))), b"%1N1", &t(&self.pref(x, 1, 1)));
            steps.push(ElectStep { strip: subst(&s, b"%q", q.as_bytes()), coin: None, timed: false, ask: false });
        }
        for &(el, c) in votes {
            let coin = candidates.iter().position(|&x| x == c);
            if self.st.persons[el as usize].human() {
                // the human's question, already answered by now (the blackmail variant is not told apart here)
                steps.push(ElectStep { strip: prompt(el, 0xFF), coin: None, timed: false, ask: true });
            } else {
                steps.push(ElectStep { strip: subst(&assets.text(0xC190C), b"%1N3", &t(&self.pref(el, 1, 3))), coin: None, timed: true, ask: false });
            }
            steps.push(ElectStep { strip: subst(&assets.text(0xC1928), b"%4N0", &t(&self.pref(c, 4, 0))), coin, timed: true, ask: false });
        }
        if let Some((el, for_)) = now {
            steps.push(ElectStep { strip: prompt(el, for_), coin: None, timed: false, ask: true });
        }
        if let Some((winner, kind)) = result {
            let wr = candidates.iter().position(|&c| c == winner).and_then(|k| rivals.get(k)).copied().unwrap_or(0xFF);
            let g = (self.st.persons[winner as usize].gender() & 1) as usize;
            let noun = t(&self.assets_text_id(if g == 1 { 1407 + o } else { 72 + o }));
            let text = match kind {
                0 => subst_seq(&subst(&assets.text(0xC1934), b"%1F", &t(&self.fpair(winner, wr, 1))), b"%s", std::slice::from_ref(&noun)),
                _ => subst(&assets.text(0xC1954), b"%2F", &t(&self.fpair(winner, wr, 2))),
            };
            steps.push(ElectStep { strip: subst(&text, b"%o", &t(self.loc(loc))), coin: None, timed: false, ask: false });
        }
        Elect { key: (loc, seat), pic, head, names, steps, pos: 0, ms: 0 }
    }

    /// Election, dismissal and concert scenes (rules-15 §2, §3), divine event boxes (rules-12 §3.10).
    fn scene_extra(&mut self, assets: &Assets, e: &SimEvent) {
        if matches!(e, SimEvent::Divine { .. } | SimEvent::Tornado { .. } | SimEvent::Insulted { .. })
            && let Some(text) = self.describe(assets, e)
        {
            let rect = match e {
                SimEvent::Tornado { .. } => [80, 80, 480, 320],
                SimEvent::Insulted { .. } => [140, 120, 360, 240],
                _ => [160, 100, 320, 280],
            };
            self.popups.push_back(Form::new(&[b"$Z$L".to_vec(), t(&text)].concat()).popup_at(rect));
        }
        if let SimEvent::Election { loc, seat, religion, kind, candidates, privileged, .. } = e
            && let Some(text) = self.describe(assets, e)
        {
            // election screen (rules-15 §3.4): place picture, candidates, votes, result in caption boxes
            let o = match *loc { 0..=9 => *seat as usize, 10..=12 => *seat as usize + 10, _ => *seat as usize + 23 };
            let pic = Self::place_picture(*loc, mad_sim::career::is_church_office(o), *religion);
            if *kind <= 1 {
                let view = self.election_view(assets, e, pic);
                match self.popups.iter_mut().find_map(|p| if let Popup::Election(el) = p { (el.key == view.key).then_some(el) } else { None }) {
                    Some(el) => el.continue_with(view),
                    None => self.popups.push_back(Popup::Election(Box::new(view))),
                }
                return;
            }
            let parts: Vec<&str> = text.split('\n').collect();
            let n = parts.len();
            let mut boxes: Vec<([i32; 4], Vec<u8>)> = vec![([0, 400, 640, 80], [b"$C".to_vec(), t(parts[n - 1])].concat())];
            // a vote waits for a click after the candidate list (rules-44 §3), then the votes come
            let list = if *kind <= 1 { (1 + candidates.len() + *privileged as usize).min(n - 1) } else { n - 1 };
            if n - 1 > list {
                boxes.insert(0, ([0, 330, 640, 150], t(&parts[list..n - 1].join(" "))));
            }
            if list > 0 {
                boxes.insert(0, ([0, 330, 640, 150], t(&parts[..list].join(" "))));
            }
            self.popups.push_back(Popup::Scene { pic, boxes });
        }
        if let SimEvent::Dismissed { person, loc, idx, .. } = e
            && let Some(text) = self.describe(assets, e)
        {
            let o = match *loc { 0..=9 => *idx as usize, 10..=12 => *idx as usize + 10, _ => *idx as usize + 23 };
            let pic = Self::place_picture(*loc, mad_sim::career::is_church_office(o), self.st.persons[*person as usize].religion());
            self.popups.push_back(Popup::Scene { pic, boxes: vec![([0, 150, 640, 130], [b"$C".to_vec(), t(&text)].concat())] });
        }
        if let SimEvent::PlotNote { kind: 36, id: id @ 0..=2, a, b, c } = e {
            self.lute_song = Some(*c as u8);
            // concert day (rules-15 §2): the lute picture and three caption boxes
            let x = *a as u8;
            let res = [0xAA968, 0xAAA9C, 0xAAC0C][(*id as usize).min(2)];
            let boxes = vec![
                ([0, 380, 640, 100], subst(&assets.text(0xAA8DE), b"%3N2", &t(&self.pref(x, 3, 2)))),
                ([0, 420, 640, 60], assets.text(0xAAD2F)),
                ([0, 160, 640, 240], subst(&subst(&assets.text(res), b"%1N1", &t(&self.pref(x, 1, 1))), b"%d", &t(&dotted(b)))),
            ];
            self.popups.push_back(Popup::Scene { pic: 0xBB, boxes });
        }
    }

    /// A side's tactics hotkey (rules-44 §2): the menu opens when the activation on screen ends. That question is
    /// either the one the game holds at, or an earlier one: then the fight is computed again up to it.
    fn battle_hotkey(&mut self, assets: &Assets) {
        let Some(Popup::Battle(v)) = self.popups.iter_mut().find(|p| matches!(p, Popup::Battle(v) if v.live)) else { return };
        let Some(Pending { cmd, answers, ask }) = self.pending.take() else { return };
        match v.rewind() {
            None => {
                let Ask::Tactics { person, .. } = &ask else { unreachable!() };
                let name = self.person_name(*person);
                if let Some(Popup::Battle(v)) = self.popups.iter_mut().find(|p| matches!(p, Popup::Battle(v) if v.live)) {
                    v.open_menu(&ask, name);
                }
                self.pending = Some(Pending { cmd, answers, ask });
            }
            Some(at) => {
                self.tactic_stop = Some(at);
                self.run_cmd(assets, cmd, answers[..at].to_vec());
            }
        }
    }

    /// Writes the options back into CONFIG.DAT (`0x421A0`, rules-44 §1): option dword, text speed, volumes, film size.
    fn store_config(&self, assets: &Assets) {
        let (opts, speed, vol, film) = (self.opts, self.text_speed, self.vol, self.black_box as u8);
        if let Err(e) = assets.write_config(|raw| {
            raw[0x17D..0x181].copy_from_slice(&opts.to_le_bytes());
            raw[0xF3] = speed;
            (raw[0x194], raw[0x195], raw[0x278], raw[0x273]) = (vol[0], vol[1], vol[2], film);
        }) {
            eprintln!("config: {e}");
        }
    }

    /// The next-year election list (EXE 0xC1FD4, lines 0xC2004 "%s in %o" for seat groups, 0xC2010 "Der %s von %o").
    fn next_elections(&self, assets: &Assets, list: &[(u8, u8)]) -> Vec<u8> {
        let mut src = assets.text(0xC1FD4).strip_prefix(b"$V$P").map(<[u8]>::to_vec).unwrap_or_default();
        let (plural, single) = (assets.name_table((0xB4A28, 34)), assets.name_table((0xB4378, 34)));
        for &(l, i) in list {
            let g = match l { 0..=9 => i as usize, 10..=12 => i as usize + 10, _ => i as usize + 23 };
            let (addr, table) = if i == 0 || (l < 10 && i == 7) { (0xC2004, &plural) } else { (0xC2010, &single) };
            let line = subst_seq(&assets.text(addr), b"%s", &[t(table.get(g).map_or("", String::as_str))]);
            src.extend(subst(&line, b"%o", &t(self.loc(l))));
        }
        src
    }

    /// `%kN<n>` of docs/re/rules-12 §1.5: case `k` (1 nominative .. 4 accusative), form `n` 0..6.
    /// The castle or camp property at place `k` (location 14 + k).
    pub(super) fn place_prop(&self, k: u8) -> Option<usize> {
        (0..mad_sim::state::PROPS).find(|&q| self.st.props[q].loc() == 14 + k && matches!(self.st.props[q].kind(), mad_sim::state::KIND_CASTLE | mad_sim::state::KIND_CAMP))
    }

    fn pref(&self, p: u8, k: u8, n: u8) -> String {
        let pi = p as usize;
        if pi >= self.st.persons.len() {
            return "Niemand".into();
        }
        let pr = &self.st.persons[pi];
        let g = (pr.gender() & 1) as usize;
        let title_no = pr.title() as usize;
        let name = self.first_name(pi);
        let name = if k == 2 { genitive(&name) } else { name };
        let title = self.tables.titles[g].get(title_no.min(9)).cloned().unwrap_or_default();
        // untitled "Herr" takes an n outside the nominative
        let tx = if k != 1 && title_no == 0 { format!("{title}n") } else { title };
        // plain spaces: the original breaks inside person references (live A/B: "…Euch, Herrn / anna,", "Ratsherr in /
        // Bergstedt"); the earlier no-break reading came from the old wrap rule (rules-47 §8 explains that capture)
        const NB: char = ' ';
        let tn = format!("{tx}{NB}{name}");
        let place = {
            let b = pr.0[0x0E];
            let pre = ["aus", "zu", "von", "von und zu"][(b >> 6) as usize];
            format!("{tn}{NB}{pre}{NB}{}", self.loc(b & 0x3F))
        };
        let has_office = pr.idx() != 15;
        let office = if has_office {
            let o = mad_sim::laws::global_office(pr);
            let noun = if g == 1 { self.assets_text_id(1407 + o as u32) } else { self.assets_text_id(72 + o as u32) };
            format!("{noun}{NB}in{NB}{}", self.loc(pr.loc()))
        } else {
            format!("ohne{NB}Amt")
        };
        let office_name = if has_office {
            let o = mad_sim::laws::global_office(pr);
            format!("{}{NB}{name}", if g == 1 { self.assets_text_id(1407 + o as u32) } else { self.assets_text_id(72 + o as u32) })
        } else {
            name.clone()
        };
        match n {
            0 => name,
            1 => tn,
            2 => place,
            3 => office_name,
            4 => office,
            5 => format!("{tn}, {office}"),
            _ => format!("{place}, {office}"),
        }
    }

    /// `%kE` (k 1..4, count n, transporter type) and, with k = 5, `%1u` of one: "Eure 5 Karren", "Euren Karren", "ein Karren".
    fn transporters(&self, assets: &Assets, k: u8, n: u8, typ: u8) -> String {
        let id = 19 + typ as u32;
        let gender = match id { 20 | 23 => 2, 24 => 1, _ => 0 };
        let s = |b: Vec<u8>| -> String { b.iter().filter_map(|&c| crate::input::from_cp437(c)).collect() };
        if k == 5 {
            return format!("{} {}", s(assets.text_id(3 + gender)), s(assets.text_id(id)));
        }
        let row = (k as u32 - 1) % 4;
        if n == 1 {
            format!("{} {}", s(assets.text_id(row * 107 + 6 + gender)), s(assets.text_id(row * 107 + id)))
        } else {
            let prow = (k as u32 + 3) * 107;
            format!("{} {n} {}", s(assets.text_id(prow + 6 + gender)), s(assets.text_id(prow + id)))
        }
    }

    /// `%kW` (rules-12 §1.7): article, condition adjective, ending, noun and extension clause of a residence word.
    fn wform(&self, assets: &Assets, k: u8, word: u16) -> String {
        const GENDER: [u32; 10] = [1, 1, 2, 2, 2, 1, 0, 1, 1, 2];
        let level = (word & 0x1F).min(9) as u32;
        let ext = (word >> 5) & 7;
        let cond = ((word >> 8) / 21).min(4) as usize;
        let row = ((k as u32).saturating_sub(1)) % 4;
        let g = GENDER[level as usize];
        let id = |base: u32| assets.text_id(row * 107 + base);
        let s = |b: Vec<u8>| -> String { b.iter().filter_map(|&c| crate::input::from_cp437(c)).collect() };
        let article = s(id((if k <= 4 { 6 } else { 3 }) + g));
        let adj = assets.name_table((0xB6F8C, 5)).get(cond).cloned().unwrap_or_default();
        let ending = s(id(16 + g));
        let noun = s(id(58 + level));
        let clause = ["", " mit Park", " mit Nebengebäuden", " mit Park und Nebengebäuden", " mit weitläufigen Ländereien", " mit Park und weitläufigen Ländereien", " mit Nebengebäuden und weitläufigen Ländereien", " mit Park, Nebengebäuden und weitläufigen Ländereien"][ext as usize];
        format!("{article} {adj}{ending} {noun}{clause}")
    }

    fn assets_text_id(&self, n: u32) -> String {
        self.tables.text_ids.get(&n).cloned().unwrap_or_default()
    }

    fn person_name(&self, p: u8) -> String {
        let p = p as usize;
        if p >= self.st.persons.len() {
            return "Niemand".into();
        }
        let title = &self.tables.titles[(self.st.persons[p].gender() & 1) as usize][(self.st.persons[p].title() as usize).min(9)];
        format!("{title} {}", self.first_name(p))
    }

    /// First name: the player's typed name, or the EXE table entry of a computer person.
    fn first_name(&self, p: usize) -> String {
        let pr = &self.st.persons[p];
        if pr.human() {
            self.st.name_str(pr.name() as usize)
        } else {
            self.tables.first[(pr.gender() & 1) as usize].get(pr.name() as usize).cloned().unwrap_or_else(|| format!("Nr. {p}"))
        }
    }

    /// A film popup (FLCFILME/<name>), or nothing when the file is missing.
    /// A film whose scene plays a song (rules-14 §4.1).
    fn film_song(assets: &Assets, name: &str, song: u8, cs: u32) -> Option<Popup> {
        let mut p = Self::film(assets, name, cs)?;
        if let Popup::Film(f) = &mut p {
            f.song = Some(song);
        }
        Some(p)
    }

    /// A film with the caller's frame delay in 1/100 s (rules-56 §10 table; 0 keeps the header speed).
    fn film(assets: &Assets, name: &str, cs: u32) -> Option<Popup> {
        super::film::Film::new(assets, name, |_| Next::Stay).map(|f| Popup::Film(Box::new(if cs > 0 { f.delay_cs(cs) } else { f })))
    }

    /// The credits screen; its picture and first line are drawn from the game RNG (rules-38 §6).
    fn credits(&mut self, ctx: &mut Ctx) -> Next {
        let (pic, idea) = (self.st.r(3), self.st.r(2));
        Next::Go(Box::new(super::credits::Credits::new(ctx.assets, pic, idea)))
    }

    /// The mourners' lines after notary step `step` (rules-17 §4.3): left/right at y 421, centred at 451, in the
    /// drawn voice; step 5 uses the testament lines 0xB5ECC (files 014..018).
    fn heckles(assets: &Assets, drawn: &[[u8; 5]], step: u8) -> Vec<([i32; 4], Vec<u8>)> {
        let (table, base) = if step < 5 { (assets.text_table((0xB5E94, 14)), 0) } else { (assets.text_table((0xB5ECC, 5)), 14) };
        drawn
            .iter()
            .filter(|h| h[0] == step)
            .map(|&[_, a, line, v, g]| {
                let (head, y) = [(&b"$L"[..], 421), (&b"$R"[..], 421), (&b"$C"[..], 451)][a as usize];
                ([0, y, 640, 60], [voice([u32::from(v), u32::from(g), 2, 3], u32::from(base + line)), b"$0T".to_vec(), head.to_vec(), table[line as usize].clone()].concat())
            })
            .collect()
    }

    /// The song the current scene asks for (rules-14 §4.1), if any: matchmaker 0x11, auction 0x12, trial and
    /// back room 0x10, church and lender 0x0F, battle and trade map from their random groups (fixed pick here).
    fn scene_song(&self) -> Option<u8> {
        match self.popups.front() {
            // the lute picture: the concert's drawn song (group 0x1D, rules-39 §6), else a fixed pick of the group
            Some(Popup::Scene { pic: 0xBB, .. }) => return Some(self.lute_song.unwrap_or([1, 4, 0x12, 0x18][self.st.year as usize % 4])),
            // the wedding banquet: its drawn song of the same group (rules-15 §7.1 step 8)
            Some(Popup::Scene { pic: 0xFF, .. }) if self.lute_song.is_some() => return self.lute_song,
            // disaster pictures (rules-19 §1: music 0x0D fire/avalanche, 0x0C flood, 0x0B plague), the cradle (0x16)
            // and the court room (0x10)
            Some(Popup::Scene { pic: 0x86, .. }) => return Some(0x0D),
            Some(Popup::Scene { pic: 0xC9, .. }) => return Some(0x0C),
            Some(Popup::Scene { pic: 0xCA, .. }) => return Some(0x0B),
            Some(Popup::Scene { pic: 0xCE, .. } | Popup::Choice { menu: Some(Menu::Naming(_)), .. }) => return Some(0x16),
            Some(Popup::Court { pic: 0xC6, .. }) => return Some(0x10),
            // death (rules-14: song 9, the funeral march from the candle film on): gravestone, notary, testament
            Some(Popup::Scene { pic: 0xC7 | 0xC8, .. } | Popup::Will(_)) => return Some(9),
            // the debt tower and the bankrupt's dungeon (rules-15 §6.1 step 7, §6.2: sound 0x0B); the mission
            // replay's tower caption sits in its own box (80, 200, ...) and keeps the replay's song
            Some(Popup::Scene { pic: 0xB9 | 0xBA, boxes }) if boxes[0].0[0] != 80 => return Some(0x0B),
            // acquitted by the jury: the night landscape with its drawn fanfare
            Some(Popup::Scene { pic: 0xB8, .. }) => return Some(self.jury_song),
            // the duel (rules-22 §2 step 4): song 0x17 from the fight bank to the result
            Some(Popup::Court { pic: 0xD9..=0xE4, .. } | Popup::Scene { pic: 0xD1, .. }) => return Some(0x17),
            Some(Popup::Film(f)) if f.song.is_some() => return f.song,
            // title charter (unspoken, unlike the certificates): the fanfare group {7, 0x0E} (rules-19 §3)
            Some(Popup::Award(src)) if speech_of(src).is_none() => return Some(self.award_song),
            Some(Popup::Chronicle(_) | Popup::YearReport(_)) if self.herald_song.is_some() => return self.herald_song,
            Some(p) if self.is_scene(p) => return Some(0x11),
            Some(Popup::Battle(v)) => return Some(v.song),
            Some(Popup::Choice { menu: Some(Menu::Question), .. }) => {
                return match self.pending.as_ref().map(|p| &p.ask) {
                    Some(Ask::AttendAuction { .. } | Ask::AuctionTick { .. }) => Some(0x12),
                    Some(Ask::JudgeVote { .. } | Ask::Sentence { .. } | Ask::TrialWitnesses { .. }) => Some(0x10),
                    Some(Ask::WeddingFeast { song, .. }) => Some(*song),
                    _ => None,
                };
            }
            _ => {}
        }
        match self.room {
            Some(Room::Kirche | Room::Kirchgang) => Some(0x0F),
            _ => None,
        }
    }

    /// The place picture of the office cycle (rules-15 §3.1): city 0xC1 / 0xC0, county 0x193 + place / 0xC2 or 0xC3,
    /// principality 0xBF / 0xBE.
    fn place_picture(place: u8, church: bool, religion: u8) -> u16 {
        match (place, church) {
            (0..=9, false) => 0xC1,
            (0..=9, true) => 0xC0,
            (10..=12, false) => 0x193 + place as u16,
            (10..=12, true) => 0xC2 + (religion & 1) as u16,
            (_, false) => 0xBF,
            (_, true) => 0xBE,
        }
    }

    /// Matchmaker screens are drawn straight onto `ICONBD` (no window frame).
    fn is_scene(&self, p: &Popup) -> bool {
        match p {
            Popup::Choice { menu: Some(Menu::Kupplerin), .. } => true,
            Popup::Choice { menu: Some(Menu::Question), .. } => matches!(self.pending.as_ref().map(|p| &p.ask), Some(Ask::Match { .. })),
            _ => false,
        }
    }

    /// Intro line of the matchmaker (EXE 0x9D9F9). The adjective follows rules-05 §2.5; the
    /// ending of the article and adjective for a male candidate is [UNVERIFIED].
    fn kupplerin_intro(&self, assets: &Assets, c: usize) -> Vec<u8> {
        const ADJ: [&str; 5] = ["unbeliebte", "unbedeutende", "geachtete", "angesehene", "sehr angesehene"];
        let l = self.st.persons[c].loc();
        let w = match l {
            13 => 0,
            10..=12 => (l - 10) * 3 + 1,
            _ => l,
        };
        let fav: Vec<i32> = (0..self.st.persons.len()).filter(|&i| i != c && self.st.persons[i].used() && self.st.persons[i].loc() == w).map(|i| self.st.persons[i].favour(c) as i32).collect();
        let k = if fav.is_empty() { 2 } else { ((fav.iter().sum::<i32>() / fav.len() as i32 - 25) / 10).clamp(0, 4) as usize };
        let (art, ending) = if self.st.persons[c].gender() == 1 { ("eine", "") } else { ("ein", "r") };
        let src = subst(&assets.text(addr::KUPPLERIN), b"ein%s%s%s", &t(&format!("{art} {}{ending}", ADJ[k])));
        [b"$g".as_slice(), &subst(&src, b"%1N4", &t(&self.rank(c)))].concat()
    }

    /// The click wait the front popup is in (rules-41 §3: pages and captions closed by a click), keyed so that
    /// the next page or caption starts a new wait; `None` for menus, films, battles and self-closing captions.
    fn wait_key(&self) -> Option<(usize, usize)> {
        let inner = match self.popups.front()? {
            Popup::Report(_) | Popup::YearReport(_) | Popup::Chronicle(_) if self.scroll.s < self.scroll.target => return None,
            Popup::Report(_) | Popup::YearReport(_) | Popup::Chronicle(_) => self.scroll.bottom as usize,
            Popup::Award(_) | Popup::Bill(..) | Popup::Will(_) | Popup::Ertraege(..) => 0,
            Popup::Scene { boxes, .. } | Popup::Court { boxes, .. } if hold_of(&boxes[0].1).is_none() => boxes.len(),
            // the election's click steps (candidate lines, result) show the wheel; votes and the prompt do not (live A/B)
            Popup::Election(el) if el.steps.get(el.pos).is_some_and(|st| !st.timed && !st.ask) => 1000 + el.pos,
            Popup::Battle(v) if let Some(k) = v.click_wait() => 2000 + k,
            _ => return None,
        };
        Some((self.popups.len(), inner))
    }

    /// Rectangle and full text height of a report at the front.
    fn report_geom(&self, assets: &Assets) -> Option<([i32; 4], i32)> {
        let rect = match self.popups.front()? {
            Popup::Report(_) => REPORT_BOX,
            Popup::YearReport(_) | Popup::Chronicle(_) => YEAR_BOX,
            _ => return None,
        };
        Some((rect, self.popups.front()?.window(assets, &[])?.laid.height))
    }

    /// Identity of a report popup (its line buffer), so the scroll state follows the popup and not the queue length.
    fn report_key(p: &Popup) -> usize {
        match p {
            Popup::Report(l) | Popup::YearReport(l) => l.as_ptr() as usize,
            Popup::Chronicle(src) => src.as_ptr() as usize,
            _ => 0,
        }
    }

    /// Scroll offset of report `p` (0 before its state exists).
    fn scroll_of(&self, p: &Popup) -> i32 {
        if self.scroll.key == Self::report_key(p) { self.scroll.s } else { 0 }
    }

    /// A new report at the front starts at its first view.
    fn scroll_sync(&mut self, assets: &Assets) {
        let key = self.popups.front().map_or(0, Self::report_key);
        // the key is the line buffer's address: forget it while no report is in front, so a report allocated where a
        // closed one was is still new (the siege window after its question came up unscrolled)
        if self.report_geom(assets).is_none() {
            self.scroll.key = 0;
        }
        if let Some(([.., h], _)) = self.report_geom(assets)
            && self.scroll.key != key
        {
            self.scroll = ReportScroll { s: 0, target: 0, bottom: 12 + 24 * ((h - 12) / 24), key };
            // a window that goes on from an earlier one of the same turn end opens where that one ended and lets the
            // next window of lines in from there
            if let Some((k, n)) = self.continued
                && k == key
                && let Some(Popup::Report(lines)) = self.popups.front()
            {
                let total = self.report_geom(assets).map_or(0, |g| g.1);
                let shown = report_window(assets, &lines[..n.min(lines.len())]).laid.height;
                let max_s = (total - h).max(0);
                // where the question window before it scrolled to (`Popup::window`), so the switch does not jump
                let s = (shown - h).clamp(0, max_s);
                // the answer is a click wait: the lines after it come in until the next would pass shown + h − 24
                // (rules-42 §1 wait test; live A/B 2026-11-22, siege: the menu's last line stays at the top)
                let bottom = shown + 24 * ((h - 24) / 24);
                self.scroll = ReportScroll { s, target: (bottom - h).clamp(s, max_s), bottom, key };
            }
        }
    }

    /// A click on a report: ignored while it scrolls, lets the next window of lines in, or `false` at the end.
    fn scroll_on(&mut self, assets: &Assets) -> bool {
        self.scroll_sync(assets);
        let Some(([.., h], total)) = self.report_geom(assets) else { return false };
        if self.scroll.s < self.scroll.target {
            return true;
        }
        if self.scroll.bottom >= total {
            return false;
        }
        // the line just finished ends at the window bottom, no margin (rules-42 §1; live 1600 chronicle A/B)
        let max_s = (total - h).max(0);
        self.scroll.bottom += 24 * (h / 24);
        self.scroll.target = (self.scroll.bottom - h).clamp(0, max_s);
        true
    }

    /// The named child's cradle closes; the next birth of the same report opens its own.
    fn cradle_done(&mut self, assets: &Assets) {
        self.popups.pop_front();
        if let Some(Popup::Choice { menu: Some(Menu::Naming(next)), .. }) = self.popups.front() {
            let next = *next;
            self.reopen(assets, Menu::Naming(next));
        }
    }

    /// Church wedding (rules-15 §7.1) step 1-4: the portal with the announcement, then the priest at the altar.
    fn church_intro(&mut self, assets: &Assets, suitor: u8, target: u8) {
        if self.church > 0 {
            return;
        }
        self.church = 1;
        let c = |addr: u32| [b"$C".to_vec(), assets.text(addr)].concat();
        let pair = [t(&self.pref(suitor, 3, 1)), t(&self.pref(target, 3, 1))];
        let door = subst(&[speech(225_900 + self.st.persons[target as usize].gender() as u32), c(0x9D075)].concat(), b"%1N5", &t(&self.pref(target, 1, 5)));
        self.popups.push_back(Popup::Scene { pic: 0xC0, boxes: vec![([20, 180, 600, 120], door), ([20, 180, 600, 120], subst_seq(&c(0x9D0F6), b"%3N1", &pair))] });
        self.popups.push_back(Popup::Scene { pic: 0xBE, boxes: vec![([20, 180, 600, 200], c(0x9D11E))] });
    }

    /// The church scenes due before a wedding question: portal and priest before the first one, "keine Einwände"
    /// and the priest's question before the vows, rings, film and the title before the feast.
    fn church_before(&mut self, assets: &Assets, q: &Ask) {
        let c = |addr: u32| [b"$C".to_vec(), assets.text(addr)].concat();
        match *q {
            Ask::BlackmailCanon { suitor, target, .. } | Ask::CanonVeto { suitor, target, .. } => self.church_intro(assets, suitor, target),
            Ask::Vows { suitor, target, .. } if self.church != 2 => {
                // rules-57 §3: a human canon's "keine Einwände" (0x9D288) came with his answer and goes straight to
                // the priest; a computer, blackmailed or absent canon gives "Niemand scheint Einwände zu haben..."
                let declined = self.church == 3;
                self.church_intro(assets, suitor, target);
                self.church = 2;
                let mut boxes = vec![];
                if !declined {
                    boxes.push(([20, 180, 600, 120], c(0x9D1E8)));
                }
                boxes.push(([20, 180, 600, 140], c(0x9D3A6)));
                self.popups.push_back(Popup::Scene { pic: 0xBE, boxes });
            }
            Ask::WeddingFeast { suitor, spouse, title, song, .. } => {
                self.lute_song = Some(song);
                self.popups.push_back(Popup::Scene { pic: 0xBE, boxes: vec![([20, 180, 600, 160], subst(&c(0x9D4B4), b"%1N1", &t(&self.pref(spouse, 1, 1))))] });
                self.popups.extend(Self::film(assets, "HOCHZEIT.FLC", 0));
                if let Some(tt) = title {
                    let name = &self.tables.titles[self.st.persons[suitor as usize].gender() as usize][(tt as usize).min(9)];
                    self.popups.push_back(Popup::Scene { pic: 0xFF, boxes: vec![([20, 340, 600, 80], subst(&c(0x9D5BE), b"%s", &t(name)))] });
                }
            }
            _ => {}
        }
    }

    /// The church texts an answer brings (rules-15 §7.1 steps 5, 7), shown before the sim goes on.
    fn church_after(&mut self, assets: &Assets, q: &Ask, v: u32) {
        let c = |addr: u32| [b"$C".to_vec(), assets.text(addr)].concat();
        let boxes = match (q, v) {
            (Ask::CanonVeto { .. }, 0) => vec![c(0x9D2A7), c(0x9D32A)],
            (&Ask::CanonVeto { canon, .. }, _) => {
                self.church = 3;
                vec![subst(&c(0x9D288), b"%1N3", &t(&self.pref(canon, 1, 3)))]
            }
            (&Ask::Vows { target, .. }, 1) => {
                let his = if self.st.persons[target as usize].gender() == 0 { "seinen" } else { "ihren" };
                vec![subst(&subst(&c(0x9D53E), b"%1N1", &t(&self.pref(target, 1, 1))), b"%s", &t(his))]
            }
            (&Ask::Vows { target, .. }, 2) => vec![subst(&c(0x9D591), b"%1N1", &t(&self.pref(target, 1, 1)))],
            // step 9: the feast verdict follows the answer at once, on the banquet picture, spoken (playtest report:
            // it came after the year report)
            (Ask::WeddingFeast { .. }, v) => {
                let (slot, a) = if v < 4 { (225_902, 0x9D752) } else { (225_903, 0x9D777) };
                self.popups.push_back(Popup::Scene { pic: 0xFF, boxes: vec![([20, 340, 600, 80], [speech(slot), b"$C".to_vec(), assets.text(a)].concat())] });
                return;
            }
            _ => return,
        };
        self.popups.push_back(Popup::Scene { pic: 0xBE, boxes: boxes.into_iter().map(|b| ([20, 180, 600, 120], b)).collect() });
    }

    /// Screens after the offer: on accepting, the candidate and the gift advice; always the
    /// farewell. Order, the decline case and the pronoun words are [UNVERIFIED] (not captured);
    /// the candidate's portrait is not drawn.
    fn kupplerin_after(&mut self, assets: &Assets, c: usize, accepted: bool) {
        let l = self.st.persons[c].loc();
        let w = match l {
            13 => 0,
            10..=12 => (l - 10) * 3 + 1,
            _ => l,
        };
        let male = self.st.persons[c].gender() == 0;
        let (dat, poss) = if male { (assets.text(0x9DAFD), assets.text(0x873DC)) } else { (assets.text(0x873E2), assets.text(0x873E6)) };
        // rules-61: decline → only the farewell (0, 420, 640, 60); accept → the candidate's portrait at (20, 390)
        // (gender byte 0: 0x18B + c mod 3, else 0x186 + c mod 5; no draw), then two click captions over it
        if !accepted {
            self.popups.push_front(Popup::Scene { pic: 0xBD, boxes: vec![([0, 420, 640, 60], assets.text(addr::KUPPLERIN_LEAVE))] });
            return;
        }
        let this = subst(&assets.text(addr::KUPPLERIN_THIS_IS), b"%1N5", &t(&self.person_name(c as u8)));
        // two `%s` in one string: dative pronoun, then possessive
        let adv = assets.text(addr::KUPPLERIN_ADVICE);
        let at = adv.windows(2).position(|x| x == b"%s").unwrap_or(0);
        let adv = [&adv[..at], &dat, &adv[at + 2..]].concat();
        let adv = subst(&subst(&adv, b"%s", &poss), b"%o", &t(self.loc(w)));
        let portrait = if male { 0x18B + c as u16 % 3 } else { 0x186 + c as u16 % 5 };
        self.popups.push_front(Popup::Court { pic: 0xBD, figures: vec![(portrait, 20, 390)], boxes: vec![([100, 390, 540, 90], this), ([100, 360, 540, 120], adv)], anim: None });
    }

    /// Mission success (rules-10 §5): "Erinnert Ihr Euch noch?" with the first history entry of
    /// each code, then the congratulation. Codes 2, 3, 5, 8, 9 need word tables that are not
    /// mapped yet and are left out.
    fn mission_success_scenes(&self, assets: &Assets, h: usize) -> Vec<Popup> {
        const RECT: [i32; 4] = [80, 200, 480, 160];
        let me = t(&self.person_name(h as u8));
        let cap = |addr: u32| [b"$C".to_vec(), assets.text(addr)].concat();
        let said = |n: u32, src: Vec<u8>| [speech(n), src].concat();
        let mut scenes = vec![Popup::Scene { pic: 0xBB, boxes: vec![(RECT, said(229904, subst(&cap(0xBA56C), b"%1N1", &me))), (RECT, said(229905, cap(0xBA5A8)))] }];
        let recs = self.st.heap.all(57, h as u8, mad_sim::heap::ANY);
        for code in 0..10u8 {
            let Some(&i) = recs.iter().find(|&&i| self.st.heap.recs[i][4] == code) else { continue };
            let r = &self.st.heap.recs[i];
            let year = mad_sim::heap::r16(r, 5).to_string();
            let said_n = match code { 0 => 906, 1 => 907, 2 => 908, 3 => 909, 4 => 911 + (r[7] != 0) as u32, 5 => 910, 6 => 913, 7 => 914, 8 => 915, _ => 916 };
            let (pic, src) = match code {
                0 => (0xB9, assets.text(0xBA5C4)),
                1 if self.st.persons[h].spouse() != 0xFF => (0xC0, subst(&assets.text(0xBA5F4), b"%3N0", &t(&self.person_name(self.st.persons[h].spouse())))),
                // feast quality 0..5 at +7, table 0xB63AC (rules-42 §4, rules-56 §3)
                2 => {
                    let words = ["furchtbares", "ziemlich mäßiges", "durchschnittliches", "nettes", "wirklich gelungenes", "grandioses"];
                    (0xFF, subst(&assets.text(0xBA620), b"%s", &t(words[(r[7] as usize).min(5)])))
                }
                // concert: musicians and their grade word (0xB646C, as in the concert offer)
                3 => {
                    let grade = assets.name_table((0xB646C, 11)).get((r[8] as i8).clamp(0, 10) as usize).cloned().unwrap_or_default();
                    (0xBB, subst(&subst(&assets.text(0xBA644), b"%d", &t(&dotted(r[7]))), b"%s", &t(&grade)))
                }
                // rules-56 §2: sex at +7 picks the text (and speech 0x38F + sex), %l = year
                4 => (0xCE, assets.text(if r[7] != 0 { 0xBA700 } else { 0xBA6DC })),
                // promotion (rules-41 §2): +7 global office, +8 place; office noun by gender, the place picture
                5 => {
                    let female = self.st.persons[h].gender() & 1 == 1;
                    let noun = assets.name_table((if female { 0xB5854 } else { 0xB4378 }, 34)).get(r[7] as usize).cloned().unwrap_or_default();
                    let pic = Self::place_picture(r[8], mad_sim::career::is_church_office(r[7] as usize), self.st.persons[h].religion());
                    (pic, subst(&subst(&assets.text(if female { 0xBA6AC } else { 0xBA67C }), b"%s", &t(&noun)), b"%o", &t(self.loc(r[8]))))
                }
                6 => (0xCA, subst(&assets.text(0xBA724), b"%o", &t(self.loc(r[7])))),
                7 => (0xC9, subst(&assets.text(0xBA748), b"%o", &t(self.loc(r[7])))),
                // 8: a ransom taken instead of a raid, +7 = battle field, its landscape (rules-41 §2)
                8 => (battle::BACKGROUND[(r[7] as usize).min(4)], assets.text(0xBA770)),
                9 => (0xA3, subst(&assets.text(0xBA7AC), b"%o", &t(self.loc(r[7])))),
                _ => continue,
            };
            scenes.push(Popup::Scene { pic, boxes: vec![(RECT, said(229000 + said_n, subst(&[b"$C".to_vec(), src].concat(), b"%l", year.as_bytes())))] });
        }
        scenes.push(Popup::Scene { pic: 0xBB, boxes: vec![(RECT, subst(&cap(0xBA7E0), b"%1N0", &me))] });
        // one outro film by CONFIG 0x273 (rules-56 §4), with speech 13 (sound card, large-memory build: no music draw)
        let name = if assets.config.borrow().as_ref().is_some_and(|c| c.raw[0x273] != 0) { "OUTROB.FLC" } else { "OUTRO.FLC" };
        if let Some(mut p) = Self::film(assets, name, 10) {
            if let Popup::Film(f) = &mut p {
                f.voice = Some("0000013".into());
            }
            scenes.push(p);
        }
        scenes
    }

    /// `%f`: "Gegenspieler", "Gegenspielerin" for a woman (rules-12 §1.6).
    fn gegenspieler(&self, p: u8) -> String {
        format!("Gegenspieler{}", if self.st.persons[p as usize].gender() & 1 == 1 { "in" } else { "" })
    }

    /// `%kF` for (a, b), b = 0xFF none (rules-12 §1.6).
    fn fpair(&self, a: u8, b: u8, k: u8) -> String {
        if b == 0xFF {
            return match k { 0 => self.pref(a, 4, 5), 1 => self.pref(a, 1, 1), 2 => self.pref(a, 4, 0), _ => self.pref(a, 1, 4) };
        }
        let (rb, f) = (self.pref(b, 2, 0), self.gegenspieler(a));
        match k {
            0 => format!("{}, {rb} {f}", self.pref(a, 4, 3)),
            1 => format!("{}, {rb} {f},", self.pref(a, 1, 0)),
            2 => format!("{}, {rb} {f}", self.pref(a, 4, 0)),
            _ => format!("{rb} {f}"),
        }
    }

    fn rank(&self, p: usize) -> String {
        let pr = &self.st.persons[p];
        let office = self.office(pr.loc(), pr.idx());
        if office.is_empty() { "ohne Amt".into() } else { format!("{office} in {}", self.loc(pr.loc())) }
    }

    fn loc(&self, l: u8) -> &str {
        self.tables.locations.get(l as usize).map_or("?", String::as_str)
    }

    fn good(&self, g: u8) -> &str {
        self.tables.goods.get(g as usize).map_or("?", String::as_str)
    }

    /// The report line of a sim event from its EXE text (docs/re); `None` where the original prints nothing here.
    fn describe(&self, assets: &Assets, e: &SimEvent) -> Option<String> {
        use SimEvent::*;
        Some(match e {
            // the next player's turn starts with its own screens, no report heading (rules-01 §3.1)
            TurnStart { .. } => return None,
            // rules-01 §3.1 (corrected): a skipped prison turn shows nothing; only the history line (code 0) remains
            TurnSkipped { .. } => return None,
            // the year-end window has no heading of its own (A/B: it starts with the first world line)
            YearEnd { .. } => return None,
            Harvest { .. } | SkillGain { .. } | TreePrices(_) | Battle { .. } => return None,
            StorageOffer { .. } => return None, // the offers are shown in the storage menu, not as news
            // rules-23 §1: production, exports and give-aways are the "Erträge" window; arrivals print nothing
            Produced { .. } | Sold { .. } | BranchCosts { .. } | Arrived { .. } | GivenAway { .. } => return None,
            Bill { .. } => return None, // shown as the bill page
            // EXE 0xC08EC; the income belongs to the player whose turn starts
            OfficeIncome { amount } => plain(&subst(&subst(&assets.text(0xC08EC), b"%1N4", &t(&self.rank(self.me()?))), b"%d", &t(&dotted(amount)))),
            // EXE 0xC1ED4 / 0xC1F30 / 0xC1F7C (rules-32 #17, rules-40 §27)
            PrivilegeIncome { person, kind, amount } => {
                let src = assets.text([0xC1ED4, 0xC1F30, 0xC1F7C][(*kind as usize).min(2)]);
                let src = subst(&subst(&subst(&src, b"%1N1", &t(&self.pref(*person, 1, 1))), b"%1N4", &t(&self.rank(*person as usize))), b"%d", &t(&dotted(amount)));
                plain(&src)
            }
            // the charter scene; titles are awarded to humans only (rules-03 §2.1)
            Title { .. } => return None,
            // election screens as report text (rules-12 §3.7): candidates, the computer electors' votes, the result
            NextElections(_) => return None,
            Election { loc, seat, candidates, votes, winner, kind, rivals, privileged, .. } => {
                let o = match *loc { 0..=9 => *seat as u32, 10..=12 => *seat as u32 + 10, _ => *seat as u32 + 23 };
                let g = (self.st.persons[*winner as usize].gender() & 1) as usize;
                let noun = t(&self.assets_text_id(if g == 1 { 1407 + o } else { 72 + o }));
                let office_name = t(assets.name_table((0xB4378, 34)).get(o as usize).map_or("", String::as_str));
                let head = subst(&subst_seq(&assets.text(0xC17C8), b"%s", &[office_name]), b"%o", &t(self.loc(*loc)));
                // the hidden election shows only its result line (rules-44 §3)
                let mut parts = if *kind == 3 { vec![] } else { vec![plain(&head)] };
                let candidates: &[u8] = if *kind == 3 { &[] } else { candidates };
                for (k, &c) in candidates.iter().enumerate() {
                    // 0xC17F8, or 0xC1818 for some human's rival: "%1N0, %2N0 %f"
                    let s = match rivals.get(k).copied().filter(|&r| r != 0xFF) {
                        Some(r) => subst(&subst(&assets.text(0xC1818), b"%2N0", &t(&self.pref(r, 2, 0))), b"%f", &t(&self.gegenspieler(c))),
                        None => subst(&assets.text(0xC17F8), b"%1N4", &t(&self.rank(c as usize))),
                    };
                    // `%m` is the coin marker (args 0x90, 0x73), not drawn in the caption box
                    parts.push(plain(&subst(&subst(&s, b"%m", b""), b"%1N0", &t(&self.pref(c, 1, 0)))));
                }
                // the privileged candidate shuts the others out (0xC183C with x, x, x: rank, name, sein/ihr)
                if *privileged {
                    let x = *winner;
                    let q = if self.st.persons[x as usize].gender() & 1 == 1 { "ihr" } else { "sein" };
                    let s = subst(&subst(&assets.text(0xC183C), b"%1N4", &t(&self.rank(x as usize))), b"%1N1", &t(&self.pref(x, 1, 1)));
                    parts.push(plain(&subst(&s, b"%q", q.as_bytes())));
                }
                for &(e, c) in votes.iter().filter(|_| *kind != 3) {
                    if !self.st.persons[e as usize].human() {
                        // 0xC190C "%1N3 stimmt für..." (e), then 0xC1928 "%4N0!" (c)
                        let a = plain(&subst(&assets.text(0xC190C), b"%1N3", &t(&self.pref(e, 1, 3))));
                        parts.push(format!("{a} {}", plain(&subst(&assets.text(0xC1928), b"%4N0", &t(&self.pref(c, 4, 0))))));
                    }
                }
                let wr = candidates.iter().position(|c| c == winner).and_then(|k| rivals.get(k)).copied().unwrap_or(0xFF);
                let result = match kind {
                    0 => subst_seq(&subst(&assets.text(0xC1934), b"%1F", &t(&self.fpair(*winner, wr, 1))), b"%s", std::slice::from_ref(&noun)),
                    1 => subst(&assets.text(0xC1954), b"%2F", &t(&self.fpair(*winner, wr, 2))),
                    2 => subst_seq(&subst(&assets.text(0xC1998), b"%1F", &t(&self.fpair(*winner, wr, 1))), b"%s", std::slice::from_ref(&noun)),
                    _ => subst_seq(&subst(&assets.text(0xC1978), b"%1F", &t(&self.fpair(*winner, wr, 1))), b"%s", std::slice::from_ref(&noun)),
                };
                parts.push(plain(&subst(&result, b"%o", &t(self.loc(*loc)))));
                parts.join("\n")
            }
            // appointments are told by the election lines (human involved) or not at all
            Appointed { .. } => return None,
            // told by the gravestone and notary scenes
            Died { .. } => return None,
            // EXE 0xBB3BC, the yearly fine of a sentence
            // only the human's record pass charges (rules-04b §3.6)
            Fine { amount, .. } => plain(&subst(&assets.text(0xBB3BC), b"%d", &t(&dotted(amount)))),
            // told by the cradle scene (rules-19 §2.1); 0xBA6DC / 0xBA700 are replay captions only (rules-56 §2)
            ChildBorn { .. } => return None,
            // EXE 0xBAA7C / 0xBAAF0 (name, age) for a human parent
            ChildDied { age, name, female, .. } => {
                let src = subst(&assets.text(if *female { 0xBAAF0 } else { 0xBAA7C }), b"%s", &t(name));
                plain(&subst(&src, b"%d", &t(&dotted(age))))
            }
            // EXE 0x9CF95 / 0x9D009 / 0x9D32A (rules-05 §3); which name goes to which code is read from the sentences
            CourtshipFailed { target, reason, .. } => {
                let name = t(&self.person_name(*target));
                match reason {
                    0 => plain(&subst(&subst(&assets.text(0x9D009), b"%4N5", &name), b"%1N0", &name)),
                    1 => plain(&subst(&assets.text(0x9CF95), b"%1N5", &name)),
                    _ => plain(&assets.text(0x9D32A)),
                }
            }
            Wedding { .. } | Vetoed { .. } => return None, // told by the church scenes (the suitor is human)
            Disaster { .. } | DisasterBranches { .. } | DisasterInsured { .. } => return None, // told by the disaster scenes
            MsgBox { .. } => return None,
            // no EXE text is known for these, or they only mark progress: not listed
            // card game reveal and outcome (rules-13 §2.11: EXE 0xA2938, 0xA2977, 0xA29AA, 0xA29D7; a win is announced by the rematch question)
            Cards { mine, theirs, x } => {
                let me = self.me().map_or(0, |h| h as u8);
                let s = subst(&subst_seq(&assets.text(0xA2938), b"%1N0", &[t(&self.pref(me, 1, 0)), t(&self.pref(*x, 1, 0))]), b"%d", b"#");
                let s = plain(&s).replacen('#', &mine.to_string(), 1).replacen('#', &theirs.to_string(), 1);
                let tail = if *mine > 21 && *theirs > 21 {
                    plain(&assets.text(0xA2977))
                } else if mine == theirs {
                    plain(&assets.text(0xA29AA))
                } else if *theirs <= 21 && (*mine > 21 || mine < theirs) {
                    plain(&assets.text(0xA29D7))
                } else {
                    String::new()
                };
                format!("{s} {tail}").trim().to_string()
            }
            // sub-plot messages with values (rules-13 §2.1, §2.7, §2.15, §2.16)
            PlotNote { kind, id, a, b, c } => {
                let tx = |addr: u32| assets.text(addr);
                let two = |addr: u32, n: i32| assets.name_table((addr, 2)).get((n != 0) as usize).cloned().unwrap_or_default();
                let nt = |addr: u32, n: usize, i: i32| t(assets.name_table((addr, n)).get(i.max(0) as usize % n).map_or("", String::as_str));
                let raw = |addr: u32, n: usize, i: i32| assets.text_table((addr, n)).get(i.max(0) as usize % n).cloned().unwrap_or_default();
                let p = |x: i32, k: u8, n: u8| t(&self.pref(x as u8, k, n));
                let src = match (*kind, *id) {
                    // answers and follow-ups (rules-13 §2): fixed texts
                    (32, 1) => tx(0xA468F),
                    (32, 2) => tx(0xA46FC),
                    (32, 3) => tx(0xA5332),
                    (32, 4) => tx(0xAA202),
                    (32, 5) => tx(0xAA580), // "hr erklärt": the EXE string lacks the I
                    (32, 6) => tx(0xAA5E4),
                    (33, 3) => tx(0x9CB84),
                    (33, 4) => tx(0x9CE87),
                    (34, 5) => tx(0xAA6AC),
                    (34, 6) => tx(0xAA7E7),
                    (34, 7) => tx(0xAA880),
                    (36, 3) => tx(0x9FF6F),
                    (36, 4) => tx(0x9FFCD),
                    (36, 5) => tx(0xAF85A),
                    (37, 3) => tx(0x94BF3),
                    (37, 4) => tx(0x94BA7),
                    (38, 4) => tx(0x88A5E),
                    (38, 5) => tx(0x88AA7),
                    (38, 7) => tx(0xA106D),
                    (38, 8) => tx(0xA123C),
                    (38, 10) => tx(0xA1304),
                    (39, 1) => tx(0x88DB3),
                    (39, 2) => tx(0x88FD3),
                    (40, 1) => tx(0x973E3),
                    (40, 3) => tx(0xA6AA9),
                    (42, 3) => tx(0x9844B),
                    (42, 6) => tx(0xADF4B),
                    (44, 4) => tx(0xA6D83),
                    (46, 8) => tx(0xAE2AD),
                    (47, 2) => tx(0xA7785),
                    (48, 4) => tx(0xA901E),
                    (49, 4) => tx(0xA7AA1),
                    (49, 5) => tx(0xA7AE4),
                    (49, 6) => tx(0xA7C17),
                    (49, 7) => tx(0xA7C58),
                    // 0xABD7E: "%4S" (count, accusative noun 22 + u), then the description 0xB7180[u]
                    (32, 7) => {
                        let u = (*b).clamp(0, 2) as u32;
                        let noun: String = assets.text_id(if *a == 1 { 3 } else { 7 } * 107 + 22 + u).iter().filter_map(|&ch| crate::input::from_cp437(ch)).collect();
                        subst(&subst(&tx(0xABD7E), b"%4S", &t(&format!("{a} {noun}"))), b"%s", &raw(0xB7180, 3, *b))
                    }
                    (33, 5) => subst(&tx(0x9CEBF), b"%s", &nt(0xB596C, 32, *a)),
                    (33, 6) => subst(&subst(&tx(0xA3218), b"%d", &t(&dotted(a))), b"%s", &nt(0xB5530, 49, *b)),
                    // the notice 0xA619E: city, person x (0xFF none), destination
                    (34, 4) => subst_seq(&subst(&tx(0xA619E), b"%1N5", &p(*b, 1, 5)), b"%s", &[t(self.loc(*a as u8)), nt(0xB6C44, 7, *c)]),
                    (38, 3) => subst(&tx(0x88988), b"%s", &t(self.st.props.get(*a as usize).map_or("", |q| self.loc(q.loc())))),
                    (38, 6) => subst(&tx(0xA0FAC), b"%2N0", &p(*a, 2, 0)),
                    (38, 9) => subst(&tx(0xA12B2), b"%d", &t(&dotted(a))),
                    (38, 11) => subst_seq(&tx(0xA1341), b"%s", &[nt(0xB4FB8, 117, *a), nt(0xB6688, 2, *b), t(self.loc(*c as u8))]),
                    (38, 12) => subst_seq(&tx(0xA8A77), b"%s", &[nt(0xB6AA8, 10, *a), nt(0xB6410, 18, *b)]),
                    (39, 3) => subst(&tx(0x8906F), b"%d", &t(&dotted(a))),
                    (40, 2) => subst(&tx(0xA6855), b"%s", &nt(0xB5A0C, 9, *a)),
                    (40, 4) => subst(&tx(0xA6A23), b"%s", &nt(0xB5A30, 2, *a)),
                    (42, 0) => subst(&tx(0x894C4), b"%s", &nt(0xB5AE4, 30, *a)),
                    // 0x97480: seat, city, attendees + 10, title, greeting
                    (42, 1) => {
                        let s = subst_seq(&tx(0x97480), b"%s", &[nt(0xB5A6C, 3, a & 15), t(self.loc(((b >> 4) & 15) as u8 % 10)), nt(0xB4E98, 4, b & 15), nt(0xB5A78, 5, (a >> 4) & 15)]);
                        subst(&s, b"%d", &t(&dotted((a >> 8) + 10)))
                    }
                    // a sermon quote 0xB5AB8[q] and a reaction 0xB5A8C[r]
                    (42, 2) => [raw(0xB5AB8, 10, *a), raw(0xB5A8C, 11, *b)].concat(),
                    (42, 4) => subst(&tx(0xADD87), b"%s", &nt(0xB4E98, 4, *a)),
                    // the invitation (rules-46 §3): the next title, the clergy person J
                    (42, 5) => subst(&subst(&tx(0xADE02), b"%s", &nt(0xB4E98, 4, *a)), b"%1N3", &t(&self.pref(*b as u8, 1, 3))),
                    (43, 0) => subst(&tx(0xAF981), b"%1N1", &p(*a, 1, 1)),
                    (43, 1) => subst(&tx(0xA2841), b"%1N1", &p(*a, 1, 1)),
                    // the deal: 0xA289E, 0xA28BF when x took k > 0 cards, 0xA28D9
                    (43, 2) => {
                        let more = if *b > 0 { subst(&subst(&tx(0xA28BF), b"%1N1", &p(*a, 1, 1)), b"%d", &t(&dotted(b))) } else { vec![] };
                        [tx(0xA289E), more, subst(&tx(0xA28D9), b"%1N1", &p(*a, 1, 1))].concat()
                    }
                    (43, 3) => subst(&tx(0xA2ACB), b"%1N0", &p(*a, 1, 0)),
                    (44, 5) => subst(&tx(0xA6E13), b"%4N1", &p(*a, 4, 1)),
                    (45, 3) => subst(&tx(0xAAFC0), b"%3N1", &p(*a, 3, 1)),
                    (45, 4) => subst(&tx(0xAAFE7), b"%3N1", &p(*a, 3, 1)),
                    (45, 5) => subst(&tx(0xA564F), b"%d", &t(&dotted(a))),
                    (46, 5) => subst(&tx(0xAB16C), b"%1N5", &p(*a, 1, 5)),
                    (46, 6) => subst(&tx(0xAB199), b"%s", &t(self.loc(10 + (*a as u8 & 3)))),
                    (46, 7) => subst(&tx(0xAE143), b"%3N1", &p(*a, 3, 1)),
                    // 0xA77F6: years, quantity, good, city
                    (47, 3) => {
                        let s = subst(&subst(&tx(0xA77F6), b"%w", &t(&a.to_string())), b"%d", &t(&dotted(b)));
                        subst_seq(&s, b"%s", &[t(self.good((c & 255) as u8)), t(self.loc((c >> 8) as u8))])
                    }
                    // contest entry (rules-13 §3.1): 0x9A3FA not entered, 0x9A437 entered
                    (4, i) => subst(&tx(if i == 0 { 0x9A3FA } else { 0x9A437 }), b"%1N1", &p(*a, 1, 1)),
                    (32, _) => subst(&tx(0xB3A1D), b"%d", &t(&dotted(a))),
                    (39, _) => {
                        // contract year, city, " " / " ein letztes Mal ", price, ".." / contract end
                        let s = subst_seq(&tx(0xAAD4A), b"%s", &[t(self.loc((b & 255) as u8)), t(&two(0xB701C, *c)), t(&two(0xB7024, *c))]);
                        subst(&subst(&s, b"%l", a.to_string().as_bytes()), b"%d", (b >> 8).to_string().as_bytes())
                    }
                    (36, _) => return None, // told by the concert scene
                    (45, band) => {
                        // 0xA8AB7 and the band texts 0xA8C17 / 0xA8D25 / 0xA8D99 (rules-13 §2.13)
                        let x = *a as u8;
                        let who = assets.name_table((0xB6E94, 2)).get((self.st.persons[x as usize].gender() & 1) as usize).cloned().unwrap_or_default();
                        let head = subst(&tx(0xA8AB7), b"%2N1", &t(&self.pref(x, 2, 1)));
                        let addr = [0xA8C17, 0xA8D25, 0xA8D99][(band as usize).min(2)];
                        let tail = subst(&subst(&subst(&tx(addr), b"%1N1", &t(&self.pref(x, 1, 1))), b"%3N1", &t(&self.pref(x, 3, 1))), b"%d", &t(&dotted(b)));
                        // the first %s of the head is the groom or bride, the second is the band text
                        subst_seq(&head, b"%s", &[t(&who), tail])
                    }
                    (38, 0) => {
                        // 0xA891C: camp, human, count (first name), place, losses (%3u of noun 32)
                        let me = self.me().map_or(0, |h| h as u8);
                        let camp = self.st.props.get(*a as usize).map_or(String::new(), |p| self.loc(p.loc()).to_string());
                        let count = assets.name_table((0xB4FB8, 117)).get(*b as usize % 117).cloned().unwrap_or_default();
                        let place = assets.name_table((0xB6E70, 3)).get(((c >> 8) & 15) as usize % 3).cloned().unwrap_or_default();
                        let lost = c & 255;
                        let word = |n: u32| assets.text_id(n).iter().filter_map(|&ch| crate::input::from_cp437(ch)).collect::<String>();
                        let lost_s = if lost == 1 { format!("{} {}", word(3 * 107 + 3), word(3 * 107 + 32)) } else { format!("{lost} {}", word(6 * 107 + 32)) };
                        let s = subst(&tx(0xA891C), b"%1N0", &t(&self.pref(me, 1, 0)));
                        subst(&subst_seq(&s, b"%s", &[t(&camp), t(&count), t(&place)]), b"%3u", &t(&lost_s))
                    }
                    (38, 2) => {
                        // arrested (0x886EA) or fled (0x888CD), the fine is gone either way
                        subst(&tx(if *a == 0 { 0x886EA } else { 0x888CD }), b"%d", &t(&dotted(*b)))
                    }
                    (38, _) => {
                        let mut s = Vec::new();
                        for (q, g) in [(*a, 11u8), (*b, 12), (*c, 13)] {
                            s.extend(subst(&subst(&tx(0xA8A6F), b"%d", &t(&dotted(q))), b"%s", &t(self.good(g))));
                        }
                        s
                    }
                    (37, w) => {
                        // wager result (rules-13 §2.5): the claim's outcome, the stake's outcome
                        let win = (*a & 1) as usize;
                        let (b5, b6) = ((*b & 255) as u8, ((*b >> 8) & 255) as u8);
                        let pick = |addrs: [u32; 2]| tx(addrs[win]);
                        let mut out = match w {
                            0 => subst_seq(&tx(0x881AF), b"%s", &[t(self.good(b6)), t(self.loc(b5)), t(&assets.name_table((0xB4BA0, 2)).get(win).cloned().unwrap_or_default())]),
                            1 => {
                                let tav = assets.name_table((0xB5AE4, 30)).get(b6 as usize % 30).cloned().unwrap_or_default();
                                [subst(&tx(0x881F2), b"%s", &t(&tav)), pick([0x8826B, 0x882D0])].concat()
                            }
                            _ => {
                                let city = ((b5 as i32 + (1 - win as i32) * (1 + (*a >> 1))) % 10) as u8;
                                let disc = assets.name_table((0xB5668, 8)).get(b6 as usize % 8).cloned().unwrap_or_default();
                                [subst_seq(&tx(0x88334), b"%s", &[t(&disc), t(self.loc(city))]), pick([0x8835B, 0x883EE])].concat()
                            }
                        };
                        out.extend(subst(&pick([0x88459, 0x884AB]), b"%d", &t(&dotted(c))));
                        out
                    }
                    (46, i) => match i {
                        0 => subst(&tx(0xAB060), b"%s", &t(self.loc(*a as u8))),
                        4 => tx(0xAB0CE),
                        1 => subst(&subst(&tx(0xAE1D9), b"%s", &t(assets.name_table((0xB4B3C, 7)).get(*a as usize % 7).map_or("", String::as_str))), b"%3N1", &t(&self.pref(*b as u8, 3, 1))),
                        2 => subst(&subst(&tx(0xAE22F), b"%3N1", &t(&self.pref(*a as u8, 3, 1))), b"%s", &t(self.loc(*b as u8))),
                        _ => {
                            // 0xAE2E1: "%3E" = Euren n Söldnern (dative plural row, noun 33)
                            let word = |n: u32| assets.text_id(n).iter().filter_map(|&c| crate::input::from_cp437(c)).collect::<String>();
                            let sold = format!("{} {a} {}", word(6 * 107 + 6), word(6 * 107 + 33));
                            let reward = 100 * a + 50 * b;
                            let mut s = subst(&tx(0xAE2E1), b"%3E", &t(&sold));
                            s = subst(&s, b"%3N1", &t(&self.pref((c & 255) as u8, 3, 1)));
                            s = subst_seq(&s, b"%d", &[t(&dotted(b)), t(&dotted(reward))]);
                            let new_city = c >> 8;
                            let tail = assets.name_table((0xB72C0, 2)).get((new_city != 0) as usize).cloned().unwrap_or_default();
                            let tail = if new_city != 0 { tail.replace("%s", self.loc((new_city - 1) as u8)) } else { String::new() };
                            subst(&s, b"%s", &t(&tail))
                        }
                    },
                    (34, i) => {
                        let dest = t(assets.name_table((0xB6C44, 7)).get(*a as usize % 7).map_or("", String::as_str));
                        let city = t(self.loc(*b as u8));
                        match i {
                            0 => subst_seq(&tx(0xAD8F6), b"%s", &[dest, city]),
                            1 => subst(&subst_seq(&tx(0xADA51), b"%s", &[dest, city]), b"%d", &t(&dotted(c))),
                            2 => subst(&tx(0xAD761), b"%s", &city), // the original passes no argument here [C quirk]
                            _ => subst(&tx(0xB3AB6), b"%s", &dest),
                        }
                    }
                    (33, i) => {
                        let book = t(assets.name_table((0xB5530, 49)).get(if i == 0 { *b } else { *a } as usize % 49).map_or("", String::as_str));
                        match i {
                            0 => subst(&subst_seq(&tx(0xAF54A), b"%s", &[book]), b"%d", &t(&dotted(a))),
                            1 => subst(&subst_seq(&tx(0xAF639), b"%s", &[book]), b"%d", &t(&dotted(b))),
                            _ => subst_seq(&subst_seq(&tx(0xB3A56), b"%s", &[book]), b"%d", &[t(&dotted(b)), t(&dotted(c))]),
                        }
                    }
                    // the bribe: nobody to ask, refused, accepted
                    (49, 1) => tx(0xA5B20),
                    (49, 2) => subst(&tx(0xA5902), b"%1N1", &t(&self.pref(*a as u8, 1, 1))),
                    (49, 3) => subst(&tx(0xA59AA), b"%1N1", &t(&self.pref(*a as u8, 1, 1))),
                    (49, _) => subst(&subst_seq(&tx(0xA56C6), b"%w", &[a.to_string().into_bytes(), b.to_string().into_bytes()]), b"%d", &t(&dotted(c))),
                    (40, _) => subst(&subst(&tx(0xB3B09), b"%s", &t(self.good(*a as u8))), b"%d", &t(&dotted(b))),
                    (44, 0) => subst(&tx(0xAF9F7), b"%4N5", &t(&self.pref(*a as u8, 4, 5))),
                    (44, 1) => {
                        // 0xAFA46 "%4N1, vormals %s in %s": office noun by the target's gender, location of the office
                        let g = (self.st.persons[*a as usize].gender() & 1) as usize;
                        let o = match *b { 0..=9 => *c as u32, 10..=12 => *c as u32 + 10, _ => *c as u32 + 23 };
                        let noun = self.assets_text_id(if g == 1 { 1407 + o } else { 72 + o });
                        subst_seq(&subst(&tx(0xAFA46), b"%4N1", &t(&self.pref(*a as u8, 4, 1))), b"%s", &[t(&noun), t(self.loc(*b as u8))])
                    }
                    (44, 2) => subst(&subst(&tx(0xAFB50), b"%d", &t(&dotted(b))), b"%3N1", &t(&self.pref(*a as u8, 3, 1))),
                    (44, _) => subst(&tx(0xAFAA1), b"%2N1", &t(&self.pref(*a as u8, 2, 1))),
                    (47, 0) => subst(&subst_seq(&tx(0xAC911), b"%s", &[t(self.good(*a as u8)), t(self.loc(*b as u8))]), b"%d", &t(&dotted(c))),
                    (47, _) => subst(&subst(&tx(0xAC85B), b"%d", &t(&dotted(a))), b"%s", &t(&two(0xB71B0, *c))),
                    (48, 0) => subst(&subst(&tx(0xAFC7E), b"%4u", &t(&if *a == 1 { "einen Karren".to_string() } else { format!("{a} Karren") })), b"%s", &t(self.loc(*b as u8))),
                    (48, 1) => subst(&subst(&subst(&tx(0xAFBDB), b"%s", &t(self.loc(*b as u8))), b"%4E", &t(&self.transporters(assets, 4, *a as u8, 0))), b"%d", &t(&dotted(c))),
                    (48, 2) => subst(&subst(&tx(0xACABD), b"%s", &t(self.loc(*b as u8))), b"%4E", &t(&self.transporters(assets, 4, *a as u8, 0))),
                    _ => {
                        let s = subst(&subst(&tx(0xAC9ED), b"%d", &t(&dotted(a))), b"%4E", &t(&self.transporters(assets, 4, (b & 255) as u8, 0)));
                        subst_seq(&s, b"%s", &[t(self.loc((b >> 8) as u8)), t(&two(0xB71C0, *c))])
                    }
                };
                plain(&src)
            }
            // plain sub-plot messages without arguments (rules-13 §2): start / end texts; progress markers stay hidden
            SubPlot { kind, step, .. } => {
                let addr = match (*kind, *step) {
                    (32, 0xFF) => 0xA5386,
                    (40, 0xFF) => 0x97450,
                    (41, 0xFF) => 0xAC7F5,
                    (41, 0) => 0xB3BC3,
                    (42, 0xFF) => 0x8957E,
                    (42, 0) => 0xB3C1E,
                    (46, 0xFF) => 0xAE16E,
                    (48, 0xFF) => 0xA9073,
                    _ => return None,
                };
                plain(&assets.text(addr))
            }
            PersonReplaced { .. } => return None,
            // stage 5 (rules-23 §3): the branch itself, and where its shipments go
            SeizedBranch { city, redirect, .. } => {
                let mut line = plain(&subst(&assets.text(0xC11C8), b"%o", &t(self.loc(*city))));
                if let Some(r) = redirect {
                    line.push_str(&plain(&subst(&assets.text(0xC11E8), b"%o", &t(self.loc(*r)))));
                }
                line.replacen('-', "- ", 1)
            }
            // stage 4: a residence or construction site at half its value
            SeizedResidence { city, word, site } => {
                let o = t(self.loc(*city));
                let src = if *site { subst(&assets.text(0xC1148), b"%o", &o) } else { subst(&subst(&assets.text(0xC119C), b"%1W", &t(&self.wform(assets, 1, *word))), b"%o", &o) };
                plain(&src).replacen('-', "- ", 1)
            }
            // the original has no refusal text for these guards and prints nothing (rules-66 §7)
            Refused(why) => {
                eprintln!("refused: {why}");
                return None;
            }
            BattlePending { .. } => "Ein Kampf konnte nicht ausgetragen werden (SZENARIO.DAT fehlt).".into(),
            AttackDetected { .. } => return None, // told by the captions on the field (rules-50 §2)
            // told in the pre-fight window (rules-52 §2)
            RansomPaid { .. } => return None,
            AllPlayersOut => return None, // told by the game-over scene
            Certificate { .. } => return None, // the charter scene (human turn start only, rules-27 §2)
            // contest announced or cancelled (EXE 0x9A45D, 0xBA8D4)
            ContestStart { city, judge, cancelled } => {
                if *cancelled {
                    plain(&assets.text(0xBA8D4))
                } else {
                    plain(&subst(&subst(&assets.text(0x9A45D), b"%s", &t(self.loc(*city))), b"%1N5", &t(&self.pref(*judge, 1, 5))))
                }
            }
            // newspaper contest result (rules-13 §3.2): opening, per discipline the places (third first), closing
            Contest { good, fee, winners, city, judge, first, last, .. } => {
                let mut out: Vec<String> = vec![];
                if *first {
                    out.push(plain(&subst(&subst(&assets.text(0xA7D49), b"%s", &t(self.loc(*city))), b"%1N1", &t(&self.pref(*judge, 1, 1)))));
                }
                let superl = assets.name_table((0xB5D58, 10)).get(*good as usize).cloned().unwrap_or_default();
                let head = |addr: u32| subst_seq(&assets.text(addr), b"%s", &[t(&superl), t(self.good(*good))]);
                if winners.is_empty() {
                    out.push(plain(&head(0xA7FC4)));
                } else {
                    out.push(plain(&head(0xA7E39)));
                    for (k, &p) in winners.iter().enumerate().rev() {
                        let addr = [0xA7E86_u32, 0xA7EEC, 0xA7F40][k];
                        let pat: &[u8] = if k == 1 { b"%4N1" } else { b"%1N1" };
                        let form = if k == 1 { 4 } else { 1 };
                        out.push(plain(&subst(&subst(&assets.text(addr), pat, &t(&self.pref(p, form, 1))), b"%d", ([4, 3, 2][k] * fee).to_string().as_bytes())));
                    }
                }
                if *last {
                    out.push(plain(&assets.text(0xA7F8E)));
                }
                out.join(" ")
            }
            // castle toll income (EXE 0xC0934 / 0xC09AC, rules-12 §3.3): castle, gross, county share, county
            CastleIncome { castle, gross, share, vogt, vogt_by, .. } => {
                let k = *castle as u32;
                let loc = |l: u32| t(self.loc(l as u8));
                let mut src = subst_seq(&assets.text(0xC0934), b"%o", &[loc(14 + k), loc(10 + k)]);
                src = subst_seq(&src, b"%d", &[t(&dotted(gross)), t(&dotted(share))]);
                if *vogt_by != 0xFF {
                    src.extend(subst(&subst(&assets.text(0xC09AC), b"%1N3", &t(&self.pref(*vogt_by, 1, 3))), b"%d", &t(&dotted(vogt))));
                }
                plain(&src)
            }
            // EXE 0xC1D14 (rules-12 §3.3): a human Vogt's share of a computer-owned castle's toll
            VogtToll { vogt, amount, castle } => {
                let src = subst(&subst(&assets.text(0xC1D14), b"%1N3", &t(&self.pref(*vogt, 1, 3))), b"%o", &t(self.loc(14 + *castle)));
                plain(&subst(&src, b"%d", &t(&dotted(amount))))
            }
            // EXE 0xC1060 (rules-12 §3.4)
            LoanDue { lender, amount } => {
                let who = t(&self.pref(*lender, 1, 3));
                let poss: &[u8] = if self.st.persons[*lender as usize].gender() == 0 { b"sein" } else { b"ihr" };
                let src = subst(&subst(&assets.text(0xC1060), b"%1N3", &who), b"%qen", &[poss, b"en"].concat());
                plain(&subst(&src, b"%d", &t(&dotted(amount))))
            }
            // sabotage in the year-end report (EXE 0xC1A8C, 0xC1B28, text ids 3139, 3141; rules-12 §3.6)
            Sabotage { saboteur, target, kind, loc, damage, rival, .. } => {
                let (s, tg) = (*saboteur, *target);
                let d = damage.to_string().into_bytes();
                let src = match kind {
                    // the human target's rival (0xC1BB4 male target / 0xC1C54 female): (s, t, s, place, damage)
                    0 if *rival => {
                        let a = if self.st.persons[tg as usize].gender() & 1 == 1 { 0xC1C54 } else { 0xC1BB4 };
                        let src = subst(&subst(&assets.text(a), b"%3N3", &t(&self.pref(s, 3, 3))), b"%2N0", &t(&self.pref(tg, 2, 0)));
                        subst(&subst(&subst(&src, b"%f", &t(&self.gegenspieler(s))), b"%o", &t(self.loc(*loc))), b"%d", &d)
                    }
                    2 | 1 => subst(&subst(&subst(&subst(&assets.text(if *kind == 2 { 0xC1A8C } else { 0xC1B28 }), b"%2N0", &t(&self.pref(s, 2, 0))), b"%o", &t(self.loc(*loc))), b"%2N3", &t(&self.pref(tg, 2, 3))), b"%d", &d),
                    0 => subst(&subst(&subst(&subst(&assets.text_id(3139), b"%2N3", &t(&self.pref(s, 2, 3))), b"%2N0", &t(&self.pref(tg, 2, 0))), b"%o", &t(self.loc(*loc))), b"%d", &d),
                    _ => subst(&subst(&subst(&assets.text_id(3141), b"%2N0", &t(&self.pref(s, 2, 0))), b"%2N3", &t(&self.pref(tg, 2, 3))), b"%d", &d),
                };
                plain(&src)
            }
            // a human insults a computer person (rules-24 §1): box 0x9B789 with place, taunt and adverb
            Insulted { insulter, target, rolls: [taunt, _, adverb, place] } => {
                let tab = |a: u32, n: usize, k: u8| assets.text_table((a, n)).get(k as usize).cloned().unwrap_or_default();
                let s = subst(&subst(&assets.text(0x9B789), b"%4N0", &t(&self.pref(*target, 4, 0))), b"%1N0", &t(&self.pref(*insulter, 1, 0)));
                plain(&subst_seq(&s, b"%s", &[tab(0xB5EF8, 6, *place), tab(0xB5434, 12, *taunt), tab(0xB5F30, 8, *adverb)]))
            }
            // the gift reaction (rules-39 §7): 0x96657 "$Z$L%1N0 ist %s.$N$N" with a word of table 0xB5818, its half
            // line above and blank line below kept (live A/B 2026-11-22)
            GiftReaction { target, word } => {
                let w = assets.name_table((0xB5818, 15)).get(*word as usize).cloned().unwrap_or_default();
                raw_line(&subst(&subst(&assets.text(0x96657), b"%1N0", &t(&self.pref(*target, 1, 0))), b"%s", &t(&w)), None)
            }
            // prisoners (rules-38 §7): castle 0xC0E4C / camp 0xC0EDC when some joined, else 0xC0F28 / 0xC0F60, then the
            // ones that vanished (0xC0F90 / 0xC0FBC)
            Prisoners { person, prop, taken, joined } if self.st.persons[*person as usize].human() => {
                let castle = self.st.props[*prop as usize].kind() == mad_sim::state::KIND_CASTLE;
                let place = t(self.loc(self.st.props[*prop as usize].loc()));
                let mut s = match (*joined > 0, castle) {
                    (true, true) => subst(&subst(&assets.text(0xC0E4C), b"%o", &place), b"%4u", &Self::u_form(assets, *joined as i32, 42)),
                    (true, false) => subst(&subst(&assets.text(0xC0EDC), b"%4u", &Self::u_form(assets, *joined as i32, 42)), b"%o", &place),
                    (false, true) => subst(&assets.text(0xC0F28), b"%o", &place),
                    (false, false) => subst(&assets.text(0xC0F60), b"%o", &place),
                };
                match *taken as i32 - *joined as i32 {
                    0 => {}
                    1 => s.extend(assets.text(0xC0FBC)),
                    d => s.extend(subst(&assets.text(0xC0F90), b"%d", &t(&dotted(d)))),
                }
                plain(&s)
            }
            Prisoners { .. } => return None,
            // EXE 0xBAB68: the year the auction is held (the sim year is already the new one)
            ManoeuvrePic { .. } | PrivCost { .. } | Song { .. } | IndulgenceCity { .. } | ApplyRivals(_) | Scores { .. } | Notice { .. } => return None,
            Herald { .. } => return None, // its own page, with the EXE's layout
            AuctionAnnounced { years } => plain(&subst(&assets.text(0xBAB68), b"%l", (self.st.year as u32 + *years as u32).to_string().as_bytes())),
            // transport kinds (rules-12 §3.9): with = good, aux = (count, type), city = old destination, amount = new destination
            Fate { kind: k @ (56 | 61 | 62 | 63 | 66 | 67 | 68), amount, city, aux, with, .. } if *city != 0xFF => {
                let (good, dest, new) = (t(self.good(*with)), t(self.loc(*city)), t(self.loc(*amount as u8)));
                let src = match k {
                    56 => subst_seq(&assets.text_id(3320), b"%s", &[good, dest]),
                    61 => subst(&subst(&assets.text_id(3324), b"%4E", &t(&self.transporters(assets, 4, aux[0], aux[1]))), b"%s", &dest),
                    62 => subst_seq(&subst(&assets.text_id(3325), b"%4E", &t(&self.transporters(assets, 4, aux[0], aux[1]))), b"%s", &[dest, new]),
                    63 => subst(&assets.text_id(3326), b"%1u", &t(&self.transporters(assets, 5, 1, aux[1]))),
                    66 => subst_seq(&assets.text_id(3329), b"%s", &[good, dest]),
                    67 => subst_seq(&assets.text_id(3330), b"%s", &[dest, good]),
                    _ => subst_seq(&assets.text_id(3331), b"%s", &[dest, new]),
                };
                plain(&src)
            }
            // orphanage donation (21), candle fire (50), building subsidy (54)
            Fate { kind: k @ (21 | 50 | 54), amount, city, aux, .. } if *city != 0xFF => {
                let loc = t(self.loc(*city));
                match k {
                    21 => {
                        let county = t(self.loc((*city - 1) / 3 + 10));
                        plain(&subst_seq(&subst(&assets.text_id(3290), b"%d", &t(&dotted(amount))), b"%s", &[loc, county]))
                    }
                    50 => {
                        let colour = assets.text_id(3014 + aux[0] as u32 % 3);
                        let s = subst_seq(&assets.text_id(3013), b"%s", &[loc, colour]);
                        plain(&subst(&s, b"%d", &t(&dotted(amount))))
                    }
                    _ => {
                        let s = subst(&assets.text_id(3318), b"%2W", &t(&self.wform(assets, 2, u16::from_le_bytes(*aux))));
                        plain(&subst(&subst_seq(&s, b"%s", &[loc]), b"%d", &t(&dotted(amount))))
                    }
                }
            }
            // gambling win of transporters (65), worker bonuses (72, 73), aunt's inheritance (77)
            Fate { kind: k @ (65 | 72 | 73 | 77), amount, aux, .. } => {
                let amt = amount.to_string().into_bytes();
                match k {
                    65 => plain(&subst(&assets.text_id(3328), b"%4u", &Self::u_form(assets, aux[0] as i32, 19 + aux[1] as u32))),
                    72 => plain(&subst(&assets.text_id(3335), b"%d", &t(&dotted(aux[0])))),
                    73 => plain(&subst_seq(&assets.text_id(3336), b"%d", &[aux[0].to_string().into_bytes(), amt])),
                    _ => plain(&subst_seq(&assets.text_id(3340), b"%d", &[aux[0].to_string().into_bytes(), amt])),
                }
            }
            // residence kinds (rules-12 §3.9, %kW of §1.7): aux = residence word, with = fire variant of 78
            Fate { kind: k @ (29 | 53 | 78), amount, city, aux, with, .. } if *city != 0xFF => {
                let word = u16::from_le_bytes(*aux);
                let src = match k {
                    29 => subst(&assets.text_id(3296), b"%2W", &t(&self.wform(assets, 2, word))),
                    53 => subst(&subst(&subst(&assets.text_id(3317), b"%1W", &t(&self.wform(assets, 1, word))), b"%s", &t(self.loc(*city))), b"%d", &t(&dotted(amount))),
                    _ => {
                        let w = word & !0xE0;
                        if *with == 1 {
                            subst(&subst(&assets.text_id(3158), b"%2W", &t(&self.wform(assets, 2, w))), b"%o", &t(self.loc(*city)))
                        } else {
                            subst(&subst(&assets.text_id(3159), b"%4W", &t(&self.wform(assets, 4, w))), b"%o", &t(self.loc(*city)))
                        }
                    }
                };
                plain(&src)
            }
            // kinds with a second person (rules-12 §3.9)
            Fate { kind: k @ (4 | 24 | 32 | 47 | 51 | 57 | 75), amount, city, with, .. } if *with != 0xFF => {
                let x = *with;
                let male = self.st.persons[x as usize].gender() == 0;
                let word = |id: u32| assets.text_id(id);
                let src = match k {
                    4 => {
                        let s = subst(&assets.text_id(3274), b"%1N5", &t(&self.pref(x, 1, 5)));
                        let s = subst(&s, b"%4p", if male { b"sein Warenlager" } else { b"ihr Warenlager" });
                        subst(&s, b"%G", &t(&genitive(&self.first_name(x as usize))))
                    }
                    24 => {
                        let s = subst(&assets.text_id(2908), b"%3N5", &t(&self.pref(x, 3, 5)));
                        let desc = word(if male { 2909 } else { 2910 });
                        let poss = word(if male { 2911 } else { 2912 });
                        subst_seq(&s, b"%s", &[t(self.loc(self.st.persons[self.me().unwrap_or(0)].loc())), desc, poss])
                    }
                    32 => subst(&subst(&assets.text_id(3299), b"%3N5", &t(&self.pref(x, 3, 5))), b"%s", &t(self.loc(*city))),
                    47 => subst(&subst(&assets.text_id(3313), b"%3N5", &t(&self.pref(x, 3, 5))), b"%d", &t(&dotted(amount))),
                    51 => subst(&subst(&assets.text_id(3073), b"%4N5", &t(&self.pref(x, 4, 5))), b"%s", &word(if male { 3074 } else { 3075 })),
                    57 => subst(&subst(&assets.text_id(3076), b"%3N5", &t(&self.pref(x, 3, 5))), b"%s", &word(if male { 3077 } else { 3078 })),
                    _ => subst(&subst(&assets.text_id(3338), b"%3N5", &t(&self.pref(x, 3, 5))), b"%d", &t(&dotted(amount))),
                };
                plain(&src)
            }
            // owner of the other camp or castle (85, 86, 94, 95) and the carter who defects (64)
            Fate { kind: k @ (85 | 86 | 94 | 95), city, with, .. } if *with != 0xFF && *city != 0xFF => {
                let id = match k { 85 => 3347, 86 => 3348, 94 => 3356, _ => 3357 };
                let s = subst(&assets.text_id(id), b"%s", &t(self.loc(*city)));
                plain(&subst(&s, b"%3N5", &t(&self.pref(*with, 3, 5))))
            }
            Fate { kind: 64, amount, aux, .. } if *amount >= 0 && (*amount as usize) < self.st.persons.len() => {
                let s = subst(&assets.text_id(3327), b"%1u", &t(&self.transporters(assets, 5, 1, aux[1])));
                plain(&subst(&s, b"%3N5", &t(&self.pref(*amount as u8, 3, 5))))
            }
            // rescue of a woman (5), cheese toast (25), directors' course (35)
            Fate { kind: 5, city, aux, .. } if *city != 0xFF => {
                let a = subst(&assets.text_id(538), b"%s", &t(self.loc(*city)));
                format!("{} {}", plain(&a), plain(&assets.text_id(539 + aux[0] as u32 % 2)))
            }
            Fate { kind: 25, aux, .. } => plain(&subst(&subst(&assets.text_id(645), b"%b", aux[0].to_string().as_bytes()), b"%s", &assets.text_id(646 + aux[1] as u32 % 2))),
            Fate { kind: 35, amount, aux, .. } => {
                let tav = assets.name_table((0xB5AE4, 30)).get(aux[0] as usize).cloned().unwrap_or_default();
                let inner = subst(&assets.text_id(753 + aux[1] as u32 % 2), b"%s", &t(&tav));
                plain(&subst(&subst(&assets.text_id(752), b"%d", &t(&dotted(amount))), b"%s", &inner))
            }
            // robber captain and relic kinds (rules-12 §3.9): aux = (nickname, epithet) or (relic, -)
            Fate { kind: k @ (82 | 87 | 98 | 99), amount, city, aux, .. } if *city != 0xFF => {
                let loc = t(self.loc(*city));
                let nick = t(assets.name_table((0xB6AA8, 18)).get(aux[0] as usize).map_or("", String::as_str));
                let epi = t(assets.name_table((0xB6410, 10)).get(aux[1] as usize).map_or("", String::as_str));
                let src = match k {
                    82 => subst_seq(&assets.text_id(3344), b"%s", &[nick, epi, loc]),
                    87 => subst(&subst_seq(&assets.text_id(3349), b"%s", &[loc, nick, epi]), b"%d", &t(&dotted(amount))),
                    98 => subst_seq(&assets.text_id(2640), b"%s", &[loc, assets.text_id(2641 + aux[0] as u32 % 9)]),
                    _ => subst(&assets.text_id(if *city <= 16 { 3160 } else { 3161 }), b"%o", &loc),
                };
                plain(&src)
            }
            // tavern and book kinds (rules-12 §3.9): names from 0xB5AE4 (30 taverns) and 0xB5530 (49 books)
            Fate { kind: k @ (11 | 12 | 13 | 17 | 18 | 45 | 46), amount, city, aux, .. } if aux[0] != 0xFF => {
                let tavern = |i: u8| assets.name_table((0xB5AE4, 30)).get(i as usize).cloned().unwrap_or_default();
                let book = |i: u8| assets.name_table((0xB5530, 49)).get(i as usize).cloned().unwrap_or_default();
                let t_city = self.loc(*city).to_string();
                let (id, names) = match k {
                    11 => (3280, vec![tavern(aux[0])]),
                    12 => (3281, vec![tavern(aux[0])]),
                    13 => (3282, vec![tavern(aux[0])]),
                    17 => (3286, vec![book(aux[0]), t_city.clone()]),
                    18 => (3287, vec![book(aux[0]), t_city.clone()]),
                    45 => (3311, vec![book(aux[0]), self.good(aux[1]).to_string()]),
                    _ => (3312, vec![book(aux[0]), self.good(aux[1]).to_string()]),
                };
                let src = subst_seq(&assets.text_id(id), b"%s", &names.iter().map(|n| t(n)).collect::<Vec<_>>());
                plain(&subst(&src, b"%d", &t(&dotted(amount))))
            }
            // kinds 14 (sales contract) and 41 print nothing (rules-12 §3.9)
            Fate { kind: 14 | 41, .. } => return None,
            // illness (rules-12 §3.9, kind 19): one of the twelve ailments of 0xB683C
            Fate { kind: 19, aux, .. } => {
                let ill = assets.text_table((0xB683C, 12)).get(aux[0] as usize).cloned().unwrap_or_default();
                plain(&subst(&assets.text_id(3288), b"%s", &ill))
            }
            // fate events whose texts take only a city and/or an amount (text ids of rules-12 §3.9)
            Fate { kind, amount, city, .. } if FATE_IDS.iter().any(|f| f.0 == *kind) => {
                let id = FATE_IDS.iter().find(|f| f.0 == *kind).unwrap().1;
                let mut src = assets.text_id(id);
                if *city != 0xFF {
                    src = subst(&subst(&src, b"%s", &t(self.loc(*city))), b"%o", &t(self.loc(*city)));
                }
                let line = plain(&subst(&src, b"%d", &t(&dotted(amount))));
                // a place text without a drawn property: the original prints with a garbage index, ours nothing (rules-66 §7)
                if line.contains('%') {
                    eprintln!("fate {kind} without a place: {line}");
                    return None;
                }
                line
            }
            // gossip (ids 2401, 2408 + n) and the mistress/lover of a human (ids 2851, 2856), rules-12 §3.8
            Gossip { person, text } => plain(&subst(&subst(&assets.text_id(2401), b"%1N5", &t(&self.pref(*person, 1, 5))), b"%s", &assets.text_id(2408 + *text as u32))),
            MistressLeft { person } | MistressDemand { person, .. } => {
                let female = self.st.persons[*person as usize].gender() == 1;
                // "Eu%s %s": Euer Geliebter for a woman, Eure Mätresse for a man
                let (end, noun): (&[u8], &[u8]) = if female { (b"er", b"Geliebter") } else { (b"re", b"M\x84tresse") };
                let (id, amount) = match e { MistressDemand { amount, .. } => (2856, *amount), _ => (2851, 0) };
                let src = subst_seq(&assets.text_id(id), b"%s", &[end.to_vec(), noun.to_vec()]);
                plain(&subst(&src, b"%d", &t(&dotted(amount))))
            }
            // told on the auction hall (`Ask::AuctionLine`)
            AuctionSold { .. } => return None,
            // trial verdict (EXE 0xA5F9E/0xA5FC0 and the sentence texts 0xA802D..0xA81CC; rules-04b §3.5)
            Verdict { defendant, guilty, sentence, fine, office } => {
                let d = *defendant;
                let who = t(&self.pref(d, 1, 1));
                if !*guilty {
                    return Some(plain(&subst(&assets.text(0xA5FC0), b"%1N1", &who)));
                }
                let head = plain(&subst(&assets.text(0xA5F9E), b"%1N1", &who));
                let body = match sentence {
                    // rules-38 §9: the pillory is always 7 hours; the office lost is named as it was before the sentence
                    0 => subst(&subst(&assets.text(0xA802D), b"%1N1", &who), b"%w", b"7"),
                    5 => {
                        let o = self.office(office.0, office.1);
                        let rank = if o.is_empty() { "ohne Amt".to_string() } else { format!("{o} in {}", self.loc(office.0)) };
                        subst(&subst(&assets.text(0xA818A), b"%3N1", &t(&self.pref(d, 3, 1))), b"%1N4", &t(&rank))
                    }
                    1 => subst(&subst(&assets.text(0xA8060), b"%1N1", &who), b"%d", &t(&dotted(fine))),
                    2 => subst(&subst(&subst(&assets.text(0xA80A7), b"%1N1", &who), b"%w", b"3"), b"%d", (fine * 2 / 3).to_string().as_bytes()),
                    3 => {
                        let g = (self.st.persons[d as usize].gender() & 1) as usize;
                        let new = self.st.persons[d as usize].title() as usize;
                        let name = |i: usize| t(self.tables.titles[g].get(i.min(9)).map_or("", String::as_str));
                        let s = subst(&subst(&assets.text(0xA80FE), b"%3N1", &t(&self.pref(d, 3, 1))), b"%1N0", &t(&self.pref(d, 1, 0)));
                        subst_seq(&s, b"%s", &[name(new + 1), name(new)])
                    }
                    4 => subst(&assets.text(0xA8154), b"%1N1", &who),
                    6 => subst(&assets.text(0xA81CC), b"%1N1", &who),
                    _ => return Some(head),
                };
                format!("{head} {}", plain(&body))
            }
            // dismissal by 1..3 computer appointers (EXE 0xC170C, 0xC1734, 0xC1790; rules-12 §3.7)
            Dismissed { person, by, loc, idx, rival } if !by.is_empty() && by.len() <= 3 && (self.st.persons[*person as usize].human() || *loc >= 13 || *rival != 0xFF) => {
                let a: Vec<String> = by.iter().map(|&x| self.pref(x, 1, 3)).collect();
                // the office is already vacated: name the one that was lost
                let g = (self.st.persons[*person as usize].gender() & 1) as usize;
                let o = match *loc { 0..=9 => *idx as u32, 10..=12 => *idx as u32 + 10, _ => *idx as u32 + 23 };
                let noun = self.assets_text_id(if g == 1 { 1407 + o } else { 72 + o });
                // %0F: the office holder (accusative), or with a rival "%4N3, %2N0 %f" (rules-12 §1.6)
                let who = match *rival {
                    0xFF => t(&format!("{}, {noun} in {}", self.pref(*person, 4, 1), self.loc(*loc))),
                    r => t(&format!("{}, {} {}", self.pref(*person, 4, 1), self.pref(r, 2, 0), self.gegenspieler(*person))),
                };
                let src = match by.len() {
                    1 => subst(&subst(&assets.text(0xC170C), b"%1N3", &t(&a[0])), b"%0F", &who),
                    2 => {
                        let mut s = subst(&subst_seq(&assets.text(0xC1734), b"%1N3", &[t(&a[0]), t(&a[1])]), b"%0F", &who);
                        // the third appointer objected in vain (0xC1768), unless it is the rival's human
                        let apps = self.st.appointers(*loc as usize, *idx as usize);
                        if let (3, Some(&o)) = (apps.len(), apps.iter().find(|&&x| !by.contains(&(x as u8))))
                            && o as u8 != *rival
                        {
                            s.extend(subst(&assets.text(0xC1768), b"%3N3", &t(&self.pref(o as u8, 3, 3))));
                        }
                        s
                    }
                    _ => subst(&subst_seq(&assets.text(0xC1790), b"%1N3", &[t(&a[0]), t(&a[1]), t(&a[2])]), b"%0F", &who),
                };
                plain(&src)
            }
            // divine events (rules-12 §3.10, text id 2350 + kind); the tornado (8) needs nine more numbers
            // tornado (EXE text id 2358, rules-15 §4.1): the nine numbers in text order
            Tornado { a, b, n, churches, museums, hospitals } => {
                let (ca, cb) = (t(self.loc(*a)), t(self.loc(*b)));
                let s = subst_seq(&assets.text_id(2358), b"%s", &[ca.clone(), ca, cb]);
                let nums = [churches[0], museums[0], hospitals[0], *n, churches[1], museums[1], hospitals[1], *n / 2 + 1];
                let s = subst_seq(&s, b"%w", &nums.map(|v| v.to_string().into_bytes()));
                plain(&s)
            }
            Divine { kind, city, with, illness } if *kind != 8 => {
                let mut src = assets.text_id(2350 + *kind as u32);
                if *with != 0xFF {
                    src = subst(&subst(&src, b"%1N5", &t(&self.pref(*with, 1, 5))), b"%1N0", &t(&self.pref(*with, 1, 0)));
                }
                if *kind == 9 {
                    let ill = assets.name_table((0xB683C, 12)).get(*illness as usize).cloned().unwrap_or_default();
                    src = subst_seq(&src, b"%s", &[t(self.loc(*city)), t(&ill)]);
                } else if *city != 0xFF {
                    src = subst(&src, b"%s", &t(self.loc(*city)));
                }
                // `%v` swallows an argument (the illness of kinds 1 and 6)
                plain(&subst(&src, b"%v", b""))
            }
            // world fate (rules-12 §3.11): ids 3360..3363, siege rumour 3079 + 3081
            // rules-32: year-end lines straight from the EXE, `%` codes filled in order
            // small values are text ids, larger ones addresses
            WorldText { text, args } => plain(&self.exe_fmt(assets, &if *text < 0x10000 { assets.text_id(*text) } else { assets.text(*text) }, args)),
            WorldFate { kind, aux } => {
                let n = |i: u8, table: (u32, usize)| t(assets.name_table(table).get(i as usize).map_or("", String::as_str));
                match kind {
                    0 | 1 => plain(&subst(&assets.text_id(3360 + *kind as u32), b"%s", &t(self.good(aux[0])))),
                    2 | 3 => plain(&subst(&assets.text_id(3360 + *kind as u32), b"%s", &t(self.loc(aux[0])))),
                    _ => {
                        // aux[1] bit 7: the shipments turned back (3080), else nothing more happens (3081)
                        let besieger = n(aux[1] & 0x7F, (0xB686C, 12));
                        let a = subst_seq(&assets.text_id(3079), b"%s", &[besieger.clone(), t(self.loc(aux[0]))]);
                        let b = if aux[1] & 0x80 != 0 { assets.text_id(3080) } else { subst(&assets.text_id(3081), b"%s", &besieger) };
                        format!("{} {}", plain(&a), plain(&b))
                    }
                }
            }
            // the assistant's hints (EXE 0xC09CC..0xC0C2C); the transporter word (`%4e`) is read as the plural name
            Tip { kind, a, b, loc } => {
                let name = |s: &str| t(s);
                let src = match (kind, a) {
                    (0, _) => subst(&subst(&assets.text(0xC09CC), b"%4e", &name(branch::TRANSPORTER_NAMES[*a as usize % branch::TRANSPORTER_NAMES.len()])), b"%o", &t(self.loc(*loc))),
                    (1, _) => subst(&assets.text(0xC0A04), b"%l", self.st.year.to_string().as_bytes()),
                    (2, _) => assets.text(0xC0A94),
                    (3, _) => assets.text(0xC0AEC),
                    (_, 0) => subst(&assets.text(0xC0B7C), b"%s", &t(self.good(*b))),
                    (_, 1) => subst(&assets.text(0xC0BD4), b"%s", &t(self.good(*b))),
                    _ => subst(&assets.text(0xC0C2C), b"%o", &t(self.loc(*loc))),
                };
                plain(&src)
            }
            // debt seizure and jury court (EXE 0xC10D0, 0xC14B0..0xC1514, 0xC1320, 0xC13A8, 0xC1470)
            Seized { person, goods, transporters, storage } => {
                let mut lines = vec![plain(&subst(&assets.text(0xC10D0), b"%1N0", &t(&self.person_name(*person))))];
                for (n, a) in [(goods, 0xC14B0), (transporters, 0xC14E0), (storage, 0xC1514)] {
                    if *n > 0 {
                        lines.push(plain(&subst(&assets.text(a), b"%d", &t(&dotted(n)))).replacen('-', "- ", 1));
                    }
                }
                lines.join(" ")
            }
            Jury { .. } => return None, // told by the jury scene
            // EXE 0xC167C, 0x9B6BC / 0x9B70C; the name is carried by the event (the person is cleared)
            Bankrupt { .. } => return None, // told by the dungeon scene
            PlayerOut { name, no_kids, .. } => plain(&subst(&assets.text(if *no_kids { 0x9B6BC } else { 0x9B70C }), b"%1N0", &t(name))),
            GameOver => return None, // the mission's quit box (rules-39 §8)
            Divine { .. } | TrialCosts { .. } => return None, // the tornado is told by `Tornado`
            Trial { .. } => return None, // told by the court scenes
            // siege texts (EXE 0x9F0E7, 0x9F21D, 0x9F9E0, 0x9F11A; rules-07 §6)
            SiegeArrives { city, besieger, army } => {
                // rules-21 §2: the human's own army (0x9F51A) or a computer's, told in the year-end report (0x9F08A)
                let src = if self.st.persons[*besieger as usize].human() { assets.text(0x9F51A) } else { subst(&assets.text(0x9F08A), b"%1N5", &t(&self.pref(*besieger, 1, 5))) };
                plain(&subst(&subst(&src, b"%d", &t(&dotted(army))), b"%o", &t(self.loc(*city))))
            }
            SpyNotified { target, .. } => plain(&subst(&assets.text(0xC0CBC), b"%4N5", &t(&self.pref(*target, 4, 5)))),
            SiegeGoesOn { city, besieger } => plain(&subst(&subst(&assets.text(0x9F0E7), b"%o", &t(self.loc(*city))), b"%4N5", &t(&self.pref(*besieger, 4, 5)))),
            SiegeWon { besieger, city, .. } if !self.st.persons[*besieger as usize].human() => plain(&subst(&subst(&assets.text(0x9F21D), b"%3N1", &t(&self.pref(*besieger, 3, 1))), b"%o", &t(self.loc(*city)))),
            Tribute { city, amount } => plain(&subst(&subst(&assets.text(0x9FAAB), b"%o", &t(self.loc(*city))), b"%d", &t(&dotted(amount)))),
            // the siege collapses (rules-21 §2.2: 0x9F11A with city, owner), or a human's conquest ends with the army dissolved (0x9F9E0)
            SiegeCollapsed { city, besieger } => plain(&subst(&subst(&assets.text(0x9F11A), b"%o", &t(self.loc(*city))), b"%3N1", &t(&self.pref(*besieger, 3, 1)))),
            // a human's conquest (rules-21 §2.2): plunder line, ransom and tribute line, army dissolved
            SiegeWon { city, choice, tribute, .. } => {
                let o = t(self.loc(*city));
                let mut out = vec![];
                if *choice == 0 || *choice == 2 {
                    out.extend(subst(&assets.text(0x9F7CB), b"%o", &o));
                }
                if *choice == 1 || *choice == 2 {
                    let (amount, years) = *tribute;
                    let src = subst_seq(&assets.text(0x9F91E), b"%d", &[t(&dotted(2 * amount)), t(&dotted(amount))]);
                    out.extend(subst(&subst(&src, b"%o", &o), b"%l", (self.st.year as u32 + u32::from(years)).to_string().as_bytes()));
                }
                out.extend(subst(&assets.text(0x9F9E0), b"%o", &o));
                // printed under the answered question with the texts' own half lines (live A/B 2026-11-22)
                raw_line(&out, None)
            }
            // relief (rules-21 §3): the meeting text of the owner's kind, then the outcome
            Relief { city, ok, besieger, owner, pop, .. } => {
                let human = self.st.persons[*owner as usize].human();
                let o = t(self.loc(*city));
                let head = if human {
                    subst(&subst(&assets.text(0xA3B25), b"%o", &o), b"%3N2", &t(&self.pref(*besieger, 3, 2)))
                } else {
                    subst(&subst(&subst(&assets.text(0xA3916), b"%1N5", &t(&self.pref(*owner, 1, 5))), b"%3N5", &t(&self.pref(*besieger, 3, 5))), b"%o", &o)
                };
                let tail = match (human, ok) {
                    (true, true) => subst(&assets.text(0xA3BA8), b"%d", &t(&dotted(pop))),
                    (true, false) => subst(&assets.text(0xA3C8A), b"%2N1", &t(&self.pref(*besieger, 2, 1))),
                    (false, true) => assets.text(0xA39D3),
                    (false, false) => subst(&assets.text(0xA396E), b"%2N1", &t(&self.pref(*owner, 2, 1))),
                };
                plain(&[head, tail].concat())
            }
            // a human's relief army finds no siege (rules-21 §3.1); computers say nothing
            ReliefNoSiege { city, owner } if self.st.persons[*owner as usize].human() => plain(&subst(&assets.text(0xA3D7D), b"%o", &t(self.loc(*city)))),
            Feast { .. } => return None, // told by the feast scene
            Duel { .. } => return None, // told by the duel scene (human duellists only)
            Succession { .. } => return None, // told by the notary scene
            TrialCharge { .. } | TrialTalk { .. } | TrialMoney { .. } => return None, // told by the court scene
            // shipment mishaps (rules-23 §2): the six loss texts of table 0xB6E54, balloon / steam cart drift
            TransporterLost { kind, dest, .. } => {
                let text = assets.text_table((0xB6E54, 6)).get(*kind as usize).cloned().unwrap_or_default();
                plain(&[b"$B".to_vec(), subst(&text, b"%s", &t(self.loc(*dest)))].concat())
            }
            OffCourse { kind, old, new, .. } => {
                let src = assets.text(if *kind == 2 { 0xBA380 } else { 0xBA3D0 });
                plain(&subst_seq(&src, b"%o", &[t(self.loc(*old)), t(self.loc(*new))]))
            }
            // the doctor (rules-23 §6): `level` is the table index 11 - min(life / 2, 11)
            Health { level } => {
                let line = assets.text_table((0xB66D4, 12)).get(*level as usize).cloned().unwrap_or_default();
                plain(&subst(&assets.text(0xA1E09), b"%s", &line))
            }
            // a computer buys a castle (rules-23 §4.1); the offer to a human is the `BuyCastle` dialog
            CastleSold { prop, buyer, price } if !self.st.persons[*buyer as usize].human() => {
                let src = subst(&subst(&assets.text(0xC1DD8), b"%o", &t(self.loc(self.st.props[*prop as usize].loc()))), b"%d", &t(&dotted(price)));
                plain(&subst(&src, b"%1N5", &t(&self.pref(*buyer, 1, 5))))
            }
            CastleSold { .. } => return None,
            DuelForbidden => plain(&assets.text(0xBCC70)),
            // residence confirmations (rules-23 §7.2): the build date, or the extension note; renovating asks, no note
            ResidenceOrdered { kind: 0, word, years, .. } => {
                let src = subst(&assets.text(0xBEAA8), b"%1W", &t(&self.wform(assets, 1, *word)));
                plain(&subst(&src, b"%l", (self.st.year as u32 + u32::from(*years)).to_string().as_bytes()))
            }
            ResidenceOrdered { kind: 1, .. } => plain(&assets.text(0xBED48)),
            ResidenceOrdered { .. } => return None,
            // opening a branch (rules-23 §7.1)
            BranchBought { prop, .. } if !self.st.persons[self.st.props[*prop as usize].owner() as usize].human() => plain(&subst(&assets.text(0xBE548), b"%s", &t(self.loc(self.st.props[*prop as usize].loc())))),
            BranchBought { .. } => return None, // a human's: the message box with speech (rules-16)
            // rules-12 §3.1 (EXE 0xC0874): the emperor confiscates part of the fortune
            BranchSurcharge { emperor, percent, amount } => {
                let name = assets.text_table((0xB707C, 5)).get(*emperor as usize).cloned().unwrap_or_default();
                let src = subst(&subst(&assets.text(0xC0874), b"%s", &name), b"%d%%", format!("{percent}%").as_bytes());
                plain(&subst(&src, b"%d", &t(&dotted(amount))))
            }
            // rules-17 §5 (EXE 0xC0C84): the blackmailer lets go
            BlackmailEnded { by } => plain(&subst(&assets.text(0xC0C84), b"%1N5", &t(&self.pref(*by, 1, 5)))),
            // bookkeeping the specs give no text for ([UNVERIFIED]: the original prints nothing): transporter lifecycle,
            // seizures and sales told elsewhere or not at all
            ReliefNoSiege { .. } => return None,
            Returned { .. } | TransportersBuilt { .. } | CargoDamaged { .. } | MissionDone { .. } | BranchSold { .. } => return None,
            Dismissed { .. } => return None, // shown only for humans and principality offices (above)
            TurnEndCash { .. } => return None, // the turn-end report's banner cash
            other => format!("{other:?}"),
        })
    }

    /// Battle result window: "^ Nach einer langen Schlacht... _" and the outcome text.
    /// The live battle view of a first tactic question, with the intro captions of a castle/camp attack.
    fn battle_view(&self, assets: &Assets, q: &Ask, at: usize) -> battle::BattleView {
        let mut v = battle::BattleView::from_ask(assets, q, at, self.cfg_end & 0x10 != 0);
        if let Ask::BattleIntro { code, intro, .. } = q {
            v.set_captions(self.battle_captions(assets, *code, *intro));
        }
        v
    }

    /// rules-52 §0: the pre-fight lines as click pages of the window `PREFIGHT_WIN`: `$P` clears the page, each `$W`
    /// shows it until a click. A raid's caption (0xBBA04/0xBBA48) is gold text at (40, 180, 560, 160) before the
    /// window. Returns the pages and the text left on the page for the question.
    fn prefight_pages(&self, assets: &Assets, lines: &[mad_sim::Line]) -> (Vec<battle::Page>, Vec<u8>) {
        let (mut caps, mut page) = (vec![], vec![]);
        for (addr, args) in lines {
            let src = self.exe_fmt(assets, &assets.text(*addr), args);
            if matches!(addr, 0xBBA04 | 0xBBA48) {
                caps.push(([40, 180, 560, 160], src, false));
                continue;
            }
            let mut rest = match src.strip_prefix(b"$P") {
                Some(r) => {
                    page.clear();
                    r
                }
                None => &src[..],
            };
            while let Some(k) = rest.windows(2).position(|w| w == b"$W") {
                page.extend_from_slice(&rest[..k]);
                caps.push((PREFIGHT_WIN, page.clone(), true));
                rest = &rest[k + 2..];
            }
            page.extend_from_slice(rest);
        }
        (caps, page)
    }

    /// Intro captions of a battle (rules-50 §1.1, four ids per code from 1881): lines 1..3 at (40, 180, 560, 160),
    /// then line 4 (`%s` = troop word `0xB601C[k]`) or the call-off text (§2) at (40, 160, 560, 160).
    /// A raid's line 1 names the shipment's two places.
    fn battle_captions(&self, assets: &Assets, code: u8, i: mad_sim::fight::Intro) -> Vec<([i32; 4], Vec<u8>)> {
        use mad_sim::WArg::{Loc, P};
        let id = 1881 + 4 * code as u32;
        let mut out: Vec<_> = [vec![Loc(i.loc), Loc(i.to)], vec![P(i.defender)], vec![]].iter().zip(id..).map(|(args, n)| ([40, 180, 560, 160], self.exe_fmt(assets, &assets.text_id(n), args))).collect();
        let last = if i.called_off {
            self.exe_fmt(assets, &assets.text(if code == 4 { 0xBB8AC } else { 0xBB950 }), &[P(i.attacker)])
        } else {
            let troop = t(&assets.name_table((0xB601C, 2))[i.troop as usize & 1]);
            self.exe_fmt(assets, &subst(&assets.text_id(id + 3), b"%s", &troop), &[P(i.attacker), Loc(i.loc)])
        };
        out.push(([40, 160, 560, 160], last));
        out
    }

    /// Page 1 of the after-battle report: the title and the verdict (rules-59 §1).
    fn battle_report_head(&self, assets: &Assets, att: u8, def: u8, verdict: u8) -> Vec<u8> {
        use mad_sim::WArg::P;
        let mut src = assets.text(0xBBB20);
        src.extend(match verdict {
            0 => self.exe_fmt(assets, &assets.text(0xBBB4C), &[P(att), P(def)]),
            1 => self.exe_fmt(assets, &assets.text(0xBBBC4), &[P(att), P(def)]),
            2 => assets.text(0xBBD0C),
            _ => assets.text(0xBBC64),
        });
        src
    }

    /// The after-battle report (rules-59) as click pages: one `print` stream of EXE texts, paged at `$P`. Page 1
    /// is left out when it came with the human attacker's occupy question (§2).
    fn battle_report(&self, assets: &Assets, code: u8, att: u8, def: u8, r: &mad_sim::fight::Report) -> Vec<battle::ReportPage> {
        use battle::Inline;
        use mad_sim::WArg::{Good, Loc, Num, Word, P};
        // `%I` takes no text argument: it becomes a width-reserving `%m` item before the arguments are filled in
        let fmt = |addr: u32, args: &[mad_sim::WArg]| self.exe_fmt(assets, &subst(&assets.text(addr), b"%I", b"%m"), args);
        // `%I` sprites in stream order (rules-59 §3, §5, §8): (sprite, raise)
        let mut sprites: Vec<(Inline, i32)> = vec![];
        let asked = r.conquest && code >= 4 && self.st.persons[att as usize].human();
        let mut s = if asked { vec![] } else { self.battle_report_head(assets, att, def, r.verdict) };
        // §2 conquest
        if r.conquest {
            match (code, r.occupied) {
                (3, _) => s.extend(fmt(0xBBD50, &[P(def), P(att)])),
                (4, Some([pigs, mus, adj, wine])) => {
                    s.extend(b"$W");
                    s.extend(fmt(0xBBE58, &[Loc(r.loc), P(att), Num(pigs as i32), Num(mus as i32), Word(0xB646C, adj), Num(wine as i32)]));
                }
                (5, Some(_)) => {
                    s.extend(b"$W");
                    s.extend(fmt(0xBBF90, &[Loc(r.loc), P(att)]));
                }
                _ => {}
            }
        }
        // §3 Gewinne
        s.extend(assets.text(0xBC07C));
        for row in &r.rows {
            s.extend(fmt(0xBC09C, &[P(row.person), Num(row.prisoners as i32), Num(row.markers as i32)]));
            sprites.extend([(Inline::Bank(691, row.colour), 18), (Inline::Bank(1444, row.colour), 18), (Inline::Icon(0x2C), 6)]);
        }
        for row in r.rows.iter().filter(|row| row.gain > 0) {
            s.extend(fmt(0xBC0C0, &[P(row.person), Good(r.good), Num(row.gain as i32)]));
        }
        // §4 Beschädigungen
        s.extend(assets.text(0xBC0F8));
        let (units, carts) = r.destroyed;
        if units > 0 || carts > 0 {
            s.extend(assets.text(0xBC100));
            if (units > 0) != (carts > 0) {
                s.extend(assets.text(0xBC11C));
            }
            if units > 0 {
                s.extend(fmt(0xBC124, &[Num(units as i32), Good(r.good)]));
            }
            if carts > 0 {
                s.extend(fmt(0xBC15C, &[Num(carts as i32)]));
            }
            s.extend(assets.text(0xBC0F8));
        }
        // §5 Verluste: the carter/sailor line, then the rows
        if r.class5 == 0 {
            s.extend(assets.text(0xBB86C));
        }
        s.extend(assets.text(0xBC1AC));
        s.extend(match (r.class5, code < 3) {
            (0, _) => assets.text(0xBC11C),
            (1, true) => assets.text(0xBC1C4),
            (1, false) => assets.text(0xBC204),
            (k, true) => fmt(0xBC23C, &[Num(k as i32)]),
            (k, false) => fmt(0xBC278, &[Num(k as i32)]),
        });
        for row in &r.rows {
            let l = row.losses;
            s.extend(fmt(0xBC2B8, &[P(row.person), Num(l[0] as i32), Num(l[1] as i32), Num(l[2] as i32), Num(l[3] as i32)]));
            sprites.extend((0..4).map(|c| (Inline::Unit(row.base + c, row.colour), 18)));
        }
        // §6 Bezahlung, castle damage, insurance
        s.extend(assets.text(0xBC2EC));
        s.extend(if r.premium > 0 { fmt(0xBC30C, &[P(att), Num(r.premium as i32)]) } else { fmt(0xBC34C, &[P(att)]) });
        if let Some((a, sec)) = r.damage {
            s.extend(fmt(if code == 4 { 0xBC378 } else { 0xBC3AC }, &[Num(a as i32)]));
            s.extend(fmt(0xBC3E0, &[Loc(r.loc), Word(0xB6814, (sec / 11).min(9))]));
        }
        match r.insurance {
            Some(Some(amount)) => s.extend(fmt(0xBC400, &[P(def), Num(amount)])),
            Some(None) => s.extend(fmt(0xBC43C, &[P(def)])),
            None => {}
        }
        // `$W` waits for the click on the current page, `$P` starts the next one
        let s: Vec<u8> = subst(&s, b"$W", b"");
        let mut it = sprites.into_iter();
        split_pages(&s).into_iter().map(|p| {
            let n = p.windows(2).filter(|w| w == b"%m").count();
            let mine = it.by_ref().take(n).collect();
            (p, mine)
        }).collect()
    }

    /// Office name for (location, index within level), via the global index (rules-00 §offices).
    fn office(&self, loc: u8, idx: u8) -> String {
        if idx == 15 {
            return String::new();
        }
        let base = match loc {
            0..=9 => 0,
            10..=12 => 10,
            _ => 23,
        };
        self.tables.offices.get(base + idx as usize).cloned().unwrap_or_default()
    }

    /// Office title for a holder: the table holds the male form.
    fn office_of(&self, p: usize) -> String {
        let pr = &self.st.persons[p];
        let m = self.office(pr.loc(), pr.idx());
        if pr.gender() == 0 {
            m
        } else if m == "Ratsherr" {
            "Ratsfrau".into()
        } else if m == "Kämmerer" {
            "Kämmerin".into()
        } else {
            format!("{m}in")
        }
    }

    fn me(&self) -> Option<usize> {
        (self.st.current < NOBODY).then_some(self.st.current as usize)
    }

    /// §9.1: `$t$|$H$3<$R%s, A.D. %l$|$C$(%1N1, %1N4$)$g$d` with values filled in.
    /// Fills an EXE format's `%` codes in order from `args` (`%kNn` person, `%o` place, `%s` text/place/good,
    /// `%d` dotted number, `%w`/`%l` plain number, `%v` swallowed).
    fn exe_fmt(&self, assets: &Assets, src: &[u8], args: &[mad_sim::WArg]) -> Vec<u8> {
        use mad_sim::WArg;
        let (mut out, mut it, mut i) = (vec![], args.iter(), 0);
        while i < src.len() {
            if src[i] != b'%' || src.get(i + 1) == Some(&b'%') {
                let n = if src[i] == b'%' { 2 } else { 1 };
                out.extend_from_slice(&src[i..i + n]);
                i += n;
                continue;
            }
            let rest = &src[i + 1..];
            let (code, len) = match rest {
                [k @ b'0'..=b'9', b'N', n @ b'0'..=b'9', ..] => ((*k - b'0', *n - b'0'), 4),
                [c, ..] => ((*c, 0xFF), 2),
                [] => ((0, 0), 1),
            };
            if matches!(code.0, b'a' | b'm' | b'M') && code.1 == 0xFF {
                // items stay for the form (`%a` bullet, `%m` item); their arguments are the form's own
                out.extend_from_slice(&src[i..i + len]);
                i += len;
                continue;
            }
            let a = if code.0 == b'v' { None } else { it.next() };
            out.extend(match (code, a) {
                ((k, n), Some(WArg::P(p))) if n != 0xFF => t(&self.pref(*p, k, n)),
                // `%q`: sein / ihr of the person (rules-12 §2)
                ((b'q', _), Some(WArg::P(p))) => t(if self.st.persons.get(*p as usize).is_some_and(|q| q.gender() & 1 == 1) { "ihr" } else { "sein" }),
                (_, Some(WArg::Loc(l))) => t(self.loc(*l)),
                (_, Some(WArg::Id(id))) => assets.text_id(*id),
                (_, Some(WArg::Good(g))) => t(self.good(*g)),
                (_, Some(WArg::Word(addr, k))) => t(assets.name_table((*addr, *k as usize + 1)).get(*k as usize).map_or("", String::as_str)),
                ((b'd', _), Some(WArg::Num(v))) => t(&dotted(v)),
                (_, Some(WArg::Num(v))) => t(&v.to_string()),
                _ => vec![],
            });
            i += len;
        }
        out
    }

    /// The year the banners show: while the turn-end and year-end windows of a year change are open
    /// they still belong to the old year (live A/B: the bill reads "Ereignisse, A.D. 1600").
    fn shown_year(&self) -> u16 {
        match self.report_year {
            Some(y) if self.popups.front().is_some_and(|p| matches!(p, Popup::Bill(..) | Popup::Report(_) | Popup::YearReport(_) | Popup::Chronicle(_) | Popup::Award(_))) => y,
            _ => self.st.year,
        }
    }

    fn banner_text(&self, title: &[u8]) -> Vec<u8> {
        let Some(h) = self.me() else { return vec![] };
        let mut src = [b"$t$|$H$3<$R".as_slice(), title, b", A.D. ", &t(&self.shown_year().to_string()), b"$|$C$("].concat();
        src.extend(t(&format!("{}, {}", self.person_name(h as u8), self.rank(h))));
        src.extend(b"$)$g$d");
        src
    }

    /// Gold display text (§9.1), highlighted when cash is negative.
    fn gold_text(&self) -> Vec<u8> {
        let Some(h) = self.me() else { return vec![] };
        // the bill page shows the cash right after paying; later pages the cash after the turn end
        let cash = match self.popups.front() {
            Some(Popup::Bill(_, c)) => *c,
            // a turn-end question is open: the cash after the bill, not the cash before the turn end (playtest report)
            _ if self.pending.is_some() && self.pending_cash.is_some() => self.pending_cash.unwrap(),
            Some(p @ Popup::Report(_)) if self.turn_cash.is_some_and(|(k, _)| k == Self::report_key(p)) => self.turn_cash.unwrap().1,
            _ => self.st.persons[h].cash(),
        };
        // thousands dotted like every %d (live A/B: "50.000.000 Taler")
        let n = t(&dotted(cash));
        if cash < 0 || self.gold_hot { [b"$t$L$h".as_slice(), &n, b" Taler$h$N"].concat() } else { [b"$t$L".as_slice(), &n, b" Taler$N"].concat() }
    }

    fn draw_banner(&self, frame: &mut Frame, assets: &Assets, title: &[u8], icon: bool) {
        if icon {
            frame.blit(&assets.icon(BANNER)[0], 1, 0);
        }
        text::draw(frame, &assets.font, &text::layout(&assets.font, &self.banner_text(title), &[], &[], 640, &[]), 0, 0);
        // Item text starts at the item's own origin; the window inset does not apply ([UNVERIFIED]).
        let gold = text::layout(&assets.font, &self.gold_text(), &[], &[], GOLD_RECT[2] + 24, &[]);
        text::draw(frame, &assets.font, &gold, GOLD_RECT[0] - 12, GOLD_RECT[1] - 12);
    }

    /// A yes/no dialog from an EXE string at a documented rectangle.
    /// Ours (pref `sell_all`): the Ja/Nein question before `Action::SellAll`.
    fn sell_all_ask(assets: &Assets) -> Action {
        let p = assets.prefs.borrow();
        let s = [&b"$C"[..], &t(p.tr("Die Vorräte aller Niederlassungen vor Ort verkaufen?", "Sell the stock of every branch locally?")), b"$N$N%m  %m"].concat();
        Action::YesNo(s, [140, 160, 360, 160], Box::new(Action::SellAll))
    }

    /// Ours (pref `sell_all`): one `SellStock` (all goods) per branch, in slot order, exactly what the empty-warehouse
    /// icon does by hand; then a summary per city.
    fn sell_all(&mut self, assets: &Assets) {
        let Some(h) = self.me() else { return };
        let mut rows = vec![];
        for b in self.st.props_of(h, mad_sim::state::KIND_BRANCH).collect::<Vec<_>>() {
            let (cash0, units) = (self.st.persons[h].cash(), (0..mad_sim::state::GOODS).map(|g| self.st.props[b].stock(g) as u32).sum::<u32>());
            self.sim(assets, Command::SellStock { branch: b as u8, good: 0xFF, qty: 0 });
            rows.push((self.st.props[b].loc(), units, self.st.persons[h].cash() - cash0));
        }
        let p = assets.prefs.borrow();
        let mut f = Form::new(b"$C^ ");
        f.txt(p.tr("Alles verkauft", "Sold everything")).raw(b" _$Z$N");
        // one row per city: "City: n Waren" left, the revenue right (the Optionen rows' `$E$R` layout)
        let total: i32 = rows.iter().map(|r| r.2).sum();
        let last = (p.tr("Zusammen", "Total").to_string(), String::new(), total);
        for (name, units, rev) in rows.iter().map(|&(c, u, r)| (self.loc(c).to_string(), format!(" {u} {}", p.tr("Waren", "goods")), r)).chain([last]) {
            if units.is_empty() {
                f.raw(b"$N");
            }
            f.raw(b"$|$L").txt(&format!("{name}:{units}")).raw(b"$E$R").txt(&format!("{} Taler", dotted(rev))).raw(b"$N");
        }
        let popup = f.popup(assets, 100, 440, None);
        self.popups.push_front(popup);
    }

    fn yes_no(assets: &Assets, src: Vec<u8>, rect: [i32; 4], yes: Action) -> Popup {
        // These strings end in `%m  %m`: Ja, Nein.
        let mut f = Form::new(&src);
        f.bind(assets.text(addr::JA), yes).bind(assets.text(addr::NEIN), Action::Close);
        f.popup_at(rect)
    }

    /// Per-frame delay of the bank animations: `delay_ticks(n)` waits whole 54.9 ms BIOS ticks of the DOS clock
    /// (rules-25 §3, rules-01 §5): duel/auction n 5 → 1 tick, verdict n 10 → 2, witness n 12 and plea n 15 → 3.
    fn frame_ms(pic: u16, anim: Option<(usize, usize)>) -> u32 {
        const TICK: u32 = 55;
        match (pic, anim) {
            (0xC6, Some((100, 133))) => 2 * TICK,
            (0xC6, _) => 3 * TICK,
            _ => TICK,
        }
    }

    fn end_turn_dialog(&self, assets: &Assets) -> Option<Popup> {
        if self.cfg_end & 0x20 != 0 {
            Some(Self::yes_no(assets, assets.text(addr::END_TURN), [160, 160, 320, 160], Action::EndTurn))
        } else if self.cfg_end & 0x10 != 0 && self.unvisited {
            Some(Self::yes_no(assets, assets.text(addr::NOT_VISITED), [140, 160, 360, 160], Action::EndTurn))
        } else {
            None
        }
    }

    /// Runs an action that was not triggered from the front popup (room hotspots).
    fn act_keep(&mut self, a: Action, ctx: &mut Ctx) -> Next {
        self.popups.push_front(Popup::Report(vec![]));
        self.act(a, ctx)
    }

    /// Ours (pref `confirmations`): the Ja/Nein question before an action the original runs at once. Selling a
    /// branch or residence and hiring a saboteur already ask in the original; a human castle buyer is asked by the sim.
    fn safety(&self, assets: &Assets, a: &Action) -> Option<Vec<u8>> {
        let p = assets.prefs.borrow();
        if !p.confirmations {
            return None;
        }
        let (de, en) = match a {
            Action::Sim(Command::Bribe { .. }, _) => ("Wirklich bestechen?", "Really bribe?"),
            Action::Sim(Command::Denounce { .. }, _) => ("Wirklich anschwärzen?", "Really denounce?"),
            Action::Sim(Command::SellCastle { .. }, _) => ("Wirklich verkaufen?", "Really sell?"),
            Action::Blackmail(_) => ("Wirklich erpressen?", "Really blackmail?"),
            Action::Dismiss(..) => ("Söldner wirklich entlassen?", "Really dismiss the troops?"),
            Action::Spied(target, _) if self.me().is_some_and(|me| self.st.spy_contract(me, *target as usize).is_none()) => ("Wirklich Spione anheuern?", "Really hire spies?"),
            _ => return None,
        };
        Some([&b"$C"[..], &t(p.tr(de, en)), b"$N$N%m  %m"].concat())
    }

    fn act(&mut self, a: Action, ctx: &mut Ctx) -> Next {
        // ours (pref `confirmations`): the question sits over the window the action came from; Ja runs the action
        // as if clicked there, Nein closes the question only
        if let Action::Sure(a) = a {
            self.popups.pop_front();
            return self.act_now(*a, ctx);
        }
        if let Some(q) = self.safety(ctx.assets, &a) {
            self.popups.push_front(Self::yes_no(ctx.assets, q, [140, 160, 360, 160], Action::Sure(Box::new(a))));
            return Next::Stay;
        }
        self.act_now(a, ctx)
    }

    fn act_now(&mut self, a: Action, ctx: &mut Ctx) -> Next {
        if let Action::Select(k) = a {
            // the list stays open; the bullets show the selection
            self.sel = if self.sel == Some(k) { None } else { Some(k) };
            self.sel_menu = match self.popups.front() { Some(Popup::Choice { menu, .. }) => *menu, _ => None };
            return Next::Stay;
        }
        if matches!(a, Action::WithSel(_, false)) && self.sel.is_none() {
            // the confirm icon without a selection: nothing happens, the list stays (live A/B)
            return Next::Stay;
        }
        // an office-list entry's action keeps the list open underneath (rules-31 §6)
        // (also Spielstand: its row boxes appear over the list)
        // (and the privileges menu: its info boxes open over it, live A/B 2026-11-15)
        let tree = matches!(self.popups.front(), Some(Popup::Choice { menu: Some(Menu::Tree(..) | Menu::Spielstand | Menu::Privileges | Menu::City(_)), .. }))
            // (and Bewerbung: the applicants window opens over its list, live A/B 2026-11-22)
            || matches!((self.popups.front(), &a), (Some(Popup::Choice { menu: Some(Menu::Apply), .. }), Action::Open(Menu::Applicants(..))));
        // a running battle view stays while its tactic answers go out; the fight's end finishes it
        let live = matches!(self.popups.front(), Some(Popup::Battle(v)) if v.live) && matches!(a, Action::Answer(_));
        if live {
        } else if !tree || matches!(a, Action::Close | Action::Nothing) {
            self.popups.pop_front();
        } else if let Action::Open(m) = a {
            // a window opened from the list goes on top of it
            let n = self.popups.len();
            self.open(ctx.assets, m);
            if self.popups.len() > n
                && let Some(p) = self.popups.pop_back()
            {
                self.popups.push_front(p);
            }
            return Next::Stay;
        }
        match a {
            Action::Close | Action::Nothing => {}
            Action::EndTurn => {
                (self.branch, self.room) = (None, None);
                self.undo = ctx.assets.prefs.borrow().undo.then(|| Box::new(self.st.clone()));
                self.sim(ctx.assets, Command::EndTurn);
            }
            Action::Undo => {
                if let Some(g) = self.undo_game(ctx.assets) {
                    return Next::Go(Box::new(g));
                }
            }
            // Spiel beenden (0x4D880, rules-38 §6): the game goes to slot 5 [UNVERIFIED: whether the options screen asks
            // for the save], then the credits, then the program ends
            Action::Credits => return self.credits(ctx),
            Action::Quit => {
                if let Err(e) = ctx.assets.write_save(5, &self.st.to_game_dat()) {
                    eprintln!("autosave: {e}");
                }
                self.store_config(ctx.assets);
                return self.credits(ctx);
            }
            Action::Branch(b) => {
                self.branch = Some(b);
                self.unvisited = false;
            }
            Action::GiveUp => {
                (self.branch, self.room) = (None, None);
                self.sim(ctx.assets, Command::GiveUp);
                // `GiveUp` leaves the turn at phase 6; run on to the next player.
                self.run_cmd(ctx.assets, None, vec![]);
            }
            Action::Room(r) => {
                // leaving the Kontor for one of these rooms re-rolls its music on the way back (rules-39 §4)
                if matches!(r, Some(Room::Schreibstube | Room::Bezahlung | Room::Hinterzimmer | Room::Handel | Room::Kirche)) {
                    self.kontor_left = true;
                }
                (self.branch, self.room) = (None, r);
                // entry music (rules-14 §4.2): trade map and Löhne (0x57E70) group 0x19, castle screen group 0x1A, back room 0x10
                let music = match r {
                    Some(Room::Handel | Room::Bezahlung) => Some((0x19, 2, false)),
                    Some(Room::Burg(_)) => Some((0x1A, 2, false)),
                    Some(Room::Hinterzimmer) => Some((0x10, 2, true)),
                    _ => None,
                };
                if let Some((song, mode, coin)) = music {
                    self.sim(ctx.assets, Command::Music { song, mode, coin });
                }
                if let Some(Room::Burg(p)) = r
                    && self.book_shown.get(self.st.props[p].loc().wrapping_sub(14) as usize) == Some(&true)
                {
                    self.open(ctx.assets, Menu::Orders(p));
                }
            }
            // the EXE's message box is 320 wide at x 160 (live A/B: Bewerbung, Direktoren)
            Action::Text(a) => self.popups.push_front(Form::new(&ctx.assets.text(a)).popup(ctx.assets, 160, 320, None)),
            Action::Bytes(src) => self.popups.push_front(Form::new(&src).popup(ctx.assets, 140, 360, None)),
            Action::Boxed(src, rect) => self.popups.push_front(Form::new(&src).popup_at(rect)),
            Action::SellAll => self.sell_all(ctx.assets),
            Action::Inline(src, acts, rect) => {
                let mut f = Form::new(b"");
                f.inline(&src, acts);
                self.popups.push_front(f.popup_at(rect));
            }
            Action::NewPlayer => {
                // rules-11 §1: before 1671, fewer than six humans, a free start city; one EXE text each
                let free = self.st.clone().pick_start_city().is_some();
                if self.st.year >= 1671 || self.st.humans().count() >= 6 || !free {
                    let refusal = if self.st.humans().count() >= 6 { 0xBDCD0 } else if self.st.year >= 1671 { 0xBDD54 } else { 0xBDDC4 };
                    let mut f = Form::new(&ctx.assets.text(refusal));
                    f.bind(ctx.assets.text(addr::JA), Action::Close);
                    self.popups.push_front(f.popup_at([140, 160, 360, 160]));
                } else {
                    return Next::Go(Box::new(super::setup::PlayerSetup::join(self.st.clone())));
                }
            }
            Action::ConfirmText(s, rect, yes) => {
                // the EXE's `%j Ja$Z%j Nein` lines are menu items, like `%a`
                let mut f = Form::new(b"");
                let mut src = s.clone();
                while let Some(k) = src.windows(2).position(|w| w == b"%j") {
                    src[k + 1] = b'a';
                }
                f.inline(&src, vec![*yes, Action::Close]);
                self.popups.push_front(f.popup_at(rect));
            }
            Action::Confirm(a, yes) => self.popups.push_front(Self::yes_no(ctx.assets, ctx.assets.text(a), [160, 160, 320, 160], *yes)),
            Action::Sure(a) => return self.act_now(*a, ctx),
            Action::YesNo(s, rect, yes) => self.popups.push_front(Self::yes_no(ctx.assets, s, rect, *yes)),
            Action::Sell(b, good, qty) => {
                let qty = if qty == 0 && good != 0xFF { self.st.props[b].stock(good as usize) } else { qty };
                self.sim(ctx.assets, Command::SellStock { branch: b as u8, good, qty });
            }
            // a privilege dialog starts fresh: first allowed city, number 0 (the army dialog raises it to its minimum)
            Action::Open(m @ (Menu::PrivCity(_) | Menu::Project)) => {
                if let Menu::PrivCity(code) = m {
                    self.priv_city = self.priv_cities(code).first().copied().unwrap_or(0);
                }
                self.priv_n = 0;
                self.open(ctx.assets, m);
            }
            // castle info (0x549C0) rolls the castle music group on opening (rules-14 §4.2, place kinds < 3)
            Action::Open(m @ Menu::CastleInfo(_)) => {
                self.sim(ctx.assets, Command::Music { song: 0x1A, mode: 2, coin: false });
                self.open(ctx.assets, m);
            }
            Action::Open(m) => self.open(ctx.assets, m),
            Action::Recruit(p, amount) => {
                self.sim(ctx.assets, Command::Recruit { prop: p as u8, money: amount });
                self.amount = 0;
                self.open(ctx.assets, Menu::Recruit(p));
            }
            Action::Dismiss(p, slot, n) => self.sim(ctx.assets, Command::Dismiss { prop: p as u8, slot, amount: n }),
            Action::Fortify(p, which, amount) => {
                self.sim(ctx.assets, Command::Fortify { prop: p as u8, which, amount });
                self.amount = 0;
                self.open(ctx.assets, Menu::Improve(p, which));
            }
            Action::Manoeuvre(p) => {
                let before: Vec<u8> = (0..4).map(|k| self.st.props[p].training(k)).collect();
                self.sim(ctx.assets, Command::Manoeuvre { prop: p as u8 });
                let camp = self.st.props[p].kind() != mad_sim::state::KIND_CASTLE;
                let names = ctx.assets.name_table((if camp { 0xB6BF0 } else { 0xB6BE0 }, 4));
                let k = (0..4).find(|&k| self.st.props[p].training(k) != before[k]).unwrap_or(0);
                let msg = if camp { 0xBFB34 } else { 0xBFB04 };
                self.popups.push_front(Form::new(&subst(&ctx.assets.text(msg), b"%s", &t(&names[k]))).popup(ctx.assets, 140, 360, None));
                self.open(ctx.assets, Menu::Manoeuvre(p));
            }
            Action::Sim(cmd, then) => {
                // church bells after a donation or candle (0x49), the organ for an indulgence (0x4A)
                match cmd {
                    Command::Candles | Command::Alms => self.sounds.push(0x49),
                    // formats §10.6: leaving the church closes the church screen
                    Command::LeaveChurch => self.room = None,
                    Command::Indulgence => {
                        self.sounds.push(0x4A);
                        // rules-04b Ablass: the human then gets 0xACF03 in a box (160, 160, 320, 160)
                        self.popups.push_back(Form::new(&ctx.assets.text(0xACF03)).popup_at([160, 160, 320, 160]));
                    }
                    // rules-12 §3.8: the human's success box 2407 (the second-mistress box 2404 is the menu's)
                    Command::TakeMistress => self.popups.push_back(Form::new(&ctx.assets.text_id(2407)).popup_at([160, 140, 320, 200])),
                    _ => {}
                }
                self.sim(ctx.assets, cmd);
                if let Some(m) = then {
                    self.open(ctx.assets, m);
                }
            }
            Action::Spied(target, price) => {
                let Some(me) = self.me() else { return Next::Stay };
                let name = self.person_name(target);
                let first = self.first_name(target as usize);
                let genitive = if first.ends_with(['s', 'z', 'x']) { format!("{first}'") } else { format!("{first}s") };
                let Some(i) = self.st.spy_contract(me, target as usize) else {
                    self.sim(ctx.assets, Command::Spies { target, price });
                    // the yearly cost is a third of the tree price (rules-04 §3.2; live A/B)
                    let src = subst(&subst(&ctx.assets.text(0xBC904), b"%3N1", &t(&self.pref(target, 3, 1))), b"%d", dotted(price / 3).as_bytes());
                    self.popups.push_front(Form::new(&src).popup(ctx.assets, 120, 400, None));
                    return Next::Stay;
                };
                // A contract exists: the EXE's menus for a running (0xBC7E8) or paused (0xBC840) one.
                let toggle = Action::Sim(Command::Spies { target, price: 0 }, None);
                let (src, acts) = if self.st.heap.recs[i][9] != 0 {
                    (ctx.assets.text(0xBC7E8), vec![Action::Evidence(target), toggle, Action::Close])
                } else {
                    (ctx.assets.text(0xBC840), vec![toggle, Action::Evidence(target), Action::Sim(Command::CancelSpies { target }, None), Action::Close])
                };
                let src = subst(&subst(&src, b"%4N3", &t(&name)), b"%2N1", &t(&genitive));
                let mut f = Form::new(b"");
                f.inline(&src, acts);
                self.popups.push_front(f.popup(ctx.assets, 120, 400, None));
            }
            Action::PrivView(code) => {
                let holder = self.priv_holder_of();
                self.sim(ctx.assets, Command::PrivilegeView { holder: holder as u8, code });
                self.open(ctx.assets, Menu::PrivAsk(code));
            }
            Action::Priv(code, a, b) => {
                let holder = self.priv_holder_of();
                self.popups.retain(|p| !matches!(p, Popup::Choice { menu: Some(Menu::PrivCity(_) | Menu::PrivArmy(_) | Menu::PrivAsk(_) | Menu::Project | Menu::Tree(..)), .. } | Popup::Map(_)));
                let loc = self.st.persons[holder].loc() as usize;
                self.sim(ctx.assets, Command::Privilege { holder: holder as u8, code, a, b });
                let tid = |n: u32| ctx.assets.text_id(n);
                let name_id = u32::from_le_bytes(ctx.assets.exe_bytes(0x86778 + 4 * code as u32, 4).try_into().unwrap());
                // the dialogs' closing messages (rules-38 §2.6, 2.10, 2.11)
                let after = match code {
                    28 | 29 => Some((tid(2147), [120, 120, 400, 240])),
                    53 => Some((subst(&tid(3056), b"%s", &t(self.loc(loc as u8))), [160, 180, 320, 120])),
                    54 | 84 if a < 2 => Some((subst_seq(&tid(name_id + 5), b"%s", &[t(self.loc(loc as u8)), tid(name_id + 2 + a as u32)]), [160, 140, 320, 200])),
                    _ => None,
                };
                if let Some((src, rect)) = after {
                    self.popups.push_front(Form::new(&src).popup_at(rect));
                }
                if code == 25 {
                    self.reopen(ctx.assets, Menu::Insurance);
                }
            }
            Action::PrivPage => {
                self.priv_page ^= 1;
                self.reopen(ctx.assets, Menu::Privileges);
            }
            Action::PrivHolder(x) => {
                (self.priv_holder, self.priv_page) = (x, 0);
                self.reopen(ctx.assets, Menu::Privileges);
            }
            Action::Medicus => {
                let Some(me) = self.me() else { return Next::Stay };
                let level = 11 - (self.st.persons[me].life() as usize / 2).min(11);
                // the twelve verdicts follow each other in the EXE, one NUL apart
                let mut a = 0xA1E50;
                let mut line = vec![];
                for _ in 0..=level {
                    line = ctx.assets.text(a);
                    a += line.len() as u32 + 1;
                    while ctx.assets.exe_bytes(a, 1) == [0] {
                        a += 1;
                    }
                }
                let src = subst(&ctx.assets.text(0xA1E09), b"%s", &line);
                self.popups.push_front(Form::new(&src).popup(ctx.assets, 120, 400, None));
            }
            Action::Blackmail(target) => self.sim(ctx.assets, Command::Blackmail { target }),
            Action::Evidence(target) => {
                let Some(me) = self.me() else { return Next::Stay };
                let n = self.st.spy_contract(me, target as usize).map_or(0, |i| mad_sim::heap::r32(&self.st.heap.recs[i], 0x0B).count_ones());
                let src = subst(&subst(&ctx.assets.text(0xBC8BC), b"%3N5", &t(&self.person_name(target))), b"%d", &t(&dotted(n)));
                // What can be done with evidence is ours: blackmail or denounce (rules-04 §3.4, §3.5).
                let mut f = Form::new(&src);
                f.raw(b"$Z$C").item(t("Erpressen"), Action::Blackmail(target)).raw(b"   ").item(t("Anzeigen"), Action::Open(Menu::Listener(target))).raw(b"   ").item(t("Zurück"), Action::Close);
                self.popups.push_front(f.popup(ctx.assets, 120, 400, None));
            }
            Action::Sabotage(target, price) => {
                let name = self.first_name(target as usize);
                let genitive = if name.ends_with(['s', 'z', 'x']) { format!("{name}'") } else { format!("{name}s") };
                let (src, acts) = if self.st.saboteur_on(self.me().unwrap_or(0), target as usize).is_some() {
                    (ctx.assets.text(0xBC994), vec![Action::Sim(Command::CancelSaboteur { target }, None), Action::Close])
                } else {
                    (ctx.assets.text(0xBCA0C), vec![Action::Sim(Command::Saboteur { target, price }, None), Action::Close])
                };
                let _ = genitive;
                let src = subst(&subst(&src, b"%2N1", &t(&self.pref(target, 2, 1))), b"%d", dotted(price).as_bytes());
                let mut f = Form::new(b"");
                f.inline(&src, acts);
                self.popups.push_front(f.popup(ctx.assets, 120, 400, None));
            }
            Action::Opt(o, m) => {
                match o {
                    Opt::Flag(bit) => {
                        self.opts ^= bit;
                        (self.cfg_end, self.cfg_announce) = (self.opts as u8, self.opts & 0x100 != 0);
                    }
                    // steps of 8, clamped at 0 and the top (rules-44 §1.2)
                    Opt::Vol(i, d) => {
                        let (v, top) = (self.vol[i], if i == 2 { 255 } else { 127 });
                        self.vol[i] = if d < 0 { v.saturating_sub(8) } else if v >= top - 8 { top } else { v + 8 };
                    }
                    // Sprachausgabe is bit 0x80 of the option dword (rules-44 §1.2)
                    Opt::Voice => self.opts ^= 0x80,
                    Opt::Film => self.black_box = !self.black_box,
                    Opt::Scroll => self.text_speed = (self.text_speed + 1) % 6,
                    Opt::Pref(k) => {
                        let mut p = ctx.assets.prefs.borrow_mut();
                        match k {
                            Pref::Fullscreen => p.fullscreen ^= true,
                            Pref::IntegerScale => p.integer_scale ^= true,
                            Pref::Speed => p.speed = p.speed % 4 + 1,
                            Pref::Autosave => p.autosave_years = match p.autosave_years { 0 => 1, 1 => 5, 5 => 10, _ => 0 },
                            Pref::Undo => p.undo ^= true,
                            Pref::Tooltips => p.tooltips ^= true,
                            Pref::Shortcuts => p.shortcuts ^= true,
                            Pref::Warnings => p.warnings ^= true,
                            Pref::Lang => p.lang = if p.lang == crate::prefs::Lang::De { crate::prefs::Lang::En } else { crate::prefs::Lang::De },
                            Pref::ProductionHelp => p.production_help ^= true,
                            Pref::DebtMercy => p.debt_mercy ^= true,
                            Pref::MoreWorkers => p.more_workers ^= true,
                            Pref::SeedChoice => p.seed_choice ^= true,
                            Pref::SellAll => p.sell_all ^= true,
                            Pref::Ledger => p.ledger ^= true,
                            Pref::PriceTable => p.price_table ^= true,
                            Pref::EventLog => p.event_log ^= true,
                            Pref::OrderCopy => p.order_copy ^= true,
                            Pref::Confirmations => p.confirmations ^= true,
                        }
                        self.st.house = p.house();
                        if let Err(e) = p.save(&ctx.assets.user_saves) {
                            eprintln!("prefs: {e}");
                        }
                    }
                }
                // the Musik screen saves the config when it is left; the others wait for the next save point (rules-44 §1.1)
                if m == Menu::Music {
                    self.store_config(ctx.assets);
                }
                self.open(ctx.assets, m);
            }
            Action::Step(..) | Action::ToggleOrder(..) => unreachable!("handled by click_form"),
            Action::SaveName(k) => {
                (self.save_edit, self.child_name) = (Some(k), vec![]);
                self.open(ctx.assets, Menu::Save);
            }
            Action::Save(k) => match ctx.assets.write_save(k, &self.st.to_game_dat()) {
                Ok(p) => {
                    eprintln!("saved {}", p.display());
                    self.store_config(ctx.assets);
                    // "Spielstand gespeichert..." (EXE 0xBDC78)
                    self.popups.push_front(Form::new(&ctx.assets.text(0xBDC78)).popup(ctx.assets, 100, 440, None));
                }
                Err(e) => self.info(ctx.assets, &format!("Fehler beim Speichern: {e}")),
            },
            Action::Load(k) => return Next::Go(menu::load_game(ctx.assets, k, Some(self.st.rng))),
            Action::AnswerPair(a, b) => {
                if let Some(Pending { cmd, mut answers, .. }) = self.pending.take() {
                    answers.extend([a, b]);
                    self.run_cmd(ctx.assets, cmd, answers);
                }
            }
            Action::Select(_) | Action::Inert => {}
            Action::WithSel(acts, _) => {
                let Some(k) = self.sel.take() else { return Next::Stay };
                if let Some(a) = acts.get(k as usize).cloned() {
                    return self.act(a, ctx);
                }
            }
            Action::Answer(v) => {
                if let Some(Pending { cmd, mut answers, ask }) = self.pending.take() {
                    answers.push(v);
                    self.church_after(ctx.assets, &ask, v);
                    // a menu printed into the report window (gifts, sub-plots) stays there, the reaction prints right
                    // under it (rules-63 §2, rules-13)
                    if let Some((src, items)) = self.report_menu.take() {
                        // the chosen gem stays lit (live A/B 2026-11-22); inline menus answer bullet k with Answer(k)
                        self.window.push(raw_line(&answered(&src, &items), Some(v as usize)));
                        // the siege goes on (0x9F69D) or is dissolved (0x9F673), printed under the answer (rules-21 §2.2)
                        if let Ask::ContinueSiege { .. } = ask {
                            self.window.push(raw_line(&ctx.assets.text(if v == 0 { 0x9F69D } else { 0x9F673 }), None));
                        }
                    }
                    self.run_cmd(ctx.assets, cmd, answers);
                    if let Ask::Match { target, .. } = ask {
                        self.kupplerin_after(ctx.assets, target as usize, v == 0);
                    }
                }
            }
        }
        Next::Stay
    }

    /// Ours: the game as it was before the last turn end (like a load of that moment), once.
    fn undo_game(&mut self, assets: &Assets) -> Option<Game> {
        let st = *self.undo.take()?;
        let mut g = Game::start(assets, st);
        g.seed = self.seed;
        Some(g)
    }

    fn info(&mut self, assets: &Assets, s: &str) {
        let mut f = Form::new(b"$C");
        f.txt(s);
        self.popups.push_front(f.popup(assets, 100, 440, None));
    }

    /// Click on a form item: words and digits step by the clicked half and the open menu is
    /// rebuilt; everything else acts.
    fn click_form(&mut self, ctx: &mut Ctx, w: &Window, acts: &[Action], pos: (i32, i32)) -> Next {
        // A window without choices (information) closes on any click; the Bewerbung applicants window only on a
        // right click (live A/B 2026-11-22: left clicks on its text and its corner do nothing)
        if acts.is_empty() {
            if !matches!(self.popups.front(), Some(Popup::Choice { menu: Some(Menu::Applicants(..)), .. })) {
                self.popups.pop_front();
            }
            return Next::Stay;
        }
        let Some(k) = w.item_at(pos) else { return Next::Stay };
        let r = w.item_rect(k);
        let up = pos.1 < r.y + r.h / 2;
        match acts[k].clone() {
            Action::Step(f, step) => self.step(ctx.assets, f, if up { step } else { -step }),
            Action::ToggleOrder(b, k) => {
                let on = self.st.props[b].order(k as usize)[7] != mad_sim::economy::ST_CONFIRMED;
                self.sim(ctx.assets, Command::ConfirmOrder { branch: b as u8, slot: k, on });
            }
            a => return self.act(a, ctx),
        }
        if let Some(Popup::Choice { menu: Some(m), .. }) = self.popups.front() {
            let m = *m;
            self.reopen(ctx.assets, m);
        }
        Next::Stay
    }

    /// Whether the item under `pos` is a number field (auto-repeat target).
    /// The auction hall runs by itself (rules-08 §1.4, rules-25 §1): a human bids by pressing his hotkey, otherwise the
    /// tick ends when its frames (50 ms each) and its speech are done; a click cuts a hammer tick's wait short
    /// (rules-25 §1.3 step 6). Ours: every tick stays at least 100 ms so a bid line can be read [UNVERIFIED].
    fn auction_input(&mut self, ctx: &mut Ctx, input: &crate::input::Input) -> Option<Next> {
        let Some(Ask::AuctionTick { strip, fresh, blow, hammer, .. }) = self.pending.as_ref().map(|q| &q.ask) else { return None };
        if !matches!(self.popups.front(), Some(Popup::Choice { menu: Some(Menu::Question), .. })) {
            return None;
        }
        let frames: u32 = strip.iter().skip(*fresh as usize).map(|&(_, a, b)| u32::from(b.saturating_sub(a))).sum();
        let (blow, hammer) = (*blow, *hammer);
        for e in &input.events {
            match *e {
                Event::Key(Key::Char(c)) => {
                    let k = self.st.humans().take(6).position(|h| self.st.persons[h].hotkey().eq_ignore_ascii_case(&c));
                    if let Some(k) = k {
                        return Some(self.act(Action::Answer(k as u32 + 1), ctx));
                    }
                }
                Event::Left(..) if blow => return Some(self.act(Action::Answer(0), ctx)),
                Event::Key(Key::Escape) => return Some(self.act(Action::Answer(mad_sim::AUCTION_ESC), ctx)),
                // any other key ends the tick's wait (rules-42 §8)
                Event::Key(_) => return Some(self.act(Action::Answer(0), ctx)),
                _ => {}
            }
        }
        // rules-42 §8: the auctioneer's frames (one BIOS tick each), 200 ms after the second blow, the blow's speech,
        // then the menu wait: 1.5 s after a tick with frames, else 2.0 s
        let anim = frames * 55 + if blow { u32::from(hammer == 2) * 200 + self.speech_ms } else { 0 };
        if self.ask_ms >= anim + if frames > 0 { 1500 } else { 2000 } {
            return Some(self.act(Action::Answer(0), ctx));
        }
        Some(Next::Stay)
    }

    fn repeatable(&self, assets: &Assets, pos: (i32, i32)) -> bool {
        // the castle order word waits for the release (rules-26 §7.3), the status glyph is not a Step
        let step = |w: &crate::screens::Window, acts: &[Action]| {
            w.item_at(pos).and_then(|k| acts.get(k)).is_some_and(|a| matches!(a, Action::Step(f, _) if !matches!(f, Field::Order(_, _, OrderField::TroopKind))))
        };
        // castle screen: the troop counts and the order book toggle repeat as well
        let room = |r| matches!(r, Some(Room::Burg(_))) && matches!(self.room_action(assets, r.unwrap(), pos), Some(Action::Dismiss(..) | Action::Open(Menu::Orders(_))));
        match self.popups.front() {
            Some(p @ Popup::Choice { acts, menu: Some(Menu::Orders(_)), rect, .. }) if self.popups.len() == 1 => {
                if hit(*rect, pos) { p.window(assets, &[]).is_some_and(|w| step(&w, acts)) } else { room(self.room) }
            }
            Some(p @ Popup::Choice { acts, .. }) => p.window(assets, &[]).is_some_and(|w| step(&w, acts)),
            Some(_) => false,
            None if matches!(self.room, Some(Room::Burg(_))) => room(self.room),
            // the wage percentages repeat too: the screen only redraws its coin pictures (rules-54 §4)
            None if self.room == Some(Room::Bezahlung) => true,
            None => self.branch.is_some_and(|b| {
                let (w, acts) = self.branch_window(assets, b, &[]);
                step(&w, &acts)
            }),
        }
    }

    /// A click or key on the room or Kontor (no window in front).
    fn handle_room_event(&mut self, ctx: &mut Ctx, e: Event) -> Next {
        match (e, self.branch) {
            (Event::Right(..) | Event::Key(Key::Escape), Some(_)) => self.branch = None,
            // leaving a room: a castle goes back to Söldner & Räuber, the Kreditbuch to the Schreibstube (live A/B)
            (Event::Right(..) | Event::Key(Key::Escape), None) if self.room.is_some() => {
                self.room = match self.room {
                    Some(Room::Burg(_)) => Some(Room::Truppen),
                    Some(Room::Kreditbuch) => Some(Room::Schreibstube),
                    _ => None,
                }
            }
            // the gold display opens the bribery map (0x4F370 = 0x45C30 mode 1) on every screen with a banner that
            // calls it: Kontor, map, branch, Schreibstube, Kirche, wages, Söldner, Hinterzimmer, Kirchgang (rules-64 §1)
            (Event::Left(x, y), _)
                if hit(GOLD_RECT, (x, y))
                    && self.me().is_some()
                    && (self.branch.is_some() || matches!(self.room, Some(Room::Handel | Room::Schreibstube | Room::Kirche | Room::Bezahlung | Room::Truppen | Room::Hinterzimmer | Room::Kirchgang))) =>
            {
                self.open(ctx.assets, Menu::Targets(Plot::Bribe));
            }
            // ours: Tab opens the Extras window (all our options and windows) wherever no window is in front
            (Event::Key(Key::Tab), _) if self.me().is_some() => self.open(ctx.assets, Menu::Extras(0)),
            // ours (pref `sell_all`): V at the Kontor or on the Handel map
            (Event::Key(Key::Char(b'v' | b'V')), None) if self.me().is_some() && matches!(self.room, None | Some(Room::Handel)) && ctx.assets.prefs.borrow().sell_all => {
                return self.act(Self::sell_all_ask(ctx.assets), ctx);
            }
            (Event::Left(x, y), None) if self.room == Some(Room::Bezahlung) => self.wage_click(ctx.assets, (x, y)),
            (Event::Left(x, y), None) if let Some(r) = self.room => {
                if let Some(a) = self.room_action(ctx.assets, r, (x, y)) {
                    self.spot_click = Some((x, y));
                    let n = self.act_keep(a, ctx);
                    if !matches!(n, Next::Stay) {
                        return n;
                    }
                }
            }
            (Event::Left(x, y), Some(b)) => {
                let n = self.click_branch(ctx, b, (x, y));
                if !matches!(n, Next::Stay) {
                    return n;
                }
            }
            // ours: `u` in the Kontor takes the last turn end back (Rückgängig option)
            (Event::Key(Key::Char(b'u' | b'U')), None) if self.room.is_none() && self.undo.is_some() => return self.act(Action::Undo, ctx),
            // Right click in the Kontor ends the round (spec 01), confirmed per config (§9.2).
            (Event::Right(..), None) if self.me().is_some() => match self.end_turn_dialog(ctx.assets) {
                Some(p) => self.popups.push_back(p),
                None => drop(self.act(Action::EndTurn, ctx)),
            },
            (Event::Left(x, y), None) => self.click_kontor(ctx.assets, (x, y)),
            // ours (`prefs.shortcuts`): Kontor keys, Space / Enter end the turn, ? the key list
            (Event::Key(k), _) if self.me().is_some() && ctx.assets.prefs.borrow().shortcuts && self.shortcut(ctx, k) => {}
            // ours: the production help of the branch screen (key H) and the Kontor keys of the ledger and the
            // price table, each behind its option
            (Event::Key(Key::Char(c)), Some(b)) if c.eq_ignore_ascii_case(&b'h') && ctx.assets.prefs.borrow().production_help => self.open(ctx.assets, Menu::ProdHelp(b)),
            // ours (pref `order_copy`): C copies the branch's four order slots, E writes them into this branch through
            // the sim, which sets each slot's status
            (Event::Key(Key::Char(c)), Some(b)) if matches!(c.to_ascii_lowercase(), b'c' | b'e') && ctx.assets.prefs.borrow().order_copy => {
                let msg = if c.eq_ignore_ascii_case(&b'c') {
                    self.clip = Some(std::array::from_fn(|k| self.st.props[b].order(k)));
                    ctx.assets.prefs.borrow().tr("Aufträge kopiert.", "Orders copied.")
                } else if let Some(clip) = self.clip {
                    for (k, order) in clip.into_iter().enumerate() {
                        self.sim(ctx.assets, Command::SetOrder { branch: b as u8, slot: k as u8, order });
                    }
                    return Next::Stay;
                } else {
                    ctx.assets.prefs.borrow().tr("Noch nichts kopiert.", "Nothing copied yet.")
                };
                self.popups.push_front(Form::new(&[&b"$C"[..], &t(msg)].concat()).popup(ctx.assets, 160, 320, None));
            }
            (Event::Key(Key::Char(c)), None) if self.room.is_none() && self.me().is_some() => {
                let (ledger, prices, log) = { let p = ctx.assets.prefs.borrow(); (p.ledger, p.price_table, p.event_log) };
                match c.to_ascii_lowercase() {
                    b'b' if ledger => self.open(ctx.assets, Menu::Ledger(0)),
                    b'p' if prices => self.open(ctx.assets, Menu::Prices),
                    b'l' if log => self.open(ctx.assets, Menu::Log(0)),
                    _ => {}
                }
            }
            _ => {}
        }
        Next::Stay
    }

    fn click_kontor(&mut self, assets: &Assets, pos: (i32, i32)) {
        self.spot_click = None;
        if self.me().is_none() {
            return;
        }
        if hit(GOLD_RECT, pos) {
            // Gold display (0x4F370): bribery (spec 03 §4.1).
            return self.open(assets, Menu::Targets(Plot::Bribe));
        }
        if hit(BANNER_RECT, pos) {
            // Banner (0x4D1E0): Spieloptionen (§10.8).
            self.room = Some(Room::Optionen);
            return;
        }
        // First match wins where rectangles overlap (Bezahlung over Handel), see `spot_table`.
        let Some(&(spot, ..)) = SPOTS.iter().find(|s| hit(s.3, pos)) else { return };
        self.spot_click = Some(pos);
        let confessionless = self.me().is_some_and(|h| self.st.persons[h].religion() == 2);
        match spot {
            Spot::Handel => self.room = Some(Room::Handel),
            Spot::Spielstand => self.open(assets, Menu::Spielstand),
            Spot::Schreibstube => self.room = Some(Room::Schreibstube),
            Spot::Bezahlung => self.room = Some(Room::Bezahlung),
            Spot::Hinterzimmer => self.room = Some(Room::Hinterzimmer),
            // §9.2: confessionless players get the confession choice instead of the church.
            Spot::Kirche if confessionless => self.open(assets, Menu::JoinChurch),
            Spot::Kirche => self.room = Some(Room::Kirche),
            Spot::Truppen => self.room = Some(Room::Truppen),
            Spot::GiveUp => self.popups.push_back(Self::yes_no(assets, assets.text(addr::GIVE_UP), [100, 140, 440, 200], Action::GiveUp)),
        }
    }

    fn spot_label(&self, assets: &Assets, spot: Spot, label: u32) -> Vec<u8> {
        let kirche_eintritt = spot == Spot::Kirche && self.me().is_some_and(|h| self.st.persons[h].religion() == 2);
        assets.text(if kirche_eintritt { KIRCHENEINTRITT } else { label })
    }

    /// §9.3 report spread: book + banner "Erträge"; left page production rows with goods sprites,
    /// right page exports. The "^ Produktion _" heading is not an EXE string we located; ours.
    /// City info (rules-11 §11.2): wealth rings, goods sprites and the warehouse icon next to the
    /// text rows. Which goods count as "Hauptproduktion" / "Begehrte Güter" is [UNVERIFIED]:
    /// the three best and three worst yields.
    fn draw_city_extras(&self, frame: &mut Frame, assets: &Assets, c: u8, w: &Window) {
        let mut rows: Vec<(i32, i32)> = vec![]; // (y, right edge) per text row
        for &(x, y, g) in w.laid.glyphs.iter().filter(|g| g.2 != text::SPACE) {
            let r = x + assets.font.glyphs[g as usize].width as i32 + 1;
            match rows.last_mut() {
                Some(last) if last.0 == y => last.1 = last.1.max(r),
                _ => rows.push((y, r)),
            }
        }
        // Begehrte Güter is the row before the blank gap above the Lager line (which may wrap to two rows)
        let Some(n) = rows.windows(2).rposition(|w| w[1].0 - w[0].0 > 30).map(|i| i + 2) else { return };
        if n < 6 {
            return;
        }
        let city = &self.st.cities[c as usize];
        let (ox, oy) = (80, 20);
        // rules-48 §1: one good per group of five, the best by yield (Hauptproduktion) and by price · 100 / (7 + g)
        // (Begehrte Güter), strict `>` from 0 so ties and all-zero groups keep the lower good
        let best = |v: &dyn Fn(usize) -> i32| -> Vec<usize> {
            (0..3).map(|k| (5 * k..5 * k + 5).fold((5 * k, 0), |(bg, bv), g| if v(g) > bv { (g, v(g)) } else { (bg, bv) }).0).collect()
        };
        let haupt = best(&|g| city.yield_(g) as i32);
        let begehrt = best(&|g| city.price(g) as i32 * 100 / (7 + g as i32));
        // %j sprites 36 px wide, 42 apart, top 6 px above the line; x from the centred lines (rules-48 §1.3)
        for (row, goods, x0) in [(n - 3, haupt, 278), (n - 2, begehrt, 273)] {
            let y = rows[row].0;
            for (k, g) in goods.into_iter().enumerate() {
                frame.blit(&assets.icon(GOOD_SPRITE + g as u16)[0], ox + x0 + 42 * k as i32, oy + y - 6);
            }
        }
        // the Lager item's icon ICON2F at the item's start (live A/B capture)
        if !w.laid.items.is_empty() {
            let it = w.item_rect(0);
            let own = self.me().is_some_and(|h| self.st.branch_at(h, c as usize).is_some());
            frame.blit(&assets.icon(if own { 0x30 } else { 0x2F })[0], it.x, it.y - ICON_DY); // open / close (formats §11.2)
        }
    }

    /// Origin of the small map `ICON25` (480x360), measured against the original.
    const MAP_AT: (i32, i32) = (80, 60);

    /// Place under the pointer on the Hinterzimmer map: a city (its 50x40 click box scaled by 3/4),
    /// else a county by its outline mask (`ICONH0..H2`), else the principality (13).
    /// rules-29 §2: 25 map-local rects, first match wins (cities, castles/camps, counties, Mittelland).
    fn map_hit(&self, _assets: &Assets, (mx, my): (i32, i32)) -> Option<u8> {
        let (x, y) = (mx - Self::MAP_AT.0, my - Self::MAP_AT.1);
        MAP_RECTS.iter().find(|&&(x1, y1, x2, y2, _)| x >= x1 && x < x2 && y >= y1 && y < y2).map(|r| r.4)
    }

    fn draw_map(&self, frame: &mut Frame, assets: &Assets, plot: Plot, mouse: (i32, i32)) {
        // over the room it was opened from (bribery comes from the Kontor's gold display, live capture)
        match self.room {
            Some(r) => self.draw_room(frame, assets, r, mouse, false),
            None => {
                frame.blit(&assets.icon(KONTOR_BG)[0], 0, 0);
                self.draw_banner(frame, assets, &assets.text(addr::KONTOR), true);
            }
        }
        let (ox, oy) = Self::MAP_AT;
        frame.blit(&assets.icon(0x25)[0], ox, oy);
        let hit = self.map_hit(assets, mouse);
        let title = match hit {
            Some(l) => format!("{} in {}", plot.verb(), self.loc(l)),
            None => format!("{}...", plot.verb()),
        };
        let src = [b"$C^ ".as_slice(), &t(&title), b" _"].concat();
        text::draw(frame, &assets.font, &text::layout(&assets.font, &src, &[], &[], 480, &[]), ox, oy);
        // rules-29 §4: the place's sprite 0x106 + place at its highlight origin
        if let Some(l) = hit {
            let (hx, hy) = MAP_HL[l as usize];
            frame.blit(&assets.icon(0x106 + l as u16)[0], ox + hx, oy + hy);
        }
    }

    /// `%4u` of docs/re/rules-12 §1.4: "einen Karren" / "5 Karren" for noun id `id`.
    fn u_form(assets: &Assets, count: i32, id: u32) -> Vec<u8> {
        let w = |n: u32| assets.text_id(n).iter().filter_map(|&c| crate::input::from_cp437(c)).collect::<String>();
        // grammatical gender of the noun (0 m, 1 f, 2 n); the residence names 58..67 by their German genders [UNVERIFIED table]
        let g = match id { 20 | 23 | 60 | 61 | 62 | 67 => 2, 24 | 58 | 59 | 63 | 65 | 66 => 1, _ => 0 };
        t(&if count == 1 { format!("{} {}", w(3 * 107 + 3 + g), w(3 * 107 + id)) } else { format!("{count} {}", w(7 * 107 + id)) })
    }

    /// The line told when a lot is sold (text ids 3037..3040, rules-12 §3.5).
    #[allow(clippy::too_many_arguments)]
    fn sold_line(&self, assets: &Assets, winner: u8, kind: u8, loc: u8, count: u8, typ: u8, word: u16) -> String {
        let w = winner;
        let src = match kind {
            12 if loc != 0xFF => subst(&subst(&subst(&assets.text_id(3037), b"%4N5", &t(&self.pref(w, 4, 5))), b"%4E", &t(&self.transporters(assets, 4, count, typ))), b"%o", &t(self.loc(loc))),
            12 => subst(&assets.text_id(3040), b"%4N5", &t(&self.pref(w, 4, 5))),
            14 => subst(&subst(&subst(&assets.text_id(3038), b"%1N1", &t(&self.pref(w, 1, 1))), b"%8W", &t(&self.wform(assets, 8, word))), b"%s", &t(self.loc(loc))),
            _ => subst(&subst(&assets.text_id(3039), b"%s", &t(self.loc(loc))), b"%4N5", &t(&self.pref(w, 4, 5))),
        };
        plain(&src)
    }

    /// A catalogue page of the auction (rules-15 §1.2): the open book, "Position n" and the lot picture on the
    /// left, the Exposé text on the right.
    #[allow(clippy::too_many_arguments)]
    fn draw_catalogue(&self, frame: &mut Frame, assets: &Assets, n: u8, kind: u8, pic: u16, seller: u8, v: [i32; 4], price: i32) {
        frame.blit(&assets.icon(BOOK)[0], 0, 40);
        frame.blit(&assets.icon(BANNER)[0], 1, 0);
        text::draw(frame, &assets.font, &text::layout(&assets.font, &assets.text(0xBAE24), &[], &[], 640, &[]), 0, 0);
        let left = [b"$t$|$4H$10>$56<$Z$C^ Position ".to_vec(), t(&(n as u32 + 1).to_string()), b" _$N$N".to_vec()].concat();
        text::draw(frame, &assets.font, &text::layout(&assets.font, &left, &[], &[], 640, &[]), 0, 0);
        frame.blit(&assets.icon(pic)[0], 118, 150);
        let u_form = |count: i32, id: u32| -> Vec<u8> {
            let w = |n: u32| assets.text_id(n).iter().filter_map(|&c| crate::input::from_cp437(c)).collect::<String>();
            // grammatical gender of the noun (0 m, 1 f, 2 n); the residence names 58..67 by their German genders [UNVERIFIED table]
        let g = match id { 20 | 23 | 60 | 61 | 62 | 67 => 2, 24 | 58 | 59 | 63 | 65 | 66 => 1, _ => 0 };
            t(&if count == 1 { format!("{} {}", w(3 * 107 + 3 + g), w(3 * 107 + id)) } else { format!("{count} {}", w(7 * 107 + id)) })
        };
        let me = t(&self.pref(seller, 3, 5));
        let right = match kind {
            12 => subst_seq(&subst_seq(&subst(&assets.text(0xBAE64), b"%4u", &u_form(v[0], 19 + v[1] as u32)), b"%3N5", std::slice::from_ref(&me)), b"%d", &[t(&v[2].to_string()), t(&price.to_string())]),
            14 => subst(&subst(&subst(&subst(&assets.text(0xBAF20), b"%5W", &t(&self.wform(assets, 5, v[0] as u16))), b"%o", &t(self.loc(v[1] as u8))), b"%3N1", &t(&self.pref(seller, 3, 1))), b"%d", &t(&dotted(price))),
            _ => {
                let adj = assets.name_table((0xB6F8C, 5)).get(v[0].clamp(0, 4) as usize).cloned().unwrap_or_default();
                let s = subst(&subst(&assets.text(0xBAFD0), b"%1N1", &t(&self.pref(seller, 1, 1))), b"%se", &t(&format!("{adj}e")));
                let s = subst(&subst(&s, b"%o", &t(self.loc(v[1] as u8))), b"%4u", &u_form(v[2], 33));
                subst_seq(&s, b"%d", &[t(&v[3].to_string()), t(&price.to_string())])
            }
        };
        text::draw(frame, &assets.font, &text::layout(&assets.font, &right, &[], &[], 640, &[]), 0, 0);
    }

    fn draw_ertraege(&self, frame: &mut Frame, assets: &Assets, prod: &[(u8, u32)], exports: Option<i64>, given: u32) {
        frame.blit(&assets.icon(BOOK)[0], 0, 40);
        self.draw_banner(frame, assets, &assets.text(addr::ERTRAEGE), true);
        // EXE 0xBD9E0 `$t$|$4H$10>$56<$Z$C^ Produktion _$Z` (rules-23 §1; live A/B: the half line before the heading)
        let mut left = assets.text(0xBD9E0);
        let mut items = vec![];
        let mut amounts = vec![];
        let mut sprites = vec![];
        if prod.is_empty() {
            left.extend(assets.text(addr::NONE_PRODUCED));
        }
        // rules-38 §8: rows in good order, at most 13 (11 with the given-away line); after the 8th row the second
        // column (the right page) starts; goods past the list are summed in "Weiteres" (0xBDA68, icon 0x150)
        let mut goods: Vec<(u8, u32)> = prod.iter().copied().filter(|p| p.1 > 0).collect();
        goods.sort_by_key(|p| p.0);
        let cap = if given > 0 { 11 } else { 13 };
        // rules-50 §3: each good's row is preceded by its harvest line for up to 1.5 s, the row then replaces it;
        // "Weiteres" and the exports follow the last row
        let (shown, n_goods) = (self.ert_reveal.0, goods.len().min(cap));
        let more: u32 = goods.iter().skip(cap).map(|p| p.1).sum();
        let rows: Vec<(u16, Vec<u8>, u32)> = goods
            .iter()
            .take(cap)
            .map(|&(g, n)| (GOOD_SPRITE + g as u16, [t(self.good(g)), b":".to_vec()].concat(), n))
            .chain((goods.len() > cap).then(|| (0x150, b"Weiteres:".to_vec(), more)))
            .collect();
        for (k, (icon, name, n)) in rows.into_iter().enumerate() {
            if k == 8 {
                left.extend(b"$|$4H$56>$10<");
            }
            if k == shown && k < n_goods {
                let g = goods[k].0 as usize;
                let q = ((129 - self.st.weather[g] as i32) / 12).clamp(0, 4) as usize;
                let word = assets.name_table((0xB62C8, 5)).get(q).cloned().unwrap_or_default();
                left.extend(subst(&assets.text_id(2061 + g as u32), b"%s", &t(&word)));
                break;
            }
            // `$L       %s:$E$R%d$Z`, the good name as an item so its line position is known
            left.extend(b"$L       %m$E$R%d$Z");
            items.push(name);
            amounts.push(t(&dotted(n)));
            sprites.push((icon, if k < 8 { 60 } else { 336 }));
        }
        let refs: Vec<&[u8]> = items.iter().map(Vec::as_slice).collect();
        let args: Vec<&[u8]> = amounts.iter().map(Vec::as_slice).collect();
        let laid = text::layout(&assets.font, &left, &refs, &args, 640, &[]);
        text::draw(frame, &assets.font, &laid, 0, 0);
        for (&(icon, x), it) in sprites.iter().zip(&laid.items) {
            frame.blit(&assets.icon(icon)[0], x, it.y - 9);
        }
        if shown < n_goods {
            return;
        }
        // the exports follow the production rows on the right page when those reach it [UNVERIFIED], else at its top
        let mut right = if sprites.len() > 8 {
            let used = laid.items.last().map_or(0, |it| it.y + 24);
            [format!("$|$56>$10<${}{{", used).into_bytes(), assets.text(addr::EXPORTS)].concat()
        } else {
            [b"$|$4H$56>$10<".as_slice(), &assets.text(addr::EXPORTS)].concat()
        };
        let rev = exports.map(|r| t(&dotted(r)));
        right.extend(assets.text(if rev.is_some() { addr::EXPORTS_BROUGHT } else { addr::NO_EXPORTS }));
        let mut args: Vec<&[u8]> = rev.iter().map(Vec::as_slice).collect();
        // rules-23 §1 quirk: a counter of exactly 1 prints as 2
        let given_s = t(&given.max(if given == 1 { 2 } else { 0 }).to_string());
        if given > 0 {
            right.extend(assets.text(addr::GIVEN_AWAY));
            args.push(&given_s);
        }
        text::draw(frame, &assets.font, &text::layout(&assets.font, &right, &[], &args, 640, &[]), 0, 0);
    }

    /// Pointer over a form item: +/- for digits, up/down for words (spec 01).
    fn form_cursor(w: &Window, acts: &[Action], mouse: (i32, i32)) -> Cursor {
        let Some(k) = w.item_at(mouse) else { return Cursor::Arrow };
        let r = w.item_rect(k);
        match acts[k] {
            Action::Step(f, _) => Cursor::for_half(mouse.1 < r.y + r.h / 2, !f.is_word()),
            _ => Cursor::Arrow,
        }
    }
}

impl Screen for Game {
    fn update(&mut self, ctx: &mut Ctx, input: &Input, dt_ms: u32) -> Next {
        ctx.audio.set_volumes(self.vol[1], self.vol[0]);
        self.scroll_sync(ctx.assets);
        let click = input.events.iter().find_map(|e| if let Event::Left(x, y) = e { Some((*x, *y)) } else { None });
        // ours (`prefs.tooltips`): the rest time of the pointer, restarted by a move or any input
        self.tip = if self.tip.0 == input.mouse && input.events.is_empty() { (input.mouse, self.tip.1.saturating_add(dt_ms)) } else { (input.mouse, 0) };
        match self.popups.front() {
            None => self.gold_hot = self.me().is_some() && hit(GOLD_RECT, click.unwrap_or(input.mouse)),
            Some(Popup::Map(Plot::Bribe)) => {}
            Some(_) => self.gold_hot = false,
        }
        let idx = self.popups.front().and_then(|p| p.window(ctx.assets, &[])).and_then(|w| w.item_at(click.unwrap_or(input.mouse)));
        if idx != self.hover.0 {
            self.hover = (idx, false);
        }
        if idx.is_some() && click.is_some() {
            self.hover.1 = true;
        }
        if self.scroll.s < self.scroll.target {
            self.scroll_ms += dt_ms;
            while self.scroll_ms >= SCROLL_MS && self.scroll.s < self.scroll.target {
                self.scroll_ms -= SCROLL_MS;
                self.scroll.s = (self.scroll.s + SCROLL_STEPS[self.text_speed as usize]).min(self.scroll.target);
            }
        } else {
            self.scroll_ms = 0;
        }
        // Erträge: one harvest line per produced good, 1.5 s each, a key or right click skips it (rules-50 §3)
        if let Some(Popup::Ertraege(prod, _, given)) = self.popups.front() {
            let n_goods = prod.iter().filter(|p| p.1 > 0).count().min(if *given > 0 { 11 } else { 13 });
            if self.ert_reveal.0 < n_goods {
                self.ert_reveal.1 += dt_ms;
                let skip = input.events.iter().any(|e| matches!(e, Event::Right(..) | Event::Key(_)));
                if skip || self.ert_reveal.1 >= 1500 {
                    self.ert_reveal = (self.ert_reveal.0 + 1, 0);
                }
                return Next::Stay;
            }
        }
        let key = self.wait_key();
        self.wait_ms = if key == self.wait_ms.1 { (self.wait_ms.0 + dt_ms, key) } else { (0, key) };
        if self.cry_ms > 0 {
            self.cry_ms = self.cry_ms.saturating_sub(dt_ms);
            if self.cry_ms == 0 {
                self.cradle_done(ctx.assets);
            }
        }
        if let Some(Popup::Film(f)) = self.popups.front_mut() {
            if let Some(n) = f.cue.take()
                && let Some(w) = ctx.assets.sample_wav(n)
            {
                ctx.audio.play_wav(w);
            }
            if let Some(v) = f.voice.take()
                && let Some(w) = ctx.assets.speech_wav(&v)
            {
                f.hold_ms = wav_ms(&w);
                ctx.audio.play_wav(w);
            }
            f.update(ctx, input, dt_ms);
            if f.done() {
                self.popups.pop_front();
            }
            return Next::Stay;
        }
        self.ask_ms += dt_ms;
        self.clock_ms = self.clock_ms.wrapping_add(dt_ms);
        for d in &mut self.fx_due {
            d.0 = d.0.saturating_sub(dt_ms);
        }
        self.sounds.extend(self.fx_due.iter().filter(|d| d.0 == 0).map(|d| d.1));
        self.fx_due.retain(|d| d.0 > 0);
        if matches!(self.popups.front(), Some(Popup::Court { .. })) {
            if self.court_len != self.popups.len() {
                (self.court_len, self.court_ms) = (self.popups.len(), 0);
            }
            self.court_ms += dt_ms;
        }
        if let Some(Popup::Election(el)) = self.popups.front_mut() {
            el.ms += dt_ms;
            self.scene_ms.0 += dt_ms;
            if el.steps.get(el.pos).is_some_and(|st| st.timed) && el.ms >= Elect::TIMEOUT_MS && !el.advance() {
                self.popups.pop_front();
            }
        }
        if let Some(Popup::Scene { boxes, .. } | Popup::Court { boxes, .. }) = self.popups.front() {
            let key = (self.popups.len(), boxes.len());
            if self.scene_ms.1 != key {
                self.scene_ms = (0, key);
            }
            self.scene_ms.0 += dt_ms;
            if hold_of(&boxes[0].1).is_some_and(|n| self.scene_ms.0 >= if self.speech_ms > 0 { self.speech_ms } else { 2000 } + n)
                && let Some(Popup::Scene { boxes, .. } | Popup::Court { boxes, .. }) = self.popups.front_mut()
            {
                boxes.remove(0);
                if boxes.is_empty() {
                    self.popups.pop_front();
                }
            }
        }
        // the duel ground and the jury court's closing pictures open with the scanline reveal (rules-16b §8)
        // (the fog picture opens from both edges, 0x2C140; the others and the duel's result bank from the centre, 0x2C460)
        match self.popups.front() {
            Some(Popup::Scene { pic, .. } | Popup::Court { pic: pic @ 0xDD..=0xE4, .. }) if matches!(*pic, 0xD1 | 0xB8 | 0xB9 | 0xDD..=0xE4) => {
                if self.revealed != Some(*pic) && self.reveal.is_none() {
                    self.revealed = Some(*pic);
                    self.reveal = Some((*pic, super::reveal::Reveal::new((*pic as u32).wrapping_mul(2_654_435_761) ^ self.st.year as u32, 0..480, *pic == 0xD1)));
                }
            }
            _ => self.revealed = None,
        }
        if let Some((_, r)) = &mut self.reveal {
            let skip = input.events.iter().any(|e| matches!(e, Event::Left(..)));
            for _ in 0..2 {
                r.step();
            }
            if skip || r.finished() {
                self.reveal = None;
            }
            return Next::Stay;
        }
        // clicks: left 0x42, other buttons 0x4C; queued effects; scene music
        for e in &input.events {
            match e {
                Event::Left(..) => self.sounds.push(0x42),
                Event::Right(..) => self.sounds.push(0x4C),
                _ => {}
            }
        }
        for n in std::mem::take(&mut self.sounds) {
            if let Some(w) = ctx.assets.sample_wav(n) {
                ctx.audio.play_wav(w);
            }
        }
        let want = match self.popups.front() {
            _ if self.opts & 0x80 == 0 => None,
            Some(Popup::Scene { boxes, .. } | Popup::Court { boxes, .. }) => speech_of(&boxes[0].1).map(|n| (boxes.len(), n)),
            Some(Popup::Choice { src, .. } | Popup::Award(src)) => speech_of(src).map(|n| (src.len(), n)),
            _ => None,
        };
        if want != self.spoken {
            if let Some((_, name)) = &want
                && let Some(w) = join_wavs(name.split('+').filter_map(|n| ctx.assets.speech_wav(n)).collect())
            {
                // the duel raises the speech gain by a tenth (rules-25 §2)
                let duel = self.popups.iter().any(|p| matches!(p, Popup::Scene { pic: 0xD1, .. } | Popup::Court { pic: 0xD9..=0xE4, .. }));
                // the default speech level 175 is the EXE's 160 %, one bar step (16) is 8 (rules-25 §2, rules-44 §1.2)
                let v = (SPEECH_GAIN as i32 + (self.vol[2] as i32 - 175) / 2).clamp(0, 255) as u32;
                self.speech_ms = wav_ms(&w);
                ctx.audio.play_wav(speech_gain(w, if duel { (v * 11 / 10).min(255) } else { v }));
            } else {
                self.speech_ms = 0;
            }
            // the cradle starts the first baby sound together with the narration (rules-19 §2.1 step 6)
            if let Some(Popup::Choice { menu: Some(Menu::Naming(slot)), .. }) = self.popups.front()
                && let Some(&(_, _, [x, _])) = self.cradles.iter().find(|c| c.0 == *slot)
                && let Some(w) = ctx.assets.speech_wav(&format!("{:07}", 14 + x))
            {
                ctx.audio.play_wav(w);
            }
            self.spoken = want;
        }
        if let Some(song) = self.scene_song() {
            ctx.audio.play_song(ctx.music_dir.as_deref(), song);
        }
        match self.room_song.take() {
            Some((song, 2)) => ctx.audio.play_song_if_idle(ctx.music_dir.as_deref(), song),
            Some((song, _)) => ctx.audio.play_song(ctx.music_dir.as_deref(), song),
            None => {}
        }
        if let Some(Popup::Battle(v)) = self.popups.front_mut() {
            v.tick();
            for n in v.take_sounds() {
                if let Some(w) = ctx.assets.sample_wav(n) {
                    ctx.audio.play_wav(w);
                }
            }
            let hotkey = match self.pending.as_ref().map(|q| &q.ask) {
                Some(Ask::Tactics { person, .. }) => Some(self.st.persons[*person as usize].hotkey()),
                _ => None,
            };
            for e in &input.events {
                match *e {
                    // intro captions wait for a click (rules-50 §1.2)
                    Event::Left(..) | Event::Right(..) if v.caption_open() => {
                        if v.next_caption() {
                            match self.pending.as_ref().map(|q| &q.ask) {
                                Some(Ask::BattleIntro { .. } | Ask::PreFight { q: mad_sim::fight::Parley::None, .. }) => return self.act(Action::Answer(0), ctx),
                                // the last page is read: the question comes up in the window (rules-52)
                                Some(Ask::PreFight { .. } | Ask::OccupyCastle { .. }) => {
                                    if let Some(k) = self.popups.iter().position(|p| matches!(p, Popup::Choice { menu: Some(Menu::Question), .. }))
                                        && let Some(p) = self.popups.remove(k)
                                    {
                                        self.popups.push_front(p);
                                    }
                                    return Next::Stay;
                                }
                                _ => {}
                            }
                        }
                    }
                    Event::Key(Key::Char(c)) if v.live && !v.menu_open() && !v.caption_open() && hotkey.is_some_and(|k| k.eq_ignore_ascii_case(&c)) => {
                        self.battle_hotkey(ctx.assets);
                        return Next::Stay;
                    }
                    Event::Left(..) | Event::Right(..) if v.report_open() => v.next_report(),
                    // a message over the field ("Wohlan", rules-52 §5) takes the click first: the fight starts
                    // after it (live A/B 2026-11-17: one click, no skip to the fight's end under the caption)
                    Event::Left(x, y) if v.overlay_open() => {
                        if let battle::Overlaid::Submit = v.click_overlay(ctx.assets, (x, y)) {
                            let a = v.answer();
                            v.close_menu();
                            return self.act(Action::Answer(a), ctx);
                        }
                    }
                    Event::Right(..) if v.overlay_open() => v.dismiss_overlay(),
                    _ if !v.done() => v.skip(),
                    Event::Left(x, y) if v.menu_open() => v.click((x, y)),
                    Event::Right(..) if v.menu_open() => {
                        if let battle::Overlaid::Submit = v.finish_menu() {
                            let a = v.answer();
                            v.close_menu();
                            return self.act(Action::Answer(a), ctx);
                        }
                    }
                    Event::Key(Key::Escape) if v.menu_open() => {
                        v.close_menu();
                        return self.act(Action::Answer(u32::MAX), ctx);
                    }
                    Event::Left(..) | Event::Right(..) | Event::Key(Key::Escape) if !v.live => {
                        self.popups.pop_front();
                        break;
                    }
                    _ => {}
                }
            }
            // the finished fight goes straight on to its report, no click (live A/B 2026-11-17, ship raid)
            if let Some(Popup::Battle(v)) = self.popups.front()
                && v.finished()
            {
                self.popups.pop_front();
                return Next::Stay;
            }
            if let Some(Popup::Battle(v)) = self.popups.front()
                && v.live && !v.menu_open() && !v.overlay_open() && v.done() && hotkey.is_some()
            {
                return self.act(Action::Answer(0), ctx);
            }
            return Next::Stay;
        }
        // ours: an auto-repeat only steps number fields; whether other items repeat depends on handlers
        // that wait for release, documented only for the castle order editor (rules-26 §7)
        if let Some(n) = self.auction_input(ctx, input) {
            return n;
        }
        let rep = (input.repeat && self.repeatable(ctx.assets, input.mouse)).then_some(Event::Left(input.mouse.0, input.mouse.1));
        for e in input.events.iter().chain(rep.iter()) {
            if matches!(self.popups.front(), Some(Popup::Choice { menu: Some(Menu::Question), .. })) && matches!(self.pending.as_ref().map(|q| &q.ask), Some(Ask::AuctionCatalog { .. } | Ask::AuctionLine { .. } | Ask::AuctionRefused { .. })) {
                // a catalogue page, hall line or refusal box is turned by a click
                if matches!(e, Event::Left(..) | Event::Key(Key::Enter)) {
                    return self.act(Action::Answer(0), ctx);
                }
                continue;
            }
            if let (Some(k), Some(Popup::Choice { menu: Some(Menu::Save), .. })) = (self.save_edit, self.popups.front()) {
                match *e {
                    Event::Key(Key::Char(c)) if self.child_name.len() < 30 && mad_assets::font::glyph_for_cp437(c).is_some() => self.child_name.push(c),
                    Event::Key(Key::Backspace) => _ = self.child_name.pop(),
                    Event::Key(Key::Enter) => {
                        let name = std::mem::take(&mut self.child_name);
                        self.save_edit = None;
                        self.popups.pop_front();
                        let _ = std::fs::create_dir_all(&ctx.assets.user_saves);
                        let _ = std::fs::write(ctx.assets.user_saves.join(format!("GAME{k}.NAM")), &name);
                        return self.act(Action::Save(k), ctx);
                    }
                    Event::Key(Key::Escape) | Event::Right(..) => {
                        (self.save_edit, self.child_name) = (None, vec![]);
                        self.popups.pop_front();
                        continue;
                    }
                    _ => continue,
                }
                self.reopen(ctx.assets, Menu::Save);
                continue;
            }
            if let Some(Popup::Choice { menu: Some(Menu::Naming(slot)), .. }) = self.popups.front() {
                let slot = *slot;
                if self.cry_ms > 0 {
                    continue;
                }
                match *e {
                    Event::Key(Key::Char(c)) if self.child_name.len() < 20 && mad_assets::font::glyph_for_cp437(c).is_some() => self.child_name.push(c),
                    Event::Key(Key::Backspace) => _ = self.child_name.pop(),
                    Event::Key(Key::Enter) if !self.child_name.is_empty() => {
                        let name: String = self.child_name.iter().filter_map(|&b| crate::input::from_cp437(b)).collect();
                        self.st.set_name_str(slot as usize, &name);
                        self.child_name.clear();
                        // step 10: the second baby sound after the name
                        if let Some(k) = self.cradles.iter().position(|c| c.0 == slot) {
                            let [x, r] = self.cradles.remove(k).2;
                            if let Some(w) = ctx.assets.speech_wav(&format!("{:07}", 14 + (x + 1 + r) % 5)) {
                                self.cry_ms = wav_ms(&w).max(1);
                                ctx.audio.play_wav(w);
                                continue;
                            }
                        }
                        self.cradle_done(ctx.assets);
                        continue;
                    }
                    _ => continue,
                }
                self.reopen(ctx.assets, Menu::Naming(slot));
                continue;
            }
            // the castle order strip is part of the castle screen, not a modal window: clicks outside it,
            // and a right click, go to the screen (A/B: Info, expansion, recruiting)
            if let (Some(Popup::Choice { rect, menu: Some(Menu::Orders(_)), .. }), true) = (self.popups.front(), self.popups.len() == 1) {
                let outside = match *e {
                    Event::Left(x, y) => !hit(*rect, (x, y)),
                    Event::Right(..) | Event::Key(Key::Escape) => true,
                    _ => false,
                };
                if outside {
                    // windows opened from the screen go in front of the strip; the seal ring toggles it
                    let n = self.handle_room_event(ctx, *e);
                    if !matches!(self.room, Some(Room::Burg(_))) {
                        self.popups.retain(|p| !matches!(p, Popup::Choice { menu: Some(Menu::Orders(_)), .. }));
                    } else if self.popups.len() > 1 && matches!(self.popups.front(), Some(Popup::Choice { menu: Some(Menu::Orders(_)), .. })) {
                        // a window queued behind the strip goes in front of it
                        let s = self.popups.pop_front().unwrap();
                        self.popups.insert(1, s);
                    }
                    if !matches!(n, Next::Stay) {
                        return n;
                    }
                    continue;
                }
            }
            if let Some(Popup::Scene { boxes, .. } | Popup::Court { boxes, .. }) = self.popups.front()
                && hold_of(&boxes[0].1).is_some()
            {
                continue;
            }
            // ours (`prefs.shortcuts`): 1-9 click the n-th bullet of the open menu (number fields do not count)
            if let Event::Key(Key::Char(c @ b'1'..=b'9')) = *e
                && ctx.assets.prefs.borrow().shortcuts
                && let Some(Popup::Choice { acts, .. }) = self.popups.front()
            {
                let acts = acts.clone();
                let k = tips::nth_bullet(&acts, (c - b'1') as usize);
                if let Some(k) = k
                    && let Some(w) = self.popups.front().and_then(|p| p.window(ctx.assets, &[]))
                {
                    let r = w.item_rect(k);
                    let n = self.click_form(ctx, &w, &acts, (r.x + r.w / 2, r.y + r.h / 2));
                    if !matches!(n, Next::Stay) {
                        return n;
                    }
                }
                continue;
            }
            if !self.popups.is_empty() {
                let pos = match *e {
                    Event::Left(x, y) => (x, y),
                    // A sim question must be answered; it cannot be closed.
                    // a right click declines the lender's offer and the director candidates (live A/B 2026-11-14/15);
                    // other questions need an answer
                    Event::Right(..) if matches!(self.pending.as_ref().map(|p| &p.ask), Some(Ask::LoanOffer { .. })) => return self.act(Action::Answer(1), ctx),
                    Event::Right(..) if matches!(self.pending.as_ref().map(|p| &p.ask), Some(Ask::PickDirector { .. })) => return self.act(Action::Answer(u8::MAX as u32), ctx),
                    Event::Right(..) if matches!(self.popups.front(), Some(Popup::Choice { menu: Some(Menu::Question), .. })) && let Some(Ask::PreFight { q, .. }) = self.pending.as_ref().map(|p| &p.ask) => {
                        use mad_sim::fight::Parley;
                        // rules-52: a right click is "Niemals" (0), withdraw (1), "Genug" (0), stay out (2)
                        let a = match q { Parley::Attack { .. } => 1, Parley::Side { .. } => 2, _ => 0 };
                        return self.act(Action::Answer(a), ctx);
                    }
                    Event::Right(..) | Event::Key(Key::Escape) if matches!(self.popups.front(), Some(Popup::Choice { menu: Some(Menu::Question), .. })) => continue,
                    // the vote must be made: no way out of the election screen at a prompt (playtest report)
                    Event::Right(..) | Event::Key(Key::Escape) if matches!(self.popups.front(), Some(Popup::Election(el)) if el.steps.get(el.pos).is_some_and(|st| st.ask)) => continue,
                    // elsewhere on the election screen a right click turns the step like a left click
                    Event::Right(x, y) if matches!(self.popups.front(), Some(Popup::Election(_))) => (x, y),
                    Event::Right(..) | Event::Key(Key::Escape) => {
                        self.popups.pop_front();
                        continue;
                    }
                    _ => continue,
                };
                if let Some(&Popup::Map(plot)) = self.popups.front() {
                    if let Some(l) = self.map_hit(ctx.assets, pos).filter(|&l| l <= 13) {
                        self.reopen(ctx.assets, Menu::Tree(plot, l, false));
                    }
                    continue;
                }
                let win = self.popups.front().and_then(|p| p.window(ctx.assets, &[]));
                let scrolled = matches!(self.popups.front(), Some(Popup::Report(_) | Popup::YearReport(_) | Popup::Chronicle(_))) && self.scroll_on(ctx.assets);
                match self.popups.front_mut() {
                    Some(Popup::Election(el)) => {
                        if el.steps.get(el.pos).is_some_and(|st| st.ask) {
                            // the human elector clicks a candidate line (step 4); the vote goes to the sim
                            let k = el.line_at(pos);
                            if let Some(k) = k.filter(|_| matches!(self.pending.as_ref().map(|p| &p.ask), Some(Ask::Vote { .. }))) {
                                return self.act(Action::Answer(k as u32), ctx);
                            }
                            if self.pending.is_none() && !el.advance() {
                                self.popups.pop_front();
                            }
                        } else if !el.advance() {
                            self.popups.pop_front();
                        }
                    }
                    Some(Popup::Scene { pic, boxes } | Popup::Court { pic, boxes, .. }) => {
                        if *pic == 0xD1
                            && let Some(fx) = self.duel_fx.pop_front()
                        {
                            self.fx_due.extend(fx.into_iter().map(|(ms, id)| (ms as u32, id)));
                        }
                        boxes.remove(0);
                        if boxes.is_empty() {
                            self.popups.pop_front();
                        }
                    }
                    Some(Popup::Report(_) | Popup::YearReport(_) | Popup::Chronicle(_)) if scrolled => {}
                    Some(Popup::Choice { acts, .. }) => {
                        let acts = acts.clone();
                        let n = self.click_form(ctx, win.as_ref().unwrap(), &acts, pos);
                        if !matches!(n, Next::Stay) {
                            return n;
                        }
                    }
                    _ => drop(self.popups.pop_front()),
                }
                continue;
            }
            let n = self.handle_room_event(ctx, *e);
            if !matches!(n, Next::Stay) {
                return n;
            }
        }
        // nobody left: the credits without the box and the save (0x4DA50, rules-38 §6)
        if self.me().is_none() && self.popups.is_empty() {
            return self.credits(ctx);
        }
        if self.kontor_left && self.room.is_none() && self.popups.is_empty() {
            (self.kontor_left, self.entered) = (false, false);
        }
        if self.popups.is_empty() && !self.entered {
            // §9.2: entering the Kontor rolls its song from the game RNG, at turn start and after every return from
            // the rooms of rules-39 §4
            self.entered = true;
            self.sim(ctx.assets, Command::KontorView);
        }
        Next::Stay
    }

    fn draw(&self, frame: &mut Frame, assets: &Assets, mouse: (i32, i32)) -> Option<Cursor> {
        let c = self.draw_screen(frame, assets, mouse);
        if self.room == Some(Room::Handel) && self.branch.is_none() && self.popups.is_empty() && assets.prefs.borrow().sell_all {
            tips::draw_label(frame, assets, assets.prefs.borrow().tr("Verkaufen (V)", "Sell all (V)"), rooms::SELL_ALL_AT);
        }
        if let Some(tip) = self.tooltip(assets, mouse) {
            tips::draw_tip(frame, assets, &tip, mouse);
        }
        // the wheel of wait_click: 16 frames, one per BIOS tick (~55 ms), restarted by every wait (rules-41 §3)
        match (c, self.wait_ms.1) {
            (Some(_), Some(_)) => Some(Cursor::Wheel((self.wait_ms.0 / 55 % 16) as u8)),
            _ => c,
        }
    }
}

impl Game {
    fn draw_screen(&self, frame: &mut Frame, assets: &Assets, mouse: (i32, i32)) -> Option<Cursor> {
        frame.begin(assets);
        match self.popups.front() {
            Some(Popup::Battle(v)) => return v.draw(frame, assets, mouse),
            Some(Popup::Film(f)) => return f.draw(frame, assets, mouse),
            Some(Popup::NextPlayer(src)) => {
                frame.blit(&assets.icon(NEXT_PLAYER_BG)[0], 0, 0);
                text::draw(frame, &assets.font, &text::layout(&assets.font, src, &[], &[], 640, &[]), 0, 0);
                return Some(Cursor::Arrow);
            }
            Some(p @ Popup::Choice { menu: Some(Menu::Naming(_)), .. }) => {
                frame.blit(&assets.icon(0xCE)[0], 0, 0);
                // the name being typed is the focused field, drawn highlighted with its caret (rules-42 §3)
                p.window(assets, &[0]).unwrap().draw_text(frame, assets);
                return Some(Cursor::Arrow);
            }
            Some(p @ Popup::Choice { items, acts, .. }) if self.is_scene(p) => {
                frame.blit(&assets.icon(KUPPLERIN_BG)[0], 0, 0);
                let hover: Vec<usize> = p.window(assets, &[]).unwrap().item_at(mouse).filter(|_| !self.hover.1).into_iter().collect();
                let w = p.window(assets, &hover).unwrap();
                w.draw_text(frame, assets);
                for k in 0..items.len() {
                    let it = w.item_rect(k);
                    let x = if it.w < 20 { it.x } else { it.x - 21 };
                    frame.blit(&assets.icon(0x73)[0], x, it.y);
                }
                return Some(Self::form_cursor(&w, acts, mouse));
            }
            Some(Popup::Choice { menu: Some(Menu::Question), .. }) if matches!(self.pending.as_ref().map(|q| &q.ask), Some(Ask::AuctionCatalog { .. })) => {
                if let Some(Ask::AuctionCatalog { n, kind, pic, seller, a, b, c, d, price }) = self.pending.as_ref().map(|q| q.ask.clone()) {
                    self.draw_catalogue(frame, assets, n, kind, pic, seller, [a, b, c, d], price);
                }
                return Some(Cursor::Arrow);
            }
            Some(p @ Popup::Choice { menu: Some(Menu::Question), acts, .. })
                if matches!(self.pending.as_ref().map(|q| &q.ask), Some(Ask::JuryBribe { .. } | Ask::BlackmailCanon { .. } | Ask::CanonVeto { .. } | Ask::Vows { .. } | Ask::WeddingFeast { .. })) =>
            {
                // caption menus on a picture, frameless gold text, ICON73 bullets: the creditors' jury on the county castle
                // (rules-15 §6.1), the wedding questions on the altar and the banquet (§7.1)
                let pic = match self.pending.as_ref().map(|q| &q.ask) {
                    Some(Ask::JuryBribe { county, .. }) => 0x193 + *county as u16,
                    Some(Ask::WeddingFeast { .. }) => 0xFF,
                    _ => 0xBE,
                };
                frame.blit(&assets.icon(pic)[0], 0, 0);
                let hover: Vec<usize> = p.window(assets, &[]).unwrap().item_at(mouse).filter(|_| !self.hover.1).into_iter().collect();
                let w = p.window(assets, &hover).unwrap();
                w.draw_text(frame, assets);
                let jury = matches!(self.pending.as_ref().map(|q| &q.ask), Some(Ask::JuryBribe { .. }));
                for k in 0..if jury { 1 } else { acts.len() } {
                    let it = w.item_rect(k);
                    frame.blit(&assets.icon(0x73)[0], it.x, it.y);
                }
                return Some(Self::form_cursor(&w, acts, mouse));
            }
            Some(p @ Popup::Choice { menu: Some(Menu::Question), .. }) if matches!(self.pending.as_ref().map(|q| &q.ask), Some(Ask::AuctionRefused { .. })) => {
                // a refused bid: the hall with the auctioneer at rest, the message box over it
                frame.blit(&assets.icon(0xD2)[0], 0, 0);
                frame.blit_clip(&assets.icon(0xD5)[0], 0, 0, false, false, [196, 28, 413, 192]);
                p.window(assets, &[]).unwrap().draw_over(frame, assets, |_| {});
                return Some(Cursor::Arrow);
            }
            Some(p @ Popup::Choice { menu: Some(Menu::Question), .. }) if matches!(self.pending.as_ref().map(|q| &q.ask), Some(Ask::AuctionTick { .. } | Ask::AttendAuction { .. } | Ask::AuctionLine { .. })) => {
                // auction hall (rules-15 §1): the room with the auctioneer, the lot picture, the text area at the bottom
                let (bank, lot) = match self.pending.as_ref().map(|q| &q.ask) {
                    Some(Ask::AuctionTick { high, pic, .. }) => (Some(0xD5 + high.map_or(0, |h| h as u16 % 4)), Some(*pic)),
                    Some(Ask::AuctionLine { step: 1..=3, pic, winner, .. }) => (Some(0xD5 + *winner as u16 % 4), Some(*pic)),
                    _ => (None, None),
                };
                frame.blit(&assets.icon(if bank.is_some() { 0xD2 } else { 0xC1 })[0], 0, 0);
                if let Some(bank) = bank {
                    // the auctioneer: frame 0 of his bank is a patch at (196, 28); the sim's strip (rules-25 §1) is composited
                    // cumulatively over it, the segments new since the last tick at 55 ms (one BIOS tick) a frame
                    let clip = [196, 28, 413, 192];
                    frame.blit_clip(&assets.icon(bank)[0], 0, 0, false, false, clip);
                    if let Some(Ask::AuctionTick { strip, fresh, .. }) = self.pending.as_ref().map(|q| &q.ask) {
                        let mut budget = self.ask_ms as usize / 55;
                        for (n, &(sb, a, b)) in strip.iter().enumerate() {
                            let bank = assets.icon(sb);
                            for k in a..b {
                                if n >= *fresh as usize {
                                    if budget == 0 {
                                        break;
                                    }
                                    budget -= 1;
                                }
                                frame.blit_clip(bank.get(k as usize).unwrap_or(&bank[0]), 0, 0, false, false, clip);
                            }
                        }
                    }
                }
                if let Some(pic) = lot {
                    let (x, y) = if (0x160..=0x169).contains(&pic) || (0x16A..=0x16C).contains(&pic) { (437, 56) } else { (445, 44) };
                    frame.blit(&assets.icon(pic)[0], x, y);
                }
                let hover: Vec<usize> = p.window(assets, &[]).unwrap().item_at(mouse).filter(|_| !self.hover.1).into_iter().collect();
                let w = p.window(assets, &hover).unwrap();
                if matches!(self.pending.as_ref().map(|q| &q.ask), Some(Ask::AttendAuction { .. })) {
                    // the attendance question is a parchment window (120, 240 − h/2, 400, h, 0x8810), rules-12 §3.5
                    // (playtest report: dark text straight on the picture)
                    w.draw(frame, assets);
                } else {
                    w.draw_text(frame, assets); // gold text straight on the hall, no parchment (live run 2026-11-10)
                }
                if let Popup::Choice { acts, .. } = p {
                    return Some(Self::form_cursor(&w, acts, mouse));
                }
                return Some(Cursor::Arrow);
            }
            Some(Popup::Court { pic, .. }) if self.reveal.as_ref().is_some_and(|(rp, _)| rp == pic) => {
                let mut full = Frame::new(assets.palette);
                full.blit(&assets.icon(*pic)[0], 0, 0);
                frame.fill_rgb([0, 0, 640, 480], [0, 0, 0]);
                self.reveal.as_ref().unwrap().1.draw(frame, &full);
                return Some(Cursor::Arrow);
            }
            Some(p @ Popup::Court { pic, figures, anim, .. }) => {
                frame.blit(&assets.icon(*pic)[0], 0, 0);
                let per = Self::frame_ms(*pic, *anim);
                let step = anim.map_or(0, |(a, b)| (a + (self.court_ms / per) as usize).min(b));
                for &(icon, x, y) in figures {
                    let bank = assets.icon(icon);
                    // bank frames after 0 are deltas over the figure's full frame 0 (their opaque parts are small), played in
                    // sequence from the segment's first frame; the court figures rest while a head talks (rules-18 §6)
                    frame.blit(&bank[0], x, y);
                    let moving = !((0xE5..=0xF0).contains(&icon) && figures.iter().any(|f| (0x18E..=0x195).contains(&f.0)));
                    if let (true, Some((from, _))) = (moving, *anim) {
                        for k in from.max(1)..=step {
                            if let Some(img) = bank.get(k) {
                                frame.blit(img, x, y);
                            }
                        }
                    }
                }
                p.window(assets, &[]).unwrap().draw_text(frame, assets);
                return Some(Cursor::Arrow);
            }
            Some(Popup::Scene { pic, .. }) if self.reveal.as_ref().is_some_and(|(rp, _)| rp == pic) => {
                let mut full = Frame::new(assets.palette);
                full.blit(&assets.icon(*pic)[0], 0, 0);
                frame.fill_rgb([0, 0, 640, 480], [0, 0, 0]);
                self.reveal.as_ref().unwrap().1.draw(frame, &full);
                return Some(Cursor::Arrow);
            }
            Some(Popup::Election(el)) => {
                el.draw(frame, assets, self.scene_ms.0 / 10, mouse);
                return Some(Cursor::Arrow);
            }
            Some(p @ Popup::Scene { pic, .. }) => {
                frame.blit(&assets.icon(*pic)[0], 0, 0);
                p.window(assets, &[]).unwrap().draw_text(frame, assets);
                return Some(Cursor::Arrow);
            }
            Some(p @ Popup::Bill(..)) => {
                frame.blit(&assets.icon(0xB7)[0], 0, 0);
                self.draw_banner(frame, assets, &assets.text(0xBB3B0), true);
                p.window(assets, &[]).unwrap().draw_over(frame, assets, |_| {});
                return Some(Cursor::Arrow);
            }
            Some(p @ Popup::Will(_)) => {
                frame.blit(&assets.icon(0xC8)[0], 0, 0);
                p.window(assets, &[]).unwrap().draw_over(frame, assets, |_| {});
                return Some(Cursor::Arrow);
            }
            Some(p @ Popup::Award(_)) => {
                frame.blit(&assets.icon(0xB7)[0], 0, 0);
                p.window(assets, &[]).unwrap().draw_over(frame, assets, |f| f.blit(&assets.icon(0xCB)[0], 80 + 26, 20 + 360));
                return Some(Cursor::Arrow);
            }
            Some(p @ (Popup::YearReport(_) | Popup::Chronicle(_))) => {
                frame.blit(&assets.icon(0xB8)[0], 0, 0);
                // banner of player 0xFF (formats §9.1): the title alone, no gold display or name
                frame.blit(&assets.icon(BANNER)[0], 1, 0);
                let title = [b"$t$|$H$Z$CEreignisse in Mittelland, A.D. ".as_slice(), &t(&self.shown_year().to_string())].concat();
                text::draw(frame, &assets.font, &text::layout(&assets.font, &title, &[], &[], 640, &[]), 0, 0);
                p.window(assets, &[]).unwrap().with_text_dy(-self.scroll_of(p)).draw_over(frame, assets, |_| {});
                return Some(Cursor::Arrow);
            }
            Some(p @ Popup::Report(lines)) => {
                // turn-end report (rules-12 §2): wooden desk ICONB7, banner "Ereignisse", text window at (30, 110, 520, 290)
                frame.blit(&assets.icon(0xB7)[0], 0, 0);
                self.draw_banner(frame, assets, &assets.text(0xBB3B0), true);
                let w = p.window(assets, &[]).unwrap().with_text_dy(-self.scroll_of(p));
                w.draw_over(frame, assets, |_| {});
                // the gift menu's choices keep their gems (rules-63 §2; playtest report), the chosen one lit (live A/B)
                let [_, y, _, h] = REPORT_BOX;
                let lit = report_lit(lines);
                for k in 0..w.laid.items.len() {
                    let it = w.item_rect(k);
                    if it.y >= y && it.y + 16 <= y + h {
                        frame.blit(&assets.icon(if lit.contains(&k) { 0x74 } else { 0x73 })[0], it.x, it.y);
                    }
                }
                return Some(Cursor::Arrow);
            }
            Some(Popup::Map(_)) => {}
            Some(Popup::Ertraege(prod, exports, given)) => {
                self.draw_ertraege(frame, assets, prod, *exports, *given);
                return Some(Cursor::Arrow);
            }
            _ => {}
        }
        let mut cursor = Cursor::Arrow;
        let free = self.popups.is_empty();
        // a question asked over a battle (pre-fight window rules-52, leader lost, occupy) shows the field under it
        let field = self.popups.iter().skip(1).find_map(|p| if let Popup::Battle(v) = p { v.live.then_some(v) } else { None });
        if let Some(v) = field {
            v.draw(frame, assets, (-1, -1));
        } else if matches!(self.popups.front(), Some(Popup::Choice { rect, .. }) if *rect == REPORT_BOX) {
            // a menu inside the turn-end report window (the courtship gifts, rules-63 §2): the desk and its banner
            frame.blit(&assets.icon(0xB7)[0], 0, 0);
            self.draw_banner(frame, assets, &assets.text(0xBB3B0), true);
        } else {
        match self.room {
            Some(r) => {
                if r == Room::Optionen {
                    // the options window floats over the Kontor
                    frame.blit(&assets.icon(KONTOR_BG)[0], 0, 0);
                    self.draw_banner(frame, assets, &assets.text(addr::KONTOR), true);
                }
                self.draw_room(frame, assets, r, mouse, free && self.branch.is_none())
            }
            None => {
                frame.blit(&assets.icon(KONTOR_BG)[0], 0, 0);
                self.draw_banner(frame, assets, &assets.text(addr::KONTOR), true);
            }
        }
        match self.branch {
            Some(b) => cursor = self.draw_branch(frame, assets, b, if free { mouse } else { (-1, -1) }),
            None if self.room.is_none() => {
                // The hovered hotspot's label in gold, centred on its §9.2 position; a dialog opened from a
                // hotspot leaves its label on screen underneath (live captures: Spielstand, Geld zum Fenster…)
                let at = if free { Some(mouse) } else { self.spot_click };
                if let Some(&(spot, label, (lx, ly), _)) = at.and_then(|m| SPOTS.iter().find(|s| hit(s.3, m))) {
                    let src = [b"$g$C".as_slice(), &self.spot_label(assets, spot, label)].concat();
                    // one line centred on x, clipped by the screen edge when it does not fit (checked
                    // against the original: "Geld zum Fenster hinauswerfen" runs off the right edge)
                    let w = 1280;
                    text::draw(frame, &assets.font, &text::layout(&assets.font, &src, &[], &[], w, &[]), lx - w / 2, ly - 12);
                }
            }
            None => {}
        }
        }
        if let Some(Popup::Map(plot)) = self.popups.front() {
            self.draw_map(frame, assets, *plot, mouse);
            return Some(Cursor::Arrow);
        }
        // a message over the office list leaves the list visible underneath (live A/B)
        let is_tree = |p: &Popup| matches!(p, Popup::Choice { menu: Some(Menu::Tree(..)), .. });
        let is_score = |p: &Popup| matches!(p, Popup::Choice { menu: Some(Menu::Spielstand), .. });
        let is_strip = |p: &Popup| matches!(p, Popup::Choice { menu: Some(Menu::Orders(_)), .. });
        if let (Some(front), Some(under)) = (self.popups.front(), self.popups.get(1))
            && is_strip(under)
            && !is_strip(front)
            && let Some(w) = under.window(assets, &[])
        {
            w.draw_over(frame, assets, |_| {});
        }
        // a privilege's info box opens over the privileges menu, which stays with its gems (live A/B; Bewerbung too)
        let is_priv = |p: &Popup| matches!(p, Popup::Choice { menu: Some(Menu::Privileges | Menu::City(_) | Menu::Apply), .. });
        if let (Some(front), Some(under)) = (self.popups.front(), self.popups.get(1))
            && is_priv(under)
            && !is_priv(front)
            && let Some(w) = under.window(assets, &[])
        {
            w.draw_over(frame, assets, |_| {});
            if let Popup::Choice { menu: Some(Menu::City(c)), .. } = under {
                // the city info's goods and Lager icons stay under its message box (live A/B)
                self.draw_city_extras(frame, assets, *c, &w);
            } else {
                for k in 0..w.laid.items.len() {
                    let it = w.item_rect(k);
                    if it.w < 20 {
                        frame.blit(&assets.icon(0x73)[0], it.x, it.y);
                    }
                }
            }
        }
        if let (Some(front), Some(under)) = (self.popups.front(), self.popups.get(1))
            && !is_tree(front)
            && !is_score(front)
            && (is_tree(under) || is_score(under))
            && let Some(w0) = under.window(assets, &[])
        {
            // the entry under the pointer stays lit under a message box (live A/B: the spy price over the tree)
            let lit: Vec<usize> = match (is_tree(under), under) {
                (true, Popup::Choice { acts, .. }) => {
                    let key = |k: usize| acts.get(k).map(|a| format!("{a:?}"));
                    let hot = w0.item_at(mouse).and_then(key);
                    (0..acts.len())
                        .filter(|&k| {
                            let a = key(k);
                            (a.is_some() && a == hot) != (a.is_some() && a == self.tree_rival)
                        })
                        .collect()
                }
                _ => vec![],
            };
            let w = under.window(assets, &lit).unwrap_or(w0);
            let tree = is_tree(under);
            w.draw_over(frame, assets, |f| if tree { f.blit(&assets.icon(0x1B9)[0], 20, 40) });
            let bars: Vec<(i32, i32, i32)> = if tree { self.tree_bars.clone() } else { (0..self.score_bars.len()).map(|k| (256, w.item_rect(k).y + 3, self.score_bars[k])).collect() };
            for (x, y, bw) in bars {
                frame.fill_rgb([x, y, bw, if tree { 9 } else { 15 }], [0, 0, 0]);
            }
            if !tree {
                for k in 0..self.score_bars.len() {
                    let it = w.item_rect(k);
                    frame.blit(&assets.icon(0x73)[0], it.x, it.y);
                }
            }
        }
        if let Some(p) = self.popups.front()
            && let Some(w) = p.window(assets, &[])
        {
            let mut hover: Vec<usize> = w.item_at(mouse).filter(|_| !self.hover.1).into_iter().collect();
            let editing = self.save_edit.filter(|_| matches!(p, Popup::Choice { menu: Some(Menu::Save), .. }));
            if let Some(k) = editing {
                // the slot being named is red with a red "<" after the text (live A/B)
                hover = vec![k as usize];
            }
            let tree = matches!(p, Popup::Choice { menu: Some(Menu::Tree(..)), .. });
            if let (true, Popup::Choice { acts, .. }) = (tree, p) {
                // rules-31 §5.2/§3.3: an entry lights up as a whole; the rival's entry is lit at rest and dark under the pointer
                let key = |k: usize| acts.get(k).map(|a| format!("{a:?}"));
                let hot = hover.first().and_then(|&k| key(k));
                hover = (0..acts.len())
                    .filter(|&k| {
                        let a = key(k);
                        let lit = a.is_some() && a == hot;
                        let rival = a.is_some() && a == self.tree_rival;
                        lit != rival
                    })
                    .collect();
            }
            p.window(assets, &hover).unwrap().draw_over(frame, assets, |f| {
                if tree {
                    // rules-31 §4: the parchment ICONR9 with the shields and arms painted in, over the frame
                    f.blit(&assets.icon(0x1B9)[0], 20, 40);
                }
            });
            if let Popup::Choice { gems: true, items, acts, .. } = p {
                // measured: the gem sits 21 px left of the choice text, centred on its row
                let w2 = p.window(assets, &[]).unwrap();
                for (k, item) in items.iter().enumerate() {
                    let it = w2.item_rect(k);
                    // lit (0x74) when selected or under the pointer: the kind-0x90 painter swaps the bullet on hover
                    // (rules-63 §2; playtest report)
                    let selected = self.sel == Some(k as u32) && self.sel_menu.is_some_and(|m| matches!(p, Popup::Choice { menu: Some(x), .. } if *x == m));
                    // the contest entry is a multi-select: every chosen discipline keeps its gem lit
                    let chosen = k < 5 && matches!(self.pending.as_ref().map(|q| &q.ask), Some(Ask::ContestEntry { .. })) && self.amount >> k & 1 == 1;
                    let live = !matches!(acts.get(k), Some(Action::Inert));
                    let id = if selected || chosen || (live && hover.contains(&k)) { 0x74 } else { 0x73 };
                    // the bullet is the item itself (`%a`), its top on the line's top (live A/B: Kerzen, Optionen);
                    // a spinner digit next to it is no bullet
                    // scrolled report-window menus: no gem above or below the window
                    let inside = !matches!(p, Popup::Choice { rect, .. } if *rect == REPORT_BOX && (it.y < rect[1] || it.y + 16 > rect[1] + rect[3]));
                    if it.w < 20 && !item.last().is_some_and(u8::is_ascii_digit) && inside {
                        frame.blit(&assets.icon(id)[0], it.x, it.y);
                    }
                }
            }
            if let Popup::Choice { acts, .. } = p {
                cursor = Self::form_cursor(&w, acts, mouse);
            }
            if let Some(k) = editing {
                let it = w.item_rect(k as usize);
                text::draw_caret(frame, &assets.font, it.x + it.w, it.y);
            }
            if tree {
                for &(x, y, w) in &self.tree_bars {
                    frame.fill_rgb([x, y, w, 9], [0, 0, 0]);
                }
            }
            if let Popup::Choice { menu: Some(m), .. } = p {
                let w2 = p.window(assets, &[]).unwrap();
                for &(_, k, id) in self.form_icons.iter().filter(|i| i.0 == *m) {
                    let it = w2.item_rect(k);
                    // the 18 px bullet's top on the line's top, the 32 px icons ICON_DY above it (live A/B)
                    frame.blit(&assets.icon(id)[0], it.x, it.y + if id == 0x73 { 0 } else { -ICON_DY });
                }
            }
            if let Popup::Choice { menu: Some(m @ Menu::Transport(..)), .. } = p
                && self.trans_pic.0 == *m
            {
                frame.blit(&assets.icon(self.trans_pic.1)[0], 100 + 12, 20 + 48);
            }
            if let Popup::Choice { menu: Some(Menu::Residence(b)), .. } = p {
                // rules-34 §1: the level's picture and the six item icons
                let pr = &self.st.props[*b];
                if pr.has_residence() {
                    frame.blit(&assets.icon(0x60 + (pr.residence() & 0x1F).min(9))[0], 60 + 186, 40 + 144);
                }
                let w2 = p.window(assets, &[]).unwrap();
                for (k, id) in [0x6A, 0x6E, 0x6F, 0x70, 0x71, 0x72].into_iter().enumerate() {
                    let it = w2.item_rect(k);
                    frame.blit(&assets.icon(id)[0], it.x, it.y - ICON_DY);
                }
            }
            if let Popup::Choice { menu: Some(Menu::Spielstand | Menu::Ledger(0)), .. } = p {
                // rules-28 §3: solid black bars from x 256, 15 px high, 3 px below the row's line top
                // (ours: the ledger's net worth bars likewise, from x 420)
                let x0 = if matches!(p, Popup::Choice { menu: Some(Menu::Spielstand), .. }) { 256 } else { 420 };
                let w2 = p.window(assets, &[]).unwrap();
                for (k, &bw) in self.score_bars.iter().enumerate() {
                    let it = w2.item_rect(k);
                    if bw > 0 {
                        frame.fill_rgb([x0, it.y + 3, bw, 15], [0, 0, 0]);
                    }
                    frame.blit(&assets.icon(0x73)[0], it.x, it.y);
                }
            }
            if let Popup::Choice { menu: Some(m @ (Menu::Expand(q) | Menu::Improve(q, _))), .. } = p {
                let pr = &self.st.props[*q];
                let castle = pr.kind() == mad_sim::state::KIND_CASTLE;
                let w2 = p.window(assets, &[]).unwrap();
                match m {
                    Menu::Expand(_) => {
                        frame.blit(&assets.icon(if castle { 0x8B + pr.loc() as u16 - 14 } else { 0x8E })[0], 160, 112);
                        for (k, id) in [if castle { 0x8F } else { 0xA7 }, 0x90].into_iter().enumerate() {
                            let it = w2.item_rect(k);
                            frame.blit(&assets.icon(id)[0], it.x, it.y - ICON_DY);
                        }
                    }
                    Menu::Improve(_, which) => {
                        let it = w2.item_rect(0);
                        frame.blit(&assets.icon(if *which == 1 { 0x90 } else if castle { 0x8F } else { 0xA7 })[0], it.x, it.y - ICON_DY);
                    }
                    _ => {}
                }
            }
            if let Popup::Choice { menu: Some(m @ (Menu::Recruit(q) | Menu::Manoeuvre(q) | Menu::Toll(q))), .. } = p {
                let castle = self.st.props[*q].kind() == mad_sim::state::KIND_CASTLE;
                let (pic, over, icon) = match m {
                    Menu::Recruit(_) => (if castle { 0x91 } else { 0xA8 }, None, 0x6A),
                    Menu::Manoeuvre(_) => (self.mv_pic, None, 0x92),
                    _ => (0x93, Some(0xA2 - self.st.castles[self.st.props[*q].loc() as usize - 14].toll_rate.min(14) as u16), 0),
                };
                frame.blit(&assets.icon(pic)[0], 160, 112);
                if let Some(o) = over {
                    frame.blit(&assets.icon(o)[0], 160, 112);
                }
                if icon != 0 {
                    let w2 = p.window(assets, &[]).unwrap();
                    let it = w2.item_rect(0);
                    frame.blit(&assets.icon(icon)[0], it.x, it.y - ICON_DY);
                }
            }
            if let Popup::Choice { menu: Some(Menu::CastleInfo(k)), .. } = p
                && let Some(q) = self.place_prop(*k)
            {
                let pr = &self.st.props[q];
                frame.blit(&assets.icon(if pr.kind() == mad_sim::state::KIND_CASTLE { 0x8B + *k as u16 } else { 0x8E })[0], 160, 122);
                let me = self.me().unwrap_or(0);
                let mine = pr.owner() as usize == me;
                let castle = pr.kind() == mad_sim::state::KIND_CASTLE;
                let icons: &[(u16, i32)] = match (mine, castle) {
                    (true, true) if !self.st.heap.has_key(16, q as u8) => &[(0x6F, 126), (0x70, 350)],
                    (true, false) => &[(0x30, -1)],
                    (false, _) if self.st.heap.find(17, me as u8, q as u8).is_some() => &[(0x6A, -1)],
                    (false, _) if self.st.heap.all(17, mad_sim::heap::ANY, q as u8).len() < 6 => &[(0x6A, -1)],
                    _ => &[],
                };
                // x -1: the centred single item, icon at the item's start
                for &(i, x) in icons {
                    let x = if x < 0 { p.window(assets, &[]).unwrap().item_rect(0).x } else { x };
                    frame.blit(&assets.icon(i)[0], x, 410);
                }
            }
            if let Popup::Choice { menu: Some(Menu::City(c)), .. } = p {
                self.draw_city_extras(frame, assets, *c, &w);
            }
            if let Popup::Choice { menu: Some(Menu::Laws(_)), items, .. } = p {
                let w2 = p.window(assets, &[]).unwrap();
                for k in 0..3 {
                    let it = w2.item_rect(items.len() - 3 + k);
                    frame.blit(&assets.icon(0xB2 + k as u16)[0], it.x, it.y - ICON_DY);
                }
            }
            if let Popup::Choice { menu: Some(Menu::Music), .. } = p {
                // Volume bars (live A/B 2026-11-14): a black 128 x 15 box every 24 px, the red fill 9 px high from its
                // left edge, one pixel per level 0..127; the speech bar shows its level halved (rules-44 §1.2)
                for (i, &v) in self.vol.iter().enumerate() {
                    let y = 183 + 24 * i as i32;
                    frame.fill_rgb([300, y, 128, 15], [0, 0, 0]);
                    let v = if i == 2 { v as i32 / 2 } else { v as i32 };
                    frame.fill_rgb([300, y + 3, v, 9], [198, 41, 41]);
                }
            }
        }
        Some(cursor)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn escape_and_subst() {
        assert_eq!(t("a$b%ü"), b"a$$b%%\x81");
        assert_eq!(subst(b"x%1N1, %1N4$N", b"%1N1", b"Anna"), b"xAnna, %1N4$N");
    }

    #[test]
    fn form_inline_and_digits() {
        let mut f = Form::new(b"");
        f.inline(b"$LFor %d?$Z%aJa.%aNein!$|", vec![Action::EndTurn, Action::Close]);
        // `%a`: the bullet is the item, the label plain text after it (rules-28 §4)
        assert_eq!(f.src, b"$LFor %d?$Z$L$2>%m$E$B$6>Ja.$N$L$2>%m$E$B$6>Nein!$N$|");
        assert_eq!(f.items, [b"   ".to_vec(), b"   ".to_vec()]);
        let mut f = Form::new(b"");
        f.digits(407, 4, Field::Bribe);
        assert_eq!(f.items.concat(), b"0.407");
        assert!(matches!(f.acts[1], Action::Step(Field::Bribe, 100)));
    }

    pub(crate) fn test_assets() -> Option<Assets> {
        let root = std::env::var("MAD_DATA").ok()?;
        let mut a = Assets::load(root.into(), None, None).unwrap();
        a.user_saves = std::env::temp_dir().join(format!("mad-app-game-{}-{:?}", std::process::id(), std::thread::current().id()));
        // the defaults, not the user's prefs.txt
        a.prefs = std::cell::RefCell::new(crate::prefs::Prefs::default());
        Some(a)
    }

    pub(super) fn ctx_for<'a>(assets: &'a Assets, audio: &'a mut crate::audio::Audio) -> Ctx<'a> {
        Ctx { assets, audio, music_dir: None }
    }

    /// App soak: late-game states from the sim soak (`mad_sim::soak`), then random clicks, right clicks and
    /// keys on the game screen for many ticks, drawing every frame. Reports every panic with seed, tick and the
    /// last input. `MAD_APP_SOAK=<seeds>:<sim years>:<ticks>` (default 6:40:2000); the sim part cheats the
    /// humans solvent, so the Kontor rooms have something to show.
    #[test]
    #[ignore]
    fn real_random_input_soak() {
        use mad_sim::soak::{Rnd, Setup, play};
        let Some(assets) = test_assets() else { return };
        let (seeds, years, ticks) = std::env::var("MAD_APP_SOAK")
            .ok()
            .and_then(|v| {
                let mut it = v.split(':').map(|x| x.parse::<u32>().ok());
                Some((it.next()??, it.next()?? as u16, it.next()??))
            })
            .unwrap_or((6, 40, 2000));
        let szenario = assets.read("SZENARIO.DAT").unwrap();
        let mut fails = vec![];
        for seed in 1..=seeds {
            let setup = Setup::from_seed(seed);
            let st = match play(&setup, years, 8, &szenario, &mut |_, _| {}) {
                Ok((_, st)) => st,
                Err(f) => {
                    fails.push(format!("sim seed {seed}: {f:?}"));
                    continue;
                }
            };
            if st.current >= NOBODY {
                continue;
            }
            let mut rnd = Rnd((seed as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x5EED);
            let mut last = String::from("start");
            let mut t = 0;
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut g = Game::start(&assets, st);
                let mut audio = crate::audio::Audio::new(false);
                let mut ctx = ctx_for(&assets, &mut audio);
                let mut frame = Frame::new(assets.palette);
                while t < ticks {
                    let (x, y) = (rnd.n(640) as i32, rnd.n(480) as i32);
                    let ev = match rnd.n(12) {
                        0..=3 => Some(Event::Left(x, y)),
                        4 => Some(Event::Right(x, y)),
                        5 => Some(Event::Key(match rnd.n(4) {
                            0 => Key::Escape,
                            1 => Key::Enter,
                            2 => Key::Backspace,
                            _ => Key::Char(b'a' + rnd.n(26) as u8),
                        })),
                        _ => None,
                    };
                    if let Some(e) = ev {
                        last = format!("tick {t}: {e:?}");
                    }
                    let input = Input { mouse: (x, y), events: ev.into_iter().collect(), ..Default::default() };
                    if !matches!(g.update(&mut ctx, &input, 10), Next::Stay) {
                        break;
                    }
                    g.draw(&mut frame, &assets, (x, y));
                    t += 1;
                }
            }));
            if let Err(p) = res {
                let msg = p.downcast_ref::<String>().cloned().or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                fails.push(format!("seed {seed} after {last}: {msg}"));
            }
        }
        let _ = std::fs::remove_dir_all(&assets.user_saves);
        assert!(fails.is_empty(), "{fails:#?}");
    }

    /// New game → both players end their turn → year 1601, back in the first player's Kontor,
    /// with the year-end autosave written to the app's own save directory.
    #[test]
    fn real_new_game_year_cycle() {
        let Some(assets) = test_assets() else { return };
        let ps: Vec<_> = [3, 5].map(mad_sim::world::HumanSetup::test).into_iter().collect();
        let mut g = Game::start(&assets, State::new_game(7, 3, &ps).unwrap());
        let first = g.me();
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        for _ in 0..2 {
            g.act(Action::EndTurn, &mut ctx);
        }
        assert_eq!((g.st.year, g.me()), (1601, first));
        assert!(g.popups.iter().any(|p| matches!(p, Popup::Ertraege(..))));
        // the year's own windows still show 1600 (live A/B); the Kontor after them shows 1601
        g.popups.retain(|p| matches!(p, Popup::Ertraege(..)));
        assert!(g.banner_text(b"Kontor").windows(4).any(|w| w == b"1601"));
        let auto = assets.user_saves.join("GAME5.DAT");
        assert_eq!(State::from_game_dat(&std::fs::read(&auto).unwrap(), 1).unwrap().year, 1601);
        // ours: no history without the option; with it, the year lands in HISTORY/<year>.DAT
        assert!(crate::assets::history_years(&assets.history_dir()).is_empty());
        assets.prefs.borrow_mut().autosave_years = 5;
        g.report(&assets, vec![SimEvent::YearEnd { year: 1600 }], vec![]);
        assert!(crate::assets::history_years(&assets.history_dir()).is_empty());
        g.st.year = 1605;
        g.report(&assets, vec![SimEvent::YearEnd { year: 1604 }], vec![]);
        assert_eq!(crate::assets::history_years(&assets.history_dir()), [1605]);
        assert_eq!(State::from_game_dat(&std::fs::read(assets.history_dir().join("1605.DAT")).unwrap(), 1).unwrap().year, 1605);
        std::fs::remove_dir_all(&assets.user_saves).unwrap();
    }

    /// Ours: with the Rückgängig option the turn end can be taken back once; off, nothing is kept.
    #[test]
    fn real_undo_restores_turn() {
        let Some(assets) = test_assets() else { return };
        let ps: Vec<_> = [3, 5].map(mad_sim::world::HumanSetup::test).into_iter().collect();
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        let mut g = Game::start(&assets, State::new_game(7, 3, &ps).unwrap());
        g.seed = Some(7);
        g.act(Action::EndTurn, &mut ctx);
        assert!(g.undo.is_none() && g.undo_game(&assets).is_none());
        assets.prefs.borrow_mut().undo = true;
        let before = g.st.clone();
        g.act(Action::EndTurn, &mut ctx);
        assert_ne!(g.st.current, before.current);
        g.popups.clear();
        g.open(&assets, Menu::Options);
        shot(&g, &assets, "options_undo");
        let back = g.undo_game(&assets).unwrap();
        assert_eq!((back.st.current, back.st.year, back.seed), (before.current, before.year, Some(7)));
        assert!(g.undo.is_none());
        std::fs::remove_dir_all(&assets.user_saves).ok();
    }

    /// rules-55 §10: at a turn start the certificate charter follows the Erträge window.
    #[test]
    fn real_certificate_after_ertraege() {
        let Some(assets) = test_assets() else { return };
        let ps: Vec<_> = [3].map(mad_sim::world::HumanSetup::test).into_iter().collect();
        let mut g = Game::start(&assets, State::new_game(7, 3, &ps).unwrap());
        let h = g.me().unwrap() as u8;
        g.popups.clear();
        g.report(&assets, vec![SimEvent::TurnStart { person: h, year: 1601 }, SimEvent::Certificate { person: h, level: 1 }], vec![]);
        let at = |f: fn(&Popup) -> bool| g.popups.iter().position(f).unwrap();
        assert!(at(|p| matches!(p, Popup::Ertraege(..))) < at(|p| matches!(p, Popup::Award(_))));
    }

    /// With `MAD_SHOT_DIR` set, writes the current frame there (visual check of dialogs that a
    /// scripted UI run cannot reach).
    pub(super) fn shot(g: &Game, assets: &Assets, name: &str) {
        shot_at(g, assets, name, (630, 470));
    }

    pub(super) fn shot_at(g: &Game, assets: &Assets, name: &str, mouse: (i32, i32)) {
        let Ok(dir) = std::env::var("MAD_SHOT_DIR") else { return };
        let mut frame = crate::gfx::Frame::new(assets.palette);
        g.draw(&mut frame, assets, mouse);
        crate::gfx::write_screenshot(&format!("{dir}/{name}.png"), &frame).unwrap();
    }

    /// rules-56 §10: after a normal end the outro's last frame waits until speech 13 has run; a click does not wait.
    #[test]
    fn real_outro_waits_for_speech() {
        use super::super::Screen;
        let Some(assets) = test_assets() else { return };
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        let Some(mut f) = super::super::film::Film::new(&assets, "OUTROB.FLC", |_| Next::Stay).map(|f| f.delay_cs(10)) else { return };
        f.hold_ms = 70_000; // longer than the film (588 frames at 100 ms)
        let none = crate::input::Input::default();
        let mut t = 0;
        while !f.done() && t < 100_000 {
            f.update(&mut ctx, &none, 1000);
            t += 1000;
        }
        assert_eq!(t, 70_000);
        let mut g = super::super::film::Film::new(&assets, "OUTROB.FLC", |_| Next::Stay).unwrap();
        g.hold_ms = 70_000;
        g.update(&mut ctx, &crate::input::Input { events: vec![Event::Right(0, 0)], ..Default::default() }, 10);
        assert!(g.done());
    }

    /// Every question dialog builds, has its answers as items, and the amount digits feed the
    /// answer (`Ask` values are made up; reachability is covered by the replay test).
    #[test]
    fn real_question_dialogs() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(4, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        let h = g.me().unwrap() as u8;
        use mad_sim::fight::Parley;
        let pre = |q| Ask::PreFight { code: 4, colours: [0, 1, 0, 0], song: 0x10, frames: vec![], lines: vec![], q };
        // rules-52: (name, question, items, the spinner's answer item, the EXE wording)
        for (name, q, n, k, words) in [
            ("ransom", pre(Parley::Ransom { defender: h, attacker: 3 }), 7, Some((0, 200)), &["einige Taler zukommen lassen", " Taler.", "Niemals, ich gewinne!"][..]),
            ("accept", pre(Parley::Attack { attacker: h }), 2, None, &["soll der Angriff stattfinden?", "Gewiss, he he!", "Ich lasse Gnade vor Recht ergehen."]),
            // the briber's own troops (row 1) have no item
            ("bribe_helper", pre(Parley::Bribe { briber: h, helpers: vec![(3, 0), (h, 1)] }), 7, Some((0, 1 + 200)), &["Ihr k", "Taler f", "Genug bestochen!"]),
            ("helper_side", pre(Parley::Side { helper: h, attacker: 3, defender: 5 }), 3, None, &["Mit wem wollt Ihr k", "Ich schaue lieber zu."]),
            ("occupy", Ask::OccupyCastle { code: 4, attacker: 1, defender: 2, loc: 14, verdict: 1 }, 2, None, &[]),
        ] {
            g.popups.clear();
            g.pending = Some(Pending { cmd: None, answers: vec![], ask: q });
            g.amts = [0; 4];
            g.step(&assets, Field::Amt(0, 99_999), 200);
            g.open(&assets, Menu::Question);
            let Some(Popup::Choice { acts, src, .. }) = g.popups.front() else { panic!("{name}") };
            assert_eq!(acts.len(), n, "{name}");
            if let Some((i, a)) = k {
                assert!(matches!(acts[i], Action::Answer(x) if x == a), "{name}: {:?}", acts[i]);
            }
            let text = String::from_utf8_lossy(src);
            assert!(words.iter().all(|w| text.contains(w)) && !text.contains("%s") && !text.contains("N0"), "{name}: {text}");
            shot(&g, &assets, name);
        }
    }

    /// "Neuer Spieler" opens the creation screen before 1671 and refuses afterwards.
    #[test]
    fn real_new_player_guard() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        assert!(matches!(g.act(Action::NewPlayer, &mut ctx), Next::Go(_)));
        g.st.year = 1671;
        assert!(matches!(g.act(Action::NewPlayer, &mut ctx), Next::Stay) && !g.popups.is_empty());
    }

    /// Mission success replays the first history entry per code, each on its picture, then congratulates.
    #[test]
    fn real_mission_success_scenes() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        let h = g.me().unwrap();
        g.st.history(h, 4, 1, 0);
        g.st.history(h, 6, 3, 0);
        g.st.history(h, 5, 16, 10);
        let scenes = g.mission_success_scenes(&assets, h);
        let pics: Vec<u16> = scenes.iter().map(|s| if let Popup::Scene { pic, .. } = s { *pic } else { 0 }).collect();
        assert_eq!(pics, vec![0xBB, 0xCE, 0x193 + 10, 0xCA, 0xBB, 0]);
        if let Popup::Scene { boxes, .. } = &scenes[2] {
            let s = String::from_utf8_lossy(&boxes[0].1).into_owned();
            assert!(!s.contains("%s") && !s.contains("%o") && s.contains("Kanzler von"), "{s}");
        }
        assert!(matches!(scenes.last(), Some(Popup::Film(f)) if f.song.is_none() && f.voice.as_deref() == Some("0000013")), "the outro ends it, spoken");
    }

    /// A courtship year asks for a gift; refusing costs nothing (rules-05 §3).
    #[test]
    fn real_courtship_gift_dialog() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        let h = g.me().unwrap();
        let (c, fee, w) = g.st.matchmaker_offer(h).expect("a candidate");
        g.st.start_courtship(h, c, fee, w);
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        g.act(Action::EndTurn, &mut ctx);
        let Some(Ask::Gift { target, .. }) = g.pending.as_ref().map(|p| p.ask.clone()) else { panic!("no gift question: {:?}", g.pending.as_ref().map(|p| &p.ask)) };
        assert_eq!(target as usize, c);
        // rules-63 §2: the turn-end lines so far come first (the bill), the menu follows in the report window
        assert!(matches!(g.popups.front(), Some(Popup::Bill(..))), "the bill before the gift menu");
        let k = g.popups.iter().position(|p| matches!(p, Popup::Choice { menu: Some(Menu::Question), .. })).expect("the gift menu");
        assert!(matches!(&g.popups[k], Popup::Choice { rect, acts, .. } if *rect == REPORT_BOX && acts.len() == 4));
        g.popups.drain(..k);
        shot(&g, &assets, "courtship_gift");
        g.act(Action::Answer(3), &mut ctx);
        // the turn-end window goes on with the menu on top (rules-63 §2; playtest: the answer vanished at once)
        assert!(matches!(g.popups.front(), Some(Popup::Report(l)) if l[0].contains("schenken") && l.len() >= 2), "{:?}", g.popups.iter().map(|p| match p { Popup::Report(l) => format!("Report {l:?}"), Popup::Choice { src, .. } => format!("Choice {}", plain(src)), Popup::Bill(..) => "Bill".into(), _ => "other".into() }).collect::<Vec<_>>());
        // its choices keep their gem bullets (playtest report)
        assert_eq!(g.popups.front().unwrap().window(&assets, &[]).unwrap().laid.items.len(), 4);
        // the chosen one stays lit (live A/B 2026-11-22)
        assert!(matches!(g.popups.front(), Some(Popup::Report(l)) if report_lit(l) == [3]));
        shot(&g, &assets, "courtship_gift_answered");
        let i = g.st.heap.find(30, h as u8, mad_sim::heap::ANY).expect("still courting");
        assert_eq!(mad_sim::heap::r32(&g.st.heap.recs[i], 4), 0, "refusing the gift is free");
    }

    /// rules-21 §2.2 (live A/B 2026-11-22): the continue question of a human's siege is printed into the turn-end
    /// window and its answer line follows under it.
    #[test]
    fn real_siege_continue_in_report() {
        let Some(assets) = test_assets() else { return };
        for seed in 1..40 {
            let mut st = State::new_game(seed, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap();
            let h = st.current as usize;
            let army = st.cities[4].population() / 5 + 1;
            let i = st.heap.alloc(25, h as u8, mad_sim::state::NONE, 3).unwrap();
            st.heap.recs[i][4] = 4;
            mad_sim::heap::w16(&mut st.heap.recs[i], 5, army as u16);
            let mut g = Game::start(&assets, st);
            g.popups.clear();
            let mut audio = crate::audio::Audio::new(false);
            let mut ctx = ctx_for(&assets, &mut audio);
            g.act(Action::EndTurn, &mut ctx);
            if !matches!(g.pending.as_ref().map(|p| &p.ask), Some(Ask::ContinueSiege { .. })) {
                continue;
            }
            let k = g.popups.iter().position(|p| matches!(p, Popup::Choice { rect, .. } if *rect == REPORT_BOX)).expect("question in the report window");
            g.popups.drain(..k);
            g.act(Action::Answer(0), &mut ctx);
            assert!(matches!(g.popups.front(), Some(Popup::Report(l)) if report_lit(l) == [0] && l.iter().any(|x| x.contains("fortgesetzt"))));
            return;
        }
        panic!("no seed asked the continue question");
    }

    /// A sub-plot question after the gift in one turn end (rules-13, rules-63 §2): the gift reaction is shown before
    /// the plot dialog, the dialog sits in the report window, and its answer prints under the dialog (playtest report:
    /// the answer landed under the gift menu).
    #[test]
    fn real_subplot_after_gift_keeps_order() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        let h = g.me().unwrap();
        let (c, fee, w) = g.st.matchmaker_offer(h).expect("a candidate");
        g.st.start_courtship(h, c, fee, w);
        // the inventor (type 32) asks at counter 3; allocated after the courtship, it comes later in the record pass
        g.st.heap.alloc(32, h as u8, mad_sim::state::NONE, 4).unwrap();
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        g.act(Action::EndTurn, &mut ctx);
        assert!(matches!(g.pending.as_ref().map(|p| &p.ask), Some(Ask::Gift { .. })));
        let k = g.popups.iter().position(|p| matches!(p, Popup::Choice { menu: Some(Menu::Question), .. })).unwrap();
        g.popups.drain(..k);
        g.act(Action::Answer(0), &mut ctx);
        assert!(matches!(g.pending.as_ref().map(|p| &p.ask), Some(Ask::Plot { kind: 32, .. })), "{:?}", g.pending.as_ref().map(|p| &p.ask));
        // first the window with the gift menu and its reaction, then the plot dialog in the report window
        let text = |p: &Popup| match p { Popup::Report(l) => l.iter().map(|x| plain(&report_src(std::slice::from_ref(x)))).collect::<Vec<_>>().join("|"), Popup::Choice { src, .. } => plain(src), _ => String::new() };
        let shown: Vec<String> = g.popups.iter().map(text).collect();
        let gift = shown.iter().position(|s| s.contains("schenken")).expect("gift window");
        let plot = g.popups.iter().position(|p| matches!(p, Popup::Choice { rect, menu: Some(Menu::Question), .. } if *rect == REPORT_BOX)).expect("plot dialog in the report window");
        assert!(gift < plot && shown[gift].contains(" ist "), "{shown:?}");
        let dialog = shown[plot].clone();
        g.popups.drain(..plot);
        shot(&g, &assets, "subplot_question");
        // a dialog longer than the window shows its last choice (rules-63 §2.3; playtest: it sat under the roll)
        let w = g.popups[0].window(&assets, &[]).unwrap();
        let last = w.item_rect(w.laid.items.len() - 1);
        assert!(last.y + last.h <= REPORT_BOX[1] + REPORT_BOX[3], "last choice at {}", last.y);
        g.act(Action::Answer(1), &mut ctx);
        // one turn-end window: gift menu, reaction, the inventor's dialog, the refusal (0xAA202) under it
        let after: Vec<String> = g.popups.iter().map(text).collect();
        let refusal = plain(&assets.text(0xAA202));
        let r = after.iter().find(|s| s.contains(&refusal[..20])).expect("refusal line");
        let at = |x: &str| r.find(x).unwrap_or_else(|| panic!("{x} missing in {r}"));
        assert!(dialog.contains("schenken") && at("schenken") < at(" ist ") && at(" ist ") < at("Erfinder") && at("Erfinder") < at(&refusal[..20]), "{r}");
        let k = g.popups.iter().position(|p| text(p).contains(&refusal[..20])).unwrap();
        g.popups.drain(..k);
        // it opens where the dialog's window ended, not at the top
        g.scroll_sync(&assets);
        assert!(g.scroll.s > 0, "continued window starts at the top");
        shot(&g, &assets, "subplot_after_gift");
    }

    /// The year-end chronicle popup (option Geschichtsschreibung) quotes the year's EXE entry.
    #[test]
    fn real_chronicle_table() {
        let Some(assets) = test_assets() else { return };
        let g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        let text = |y: usize| String::from_utf8_lossy(&assets.text(g.tables.chronicle[y - 1600])).into_owned();
        assert!(text(1600).contains("Ostindische"), "{}", text(1600));
        assert!(text(1603).contains("Elisabeth"), "{}", text(1603));
        assert!(text(1699).contains("Karlowitz"), "{}", text(1699));
    }

    /// Every courtship/siege/vote question builds a dialog with one item per answer.
    #[test]
    fn real_new_question_dialogs() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        let h = g.me().unwrap() as u8;
        let c = (0..mad_sim::state::PERSONS).find(|&p| g.st.persons[p].used() && !g.st.persons[p].human()).unwrap() as u8;
        let qs = [
            (Ask::GiveUpCourtship { target: c, city: 3 }, 2),
            (Ask::Vows { suitor: h, target: c, canon: None }, 3),
            (Ask::WeddingFeast { suitor: h, spouse: c, city: 3, prices: [900, 600, 300, 100], title: None, song: 4 }, 5),
            (Ask::BlackmailCanon { by: h, canon: c, suitor: h, target: c }, 2),
            (Ask::CanonVeto { canon: c, suitor: h, target: c }, 2),
            (Ask::ContinueSiege { city: 3, cost: 500 }, 2),
            (Ask::SiegeWon { city: 3 }, 3),
            (Ask::Vote { elector: h, candidates: vec![c, h], for_: 0xFF, loc: 3, seat: 6, religion: 0, rivals: vec![0xFF, 0xFF], privileged: false, votes: vec![] }, 2),
            (Ask::Vote { elector: h, candidates: vec![c, h], for_: c, loc: 3, seat: 6, religion: 0, rivals: vec![0xFF, 0xFF], privileged: false, votes: vec![] }, 2),
            (Ask::LeaderLost { person: h }, 1),
            (Ask::AttendAuction { person: h }, 2),
            (Ask::Plea, 2),
            (Ask::AuctionLine { step: 0, pic: 0xC1, kind: 0, seller: 0, winner: 0, a: 1620, b: 3, c: 0, d: 0 }, 0),
            (Ask::AuctionLine { step: 1, pic: 0x15A, kind: 12, seller: h, winner: 0, a: 3, b: 0, c: 0, d: 0 }, 0),
            (Ask::AuctionLine { step: 2, pic: 0x160, kind: 14, seller: h, winner: h, a: 5 | 60 << 8, b: 0, c: 3, d: 800 }, 0),
            (Ask::AuctionLine { step: 3, pic: 0x160, kind: 14, seller: h, winner: 0, a: 0, b: 0, c: 0, d: 0 }, 0),
            (Ask::AuctionLine { step: 4, pic: 0xC1, kind: 0, seller: 0, winner: 0, a: 1, b: 0, c: 0, d: 0 }, 0),
            (Ask::AuctionCatalog { n: 0, kind: 12, pic: 0x15A, seller: h, a: 3, b: 0, c: 80, d: 0, price: 120 }, 0),
            (Ask::AuctionCatalog { n: 1, kind: 14, pic: 0x162, seller: h, a: 2 | 60 << 8, b: 3, c: 0, d: 0, price: 800 }, 0),
            (Ask::AuctionCatalog { n: 2, kind: 16, pic: 0x15C, seller: h, a: 2, b: 14, c: 40, d: 30, price: 3000 }, 0),
            (Ask::BlackmailVerdict { by: h, judge: c }, 2),
            (Ask::BuyCastle { prop: 0, price: 4000 }, 2),
            (Ask::LoanOffer { lender: 0, amount: 5000, rate: 8, years: 3 }, 2),
            (Ask::ResidenceSwap { branch: 0, new_word: 5 | 80 << 8 }, 2),
            (Ask::AuctionTick { lot: 0, price: 120, high: None, pic: 0x15A, strip: vec![], fresh: 0, blow: false, hammer: 0 }, 0),
            (Ask::AuctionTick { lot: 0, price: 150, high: Some(h), pic: 0x160, strip: vec![], fresh: 0, blow: false, hammer: 0 }, 0),
            (Ask::AuctionRefused { person: h, text: 0xBABB0, city: 3 }, 0),
            (Ask::FencingLesson { cost: 300 }, 2),
            (Ask::Plot { kind: 32, step: 0, arg: 3, arg2: 0 }, 3),
            (Ask::Plot { kind: 32, step: 1, arg: 0, arg2: 0 }, 2),
            (Ask::Plot { kind: 32, step: 2, arg: 0, arg2: 0 }, 2),
            (Ask::Plot { kind: 32, step: 3, arg: 0, arg2: 0 }, 2),
            (Ask::Plot { kind: 32, step: 10, arg: 0, arg2: 0 }, 2),
            (Ask::Plot { kind: 41, step: 0, arg: 800, arg2: 0 }, 2),
            (Ask::Plot { kind: 42, step: 0, arg: 90, arg2: 0x123 }, 2),
            (Ask::Plot { kind: 48, step: 0, arg: 400, arg2: 5 | 1 << 8 }, 2),
            (Ask::Plot { kind: 49, step: 0, arg: 4, arg2: 0 }, 2),
            (Ask::Plot { kind: 49, step: 10, arg: c as i32, arg2: 3 }, 1 + 4),
            (Ask::Plot { kind: 34, step: 10, arg: 7, arg2: 2 | 30 << 4 | 90 << 14 }, 3 + 4 + 1),
            (Ask::Plot { kind: 39, step: 0, arg: 3 | 1 << 4 | 2 << 6, arg2: 0 }, 2),
            (Ask::Plot { kind: 39, step: 1, arg: 300, arg2: 0 }, 4),
            (Ask::Plot { kind: 40, step: 0, arg: 2 | 1 << 4 | 3 << 5 | 4 << 9 | 5 << 13 | 9 << 16, arg2: 0 }, 2),
            (Ask::Plot { kind: 40, step: 10, arg: 0, arg2: 0 }, 3),
            (Ask::Plot { kind: 40, step: 11, arg: 3, arg2: 0 }, 2),
            (Ask::Plot { kind: 40, step: 12, arg: 900, arg2: 3 }, 6),
            (Ask::Plot { kind: 37, step: 0, arg: 120, arg2: 3 | 1 << 5 | 2 << 8 | 4 << 11 | 1 << 14 | 2 << 16 | 4 << 18 | 3 << 22 }, 2),
            (Ask::Plot { kind: 33, step: 0, arg: 5 | 3 << 7 | 1 << 12 | 2 << 13, arg2: 0 }, 2),
            (Ask::Plot { kind: 33, step: 1, arg: 300, arg2: 4 | 2 << 6 | 1 << 10 | 5 << 11 }, 2),
            (Ask::Plot { kind: 33, step: 10, arg: 1 | 6 << 4 | 15 << 8, arg2: 4 | 2 << 6 }, 3),
            (Ask::Plot { kind: 33, step: 11, arg: 6, arg2: 4 }, 3 + 1),
            (Ask::Plot { kind: 38, step: 0, arg: 5 | 1 << 8, arg2: 0 }, 2),
            (Ask::Plot { kind: 38, step: 1, arg: 0, arg2: 0 }, 7),
            (Ask::Plot { kind: 38, step: 10, arg: 40, arg2: 9 }, 2 + 1),
            (Ask::Plot { kind: 38, step: 11, arg: 0, arg2: 0 }, 2),
            (Ask::Plot { kind: 36, step: 0, arg: 5 | 20 << 8 | 18 << 16 | 3 << 22, arg2: 0 }, 2),
            (Ask::Plot { kind: 36, step: 10, arg: 20 | 25 << 7 | 30 << 14 | 5 << 21, arg2: 10 | 12 << 5 | 8 << 10 | 1 << 15 | 2 << 19 | 3 << 23 }, 3),
            (Ask::Plot { kind: 46, step: 0, arg: 5 | 1 << 8, arg2: 0 }, 2),
            (Ask::Plot { kind: 46, step: 10, arg: 120, arg2: 5 | 1 << 8 }, 4),
            (Ask::Plot { kind: 45, step: 0, arg: 10 << 8, arg2: 0 }, 2),
            (Ask::Plot { kind: 45, step: 10, arg: 150 | 200 << 9 | 300 << 18, arg2: 5 << 8 }, 3),
            (Ask::Plot { kind: 44, step: 0, arg: 5000, arg2: 0 }, 2),
            (Ask::Plot { kind: 47, step: 0, arg: 3000, arg2: 2 | 3 << 3 | 1 << 7 | 5 << 11 | 1 << 18 | 400 << 20 }, 2),
            (Ask::Plot { kind: 43, step: 10, arg: 4, arg2: 1 << 8 }, 6),
            (Ask::Plot { kind: 43, step: 11, arg: 15, arg2: 0 }, 2),
            (Ask::Plot { kind: 43, step: 12, arg: 1, arg2: 0 }, 2),
            (Ask::Plot { kind: 49, step: 1, arg: 0, arg2: 0 }, 2),
            (Ask::Plot { kind: 49, step: 2, arg: 0, arg2: 0 }, 2),
            (Ask::Plot { kind: 41, step: 10, arg: 3, arg2: 0 }, 2),
            (Ask::Plot { kind: 41, step: 11, arg: 800, arg2: 3 }, 2),
            (Ask::Plot { kind: 41, step: 11, arg: 800, arg2: -1 }, 2),
            (Ask::Plot { kind: 32, step: 11, arg: 700, arg2: 0 }, 2),
            (Ask::Plot { kind: 40, step: 10, arg: 5, arg2: 2 | 1 << 4 | 3 << 5 | 4 << 9 | 1 << 13 }, 3),
            (Ask::Plot { kind: 41, step: 11, arg: 800, arg2: 3 | 1 << 4 }, 2),
            (Ask::ContestEntry { person: h, fee: 300, goods: [0, 3, 6, 9, 12], paper: 1 }, 6),
            (Ask::JudgeVote { judge: h }, 2),
            (Ask::Sentence { defendant: c }, 2),
            (Ask::JuryBribe { person: h, county: 11 }, 1 + 4),
            (Ask::TrialWitnesses { person: h }, 5 + 5 + 1),
        ];
        for (k, (q, n)) in qs.into_iter().enumerate() {
            g.popups.clear();
            g.pending = Some(Pending { cmd: None, answers: vec![], ask: q.clone() });
            g.open(&assets, Menu::Question);
            let Some(Popup::Choice { acts, .. }) = g.popups.back() else { panic!("{q:?}: no dialog") };
            assert_eq!(acts.len(), n, "{q:?}");
            shot(&g, &assets, &format!("question_new{k}"));
        }
        // the lead-in texts (rules-13 §2.8, §2.9) and the contest lines (§3.1) are filled
        for (q, want) in [
            (Ask::Plot { kind: 40, step: 10, arg: 5, arg2: 2 | 1 << 4 | 3 << 5 | 4 << 9 }, "Stunden"),
            (Ask::Plot { kind: 40, step: 10, arg: 5, arg2: 1 << 13 }, "gähnen"),
            (Ask::Plot { kind: 41, step: 11, arg: 800, arg2: 3 }, "beeindruckt"),
            (Ask::Plot { kind: 41, step: 11, arg: 800, arg2: 3 | 1 << 4 }, "Reichtum"),
            (Ask::ContestEntry { person: h, fee: 300, goods: [0, 3, 6, 9, 12], paper: 1 }, "Fertig"),
        ] {
            g.popups.clear();
            g.pending = Some(Pending { cmd: None, answers: vec![], ask: q.clone() });
            g.open(&assets, Menu::Question);
            let Some(Popup::Choice { src, items, .. }) = g.popups.back() else { panic!("{q:?}: no dialog") };
            let all = [plain(src), items.iter().map(|i| plain(i)).collect::<Vec<_>>().join(" ")].join(" ");
            assert!(all.contains(want) && !all.contains("%s") && !all.contains("%d") && !all.contains("$|"), "{q:?}: {all}");
        }
    }

    /// rules-18 §4, §10: the cycling plea wraps both ways, the sentence skips "Entzug des Titels" when asked to.
    #[test]
    fn real_choice_cycles() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(4, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.amount = 7;
        g.step(&assets, Field::Choice(9, false), 1);
        g.step(&assets, Field::Choice(9, false), 1);
        assert_eq!(g.amount, 0);
        g.step(&assets, Field::Choice(9, false), -1);
        assert_eq!(g.amount, 8);
        g.amount = 2;
        g.step(&assets, Field::Choice(7, true), 1);
        assert_eq!(g.amount, 4);
        g.step(&assets, Field::Choice(7, true), -1);
        assert_eq!(g.amount, 2);
        g.step(&assets, Field::Choice(7, false), 1);
        assert_eq!(g.amount, 3);
    }

    /// A newborn child of a human gets a naming window; typed letters and Enter write the name.
    #[test]
    fn real_child_naming() {
        use crate::input::{Input, Key};
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        let h = g.me().unwrap();
        g.st.names[7][0] = b'?';
        g.report(&assets, vec![SimEvent::ChildBorn { parent: h as u8, slot: 7, female: false, cry: [4, 3] }], vec![]);
        // no report line (rules-56 §2): the naming window on the cradle
        assert_eq!(g.scene_song(), Some(0x16));
        assert!(matches!(g.popups.front(), Some(Popup::Choice { menu: Some(Menu::Naming(7)), .. })));
        shot(&g, &assets, "child_naming");
        // a year-end window waits behind the cradle; a typed letter keeps the cradle in front (playtest report)
        g.popups.push_back(Popup::Report(vec!["Jahresende".into()]));
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        g.update(&mut ctx, &Input { events: vec![Event::Key(Key::Char(b'X')), Event::Key(Key::Backspace)], ..Default::default() }, 0);
        assert!(matches!(g.popups.front(), Some(Popup::Choice { menu: Some(Menu::Naming(7)), .. })), "the cradle went behind the waiting window");
        let input = Input { events: vec![Event::Key(Key::Char(b'H')), Event::Key(Key::Char(b'a')), Event::Key(Key::Char(b'n')), Event::Key(Key::Char(b's')), Event::Key(Key::Backspace), Event::Key(Key::Char(b'z')), Event::Key(Key::Enter)], ..Default::default() };
        g.update(&mut ctx, &input, 0);
        assert_eq!(g.st.name_str(7), "Hanz");
        // the cradle stays while the second cry plays
        assert!(g.cry_ms > 0 && !g.popups.is_empty());
        g.update(&mut ctx, &Input::default(), 5000);
        assert!(matches!(g.popups.front(), Some(Popup::Report(_))) && g.popups.len() == 1);
    }

    /// rules-42 §1, rules-43 §2: the Grafik screen's scrolling row is the text speed with the EXE's six labels.
    #[test]
    fn real_text_speed_labels() {
        let Some(assets) = test_assets() else { return };
        let labels = assets.name_table((0xB6E3C, 6));
        assert_eq!(labels.len(), 6);
        assert!(labels[2].contains("Normal") && labels[5].starts_with("T"), "{labels:?}");
    }

    /// rules-44 §1.2: volumes step by 8 between 0 and the top, the speech toggle is option bit 0x80, and leaving a
    /// Musik change writes the bytes to CONFIG.DAT.
    #[test]
    fn real_music_options_stored() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        g.vol = [127, 5, 250];
        g.act(Action::Opt(Opt::Vol(0, 1), Menu::Music), &mut ctx);
        g.act(Action::Opt(Opt::Vol(1, -1), Menu::Music), &mut ctx);
        g.act(Action::Opt(Opt::Vol(2, 1), Menu::Music), &mut ctx);
        assert_eq!(g.vol, [127, 0, 255]);
        g.act(Action::Opt(Opt::Vol(1, 1), Menu::Music), &mut ctx);
        assert_eq!(g.vol[1], 8);
        let on = g.opts & 0x80;
        g.act(Action::Opt(Opt::Voice, Menu::Music), &mut ctx);
        assert_eq!(g.opts & 0x80, on ^ 0x80);
        let raw = std::fs::read(assets.user_saves.join("CONFIG.DAT")).unwrap();
        assert_eq!((raw[0x194], raw[0x195], raw[0x278], raw[0x17D] & 0x80), (127, 8, 255, (on ^ 0x80) as u8));
        let _ = std::fs::remove_dir_all(&assets.user_saves);
    }

    /// rules-42 §1: a long report scrolls (no paging): a click lets the next window of lines in, scrolled up 8 px a step;
    /// clicks during the scroll do nothing; the last click closes it.
    #[test]
    fn real_report_scrolls() {
        use crate::input::Input;
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        g.popups.push_back(Popup::Report((0..30).map(|i| format!("Zeile {i} des Berichts.")).collect()));
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        let click = Input { events: vec![Event::Left(300, 300)], ..Default::default() };
        g.update(&mut ctx, &Input::default(), 10);
        assert_eq!((g.scroll.s, g.scroll.target), (0, 0));
        g.update(&mut ctx, &click, 0);
        assert!(g.scroll.target > 0 && g.scroll.s == 0);
        g.update(&mut ctx, &Input::default(), 3 * SCROLL_MS);
        assert_eq!(g.scroll.s, 3 * SCROLL_STEPS[g.text_speed as usize]);
        shot(&g, &assets, "report_scrolling");
        g.update(&mut ctx, &click, 0);
        assert_eq!(g.scroll.s, 3 * SCROLL_STEPS[g.text_speed as usize], "a click while scrolling is ignored");
        let mut n = 0;
        while !g.popups.is_empty() {
            g.update(&mut ctx, &Input::default(), 10_000);
            g.update(&mut ctx, &click, 0);
            n += 1;
            assert!(n < 10);
        }
        assert_eq!(n, 2, "30 lines: three views (11, 23, 30 lines), then close");
    }

    /// Exit events carry the player's name (the person is cleared by then) and use the EXE texts.
    #[test]
    fn real_exit_lines() {
        let Some(assets) = test_assets() else { return };
        let g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        let line = |e: &SimEvent| g.describe(&assets, e).unwrap().replace(HL, "");
        let line_none = |e: &SimEvent| g.describe(&assets, e).is_none();
        let cards = line(&SimEvent::Cards { mine: 18, theirs: 20, x: 5 });
        assert!(cards.contains("18") && cards.contains("20") && cards.contains("verloren"), "{cards}");
        let sg = line(&SimEvent::SiegeGoesOn { city: 3, besieger: g.me().unwrap() as u8 });
        assert!(sg.contains("Belagerung") && !sg.contains('%'), "{sg}");
        let me = g.me().unwrap() as u8;
        let ill = line(&SimEvent::Fate { person: me, kind: 19, amount: 0, city: 0xFF, aux: [3, 0xFF], with: 0xFF });
        assert!(ill.contains("Krankheit") && !ill.contains('%'), "{ill}");
        let sur = line(&SimEvent::BranchSurcharge { emperor: 2, percent: 14, amount: 9000 });
        assert!(sur.contains("Ferdinand II.") && sur.contains("14%") && sur.contains("9.000"), "{sur}");
        let bl = line(&SimEvent::BlackmailEnded { by: me });
        assert!(bl.contains("erpressen") && !bl.contains("%1"), "{bl}");
        // a markup line (`RAW`, then the lit mark): back to text
        let won = plain(&line(&SimEvent::SiegeWon { besieger: me, city: 3, choice: 2, tribute: (1500, 4) }).chars().skip(2).map(|c| c as u8).collect::<Vec<_>>());
        assert!(won.contains("Lösegeld") && won.contains("3.000") && won.contains("1.500") && won.contains("aufgelöst") && !won.contains('%'), "{won}");
        let lost = line(&SimEvent::TransporterLost { owner: me, kind: 1, dest: 3 });
        assert!(lost.contains("Riff") && !lost.contains('%'), "{lost}");
        let off = line(&SimEvent::OffCourse { owner: me, kind: 2, old: 1, new: 4 });
        assert!(off.contains("Winde") && !off.contains('%'), "{off}");
        let doc = line(&SimEvent::Health { level: 11 });
        assert!(doc.contains("Testament") && doc.contains("Arzt") && !doc.contains('%'), "{doc}");
        let built = line(&SimEvent::ResidenceOrdered { branch: 0, cost: 900, kind: 0, word: 0x5003, years: 2 });
        assert!(built.contains("errichtet") && !built.contains('%'), "{built}");
        let sb = line(&SimEvent::SeizedBranch { prop: 0, city: 2, redirect: Some(4) });
        assert!(sb.contains("Niederlassung") && sb.contains("zurückkehren") && !sb.contains('%'), "{sb}");
        let sr = line(&SimEvent::SeizedResidence { city: 2, word: 0x5003, site: true });
        assert!(sr.contains("Baustelle") && !sr.contains('%'), "{sr}");
        let arr = line(&SimEvent::SiegeArrives { city: 3, besieger: me, army: 1234 });
        assert!(arr.contains("1.234") && arr.contains("Toren") && !arr.contains('%'), "{arr}");
        let spy = line(&SimEvent::SpyNotified { owner: me, target: me, film: false });
        assert!(spy.contains("Beweise") && !spy.contains('%'), "{spy}");
        let rl = line(&SimEvent::Relief { city: 3, ok: true, besieger: g.me().unwrap() as u8, owner: g.me().unwrap() as u8, pop: 4000, song: 7 });
        assert!(rl.contains("Aufeinandertreffen") && rl.contains("4.000") && !rl.contains('%'), "{rl}");
        let tor = line(&SimEvent::Tornado { a: 2, b: 5, n: 70, churches: [7, 4], museums: [6, 3], hospitals: [5, 2] });
        assert!(!tor.contains('%') && tor.contains("36") && tor.len() > 60, "{tor}");
        for (k, i) in [(45u8, 0u8), (45, 1), (45, 2), (38, 0), (38, 1), (38, 2), (37, 0), (37, 1), (37, 2), (46, 0), (46, 1), (46, 2), (46, 3), (46, 4), (34, 0), (34, 1), (34, 2), (34, 3), (33, 0), (33, 1), (33, 2), (49, 0), (49, 1), (49, 2), (49, 3), (40, 0), (44, 0), (44, 1), (44, 2), (44, 3), (32, 0), (39, 0), (47, 0), (47, 1), (48, 0), (48, 1), (48, 2), (48, 3)] {
            let n = line(&SimEvent::PlotNote { kind: k, id: i, a: 3, b: 2 | 300 << 8, c: 1 });
            assert!(!n.contains('%') && n.len() > 15, "{k}/{i}: {n}");
        }
        // answer reactions and follow-ups (rules-13 §2, §3.1): each id has its own text, none falls back to Debug
        let mut seen = std::collections::HashSet::new();
        for (k, i) in [
            (4u8, 0u8), (4, 1), (32, 1), (32, 2), (32, 3), (32, 4), (32, 5), (32, 6), (32, 7), (33, 3), (33, 4), (33, 5), (33, 6),
            (34, 4), (34, 5), (34, 6), (34, 7), (36, 3), (36, 4), (36, 5), (37, 3), (37, 4), (38, 3), (38, 4), (38, 5), (38, 6), (38, 7),
            (38, 8), (38, 9), (38, 10), (38, 11), (38, 12), (39, 1), (39, 2), (39, 3), (40, 1), (40, 2), (40, 3), (40, 4), (42, 0), (42, 1),
            (42, 2), (42, 3), (42, 4), (42, 5), (42, 6), (43, 0), (43, 1), (43, 2), (43, 3), (44, 4), (44, 5), (45, 3), (45, 4), (45, 5), (46, 5),
            (46, 6), (46, 7), (46, 8), (47, 2), (47, 3), (48, 4), (49, 4), (49, 5), (49, 6), (49, 7),
        ] {
            let n = line(&SimEvent::PlotNote { kind: k, id: i, a: 3, b: 2, c: 1 });
            assert!(!n.contains('%') && !n.contains("PlotNote") && n.len() > 15, "{k}/{i}: {n}");
            assert!(seen.insert(n.clone()), "{k}/{i} repeats: {n}");
        }
        let deal = line(&SimEvent::PlotNote { kind: 43, id: 2, a: 3, b: 2, c: 0 });
        assert!(deal.contains("2 weitere") && deal.contains("zufrieden"), "{deal}");
        let built = line(&SimEvent::PlotNote { kind: 32, id: 7, a: 3, b: 2, c: 0 });
        assert!(built.contains("3 Flugmaschinen. Eine Flugmaschine"), "{built}");
        assert!(line(&SimEvent::SubPlot { person: 0, kind: 41, step: 0xFF }).contains("Grabung"));
        assert!(line_none(&SimEvent::SubPlot { person: 0, kind: 41, step: 3 }));
        assert!(line(&SimEvent::PlayerOut { person: 0, name: "Anna".into(), no_kids: true }).contains("keine Kinder"));
        assert!(line(&SimEvent::PlayerOut { person: 0, name: "Anna".into(), no_kids: false }).contains("Erzbistum"));
        assert!(line_none(&SimEvent::AllPlayersOut));
        let h = g.me().unwrap() as u8;
        assert!(line(&SimEvent::Tip { kind: 1, a: 0, b: 0, loc: 0 }).contains("Wohnsitz"));
        assert!(line(&SimEvent::Tip { kind: 4, a: 2, b: 0, loc: 3 }).contains("Füßen"));
        let f = line(&SimEvent::Fate { person: 0, kind: 9, amount: 321, city: 0xFF, aux: [0xFF; 2], with: 0xFF });
        assert!(f.contains("321") && !f.contains('%'), "{f}");
        let f = line(&SimEvent::Fate { person: 0, kind: 31, amount: 0, city: 3, aux: [0xFF; 2], with: 0xFF });
        assert!(f.contains("Dach") && !f.contains('%'), "{f}");
        let f = line(&SimEvent::Fate { person: 0, kind: 45, amount: 0, city: 0xFF, aux: [2, 3], with: 0xFF });
        assert!(!f.contains('%') && f.contains('>'), "{f}");
        for (k, a) in [(0, [2, 0]), (1, [2, 0]), (2, [3, 0]), (3, [3, 0]), (4, [3, 5]), (4, [3, 0x85])] {
            let w = line(&SimEvent::WorldFate { kind: k, aux: a });
            assert!(!w.contains('%') && w.len() > 20, "{w}");
        }
        assert_eq!(g.wform(&assets, 1, 5 | (100 << 8) | 0x20), "Eure prächtige Villa mit Park");
        for k in [85, 86, 94, 95] {
            let f = line(&SimEvent::Fate { person: 0, kind: k, amount: 0, city: 15, aux: [0xFF; 2], with: g.me().unwrap() as u8 });
            assert!(!f.contains('%') && f.len() > 20, "{k}: {f}");
        }
        for k in [65, 72, 73, 77] {
            let f = line(&SimEvent::Fate { person: 0, kind: k, amount: 90, city: 0xFF, aux: [3, 2], with: 0xFF });
            assert!(!f.contains('%') && f.len() > 20, "{k}: {f}");
        }
        for k in [21, 50, 54] {
            let f = line(&SimEvent::Fate { person: 0, kind: k, amount: 90, city: 4, aux: [1, 0], with: 0xFF });
            assert!(!f.contains('%') && f.len() > 20, "{k}: {f}");
        }
        for k in [5, 25, 35] {
            let f = line(&SimEvent::Fate { person: 0, kind: k, amount: 40, city: 3, aux: [4, 1], with: 0xFF });
            assert!(!f.contains('%') && f.len() > 20, "{k}: {f}");
        }
        let f = line(&SimEvent::Fate { person: 0, kind: 64, amount: g.me().unwrap() as i32, city: 3, aux: [2, 0], with: 1 });
        assert!(!f.contains('%') && f.contains("durch"), "{f}");
        for k in [82, 87, 98, 99, 17, 18] {
            let f = line(&SimEvent::Fate { person: 0, kind: k, amount: 9, city: 15, aux: [1, 2], with: 0xFF });
            assert!(!f.contains('%') && f.len() > 20, "{k}: {f}");
        }
        for k in [56, 61, 62, 63, 66, 67, 68] {
            let f = line(&SimEvent::Fate { person: 0, kind: k, amount: 4, city: 2, aux: [3, 0], with: 1 });
            assert!(!f.contains('%') && f.len() > 20, "{k}: {f}");
        }
        assert_eq!(g.transporters(&assets, 4, 3, 0), "Eure 3 Karren");
        let c = (0..mad_sim::state::PERSONS).find(|&p| g.st.persons[p].used() && !g.st.persons[p].human() && g.st.persons[p].idx() != 15).unwrap() as u8;
        for k in [0, 1, 2, 4, 5, 6, 7, 9, 10] {
            let d = line(&SimEvent::Divine { kind: k, city: 3, with: 0xFF, illness: 2 });
            assert!(!d.contains('%') && d.len() > 15, "{k}: {d}");
        }
        let d = line(&SimEvent::Divine { kind: 3, city: 3, with: c, illness: 0 });
        assert!(d.contains("Umnachtung") && !d.contains('%'), "{d}");
        for k in [4, 24, 32, 47, 51, 57, 75] {
            let f = line(&SimEvent::Fate { person: 0, kind: k, amount: 50, city: 3, aux: [0xFF; 2], with: g.me().unwrap() as u8 });
            assert!(!f.contains('%') && f.len() > 20, "{k}: {f}");
        }
        let c1 = line(&SimEvent::Contest { discipline: 0, good: 3, fee: 40, winners: vec![c, g.me().unwrap() as u8], city: 2, judge: c, first: true, last: true });
        assert!(c1.contains("Platz") && c1.contains("160") && !c1.contains('%'), "{c1}");
        assert!(line(&SimEvent::ContestStart { city: 0, judge: 0, cancelled: true }).contains("abgesagt"));
        assert!(line(&SimEvent::ContestStart { city: 2, judge: g.me().unwrap() as u8, cancelled: false }).contains("Wertungsrichter"));
        let c2 = line(&SimEvent::Contest { discipline: 1, good: 3, fee: 40, winners: vec![], city: 2, judge: c, first: false, last: false });
        assert!(c2.contains("nicht ausreichend"), "{c2}");
        let m = line(&SimEvent::MistressDemand { person: g.me().unwrap() as u8, amount: 55 });
        assert!(m.contains("55") && (m.contains("tresse") || m.contains("Geliebter")) && !m.contains('%'), "{m}");
        let gs = line(&SimEvent::Gossip { person: c, text: 1 });
        assert!(gs.contains("Geliebten") && !gs.contains('%'), "{gs}");
        let sab = line(&SimEvent::Sabotage { saboteur: c, target: h, kind: 2, loc: 14, damage: 777, film: false, rival: false });
        assert!(sab.contains("Saboteure") && sab.contains("777") && !sab.contains('%'), "{sab}");
        let sab = line(&SimEvent::Sabotage { saboteur: c, target: h, kind: 0, loc: 3, damage: 777, film: false, rival: true });
        assert!(sab.contains("Gegenspieler, gelingt") && !sab.contains('%'), "{sab}");
        for (sentence, word) in [(0u8, "7 Stunden"), (5, "Amt")] {
            let v = line(&SimEvent::Verdict { defendant: c, guilty: true, sentence, fine: 0, office: (3, 6) });
            assert!(v.contains(word) && !v.contains('%'), "{v}");
        }
        let ins = line(&SimEvent::Insulted { insulter: h, target: c, rolls: [3, 1, 2, 5] });
        assert!(ins.contains("Satisfaktion") && ins.contains("Markt") && !ins.contains('%'), "{ins}");
        let gr = line(&SimEvent::GiftReaction { target: c, word: 10 });
        assert!(gr.contains("ernst meint") && !gr.contains('%'), "{gr}");
        for (taken, joined) in [(9, 4), (5, 0), (2, 1)] {
            let castle = (0..mad_sim::state::PROPS).find(|&p| g.st.props[p].kind() == mad_sim::state::KIND_CASTLE).unwrap() as u8;
            let pl = line(&SimEvent::Prisoners { person: h, prop: castle, taken, joined });
            assert!(!pl.contains('%') && pl.contains("Burg"), "{pl}");
        }
        let el = line(&SimEvent::Election { loc: 13, seat: 8, candidates: vec![c], votes: vec![], winner: c, kind: 2, rivals: vec![0xFF], privileged: true, religion: 0 });
        assert!(el.contains("Privilegs") && !el.contains('%'), "{el}");
        let sab = line(&SimEvent::Sabotage { saboteur: c, target: h, kind: 0, loc: 3, damage: 777, film: false, rival: false });
        assert!(sab.contains("Saboteuren gelingt") && !sab.contains('%'), "{sab}");
        let toll = line(&SimEvent::CastleIncome { castle: 0, amount: 600, gross: 1000, share: 250, vogt: 150, vogt_by: c });
        assert!(toll.contains("1.000") && toll.contains("250") && toll.contains("150") && !toll.contains('%'), "{toll}");
        let due = line(&SimEvent::LoanDue { lender: c, amount: 4321 });
        assert!(due.contains("4.321") && (due.contains("seinen") || due.contains("ihren")) && !due.contains('%'), "{due}");
        // converted lines: EXE texts, no `%` and no Debug text (rules-12 §3.7, rules-05, rules-04b §3.6)
        let el = line(&SimEvent::Election { loc: 3, seat: 6, candidates: vec![c, h], votes: vec![(c, h)], winner: h, kind: 0, rivals: vec![h, 0xFF], privileged: false, religion: 0 });
        assert!(el.contains("stimmt für...") && el.contains("Gegenspieler") && !el.contains('%') && !el.contains('{'), "{el}");
        for e in [
            SimEvent::Fine { person: h, amount: 50 },
            SimEvent::ChildDied { parent: h, age: 3, name: "Hans".into(), female: false },
            SimEvent::CourtshipFailed { person: h, target: c, reason: 1 },
            SimEvent::OfficeIncome { amount: 5 },
        ] {
            let l = line(&e);
            assert!(!l.contains('%') && !l.contains('{') && l.len() > 15, "{l}");
        }
        // shown elsewhere or not at all (rules-23 §1, rules-27 §2, rules-12 §3.9, rules-16)
        for e in [
            SimEvent::Arrived { branch: 0, good: 1, qty: 5 },
            SimEvent::Produced { branch: 0, good: 1, amount: 5 },
            SimEvent::Title { person: c, title: 2, city: 1, song: 0, awarder: 0xFF },
            SimEvent::Wedding { person: c, spouse: h, title: None, feast: false },
            SimEvent::Certificate { person: h, level: 1 },
            SimEvent::Fate { person: h, kind: 14, amount: 0, city: 3, aux: [0; 2], with: 0xFF },
            SimEvent::Fate { person: h, kind: 41, amount: 0, city: 3, aux: [0; 2], with: 0xFF },
        ] {
            assert!(line_none(&e), "{e:?}");
        }
        // plain spaces inside person references: the original breaks lines there (live A/B)
        assert!(g.pref(c, 5, 5).contains(" in ") && !g.pref(c, 5, 5).contains('\u{a0}'), "{}", g.pref(c, 1, 5));
    }

    /// Aid: lists the report lines that still fall back to Rust debug text over a few years.
    #[test]
    #[ignore]
    fn real_debug_fallbacks() {
        let Some(assets) = test_assets() else { return };
        let mut seen = std::collections::BTreeSet::new();
        for seed in 1..6 {
            let mut st = State::new_game(seed, 3, &[mad_sim::world::HumanSetup::test(4), mad_sim::world::HumanSetup::test(5)]).unwrap();
            st.szenario = assets.read("SZENARIO.DAT").unwrap();
            let mut g = Game::start(&assets, st);
            let mut audio = crate::audio::Audio::new(false);
            let mut ctx = ctx_for(&assets, &mut audio);
            for _ in 0..40 {
                g.popups.clear();
                g.act(Action::EndTurn, &mut ctx);
                while g.pending.is_some() {
                    g.popups.clear();
                    g.act(Action::Answer(0), &mut ctx);
                }
                if g.me().is_none() {
                    break;
                }
                for p in &g.popups {
                    if let Popup::Report(l) | Popup::YearReport(l) = p {
                        for line in l {
                            if line.contains(" { ") || line.contains("(Frage") || line.starts_with("Schicksal") {
                                seen.insert(if line.starts_with("Schicksal") || line.starts_with("Fate") { line.clone() } else { line.split_whitespace().take(2).collect::<Vec<_>>().join(" ") });
                            }
                        }
                    }
                }
            }
            eprintln!("seed {seed}: year {}", g.st.year);
        }
        eprintln!("{seen:#?}");
    }

    /// The contest entry toggles one discipline per click and answers the mask.
    #[test]
    fn real_contest_entry_toggle() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        let h = g.me().unwrap() as u8;
        g.pending = Some(Pending { cmd: None, answers: vec![], ask: Ask::ContestEntry { person: h, fee: 40, goods: [0, 4, 7, 9, 12], paper: 1 } });
        g.amount = 0;
        g.open(&assets, Menu::Question);
        let Some(Popup::Choice { acts, .. }) = g.popups.back() else { panic!("no dialog") };
        assert_eq!(acts.len(), 6);
        g.step(&assets, Field::Mask, 1 << 2);
        g.step(&assets, Field::Mask, 1 << 4);
        g.step(&assets, Field::Mask, 1 << 2);
        assert_eq!(g.amount, 1 << 4);
    }

    /// The lender's offer waits for the player (it used to be accepted automatically).
    #[test]
    fn real_loan_offer_waits() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        let h = g.me().unwrap();
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        let loans = |g: &Game| g.st.heap.all(56, h as u8, mad_sim::heap::ANY).len();
        for _ in 0..8 {
            g.act(Action::Sim(Command::AskLender, None), &mut ctx);
            if g.pending.is_some() {
                break;
            }
        }
        if let Some(Pending { ask: Ask::LoanOffer { .. }, .. }) = &g.pending {
            assert_eq!(loans(&g), 0, "nothing is signed before the answer");
            g.act(Action::Answer(1), &mut ctx);
            assert_eq!(loans(&g), 0, "declined");
        }
    }

    /// A report is drawn on the desk background with the "Ereignisse" banner (screenshot with MAD_SHOT_DIR).
    #[test]
    fn real_report_window() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        g.popups.push_back(Popup::Report(vec!["Ihr zahlt Eure jährliche Geldstrafe von 120 Talern.".into(), "Euer Sohn Hans ist verstorben.".into()]));
        shot(&g, &assets, "report_window");
        g.popups.clear();
        g.report(&assets, vec![SimEvent::Bill { wages: 0, troops: 0, robbers: 0, means: 0, transport: 0, toll: 0, directors: 0, tax: 0, tithe: 0, spies: 0, saboteurs: 0, interest: 0, insurance: 0, cash: 0 }], vec![]);
        shot(&g, &assets, "report_nothing");
        g.popups.clear();
        let mut body = assets.text(0x89AB8);
        body.truncate(body.len() - 4);
        let head = assets.text(0xBA880)[4..].to_vec();
        g.popups.push_back(Popup::Chronicle([head, b"$C".to_vec(), body].concat()));
        shot(&g, &assets, "chronicle1");
        assert!(g.scroll_on(&assets), "the 1600 chronicle overflows the window");
        g.scroll.s = g.scroll.target;
        assert!(g.scroll.s > 0);
        shot(&g, &assets, "chronicle2");
        // rules-44 §3: the next-year list, plural for the seat groups, singular otherwise
        let src = g.next_elections(&assets, &[(13, 8), (4, 7), (4, 5), (4, 0)]);
        let txt = plain(&src);
        assert!(txt.contains("neu gew") && txt.contains("Priester in") && txt.contains("Ratsherren in") && txt.contains("Der "), "{txt}");
        g.popups.clear();
        g.popups.push_back(Popup::Chronicle(src));
        shot(&g, &assets, "next_elections");
        g.popups.clear();
        g.report(&assets, vec![SimEvent::Herald { song: 0x0E }], vec![]);
        shot(&g, &assets, "herald");
        assert_eq!(g.scene_song(), Some(0x0E));
        g.herald_song = None;
        let me = g.me().unwrap() as u8;
        for (title, name) in [(1, "award1"), (2, "award2"), (5, "award5")] {
            g.popups.clear();
            g.report(&assets, vec![SimEvent::Title { person: me, title, city: 3, song: 7, awarder: 0xFF }], vec![]);
            assert!(matches!(g.popups.front(), Some(Popup::Award(_))), "{:?}", g.popups.iter().map(|p| match p { Popup::Report(l) => l.join("|"), Popup::Bill(..) => "bill".into(), Popup::Award(_) => "award".into(), _ => "other".into() }).collect::<Vec<_>>());
            shot(&g, &assets, name);
        }
        g.popups.clear();
        g.popups.clear();
        g.report(&assets, vec![SimEvent::YearEnd { year: 1601 }], vec![]);
        shot(&g, &assets, "year_nothing");
        g.popups.clear();
        g.report(&assets, vec![SimEvent::Bill { wages: 281, troops: 0, robbers: 0, means: 412, transport: 0, toll: 0, directors: 0, tax: 0, tithe: 0, spies: 0, saboteurs: 0, interest: 0, insurance: 0, cash: 0 }], vec![]);
        assert!(matches!(g.popups.front(), Some(Popup::Bill(..))));
        shot(&g, &assets, "bill_page");
        g.popups.clear();
        g.popups.push_back(Popup::YearReport(vec!["Im Frieden von Karlowitz findet der große Türkenkrieg sein Ende.".into()]));
        shot(&g, &assets, "year_report_window");
        assert!(matches!(g.popups.front(), Some(Popup::YearReport(_))));
        // rules-45 §1: the denunciation, actor and victim highlighted (live A/B: red on the parchment)
        let me = g.me().unwrap() as u8;
        let cs: Vec<u8> = (0..mad_sim::state::PERSONS as u8).filter(|&p| g.st.persons[p as usize].used() && !g.st.persons[p as usize].human() && g.st.persons[p as usize].has_office()).take(2).collect();
        let den = g.describe(&assets, &SimEvent::WorldText { text: 0xBFCFC, args: vec![mad_sim::WArg::P(cs[0]), mad_sim::WArg::P(me), mad_sim::WArg::P(cs[1])] }).unwrap();
        assert!(den.starts_with(HL) && den.matches(HL).count() == 2 && den.contains("schw"), "{den:?}");
        g.popups.clear();
        g.popups.push_back(Popup::YearReport(vec![den]));
        shot(&g, &assets, "year_denounce");
    }

    /// The concert day is a picture with three caption boxes, each closed by a click.
    #[test]
    fn real_concert_scene() {
        use crate::input::Input;
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        let c = (0..mad_sim::state::PERSONS).find(|&p| g.st.persons[p].used() && !g.st.persons[p].human()).unwrap() as i32;
        g.report(&assets, vec![SimEvent::PlotNote { kind: 36, id: 2, a: c, b: 300, c: 0 }], vec![]);
        assert!(matches!(g.popups.front(), Some(Popup::Scene { pic: 0xBB, boxes }) if boxes.len() == 3));
        shot(&g, &assets, "concert_scene");
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        for left in [2, 1] {
            g.update(&mut ctx, &Input { events: vec![Event::Left(10, 10)], ..Default::default() }, 0);
            assert!(matches!(g.popups.front(), Some(Popup::Scene { boxes, .. }) if boxes.len() == left));
        }
        g.update(&mut ctx, &Input { events: vec![Event::Left(10, 10)], ..Default::default() }, 0);
        assert!(g.popups.is_empty());
    }

    /// rules-15 §3.4 step 4: a human elector votes on the election screen (playtest report: "this should be in one
    /// screen"): the computer vote before him plays, his prompt waits for a candidate line, and the finished election
    /// goes on in the same screen after his vote.
    #[test]
    fn real_election_human_vote_in_screen() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        let h = g.me().unwrap() as u8;
        let mut comp = (0..mad_sim::state::PERSONS).filter(|&p| g.st.persons[p].used() && !g.st.persons[p].human()).map(|p| p as u8);
        let (c, e) = (comp.next().unwrap(), comp.next().unwrap());
        let cands = [c, h];
        let mut el = g.election_screen(&assets, (3, 6), 0xC1, &cands, &[0xFF, 0xFF], false, &[(e, c)], None, Some((h, 0xFF)));
        assert_eq!(el.steps.iter().map(|st| (st.coin, st.timed, st.ask)).collect::<Vec<_>>(), vec![(None, false, false), (None, true, false), (Some(0), true, false), (None, false, true)]);
        el.pos = 3;
        assert_eq!(el.line_at((200, 150 + 24)), Some(1), "the second candidate's line");
        let done = g.election_screen(&assets, (3, 6), 0xC1, &cands, &[0xFF, 0xFF], false, &[(e, c), (h, h)], Some((h, 1)), None);
        el.continue_with(done);
        assert_eq!(el.pos, 4, "past the human's prompt, on his vote");
        assert_eq!((el.steps[4].coin, el.coins()), (Some(1), vec![1, 1]));
    }

    /// Elections and dismissals are place pictures with caption boxes (screenshot with MAD_SHOT_DIR).
    #[test]
    fn real_office_scenes() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        let h = g.me().unwrap() as u8;
        let c = (0..mad_sim::state::PERSONS).find(|&p| g.st.persons[p].used() && !g.st.persons[p].human() && g.st.persons[p].idx() != 15).unwrap() as u8;
        g.report(&assets, vec![SimEvent::Election { loc: 3, seat: 6, candidates: vec![c, h], votes: vec![(c, h)], winner: h, kind: 0, rivals: vec![h, 0xFF], privileged: false, religion: 1 }], vec![]);
        // rules-15 §3.4: the candidate lines, then each computer vote in two timed strip lines (the coin with the
        // second), then the result waiting for a click
        let Some(Popup::Election(el)) = g.popups.front() else { panic!("the election screen") };
        assert_eq!((el.pic, el.names.len(), el.steps.len()), (0xC1, 2, 4));
        assert_eq!(el.steps.iter().map(|st| (st.coin, st.timed)).collect::<Vec<_>>(), vec![(None, false), (None, true), (Some(1), true), (None, false)]);
        shot(&g, &assets, "election_scene");
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        // the candidate lines wait for a click, then the votes run timed
        g.update(&mut ctx, &Input::default(), 10 * Elect::TIMEOUT_MS);
        assert!(matches!(g.popups.front(), Some(Popup::Election(el)) if el.pos == 0));
        g.update(&mut ctx, &Input { events: vec![Event::Left(10, 10)], ..Default::default() }, 0);
        g.update(&mut ctx, &Input::default(), Elect::TIMEOUT_MS);
        assert!(matches!(g.popups.front(), Some(Popup::Election(el)) if el.pos == 2 && el.coins() == vec![0, 1]));
        shot(&g, &assets, "election_vote");
        g.update(&mut ctx, &Input::default(), Elect::TIMEOUT_MS);
        g.update(&mut ctx, &Input::default(), 10 * Elect::TIMEOUT_MS);
        assert!(matches!(g.popups.front(), Some(Popup::Election(el)) if el.pos == 3), "the result waits for a click");
        g.update(&mut ctx, &Input { events: vec![Event::Left(10, 10)], ..Default::default() }, 0);
        assert!(g.popups.is_empty());
        g.popups.clear();
        g.report(&assets, vec![SimEvent::Dismissed { person: h, by: vec![c], loc: 12, idx: 3, rival: 0xFF }], vec![]);
        assert!(matches!(g.popups.front(), Some(Popup::Scene { pic: 0x19F, .. })));
        shot(&g, &assets, "dismissal_scene");
    }

    /// Death, bankruptcy, jury and wedding are picture scenes (screenshots with MAD_SHOT_DIR).
    #[test]
    fn real_event_scenes() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        let h = g.me().unwrap() as u8;
        let c = (0..mad_sim::state::PERSONS).find(|&p| g.st.persons[p].used() && !g.st.persons[p].human()).unwrap() as u8;
        g.report(&assets, vec![SimEvent::Died { person: h, name: "Anna".into(), female: true, nw: 12345, epitaph: 3, heckles: vec![[1, 2, 0, 15, 1], [5, 0, 4, 1, 0]] }], vec![]);
        assert!(matches!(g.popups.pop_front(), Some(Popup::Film(_))), "the candle film comes first");
        assert!(matches!(g.popups.front(), Some(Popup::Scene { pic: 0xC7, .. })));
        shot(&g, &assets, "scene_gravestone");
        g.popups.pop_front();
        // five notary lines and the one heckle drawn after step 1 (centred, "Endlich!" in voice F, female)
        assert!(matches!(g.popups.front(), Some(Popup::Scene { pic: 0xC8, boxes }) if boxes.len() == 6 && boxes[2].0[1] == 451 && speech_of(&boxes[2].1).as_deref() == Some("F123000")));
        shot(&g, &assets, "scene_notary");
        // the notary's lines take no clicks and close by themselves (rules-17 §4.1)
        {
            let mut audio = crate::audio::Audio::new(false);
            let mut ctx = ctx_for(&assets, &mut audio);
            g.update(&mut ctx, &crate::input::Input { events: vec![Event::Left(320, 240)], ..Default::default() }, 10);
            assert!(matches!(g.popups.front(), Some(Popup::Scene { boxes, .. }) if boxes.len() == 6));
            g.update(&mut ctx, &crate::input::Input::default(), 60_000);
            assert!(matches!(g.popups.front(), Some(Popup::Scene { boxes, .. }) if boxes.len() == 5));
        }
        assert_eq!((hold_of(b"$12S$4000T$CText"), hold_of(b"$CText")), (Some(4000), None));
        // the testament heckle (step 5) follows the notary's last words, from the second table
        g.popups.clear();
        g.report(&assets, vec![SimEvent::Died { person: h, name: "Anna".into(), female: true, nw: 1, epitaph: 3, heckles: vec![[5, 0, 4, 1, 0]] }, SimEvent::Succession { person: h, kids: 2 }], vec![]);
        while !matches!(g.popups.front(), Some(Popup::Will(_))) {
            g.popups.pop_front().expect("the testament window");
        }
        shot(&g, &assets, "scene_will");
        let Some(Popup::Scene { boxes, .. }) = g.popups.back() else { panic!() };
        assert_eq!((boxes.len(), speech_of(&boxes[1].1).as_deref()), (2, Some("1023018")));
        g.popups.clear();
        g.report(&assets, vec![SimEvent::Trial { plaintiff: h, defendant: c, judges: [c, c, h], kind: 0, loc: 3, votes: [true, true, false] }], vec![]);
        assert!(matches!(g.popups.pop_front(), Some(Popup::Scene { pic: 0xC1, .. })));
        assert!(matches!(g.popups.front(), Some(Popup::Court { boxes, .. }) if boxes.len() == 6));
        shot(&g, &assets, "scene_court");
        if let Some(Popup::Court { boxes, .. }) = g.popups.front_mut() {
            boxes.drain(..3);
        }
        shot(&g, &assets, "scene_court_verdict");
        g.popups.pop_front();
        assert!(matches!(g.popups.front(), Some(Popup::Court { anim: Some((100, 133)), .. })));
        g.court_ms = 1800;
        shot(&g, &assets, "scene_court_anim");
        g.popups.clear();
        for (name, ask) in [
            ("auction_stairs", Ask::AuctionLine { step: 0, pic: 0xC1, kind: 0, seller: 0, winner: 0, a: 1620, b: 3, c: 0, d: 0 }),
            ("auction_catalog", Ask::AuctionCatalog { n: 0, kind: 12, pic: 0x15A, seller: h, a: 6, b: 0, c: 75, d: 0, price: 900 }),
        ] {
            g.pending = Some(Pending { cmd: None, answers: vec![], ask });
            g.open(&assets, Menu::Question);
            shot(&g, &assets, name);
            g.popups.clear();
        }
        for (name, ms) in [("auction_rest", 0), ("auction_hammer", 300)] {
            g.pending = Some(Pending { cmd: None, answers: vec![], ask: Ask::AuctionTick { lot: 0, price: 100, high: Some(1), pic: 0x160, strip: vec![(0xD6, 1, 19)], fresh: 0, blow: false, hammer: 0 } });
            g.ask_ms = ms;
            g.open(&assets, Menu::Question);
            shot(&g, &assets, name);
            g.popups.clear();
        }
        let talk = SimEvent::TrialTalk { witnesses: vec![(0, 3, 1, 2), (1, 9, 8, 7)], plea: [6, 5, 3, 2, 1, 1] };
        let charge = SimEvent::TrialCharge { evidence: 0b101 | 1 << 12, groups: vec![(0, 1, 0), (1, 2, 3)], plea: 6 };
        let money = SimEvent::TrialMoney { invested: vec![c], bribes: vec![(h, 500), (c, 0)] };
        for (level, name) in [(1, "cert2"), (2, "cert3")] {
            g.popups.clear();
            g.report(&assets, vec![SimEvent::Certificate { person: h, level }], vec![]);
            assert!(matches!(g.popups.front(), Some(Popup::Award(_))));
            shot(&g, &assets, name);
        }
        g.popups.clear();
        g.report(&assets, vec![SimEvent::TrialCosts { plaintiff: h, defendant: c, judges: [c, c, h], payer: c, amount: 240, insured: true }], vec![]);
        assert!(matches!(g.popups.front(), Some(Popup::Court { boxes, .. }) if boxes.len() == 2));
        shot(&g, &assets, "trial_costs");
        g.popups.clear();
        g.report(&assets, vec![money, charge, talk, SimEvent::Trial { plaintiff: h, defendant: c, judges: [c, c, h], kind: 0, loc: 3, votes: [true, true, false] }], vec![]);
        g.popups.pop_front();
        // charge part, one popup per witness (talking head, rules-18 §6), the rest up to the decision and verdicts
        let courts: Vec<&Popup> = g.popups.iter().collect();
        let total: usize = courts.iter().filter_map(|p| if let Popup::Court { boxes, .. } = p { Some(boxes.len()) } else { None }).sum();
        assert_eq!(total, 5 + 1 + 2 + 1 + 4 + 1 + 3 + 2 + 2 + 1 + 3 + 3 + 1);
        assert!(matches!(courts[1], Popup::Court { figures, anim: Some((0, 22)), .. } if figures.len() == 6));
        g.popups.pop_front();
        shot(&g, &assets, "scene_court_witness_charge");
        g.popups.pop_front();
        g.court_ms = 1000;
        shot(&g, &assets, "scene_court_witness");
        g.popups.clear();
        // rules-19 §4: lines before a scene are flushed ahead of it, lines after it come last
        g.report(&assets, vec![SimEvent::Fine { person: h, amount: 120 }, SimEvent::Duel { winner: c, loser: h, office_lost: true, scene: mad_sim::DuelScene { insulter: c, place: 3, answers: [0, 7], fog: 2, after: 1, taunts: vec![(0, 5, 100, 150), (1, 62, 200, 200)], fx: vec![vec![(150, 0x83), (250, 0x39)], vec![]] } }, SimEvent::Fine { person: h, amount: 120 }], vec![]);
        assert!(matches!(&g.popups[0], Popup::Report(l) if l.len() == 1));
        assert!(matches!(g.popups[3], Popup::Scene { pic: 0xD1, .. }));
        assert!(matches!(&g.popups[5], Popup::Report(l) if l.len() == 1));
        g.popups.clear();
        g.report(&assets, vec![SimEvent::Duel { winner: c, loser: h, office_lost: true, scene: mad_sim::DuelScene { insulter: c, place: 3, answers: [0, 7], fog: 2, after: 1, taunts: vec![(0, 5, 100, 150), (1, 62, 200, 200)], fx: vec![vec![(150, 0x83), (250, 0x39)], vec![]] } }], vec![]);
        // rules-22: fight bank with six captions, fog line over the animation, taunts + after-fog on the fog picture, result bank
        assert!(matches!(g.popups.front(), Some(Popup::Court { boxes, anim: None, .. }) if boxes.len() == 6));
        shot(&g, &assets, "scene_duel_open");
        g.popups.pop_front();
        assert!(matches!(g.popups.front(), Some(Popup::Court { boxes, anim: Some((0, 104)), .. }) if boxes.len() == 1));
        g.court_ms = 2000;
        shot(&g, &assets, "scene_duel_fight");
        g.popups.pop_front();
        assert!(matches!(g.popups.front(), Some(Popup::Scene { pic: 0xD1, boxes }) if boxes.len() == 3));
        assert_eq!(g.duel_fx.len(), 2, "one effect list per taunt box");
        shot(&g, &assets, "scene_duel");
        g.popups.pop_front();
        assert!(matches!(g.popups.front(), Some(Popup::Court { anim: Some((0, 18)), boxes, .. }) if boxes.len() == 2));
        g.court_ms = 1500;
        shot(&g, &assets, "scene_duel_result");
        g.popups.clear();
        g.report(&assets, vec![SimEvent::Disaster { city: 3, kind: 1, variant: 1, lost: 321, adj: 2 }, SimEvent::DisasterBranches { person: h, goods: true }, SimEvent::DisasterInsured { person: h, loss: 4000 }], vec![]);
        assert!(matches!(g.popups.front(), Some(Popup::Scene { pic: 0xC9, boxes }) if boxes.len() == 2));
        shot(&g, &assets, "scene_flood");
        g.popups.pop_front();
        assert!(matches!(g.popups.front(), Some(Popup::Scene { boxes, .. }) if String::from_utf8_lossy(&boxes[0].1).contains("321")));
        shot(&g, &assets, "scene_flood2");
        // the plague as captured on the battle save (2026-11-16): city 3 lost 1.710
        let saved = std::mem::take(&mut g.popups);
        g.report(&assets, vec![SimEvent::Disaster { city: 3, kind: 2, variant: 0, lost: 1710, adj: 0 }], vec![]);
        shot(&g, &assets, "ab_plague1");
        g.popups.pop_front();
        shot(&g, &assets, "ab_plague2");
        g.popups = saved;
        assert_eq!(g.popups.len(), 3);
        g.popups.clear();
        g.report(&assets, vec![SimEvent::Bankrupt { person: h, name: "Anna".into() }], vec![]);
        assert!(matches!(g.popups.front(), Some(Popup::Scene { pic: 0xBA, boxes }) if boxes.len() == 4));
        g.popups.clear();
        g.report(&assets, vec![SimEvent::Jury { person: h, guilty: 9, tower: true, last_branch: false, song: 0x0B }], vec![]);
        assert!(matches!(g.popups.front(), Some(Popup::Court { figures, .. }) if figures.len() == 12));
        shot(&g, &assets, "scene_jury");
        g.popups.pop_front();
        assert!(matches!(g.popups.front(), Some(Popup::Court { figures, .. }) if figures.len() <= 12));
        g.popups.pop_front();
        assert!(matches!(g.popups.front(), Some(Popup::Scene { pic: 0xB9, .. })));
        shot(&g, &assets, "scene_tower");
        g.popups.clear();
        g.report(&assets, vec![SimEvent::Feast { host: h, guests: 40, quality: 4, loc: 3, cost: 2500, food_short: false, drink_short: true }], vec![]);
        assert!(matches!(g.popups.front(), Some(Popup::Scene { pic: 0xBB, boxes }) if boxes.len() == 5));
        shot(&g, &assets, "scene_feast");
        g.popups.clear();
        // the church scenes come with the questions (rules-15 §7.1)
        g.church_before(&assets, &Ask::CanonVeto { canon: c, suitor: h, target: c });
        assert_eq!(g.popups.len(), 2, "portal, priest");
        assert!(matches!(g.popups.front(), Some(Popup::Scene { pic: 0xC0, .. })));
        g.church_after(&assets, &Ask::CanonVeto { canon: c, suitor: h, target: c }, 1);
        g.church_before(&assets, &Ask::Vows { suitor: h, target: c, canon: Some(c) });
        g.church_before(&assets, &Ask::Vows { suitor: h, target: c, canon: Some(c) });
        assert_eq!(g.popups.len(), 4, "portal, priest, no objection, priest's question once");
        assert!(matches!(g.popups.back(), Some(Popup::Scene { boxes, .. }) if boxes.len() == 1), "a declining human canon: no 0x9D1E8 (rules-57 §3)");
        g.church_before(&assets, &Ask::WeddingFeast { suitor: h, spouse: c, city: 3, prices: [4, 3, 2, 1], title: Some(3), song: 4 });
        assert!(matches!(g.popups.back(), Some(Popup::Scene { pic: 0xFF, .. })), "title line on the banquet");
        shot(&g, &assets, "scene_wedding");
        g.popups.clear();
        g.church = 0;
        // the feast verdict comes with the answer (rules-15 §7.1 step 9), the event adds nothing later
        g.church_after(&assets, &Ask::WeddingFeast { suitor: h, spouse: c, city: 3, prices: [4, 3, 2, 1], title: Some(3), song: 4 }, 0);
        assert!(matches!(g.popups.back(), Some(Popup::Scene { pic: 0xFF, boxes }) if boxes.len() == 1 && String::from_utf8_lossy(&boxes[0].1).contains("schönes Fest") || String::from_utf8_lossy(&boxes[0].1).contains("Fest")));
        let n = g.popups.len();
        g.report(&assets, vec![SimEvent::Wedding { person: h, spouse: c, title: Some(3), feast: true }], vec![]);
        assert_eq!(g.popups.len(), n, "no second feast screen after the year report");
    }

    /// "Nein" after an accomplished mission asks "Spiel beenden?" (rules-39 §8): Ja rolls the credits, Nein plays on.
    #[test]
    fn real_mission_quit() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        let h = g.me().unwrap() as u8;
        g.report(&assets, vec![SimEvent::MissionDone { person: h, mission: 1 }, SimEvent::GameOver], vec![]);
        let Some(Popup::Choice { acts, rect, .. }) = g.popups.back() else { panic!() };
        assert_eq!((rect, acts.len()), (&[160, 160, 320, 160], 2));
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        assert!(matches!(g.act(Action::Credits, &mut ctx), Next::Go(_)));
    }

    /// The privileges menu per rules-38 §1: entries, setting items and the settings acting on the sim.
    #[test]
    fn real_privileges_menu() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        let h = g.me().unwrap();
        // Bürgermeister of city 3 (global office 6): Umsatzsteuer is a setting
        g.st.persons[h].set_loc(3);
        g.st.persons[h].set_idx(6);
        g.open(&assets, Menu::Privileges);
        shot(&g, &assets, "privileges_mayor");
        let tax = g.st.cities[3].tax();
        g.step(&assets, Field::Priv(22), 1);
        assert_eq!(g.st.cities[3].tax(), (tax + 1).min(33));
        // Kanzler of county 10 (global 16): Berater vertauschen, plus the self entries
        g.st.persons[h].set_loc(10);
        g.st.persons[h].set_idx(6);
        g.popups.clear();
        g.open(&assets, Menu::Privileges);
        shot(&g, &assets, "privileges_kanzler");
        assert!(matches!(g.popups.front(), Some(Popup::Choice { src, .. }) if !src.windows(2).any(|w| w == b"%d")));
    }

    /// rules-50 §1.1: raid captions (codes 0..3) name the shipment's two places in line 1 and leave no `%`.
    #[test]
    fn raid_intro_captions() {
        let Some(assets) = test_assets() else { return };
        let g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        let intro = mad_sim::fight::Intro { loc: 0, to: 7, attacker: 1, defender: 2, troop: 1, called_off: false };
        for code in 0..4 {
            let caps: Vec<String> = g.battle_captions(&assets, code, intro).iter().map(|c| String::from_utf8_lossy(&c.1).into_owned()).collect();
            assert_eq!(caps.len(), 4);
            assert!(caps.iter().all(|c| !c.contains('%')), "{caps:#?}");
            assert!(caps[0].contains("zwischen") && caps[0].contains("Waldstein"), "{caps:#?}");
            assert!(caps[3].contains("uber"), "troop word: {caps:#?}");
        }
    }

    /// Sim refusals a human can reach are answered the original way, never with our "Nicht möglich" line:
    /// mistress (rules-12 §5), the city info's Lager line (rules-23 §7.1, formats §11.2), a build order of 0 (rules-34 §4).
    #[test]
    fn refusals_use_exe_messages() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        let h = g.me().unwrap();
        g.st.persons[h].set_cash_raw(5_000_000);
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        let nicht = |g: &Game| g.popups.iter().any(|p| matches!(p, Popup::Report(l) if l.iter().any(|s| s.contains("Nicht möglich"))));
        let front = |g: &Game| match g.popups.front() { Some(Popup::Choice { src, .. }) => String::from_utf8_lossy(src).into_owned(), _ => String::new() };
        // the first one: the sim takes her, then box 2407 (rules-12 §3.8)
        g.st.persons[h].set_spouse(0);
        let a = g.priv_action(&assets, h, 4);
        g.act(a, &mut ctx);
        assert!(g.st.heap.has(29, h as u8));
        assert!(g.popups.iter().any(|p| matches!(p, Popup::Choice { src, .. } if String::from_utf8_lossy(src).contains("Ansehen"))));
        g.popups.clear();
        // a second mistress: box 2404 instead of the sim's refusal
        let a = g.priv_action(&assets, h, 4);
        assert!(matches!(&a, Action::Boxed(_, [160, 140, 320, 200])), "{a:?}");
        g.act(a, &mut ctx);
        assert!(front(&g).contains("wohl") && !front(&g).contains('%'), "{}", front(&g));
        g.popups.clear();
        // the only branch: line 0xBE4D4, no sell item
        let c = g.st.props[g.st.props_of(h, mad_sim::state::KIND_BRANCH).next().unwrap()].loc();
        g.open(&assets, Menu::City(c));
        assert!(front(&g).contains("einziges Lager"), "{}", front(&g));
        // with a second branch: the close item asks 0xBE58C, Ja sells
        let other = (0..10u8).find(|&x| x != c && g.st.rights_holder(x as usize).is_none()).unwrap();
        g.act(Action::Sim(Command::BuyBranch { city: other }, None), &mut ctx);
        g.popups.clear();
        g.open(&assets, Menu::City(c));
        let Some(Popup::Choice { acts, .. }) = g.popups.front() else { panic!() };
        let sell = acts[0].clone();
        g.act(sell, &mut ctx);
        assert!(front(&g).contains("wirklich"), "{}", front(&g));
        let Some(Popup::Choice { acts, .. }) = g.popups.front() else { panic!() };
        let yes = acts[0].clone();
        g.act(yes, &mut ctx);
        assert_eq!(g.st.branches(h), 1);
        // a build order of 0 transporters does nothing
        let b = g.st.props_of(h, mad_sim::state::KIND_BRANCH).next().unwrap();
        g.act(Action::Sim(Command::BuildTransporters { branch: b as u8, t: 0, n: 0 }, None), &mut ctx);
        assert!(!nicht(&g));
    }

    /// rules-56 §7: 0xBBA98 has no `$W`; the next `$P` clears it unseen.
    #[test]
    fn prefight_page_without_wait_is_dropped() {
        let Some(assets) = test_assets() else { return };
        let g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        let p = mad_sim::WArg::P(g.me().unwrap() as u8);
        let (pages, rest) = g.prefight_pages(&assets, &[(0xBBA98, vec![p.clone()]), (0xBBAE0, vec![p])]);
        let all: Vec<u8> = pages.iter().flat_map(|c| c.1.clone()).chain(rest).collect();
        let s = String::from_utf8_lossy(&all);
        assert!(!s.contains("freut sich") && s.contains("eilen"), "{s}");
    }

    /// Back-room refusals and outcomes (rules-04 §5) open the EXE boxes at the spec rects, with the person args filled.
    #[test]
    fn backroom_boxes() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        let h = g.me().unwrap();
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        let mut cs = (0..mad_sim::state::PERSONS).filter(|&i| g.st.persons[i].used() && !g.st.persons[i].human() && g.st.persons[i].has_office());
        let (v, l) = (cs.next().unwrap() as u8, cs.next().unwrap() as u8);
        fn check(g: &mut Game, ctx: &mut Ctx, a: Action, rect: [i32; 4], has: &str) {
            g.popups.clear();
            g.act(a, ctx);
            let Some(Popup::Choice { src, rect: r, .. }) = g.popups.front() else { panic!("{has}: {:?}", g.popups.len()) };
            let s = String::from_utf8_lossy(src).into_owned();
            assert!(s.contains(has) && !s.contains('%') && *r == rect, "{has}: {s} {r:?}");
            assert!(!g.popups.iter().any(|p| matches!(p, Popup::Report(l) if l.iter().any(|s| s.contains("Nicht möglich")))));
        }
        g.st.persons[h].set_cash_raw(0);
        check(&mut g, &mut ctx, Action::Sim(Command::Bribe { target: v, amount: 0 }, None), [160, 160, 320, 160], "Geldbeutel");
        g.st.persons[h].set_cash_raw(5000);
        g.popups.clear();
        g.act(Action::Sim(Command::Bribe { target: v, amount: 0 }, None), &mut ctx);
        assert!(g.popups.is_empty(), "amount 0 closes silently");
        check(&mut g, &mut ctx, Action::Blackmail(v), [160, 160, 320, 160], "zu wenig");
        g.st.spies(h, v as usize, 0);
        let c = g.st.spy_contract(h, v as usize).unwrap();
        mad_sim::heap::w32(&mut g.st.heap.recs[c], 0x0B, 0b111);
        g.st.persons[v as usize].set_cash_raw(6);
        check(&mut g, &mut ctx, Action::Blackmail(v), [160, 140, 320, 200], " Verbrechen");
        check(&mut g, &mut ctx, Action::Blackmail(v), [160, 180, 320, 120], "schon");
        g.st.persons[l as usize].set_favour(h, 0);
        check(&mut g, &mut ctx, Action::Sim(Command::Denounce { victim: v, listener: l }, None), [160, 160, 320, 160], "kein Wort");
        g.st.globals.set_b(mad_sim::state::ga::DENOUNCE + l as usize, 1);
        check(&mut g, &mut ctx, Action::Sim(Command::Denounce { victim: v, listener: l }, None), [160, 160, 320, 160], "nichts mehr");
        check(&mut g, &mut ctx, Action::Sim(Command::Denounce { victim: v, listener: l }, None), [160, 160, 320, 160], "gereizt");
        check(&mut g, &mut ctx, Action::Sim(Command::Denounce { victim: v, listener: l }, None), [160, 140, 320, 200], "Miene");
        // Klage: refused for a higher office, else filed; an open one asks 0xBCB04
        g.st.persons[h].set_idx(0);
        g.st.persons[h].set_loc(0);
        g.st.persons[v as usize].set_loc(13);
        check(&mut g, &mut ctx, Action::Sim(Command::FileTrial { target: v }, None), [160, 160, 320, 160], "lacht");
        g.st.persons[v as usize].set_loc(1);
        check(&mut g, &mut ctx, Action::Sim(Command::FileTrial { target: v }, None), [160, 180, 320, 120], "Prozess gemacht");
        g.popups.clear();
        g.open(&assets, Menu::Tree(Plot::Trial, 1, false));
        let Some(Popup::Choice { acts, .. }) = g.popups.front() else { panic!() };
        let ask = acts.iter().find(|a| matches!(a, Action::Inline(..))).cloned().expect("the open trial asks");
        g.act(ask, &mut ctx);
        let Some(Popup::Choice { acts, .. }) = g.popups.front() else { panic!() };
        let drop = acts[0].clone();
        g.act(drop, &mut ctx);
        assert!(g.st.open_trial(h, v as usize, 0).is_none());
        // inquisition (rules-56 §6): an open one asks 0xBCCD4 at (120, 140, 400, 200)
        let i = g.st.heap.alloc(18, h as u8, mad_sim::state::NONE, 1).unwrap();
        (g.st.heap.recs[i][4], g.st.heap.recs[i][5]) = (v, 1);
        assert!(g.st.open_trial(h, v as usize, 1).is_some());
        g.popups.clear();
        g.open(&assets, Menu::Tree(Plot::Priv(81), 1, false));
        let Some(Popup::Choice { acts, .. }) = g.popups.front() else { panic!() };
        assert!(acts.iter().any(|a| matches!(a, Action::Inline(_, _, [120, 140, 400, 200]))));
    }

    /// The bribery map (rules-64 §1): a place opens its office tree, and the level toggle shows the church level in
    /// the same window, in front of the map (playtest: the church level could not be reached).
    #[test]
    fn real_bribe_tree_church_level() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        g.popups.clear();
        g.open(&assets, Menu::Targets(Plot::Bribe));
        assert!(matches!(g.popups.front(), Some(Popup::Map(Plot::Bribe))));
        // what a click on a place does
        g.reopen(&assets, Menu::Tree(Plot::Bribe, 3, false));
        let Some(Popup::Choice { menu: Some(Menu::Tree(_, 3, false)), acts, .. }) = g.popups.front() else { panic!("the secular tree in front") };
        let toggle = acts.iter().find(|a| matches!(a, Action::Open(Menu::Tree(_, _, true)))).cloned().expect("the level toggle");
        g.act(toggle, &mut ctx);
        assert!(matches!(g.popups.front(), Some(Popup::Choice { menu: Some(Menu::Tree(_, 3, true)), .. })), "the church tree in front");
        assert_eq!(g.popups.iter().filter(|p| matches!(p, Popup::Choice { menu: Some(Menu::Tree(..)), .. })).count(), 1);
    }

    /// Privilege dialogs (rules-38 §2): insurance, bless a county, law editing, a donation, an army, a project.
    #[test]
    fn real_privilege_dialogs() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        let h = g.me().unwrap();
        g.st.persons[h].set_cash_raw(5_000_000);
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        // Bischof of county 10 (global 22): insurance, bless, convert, miracle
        g.st.persons[h].set_loc(10);
        g.st.persons[h].set_idx(12);
        g.act(Action::Open(Menu::Insurance), &mut ctx);
        shot(&g, &assets, "priv_insurance");
        g.act(Action::Priv(25, 2, 0), &mut ctx);
        assert_eq!(g.st.persons[h].insurance() & 2, 2);
        assert!(matches!(g.popups.front(), Some(Popup::Choice { menu: Some(Menu::Insurance), .. })));
        g.popups.clear();
        g.act(Action::PrivView(53), &mut ctx);
        assert!(g.priv_cost > 0);
        shot(&g, &assets, "priv_bless");
        let cash = g.st.persons[h].cash();
        g.act(Action::Priv(53, 0, g.priv_cost), &mut ctx);
        assert_eq!(g.st.persons[h].cash(), cash - g.priv_cost);
        assert!(!g.st.heap.all(23, mad_sim::heap::ANY, mad_sim::heap::ANY).is_empty());
        g.popups.clear();
        // Finanzminister (global 28): the finance law book in edit mode, Projekte unterstützen
        g.st.persons[h].set_loc(13);
        g.st.persons[h].set_idx(5);
        g.open(&assets, Menu::LawEdit(0));
        shot(&g, &assets, "priv_lawedit");
        let tax = g.st.globals.law(0, 1);
        g.step(&assets, Field::Law(0, 1), 1);
        assert_eq!(g.st.globals.law(0, 1), (tax + 1).min(33));
        let before = g.st.globals.law(0, 0);
        g.act(Action::Sim(Command::EditLaw { book: 0, i: 0, delta: 0 }, None), &mut ctx);
        assert_eq!(g.st.globals.law(0, 0), before ^ 1);
        g.popups.clear();
        g.act(Action::Open(Menu::Project), &mut ctx);
        g.step(&assets, Field::PrivN(0, 9999), 100);
        shot(&g, &assets, "priv_project");
        assert_eq!(g.priv_n, 100);
        g.popups.clear();
        // a donation: city, then the building question with five prices
        g.act(Action::Open(Menu::PrivCity(9)), &mut ctx);
        shot(&g, &assets, "priv_city");
        g.act(Action::Priv(9, 3, 0), &mut ctx);
        assert!(matches!(g.pending.as_ref().map(|q| &q.ask), Some(Ask::Building { city: 3, .. })));
        shot(&g, &assets, "priv_building");
        g.pending = None;
        g.popups.clear();
        // a siege: the army dialog starts at the city's minimum
        g.act(Action::Open(Menu::PrivCity(28)), &mut ctx);
        g.open(&assets, Menu::PrivArmy(28));
        shot(&g, &assets, "priv_army");
        assert!(g.priv_n >= 1);
    }

    /// The Produktion page with more goods than rows (rules-38 §8): 8 rows left, 5 right, then "Weiteres".
    #[test]
    fn real_ertraege_overflow() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        let prod: Vec<(u8, u32)> = (0..15u8).map(|k| (k, 100 + 37 * k as u32)).collect();
        g.popups.push_back(Popup::Ertraege(prod, Some(12345), 0));
        g.ert_reveal = (99, 0);
        shot(&g, &assets, "ertraege_full");
        // live A/B (1601 Erträge of the 1600 capture): 174 Wolle, nothing exported
        g.popups.clear();
        g.popups.push_back(Popup::Ertraege(vec![(3, 174)], None, 0));
        g.ert_reveal = (0, 0);
        shot(&g, &assets, "ertraege_harvest");
        // rules-50 §3: the harvest line waits 1.5 s, then the row replaces it
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        g.update(&mut ctx, &crate::input::Input::default(), 1000);
        assert_eq!(g.ert_reveal, (0, 1000));
        g.update(&mut ctx, &crate::input::Input::default(), 600);
        assert_eq!(g.ert_reveal.0, 1);
        shot(&g, &assets, "ertraege_ab");
    }

    /// Ours: the ledger pages, the price table, the warnings and the production help (all behind their options),
    /// the ledger's recording from the sim events, and the Kontor keys only with the option on.
    #[test]
    fn real_extras_windows() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(3, 3, &[mad_sim::world::HumanSetup::test(3)]).unwrap());
        g.popups.clear();
        let h = g.me().unwrap();
        let b = g.st.branch_at(h, 3).unwrap();
        let loc = g.st.props[b].loc();
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        // the keys do nothing while the options are off
        g.update(&mut ctx, &Input { events: vec![Event::Key(Key::Char(b'b')), Event::Key(Key::Char(b'p'))], ..Default::default() }, 0);
        assert!(g.popups.is_empty());
        {
            let mut p = assets.prefs.borrow_mut();
            (p.ledger, p.price_table, p.warnings, p.production_help) = (true, true, true, true);
        }
        // a turn end's costs and the next turn start's sales make the year's line
        let sold = SimEvent::Sold { branch: b as u8, good: 0, city: 0, qty: 10, revenue: 300 };
        g.report(&assets, vec![SimEvent::BranchCosts { branch: b as u8, wages: 100, means: 50, transport: 0 }], vec![]);
        g.report(&assets, vec![SimEvent::TurnStart { person: h as u8, year: 1601 }, sold], vec![]);
        assert_eq!(g.ledger.last().map(|y| (y.year, y.branches.clone())), Some((1600, vec![(loc, 300, 150)])));
        g.popups.clear();
        for (year, worth, cash) in [(1601, 12_500, 900), (1602, 9_800, -400), (1603, 21_000, 3_100)] {
            g.ledger.push(LedgerYear { person: h as u8, year, worth, cash, branches: vec![(loc, worth / 10, worth / 20), (0, 500, 900)] });
        }
        g.update(&mut ctx, &Input { events: vec![Event::Key(Key::Char(b'b'))], ..Default::default() }, 0);
        assert!(matches!(g.popups.front(), Some(Popup::Choice { menu: Some(Menu::Ledger(0)), .. })));
        shot(&g, &assets, "ledger0");
        g.popups.clear();
        g.open(&assets, Menu::Ledger(1));
        shot(&g, &assets, "ledger1");
        g.popups.clear();
        g.update(&mut ctx, &Input { events: vec![Event::Key(Key::Char(b'p'))], ..Default::default() }, 0);
        assert!(matches!(g.popups.front(), Some(Popup::Choice { menu: Some(Menu::Prices), .. })));
        shot(&g, &assets, "prices");
        g.popups.clear();
        // warnings: nothing while solvent (the uncle's 802 Taler order exceeds the 750 start cash: a warning);
        // then a loan due this year, orders over the cash and a seizure ahead
        assert!(g.warnings(&assets, h).is_some());
        g.st.persons[h].set_cash_raw(5000);
        assert!(g.warnings(&assets, h).is_none());
        let i = g.st.heap.alloc(56, h as u8, mad_sim::state::NONE, 1).unwrap();
        g.st.heap.recs[i][4] = 20;
        mad_sim::heap::w32(&mut g.st.heap.recs[i], 5, 4000);
        g.st.persons[h].set_cash_raw(-3000);
        let w = g.warnings(&assets, h).expect("warnings");
        g.popups.push_back(w);
        shot(&g, &assets, "warnings");
        g.popups.clear();
        // production help on the branch screen
        g.st.persons[h].set_cash_raw(750);
        g.branch = Some(b);
        shot(&g, &assets, "branch_help");
        g.update(&mut ctx, &Input { events: vec![Event::Key(Key::Char(b'h'))], ..Default::default() }, 0);
        assert!(matches!(g.popups.front(), Some(Popup::Choice { menu: Some(Menu::ProdHelp(_)), .. })));
        shot(&g, &assets, "prodhelp");
        g.popups.clear();
        assets.prefs.borrow_mut().production_help = false;
        g.update(&mut ctx, &Input { events: vec![Event::Key(Key::Char(b'h'))], ..Default::default() }, 0);
        assert!(g.popups.is_empty());
    }

    /// Ours: the Extras window (modern options) and one toggle written to prefs.txt.
    #[test]
    fn real_extras_window() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        g.seed = Some(1234);
        // Tab opens it from the Kontor
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        let mut input = crate::input::Input::default();
        input.events.push(Event::Key(Key::Tab));
        g.update(&mut ctx, &input, 10);
        assert!(g.popups.iter().any(|p| matches!(p, Popup::Choice { menu: Some(Menu::Extras(0)), .. })));
        g.popups.clear();
        g.open(&assets, Menu::Extras(0));
        shot(&g, &assets, "extras");
        g.popups.clear();
        g.open(&assets, Menu::Extras(1));
        shot(&g, &assets, "extras1");
        assert!(g.st.house == mad_sim::state::House::default());
    }

    /// Ours (pref `sell_all`): off, V does nothing and the Handel map has no spot; on, the question, then one
    /// `SellStock` per branch and the summary. The stock afterwards is what selling each branch by hand leaves.
    #[test]
    fn real_sell_all() {
        let Some(assets) = test_assets() else { return };
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        let h = g.me().unwrap();
        let b = g.st.props_of(h, mad_sim::state::KIND_BRANCH).next().unwrap();
        g.st.props[b].set_stock(3, 20);
        let key = |g: &mut Game, ctx: &mut Ctx| g.update(ctx, &Input { events: vec![Event::Key(Key::Char(b'v'))], ..Default::default() }, 0);
        key(&mut g, &mut ctx);
        assert!(g.popups.is_empty());
        assert!(g.room_def(&assets, Room::Handel).hots.iter().all(|hs| !matches!(hs.act, Action::YesNo(_, _, ref a) if matches!(**a, Action::SellAll))));
        assets.prefs.borrow_mut().sell_all = true;
        g.room = Some(Room::Handel);
        shot_at(&g, &assets, "sell_all_spot", (300, 380));
        key(&mut g, &mut ctx);
        assert!(matches!(g.popups.front(), Some(Popup::Choice { .. })));
        shot(&g, &assets, "sell_all_ask");
        let mut by_hand = g.st.clone();
        for b in by_hand.props_of(h, mad_sim::state::KIND_BRANCH).collect::<Vec<_>>() {
            by_hand.apply(Command::SellStock { branch: b as u8, good: 0xFF, qty: 0 }, &mut mad_sim::Auto);
        }
        g.popups.clear();
        g.act(Action::SellAll, &mut ctx);
        shot(&g, &assets, "sell_all_done");
        assert_eq!(g.st.props[b].stock(3), 0);
        assert_eq!(g.st.persons[h].cash(), by_hand.persons[h].cash());
        assert_eq!(g.st.rng, by_hand.rng);
        assets.prefs.borrow_mut().sell_all = false;
    }

    /// Ours: the Chronik pages pack entries of varying height greedily, one entry at least per page.
    #[test]
    fn log_pages_pack() {
        assert_eq!(pack_pages([1, 2, 3, 1, 5, 1].into_iter(), 4), vec![0..2, 2..4, 4..5, 5..6]);
        assert_eq!(pack_pages([9].into_iter(), 4), vec![0..1]);
        assert!(pack_pages(std::iter::empty(), 4).is_empty());
    }

    /// Ours (prefs `event_log`, `order_copy`, `confirmations`): off, nothing is collected and the keys and actions
    /// behave as before; on, the Chronik holds the report lines of turn-end and year-end windows (key L shows them
    /// newest first), C/E copy a branch's orders through the sim, and a plot asks Ja/Nein first.
    #[test]
    fn real_extras_log_orders_confirm() {
        let Some(assets) = test_assets() else { return };
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        let mut g = Game::start(&assets, State::new_game(3, 3, &[mad_sim::world::HumanSetup::test(3)]).unwrap());
        g.popups.clear();
        let h = g.me().unwrap();
        let key = |g: &mut Game, ctx: &mut Ctx, c: u8| g.update(ctx, &Input { events: vec![Event::Key(Key::Char(c))], ..Default::default() }, 0);
        let fine = |n: i32| SimEvent::Fine { person: h as u8, amount: n };
        // off: no log, no key
        g.report(&assets, vec![fine(100)], vec![]);
        g.popups.clear();
        key(&mut g, &mut ctx, b'l');
        assert!(g.log.is_empty() && g.popups.is_empty());
        {
            let mut p = assets.prefs.borrow_mut();
            (p.event_log, p.order_copy, p.confirmations) = (true, true, true);
        }
        // a turn-end window and a year-end window, then a continued window: only its new lines are logged
        g.report(&assets, vec![fine(120), SimEvent::YearEnd { year: 1600 }, fine(130)], vec![]);
        g.popups.clear();
        g.window = vec!["Ihr zahlt Eure jährliche Geldstrafe von 120 Talern.".into()];
        g.report(&assets, vec![fine(140)], vec![]);
        g.popups.clear();
        let amounts: Vec<String> = g.log.iter().map(|(_, l)| l.split(" von ").nth(1).unwrap_or(l).to_string()).collect();
        assert_eq!(amounts, ["120 Talern.", "130 Talern.", "140 Talern."], "{:?}", g.log);
        for n in 0..30 {
            g.log.push((1601 + n / 10, format!("Eintrag {n}: {}", "lange Zeile ".repeat(n as usize % 4 + 1))));
        }
        key(&mut g, &mut ctx, b'l');
        assert!(matches!(g.popups.front(), Some(Popup::Choice { menu: Some(Menu::Log(0)), .. })));
        shot(&g, &assets, "log0");
        let pages = g.log_pages(&assets);
        assert!(pages.len() > 1 && pages[0].start == 0 && pages.last().unwrap().end == g.log.len());
        g.popups.clear();
        g.open(&assets, Menu::Log(1));
        shot(&g, &assets, "log1");
        g.popups.clear();
        // orders: the uncle's production order copied, pasted into another branch (or the same one, cleared)
        let b = g.st.branch_at(h, 3).unwrap();
        let b2 = g.st.props_of(h, mad_sim::state::KIND_BRANCH).find(|&p| p != b).unwrap_or(b);
        g.branch = Some(b);
        key(&mut g, &mut ctx, b'c');
        let clip = g.clip.expect("copied");
        assert_eq!(clip[0][0], 1);
        g.popups.clear();
        if b2 == b {
            g.sim(&assets, Command::SetOrder { branch: b as u8, slot: 0, order: [0; 8] });
        }
        g.branch = Some(b2);
        key(&mut g, &mut ctx, b'E');
        let pasted = g.st.props[b2].order(0);
        assert_eq!(pasted[..7], clip[0][..7]);
        assert!(matches!(pasted[7], mad_sim::economy::ST_POSSIBLE | mad_sim::economy::ST_IMPOSSIBLE), "the sim sets the status");
        shot(&g, &assets, "orders_pasted");
        g.branch = None;
        g.popups.clear();
        // confirmations: the bribe waits for Ja; Nein leaves the cash alone
        let v = (0..mad_sim::state::PERSONS).find(|&i| g.st.persons[i].used() && !g.st.persons[i].human() && g.st.persons[i].has_office()).unwrap() as u8;
        g.st.persons[h].set_cash_raw(0);
        g.act(Action::Sim(Command::Bribe { target: v, amount: 0 }, None), &mut ctx);
        let Some(Popup::Choice { src, acts, .. }) = g.popups.front() else { panic!("question") };
        assert!(String::from_utf8_lossy(src).contains("bestechen") && matches!(acts[0], Action::Sure(_)) && g.popups.len() == 1);
        shot(&g, &assets, "confirm_bribe");
        let yes = acts[0].clone();
        g.act(Action::Close, &mut ctx);
        assert!(g.popups.is_empty());
        g.act(yes, &mut ctx);
        assert!(g.popups.iter().any(|p| matches!(p, Popup::Choice { src, .. } if String::from_utf8_lossy(src).contains("Geldbeutel"))), "the bribe ran");
        g.popups.clear();
        {
            let mut p = assets.prefs.borrow_mut();
            (p.event_log, p.order_copy, p.confirmations) = (false, false, false);
        }
        g.act(Action::Sim(Command::Bribe { target: v, amount: 0 }, None), &mut ctx);
        assert!(g.popups.iter().any(|p| matches!(p, Popup::Choice { src, .. } if String::from_utf8_lossy(src).contains("Geldbeutel"))), "off: no question");
    }

    /// Live A/B screens of the 1600 capture (player "anna", 57 Taler, no office): Nächster Spieler, Kontor 1601, bill.
    #[test]
    fn real_live_ab_screens() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        let h = g.me().unwrap();
        let slot = g.st.persons[h].name() as usize;
        g.st.set_name_str(slot, "anna");
        g.st.persons[h].set_cash_raw(57);
        g.st.year = 1601;
        g.popups.clear();
        let src = subst(&assets.text(addr::NEXT_PLAYER), b"%1N1", &t(&g.person_name(h as u8)));
        g.popups.push_back(Popup::NextPlayer(subst(&src, b"%1N4", &t(&g.rank(h)))));
        shot(&g, &assets, "ab_next");
        g.popups.clear();
        shot(&g, &assets, "ab_kontor");
        g.report_year = Some(1600);
        g.report(&assets, vec![SimEvent::Bill { wages: 281, troops: 0, robbers: 0, means: 412, transport: 0, toll: 0, directors: 0, tax: 0, tithe: 0, spies: 0, saboteurs: 0, interest: 0, insurance: 0, cash: 0 }], vec![]);
        shot(&g, &assets, "ab_bill");
        // Spieloptionen and its sub-pages (capture 2026-11-14, 750 Taler in 1600)
        g.popups.clear();
        g.report_year = None;
        g.st.year = 1600;
        g.st.persons[h].set_cash_raw(750);
        g.room = Some(Room::Optionen);
        shot(&g, &assets, "ab_opt");
        // the clicked spot's label stays under its page (the capture's click points)
        for (m, name, at) in [(Menu::Music, "ab_music", (235, 319)), (Menu::Graphics, "ab_graphics", (398, 234)), (Menu::Options, "ab_options", (296, 235))] {
            g.popups.clear();
            g.spot_click = Some(at);
            g.open(&assets, m);
            shot(&g, &assets, name);
        }
        // Kirchgang, Kerzen stiften (capture 2026-11-14)
        g.popups.clear();
        g.room = Some(Room::Kirchgang);
        g.spot_click = Some((510, 330));
        g.open(&assets, Menu::Candles);
        shot(&g, &assets, "ab_candles");
        g.popups.clear();
        g.spot_click = Some((267, 345));
        g.open(&assets, Menu::Alms);
        shot(&g, &assets, "ab_alms");
        // the Kontor rooms with the pointer where the capture clicked
        g.popups.clear();
        g.spot_click = None;
        for (r, name, at) in [(Room::Truppen, "ab_truppen", (364, 241)), (Room::Schreibstube, "ab_schreib", (461, 427)), (Room::Bezahlung, "ab_bezahlung", (199, 346)), (Room::Hinterzimmer, "ab_hinter", (81, 289)), (Room::Handel, "ab_handel", (60, 440))] {
            g.room = Some(r);
            shot_at(&g, &assets, name, at);
        }
        // Schreibstube: each spot clicked, then a right click (capture 2026-11-14)
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        let input = |e: Event| crate::input::Input { events: vec![e], ..Default::default() };
        for (name, at) in [("bewerbung", (275, 417)), ("lender", (86, 370)), ("gesetze", (569, 315)), ("privileg", (460, 414)), ("direktoren", (540, 180)), ("kreditbuch", (100, 180))] {
            g.room = Some(Room::Schreibstube);
            g.popups.clear();
            g.update(&mut ctx, &input(Event::Left(at.0, at.1)), 0);
            shot_at(&g, &assets, &format!("ab_s_{name}"), at);
            g.update(&mut ctx, &input(Event::Right(320, 470)), 0);
            shot_at(&g, &assets, &format!("ab_s_{name}_back"), (320, 470));
        }
        // Hinterzimmer plots and the church's Konvertieren / Austreten (capture 2026-11-15)
        // the capture's player is evangelical (she is offered the catholic faith)
        let rel = g.st.persons[h].religion();
        g.st.persons[h].set_religion(if rel == 0 { 1 } else { 0 });
        assert_ne!(rel, g.st.persons[h].religion());
        for (room, list) in [(Room::Hinterzimmer, [("spies", (424, 289)), ("sabot", (51, 233)), ("blackmail", (185, 447)), ("denounce", (199, 351))].as_slice()), (Room::Kirche, [("convert", (550, 402)), ("leave", (90, 402))].as_slice())] {
            for &(name, at) in list {
                g.room = Some(room);
                g.popups.clear();
                g.update(&mut ctx, &input(Event::Left(at.0, at.1)), 0);
                shot_at(&g, &assets, &format!("ab_h_{name}"), at);
                g.update(&mut ctx, &input(Event::Right(600, 470)), 0);
                shot_at(&g, &assets, &format!("ab_h_{name}_back"), (600, 470));
            }
        }
        // Kirchgang: Beten, then Ablass (capture 2026-11-15)
        for (name, at) in [("pray", (100, 260)), ("indulgence", (382, 317))] {
            g.room = Some(Room::Kirchgang);
            g.popups.clear();
            g.update(&mut ctx, &input(Event::Left(at.0, at.1)), 0);
            shot_at(&g, &assets, &format!("ab_k_{name}"), at);
        }
        // Handel: the city info of Fürstenbrück (capture 2026-11-15)
        g.room = Some(Room::Handel);
        g.popups.clear();
        g.update(&mut ctx, &input(Event::Left(333, 186)), 0);
        shot_at(&g, &assets, "ab_c0", (333, 186));
    }

    /// Live A/B on the original's own save (`MAD_ORIG_SAVE` = a GAMEn.DAT; skipped without it): the state-dependent
    /// screens of the 2026-11-14/15 captures (Kontor, Handel with its flags, Spielstand, Kerzen, Spenden, Austreten).
    #[test]
    fn real_orig_save_screens() {
        let Some(assets) = test_assets() else { return };
        let Some(data) = std::env::var_os("MAD_ORIG_SAVE").and_then(|p| std::fs::read(p).ok()) else { return };
        let mut g = Game::start(&assets, State::from_game_dat(&data, 1).unwrap());
        g.popups.clear();
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        let input = |e: Event| crate::input::Input { events: vec![e], ..Default::default() };
        shot(&g, &assets, "os_kontor");
        for (room, name, at) in [(None, "os_spielstand", (200, 225)), (None, "os_handel", (60, 440)), (Some(Room::Kirchgang), "os_candles", (510, 330)), (Some(Room::Kirchgang), "os_alms", (267, 345)), (Some(Room::Kirche), "os_leave", (90, 402)), (Some(Room::Kirche), "os_convert", (550, 402))] {
            g.room = room;
            g.popups.clear();
            g.update(&mut ctx, &input(Event::Left(at.0, at.1)), 0);
            shot_at(&g, &assets, name, at);
        }
        // Handel: Greifenheim's city info, the branch in Bergstedt; Söldner & Räuber: Hohenfels and a camp
        for (room, name, at) in [(Room::Handel, "os_g4", (425, 110)), (Room::Handel, "os_b5", (564, 128)), (Room::Truppen, "os_c0", (456, 191)), (Room::Truppen, "os_c3", (283, 128))] {
            g.room = Some(room);
            g.branch = None;
            g.popups.clear();
            g.update(&mut ctx, &input(Event::Left(at.0, at.1)), 0);
            shot_at(&g, &assets, name, at);
        }
        // the Bergstedt branch's six controls (capture 2026-11-15): click, shot, right click
        g.room = Some(Room::Handel);
        g.branch = None;
        g.popups.clear();
        g.update(&mut ctx, &input(Event::Left(564, 128)), 0);
        for i in 0..6 {
            let at = (70 + 100 * i, 410);
            g.update(&mut ctx, &input(Event::Left(at.0, at.1)), 0);
            shot_at(&g, &assets, &format!("os_ctl{i}"), at);
            g.update(&mut ctx, &input(Event::Right(600, 200)), 0);
        }
        // the empty order slot clicked once: the first production verb (capture 2026-11-15)
        g.popups.clear();
        g.update(&mut ctx, &input(Event::Left(320, 332)), 0);
        shot_at(&g, &assets, "os_order1", (320, 332));
        for i in 1..=2 {
            g.update(&mut ctx, &input(Event::Left(60, 332)), 0);
            shot_at(&g, &assets, &format!("os_verb{i}"), (60, 332));
        }
        // the law book's three tabs (capture 2026-11-15)
        g.branch = None;
        g.popups.clear();
        g.room = Some(Room::Schreibstube);
        g.update(&mut ctx, &input(Event::Left(569, 315)), 0);
        for (name, at) in [("justiz", (300, 398)), ("kirche", (427, 398)), ("finanzen", (134, 398))] {
            g.update(&mut ctx, &input(Event::Left(at.0, at.1)), 0);
            shot_at(&g, &assets, &format!("os_law_{name}"), at);
        }
        // Privilegien: the Medicus and the certificate info by their gems (capture 2026-11-15)
        g.popups.clear();
        g.room = Some(Room::Schreibstube);
        g.update(&mut ctx, &input(Event::Left(460, 414)), 0);
        g.update(&mut ctx, &input(Event::Left(120, 108)), 0);
        shot_at(&g, &assets, "os_priv_medicus", (120, 108));
        g.update(&mut ctx, &input(Event::Right(600, 460)), 0);
        shot_at(&g, &assets, "os_priv_back", (600, 460));
        g.update(&mut ctx, &input(Event::Left(120, 132)), 0);
        shot_at(&g, &assets, "os_priv_zert", (120, 132));
        // Hinterzimmer: Spionage, then a click on the map (Bergenhöh's office tree, capture 2026-11-15)
        g.popups.clear();
        g.room = Some(Room::Hinterzimmer);
        g.update(&mut ctx, &input(Event::Left(424, 289)), 0);
        g.update(&mut ctx, &input(Event::Left(491, 145)), 0);
        shot_at(&g, &assets, "os_spy_tree", (491, 145));
        g.update(&mut ctx, &input(Event::Left(320, 112)), 0);
        shot_at(&g, &assets, "os_spy_person", (320, 112));
        // rules-55 §15: with Warnungen on and Runde bestätigen off, ending the turn without a branch visit warns (0xC0324)
        let cfg = g.cfg_end;
        (g.cfg_end, g.unvisited) = (0x10, true); // as at a turn start (earlier steps opened a branch)
        assert!(matches!(g.end_turn_dialog(&assets), Some(Popup::Choice { .. })));
        g.popups.clear();
        g.room = None;
        g.popups.extend(g.end_turn_dialog(&assets));
        shot_at(&g, &assets, "os_not_visited", (320, 300));
        g.popups.clear();
        let b = (0..mad_sim::state::CITIES).find_map(|c| g.st.branch_at(g.me().unwrap(), c)).unwrap();
        g.act_keep(Action::Branch(b), &mut ctx);
        assert!(g.end_turn_dialog(&assets).is_none());
        (g.cfg_end, g.branch) = (cfg, None);
        // Kirchgang: Spenden, then its first bullet, on a freshly loaded game
        // (capture 2026-11-16; the price standing(p, 40) is drawn at the turn start: 267 and 334 in two runs)
        {
            let mut g = Game::start(&assets, State::from_game_dat(&data, 1).unwrap());
            g.popups.clear();
            g.room = Some(Room::Kirchgang);
            for (k, at) in [(267, 345), (161, 256)].into_iter().enumerate() {
                g.update(&mut ctx, &input(Event::Left(at.0, at.1)), 0);
                shot_at(&g, &assets, &format!("os_alms{k}"), at);
            }
        }
        // Kirchgang: Kerzen stiften, then "Ja." on a fresh game (capture 2026-11-16)
        {
            let mut g = Game::start(&assets, State::from_game_dat(&data, 1).unwrap());
            g.popups.clear();
            g.room = Some(Room::Kirchgang);
            g.update(&mut ctx, &input(Event::Left(510, 330)), 0);
            if let Some(w) = g.popups.front().and_then(|p| p.window(&assets, &[])) {
                let it = w.item_rect(0);
                g.update(&mut ctx, &input(Event::Left(it.x + 9, it.y + 12)), 0);
                shot_at(&g, &assets, "os_candles1", (it.x + 9, it.y + 12));
            }
        }
        // Kirchgang: Beten and the confessional (Ablass) (capture 2026-11-16)
        for (name, at) in [("os_pray", (100, 260)), ("os_ablass", (380, 320))] {
            g.popups.clear();
            g.room = Some(Room::Kirchgang);
            g.update(&mut ctx, &input(Event::Left(at.0, at.1)), 0);
            shot_at(&g, &assets, name, at);
        }
        g.popups.clear();
        g.room = None;
        // banner gold display (capture 2026-11-16)
        g.popups.clear();
        g.room = None;
        g.update(&mut ctx, &input(Event::Left(70, 22)), 0);
        shot_at(&g, &assets, "os_gold", (70, 22));
        for (k, at) in [(491, 145), (320, 112)].into_iter().enumerate() {
            g.update(&mut ctx, &input(Event::Left(at.0, at.1)), 0);
            shot_at(&g, &assets, &format!("os_bribe{k}"), at);
        }
        // Kontor window spot "Geld zum Fenster hinauswerfen" (capture 2026-11-16)
        g.popups.clear();
        g.room = None;
        g.update(&mut ctx, &input(Event::Left(607, 257)), 0);
        shot_at(&g, &assets, "os_giveup", (607, 257));
        // Sabotage: Fürstenbrück on the small map, then Richter Gunther (capture 2026-11-16), on a freshly loaded game;
        // the price draws the RNG (rules-04 §2) and differs between runs of the original too (350, 370)
        {
            let mut g = Game::start(&assets, State::from_game_dat(&data, 1).unwrap());
            g.popups.clear();
            g.room = Some(Room::Hinterzimmer);
            for (k, at) in [(51, 233), (342, 221), (320, 215)].into_iter().enumerate() {
                g.update(&mut ctx, &input(Event::Left(at.0, at.1)), 0);
                shot_at(&g, &assets, &format!("os_sab{k}"), at);
            }
        }
        // Erpressung against Bergenhöh's Bürgermeister without evidence: 0xBC698 (rules-04 §5.2)
        g.popups.clear();
        g.room = Some(Room::Hinterzimmer);
        for at in [(180, 445), (491, 145), (320, 112)] {
            g.update(&mut ctx, &input(Event::Left(at.0, at.1)), 0);
        }
        shot_at(&g, &assets, "os_blackmail", (320, 112));
        // Anschwärzen: victim Vogt Benno (Bergenhöh), listener Kanzler Arnd (capture 2026-11-16)
        g.popups.clear();
        g.room = Some(Room::Hinterzimmer);
        for (k, at) in [(199, 351), (491, 145), (320, 112), (320, 186)].into_iter().enumerate() {
            g.update(&mut ctx, &input(Event::Left(at.0, at.1)), 0);
            shot_at(&g, &assets, &format!("os_denounce{k}"), at);
        }
        // formats §8.7: after a click the highlight stays off until the hovered index changes
        assert!(g.hover.1);
        g.update(&mut ctx, &crate::input::Input { mouse: (5, 5), ..Default::default() }, 0);
        assert!(!g.hover.1);
        // Fürstenbrück's Lager icon without the cash (capture 2026-11-16)
        g.popups.clear();
        g.room = Some(Room::Handel);
        g.update(&mut ctx, &input(Event::Left(333, 186)), 0);
        g.update(&mut ctx, &input(Event::Left(133, 365)), 0);
        shot_at(&g, &assets, "os_lager_nocash", (133, 365));
        // city info of anna's only branch city: the 0xBE4D4 line instead of a close item (rules-23 §7.1)
        g.popups.clear();
        g.room = Some(Room::Handel);
        if let Some(hs) = g.room_def(&assets, Room::Handel).hots.into_iter().find(|hs| matches!(hs.act, Action::Open(Menu::City(c)) if g.st.branch_at(g.me().unwrap(), c as usize).is_some())) {
            let pos = (hs.rect[0] + hs.rect[2] / 2, hs.rect[1] + hs.rect[3] / 2);
            eprintln!("OWN CITY at {pos:?}");
            g.update(&mut ctx, &input(Event::Left(pos.0, pos.1)), 0);
            shot_at(&g, &assets, "os_city_own", pos);
        }
        // Löhne (rules-54): the stacks are pictures, the percentage is a spinner (live: a stack click does nothing)
        g.popups.clear();
        g.room = Some(Room::Bezahlung);
        let pay = |g: &Game| g.st.persons[g.me().unwrap()].pay_workers();
        let before = pay(&g);
        g.wage_click(&assets, (270, 250));
        assert_eq!(pay(&g), before);
        let it = Game::wage_pct(&assets, 100).items[0];
        let (x0, y0) = (270 - 62 + it.x, 396 + it.y);
        g.wage_click(&assets, (x0 + it.w - 2, y0 + it.h - 2));
        assert_eq!(pay(&g), 98, "units digit down: −1, then even");
        g.wage_click(&assets, (x0 + 2, y0 + 2));
        assert_eq!(pay(&g), 150, "hundreds up, clamped");
        shot_at(&g, &assets, "os_wages150", (x0 + 2, y0 + 2));
        g.sim(&assets, Command::SetPay { which: 1, pct: 94 });
        shot_at(&g, &assets, "os_wages94", (284, 418));
        g.sim(&assets, Command::SetPay { which: 1, pct: before });
        // Kreditbuch with the loan the capture took: Ratsherr Ingo, 391 Taler at 22 % until 1609 (2026-11-16)
        let h = g.me().unwrap();
        if let Some(ingo) = (0..mad_sim::state::PERSONS).find(|&p| g.st.persons[p].used() && g.st.persons[p].has_office() && g.first_name(p) == "Ingo") {
            let saved = (g.st.heap.clone(), g.st.persons[h].cash());
            let r = g.st.heap.alloc(56, h as u8, mad_sim::state::NONE, 9).unwrap();
            g.st.heap.recs[r][4] = ingo as u8;
            mad_sim::heap::w32(&mut g.st.heap.recs[r], 5, 391);
            g.st.heap.recs[r][9] = 22;
            g.st.persons[h].set_cash_raw(1141);
            g.popups.clear();
            g.room = Some(Room::Kreditbuch);
            shot_at(&g, &assets, "os_kreditbuch", (100, 180));
            g.st.heap = saved.0;
            g.st.persons[h].set_cash_raw(saved.1);
        }
        // Spielstand: a click on each row's gem (capture 2026-11-16)
        for (name, at) in [("os_score1", (60, 236)), ("os_score2", (60, 260))] {
            g.popups.clear();
            g.room = None;
            g.update(&mut ctx, &input(Event::Left(200, 225)), 0);
            g.update(&mut ctx, &input(Event::Left(at.0, at.1)), 0);
            shot_at(&g, &assets, name, at);
        }
    }

    /// Live A/B of the battle intro on the `mkbattle … fight` save (`MAD_BATTLE_SAVE`; skipped without it): the end of
    /// the turn opens the castle field with intro line 1 over the defenders.
    #[test]
    fn real_battle_save_intro() {
        let Some(assets) = test_assets() else { return };
        let Some(data) = std::env::var_os("MAD_BATTLE_SAVE").and_then(|p| std::fs::read(p).ok()) else { return };
        let mut st = State::from_game_dat(&data, 1).unwrap();
        st.szenario = assets.read("SZENARIO.DAT").unwrap();
        let mut g = Game::start(&assets, st);
        g.popups.clear();
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        g.act(Action::EndTurn, &mut ctx);
        assert!(matches!(g.pending.as_ref().map(|p| &p.ask), Some(Ask::BattleIntro { .. })));
        shot_at(&g, &assets, "bs_intro1", (320, 300));
        // click through the four captions, then the ransom offer and the attack question follow (capture f5..f7)
        for k in 0..12 {
            if !matches!(g.pending.as_ref().map(|p| &p.ask), Some(Ask::BattleIntro { .. })) {
                break;
            }
            g.update(&mut ctx, &crate::input::Input { events: vec![Event::Left(320, 300)], ..Default::default() }, 0);
            if k == 0 {
                shot_at(&g, &assets, "bs_intro2", (320, 300));
            }
        }
        shot_at(&g, &assets, "bs_after_intro", (320, 300));
    }

    /// Kreditbuch lines per formats §10.2: the EXE sentence, the year highlighted with at most 2 years left.
    #[test]
    fn real_kreditbuch_lines() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        let h = g.me().unwrap();
        let c = (0..mad_sim::state::PERSONS).find(|&p| g.st.persons[p].used() && !g.st.persons[p].human()).unwrap() as u8;
        for (years, amount) in [(5u8, 12000u32), (2, 3000)] {
            let i = g.st.heap.alloc(56, h as u8, 0xFF, years).unwrap();
            let r = &mut g.st.heap.recs[i];
            r[4] = c;
            mad_sim::heap::w32(r, 5, amount);
            r[9] = 8;
        }
        g.room = Some(Room::Kreditbuch);
        shot(&g, &assets, "kreditbuch");
        assert!(rooms::kreditbuch_ok(&g.room_def(&assets, Room::Kreditbuch)));
    }

    /// Divine events are message boxes (rules-12 §3.10), the tornado a larger one.
    #[test]
    fn real_divine_boxes() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        g.report(&assets, vec![SimEvent::Divine { kind: 0, city: 3, with: 0xFF, illness: 0 }, SimEvent::Tornado { a: 2, b: 5, n: 70, churches: [7, 4], museums: [6, 3], hospitals: [5, 2] }], vec![]);
        let rects: Vec<[i32; 4]> = g.popups.iter().filter_map(|p| if let Popup::Choice { rect, .. } = p { Some(*rect) } else { None }).collect();
        assert_eq!(rects, [[160, 100, 320, 280], [80, 80, 480, 320]]);
        assert!(!g.popups.iter().any(|p| matches!(p, Popup::Report(_) | Popup::YearReport(_))), "no report line twice");
    }

    /// A film inside the game plays and a click skips it.
    #[test]
    fn real_film_popup() {
        use crate::input::Input;
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        g.popups.extend(Game::film(&assets, "KERZE.FLC", 10));
        assert!(matches!(g.popups.front(), Some(Popup::Film(_))));
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        for _ in 0..5 {
            g.update(&mut ctx, &Input::default(), 100);
        }
        assert!(matches!(g.popups.front(), Some(Popup::Film(_))), "still playing after half a second");
        g.update(&mut ctx, &Input { events: vec![Event::Left(5, 5)], ..Default::default() }, 0);
        assert!(g.popups.is_empty());
    }

    /// A speech marker names its WAV and does not show in the caption.
    #[test]
    fn speech_marker() {
        let src = [speech(229906), b"$CHallo".to_vec()].concat();
        assert_eq!(speech_of(&src).as_deref(), Some("0229906"));
        assert_eq!(speech_of(&voice([11, 0, 0, 2], 74)).as_deref(), Some("B002074"));
        assert_eq!(speech_of(b"$CHallo"), None);
        assert_eq!(speech_of(&[speech(224_900), speech(224_901)].concat()).as_deref(), Some("0224900+0224901"));
        let Some(assets) = test_assets() else { return };
        let glyphs = |s: &[u8]| text::layout(&assets.font, s, &[], &[], 640, &[]).glyphs.len();
        assert_eq!(glyphs(&src), glyphs(b"$CHallo"), "the marker draws nothing");
        assert!(assets.speech_wav("0229906").is_some());
        for f in ["B002074", "C022080", "1022081", "0222924"] {
            assert!(assets.speech_wav(f).is_some(), "{f}");
        }
        for n in [213_900, 223_901, 223_902, 223_903, 223_904, 223_905, 223_906, 223_907, 225_900, 225_901, 225_902] {
            assert!(assets.speech_wav(&format!("0{n}")).is_some(), "{n}");
        }
    }

    #[test]
    fn plain_strips_codes() {
        assert_eq!(plain(b"$C1601 - Ihr bekamt $$ einen Sohn...$N"), "1601 - Ihr bekamt $ einen Sohn...");
        assert_eq!(plain(b"-$E$6>5 m$18c Lagerraum$|"), "-5 m² Lagerraum");
    }

    /// rules-61: accepting the matchmaker's offer shows the candidate's portrait (no draw) under two click captions,
    /// "Dies ist …" and the gift advice; the farewell belongs to declining only.
    #[test]
    fn real_matchmaker_follow_up() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        g.act(Action::Sim(Command::Matchmaker, None), &mut ctx);
        let Some(Ask::Match { target, .. }) = g.pending.as_ref().map(|p| p.ask.clone()) else { panic!("no offer") };
        g.popups.clear();
        g.act(Action::Answer(0), &mut ctx);
        shot(&g, &assets, "kupplerin_this_is");
        let Some(Popup::Court { pic: 0xBD, figures, boxes, .. }) = g.popups.front() else { panic!("{}", g.popups.len()) };
        let c = target as u16;
        let want = if g.st.persons[target as usize].gender() == 0 { 0x18B + c % 3 } else { 0x186 + c % 5 };
        assert_eq!(figures.as_slice(), &[(want, 20, 390)]);
        let texts: Vec<String> = boxes.iter().map(|b| String::from_utf8_lossy(&b.1).into_owned()).collect();
        assert_eq!((boxes[0].0, boxes[1].0), ([100, 390, 540, 90], [100, 360, 540, 120]));
        assert!(texts[0].contains("bsch, nicht wahr"), "{}", texts[0]);
        assert!(texts[1].contains("Geschenke") && !texts[1].contains("%s") && !texts[1].contains("%o"), "{}", texts[1]);
        assert_eq!(texts.len(), 2, "no farewell after accepting");
        assert!(g.st.heap.has(30, g.me().unwrap() as u8) && g.st.persons[target as usize].used());
    }

    /// A human camp attacks every castle and camp at turn end (rules-07 §4 type-51 orders). Each
    /// sim question parks the command; answering re-runs it from the saved state until it
    /// completes, and every fight ends in a battle report.
    #[test]
    fn real_battle_questions_replay() {
        use mad_sim::state::{KIND_CAMP, KIND_CASTLE, PROPS};
        let Some(assets) = test_assets() else { return };
        let mut st = State::new_game(9, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap();
        st.szenario = assets.read("SZENARIO.DAT").unwrap();
        let mut g = Game::start(&assets, st);
        g.popups.clear();
        let h = g.me().unwrap();
        let camp = (0..PROPS).find(|&p| g.st.props[p].kind() == KIND_CAMP).unwrap();
        g.st.props[camp].set_owner(h as u8);
        let targets: Vec<usize> = (0..PROPS).filter(|&p| matches!(g.st.props[p].kind(), KIND_CASTLE | KIND_CAMP) && p != camp).collect();
        // the first attack is always called off (rules-50 §2: rand(200) <= 199)
        g.st.props[targets[0]].set_b3(199);
        for &tp in &targets {
            let r = g.st.heap.alloc(51, h as u8, camp as u8, 1).unwrap();
            g.st.heap.recs[r][4] = g.st.props[tp].loc() - 14;
            g.st.heap.recs[r][5..9].fill(250);
        }
        // the player must survive the turn end (troop bill, life) to watch the fights
        g.st.persons[h].set_cash_raw(50_000_000);
        g.st.persons[h].set_life(60);
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = ctx_for(&assets, &mut audio);
        g.act(Action::EndTurn, &mut ctx);
        let (mut asked, mut tactics, mut intros, mut called_off, mut prefights) = (0, 0, 0, 0, 0);
        while let Some(p) = &g.pending {
            assert!(has_dialog(&p.ask));
            if let Ask::BattleIntro { code, intro, .. } = p.ask {
                // rules-50 §1: four captions over the defender-only field, which does not play on under them
                intros += 1;
                let Some(Popup::Battle(v)) = g.popups.front() else { panic!("the field opens with the captions") };
                let (caps, pos) = v.captions();
                let caps: Vec<String> = caps.iter().map(|c| plain(c)).collect();
                assert_eq!((caps.len(), pos), (4, 0), "{caps:#?}");
                assert!(caps.iter().all(|c| !c.contains('%')), "{caps:#?}");
                let place = g.loc(intro.loc).to_string();
                assert_eq!(caps[0], format!("{}{place}...", if code == 4 { "Burg " } else { "Räuberlager " }));
                let last = match (code, intro.called_off) {
                    (4, true) => "von einem Angriff abzusehen.",
                    (_, true) => "das Räubernest bleibt verborgen.",
                    (4, false) => "greifen an...",
                    _ => "wird angegriffen!",
                };
                assert!(caps[3].ends_with(last) && (intro.called_off || caps[3].contains("Räuber")), "{caps:#?}");
                called_off += intro.called_off as usize;
                if intros == 1 {
                    shot(&g, &assets, "battle_intro");
                }
                for k in 0..4 {
                    for _ in 0..20 {
                        g.update(&mut ctx, &Input::default(), 10);
                    }
                    let Some(Popup::Battle(v)) = g.popups.front() else { unreachable!() };
                    assert_eq!((v.captions().0.len(), v.captions().1), (4 - k, 0), "caption {k} waits for a click");
                    g.update(&mut ctx, &Input { events: vec![Event::Left(320, 240)], ..Default::default() }, 0);
                }
                assert!(!matches!(&g.pending, Some(p) if matches!(p.ask, Ask::BattleIntro { .. }) && p.answers.len() < intros), "the last click answers");
                continue;
            }
            if let Ask::Tactics { .. } = p.ask {
                // the tactic menu is the battlefield and opens only on the hotkey (rules-44 §2): pressed during the
                // first activation, it opens when that activation ends; the menu is shot, then everyone keeps his tactic
                tactics += 1;
                assert_eq!(matches!(g.popups.front(), Some(Popup::Battle(v)) if v.menu_open()), tactics == 2, "ask {tactics}");
                if tactics == 1 {
                    // the hotkey while the first activation is on screen: the fight is computed again up to its end
                    g.battle_hotkey(&assets);
                    continue;
                }
                if tactics == 2 {
                    shot(&g, &assets, "tactic_intro");
                    let Some(Popup::Battle(v)) = g.popups.front_mut() else { unreachable!() };
                    assert!(v.overlay_open(), "the first menu opens with the message box");
                    assert!(matches!(v.click_overlay(&assets, (10, 10)), battle::Overlaid::None) && !v.overlay_open());
                    // right click with the confirmation on asks "Seid Ihr zufrieden?"; Nein returns to the menu
                    v.confirm = true;
                    assert!(matches!(v.finish_menu(), battle::Overlaid::None) && v.overlay_open());
                    shot(&g, &assets, "tactic_satisfied");
                    let Some(Popup::Battle(v)) = g.popups.front_mut() else { unreachable!() };
                    v.dismiss_overlay();
                    v.confirm = false;
                    assert!(matches!(v.finish_menu(), battle::Overlaid::Submit));
                    shot(&g, &assets, "tactic_menu");
                    // a click on the first group moves it to its next allowed tactic; the answer says so
                    let Some(Popup::Battle(v)) = g.popups.front_mut() else { unreachable!() };
                    let before = v.answer();
                    v.click(v.group_pos(0));
                    assert_ne!(v.answer(), before, "click cycles the tactic");
                    let mut frame = crate::gfx::Frame::new(assets.palette);
                    let pos = v.group_pos(0);
                    v.draw(&mut frame, &assets, pos);
                    if let Ok(dir) = std::env::var("MAD_SHOT_DIR") {
                        crate::gfx::write_screenshot(&format!("{dir}/tactic_hover.png"), &frame).unwrap();
                    }
                    v.click(v.group_pos(0)); // cycle on; tactics 0 -> next -> ... answer stays valid
                    v.close_menu();
                }
                g.act(Action::Answer(0), &mut ctx);
                continue;
            }
            if let Ask::PreFight { q, .. } = &p.ask {
                // rules-52: the pages wait for clicks over the field, then the question comes up in the window
                let (q, at) = (q.clone(), p.answers.len());
                prefights += 1;
                let mut pages = vec![];
                while g.pending.as_ref().is_some_and(|p| p.answers.len() == at)
                    && let Some(Popup::Battle(v)) = g.popups.front()
                    && v.caption_open()
                {
                    pages.push(plain(&v.captions().0[0]));
                    if prefights == 1 {
                        shot(&g, &assets, "prefight_page");
                    }
                    g.update(&mut ctx, &Input { events: vec![Event::Left(320, 240)], ..Default::default() }, 0);
                }
                if prefights == 1 {
                    // a computer defender's first line (2882/2883/2886) before 3069
                    assert!(pages.first().is_some_and(|p| ["bietet Euch", "lacht höhnisch", "letzte Anweisungen"].iter().any(|w| p.contains(w))), "{pages:#?}");
                    assert!(matches!(q, mad_sim::fight::Parley::Attack { .. }), "{q:?}");
                }
                if q != mad_sim::fight::Parley::None {
                    let Some(Popup::Choice { src, menu: Some(Menu::Question), .. }) = g.popups.front() else { panic!("the question after the pages") };
                    if let mad_sim::fight::Parley::Attack { .. } = q {
                        assert!(plain(src).contains("soll der Angriff stattfinden?"), "{}", plain(src));
                    }
                    shot(&g, &assets, &format!("prefight{prefights}"));
                    // 0: fight, "Genug", the attacker's side
                    g.act(Action::Answer(0), &mut ctx);
                }
                continue;
            }
            // the occupy question comes after the report's first page over the field (rules-59 §2)
            if let Some(Ask::OccupyCastle { .. }) = g.pending.as_ref().map(|p| &p.ask)
                && let Some(Popup::Battle(v)) = g.popups.front()
            {
                assert!(v.caption_open() && plain(&v.captions().0[0]).contains("Nach einer langen Schlacht"));
                g.update(&mut ctx, &Input { events: vec![Event::Left(320, 240)], ..Default::default() }, 0);
            }
            // the open dialog is the question and cannot be skipped
            assert!(matches!(g.popups.front(), Some(Popup::Choice { menu: Some(Menu::Question), .. })));
            asked += 1;
            shot(&g, &assets, &format!("question{asked}"));
            g.act(Action::Answer(0), &mut ctx);
        }
        let texts: Vec<String> = g.popups.iter().map(|p| match p { Popup::Report(l) | Popup::YearReport(l) => l.join(" | "), Popup::Choice { src, .. } => String::from_utf8_lossy(src).into_owned(), _ => "<screen>".into() }).collect();
        assert!(intros >= 2 && called_off >= 1 && prefights >= 1, "{intros} intros, {called_off} called off, {prefights} pre-fight");
        assert!(tactics >= 3, "hotkey stop, menu, then the held last question: {tactics}");
        assert!(asked >= 1, "an overwhelming attack conquers and asks to occupy: {texts:#?}");
        // the report pages wait in the finished battle views (rules-59 §0)
        let reports = g.popups.iter().filter(|p| matches!(p, Popup::Battle(v) if v.report_pages().iter().any(|s| s.windows(9).any(|w| w == b"Bezahlung")))).count();
        assert!(reports >= 1, "battle reports: {reports}, questions: {asked}");
        shot(&g, &assets, "battle_report");
        let mut views = 0;
        while let Some(Popup::Battle(v)) = g.popups.iter_mut().find(|p| matches!(p, Popup::Battle(_))) {
            views += 1;
            v.seek(0);
            let sounds = v.all_sounds();
            assert!(sounds.iter().all(|&n| (0x31..=0x5F).contains(&n)), "{sounds:?}");
            assert!(!sounds.is_empty(), "a fight makes some noise");
            let n = v.frames();
            for k in [0, n / 4, n / 2, n - 1] {
                v.seek(k);
                let mut frame = crate::gfx::Frame::new(assets.palette);
                v.draw(&mut frame, &assets, (630, 470));
                if let Ok(dir) = std::env::var("MAD_SHOT_DIR") {
                    crate::gfx::write_screenshot(&format!("{dir}/fight{views}_{k}.png"), &frame).unwrap();
                }
            }
            let i = g.popups.iter().position(|p| matches!(p, Popup::Battle(_))).unwrap();
            g.popups.remove(i);
        }
        assert!(views >= 1 && tactics >= 1, "a human battle brings its playback and its tactic menu: {views} {tactics} {asked}");
        std::fs::remove_dir_all(&assets.user_saves).ok();
    }

    /// rules-23 §7.2: selling and renovating a residence ask first, with the word and price filled in.
    #[test]
    fn real_residence_confirmations() {
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(4, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        let h = g.me().unwrap();
        let b = (0..mad_sim::state::PROPS).find(|&p| g.st.props[p].kind() == mad_sim::state::KIND_BRANCH && g.st.props[p].owner() as usize == h).unwrap();
        g.st.set_residence(b, 0x3201);
        g.open(&assets, Menu::Residence(b));
        let texts: Vec<Vec<u8>> = g.popups.iter().filter_map(|p| if let Popup::Choice { acts, .. } = p { Some(acts) } else { None }).flatten().filter_map(|a| if let Action::ConfirmText(s, ..) = a { Some(s.clone()) } else { None }).collect();
        assert_eq!(texts.len(), 2);
        assert!(texts.iter().all(|s| !s.windows(2).any(|w| w == b"%d" || w == b"%l" || w == b"%4" || w == b"%2")));
        std::fs::remove_dir_all(&assets.user_saves).ok();
    }

    /// Castle/camp order book (rules-07 §3a) through the menu helpers: pick "Plündert", set men,
    /// confirm; the garrison gives the men up.
    #[test]
    fn real_troop_order_book() {
        use mad_sim::state::{KIND_CAMP, PROPS};
        let Some(assets) = test_assets() else { return };
        let mut g = Game::start(&assets, State::new_game(4, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        g.popups.clear();
        let h = g.me().unwrap();
        let camp = (0..PROPS).find(|&p| g.st.props[p].kind() == KIND_CAMP).unwrap();
        g.st.props[camp].set_owner(h as u8);
        g.st.props[camp].set_troops(0, 20);
        for _ in 0..3 {
            g.edit_troop_order(&assets, camp, 0, OrderField::TroopKind, 1); // none -> Überfallt -> Schickt -> Plündert
        }
        assert_eq!(g.st.props[camp].order(0)[0], 0x23);
        g.edit_troop_order(&assets, camp, 0, OrderField::Troop(0), 10);
        g.edit_troop_order(&assets, camp, 0, OrderField::Troop(0), 1);
        assert_eq!(g.st.props[camp].order(0)[3], 11);
        assert_eq!(g.st.props[camp].order(0)[7], mad_sim::economy::ST_POSSIBLE);
        g.sim(&assets, Command::ConfirmOrder { branch: camp as u8, slot: 0, on: true });
        assert_eq!((g.st.props[camp].order(0)[7], g.st.props[camp].troops(0)), (mad_sim::economy::ST_CONFIRMED, 9));
        g.open(&assets, Menu::Orders(camp));
        shot(&g, &assets, "camp_orders");
        g.popups.clear();
        let castle = (0..PROPS).find(|&p| g.st.props[p].kind() == mad_sim::state::KIND_CASTLE).unwrap();
        g.st.props[castle].set_owner(h as u8);
        let k = g.st.props[castle].loc() - 14;
        g.st.heap.alloc(17, 0, castle as u8, 4);
        g.open(&assets, Menu::CastleInfo(k));
        g.popups.clear();
        g.room = Some(Room::Burg(castle));
        g.st.props[castle].set_troops(0, 20);
        g.open(&assets, Menu::Orders(castle));
        shot(&g, &assets, "castle_screen");
        // held button (rules-26 §7.3): the ring toggle (544, 408) repeats, the order word does not
        assert!(g.repeatable(&assets, (560, 430)));
        assert!(!g.repeatable(&assets, (320, 345)));
        g.popups.clear();
        g.room = Some(Room::Burg(camp));
        g.open(&assets, Menu::Orders(camp));
        shot(&g, &assets, "camp_screen");
        g.room = None;
        g.popups.clear();
        for (n, m) in [("recruit", Menu::Recruit(castle)), ("manoeuvre", Menu::Manoeuvre(castle)), ("toll", Menu::Toll(castle)), ("expand", Menu::Expand(castle)), ("improve", Menu::Improve(castle, 1))] {
            g.open(&assets, m);
            shot(&g, &assets, n);
            g.popups.clear();
        }
        g.room = None;
        g.open(&assets, Menu::CastleInfo(k));
        shot(&g, &assets, "castle_info");
        g.popups.clear();
        g.open(&assets, Menu::CastleSell(k));
        shot(&g, &assets, "castle_sell");
        g.popups.clear();
        g.open(&assets, Menu::CastleInfo(g.st.props[camp].loc() - 14));
        shot(&g, &assets, "camp_info");
        g.popups.clear();
        let other = (0..PROPS).find(|&p| g.st.props[p].kind() == mad_sim::state::KIND_CASTLE && p != castle).unwrap();
        g.open(&assets, Menu::CastleInfo(g.st.props[other].loc() - 14));
        shot(&g, &assets, "castle_info_foreign");
        std::fs::remove_dir_all(&assets.user_saves).ok();
    }

    /// Of the §9.2 rectangles only Bezahlung and Handel overlap (143..255 × 350..385). The first
    /// table entry wins there (Bezahlung); the original's priority is [UNVERIFIED].
    /// rules-29 §2 against the live hover probes (screen coordinates).
    #[test]
    fn plot_map_hits() {
        let Some(assets) = test_assets() else { return };
        let g = Game::start(&assets, State::new_game(4, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap());
        for (p, want) in [((330, 240), Some(0)), ((300, 150), Some(10)), ((130, 290), Some(10)), ((470, 140), Some(11)), ((500, 250), Some(11)), ((460, 350), Some(12)), ((160, 380), None), ((215, 320), None), ((200, 100), None)] {
            assert_eq!(g.map_hit(&assets, p), want, "{p:?}");
        }
    }

    #[test]
    fn speech_gain_scales_around_silence() {
        let mut wav = vec![0u8; 44];
        wav[..4].copy_from_slice(b"RIFF");
        wav[34] = 8;
        wav[36..40].copy_from_slice(b"data");
        wav.extend([128, 138, 118, 250, 0]);
        assert_eq!(&speech_gain(wav, 160)[44..], [128, 144, 112, 255, 0]);
    }

    #[test]
    fn spot_table() {
        let overlaps = |a: [i32; 4], b: [i32; 4]| a[0] < b[0] + b[2] && b[0] < a[0] + a[2] && a[1] < b[1] + b[3] && b[1] < a[1] + a[3];
        for (i, a) in SPOTS.iter().enumerate() {
            for b in &SPOTS[i + 1..] {
                let known = (a.0, b.0) == (Spot::Bezahlung, Spot::Handel);
                assert_eq!(overlaps(a.3, b.3), known, "{:?} {:?}", a.0, b.0);
            }
        }
    }
}
