//! The words: one sheet per word in `assets/data/words/`, one file per compound in its
//! `compounds/` folder, and the target classes in `targets.ron`. The bake reads them,
//! fails on anything missing, and compiles the one table the code reads. It also writes
//! `docs/WORDS.md`, a page per word in plain language for the owner.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;
use vellmark_content::{Compound, EffectLook, Outcome, StringId, Typo, Word, WordTable};

use crate::BakeError;

/// Every effect the rules carry out. A sheet may name only these.
pub const EFFECTS: &[&str] = &[
    // The seven words.
    "open", "gape", "unfold", "light", "glow", "repel", "flare", "mend", "heal", "pacify", "still", "subdue", "ignite", "heat", "steam",
    "freeze", "chill", "slow", "douse", "frost", "sharpen", "thorns", "edge", "cutting",
    // Their typos.
    "bake", "answer", "gap", "shut", "turn", "gilt", "bite",
    // The compounds.
    "rapt", "grateful", "blaze", "glint", "rime",
];

pub const CLASSES: &[&str] = &["Access", "Light", "Temperature", "Weight", "Mind", "Integrity", "Concealment", "Sound", "Motion", "Size"];

#[derive(Deserialize)]
struct TargetClass {
    key: String,
    what: String,
}

#[derive(Deserialize, Clone)]
enum SheetOutcome {
    Do {
        effect: String,
        #[serde(default = "full")]
        strength: f32,
        #[serde(default)]
        preview: Option<String>,
    },
    Refuse(String),
}

fn full() -> f32 {
    1.0
}

