//! The words a developer says that Whisper has never heard enough of.
//!
//! Dibur transcribes Hebrew, but a Hebrew speaker dictating to a coding agent
//! reaches for English every time they name something in their stack. The model
//! deals with that in one of two ways, and gets it wrong both times.
//!
//! **It writes Latin script and misspells the name.** `Claude` becomes `cloud`,
//! because "Supabase" and "pgvector" are not words it saw often in training.
//! This is handled by the existing fuzzy custom-word correction, which scores
//! on Soundex and edit distance; [`DEV_VOCABULARY`] simply gives it the words.
//!
//! **It writes the name phonetically in Hebrew letters.** "Claude Code" spoken
//! with a Hebrew accent comes out `קלוד קוד`, Vercel as `ורסל`, GitHub as
//! `גיטאב`. The fuzzy matcher cannot touch any of it: it discards non-ASCII
//! candidates, and Soundex is meaningless across scripts anyway. That is what
//! [`apply_hebrew_terms`] is for.
//!
//! Which of the two happens is not predictable. One recording contains
//! `GitHub` and `workflow` in Latin script alongside `סקילים` and `Cloud Code`
//! in the same breath, so both passes have to run on every transcription.
//!
//! Alongside the product names there is Claude Code's own vocabulary, taken
//! from the official glossary and spelled the way it spells them: `skill`,
//! `subagent`, `hook`, `plan mode`, `worktree`, `goal`. These matter more than
//! the brand names do. Claude Code recognises its own words, and a request for
//! a `סקיל` is a guess where a request for a `skill` is a skill.
//!
//! # What belongs in the table
//!
//! The two passes carry opposite risks, so they do not take the same words.
//!
//! The Latin pass gives every entry a phonetic net that also catches whatever
//! merely sounds like it. That is exactly what turns `cloud` into `Claude`, and
//! exactly why an ordinary English word does not belong there: Whisper already
//! spells `commit`, `deploy` and `refactor` correctly in Latin script, so
//! listing them is all risk and no gain.
//!
//! The Hebrew pass is a lookup against Hebrew letters, which no English word
//! can collide with, so it takes those same everyday words happily: `קומיט`
//! becomes `commit`. What it cannot take is a spelling that is also a real
//! Hebrew word. `קוד` and `סופר` are ordinary Hebrew, so a key must always be
//! the whole term: `קלודקוד` and `סופרבייס` are safe, `קוד` and `סופר` never
//! are.
//!
//! Nothing here ever emits a leading slash. `/goal` and `/compact` are real
//! commands, and text pasted into a prompt starting with one would invoke
//! them, so the words go in bare and the slash stays the typist's decision.

use std::collections::HashMap;

/// One name, with the Hebrew spellings the model produces for it.
pub struct Term {
    /// How the name should be written once corrected.
    pub canonical: &'static str,
    /// Hebrew renderings to replace with `canonical`. Written here as they are
    /// spoken; [`normalize_hebrew`] handles final letter forms, doubled vav and
    /// yod, niqqud and word joining, so only genuinely different spellings need
    /// listing. Empty when the name is never said in Hebrew letters.
    pub hebrew: &'static [&'static str],
    /// Whether the Latin fuzzy pass may correct toward this name.
    ///
    /// The two passes do not carry the same risk, so they do not take the same
    /// words. The Latin pass matches phonetically, so every name it knows also
    /// swallows whatever merely sounds like it; an ordinary English word there
    /// is all risk, because Whisper already spells `commit` and `deploy`
    /// correctly in Latin script. The Hebrew pass is a lookup against Hebrew
    /// letters, which no English word can collide with, so `קומיט` is free to
    /// become `commit`.
    pub latin_pass: bool,
}

/// A name only the Latin pass corrects, because it is never said in Hebrew
/// letters.
const fn latin_only(canonical: &'static str) -> Term {
    Term {
        canonical,
        hebrew: &[],
        latin_pass: true,
    }
}

