# Phase S03: Hands

**Goal:** A player who can walk, read, write seven words, and fight with a weapon in one hand and a word in the other, in the real world with grey-box enemies.

**Suggested model:** Fable 5.2, medium effort. Feel decides this phase.

**Hand to Haiku 5.5:** Filling word sheets from the bible's tables, input mapping tables, button prompt text, and scripted run steps. Rules are in `CLAUDE.md` under Models and helpers.

## Read first

- `full-game/docs/brief/5a-core-gameplay.md`: Combat, Blade and word together, Emendation, One sheet per word, Compounds
- `full-game/docs/brief/5h-signature-mechanics.md`: Reading and Typos only
- `full-game/docs/brief/3b-renderer-surfaces.md`: 3.13
- `full-game/docs/brief/5i-physics-and-destruction.md`
- `full-game/docs/brief/5g-input.md`
- `docs/SHOWCASE.md`: The words
- `full-game/docs/bible/08-words-items-craft.md`: the seven words, their typos, and the five compounds

## In this phase

The first-person controller, reading, Emendation with seven word sheets and the five-part effect, physics props and one repaired bridge, and blade-and-word combat against a grey-box Inkling.

## Tasks

Do the tasks of one lane in order, following the task loop in `CLAUDE.md`. Lanes run side by side as `docs/CLOUD.md` sets out. A task marked **after** waits for the task it names.

- [x] **S03-T01** [Sim] Build the first-person controller on `rapier3d`: walk, run, jump, climb, crouch, swim, and the sidestep
- [x] **S03-T02** [Shell] Add the two-hand input model, button prompts drawn in letters, and settings for sensitivity, invert, and remapping
- [x] **S03-T03** [Sim] Build reading: the Read action, close reading, reading mode with enlarged text, and hidden lines
- [x] **S03-T04** [Content] Build the word sheet format and its bake, with sheets for the seven words, and generate the word matrix from them
- [x] **S03-T05** [Sim] Implement Inscriptions on targets, tap and charge from either hand, both hands together, resistance, fading, Ink, and the plain preview at the crosshair
- [x] **S03-T06** [Render] Build the five-part magic effect on the shared particle system: charge, flight as a moving light, impact, the rewrite wave in the surface shader, and lasting states
- [x] **S03-T07** [Sim] Add typos for the seven words, the five compounds, and the systemic chain between neighbouring targets
- [x] **S03-T08** [Sim] Add physics props, breakable props that fracture along their text, and the bridge as a marked structure that `WHOLE` pulls back together. Loose letters are already built (see `docs/PLAN_CHANGES.md`, 2026-11-11): fracture pieces should crumble into them, and `WHOLE` should use each letter's home
- [x] **S03-T09** [Sim] Implement melee: the tap chain, one heavy attack, blocking, the shield bash, Stamina, Ink returned by hits, and hit stop for each view
- [x] **S03-T10** [Sim] Add words on the held weapon, each word's combat line, the bow, and sneak attacks
- [x] **S03-T11** [Render] Build the first-person hands and weapons: procedural arm IK, swing, block, draw, and writing gestures with a visible charge
- [x] **S03-T12** [Sim] Add a grey-box Inkling with telegraphed attacks, word reactions, and three annotations, in a yard beside the road
- [x] **S03-T13** [Shell] Give every word its motif, every hit its sound, and the wrong note for a typo
- [x] **S03-T14** [Sim] Build the scripted run: read, learn, open, mend, light, fight, and win with blade and word, with a word alone, and with steel alone
- [x] **S03-T15** [Shell] Run the exit gate, save the evidence, and write the Play Gate steps

## Exit gate

The scripted run passes. The word matrix has a result or a refusal for every word on every kind of target that exists. All three builds win the fight. The frame budget still holds on the owner's Mac during a fight with every effect on screen.

## Owner gate

**Stop. Give the owner the macOS build and fifteen minutes of steps.**

The owner checks:

- Moving and looking feel right at once, on a controller and with a mouse
- Reading a wall was worth doing
- The first word obeyed, and it felt like power
- The bridge coming back together made you do it twice
- Swinging with one hand and writing with the other felt like one way of fighting
- The sidestep was enough defence
- A typo or a compound happened by accident and made you laugh or try again
- Every word looks and sounds different, and expensive
- It stayed smooth through all of it

**It passes when** the owner wants to keep playing after the steps run out.

Set `docs/STATE.md` to **Waiting on owner**.
