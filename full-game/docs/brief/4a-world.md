# Section 4: world, geography, persistence

## 4. The world

**Premise.** The land of **Vellmark** was written into being, and every person, stone, and river has an entry in the Great Ledger. Now the **Erasure**, a silent white blankness, is advancing from the north, scraping the world back to bare paper. Winged black horrors called **Redactors** strike whole villages out of existence. You are one of the **Unentered**: a person with no entry in the Ledger. Because the world has no record of you, you alone can change what it says. In co-op, two Unentered travel together.

**Scale.** This is a focused open-world RPG in the spirit of Skyrim and Oblivion, with a map of comparable size and far less, but better, content: a realm you can spend dozens of hours in. The world must feel dense, inhabited, and worth wandering through. Scale must never be achieved by padding out empty distance.

**Built in releases.** The realm is built one Quire at a time and released in stages, starting with Scrivenhold Vale. The whole heightfield and every Quire's distant shape exist from Phase 03, so each vista is true from the first release. See `0a-pillars-and-releases.md`.

**Scope principle.** Spend effort where this game is unique and where code and writing do the work: the renderer, Emendation, readable text, systems, generators, quests, and books. Limit whatever depends on large amounts of hand-tuned animation, bespoke models, or character acting.

**Geography.** One continuous 6 km x 6 km realm (about 36 km² of playable land and coast) divided into seven **Quires** (a quire is a gathering of pages in a bound book), each with its own landscape, politics, and history:

1. **The Whitemarch:** snowbound northern highlands, where the Erasure has begun
2. **Scrivenhold Vale:** the central river heartland and seat of the capital
3. **The Foxing Wood:** an ancient autumn forest of colossal trees, in rust-brown ink
4. **Saltmargin:** a rugged coast of fishing towns and shipwrecks
5. **The Blotmere:** teal, fog-bound marshes
6. **The Cinder Folio:** a volcanic ember-land with red ink on charcoal paper
7. **The Erasure:** nearly pure white paper with characters thinning out, growing as the story advances. It should be eerie and beautiful.

**One further world: the Verso.** The reverse side of the page, about 2 km², entered through Torn Pages at key points in the story. Ink and paper invert to white on black, all text reads backward, and familiar places return in twisted forms. Build it mostly by transforming existing terrain, kits, and text. Mirrored letter shapes are allowed here as an exception to ASCII purity, and reading mode always shows them unmirrored.

**How the Erasure advances.** It grows in three authored stages tied to story beats, using baked masks. It never destroys unfinished required content: each affected location is either already concluded, moved elsewhere, or still reachable through the Verso, and the game warns the player before each stage. Redactor attacks are authored events. Damaged settlements can recover, and no named NPC is permanently removed outside the authored story.

**Generation.** A deterministic offline tool produces terrain with real hydraulic and thermal erosion, rivers, roads, and biome maps. It is then hand-shaped by data files for landmarks. Runtime uses clipmap terrain and async chunk streaming with no loading screens outdoors. Every part of the world is visible from a high enough peak. Interiors are entered by a short dive through a letter of the door's Inscription, as set out in `2a-signature-visuals.md`.

**A persistent world.** Every placed object has an identity. Things the player moves, drops, steals, or breaks stay that way, with saves storing only changes from the baked world. Containers restock and dungeons repopulate on a slow timer, except for hand-placed unique items and cleared story locations. NPCs far from every player continue their schedules through a low-cost simulation, so the world keeps living at this scale.
