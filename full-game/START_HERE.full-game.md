# Building Vellmark in phases

## Start

1. Create a **private** GitHub repository and clone it, or use an existing one.
2. Copy everything inside this folder into the repository's root, including the hidden `.claude` folder, which holds the Haiku helpers. In Finder, press Command, Shift, and full stop to show it. If the repository already has a `CLAUDE.md`, merge the two.
3. Open the repository in Claude Code.
4. Paste this:

```
Read CLAUDE.md and docs/STATE.md, then continue the Vellmark build from the next unfinished task.
```

## The shape of the plan

Vellmark is built around three things: reading the world, rewriting it, and wandering it. The plan proves the look in one scene, then proves the fun in one room and one arena, then builds one region completely and releases it as early access. Each later region arrives as its own update, and the endings come last. Every stage stops for you to play it.

## Following the plan

Open `docs/PLAN.md` in your repository at any time. It shows progress, the current task, what is waiting on you, the full scope, every task, and every plan change. Claude Code regenerates and pushes it as work moves forward.

To change the plan, just tell Claude Code. It updates the design and task files, records the change, regenerates the plan, and pushes before continuing.

## When usage runs out

Nothing is lost. Each task is saved to git in small steps and pushed to your repository, and `docs/STATE.md` always says what comes next.

Recent versions of Claude Code can also offer to wait for your limit to reset and continue automatically. Otherwise, after the reset, open the same folder and paste the same line. `claude -c` also reopens your latest conversation, but a fresh session costs less and works just as well.

## To use less of your limit

- Run `/clear` after each phase, and whenever you switch to something unrelated. The project's files hold everything needed to continue.
- Use the model and effort suggested for each phase, listed in `phases/README.md` and at the top of each phase file. Fable 5.2 at medium effort is kept for the phases where a wrong call is costly, Opus 5.5 at high effort does most of the build, Sonnet 5.5 at high effort builds most of each later region, and Claude Code hands routine jobs to Haiku 5.5 helpers by itself. Set them with `/model fable`, `/model opus`, or `/model sonnet`, then `/effort medium` or `/effort high`. Claude Code tells you the two commands at the end of each phase.
- Avoid long open-ended requests. "Continue the build" is enough.

## When it stops for you

| Stop | What you do |
|---|---|
| Art gate (after Phase 01) | Look at the images and say what to change, or "approved" |
| Hook gate (after Phase 02) | Play one room and one arena for twenty minutes, and say whether reading, rewriting, and fighting for loot were fun |
| Demo (after the Hook gate) | Say whether to offer the hook room as a free public demo. If yes, publish the private itch page yourself, and paste what players tell you into `docs/DEMO_FEEDBACK.md` |
| Feel gate (after Phase 06) | Play the build, run the checklist, and report what feels wrong |
| Slice gate (after Phase 09) | Play the finished Vale, read the cost report, and confirm what gets built after it |
| Release gate (after Phase 10) | Try the private early-access build and say whether to publish |
| Update gates (Phases 11 to 18) | Try each new Quire and say whether to release it |
| Downloads | Download what `third_party/DOWNLOADS.md` lists into `third_party/inbox/` |
| AI assets | Review the sheet, and approve or reject each batch |
| Voices | Create the 10 voices, add their IDs to `docs/voice/voice_ids.json`, and approve each generation batch |
| Publishing (Phases 10 to 20) | Approve the store materials and every release |

## What is in this folder

| Path | Purpose |
|---|---|
| `CLAUDE.md` | Rules and resume steps, loaded automatically in every session |
| `docs/PLAN.md` | The whole plan and live progress, generated automatically |
| `docs/PLAN_CHANGES.md` | The history of plan changes |
| `docs/WORDS.md` | Every word of magic and what it does, in plain language. It appears during Phase 02 and is generated from the game's own data |
| `docs/SCOPE.md` | What is built in which release, what is built in a lighter form, and what is held back |
| `tools/update_plan.py` | Regenerates the plan; it needs Python 3 |
| `docs/STATE.md` | The current position in the build |
| `phases/` | One checklist per build phase, with what you check at each gate |
| `docs/brief/` | The full design, split into small topic files |
| `docs/WORLD_BIBLE.md`, `docs/bible/` | The world, characters, creatures, items, and books |
| `docs/script/` | The story script, endings, and all quests |
| `docs/voice/` | The cast, character presets, recording script, and credit budget |
