# Phase 20: Version 1.0

**Goal:** Polish the finished game and prepare the full release for approval.

**Suggested model:** Opus 5.5, high effort. A final exacting pass, with nothing new to design.

**Hand to Haiku 5.5:** Final credits and notices, store text updates, and collecting test evidence. Rules are in `CLAUDE.md` under Models and helpers.

**Release:** 1.0

**Requires:** Phase 19 finished and tagged `phase-19-done`.

## Read for this phase

Read only these files in `docs/brief/`, and only the named part where one is given.

- `1b-storefront-and-release.md`
- `1.1a-licenses-and-ai-gate.md`: Disclosure
- `5g-input.md`: Steam Deck requirements
- `8-proof-tooling.md`: Storefront, Deck, and License audit
- `10-definition-of-done.md`

## Canon for this phase

These files are under `docs/`.

- bible/03-voice-and-dialects.md: for store text

## Scope

Performance and bug pass across the whole realm, the density map and variety report for the full game, final Steam Deck pass, final packaging, and a 1.0 store kit for my approval

## Tasks

Do these in order, one at a time, following the task loop in `CLAUDE.md`.

- [ ] **P20-T01** Run the density map and variety report across the full realm, and fix empty or repetitive areas
- [ ] **P20-T02** Run the performance and memory pass across every Quire, solo and in split-screen
- [ ] **P20-T03** Run the world soak test and the bug pass, and test a save from every earlier release
- [ ] **P20-T04** Run the final Steam Deck pass and update the Verified checklist
- [ ] **P20-T05** Check every item in `docs/SCOPE.md` marked for 1.0, and report anything unbuilt to the owner
- [ ] **P20-T06** Generate final credits and `THIRD_PARTY_NOTICES.txt`, and run the license audit
- [ ] **P20-T07** Update the store kit for 1.0, in every supported language: text, screenshots, trailer capture, achievement list, and the end of early access
- [ ] **P20-T08** Push private Steam beta and private itch builds, with the owner's approval
- [ ] **P20-T09** Have the owner complete `docs/PLAYTEST_CHECKLIST.md`, and fix what is found
- [ ] **P20-T10** Run the final bots, and write the release report

## Exit gate

Private builds install and run on Windows, macOS, Linux, and Steam Deck, with full solo and co-op bot runs to every ending, my completed playtest checklist, and a written release report

Save the evidence in `docs/evidence/P20/`.

## Owner gate: Release Gate

**Stop. Give me the private 1.0 builds, the store kit, and the release report. Publish nothing. Wait for approval before continuing.**

What the owner checks:

- You, or someone you trust, finished the main quest on a private build
- All four endings were reached at least once
- Everything in `docs/SCOPE.md` marked for 1.0 is built, or you chose to leave it out
- The store text describes the finished game

**It passes when** the owner says to publish.

If it does not pass, fix what the owner reports inside this phase and ask again. Do not start the next phase.

Set `docs/STATE.md` to **Waiting on owner**, and tag `phase-20-done` only after approval.

## Next

The project is complete when `docs/brief/10-definition-of-done.md` is satisfied. Do not publish without the owner's explicit approval.
