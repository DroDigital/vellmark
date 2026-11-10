# Start here

This folder is a build kit for the **Vellmark Showcase**: one village, one hill, and one dungeon from the game, built to final quality by Claude Code in cloud sessions. When it is finished and you approve it, the same repository carries on into the full game.

## Read these two first

| File | What it tells you |
|---|---|
| `docs/SHOWCASE.md` | What the Showcase is, what is in it, and what "done" means |
| `phases/README.md` | The eight phases, the three gates, the models, and how long it may take |

## Set up once

1. Create a private GitHub repository.
2. Copy everything inside this folder into its root, including the hidden `.claude` folder. In Finder, press Command, Shift, and full stop to show it. Commit and push.
3. In Claude Code's cloud settings, connect the repository, set the setup script to `scripts/cloud-setup.sh`, and allow the asset sites listed in `docs/CLOUD.md`.
4. Start a cloud session on the repository with: **"Read CLAUDE.md and continue the build."**

## Each time after that

- **To continue:** open a session and say "Continue the build."
- **To go faster:** open more sessions and give each a lane: "Continue the build in the Render lane." The lanes are Render, Sim, Content, and Shell, and `phases/README.md` says how many each phase can use. One session with no lane named does the merging.
- **Set the model** each phase asks for with `/model fable`, `/model opus`, or `/model sonnet`, then `/effort medium` or `/effort high`. Claude Code tells you the two commands at the end of each phase.
- **More sessions use your allowance faster,** in proportion. They shorten the calendar and not the cost.

## When it stops for you

| When | What you do |
|---|---|
| Asset downloads | Approve the list. If a session cannot reach a site, add the files to the repository yourself |
| Look Gate, after Phase S01 | Download the macOS build, look at one yard for five minutes, and say whether it astonishes |
| Play Gate, after Phase S03 | Play for fifteen minutes and say whether reading, magic, and fighting are fun |
| Voices, in Phase S06 | Choose generated lines for the few scenes in the Showcase, or the murmur |
| Showcase Gate, after Phase S07 | Play the whole thing, run the performance check, and say whether it is ready for strangers |
| Any time | Say what looks or feels wrong. A sentence is enough |

At each gate, Claude Code gives you a build to download. Double-click `perf-run.command` on your Mac when asked, and send back the report file it writes. That is the only thing that ever runs on your computer.

## After the Showcase

Everything for the full game is already here, in `full-game/`: the specification, the story, and all 21 phases. When the Showcase Gate passes, Claude Code works out which of those tasks the Showcase has already finished and carries on from the next one.

## What is where

| Path | What it is |
|---|---|
| `CLAUDE.md` | The standing rules Claude Code follows |
| `docs/SHOWCASE.md` | The design of the Showcase |
| `docs/TECH.md` | The graphics stack and the performance targets |
| `docs/CLOUD.md` | How cloud sessions work on this project |
| `docs/STATE.md` | Where the build is right now |
| `docs/PLAN_CHANGES.md` | The history of plan changes |
| `phases/` | The Showcase's phases |
| `scripts/cloud-setup.sh` | What each new cloud machine installs |
| `.claude/agents/` | The four Haiku helpers |
| `full-game/` | The full game's specification, canon, and plan |
