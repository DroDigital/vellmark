# Phase 07: RPG systems

**Goal:** The RPG systems work together through one complete quest, solo and in co-op.

**Suggested model:** Opus 5.5, high effort. Many systems, each closely specified, with tests to hold them.

**Hand to Haiku 5.5:** Perk and bonus data entry, menu screens that follow an existing one, accessibility option plumbing, and save round-trip tests. Rules are in `CLAUDE.md` under Models and helpers.

**Release:** Release 1

**Requires:** Phase 06 finished and tagged `phase-06-done`.

## Read for this phase

Read only these files in `docs/brief/`, and only the named part where one is given.

- `0a-pillars-and-releases.md`: The pillar test
- `5a-core-gameplay.md`: Emendation, growth, and scaling
- `5h-signature-mechanics.md`: An earned name
- `5i-physics-and-destruction.md`: how words change physics
- `2a-signature-visuals.md`: Travel through a letter
- `5b-rpg-systems.md`
- `5d-loot.md`
- `5e-ui-and-accessibility.md`
- `5f-coop.md`: Co-op rules
- `8-proof-tooling.md`: Bot playthrough, Content validator, Save test, Input test, and Scope check

## Canon for this phase

These files are under `docs/`.

- bible/02-peoples-and-daily-life.md
- bible/03-voice-and-dialects.md
- bible/08-words-items-craft.md
- script/01-title-intro-prologue.md: Format only

## Scope

Stats, the eleven skills and their perk trees, inventory, levelled loot, the six rarity tiers, bonuses, clauses, word clauses, Deeds, dialogue, quests, full Emendation, NPC schedules, save/load, UI, all co-op rules

## Tasks

Do these in order, one at a time, following the task loop in `CLAUDE.md`.

- [ ] **P07-T01** Implement stats, the eleven skills that grow through use, levels, perk points, and a perk tree of about eight perks per skill, with about sixteen for Emendation in two branches, and weapon perks that add bonuses and never moves
- [ ] **P07-T01a** Build the earned name: deed counters, the epithet format, and its appearance on clothing text, in greetings, on the journal cover, and on a grave marker
- [ ] **P07-T02** Add character creation and appearance
- [ ] **P07-T03** Implement inventory with the turning full-body character and item preview, equipment in two hands, encumbrance, materials, levelled lists for chests, enemies, and merchants, and rarity rolled by source
- [ ] **P07-T04** Implement merchants and Marks
- [ ] **P07-T05** Write the 30 clauses, the 12 word clauses, and all 20 bonuses, and implement the Endorsed, Engrossed, and Illuminated tiers with their names, ink treatments, ornate parts, drop effects, and weapon charge
- [ ] **P07-T05a** Build a weapon's history: the count printed on each item, the 8 Deeds and how each is chosen, the engraved line, and saving it with the item
- [ ] **P07-T06** Implement dialogue trees with skill checks, and the Light character format with no tree
- [ ] **P07-T07** Build the quest data language and its validator
- [ ] **P07-T08** Build the journal with its Errata, Compounds, and Commonplace pages, keeping a line from reading mode and exporting it as text or as a typeset page, and copy every clue read into it
- [ ] **P07-T09** Finish all Emendation classes and rules, including the compounds made from Standing Page words, loose words learned by reading, the Sound, Motion, and Size classes with real changes of size, Emendation inside dialogue, `HEAVY` and `LIGHT` changing real mass, and `COLD` water bearing weight
- [ ] **P07-T10** Implement NPC schedules, plus the low-cost distant simulation
- [ ] **P07-T11** Implement versioned save and load, with stable IDs, autosave, and quicksave
- [ ] **P07-T11a** Implement fast travel by Bookmarks with the dive through a letter on the map, plus wait and sleep
- [ ] **P07-T11b** Add waiting places: the dim light and plain line for a state no owned word answers, the journal record and map mark made by reading one, and the notice and changed marks when a new word answers some
- [ ] **P07-T12** Build the HUD, compass, map, and book-styled menus, using the plain-language terms in the brief, with quest markers on the compass and map by default and an option to hide them
- [ ] **P07-T12a** Build photo mode: the free camera and its controls, showing the player's own character with its poses, text view, split frame, saving to PNG and to Steam's library, and its co-op behaviour
- [ ] **P07-T13** Build every accessibility option required by the brief
- [ ] **P07-T14** Implement difficulty, level bands, death, and quest safety
- [ ] **P07-T15** Finish all co-op rules, including character profiles
- [ ] **P07-T16** Build the bot harness, driven by real game actions, and the Scope check
- [ ] **P07-T17** Write one complete quest that uses every system
- [ ] **P07-T18** Run the exit gate solo and in co-op, on controller input only

## Exit gate

One complete quest using all systems passes the bot solo and in co-op, using controller input only

Save the evidence in `docs/evidence/P07/`.

## Next

Move to `phases/P08-living-systems.md`.
