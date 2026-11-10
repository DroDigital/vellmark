# Showcase roadmap

Eight phases. Each phase file lists what to read, its tasks by lane, its exit gate, and what the owner checks.

| Phase | File | Tasks | Stops for owner | Model and effort |
|---|---|---|---|---|
| S00: Foundation and tools | `S00-foundation-and-tools.md` | 16 | No | Fable 5.2, medium |
| S01: The look | `S01-the-look.md` | 13 | Look Gate | Fable 5.2, medium |
| S02: The ground | `S02-the-ground.md` | 15 | No | Opus 5.5, high |
| S03: Hands | `S03-hands.md` | 15 | Play Gate | Fable 5.2, medium |
| S04: Bodies | `S04-bodies.md` | 16 | No | Opus 5.5, high |
| S05: Things | `S05-things.md` | 13 | No | Opus 5.5, high; Sonnet 5.5, high for data and interface |
| S06: Little Penance | `S06-little-penance.md` | 17 | No | Opus 5.5, high; Fable 5.2, medium for new text |
| S07: Showcase | `S07-showcase.md` | 11 | Showcase Gate | Opus 5.5, high |

Total: 116 tasks. The full-game plan it leads into has 338.

## Why this order

1. **Tools first.** Every later task is checked by a capture, a reference image, a bot, or the performance run, so those exist before the renderer does.
2. **The look before anything else.** If a wall does not astonish, nothing built on it will.
3. **The ground before the player,** so the frame budget is proven in an empty valley and every later addition is measured against it.
4. **Hands before bodies.** The fun is proven against a grey-box enemy before characters and creatures are made.
5. **The village last,** when everything it needs exists and is fast.

## What can run side by side

| Phases | How many sessions | Why |
|---|---|---|
| S00, S01 | One, with a second for tools | Almost all one lane |
| S02 | Two or three | Render and Content divide cleanly |
| S03, S04 | Three or four | All four lanes have work |
| S05 and S06 | Four. Phase S05's Content and Shell tasks can begin during S04 | Mostly data and interface |
| S07 | One or two | Tuning waits on the owner's Mac |

## The gates

| Gate | After | The question it answers |
|---|---|---|
| Look Gate | S01 | Does it look real from afar, and does the fine print astonish up close? |
| Play Gate | S03 | Is reading, rewriting, and fighting with blade and word fun? |
| Showcase Gate | S07 | Is this ready to show to strangers? |

A failed gate is fixed inside its own phase.

## How long

Roughly 450 to 800 hours of Claude Code work. With three or four cloud sessions running side by side through the middle phases, that is about four to eight weeks, if the account's usage allows it and the two early gates pass first time. This is an estimate and not a promise. Phase S00 is small, and how long it takes will show how far the estimate is out.
