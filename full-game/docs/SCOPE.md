# Scope ledger

This file records what Vellmark builds, in what form, and in which release. The canon in `docs/bible/` and `docs/script/` is the catalogue. This ledger is the commitment.

Tiers and release rules are defined in `docs/brief/0a-pillars-and-releases.md`. To change anything here, edit this file, add a line to `docs/PLAN_CHANGES.md`, and run `python3 tools/update_plan.py`.

Entries marked **proposed** were chosen when the plan was refocused on 2026-11-09 and have not been tested against real build cost. The owner confirms or changes them at the Slice Gate.

## Not building

These were in the original brief and have been removed. Do not reintroduce them without the owner's instruction.

| Removed | Replaced by | Why |
|---|---|---|
| 80 random affixes | 16 fixed traits, 20 bonuses, and 30 clauses | Few enough to read on an item at a glance. The six rarity tiers were restored on 2026-11-10 |
| Eight separate movesets | Four movesets across the eight weapon types | Melee is shallow by the owner's decision on 2026-11-10, and first-person animation is the costliest content in the game |
| Directional power attacks, power bashes, timed blocks, and dual-wield special moves | Tap, one heavy attack, block, and one shield bash | Owner's decision on 2026-11-10: weapons get their depth from loot |
| Fourteen skills, with One-Handed, Two-Handed, Block, Light Armour, and Heavy Armour kept apart | Eleven skills: Weapons and Armour replace those five | Melee is shallow, so three weapon trees had nothing to say. About 24 fewer perks to write and balance |
| Standing stones that grant lasting abilities | Standing Pages | The game already has stones that grant power, and they are its own |
| Multiple phases on all 30 bosses | 14 Full bosses and 16 Light bosses | Tyrants and dungeon bosses are remembered for who they are |

## Blade and word

Set on the owner's instruction on 2026-11-10, replacing the Skyrim-style combat and loot restored the day before. Rules are in `docs/brief/5a-core-gameplay.md` and `docs/brief/5d-loot.md`.

| Decision | Where it is built |
|---|---|
| A weapon in one hand and a word in the other is the default and the pairing fights are tuned for | Phase 02 in the hook room, finished in Phase 06 |
| Shallow melee: tap, one heavy attack, block, one shield bash, four movesets | Phase 06 |
| Hits give Ink, words open enemies to steel, and words can be written on the held weapon | Phase 02 in the arena for five words, Phase 06 for the rest |
| Six rarity tiers, 16 traits, 20 bonuses, and varied looks | Phase 02 in the arena for Plain and Fair on three weapon types. Phase 06 for every type and real looks. Phase 07 for the other tiers. Tuned in Phase 09 |
| An Emendation perk tree twice the size of any other | Phase 07 |
| A sidestep for every pairing | Phase 02 in the hook room, finished in Phase 06 |
| Annotated enemies: 15 annotations, at least three for each species | Phase 02 in the arena for the Inkling, Phase 06 for the first four species, then with each species as it is built |
| Compounds: 30 | Phase 02 in the arena for the system and three of them. Phase 07 for those made from Standing Page words. The rest arrive with their loose words, Quire by Quire |
| Loose words: 12, two in each Quire, learned by reading | Phase 07 for the system, Phase 09 for SHARP and HONEST, then each Quire |
| A weapon's history: 8 Deeds | Phase 07 |
| Word clauses: 12, on Illuminated items only | Phase 07 |

Unchanged: skills that grow through use, now eleven of them, three crafts (Smithing, Inscribing, Distilling), bows, sneak attacks, and words equipped to a hand and charged.

## A wider audience

Added on the owner's instruction on 2026-11-10. Nothing here changes what the game is: the world made of text and the magic that rewrites it stay as they were.

