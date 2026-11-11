# Vellmark: the Showcase

## What this is

The Showcase is one village, one hill, and one dungeon from Vellmark, built to final quality. It exists to prove, to the owner and to strangers, that the game looks astonishing, runs smoothly, and is fun to play, before a world is built around it.

**The rule: strip content, never quality.** The Showcase has a fraction of the full game's places, people, words, and items. What it does have is finished: the renderer, the light, the combat, the magic, and the loot work as they will in the full game, and nothing in it is a placeholder.

It is also the first piece of the real game. Everything is built to the full game's specification in `full-game/docs/brief/`, and nothing here is thrown away. When the Showcase Gate passes, the build continues from this code with the full-game plan in `full-game/`, as set out at the end of this document.

**A player's first impression in one line:** an open-world RPG where you rewrite reality with a word.

## The thirty minutes

The Showcase is played in 30 to 45 minutes. This is the tour, and it is the acceptance test. Every beat shows at least one reason to buy the game.

1. **The title.** The camera dives through letters within letters and comes out above a valley at dawn.
2. **The grave.** The player wakes in an open grave on Tithe Hill. Wick, the gravedigger, is unimpressed. Dialogue, with choices.
3. **The climb.** A Blot Hound Runt decides it likes the player. A low wall is covered in small text, and the game teaches reading on a gravestone.
4. **The vista.** The whole valley, the village below, and a band of white along the north. Far off, something black crosses the sky once.
5. **The Standing Page.** The stone is writing down what happens as it happens. The player takes the word `OPEN` from it.
6. **The doors.** Two bronze doors, three times the player's height, shut for forty years. `LOCKED` becomes `OPEN`, and they are thrown back.
7. **The ossuary.** A short dungeon: Shambling Entries muttering their debts, a pressure puzzle, loot on the floor, and a Bone Clerk at the end. Combat with a weapon in one hand and a word in the other.
8. **The road down.** A pack of Blot Hounds, one of them annotated. Beyond the village, a fallen bridge with a Standing Page at its head.
9. **Little Penance at dusk.** Lanterns being filled, smoke, a mill wheel turning. Supper with the Oakes. Tamsin shows her stones.
10. **The village's words.** The Lamp That Was Promised, on the green, gives `LIT`, and the lanterns of the village can be lit one by one. The miller's whetstone, read up close, gives `SHARP`. The smith's forge gives `BURNING`, and the mill race gives `COLD`.
11. **Free play.** The Page at the Mended Bridge gives `WHOLE`, and the bridge comes back together under the player's feet. On the far bank, the Quiet Orchard gives `CALM`. A merchant, an inventory, a character to dress. Compounds found by accident. A cellar under the mill with an Inkling in it, and the best loot in the Showcase.
12. **Night.** Stars, firelight in windows, and the wind writing a sentence across the barley, once.
13. **The Line.** The birds stop. Obelus comes down the road and strikes a black rule through a hedge, a milestone, and the edge of the village. The player gets clear. It goes north.
14. **The end card.** What the full game will be, and where to wishlist it.

After the end card the player may keep playing. Nothing more is scripted.

## What each selling point is shown by

| Selling point | Shown by |
|---|---|
| A world secretly made of text | The title dive, every surface up close, close reading, photo mode's split frame |
| Magic that rewrites the world | The ossuary doors, the bridge, seven words on doors, fires, creatures, props, and the player's own weapon |
| Blade in one hand, word in the other | The ossuary, the hounds, the cellar |
| Reading is rewarded | Gravestones, Tamsin's sleeve, the whetstone, a hidden line that opens the mill cellar |
| Loot worth stopping for | Six rarity tiers on show, looks that differ, two artefacts, the turning character in the inventory |
| Redactors | The far sky at the vista, and the Line |
| A hand-written world with a voice | Wick, the Oakes, three books, forty gravestones |
| Local co-op | Not in the Showcase. The code stays ready for two players, and split-screen is the first thing added after the gate |