/// A name both passes correct.
const fn term(canonical: &'static str, hebrew: &'static [&'static str]) -> Term {
    Term {
        canonical,
        hebrew,
        latin_pass: true,
    }
}

/// A name only the Hebrew pass corrects. For ordinary English words, which are
/// safe to recover from Hebrew letters and unsafe to hand a phonetic matcher.
const fn hebrew_only(canonical: &'static str, hebrew: &'static [&'static str]) -> Term {
    Term {
        canonical,
        hebrew,
        latin_pass: false,
    }
}

/// Terms corrected when `dev_vocabulary` is enabled.
///
/// Ordering is by theme for the benefit of whoever edits this next; neither
/// pass depends on it.
pub const DEV_VOCABULARY: &[Term] = &[
    // Claude and agent tooling. `Claude` earns its place in the Latin pass
    // despite colliding with `cloud`: a Hebrew speaker says `ענן` for the sky,
    // so a Latin-script `cloud` in the output means they said "Claude".
    term("Claude", &["קלוד", "קלאוד"]),
    term("Claude Code", &["קלוד קוד", "קלאוד קוד"]),
    term("Anthropic", &["אנתרופיק", "אנטרופיק"]),
    latin_only("Opus"),
    latin_only("Sonnet"),
    latin_only("Haiku"),
    term("Codex", &["קודקס"]),
    term("Cursor", &["קרסר", "קורסר"]),
    term("Copilot", &["קופיילוט"]),
    // Claude Code's own vocabulary, spelled as the official glossary spells
    // it. These are the words that have to survive dictation intact, because
    // they are what Claude Code recognises: asking for a `skill` gets a skill,
    // asking for a `סקיל` gets a guess.
    term("MCP", &["אמסיפי"]),
    term("skill", &["סקיל"]),
    term("skills", &["סקילים"]),
    term("subagent", &["סאב אייג'נט", "סאבאג'נט"]),
    term("subagents", &["סאב אייג'נטים"]),
    term("agent", &["אייג'נט", "אג'נט"]),
    term("agents", &["אייג'נטים", "אג'נטים"]),
    term("hook", &["הוק"]),
    term("hooks", &["הוקים"]),
    term("plugin", &["פלאגין"]),
    term("goal", &["גול"]),
    term("plan mode", &["פלאן מוד"]),
    term("checkpoint", &["צ'קפוינט"]),
    term("worktree", &["וורקטרי"]),
    term("artifact", &["ארטיפקט"]),
    term("teleport", &["טלפורט"]),
    term("compact", &["קומפקט"]),
    term("context", &["קונטקסט"]),
    term("context window", &["קונטקסט וינדו"]),
    term("prompt", &["פרומפט"]),
    term("token", &["טוקן"]),
    term("tokens", &["טוקנים"]),
    // Everyday development words. Hebrew letters cannot collide with an
    // English word, so these are safe to recover from speech even though they
    // must stay out of the phonetic Latin pass.
    hebrew_only("commit", &["קומיט"]),
    hebrew_only("rebase", &["ריבייס"]),
    hebrew_only("pull request", &["פול ריקווסט"]),
    hebrew_only("deploy", &["דיפלוי"]),
    hebrew_only("refactor", &["ריפקטור"]),
    hebrew_only("endpoint", &["אנדפוינט"]),
    hebrew_only("workflow", &["וורקפלו"]),
    hebrew_only("branch", &["ברנץ'"]),
    hebrew_only("repo", &["ריפו"]),
    hebrew_only("build", &["בילד"]),
    hebrew_only("lint", &["לינט"]),
    hebrew_only("plan", &["פלאן"]),
    // Languages and runtimes. Only the ones that get mangled. "Python" and
    // "JavaScript" are spelled correctly by the model and are left out.
    term("TypeScript", &["טייפסקריפט"]),
    latin_only("SwiftUI"),
    latin_only("Rust"),
    latin_only("Node.js"),
    latin_only("Deno"),
    latin_only("Bun"),
    latin_only("npm"),
    latin_only("npx"),
    latin_only("pnpm"),
    latin_only("Cargo"),
    // Frameworks and build tools.
    // Normalization drops the space, so the joined spelling is the same key.
    term("Next.js", &["נקסט ג'יאס"]),
    term("Tauri", &["טאורי", "טאוארי"]),
    latin_only("Vite"),
    latin_only("Turbopack"),
    term("Tailwind", &["טיילווינד"]),
    term("shadcn", &["שדסיאן"]),
    term("Expo", &["אקספו"]),
    latin_only("Playwright"),
    latin_only("Vitest"),
    latin_only("ESLint"),
    latin_only("Prettier"),
    term("Prisma", &["פריזמה"]),
    latin_only("Drizzle"),
    latin_only("Zod"),
    latin_only("tRPC"),
    // Services and platforms.
    term(
        "Supabase",
        &["סופרבייס", "סופהבייס", "סופאבייס", "סאפאבייס"],
    ),
    term("Vercel", &["ורסל", "ורצל"]),
    term("Netlify", &["נטליפיי"]),
    term("Cloudflare", &["קלאודפלייר", "קלאודפלר"]),
    term("GitHub", &["גיטאב", "גיטהאב", "גיטהב"]),
    term("GitLab", &["גיטלאב"]),
    term("Postgres", &["פוסטגרס"]),
    latin_only("PostgreSQL"),
    latin_only("pgvector"),
    latin_only("Neon"),
    term("Redis", &["רדיס"]),
    term("SQLite", &["אסקיולייט"]),
    term("Firebase", &["פיירבייס"]),
    term("Stripe", &["סטרייפ"]),
    latin_only("Polar"),
    term("RevenueCat", &["רבניוקט", "רוונקט"]),
    term("PostHog", &["פוסטהוג"]),
    term("Sentry", &["סנטרי"]),
    term("Resend", &["ריסנד"]),
    term("Twilio", &["טוויליו"]),
    term("Snyk", &["סניק"]),
    term("Docker", &["דוקר"]),
    term("Figma", &["פיגמה"]),
    latin_only("Linear"),
    latin_only("Notion"),
    term("Xcode", &["אקסקוד"]),
    term("TestFlight", &["טסטפלייט"]),
    latin_only("Homebrew"),
    latin_only("CardCom"),
    latin_only("Bunny Stream"),
    term("Whisper", &["ויספר"]),
    // Jargon that is not an ordinary English word. Anything that reads as
    // normal English (commit, deploy, merge, branch, build) is deliberately
    // absent, and so is anything that reads naturally in Hebrew.
    latin_only("monorepo"),
    latin_only("webhook"),
    latin_only("middleware"),
    latin_only("changelog"),
    latin_only("linter"),
    latin_only("boilerplate"),
    latin_only("pgAdmin"),
    latin_only("localhost"),
    latin_only("favicon"),
    latin_only("OAuth"),
    latin_only("JWT"),
    latin_only("CORS"),
    latin_only("GraphQL"),
    latin_only("WebSocket"),
    latin_only("JSON"),
    latin_only("YAML"),
    latin_only("SDK"),
    latin_only("CLI"),
    latin_only("API"),
    latin_only("UUID"),
    latin_only("CRUD"),
    latin_only("regex"),
    latin_only("async"),
    latin_only("enum"),
    latin_only("struct"),
];

/// Prefixes Hebrew glues onto the front of a noun, including a foreign one.
/// "on Vercel" is one word, `בוורסל`, so the term has to be found underneath
/// them and the prefix put back with a maqaf: `ב-Vercel`.
const PREFIX_LETTERS: &[char] = &['ו', 'ה', 'ב', 'ל', 'כ', 'ש', 'מ'];

/// Below this many normalized characters a Hebrew key is too easy to collide
/// with an ordinary word, so it is rejected outright.
const MIN_HEBREW_KEY_LEN: usize = 4;

/// At or above this length a key tolerates a single wrong letter. Shorter keys
/// must match exactly.
///
/// Seven rather than six because six-letter keys have real Hebrew words one
/// letter away: `סקילים` (skills) and `מקילים` (they ease) differ by their
/// first letter alone. At seven the neighbourhood is empty enough to be worth
/// the reach.
const FUZZY_HEBREW_KEY_LEN: usize = 7;

/// Hebrew keys allowed under [`MIN_HEBREW_KEY_LEN`], each because it is a
/// Claude Code term with no ordinary Hebrew word to collide with in the
/// context this app is used in. Kept explicit so admitting one stays a
/// deliberate act rather than a side effect of lowering the floor.
const SHORT_KEYS_ALLOWED: &[&str] = &[
    // Claude Code's `/goal`. Also the Hebrew for a football goal, which is not
    // a word that turns up while dictating to a coding agent.
    "גול", // Claude Code's hooks. Not a Hebrew word at all.
    "הוק",
];

/// The longest run of words that can be joined into one term. `סופר בייס` is
/// two, and nothing in the table is longer than three.
const MAX_HEBREW_NGRAM: usize = 3;

/// Reduces a Hebrew string to the form used for lookups.
///
/// Speech transcription varies in ways that carry no meaning here: final letter
/// forms depend on position, vav and yod get doubled or not, niqqud and
/// gershayim come and go, and a two-word name may arrive joined. Folding all of
/// that away lets one table entry cover every spelling of it.
pub fn normalize_hebrew(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        let folded = match ch {
            'ך' => 'כ',
            'ם' => 'מ',
            'ן' => 'נ',
            'ף' => 'פ',
            'ץ' => 'צ',
            // Niqqud, cantillation and the Hebrew punctuation marks, all of
            // which the model emits inconsistently.
            '\u{0591}'..='\u{05C7}' | '\u{05F3}' | '\u{05F4}' | '\'' | '"' | '`' => continue,
            other if other.is_alphabetic() => other,
            _ => continue,
        };
        // Doubled vav and yod are a spelling convention, not a sound.
        if out.ends_with(folded) && (folded == 'ו' || folded == 'י') {
            continue;
        }
        out.push(folded);
    }
    out
}

fn is_indexable_hebrew_key(key: &str) -> bool {
    key.chars().count() >= MIN_HEBREW_KEY_LEN || SHORT_KEYS_ALLOWED.contains(&key)
}

/// Hebrew keys mapped to the canonical spelling that replaces them.
fn hebrew_index(terms: &[Term]) -> HashMap<String, &'static str> {
    let mut index = HashMap::new();
    for term in terms {
        for spelling in term.hebrew {
            let key = normalize_hebrew(spelling);
            if is_indexable_hebrew_key(&key) {
                index.insert(key, term.canonical);
            }
        }
    }
    index
}

