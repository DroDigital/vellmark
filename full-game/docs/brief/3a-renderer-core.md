# Section 3: renderer core (3.1-3.7, 3.10, 3.11)

## 3. The core tech: Glyph Cascade renderer

Build a forward+ HDR renderer with depth pre-pass, clustered lights, soft cascaded sun shadows, ambient occlusion, and 4x MSAA for geometry edges, shading surfaces realistically as set out in 3.11. Do not use TAA. Glyph coverage is antialiased analytically in the shader.

The renderer has two jobs. From a distance it draws a naturally lit world with no sign of text. Near the camera it reveals that every surface is made of fine print. The second must never disturb the first.

Every opaque material calls one shared WGSL function, `glyph_surface()`.

### 3.1 Anchoring

Glyphs belong to surfaces, never to the screen.

- **Architecture and props:** per-vertex 2D anchor coordinates in metres, emitted by the mesh builders and the import pipeline with reading direction upright.
- **Terrain and rock:** projection chosen per cell, hard-selected by dominant axis. Never blend projections, since that doubles the glyphs.
- **Skinned or moving meshes:** anchor coordinates come from the rest pose so letters ride on the body and never swim.
- **Precision:** use camera-relative rendering and chunk-local anchors with integer cell offsets. Text at the far edge of the map must be as stable as text at the origin.

### 3.2 Cascade levels

Cells are square. Each glyph is centred in its cell, including those from proportional typefaces. Cell size at level `k` is `S_k = 0.0064 m * 4^k`, for `k = 0..5` (0.64 cm, 2.56 cm, 10 cm, 41 cm, 1.6 m, 6.6 m).

- **Fine levels (0 and 1)** are the fine print, and are the only levels drawn in normal play.
- **Coarse levels (2 to 5)** are drawn only in close reading, in the title sequence, in dives through a letter, in the Erasure, and by authored effects.

Compute on-screen cell height `h_k` from screen-space derivatives of the anchor coordinate. Each level has an influence `w(h)` that varies smoothly with `log(h)`:

- zero below about 3 px, where the cell must be indistinguishable from the plain surface
- rising to full by about 8 px
- full through about 40 px
- fading to zero by about 160 px

**Exception:** level 0 never fades out as it grows, so getting close makes the finest text larger and readable. Crossfades must be continuous with no visible popping.

With the default field of view at 1080p, this puts the first trace of letters at about 6 m and legible characters at about 2 m. Scale the thresholds with the height of each player's view, and expose them as the reveal distance.

At grazing angles, use an estimate biased against aliasing, and fade contrast when a cell's minor axis falls below about 3 px.

### 3.3 Letters made of letters

Parent glyph coverage modulates its children at every pixel, so a 2.56 cm character is visibly constructed from 0.64 cm characters inside its strokes. In close reading and in the title sequence the same rule runs through every level, so approaching a cliff feels like an infinite zoom through text. This is the signature effect: make it gorgeous.

### 3.4 Stable glyph identity

This is the most important rule in the renderer.

- **Which character a cell shows** depends only on stable data: cell ID, cascade level, material, and low-frequency albedo or authored text. It never depends on lighting or camera position.
- **Lighting is computed per pixel,** as for any realistic surface, and is never quantised to cells.
- **How heavy a character is** follows the surface's own local tone, by shifting the SDF threshold, so darker patches of a material are written in heavier strokes. Moving shadows never make characters change identity.

### 3.5 Tone and colour preservation

The letters must be invisible in aggregate.

- Each material's letters are drawn in an **ink** darker than its albedo on a **paper** lighter than it, chosen so that a cell's average equals the albedo exactly.
- Bake a per-glyph lookup from target tone to SDF threshold, so mean coverage matches target tone. Where a glyph cannot reach the required coverage, the remainder goes into ink and paper tint.
- Each level contributes zero-mean modulation around the source tone, so mean brightness and hue do not change as letters fade in and out with distance.
- Up close, glyph coverage also perturbs the surface normal slightly, cutting letters into hard materials and raising them on soft ones. This relief fades out with the letters.

### 3.6 Font

Use the two approved typefaces in Section 1.1. The bake tool converts all 95 characters of each into a multi-channel SDF texture array with enough resolution for full-screen close-ups. Small cells use the pixel typeface, which stays legible at a few pixels, and large cells use the printed typeface, crossfading through the cascade. Legibility at 6 to 8 px is a hard requirement, and the characters must look beautiful at 300 px.

### 3.7 The Three Layers of Text