## In and out

| In the Showcase, at full quality | Left for the full game |
|---|---|
| The Glyph Cascade renderer, the realistic frame, close reading, and the title dive | The Erasure's unravelling, the Verso, snow that buries text |
| Time of day, clear weather, rain, fog, wind in grass, wind that reads | Seasons, festivals, the other weather |
| One area of about 500 m by 500 m, loaded whole | World streaming, the other 35 km², the capital |
| First-person movement, climbing, swimming in the mill pond | Fast travel, the map, mounts were never planned |
| Combat in full: tap, heavy attack, block, shield bash, sidestep, bows, sneak attacks | Nothing. Combat is not cut |
| Seven words, their typos, five compounds, words on the weapon | The other 28 words, sorts, typing a word, the other compounds |
| Four creatures, four annotations, typographic body language, ragdolls | The other 20 species, variants, bosses |
| Item generator, six tiers, traits, bonuses, clauses, two artefacts | The full tables, sets, Deeds, crafting |
| Inventory with the turning character, one merchant, pickups, comparison | Encumbrance tuning, housing, storage |
| Dialogue with choices, five named people with daily routines | Factions, crime, companions beyond the hound, the quest director |
| One quest, one supper, one set piece, three books | Everything else in the script |
| Physics props, breakable props, one marked structure repaired by `WHOLE` | Hero dungeon destruction |
| Motifs for seven words, generated score, ambience, footsteps, combat sound | The full score and sample library |
| Photo mode with text view and split frame | The Commonplace book |
| Save and load, settings, controller and mouse and keyboard, the three presets | Steam features, translation, achievements |
| A character with fixed abilities | Levels, the eleven skills, perks |

Voices are the owner's choice at Phase S06: either the approved lines for these scenes from `full-game/docs/voice/voice_lines.tsv`, generated as one batch, or the synthesized murmur for every line.

## The place

One continuous area, about 500 m by 500 m of playable ground, loaded whole. Beyond it a ring of far terrain, forest, and the white band of the Erasure is drawn as distant forms that cannot be reached. Low walls, the river, and steep ground close the edges, and nothing invisible ever stops the player.

This is a **showcase arrangement**. The canon's places are real, but they are set closer together here than they will be in the full realm, and a few are borrowed from elsewhere in the Vale. `full-game/docs/bible/04-quires-and-places.md` remains the truth for the full game.

| Place | What it is | What it shows |
|---|---|---|
| Tithe Hill | A graveyard hill with forty stones, each with its own inscription, and the Tithe Hill Stone at the top | Reading, the vista, the first Standing Page |
| Wick's Rest | The gravedigger's cottage, enterable | An interior, firelight, a readable home |
| Tithe Hill Ossuary | A crypt of four rooms and a boss chamber | Combat, loot, a pressure puzzle, breakable props |
| The road | Hedges, a milestone, a footbridge, barley on both sides | Grass and wind, the hounds, the Line |
| Little Penance | Twenty houses, a pump, a green, and a mill on the river. Five houses are enterable | A place worth standing in at dusk |
| The mill and its cellar | A turning wheel, a pond, a whetstone, a hidden cellar | Water, the loose word, a secret found by reading |
| The Lamp That Was Promised | A tall lamp on the green that has never been lit, with a Standing Page for its base | A learning spectacle at dusk, and a village lit by hand |
| The Mended Bridge | A marked structure, `BROKEN`, with a Standing Page at its head | `WHOLE` pulling it back together underfoot |
| The Quiet Orchard | On the far bank, reached only by the mended bridge. A Standing Page, a chest, and a second pack of hounds | A place that waits for a word, and `CALM` used on what guards it |

Every building has an outside and a reason to be there. Every enterable building has a hearth, a bed, a table, and at least two props nobody else has. Nothing is locked without a way in.

## The people

