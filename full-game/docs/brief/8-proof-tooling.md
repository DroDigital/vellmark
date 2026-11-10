# Section 8: proof tooling

## 8. Proof tooling (build this first)

You must be able to see your work. Build the capture tool before any graphics feature, build every other tool before the feature it proves, and use them at every step.

**Developer tools.** Dev builds include a command console, free camera, teleport, spawn commands, quest-state controls, and overlays for cells, the reveal distance, and streaming, and a switch for the glyph layer. None of these may exist in shipping builds.

**What you cannot verify yourself.** You cannot judge combat feel, co-op comfort, real controllers, Steam Deck behaviour, real-GPU performance on other machines, or Remote Play. Never mark these as passed from automated results alone. Maintain `docs/PLAYTEST_CHECKLIST.md` with exact steps for me to run, and a one-command performance capture that I can run on my hardware and send back.

- `cargo run --release -- --capture <scene.ron>`: deterministic, fixed-timestep rendering of scripted camera shots to PNG, with no window required where the platform allows. **Open and inspect the images yourself.**
- **The Approach:** a shot sequence of one cliff at 2000, 500, 100, 20, 5, 1, and 0.3 m, plus a 10-second video-style frame dump. It is the primary art benchmark.
- **License audit:** every shipped asset matches a manifest entry with an allowed license, every AI asset has complete provenance and my approval, and `cargo-deny` passes.
- **Realism read test:** captures beyond the reveal distance must pass review as a conventionally rendered game: natural light, no visible grid, grain, or moiré. Each is compared with the same capture taken with the glyph layer off, and must match within a small threshold.
- **Reveal test:** along The Approach, letters must appear smoothly inside the reveal distance, be legible at 2 m, and be crisp at 0.3 m.
- **Temporal stability test:** two frames with a tiny camera move, reprojected and differenced. Fails if identity flicker exceeds a threshold.
- **Tone test:** mean luminance stays within 3% across a full cascade sweep.
- **Performance capture:** per-pass GPU timing and frame-time percentiles to JSON along a scripted traversal.
- **Bot playthrough:** a headless logic harness that runs each quest to completion through real game systems and fails on a dead end.
- **Content validator:** run on every bake. Fails on broken doors and references, unreachable items or NPCs, quests with no valid path, invalid schedules, missing navigation meshes, a location with no authored identity, or a waiting place that breaks the rules in `4c-content-rules.md`.
- **World soak test:** a bot population simulates 100 in-game days at high speed and fails if NPCs get stuck, schedules break, save size grows without limit, or memory leaks.
- **Density map:** a report showing points of interest per square kilometre and walking time between them, used to find empty regions.
- **Critical-path test:** a bot finishes the main quest while skipping all optional content, and another completes every questline in a single save to catch conflicts.
- **Ending test:** bots reach every ending, solo and in co-op, and verify each outro and epilogue against the save.
- **Variety report:** lists enemies, bosses, and items that are too similar in stats, silhouette, or behaviour.
- **Loot simulation:** runs millions of drops to check the rates of the six rarity tiers by source, power curves across materials and tiers, and broken combinations of traits, bonuses, clauses, word clauses, and Deeds.
- **Hook room:** a scripted run through the Phase 02 test room, solo and with two views, that reads a wall, equips a word, opens a lock, lights a lamp, starts a chain reaction, and wins a fight three times: with blade and word together, with a word alone, and with steel alone. A second run takes all three builds through the arena, where an annotation is rewritten, a compound fires, and a dropped weapon is equipped. Both stay in the suite for the whole project.
- **Emendation matrix:** every word against every target class and every species, generated from the word sheets and checked against them. It includes every typo, every sort, every compound, and every annotation a species supports. Fails on a missing result or refusal line, and reports any species that answers fewer than two words in its own way.
- **Scope check:** fails if built content references a Reserve entry in `docs/SCOPE.md`, or if a catalogue entry has no tier.
- **Combat simulation:** tests each enemy and boss against builds at each level and flags fights that are impossible or trivial. Each fight is tuned against a blade-and-word build, and is also run with a steel-only build and a word-only build. Groups are also run with each annotation their species supports. A fight that either of those cannot win is flagged, and so is a fight where steel alone does as well as blade and word together.
- **Minimum machine capture:** the one-command performance capture, run by the owner on the M1 with 8 GB at each gate. It records frame-time percentiles, dynamic resolution, and peak memory, and is the only accepted evidence for the minimum target.
- **Physics soak:** a scripted scene that breaks, throws, kills, and repairs far past the budgets in Section 5i, and fails if the limits are exceeded, the physics step runs over, or anything is left that cannot be saved.
- **Destruction test:** every marked structure is broken and repaired by a bot, saved and loaded in both states, and checked by the validator for blocked paths.
- **Save test:** save, load, and compare state across every phase, and load a save from every earlier release.
- **Co-op test:** a scripted session with two simulated players that joins and leaves repeatedly, splits up across the map, enters interiors separately, completes a quest together, and saves and loads. Also captures both views.
- **Input test:** a simulated device layer that replays hot-plugs, disconnects, and full controller-only runs through menus and quests.
- **Platform check:** CI publishes capture images, test results, and packaged builds for all three operating systems.
- **Bake determinism test:** two clean bakes produce byte-identical packs, and changing one asset changes only the chunks around that asset.
- **Stutter test:** the scripted traversal fails if any render pipeline is created during gameplay.
- **Storefront test:** the Steam and itch builds both pass the bot runs, and the itch build contains no Steam library.
- **Deck test:** run the 1280x800 preset, UI size checks, and controller-only run, and write a checklist against Valve's current Verified criteria.
