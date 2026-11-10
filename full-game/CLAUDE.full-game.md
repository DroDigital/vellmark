# Vellmark

A first-person open-world fantasy RPG with drop-in local co-op and no game engine (Rust, SDL3, wgpu). The world looks realistic, in the manner of Skyrim, and every surface is secretly written in ASCII letters too small to see until you are close enough to read them.

The game stands on three pillars: **Read** (the world is text worth reading), **Rewrite** (Emendation changes it), and **Wander** (a lantern-lit realm worth standing still in). Every system must serve one of them. The game is played with a weapon in one hand and a word of Emendation in the other: melee is kept shallow, magic is a main feature, and the depth of weapons is in loot with six rarity tiers. The game ships in stages: Release 1 is Scrivenhold Vale, then one update per Quire, then 1.0.

The spec is split across `docs/brief/`. The work is split across `phases/`. **Do not read the whole brief.** Each phase file lists exactly what to read.

## Start of every session

Usage can run out mid-task without warning, so files and git are the only memory.

1. Read `docs/STATE.md`.
2. If a remote exists and the working tree is clean, run `git pull --ff-only`. Then run `git status --short` and `git log --oneline -5`.
3. Open only the phase file named in `docs/STATE.md`.
4. Act on the status in `docs/STATE.md`:
   - **In progress:** inspect the uncommitted diff and the last `wip` commits, then finish that task. Do not restart it, and do not discard work unless you are certain it belongs to this task and is broken.
   - **Ready:** start the first unchecked task in the phase file.
   - **Waiting on owner:** check whether the approval or files have arrived. If so, continue. If not, do any task that does not depend on them. If nothing can proceed, state exactly what you need and stop.
5. Say in one line which task you are on, and which model and effort its phase file suggests.

## Task loop

1. Update `docs/STATE.md`: status **In progress**, task ID, and a 1 to 3 line plan. Run `python3 tools/update_plan.py`, commit as `wip(P05-T03): start`, and push.
2. Work in small steps. Commit `wip(P05-T03): ...` whenever a step compiles, and commit and push before a long-running command.
3. Verify with the tests the task needs, and save any gate evidence in `docs/evidence/Pxx/`.
4. Tick the task's box in the phase file, update `docs/STATE.md` with the exact next step, run `python3 tools/update_plan.py`, commit as `P05-T03: summary`, and push.
5. Move to the next task.

If a task will not fit in one short sitting, first split it into smaller checkboxes in the phase file (such as `P05-T03a`), and commit that split.

## Finishing a phase

Run the phase's exit gate and save the evidence. Tag the commit `phase-05-done`, set `docs/STATE.md` to the next phase with status **Ready**, run `python3 tools/update_plan.py`, commit, and push with tags. If the phase ends in an owner gate, set status to **Waiting on owner** and stop. Otherwise, tell the owner that this is a good moment to run `/clear`, give the `/model` and `/effort` commands the next phase's file asks for, then continue if asked.

## Keeping the plan current in the repository

`docs/PLAN.md` is the owner's live view of the whole plan. It is generated, so never edit it by hand, and do not read it during normal work.

**Chat is not the plan.** When the owner changes anything about scope, design, order, tools, or assets, do this before continuing with other work:

1. Update the affected `docs/brief/`, `docs/SCOPE.md`, and `phases/` files, including tasks, gates, and reading lists.
2. Add a dated entry at the top of `docs/PLAN_CHANGES.md`, and any technical reasoning to `docs/DECISIONS.md`.
3. Run `python3 tools/update_plan.py`, commit as `plan: summary`, and push.

Keep the repository private and never force-push. If a push or pull fails, do not discard anything. Record the problem under **Known issues** in `docs/STATE.md`, keep working locally, and tell the owner.

## Spending usage carefully

- Read only the brief files and sections the phase names. Do not reread files already in context.
- Search code with grep, and read line ranges of large files.
- Send long build, test, and bake output to `logs/`. Read only failures and a short tail.
- Review images as contact sheets at the smallest useful size. Open one at full size only to judge fine detail.
- Keep tasks scoped. No unrelated refactors or reformatting.
- After 3 failed attempts at the same problem, record it under **Blockers** in `docs/STATE.md` and move to the next task that does not depend on it.
- Hand routine work to Haiku 5.5 helpers, as set out under Models and helpers. Use a helper on the main model only for an independent review.

## Models and helpers

Each phase file names the model and effort the owner should run it on. The owner sets these. You cannot change your own model, so if a task is plainly beyond or beneath the suggested one, say so in one line and carry on. Where a phase file asks for a different model for named tasks, stop before the first of them, give the owner the two commands, and continue when they say so.

