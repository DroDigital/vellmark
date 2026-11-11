# Plan changes

Newest first. Record every change to scope, design, order, or tools here. The full-game plan keeps its own history in `full-game/docs/PLAN_CHANGES.md`.

## 2026-11-11: the living lattice is built and made the default; every material has its words

Owner's direction, given while the living lattice was being built. Seeing into things up close is the game's key selling point.

- **Look 4, living,** is built and is the default in the build until the owner chooses at the Look Gate (F6 still steps through all five). Up close the surface's front print lets go letter by letter until about three in four are gone; behind it the lattice's characters take the colour of the material where they stand, light dies away inward and the deep takes on the material's ink, the gaps hold a haze of the thing's own colour so the model is never lost, the corners are glowing atoms, and the bonds are single characters every third cell. Every character turns over by crossfades on its own clock, pushed on by the eye's movement. No character is ever seen mirrored.
- **State words glitch.** While a thing is hot or cold, its words (`BURNING`, `MELTING`, and so on) live about a second, jump to a new place each time, stutter, slip a cell, and tear into noise, in the lattice and in the surface's own print. A hot enough thing glows at any distance, as hot iron does.
- **Names and elements are a hook: often, and in the grain's own style.** The surface's larger print types out what a thing is and, two times in three, what it is made of, laid out like bricks in a bond so they are always evenly spread, more of them the further back the eye is. They are written exactly as the grain is, neither brighter nor bolder: they are found by reading. Water will say `water`, `H2O`, and, as it changes, `STEAMING`, `BOILING`, `GAS`, `FREEZING`, `ICE`.
- **`H2O` only where water is what the thing is** (water, brine, snow, ice, mud), at the owner's word: never on dry wood, stone, or metal.
- **Every material of the full game has its words,** 48 of them, in `assets/materials/lexicon.ron`, written out as a table in `full-game/docs/brief/2-art-direction.md`; a test fails if the two disagree. The Erasure has none.
- **The capture tool** gains `heat` and `frame_seconds`; the console gains `heat <name> <-1 to 1>`.
- **Not yet known:** the cost of look 4 on the owner's Mac. It follows more rays than the other looks and samples the material at every character. If it is over the opaque pass's share, the governor and a cheaper preset must answer for it before the Look Gate.
- Recorded in `full-game/docs/PLAN_CHANGES.md` as well.

## 2026-11-11: the lattice comes alive, names what it is made of, and shows what is happening to it

Owner's direction, given while S02-T01 was under way. To be done straight after S02-T01, before S02-T02.

- **The letters and the model must merge better:** colour, light, and effects of the lattice to be taken from the surface it is inside, so that the matrix reads as the same thing as the model and not as a layer laid over it.
- **The whole lattice is animated.** Every character in it changes over time, and words and small structures form and dissolve within it, changing with the angle the eye looks in. This applies inside the lattice only. A surface's own fine print and its authored sentences keep their stable identity (rule 3.4 of `full-game/docs/brief/3a-renderer-core.md`), and every change in the lattice is a crossfade, never a cut, so that the shimmer check can still pass.
- **An easter egg in every material.** Now and then, briefly and rarely, the lattice of a material spells its own name: `stone` in a stone wall, `wood` in a door, `iron` in a strap. Rarer still, deeper in, it names what the material is made of: the elements and compounds, such as `SiO2` in stone and `C6H10O5` in timber. All of it in the 95 printable characters.
- **A thing's state shows as a word in its letters.** A door on fire reads `BURNING` up close; metal hot enough reads `MELTING`; water reads `BOILING`; and so on for the temperature, light, and integrity states the words can cause. This joins Section 2a's typographic body language: for things, a change of state is a change of the underlying text, which rule 3.4 allows. The rules carry each object's state, so the renderer only shows it. The words themselves, and their effects, still come in S05.
- Recorded in `full-game/docs/PLAN_CHANGES.md` as well.

## 2026-11-11: broken things scatter as their letters, brought forward from S03

Owner's direction: when something is destroyed, the tens of thousands of characters it is made of scatter across the floor with real physics.

