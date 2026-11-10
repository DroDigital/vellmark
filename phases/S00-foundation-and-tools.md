# Phase S00: Foundation and tools

**Goal:** A window, a frame loop, and every tool that later work is checked by, building in the cloud and running on the owner's Mac.

**Suggested model:** Fable 5.2, medium effort. Decisions made here last for the whole project.

**Hand to Haiku 5.5:** CI workflow files, crate scaffolding, the asset manifest's first entries, and the reference image list. Rules are in `CLAUDE.md` under Models and helpers.

## Read first

- `docs/SHOWCASE.md`
- `docs/TECH.md`
- `docs/CLOUD.md`
- `full-game/docs/brief/1a-core-constraints.md`
- `full-game/docs/brief/1.1a-licenses-and-ai-gate.md`: License rules and the manifest
- `full-game/docs/brief/1.1c-libraries-and-import.md`

## In this phase

The workspace, the platform layer, the GPU layer, the fixed-step loop, hot reload, the capture and performance tools, string tables, the pack format, and CI that hands the owner a macOS build. No game yet.

## Tasks

Do the tasks of one lane in order, following the task loop in `CLAUDE.md`. Lanes run side by side as `docs/CLOUD.md` sets out. A task marked **after** waits for the task it names.

- [ ] **S00-T01** [Shell] Run `scripts/cloud-setup.sh` in a cloud session, fix what fails, record what the machine offers for software rendering, and commit the working script
- [ ] **S00-T02** [Shell] Create `docs/ARCHITECTURE.md`, `docs/DECISIONS.md`, `docs/PLAYTEST_CHECKLIST.md`, and `docs/AI_DISCLOSURE.md`, the Cargo workspace and crates from the brief, build profiles, `Cargo.lock`, and `cargo-deny`
- [ ] **S00-T03** [Shell] Build the SDL3 `platform` crate: window, high-DPI, events, controller and mouse and keyboard through one action layer, and a simulated device for tests
- [ ] **S00-T04** [Render] Set up `wgpu` with per-pass GPU timers that are always on in development builds, a pipeline registry that forbids creation during play, and a readable message for an unsupported GPU
- [ ] **S00-T05** [Sim] Build the fixed-timestep loop and the immutable snapshot, with `sim` running headless and a list of players and views from the first day
- [ ] **S00-T06** [Render] Build the renderer skeleton that draws a list of views, with one view in use
- [ ] **S00-T07** [Content] Define the versioned binary pack format, the tile layout the full game will stream, and the reader, and pass the bake determinism test
- [ ] **S00-T08** [Content] Build the string tables with stable IDs and the check that no player-facing text is written in code
- [ ] **S00-T09** [Shell] Add hot reload of shaders, data sheets, and text in development builds
- [ ] **S00-T10** [Render] Build the capture tool: scripted camera paths on a fixed clock, single images, contact sheets, the glyph-on and glyph-off pair, and the shimmer check
- [ ] **S00-T11** [Render] Add software-rendered captures in the cloud at low resolution, the reference image set, and the comparison that fails on an unaccepted change
- [ ] **S00-T12** [Shell] Build the performance run and `perf-run.command` for the owner's Mac, writing every number in the Targets table of `docs/TECH.md` to one report file
- [ ] **S00-T13** [Shell] Create CI for Windows, macOS, and Linux: fast tests on every push, and a macOS app the owner can download from every push to `main`
- [ ] **S00-T14** [Content] Implement `third_party/manifest.ron` validation, and write `third_party/DOWNLOADS.md` for the assets Phases S01 and S02 need
- [ ] **S00-T15** [Shell] Add developer tools: console, free camera, and the profiler view, excluded from shipping builds
- [ ] **S00-T16** [Shell] Run the exit gate and save the evidence

## Exit gate

A cleared window opens on all three platforms from CI builds. The headless simulation runs a thousand steps identically twice. A shader edit shows in under a second. The capture tool produces identical frames on two runs. The owner has downloaded the macOS build, opened it, and run the performance run once.
