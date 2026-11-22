//! Modern options ("Extras"): ours only; the defaults are the game as before. Kept in `prefs.txt` next to our saves, never in
//! CONFIG.DAT or GAMEn.DAT, so the original still reads every file we write and the default game stays the original.

use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    De,
    En,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prefs {
    pub fullscreen: bool,
    /// Scale by whole multiples only (sharp pixels, black borders; on by default, as before the option).
    pub integer_scale: bool,
    /// Logic ticks per real tick: 1 = original speed, up to 4.
    pub speed: u8,
    /// Autosave every n years into the rolling history; 0 = off.
    pub autosave_years: u8,
    /// Number of rolling history saves kept.
    pub history: u8,
    /// "Rückgängig": the turn end can be taken back once.
    pub undo: bool,
    pub tooltips: bool,
    /// Warnings before debt seizure, loans due and similar.
    pub warnings: bool,
    /// Language of our own texts (the original's texts stay as in the user's game files).
    pub lang: Lang,
    /// Production helper on the branch screen: best ratio, means needed, expected yield.
    pub production_help: bool,
    /// House rule: debt stage 5 always goes to the jury instead of rolling for the last branch.
    pub debt_mercy: bool,
    /// House rule: production orders up to 255 workers (the order byte) instead of 40.
    pub more_workers: bool,
    /// New game: ask for the game number (fixed seed, daily game) before the player setup.
    pub seed_choice: bool,
    /// "Alles verkaufen" at the Kontor: one action sells every branch's stock locally.
    pub sell_all: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            fullscreen: false,
            integer_scale: true,
            speed: 1,
            autosave_years: 0,
            history: 10,
            undo: false,
            tooltips: false,
            warnings: false,
            lang: Lang::De,
            production_help: false,
            debt_mercy: false,
            more_workers: false,
            seed_choice: false,
            sell_all: false,
        }
    }
}

impl Prefs {
    pub fn load(dir: &Path) -> Self {
        let mut p = Self::default();
        let Ok(s) = std::fs::read_to_string(dir.join("prefs.txt")) else { return p };
        for (k, v) in s.lines().filter_map(|l| l.split_once('=')).map(|(k, v)| (k.trim(), v.trim())) {
            let b = v == "1";
            let n = v.parse().unwrap_or(0);
            match k {
                "fullscreen" => p.fullscreen = b,
                "integer_scale" => p.integer_scale = b,
                "speed" => p.speed = n.clamp(1, 4),
                "autosave_years" => p.autosave_years = n,
                "history" => p.history = n.max(1),
                "undo" => p.undo = b,
                "tooltips" => p.tooltips = b,
                "warnings" => p.warnings = b,
                "lang" => p.lang = if v == "en" { Lang::En } else { Lang::De },
                "production_help" => p.production_help = b,
                "debt_mercy" => p.debt_mercy = b,
                "more_workers" => p.more_workers = b,
                "seed_choice" => p.seed_choice = b,
                "sell_all" => p.sell_all = b,
                _ => {}
            }
        }
        p
    }

    pub fn save(&self, dir: &Path) -> std::io::Result<()> {
        let b = |v: bool| if v { 1 } else { 0 };
        let s = format!(
            "fullscreen={}\ninteger_scale={}\nspeed={}\nautosave_years={}\nhistory={}\nundo={}\ntooltips={}\nwarnings={}\nlang={}\nproduction_help={}\ndebt_mercy={}\nmore_workers={}\nseed_choice={}\nsell_all={}\n",
            b(self.fullscreen),
            b(self.integer_scale),
            self.speed,
            self.autosave_years,
            self.history,
            b(self.undo),
            b(self.tooltips),
            b(self.warnings),
            if self.lang == Lang::En { "en" } else { "de" },
            b(self.production_help),
            b(self.debt_mercy),
            b(self.more_workers),
            b(self.seed_choice),
            b(self.sell_all),
        );
        std::fs::create_dir_all(dir)?;
        std::fs::write(dir.join("prefs.txt"), s)
    }

    /// Our own UI text in the chosen language.
    pub fn tr<'a>(&self, de: &'a str, en: &'a str) -> &'a str {
        if self.lang == Lang::En { en } else { de }
    }

    /// The sim's house rules from these options.
    pub fn house(&self) -> mad_sim::state::House {
        mad_sim::state::House { debt_mercy: self.debt_mercy, max_workers: if self.more_workers { 255 } else { 40 } }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let dir = std::env::temp_dir().join(format!("mad-prefs-{}", std::process::id()));
        let p = Prefs { speed: 3, autosave_years: 5, lang: Lang::En, debt_mercy: true, seed_choice: true, ..Prefs::default() };
        p.save(&dir).unwrap();
        assert_eq!(Prefs::load(&dir), p);
        std::fs::remove_dir_all(&dir).ok();
        assert_eq!(Prefs::load(&dir), Prefs::default());
    }
}