- **Brought forward from S03-T08.** Brief 5i already says that debris "crumbles into loose characters within seconds". The owner asked for the whole thing to come apart into its letters, so the loose-letter part is built now. Fracture along the lines of text, physics props, and `WHOLE` stay in S03-T08.
- **What is built:** the rules (`crates/sim/src/debris.rs`) fill a broken thing's boxes with letters about 1.6 cm apart (seven cells in ten hold one), throw them from the blow, and step each one at 60 Hz with gravity, drag, spin, bounce, and friction against the ground and the scene's solid boxes, until it lies flat, face up. Each letter remembers where it stood, for `WHOLE` later. The yard's door comes apart into 38,304 letters, all at rest within fifteen seconds. The renderer hides the broken thing and draws every letter as a lit character of its material, cut out by coverage. Too small to read, a letter is a chip of its material: no character shows beyond the reveal distance, loose or not.
- **New numbers:** at most 60,000 loose letters at once; the oldest at rest are swept first. The brief's limit of 200 loose fracture pieces is unchanged.
- **Heaps and shadows (the owner's follow-up, the same day).** Letters land on the letters already at rest and build heaps that stand no steeper than about 35 degrees; a letter that would make one steeper slides down it, and off the edge of a step. Each letter adds 3 mm to the heap it lies in. A blow has a strength: a soft one (`break door 0.1`) crumbles the door into a ridge 14 cm high on its threshold. Loose letters cast shadows from the sun and from the shadowed lantern.
- **What it does not do yet:** two letters in flight do not strike each other; a heap, once laid, is never knocked down again; a thing's turned boxes are treated as the box that bounds them; and there is no sound. The cost on the owner's Mac is not measured.
- **To try it:** in the development build, F7 breaks the nearest breakable thing; the console's `break door` breaks the door by name. `captures/breaking.ron` records it.
- Recorded in `full-game/docs/PLAN_CHANGES.md` as well.

## 2026-11-11: the close-up look gains depth, and the Look Gate is deferred

Owner's direction, given in a session after Phases S00 and S01 were merged.

- **The Look Gate is deferred, not passed.** The owner asked for the next phase to begin before playing the yard on their Mac. The gate's checks stay open in `docs/PLAYTEST_CHECKLIST.md`, and S00-T16, S01-T12, and S01-T13 stay unticked.
- **Up close, a surface is no longer a flat page.** The owner found the fine print flat. What they asked for: near the eye the realistic texture dithers away and the eye looks *into* the thing, at a three-dimensional grid of the same small characters, as deep as the thing is thick, sliding with the camera. The grid is a lattice whose lines are never drawn, only implied by rows of characters along its edges on all three axes. The characters are to feel like the atoms the model and its texture are made of.
- **Authored sentences stay.** The owner liked reading them. In the new look they stand whole at the front, with the lattice hushed behind them.
- **Three variants are built for the owner to choose between,** switchable with F6 or `look 0` to `look 3`: *atoms* (the texture goes entirely, the lattice keeps the thing's own colours, and the characters tremble), *lattice* (the texture stays as a half-dithered film over a still lattice), and *source* (the lattice is every printable character in gold, streaming along its edges). Look 0 is the flat fine print of Phase S01 and remains the default until the owner picks.
- **What this changes in the brief.** Rule 3.5 of `full-game/docs/brief/3a-renderer-core.md`, that letters are invisible in aggregate, now holds only beyond the depth look's range: within about two metres a depth look darkens the surface on purpose. The first promise of `docs/TECH.md` is untouched: beyond the reveal distance the frame is still exactly the frame with the glyph layer off. The source look's motion is confined to the layers behind the surface; a surface's own sentences still never move.
- **Each surface now carries its object's thickness,** baked for every vertex, so the lattice reaches the far side of the thing and no further.
- Recorded in `full-game/docs/PLAN_CHANGES.md` as well, because it changes the full game's look.

## 2026-11-10: the Showcase comes first

Owner's decision.

- **A showcase before the game.** One village, one hill, and one dungeon are built to final quality first: Little Penance, Tithe Hill, and the ossuary. Content is stripped. Visual quality and combat are not.
- **In it:** the full renderer, time of day and three kinds of weather, combat in full, seven words with typos and five compounds, four creatures with annotations, the item generator with all six tiers, an inventory with the turning character, five named people, the opening quest, the supper, the Line, and photo mode.
- **Left out for now:** streaming and the rest of the world, split-screen, levels and skills, crafting, factions, the quest director, translation, and Steam features.
- **A higher performance target:** 60 frames a second at 1080p on the owner's M1, where the full game's minimum is 30.
- **Built in cloud sessions,** in four lanes that run side by side. The owner's Mac is used only to play and to measure.
- **Tools first.** Hot reload, captures, reference images, and the performance run are built before the renderer.
- **The full-game plan is kept whole** in `full-game/` and becomes the live plan when the Showcase Gate passes. Nothing built for the Showcase is thrown away.
- **Showcase arrangements,** which the full game undoes: places set closer together than in the canon, Tobit Rudd visiting the village, and `BURNING` and `COLD` learned from the forge and the mill race.
- **Estimate:** 116 tasks, about 450 to 800 hours of Claude Code work, about four to eight weeks with sessions side by side.