Whatever the main model is, hand routine work to a subagent running **Haiku 5.5**. Each phase file lists the jobs it expects to be handed over.

**Hand a job over only if all of these are true:**

- It is fully specified: you can name the files to read, the files to write, and the pattern to follow.
- It has a check a machine can run: a test, the validator, the bake, a schema, or a diff against a table in the canon.
- It makes no decision about architecture, feel, balance, or scope.
- It writes no dialogue for Full characters, no quest text, no books, and no wall text. Haiku may copy and format canon text, and may write Light characters' few lines from their canon entry.
- It is big enough to be worth it. Do a two-minute job yourself.

**Never hand over:** the renderer core, shaders, anything measured on the minimum machine, anything an owner gate judges, the word sheets' effects, voice generation, publishing, or changes to the plan.

**Effort for each job.** Pick the lowest that will pass the check.

| Haiku 5.5 effort | For |
|---|---|
| Low | Copying tables from the canon into data files, renaming, moving files, running a command and reporting only its failures |
| Medium | Data entry that needs cross-checking, text formatting, summarising long logs, sorting feedback or validator output into lists |
| High | Routine code that follows an existing example closely: another menu screen, another input mapping, another test of a known kind |
| Max | Self-contained code with a clear specification and tests that already exist and must pass. If a job seems to need more than this, it is not routine: do it yourself |

**How to hand over.**

1. Use the helper whose effort the table gives: `helper-low`, `helper-medium`, `helper-high`, or `helper-max`, defined in `.claude/agents/`. All four run Haiku 5.5. Give it the exact files, the pattern to copy, the check it must pass, and the words "change nothing else".
2. Run independent jobs side by side, and never two that write the same file.
3. Helpers do not commit, push, or edit `docs/STATE.md`, phase files, or the plan.
4. When a helper returns, read its diff and run the check yourself before committing. You answer for its work as if it were your own.
5. If a helper fails the same job twice, do it yourself and note in `docs/DECISIONS.md` that this kind of job should not be handed over.

## The owner's machine

The owner works on an Apple M1 Mac with 8 GB of memory. It is the game's minimum target and the only machine performance is ever judged on.

- Build and run locally with the dev profile. Leave full release builds with LTO, and full bakes, to CI, and download their results.
- The bake tool must work tile by tile and never hold the whole realm in memory. Keep its peak under about 4 GB.
- Run one heavy job at a time. Do not run a bake, a release build, and the game together.
- Ask the owner to run the minimum machine capture at each gate. Never mark the minimum target as met from CI numbers.

## Story and content

The canon is written. `docs/WORLD_BIBLE.md` lists what is in `docs/bible/` and `docs/script/`. Build from it, read only the files your task names, and expand beat sheets into full text without contradicting them. Follow `docs/bible/03-voice-and-dialects.md` for all game text.

## Voice credits

The owner has 40,000 ElevenLabs credits and 10 voice slots. Only `tools/generate_voice.py` may spend them, and only with the owner's approval for each batch. Do not add voiced lines or change the text of generated lines without approval. See `docs/voice/cast-and-budget.md`.

## Rules that always apply

- No game engine, and no dependency or asset outside `docs/brief/1.1a` to `1.1c`.
- Every surface is made of the 95 printable ASCII characters in fine print, and none of it may show beyond the reveal distance except where `docs/brief/2-art-direction.md` allows.
- Never fake completion: no stubs presented as features, and no gate passed without evidence on disk.
- Apply the pillar test in `docs/brief/0a-pillars-and-releases.md` before building or expanding any system. Do not add systems the brief does not ask for.
- `docs/SCOPE.md` says what is built in full, built light, or held in reserve. Never change a tier, and never build or reference a Reserve entry, without recording it there and in `docs/PLAN_CHANGES.md`.
- Support a list of players and views from the start. Never assume a single player or camera.
- Keep all three platforms building: Windows, macOS, and Linux.
- No player-facing text in code. Every string lives in the string tables, and a release is not pushed until the localization check passes for every language in `docs/brief/4c-content-rules.md`.
- Never commit credentials. Never publish a build or store page without the owner's explicit approval.
- Log decisions in `docs/DECISIONS.md`.
- You cannot judge feel, real controllers, Steam Deck, or performance on other machines. Add them to `docs/PLAYTEST_CHECKLIST.md` instead of marking them passed.

## Stop for the owner only for

Owner gates in phase files (a failed gate is fixed inside its own phase), AI asset approval, downloads, paid assets, credentials, publishing, or a blocker that cannot be undone. While waiting, continue work that does not depend on the answer.

## Compact instructions

When compacting, preserve the current phase, task ID, files being changed, failing test names, and the exact next step.
