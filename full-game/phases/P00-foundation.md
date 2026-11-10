# Phase 00: Foundation

**Goal:** A clean cross-platform skeleton that can open a window, draw multiple views, take deterministic screenshots, and build for every target.

**Suggested model:** Fable 5.2, medium effort. Everything later rests on these decisions: the multi-view renderer skeleton, the action layer, and the pack format.

**Hand to Haiku 5.5:** CI workflow files, crate scaffolding, the asset manifest's first entries, and the local check script. Rules are in `CLAUDE.md` under Models and helpers.

## Read for this phase

Read only these files in `docs/brief/`, and only the named part where one is given.

- `0-overview.md`
- `0a-pillars-and-releases.md`
- `1a-core-constraints.md`
- `1b-storefront-and-release.md`: storefront trait only
- `1.1a-licenses-and-ai-gate.md`: License rules and Asset manifest only
- `1.1c-libraries-and-import.md`: Libraries only
- `5f-coop.md`: first paragraph only
- `5g-input.md`
- `8-proof-tooling.md`: capture and developer tools only
- `9-how-to-work.md`

## Scope

Workspace, SDL3 platform layer, wgpu setup, fixed timestep, multi-view renderer skeleton, multi-device action layer, storefront trait, pack format, asset manifest, capture tooling and developer tools, three-platform CI

## Tasks

Do these in order, one at a time, following the task loop in `CLAUDE.md`.

- [ ] **P00-T01** Create `docs/ARCHITECTURE.md`, `docs/DECISIONS.md`, `docs/PLAYTEST_CHECKLIST.md`, and `docs/AI_DISCLOSURE.md`, set up git and Git LFS in the private repository, confirm that pushing works, and add `.gitignore` with `logs/` excluded
- [ ] **P00-T02** Create the Cargo workspace and crates, build profiles, committed `Cargo.lock`, and `deny.toml` for `cargo-deny`
- [ ] **P00-T03** Build the SDL3 `platform` crate: window, high-DPI, events, and clean shutdown
- [ ] **P00-T04** Set up `wgpu` with per-pass timing, a pipeline registry that forbids creation during gameplay, and a readable unsupported-GPU error
- [ ] **P00-T05** Build the fixed-timestep loop and immutable `sim` snapshot, with `sim` running headless
- [ ] **P00-T06** Build the multi-view renderer skeleton, with one view, then vertical and horizontal split
- [ ] **P00-T07** Build the action layer and multi-device input, with a simulated device for tests
- [ ] **P00-T08** Create the `Storefront` trait with null, Steam, and itch feature builds
- [ ] **P00-T09** Define the versioned binary pack format and reader, then pass the bake determinism test
- [ ] **P00-T09a** Build the string tables: stable IDs, one table per language, placeholders, the allowed letter set, the pseudo-language, and the localization check in the bake
- [ ] **P00-T10** Implement `third_party/manifest.ron` validation, and write `third_party/DOWNLOADS.md` for the Phase 01 assets
- [ ] **P00-T11** Build the capture tool: scripted, deterministic shots to PNG, with contact sheets, and the one-command minimum machine capture
- [ ] **P00-T12** Add developer tools: console, free camera, and overlays, excluded from shipping builds
- [ ] **P00-T13** Create tiered CI for Windows, macOS, and Linux, including `python3 tools/update_plan.py --check`, and a script that runs the fast checks locally. CI produces the release builds, and the local script uses the dev profile so it runs comfortably in 8 GB
- [ ] **P00-T14** Run the exit gate and save evidence

## Exit gate

Scripted PNG capture works with one and two views, the bake determinism test passes, and CI builds Steam and itch variants on all three operating systems

Save the evidence in `docs/evidence/P00/`.

## Next

Move to `phases/P01-look-prototype.md`.
