# The graphics stack, and how it stays smooth

This file sets the Showcase's technical targets and the order in which the renderer is built. The detailed rules are in the full-game brief and are not repeated here. Read the brief file a section names before building that part.

| Subject | Brief file in `full-game/docs/brief/` |
|---|---|
| Stack, crates, threads, platforms, pack files | `1a-core-constraints.md` |
| Approved assets, fonts, and libraries | `1.1a`, `1.1b`, and `1.1c` |
| What the game must look like | `2-art-direction.md`, `2a-signature-visuals.md` |
| The Glyph Cascade renderer | `3a-renderer-core.md` |
| Grass, trees, water, sky, damage, magic effects | `3b-renderer-surfaces.md` |
| Physics, ragdolls, destruction | `5i-physics-and-destruction.md` |
| Characters and animation | `7-characters-and-animation.md` |

Where this file and the brief disagree, this file wins for the Showcase, and the difference is listed under Differences at the end.

## The two promises

1. **Beyond the reveal distance it is a real place.** No grid, no grain, no shimmer, no sign of text. A capture with the glyph layer off must match one with it on, outside the reveal distance, within a small threshold.
2. **Within it, everything is written.** Letters stay fixed to their surfaces, stay sharp, never swim, never pop, and can be read at arm's length.

Every decision below serves one or both.

## Targets

All measured on the owner's Apple M1 with 8 GB, at 1920 by 1080, on the Medium preset, by the performance run described below.

| Measure | Target |
|---|---|
| Average frame rate | 60 frames a second or better |
| Slowest 1% of frames | No longer than 20 ms |
| Longest single frame after loading | No longer than 33 ms |
| Scene resolution | Never below 85% of the display |
| Memory in use | No more than 4 GB |
| Time from launch to the title | No more than 8 seconds |
| Time from the title to play | No more than 10 seconds |
| Pipelines created during play | None |

Low exists for weaker machines and must hold 60 on the M1 with room to spare. High is for stronger machines and is judged by eye and by CI captures.

If 60 cannot be held at the gate, the owner is shown the measured numbers and the three cheapest changes that would reach it. Never lower the target silently, and never meet it by breaking one of the two promises.

## The frame budget

16.6 ms a frame. Each pass has a share, measured by GPU timers that are always on in development builds. A pass over its share fails the nightly check.

| Pass | Share on the GPU |
|---|---|
| Depth pre-pass | 1.0 ms |
| Culling compute and depth pyramid | 0.4 ms |
| Sun shadows, with cached far cascades | 2.0 ms |
| Opaque surfaces, with `glyph_surface()` | 5.0 ms |
| Grass and foliage | 2.0 ms |
| Water and other transparent surfaces | 0.9 ms |
| Fog and god rays | 1.0 ms |
| Particles, including flying words | 0.7 ms |
| Contact occlusion at half resolution | 0.6 ms |
| Tone map, bloom, edge filter, upscale | 0.8 ms |
| Interface | 0.4 ms |
| Unspent | 1.8 ms |

| Work on the processor, per frame | Share |
|---|---|
| Simulation step, with AI | 2.0 ms |
| Physics | 2.0 ms at worst |
| Animation | 1.5 ms |
| Building and submitting the frame | 2.5 ms |
| Audio | Its own thread. The callback never allocates or locks |

## One governor

A single controller holds the frame time. Nothing else is allowed to trade quality for speed on its own.

- It reads the GPU timers and the frame time every frame, smoothed over half a second.
- When the frame runs long, it gives things up in this order, and takes them back in reverse: far shadow cascade update rate, particle counts, grass density beyond 30 m, fog grid resolution, contact occlusion, and last the scene resolution, down to the 85% floor.
- It never touches anything inside the reveal distance. Letters near the player are the last thing to lose quality, and the scene resolution returns to 100% whenever readable text fills much of the view.
- It changes one thing a little at a time, so nothing is seen to switch.
- Its decisions are logged in every performance run, so a number that was met by giving something up is visible.

## Shared work

The systems are built to feed each other. Each of these is done once and used several times.

