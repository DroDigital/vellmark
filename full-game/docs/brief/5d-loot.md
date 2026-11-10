# Section 5: loot

### Loot

Loot is where a weapon's depth lives. A sword swings the same way everywhere, so which sword it is has to matter. Every drop should be worth a look: what it is made of, how rare it is, what it adds, and how it looks in the hand.

**Item tiers,** each shown through ink treatment, a tier mark beside the name, drop sound, and pickup effect, and never through colour alone:

| Tier | On the interface | What it carries | Ink treatment |
|---|---|---|---|
| Plain | Common | Its material and its traits | Clean ink in the material's colour |
| Fair | Uncommon | One bonus | Ruled borders and a maker's mark |
| Endorsed | Rare | One clause | A line of coloured capitals along the item |
| Engrossed | Epic | One clause and one bonus | Coloured capitals with red rubric initials |
| Illuminated | Legendary | One clause, one word clause, and two bonuses, with ornate parts no lower tier uses | Painted initials and a slow shimmer in the ink |
| Canonical | Unique | A hand-designed effect | Gold-leaf glyphs and moving text |

**Content targets:**

| Item content | Count |
|---|---|
| Rarity tiers | 6 |
| Movesets, covering the eight weapon types | 4 |
| Base weapon designs: the weapon types across the materials | 48 |
| Assembled looks for each weapon type, before material | At least 12 |
| Traits | 16: one for each weapon type and one for each material |
| Bonuses | 20 |
| Clauses | 30: 18 for armour and trinkets, 12 for weapons |
| Word clauses, found only on Illuminated items | 12 |
| Deeds a weapon can earn | 8 |
| Armour sets: light and heavy by material, plus the 8 collectable sets | 16 |
| Rings, amulets, charms, and other trinkets | 16 |
| Collectable item sets with set bonuses | 8 |
| Canonical artefacts, each with custom effect and lore | 40 |

- A deterministic item generator combines base type, material, rarity, bonuses, and clauses, and writes a name and history for each item. Chests, enemies, and merchants each draw from their own lists, and merchants restock.
- Models are assembled from approved weapon models split into blades, hilts, plates, and trims, plus parametric parts and engraved patterns, so each material is recognisable at a glance. The generator varies the parts, so two swords of the same material and tier rarely look alike, and each tier adds its own trim.
- Stats stay few and readable: damage, armour, speed, reach, and weight. Everything else comes from a trait, a bonus, a clause, or a perk.
- Aiming at a dropped item shows its tier and how its damage or armour compares with what is equipped in that hand or slot.
- Gear is improved with Smithing in five steps, shown as tally marks after the name.
- Canonical items change how you play, such as a sword that rewrites its victim's `HOSTILE` to `AFRAID`. They must never be mere stat upgrades, they cannot be unmade, and all 40 in the canon are built.
- Item surfaces carry generated text, so a blade can show its own maker, its clauses, and its kills.
- The loot simulation checks levelled lists and rarity rates at every level band. It fails if crafted gear at a given skill out-performs the Canonical items of that band, or if an Illuminated item of one material is weaker than a Plain item of the next. Deeds are counted as bonuses in every check.

**What makes an item.** Four things, and the interface names each of them plainly.

- **Material** sets base damage, armour, and weight, and gives one trait. Materials appear on levelled lists as the player grows: Iron, Marcher Steel, Saltwork, Rootwood, Slagglass, Blacksteel, and Gilt, with Bookplate as light vellum armour. Higher materials need higher Smithing to forge and improve.
- **Traits** are fixed and always present, on every tier. A weapon has two: one from its type and one from its material.
- **Bonuses** are small rolled improvements, each tied to a maker whose name becomes the item's prefix. There are 20, written in full in Phase 07. They cover damage, speed, reach, weight, stagger, Stamina cost, damage against one family of enemies, and the blade-and-word rules.
- **Clauses** are this world's enchantments, kept distinct from the Inscriptions on targets. Each has one effect whose strength depends on the Inscribing skill of whoever wrote it, and weapon clauses carry a charge that is refilled with bottled Ink. Their names come from legal language, as in the canon.

