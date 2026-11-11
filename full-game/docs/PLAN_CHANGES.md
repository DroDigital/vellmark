# Plan changes

Newest first. Record every change to scope, design, order, or tools here.

## 2026-11-11: a broken thing comes apart into its letters

Owner's direction during the Showcase. When a thing is destroyed, all of it, not only its small debris, scatters as tens of thousands of loose characters, each simulated with gravity, bounce, and friction against the world until it lies flat. At most 60,000 loose letters at once. Each letter keeps the place it came from, so that `WHOLE` can pull them back along their paths. Details are in the Showcase's `docs/PLAN_CHANGES.md`.

## 2026-11-11: up close, surfaces open onto a lattice of characters

Owner's direction during the Showcase. Within about two metres the realistic texture dithers away and the eye looks into a three-dimensional lattice of small characters, as deep as the object is thick, with exact parallax. Authored text stays whole at the front. Three variants (atoms, lattice, source) are built for the owner to choose between. Rule 3.5 of the brief, tone preservation, now holds only outside that range; the glyph-off comparison beyond the reveal distance is unchanged. Details are in the Showcase's `docs/PLAN_CHANGES.md`.

## 2026-11-10: more words and more compounds

Owner's decision.

- **Twelve loose words,** making 35: SHARP, HONEST, QUIET, LOST, LOUD, LARGE, SLOW, SMALL, FAST, BRAVE, STILL, and BLUNT. They are learned by reading small objects in waiting places, two in each Quire, and need no new Standing Page. The Vale has SHARP and HONEST.
- **Three new classes:** Sound, Motion, and Size, making ten. `LARGE` and `SMALL` really change a target's size and mass.
- **Eighteen new compounds,** making 30. Four can be made in the Vale.
- **Eleven new typos,** making 32. HONEST has none.
- **Four new annotations,** making 15: FAST, LARGE, LOUD, and SHARP.
- **Unchanged:** the 20 Standing Pages, the three Redactor words, and every voiced line.
- **Canon touched:** the words, typos, and compounds in `docs/bible/08-words-items-craft.md`, and the annotations in `docs/bible/07-bestiary-and-bosses.md`.
- **Risks:** every word must have a result or refusal for every kind of target and every species, so twelve words add about half again to that work and its testing. Changing size touches collision, movement, and physics, and is the most expensive of the new words.
- **Cost:** no new tasks, and about twelve widened ones. One sheet per word keeps each addition to one file.

## 2026-11-10: a wider audience

Owner's decision, to keep the game from being too niche.

- **Presentation:** the game is sold as "an open-world RPG where you rewrite reality with a word". ASCII and reading stay out of the first lines of any page, and the text view is the third screenshot.
- **Redactors lead.** One is on the key art. Obelus is seen from the first vista and runs its line down the Ambry road in Q2, where the player can only get clear.
- **Languages:** German, French, Spanish, and Brazilian Portuguese join English. Menus, subtitles, dialogue, quests, books, and reading mode are translated. The world's surfaces, the words of power, names, and the audio stay English. This replaces "English only".
- **Spectacle:** large casts and long chains escalate, and big moments are marked on Steam's recording timeline. The game records no video itself, because that needs libraries outside the approved list.
- **Seeing your character:** a turning full-body view in the inventory with item preview, and the character shown in photo mode. Play stays first-person.
- **Tone:** on the main path, danger and loss lead and the dry humour follows. Side content keeps it in full. Every cutscene can be skipped on first viewing.
- **Canon touched:** one stage direction added to Q1 and one scene of stage directions, The Line, added to Q2. One writing rule added to `docs/bible/03-voice-and-dialects.md`. No dialogue and no voiced line was changed.
- **Risks:** translated humour may fall flat, so each language is read by the owner or a native reader before it loses its beta label. Translation is the largest new cost in the plan and recurs with every update.
- **Cost:** three new tasks, in Phases 00, 06, and 10, and about fifteen widened ones.

## 2026-11-10: generated voices restored

Owner's decision. The change "letter speech replaces generated voices" is withdrawn in full, and its entry is removed from this file.

