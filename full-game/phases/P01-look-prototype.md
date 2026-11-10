# Phase 01: Look prototype

**Goal:** Prove Vellmark's core look on a small test scene before anything else is built on it.

**Suggested model:** Fable 5.2, medium effort. The fine-print technique is new and nothing else in the game matters if it fails.

**Hand to Haiku 5.5:** Capture scene files, contact sheets, asset manifest entries, and license records for downloads. Rules are in `CLAUDE.md` under Models and helpers.

**Requires:** Phase 00 finished and tagged `phase-00-done`.

## Read for this phase

Read only these files in `docs/brief/`, and only the named part where one is given.

- `1.1b-asset-catalog.md`: Fonts, plus one building, character, and animation source
- `1.1c-libraries-and-import.md`
- `2-art-direction.md`
- `3a-renderer-core.md`
- `8-proof-tooling.md`: The Approach, realism read, reveal, stability, and tone tests

## Scope

Font bake, import pipeline, the realistic frame, the fine-print Glyph Cascade, a colour grade, a simple sun, sky, and lantern light, on a test scene with one sculpted cliff, one stone wall with authored text, one imported building, and one imported animated character

## Tasks

Do these in order, one at a time, following the task loop in `CLAUDE.md`.

- [ ] **P01-T01** Bake the two typefaces into a multi-channel SDF array
- [ ] **P01-T02** Build the test scene: one sculpted cliff, with anchors and dominant-axis projection
- [ ] **P01-T03** Implement the fine cascade levels, square cells, and the shared `glyph_surface()` function, with letters vanishing completely below about 3 px
- [ ] **P01-T04** Implement letters made of letters
- [ ] **P01-T05** Implement stable glyph identity and the tone lookup, then pass the tone test
- [ ] **P01-T06** Implement the realistic frame: material maps, physically based shading, soft sun shadows, sky ambient, and the filmic tone map
- [ ] **P01-T07** Implement ink and paper per material with exact colour preservation, letters as relief, and the reveal distance controls
- [ ] **P01-T08** Add a simple sun, sky, a lantern with soft falloff, restrained bloom, and one colour grade for dawn, dusk, and night
- [ ] **P01-T08a** Add dynamic resolution with a sharpening upscale, a full-resolution HUD, the early exit for letters beyond the reveal distance, and the Low preset
- [ ] **P01-T09** Add the stone wall with authored text and text pages
- [ ] **P01-T10** Build the minimum import pipeline: glTF to anchored, material-classed meshes
- [ ] **P01-T11** Add one imported building, with near and far forms
- [ ] **P01-T12** Add one imported animated character, with rest-pose anchors and a retargeted animation
- [ ] **P01-T13** Build The Approach sequence, the stability test, the realism read test with the glyph layer on and off, and the reveal test
- [ ] **P01-T13a** Add the coarse cascade levels and a first close reading view of the test scene, blending from the realistic frame to text
- [ ] **P01-T14** Critique the captures against the art direction, and iterate until they are beautiful
- [ ] **P01-T15** Assemble the art gate sheet: The Approach, dawn, dusk, lantern-lit night, each far shot with and without letters, and one close reading shot. Ask the owner to run the minimum machine capture on the test scene

## Exit gate

The Approach, realism read, reveal, stability, and tone tests pass

Save the evidence in `docs/evidence/P01/`.

## Owner gate: Art Gate

**Stop. Show me The Approach images plus a dawn shot, a dusk shot, and a lantern-lit night shot. Wait for approval before continuing.**

What the owner checks:

- The shots from 20 m and beyond look like a real place, with no grid, grain, or shimmer, and match the same shots with the letters switched off
- Walking in, the letters appear so gradually that you cannot say where they began
- At 1 m and 0.3 m the letters are crisp, catch the light like carving, and the authored wall text can be read
- The close reading shot turns the whole scene into text and is striking in its own right
- The materials and light hold up without the letters. If the assets look cheap from afar, say so now
- On your M1, the test scene holds 30 fps at 1080p on the Low preset, and the capture is saved as evidence
- The dusk and lantern shots are the best images in the set
- Nothing shimmers or swims in the 10-second frame dump

**It passes when** the owner would post a far shot as a good-looking game, and a near shot as something nobody has seen before.

If it does not pass, fix what the owner reports inside this phase and ask again. Do not start the next phase.

Set `docs/STATE.md` to **Waiting on owner**, and tag `phase-01-done` only after approval.

## Next

Move to `phases/P02-hook-prototype.md`.
