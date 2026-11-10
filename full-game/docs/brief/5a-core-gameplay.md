# Section 5: player, combat, Emendation, growth

## 5. Gameplay

**Player.** First-person only. Responsive movement, sprint, jump, swim, climb steep ledges, and a generous mantle. Health, Stamina, and Ink (magic). Jog at about 5 m/s and sprint at about 7.5 m/s. One in-game day lasts 60 real minutes.

**Death.** A slain player's line is struck through, and they return at the latest autosave, with no permanent penalty.

**Combat: blade and word.** First-person combat built around one pairing above all others: a weapon in one hand and a word of Emendation in the other. Swinging is simple and weighty. The depth is in what the words do and in which weapon is being swung, never in the moves.

- **Two hands.** Each hand has its own button and holds a weapon, a shield, or a word. A one-handed weapon with a word is the default, and it is the pairing every fight is tuned for. The prologue puts a weapon in one hand first, and `OPEN` settles into the free hand when it is learned, so a new player is holding blade and word within three minutes without choosing to. Every other pairing still works: weapon and shield, two weapons, or two words. A two-handed weapon or a bow takes both hands. A favourites wheel swaps what is equipped without opening the inventory, and remembers whole pairings.
- **Melee is shallow on purpose.** Tap to attack, in a short chain of alternating swings. Hold for one heavy attack, which costs Stamina and can stagger. That is the whole moveset. There are no directional power attacks, no combos to learn, no timed blocks, and no special moves for two weapons, which simply swing one per hand.
- **Blocking.** Hold to block with a shield, with a two-handed weapon, or with a one-handed weapon when the other hand is empty. A shield can bash once from a block to interrupt. A hand that holds a word writes and does not block: that player's defence is the sidestep and what they write.
- **Sidestep.** One button gives a quick step of about two metres in the direction of movement, or backwards when standing still. It costs Stamina, attacks pass through its first moments, and it never drops a charge held in either hand. Every pairing has it. There is no roll and nothing to chain.
- **Bows.** Draw strength sets damage and range, arrows drop, and a held draw drains Stamina. Arrows can be recovered.
- **Sneak attacks.** An unaware target takes multiplied damage, highest with a dagger.
- **Four movesets cover the eight weapon types in the canon.** One-handed (Sword, Axe, Mace, Dagger), Two-handed (Greatsword, Staff), Bow, and Shield. Types that share a moveset swing the same way. They differ in damage, speed, reach, and one fixed trait each, listed in `5d-loot.md`: an axe bleeds, a mace ignores some armour. A Staff holds one word and writes it from its own charge, so anyone can use it.
- **Where the depth lives.** One fight differs from another because of the weapon in the hand (its material, rarity, traits, bonuses, and clauses, all in `5d-loot.md`) and the words written. A better sword is found, never learned.
- **Feel.** Hit stop, screen response, and audio must make every hit and every written word land. Hit stop is purely visual and per view, and never pauses the shared simulation.
- **Enemy behaviour.** Enemies telegraph clearly and flank, retreat, call for help, and use cover. They also react to words written on them, their allies, and the room.

**Blade and word together.** Four rules make the two hands one way of fighting. Build all four, and tune them before anything else in combat.

- **Hits give Ink.** Every weapon hit returns a little Ink, and a heavy attack or a sneak attack returns more. Strike to earn the next word.
- **Words open, steel finishes.** Every word leaves a creature in a state that a weapon exploits, held on that word's sheet as its combat line. `BROKEN` armour lets full damage through. A `COLD` target is slowed, and a heavy attack shatters the frost for extra damage. A `HEAVY` target cannot dodge or turn quickly. An `AFRAID` target shows its back to a sneak attack. All 35 words have at least one combat line, and the validator fails if one is missing.
- **Words on your own weapon.** The weapon in a player's hand is a target like any other. Hold both hands' buttons together to write the equipped word on the held weapon, where it fades like any other change: `BURNING` sets what it hits alight, `COLD` slows what it strikes, `LIGHT` swings faster, `HEAVY` swings slower and staggers, `LIT` makes it a lamp, `HIDDEN` makes it silent, `SHARP` makes it cut deeper, `FAST` quickens every swing. A word that means nothing on a weapon gives its refusal line.
- **A tap fits between two swings.** A tapped word never roots the player and never cancels the weapon hand's chain, so swing, write, swing is one rhythm.