- **ElevenLabs is back** exactly as in the entry of 2026-11-09, "voice added on a fixed budget": 40,000 credits, 10 voice slots, 263 approved lines, 43 character presets, and a synthesized murmur for every other line.
- **No letter speech.** Nothing sounds out the letters of a line.
- **If the withdrawn version was copied into a repository:** restore `tools/generate_voice.py`, `docs/voice/voice_ids.example.json`, and `docs/voice/cast-and-budget.md` from this pack, and delete `docs/voice/cast-and-speech.md`.
- **Cost:** none. The voice tasks are as they were.

## 2026-11-10: Sonnet for the later regions, and named helpers

Owner's decision, refining the entry "models, effort, and helpers" below.

- **Sonnet 5.5 at high effort** now builds most of Phases 11, 12, 14, 15, and 16, which are content made to the standard the Vale sets. Opus 5.5 at high effort is kept in those phases for species and bosses, for quests, books, and surface text, and for each phase's one special piece of work.
- **Unchanged:** Fable 5.2 at medium effort for Phases 00, 01, 02, 06, 18, and 19 and the writing in 09. Opus 5.5 at high effort for Phases 03, 04, 05, 07, 08, 09, 10, 13, 17, and 20.
- **Four helpers** are defined in `.claude/agents/`: `helper-low`, `helper-medium`, `helper-high`, and `helper-max`, all on Haiku 5.5, one for each row of the effort table in `CLAUDE.md`.
- **Switching:** the owner sets the model with `/model` and `/effort`. Claude Code gives the two commands at the end of each phase and before any task that wants a different model.
- **If Fable is unavailable or billed separately,** its phases run on Opus 5.5 at xhigh effort.
- **Cost:** none in tasks.

## 2026-11-10: models, effort, and helpers

Owner's decision.

- **Three models.** Fable 5.2 at medium effort for Phases 00, 01, 02, 06, 18, and 19, and for the quests, books, and slice report in Phase 09. Opus 5.5 at high effort for every other phase. This replaces the earlier Opus and Sonnet suggestions.
- **Haiku 5.5 helpers.** The main session hands routine, fully specified, machine-checkable jobs to Haiku 5.5 subagents, at an effort chosen for each job from low to max. Each phase file lists the jobs it expects to hand over, and `CLAUDE.md` holds the rules.
- **Limits.** Helpers never touch the renderer core, anything a gate judges, creative writing for Full characters, quests, or books, voice credits, publishing, or the plan. The main session checks every helper's work before committing it.
- **Cost:** none in tasks. The split is a starting point, to be corrected by what the first phases show.

## 2026-11-10: prove the fight early, and build less by hand

Owner's decision. Four changes that make the game better and cheaper to build.

- **The fight is proven in Phase 02.** A grey-box arena beside the hook room tests blade and word, the sidestep, words on the weapon, three annotated Inklings, three compounds, and weapons dropping in two tiers. It is judged at the Hook Gate, before terrain, nature, or a town exist. Phase 06 extends that code and no longer builds these systems from nothing. The demo includes the arena.
- **One variant per species.** 21 variants are built in place of 48, and 59 are in Reserve in place of 32. Annotated enemies give each species at least three further forms that change how it fights. A variant named by a built quest or book is still built.
- **Eleven skills in place of fourteen.** Weapons replaces One-Handed, Two-Handed, and Block. Armour replaces Light Armour and Heavy Armour. About 96 perks in place of about 120. Marchers start with a Weapons bonus.
- **One sheet per word.** Everything about a word lives in one data file: effects, previews, combat line, typo, motif, and magic effect. Tests are generated from the sheets, and the bake writes `docs/WORDS.md` for the owner.
- **Canon touched:** the totals line of `docs/bible/07-bestiary-and-bosses.md`, and the Marchers' starting bonus in `docs/bible/02-peoples-and-daily-life.md`. No dialogue and no voiced line was changed.
- **Cost:** six new tasks in Phase 02, with five tasks in Phase 06 made smaller to match. Phase 02 is now about a third longer. The work removed is 27 variants and about 24 perks.

## 2026-11-10: waiting places, word motifs, photo mode, and a public demo

Owner's decision.

