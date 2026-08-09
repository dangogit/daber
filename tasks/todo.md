# Dibur: a built-in vocabulary for the words developers actually say

Dibur is being positioned as the dictation app for working with Claude Code in
Hebrew. That claim fails on the first sentence if the app cannot spell "Claude".

## The evidence

Daniel dictated a message about this feature. The model transcribed it as
**"Cloud Code"**, twice, in the same message where he asked for the fix. He then
dictated the confirmation, and it happened twice more.

Two things are true about the model's behaviour, both measured on that audio:

1. **It already emits Latin script for English words.** `built-in` and `value`
   came out correctly. So this is not a transliteration problem, it is a
   spelling problem on proper nouns.
2. **The whisper initial prompt does not fix it.** The same clip was run with
   `--prompt "Claude Code, Supabase, Tauri, Vercel, ..."` and still produced
   "Cloud Code", twice. The ivrit fine-tune is biased hard enough toward Hebrew
   that decoder priming does not move it.

## The bug

Dibur has two mechanisms for custom words, and on our models exactly the wrong
one is active.

`post_process_transcription_text` is called with `custom_words_already_prompted:
model_is_whisper` (`managers/transcription.rs:1401`). Every model Dibur ships is
whisper, so the fuzzy post-correction is always skipped, on the assumption that
the initial prompt already handled it. Finding 2 above shows it does not.

The correction path is the one that works here: `cloud` and `claude` share the
Soundex code C430, which drops the score to 0.1, under the 0.18 threshold.

## Plan

- [x] Always run `apply_custom_words`, on every model. Keep the initial prompt
      for user words, but stop treating it as proof the work is done.
- [x] Add a curated built-in vocabulary, merged with the user's own words.
- [x] Setting `dev_vocabulary`, default on, so a non-developer can turn it off.
- [x] UI toggle, English copy, Hebrew copy, seeded translations.
- [x] Tests, including "Cloud Code" -> "Claude Code" on the real failure.

## What goes in the list, and what does not

The matcher only ever looks at ASCII tokens, so Hebrew words are never touched.
The risk is that an incidental English word gets pulled toward a listed term.

So the rule is: **product names, brand names and jargon that is not an ordinary
English word.** Whisper already spells `commit`, `deploy` and `refactor`
correctly, so listing them buys nothing and puts near-homophones at risk
(`march` -> `merge`, `stayed` -> `state`). Listing `Supabase`, `pgvector`,
`shadcn` and `Claude Code` buys everything, because those are exactly the words
the model has never seen enough of.

`Claude` is listed on its own despite colliding with `cloud`. A Hebrew speaker
says `ענן`, not `cloud`, so for a Latin-script `cloud` to appear at all the
speaker almost certainly said "Claude".

## Review

**Proved end to end, not just in a unit test.** The debug binary was run with
`--transcribe-file` over six of Daniel's own recordings. The two clips that
previously produced "Cloud Code" now produce "Claude Code", four occurrences in
total, and `built-in` and `value` came through untouched. The other four clips
are byte-identical to what the old build produced, so the wider vocabulary
introduced no collateral damage on real Hebrew speech.

**Cost.** Correction over the full ~120-term list takes 25 ms for a 240-word
transcription, measured. Transcription itself is seconds, and correction runs
once per utterance, so this does not register.

**A limitation worth naming.** The matcher only ever inspects ASCII candidates,
so it cannot help when the model writes a technical term in Hebrew letters. In
these same recordings, "Modes" came out as `מוז`, and nothing in this change
touches that. Fixing it needs a Hebrew-to-English term map, which is a separate
piece of work.

## Follow-up: the Hebrew half, which turned out to be the larger half

Daniel tried the shipped build and reported the real shape of the problem: he
says "קלוד קוד", "ורסל", "גיטאב", "סופר בייס", and they come out in Hebrew
letters, where the pass above cannot see them. His own history shows how
arbitrary the model is about it: one recording contains `GitHub` and `workflow`
in Latin script next to `סקילים` and `Cloud Code`, in one breath.

So `apply_hebrew_terms` was added, running before the Latin pass. It is a
lookup rather than a phonetic matcher, because Soundex means nothing across
scripts:

- `normalize_hebrew` folds away what carries no meaning: final letter forms,
  doubled vav and yod, niqqud, gershayim, and the space inside a two-word name.
  One table entry then covers `וורסל`, `ורסל` and `ורסל,` alike. Five spellings
  were removed from the table once a test proved normalization already covered
  them.
- Keys under 4 characters are rejected, and one wrong letter is tolerated only
  at 6 characters or more. `קוד` and `סופר` are ordinary Hebrew, so only the
  complete term is ever a key.
- A glued prefix is peeled and put back with a maqaf, so `בוורסל` becomes
  `ב-Vercel` rather than being missed.

Verified end to end on the two clips where Daniel named the terms out loud:
`זה נגיד אני אומר Claude Code` and `נגיד Vercel, GitHub, Supabase`. The other
seven recordings are unchanged.

Left in Hebrew on purpose: `סקיל` and `וורקפלו`. They read naturally in a
Hebrew sentence, and this table is for product names.

**Deliberately left alone.** `cargo clippy -D warnings` fails on this repo, but
every finding predates this branch (`portable.rs` `write_with_newline`,
`items_after_test_module` in `transcription.rs`, and an unused assignment at
`transcription.rs:1177`). CI runs `bun run lint` and `cargo test`, not clippy,
so none of it gates this work and none of it is mine to widen scope over.
