# Section 5: enemies and bosses

### Enemies

| Target | In the canon | 1.0 |
|---|---|---|
| Distinct species and enemy types | 24 | 21, with 3 in Reserve |
| Regional variants and ranks | 80 | 21: one per species. Annotations do the rest |
| Rig families | 6 | 6 |

**Families:** beasts, giant insects and spiders, undead, human factions, ink and paper creatures, Verso aberrations, and Redactors. Ink and paper creatures are shape-shifting masses of characters that need little traditional animation, so lean on them for the strangest enemies.

A species is distinct only if it differs in silhouette, movement, glyph family, attack set, and behaviour. Recolours and resizes count as variants and never as species. Each Quire and the Verso has at least two native species. Rival factions fight, and species react to Emendation. Every species has its own sound, loot, and bestiary entry that fills in as the player studies it. Every species answers at least two Emendation words in its own way, and the bestiary entry records each one the player discovers. At least one of those words must open the species to steel, through that word's combat line in `5a-core-gameplay.md`.

### Annotated enemies

An annotated enemy is an ordinary creature with one extra word written on it, shown on the interface as an elite and named for its word: a Burning Bandit, a Locked Bone Clerk. They make common fights differ and are the commonest source of good loot.

- The extra word comes from the 15 annotations in `docs/bible/07-bestiary-and-bosses.md`. It stands in the creature's Inscription with the rest, glows like any word the player can change, and shows in its letters.
- The word is its strength and its weakness. While it stands, the creature has that word's power. A charged word of the same class replaces it, which strips the power and staggers the creature. Steel alone still wins, against a harder fight.
- An annotation is an ordinary state, so it can complete a compound.
- Every species supports at least three annotations, and they do not count towards its variants. They are why only one hand-made variant is built for each species: an annotation changes how a creature fights, and most variants only changed how it looked. Bosses are never annotated, and no creature carries two.
- About one group in ten holds an annotated enemy, more often in higher level bands. Two times in three, the generator picks an annotation the player already owns an answer to.
- An annotated enemy always drops an item of the Endorsed tier or better.
- The Emendation matrix covers every annotation on every species that supports it.

### Bosses

| Tier | Count | Build tier |
|---|---|---|
| Redactors (world-event bosses) | 3 | Full |
| Story and questline bosses | 9 | Full |
| Hidden superbosses | 2 | Full |
| Quire tyrants | 6 | Light |
| Dungeon bosses | 10 | Light |

All 30 named bosses in the canon are built. Every boss has a named identity, a suitable arena, attacks learned through telegraphing, a weakness that rewards Emendation or preparation, and a unique reward. Bosses scale for co-op.

- **Full bosses (14)** have multiple phases and a moveset no other boss shares.
- **Light bosses (16)** have one phase. Each is built on an existing rig and moveset, with one signature attack, one authored Emendation weakness, and its own arena. A Quire tyrant's fight is Light because the interesting choices sit outside it: each can be defeated, bought off, exposed, or rewritten.