- **Waiting places:** about 40 places the player can see before they can enter, each closed until a word they do not yet own is learned. They are existing points of interest, secrets, and dungeon side rooms, so no place is added. They are marked on the map once read, and each new word says how many it opens. The Vale has six, two of which wait for later updates.
- **Word motifs:** each of the 23 words has a short musical phrase, with forms for charging, typos, sorts, compounds, annotated enemies, and words on weapons. The main theme is built from the Vale's four. Synthesis only, with no voice credits.
- **Photo mode:** a free camera with text view and a split frame, realistic on one side and raw text on the other. Store screenshots are now taken with it.
- **Commonplace book:** a journal page for lines the player keeps, exportable as text or as a typeset image. It has no effect on play.
- **Public demo:** after the Hook Gate, the owner is asked whether to offer the hook room as a free demo on itch.io. Feedback comes back through the owner into `docs/DEMO_FEEDBACK.md` and is sorted there. There is still no telemetry. Phase 03 does not wait for it.
- **Canon touched:** none. Waiting places are chosen from entries the canon already has.
- **Cost:** five new tasks, in Phases 02 and 07, and about fifteen widened ones. Nothing was moved to Reserve to pay for it.

## 2026-11-10: five additions to the fight

Owner's decision, building on the entry below.

- **Sidestep:** one button, a short step on Stamina that keeps a held charge. It is the defence for a player holding a word.
- **Annotated enemies:** ordinary creatures carrying one extra word, shown as elites and named for it. The word is their power, and a charged word of the same class strips it and staggers them. They always drop Endorsed or better. There are 11 annotations, and each species supports at least three.
- **A weapon's history:** a weapon counts its kills and, at about 100, earns one of 8 Deeds chosen from how it was used. A Deed is worth one bonus and stays with the item.
- **Compounds:** two words of different classes on one target make a named third effect. There are 12, found by experiment and kept on a new journal page. Three can be made in the Vale.
- **Word clauses:** 12 enchantments that change what one word does. They are the second enchantment on every Illuminated item, and cannot be crafted.
- **Canon touched:** new Compounds section in `docs/bible/08-words-items-craft.md` and new Annotated creatures section in `docs/bible/07-bestiary-and-bosses.md`. No dialogue and no voiced line was changed.
- **Cost:** three new tasks, in Phases 06 and 07, and about fifteen widened ones. Every species now needs three annotations authored and tested.

## 2026-11-10: blade and word

Owner's decision. This replaces the combat and loot items in the entry of 2026-11-09, "combat, loot, and magic follow Skyrim".

- **Magic is a main feature.** A weapon in one hand and a word in the other is the default pairing, given to the player in the prologue, and every fight is tuned for it. Emendation stays the only magic and keeps all 23 words.
- **Blade and word together:** every weapon hit returns Ink, every word has a combat line that opens an enemy to steel, a word can be written on the held weapon by holding both hands, and a tapped word fits between two swings.
- **Melee is shallow.** Tap, one heavy attack, block, and one shield bash. Directional power attacks, the power bash, the timed block, and dual-wield special moves are gone. Six movesets became four: One-handed, Two-handed, Bow, and Shield. A hand holding a word does not block.
- **Loot carries the depth.** Six rarity tiers in place of three: Plain, Fair, Endorsed, Engrossed, Illuminated, and Canonical, shown on the interface as Common to Unique. An item is made of a material, two fixed traits, rolled bonuses, and clauses. There are 16 traits and 20 bonuses, the 30 clauses stay, and looks vary within a material and tier.
- **Growth:** still fourteen skills. Weapon perks add bonuses and never moves, and Emendation's perk tree doubles to about sixteen.
- **Still true:** every fight can be won with steel alone and with words alone. Crafted gear stops at Engrossed.
- **Decided without the owner, to be judged at the Feel Gate:** the three blade-and-word rules beyond Ink on hit, no block while a word is held, the tier names, the trait of each material, the drop rates, and improvement shown as tally marks so that "Legendary" means only a rarity.
- **Canon touched:** the Rarity, Weapon types, Materials, and Item naming sections of `docs/bible/08-words-items-craft.md`. No dialogue and no voiced line was changed.
- **Cost:** three new tasks in Phase 06 and about ten widened ones. Less first-person weapon animation, and more item generation, item art parts, and tuning.

## 2026-11-10: plain first, strange underneath

Owner's decision, to make the game easier to pick up and less abstract.