/// The one wrong letter allowed on longer keys, as a plain edit distance rather
/// than the ratio the Latin pass uses. A ratio would let a nine-letter key
/// absorb two errors, which is enough to reach a different word.
fn within_one_edit(a: &str, b: &str) -> bool {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len().abs_diff(b.len()) > 1 {
        return false;
    }
    let (short, long) = if a.len() <= b.len() {
        (&a, &b)
    } else {
        (&b, &a)
    };
    let mut i = 0;
    let mut j = 0;
    let mut edited = false;
    while i < short.len() && j < long.len() {
        if short[i] == long[j] {
            i += 1;
            j += 1;
            continue;
        }
        if edited {
            return false;
        }
        edited = true;
        if short.len() == long.len() {
            i += 1;
        }
        j += 1;
    }
    true
}

/// Looks up a candidate built from `word_count` words.
///
/// The single wrong letter is only forgiven for a single word. Across a span,
/// the joined candidate is long enough that one edit reaches the *next* word:
/// `סאב אייג'נט עם` differs from the key for `subagents` by exactly the one
/// letter that turns `עם` into the plural suffix, so a three-word span would
/// eat the preposition after it. A multi-word term has to match exactly.
fn lookup<'a>(key: &str, word_count: usize, index: &HashMap<String, &'a str>) -> Option<&'a str> {
    if let Some(hit) = index.get(key) {
        return Some(hit);
    }
    if word_count > 1 || key.chars().count() < FUZZY_HEBREW_KEY_LEN {
        return None;
    }
    index
        .iter()
        .find(|(candidate, _)| {
            candidate.chars().count() >= FUZZY_HEBREW_KEY_LEN && within_one_edit(key, candidate)
        })
        .map(|(_, canonical)| *canonical)
}

