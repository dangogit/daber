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
//! There is a second, sharper rule for the Latin list that the first one does
//! not cover: **no short brand name that sounds like an ordinary English
//! word.** `Vaul`, `Deno`, `CLI` and `OAuth` are not ordinary English, and all
//! four still had to be removed, because they quietly rewrote `value`, `done`,
//! `call` and `out`. Nothing catches this by reading; the table is checked
//! against a list of common English words by
//! `common_english_survives_the_latin_pass`, and anything that fails belongs
//! in `hebrew_only` or nowhere.
//!
//! The Israeli entries deserve their own note. CardCom, Rav Messer, Green
//! Invoice, Isracard, Tranzila and Priority are the payment and invoicing
//! services these projects actually integrate with (CardCom alone appears
//! thousands of times across this workspace), and no general-purpose
//! vocabulary anywhere will ever contain them. That is the whole argument for
//! a Hebrew-first dictation app shipping its own dictionary.
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
    // `Fable` is an ordinary English word, so it stays out of the phonetic
    // pass and is recovered only from its Hebrew spelling.
    hebrew_only("Fable", &["פייבל", "פאבל"]),
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
    // The files a coding agent is told to read. `CLAUDE.md` is the single most
    // costly miss in this table: the model writes it `CloudMD`, and a request
    // to update `CloudMD` is a request to update nothing.
    term("CLAUDE.md", &["קלוד אמדי", "קלאוד אמדי", "קלוד דוט אמדי"]),
    term("AGENTS.md", &["אג'נטס אמדי", "אג'נט אמדי"]),
    term("statusline", &["סטטוסליין"]),
    term("ultrathink", &["אולטרהthink", "אולטרה ת'ינק"]),
    hebrew_only("slash command", &["סלאש קומנד"]),
    hebrew_only("output style", &["אאוטפוט סטייל"]),
    hebrew_only("session", &["סשיין"]),
    hebrew_only("transcript", &["טרנסקריפט"]),
    hebrew_only("worktrees", &["וורקטריז"]),
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
    hebrew_only("staging", &["סטייג'ינג"]),
    hebrew_only("production", &["פרודקשן"]),
    hebrew_only("database", &["דאטהבייס"]),
    hebrew_only("schema", &["סכמה"]),
    hebrew_only("query", &["קווארי"]),
    hebrew_only("component", &["קומפוננטה"]),
    hebrew_only("state", &["סטייט"]),
    hebrew_only("props", &["פרופס"]),
    hebrew_only("server", &["סרבר"]),
    hebrew_only("client", &["קלייאנט"]),
    hebrew_only("backend", &["בקאנד"]),
    hebrew_only("frontend", &["פרונטאנד"]),
    hebrew_only("rollback", &["רולבק"]),
    hebrew_only("tests", &["טסטים"]),
    hebrew_only("issue", &["אישיו"]),
    hebrew_only("code review", &["קוד רוויו"]),
    hebrew_only("landing page", &["לנדינג פייג'"]),
    hebrew_only("README", &["רידמי", "ריתמי", "רדמי"]),
    hebrew_only(".gitignore", &["גיטיגנור", "גיט איגנור"]),
    hebrew_only("fork", &["פורק"]),
    hebrew_only("template", &["טמפלייט", "טמפלייד"]),
    hebrew_only("docs", &["דוקס"]),
    hebrew_only("audit", &["אודיט", "אודית"]),
    hebrew_only("remote", &["רימוט"]),
    hebrew_only("merge", &["מרג'ים", "מירג'"]),
    hebrew_only("stash", &["סטאש"]),
    hebrew_only("squash", &["סקווש"]),
    hebrew_only("cherry-pick", &["צ'רי פיק"]),
    hebrew_only("pipeline", &["פייפליין"]),
    hebrew_only("release", &["ריליס"]),
    hebrew_only("onboarding", &["אונבורדינג"]),
    hebrew_only("overlay", &["אוברליי"]),
    hebrew_only("dashboard", &["דאשבורד"]),
    hebrew_only("Markdown", &["מרקדאון", "מארקדאון"]),
    hebrew_only("pricing", &["פרייסינג"]),
    hebrew_only("SSH", &["אס אס איץ'"]),
    hebrew_only("CLI", &["סי אל איי"]),
    hebrew_only("CI", &["סי איי"]),
    hebrew_only("QA", &["קיו איי"]),
    hebrew_only("TDD", &["טי די די"]),
    hebrew_only("DB", &["די בי"]),
    // Shipping a signed Mac app. These arrived the week Dibur got its Developer
    // ID, and the model had never heard any of them: `Developer ID` came out
    // `דיבלופר ID`, `Liquid Glass` came out `Lick with Glass`.
    term("Developer ID", &["דיבלופר איידי", "דבלופר איידי"]),
    term("Liquid Glass", &["ליקוויד גלאס"]),
    term("Rosetta", &["רוזטה"]),
    term("DMG", &["די אם ג'י"]),
    term("App Store Connect", &["אפ סטור קונקט"]),
    hebrew_only("notarization", &["נוטריזציה"]),
    hebrew_only("Keychain", &["קיצ'יין"]),
    hebrew_only("Gatekeeper", &["גייטקיפר"]),
    hebrew_only("Simulator", &["סימולטור"]),
    hebrew_only("Desktop", &["דסקטופ", "דסקטוב"]),
    // Daniel's own tooling. These are skill and repo names he says out loud to
    // Claude Code, so a wrong spelling does not just read badly, it fails to
    // invoke the thing he asked for.
    hebrew_only("grill-me", &["גרילמי"]),
    term("gstack", &["ג'יסטאק"]),
    term("hyperframes", &["הייפרפריימס"]),
    hebrew_only("humanizer", &["יומניזר", "היומניזר"]),
    hebrew_only("handoff", &["הנדאוף", "האנדאוף"]),
    hebrew_only("retro", &["רטרו"]),
    hebrew_only("triage", &["טריאז'"]),
    term("Hermes", &["הרמס", "הרמץ"]),
    hebrew_only("Excel", &["אקסל"]),
    // AI tools. Daniel names these to Claude Code constantly, and every one of
    // them is a brand the model has no reason to spell right.
    term("ChatGPT", &["צ'אט ג'יפיטי"]),
    term("OpenAI", &["אופן איי איי"]),
    term("Gemini", &["ג'מיני"]),
    term("Midjourney", &["מידג'רני"]),
    term("ElevenLabs", &["אילבן לאבס"]),
    term("Perplexity", &["פרפלקסיטי"]),
    term("Ollama", &["אולמה"]),
    term("LangChain", &["לאנגצ'יין"]),
    term("Zapier", &["זאפייר"]),
    latin_only("Runway"),
    // The stack these projects are actually built on, taken from the
    // dependencies in this workspace rather than from a list of popular
    // libraries.
    term("Radix UI", &["רדיקס"]),
    term("TipTap", &["טיפטאפ"]),
    term("TanStack", &["טאנסטאק"]),
    hebrew_only("React", &["ריאקט"]),
    term("Angular", &["אנגולר"]),
    term("Svelte", &["סבלט"]),
    term("Zustand", &["זוסטנד"]),
    term("Framer Motion", &["פריימר מושן"]),
    term("Remotion", &["רמושן"]),
    term("Recharts", &["ריצ'ארטס"]),
    term("Capacitor", &["קפסיטור"]),
    term("Storybook", &["סטוריבוק"]),
    term("Upstash", &["אפסטאש"]),
    term("Shiki", &["שיקי"]),
    term("Sonner", &["סונר"]),
    term("Lucide", &["לוסייד"]),
    latin_only("Radix"),
    latin_only("Embla"),
    latin_only("FullCalendar"),
    latin_only("Streamdown"),
    latin_only("Solana"),
    // Israeli services. These are the ones that turn up across this workspace,
    // by a wide margin: CardCom alone appears thousands of times. No general
    // vocabulary would ever contain them, which is exactly why they belong in
    // a Hebrew-first dictation app.
    term("CardCom", &["קארדקום", "קרדקום"]),
    term("Green Invoice", &["גרין אינבויס"]),
    hebrew_only("Morning", &["מורנינג"]),
    term("Rav Messer", &["רב מסר"]),
    term("Priority", &["פריוריטי"]),
    term("Isracard", &["ישראכרט"]),
    term("Tranzila", &["טרנזילה"]),
    term("PayPlus", &["פייפלוס"]),
    term("PayBox", &["פייבוקס"]),
    term("Wix", &["ויקס"]),
    hebrew_only("Monday", &["מאנדיי"]),
    // Platforms and services people name out loud.
    term("WordPress", &["וורדפרס"]),
    term("WooCommerce", &["ווקומרס"]),
    term("Elementor", &["אלמנטור"]),
    term("Shopify", &["שופיפיי"]),
    term("HubSpot", &["האבספוט"]),
    term("Mailchimp", &["מיילצ'ימפ"]),
    term("Airtable", &["אירטייבל"]),
    term("Notion", &["נושן"]),
    hebrew_only("Slack", &["סלאק"]),
    term("Discord", &["דיסקורד"]),
    term("Telegram", &["טלגרם"]),
    term("WhatsApp", &["וואטסאפ"]),
    term("Instagram", &["אינסטגרם"]),
    term("TikTok", &["טיקטוק"]),
    term("YouTube", &["יוטיוב"]),
    term("LinkedIn", &["לינקדאין"]),
    term("Facebook", &["פייסבוק"]),
    term("Google", &["גוגל"]),
    term("Railway", &["רילוויי"]),
    term("Heroku", &["הרוקו"]),
    term("DigitalOcean", &["דיגיטל אושן"]),
    latin_only("LemonSqueezy"),
    // Languages and runtimes. Only the ones that get mangled. "Python" and
    // "JavaScript" are spelled correctly by the model and are left out.
    term("TypeScript", &["טייפסקריפט"]),
    latin_only("SwiftUI"),
    latin_only("Node.js"),
    latin_only("npm"),
    latin_only("npx"),
    latin_only("pnpm"),
    latin_only("Cargo"),
    // Frameworks and build tools.
    // Normalization drops the space, so the joined spelling is the same key.
    term("Next.js", &["נקסט ג'יאס"]),
    term("Tauri", &["טאורי", "טאוארי"]),
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
    term("Redis", &["רדיס"]),
    term("SQLite", &["אסקיולייט"]),
    term("Firebase", &["פיירבייס"]),
    term("Stripe", &["סטרייפ"]),
    term("RevenueCat", &["רבניוקט", "רוונקט"]),
    term("PostHog", &["פוסטהוג"]),
    term("Sentry", &["סנטרי"]),
    term("Resend", &["ריסנד"]),
    term("Twilio", &["טוויליו"]),
    term("Snyk", &["סניק"]),
    term("Docker", &["דוקר"]),
    term("Figma", &["פיגמה"]),
    latin_only("Linear"),
    term("Xcode", &["אקסקוד"]),
    term("TestFlight", &["טסטפלייט"]),
    latin_only("Homebrew"),
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
    latin_only("JWT"),
    latin_only("GraphQL"),
    latin_only("WebSocket"),
    latin_only("JSON"),
    latin_only("YAML"),
    latin_only("SDK"),
    latin_only("API"),
    latin_only("UUID"),
    latin_only("CRUD"),
    latin_only("regex"),
    latin_only("async"),
    latin_only("enum"),
    // ---- Round two. Everything below was added after reading Daniel's own
    // transcription log: the words that actually came out wrong, plus the rest
    // of the stack they belong to. Entries that the table's own invariants
    // rejected were dropped rather than forced.

    // Models and AI products named in conversation.
    term("Claude Desktop", &["קלוד דסקטופ", "קלאוד דסקטופ"]),
    term("DeepSeek", &["דיפסיק"]),
    term("Grok", &["גרוק"]),
    term("Llama", &["לאמה"]),
    term("Mistral", &["מיסטרל"]),
    term("Qwen", &[]),
    term("Veo", &["ויאו"]),
    term("Kling", &["קלינג"]),
    term("Flux", &["פלאקס"]),
    term("Stable Diffusion", &["סטייבל דיפיוז'ן"]),
    term("Replicate", &["רפליקייט"]),
    term("Hugging Face", &["האגינג פייס"]),
    term("LangGraph", &["לנגגרף"]),
    term("NotebookLM", &["נוטבוק אל אם"]),
    term("Firecrawl", &["פיירקרול"]),
    term("Tavily", &["טאבילי"]),
    hebrew_only("LLM", &["אל אל אם"]),
    hebrew_only("RAG", &[]),
    hebrew_only("embedding", &["אמבדינג"]),
    hebrew_only("embeddings", &["אמבדינגים"]),
    hebrew_only("fine-tune", &["פיין טיון"]),
    hebrew_only("inference", &["אינפרנס"]),
    hebrew_only("tool use", &["טול יוז"]),
    hebrew_only("streaming", &["סטרימינג"]),
    hebrew_only("multimodal", &["מולטימודאלי"]),
    // Languages and runtimes.
    hebrew_only("Python", &["פייתון"]),
    hebrew_only("Rust", &["ראסט"]),
    hebrew_only("Swift", &["סוויפט"]),
    hebrew_only("Kotlin", &["קוטלין"]),
    hebrew_only("PHP", &["פי אייץ' פי"]),
    hebrew_only("Ruby", &["רובי"]),
    hebrew_only("SQL", &["אס קיו אל"]),
    // Frontend.
    term("Webpack", &["ווב פאק"]),
    term("Turborepo", &["טורבו ריפו"]),
    term("GSAP", &["ג'יסאפ"]),
    term("Three.js", &["ת'רי ג'יאס"]),
    term("Redux", &["רידאקס"]),
    term("Jotai", &["ג'וטאי"]),
    term("React Query", &["ריאקט קווארי"]),
    term("Lighthouse", &["לייטהאוס"]),
    hebrew_only("SSR", &["אס אס אר"]),
    hebrew_only("hydration", &["הידרציה"]),
    hebrew_only("responsive", &["רספונסיבי"]),
    hebrew_only("viewport", &["ויופורט"]),
    hebrew_only("breakpoint", &["ברייקפוינט"]),
    hebrew_only("accessibility", &["אקססביליטי"]),
    // Backend, data and infrastructure.
    term("PlanetScale", &["פלאנטסקייל"]),
    term("Turso", &["טורסו"]),
    term("Kubernetes", &["קוברנטיס"]),
    term("Terraform", &["טרהפורם"]),
    term("nginx", &["אנג'ינקס"]),
    term("Kafka", &["קפקא"]),
    hebrew_only("migration", &["מייגרישן", "מיגרציה"]),
    hebrew_only("migrations", &["מיגרציות"]),
    hebrew_only("index", &["אינדקס"]),
    hebrew_only("transaction", &["טרנזקציה"]),
    hebrew_only("cache", &[]),
    hebrew_only("CDN", &["סי די אן"]),
    hebrew_only("DNS", &["די אן אס"]),
    hebrew_only("SSL", &["אס אס אל"]),
    hebrew_only("OAuth", &["אוהאות'"]),
    hebrew_only("SSO", &["אס אס או"]),
    hebrew_only("CORS", &["קורס"]),
    hebrew_only("REST", &[]),
    hebrew_only("cron", &["קרון"]),
    hebrew_only("queue", &[]),
    hebrew_only("worker", &["וורקר"]),
    hebrew_only("container", &["קונטיינר"]),
    hebrew_only("rate limit", &["רייט לימיט"]),
    hebrew_only("payload", &["פיילואד"]),
    hebrew_only("RLS", &["אר אל אס"]),
    // Cloud and hosting.
    hebrew_only("Render", &["רנדר"]),
    hebrew_only("S3", &["אס תרי"]),
    hebrew_only("Lambda", &["למבדה"]),
    // Testing and quality.
    term("Cypress", &["סייפרס"]),
    hebrew_only("e2e", &["איטואי"]),
    hebrew_only("coverage", &["קאברג'"]),
    hebrew_only("mock", &[]),
    hebrew_only("fixture", &["פיקסצ'ר"]),
    hebrew_only("regression", &["רגרסיה"]),
    hebrew_only("snapshot", &["סנאפשוט"]),
    // Marketing, content and analytics.
    term("Metricool", &["מטריקול"]),
    term("Klaviyo", &["קלביו"]),
    term("ActiveCampaign", &["אקטיב קמפיין"]),
    term("Meta Ads", &["מטא אדס"]),
    term("Google Ads", &["גוגל אדס"]),
    hebrew_only("funnel", &["פאנל"]),
    hebrew_only("CTA", &["סי טי איי"]),
    hebrew_only("CTR", &["סי טי אר"]),
    hebrew_only("ROAS", &["רואס"]),
    hebrew_only("CPM", &["סי פי אם"]),
    hebrew_only("retargeting", &["ריטרגטינג"]),
    hebrew_only("lead magnet", &["ליד מגנט"]),
    hebrew_only("newsletter", &["ניוזלטר"]),
    hebrew_only("thumbnail", &["תמבנייל"]),
    hebrew_only("voiceover", &["voice over", "וויסאובר"]),
    hebrew_only("storyboard", &["סטוריבורד"]),
    hebrew_only("reels", &["ריעלס", "רילז"]),
    // Names the phonetic pass had to give up: each one quietly rewrote an
    // ordinary English word (AWS ate "as", Neon ate "none", Sora ate "sore",
    // Deno ate "done"). They keep working from their Hebrew spelling, which no
    // English word can collide with.
    hebrew_only("AWS", &["איי דאבליו אס"]),
    hebrew_only("Neon", &["ניאון"]),
    hebrew_only("Sora", &["סורה"]),
    hebrew_only("Deno", &["דינו"]),
    hebrew_only("Jest", &["ג'סטיס"]),
    // Israeli services and money.
    term("Meshulam", &["משולם"]),
    term("iCount", &["איי קאונט"]),
    term("Hashavshevet", &["חשבשבת"]),
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
fn lookup<'a>(
    key: &str,
    word_count: usize,
    index: &HashMap<String, &'a str>,
    fuzzy: bool,
) -> Option<&'a str> {
    if let Some(hit) = index.get(key) {
        return Some(hit);
    }
    if !fuzzy || word_count > 1 || key.chars().count() < FUZZY_HEBREW_KEY_LEN {
        return None;
    }
    index
        .iter()
        .find(|(candidate, _)| {
            candidate.chars().count() >= FUZZY_HEBREW_KEY_LEN && within_one_edit(key, candidate)
        })
        .map(|(_, canonical)| *canonical)
}