| Change | Where it is built |
|---|---|
| The game is presented by its power, with the letters as the surprise | Phase 02 for the demo page, Phase 10 for the store kit |
| A Redactor is seen within five minutes and strikes at the player within the first hour | Phase 09 |
| Four more languages: German, French, Spanish, and Brazilian Portuguese | Phase 00 for string tables, Phase 10 for Release 1's text, then each update |
| Spectacle: escalated effects and moments marked for Steam's recording | Phase 06, then Phase 10 |
| The player's character seen in the inventory and in photo mode | Phase 07 |
| Stakes first on the main path, and every cutscene skippable | Phase 09 onward, as text is written |

Not adopted: recording video inside the game, online co-op, a third-person camera in play, and translating the world's surfaces or the words of power.

## Beyond the fight

Added on the owner's instruction on 2026-11-10.

| Addition | Where it is built |
|---|---|
| Waiting places: about 40, made from entries already on the build lists | Phase 07 for the system, Phase 09 for the Vale's six, then each Quire |
| A motif for each of the 35 words | Phase 02 for five, Phase 08 for all |
| Photo mode with text view and split frame | Phase 07, used for the store kit in Phase 10 |
| The Commonplace book | Phase 07 |
| A free public demo of the hook room, if the owner approves | Phase 02, after the Hook Gate |

Nothing was moved to Reserve to pay for these. The owner decides that at the Slice Gate, with the real costs in hand.

## Kept as it was

Emendation and its first 23 words, to which 12 loose words were added on 2026-11-10, the Three Layers of Text, the 100 books, the main story and all four endings, the five questlines, the six Author quests, the eight secret chains, drop-in local co-op, stealth and heists, the Redactors, the quest director, followers and the home, the persistent world, the voice plan and its 263 approved lines, the technology, and all three platforms plus Steam Deck.

## Wider appeal

Added on the owner's instruction on 2026-11-10. The rule is "Plain first, strange underneath" in `docs/brief/0a-pillars-and-releases.md`.

| Change | Where it is built |
|---|---|
| Words shown as spells: targets light up and results are previewed | Phase 02, finished in Phase 06 |
| Plain language on the interface | Phase 07 |
| Quest markers by default, with an option to hide them | Phase 07 |
| Standing Pages as events | Phase 09 for the Vale's four, then each Quire |
| A first cast on something enormous, within three minutes | Phase 09 |
| Struck-out places return when their Redactor falls | Phase 08 for the system, then Phases 12, 15, and 18 |
| The animal companion | Phase 09 |

Not adopted, to keep the game its own: mounts, a third-person camera in play, and removing the optional depth (typing, typos, sorts, close reading, the Remembered).

## Physics, destruction, and effects

Added on the owner's instruction on 2026-11-10. Rules and budgets are in `docs/brief/5i-physics-and-destruction.md` and sections 3.12 and 3.13 of the renderer brief.

| Feature | In | Out |
|---|---|---|
| Physics | Loose props, joints, carrying and throwing, words that change mass | Vehicles, cloth simulation beyond spring chains, fluid simulation |
| Ragdolls | Death and partial hit reactions for every rig family but the amorphous one | Ragdolls for the player's own view |
| Destruction | Pre-fractured props, and marked structures with authored broken states that `WHOLE` can repair | Free-form destruction of terrain or whole buildings |
| Magic effects | One five-part effect per word, per typo, and per sort | Unique effect systems per word |
| Minimum machine | Apple M1 with 8 GB: 30 fps at 1080p on Low, 5 GB of memory | Any target that cannot be measured on the owner's machine |

## Signature ideas

Added on the owner's instruction on 2026-11-09. Rules are in `docs/brief/2a-signature-visuals.md` and `docs/brief/5h-signature-mechanics.md`. None of these may be moved to Reserve without the owner's say.

