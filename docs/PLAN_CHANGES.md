# Plan changes

Newest first. Record every change to scope, design, order, or tools here. The full-game plan keeps its own history in `full-game/docs/PLAN_CHANGES.md`.

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
