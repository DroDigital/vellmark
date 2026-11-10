# Phase S05: Things

**Goal:** Loot worth stopping for, and an inventory that shows you wearing it.

**Suggested model:** Opus 5.5, high effort. Switch to Sonnet 5.5 at high effort for the data and interface tasks marked Content and Shell.

**Hand to Haiku 5.5:** Bonus and clause data entry, name lists for the item generator, menu screens that follow an existing one, and save round-trip tests. Rules are in `CLAUDE.md` under Models and helpers.

## Read first

- `full-game/docs/brief/5d-loot.md`
- `full-game/docs/brief/5e-ui-and-accessibility.md`: General, Plain names, The character in the inventory
- `docs/SHOWCASE.md`: Loot and the inventory
- `full-game/docs/bible/08-words-items-craft.md`: Rarity, Weapon types, Materials, Item naming, and the two artefacts

## In this phase

The item generator and the Showcase loot table, the six tiers with their looks, armour and clothes on the character, the inventory, one merchant, and save and load.

## Tasks

Do the tasks of one lane in order, following the task loop in `CLAUDE.md`. Lanes run side by side as `docs/CLOUD.md` sets out. A task marked **after** waits for the task it names.

- [ ] **S05-T01** [Sim] Build the deterministic item generator: type, material, tier, traits, bonuses, clauses, word clauses, name, and history
- [ ] **S05-T02** [Content] Write the Showcase loot table: seven weapon types, three materials, eight bonuses, six clauses, two word clauses, four trinkets, and the drop rates for each source
- [ ] **S05-T03** [Content] Assemble weapon looks from the approved parts, with at least twelve looks for each type before material
- [ ] **S05-T04** [Render] Add the six tiers' ink treatments, trims, tier marks, and the generated text on each blade
- [ ] **S05-T05** [Content] Build the light and heavy armour sets and village clothes as layers on the character assembler
- [ ] **S05-T06** [Shell] Build the interface kit in letters: panels, lists, focus, the two-hand HUD, Ink, Health, Stamina, and prompts
- [ ] **S05-T07** [Shell] Build the inventory with the turning full-body character, lit as the place, with item preview
- [ ] **S05-T08** [Sim] Add pickups, the comparison with what is held, drop sounds and pickup effects for each tier, and the favourites wheel
- [ ] **S05-T09** [Sim] Add the merchant: buying, selling, and restocking each morning
- [ ] **S05-T10** [Content] Build Wick's Spade and The Promised Lamp, each with its own effect
- [ ] **S05-T11** [Sim] Add save and load with atomic writes, and settings saved apart from the game
- [ ] **S05-T12** [Sim] Build the loot simulation against the Showcase table and tune the rates until every tier appears in a 45 minute run
- [ ] **S05-T13** [Shell] Run the exit gate and save the evidence

## Exit gate

The loot simulation passes. Two items of the same type, material, and tier look different in a contact sheet of fifty. A save made in the ossuary loads in the same state. The inventory holds the frame budget while open.
