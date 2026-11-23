//! Tactical battle engine (docs/re/rules-07-castles-bandits-combat.md §5.2, §5.4, §5.6-§5.13).
//! Pure grid simulation over one `SZENARIO.DAT` field; setup and outcome live in `fight.rs`.
//! All units are computer-controlled (the original's ESC mode), and only ESC-mode RNG draws are
//! made (rules-07 §5.11 recommendation): presentation-only `rand` calls are skipped.

use crate::rng::Rng;

pub const W: i32 = 30;
pub const H: i32 = 46;
const EMPTY: u8 = 0xFF;
const BLOCK: u8 = 0xFE; // obstacle, rubble or a reserved destination
const WALL: u8 = 0xFB;
const MARK2: u8 = 0xFD; // right half of a goods marker (banner slot)

pub const DX: [i32; 9] = [0, 1, 1, 1, 0, -1, -1, -1, 0];
pub const DY: [i32; 9] = [-1, -1, 0, 1, 1, 1, 0, -1, 0];
/// Neighbour order of the path search (`0x858C0`).
const NB: [usize; 8] = [0, 4, 6, 2, 7, 1, 5, 3];
/// Musket rays (`0x85000` / `0x85100`), 32 directions x 4 steps (§5.10).
const RAY: [([i32; 4], [i32; 4]); 32] = [
    ([-1, -1, -1, -1], [-1, -1, -1, -1]),
    ([-1, -1, 0, -1], [-1, -1, -1, -1]),
    ([-1, 0, -1, 0], [-1, -1, -1, -1]),
    ([0, 0, -1, 0], [-1, -1, -1, -1]),
    ([0, 0, 0, 0], [-1, -1, -1, -1]),
    ([0, 0, 1, 0], [-1, -1, -1, -1]),
    ([1, 0, 1, 0], [-1, -1, -1, -1]),
    ([1, 1, 0, 1], [-1, -1, -1, -1]),
    ([1, 1, 1, 1], [-1, -1, -1, -1]),
    ([1, 1, 1, 1], [-1, -1, 0, -1]),
    ([1, 1, 1, 1], [-1, 0, -1, 0]),
    ([1, 1, 1, 1], [0, 0, -1, 0]),
    ([1, 1, 1, 1], [0, 0, 0, 0]),
    ([1, 1, 1, 1], [0, 0, 1, 0]),
    ([1, 1, 1, 1], [1, 0, 1, 0]),
    ([1, 1, 1, 1], [1, 1, 0, 1]),
    ([1, 1, 1, 1], [1, 1, 1, 1]),
    ([1, 1, 0, 1], [1, 1, 1, 1]),
    ([1, 0, 1, 0], [1, 1, 1, 1]),
    ([0, 0, 1, 0], [1, 1, 1, 1]),
    ([0, 0, 0, 0], [1, 1, 1, 1]),
    ([0, 0, -1, 0], [1, 1, 1, 1]),
    ([-1, 0, -1, 0], [1, 1, 1, 1]),
    ([-1, -1, 0, -1], [1, 1, 1, 1]),
    ([-1, -1, -1, -1], [1, 1, 1, 1]),
    ([-1, -1, -1, -1], [1, 1, 0, 1]),
    ([-1, -1, -1, -1], [1, 0, 1, 0]),
    ([-1, -1, -1, -1], [0, 0, 1, 0]),
    ([-1, -1, -1, -1], [0, 0, 0, 0]),
    ([-1, -1, -1, -1], [0, 0, -1, 0]),
    ([-1, -1, -1, -1], [-1, 0, -1, 0]),
    ([-1, -1, -1, -1], [-1, -1, 0, -1]),
];
/// Animation lengths by man type and state 0..6 (`0x855C8`); 0 = unused.
const ANIM: [[i32; 7]; 12] = [
    [1, 8, 7, 6, 7, 0, 0],
    [1, 14, 7, 6, 6, 0, 0],
    [1, 0, 0, 6, 8, 14, 0],
    [1, 8, 8, 6, 6, 0, 0],
    [1, 8, 8, 6, 6, 0, 0],
    [1, 8, 6, 6, 7, 0, 0],
    [1, 14, 7, 6, 6, 0, 0],
    [1, 0, 0, 6, 8, 14, 0],
    [1, 7, 8, 6, 10, 0, 10],
    [1, 8, 7, 6, 7, 0, 0],
    [1, 10, 7, 6, 6, 0, 0],
    [1, 10, 7, 6, 6, 0, 0],
];
pub const MOVES: [i32; 12] = [6, 5, 6, 7, 6, 6, 5, 6, 9, 6, 6, 6];
const HP_BASE: [i32; 12] = [40, 30, 20, 60, 70, 40, 30, 20, 40, 70, 30, 30];
/// Default tactic by type (`0x857D8`).
fn default_tactic(t: u8) -> u8 {
    match t {
        2 | 7 => 1,
        4 | 9 => 8,
        _ => 0,
    }
}
/// Unit class (`0x85847`): 0..4 by type mod 5 for 0..9, 5 for carters and sailors.
pub fn class(t: u8) -> u8 {
    if t >= 10 { 5 } else { t % 5 }
}
fn is_cannon(t: u8) -> bool {
    t == 2 || t == 7
}
/// Computer tactic weights (`0x84E4E`) by side and strength.
const WEIGHTS: [[(u32, u32, u32); 5]; 2] = [
    [(20, 30, 50), (50, 20, 30), (80, 20, 0), (90, 10, 0), (100, 0, 0)],
    [(70, 20, 10), (60, 30, 10), (80, 10, 10), (80, 0, 20), (50, 0, 50)],
];

#[derive(Clone, Debug)]
pub struct Man {
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    /// hit points at deployment (for the condition percentage of the tactic menu)
    pub max_hp: i32,
    pub cond: i32,
    pub typ: u8,
    pub part: u8,
    pub side: u8,
    /// troop slot 0..3 of the participant's order/garrison, 4 leader, 0xFF other
    pub slot: u8,
    pub unit: usize,
    state: u8,
    frames: i32,
    frame_idx: i32,
    moves: i32,
    /// target man; `Some(self)` = bomber fleeing his bomb
    target: Option<usize>,
    dest: Option<(i32, i32)>,
    pub fallen: bool,
    pub captured: bool,
    /// participant of the man who captured him (rules-42 §7.3: the prisoner's +0x10 at the fall's end)
    pub captor: u8,
    pub left: bool,
    marker_hit: bool,
    cannon: Option<usize>,
    op: (i32, i32),
    manned: bool,
    fired: bool,
    shots: i32,
    /// 0..7 (N, NE, ...); presentation only, no rule reads it
    facing: u8,
}

#[derive(Clone, Debug)]
struct Obj {
    typ: u8,
    x: i32,
    y: i32,
    part: u8,
    side: u8,
    frames: i32,
    frame_idx: i32,
    target: (i32, i32),
    facing: u8,
    alive: bool,
}

#[derive(Clone, Debug)]
pub struct Unit {
    pub typ: u8,
    pub part: u8,
    pub side: u8,
    pub tactic: u8,
    pub men: Vec<usize>,
    score: i32,
}

pub struct Battle {
    /// field 0..4 (rules-07 §5.1)
    pub field: usize,
    terrain: Vec<u8>,
    men_layer: Vec<u8>,
    obj_layer: Vec<Option<usize>>,
    pub men: Vec<Man>,
    objs: Vec<Obj>,
    pub units: Vec<Unit>,
    influence: Vec<i32>,
    pub idle: i32,
    ambience: u32,
    /// the battle song drawn at setup (group 0x1B)
    pub song: u8,
    pending_fx: Vec<Fx>,
    /// captured goods markers per participant (`0xDB800`)
    pub banners: [i32; 4],
    /// prisoners per participant (`0xDB804`)
    pub prisoners: [i32; 4],
    pub bombard: i32,
    pub wagons_lost: i32,
    pub markers_lost: i32,
    pub undecided: bool,
    pub held: i32,
    /// Picture of the fight, one entry per frame tick (`record` must be set before `fight`).
    pub record: bool,
    pub trace: Vec<Snap>,
    /// trace frames already handed out by `take_new_frames`
    shown: usize,
    /// Participants under human control (0 attacker owner, 1 defender owner; helpers never).
    pub human: [bool; 4],
}

