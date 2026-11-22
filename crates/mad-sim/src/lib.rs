//! Deterministic, UI-free simulation core for Mittelland AD.
//!
//! Built only from `docs/spec/` (handbook) and `docs/re/` (behaviour specs). `State` + `Command`
//! -> `Event`s; questions the original asks mid-processing (votes, ...) go through the `Choices`
//! callback so the core stays a plain function of its inputs. No wall clock, no floats, seeded
//! Watcom RNG only.

// Index loops mirror the original table walks (city c, good g, ...).
#![allow(clippy::needless_range_loop)]

pub mod ai;
pub mod auction;
pub mod backroom;
pub mod battle;
pub mod career;
pub mod castles;
pub mod debt;
pub mod director;
pub mod economy;
pub mod events;
pub mod family;
pub mod fight;
pub mod heap;
pub mod kontor;
pub mod laws;
pub mod missions;
pub mod privileges;
pub mod rng;
pub mod save;
pub mod siege;
pub mod soak;
pub mod social;
pub mod state;
pub mod subplots;
pub mod trials;
pub mod turn;
pub mod world;

pub use state::State;
pub use turn::Command;

/// The duel's presentation rolls (rules-22 §9): insulter, time/place line, the two answers, fog lines before and after,
/// and one taunt per half-exchange as (side 0 = insulter, line, x, y).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DuelScene {
    pub insulter: u8,
    pub place: u8,
    pub answers: [u8; 2],
    pub fog: u8,
    pub after: u8,
    pub taunts: Vec<(u8, u8, u16, u16)>,
    /// Effect samples (ms after the box closes, sample) of each taunt box (swings then hits).
    pub fx: Vec<Vec<(u16, u8)>>,
}

/// Something the UI may want to show. Persons are slot indices, places location ids.
/// An argument of a [`Event::WorldText`] line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WArg {
    /// A person (`%kNn`).
    P(u8),
    /// A place name (`%o`, or `%s` for a city name).
    Loc(u8),
    /// A text by id (`%s`).
    Id(u32),
    /// A good's name (`%s`).
    Good(u8),
    /// A number (`%d`, `%w`, `%l`).
    Num(i32),
    /// Entry `i` of the EXE word table at `addr` (`%s`): troop word `0xB601C`, pronoun `0xB6F68`.
    Word(u32, u8),
}

