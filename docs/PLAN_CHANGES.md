# Plan changes

Newest first. Record every change to scope, design, order, or tools here. The full-game plan keeps its own history in `full-game/docs/PLAN_CHANGES.md`.

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
