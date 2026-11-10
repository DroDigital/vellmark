# Phase 06: Combat and first dungeon

**Goal:** Blade-and-word combat, with simple melee, a word in the other hand, and loot worth stopping for, proven in one dungeon, solo and in co-op.

**Suggested model:** Fable 5.2, medium effort. Feel, enemy behaviour, and first-person animation need the best judgement, and this phase ends in the Feel Gate.

**Hand to Haiku 5.5:** Annotation tables for each species from the bible, item generator name lists, sound assignment tables, and the feel-gate checklist. Rules are in `CLAUDE.md` under Models and helpers.

**Release:** Release 1

**Requires:** Phase 05 finished and tagged `phase-05-done`.

**Carried from Phase 02:** blade and word together, annotations, compounds, the item generator, and the combat simulation were proven in the arena with five words and one species. This phase extends that code to real characters, animation, four species, and a dungeon. It does not rebuild it, and it does not reopen what the Hook Gate settled unless the owner asks.

## Read for this phase

Read only these files in `docs/brief/`, and only the named part where one is given.

- `0a-pillars-and-releases.md`: Blade and word
- `5a-core-gameplay.md`
- `5h-signature-mechanics.md`: Close reading, Sorts, and Typos
- `2a-signature-visuals.md`: Typographic body language
- `5c-enemies-and-bosses.md`
- `5d-loot.md`
- `5f-coop.md`: Co-op rules
- `6-audio.md`: combat sound only
- `7-characters-and-animation.md`
- `5i-physics-and-destruction.md`
- `3b-renderer-surfaces.md`: 3.13 Magic effects
- `3b-renderer-surfaces.md`: 3.9 Damage
- `1.1b-asset-catalog.md`: Characters, monsters, weapons, Animation, and Sound effects
- `1.1c-libraries-and-import.md`: Libraries: navigation
- `8-proof-tooling.md`: Combat simulation and Emendation matrix

## Canon for this phase

These files are under `docs/`.

- bible/07-bestiary-and-bosses.md: the species and boss being built, and Annotated creatures
- bible/08-words-items-craft.md: words, compounds, rarity, weapon types, and materials
- script/01-title-intro-prologue.md: Q1, the ossuary only

## Scope

Characters, animation, first-person view-models, the four movesets, damage mask, shallow melee, combat audio, enemy AI, creature pipeline, the item generator for Plain and Fair items with traits and varied looks, blade and word together in real fights, the sidestep, annotated enemies, the first three compounds, the first 4 species, one Light boss, and Tithe Hill Ossuary

## Tasks

Do these in order, one at a time, following the task loop in `CLAUDE.md`.

