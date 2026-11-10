# Phase 02: Hook prototype

**Goal:** Prove that reading and rewriting the world is fun in one small room, and that fighting with blade and word for loot is fun in the yard beside it, before a world is built around either.

**Suggested model:** Fable 5.2, medium effort. This phase decides whether the game is fun, and its systems are kept for good.

**Hand to Haiku 5.5:** Filling word sheets from the bible's tables, the scripted run steps, the demo's title card, controls screen, and settings, and sorting demo feedback. Rules are in `CLAUDE.md` under Models and helpers.

**Release:** Release 1

**Requires:** Phase 01 finished and tagged `phase-01-done`.

## Read for this phase

Read only these files in `docs/brief/`, and only the named part where one is given.

- `0a-pillars-and-releases.md`: including Plain first, strange underneath
- `5a-core-gameplay.md`: Combat, Blade and word together, Emendation, and Compounds
- `5c-enemies-and-bosses.md`: Annotated enemies only
- `5d-loot.md`: tiers, what makes an item, traits, and rarity rules
- `5h-signature-mechanics.md`: Close reading and Typos
- `2a-signature-visuals.md`: Typographic body language
- `5e-ui-and-accessibility.md`: reading mode only
- `5f-coop.md`: first paragraph, and combination Emendations
- `5g-input.md`
- `2-art-direction.md`: gold leaf only
- `8-proof-tooling.md`: Hook room, Emendation matrix, Combat simulation, and Loot simulation

## Canon for this phase

These files are under `docs/`.

- bible/08-words-items-craft.md: Emendation words, refusal lines, Typos, Compounds, Rarity, and Materials
- bible/07-bestiary-and-bosses.md: the Inkling row and Annotated creatures
- bible/03-voice-and-dialects.md: for the wall text and refusal lines

## Scope

A walker, the Read action, Inscriptions, Emendation with five words, each on its own sheet, a weapon in one hand and a word in the other, one systemic chain reaction, one Inkling that can be beaten by both together, by word, or by steel, and one room with a puzzle, playable by one or two people. Then the arena: a second yard beyond the gate where the same systems are tested as a fight, with words on the weapon, three annotated Inklings, three compounds, and weapons that drop in two tiers. The arena is grey-box, with parametric weapons and no new art. Everything built here is kept: the room and the arena stay in the test suite, the word sheets are the real ones, and Phase 06 extends this code and does not replace it.

## Tasks

Do these in order, one at a time, following the task loop in `CLAUDE.md`.

- [ ] **P02-T01** Add a simple first-person walker to the look-prototype scene, with collision against the scene's own geometry, for a list of players and views
- [ ] **P02-T02** Build the hook room as data: a walled yard with a locked gate, a dark lamp, a dry thatch lean-to, a broken footbridge, and a wall of authored text that holds a clue
- [ ] **P02-T03** Add the Read action and a first reading mode: focus any surface text, show it in large clean type, and copy the clue to a plain journal list
- [ ] **P02-T03a** Add close reading: hold Read to show the world as raw text with Inscriptions and hidden lines, with slowed time for one player and a shared target mark for two
- [ ] **P02-T04** Build Inscriptions: one to three state words per target, with class, resistance tier, and Sealed words in gold leaf
- [ ] **P02-T05** Build the word sheet format, its bake, and the matrix validator generated from it, with sheets for OPEN, LIT, WHOLE, CALM, and BURNING, a refusal line for every pair without an effect, and `docs/WORDS.md` written by the bake
- [ ] **P02-T06** Add Ink and casting: equip a word to either hand, tap to write, hold to charge, and the typing and letter-wheel bonuses, with a simple favourites wheel. With a word equipped, light up everything it can change and show the result in plain words before the cast
- [ ] **P02-T07** Add systemic reactions and fading: a `BURNING` gate ignites `DRY` thatch, a `LIT` lamp reveals a hidden line, and changes fade on a timer
- [ ] **P02-T07a** Add typos for the five words and the Errata list: a full charge that is hit smudges into its typo, and typing the typo casts it on purpose
- [ ] **P02-T08** Add one Inkling on the amorphous rig, with a `HOSTILE` Inscription, one telegraphed attack, and its two authored word reactions
- [ ] **P02-T08a** Add typographic body language to the Inkling: loud, quiet, afraid, near death, and the gold flash when rewritten
- [ ] **P02-T09** Add a one-handed weapon for the other hand with a tap attack and one held heavy attack, add the sidestep, and return a little Ink for every hit, so the Inkling can be beaten with steel alone and is best beaten with both hands
- [ ] **P02-T10** Add minimal one-shot audio and give writing a word its synthesized sound and its flash of letters, with a motif for each of the five words and the wrong note for a typo
- [ ] **P02-T11** Make the room solvable in at least three ways, with no order of actions that leaves it unsolvable, and add a combination Emendation for two players
- [ ] **P02-T12** Build the Hook room scripted run for one and two views, and pass it with the Emendation matrix
- [ ] **P02-T12a** Build the arena as data: a second yard beyond the gate with a fire, a lamp, cover, and three waves of Inklings, the last led by an annotated one
- [ ] **P02-T12b** Add blade and word together for the five words: each word's combat line, words written on the held weapon by holding both hands, and tap casts that fit between swings
- [ ] **P02-T12c** Add annotated enemies with three annotations for the Inkling, LOCKED, DARK, and COLD: their names and letters, replacing the word with a charged one of its class, and the guaranteed drop
- [ ] **P02-T12d** Add compounds with Rapt, Grateful, and Blazing, the stronger form for two players, and the first entry on a plain Compounds list
- [ ] **P02-T12e** Add a first item generator: Sword, Axe, and Mace on the one swing, in Iron and Marcher Steel, Plain and Fair with four bonuses, their traits, parametric looks that differ, the tier mark, drop sound, pickup effect, and the comparison with what is held
- [ ] **P02-T12f** Build the arena scripted run for one and two views, and a first combat simulation with blade-and-word, steel-only, and word-only builds
- [ ] **P02-T13** Assemble the playable build and write the Hook Gate steps in `docs/PLAYTEST_CHECKLIST.md`

