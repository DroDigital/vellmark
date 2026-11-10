# Vellmark: the Showcase

Vellmark is a first-person open-world fantasy RPG with no game engine (Rust, SDL3, wgpu). The world looks realistic, in the manner of Skyrim, and every surface is secretly written in ASCII letters too small to see until you are close enough to read them. It is played with a weapon in one hand and a word of magic in the other.

**This repository is building the Showcase first:** one village, one hill, and one dungeon, at final quality. `docs/SHOWCASE.md` says what it is. Content is stripped. Visual quality, smoothness, and combat are never traded away.

The Showcase is the first piece of the real game. Build everything to the full game's specification in `full-game/docs/brief/`, and build nothing that would be thrown away.

## Where things are

| Path | What it is |
|---|---|
| `docs/SHOWCASE.md` | What the Showcase contains and when it is done |
| `docs/TECH.md` | The graphics stack, the performance targets, and the tools |
| `docs/CLOUD.md` | How cloud sessions are set up and run side by side |
| `phases/` | The Showcase's eight phases. This is the live plan |
| `docs/STATE.md` | Where the build is, lane by lane |
| `full-game/docs/brief/` | The full specification. Read only the files and sections a phase names |
| `full-game/docs/bible/`, `full-game/docs/script/` | The canon |
| `full-game/phases/` | The full-game plan. Do not start it until the Showcase Gate passes |

`full-game/CLAUDE.full-game.md` is kept for after the gate. Until then this file is the only set of standing rules, and where the two differ this one wins.

## Start of every session

Sessions run in the cloud and the machine is reclaimed when idle, so files and git are the only memory.

1. Read `docs/STATE.md`, and `docs/CLOUD.md` if you have not this session.
2. Run `git fetch`, then `git status --short` and `git log --oneline -5`.
3. Know your lane. If the owner did not name one, you are the integration session.
4. Open only the phase file named in `docs/STATE.md`.
5. Act on your lane's entry: finish a task that is in progress, or start the first unchecked task for your lane whose **after** condition is met.
6. Say in one line which task you are on.

## Task loop

1. Create the task's branch. Write the task ID and a 1 to 3 line plan in your lane's section of `docs/STATE.md`, commit as `wip(S03-T05): start`, and push.
2. Work in small steps. Commit and push whenever a step compiles, and before any long command.
3. Verify with the checks the task needs, and save gate evidence in `docs/evidence/Sxx/`.
4. Open a pull request, rebase on `main`, and state in it what was checked and how.
5. The integration session merges it, runs the full suite on `main`, and ticks the box.

If a task will not fit in one sitting, split it into smaller checkboxes first and commit the split.

## Finishing a phase

The integration session runs the phase's exit gate and saves the evidence, tags the commit `showcase-03-done`, sets `docs/STATE.md` to the next phase, and tells the owner the `/model` and `/effort` commands the next phase asks for. If the phase ends in an owner gate, set the status to **Waiting on owner** and stop.

## Keeping the plan in the repository

Chat is not the plan. When the owner changes anything about scope, design, order, tools, or assets, update `docs/SHOWCASE.md`, `docs/TECH.md`, and the phase files first, add a dated entry at the top of `docs/PLAN_CHANGES.md`, and commit as `plan: summary` before other work. A change that also affects the full game is recorded in `full-game/docs/PLAN_CHANGES.md` as well.

## Evidence

- A cloud session proves correctness: tests, bots, reference images, the glyph-off comparison, and the shimmer check, with software rendering.
- Only the owner's Mac proves speed and feel. Never mark a performance target, a look, or a feel as met from a cloud session or a CI runner. Add it to `docs/PLAYTEST_CHECKLIST.md` and ask.
- Never fake completion: no stubs presented as features, no placeholder at a gate, and no gate passed without evidence on disk.

## Spending usage carefully

- Read only the files and sections the phase names. Do not reread what is already in context.
- Search with grep, and read line ranges of large files.
- Send long build, test, and bake output to `logs/`, and read only failures and a short tail.
- Review images as contact sheets at the smallest useful size.
- Keep tasks scoped. No unrelated refactors.
- After 3 failed attempts at the same problem, record it under Blockers in `docs/STATE.md` and move to the next task that does not depend on it.
- Hand routine work to Haiku 5.5 helpers, as set out below.

## Models and helpers

Each phase file names the model and effort the owner should run it on. You cannot change your own model, so if a task is plainly beyond or beneath the suggested one, say so in one line and carry on.

Whatever the main model is, hand routine work to a subagent running Haiku 5.5. Each phase file lists the jobs it expects to be handed over.

**Hand a job over only if all of these are true:** it is fully specified, it has a check a machine can run, it makes no decision about architecture, feel, balance, or scope, it writes no dialogue for named people and no books or wall text until the voice for that text is set, and it is big enough to be worth it.

**Never hand over:** the renderer core, shaders, anything measured on the owner's Mac, anything a gate judges, the word sheets' effects, voice generation, publishing, or changes to the plan.

| Helper | Effort | For |
|---|---|---|
| `helper-low` | Low | Copying tables from the canon into data files, renaming, running a command and reporting only its failures |
| `helper-medium` | Medium | Data entry that needs cross-checking, text formatting, summarising long logs |
| `helper-high` | High | Routine code that follows an existing example closely |
| `helper-max` | Max | Self-contained code with a clear specification and tests that already exist and must pass |

Give the helper the exact files, the pattern to copy, the check it must pass, and the words "change nothing else". Helpers do not commit or edit the plan. Read the diff and run the check yourself before committing. If a helper fails the same job twice, do it yourself.

## Voice credits

The owner has 40,000 ElevenLabs credits and 10 voice slots. Only `full-game/tools/generate_voice.py` may spend them, and only with the owner's approval for each batch. The Showcase uses voiced lines only if the owner chooses that in Phase S06. Do not change the text of any line in `full-game/docs/voice/voice_lines.tsv`.

## Rules that always apply

- No game engine, and no dependency or asset outside `full-game/docs/brief/1.1a` to `1.1c`.
- Every surface is made of the 95 printable ASCII characters in fine print, and none of it may show beyond the reveal distance except where `full-game/docs/brief/2-art-direction.md` allows.
- The two promises in `docs/TECH.md` are never broken to meet a number, and a number is never lowered without the owner.
- Build only what `docs/SHOWCASE.md` lists as in. If something it lists as left out seems necessary, ask.
- Support a list of players and views from the start, though one is drawn.
- Use the full game's formats: tiles, pack files, permanent IDs, string tables, and one sheet per word.
- Keep all three platforms building.
- No player-facing text in code.
- Never commit credentials. Never publish a build or a page without the owner's explicit approval. Never force-push.
- Log decisions in `docs/DECISIONS.md`.

## Stop for the owner only for

Owner gates, asset approval, downloads a session cannot reach, paid assets, credentials, voice batches, publishing, or a blocker that cannot be undone. While waiting, continue work that does not depend on the answer.

## Compact instructions

When compacting, preserve the lane, the phase, the task ID, the branch, the files being changed, failing test names, and the exact next step.
