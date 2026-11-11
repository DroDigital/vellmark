# Build state

**Phase file:** `phases/S02-the-ground.md` (S01's Look Gate deferred by the owner)
**Status:** Waiting on owner: choose a depth look

## Integration

**Branch:** `claude/charming-bardeen-5aqf5c`, restarted from `main` after pull request 2 was merged.
**Current task:** the depth looks, at the owner's direction (see `docs/PLAN_CHANGES.md`, 2026-11-11). Three variants are built and switchable; captures are in `docs/looks/`.
**Also done:** broken things scatter as their letters (F7, or `break door`), at the owner's direction; see `docs/PLAN_CHANGES.md`.
**Exact next step:** the owner picks a look (atoms, lattice, or source) or asks for changes. Make the pick the default in `GlyphTuning`, re-accept the yard's reference images, and adjust the capture limits for the depth range. Then begin `phases/S02-the-ground.md` at S02-T01.

## Lane: Render

Nothing yet.

## Lane: Sim

Nothing yet.

## Lane: Content

Nothing yet.

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

- The yard is built from boxes with generated materials. From afar it reads as a clean, lit place, not yet as a real one. Real textures (S01-T02b) and richer shapes are what is missing, and both are content, not renderer.
- Full stops and commas in authored text are faint at reading distance: a small glyph cannot reach the weight its cell asks for.
- Loose letters do not strike one another, cast no shadows yet, and make no sound.
- The dive's middle distances are low in contrast. It works; it is not yet gorgeous.
- Two captures of the same shot on the macOS CI runner's GPU differ by a few last bits. The software renderer, which is the reference, is exact.