/// One man in a [`Snap`]; (tx, ty) is the cell he is walking to (equal to (x, y) otherwise).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnapMan {
    pub x: i8,
    pub y: i8,
    pub tx: i8,
    pub ty: i8,
    pub typ: u8,
    pub side: u8,
    /// participant whose colour he is drawn in (`0xDB7FC[+0x10]`): a prisoner shows his captor's (rules-42 §7.3)
    pub part: u8,
    pub state: u8,
    pub facing: u8,
    pub frame: u8,
    /// walk progress 0..=255
    pub progress: u8,
    pub fallen: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnapObj {
    pub typ: u8,
    pub x: i8,
    pub y: i8,
    pub tx: i8,
    pub ty: i8,
    pub facing: u8,
    pub frame: u8,
    /// flight progress 0..=255
    pub progress: u8,
    pub side: u8,
    /// owner participant: cannons take its colour (rules-51 §1)
    pub part: u8,
}

/// A sound-worthy moment of a frame that is not visible in the positions (presentation only).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fx {
    /// a man took damage (scream)
    Hit,
    /// a bomb or cannon ball exploded
    Explosion,
    /// the crowd/wind ambience roll fired (rules-07 §5.11)
    Ambience,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Snap {
    pub men: Vec<SnapMan>,
    pub objs: Vec<SnapObj>,
    pub fx: Vec<Fx>,
}

/// Frame cap of a recording (about an hour of game time at 20 frames per second).
const TRACE_CAP: usize = 72_000;

/// Tactic availability masks (`0x84E38`, rules-07 §5.8): bit `side + 6`, bit `class`, bit `field + 8`.
const TACTIC_MASK: [u16; 11] = [0x1FFB, 0x1FC4, 0x1C44, 0x077B, 0x07BB, 0x187B, 0x18BB, 0x1FFB, 0x1FFF, 0x1FFB, 0x1FFF];

/// One group of the tactic menu: the units of one troop type of a participant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Group {
    pub typ: u8,
    pub men: u8,
    /// average hit points as a percentage of the deployment value
    pub strength: u8,
    pub tactic: u8,
    /// bit t set = tactic t may be chosen
    pub allowed: u16,
}

fn idx(x: i32, y: i32) -> usize {
    (y * W + x) as usize
}
fn inside(x: i32, y: i32) -> bool {
    (0..W).contains(&x) && (0..H).contains(&y)
}

impl Battle {
    /// `cells` = the 46 x 30 bytes of one field of SZENARIO.DAT.
    pub fn new(field: usize, cells: &[u8]) -> Self {
        let terrain = cells.to_vec();
        let men_layer = terrain
            .iter()
            .map(|&c| match c & 0x1F {
                1 => BLOCK,
                6 => WALL,
                _ => EMPTY,
            })
            .collect();
        Self {
            field,
            terrain,
            men_layer,
            obj_layer: vec![None; (W * H) as usize],
            men: vec![],
            objs: vec![],
            units: vec![],
            influence: vec![0; (W * H) as usize],
            idle: 10,
            ambience: 0,
            song: 0x10,
            pending_fx: vec![],
            banners: [0; 4],
            prisoners: [0; 4],
            bombard: 0,
            wagons_lost: 0,
            markers_lost: 0,
            undecided: false,
            held: 0,
            record: false,
            trace: vec![],
            shown: 0,
            human: [false; 4],
        }
    }

    fn code(&self, x: i32, y: i32) -> u8 {
        self.terrain[idx(x, y)] & 0x1F
    }
    fn is_edge(&self, x: i32, y: i32) -> bool {
        self.code(x, y) == 2
    }
    fn is_entrance(&self, x: i32, y: i32) -> bool {
        self.code(x, y) == 3
    }
    fn free(&self, x: i32, y: i32) -> bool {
        inside(x, y) && self.men_layer[idx(x, y)] == EMPTY && self.obj_layer[idx(x, y)].is_none()
    }
    fn walkable(&self, x: i32, y: i32) -> bool {
        self.free(x, y) && !self.is_edge(x, y)
    }
    fn first_entrance(&self) -> Option<(i32, i32)> {
        (0..W * H).map(|i| (i % W, i / W)).find(|&(x, y)| self.is_entrance(x, y))
    }
    fn living(&self, m: &Man) -> bool {
        !m.fallen && !m.left && m.typ < 12
    }

    // ---- deployment (§5.4) ----

    fn anchor(&self, code: u8, side: u8) -> Option<(i32, i32, i32)> {
        let cand = (0..W * H).map(|i| (i % W, i / W)).filter(|&(x, y)| self.code(x, y) == code && self.free(x, y));
        let score = |x: i32, y: i32| (y - 20) * (y - 20) + 4 * (x - 15) * (x - 15);
        // rules-40 §30: every type on side 1, field >= 2 goes to the cell nearest the first entrance
        // ((30, 46) if none); the unit score stays the far-from-centre maximum of all candidates.
        let (ex, ey) = self.first_entrance().unwrap_or((W, H));
        let mut best: Option<(i32, i32, i32)> = None;
        let mut close: Option<(i32, i32, i32)> = None;
        for (x, y) in cand {
            let s = score(x, y);
            if best.is_none_or(|b| s > b.2) {
                best = Some((x, y, s));
            }
            let e = (x - ex) * (x - ex) + (y - ey) * (y - ey);
            if close.is_none_or(|c| e < c.2) {
                close = Some((x, y, e));
            }
        }
        let (bx, by, s) = best?;
        let (x, y, _) = if side == 1 && self.field >= 2 { close? } else { (bx, by, 0) };
        Some((x, y, s))
    }

    #[allow(clippy::too_many_arguments)]
    fn add_man(&mut self, x: i32, y: i32, typ: u8, part: u8, side: u8, slot: u8, training: i32, unit: usize) -> usize {
        let hp = if typ >= 20 { 100 } else { HP_BASE[typ as usize] + training / 4 };
        let i = self.men.len();
        self.men.push(Man {
            x,
            y,
            hp,
            max_hp: hp,
            cond: training,
            typ,
            part,
            side,
            slot,
            unit,
            state: 0,
            frames: 0,
            frame_idx: 0,
            moves: 0,
            target: None,
            dest: None,
            fallen: false,
            captured: false,
            captor: 0,
            left: false,
            marker_hit: false,
            cannon: None,
            op: (x, y),
            manned: false,
            fired: false,
            shots: training / 20 + 1,
            facing: 0,
        });
        self.men_layer[idx(x, y)] = i as u8;
        i
    }

    fn add_obj(&mut self, typ: u8, x: i32, y: i32, part: u8, side: u8, frames: i32) -> usize {
        let i = self.objs.len();
        self.objs.push(Obj { typ, x, y, part, side, frames, frame_idx: 0, target: (x, y), facing: 0, alive: true });
        if typ != 17 {
            self.obj_layer[idx(x, y)] = Some(i);
        }
        i
    }