- **Words are shown as spells.** Whatever a word can change lights up, and the result is previewed in plain words. Classes and resistance tiers no longer appear on screen. The Query sort now reveals what a target answers to.
- **Plain language on the interface:** Magic, Words, Region, Enchantment, Alchemy, Fast travel point. The world's own terms stay in dialogue and books. Marks stay Marks.
- **Quest markers** are on by default, with an option to hide them. Reading is rewarded and never required.
- **Standing Pages are events,** each with a guardian or puzzle and a learning spectacle.
- **Power first:** the first cast is on the ossuary's bronze doors, within three minutes.
- **Taking it back:** what a Redactor struck out returns when it dies. Little Penance does not.
- **An animal companion:** the Blot Hound runt from Tithe Hill.
- **Canon touched:** four stage directions, in Q1, Q7, and Q16. No dialogue and no voiced line was changed.
- **Cost:** one new task and about fifteen widened ones.

## 2026-11-10: physics, ragdolls, destruction, magic effects, and a minimum machine

Owner's decision.

- **Minimum machine:** the owner's Apple M1 with 8 GB. At least 30 fps at 1080p on a new Low preset, and at most 5 GB of memory. Every performance gate is now measured on it, and release builds and full bakes move to CI.
- **Performance techniques:** letters that cost nothing at distance, baked sky occlusion, GPU culling and instancing, cached shadows, froxel fog, dynamic resolution, rationed shadowed lights, and compressed, streamed textures. A texture encoder must be chosen in Phase 03.
- **Animation:** procedural layers, spring-chain secondary motion, inertial blending, and GPU skinning with levels of detail.
- **Physics:** loose and jointed props on `rapier3d`, carried, thrown, and saved where they rest. `HEAVY` and `LIGHT` change real mass.
- **Ragdolls:** on death, dissolving into letters at rest, plus partial-ragdoll hit reactions.
- **Destruction:** pre-fractured props and marked structures. `BROKEN` shatters them and `WHOLE` reassembles them. No free-form destruction of terrain or buildings.
- **Magic effects:** every word's effect is built from five shared parts, including a rewrite wave across the target's own fine print.
- **Cost:** 13 new tasks and many widened ones, mostly in Phases 01, 03, 05, and 06. Total load is higher than before. The techniques pay for the new features and do not raise frame rates beyond the old targets.

## 2026-11-09: a realistic world written in fine print

Owner's decision. This replaces the pixel-art look.

- **Look:** realistic and naturally lit, in the manner of Skyrim. Every surface is still made of ASCII characters, now too small and too many to see from a distance. Letters first show at about 6 m and can be read at about 2 m.
- **Renderer:** material maps and physically based shading, soft shadows, sky ambient, volumetric fog, a filmic tone map, and a colour grade per Quire. Flat cells, restricted palettes, stepped shading, dither, and `^` forests are removed. Letters are prefiltered away below about 3 px, keep each surface's exact average colour, and are cut into surfaces as relief up close.
- **Close reading** is now the way to see the world as text from afar. Coarse letter levels are drawn only there, in the title sequence, in dives through a letter, and in the Erasure.
- **The Erasure** is where the fine print fails and coarsens, so it is the one region whose writing shows from a distance.
- **Signature visuals** were adapted: body language reads within fighting distance, wind writes by bending real grass, and rain, snow, and fog look natural until they are close.
- **Tests:** the pixel-art read test became a realism read test, comparing captures with the letters on and off, plus a reveal test.
- **Risk to judge at the Art Gate:** a realistic look depends on the quality of the approved free models and textures far more than the old look did, and nothing hides weak assets. Performance is also harder to hold on Steam Deck. The Art Gate now asks about both.

## 2026-11-09: nine signature ideas

Owner's decision. Rules are in `docs/brief/5h-signature-mechanics.md` and `docs/brief/2a-signature-visuals.md`, and `docs/SCOPE.md` says where each is built.

- **Close reading:** hold Read to slow time and see the world as raw text.
- **Sorts:** four punctuation modifiers for any word: `!`, `,`, `...`, and `?`.
- **Typos:** 21 words can misfire into a neighbouring word, by accident or on purpose. The table is canon in `docs/bible/08-words-items-craft.md`.
- **The Remembered:** keep three things from each of four lists the Erasure takes, and read them back later.
- **An earned name:** the world writes the player's deeds as epithets.
- **Typographic body language:** creatures show their state in their letters.
- **The Erasure unravels:** text thins in five stages, and the player's footprints print what stood there.
- **Weather as typography:** wind that reads, rain, snow that buries text, and fog that lifts letters out, adapted in the entry above.
- **Travel through a letter:** doors, fast travel, and Torn Pages reuse the title dive. This replaces the ink wash.
- **Cost:** 15 new tasks and several widened ones, mostly in Phases 02, 06, 07, and 09.

