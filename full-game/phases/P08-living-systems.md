# Phase 08: Society and sound

**Goal:** Crime, factions, crafting, stealth, a full soundscape, and the first Redactor attack.

**Suggested model:** Opus 5.5, high effort. Crime, stealth, crafts, and the audio engine are specified and testable.

**Hand to Haiku 5.5:** Footstep and ambient tables, sample library import, voice preset data from the cast sheet, and the motif data once the phrases are composed. Rules are in `CLAUDE.md` under Models and helpers.

**Release:** Release 1

**Requires:** Phase 07 finished and tagged `phase-07-done`.

## Read for this phase

Read only these files in `docs/brief/`, and only the named part where one is given.

- `5b-rpg-systems.md`
- `5c-enemies-and-bosses.md`: Bosses
- `6-audio.md`
- `1.1b-asset-catalog.md`: Sound effects and Music
- `1.1a-licenses-and-ai-gate.md`: License rules only

## Canon for this phase

These files are under `docs/`.

- voice/cast-and-budget.md
- bible/05-factions.md
- bible/07-bestiary-and-bosses.md: Redactors

## Scope

Crime, factions, the three crafts, stealth, Redactors, audio, score, and the voice pipeline

## Tasks

Do these in order, one at a time, following the task loop in `CLAUDE.md`.

- [ ] **P08-T01** Implement factions, reputation, witnesses, crime, bounties, and guards
- [ ] **P08-T02** Implement stealth: light, sound, text light meter, and backstabs
- [ ] **P08-T03** Implement dousing and relighting lights, and lockpicking under the Lockwork skill
- [ ] **P08-T04** Implement the three crafts: Smithing (forge and improve in five steps), Inscribing (learn by unmaking, write with bottled Ink, and never past the Engrossed tier), and Distilling (discover properties, brew potions and poisons)
- [ ] **P08-T05** Build the audio mixer: two listeners, spatialization, reverb, and ducking
- [ ] **P08-T06** Add ambient beds and surface footsteps, with procedural variation
- [ ] **P08-T07** Finish the writing sound signature and all 35 word motifs, with their charged, two-handed, typo, sort, compound, annotation, and weapon forms and their captions, and add the dialogue murmur
- [ ] **P08-T08** Import the sample libraries, and build the sequencer and instruments
- [ ] **P08-T09** Compose the main theme from the motifs of the Vale's four words, and build the adaptive score system
- [ ] **P08-T10** Build the Redactor: flight, attacks, crossing-out, and reward
- [ ] **P08-T11** Build a Redactor attack on a town, end to end, including the struck-out area on the map, the ruled black lines in the world, and everything writing itself back when the Redactor falls
- [ ] **P08-T12** Build the voice pipeline and preset data from `docs/voice/cast-and-budget.md`, including pitch, formant, pacing, effects, murmur, and scene, distance, and budget checks
- [ ] **P08-T13** With the owner, create the 10 voices, generate one test line per base voice, and tune each preset. Spend no more than the budget's calibration share
- [ ] **P08-T14** Run the exit gate and save evidence

## Exit gate

A Redactor attack on a town works end to end

Save the evidence in `docs/evidence/P08/`.

## Next

Move to `phases/P09-the-vale.md`.