    /// `deploy(field, type, n, participant, side, training)`.
    pub fn deploy(&mut self, typ: u8, mut n: u32, part: u8, side: u8, slot: u8, training: i32) {
        while n > 0 && self.men.len() < 200 && self.units.len() < 100 {
            let code = match typ {
                20 => 4,
                21 => 5,
                _ => 8 + class(typ) + 6 * side,
            };
            let Some((ax, ay, score)) = self.anchor(code, side) else { return };
            let u = self.units.len();
            self.units.push(Unit { typ, part, side, tactic: default_tactic(typ), men: vec![], score });
            let d = (self.terrain[idx(ax, ay)] >> 5) as usize;
            let line = (self.terrain[idx(ax, ay)] & 0xE0) + 7;
            let step = if is_cannon(typ) { 2 } else { 1 };
            let dir = (d + 2) % 8;
            let (mut x, mut y) = (ax, ay);
            let mut placed = 0;
            loop {
                if is_cannon(typ) {
                    let (cx, cy) = (x + DX[dir], y + DY[dir]);
                    // rules-40 §31: the line stops; the rest goes on at a new anchor (§33).
                    if !self.free(cx, cy) {
                        break;
                    }
                    let m = self.add_man(x, y, typ, part, side, slot, training, u);
                    // every man and his cannon face the anchor's direction (rules-51 §3)
                    self.men[m].facing = d as u8;
                    let c = self.add_obj(16, cx, cy, part, side, 0);
                    self.objs[c].facing = d as u8;
                    self.men[m].cannon = Some(c);
                    self.units[u].men.push(m);
                } else {
                    let m = self.add_man(x, y, typ, part, side, slot, training, u);
                    self.men[m].facing = d as u8;
                    self.units[u].men.push(m);
                    // rules-40 §32: a goods marker's right cell takes 0xFD unconditionally.
                    if typ == 20 {
                        self.men_layer[idx(x + 1, y)] = MARK2;
                    }
                }
                placed += 1;
                n -= 1;
                if n == 0 || placed == 10 || typ >= 20 {
                    break;
                }
                let (nx, ny) = (x + step * DX[dir], y + step * DY[dir]);
                if !inside(nx, ny) || self.terrain[idx(nx, ny)] != line || !self.free(nx, ny) {
                    // rules-40 §33: men left over start a new unit at a new anchor.
                    break;
                }
                x = nx;
                y = ny;
            }
            if placed == 0 {
                self.units.pop();
                return;
            }
        }
    }

    /// Act order (rules-40 §34): ascending score, ties later-deployed first. Goods markers and
    /// wagons are not in the list.
    fn act_order(&self) -> Vec<usize> {
        let mut v: Vec<usize> = (0..self.units.len()).filter(|&u| self.units[u].typ < 12).collect();
        v.sort_by_key(|&u| (self.units[u].score, std::cmp::Reverse(u)));
        v
    }

    // ---- path search (§5.8, §5.11) ----

    fn bfs(&self, goals: &[bool], pass: &dyn Fn(i32, i32) -> bool) -> Vec<u8> {
        let mut dist = vec![EMPTY; (W * H) as usize];
        let mut q = std::collections::VecDeque::new();
        for i in 0..(W * H) as usize {
            if goals[i] {
                dist[i] = 0;
                q.push_back(i);
            }
        }
        while let Some(i) = q.pop_front() {
            let (x, y) = (i as i32 % W, i as i32 / W);
            for &d in &NB {
                let (nx, ny) = (x + DX[d], y + DY[d]);
                if inside(nx, ny) && dist[idx(nx, ny)] == EMPTY && pass(nx, ny) && dist[i] < 0xFD {
                    dist[idx(nx, ny)] = dist[i] + 1;
                    q.push_back(idx(nx, ny));
                }
            }
        }
        dist
    }

    fn enemy_goals(&self, m: usize) -> Vec<bool> {
        let mut g = vec![false; (W * H) as usize];
        for o in &self.men {
            if self.living(o) && o.side != self.men[m].side {
                g[idx(o.x, o.y)] = true;
            }
        }
        g
    }

    /// `path(mode, man)` → (r, first-step direction, content of that neighbour cell).
    fn path(&self, mode: u8, m: usize) -> (u8, usize, u8) {
        let me = &self.men[m];
        let n = (W * H) as usize;
        let walk = |x: i32, y: i32| self.walkable(x, y);
        let (goals, dist) = match mode {
            1 | 2 => {
                let mut g = vec![false; n];
                for i in 0..n {
                    let (x, y) = (i as i32 % W, i as i32 / W);
                    let flag = if mode == 1 { self.is_edge(x, y) } else { self.is_entrance(x, y) };
                    if flag && (self.free(x, y) || (x, y) == (me.x, me.y)) {
                        g[i] = true;
                    }
                }
                let d = self.bfs(&g, &walk);
                (g, d)
            }
            4 => {
                let ed = self.bfs(&self.enemy_goals(m), &walk);
                let g: Vec<bool> = (0..n).map(|i| ed[i] == 4 && (walk(i as i32 % W, i as i32 / W) || i == idx(me.x, me.y))).collect();
                let d = self.bfs(&g, &|x, y| walk(x, y) && ed[idx(x, y)] < 4);
                (g, d)
            }
            6 => {
                let mut g = vec![false; n];
                g[idx(me.op.0, me.op.1)] = true;
                let d = self.bfs(&g, &walk);
                (g, d)
            }
            8 | 9 => {
                let mut g = vec![false; n];
                for o in &self.men {
                    if o.typ == 20 && !o.fallen {
                        let banner = self.banner_at(o.x + 1, o.y).map(|b| self.objs[b].side);
                        let goal = if mode == 8 { banner != Some(me.side) } else { banner.is_some_and(|s| s != me.side) };
                        if goal {
                            g[idx(o.x, o.y)] = true;
                        }
                    }
                }
                let d = self.bfs(&g, &walk);
                (g, d)
            }
            // rules-40 §35: mode 5 also blocks goal neighbours next to a same-side record of another unit
            5 => {
                let g = self.enemy_goals(m);
                let own = |x: i32, y: i32| {
                    inside(x, y) && {
                        let v = self.men_layer[idx(x, y)] as usize;
                        let man = v < self.men.len() && {
                            let o = &self.men[v];
                            o.side == me.side && o.unit != me.unit && !o.fallen && !o.left
                        };
                        man || self.obj_layer[idx(x, y)].is_some_and(|o| matches!(self.objs[o].typ, 12 | 16) && self.objs[o].side == me.side)
                    }
                };
                let mut blocked = vec![false; n];
                for i in (0..n).filter(|&i| g[i]) {
                    let (gx, gy) = (i as i32 % W, i as i32 / W);
                    for d in 0..8 {
                        let (ax, ay) = (gx + DX[d], gy + DY[d]);
                        if walk(ax, ay) && (0..8).any(|e| own(ax + DX[e], ay + DY[e])) {
                            blocked[idx(ax, ay)] = true;
                        }
                    }
                }
                let d = self.bfs(&g, &|x, y| walk(x, y) && !blocked[idx(x, y)]);
                (g, d)
            }
            _ => {
                let g = self.enemy_goals(m);
                let d = self.bfs(&g, &walk);
                (g, d)
            }
        };
        if goals[idx(me.x, me.y)] {
            return (0, 0, EMPTY);
        }
        let mut best = EMPTY;
        for &d in &NB {
            let (nx, ny) = (me.x + DX[d], me.y + DY[d]);
            if inside(nx, ny) && dist[idx(nx, ny)] < best {
                best = dist[idx(nx, ny)];
            }
        }
        if best == EMPTY {
            return (EMPTY, 0, EMPTY);
        }
        let r = best + 1;
        let d = *NB.iter().find(|&&d| {
            let (nx, ny) = (me.x + DX[d], me.y + DY[d]);
            inside(nx, ny) && dist[idx(nx, ny)] == best
        })
        .unwrap();
        (r, d, self.men_layer[idx(me.x + DX[d], me.y + DY[d])])
    }

    fn banner_at(&self, x: i32, y: i32) -> Option<usize> {
        if !inside(x, y) {
            return None;
        }
        self.obj_layer[idx(x, y)].filter(|&o| self.objs[o].typ == 12)
    }

    // ---- actions ----

    fn go_idle(&mut self, m: usize) {
        let me = &mut self.men[m];
        me.state = 0;
        me.moves = 0;
        me.frames = 1;
        me.frame_idx = 0;
    }