/// Splits a token into any leading Hebrew prefix letters and the rest.
///
/// Only one prefix is peeled. Two-letter stacks like `ולב` exist but are rare
/// in speech, and every extra letter peeled is another chance to turn an
/// ordinary word into a false match.
fn strip_prefix(key: &str) -> Option<(char, String)> {
    let mut chars = key.chars();
    let first = chars.next()?;
    if !PREFIX_LETTERS.contains(&first) {
        return None;
    }
    let rest: String = chars.collect();
    is_indexable_hebrew_key(&rest).then_some((first, rest))
}

/// Rewrites Hebrew renderings of product names back into their real spelling.
///
/// Runs before the Latin pass, so a name recovered here arrives there as an
/// exact match and is left alone.
pub fn apply_hebrew_terms(text: &str, terms: &[Term]) -> String {
    let index = hebrew_index(terms);
    if index.is_empty() {
        return text.to_string();
    }

    let words: Vec<&str> = text.split_whitespace().collect();
    let mut result: Vec<String> = Vec::with_capacity(words.len());
    let mut i = 0;

    while i < words.len() {
        let mut matched = None;

        // Longest first: `קלוד קוד` is Claude Code, not Claude followed by a
        // stray word.
        for n in (1..=MAX_HEBREW_NGRAM.min(words.len() - i)).rev() {
            let span = &words[i..i + n];
            // Punctuation closes a term, so `קלוד, קוד` is two things.
            if span[..n - 1]
                .iter()
                .any(|word| word.chars().last().is_some_and(|c| !c.is_alphabetic()))
            {
                continue;
            }

            let key = span
                .iter()
                .map(|word| normalize_hebrew(word))
                .collect::<String>();
            if !is_indexable_hebrew_key(&key) {
                continue;
            }

            if let Some(canonical) = lookup(&key, n, &index) {
                matched = Some((n, canonical.to_string()));
                break;
            }
            // A glued prefix is only meaningful on the first word of the span.
            if let Some((prefix, rest)) = strip_prefix(&key) {
                if let Some(canonical) = lookup(&rest, n, &index) {
                    matched = Some((n, format!("{prefix}-{canonical}")));
                    break;
                }
            }
        }

        match matched {
            Some((n, replacement)) => {
                // Trailing punctuation belongs to the sentence, not the name.
                let last = words[i + n - 1];
                let suffix: String = last
                    .chars()
                    .rev()
                    .take_while(|c| !c.is_alphabetic())
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect();
                result.push(format!("{replacement}{suffix}"));
                i += n;
            }
            None => {
                result.push(words[i].to_string());
                i += 1;
            }
        }
    }

    result.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_toolkit::text::apply_custom_words;

    /// Mirrors `default_word_correction_threshold` in `settings.rs`.
    const THRESHOLD: f64 = 0.18;

    /// Mirrors `effective_custom_words` in the transcription manager.
    fn latin_words() -> Vec<String> {
        DEV_VOCABULARY
            .iter()
            .filter(|term| term.latin_pass)
            .map(|term| term.canonical.to_string())
            .collect()
    }

    fn correct(text: &str) -> String {
        apply_custom_words(
            &apply_hebrew_terms(text, DEV_VOCABULARY),
            &latin_words(),
            THRESHOLD,
        )
    }

    /// Daniel's own recording, the one that reported this gap.
    #[test]
    fn recovers_names_spoken_in_hebrew() {
        assert_eq!(
            correct("זה נגיד אני אומר קלוד קוד"),
            "זה נגיד אני אומר Claude Code"
        );
        assert_eq!(
            correct("ואם אני אומר נגיד ורסל, גיטאב, סופר בייס"),
            "ואם אני אומר נגיד Vercel, GitHub, Supabase"
        );
    }

    /// The other half of the same problem, from a recording where the model
    /// wrote Latin script and misspelled it.
    #[test]
    fn still_fixes_the_latin_misspelling() {
        assert_eq!(
            correct("הפוקוס העיקרי שלי זה העבודה עם Cloud Code."),
            "הפוקוס העיקרי שלי זה העבודה עם Claude Code."
        );
    }

    /// A glued Hebrew prefix is idiomatic and has to survive as one.
    #[test]
    fn keeps_a_glued_hebrew_prefix() {
        assert_eq!(correct("הכל רץ בוורסל"), "הכל רץ ב-Vercel");
        assert_eq!(correct("תעלה את זה לגיטאב"), "תעלה את זה ל-GitHub");
    }

    /// `קוד` and `סופר` are ordinary Hebrew. Only the complete term may match,
    /// which is why no key in the table is a standalone Hebrew word.
    #[test]
    fn does_not_touch_ordinary_hebrew_words() {
        for sentence in [
            "תכתוב לי את הקוד הזה",
            "זה סופר חשוב בשבילי",
            "הענן שלנו עובד טוב",
            "תעשה סדר ביומן ותבדוק שהשעות נכונות",
        ] {
            assert_eq!(correct(sentence), sentence, "corrupted: {sentence}");
        }
    }

    /// Punctuation between two words closes the term, so this is Claude
    /// followed by the Hebrew word for code, not Claude Code.
    #[test]
    fn punctuation_closes_a_multi_word_term() {
        assert_eq!(correct("קלוד, קוד"), "Claude, קוד");
    }

    #[test]
    fn normalization_folds_the_spellings_that_carry_no_meaning() {
        assert_eq!(normalize_hebrew("וורסל"), normalize_hebrew("ורסל"));
        assert_eq!(normalize_hebrew("גיטהאב!"), "גיטהאב");
        // Final letter forms.
        assert_eq!(normalize_hebrew("םןףץך"), "מנפצכ");
    }

    #[test]
    fn one_edit_is_tolerated_only_on_longer_keys() {
        // 8 letters, one wrong: still Supabase.
        assert_eq!(correct("תעלה את זה לסופרבייז"), "תעלה את זה ל-Supabase");
        // Distance 2 is a different word, and must not match.
        assert_eq!(correct("תעלה את זה לסופרבזז"), "תעלה את זה לסופרבזז");
    }

    /// The fuzzy Latin matcher discards any candidate that is not ASCII
    /// alphanumeric, so a non-ASCII canonical name would be silently inert.
    #[test]
    fn every_canonical_name_survives_the_latin_ascii_gate() {
        for term in DEV_VOCABULARY {
            let key: String = term
                .canonical
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase();
            assert!(
                !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric()),
                "{:?} reduces to {key:?}, which the matcher will never consider",
                term.canonical
            );
        }
    }

    /// Both passes join at most three words.
    #[test]
    fn no_term_exceeds_the_three_word_window() {
        for term in DEV_VOCABULARY {
            assert!(term.canonical.split_whitespace().count() <= MAX_HEBREW_NGRAM);
            for spelling in term.hebrew {
                assert!(
                    spelling.split_whitespace().count() <= MAX_HEBREW_NGRAM,
                    "{spelling:?} is longer than the {MAX_HEBREW_NGRAM}-word window"
                );
            }
        }
    }

    /// Claude Code's own words, in the form the official glossary uses. A
    /// request for a `סקיל` is a guess; a request for a `skill` is a skill.
    #[test]
    fn writes_claude_code_terms_the_way_claude_code_spells_them() {
        assert_eq!(
            correct("תכין לי סקיל או וורקפלו לדבר הזה"),
            "תכין לי skill או workflow לדבר הזה"
        );
        assert_eq!(
            correct("כל מיני סקילים שאפשר לתת להם"),
            "כל מיני skills שאפשר לתת להם"
        );
        assert_eq!(correct("תפתח גול על זה"), "תפתח goal על זה");
        assert_eq!(
            correct("תריץ סאב אייג'נט עם הוקים"),
            "תריץ subagent עם hooks"
        );
        assert_eq!(
            correct("תעשה קומיט ופול ריקווסט"),
            "תעשה commit ו-pull request"
        );
    }

    /// Everyday development words are safe to recover from Hebrew letters and
    /// unsafe to hand the phonetic Latin matcher, so they are Hebrew-only and
    /// must not appear in the list the Latin pass sees.
    #[test]
    fn hebrew_only_words_stay_out_of_the_latin_pass() {
        let latin = latin_words();
        for word in ["commit", "deploy", "refactor", "branch", "build", "plan"] {
            assert!(
                !latin.iter().any(|listed| listed == word),
                "{word:?} would give the phonetic matcher an ordinary English word"
            );
        }
        // ...while still being recovered from Hebrew.
        assert_eq!(correct("תעשה דיפלוי"), "תעשה deploy");
    }

    /// `סקילים` and `מקילים` differ by one letter, which is why the fuzzy
    /// reach starts above six characters rather than at it.
    #[test]
    fn a_real_hebrew_word_one_letter_away_is_left_alone() {
        assert_eq!(correct("הם מקילים על התנאים"), "הם מקילים על התנאים");
    }

    /// Joining words makes a long candidate, and on a long candidate one
    /// forgiven letter is enough to reach into the next word. Caught by
    /// `subagents` eating the `עם` that followed `סאב אייג'נט`.
    #[test]
    fn a_multi_word_span_never_spends_its_edit_on_the_following_word() {
        assert_eq!(
            correct("תריץ סאב אייג'נט עם הוקים"),
            "תריץ subagent עם hooks"
        );
    }

    /// A Hebrew spelling short enough to collide with an ordinary word is
    /// dropped by `hebrew_index`, which makes it dead weight in the table.
    #[test]
    fn every_hebrew_spelling_is_long_enough_to_be_indexed() {
        let index = hebrew_index(DEV_VOCABULARY);
        for term in DEV_VOCABULARY {
            for spelling in term.hebrew {
                let key = normalize_hebrew(spelling);
                assert!(
                    index.contains_key(&key),
                    "{spelling:?} normalizes to {key:?}, under the {MIN_HEBREW_KEY_LEN} character floor"
                );
            }
        }
    }

    #[test]
    fn there_are_no_duplicates() {
        let mut seen = std::collections::HashSet::new();
        for term in DEV_VOCABULARY {
            assert!(
                seen.insert(term.canonical.to_lowercase()),
                "{:?} appears twice in the vocabulary",
                term.canonical
            );
        }
        let mut spellings = std::collections::HashSet::new();
        for term in DEV_VOCABULARY {
            for spelling in term.hebrew {
                let key = normalize_hebrew(spelling);
                assert!(
                    spellings.insert(key.clone()),
                    "{spelling:?} normalizes to {key:?}, which another term already claims"
                );
            }
        }
    }
}
