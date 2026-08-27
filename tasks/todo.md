# Dibur 2.0: reliable capture and fast Hebrew transcription

## Outcome

Ship a distribution-ready macOS and Windows release that never loses recorded
audio, captures the first spoken word, cannot finish setup without working local
transcription, and formats Hebrew output without a second language model.

## Work plan

- [x] Add deterministic red-capable regression tests for VAD-empty audio,
      first-frame capture, and incomplete model provisioning.
- [x] Preserve raw audio separately from VAD-filtered audio and fall back to it
      whenever speech filtering yields no samples.
- [x] Move capture ahead of visual side effects and add a bounded in-memory
      pre-roll for instant mode without persisting pre-trigger audio.
- [x] Make model provisioning and a real test dictation a hard onboarding gate,
      including returning-user recovery for missing/corrupt models.
- [x] Remove the slow local text model and its runtime after a real dictation
      spent 31 seconds waiting for it and then fell back to the original text.
- [x] Normalize whitespace and add paragraph breaks at existing sentence
      boundaries without changing letters or numbers.
- [x] Prove offline operation and real microphone behavior on this Mac.
- [ ] Build and verify signed/notarized macOS artifacts and build/verify the
      Windows release pipeline artifacts.
- [ ] Run final review, security gates, CI, merge, release, and cleanup.

## Review

Current Mac proof passed in a Developer ID signed app installed under
`/Applications`: the packaged window opens without blocking on CoreAudio, the
always-on stream initializes, and Carmit began speaking 250 ms before the
shortcut. Dibur preserved the first word, captured 118,080 raw samples,
transcribed the full Hebrew sentence and pasted the result into TextEdit. A later
16.47-second dictation took 1.99 seconds in the transcription engine, while the
retired text model added 31 seconds and returned no usable result. Mandatory
speech-model provisioning also passed. Notarized release artifacts, Windows CI
package proof, merge, publication, and cleanup remain open.

---

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

## Follow-up: Claude Code's own words

That last decision was wrong, and Daniel said so: the point of the tool is
working with Claude Code, so its vocabulary is the vocabulary that matters most.
Asking for a `סקיל` is a guess; asking for a `skill` is a skill.

Read the official glossary at code.claude.com and took the spellings from it:
skill, subagent, hook, plugin, goal, plan mode, checkpoint, worktree, artifact,
teleport, compact, context window, MCP, prompt, token. `/goal` is a real
command, which settles the `גול` question.

Three things fell out of doing this properly:

**The two passes want different words.** The Latin pass matches phonetically, so
every word it knows also swallows what merely sounds like it, and an ordinary
English word there is pure risk. The Hebrew pass is a lookup against Hebrew
letters, which no English word can collide with. So `קומיט` can safely become
`commit` while `commit` itself stays out of the Latin list. `Term` grew a
`latin_pass` flag and a `hebrew_only` constructor.

**Six characters was too short a fuzzy floor.** `סקילים` (skills) and `מקילים`
(they ease) differ by one letter. Raised to seven, with a test.

**A joined span spends its edit on the next word.** `סאב אייג'נט עם` is one
letter from the key for `subagents`, so a three-word span ate the preposition
after it and produced "subagents hooks". Fuzzy matching is now single-word only;
multi-word terms must match exactly. Caught by a test, not by reading.

Nothing emits a leading slash. `/goal` and `/compact` are real commands, and
pasted text starting with one would invoke them.

Verified on the real recording: `כל מיני סקילים` now transcribes as
`כל מיני skills`. The other eight clips are unchanged.

## Follow-up: breadth, and the bug that breadth exposed

Daniel asked for the rest of it: WordPress, Rav Messer, "the language people
speak with Claude Code, all the tools, everything people work with in Israel."

The list was built from evidence rather than recall. Dependency manifests
across this workspace name the real stack (Radix, TipTap, TanStack, Remotion,
Capacitor, Zustand, Upstash). A grep for Israeli services ranked them by how
often they actually appear: CardCom 2861, Priority 1018, Wix 591, Rav Messer 83.
The MCP servers on this machine named the rest. That took the table to 160
Hebrew keys.

**The breadth is what broke it.** End-to-end on real audio, `value` came out as
`Vaul`, three times. A four-letter UI library in the phonetic pass had eaten
one of the most common words in his vocabulary. Reading the table did not catch
it. The existing tests did not catch it. Only running the app did.

So the check became mechanical: push a list of common English words through the
Latin pass and fail on any that change. That found eleven more of exactly the
same shape.

| word           | was becoming     |
| -------------- | ---------------- |
| `out`          | OAuth            |
| `call`, `cell` | CLI              |
| `done`, `dino` | Deno             |
| `rest`         | Rust, then React |
| `been`         | Bun              |
| `none`         | Neon             |
| `course`       | CORS             |
| `vote`         | Vite             |
| `sore`, `soar` | Sora             |
| `strict`       | struct           |
| `sleek`        | Slack            |
| `pillar`       | Polar            |
| `mourning`     | Morning          |

Thirteen names were dropped from the Latin pass outright; Slack, Morning and
React kept their Hebrew spellings and lost their phonetic entry. `cloud` ->
`Claude` is now the single deliberate exception, and has its own test saying so.

