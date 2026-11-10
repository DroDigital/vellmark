# Build roadmap

Work through these phases in order. Each phase file lists what to read, its tasks, its exit gate, and, where there is one, exactly what the owner checks.

| Phase | File | Tasks | Stops for owner | Model and effort | Part of |
|---|---|---|---|---|---|
| 00: Foundation | `P00-foundation.md` | 14 | No | Fable 5.2, medium | Release 1 |
| 01: Look prototype | `P01-look-prototype.md` | 15 | Art Gate | Fable 5.2, medium | Release 1 |
| 02: Hook prototype | `P02-hook-prototype.md` | 13 | Hook Gate | Fable 5.2, medium | Release 1 |
| 03: Terrain, light, and streaming | `P03-terrain-and-sky.md` | 13 | No | Opus 5.5, high | Release 1 |
| 04: Living nature | `P04-living-nature.md` | 11 | No | Opus 5.5, high | Release 1 |
| 05: Player, co-op, and first town | `P05-player-and-town.md` | 14 | No | Opus 5.5, high | Release 1 |
| 06: Combat and first dungeon | `P06-combat-slice.md` | 18 | Feel Gate | Fable 5.2, medium | Release 1 |
| 07: RPG systems | `P07-rpg-systems.md` | 18 | No | Opus 5.5, high | Release 1 |
| 08: Society and sound | `P08-living-systems.md` | 14 | No | Opus 5.5, high | Release 1 |
| 09: The complete Vale | `P09-the-vale.md` | 20 | Slice Gate | Opus 5.5, high; Fable 5.2, medium for the writing | Release 1 |
| 10: Release 1 | `P10-release-one.md` | 13 | Release Gate | Opus 5.5, high | Release 1 |
| 11: The Foxing Wood | `P11-the-foxing-wood.md` | 13 | Update Gate | Sonnet 5.5, high; Opus 5.5, high for creatures and writing | Update: The Foxing Wood |
| 12: Saltmargin | `P12-saltmargin.md` | 15 | Update Gate | Sonnet 5.5, high; Opus 5.5, high for creatures and writing | Update: Saltmargin |
| 13: The Verso | `P13-the-verso.md` | 15 | Update Gate | Opus 5.5, high | Update: The Verso |
| 14: The Blotmere | `P14-the-blotmere.md` | 13 | Update Gate | Sonnet 5.5, high; Opus 5.5, high for creatures and writing | Update: The Blotmere |
| 15: The Cinder Folio | `P15-the-cinder-folio.md` | 14 | Update Gate | Sonnet 5.5, high; Opus 5.5, high for creatures and writing | Update: The Cinder Folio |
| 16: The Whitemarch | `P16-the-whitemarch.md` | 13 | Update Gate | Sonnet 5.5, high; Opus 5.5, high for creatures and writing | Update: The Whitemarch |
| 17: Act Two closes | `P17-act-two-closes.md` | 12 | Update Gate | Opus 5.5, high | Update: Act Two closes |
| 18: The Erasure | `P18-the-erasure.md` | 15 | Update Gate | Fable 5.2, medium | Update: The Erasure |
| 19: Endings | `P19-endings.md` | 14 | No | Fable 5.2, medium | 1.0 |
| 20: Version 1.0 | `P20-version-one.md` | 10 | Release Gate | Opus 5.5, high | 1.0 |

Total: 297 tasks. Phases 00 to 10 (163 tasks) make Release 1.

## Which model does what

Four models are used, each where it pays for itself. Set them at the start of a phase with `/model` and `/effort`: for example `/model opus` and then `/effort high`.

| Model and effort | Used for | Phases |
|---|---|---|
| **Fable 5.2, medium** | Work where a wrong decision is expensive for the rest of the project, or where judgement and taste decide the result: the foundation, the look, the hook, combat feel, the Erasure, and the endings | 00, 01, 02, 06, 18, 19, and the writing in 09 |
| **Opus 5.5, high** | Demanding work that the brief already specifies closely: renderer features, systems, the first region, releases, and the creatures and writing of every region | 03, 04, 05, 07 to 10, 13, 17, and 20, and parts of 11, 12, 14, 15, and 16 |
| **Sonnet 5.5, high** | Content built to a standard the Vale has already set: terrain passes, settlements, kit dungeons, characters, loot, and tests | Most of 11, 12, 14, 15, and 16 |
| **Haiku 5.5, as a helper** | Routine, fully specified jobs handed over by the main session and checked by it. Each phase file names its own | Every phase |

Haiku 5.5 is never the main model for a phase. Its effort is chosen for each job, by the table in `CLAUDE.md`, through the four helpers defined in `.claude/agents/`.

Fable is not the default model on any plan and may be billed separately. If it is unavailable or costs more than it is worth to you, run its phases on Opus 5.5 at xhigh effort.

## Why this order

1. **Prove the look, then prove the hook.** Phase 01 shows that a realistic world can be written in letters too small to see. Phase 02 shows that reading and rewriting are fun, and then, in a grey-box arena, that fighting with blade and word for loot is fun too. Nothing large is built until the owner has approved all of it.
2. **Build one place completely.** Phases 03 to 09 build only what Scrivenhold Vale needs, to final quality.
3. **Ship it.** Phase 10 releases the Vale as early access.
4. **Add one Quire at a time,** in the order the main story visits them. Each update is checked against the cost the slice report predicted.
5. **End the story last,** when every place it depends on exists.

## The gates

| Gate | After | The question it answers |
|---|---|---|
| Art Gate | Phase 01 | Does it look real from afar, and does the fine print astonish up close? |
| Hook Gate | Phase 02 | Is reading and rewriting the world fun in one room, and is the fight in the yard beside it worth a second go? |
| Feel Gate | Phase 06 | Does a weapon in one hand and a word in the other feel right, and is the loot worth stopping for? |
| Slice Gate | Phase 09 | Is the Vale worth paying for, and what will the rest really cost? |
| Release Gate | Phase 10 | Is Release 1 ready to go live? |
| Update Gate | Phases 11 to 18 | Is this update ready, and did it cost what was predicted? |
| Release Gate | Phase 20 | Is 1.0 ready to go live? |

A gate that does not pass is fixed inside its own phase. The next phase never starts on a failed gate.

**The title sequence:** the screen begins as what appears to be a single enormous character. The camera pushes in through level after level of letters and emerges in a vast dawn landscape. This should be the most beautiful 30 seconds in the game.

**The first five minutes:** the player must reach a high vista quickly, see the Erasure on the horizon, walk up to a stone, and read something that changes how they see everything around them. Then they must cast their first word on something enormous and watch the world obey.
