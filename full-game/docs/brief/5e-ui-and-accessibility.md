# Section 5: UI and accessibility

**UI.** Everything is typeset on the same glyph system and designed like a beautiful old book. The compass is a line of text. The map unfolds as an engraved chart where terrain is written in characters. The HUD shows what each hand holds, and Ink beside the hand that writes. No health bars on enemies: read how many letters are left on them, and see an eroding name when you target them. All HUD and menu text must be at least 12 px tall at 1280x800, including in split-screen. In-world surface text is exempt, because reading mode enlarges it.

**Plain language.** The HUD, menus, prompts, and tutorials use everyday words. The world's own terms appear in dialogue, books, and item lore, and never where a player needs to understand a control.

| In the world | On the interface |
|---|---|
| Ink | Magic |
| Emendation | Words, as in "Words" for the skill and "Equip word" for the action |
| Standing Page | Word stone |
| Quire | Region |
| Clause, word clause | Enchantment, word enchantment |
| Annotated | Elite |
| Compound | Combination |
| Plain, Fair, Endorsed, Engrossed, Illuminated, Canonical | Common, Uncommon, Rare, Epic, Legendary, Unique |
| Inscribing, Distilling, Lockwork | Enchanting, Alchemy, Lockpicking |
| Sort | Punctuation |
| Bookmark | Fast travel point |
| Commonplace book | Saved lines |
| Close reading | Focus |

Marks stay Marks, since a currency may have a name. Place names, people, and the Unentered keep theirs.

**The character in the inventory.** Loot is worn, so it must be seen. The inventory shows the player's character in full, lit as the place they are standing in, turning under the stick or the mouse. Equipping anything changes it at once, weapons and words show in the hands, and aiming at an item in a shop or a chest previews it on the character before it is bought or taken. It uses the full-body model a co-op partner already sees, so it needs no new art.

**Photo mode.** The look is the reason people will stop scrolling, so let players show it.

- Opens from the pause menu and from one button, fully usable on a controller. In solo play the world freezes. In co-op it opens in that player's viewport, the world keeps running, and the player is in the margin.
- A free camera within a few metres of the player, with field of view, tilt, exposure, depth of field, and switches to hide the HUD, the player's hands, and the partner.
- The player's own character can be shown in full, in everything they are wearing, with a small set of poses and a word held in the hand. Play stays first-person. This is where a player sees what they look like in the world.
- **Text view** shows the frame as raw text, as close reading does. **Split frame** divides it, realistic on one side and text on the other, with a divider that can be moved and turned. This is the picture that explains the game.
- Saves a PNG at the display's resolution to a folder the settings name, and to Steam's screenshot library in the Steam build. Nothing is uploaded anywhere by the game.
- It never shows anything Sealed or hidden that the player has not found, and it adds nothing the minimum machine cannot afford: one frame is rendered and held.

**Commonplace book.** The player keeps the lines they liked.

- In reading mode, one button copies the focused line to the journal's **Commonplace** page, with where it was found. Any text can be kept: walls, books, blades, creatures, and people.
- A kept line can be exported as a text file, or as a **page**: a typeset PNG of the line with its source and the game's title, made to be shared.
- It is a keepsake and never a mechanic. Nothing reads it, counts it, or rewards it.

**Accessibility.** Build these along with the UI, not at the end:

- A language setting, changeable at any time without a restart
- Letter-size slider, UI scale, high-contrast mode, and a reduced letter-noise mode
- Reading mode that shows any focused text in large, clean type, on keyboard and controller
- Colour-blind palette options. Rarity, faction, and danger must never rely on colour alone
- Remappable controls, hold or toggle options, and adjustable aim assist
- Reduced motion, reduced screen response, and a photosensitivity option limiting flashes and flicker. Reduced motion turns the dives through letters into fades
- Options to hold or toggle close reading, to turn off its slowed time, and to turn off HUD thinning in the Erasure
- A setting that states every creature's state in words beside its Inscription, for players who cannot read it from its letters
- Subtitles, captions for important sounds, and separate volume sliders
