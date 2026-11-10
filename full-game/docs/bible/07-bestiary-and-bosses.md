# Bestiary and bosses

Totals in the canon: 24 species, 80 variants, 30 bosses. `docs/SCOPE.md` records the build tier of each: 21 species and 21 variants are built for 1.0, and all 30 bosses are built, 14 Full and 16 Light. Every species uses one rig family: biped (B), quadruped (Q), multi-legged (M), serpent (S), winged (W), or amorphous (A).

## Species

| # | Species | Family | Rig | Habitat | Glyphs | Behaviour and Emendation reaction |
|---|---|---|---|---|---|---|
| 1 | **Blot Hound** | Beast | Q | Vale, Blotmere | `# %` | Packs; bites drop Ink. CALM makes it follow you for a while |
| 2 | **Long Goat** | Beast | Q | Whitemarch | `Y v` | Charges from high ground. HEAVY stops the climb |
| 3 | **Tally Bear** | Beast | Q | Foxing Wood, Whitemarch | `8 B` | Marks territory by scratching accounts. Hibernates if ASLEEP |
| 4 | **Ash Boar** | Beast | Q | Cinder Folio | `% &` | Leaves burning trails. WET extinguishes its hide |
| 5 | **Quire Moth** | Insect | W | Foxing Wood, College | `w x` | Swarms and eats written text. LIT draws it away |
| 6 | **Staple Wasp** | Insect | W | Vale, Cinder | `v '` | Pins prey in place. COLD slows the nest |
| 7 | **Silverfish Elder** | Insect | M | Libraries, Stacks | `~ s` | Bursts from books. Burrows in paper walls |
| 8 | **Spine Spider** | Spider | M | Saltmargin, Foxing | `H X` | Webs of binding thread. BURNING cuts the webs |
| 9 | **Margin Crab** | Beast | M | Saltmargin | `[ ]` | Shell blocks frontal attacks. LIGHT flips it |
| 10 | **Eelmother's Brood** | Beast | S | Blotmere, sewers | `S ~` | Ambush from water. DRY strands them |
| 11 | **Slag Worm** | Beast | S | Cinder Folio | `@ o` | Tunnels under you. COLD hardens it |
| 12 | **The Shambling Entry** | Undead | B | Crypts | `E F` | A dead person's record walking around. Reads its charges aloud |
| 13 | **Bone Clerk** | Undead | B | Barrows, ossuaries | `I l` | Summons paperwork that traps you. BROKEN shatters armour |
| 14 | **Drowned Parishioner** | Undead | B | Saltmargin, Blotmere | `o 0` | Slow, waterlogged, and grabbing |
| 15 | **Footnote** | Spirit | A | Ruins, Verso | `* 1` | Small floating numeral that attaches to larger enemies and buffs them |
| 16 | **Bandit** | Human | B | Everywhere | Cloth, metal | Ambushes and retreats. AFRAID scatters them |
| 17 | **Choir Zealot** | Human | B | Blotmere, Erasure | `x +` | Silences your Emendation in an area |
| 18 | **Rogue Guard** | Human | B | Vale, roads | Metal | Shield formations |
| 19 | **Marcher Reaver** | Human | B | Whitemarch | Tattoo | Two-handed berserkers |
| 20 | **Inkling** | Ink | A | Everywhere after Act 1 | `.,;:` | Formless, fast, and splits when struck. DRY cracks it |
| 21 | **Paper Golem** | Paper | A | College, Erasure | `# =` | A folded giant. BURNING is very effective, but sets fire to the room |
| 22 | **Scraper** | Paper | W | Erasure | `- _` | Thin blade-like flyer that removes Inscriptions |
| 23 | **Misprint** | Verso | A | Verso, Erasure | Mirrored | Copies your last attack back at you |
| 24 | **Redactor** | Redactor | W | World events | Solid black | Crosses out whatever it touches |

## Annotated creatures

Now and then the Ledger adds a word to an entry and does not say why. Any species may be found annotated, named for the word it carries. Rules are in `docs/brief/5c-enemies-and-bosses.md`. Annotations are separate from the variants below.