#[derive(Deserialize, Clone, Copy, Debug)]
enum Class {
    Access,
    Light,
    Temperature,
    Weight,
    Mind,
    Integrity,
    Concealment,
    Sound,
    Motion,
    Size,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Look {
    colour: (f32, f32, f32),
    charge_glyphs: u32,
    flight_speed: f32,
    impact: String,
    wave_speed: f32,
    lasting: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TypoSheet {
    word: String,
    caption: String,
    targets: BTreeMap<String, SheetOutcome>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Sheet {
    word: String,
    class: Class,
    ink: f32,
    motif: Vec<String>,
    caption: String,
    combat: String,
    on_weapon: SheetOutcome,
    effect: Look,
    targets: BTreeMap<String, SheetOutcome>,
    typo: TypoSheet,
    /// What the sorts change. The Showcase has none yet; the field must still be there.
    sorts: Vec<(String, String)>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CompoundSheet {
    name: String,
    words: (String, String),
    caption: String,
    targets: BTreeMap<String, SheetOutcome>,
}

const IMPACTS: &[&str] = &["burst", "creep", "knit", "crack", "drift", "drop", "thin"];

/// A note name such as `D4` or `F#5` as a MIDI number.
pub fn note(name: &str) -> Option<u8> {
    let mut chars = name.chars();
    let letter = chars.next()?;
    let base = match letter {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };
    let rest: String = chars.collect();
    let (shift, octave) = if let Some(o) = rest.strip_prefix('#') { (1, o) } else if let Some(o) = rest.strip_prefix('b') { (-1, o) } else { (0, rest.as_str()) };
    let octave: i32 = octave.parse().ok()?;
    u8::try_from((octave + 1) * 12 + base + shift).ok()
}

pub struct BakedWords {
    pub table: WordTable,
    /// `docs/WORDS.md`.
    pub page: String,
}

pub fn bake_words(dir: &Path, strings: &BTreeMap<String, String>) -> Result<BakedWords, BakeError> {
    let read = |path: &Path| std::fs::read_to_string(path).map_err(|e| BakeError(format!("{}: {e}", path.display())));
    let targets_file = dir.join("targets.ron");
    let targets: Vec<TargetClass> = ron::from_str(&read(&targets_file)?).map_err(|e| BakeError(format!("{}: {e}", targets_file.display())))?;
    let classes: Vec<String> = targets.iter().map(|t| t.key.clone()).collect();
    let string = |file: &Path, key: &str| -> Result<StringId, BakeError> {
        if strings.contains_key(key) {
            Ok(StringId::from_key(key))
        } else {
            Err(BakeError(format!("{}: no string `{key}` in assets/text", file.display())))
        }
    };
    let outcome = |file: &Path, what: &str, o: &SheetOutcome| -> Result<Outcome, BakeError> {
        match o {
            SheetOutcome::Do { effect, strength, preview } => {
                if !EFFECTS.contains(&effect.as_str()) {
                    return Err(BakeError(format!("{}: {what}: no effect called `{effect}`", file.display())));
                }
                let preview = preview.clone().unwrap_or_else(|| format!("preview.{effect}"));
                Ok(Outcome::Effect { effect: effect.clone(), strength: *strength, preview: string(file, &preview)? })
            }
            SheetOutcome::Refuse(line) => Ok(Outcome::Refusal(string(file, line)?)),
        }
    };
    // Every class, and no other, in the class order of `targets.ron`.
    let every = |file: &Path, what: &str, map: &BTreeMap<String, SheetOutcome>| -> Result<Vec<Outcome>, BakeError> {
        if let Some(stray) = map.keys().find(|k| !classes.contains(k)) {
            return Err(BakeError(format!("{}: {what}: no target class `{stray}`", file.display())));
        }
        classes
            .iter()
            .map(|c| {
                let o = map.get(c).ok_or_else(|| BakeError(format!("{}: {what} has no result or refusal for `{c}`", file.display())))?;
                outcome(file, &format!("{what} on {c}"), o)
            })
            .collect()
    };

    let mut words = Vec::new();
    for file in crate::sorted_files(dir, "ron")? {
        if file.file_name().is_some_and(|n| n == "targets.ron") || file.parent().is_some_and(|p| p.ends_with("compounds")) {
            continue;
        }
        let sheet: Sheet = ron::from_str(&read(&file)?).map_err(|e| BakeError(format!("{}: {e}", file.display())))?;
        let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
        if !sheet.word.eq_ignore_ascii_case(stem) || sheet.word != sheet.word.to_ascii_uppercase() {
            return Err(BakeError(format!("{}: the word `{}` must be in capitals and named by its file", file.display(), sheet.word)));
        }
        let motif: Vec<u8> = sheet.motif.iter().map(|n| note(n).ok_or_else(|| BakeError(format!("{}: `{n}` is not a note", file.display())))).collect::<Result<_, _>>()?;
        if !(3..=5).contains(&motif.len()) {
            return Err(BakeError(format!("{}: a motif has three to five notes", file.display())));
        }
        if !IMPACTS.contains(&sheet.effect.impact.as_str()) {
            return Err(BakeError(format!("{}: no impact called `{}`", file.display(), sheet.effect.impact)));
        }
        if !sheet.sorts.is_empty() {
            return Err(BakeError(format!("{}: sorts are not in the Showcase", file.display())));
        }
        let look = EffectLook {
            colour: [sheet.effect.colour.0, sheet.effect.colour.1, sheet.effect.colour.2],
            charge_glyphs: sheet.effect.charge_glyphs,
            flight_speed: sheet.effect.flight_speed,
            impact: sheet.effect.impact.clone(),
            wave_speed: sheet.effect.wave_speed,
            lasting: sheet.effect.lasting.clone(),
        };
        words.push(Word {
            word: sheet.word.clone(),
            class: format!("{:?}", sheet.class),
            ink: sheet.ink,
            motif,
            caption: string(&file, &sheet.caption)?,
            combat: string(&file, &sheet.combat)?,
            weapon: outcome(&file, "on a held weapon", &sheet.on_weapon)?,
            look,
            outcomes: every(&file, &sheet.word, &sheet.targets)?,
            typo: Typo { word: sheet.typo.word.clone(), caption: string(&file, &sheet.typo.caption)?, outcomes: every(&file, &sheet.typo.word, &sheet.typo.targets)? },
        });
    }
    // No two motifs share their intervals: each word is known by ear.
    for (i, a) in words.iter().enumerate() {
        for b in &words[i + 1..] {
            let steps = |m: &[u8]| m.windows(2).map(|w| w[1] as i32 - w[0] as i32).collect::<Vec<_>>();
            if steps(&a.motif) == steps(&b.motif) {
                return Err(BakeError(format!("the motifs of {} and {} share their intervals", a.word, b.word)));
            }
        }
    }

    let mut compounds = Vec::new();
    let compound_dir = dir.join("compounds");
    if compound_dir.is_dir() {
        for file in crate::sorted_files(&compound_dir, "ron")? {
            let c: CompoundSheet = ron::from_str(&read(&file)?).map_err(|e| BakeError(format!("{}: {e}", file.display())))?;
            let class_of = |w: &str| words.iter().find(|x| x.word == w).map(|x| x.class.clone());
            let (a, b) = (class_of(&c.words.0), class_of(&c.words.1));
            match (a, b) {
                (Some(a), Some(b)) if a != b => {}
                (Some(_), Some(_)) => return Err(BakeError(format!("{}: a compound's two words are of different classes", file.display()))),
                _ => return Err(BakeError(format!("{}: both words of a compound need sheets", file.display()))),
            }
            compounds.push(Compound {
                name: c.name.clone(),
                caption: string(&file, &c.caption)?,
                words: [c.words.0.clone(), c.words.1.clone()],
                outcomes: every(&file, &c.name, &c.targets)?,
            });
        }
    }
    let table = WordTable { classes, words, compounds };
    let page = words_page(&table, &targets, strings);
    Ok(BakedWords { table, page })
}

/// `docs/WORDS.md`: each word in plain language, from its sheet.
fn words_page(t: &WordTable, targets: &[TargetClass], strings: &BTreeMap<String, String>) -> String {
    let text = |id: StringId| strings.iter().find(|(k, _)| StringId::from_key(k) == id).map(|(_, v)| v.as_str()).unwrap_or("?");
    let says = |o: &Outcome| match o {
        Outcome::Effect { effect, strength, preview } => {
            let s = if (*strength - 1.0).abs() > 1.0e-3 { format!(", strength {strength}") } else { String::new() };
            format!("{} *({effect}{s})*", text(*preview))
        }
        Outcome::Refusal(line) => format!("Refuses: \"{}\"", text(*line)),
    };
    let names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let mut out = String::from(
        "# The words\n\nWritten by the bake from `assets/data/words/`. Never edit this page: change the sheets,\nand bake. It is here for the owner to read and correct.\n\n",
    );
    out.push_str("## Target classes\n\n| Class | What |\n|---|---|\n");
    for c in targets {
        out.push_str(&format!("| {} | {} |\n", c.key, c.what));
    }
    for w in &t.words {
        let motif: Vec<String> = w.motif.iter().map(|n| format!("{}{}", names[(*n % 12) as usize], *n as i32 / 12 - 1)).collect();
        out.push_str(&format!("\n## {}\n\n{} word, {} Ink. Motif: {}. In a fight: {}\n\nOn a held weapon: {}\n\n", w.word, w.class, w.ink, motif.join(" "), text(w.combat), says(&w.weapon)));
        out.push_str(&format!("| Target | {} | Its typo, {} |\n|---|---|---|\n", w.word, w.typo.word));
        for (i, c) in t.classes.iter().enumerate() {
            out.push_str(&format!("| {c} | {} | {} |\n", says(&w.outcomes[i]), says(&w.typo.outcomes[i])));
        }
    }
    out.push_str("\n## Compounds\n");
    for c in &t.compounds {
        out.push_str(&format!("\n### {}: {} and {}\n\n| Target | Result |\n|---|---|\n", c.name, c.words[0], c.words[1]));
        for (i, class) in t.classes.iter().enumerate() {
            out.push_str(&format!("| {class} | {} |\n", says(&c.outcomes[i])));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sheet_missing_a_target_class_fails_the_bake() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let strings = crate::strings::bake_language(&repo.join("assets/text/en")).unwrap().text;
        // The real sheets bake.
        assert!(bake_words(&repo.join("assets/data/words"), &strings).is_ok());
        // OPEN's sheet without its line for stone does not.
        let dir = std::env::temp_dir().join(format!("vellmark-words-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(repo.join("assets/data/words/targets.ron"), dir.join("targets.ron")).unwrap();
        let open = std::fs::read_to_string(repo.join("assets/data/words/open.ron")).unwrap();
        let first = open.find("\"stone\":").unwrap();
        let end = first + open[first..].find('\n').unwrap() + 1;
        std::fs::write(dir.join("open.ron"), format!("{}{}", &open[..first], &open[end..])).unwrap();
        let err = bake_words(&dir, &strings).err().expect("the bake fails").0;
        assert!(err.contains("no result or refusal for `stone`"), "{err}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn notes_are_read() {
        assert_eq!(note("C4"), Some(60));
        assert_eq!(note("D4"), Some(62));
        assert_eq!(note("F#5"), Some(78));
        assert_eq!(note("Bb3"), Some(58));
        assert_eq!(note("H2"), None);
    }
}