## 2026-11-09: combat, loot, and magic follow Skyrim

Owner's decision, replacing the combat, loot, and skill items in the entry below.

- **Combat:** two independent hands, power attacks, blocking, bash, dual wielding, bows, and sneak attacks. Six movesets cover the eight weapon types.
- **Magic:** Emendation stays the only magic and keeps all 23 words and its rules. It is now cast like a Skyrim spell: equipped to a hand, tapped or charged, and doubled with both hands. A Staff holds one word. Typing a word still earns a bonus.
- **Loot:** materials on levelled lists, 30 clauses as enchantments, and the 40 Canonical artefacts. Marginal Notes and the three rarity tiers are gone.
- **Skills and crafts:** fourteen skills with perk trees, and three crafts: Smithing, Inscribing, and Distilling.
- **Dropped:** the rule that combat must serve Emendation, and the test that failed fights which played the same without it. Every fight must now be winnable with steel alone and with words.
- **Cost:** more first-person animation in Phases 06 to 08. Phase 06 remains the Feel Gate.

## 2026-11-09: refocused on three pillars and staged releases

- **Pillars:** Read, Rewrite, and Wander now decide what is built. See `docs/brief/0a-pillars-and-releases.md`.
- **Emendation proven early:** a new Phase 02 proves it in one room before the world is built, and ends in a new Hook Gate.
- **Combat, loot, and skills:** shrunk around Emendation in this entry, then replaced by the entry above. Standing stones are folded into Standing Pages. All 40 Canonical items stay.
- **Bosses:** all 30 stay. 14 are Full and 16 are Light, with one phase each.
- **Scope:** the canon is unchanged and is now treated as a catalogue. `docs/SCOPE.md` records what is built in full, built light, or held in reserve: 28 of 40 dungeons, 110 of 200 interiors, 100 of 120 points of interest, 36 of 50 side quests, 24 of 40 secrets, 60 of 100 small tasks, 21 of 24 species, and 90 of 150 named characters with full dialogue. All 100 books, the main story, the questlines, and the endings stay.
- **Releases:** Release 1 is Scrivenhold Vale with Q1 to Q4, as early access. One update per Quire follows in story order, then the endings and 1.0.
- **Phases:** 21 in place of 12. Phase 09's 65 region tasks became one phase per Quire, each with its own Update Gate. Every gate now lists what the owner checks and what counts as a pass.
- **Unchanged:** co-op, the renderer, the technology, the platforms, and the voice plan.

## 2026-11-09: voice added on a fixed budget

- Budget: 40,000 ElevenLabs credits and 10 voice slots, on Eleven v4.
- All written quests remain. Voice covers 263 lines and 13,705 characters: every key story scene, plus selected follower, faction, local, and crowd lines.
- Ten base voices become 43 named character presets through the game's own audio processing.
- The Ledger and the player stay unvoiced.

## 2026-11-09: canon written

- Added the full world bible in `docs/bible/`, indexed by `docs/WORLD_BIBLE.md`.
- Added the story script and all quests in `docs/script/`.
- The Great Ledger's shortage, the Authors' true origin, and the Clerk's self-erasure are now canon.
- Phases now list the canon each one needs.

## 2026-11-09: starting plan

- **Title:** Vellmark.
- **Look:** every pixel is a letter, blending lantern-lit 3D pixel art with readable text. Project Shadowglass is a visual inspiration only.
- **Scope:** scaled down to a 36 km² realm of seven Quires and one small mirror world, so the game is buildable with Claude Code.
- **Story:** three main endings and a secret fourth, with an intro, outros, and an epilogue generated from the save.
- **Play:** first-person, with drop-in two-player local co-op and full controller support.
- **Platforms and stores:** Windows, macOS (Apple Silicon), Linux, and Steam Deck, sold on Steam and itch.io.
- **Technology:** Rust, SDL3, and wgpu, with no game engine.
- **Assets:** approved free CC0 assets. Free AI-generated assets are allowed only after confirmed quality, confirmed legality, and owner approval.
- **Not included:** mod support, localization (English only), telemetry, and online services beyond Steam's own.
- **Build method:** 12 resumable phases, with owner gates for art, game feel, the first complete region, and publishing.