Five named people with daily routines, drawn from the canon. Each has a face that holds up in a close-up and clothing that carries their true biography in its fine print.

| Person | Where | What they want | What they show |
|---|---|---|---|
| Wick | Tithe Hill and his cottage | To be paid | The opening, dialogue choices, the game's voice |
| Mabel Oake | The Oake house and the green | To feed you | Supper, warmth |
| Gerrit Oake | The Oake house and the barley | To get to market | Routine: he works the field by day |
| Tamsin Oake, nine | Everywhere she should not be | To show you her stones | Reading a person's sleeve, with her permission |
| Tobit Rudd | A handcart on the green | To sell you something | The merchant, the forge, `BURNING` |

Their lines for the opening and the supper are written in full in `full-game/docs/script/01-title-intro-prologue.md` and `full-game/docs/script/02-act-one.md`, and are used as written. Tobit's lines and all incidental talk are new, written to `full-game/docs/bible/03-voice-and-dialects.md`. About ten unnamed villagers fill the green and the lanes with a few lines each.

## The creatures

| Creature | Body | Where | Behaviour | Answers to |
|---|---|---|---|---|
| Blot Hound | Four legs | The road, in a pack of four | Circles, bites, drops Ink when struck | `CALM` makes it follow the player for a while |
| The Shambling Entry | Two legs | The ossuary | Slow, heavy, mutters its debts | `WHOLE` settles its account, and it lies down |
| Bone Clerk | Two legs | The end of the ossuary | Summons paperwork that binds the player, then closes in | `BURNING` clears the paperwork. `SHARP` on a weapon cuts its armour |
| Inkling | Formless | The mill cellar | Fast, and splits when struck | `COLD` stops it splitting. `LIT` drives it back |

- A Blot Hound Runt follows the player from the hill and can be named. It does not fight.
- **Annotations** in the Showcase: `LOCKED`, `DARK`, `BURNING`, and `COLD`. One hound in the pack and one Shambling Entry are always annotated, and others are by chance.
- Every creature shows its state in its letters, as `full-game/docs/brief/2a-signature-visuals.md` sets out.
- Obelus is not a creature to fight here. It appears twice, far off at the vista and in the Line, and is built as a flying form with its strike effect and nothing else.

## The words

Seven words, each on its own sheet, with every effect a real one.

| Word | Class | Learned from | A use the tour shows |
|---|---|---|---|
| `OPEN` | Access | The Standing Page | The ossuary doors |
| `LIT` | Light | The Lamp That Was Promised | Lanterns at dusk, and driving back the Inkling |
| `WHOLE` | Integrity | The Mended Bridge | The bridge itself |
| `CALM` | Mind | The Quiet Orchard | The hounds |
| `BURNING` | Temperature | The smith's forge | The Bone Clerk's paperwork, a blade alight |
| `COLD` | Temperature | The mill race | Ice across the pond, the Inkling |
| `SHARP` | Integrity | The miller's whetstone, by reading it | A blade that cuts armour |

- **Typos:** `OVEN`, `LIP`, `HOLE`, `CLAM`, `TURNING`, `GOLD`, and `SHARK`, as the canon gives them.
- **Compounds:** Rapt, Grateful, Blazing, Glint, and Rime, as the canon gives them.
- **Words on the weapon:** `BURNING`, `COLD`, `LIT`, and `SHARP` each do something. The others refuse in their own words.
- Tap and charge, either hand, both hands together, resistance, fading, and the plain preview at the crosshair all work as `full-game/docs/brief/5a-core-gameplay.md` sets out.
- The four Standing Pages are the canon's own, each learned with its full spectacle. `BURNING` from the forge and `COLD` from the mill race are a showcase arrangement: in the full game those two words come from Pages in other regions.

## Loot and the inventory

An early table, small enough to tune by hand and rich enough to show what loot is in this game.