The game is tuned for blade and word together. Every fight must still be winnable with steel alone and with words alone, so that no build is a dead end.

**Difficulty.** Story, Standard, and Hard, changeable at any time per world. Enemy damage, health, and aggression change, but never quest logic.

**The first hours.** The prologue teaches movement, reading, one Emendation word, and combat through play, with a short skippable prompt for each. Quest clues must never depend on a single tutorial message.

**Power first.** The player learns `OPEN` within three minutes of taking control, and the first thing they cast it on is large: the ossuary's bronze doors, three times their height, which are thrown back with the full magic effect while every lesser lock on the hill springs with them. Explanation comes after the spectacle, and briefly.

**Emendation (the magic system and the game's big idea).** Interactive objects and creatures have a visible **Inscription** of 1 to 3 state words, such as `LOCKED`, `DARK`, `COLD`, `HEAVY`, `ASLEEP`, `HOSTILE`, `WHOLE`, `HIDDEN`. Your words come from **Standing Pages**, ancient stones hidden across the world. Replace a word on a target with one you own and the world changes:

- `LOCKED` to `OPEN`, `DARK` to `LIT`, `COLD` to `BURNING`
- `HEAVY` to `LIGHT` to throw a boulder, `HOSTILE` to `CALM`, `WHOLE` to `BROKEN`

Words have an Ink cost based on the target's size and resistance, with systemic interactions (a `BURNING` door ignites nearby `DRY` thatch). **Casting.** Equip a word to a hand, aim at a target, and tap that hand's button to write it. Hold to charge: a charged word reaches tougher resistance tiers, lasts longer, and costs more Ink. The same word in both hands can be written together for a stronger effect at a higher cost. Ink refills over time, faster out of combat, and every weapon hit returns a little. Being hit while charging loses the charge, and a full charge smudges into the word's typo, as set out in `5h-signature-mechanics.md`. A staggered or unaware target resists less. While charging, a player may also type the word on a keyboard, or enter it on the controller's letter wheel, for a bonus, and the two methods must be equally rewarding. Late-game words can do frightening things. Rewriting a person's name or allegiance has consequences, and NPCs remember.

**How it is shown.** The player should feel they are casting spells, and never that they are solving grammar.

- With a word equipped, everything in view that it can change is picked out in that word's light, and the crosshair shows the result in two or three plain words before the cast: "Opens the gate", "Sets it alight", "It will flee".
- A target that needs a charged cast shows this as a ring that fills. A Sealed target shows its gold lettering and its refusal line.
- The words class and resistance never appear on screen. The same-class rule is felt only as some things lighting up and others not.
- A target's Inscription is shown as the word alone, floating small beside it when aimed at, with no label.

**Standing Pages are events.** Each of the 20 Standing Pages is a landmark visible from a distance, reached through a guardian, a climb, or a puzzle that suits its word. Learning the word is a spectacle: the word lifts from the stone, the screen holds on it, and the word acts once on everything around the Page, so that learning `LIT` kindles every lamp in the valley below. The player leaves with a new power and has already seen what it does.

**Emendation rules.**

- Each word belongs to one of ten classes: Access, Light, Temperature, Weight, Mind, Integrity, Concealment, Sound, Motion, or Size. A word can replace only a word of the same class.
- There are 35 words. Twenty come from Standing Pages and three from Redactors. The other twelve are **loose words**, learned with the Read action from small things in waiting places, two in each Quire. A loose word lifts from its object with its motif and a small version of the learning spectacle, and acts once on what is nearest.
- `LARGE` and `SMALL` change a target's size by about half, with its reach and real mass, within the physics budgets of Section 5i. They are refused indoors where there is no room, and by bosses and protected characters.
- Every target has a resistance tier. **Sealed** words appear in gold leaf and cannot be changed, except through authored story moments.
- Most changes fade after a set in-game time, and active changes are saved. Quest-authored changes can be permanent.
- Emendation can never softlock the game. Quest-critical objects and doors are Sealed, or have an authored outcome for every available word.
- Bosses resist Mind words, except for their authored weaknesses.
- Rewriting a name or allegiance is available only on about 12 authored characters, each with written consequences.
- Word effects live in data, one sheet per word, as set out below. The validator fails unless every word and target class has either a result or an explicit in-world refusal line.

**One sheet per word.** Everything about a word is written in one file, `assets/data/words/<word>.ron`, and nowhere else. Changing a word is one edit, and adding one is one new file.

- A sheet holds the word's class and Ink cost, its result or refusal line for every target class, the plain preview shown at the crosshair for each, its combat line, what it does on a held weapon or the refusal, what it gives a creature as an annotation if it is one of the 11, its typo and the typo's effects, anything a sort changes for it, its motif as notes, its caption, and the settings of its five-part magic effect.
- Compounds have one small file of their own, since each belongs to two words. What a species does in its own way when a word is written on it stays on the species' sheet.
- Nothing about a word may be written in code. The bake fails on a sheet with a missing field, and the code reads only the baked table.
- The Emendation matrix, the validator's word checks, and the motif check are generated from the sheets, so a new word is tested without a new test being written.
- The bake also writes `docs/WORDS.md`, one page per word in plain language, for the owner to read and correct. Like `docs/PLAN.md`, it is never edited by hand.

**Compounds.** Two words standing on one target at once can make a third thing happen. This is where words reward experiment in a fight.

- A compound fires once, at the moment the second word lands, however each word got there: a cast, a Staff, an arrow, a typo's side effect, the room, an annotation on an enemy, or a word on the weapon that struck it. A word on a weapon counts at its lowest strength.
- The two words are always of different classes, since a target holds only one word of each class.
- There are 30 compounds, listed in `docs/bible/08-words-items-craft.md`. `COLD` and `BROKEN` shatter a target and hurt everything near it. `WHOLE` and `CALM` leave a creature grateful, and it fights beside the writer until the fight ends. Four can be made with the Vale's words alone.
- Nothing teaches them. The first time one fires it is named and recorded on the journal's **Compounds** page, which shows how many are still unfound. After that, the preview at the crosshair names a compound the next cast would complete.
- When two players supply one word each, the compound is stronger.
- Compounds obey resistance, never touch Sealed words, and do nothing to a boss that resists either word. No compound may complete, block, or break a quest, and the validator requires a result or a refusal for every compound on every target class.

**Spectacle.** The game's best advertisement is what its systems do by accident. Make the large moments large, and make them easy to share.

- When one word changes five or more targets at once, or a chain of reactions passes through four or more links, the effect escalates: the rewrite wave joins into one, the motif swells into the score, and each view answers with a stronger screen response. The physics and effect budgets of Section 5i still hold.
- These moments, along with a first typo, a first compound, a Redactor's pass, a boss falling, and a place writing itself back, are **marked**. A marked moment is added to Steam's game recording timeline, so a player who has Steam recording switched on can clip it afterwards without having planned to.
- The game records no video itself. That would need encoders outside the approved libraries, and Steam, consoles, and every operating system already do it. On itch the marks do nothing.
- If the `steamworks` crate does not expose the timeline, do not work around it. Record it under Not building in `docs/SCOPE.md` and tell the owner.
- Reduced motion and reduced screen response soften the escalation and never remove the mark.

**Character growth.** Eleven skills improve through use, as in Skyrim:

| Group | Skills |
|---|---|
| Arms | Weapons, Archery, Armour |
| Craft | Smithing, Inscribing, Distilling |
| Guile | Sneak, Lockwork, Speech |
| Letters | Emendation, Perception |

Level-ups increase Health, Stamina, or Ink and grant a perk point. Each skill has a perk tree of about eight perks. Emendation is the exception, with about sixteen in two branches: one for the field and one for the fight. Weapons covers every melee weapon and blocking, and its perks strengthen the traits of each weapon type. Armour covers light and heavy, with a short branch for each. Perks in both add bonuses and never add moves. That makes about 96 perks in all. Perks everywhere else should change what the player can do more often than they raise a number.

**Level scaling.** Each Quire and dungeon has a level band, so danger varies by place, and enemies scale only within that band. In co-op, encounters use the higher player's level, and the lower-level player gets a damage and health adjustment so both stay useful.