- [ ] **P06-T01** Build the character assembler, with base bodies, outfits, and hair
- [ ] **P06-T02** Build animation playback: blending, layers, IK, look-at, and reduced rates for very distant characters, with inertial blending, the procedural layers, and spring-chain secondary motion
- [ ] **P06-T03** Retarget the approved libraries to the shared skeleton, and review them
- [ ] **P06-T04** Build the first-person view-model system with two independent hands, the One-handed, Shield, and Bow movesets, and the casting gestures for a word in either hand
- [ ] **P06-T05** Implement melee: the tap chain, one heavy attack for each moveset, blocking, the shield bash, the sidestep that keeps a held charge, and Stamina costs, with per-view hit stop, and Ink returned on every hit
- [ ] **P06-T06** Implement bows with draw strength, drop, and recoverable arrows, and sneak attacks with their multipliers
- [ ] **P06-T07** Add combat audio from approved sounds
- [ ] **P06-T08** Add damage masks and letters knocked off bodies
- [ ] **P06-T08a** Add ragdolls for the biped and quadruped rig families: death with inherited momentum, dissolving into characters at rest, partial-ragdoll hit reactions, and the active limit
- [ ] **P06-T09** Bake navigation meshes and add runtime agent movement
- [ ] **P06-T10** Build enemy AI: telegraphing, flanking, retreat, calling for help, cover, and reactions to words written on allies and on the room
- [ ] **P06-T11** Build the creature pipeline, and add the Blot Hound, the Shambling Entry, the Bone Clerk, and the Bandit, each with its word reactions from the bestiary
- [ ] **P06-T11b** Extend annotated enemies from the arena: the annotations whose answers exist so far, at least three on each of the four species, and their place in encounter groups
- [ ] **P06-T11a** Give each of the four species its loud and quiet glyph sets and the full typographic body language
- [ ] **P06-T12** Carry Phase 02 casting into full combat: charge levels, both hands together, losing a charge when hit, and lower resistance on staggered or unaware targets, with target highlighting and plain-word previews that stay readable in a fight
- [ ] **P06-T12a** Add the four sorts as cast modifiers for hands, Staffs, and arrows, and finish close reading for combat aiming
- [ ] **P06-T12b** Add the full typo table from the bible, and extend the Emendation matrix to every typo and sort
- [ ] **P06-T12c** Build the magic effect system: GPU glyph particles, the charge, the flight as a moving light, impact with haze, the rewrite wave, and lasting states
- [ ] **P06-T12d** Give OPEN, LIT, WHOLE, CALM, and BURNING their full signature effects, their typos' effects, and the delivery of each sort
- [ ] **P06-T12e** Extend blade and word together from the arena to the four species and the full view-models, with the writing-on-the-weapon gesture, and the validator rule that every word sheet has a combat line
- [ ] **P06-T12f** Extend compounds from the arena: firing from every source including arrows, Staffs, and annotations, their full effects on the four species, the crosshair preview for known compounds, and the matrix rule
- [ ] **P06-T12g** Add spectacle: the escalated effect for five or more targets and for chains of four or more links, within the budgets, and the list of marked moments
- [ ] **P06-T13** Add the Two-handed moveset for the Greatsword and Staff, the Dagger on the One-handed moveset, two weapons swung one per hand with no special moves, the favourites wheel with saved pairings, and the arena's item generator extended to all eight weapon types
- [ ] **P06-T13a** Add the rest of the 16 traits, and replace the arena's parametric looks with looks assembled from the approved weapon parts
- [ ] **P06-T14** Build the Senior Clerk of the Nine as the first Light boss, in an arena from the Crypt kit
- [ ] **P06-T14a** Add marked structures: authored broken states, `BROKEN` to shatter, `WHOLE` to reassemble along the paths the pieces fell, saving as a word, and the destruction test
- [ ] **P06-T15** Build Tithe Hill Ossuary using the layout format, with at least three marked structures that open a route, spring a trap, and change the boss arena
- [ ] **P06-T16** Extend the arena's combat simulation to the dungeon, tune every fight to the blade-and-word build, and fix any fight the other two cannot win
- [ ] **P06-T17** Run the dungeon solo and in co-op with bots, and pass the automated tests
- [ ] **P06-T18** Assemble the playable build and feel-gate checklist

## Exit gate

Dungeon is playable start to finish, solo and in co-op, with all automated tests passing, and every fight tuned for blade and word and still winnable by a steel-only build and by a word-only build

Save the evidence in `docs/evidence/P06/`.

## Owner gate: Feel Gate

**Stop. Give me a playable build and a playtest checklist for combat, Emendation, and co-op. Wait for approval before continuing.**

What the owner checks:

- Every hit and every written word lands with weight, on screen and in sound
- You read an enemy before you fought it, at least once without thinking about it
- Attacking, heavy attacking, and blocking took no learning and never felt thin
- A weapon in one hand and a word in the other is how you played without being told to
- Hitting to earn Ink and writing to open an enemy became one rhythm
- You wrote a word on your own weapon, and did it again
- With a word in the off hand and no block, the sidestep was enough, and fights were tense and fair
- An annotated enemy made you stop and read it, and rewriting its word felt better than cutting it down
- You found a compound by accident and went looking for another
- You stopped to compare a dropped weapon with the one in your hand, and two weapons of the same kind looked different
- Each of the four species asked for a different word, and you could read each one's mood from its letters
- A sort changed how you used a word you already had
- Enemies fall believably, flinch where they are struck, and dissolve into letters in a way you want to watch
- Breaking a wall and writing it `WHOLE` again is the best thing in the build
- Every spell looks and sounds expensive, and you can tell the five words apart by their light alone, and by their sound alone
- The dungeon holds 30 fps on your M1 through its busiest fight, and the capture is saved as evidence
- The boss's weakness could be worked out from what was written in its arena
- In co-op, one player held the line with steel while the other cast, and you swapped without discussing it
- You finished one run of the dungeon without casting and one without swinging. Both worked, and neither was as good as using both hands

**It passes when** the owner says the combat feels right with blade and word together, wants to keep the loot they found, and reports nothing that feels wrong.

If it does not pass, fix what the owner reports inside this phase and ask again. Do not start the next phase.

Set `docs/STATE.md` to **Waiting on owner**, and tag `phase-06-done` only after approval.

## Next

Move to `phases/P07-rpg-systems.md`.