| Annotation | What it gives the creature | Answered by |
|---|---|---|
| LOCKED | Armoured shut. Blows from the front do little | OPEN |
| DARK | Puts out lights near it, and is hard to see | LIT |
| LIT | Dazzles at close range, and can never hide | DARK |
| BURNING | Sets alight what it touches | COLD, WET |
| COLD | Slows whatever it strikes | WARM, BURNING |
| WET | Puts out fire, and cannot be set alight | DRY |
| HEAVY | Slow, and cannot be staggered | LIGHT |
| LIGHT | Fast, and leaps | HEAVY |
| HIDDEN | Unseen until it strikes | SEEN |
| WHOLE | Mends itself | BROKEN, ROTTEN |
| ROTTEN | Its touch weakens armour | WHOLE |
| FAST | Quick on its feet, and strikes twice | SLOW, STILL |
| LARGE | Half as big again, with reach and weight to match | SMALL |
| LOUD | Every cry it gives calls others | QUIET |
| SHARP | Its blows cut through armour | BLUNT |

## Variants (80)

Each entry is a named variant, in addition to the base species. Variants may change stats, size, palette, or one attack.

| Species | Variants |
|---|---|
| Blot Hound | Runt, Alpha, Mange, Bog |
| Long Goat | Kid, Ram, Old Debt |
| Tally Bear | Cub, Auditor, Great |
| Ash Boar | Sow, Tusker, Cinder King |
| Quire Moth | Common, Bookworm, Gilded, Death's-Page |
| Staple Wasp | Worker, Soldier, Queen's Guard |
| Silverfish Elder | Nymph, Bibliophage, Vellum-Eater |
| Spine Spider | Thread, Binder, Bonecloth, Queen |
| Margin Crab | Hermit, Fiddler, Big Margin |
| Eelmother's Brood | Elver, Conger, Lamprey |
| Slag Worm | Grub, Molten, Armoured |
| Shambling Entry | Pauper's, Merchant's, Sheriff's, Bride's |
| Bone Clerk | Junior, Senior, Auditor-General |
| Drowned Parishioner | Sexton, Bell-Ringer, Bloated, Choirboy |
| Footnote | Minor, Ibid, Op Cit |
| Bandit | Cutpurse, Archer, Bruiser, Captain |
| Choir Zealot | Hush, Psalmist, Whiterobe |
| Rogue Guard | Tallyman, Crossbow, Sergeant, Deserter |
| Marcher Reaver | Young, Tattooed, Debt-Mad |
| Inkling | Droplet, Spatter, Pool, Long Blot |
| Paper Golem | Folio, Quarto, Elephant Folio |
| Scraper | Razor, Pumice, Bone Folder |
| Misprint | Reversed, Doubled, Offset |
| Redactor | Sub-Editor, Black Line, Censor |

## Bosses

### Redactors (3)

| Boss | First seen | Fought in | Reward word |
|---|---|---|---|
| **Obelus** | Q3 | Q7, over Scrivenhold | ABSENT |
| **Caret** | Q10 | Saltmargin world event | LOYAL |
| **Deleatur** | Q16 | Road through the Erasure | NAMELESS |

### Quire tyrants (6)

Sir Pym Vexley, Tor Hask, Madam Alder Rue, Nan Twice, Ollo Lull, and Gudrun Anvil. Each can be defeated, bought off, exposed, or rewritten, but their boss fight remains available.

### Story and questline bosses (9)

| Boss | Quest | Note |
|---|---|---|
| **The Indexer** | Q4 | Archive automaton made of card drawers |
| **Bailiff of the Verso** | Q8 | Collects what the reverse is owed |
| **Precentor Aldhelm Sour** | Q9 | Voice attacks that silence words |
| **Ser Anselm Gall-Hand** | Q12 | Knight with no past and no hesitation |
| **The Second Edition** | Q14 | A corrected copy of you |
| **The Cantor** | Restoration route | Mercy as a boss fight |
| **Lord Registrar Thale** | Blank route | Fights with terrifying records of you |
| **The Corrector** | Free Hand route | Rewrites the arena |
| **The Clerk** | Colophon | Your past self |

### Dungeon bosses (10)

| Boss | Dungeon |
|---|---|
| Eelmother | Eelmother's Grotto |
| The Goatwife | Goatwife's Cave |
| Senior Clerk of the Nine | Barrow of the Nine Clerks |
| The Toll | Old Tollway Fort |
| Lamp Keeper Slip | Lighthouse of Saint Slip |
| The Last Foreman | Old Sawmill Cut |
| The Commander in the Tunnels | War Tunnels of Line 1206 |
| The Surveyor | Old Survey Tower |
| Queen of Bonecloth | Tomb of the Polite Dead |
| The Winter Abbot | Icefall Mine |

### Hidden superbosses (2)

| Boss | Where | Note |
|---|---|---|
| **The Sum of All Goats** | A hidden valley in the Whitemarch, after the Goat Debts chain | An impossibly large accumulated goat. Entirely serious |
| **Saint Slip's First Mistake** | The Verso, after all secret chains | The blot from which the Blotmere came |
