# Building in cloud sessions

The Showcase is built by Claude Code running in cloud sessions against this GitHub repository. Nothing is built on the owner's computer. The owner's Mac is used only to play builds and to run the performance check.

## What a cloud session can and cannot do

| It can | It cannot |
|---|---|
| Build the game and every tool | Judge how the game looks in motion, or how it feels |
| Run the headless simulation, the bots, and all tests | Measure performance. It has no real graphics card |
| Render low-resolution captures with software rendering | Play sound to anyone |
| Run for hours without the owner present | Reach websites outside the environment's allowed list |
| Run beside other sessions on other branches | Keep anything that was not pushed. The machine is reclaimed when idle |

Files and git are the only memory. Push after every step that compiles.

## Setting up the environment once

The owner does these once, in the cloud environment's settings.

1. Connect the GitHub repository.
2. Set the setup script to `scripts/cloud-setup.sh`.
3. Add these sites to the allowed list, so approved assets can be downloaded: `quaternius.com`, `kenney.nl`, `kaylousberg.itch.io`, `polyhaven.com`, `ambientcg.com`, `versilian-studios.com`, `sonniss.com`, and `freesound.org`. The package registries and GitHub are normally allowed already.

`scripts/cloud-setup.sh` is a starting point. The first task of Phase S00 runs it, fixes whatever fails on the real machine, and commits the working version. A setup script is cached only if it finishes within about five minutes, so keep it short and move slow steps into the build.

If an asset site cannot be reached from a session, do not work around it. List the files needed in `third_party/DOWNLOADS.md`, ask the owner to add them to the repository, and carry on with other work.

## Lanes

Work that touches different parts of the code runs side by side, one session to a lane. Each phase file says which of its tasks belong to which lane and which must wait for another.

| Lane | Owns these crates and folders | Typical work |
|---|---|---|
| Render | `gpu`, `render`, shaders | The renderer, effects, the governor |
| Sim | `sim`, `harness` | Rules, combat, words, AI, bots, tests |
| Content | `assets/data`, `content`, `bake` | Sheets, places, people, items, text, the bake |
| Shell | `app`, `platform`, `ui`, `audio`, CI | Window, input, interface, sound, builds |

## Rules for working side by side

- **One branch for each task,** named for it: `s03-t05-rewrite-wave`. Never work on `main`.
- **One session, one lane, one task at a time.** A session never edits another lane's crates. If a task needs a change there, it writes the request in `docs/STATE.md` under Requests, and the session in that lane picks it up.
- **Interfaces first.** When two lanes must meet, the types and function signatures between them are agreed and merged before either side is built.
- **Merge small and often.** Open a pull request when a task's checks pass. Rebase on `main` before merging. A task that will take more than a day is split.
- **`docs/STATE.md` has one section for each lane.** A session edits only its own section, which keeps merges clean.
- **The integration session.** One session, on the phase's main model, merges pull requests, runs the full test suite and the captures on `main`, and is the only one that ticks boxes in phase files or changes the plan.
- **Never force-push.** If a merge conflicts, the later session rebases and resolves it. If it cannot, it stops and records the conflict.

## How many sessions to run

Cloud sessions count against the same usage as everything else on the account, so three sessions finish a phase about three times sooner and use the allowance about three times faster.

- Phases S00 and S01 are mostly one lane. Run one session, with a second for tools.
- Phases S02 to S06 divide well. Run up to four.
- Phase S07 is tuning on the owner's Mac. Run one or two.

## What the owner does

| When | What |
|---|---|
| Once | The environment setup above |
| When asked | Approve asset downloads, and add any file a session could not fetch |
| At each gate | Download the macOS build from the CI run, play it, and run `./perf-run.command` by double-clicking it. Send back the report file it writes |
| Any time | Open a session and say what looks or feels wrong. That is the most valuable input in the project |