/// An EXE text (address) with its arguments, printed by the app.
pub type Line = (u32, Vec<WArg>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    TurnStart { person: u8, year: u16 },
    TurnSkipped { person: u8 },
    YearEnd { year: u16 },
    AllPlayersOut,
    GameOver,
    Harvest { good: u8, quality: u8 },
    Produced { branch: u8, good: u8, amount: u32 },
    SkillGain { good: u8, skill: u8 },
    /// `branch`: the shipment's source branch.
    Sold { branch: u8, good: u8, city: u8, qty: u16, revenue: i32 },
    /// Ours (ledger): one branch's share of the bill's wages, means and transport rows.
    BranchCosts { branch: u8, wages: i32, means: i32, transport: i32 },
    Arrived { branch: u8, good: u8, qty: u16 },
    Returned { branch: u8 },
    GivenAway { branch: u8, amount: u32 },
    TransportersBuilt { branch: u8, t: u8, n: u8 },
    StorageOffer { city: u8, offers: [(u16, bool); 3] },
    /// The bill's rows (EXE 0xC06D0..0xC0830): `troops`/`robbers` castle and camp pay, `spies`/`saboteurs` the type-6 and type-7 contracts.
    /// The human's cash at the end of the turn-end settlement: the turn-end report window's banner shows it (rules-60:
    /// the banner repaints on every cash change, so the window's last wait shows the cash after its last line).
    TurnEndCash { person: u8, cash: i32 },
    /// `cash`: the player's cash right after paying, which the bill page's banner shows (live A/B)
    Bill { wages: i32, troops: i32, robbers: i32, means: i32, transport: i32, toll: i32, directors: i32, tax: i32, tithe: i32, spies: i32, saboteurs: i32, interest: i32, insurance: i32, cash: i32 },
    /// Branches over the limit: emperor index (0..4), percent of the net worth taken, amount (rules-12 §3.1).
    BranchSurcharge { emperor: u8, percent: u8, amount: i32 },
    OfficeIncome { amount: i32 },
    /// Year-end privilege income of a human office holder: kind 0 turnover, 1 tithe, 2 indulgences (rules-40 §27).
    PrivilegeIncome { person: u8, kind: u8, amount: i32 },
    /// `gross` = the accumulator, `share` = the county's part, `vogt` goes to `vogt_by` (0xFF = none).
    /// A human Vogt gets his share of a computer-owned castle's toll (rules-12 §3.3, 0xC1D14).
    VogtToll { vogt: u8, amount: i32, castle: u8 },
    CastleIncome { castle: u8, amount: i32, gross: i32, share: i32, vogt: i32, vogt_by: u8 },
    CargoDamaged { owner: u8 },
    /// One transporter of a shipment lost on the way to `dest` (rules-23 §2); `kind` is the transporter type.
    TransporterLost { owner: u8, kind: u8, dest: u8 },
    /// A balloon / steam cart shipment bound for `old` goes to `new` instead.
    OffCourse { owner: u8, kind: u8, old: u8, new: u8 },
    /// End-of-turn hint (rules-02 §7.13): kind 0 transporter type `a` is worn in city `loc`, 1 no residence,
    /// 2 idle cash, 3 no heir in sight, 4 production order of good `b` in `loc` (`a` 0 too many workers,
    /// 1 too few, 2 crowded).
    Tip { kind: u8, a: u8, b: u8, loc: u8 },
    BlackmailEnded { by: u8 },
    /// `city`: location of a random own branch (the awarder's place, rules-27 §1); `song`: the fanfare drawn.
    /// `awarder`: the mayor / Vogt / Regent signing the charter (rules-41 §1), 0xFF for the Emperor's titles.
    Title { person: u8, title: u8, city: u8, song: u8, awarder: u8 },
    Certificate { person: u8, level: u8 },
    /// `by`: the computer appointers whose favour fell below 30, in appointer order.
    /// `rival`: the human whose current rival `person` is (0xFF none), the `b` of `%F` (rules-12 §1.6).
    Dismissed { person: u8, by: Vec<u8>, loc: u8, idx: u8, rival: u8 },
    Appointed { person: u8, loc: u8, idx: u8 },
    /// The tornado (divine kind 8): victim city `a`, city `b` that gains, the numbers of the text (`[victim, winner]` pairs).
    Tornado { a: u8, b: u8, n: u32, churches: [u32; 2], museums: [u32; 2], hospitals: [u32; 2] },
    /// An election shown to the players (rules-44 §3): `kind` 0 majority, 1 tie decided by lot (both displayed: a
    /// human elector or applicant), 2 single candidate without a vote (0xC1998), 3 hidden election won by a human's
    /// rival (0xC1978). `rivals`: per candidate, the human whose rival it is (0xFF none), from the year-start
    /// snapshot. `privileged`: a privileged candidate shut the others out (0xC183C). `religion`: the place
    /// picture's religion (the displayed election's `rand(2)`, else the winner's).
    Election { loc: u8, seat: u8, candidates: Vec<u8>, votes: Vec<(u8, u8)>, winner: u8, kind: u8, rivals: Vec<u8>, privileged: bool, religion: u8 },
    /// "Im nächsten Jahr werden neu gewählt" (rules-44 §3): (place, office index of the seat or group base) where a
    /// human has eligibility 2, places 13 down to 0, offices high to low.
    NextElections(Vec<(u8, u8)>),
    PersonReplaced { person: u8 },
    /// `name` is kept because the person is cleared right after.
    Bankrupt { person: u8, name: String },
    /// `nw`: net worth at death, `epitaph`: the deed bit 0..7 of the gravestone line, 8 = none.
    /// `heckles`: the mourners' lines (rules-17 §4.3) as [notary step 1..5, alignment, line, voice digit, gender].
    Died { person: u8, name: String, female: bool, nw: i32, epitaph: u8, heckles: Vec<[u8; 5]> },
    /// `kids`: born children (more than one: the fifth part goes to the others).
    Succession { person: u8, kids: u8 },
    /// `no_kids`: false when the estate went to the Erzbistum instead (rules-05 §7).
    PlayerOut { person: u8, name: String, no_kids: bool },
    MissionDone { person: u8, mission: u8 },
    Fine { person: u8, amount: i32 },
    /// `film`: the Informant film plays (rules-21 §4.2).
    SpyNotified { owner: u8, target: u8, film: bool },
    /// `msgbox 0x368E0` (rules-04 §5): an EXE text with its args in a framed OK window at `rect`; waits for a click.
    MsgBox { text: u32, args: Vec<WArg>, rect: [i32; 4] },
    Refused(&'static str),
    /// `cry`: the cradle sound rolls rand(5), rand(4) (rules-19 §2.1 step 5).
    ChildBorn { parent: u8, slot: u8, female: bool, cry: [u8; 2] },
    ChildDied { parent: u8, age: u8, name: String, female: bool },
    /// `text` 0..4 picks the gossip phrase (text id 2408 + text).
    Gossip { person: u8, text: u8 },
    MistressLeft { person: u8 },
    MistressDemand { person: u8, amount: i32 },
    /// reason 0 rival, 1 gifts too small, 2 canon veto
    CourtshipFailed { person: u8, target: u8, reason: u8 },
    /// `title`: the suitor's new title when the marriage raised it (rules-15 §7.1, text 0x9D5BE).
    Wedding { person: u8, spouse: u8, title: Option<u8>, feast: bool },
    /// The canon forbade the wedding (rules-15 §7.1 step 5).
    Vetoed { person: u8, target: u8, canon: u8 },
    /// Office-tree prices per person slot (rules-04 §2).
    TreePrices(Vec<u32>),
    /// `variant` picks the sentence, `lost` the dead, `adj` the damage word
    Disaster { city: u8, kind: u8, variant: u8, lost: u32, adj: u8 },
    /// a human's branches in the city were hit (`goods`: stock was lost)
    DisasterBranches { person: u8, goods: bool },
    DisasterInsured { person: u8, loss: i32 },
    /// Medicus text index 0..11 (rules-04b §4.3).
    Health { level: u8 },
    AuctionAnnounced { years: u8 },
    /// The herald's call one year before the auction (rules-12 §3.6); `song` is the fanfare group draw of sound 0x1E.
    Herald { song: u8 },
    /// The picture draw of a manoeuvre window opening (rules-27 §4): 0x199 + r.
    /// Prisoners after a battle (rules-38 §7): `joined` of `taken` stayed at castle or camp `prop`.
    /// `joined` is negative on an over-full property (quirk, rules-41 §4).
    Prisoners { person: u8, prop: u8, taken: u8, joined: i16 },
    /// The cost a privilege dialog shows (`Command::PrivilegeView`).
    PrivCost { code: u8, cost: i32 },
    /// The courted person's reaction to a gift (rules-39 §7): word index 0..14 of table 0xB5818.
    GiftReaction { target: u8, word: u8 },
    /// A human insults a computer person (rules-24 §1, box 0x9B789): taunt, voice, adverb rand(8), place rand(6).
    Insulted { insulter: u8, target: u8, rolls: [u8; 4] },
    ManoeuvrePic { pic: u16 },
    /// A song for the screen: mode 0 starts it always, 2 only when no song plays (rules-14 §4.1).
    Song { song: u8, mode: u8 },
    /// The city the indulgence offer names (rules-04b Ablass).
    IndulgenceCity { city: u8 },
    /// The Bewerbung window's rival table (rules-64 §2): (human, his rival or 0xFF), humans in slot order.
    ApplyRivals(Vec<(u8, u8)>),
    /// A line of the year-end window from an EXE text (rules-32): `text` is its address (or a text id
    /// below 0x10000), `args` fill its
    /// `%` codes in order.
    WorldText { text: u32, args: Vec<WArg> },
    /// A message box with an EXE text (money-lender and matchmaker refusals).
    Notice { text: u32 },
    /// The viewer's tax and tithe of this turn, from opening Spielstand (rules-28 §2).
    Scores { tax: i32, tithe: i32 },
    /// Costs at the end of a trial (rules-18 §11): who pays how much, whether the insurance refunds it.
    TrialCosts { plaintiff: u8, defendant: u8, judges: [u8; 3], payer: u8, amount: i32, insured: bool },
    /// `kind` is the lot record type (12 transporters, 14 residence, 16 castle); `loc` the receiving branch city
    /// (transporters, 0xFF without one), the residence city or the castle location; `word` the residence word.
    AuctionSold { lot: u8, winner: u8, price: i32, kind: u8, loc: u8, count: u8, typ: u8, word: u16 },
    /// `kind`: property kind hit (0 branch, 1 camp, 2 castle), `loc` its location.
    /// `rival`: a computer saboteur that is the human target's rival (0xC1BB4/0xC1C54).
    Sabotage { saboteur: u8, target: u8, kind: u8, loc: u8, damage: i32, film: bool, rival: bool },
    /// The doctor forbids a second duel in the year (EXE 0xBCC70).
    DuelForbidden,
    CastleSold { prop: u8, buyer: u8, price: i32 },
    /// The army's first turn end: text only (rules-21 §2).
    SiegeArrives { city: u8, besieger: u8, army: u16 },
    SiegeGoesOn { city: u8, besieger: u8 },
    /// `choice`: the human's pick (0 plunder, 1 tribute, 2 both), 3 for a computer; `tribute`: yearly amount and years.
    SiegeWon { besieger: u8, city: u8, choice: u8, tribute: (u32, u8) },
    SiegeCollapsed { city: u8, besieger: u8 },
    /// Relief army of `owner` against `besieger`'s siege; `pop` is the city's population (the human's reward),
    /// `song` the fanfare drawn for a success (7 or 0x0E, rules-14 §4.1 code 0x1E).
    Relief { city: u8, ok: bool, besieger: u8, owner: u8, pop: u32, song: u8 },
    ReliefNoSiege { city: u8, owner: u8 },
    /// Yearly tribute paid to the conquering besieger (rules-21 §2.4, text 0x9FAAB).
    Tribute { city: u8, amount: i32 },
    Seized { person: u8, goods: u32, transporters: u32, storage: u32 },
    /// A residence (or its construction site when `site`) seized at half value (rules-23 §3).
    SeizedResidence { city: u8, word: u16, site: bool },
    /// A whole branch seized; `redirect` is the city of the branch the shipments return to (rules-23 §3).
    SeizedBranch { prop: u8, city: u8, redirect: Option<u8> },
    /// `song`: the acquittal's fanfare drawn from group 0x1E, 0x0B for the tower (rules-15 §6.1 steps 6, 7).
    Jury { person: u8, guilty: u8, tower: bool, last_branch: bool, song: u8 },
    /// A battle could not be simulated because no `SZENARIO.DAT` is attached.
    BattlePending { code: u8, order: u16 },
    Battle { code: u8, attacker: u8, defender: u8, conquest: bool, undecided: bool, report: Box<fight::Report> },
    /// The call-off roll stopped an attack on castle/camp `prop` (rules-50 §2); no other effects.
    AttackDetected { prop: u8, attacker: u8 },
    RansomPaid { attacker: u8, amount: i32 },
    /// Sub-plot progress: kind = record type 32..49, step = counter (0xFF started, 0xFE lost).
    SubPlot { person: u8, kind: u8, step: u8 },
    /// A sub-plot message that needs values (docs/re/rules-13): `id` picks the text of the plot, `a`, `b`, `c` its numbers.
    PlotNote { kind: u8, id: u8, a: i32, b: i32, c: i32 },
    Cards { mine: u8, theirs: u8, x: u8 },
    /// The contest was announced (city and judge) or cancelled for lack of entries.
    ContestStart { city: u8, judge: u8, cancelled: bool },
    /// One judged discipline of the newspaper contest; `first`/`last` mark the opening and closing text.
    Contest { discipline: u8, good: u8, fee: i32, winners: Vec<u8>, city: u8, judge: u8, first: bool, last: bool },
    BranchBought { prop: u8, price: i32 },
    BranchSold { prop: u8, price: i32 },
    /// `kind`: 0 build, 1 extend, 2 renovate (rules-23 §7.2).
    ResidenceOrdered { branch: u8, cost: i32, kind: u8, word: u16, years: u8 },
    LoanDue { lender: u8, amount: i32 },
    Feast { host: u8, guests: u32, quality: u8, loc: u8, cost: i32, food_short: bool, drink_short: bool },
    Duel { winner: u8, loser: u8, office_lost: bool, scene: DuelScene },
    /// `city` and `with` (the person of kind 3) 0xFF when unused; `illness` indexes the illness names (kinds 1, 6, 9).
    Divine { kind: u8, city: u8, with: u8, illness: u8 },
    /// aux: good (kinds 0, 1), city (2, 3), city and besieger index (4)
    WorldFate { kind: u8, aux: [u8; 2] },
    /// `amount` = the player's cash change, `city` = location of the last property drawn (0xFF none).
    Fate { person: u8, kind: u8, amount: i32, city: u8, aux: [u8; 2], with: u8 },
    // aux: tavern or book index, good (kinds 11-13, 45, 46)
    /// sentence index (rules-04b §3.5 table) when guilty
    /// A trial about to be decided: `kind` 1 = inquisition, `votes[j]` true = the judge finds the defendant innocent.
    /// Option "Gericht bestechen": computer sides that invested in witnesses, then each side's bribe (rules-18 §3, §8).
    TrialMoney { invested: Vec<u8>, bribes: Vec<(u8, i32)> },
    /// Evidence word, per charge group (group, text roll, connector roll) and the defendant's plea 0..8 (rules-18 §4).
    TrialCharge { evidence: u32, groups: Vec<(u8, u8, u8)>, plea: u8 },
    /// Witness (side, voice 3..10, line, head 0..7) in order, then the prosecution's three and the defence's three line rolls (rules-18 §6, §7).
    TrialTalk { witnesses: Vec<(u8, u8, u8, u8)>, plea: [u8; 6] },
    Trial { plaintiff: u8, defendant: u8, judges: [u8; 3], kind: u8, loc: u8, votes: [bool; 3] },
    /// `office`: the defendant's (location, index) before the sentence, named by sentence 5 (rules-38 §9).
    Verdict { defendant: u8, guilty: bool, sentence: u8, fine: i32, office: (u8, u8) },
}

/// The answer to `Ask::AuctionTick` for the Esc key.
pub const AUCTION_ESC: u32 = 0xFFFF;

/// A question the original asks in the middle of processing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    /// Human elector picks a candidate index (rules-03 §4.6).
    /// `for_`: the appointer the elector votes for by blackmail (0xFF none; text 0xC18C4).
    /// A human elector votes (rules-15 §3.4 step 4), asked over the election screen: the election so far comes with
    /// it (place, seat, religion drawn for the picture, candidates' rival owners, privileged flag, earlier votes).
    Vote { elector: u8, candidates: Vec<u8>, for_: u8, loc: u8, seat: u8, religion: u8, rivals: Vec<u8>, privileged: bool, votes: Vec<(u8, u8)> },
    /// Matchmaker proposal: 0 accept (pay the fee), 1 decline (rules-05 §2).
    Match { target: u8, fee: i32 },
    /// Courtship year: gift 0..2 or 3 = "too expensive" (rules-05 §3).
    Gift { target: u8, prices: [i32; 3], names: [u8; 3] },
    /// Matchmaker while already courting: 0 give the courtship up, 1 keep courting (rules-05 §2.1).
    GiveUpCourtship { target: u8, city: u8 },
    /// Someone blackmailing the canon: 0 "untersagen lassen" (lets the wedding go ahead, quirk), 1 continue.
    BlackmailCanon { by: u8, canon: u8, suitor: u8, target: u8 },
    /// Human canon: 0 forbid the wedding, 1 allow.
    CanonVeto { canon: u8, suitor: u8, target: u8 },
    /// 0 "Ja!", 1 "eigentlich nicht", 2 hesitate (asks again).
    Vows { suitor: u8, target: u8, canon: Option<u8> },
    /// Wedding feast option 0..4 (4 = nobody).
    /// `title` = the title the suitor gains by the marriage (0x9D5BE is shown before the question).
    WeddingFeast { suitor: u8, spouse: u8, city: u8, prices: [i32; 4], title: Option<u8>, song: u8 },
    /// Trial investments of a human side: witness money W, then bribe money B (rules-04b §3.3).
    TrialWitnesses { person: u8 },
    TrialBribe { person: u8 },
    /// Human defendant's plea 0 ("Ich gestehe Alles!") .. 8 ("Lüge, alles Lüge!"); the original's default is 7.
    Plea,
    /// A blackmailer of a judge: 0 guilty, 1 innocent.
    BlackmailVerdict { by: u8, judge: u8 },
    /// Human judge: 0 guilty, 1 innocent.
    JudgeVote { judge: u8 },
    /// Human presiding judge picks the sentence 0..6.
    Sentence { defendant: u8 },
    /// One auction tick: 0 = no human presses, k = the k-th human (slot order) presses his hotkey, `AUCTION_ESC` Esc.
    /// `pic`: the lot's catalogue picture (transporter 0x15A.., residence 0x160 + level, castle 0x15C + location).
    /// `strip`: the auctioneer's frame segments played since the lot began as (bank, from, to), end exclusive, the ones
    /// from index `fresh` on being new since the last tick; `blow`: a hammer blow fell since the last tick (rules-25 §1).
    /// Bauwerk stiften (rules-38 §2.3): the five buildings' prices for `city`; answer 0..4 donates, 5 nothing.
    Building { city: u8, prices: [i32; 5] },
    /// `hammer`: the count of the blow shown (1..3), 0 without one; count 2 adds 200 ms (rules-42 §8).
    AuctionTick { lot: u8, price: i32, high: Option<u8>, pic: u16, strip: Vec<(u16, u8, u8)>, fresh: u8, blow: bool, hammer: u8 },
    /// A human's hotkey bid was refused (rules-12 §3.5): `text` is the EXE string (0xBABB0 no branch, 0xBAC1C already
    /// pressed, 0xBAC78 cash, 0xBACD8 no port, 0xBAD48 debtors' tower); the answer is a click.
    AuctionRefused { person: u8, text: u32, city: u8 },
    /// A text line of the auction hall that waits for a click (docs/re/rules-15 §1); `step` 0 opening (a = year, b = city),
    /// 1 lot intro, 2 sold (c = receiving city, d = price), 3 unsold, 4 end (a = 1 when lots were offered).
    AuctionLine { step: u8, pic: u16, kind: u8, seller: u8, winner: u8, a: i32, b: i32, c: i32, d: i32 },
    /// A catalogue page (answer is ignored, a click): `kind` 12/14/16; transporters a = count, b = type, c = condition;
    /// residence a = word, b = city; castle a = security class, b = location, c = troops, d = casemates.
    AuctionCatalog { n: u8, kind: u8, pic: u16, seller: u8, a: i32, b: i32, c: i32, d: i32, price: i32 },
    /// No human sells: attend the auction? 0 yes, 1 no.
    AttendAuction { person: u8 },
    /// Won a residence where one exists: 0 take the new one, 1 keep the old (rules-08 §1.3).
    ResidenceSwap { branch: u8, new_word: u16 },
    /// A computer owner offers its castle/camp: 0 buy, 1 decline (rules-07 §3).
    BuyCastle { prop: u8, price: i32 },
    /// A human buyer of a castle is asked to shake hands (EXE 0xBF4C0); 0 = yes.
    ConfirmCastleSale { buyer: u8 },
    /// Fechtunterricht: 0 accept the cost, 1 decline.
    FencingLesson { cost: i32 },
    /// The pre-fight window (rules-52): `lines` (EXE text address, args) print in order, then `q`. `frames`: the field
    /// since the last question (the attackers deployed).
    PreFight { code: u8, colours: [u8; 4], song: u8, frames: Vec<battle::Snap>, lines: Vec<Line>, q: fight::Parley },
    /// Human conqueror: 0 occupy the castle/camp, 1 leave it.
    /// rules-59 §2: the occupy question on page 2 of the report (page 1 = the verdict)
    OccupyCastle { code: u8, attacker: u8, defender: u8, loc: u8, verdict: u8 },
    /// Sub-plot dialog / spinner (see `subplots.rs` for the step numbering).
    /// `arg`: the first cosmetic value of the dialog's text where the spec needs one (docs/re/rules-13), else 0.
    Plot { kind: u8, step: u8, arg: i32, arg2: i32 },
    /// Newspaper contest: discipline bit mask 0..31 (0 = do not enter).
    /// `goods` are the five disciplines' goods, `paper` the newspaper index, `fee` the entry fee per discipline.
    ContestEntry { person: u8, fee: i32, goods: [u8; 5], paper: u8 },
    /// Money lender offer: 0 accept, 1 decline (rules-11 §4).
    LoanOffer { lender: u8, amount: i32, rate: u8, years: u8 },
    /// Director candidates with their fee %: index to hire, anything else declines (rules-11 §7).
    PickDirector { candidates: Vec<u8>, fees: Vec<u8> },
    /// Siege won: 0 plunder, 1 tribute, 2 both (rules-07 §6.3).
    SiegeWon { city: u8 },
    /// Continue the siege for `cost`: 0 yes, 1 no.
    ContinueSiege { city: u8, cost: i32 },
    /// Jury court bribe amount (rules-02 §10.3).
    /// The creditors' jury court (rules-15 §6.1): `county` 10..12, its castle picture is 0x193 + county.
    JuryBribe { person: u8, county: u8 },
    /// Insulted human: 0 accept the duel, 1 refuse (rules-08 §3).
    /// The insulter may take back an insult he already made (EXE 0xBCC10); 0 = apologise.
    Apologise { target: u8 },
    /// `rolls`: taunt rand(12), voice rand(2), reason rand(8) (computer insulter only), place rand(6) (rules-24 §1,
    /// rules-39 §2).
    AcceptDuel { insulter: u8, insulted: u8, rolls: [u8; 4] },
    /// Battle tactic menu of participant `part` before round `round` (rules-07 §5.8). `frames` is
    /// the picture since the previous menu. Answer: nibble k (bits 4k..4k+3) = 0 keep, else
    /// tactic + 1 for group k; `u32::MAX` = hand the rest of the battle to the computer (ESC).
    Tactics { part: u8, person: u8, round: u8, code: u8, colours: [u8; 4], groups: Vec<battle::Group>, frames: Vec<battle::Snap>, song: u8 },
    /// A battle with a human side: captions 1..3 and line 4 or the call-off text (rules-50 §1, §2) over
    /// `frames` (the defender-only field, frame 0 of the trace); the answer is ignored.
    BattleIntro { code: u8, colours: [u8; 4], song: u8, frames: Vec<battle::Snap>, intro: fight::Intro },
    /// A human lost the leader: the message is shown once, then no more tactic menus (rules-07 §5.8).
    LeaderLost { person: u8 },
    /// Continue after mission success? 0 yes, 1 no (rules-10 §5).
    /// `song`: the history screen's music (group 0x1D), 0 = none drawn (no history entries).
    ContinueAfterMission { person: u8, song: u8 },
}

pub trait Choices {
    fn ask(&mut self, q: &Ask) -> u32;
    /// The events of the command so far, at the point where the original shows them (the turn-end windows before
    /// the year end's questions). They stay in the returned list; a front end may show them early.
    fn show(&mut self, _ev: &[Event]) {}
}

/// Answers every question with its first option.
pub struct Auto;
impl Choices for Auto {
    fn ask(&mut self, _q: &Ask) -> u32 {
        0
    }
}