    /// Turns man `m` toward (x, y) for the picture.
    fn face(&mut self, m: usize, x: i32, y: i32) {
        let (dx, dy) = ((x - self.men[m].x).signum(), (y - self.men[m].y).signum());
        if let Some(d) = (0..8).find(|&d| DX[d] == dx && DY[d] == dy) {
            self.men[m].facing = d as u8;
        }
    }

    fn walk(&mut self, m: usize, d: usize, extra: i32) {
        let (x, y) = (self.men[m].x + DX[d], self.men[m].y + DY[d]);
        let ok = self.free(x, y) && (!self.is_edge(x, y) || self.units[self.men[m].unit].tactic == 10);
        if !ok {
            // rules-40 §36: dead in the original (path just checked the cell); kept as a safety net.
            self.go_idle(m);
            return;
        }
        self.men_layer[idx(x, y)] = BLOCK;
        let me = &mut self.men[m];
        me.dest = Some((x, y));
        me.facing = d as u8;
        me.state = 3;
        me.frames = ANIM[me.typ as usize][3] + extra;
        me.frame_idx = 0;
        me.moves -= 1;
    }

    fn strike(&mut self, m: usize, t: usize) {
        self.face(m, self.men[t].x, self.men[t].y);
        let ty = self.men[m].typ as usize;
        let me = &mut self.men[m];
        me.state = 1;
        me.frames = ANIM[ty][1].max(1);
        me.frame_idx = 0;
        me.target = Some(t);
        // a captive's fall is not interrupted (rules-42 §7.3 converts him when it ends)
        if !is_cannon(self.men[t].typ) && self.men[t].typ < 12 && !self.men[t].captured {
            let tt = self.men[t].typ as usize;
            let tm = &mut self.men[t];
            tm.state = 2;
            tm.frames = ANIM[tt][2];
            tm.frame_idx = 0;
        }
    }

    fn fall(&mut self, t: usize) {
        let ty = self.men[t].typ as usize;
        let tm = &mut self.men[t];
        tm.hp = 0;
        tm.fallen = true;
        tm.state = 4;
        tm.frames = if ty < 12 { ANIM[ty][4] } else { 0 };
        tm.frame_idx = 0;
        tm.target = None;
    }

    fn set_idle_passive(&mut self, t: usize) {
        let tm = &mut self.men[t];
        tm.state = 0;
        tm.frames = 0;
    }

    fn adjacent_enemy(&self, content: u8) -> Option<usize> {
        let c = content as usize;
        (c < self.men.len()).then_some(c)
    }

    fn musket(&mut self, m: usize, d: usize, rng: &mut Rng) -> bool {
        let _ = rng;
        let a0 = (4 * d as i32 + 4) % 32;
        let me = self.men[m].clone();
        for k in 0..32i32 {
            let a = ((a0 + 32 + (1 - 2 * (k % 2)) * ((k + 1) / 2)) % 32) as usize;
            let (mut x, mut y) = (me.x, me.y);
            for j in 0..4 {
                x += RAY[a].0[j];
                y += RAY[a].1[j];
                if !(1..W - 1).contains(&x) || !(1..H - 1).contains(&y) {
                    break;
                }
                let v = self.men_layer[idx(x, y)];
                if v == EMPTY {
                    continue;
                }
                let t = v as usize;
                if t < self.men.len() && self.men[t].side != me.side && self.living(&self.men[t]) {
                    self.strike(m, t);
                    return true;
                }
                break;
            }
        }
        false
    }

    fn plant_banner(&mut self, m: usize, marker: usize) {
        let (bx, by) = (self.men[marker].x + 1, self.men[marker].y);
        let (part, side) = (self.men[m].part, self.men[m].side);
        match self.banner_at(bx, by) {
            None if inside(bx, by) => {
                let b = self.add_obj(12, bx, by, part, side, 6);
                self.objs[b].part = part;
            }
            Some(b) => {
                self.banners[self.objs[b].part as usize] -= 1;
                self.objs[b].part = part;
                self.objs[b].side = side;
                self.objs[b].frames = 6;
            }
            None => return,
        }
        self.banners[part as usize] += 1;
        self.go_idle(m);
    }

    /// Melee/musket resolution when a strike's animation has ended (§5.9 step 1).
    fn resolve_strike(&mut self, m: usize, t: usize, rng: &mut Rng) {
        self.men[m].target = None;
        self.go_idle(m);
        if self.men[t].fallen || self.men[t].typ >= 12 || self.men[t].left || self.men[t].captured {
            return;
        }
        let cm = self.men[m].cond;
        if self.units[self.men[m].unit].tactic == 7 {
            if (rng.n((5 * self.men[t].cond + 1) as u32) as i32) < cm {
                // rules-42 §7.3: fall with HP untouched; still an enemy goal until the fall ends
                let ty = self.men[t].typ as usize;
                let captor = self.men[m].part;
                let tm = &mut self.men[t];
                tm.state = 4;
                tm.frames = ANIM[ty][4];
                tm.frame_idx = 0;
                tm.target = None;
                tm.captured = true;
                tm.captor = captor;
                self.prisoners[self.men[m].part as usize] += 1;
            } else {
                self.set_idle_passive(t);
            }
            return;
        }
        let hit = (rng.n(100) as i32) < cm;
        let dmg = hit as i32 * rng.n((self.men[m].hp + 1) as u32) as i32;
        if dmg > 0 {
            self.idle = 10;
            self.fx(Fx::Hit);
        }
        if dmg < self.men[t].hp {
            self.men[t].hp -= dmg;
            self.set_idle_passive(t);
        } else {
            self.fall(t);
        }
    }

    /// §5.9 / §5.12 step logic of one man whose animation has run out.
    fn step(&mut self, m: usize, rng: &mut Rng) {
        let me = self.men[m].clone();
        if let Some(t) = me.target {
            if t != m {
                self.resolve_strike(m, t, rng);
                return;
            }
            let (r, d, _) = self.path(4, m);
            if r == EMPTY || r == 0 {
                self.go_idle(m);
            } else {
                self.walk(m, d, 0);
            }
            if self.men[m].moves <= 0 {
                self.men[m].target = None;
            }
            return;
        }
        let tactic = self.units[me.unit].tactic;
        match tactic {
            1 | 2 => self.cannon_step(m, rng),
            0 | 7 => {
                let (r, d, c) = self.path(if me.typ == 8 { 5 } else { 0 }, m);
                if matches!(me.typ, 1 | 6) && tactic == 0 && (1..=3).contains(&r) {
                    if !self.musket(m, d, rng) {
                        self.go_idle(m);
                    }
                } else if r == EMPTY || r == 0 {
                    self.go_idle(m);
                } else if r >= 2 {
                    self.walk(m, d, 0);
                } else if me.typ == 8 && tactic == 0 && me.moves >= 3 {
                    let mm = &mut self.men[m];
                    mm.state = 6;
                    mm.frames = ANIM[8][6];
                    mm.frame_idx = 0;
                    mm.target = Some(m);
                } else if let Some(t) = self.adjacent_enemy(c) {
                    self.strike(m, t);
                } else {
                    self.go_idle(m);
                }
            }
            8 => {
                let (r, _, c) = self.path(0, m);
                match self.adjacent_enemy(c).filter(|_| r == 1 && !is_cannon(me.typ)) {
                    Some(t) => self.strike(m, t),
                    None => self.go_idle(m),
                }
            }
            9 | 10 => {
                let (r, d, _) = self.path(if tactic == 9 { 4 } else { 1 }, m);
                if r == EMPTY || r == 0 {
                    self.go_idle(m);
                } else {
                    self.walk(m, d, (tactic == 10 && r == 1) as i32);
                }
            }
            3..=6 => {
                let mode = match tactic {
                    3 => 8,
                    4 => 9,
                    _ => 2,
                };
                let (r, d, c) = self.path(mode, m);
                if r == 1 && mode != 2 {
                    match self.adjacent_enemy(c) {
                        Some(mk) if self.men[mk].typ == 20 => self.plant_banner(m, mk),
                        // the goal neighbour is the marker's right half (0xFD)
                        _ => {
                            let (mx, my) = (self.men[m].x + DX[d] - 1, self.men[m].y + DY[d]);
                            match inside(mx, my).then(|| self.men_layer[idx(mx, my)] as usize).filter(|&k| k < self.men.len() && self.men[k].typ == 20) {
                                Some(mk) => self.plant_banner(m, mk),
                                None => self.go_idle(m),
                            }
                        }
                    }
                } else if r != EMPTY && r != 0 {
                    self.walk(m, d, 0);
                } else {
                    let (r2, _, c2) = self.path(0, m);
                    match self.adjacent_enemy(c2).filter(|_| r2 == 1 && !is_cannon(me.typ)) {
                        Some(t) => self.strike(m, t),
                        None => self.go_idle(m),
                    }
                }
            }
            _ => self.go_idle(m),
        }
    }

