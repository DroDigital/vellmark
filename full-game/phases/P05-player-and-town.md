# Phase 05: Player, co-op, and first town

**Goal:** Walk through the world, enter a real town, and have a second player join at any moment.

**Suggested model:** Opus 5.5, high effort. The controller, co-op split, and first town are intricate, and the brief specifies them closely.

**Hand to Haiku 5.5:** Input mapping tables for each controller, button prompt text, building layouts from the placement rules, and prop lists. Rules are in `CLAUDE.md` under Models and helpers.

**Release:** Release 1

**Requires:** Phase 04 finished and tagged `phase-04-done`.

## Read for this phase

Read only these files in `docs/brief/`, and only the named part where one is given.

- `5a-core-gameplay.md`: Player only
- `5e-ui-and-accessibility.md`: reading mode only
- `2a-signature-visuals.md`: Travel through a letter
- `5f-coop.md`
- `5i-physics-and-destruction.md`: Physics, Destruction (breakable props only), and Budgets
- `5g-input.md`
- `4c-content-rules.md`: Authoring without an editor
- `1.1b-asset-catalog.md`: Towns, castles, and props
- `1.1c-libraries-and-import.md`: Import pipeline and `rapier3d`

## Canon for this phase

These files are under `docs/`.

- bible/04-quires-and-places.md: Scrivenhold Vale only

## Scope

Player controller, drop-in second player, per-view streaming, collision, modular architecture, interiors, Ambry as one complete town, full controller support

## Tasks

Do these in order, one at a time, following the task loop in `CLAUDE.md`.

- [ ] **P05-T01** Integrate `rapier3d`, with streamed terrain and static colliders
- [ ] **P05-T01a** Add physics props: rigid bodies, joints for signs, chains, and lanterns, sleeping, carrying and throwing, the budgets, and saving where props come to rest
- [ ] **P05-T02** Replace the Phase 02 walker with the full first-person controller: jog, sprint, jump, swim, climb, and mantle
- [ ] **P05-T03** Finish the import pipeline for collision and far forms across kits
- [ ] **P05-T03a** Add the pre-fracture pass to the bake, and breakable props that swap to falling pieces and crumble into characters
- [ ] **P05-T04** Build the interior system and the dive through a letter of the door's Inscription, per player, with a fade when reduced motion is on
- [ ] **P05-T05** Build town placement rules and landmark overrides
- [ ] **P05-T06** Output top-down maps, orbit captures, and walkable-route checks
- [ ] **P05-T07** Build Ambry at final quality, with authored text on its walls, signs, and graves. Fill it with loose and breakable props within the budgets
- [ ] **P05-T08** Finish reading mode and the Read action for every surface type, on keyboard and controller
- [ ] **P05-T09** Implement drop-in, drop-out second player, including the animated split
- [ ] **P05-T10** Implement independent streaming, shadows, and level of detail per view
- [ ] **P05-T11** Finish controller support: hot-plug, per-player settings, prompts, and the letter wheel
- [ ] **P05-T12** Build the co-op test and input test, and keep the Hook room run passing, and build the physics soak
- [ ] **P05-T13** Update `docs/PLAYTEST_CHECKLIST.md` with real controller checks
- [ ] **P05-T14** Run the exit gate and save evidence

## Exit gate

Walk from wilderness into a building without a hitch, while a second player joins, leaves, and roams separately on a controller

Save the evidence in `docs/evidence/P05/`.

## Next

Move to `phases/P06-combat-slice.md`.
