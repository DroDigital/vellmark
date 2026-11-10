# Phase 15: The Cinder Folio

**Goal:** Add the Cinder Folio and Q10, and bring Caret down on the coast.

**Suggested model:** Sonnet 5.5, high effort, for most of the phase: the build list, terrain, settlements, dungeons, characters, loot, presets, tests, and notes are content built to the Vale's standard. Switch to Opus 5.5 at high effort for the task that adds species and bosses, for the tasks that write quests, books, and surface text, and for Caret's world event.

**Hand to Haiku 5.5:** Light characters and their schedules, small tasks, loot and ingredient placement, kit interiors from the layout rules, and voice line integration. Haiku never writes dialogue for Full characters, quests, or books. Rules are in `CLAUDE.md` under Models and helpers.

**Release:** Update: The Cinder Folio

**Requires:** Phase 14 finished and tagged `phase-14-done`.

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

- SCOPE.md: The Cinder Folio
- voice/cast-and-budget.md: Rules and Credit budget
- bible/03-voice-and-dialects.md: The Cinder Folio
- bible/04-quires-and-places.md: The Cinder Folio
- bible/06-characters.md: The Cinder Folio
- bible/07-bestiary-and-bosses.md: this Quire's species and bosses
- bible/08-words-items-craft.md: items and ingredients found here
- bible/09-books.md: this Quire's books
- script/03-act-two.md: Q10
- script/06-questlines.md: quests set in The Cinder Folio
- script/07-authors-and-side-quests.md: The Cinder Folio
- script/08-secrets-and-small-tasks.md: The Cinder Folio

## Scope

The Cinder Folio to the standard the Vale set, at the tiers in `docs/SCOPE.md`: terrain, settlements, dungeons, NPCs, species, bosses, loot, quests, books, and voice. Also Caret's world event in Saltmargin.

## Tasks

Do these in order, one at a time, following the task loop in `CLAUDE.md`.

- [ ] **P15-T01** The Cinder Folio: write the build list in `docs/SCOPE.md` from the canon, naming its waiting places and opening any earlier ones that waited for this Quire's words, using the tier rules and the owner's Slice Gate decisions, and tell the owner what is Light and what is Reserve
- [ ] **P15-T02** The Cinder Folio: terrain and landmark pass, materials, colour grade by time of day, and this Quire's weather and particles
- [ ] **P15-T03** The Cinder Folio: build its settlements, small sites, and points of interest, including its loose words FAST and BRAVE with their sheets, typos, motifs, and the compounds and annotations they complete, and every Standing Page as an event with its guardian or puzzle, its learning spectacle, and the full signature effect of its word
- [ ] **P15-T04** The Cinder Folio: build its Hero dungeons and the kit-built dungeons on the build list, each with its marked structures
- [ ] **P15-T05** The Cinder Folio: add its named NPCs at their Full or Light tier, with schedules and dialogue
- [ ] **P15-T06** The Cinder Folio: add its native species, variants, and bosses, each with its word reactions and annotations and its loud and quiet glyph sets, and ragdolls for any rig family not yet covered
- [ ] **P15-T07** The Cinder Folio: add its loot, ingredients, and the Canonical items found here
- [ ] **P15-T08** The Cinder Folio: write Q10, Red Ink
- [ ] **P15-T09** The Cinder Folio: write the questline quests, Author quests, side quests, small tasks, and secrets on the build list
- [ ] **P15-T10** The Cinder Folio: write its books and surface text
- [ ] **P15-T11** Saltmargin: build Caret's attack as a world event, with Caret as a Full boss, and everything it struck out writing itself back when it falls
- [ ] **P15-T12** The Cinder Folio: lock and generate this update's voice lines by priority, after owner approval and within the credit budget, and integrate them
- [ ] **P15-T13** The Cinder Folio: run all region tests, the scope check, and the old-save test, and fix what they find
- [ ] **P15-T14** The Cinder Folio: add this update's steps to `docs/PLAYTEST_CHECKLIST.md`, translate this update's text into every supported language and pass the localization check, write the update notes, and push a private build

## Exit gate

The region passes its bots, validator, density map, variety report, scope check, and old-save test before the next begins

Save the evidence in `docs/evidence/P15/`.

## Owner gate: Update Gate

**Stop. Give me a private build with The Cinder Folio, its update notes, and the effort it took against the slice report's estimate. Wait for approval before continuing.**

What the owner checks:

- A save from the previous release loads and carries on
- The new Quire holds 30 fps on your M1, with the capture saved as evidence
- The new main-story quest can be finished, alone and with a partner
- The Quire has its own light, colour, speech, and grudges, and does not feel like the Vale repainted
- At least one new word changed how you play
- The effort matched the estimate, or the tiers for later Quires have been adjusted

**It passes when** the owner approves the update for release, or asks to hold it. Either answer lets the next phase start.

If it does not pass, fix what the owner reports inside this phase and ask again. Do not start the next phase.

Set `docs/STATE.md` to **Waiting on owner**, and tag `phase-15-done` only after approval.

## Next

Move to `phases/P16-the-whitemarch.md`.