    fn cannon_step(&mut self, m: usize, rng: &mut Rng) {
        let me = self.men[m].clone();
        let Some(c) = me.cannon.filter(|_| is_cannon(me.typ)) else {
            self.go_idle(m);
            return;
        };
        if me.fired {
            self.go_idle(m);
            let mm = &mut self.men[m];
            mm.manned = false;
            mm.fired = false;
            return;
        }
        let (r, d, _) = self.path(6, m);
        if !me.manned {
            // the condition roll is only drawn when the cannon can still be reached and has shots
            if r == EMPTY || me.shots == 0 || rng.n(100) as i32 >= me.cond {
                self.go_idle(m);
            } else if r > 0 {
                self.walk(m, d, 0);
            } else {
                let mm = &mut self.men[m];
                mm.state = 5;
                mm.frames = 14;
                mm.frame_idx = 0;
                mm.manned = true;
            }
            return;
        }
        if r == EMPTY {
            self.go_idle(m);
            return;
        }
        if r > 0 {
            self.walk(m, d, 0);
            return;
        }
        let (cx, cy, f) = (self.objs[c].x, self.objs[c].y, self.objs[c].facing as usize);
        let (mut best, mut bk) = (-10000, 0);
        for k in 5..=24 {
            let (x, y) = (cx + k * DX[f], cy + 2 * k * DY[f]);
            if inside(x, y) && self.influence[idx(x, y)] > best {
                best = self.influence[idx(x, y)];
                bk = k;
            }
        }
        self.go_idle(m);
        if best > 0 {
            self.men[m].fired = true;
            self.men[m].shots -= 1;
            let (mut tx, mut ty) = (cx + bk * DX[f], cy + 2 * bk * DY[f]);
            if bk < 10 {
                tx += rng.n(3) as i32 - 1;
                ty += rng.n(3) as i32 - 1;
            } else {
                tx += rng.n(5) as i32 - 2;
                ty += rng.n(5) as i32 - 2;
            }
            tx = tx.clamp(1, 28);
            ty = ty.clamp(1, 44);
            let fl = ((24 * (cx - tx).abs()).max(12 * (cy - ty).abs()) / 10).max(1);
            let b = self.add_obj(17, cx, cy, me.part, me.side, fl);
            self.objs[b].target = (tx, ty);
        }
    }

    fn build_influence(&mut self, u: usize) {
        let (side, tactic) = (self.units[u].side, self.units[u].tactic);
        self.influence.iter_mut().for_each(|v| *v = 0);
        // rules-40 §37: living fighters (type < 12).
        let men: Vec<(i32, i32, i32)> =
            self.men.iter().filter(|m| self.living(m)).map(|m| (m.x, m.y, if m.side != side { m.hp } else { -m.hp })).collect();
        for (mx, my, s) in men {
            for y in my - 2..=my + 2 {
                for x in mx - 2..=mx + 2 {
                    if inside(x, y) {
                        self.influence[idx(x, y)] += s;
                    }
                }
            }
        }
        if tactic == 2 {
            for i in 0..(W * H) as usize {
                if self.men_layer[i] == WALL {
                    let (wx, wy) = (i as i32 % W, i as i32 / W);
                    for y in wy - 2..=wy + 2 {
                        for x in wx - 2..=wx + 2 {
                            if inside(x, y) {
                                self.influence[idx(x, y)] += 100;
                            }
                        }
                    }
                }
            }
        }
    }

    // ---- explosions (§5.13) ----

    fn explode(&mut self, o: usize, rng: &mut Rng) {
        self.fx(Fx::Explosion);
        if self.objs[o].typ == 17 {
            let (tx, ty) = self.objs[o].target;
            self.objs[o].x = tx;
            self.objs[o].y = ty;
            let v = self.men_layer[idx(tx, ty)];
            self.objs[o].typ = if (0xFB..=0xFE).contains(&v) { 14 + rng.n(2) as u8 } else { 13 };
            // rules-42 §7.2: the explosion enters the object layer at the target cell
            self.obj_layer[idx(tx, ty)].get_or_insert(o);
        } else {
            self.objs[o].typ = 13;
        }
        rng.n(100);
        // rules-40 §38: table value 8 + 1.
        self.objs[o].frames = 9;
        self.objs[o].frame_idx = 0;
        let (x0, y0, side) = (self.objs[o].x, self.objs[o].y, self.objs[o].side);
        for k in 0..9 {
            let (mut x, y) = (x0 + DX[k], y0 + DY[k]);
            if !(1..W - 1).contains(&x) || !(1..H - 1).contains(&y) {
                continue;
            }
            if self.men_layer[idx(x, y)] == MARK2 {
                x -= 1;
            }
            let v = self.men_layer[idx(x, y)];
            if v == WALL {
                self.men_layer[idx(x, y)] = BLOCK;
                self.bombard += 1;
                continue;
            }
            let t = v as usize;
            if t >= self.men.len() {
                continue;
            }
            let tm = self.men[t].clone();
            if tm.typ == 20 || tm.typ == 21 {
                if tm.hp > 50 {
                    self.men[t].hp = 50;
                    if !self.men[t].marker_hit {
                        self.men[t].marker_hit = true;
                        if tm.typ == 20 { self.markers_lost += 1 } else { self.wagons_lost += 1 }
                    }
                }
                continue;
            }
            if tm.typ >= 12 || tm.fallen || tm.left {
                continue;
            }
            // rules-40 §38: the centre takes 100 on both sides; halving and dodge only off-centre
            let mut dmg = 100;
            if k != 8 {
                dmg = if k % 2 == 0 { rng.n(80) as i32 } else { rng.n(60) as i32 };
                if (rng.n(100) as i32) < self.men[t].cond {
                    dmg = 0;
                }
                if self.men[t].side == side {
                    dmg >>= 1;
                }
            }
            if dmg > 0 {
                self.idle = 10;
            }
            if self.men[t].hp <= dmg {
                self.fall(t);
            } else {
                self.men[t].hp -= dmg;
            }
        }
        // rules-40 §38: crater decal loop, drawn in ESC mode too; only its rand calls matter here
        let (px, py) = (24 * x0 - 28, 12 * y0 - 34);
        for dx in (-36).max(-px)..36.min(640 - px) {
            for dy in (-18).max(-py)..18.min(479 - py) {
                if rng.n(5) < 2 && rng.n(300) as i32 > dx * dx / 2 + dy * dy {
                    rng.n(2);
                }
            }
        }
    }

    // ---- frame loop (§5.11) ----

