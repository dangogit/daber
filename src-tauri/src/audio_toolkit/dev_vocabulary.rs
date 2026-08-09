//! The words a developer says that Whisper has never heard enough of.
//!
//! Dibur transcribes Hebrew, but a Hebrew speaker dictating to a coding agent
//! switches to English for every proper noun in their stack. The model handles
//! that switch well, emitting Latin script without being asked, and then
//! spells the name wrong, because "Supabase" and "pgvector" are not words it
//! saw often during training. The most visible case is `Claude`, which comes
//! out as `cloud` essentially every time.
//!
//! These terms are merged into the user's own custom words and go through the
//! same fuzzy correction, so no new matching logic exists here. This module is
//! only the list, and the rule for what belongs on it.
//!
//! # What belongs here
//!
//! **Product names, brand names, and jargon that is not an ordinary English
//! word.** Nothing else.
//!
//! The matcher scores on Soundex plus edit distance, which means every entry
//! also captures the words that merely sound like it. That is precisely what
//! fixes `cloud` -> `Claude`, and precisely what would break `march` ->
//! `merge` if `merge` were listed. Ordinary English verbs are already spelled
//! correctly by the model, so listing them carries the risk without the
//! benefit. If you are about to add a word that appears in a normal
//! dictionary, it almost certainly does not belong.

/// Terms merged into custom-word correction when `dev_vocabulary` is enabled.
///
/// Ordering is by theme for the benefit of whoever edits this next; the matcher
/// itself is order-independent and picks the lowest score.
pub const DEV_VOCABULARY: &[&str] = &[
    // Claude and agent tooling. `Claude` earns its place despite colliding with
    // `cloud`: a Hebrew speaker says `ענן` for the sky, so a Latin-script
    // `cloud` in the output means they almost certainly said "Claude".
    "Claude",
    "Claude Code",
    "Anthropic",
    "Opus",
    "Sonnet",
    "Haiku",
    "MCP",
    "subagent",
    "Codex",
    "Cursor",
    "Copilot",
    // Languages and runtimes. Only the ones that get mangled. "Python" and
    // "JavaScript" are spelled correctly by the model and are left out.
    "TypeScript",
    "SwiftUI",
    "Rust",
    "Node.js",
    "Deno",
    "Bun",
    "npm",
    "npx",
    "pnpm",
    "Cargo",
    // Frameworks and build tools.
    "Next.js",
    "Tauri",
    "Vite",
    "Turbopack",
    "Tailwind",
    "shadcn",
    "Expo",
    "Playwright",
    "Vitest",
    "ESLint",
    "Prettier",
    "Prisma",
    "Drizzle",
    "Zod",
    "tRPC",
    // Services and platforms.
    "Supabase",
    "Vercel",
    "Netlify",
    "Cloudflare",
    "GitHub",
    "GitLab",
    "Postgres",
    "PostgreSQL",
    "pgvector",
    "Neon",
    "Redis",
    "SQLite",
    "Firebase",
    "Stripe",
    "Polar",
    "RevenueCat",
    "PostHog",
    "Sentry",
    "Resend",
    "Twilio",
    "Snyk",
    "Docker",
    "Figma",
    "Linear",
    "Notion",
    "Xcode",
    "TestFlight",
    "Homebrew",
    "CardCom",
    "Bunny Stream",
    // Jargon that is not an ordinary English word. Anything that reads as
    // normal English (commit, deploy, merge, branch, build) is deliberately
    // absent; see the module docs.
    "monorepo",
    "webhook",
    "middleware",
    "changelog",
    "linter",
    "boilerplate",
    "pgAdmin",
    "localhost",
    "favicon",
    "OAuth",
    "JWT",
    "CORS",
    "GraphQL",
    "WebSocket",
    "JSON",
    "YAML",
    "SDK",
    "CLI",
    "API",
    "UUID",
    "CRUD",
    "regex",
    "async",
    "enum",
    "struct",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_toolkit::text::apply_custom_words;

    /// Mirrors `default_word_correction_threshold` in `settings.rs`.
    const THRESHOLD: f64 = 0.18;

    /// The sentence that motivated this module, transcribed from Daniel's own
    /// microphone by the shipped model. `Cloud Code` twice, `built-in` and
    /// `value` already correct.
    #[test]
    fn fixes_the_claude_code_misspelling_from_real_audio() {
        let transcribed = "כשאני נגיד עובד איתך עם Cloud Code, הייתי רוצה כל מיני \
                           מילים של תכנות ודברים כאלה שיהיו built-in. הפוקוס \
                           העיקרי שלי זה באמת העבודה עם Cloud Code.";

        let corrected = apply_custom_words(transcribed, &owned(DEV_VOCABULARY), THRESHOLD);

        assert_eq!(corrected.matches("Claude Code").count(), 2);
        assert!(!corrected.contains("Cloud Code"));
        // The words the model already got right must survive untouched.
        assert!(corrected.contains("built-in"));
    }

    /// The matcher discards non-ASCII candidates, so a sentence with no Latin
    /// script in it cannot be altered no matter what the vocabulary contains.
    #[test]
    fn leaves_pure_hebrew_untouched() {
        let hebrew = "תעשה סדר ביומן ותבדוק שהשעות של הסטודנטים נכונות.";

        assert_eq!(
            apply_custom_words(hebrew, &owned(DEV_VOCABULARY), THRESHOLD),
            hebrew
        );
    }

    /// Every entry is a magnet for anything that sounds like it, which is the
    /// point and also the danger. Ordinary English that has nothing to do with
    /// the stack has to come through unchanged.
    #[test]
    fn does_not_pull_in_unrelated_english() {
        let sentence = "I told them the price was fair and the meeting was short.";

        assert_eq!(
            apply_custom_words(sentence, &owned(DEV_VOCABULARY), THRESHOLD),
            sentence
        );
    }

    fn owned(terms: &[&str]) -> Vec<String> {
        terms.iter().map(|term| term.to_string()).collect()
    }

    /// The fuzzy matcher discards any candidate that is not ASCII
    /// alphanumeric, so a non-ASCII entry here would be silently inert.
    #[test]
    fn every_term_survives_the_matchers_ascii_gate() {
        for term in DEV_VOCABULARY {
            let key: String = term
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase();
            assert!(
                !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric()),
                "{term:?} reduces to {key:?}, which the matcher will never consider"
            );
        }
    }

    /// An n-gram is built from at most three words, so a longer entry can never
    /// be matched against spoken input.
    #[test]
    fn no_term_exceeds_the_three_word_ngram_window() {
        for term in DEV_VOCABULARY {
            let words = term.split_whitespace().count();
            assert!(
                words <= 3,
                "{term:?} is {words} words; the matcher only builds 3-grams"
            );
        }
    }

    #[test]
    fn there_are_no_duplicates() {
        let mut seen = std::collections::HashSet::new();
        for term in DEV_VOCABULARY {
            assert!(
                seen.insert(term.to_lowercase()),
                "{term:?} appears twice in the vocabulary"
            );
        }
    }
}
