# Section 0: pillars, cost tiers, and releases

Read this file at the start of every phase. It decides arguments the other files cannot.

## The three pillars

Vellmark is worth building because of three things no other game does together.

| Pillar | The promise | Lives in |
|---|---|---|
| **Read** | The world is text, and the text is worth reading. Walls, robes, bark, and blades say true things, and some of them matter | The Glyph Cascade, the Three Layers of Text, reading mode, close reading, typographic body language, books, Perception, the journal |
| **Rewrite** | You can change what the world says, and it changes. This is the world's magic, used in fights, locks, stealth, puzzles, and conversations | Emendation, Inscriptions, Standing Pages, sorts, typos, the Remembered, rewritable characters |
| **Wander** | A lantern-lit realm you want to stand still in. Dusk, weather, long views, and something worth stopping for every minute or so | Art direction, lighting, weather as typography, the unravelling Erasure, travel through a letter, the density rule, the quest director, co-op free roaming |

## The pillar test

Before building or expanding any system, name the pillar it serves.

- If it serves one directly, build it well.
- If it only supports the pillars (saving, inventory, merchants, movement), build the plainest version that works and stop.
- If it serves none, do not build it. Record the idea under **Not building** in `docs/SCOPE.md`.

When two tasks compete for effort, the one closer to a pillar wins. Combat, loot, and magic are exempt from this test, as set out under Blade and word below.

## Plain first, strange underneath

Vellmark must be easy to pick up for anyone who has played an open-world RPG. Its strangeness is a reward for looking closer, never a toll at the door.

- A new player must be able to play the first hour without learning a single invented term or rule.
- Everything the player must understand is shown, not explained: what can be changed glows, and what a word will do is shown before it is cast.
- Reading is always rewarded and never required. Nothing on the critical path depends on text the player has not been pointed to.
- The interface speaks plainly. The world keeps its own words for those who listen.
- Depth stays for those who want it: typing words, typos, sorts, close reading, hidden lines, and the Remembered are never taught by a prompt and never needed to finish the game.

## Blade and word

By the owner's decision, the game is played with a weapon in one hand and a word in the other.

- **Magic is a main feature.** Emendation is the only magic, and it is in the player's hand in every fight, not kept for locks and puzzles.
- **Melee stays shallow.** Tap, hold, and block are all of it. No effort goes into deeper movesets.
- **Loot carries the depth of weapons.** Six rarity tiers, materials, traits, bonuses, clauses, and looks decide what a weapon is.

These systems do not need to justify themselves against the pillars. Keep the controls familiar to anyone who has played Skyrim, and put the invention into what the words do and what the loot is.

## Cost tiers

The canon in `docs/bible/` and `docs/script/` is a catalogue. It is larger than what one release can carry. Every catalogue entry has one of three build tiers, recorded in `docs/SCOPE.md`.

| Tier | Meaning |
|---|---|
| **Full** | Built to the complete standard in the brief |
| **Light** | Built in a cheaper, named form that keeps the entry's identity. A Light boss has one phase on an existing rig. A Light character has a name, job, home, schedule, readable clothing text, and a few lines, with no dialogue tree |
| **Reserve** | Stays in the canon, is not built, and is never referenced by built content. A Reserve entry moves up only when the owner says so, normally at a gate |

Nothing changes tier silently. To move an entry, edit `docs/SCOPE.md`, add a line to `docs/PLAN_CHANGES.md`, and regenerate the plan.

**Entries that can never be Reserve:** anything on the main story's critical path, anything in the five major questlines or the six Author quests, anything a secret chain or the Colophon ending depends on, every Standing Page, and any scene with an approved voice line in `docs/voice/voice_lines.tsv`.

## Releases

Vellmark ships in stages. Each stage is a complete, polished thing to play, and each ends in an owner gate.

| Release | Phases | What a player gets |
|---|---|---|
| **Demo** (free, if the owner approves) | P02 | The hook room and its arena: five words, one creature in four forms, and weapons to find, for one or two players |
| **Release 1** (early access) | P00 to P10 | The title sequence, intro, prologue, and main story Q1 to Q4. Scrivenhold Vale at final quality: the capital, Ambry, Little Penance, six dungeons, four Standing Pages, the opening quests of the five questlines. The other Quires and the Erasure are visible on the horizon and closed by the story |
| **Update: The Foxing Wood** | P11 | Q5 and the Wood |
| **Update: Saltmargin** | P12 | Q6, the coast, and Q7 over Scrivenhold. Act One ends and Erasure Stage 1 begins |
| **Update: The Verso** | P13 | Q8 and the reverse of the page |
| **Update: The Blotmere** | P14 | Q9 and the marshes |
| **Update: The Cinder Folio** | P15 | Q10, the ember-land, and Caret's attack on the coast |
| **Update: The Whitemarch** | P16 | Q11 and the northern highlands |
| **Update: Act Two closes** | P17 | Q12 to Q14 in Scrivenhold and the Verso. Erasure Stage 2 |
| **Update: The Erasure** | P18 | Q15 to Q17, Deleatur, and the Last Scriptorium. Erasure Stage 3 |
| **1.0** | P19 and P20 | The final dungeon, the four endings, the epilogue, and the full release |

**Release rules.**

- A closed road is always explained in the world, never by an invisible wall or a message about early access. In Release 1 the Ledger Guard holds the three old roads out of the Vale, and each post has its own written reason.
- Every release loads every save from every earlier release.
- The Slice Gate report measures real effort per content type. The owner uses it to confirm or change the tiers for everything after Release 1.
- The store page says what the current release contains, and makes no promise about dates.
