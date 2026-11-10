# Section 5: RPG systems, quest safety, Redactors

**RPG systems.**

- Character creation with three peoples of the realm, each with their own abilities, and a simple appearance editor
- Inventory with visible equipment, encumbrance, and levelled enemy tables. Keep these plain: they support the pillars and are not one. Loot is the exception and is built in full, as set out in `5d-loot.md`
- Three crafts, as in Skyrim. **Smithing** forges gear from materials and improves it at a grindstone or workbench. **Inscribing** is enchanting: unmake an item that carries a clause at a writing desk to learn it, then write it onto other gear using bottled Ink. Crafted gear reaches the Engrossed tier at most, as set out in `5d-loot.md`. **Distilling** is alchemy: combine ingredients into potions and poisons, and learn each ingredient's properties by tasting and mixing
- Dialogue trees with skill checks, merchants, trainers, and persuasion
- A single currency, **Marks**, with merchants whose stock and limited funds keep the economy tight
- Factions, reputation, crime, witnesses, bounties, and guards. When caught, a player can pay, fight, or surrender stolen goods. There is no jail system
- Stealth with light and sound, backstabs, lockpicking, and a text light meter. Lanterns and torches can be put out with water arrows or by writing `DARK`, and guards notice and relight them
- Walkable rooftop routes through the capital and Quire seats, and at least three heist quests in the Margin Folk line with a daytime scouting phase
- Eight recruitable followers, with basic commands and one following each player at a time
- One animal companion for the world: the Blot Hound runt met on Tithe Hill, which can be befriended after the prologue and named by the player. It follows, fights a little, grows over the game, cannot die, and barks when hidden text or a secret is near. It can be sent home or told to wait. In co-op it follows whoever called it last
- One buyable home in the capital, with storage
- NPC daily routines, sleep, work, and combat reactions
- Fast travel, wait, sleep, and a journal with tracked quests. The tracked quest's next step is marked on the compass and the map by default, as in Skyrim, and an option hides all markers for players who want to navigate by reading
- A dynamic quest director that spawns repeatable work and connects rumours to real, unvisited places
- Dynamic weather and a full day/night cycle
- Autosave, quicksave, and multiple save slots with versioning

Quests are defined in a small internal data language with stages, conditions, aliases, and scripted scenes, so hundreds of them can be written and validated without changing engine code. This format is for development only and is not exposed to players.

**Quest safety.** The game must never become unwinnable. NPCs required by the main story are protected from death. If any other quest NPC dies, the Ledger passes their entry to a named heir or an alternate path. Every clue the player reads is copied into their journal, so a quest never depends on finding the same wall again.

**Redactors.** The face of the game: they are on the key art, they are the first danger a new player meets, and they are what a player describes to a friend. Flying boss creatures attack settlements as world events. They are drawn in solid black strokes that cross out whatever they touch. Survivable, terrifying, and the game's highest spectacle moment. Killing one grants a powerful new word. **Seen early.** A Redactor is seen from the first vista, within five minutes, and strikes at the player inside the first hour: Obelus runs its line down the Ambry road in Q2, as the script sets out, and the player can only get out of its way. Nobody can fight one before Q7, and the game never pretends otherwise.

**Taking it back.** What a Redactor crosses out is struck through on the map and stands in the world as black ruled lines. When that Redactor dies, everything it struck writes itself back in front of the player, wall by wall and roof by roof, using the same reassembly as `WHOLE`, and the people there know who did it. This applies to Obelus at Scrivenhold, Caret on the Saltmargin coast, and Deleatur on the northern road. It never applies to what the Erasure itself has taken: Little Penance does not return.