/// A geresh marks a Hebrew letter that stands for a foreign sound, so it
/// belongs to the word rather than to the sentence around it. Both the ASCII
/// apostrophe the model emits and the Unicode geresh count.
fn is_geresh(c: char) -> bool {
    c == '\'' || c == '\u{05F3}' || c == '\u{2019}'
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

            // Exact matches are exhausted before any fuzzy one is allowed,
            // including the prefix-stripped form. Fuzzy forgives a single
            // letter and a glued Hebrew prefix is exactly one letter, so
            // `וסופרבייס` would otherwise match `סופרבייס` by spending its
            // edit on the ו, and the conjunction would disappear from the
            // sentence. Which of the two won depended on HashMap order, so
            // the same dictation did not always come out the same way.
            for fuzzy in [false, true] {
                if let Some(canonical) = lookup(&key, n, &index, fuzzy) {
                    matched = Some((n, canonical.to_string()));
                    break;
                }
                // A glued prefix is only meaningful on the first word of the span.
                if let Some((prefix, rest)) = strip_prefix(&key) {
                    if let Some(canonical) = lookup(&rest, n, &index, fuzzy) {
                        matched = Some((n, format!("{prefix}-{canonical}")));
                        break;
                    }
                }
            }
            if matched.is_some() {
                break;
            }
        }

        match matched {
            Some((n, replacement)) => {
                // Trailing punctuation belongs to the sentence, not the name.
                // A geresh does not: in Hebrew it is part of the letter it
                // follows (ג׳, ז׳, ץ׳ are single sounds), so `ברנץ'` is one
                // word and used to come out as `branch'`.
                let last = words[i + n - 1];
                let suffix: String = last
                    .chars()
                    .rev()
                    .take_while(|c| !c.is_alphabetic() && !is_geresh(*c))
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

    /// Every name in the Latin list is a magnet for whatever sounds like it,
    /// which is the point and also the danger. Short brand names are the worst
    /// offenders: `Vaul` swallowed `value`, `Deno` swallowed `done`, `CLI`
    /// swallowed `call`, `OAuth` swallowed `out`. All four survived a reading
    /// of the table and every other test here, and were caught only by running
    /// real audio through the app.
    ///
    /// So the list is checked against ordinary English mechanically. A name
    /// that fails this belongs in `hebrew_only`, or nowhere.
    #[test]
    fn common_english_survives_the_latin_pass() {
        const COMMON_ENGLISH: &str = "\
the be to of and a in that have it for not on with he as you do at this but his by from they we
say her she or an will my one all would there their what so up out if about who get which go me
when make can like time no just him know take people into year your good some could them see
other than then now look only come its over think also back after use two how our work first
well way even new want because any these give day most us value done rest been none strict vote
sore soar sleek learner pillar mourning weeks dino stark start stop store sort short shirt
search server serve save size site style state stage scale skill still call cell sell fell full
fill file five love live leave list last least less lost cost case cause close clear clean click
clock course source force fourth forth north word world worse model module modal medal metal
mental mode more moon month main mean men man many money
fable fork audit remote merge stash session release excel desktop render neon vite sora deno bun
jest bit wolt was aws are art part past pass post pest best test text next note nose noise
grow grew green great grand grade grill guide gate gates keep kept help held hold hand land lane
line link lift left life like lake luck lock long lung sing song sung ring rang rung king kind";

        for word in COMMON_ENGLISH.split_whitespace() {
            let corrected = apply_custom_words(word, &latin_words(), THRESHOLD);
            assert_eq!(
                corrected, word,
                "{word:?} was rewritten to {corrected:?}; that name belongs in hebrew_only, or nowhere"
            );
        }
    }

    /// Taken verbatim from Daniel's transcription log. Every one of these was
    /// produced by the model on real audio and every one was wrong, which is
    /// the only reason the matching entries exist.
    #[test]
    fn the_words_that_actually_came_out_wrong_are_recovered() {
        for (spoken, expected) in [
            ("תעדכן את הקלוד אמדי", "תעדכן את ה-CLAUDE.md"),
            ("היא כתבה ריתמי וגיטיגנור", "היא כתבה README ו-.gitignore"),
            ("תשתמש בסקיל של גרילמי", "תשתמש ב-skill של grill-me"),
            ("תתייעץ עם פייבל", "תתייעץ עם Fable"),
            ("תריץ את הרמס במיני", "תריץ את Hermes במיני"),
            ("יש לי דיבלופר איידי", "יש לי Developer ID"),
            ("בסטייל של ליקוויד גלאס", "בסטייל של Liquid Glass"),
        ] {
            assert_eq!(correct(spoken), expected, "on {spoken:?}");
        }
    }

    /// A geresh is part of the Hebrew letter it follows, not punctuation
    /// closing the sentence. `ברנץ'` used to come out `branch'`.
    #[test]
    fn a_geresh_is_part_of_the_word_not_the_sentence() {
        assert_eq!(correct("תעשה ברנץ' חדש"), "תעשה branch חדש");
        assert_eq!(correct("טריאז' מהיר"), "triage מהיר");
        // Real punctuation after a geresh still belongs to the sentence.
        assert_eq!(correct("תעשה ברנץ'."), "תעשה branch.");
    }

    /// A glued prefix is one letter, and so is the edit the fuzzy pass
    /// forgives. Before the exact forms were exhausted first, `וסופרבייס`
    /// matched `סופרבייס` by spending its edit on the ו and the conjunction
    /// vanished, turning "and Supabase" into "Supabase". Which one won came
    /// down to HashMap order, so it did not even fail consistently.
    #[test]
    fn a_glued_prefix_is_never_spent_as_the_forgiven_edit() {
        for (spoken, expected) in [
            ("וסופרבייס", "ו-Supabase"),
            ("בטמפלייט", "ב-template"),
            ("הפייפליין", "ה-pipeline"),
            ("לגיטהאב", "ל-GitHub"),
        ] {
            assert_eq!(correct(spoken), expected, "on {spoken:?}");
        }
    }

    /// The same input has to give the same output. The fuzzy branch scans a
    /// HashMap and returns the first candidate within one edit, so an
    /// ambiguous key used to resolve differently between runs.
    #[test]
    fn the_same_dictation_always_produces_the_same_text() {
        let line = "תעלה וסופרבייס ובטמפלייט עם הקלוד אמדי וגיטיגנור";
        let first = correct(line);
        for _ in 0..200 {
            assert_eq!(correct(line), first);
        }
    }

    /// The everyday words that only reach the table through Hebrew letters.
    #[test]
    fn everyday_terms_are_recovered_from_hebrew_only() {
        assert_eq!(
            correct("תעשה אודיט על הטמפלייט"),
            "תעשה audit על ה-template"
        );
        assert_eq!(correct("תפתח פורק ותוסיף רימוט"), "תפתח fork ותוסיף remote");
        assert_eq!(
            correct("הסשיין נגמר בפייפליין"),
            "ה-session נגמר ב-pipeline"
        );
    }

    /// The one deliberate exception to the test above. A Hebrew speaker says
    /// `ענן` for the sky, so Latin-script `cloud` in a Hebrew transcription
    /// means they said "Claude", and correcting it is the most valuable single
    /// thing the Latin pass does.
    #[test]
    fn cloud_is_the_one_english_word_deliberately_swallowed() {
        assert_eq!(
            apply_custom_words("cloud", &latin_words(), THRESHOLD),
            "Claude"
        );
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

    /// The table is large enough that eyeballing it proves nothing. These are
    /// real sentences Daniel dictated, taken verbatim from session history and
    /// picked because they contain no term at all. A key that starts matching
    /// ordinary Hebrew shows up here first.
    #[test]
    fn ordinary_dictation_passes_through_untouched() {
        for sentence in [
            "אוקיי, אז מה נשאר כדי לעלות עם זה לאוויר, כדי שאני אוכל לתת לאנשים להתקין את זה",
            "אז אולי באמת להתרכז רגע בפרופיל של, לא יודע, אני חושב איתך ביחד",
            "אני רוצה לשנות את השם של האפליקציה מדבר ל דיבור",
            "אני שקלתי גם ליצור לזה איזה עמוד אינטרנט, משהו מגניב, שהם יוכלו להיכנס ולראות",
            "אתה חייב לשפר את זה שזה ייראה קטלני, שזה ייראה פרימיום, שזה ייראה טוב",
            "בנוסף, הייתי רוצה לארוז את זה בצבעים אחרים, עם שם אחר, לוגו אחר, והכל",
            "הרי אני הולך לתת את זה בחינם בהתחלה, ואולי אחר כך לגבות על זה כסף",
            "צריך רגע להבין מבחינת שעות, מבחינת פרקטיקה, כמה ימים לעשות את זה",
            "תעשה סדר ביומן ותבדוק שהשעות של הסטודנטים נכונות",
            "הם רוצים לעבוד כצוות, כקבוצה, וזה נראה לי יותר חשוב",
        ] {
            assert_eq!(correct(sentence), sentence, "corrupted: {sentence}");
        }
    }

    /// The other half of the same corpus: sentences that do contain a term,
    /// with the correction each one is supposed to get.
    #[test]
    fn real_dictation_gets_the_corrections_it_should() {
        for (spoken, expected) in [
            (
                "עדיין יש עוד מושגים שלא הכנסת, כמו רב מסר",
                "עדיין יש עוד מושגים שלא הכנסת, כמו Rav Messer",
            ),
            (
                "תקרא את המסמכים הרשמיים של קלוד קוד החדשים",
                "תקרא את המסמכים הרשמיים של Claude Code החדשים",
            ),
            (
                "המודל שמתאים לעברית זה הוויספר מידיום",
                "המודל שמתאים לעברית זה ה-Whisper מידיום",
            ),
            (
                "ונגיד שאני אולי עובד בוואטסאפ",
                "ונגיד שאני אולי עובד ב-WhatsApp",
            ),
            (
                "שתעבוד על כל המילים שהן רלוונטיות לקלוד קוד",
                "שתעבוד על כל המילים שהן רלוונטיות ל-Claude Code",
            ),
            (
                "תבנה את האתר בוורדפרס ותחבר את קארדקום",
                "תבנה את האתר ב-WordPress ותחבר את CardCom",
            ),
        ] {
            assert_eq!(correct(spoken), expected);
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
