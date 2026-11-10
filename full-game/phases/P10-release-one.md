# Phase 10: Release 1

**Goal:** Turn the Vale into private, installable early-access builds and a store kit for approval.

**Suggested model:** Opus 5.5, high effort. Signing, storefronts, and the performance pass are exacting and have no room for invention.

**Hand to Haiku 5.5:** Credits and third-party notices, the achievement list, store text drafts for the owner to edit, and playtest checklist upkeep. Rules are in `CLAUDE.md` under Models and helpers.

**Release:** Release 1

**Requires:** Phase 09 finished and tagged `phase-09-done`.

## Read for this phase

Read only these files in `docs/brief/`, and only the named part where one is given.

- `1b-storefront-and-release.md`
- `1.1a-licenses-and-ai-gate.md`: Disclosure
- `5g-input.md`: Steam Deck requirements
- `8-proof-tooling.md`: Storefront, Deck, and License audit
- `10-definition-of-done.md`: Every release

## Canon for this phase

These files are under `docs/`.

- script/01-title-intro-prologue.md: Title sequence
- bible/03-voice-and-dialects.md: for store text

## Scope

Title sequence, final settings, Steam and itch integration, Steam Deck pass, notarized packaging, performance and bug pass, and an early-access store kit for my approval: store text, screenshots, a captured trailer, achievement list, draft Steam content survey, early-access statement, and AI disclosure

## Tasks

Do these in order, one at a time, following the task loop in `CLAUDE.md`.

- [ ] **P10-T01** Build the title sequence
- [ ] **P10-T02** Finish settings and graphics presets
- [ ] **P10-T03** Finish Steam integration: cloud, achievements, rich presence, invites, and timeline marks for marked moments if the crate exposes them
- [ ] **P10-T04** Finish the itch.io build and `.itch.toml`
- [ ] **P10-T05** Run the Steam Deck pass and write the Verified checklist
- [ ] **P10-T06** Create the signing and notarization scripts, using credentials from the owner
- [ ] **P10-T07** Run the performance and memory pass, with the owner's minimum machine capture across the whole Vale, solo and in split-screen
- [ ] **P10-T08** Run the bug pass, and set up the old-save test that every later release must pass
- [ ] **P10-T09** Generate credits and `THIRD_PARTY_NOTICES.txt`, update `docs/AI_DISCLOSURE.md`, and run the license audit
- [ ] **P10-T10** Create the store kit, following How the game is presented in `1b-storefront-and-release.md`: key art, text that states exactly what Release 1 contains, screenshots taken in photo mode with at least three split frames, trailer capture, achievement list, Steam content survey draft, early-access statement, and AI disclosure
- [ ] **P10-T10a** Write the glossaries in `docs/loc/`, translate all of Release 1's text into German, French, Spanish, and Brazilian Portuguese, pass the localization check, give the owner the samples to read, and mark any unread language as beta in the store text
- [ ] **P10-T11** Push private Steam beta and private itch builds, with the owner's approval
- [ ] **P10-T12** Have the owner complete `docs/PLAYTEST_CHECKLIST.md`, and fix what is found
- [ ] **P10-T13** Run the final bots, and write the release report

## Exit gate

Private Steam beta and private itch builds install and run on Windows, macOS, Linux, and Steam Deck, with full solo and co-op bot runs, my completed playtest checklist, and a written release report

Save the evidence in `docs/evidence/P10/`.

## Owner gate: Release Gate

**Stop. Give me the private builds, the store kit, and the release report. Publish nothing. Wait for approval before continuing.**

What the owner checks:

- The private build installs and runs on each machine you own
- The playtest checklist is complete, including real controllers and Steam Deck if you have one
- The store text says what is in Release 1 and promises no dates
- The trailer and screenshots were captured from the game
- The AI disclosure and credits are accurate

**It passes when** the owner says, in so many words, to publish, or says to hold. Either answer lets work continue on the next phase.

If it does not pass, fix what the owner reports inside this phase and ask again. Do not start the next phase.

Set `docs/STATE.md` to **Waiting on owner**, and tag `phase-10-done` only after approval.

## Next

Move to `phases/P11-the-foxing-wood.md`. Do not publish without the owner's explicit approval.