The lesson is narrower than "no ordinary English words", which was already the
rule. It is: **no short brand name that sounds like an ordinary English word**,
and no way to know which those are except to test them.

Also added a corpus test built from Daniel's real dictation history: ten
sentences that must pass through untouched, six that must be corrected. All
forty sentences in the extracted corpus were checked by hand first, and every
change the table made to them was correct.

**Deliberately left alone.** `cargo clippy -D warnings` fails on this repo, but
every finding predates this branch (`portable.rs` `write_with_newline`,
`items_after_test_module` in `transcription.rs`, and an unused assignment at
`transcription.rs:1177`). CI runs `bun run lint` and `cargo test`, not clippy,
so none of it gates this work and none of it is mine to widen scope over.

## Follow-up: an update mechanism that was wired to fail

Daniel has the Apple Developer membership, and asked before launch whether the
app can ship updates to people who already installed it. It could not, and the
way it could not was the dangerous kind.

`tauri-plugin-updater` was registered, `update_checks_enabled` defaulted to
**true**, and the app really did check for updates on launch. So every install
believed it was covered. Four things underneath meant it never was:

1. **The public key belonged to cjpais**, inherited with the fork
   (`BAB72095206601F9`). Only updates signed with the matching private key are
   accepted, and that key is not ours. Anything Daniel signed would be rejected.
2. **The endpoint already pointed at `dangogit/daber`.** Worst of both: it looks
   in our repo and trusts someone else's key, so neither side can ever ship.
3. **`createUpdaterArtifacts` was `false`**, so no `.tar.gz` or `.sig` was ever
   produced.
4. **Nothing generated `latest.json`**, the file the endpoint asks for.

Fixed: new minisign keypair, private half in the Keychain as
`dibur-updater-private-key` / `dibur-updater-key-password` alongside the other
Dibur secrets, public half in `tauri.conf.json`, artifacts enabled, and both
halves uploaded to the repo's Actions secrets, which previously held none at all.

`latest.json` turned out not to need writing: `tauri-action` generates it, and
reading its source (`upload-version-json.ts`) shows it downloads the copy
already attached to the release and merges its own platform in, so the seven-way
matrix does compose rather than overwrite.

That merge is still a read-modify-write from seven parallel jobs against one
file, and its asset listing is capped at `per_page: 50`. Both failure modes end
in a manifest that is merely _incomplete_, which an installed app cannot
distinguish from "no update for my platform". It just goes quiet, so
`verify-updater-manifest` now downloads the finished manifest and fails the
release unless all six platform keys carry a url and a signature, and the
version matches the tag.

Also: `release.yml` still passed `asset-prefix: "handy"`, left from the rename.
It happens to be inert, because the step that consumes it is gated on
`asset-name-pattern` which the release never sets. Corrected anyway rather than
left as a trap for whoever reads it next.

Version moved 0.9.4 -> 1.0.0 across `tauri.conf.json`, `package.json` and
`Cargo.toml`. The release workflow reads the first of those for the tag.

**What this does not fix.** Every copy already installed, including the one sent
to Natalie, has cjpais's public key compiled in. Those installs can never accept
an update from us, by design. They need one manual reinstall of 1.0.0, and from
that build onward updates flow on their own.

## Release: v1.0.0, shipped 2026-08-17

Published at https://github.com/dangogit/daber/releases/tag/v1.0.0, seven build
targets, 27 assets, signed and notarized on macOS.

### What the last two rebuilds were actually for

CI went green on a release whose macOS disk images were **signed but not
notarized**. `tauri-action` notarizes the `.app` and leaves the `.dmg` around it
alone, and nothing in the pipeline looked at the file a person downloads. It was
caught by fetching the real asset and setting `com.apple.quarantine` by hand:

```
spctl -a -t open --context context:primary-signature Dibur_1.0.0_aarch64.dmg
rejected
source=Unnotarized Developer ID
```

while the app inside reported `accepted / source=Notarized Developer ID`. A
green pipeline was describing a build that Gatekeeper would have refused on a
stranger's Mac.

CI now notarizes and staples the image itself and runs `spctl` before upload, so
this fails the release instead of reaching a user.

### Verified on the published artifacts, not on the logs

|                                | ARM                              | Intel                            |
| ------------------------------ | -------------------------------- | -------------------------------- |
| `spctl` on the quarantined DMG | accepted, Notarized Developer ID | accepted, Notarized Developer ID |
| `stapler validate`             | passes                           | passes                           |
| app inside                     | accepted, Notarized              | accepted, Notarized              |
| architecture                   | arm64                            | x86_64                           |

The Intel bundle carried its own history: `libonnxruntime` used to be signed by
another team, which made dyld refuse to load it. It now signs under `HK5L5QHW96`
with no unsatisfied `@rpath` dependencies, and the binary ran for eight seconds
under Rosetta with an empty stderr.

All 18 platform entries in `latest.json` are signed with `77CE8BFE3263B401`, the
key compiled into this build. Updates from 1.0.0 onward will be accepted.

### Note on publishing

`make_latest` does not take effect when it is sent in the same PATCH that clears
`draft`. The release published correctly but `releases/latest` kept returning
`models-v1` until `make_latest` was sent again on its own. Worth knowing before
the next release: send it as a second call, then check.
