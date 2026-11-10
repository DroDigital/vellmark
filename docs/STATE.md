# Build state

**Phase file:** `phases/S00-foundation-and-tools.md`
**Status:** Ready

## Integration

**Current task:** S00-T13 and S00-T16, on branch `claude/charming-bardeen-5aqf5c`
**Plan:** One integration session is doing every lane on the session's assigned branch, a commit per task, one pull request.
**Done:** S00-T01 to T12, T14, T15. Evidence is in `docs/evidence/S00/`.
**Exact next step:** Read the first CI run. S00-T13 is done when all three platforms build and test in CI and the macOS app is an artifact. Then finish S00-T16.

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

- **S00 exit gate:** download the macOS build from the CI run, open it, and run `perf-run.command` once. Steps are in `docs/PLAYTEST_CHECKLIST.md`.
- **Asset approval:** the material list in `third_party/DOWNLOADS.md`, before S01-T02 fetches textures.

## Blockers

None.

## Known issues

None.
