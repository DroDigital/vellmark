# Section 5: input and controllers

### Input and controllers

Every feature in the game must be fully playable on a controller alone, from the title screen to the ending. Keyboard and mouse must be equally polished.

- Support Xbox, PlayStation, Switch Pro, Steam Deck, Steam Controller, 8BitDo, and generic controllers through SDL3's gamepad API on all three operating systems.
- The game must work with Steam Input turned on or off. Build one action layer so game code never reads a raw button.
- Each hand has its own button: by default the two triggers on a controller and the two mouse buttons. Tap and hold are the only melee inputs, and the sidestep has a button of its own. Holding both hands' buttons writes the word on the held weapon, or writes one word with both hands when both hold it.
- Meet the Steam Deck Verified requirements: the default controls reach all content with no settings changes, and no keyboard or mouse prompts appear during controller play.
- Unknown controllers get an in-game mapping wizard, and mappings are saved per device.
- Hot-plugging must work at any moment. If a controller disconnects mid-game, only that player gets a reconnect prompt.
- Button prompts update instantly to match the device last used, including Deck and Steam Controller layouts, and they are ASCII text such as `[A]`, `[CROSS]`, or `[LB]`.
- Per-player remapping, sensitivity, dead zones, invert axes, aim acceleration, toggle or hold options, and rumble where hardware supports it.
- Gentle aim assist for controller players, adjustable and optional.
- Any device can be assigned to either player, including keyboard and mouse for one player and a controller for the other.
- Text entry on controller uses the letter wheel, including for naming characters and saves.
- Remote Play Together is enabled through the store page's local co-op tags. Controllers from remote guests must be able to join as Player 2 like any other device. Verify this in a real Steam session, because a guest's keyboard and mouse may not arrive as a separate device.