| Idea | First built | Finished |
|---|---|---|
| Close reading | Phase 02, in the hook room | Phase 06 |
| Typos | Phase 02, for five words | Phase 06, full table |
| Typographic body language | Phase 02, on the Inkling | Phase 06, every species as it is built |
| Sorts | Phase 06 | Phase 06. Query is given in the prologue. Place the other three in Phase 09 (Exclaim), Phase 11 (Comma), and Phase 14 (Ellipsis), each at a point of interest on that phase's build list |
| An earned name | Phase 07 | Phase 09, with the Vale's epithets. Each Quire adds a few |
| Travel through a letter | Phase 05, doors | Phase 07, fast travel. Phase 13, Torn Pages |
| Weather as typography | Phase 04, wind and rain | Phase 14, fog. Phase 16, snow |
| The Remembered | Phase 09, after Little Penance | Phases 12, 17, and 18, one list per Erasure stage |
| The Erasure unravels | Phase 04, its frayed edge from a distance | Phase 18 |

## Totals

| Content | In the canon | Built for 1.0 | Reserve |
|---|---|---|---|
| Dungeons | 40 | 28: 12 Hero and 16 kit-built | 12 kit-built |
| Interiors | 200 | 110 | 90 |
| Points of interest | 120 | 100 | 20 |
| Small sites | 20 | 20 | 0 |
| Named NPCs | 150 | 90 Full and 60 Light | 0 |
| Side quests | 50 | 36 | 14 |
| Standalone secrets | 40 | 24 | 16 |
| Small tasks | 100 | 60 | 40 |
| Species | 24 | 21 | 3 |
| Variants | 80 | 21 | 59 |
| Bosses | 30 | 14 Full and 16 Light | 0 |
| Canonical items | 40 | 40 | 0 |
| Books | 100 | 100 | 0 |

## Rules for choosing a build list

The first task of each Quire phase writes that Quire's build list below, using these rules in order.

1. Build anything that can never be Reserve: the critical path, the questlines, the Author quests, the secret chains, every Standing Page, and any scene with an approved voice line.
2. Build any place, character, or creature that a built quest, boss, or Canonical item needs.
3. Fill the Quire's remaining share with the entries that make the most use of Read and Rewrite, and that differ most from what is already built.
4. Give Full dialogue to the 30 deep characters and to anyone a built quest speaks through. Everyone else named in the canon is Light.
5. Put the rest in Reserve, list it here by name, and tell the owner in one message. Do not stop for an answer.
6. From the entries now on the list, name the Quire's waiting places and the word each waits for, following `docs/brief/4c-content-rules.md`. Choosing one never adds an entry.

## Release 1: Scrivenhold Vale

| Content | Built in Release 1 | Later or Reserve |
|---|---|---|
| Settlements | Scrivenhold with all five districts, Ambry, Little Penance | None |
| Small sites | Wick's Rest, Tollow Mill, The Second Helping, Oake Farm, Barley Reckoning, Ferry-at-Quill | None |
| Hero dungeons | Tithe Hill Ossuary, The Index Stacks | Debtors' Vault is sealed until Phase 17 |
| Kit-built dungeons | Cellars of the Second Helping, Old Tollway Fort, Barrow of the Nine Clerks, Chalk Mine at Sums | Quill Sluice Tunnels (Reserve, proposed) |
| Points of interest | All 17, including the Pages for OPEN, LIT, WHOLE, and CALM | None |
| Main story | Q1 to Q4 | Q7 in Phase 12, Q12 and Q13 in Phase 17 |
| Questlines | Each line's quests that are set in the Vale and need no other Quire | The rest, in the phase that builds their Quire |
| Side quests | 1 to 8, 10, and 11 | 9, The Key That Fits Nothing, in Phase 13 |
| Species | Blot Hound, Shambling Entry, Bone Clerk, Bandit, Rogue Guard, Inkling, Quire Moth, Silverfish Elder, Staple Wasp | None |
| Bosses | The Indexer (Full), Senior Clerk of the Nine (Light), The Toll (Light), and Sir Pym Vexley (Light) if Guard quest G4 is on the Release 1 list | Obelus is seen in Q3 and fought in Phase 12 |
| Named NPCs | The Vale's deep characters and anyone a Release 1 quest speaks through are Full, to a total of about 30. The rest of the Vale's named characters are Light | None |
| Books | 25, chosen so that every secret chain with a first clue in the Vale can be started | The rest, with their Quires |
| Waiting places | Six: four opened by the Vale's own words, and two that wait for words from later Quires | None |
| Closed roads | The three old roads out of the Vale, each held by the Ledger Guard with a written reason | Opened one by one as the updates arrive |

