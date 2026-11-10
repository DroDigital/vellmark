# Section 4: content targets, questlines, endings, cinematics

**Content target (shippable, all completable).** The canon is a catalogue. The **1.0** column is what the finished game commits to building, and **Release 1** is the Vale's share. Whatever the catalogue holds beyond the 1.0 column is Reserve. Tiers are defined in `0a-pillars-and-releases.md` and tracked entry by entry in `docs/SCOPE.md`.

| Content | In the canon | 1.0 | Release 1 |
|---|---|---|---|
| Walled capital (**Scrivenhold**), with districts, a castle, the College, and key interiors | 1 | 1 | 1 |
| Quire seats | 5 | 5 | 0 |
| Towns and villages | 8 | 8 | 2 |
| Farms, mills, inns, camps, and hamlets | 20 | 20 | 6 |
| Dungeons (crypts, mines, forts, caves, ruins, towers) | 40 | 28: all 12 Hero and 16 kit-built | 6: 2 Hero and 4 kit-built |
| Enterable buildings and other interiors, mostly built from kits | 200 | 110 | 35 |
| Points of interest (shrines, shipwrecks, camps, lone graves) | 120 | 100 | 17 |
| Named NPCs | 150 | 150: 90 Full, of which 30 have deep storylines, and 60 Light | 60: 30 Full and 30 Light |
| Main story | 20 quests in three acts, with three branching final routes | 20 | Q1 to Q4 |
| Major questlines | 5 lines of 7 quests | 35 | The opening quests set in the Vale |
| Patron quests (the **Authors**), each ending in a unique artefact | 6 | 6 | Any that begin and end in the Vale |
| Hand-written side quests | 50 | 36 | 10 |
| Secret questlines, found only by reading the world closely | 8 | 8 | Their Vale steps |
| Standalone secrets, puzzles, and hidden rooms | 40 | 24 | 6 |
| Small tasks and encounters | 100 | 60 | 15 |
| Repeatable generated bounties, errands, and rumours | Authored templates | Unlimited | Unlimited |
| In-world books, written out in full | 100 | 100 | 25 |
| Harvestable ingredients | 40 | 40 | 7 |
| Standing Pages (words of power) | 20 | 20 | 4 |

Books and readable text are never reduced to save effort. They are the cheapest content in the game and the reason it exists.

Enemy, boss, and loot targets are in Section 5.

**Major questlines:** the **Scriveners' College** (mages), the **Margin Folk** (thieves and outlaws), the **Ledger Guard** (warriors), the **Blank Choir** (a cult that worships the Erasure), and the **Errata** (monster hunters).

**Endings.** The game has at least three distinct endings, reached through the main story's final routes, and each changes the world's fate:

1. **The Restoration:** rebind the Great Ledger. The world is saved exactly as written, every fate is fixed again, and you are forgotten.
2. **The Blank:** complete the Erasure, ending all suffering and everything else, leaving one clean sheet.
3. **The Free Hand:** destroy the Ledger. The Erasure stops, magic begins to fade, and for the first time nobody knows what happens next.
4. **The Colophon (secret):** found only by completing a hidden chain spread across the entire world. You write your own name in the Ledger and become its next Author.

All routes share one final dungeon. Each ending has its own last stretch, final boss, and outro. In co-op, both players must agree on the final choice.

Warn the player and autosave before the point of no return. After the outro, return to that save with the ending recorded on a profile-wide **Ledger of Endings**, so players can finish remaining content and see every other ending without replaying the game.

**Cinematics.** All cutscenes run in real time through the glyph renderer, with no pre-rendered video. Build a data-driven timeline system for cameras, typeset narration, particles, and music cues. Cinematics are built around landscapes, moving text, and silhouettes, so they do not depend on character acting.

- **Intro cutscene (2 to 3 minutes):** follows the title sequence on a new game. It tells of the writing of the world and the coming of the Erasure, and ends with the player being struck from the record.
- **Outro cutscenes (2 to 3 minutes each):** one per ending, followed by an epilogue written in the Ledger's own hand and generated from the actual save, naming who you saved, betrayed, and killed, and what became of each Quire and faction.
- Boss introductions and Redactor attacks get short camera moves, shown only in the viewports of players who are present. Other story moments play out in normal gameplay and dialogue.
- Every cutscene can be skipped by holding a button, including on its first viewing. The intro and outros play full-screen, and in co-op both players must agree to skip them.
