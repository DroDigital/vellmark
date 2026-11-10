# Phase 03: Terrain, light, and streaming

**Goal:** A streamable 6 km realm under final lighting, with the Vale shaped in full and every other Quire true at a distance.

**Suggested model:** Opus 5.5, high effort. Hard, well-understood graphics and streaming work, with performance measured on the minimum machine.

**Hand to Haiku 5.5:** Biome and road data files, capture scripts, and reading long bake logs for failures. Rules are in `CLAUDE.md` under Models and helpers.

**Release:** Release 1

**Requires:** Phase 02 finished and tagged `phase-02-done`.

## Read for this phase

Read only these files in `docs/brief/`, and only the named part where one is given.

- `2-art-direction.md`: Beauty rules
- `3a-renderer-core.md`: 3.1, 3.10, and 3.11, and 3.12
- `3b-renderer-surfaces.md`: Sky and Fog
- `4a-world.md`
- `1.1b-asset-catalog.md`: Textures and Shader references
- `1.1c-libraries-and-import.md`: Texture compression

## Canon for this phase

These files are under `docs/`.

- bible/04-quires-and-places.md: the first paragraph of each Quire, and all of Scrivenhold Vale

## Scope

Lighting, atmosphere, terrain tool, clipmaps, streaming, the whole realm as a heightfield with far forms, Scrivenhold Vale's terrain, biomes, and roads in full, and the Vale's colour grades

## Tasks

Do these in order, one at a time, following the task loop in `CLAUDE.md`.

- [ ] **P03-T01** Build the deterministic terrain bake for the whole realm: heightfield, hydraulic and thermal erosion
- [ ] **P03-T02** Generate rivers, coast, biome masks, and Quire boundaries for the whole realm
- [ ] **P03-T03** Add landmark and road data files with stable IDs and terrain-shaping overrides, and shape the Vale in full
- [ ] **P03-T04** Shape each other Quire's skyline and far forms only, so every vista from the Vale is true
- [ ] **P03-T05** Implement clipmap terrain rendering
- [ ] **P03-T05a** Add instanced batching and the compute culling pass against the view and the depth pyramid, per view
- [ ] **P03-T06** Implement async chunk streaming per view, and pass the hitch test
- [ ] **P03-T07** Apply terrain materials with blended albedo, normal, and roughness maps, their glyph families, and texture-derived per-cell glyph data
- [ ] **P03-T07a** Choose and record the texture encoder, then add block-compressed textures per platform and distance-based texture streaming under a fixed memory budget
- [ ] **P03-T08** Implement full soft cascaded shadows and clustered lights, with far cascades updated on a rota, static geometry cached, and only the nearest few lights casting shadows
- [ ] **P03-T09** Port the atmosphere, then add fog, god rays, and the day-night cycle, with fog and god rays computed on a coarse froxel grid
- [ ] **P03-T10** Port ambient occlusion, and bake sky occlusion for terrain and static geometry so ambient light needs only half-resolution contact detail at run time
- [ ] **P03-T11** Create the Vale's colour grade by time of day and the distant look of each other Quire, and review with contact sheets
- [ ] **P03-T12** Add the long-range vista shot from Tithe Hill and a free-camera traversal capture across the whole realm, and have the owner run the minimum machine capture along it
- [ ] **P03-T13** Run the exit gate and save evidence

## Exit gate

The long-range vista from Tithe Hill and the whole-realm traversal both hit budget, including at least 30 fps and at most 5 GB on the owner's M1

Save the evidence in `docs/evidence/P03/`.

## Next

Move to `phases/P04-living-nature.md`.