1. **Grain:** material glyph families at all scales.
2. **Generated truth:** surfaces of notable entities carry text generated from real game data. An NPC's robe contains their actual biography, schedule, and faction. A tree's bark records its age. A sword's blade lists who it has killed.
3. **Authored lore:** hand-written text on ruins, tombs, and walls in level 0. Some of it holds quest clues, passwords, and secret locations. A higher Perception skill reveals additional hidden lines.

Text runs in reading order across cells and wraps sensibly around architecture. Baked text pages stream with their world chunks. Generated text lives in a small GPU buffer per nearby entity, is rebuilt only when its data changes, and has a fixed memory budget. A cell's character is read from these pages by cell ID. Identity may change only when the underlying text actually changes, as when a sword records a new kill.


### 3.10 Performance targets

- **Minimum machine: an Apple M1 with 8 GB of memory.** At least 30 fps at 1080p on the Low preset, solo, with dynamic resolution allowed down to 70%, and at most 5 GB of memory in use. This is the owner's machine, so every performance gate is run on it.
- 60 fps at 1440p on an RTX 3060 or Apple M2-class GPU.
- 60 fps at 1080p with reduced settings on integrated graphics of recent years.
- Steam Deck: stable 40 fps at 1280x800 solo and stable 30 fps in split-screen, with a dedicated preset that is chosen automatically.
- No streaming hitch over 8 ms in a traversal test.
- 60 fps at 1440p in two-player split-screen on the RTX 3060 / M2 reference hardware, with the players far apart. Per-view budgets for grass, shadows, and far-field detail may scale down, but the look must hold up.
- Memory: at most 8 GB of RAM and 4 GB of GPU memory on the Steam Deck preset. Install size: at most 15 GB.
- Keep per-pass GPU timings and fail a gate if a budget is exceeded.

### 3.11 The realistic frame

This is what makes the world read as a real place from any distance beyond the reveal distance. The bar is the look of Skyrim: natural proportions, textured and normal-mapped surfaces, and above all good light and atmosphere. It is not the fidelity of current big-budget games.

- **Materials.** Every surface has albedo, normal, and roughness maps from the approved texture sources, with a simple physically based shading model. Terrain blends materials by slope, height, and biome.
- **Light.** A sun with soft, filtered cascaded shadows. Ambient light from the sky, shaped by ambient occlusion. Clustered point and spot lights with soft falloff for fires, lanterns, and windows.
- **Atmosphere.** Physically based sky, volumetric fog, god rays, and aerial perspective, so distance is soft and deep.
- **Colour.** HDR with a filmic tone map, restrained bloom, and one colour grade per Quire and time of day, blended as the player travels and as the day turns.
- **Edges.** 4x MSAA with alpha-to-coverage for foliage, and a spatial edge filter. No temporal accumulation, so close type stays crisp.
- **Level of detail.** Conventional mesh levels and impostors for trees and distant buildings, crossfaded without popping. Far forms keep true silhouettes.
- **The letters stay out of it.** Beyond the reveal distance the glyph layer contributes nothing. A capture taken with the glyph layer switched off must match one taken with it on, outside the reveal distance, within a small threshold.
- **Close reading.** While it is held, blend from the realistic frame to a text frame: the coarse cascade levels switch on across the whole view, colour drains toward each material's ink and paper, and lighting is kept only as ink weight.

### 3.12 Performance techniques

Each of these makes the frame look better or cost less, and most do both. Build them as part of the renderer, not as late optimisation.

- **The fine print is free at distance.** `glyph_surface()` exits before sampling any glyph when every active level is below 3 px, so only pixels near the camera pay for letters.
- **Baked sky occlusion.** The bake computes, for every surface, how much of the sky it can see and from which direction. Ambient light uses this at run time, with screen-space occlusion adding only fine contact detail at half resolution.
- **GPU culling and instancing.** Draw everything through instanced batches. A compute pass culls instances against the view and a depth pyramid from the previous frame, per view.
- **Cached shadows.** Near cascades update every frame. Far cascades update on a rota, and static geometry is drawn into them once and kept until the sun has moved enough to matter.
- **Froxel fog.** Volumetric fog and god rays are computed on a coarse view-space grid and upsampled.
- **Dynamic resolution.** The 3D scene renders at 70% to 100% of the view's resolution to hold the frame budget, and is upscaled with a sharpening spatial filter. The HUD is always drawn at full resolution, and the scale rises to 100% whenever readable text fills much of the view.
- **Shadowed lights are rationed.** Only the few nearest fires and lanterns cast shadows. The rest light without them.
- **Compressed, streamed textures.** Textures are stored block-compressed for each platform and streamed by distance under a fixed memory budget, with the lowest level always resident.
- **Animation and physics levels of detail.** See Sections 5i and 7.
- **Presets.** Low, Medium, High, and Steam Deck. Low is tuned on the minimum machine, and is chosen automatically there.
