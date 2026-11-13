# Phase S02: The ground

**Goal:** A valley you can stand in: sky, light, air, earth, grass, trees, water, and weather, at the frame rate the Showcase promises.

**Suggested model:** Opus 5.5, high effort. Renderer features the brief already specifies closely.

**Hand to Haiku 5.5:** Plant and prop placement data, wind and weather tables, asset import records, and reading long bake logs for failures. Rules are in `CLAUDE.md` under Models and helpers.

## Read first

- `docs/TECH.md`: Frame budget, One governor, Shared work, stages 5 to 7
- `full-game/docs/brief/3a-renderer-core.md`: 3.11 and 3.12
- `full-game/docs/brief/3b-renderer-surfaces.md`: 3.8
- `full-game/docs/brief/2a-signature-visuals.md`: Weather as typography
- `docs/SHOWCASE.md`: The place

## In this phase

The 500 m area as terrain with its far ring, the sky and time of day, fog, grass, trees, the river and pond, rain, and wind. The governor. No buildings or people yet.

## Tasks

Do the tasks of one lane in order, following the task loop in `CLAUDE.md`. Lanes run side by side as `docs/CLOUD.md` sets out. A task marked **after** waits for the task it names.

- [x] **S02-T01** [Content] Build the terrain for the Showcase area as tiles in the full game's format: the hill, the road, the green, the river, and the far bank, with the far ring and the white band as distant forms
- [x] **S02-T02** [Render] Add terrain shading with materials blended by slope and height, and per-cell projection for its letters
- [x] **S02-T03** [Render] Build the physically based sky, the sun and moon, the star field, layered clouds, and time of day with the Vale's colour grade
- [x] **S02-T04** [Render] Add froxel fog, god rays, and aerial perspective
- [x] **S02-T05** [Content] Bake sky visibility for every surface, and use it for ambient light with contact occlusion at half resolution
- [x] **S02-T06** [Render] Add GPU culling and instancing through one instance buffer, with the depth pyramid from the last frame
- [x] **S02-T07** [Render] Add grass as instanced blades that sway and bend around a moving point, with each blade a line of characters up close
- [ ] **S02-T07b** [Render] Grass seen almost edge on passes the shimmer check: the pasture east from x 160, z 120 at pitch 9, with and without the wood behind it (found in T08c)
- [x] **S02-T08a** [Content] Generate tree meshes in the bake: trunks and branches in bark, leaf cards, three levels of detail and a far form, for a few kinds
- [x] **S02-T08b** [Content] Place the woods, the orchard, and hedgerow trees in the valley as instances with permanent IDs
- [x] **S02-T08c** [Render] Draw trees through the instance buffer, the level chosen in culling and crossfaded; leaf cards cut out, their leaves written as `^ A v` at arm's length (after T08a)
- [ ] **S02-T08d** [Content] Bark that records each tree's age, as generated truth (after T08b)
- [x] **S02-T09** [Render] Add water for the river and pond: flow, reflection, refraction, foam, and ripples that resolve into characters at the edge
- [x] **S02-T10a** [Render] Add rain with its close-up characters and wet surfaces
- [ ] **S02-T10b** [Render] Add the wind that reads, from a pool of six sentences, legible in motion on the owner's screen
- [x] **S02-T11** [Render] Cache the far shadow cascades and ration shadowed lights
- [x] **S02-T12** [Render] Build the governor and the Low, Medium, and High presets
- [ ] **S02-T13** [Shell] Add wind, water, rain, and birdsong ambience by synthesis and approved samples
- [x] **S02-T14** [Render] Capture the valley at dawn, noon, dusk, night, rain, and fog, and pass the shimmer check across the barley
- [ ] **S02-T15** [Shell] Run the performance run on the owner's Mac along the tour's camera path, bring every pass inside its share, and save the evidence

## Exit gate

The camera path from the hill to the far bank holds every number in the Targets table on the owner's Mac, with the governor's log showing resolution at 100% for most of it. The glyph-off comparison and the shimmer check pass in every weather. Dusk from the hill is a picture worth keeping.
