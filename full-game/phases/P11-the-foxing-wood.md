# Phase 11: The Foxing Wood

**Goal:** Add the Foxing Wood and Q5 as the first update.

**Suggested model:** Sonnet 5.5, high effort, for most of the phase: the build list, terrain, settlements, dungeons, characters, loot, presets, tests, and notes are content built to the Vale's standard. Switch to Opus 5.5 at high effort for the task that adds species and bosses, for the tasks that write quests, books, and surface text, and for the new dialect's first dialogue.

**Hand to Haiku 5.5:** Light characters and their schedules, small tasks, loot and ingredient placement, kit interiors from the layout rules, and voice line integration. Haiku never writes dialogue for Full characters, quests, or books. Rules are in `CLAUDE.md` under Models and helpers.

**Release:** Update: The Foxing Wood

**Requires:** Phase 10 finished and tagged `phase-10-done`.

## Read for this phase

Read only these files in `docs/brief/`, and only the named part where one is given.

- `0a-pillars-and-releases.md`: Cost tiers
- `4a-world.md`
- `2a-signature-visuals.md` and `5h-signature-mechanics.md`: only the parts `docs/SCOPE.md` assigns to this phase
- `4b-content-and-story.md`: Content target only
- `4c-content-rules.md`
- `5c-enemies-and-bosses.md`
- `5d-loot.md`: Content targets
- `3b-renderer-surfaces.md`: only what this Quire's weather and surfaces need
- `1.1a-licenses-and-ai-gate.md`: AI gate, only if using AI assets

## Canon for this phase

These files are under `docs/`.

- SCOPE.md: The Foxing Wood
- voice/cast-and-budget.md: Rules and Credit budget
- bible/03-voice-and-dialects.md: The Foxing Wood
- bible/04-quires-and-places.md: The Foxing Wood
- bible/06-characters.md: The Foxing Wood
- bible/07-bestiary-and-bosses.md: this Quire's species and bosses
- bible/08-words-items-craft.md: items and ingredients found here
- bible/09-books.md: this Quire's books
- script/02-act-one.md: Q5
- script/06-questlines.md: quests set in The Foxing Wood
- script/07-authors-and-side-quests.md: The Foxing Wood
- script/08-secrets-and-small-tasks.md: The Foxing Wood

## Scope

The Foxing Wood to the standard the Vale set, at the tiers in `docs/SCOPE.md`: terrain, settlements, dungeons, NPCs, species, bosses, loot, quests, books, and voice. Opens the Wood road from the Vale. Place the Comma sort at a point of interest on the build list.

## Tasks

Do these in order, one at a time, following the task loop in `CLAUDE.md`.

- [ ] **P11-T01** The Foxing Wood: write the build list in `docs/SCOPE.md` from the canon, naming its waiting places and opening any earlier ones that waited for this Quire's words, using the tier rules and the owner's Slice Gate decisions, and tell the owner what is Light and what is Reserve
- [ ] **P11-T02** The Foxing Wood: terrain and landmark pass, materials, colour grade by time of day, and this Quire's weather and particles
- [ ] **P11-T03** The Foxing Wood: build its settlements, small sites, and points of interest, including its loose words QUIET and LOST with their sheets, typos, motifs, and the compounds and annotations they complete, and every Standing Page as an event with its guardian or puzzle, its learning spectacle, and the full signature effect of its word
- [ ] **P11-T04** The Foxing Wood: build its Hero dungeons and the kit-built dungeons on the build list, each with its marked structures
- [ ] **P11-T05** The Foxing Wood: add its named NPCs at their Full or Light tier, with schedules and dialogue
- [ ] **P11-T06** The Foxing Wood: add its native species, variants, and bosses, each with its word reactions and annotations and its loud and quiet glyph sets, and ragdolls for any rig family not yet covered
- [ ] **P11-T07** The Foxing Wood: add its loot, ingredients, and the Canonical items found here
- [ ] **P11-T08** The Foxing Wood: write Q5, What the Wood Keeps
- [ ] **P11-T09** The Foxing Wood: write the questline quests, Author quests, side quests, small tasks, and secrets on the build list
- [ ] **P11-T10** The Foxing Wood: write its books and surface text
- [ ] **P11-T11** The Foxing Wood: lock and generate this update's voice lines by priority, after owner approval and within the credit budget, and integrate them
- [ ] **P11-T12** The Foxing Wood: run all region tests, the scope check, and the old-save test, and fix what they find
- [ ] **P11-T13** The Foxing Wood: add this update's steps to `docs/PLAYTEST_CHECKLIST.md`, translate this update's text into every supported language and pass the localization check, write the update notes, and push a private build

## Exit gate

The region passes its bots, validator, density map, variety report, scope check, and old-save test before the next begins

Save the evidence in `docs/evidence/P11/`.

## Owner gate: Update Gate

**Stop. Give me a private build with The Foxing Wood, its update notes, and the effort it took against the slice report's estimate. Wait for approval before continuing.**

What the owner checks:

- A save from the previous release loads and carries on
- The new Quire holds 30 fps on your M1, with the capture saved as evidence
- The new main-story quest can be finished, alone and with a partner
- The Quire has its own light, colour, speech, and grudges, and does not feel like the Vale repainted
- At least one new word changed how you play
- The effort matched the estimate, or the tiers for later Quires have been adjusted

**It passes when** the owner approves the update for release, or asks to hold it. Either answer lets the next phase start.

If it does not pass, fix what the owner reports inside this phase and ask again. Do not start the next phase.

Set `docs/STATE.md` to **Waiting on owner**, and tag `phase-11-done` only after approval.

## Next

Move to `phases/P12-saltmargin.md`.