| Kind | In the Showcase |
|---|---|
| Weapon types | Sword, Axe, Mace, Dagger, Bow, Shield, and one Staff |
| Materials | Iron, Marcher Steel, and Rootwood |
| Tiers | All six. Plain and Fair drop often, Endorsed and Engrossed now and then, and one Illuminated item is guaranteed from the Bone Clerk |
| Traits | The seven weapon types' traits and the three materials' traits |
| Bonuses | 8 |
| Clauses | 6 |
| Word clauses | 2, for `BURNING` and `COLD` |
| Armour | One light set and one heavy set, in two materials, and village clothes |
| Trinkets | 4 |
| Artefacts | Wick's Spade, earned by finishing his errand, and The Promised Lamp, in the mill cellar |

- Looks differ. Two swords of the same material and tier rarely look alike, and each tier adds its own trim and ink treatment.
- The inventory shows the player's character in full, turning, lit as the place they stand in. Equipping changes it at once.
- Aiming at a dropped item compares it with what is in the hand.
- Tobit buys, sells, and restocks each morning.
- Every blade prints its own count of what it has done. Deeds are left for the full game.
- The loot simulation runs against this table at the gate: every tier appears in a 45 minute run, and nothing found outclasses the two artefacts.

## The look, and how it runs

This is where the Showcase spends its effort. The full specification is `docs/TECH.md`, which gathers the renderer rules from the full-game brief and sets the Showcase's own, stricter, targets.

- From any distance beyond about six metres, a screenshot must pass as a conventionally rendered, naturally lit game.
- Within a few steps, every surface is fine print. At arm's length it can be read, and it says true things.
- **The selling point: the closer you come, the further you see into things.** Within arm's reach a surface opens letter by letter onto a living lattice of characters inside it, coloured and lit by the material it is part of, turning over as the eye moves. Now and then the letters spell what the thing is and what it is made of, and while it is hot or cold they say what is happening to it (`BURNING`, `MELTING`, `BOILING`), glitching in and out. Every material's words are in `assets/materials/lexicon.ron`.
- Dusk, night, firelight, and fog are the images the game is sold on.
- **60 frames a second at 1080p on an Apple M1 with 8 GB,** with no hitch a player can feel. This is twice the full game's minimum, and it is possible here because the area is small and loaded whole.

## Done means

The Showcase Gate passes when all of these are true, with evidence on disk.

- The tour above can be played start to finish without a prompt from the owner, a crash, or a placeholder.
- The owner has played it on their own Mac and wants to show it to someone.
- A capture beyond the reveal distance cannot be told from one with the glyph layer off, and a capture at arm's length is legible everywhere it is pointed.
- The performance run in `docs/TECH.md` passes on the owner's Mac.
- The scripted bot plays the whole tour, and the word matrix has a result or a refusal for every word on every kind of target in the Showcase.
- A stranger given the build and no instructions casts a word within three minutes and reads something within five.
- Builds exist for Windows, macOS, and Linux.

## After the Showcase

The Showcase covers the first three phases of the full-game plan and parts of the next four. When the gate passes, Claude Code writes `docs/CONTINUE.md`, which lists every task in `full-game/phases/` as done, partly done, or not started, and moves the full-game plan to the root of the repository as the live plan.

| Full-game phase | State after the Showcase |
|---|---|
| 00 Foundation | Done |
| 01 Look prototype | Done, at a higher standard |
| 02 Hook room and arena | Done, in the village and the ossuary |
| 03 Terrain and streaming | The renderer features are done. Streaming and the full terrain are not started |
| 04 Living nature | Grass, trees, water, rain, and fog are done. The rest is not |
| 05 Player and first town | The controller and input are done. Split-screen and Ambry are not started |
| 06 Combat and first dungeon | Combat, four species, and the item generator are done. The dungeon standard is set |
| 07 RPG systems | The inventory, dialogue, and save are started. Skills, crafting, and the rest are not |
| 08 onward | Not started |

The first work after the gate is split-screen co-op, then streaming, then the rest of the Vale.
