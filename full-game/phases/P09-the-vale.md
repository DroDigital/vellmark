# Phase 09: The complete Vale

**Goal:** Finish Scrivenhold Vale to shipping quality as Release 1's world, and use it to measure what the rest will cost.

**Suggested model:** Opus 5.5, high effort. Most of this phase is building to a set standard. Switch to Fable 5.2 at medium effort for P09-T14, P09-T15, and P09-T20: the quests and books set the writing standard for the whole game, and the slice report decides its scope.

**Hand to Haiku 5.5:** Light characters and their schedules, kit interiors, loot and ingredient placement, fixing lists of validator failures, and voice line integration. Rules are in `CLAUDE.md` under Models and helpers.

**Release:** Release 1

**Requires:** Phase 08 finished and tagged `phase-08-done`.

## Read for this phase

Read only these files in `docs/brief/`, and only the named part where one is given.

- `0a-pillars-and-releases.md`
- `5h-signature-mechanics.md`: The Remembered, and An earned name
- `4a-world.md`
- `4b-content-and-story.md`
- `4c-content-rules.md`
- `5b-rpg-systems.md`
- `5c-enemies-and-bosses.md`
- `5d-loot.md`
- `9-how-to-work.md`: Writing process and Scope
- `8-proof-tooling.md`: Validator, Density map, Variety report, Soak test, and Scope check

## Canon for this phase

These files are under `docs/`.

- SCOPE.md: Release 1
- voice/cast-and-budget.md: Rules and Credit budget
- WORLD_BIBLE.md
- bible/01-history-and-truth.md
- bible/03-voice-and-dialects.md
- bible/04-quires-and-places.md: Scrivenhold Vale
- bible/06-characters.md: Vale characters
- script/01-title-intro-prologue.md
- script/02-act-one.md: Q2 to Q4
- script/06-questlines.md: quests set in the Vale
- script/07-authors-and-side-quests.md: Vale quests
- script/08-secrets-and-small-tasks.md: Vale entries

## Scope

Followers, home, quest director, persistent world, content validator, cutscene system, the Release 1 list in `docs/SCOPE.md` at final quality (the capital, towns, dungeons, NPCs, native species, bosses, loot, quests, secrets, and books), the intro and prologue, and the three closed roads

## Tasks

Do these in order, one at a time, following the task loop in `CLAUDE.md`.

- [ ] **P09-T01** Confirm the Release 1 build list in `docs/SCOPE.md` against the canon and what has been built, and record any change in `docs/PLAN_CHANGES.md`
- [ ] **P09-T02** Add writing lint and the batch review process
- [ ] **P09-T03** Implement persistent world saves storing only changes
- [ ] **P09-T04** Implement followers and the buyable home
- [ ] **P09-T04a** Add the animal companion: befriending and naming the Tithe Hill runt, following, light combat, growth, barking near hidden text, and send-home and wait commands
- [ ] **P09-T05** Build the quest director
- [ ] **P09-T06** Finish the loot simulation, and tune levelled lists, rarity rates by source, traits, bonuses, clauses, word clauses, Deeds, the share of annotated enemies, crafting, and the Vale's Canonical items against it
- [ ] **P09-T07** Build the cutscene timeline system and the intro cutscene
- [ ] **P09-T08** Complete the content validator, density map, variety report, and soak test
- [ ] **P09-T09** Vale: build the capital, Scrivenhold
- [ ] **P09-T10** Vale: build Little Penance, the small sites, roads, and the 17 points of interest, including four Standing Pages, each Page as an event with its guardian, climb, or puzzle and its learning spectacle, and the Vale's six waiting places, chosen from this list, with SHARP and HONEST as loose words in two of them and the Glint compound
- [ ] **P09-T11** Vale: build the Index Stacks and the four kit-built dungeons on the Release 1 list, each with its marked structures
- [ ] **P09-T12** Vale: add its named NPCs at their Full or Light tier, with schedules and dialogue
- [ ] **P09-T13** Vale: add its remaining native species with their annotations, variants, bosses, and loot
- [ ] **P09-T14** Vale: write Q2 to Q4, including the Line on the Ambry road as a set piece with its struck-out stretch saved and marked on the map, the opening questline quests, side quests, small tasks, and secrets on the Release 1 list
- [ ] **P09-T15** Vale: write its 25 books and its surface text, with a Perception line on every notable wall
- [ ] **P09-T16** Build the prologue and the first five minutes, with the bronze ossuary doors as the first cast, control of a word within three minutes, and Obelus crossing the far sky at the vista
- [ ] **P09-T16a** Build the Remembered: the journal page, choosing three, reading back, and its first list after Little Penance in Q3 with about eight authored outcomes, without changing Q3's script
- [ ] **P09-T16b** Write the Vale's epithets, place the Query sort in the prologue's last room and the Exclaim sort at a Vale point of interest, and write the books that hint at typos
- [ ] **P09-T17** Build the three closed roads, each held by the Ledger Guard with its own written reason, and a last scene that looks north
- [ ] **P09-T18** Lock and generate voice lines for Q1 to Q4 only, after owner approval, and integrate them
- [ ] **P09-T19** Run every test on the Vale and fix what they find, including the physics soak and the destruction test
- [ ] **P09-T20** Write the slice report: effort used per content type, and a proposed tier for every later entry in `docs/SCOPE.md`

## Exit gate

The Vale passes all bots, validator, density map, variety report, scope check, and soak test

Save the evidence in `docs/evidence/P09/`.

## Owner gate: Slice Gate

**Stop. Give me a build of the complete Vale, plus the slice report and its proposed tiers for the rest of the world. Wait for approval before continuing.**

What the owner checks:

- The first five minutes reach a high view, show the Erasure, give you a word, and let you throw open something enormous
- Someone who has not followed the project played the first hour with no help from you, and wanted to keep going
- You can finish Q1 to Q4 and every Vale quest, alone and with a partner
- Walking the Vale, something worth stopping for turns up about once a minute
- At least one wall, robe, or gravestone told you something you acted on
- Choosing what to remember after Little Penance was hard, and reading one back was worth it
- The world gave you a name you recognised yourself in
- A gust of wind wrote something across a field and you stopped to watch
- The closed roads feel like part of the story
- The slice report's cost per dungeon, quest, character, and boss looks believable against what you watched being built
- The proposed tiers in `docs/SCOPE.md` for the remaining Quires are ones you accept, or you have changed them

**It passes when** the owner would pay for the Vale on its own, and has confirmed or changed the tiers for everything after it.

If it does not pass, fix what the owner reports inside this phase and ask again. Do not start the next phase.

Set `docs/STATE.md` to **Waiting on owner**, and tag `phase-09-done` only after approval.

## Next

Move to `phases/P10-release-one.md`.
