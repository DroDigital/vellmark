# Build state

**Phase file:** `phases/S01-the-look.md`
**Status:** Waiting on owner

## Integration

**Branch:** `claude/charming-bardeen-5aqf5c`, pull request 2. One integration session did every lane.
**Phase S00:** every task done but S00-T16, whose last line is the owner's: open the macOS build and run the performance run once.
**Phase S01:** T01, T02a, T03 to T11 done in the cloud. Evidence is in `docs/evidence/S01/`.
**Not done:** S01-T02b (textures, waits on approval), S01-T12 (measure on the owner's Mac), S01-T13 (the gate itself).
**Exact next step:** When the owner has answered, either fix what the Look Gate found or, if it passed, tag `showcase-01-done` and open `phases/S02-the-ground.md`. If textures are approved first, do S01-T02b: fetch them into `third_party/`, add manifest entries, and switch `assets/materials/materials.ron` from generators to maps.

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

1. **Look Gate.** Download the macOS build from the newest green CI run and follow "Phase S01: the Look Gate" in `docs/PLAYTEST_CHECKLIST.md`. Five minutes.
2. **The performance run.** Right-click `perf-run.command`, Open, and send back `vellmark-perf-report.txt`. This closes S00-T16 and S01-T12.
3. **Texture approval.** Say yes to the list in `third_party/DOWNLOADS.md`, or change it. The yard is on generated materials until then, and it will not look its best without real ones.

## Blockers

None.

## Known issues

- The yard is built from boxes with generated materials. From afar it reads as a clean, lit place, not yet as a real one. Real textures (S01-T02b) and richer shapes are what is missing, and both are content, not renderer.
- Full stops and commas in authored text are faint at reading distance: a small glyph cannot reach the weight its cell asks for.
- The dive's middle distances are low in contrast. It works; it is not yet gorgeous.
- Two captures of the same shot on the macOS CI runner's GPU differ by a few last bits. The software renderer, which is the reference, is exact.