- [ ] **P02-T13a** After the Hook Gate passes, and only if the owner asks for the demo: add the title card, controls screen, basic settings, end card with the owner's links, and return to start, run the license audit, write the demo page text as the brief says the game is presented, and push a private itch build for the owner to publish
- [ ] **P02-T13b** Create `docs/DEMO_FEEDBACK.md` with its three lists. Whenever the owner adds feedback, sort it, fix what belongs in the hook room, and record the rest against a later phase

## Exit gate

The Hook room and arena runs pass with one and two views, the Emendation matrix has a result or refusal for every pair, and all three builds win the arena

Save the evidence in `docs/evidence/P02/`.

## Owner gate: Hook Gate

**Stop. Give me a build of the hook room and the arena, and twenty minutes of steps to try, alone and with a second controller. Wait for approval before continuing.**

What the owner checks:

- You knew what each word would do before you cast it, and nobody had to explain a rule
- You stopped to read the wall without being told to
- Close reading made you want to look at everything, and you used it to aim
- A word went wrong at least once, and it made you laugh or gave you an idea
- You could tell what the Inkling was about to do from its letters
- Changing a word felt like the world obeyed: it was instant, it had a sound, and something visibly changed
- You found a second way through the room and wanted to look for a third
- Equipping a word to a hand and charging it needed no explanation, and typing it for the bonus felt worth doing
- Swinging with one hand and writing with the other felt like one way of fighting, and hitting to earn Ink made you do both
- You also won the fight with a word alone and with steel alone
- A second player joined and you wrote a combination together
- In the arena, the sidestep was enough defence with a word in the other hand
- You wrote a word on your weapon without being reminded
- An annotated Inkling made you stop and read it, and you rewrote its word
- A compound fired by accident and you tried to do it again
- A weapon dropped and you stopped to compare it with the one in your hand
- You played the arena a second time without being asked

**It passes when** the owner wants to do it again in a bigger place. If reading or rewriting feels like a chore, or the arena is not worth a second go, the rest of the plan cannot fix it. The arena is grey-box, so judge how it plays and not how it looks.

If it does not pass, fix what the owner reports inside this phase and ask again. Do not start the next phase.

**The demo.** When the gate passes, ask the owner one question: whether to offer the hook room and arena as a free public demo, as set out in `1b-storefront-and-release.md`. If yes, do P02-T13a and P02-T13b alongside Phase 03. If no, tick both as not wanted and record the answer in `docs/PLAN_CHANGES.md`. Publish nothing without the owner saying so.

Set `docs/STATE.md` to **Waiting on owner**, and tag `phase-02-done` only after approval. The two demo tasks do not hold the tag back.

## Next

Move to `phases/P03-terrain-and-sky.md`.
