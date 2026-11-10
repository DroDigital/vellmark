# Section 5: local co-op

### Seamless local co-op

The entire game, including the main quest, is playable by two people on one machine. Design for this from Phase 00, since it cannot be bolted on later. All rendering, input, UI, audio listeners, streaming, and AI code must accept a list of players and views, with no global "the player" or "the camera".

**Drop-in, drop-out.** While playing, a second person presses Start on any unassigned controller, or the join key on an unassigned keyboard, and is in the game within a few seconds. The screen animates into a split without a menu, a reload, or a pause. The new player chooses a saved character or takes a ready-made Unentered matched to the host's level, which they can customize later in their own viewport, and appears beside the host, emerging from a swirl of letters. Leaving reverses this just as smoothly. Joining is available once the prologue ends, and is paused during the intro and outros. The host's save owns the world, and the second player's character is saved to both the world and their profile. Either player's device may disconnect or reconnect without disrupting the other.

**Free roaming.** There is no leash and no shared camera, since each player has their own first-person view. Players may be on opposite sides of the map, or one in a dungeon and one outdoors. Streaming, shadows, culling, and level of detail run independently per view, and NPC simulation stays at full detail around either player. Interior transitions affect only the player entering.

**Split-screen presentation.** Vertical split by default, with a horizontal option. The divider is a fine typeset column of characters. Glyph cascades are computed in each view's own pixels so the look holds up at any viewport size. Each view has its own HUD, compass, and reading mode. Menus open within that player's viewport.

**Two real characters.** Each player has their own character, skills, perks, inventory, and appearance. A character can be saved to a local profile and brought into another world. World state, quest progress, faction standing, and Emendation changes are shared.

**Co-op rules.**

- Menus pause the game in solo play. In co-op the world keeps running, and a player in a menu is "in the margin": they take greatly reduced damage and enemies prefer other targets.
- Either player can start a conversation, and the other can join in and hear it with subtitles. Major quest choices can be made by whoever started the conversation, with a visible prompt for the partner to weigh in.
- Sleeping, waiting, and fast travel require both players to confirm. If only one player is nearby, a fast-travel confirmation pulls the partner along.
- A player at zero health is downed and can be revived by their partner or recovers after a timer. The game ends only if both are down.
- Enemy difficulty, loot, and encounter sizes scale with player count. Loot is per player so nobody fights over it.
- Audio is mixed for two listeners, prioritizing what is happening near each player and ducking redundant sound.
- Combination Emendations reward teamwork: one player writes `DRY` on a target while the other writes `BURNING`.