    fn advance(&mut self, rng: &mut Rng) -> bool {
        #[derive(Clone, Copy)]
        enum E {
            M(usize),
            O(usize),
        }
        // rules-40 §39: by row, then pass (0 men moving up, 1 men level, 2 objects, 3 men moving
        // down), then x; a walking man stays in his origin cell's row. A cannon ball in flight
        // stays at its launch cell (pass 2 of the launch row) until it lands (rules-42 §7.2).
        let mut order: Vec<(i32, u8, i32, E)> = vec![];
        for (i, m) in self.men.iter().enumerate() {
            if !m.left && m.frames > 0 {
                let pass = match m.facing {
                    _ if m.state != 3 || m.frame_idx == 0 => 1,
                    0 | 1 | 7 => 0,
                    3..=5 => 3,
                    _ => 1,
                };
                order.push((m.y, pass, m.x, E::M(i)));
            }
        }
        for (i, o) in self.objs.iter().enumerate() {
            if o.alive && o.frames > 0 {
                order.push((o.y, 2, o.x, E::O(i)));
            }
        }
        order.sort_by_key(|e| (e.0, e.1, e.2));
        let active = !order.is_empty();
        let mut k = 0;
        while k < order.len() {
            let e = order[k].3;
            k += 1;
            match e {
                E::M(i) => {
                    let m = &mut self.men[i];
                    // rules-40 §39: walk-sound roll at walk frames 2 and 5, in ESC mode too
                    if m.state == 3 && matches!(m.frame_idx, 2 | 5) {
                        rng.n(100);
                    }
                    m.frame_idx += 1;
                    m.frames -= 1;
                    if m.state == 6 && m.frame_idx == 6 {
                        // facing is not tracked; 18 vs 19 only picks the sprite, both behave the same
                        let (x, y, p, s) = (m.x, m.y, m.part, m.side);
                        let b = self.add_obj(19, x, y, p, s, 18);
                        // rules-42 §7.1: the bomb runs in pass 2 of this row in the same frame
                        let at = order.partition_point(|e| (e.0, e.1, e.2) <= (y, 2, x));
                        order.insert(at.max(k), (y, 2, x, E::O(b)));
                    }
                    let m = &mut self.men[i];
                    if m.frames == 0 && m.state == 3 {
                        if let Some((x, y)) = m.dest.take() {
                            let (ox, oy) = (m.x, m.y);
                            m.x = x;
                            m.y = y;
                            m.state = 0;
                            self.men_layer[idx(ox, oy)] = EMPTY;
                            self.men_layer[idx(x, y)] = i as u8;
                            if self.is_edge(x, y) {
                                self.men[i].left = true;
                                self.men_layer[idx(x, y)] = EMPTY;
                            }
                        }
                    } else if m.frames == 0 && m.typ == 12 {
                        // end of the wave: frame 5 held (rules-43 §1)
                        m.frame_idx = 5;
                    } else if m.frames == 0 && m.state == 2 {
                        m.state = 0;
                    } else if m.frames == 0 && m.state == 4 && m.captured && !m.fallen {
                        // rules-42 §7.3: at the fall's end the captive becomes type 12 of the
                        // captor's side, keeping his old unit index (so path mode 5 counts him as a
                        // same-side record of another unit). He stays in his unit's slot (rules-71 §4):
                        // his HP keeps the unit activated, with no moves. His participant stays (colour
                        // is presentation; the end counts skip type 12, fight.rs books him as a loss).
                        m.typ = 12;
                        m.side ^= 1;
                        m.state = 0;
                        // drawn as a banner: one 6-frame wave, facing 0 (rules-43 §1)
                        (m.facing, m.frame_idx, m.frames) = (0, 0, 6);
                    }
                }
                E::O(i) => {
                    let o = &mut self.objs[i];
                    if o.typ != 18 && o.typ != 19 {
                        o.frame_idx += 1;
                    }
                    o.frames -= 1;
                    if o.frames == 0 {
                        match o.typ {
                            17..=19 => self.explode(i, rng),
                            13..=15 => {
                                self.objs[i].alive = false;
                                let c = idx(self.objs[i].x, self.objs[i].y);
                                if self.obj_layer[c] == Some(i) {
                                    self.obj_layer[c] = None;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        active
    }

    fn strength(&self, s: u8) -> u32 {
        let (mut a, mut b) = (0i64, 0i64);
        for m in self.men.iter().filter(|m| self.living(m)) {
            if m.side == s { a += m.hp as i64 } else { b += m.hp as i64 }
        }
        if a > 3 * b {
            0
        } else if a > 2 * b {
            1
        } else if 2 * a < b {
            3
        } else if 3 * a < b {
            4
        } else {
            2
        }
    }

    pub(crate) fn leader_alive(&self, side: u8) -> bool {
        self.men.iter().any(|m| m.side == side && matches!(m.typ, 4 | 9) && self.living(m))
    }

    /// Whether the tactic menu of participant `part` is open to a human (§5.8: his leader lives).
    pub fn controls(&self, part: u8) -> bool {
        self.human.get(part as usize).copied().unwrap_or(false) && self.leader_alive(part & 1)
    }

    /// Tactic ids allowed for a unit of type `typ` on `side` on this field (§5.8 masks).
    pub fn allowed(&self, typ: u8, side: u8) -> u16 {
        let need = (1u16 << (side + 6)) | (1 << class(typ)) | (1 << (self.field + 8));
        (0..11).filter(|&t| TACTIC_MASK[t] & need == need).fold(0, |a, t| a | 1 << t)
    }

    /// The groups of participant `part`, one per troop type present, in type order.
    pub fn groups(&self, part: u8) -> Vec<Group> {
        let mut typs: Vec<u8> = self.units.iter().filter(|u| u.part == part).map(|u| u.typ).collect();
        typs.sort_unstable();
        typs.dedup();
        typs.into_iter()
            .filter_map(|typ| {
                let us: Vec<&Unit> = self.units.iter().filter(|u| u.part == part && u.typ == typ).collect();
                let men: Vec<&Man> = us.iter().flat_map(|u| u.men.iter().map(|&i| &self.men[i])).filter(|m| self.living(m)).collect();
                if men.is_empty() {
                    return None;
                }
                let (hp, max): (i32, i32) = men.iter().fold((0, 0), |a, m| (a.0 + m.hp, a.1 + m.max_hp));
                Some(Group { typ, men: men.len().min(255) as u8, strength: (hp * 100 / max.max(1)).clamp(0, 100) as u8, tactic: us[0].tactic, allowed: self.allowed(typ, us[0].side) })
            })
            .collect()
    }

    /// Sets the tactic of every unit of type `typ` of participant `part` if the masks allow it.
    pub fn set_tactic(&mut self, part: u8, typ: u8, t: u8) {
        if t > 10 {
            return;
        }
        for u in 0..self.units.len() {
            let un = &self.units[u];
            if un.part == part && un.typ == typ && self.allowed(typ, un.side) & (1 << t) != 0 {
                self.units[u].tactic = t;
            }
        }
    }

    /// Trace frames recorded since the last call.
    pub fn take_new_frames(&mut self) -> Vec<Snap> {
        let v = self.trace[self.shown.min(self.trace.len())..].to_vec();
        self.shown = self.trace.len();
        v
    }

    /// §5.7 computer tactic choice.
    fn choose_tactic(&mut self, u: usize, rng: &mut Rng) {
        let unit = self.units[u].clone();
        let men: Vec<&Man> = unit.men.iter().map(|&i| &self.men[i]).filter(|m| self.living(m)).collect();
        let n = men.len() as i32;
        let hsum: i32 = men.iter().map(|m| m.hp).sum();
        let cannon = class(unit.typ) == 2;
        let mut shots: i32 = men.iter().map(|m| m.shots).sum();
        if cannon && rng.n(100) < 50 && men.iter().any(|m| (0..8).any(|d| self.enemy_at(m.x + DX[d], m.y + DY[d], m.side))) {
            shots = 0;
        }
        let p = if self.field <= 1 || cannon {
            -100
        } else {
            match self.first_entrance() {
                Some((ex, ey)) => self
                    .men
                    .iter()
                    .filter(|m| self.living(m) && !is_cannon(m.typ) && (ex - 5..ex + 5).contains(&m.x) && (ey - 5..ey + 5).contains(&m.y))
                    .map(|m| if m.side == 0 { 1 } else { -1 })
                    .sum(),
                None => -100,
            }
        };
        let t = &mut self.units[u].tactic;
        if hsum <= 10 * n || (cannon && shots == 0) {
            *t = 10;
        } else if hsum <= 20 * n && rng.n(100) < 50 {
            *t = 9;
        } else if unit.side == 1 && p >= -2 {
            *t = 6;
        } else if unit.side == 0 && p >= 0 {
            *t = 5;
        } else if rng.n(100) < 50 {
            let r = self.strength(unit.side) as usize;
            let (a, b0, c0) = WEIGHTS[unit.side as usize][r];
            let b = if self.field <= 2 { b0 } else { 0 };
            let c = if self.field >= 2 { c0 } else { 0 };
            let x = rng.n(a + b + c + 1);
            let t = &mut self.units[u].tactic;
            if x <= a {
                *t = if cannon {
                    1
                } else if rng.n(100) < 10 {
                    7
                } else {
                    0
                };
            } else if x <= a + b {
                if !cannon {
                    *t = if unit.side == 0 { 3 } else { 4 };
                }
            } else if cannon {
                if unit.side == 0 {
                    *t = 2;
                }
            } else {
                *t = if unit.side == 0 { 5 } else { 6 };
            }
        }
    }

    fn enemy_at(&self, x: i32, y: i32, side: u8) -> bool {
        inside(x, y) && {
            let v = self.men_layer[idx(x, y)] as usize;
            v < self.men.len() && self.men[v].side != side && self.living(&self.men[v])
        }
    }

    fn activate(&mut self, u: usize, rng: &mut Rng) {
        // the "men left" test sums the slots' HP (rules-71 §4): a converted prisoner keeps his, so a unit of
        // prisoners is still activated (its tactic is still chosen) and does nothing
        if !self.units[u].men.iter().any(|&m| self.living(&self.men[m]) || (self.men[m].captured && !self.men[m].fallen && !self.men[m].left)) {
            return;
        }
        // §5.7: a unit is computer-controlled unless its participant is a human whose leader lives
        if !self.controls(self.units[u].part) {
            self.choose_tactic(u, rng);
        }
        for &m in &self.units[u].men.clone() {
            let mm = &mut self.men[m];
            // MOVES ends after type 11; the bytes after it are 0, so a prisoner (type 12) never moves (rules-71 §4)
            mm.moves = MOVES.get(mm.typ as usize).copied().unwrap_or(0);
            mm.frames = 0;
            mm.target = None;
        }
        if matches!(self.units[u].tactic, 1 | 2) {
            self.build_influence(u);
        }
        // ponytail: iteration cap guards against a non-terminating frame loop; the original has none.
        for _ in 0..100_000 {
            for &m in &self.units[u].men.clone() {
                let me = &self.men[m];
                if me.frames == 0 && me.moves > 0 && self.living(me) {
                    self.step(m, rng);
                }
            }
            let more = self.advance(rng);
            self.snap();
            if !more {
                break;
            }
        }
    }

    fn fx(&mut self, f: Fx) {
        if self.record {
            self.pending_fx.push(f);
        }
    }

    /// One trace frame of the field as it stands (no draws); the defender-only picture under the intro captions
    /// (rules-50 §1.2).
    pub fn snap(&mut self) {
        if !self.record || self.trace.len() >= TRACE_CAP {
            return;
        }
        let men = self
            .men
            .iter()
            .filter(|m| !m.left)
            .map(|m| {
                let (tx, ty) = m.dest.unwrap_or((m.x, m.y));
                let total = m.frame_idx + m.frames;
                SnapMan {
                    x: m.x as i8,
                    y: m.y as i8,
                    tx: tx as i8,
                    ty: ty as i8,
                    typ: m.typ,
                    side: m.side,
                    part: if m.captured && m.typ == 12 { m.captor } else { m.part },
                    state: m.state,
                    facing: m.facing,
                    frame: m.frame_idx.clamp(0, 255) as u8,
                    progress: if m.dest.is_some() && total > 0 { (m.frame_idx * 255 / total) as u8 } else { 0 },
                    fallen: m.fallen,
                }
            })
            .collect();
        let objs = self
            .objs
            .iter()
            .filter(|o| o.alive)
            .map(|o| {
                let total = o.frame_idx + o.frames;
                SnapObj {
                    typ: o.typ,
                    x: o.x as i8,
                    y: o.y as i8,
                    tx: o.target.0 as i8,
                    ty: o.target.1 as i8,
                    facing: o.facing,
                    frame: o.frame_idx.clamp(0, 255) as u8,
                    progress: if total > 0 { (o.frame_idx * 255 / total) as u8 } else { 255 },
                    side: o.side,
                    part: o.part,
                }
            })
            .collect();
        let fx = std::mem::take(&mut self.pending_fx);
        self.trace.push(Snap { men, objs, fx });
    }

    /// (entrance squares held by attackers, living fighters per side)
    fn census(&self) -> (i32, [i32; 2]) {
        let mut alive = [0; 2];
        let mut held = 0;
        for m in self.men.iter().filter(|m| self.living(m)) {
            alive[m.side as usize & 1] += 1;
            if m.side == 0 && self.is_entrance(m.x, m.y) {
                held += 1;
            }
        }
        (held, alive)
    }

    /// The round loop. Returns when the battle ends.
    pub fn fight(&mut self, rng: &mut Rng) {
        self.music(rng);
        self.fight_with(rng, &mut |_, _, _| {});
    }

    /// `music(0x1B)` after the defender deployment draws its group's `rand(3)`, in ESC mode too (rules-39 §5,
    /// rules-50 §1.3 step 7).
    pub fn music(&mut self, rng: &mut Rng) {
        self.song = [0x10, 6, 3][rng.n(3) as usize];
    }

    /// `fight` (after [`Battle::music`]) with a hook called at the start of every round (round 0 = before the first), where
    /// the human sides set their tactics (§5.8, formats §11.5).
    /// `hook(b, round, start)`: at each round start (`start`), and after every unit's activation, where the original
    /// polls the tactics hotkeys and opens a queued menu (rules-44 §2).
    pub fn fight_with(&mut self, rng: &mut Rng, hook: &mut dyn FnMut(&mut Battle, u8, bool)) {
        self.idle = 10;
        self.snap();
        let mut round = 0u8;
        loop {
            hook(self, round, true);
            let r = round;
            round = round.saturating_add(1);
            for u in self.act_order() {
                // music(0x1C) before every unit's ambience step: rand(4), drawn in ESC mode too (rules-39 §5)
                rng.n(4);
                if self.ambience < 7 {
                    self.ambience += 1;
                } else if rng.n(100) < 20 {
                    self.ambience = 0;
                    self.fx(Fx::Ambience);
                }
                self.activate(u, rng);
                hook(self, r, false);
                let (held, alive) = self.census();
                self.held = held;
                if held >= 3 || alive[0] == 0 || alive[1] == 0 {
                    return;
                }
            }
            self.idle -= 1;
            if self.idle <= 0 {
                self.undecided = true;
                return;
            }
        }
    }

    /// living men per side (rules-59 §0: men0, men1)
    pub fn alive(&self) -> [i32; 2] {
        self.census().1
    }

    pub fn defender_alive(&self) -> bool {
        self.census().1[1] > 0
    }

    pub fn total_hp(&self, side: u8) -> i64 {
        self.men.iter().filter(|m| self.living(m) && m.side == side).map(|m| m.hp as i64).sum()
    }

    pub fn strength_of(&self, side: u8) -> u32 {
        self.strength(side)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// A plain test field: border code 2, open ground, two anchor lines facing each other.
    pub fn field() -> Vec<u8> {
        let mut c = vec![0u8; (W * H) as usize];
        for y in 0..H {
            for x in 0..W {
                if x < 1 || y < 1 || x >= W - 1 || y >= H - 1 {
                    c[idx(x, y)] = 2;
                }
            }
        }
        // attacker class 0 anchor (code 8) at (5, 5), line to the right (d = 0 → dir 2)
        c[idx(5, 5)] = 8;
        for x in 6..12 {
            c[idx(x, 5)] = 7;
        }
        // defender class 0 anchor (code 14) at (5, 12)
        c[idx(5, 12)] = 14;
        for x in 6..12 {
            c[idx(x, 12)] = 7;
        }
        c
    }

    /// rules-07 §5.4 deployment along a code-7 line; §5.6 a fight ends with one side gone or
    /// undecided, deterministically for a seed.
    #[test]
    fn rules07_s5_melee() {
        let run = |seed| {
            let mut b = Battle::new(0, &field());
            b.deploy(0, 5, 0, 0, 0, 80);
            b.deploy(5, 5, 1, 1, 0, 30);
            assert_eq!(b.men.len(), 10);
            assert_eq!((b.men[4].x, b.men[4].y), (9, 5));
            let mut rng = Rng::new(seed);
            b.fight(&mut rng);
            let (_, alive) = b.census();
            (alive, b.undecided, rng.s)
        };
        let r = run(3);
        assert_eq!(r, run(3));
        assert!(r.0[0] == 0 || r.0[1] == 0 || r.1);
        assert!(r.0[0] + r.0[1] < 10, "someone fell");
    }

    /// §5.8 availability masks: cannons shoot (1), others attack (0); storming only on fields 3, 4.
    #[test]
    fn rules07_s5_8_tactic_masks() {
        let b = Battle::new(0, &field());
        assert!(b.allowed(0, 0) & 1 != 0 && b.allowed(2, 0) & 1 == 0 && b.allowed(2, 0) & 2 != 0);
        assert!(b.allowed(0, 0) & (1 << 5) == 0 && b.allowed(0, 0) & (1 << 3) != 0 && b.allowed(0, 1) & (1 << 3) == 0);
        let b = Battle::new(3, &field());
        assert!(b.allowed(0, 0) & (1 << 5) != 0 && b.allowed(0, 1) & (1 << 6) != 0 && b.allowed(2, 0) & (1 << 2) != 0);
    }

    /// §5.8: a human side keeps the tactic it was given; "Flucht" (10) sends its men off the field.
    #[test]
    fn rules07_s5_8_human_tactics() {
        let mut f = field();
        f[idx(5, 8)] = 12; // attacker leader anchor (8 + class 4)
        let mut b = Battle::new(0, &f);
        b.deploy(0, 5, 0, 0, 0, 80);
        b.deploy(4, 1, 0, 0, 4, 80);
        b.deploy(5, 5, 1, 1, 0, 30);
        b.human[0] = true;
        let mut asked = vec![];
        let mut rng = Rng::new(3);
        b.music(&mut rng);
        b.fight_with(&mut rng, &mut |b, round, start| {
            if b.controls(0) && start {
                let g = b.groups(0);
                asked.push((round, g.len()));
                for g in g {
                    b.set_tactic(0, g.typ, 10);
                }
            }
        });
        assert_eq!(asked[0], (0, 2), "two groups at the first menu");
        assert!(b.men.iter().filter(|m| m.side == 0).all(|m| m.left || m.fallen), "everyone fled or fell");
    }

    /// §5.13: a bomb centre hit on a man of the other side does 100 damage.
    #[test]
    fn rules07_s5_13_explosion_centre() {
        let mut b = Battle::new(0, &field());
        b.deploy(0, 1, 0, 0, 0, 0);
        let o = b.add_obj(19, b.men[0].x, b.men[0].y, 1, 1, 1);
        let mut rng = Rng::new(1);
        b.explode(o, &mut rng);
        assert!(b.men[0].fallen);
    }

    /// rules-42 §7.1: a dropped bomb advances in the same frame; §7.3: a captive turns into a
    /// type-12 record of the captor's side, keeping his unit, when his fall ends.
    #[test]
    fn rules42_s7_bomb_and_captive() {
        let mut b = Battle::new(0, &field());
        b.deploy(0, 2, 0, 0, 0, 0);
        let m = &mut b.men[0];
        (m.state, m.frames, m.frame_idx) = (6, 5, 5);
        let m = &mut b.men[1];
        (m.state, m.frames, m.frame_idx, m.captured) = (4, 1, 0, true);
        b.advance(&mut Rng::new(1));
        assert_eq!(b.objs.last().map(|o| (o.typ, o.frames)), Some((19, 17)));
        let p = &b.men[1];
        assert_eq!((p.typ, p.side, p.unit, p.hp > 0), (12, 1, 0, true));
        // rules-71 §4: he keeps his unit slot; its next activation gives him no moves
        assert!(b.units[0].men.contains(&1));
        b.activate(0, &mut Rng::new(1));
        assert_eq!(b.men[1].moves, 0);
    }

    /// rules-40 §34: lowest score acts first; among equal scores the later-deployed unit first.
    #[test]
    fn rules40_s34_act_order() {
        let mut b = Battle::new(0, &field());
        for (typ, score) in [(0, 5), (0, 3), (0, 5), (20, 0)] {
            b.units.push(Unit { typ, part: 0, side: 0, tactic: 0, men: vec![], score });
        }
        assert_eq!(b.act_order(), vec![1, 2, 0]);
    }

    /// §5.8 path: r = 1 next to an enemy, direction toward it.
    #[test]
    fn rules07_s5_8_path_adjacent() {
        let mut b = Battle::new(0, &field());
        b.deploy(0, 1, 0, 0, 0, 50);
        let i = b.men.len();
        b.add_man(5, 6, 5, 1, 1, 0, 50, 0);
        let (r, d, c) = b.path(0, 0);
        assert_eq!((r, d, c as usize), (1, 4, i));
    }
}

#[cfg(test)]
mod real {
    use super::*;
    /// `$MAD_DATA`: a fight on each of the five real fields deploys both sides and costs men.
    #[test]
    fn rules07_s5_real_fields() {
        let Some(d) = std::env::var("MAD_DATA").ok().and_then(|p| std::fs::read(std::path::Path::new(&p).join("SZENARIO.DAT")).ok()) else { return };
        for f in 0..5 {
            let mut b = Battle::new(f, &d[f * 1380..(f + 1) * 1380]);
            if f <= 2 {
                b.deploy(20, 5, 1, 1, 0xFF, 0);
                b.deploy(10, 3, 1, 1, 0xFF, 50);
                b.deploy(21, 3, 1, 1, 0xFF, 0);
            } else {
                for k in 0..4 { b.deploy(k, 6, 1, 1, k, 50); }
                b.deploy(4, 1, 1, 1, 4, 87);
            }
            for k in 5..9 { b.deploy(k, 6, 0, 0, k - 5, 50); }
            b.deploy(9, 1, 0, 0, 4, 0);
            let n0: Vec<usize> = (0..2).map(|s| b.men.iter().filter(|m| m.side == s && m.typ < 12).count()).collect();
            let mut rng = Rng::new(5);
            b.fight(&mut rng);
            let n1: Vec<usize> = (0..2).map(|s| b.men.iter().filter(|m| m.side == s && b.living(m)).count()).collect();
            assert!(n0[0] > 0 && n0[1] > 0, "field {f} deploys both sides");
            if f >= 2 {
                // RE: every attacker anchor reaches the entrances without breaching (codes 3..5,
                // 7..19 walkable; 1, 2, 6 not)
                let clean = Battle::new(f, &d[f * 1380..(f + 1) * 1380]);
                let mut goals = vec![false; (W * H) as usize];
                for i in 0..(W * H) as usize {
                    goals[i] = clean.is_entrance(i as i32 % W, i as i32 / W);
                }
                let dist = clean.bfs(&goals, &|x, y| clean.walkable(x, y));
                for i in 0..(W * H) as usize {
                    if (8..14).contains(&(clean.terrain[i] & 0x1F)) {
                        assert!(dist[i] != EMPTY, "field {f}: attacker anchor {i} cut off");
                    }
                }
            }
            assert!(n1[0] + n1[1] < n0[0] + n0[1], "field {f}: someone falls");
        }
    }
}
