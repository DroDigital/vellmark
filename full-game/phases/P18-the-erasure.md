# Phase 18: The Erasure

**Goal:** Add the Erasure as a place to walk into, and Act Three up to the Last Scriptorium.

**Suggested model:** Fable 5.2, medium effort. The unravelling world is the renderer's hardest effect and the story's turn.

**Hand to Haiku 5.5:** Light characters and their schedules, small tasks, loot and ingredient placement, kit interiors from the layout rules, and voice line integration. Haiku never writes dialogue for Full characters, quests, or books. Rules are in `CLAUDE.md` under Models and helpers.

**Release:** Update: The Erasure

**Requires:** Phase 17 finished and tagged `phase-17-done`.

## Read for this phase

Read only these files in `docs/brief/`, and only the named part where one is given.

- `0a-pillars-and-releases.md`: Cost tiers
- `4a-world.md`
- `2a-signature-visuals.md`: The Erasure unravels
- `5h-signature-mechanics.md`: The Remembered
- `4b-content-and-story.md`: Content target only
- `4c-content-rules.md`
- `5c-enemies-and-bosses.md`
- `5d-loot.md`: Content targets
- `3b-renderer-surfaces.md`: only what this Quire's weather and surfaces need
- `1.1a-licenses-and-ai-gate.md`: AI gate, only if using AI assets

## Canon for this phase

These files are under `docs/`.

- SCOPE.md: The Erasure
- voice/cast-and-budget.md: Rules and Credit budget
- bible/03-voice-and-dialects.md: The Erasure
- bible/04-quires-and-places.md: The Erasure
- bible/06-characters.md: The Erasure
- bible/07-bestiary-and-bosses.md: this Quire's species and bosses
- bible/08-words-items-craft.md: items and ingredients found here
- bible/09-books.md: this Quire's books
- script/04-act-three.md: Q15 to Q17
- `docs/brief/4a-world.md`: How the Erasure advances
- script/06-questlines.md: quests set in The Erasure
- script/07-authors-and-side-quests.md: The Erasure
- script/08-secrets-and-small-tasks.md: The Erasure

## Scope

The Erasure to the standard the Vale set, at the tiers in `docs/SCOPE.md`: terrain, settlements, dungeons, NPCs, species, bosses, loot, quests, books, and voice. Also Deleatur and Erasure Stage 3. The Gutter is built in Phase 19.

## Tasks

Do these in order, one at a time, following the task loop in `CLAUDE.md`.

- [ ] **P18-T01** The Erasure: write the build list in `docs/SCOPE.md` from the canon, naming its waiting places and opening any earlier ones that waited for this Quire's words, using the tier rules and the owner's Slice Gate decisions, and tell the owner what is Light and what is Reserve
- [ ] **P18-T02** The Erasure: terrain and landmark pass, materials, colour grade by time of day, and this Quire's weather and particles
- [ ] **P18-T03** The Erasure: build its settlements, small sites, and points of interest, including every Standing Page as an event with its guardian or puzzle, its learning spectacle, and the full signature effect of its word
- [ ] **P18-T04** The Erasure: build its Hero dungeons and the kit-built dungeons on the build list, each with its marked structures
- [ ] **P18-T05** The Erasure: add its named NPCs at their Full or Light tier, with schedules and dialogue
- [ ] **P18-T06** The Erasure: add its native species, variants, and bosses, each with its word reactions and annotations, and ragdolls for any rig family not yet covered
- [ ] **P18-T07** The Erasure: add its loot, ingredients, and the Canonical items found here
- [ ] **P18-T08** The Erasure: write Q15, Whoever Remains, Q16, The Unwritten Road, and Q17, The Last Scriptorium
- [ ] **P18-T09** The Erasure: write the questline quests, Author quests, side quests, small tasks, and secrets on the build list
- [ ] **P18-T10** The Erasure: write its books and surface text
- [ ] **P18-T11** Build Deleatur as a Full boss on the road through the Erasure, and everything it struck out writing itself back when it falls
- [ ] **P18-T11a** Build the Erasure's unravelling in full: the five stages of thinning text, thinning sound and HUD, and footprints that print what once stood there
- [ ] **P18-T12** Build the baked mask and warnings for Erasure Stage 3, and move or conclude everything it touches, and write this stage's Remembered list with about eight authored outcomes
- [ ] **P18-T13** The Erasure: lock and generate this update's voice lines by priority, after owner approval and within the credit budget, and integrate them
- [ ] **P18-T14** The Erasure: run all region tests, the scope check, and the old-save test, and fix what they find
- [ ] **P18-T15** The Erasure: add this update's steps to `docs/PLAYTEST_CHECKLIST.md`, translate this update's text into every supported language and pass the localization check, write the update notes, and push a private build

## Exit gate

The region passes its bots, validator, density map, variety report, scope check, and old-save test before the next begins

Save the evidence in `docs/evidence/P18/`.

## Owner gate: Update Gate

**Stop. Give me a private build with The Erasure, its update notes, and the effort it took against the slice report's estimate. Wait for approval before continuing.**

What the owner checks:

- A save from the previous release loads and carries on
- The new Quire holds 30 fps on your M1, with the capture saved as evidence
- The new main-story quest can be finished, alone and with a partner
- The Quire has its own light, colour, speech, and grudges, and does not feel like the Vale repainted
- At least one new word changed how you play
- The effort matched the estimate, or the tiers for later Quires have been adjusted

**It passes when** the owner approves the update for release, or asks to hold it. Either answer lets the next phase start.

If it does not pass, fix what the owner reports inside this phase and ask again. Do not start the next phase.

Set `docs/STATE.md` to **Waiting on owner**, and tag `phase-18-done` only after approval.

## Next

Move to `phases/P19-endings.md`.
