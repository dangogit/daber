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
//! # What belongs in the table
//!
//! **Product names and brand names.** Not ordinary English, and not jargon
//! that reads naturally in a Hebrew sentence.
//!
//! The Latin pass gives every entry a phonetic net that also catches whatever
//! merely sounds like it. That is exactly what turns `cloud` into `Claude`, and
//! exactly what would turn `march` into `merge` if `merge` were listed. Whisper
//! already spells `commit`, `deploy` and `refactor` correctly, so listing them
//! is all risk and no gain.
//!
//! The Hebrew pass has the opposite failure mode. `קוד` and `סופר` are real
//! Hebrew words, so a Hebrew spelling must always be the whole term: `קלודקוד`
//! and `סופרבייס` are safe keys, `קוד` and `סופר` never are. Words a Hebrew
//! speaker would keep in Hebrew anyway, like `סקיל` or `וורקפלו`, are
//! deliberately absent.

use std::collections::HashMap;

/// One product name, with the Hebrew spellings the model produces for it.
pub struct Term {
    /// How the name should be written once corrected.
    pub canonical: &'static str,
    /// Hebrew renderings to replace with `canonical`. Written here as they are
    /// spoken; [`normalize_hebrew`] handles final letter forms, doubled vav and
    /// yod, niqqud and word joining, so only genuinely different spellings need
    /// listing. Empty when the name is never said in Hebrew letters.
    pub hebrew: &'static [&'static str],
}

/// Shorthand for a term that only ever needs the Latin pass.
const fn latin_only(canonical: &'static str) -> Term {
    Term {
        canonical,
        hebrew: &[],
    }
}

const fn term(canonical: &'static str, hebrew: &'static [&'static str]) -> Term {
    Term { canonical, hebrew }
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
    latin_only("MCP"),
    latin_only("subagent"),
    term("Codex", &["קודקס"]),
    term("Cursor", &["קרסר", "קורסר"]),
    term("Copilot", &["קופיילוט"]),
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
const FUZZY_HEBREW_KEY_LEN: usize = 6;

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

/// Hebrew keys mapped to the canonical spelling that replaces them.
fn hebrew_index(terms: &[Term]) -> HashMap<String, &'static str> {
    let mut index = HashMap::new();
    for term in terms {
        for spelling in term.hebrew {
            let key = normalize_hebrew(spelling);
            if key.chars().count() >= MIN_HEBREW_KEY_LEN {
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

fn lookup<'a>(key: &str, index: &HashMap<String, &'a str>) -> Option<&'a str> {
    if let Some(hit) = index.get(key) {
        return Some(hit);
    }
    if key.chars().count() < FUZZY_HEBREW_KEY_LEN {
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
    (rest.chars().count() >= MIN_HEBREW_KEY_LEN).then_some((first, rest))
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
            if key.chars().count() < MIN_HEBREW_KEY_LEN {
                continue;
            }

            if let Some(canonical) = lookup(&key, &index) {
                matched = Some((n, canonical.to_string()));
                break;
            }
            // A glued prefix is only meaningful on the first word of the span.
            if let Some((prefix, rest)) = strip_prefix(&key) {
                if let Some(canonical) = lookup(&rest, &index) {
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

    fn latin_words() -> Vec<String> {
        DEV_VOCABULARY
            .iter()
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
