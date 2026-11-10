# Phase S01: The look

**Goal:** One yard and one wall that pass as a real place from six metres and turn into readable text at arm's length. This is the picture the whole game depends on.

**Suggested model:** Fable 5.2, medium effort. Judgement and taste decide the result.

**Hand to Haiku 5.5:** Capture scene files, contact sheets, asset manifest entries, and license records for downloads. Rules are in `CLAUDE.md` under Models and helpers.

## Read first

- `docs/TECH.md`: The two promises, Targets, The order of building stages 1 to 4
- `full-game/docs/brief/2-art-direction.md`
- `full-game/docs/brief/3a-renderer-core.md`
- `full-game/docs/brief/1.1b-asset-catalog.md`: textures and fonts

## In this phase

Stages 1 to 4 of the order of building: the realistic frame, the fine print, letters made of letters, close reading, and a yard with several materials and a lantern. One camera, no player.

## Tasks

Do the tasks of one lane in order, following the task loop in `CLAUDE.md`. Lanes run side by side as `docs/CLOUD.md` sets out. A task marked **after** waits for the task it names.

- [ ] **S01-T01** [Render] Build the forward+ HDR frame: depth pre-pass, 4x MSAA, one sun with soft cascaded shadows, and physically based shading from albedo, normal, and roughness
- [ ] **S01-T02** [Content] Import the approved textures and the two typefaces, and bake the multi-channel SDF atlas for all 95 characters of each
- [ ] **S01-T03** [Render] Write `glyph_surface()`: anchoring from mesh coordinates, cascade levels 0 and 1, the influence curve, and the early exit beyond the reveal distance
- [ ] **S01-T04** [Render] Add stable glyph identity and tone preservation, with the per-glyph lookup from tone to threshold baked, so the glyph-off comparison passes
- [ ] **S01-T05** [Render] Add letter relief up close, the grazing-angle rules, and analytic antialiasing, and pass the shimmer check on a wall and a floor
- [ ] **S01-T06** [Render] Add letters made of letters through all six levels, close reading as a blend to the text frame, and the dive through a letter
- [ ] **S01-T07** [Content] Build the yard as data: masonry, timber, cloth, metal, and ground, each in its glyph family, with a door, a bench, and a hanging sign
- [ ] **S01-T08** [Render] Add clustered point lights with soft falloff, a lantern that casts shadows, bloom, the filmic tone map, and the edge filter
- [ ] **S01-T09** [Content] Add authored text that runs in reading order and wraps around the door and the corners, and the first text pages read by cell
- [ ] **S01-T10** [Render] Expose the reveal distance and letter contrast for tuning, and add the tuning panel
- [ ] **S01-T11** [Render] Capture the yard at noon, dusk, and night, from 20 m, 6 m, 2 m, and 0.5 m, as contact sheets, with the glyph-off pairs and the shimmer runs
- [ ] **S01-T12** [Render] Measure the yard on the owner's Mac with the performance run, and bring the opaque pass inside its share
- [ ] **S01-T13** [Shell] Run the exit gate, save the evidence, and write the Look Gate steps

## Exit gate

The glyph-off comparison passes at every distance beyond the reveal distance. Text is legible at 6 to 8 pixels and handsome at 300. The shimmer check passes. The yard holds 60 frames a second on the owner's Mac.

## Owner gate

**Stop. Give the owner the macOS build and five minutes of steps: stand far, walk in, hold Read, turn the lantern on and off.**

The owner checks:

- From across the yard it looks like a real place, and nothing gives the letters away
- Walking up to the wall, the letters arrive without a pop or a crawl
- At arm's length the wall can be read, and it is worth reading
- Close reading made you say something out loud
- Dusk and lantern light are beautiful
- It is smooth

**It passes when** the owner would put a screenshot of it on a store page. If the look is not right, nothing built on it will be, so fix it here.

Set `docs/STATE.md` to **Waiting on owner**.
