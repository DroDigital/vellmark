# Build state

**Phase file:** `phases/S02-the-ground.md` (S01's Look Gate deferred by the owner)
**Status:** In progress. The owner asked to continue; the flat look stays the default until they choose one

## Integration

**Branch:** `claude/charming-bardeen-5aqf5c`, restarted from `main` after pull request 2 was merged.
**Current task:** the depth looks, at the owner's direction (see `docs/PLAN_CHANGES.md`, 2026-11-11). Three variants are built and switchable; captures are in `docs/looks/`.
**Also done:** broken things scatter as their letters (F7, or `break door`), at the owner's direction; see `docs/PLAN_CHANGES.md`.
**Done, at the owner's direction:** the living lattice, now the default look (see `docs/PLAN_CHANGES.md`, 2026-11-11), and every material's words in `assets/materials/lexicon.ron`. Captures: `captures/living.ron`, `living_motion.ron`, `names.ron`.
**Exact next step:** the owner picks a look (atoms, lattice, or source) or asks for changes. Make the pick the default in `GlyphTuning`, re-accept the yard's reference images, and adjust the capture limits for the depth range. Then begin `phases/S02-the-ground.md` at S02-T01.

## Lane: Render

**Done:** S02-T02, the terrain drawn (`--scene valley`); S02-T03, the sky: a scattering atmosphere, sun, moon with its phase and faint `@`, stars, two cloud layers, and the Vale's grade by the hour. Evidence: `docs/evidence/S02/sky.jpg`, `captures/s02_sky.ron`.
**Not yet:** stars as characters in close reading; cloud shadows on the ground; dawn on the placeholder textures reads golden-brown (judge on the Mac).
**Done:** S02-T04, the air: froxel mist by the hour, god rays from the shadow maps (faint until there are trees), aerial perspective. Evidence: `docs/evidence/S02/air.jpg`.
**Done:** S02-T05, the sky each place sees (terrain and meshes, baked) and contact occlusion at half resolution. Evidence: `docs/evidence/S02/sky_seen.jpg`.
**Done:** S02-T06, GPU culling and instancing: one mesh pool and instance buffer, culled per view against the frustum and last frame's depth pyramid, drawn by indirect batches. Test: `crates/harness/tests/culling.rs`.
**Done:** S02-T07, grass: patches in the one instance buffer, culled per view, four levels without popping; blades bent by the wind and by whoever walks through; each blade a line of its lexicon's words, and at arm's length nothing but them. Evidence: `docs/evidence/S02/grass.jpg`, `captures/s02_grass.ron`. Test: `crates/harness/tests/grass.rs`.
**Not yet:** grass casts no shadows (S02-T11's rota); the wind wanders on its own until the weather drives it (S02-T10); density and cost to be judged on the Mac.
**Done:** S02-T09, water: refraction through peaty water by depth, screen-space reflection with the sky behind it, sun glints, flow down the surface, foam at the edge and in fast water written as `.` and `,`, and the water's own letters on its ripple crests. Evidence: `docs/evidence/S02/water.jpg`, `captures/s02_water.ron`.
**Done:** S02-T10a, rain: weather kept by the rules (rain, wetness, wind) and sent in the snapshot; overcast skies; wet, darkened, glossy surfaces; puddles on roads, earth, and banks that ring in rain and are written as water; streaks around each view; splashes that are `'` close by; runoff down walls as `|`; rings on the river. Evidence: `docs/evidence/S02/rain.jpg`, `captures/s02_rain.ron`, `captures/s02_rain_wall.ron`. Console: `rain 0..1`.
**In progress:** S02-T10b, the wind that reads. Built: six canon sentences in `assets/text/en/wind.ron`; a gust at most once in each 150 s, in a strong enough wind, laid where the gaze meets the grass; the blades in its strokes bow and shine, and the ground's grass with them. Not yet: it is not legible in a still capture on the software renderer (`captures/s02_wind.ron`). It must be judged in motion on the Mac (console `wind 0`..`wind 5`), and tuned there: strength, size, and where it is laid.
**Done:** S02-T11: the far cascade drawn once and kept until the light, the eye's cell, or a still thing changes; three shadowed lights in a cube array, each cube kept until its surroundings change, at most one drawn a frame. Test: `crates/harness/tests/shadows.rs`.
**Done:** S02-T12: the governor, giving up six things in order a tenth at a time and logging each step into the performance report, and the Low, Medium (default), and High presets (`--preset`, console `preset`). Tests: the governor's own, and `crates/harness/tests/governor.rs`. Evidence: `docs/evidence/S02/governor.jpg`.
**Next:** S02-T14, the valley in every weather and the shimmer check across the barley (S02-T13 is the Shell lane's). S02-T10b waits for the owner's eye.
**Not yet:** rain does not know about roofs; it falls everywhere until there are buildings to stand under (S04, with a map of what is overhead). Rain falls only where there is terrain.

## Lane: Sim

Nothing yet.

## Lane: Content

**Asked by Render:** the brook's mouth into the River Quill (about x -12, z -106) is carved as a steep, pale gully that reads as a block from the north bank. Soften it.

**Done:** S02-T01, the terrain as tiles (192 tiles in three levels). Maps in `docs/evidence/S02/`.

## Lane: Shell

Nothing yet.

## Requests between lanes

None.

## Waiting on the owner

0. **Choose a depth look.** See `docs/looks/`, or press F6 in the build. Atoms, lattice, or source.
1. **Look Gate.** Download the macOS build from the newest green CI run and follow "Phase S01: the Look Gate" in `docs/PLAYTEST_CHECKLIST.md`. Five minutes.
2. **The performance run.** Right-click `perf-run.command`, Open, and send back `vellmark-perf-report.txt`. This closes S00-T16 and S01-T12.
3. **Texture approval.** Say yes to the list in `third_party/DOWNLOADS.md`, or change it. The yard is on generated materials until then, and it will not look its best without real ones.

## Blockers

None.

## Known issues

- The grass `letters` capture passes the glyph-off comparison with little room (max 0.008, limit 0.020). Where a near blade dissolves into letters against bright sky, the bloom carries the change a few pixels beyond the edge filter's three, and a small change in the blades' places once took it to 0.024. S02-T14 runs the comparison in every weather and must look at this; widening the band the comparison exempts would be the owner's decision.

- The yard is built from boxes with generated materials. From afar it reads as a clean, lit place, not yet as a real one. Real textures (S01-T02b) and richer shapes are what is missing, and both are content, not renderer.
- Full stops and commas in authored text are faint at reading distance: a small glyph cannot reach the weight its cell asks for.
- Loose letters in flight do not strike one another, a heap is never knocked down again, and they make no sound.
- The dive's middle distances are low in contrast. It works; it is not yet gorgeous.
- Two captures of the same shot on the macOS CI runner's GPU differ by a few last bits. The software renderer, which is the reference, is exact.