| Done once | Used by |
|---|---|
| The depth pyramid from the last frame | Culling, contact occlusion, fog, particle collision |
| Sky visibility baked for every surface | Ambient light, fog colour, where rain falls, where snow would settle |
| One glyph atlas for both typefaces | Every surface, the interface, flying words, damage letters |
| Text pages by cell | Fine print, reading mode, generated truth, body language, the rewrite wave |
| One GPU particle system | Flying words, rain, embers, smoke, letters knocked from a body |
| One instance buffer, culled in compute | Buildings, props, trees, grass, creatures |
| The word sheets | Effects on targets, the five-part visual effect, the motif, the preview text, the tests |
| The simulation's fixed snapshot | Drawing between steps, the bots, replays of a failed test |
| The anchor coordinates | Letter placement, the rewrite wave, fracture lines, damage masks |

## The order of building

Each stage is judged by a capture before the next begins. The stages are spread across the phases in `phases/`.

1. **A lit wall.** Forward+ HDR, depth pre-pass, 4x MSAA, one sun with soft shadows, one material with albedo, normal, and roughness. It must already look real.
2. **Letters on the wall.** Anchoring, cascade levels 0 and 1, stable identity, tone preservation, relief, and the early exit. The glyph-off comparison passes from this day on and never stops passing.
3. **Letters made of letters.** Coarse levels, close reading, and the dive.
4. **A yard.** Several materials, masonry text that wraps, a door, a lantern, clustered lights, and the first authored text.
5. **Sky and air.** Physically based sky, time of day, froxel fog, god rays, and the colour grade.
6. **Ground and growth.** Terrain with blended materials, grass, trees with far forms, wind.
7. **Water and weather.** The river and pond, rain, and the wind that reads.
8. **Bodies.** Skinned characters with anchored letters, damage masks, body language.
9. **Words.** The five-part effect, the rewrite wave, lasting states.
10. **The governor and the presets.**

## Tools that keep it honest

Build these in Phase S00, before the renderer, because every later task is checked by them.

- **Hot reload** of shaders, data sheets, and text, in development builds, in under a second.
- **The capture tool:** a scripted camera path and a fixed clock produce identical frames every time. It writes single images, contact sheets, and the glyph-on, glyph-off pair.
- **Reference images:** a small set of captures is kept in the repository at low resolution. A change that moves a reference by more than a threshold fails until someone has looked at it and accepted it.
- **The performance run:** one command on the owner's Mac flies the tour's camera path twice and plays the bot through the fights once. It writes a report with every number in the Targets table, the per-pass timings, and the governor's log.
- **The shimmer check:** a slow sideways camera move across a far field and a near wall, with frame-to-frame differences measured. This catches moiré and swimming letters, which a still image cannot.
- **A profiler view** in development builds, with the per-pass bars drawn in the interface's own letters.

## Building in the cloud

Cloud sessions have no real graphics card, so the two kinds of evidence are kept apart.

- **In a cloud session:** build, run the headless simulation and bots, and render captures with software rendering at low resolution for the reference images, the glyph-off comparison, and the shimmer check. These prove correctness and never speed.
- **On the owner's Mac:** the performance run, and every judgement of how it looks and feels. CI builds the macOS app on every push to the main branch, and the owner downloads it.
- No performance target may be marked as met from a cloud session or a CI runner.

How sessions are set up and run side by side is in `docs/CLOUD.md`.

## Differences from the full-game brief

| Subject | The brief | The Showcase |
|---|---|---|
| Minimum machine target | 30 frames a second on Low, resolution down to 70% | 60 on Medium, resolution never below 85% |
| World | Streamed in tiles | One area loaded whole. The tile format is still used, so streaming is added later without rebaking |
| Views | Two, for split-screen | The renderer supports a list of views from the first day, and one is drawn |
| Texture streaming | By distance, under a budget | Everything resident, under the 4 GB limit |
| Visible grass | Over 300,000 blades | 150,000 on Medium on the M1, more on High |
| Platforms | All three are release targets | All three build and run. Only macOS is tuned for the gate |
