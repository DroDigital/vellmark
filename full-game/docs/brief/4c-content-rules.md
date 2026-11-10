# Section 4: content quality, authoring, scope rules

**Quality rules for content at this scale.**

- Use three authoring tiers. These describe how content is made, and they are separate from the Full, Light, and Reserve build tiers in `0a-pillars-and-releases.md`. **Hero** content is entirely hand-written and hand-placed: the main quest, questlines, capitals, and major dungeons. **Authored** content has its layout built from kits, but its story, inhabitants, and reward are specific. **Generated** content is limited to repeatable bounties, wandering encounters, and background villagers, and it must always be labelled as such in the data.
- Every dungeon needs a reason to exist, a recognisable identity, and something the player will remember. No two adjacent dungeons may share a kit and a premise.
- A traveller should come across something worth stopping for about every 60 to 90 seconds at normal jogging speed (about 5 m/s) in any part of the map.
- Follow the road network, so towns, inns, and signposts make the realm feel like a real place.
- All dialogue is displayed as text on the same glyph system. Key scenes and selected lines are voiced from the budgeted list in `docs/voice/`, and all other lines use a synthesized murmur shaped to each speaker.

**Waiting places.** A waiting place is somewhere the player can see long before they can enter it, closed by a state that only a word they do not yet own can change: a lake that must be written `COLD` to bear weight, a fallen bridge waiting for `WHOLE`, a flooded cellar waiting for `DRY`, a boulder that will not move until it is `LIGHT`. They give the player reasons to come back, and make each new word open the map as well as the fight.

- There are about 40: six in each of the six Quires, and two each in the Verso and the Erasure. They are not new places. Each is a point of interest, a standalone secret, or a side room of a dungeon that is already on the build list, closed in this way.
- In every Quire, at least two wait for a word found in a later Quire, and the rest open with words found in that Quire. In Release 1 the Vale has four that the Vale's own words open and two that wait for an update.
- What is inside is always worth the return: a secret, a book, a high-tier item, a view, or a shortcut. It is never anything a quest needs, and never a Standing Page.
- Twelve of them hold a **loose word**: two in each of the six Quires, always in places that the Quire's own words open, so a player never needs a later update to reach one. The Verso and the Erasure hold none.
- A waiting place is visible from a road or a path, and what closes it can be read from where the player stands.
- Aiming at one with a word equipped shows it in a dim light, with its state word beside it and a plain line such as "Nothing you know answers this yet". Reading it records it in the journal and marks it on the map.
- When the player learns a word, the journal says how many marked places now answer to it, and their marks change on the map.
- In co-op, either player's word opens a waiting place for both.
- The content validator fails if a waiting place holds anything a quest requires, cannot be reached once its word is owned, or waits for a word the game never gives.

Write with a strong, consistent voice: melancholy, dry-witted, folkloric, and specific. On the main path, danger and loss lead and the wit follows, as the voice rules in `docs/bible/03-voice-and-dialects.md` set out. Avoid fantasy cliché and filler. Each Quire must have its own dialect, humour, and grudges. Keep all content in data files (`assets/data/*.ron`, with text as separate readable files).

**Authoring without an editor.** There is no visual level editor, so layouts must be authored as data and reviewed through captures.

- Town and building layouts use constrained placement rules for plots, streets, and props, with hand-written overrides for every landmark.
- Dungeons use a small layout format of rooms, connections, encounters, keys, and sightlines, assembled from kits and hand-specified for Hero dungeons.
- Terrain edits, roads, and landmark positions are data files with stable IDs.
- For every location, the bake outputs a top-down map, an orbit of captures, and a walkable route check. Review these before calling a location finished.

**If the schedule is at risk.** Recommend moving entries from Full to Light, or from Light to Reserve, in this order, and let me decide: small tasks, kit-built dungeons, interiors, variants, Light bosses, named NPCs without storylines, side quests, questlines. Never recommend cutting the renderer's signature effects, the signature ideas in `2a` and `5h`, Emendation, readable text, books, co-op, the main story, or the endings. Never add a system to make up for cut content.

**No mod support.** Do not build mod loading, plugin APIs, a scripting interface for players, Steam Workshop integration, or tooling intended for players. Data files are internal and shipped baked into pack files.

**Languages.** The world is written in English and stays so. Everything the player must understand is translated. The game ships in English, German, French, Spanish, and Brazilian Portuguese.

- **Never translated:** the fine print on the world's surfaces, the 35 words of Emendation, typos, the names of people, places, and items, and the voiced audio. These are the world's own writing.
- **Always translated:** menus, the HUD, prompts, tutorials, subtitles, dialogue and its choices, quests, the journal, item descriptions, books, the plain preview at the crosshair, refusal lines, captions, and the epilogue.
- **Reading mode** shows the English line large, as it is written in the world, with the translation beneath it. A player never needs English to follow anything.
- **Words of power** stay English on screen and in the hand, with their meaning beside them in the player's language. Typing a word for its bonus means typing the English word, which the interface shows.
- **Typos** are English wordplay and cannot be carried over. The Errata page explains each one in the player's language once it has fired.
- **Letters.** The interface, subtitles, and the translation in reading mode may use the accented Latin letters these languages need, from the same two typefaces. This is the only exception to ASCII purity outside the Verso, and it never applies to a surface in the world.
- **String tables.** No player-facing text is written in code. Every string has a stable ID in a table per language, layouts must hold text 35% longer than the English, and nothing is built by joining fragments of sentences.
- **How it is translated.** Claude Code translates each release's text itself, from a glossary per language in `docs/loc/` that fixes the interface terms and the tone of each dialect. The dry humour matters more than literal accuracy. This is recorded in `docs/AI_DISCLOSURE.md`.
- **The owner's check.** For each language and each release, the owner or a native reader they choose reads a sample of at least 200 lines, including a book and a main quest scene, before it ships. A language that has not been checked is offered as "beta" on the store page.
- **The localization check** runs on every bake. It fails on a missing or unchanged string, a broken placeholder, a letter outside the allowed set, or a line too long for its place. A pseudo-language with long, accented text is always built for testing.
- The store pages list interface and subtitles for all five languages and full audio for English only.
- The demo is English only.