About a third of the bonuses and clauses serve blade and word: more Ink from each hit, cheaper words, a word that stays longer on the weapon. The best weapon for a player is the one that suits the word in their other hand.

**Word clauses** change what one word does, and never a number. They are the reason to chase the Illuminated tier.

- There are 12, written in full in Phase 07, each naming one word. Examples: a `COLD` you write passes on to the next enemy your weapon strikes. A word written on this weapon stays until the weapon is put away. A tapped `OPEN` also staggers what it opens. `WHOLE` written on yourself mends your armour as well.
- They are found only on Illuminated items, cannot be learned by unmaking, and cannot be written by Inscribing.
- Two times in three, the generator picks a word clause for a word the finder owns.
- No word clause may be stronger than a Canonical item's effect. The loot simulation checks this.

**A weapon's history.** A weapon that is used earns something of its own, so a favourite is worth keeping.

- Every weapon, bow, staff, and shield counts what it has done in its owner's hands, and prints the count on itself.
- At 100 kills, a starting value for the loot simulation to tune, it earns one **Deed**, chosen from how it was used. A Deed is as strong as one bonus. It is engraved on the item as a line of text, belongs to the item for good, and goes with it if it is sold or handed to a partner.
- The 8 Deeds: Quiet (mostly sneak kills: a higher sneak multiplier), Lettered (mostly kills while it carried a word: words stay on it longer), Thirsty (mostly kills on targets under the wielder's own word: more Ink from each hit), Heavy-handed (mostly heavy attacks: easier staggers), Opportunist (mostly kills on staggered targets: more damage to them), Hunter (mostly one family of enemy: more damage to that family), Steadfast (mostly kills straight after a block: cheaper blocks), and Long-sighted (mostly kills at range: less arrow drop).
- A weapon earns one Deed and no more. A Deed does not change the item's tier. Canonical items keep their count and earn no Deed.

**Traits by weapon type.**

| Type | Trait |
|---|---|
| Sword | Balanced: each hit returns more Ink |
| Axe | Bleeds: hits deal damage over time |
| Mace | Crushes: ignores part of the target's armour |
| Dagger | Quick: the fastest swing and the highest sneak multiplier |
| Greatsword | Sweeps: hits everything in its arc |
| Staff | Holds one word, written from its own charge |
| Bow | Draw strength sets damage and range |
| Shield | Blocks, and bashes once from a block |

**Traits by material.**

| Material | Trait |
|---|---|
| Iron | Honest: no trait, and the cheapest to forge and improve |
| Marcher Steel | Keen: heavy attacks stagger more easily |
| Saltwork | Brined: extra damage to `WET` targets |
| Rootwood | Living: light, and Ink refills faster while it is held |
| Slagglass | Sharp: extra damage to staggered and unaware targets |
| Blacksteel | Dense: the heaviest, with the highest base damage and armour |
| Gilt | Fine: a word written on it lasts longer |
| Bookplate | Lettered: words cost less Ink while it is worn |

**Rarity rules.**

- Material and rarity are rolled separately. Material comes from the levelled list for the place and the player's level. Rarity comes from the source.
- Starting rates for an ordinary chest, to be tuned by the loot simulation: Plain 50%, Fair 28%, Endorsed 14%, Engrossed 6.5%, Illuminated 1.5%. Common enemies drop mostly Plain and Fair. Annotated enemies always drop Endorsed or better. Bosses and the guardians of Standing Pages always drop Engrossed or better.
- Merchants never stock Illuminated items. Canonical items are never rolled: each has its one authored source.
- Crafting stops below the top. Smithing forges Plain gear, and Fair gear with a perk. Inscribing writes one clause onto it, making it Endorsed or Engrossed. Illuminated and Canonical items can only be found.
- A name states what an item carries. A bonus adds its maker as a prefix, a clause adds its legal suffix, and a word clause follows a comma: "Tally's Iron Mace", "Iron Mace of Prior Claim", "Tally's Iron Mace of Prior Claim", "Tally's Gilt Sword of Reasonable Doubt, Without Prejudice".
- In co-op, rarity is rolled for each player, as all loot is.