## Kit-built dungeons in Reserve (proposed)

None of these is used by a quest, boss, Canonical item, book, or voice line in the canon.

| Quire | Reserve |
|---|---|
| Scrivenhold Vale | Quill Sluice Tunnels |
| The Whitemarch | Cairn of the Long Count, Windtooth Tower |
| The Foxing Wood | Rootcellars of Lowbough, Owl-Reader's Tower |
| Saltmargin | Sea Cave of Returned Things, Salt Mine Under Gull's Errand, Sailors' Ossuary |
| The Blotmere | Bog Barrow, Peat Diggings |
| The Cinder Folio | Ashen Barrow, Ember Vent Caves |

## Species in Reserve (proposed)

| Species | Why it can wait |
|---|---|
| Margin Crab | No quest, boss, or book depends on it. Saltmargin keeps the Spine Spider and the Drowned Parishioner |
| Marcher Reaver | No quest depends on it. The Whitemarch keeps the Long Goat and the Tally Bear, and Bandits cover human enemies |
| Paper Golem | No quest depends on it. The Erasure keeps the Scraper, the Choir Zealot, and the Misprint |

Variants: build one per species, choosing the one that differs most from the base in behaviour. A variant that a built quest or book names is built as well, and recorded here. The other 59 stay in Reserve, because annotated enemies now give every species at least three further forms.

## Bosses

| Tier | Bosses |
|---|---|
| Full (14) | Obelus, Caret, Deleatur; The Indexer, Bailiff of the Verso, Precentor Aldhelm Sour, Ser Anselm Gall-Hand, The Second Edition, The Cantor, Lord Registrar Thale, The Corrector, The Clerk; The Sum of All Goats, Saint Slip's First Mistake |
| Light (16) | Sir Pym Vexley, Tor Hask, Madam Alder Rue, Nan Twice, Ollo Lull, Gudrun Anvil; Eelmother, The Goatwife, Senior Clerk of the Nine, The Toll, Lamp Keeper Slip, The Last Foreman, The Commander in the Tunnels, The Surveyor, Queen of Bonecloth, The Winter Abbot |

## Share for each later Quire (proposed)

The first task of each phase turns these numbers into names.

| Phase | Quire | Kit-built dungeons | Points of interest | Side quests | Secrets | Small tasks |
|---|---|---|---|---|---|---|
| P11 | The Foxing Wood | 3 of 5 | 13 of 17 | 4 of 6 | 3 | 8 |
| P12 | Saltmargin | 2 of 5 | 13 of 17 | 5 of 8 | 3 | 8 |
| P13 | The Verso | 1 of 1 | 8 of 8 | Vale quest 9 | 2 | 3 |
| P14 | The Blotmere | 2 of 4 | 13 of 17 | 5 of 7 | 3 | 8 |
| P15 | The Cinder Folio | 2 of 4 | 13 of 17 | 5 of 7 | 3 | 8 |
| P16 | The Whitemarch | 2 of 4 | 13 of 17 | 2 of 2 | 2 | 6 |
| P17 | Across Vellmark | None | None | 4 of 9 | None | None |
| P18 | The Erasure | None: both dungeons are Hero | 10 of 10 | None | 2 | 4 |

With the Vale's 4 kit-built dungeons, 17 points of interest, 11 side quests, 6 secrets, and 15 small tasks, these add up to the 1.0 totals above.

## Build lists for later Quires

Each Quire phase adds its list here.
